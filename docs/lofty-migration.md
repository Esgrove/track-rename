# Moving Tag Handling To lofty

This document evaluates replacing the rust-id3 fork and metaflac with [lofty](https://github.com/Serial-ATA/lofty-rs),
and plans the switch.
Nothing has been changed yet.

## Summary

track-rename reads and writes tags with two crates:
the [Esgrove fork of rust-id3](https://github.com/Esgrove/rust-id3) for MP3 and AIFF,
and metaflac for FLAC.
lofty handles all of these formats in one crate, and adds MP4 (M4A), raw AAC, WAV, Ogg, and others.
The AudioBatch Rust port already uses lofty 0.25 to read tags and Serato `GEOB` frames,
and to write `ID3v2.4` tags with Serato frames into AIFF files.

Switching is recommended, after the read comparison in [Evaluation](#evaluation) passes on the real library.
It removes the fork, the metaflac dependency, and about 350 lines of workaround code,
and it is what makes M4A and WAV support possible.
The estimated effort is one to two days for the switch, and about one more day for M4A and WAV support.

## Current State

- `FileFormat` accepts only `mp3`, `aif`, `aiff`, and `flac`.
- `Track::try_from_path` skips `wav` and `m4a` files and prints
  "M4A file should be converted to a supported format".
  The reason is library support.
  rust-id3 only handles ID3 tags, metaflac only FLAC, and neither reads MP4 metadata atoms.
- `src/tags.rs` wraps both libraries in the `FileTags` enum.
- The fork exists for its structured frame errors.
  `FrameErrorKind::DelimiterNotFound` lets `is_malformed_frame_error` detect broken `UFID` and `PRIV` frames.
  The fork is rebased by hand on upstream, which moved from GitHub to Codeberg and archived the GitHub repository.
  Upstream is maintained, with 1.17.2 released in September 2026 including a decompression bomb fix.

### Workarounds For rust-id3

rust-id3 decodes frames one after another and stops at the first frame it cannot parse.
It returns the frames read so far as `partial_tag`, and every frame after the broken one is lost.
track-rename carries three workarounds for this:

1. **Two-phase write** in `write_id3_tags_to_path`.
   `GEOB` and `APIC` frames are removed, the text frames are written, then the file is read again
   and the binary frames are added back.
   Without this, new text frames written after a large Serato `GEOB` frame can be lost on the next read.
2. **Raw frame patching** in `fix_malformed_frames_raw`.
   When a `UFID` or `PRIV` frame lacks the null terminator after its owner, as Beatport files often do,
   the tool walks the raw `ID3v2` bytes itself and overwrites one byte in the file.
3. **MP3 to AIFF conversion** with ffmpeg in `read_tags_or_convert`,
   for MP3 files whose tags still cannot be read.

## Comparison

| | rust-id3 fork and metaflac | lofty 0.25 |
| --- | --- | --- |
| Formats | ID3 in MP3, AIFF, and WAV, FLAC through a second crate | MP3, AIFF, WAV, FLAC, MP4, AAC, Ogg, Opus, APE, WavPack, Musepack |
| Maintenance | Fork rebased by hand, metaflac last pushed in January 2025 | Active, five releases between August and October 2026 |
| API stability | Upstream is stable 1.x, the fork is a breaking 2.x | Before 1.0, minor releases break the API |
| Error detail | Structured frame errors and a partial tag | Logged warnings, no partial tag |
| Serato `GEOB` | Parsed when the tag is read | Kept as raw bytes and parsed on request |
| Numeric genre such as `(31)` | `genre_parsed()` | `Accessor::genre()` |
| Printing frames | `frame.content()` implements `Display` | No `Display`, needs a formatter per frame type |

### Robustness

lofty is not more robust in every case.
It is more robust against the failures track-rename has actually seen.
This is based on reading the lofty 0.25.5 source, and needs confirming on real files.

Where lofty is more tolerant, in its default `ParsingMode::BestAttempt`:

- `GEOB` and unknown frames are stored as raw bytes.
  A broken Serato payload cannot stop the rest of the tag from being read,
  so the two-phase write should no longer be needed.
- A `UFID` or `PRIV` owner without a terminator is read to the end of the frame instead of failing.
  A `UFID` with an empty owner is accepted.
  The raw patching is no longer needed.
- Frames with a broken header or zero size are skipped, and reading continues with the next frame.
- Each frame is parsed from a reader limited to the frame's declared size,
  so a bad frame cannot shift the frames after it.

Where lofty is stricter:

- A content error in a frame type it parses, such as `APIC`, `TXXX`, `COMM`, or a bad text encoding,
  fails the whole read.
  There is no partial tag to fall back to.
- Skipped frames are reported only through `log::warn!`,
  so track-rename's messages about repaired frames would disappear without a logger.
- `ParsingMode::Relaxed` accepts more, and should be compared too.

## What Changes

### Dependencies

- Replace the `id3` git dependency and `metaflac` with `lofty`.
- Add `log` and a small logger that turns lofty's warnings into track-rename's yellow messages,
  so damaged files are still reported.

### `src/tags.rs`

- `FileTags` keeps two variants backed by lofty types.
  `Id3v2Tag` is read through `MpegFile` and `AiffFile`, and `VorbisComments` through `FlacFile`.
  M4A support adds an `Ilst` variant.
- Read with `ParseOptions::new().read_properties(false)`, because track-rename never needs the audio properties.
  Keep cover art reading enabled, or a later write would drop the pictures.
- Write through the file type's `save_to_path`, for example `MpegFile::save_to_path`,
  so pictures and any other tags in the file are written back too.
  Keep the staged write through a temporary copy.
- Delete `fix_malformed_frames_raw`, `find_id3_header_offset`, `decode_synchsafe`, `is_malformed_frame_error`,
  `repair_malformed_frame`, the two-phase write, and their tests.
- `version_label` reports `Id3v2Tag::original_version`, because lofty upgrades every tag to `ID3v2.4` in memory.
- `print_id3_tag_data` needs a formatter for lofty's `Frame` variants,
  so `trackprint` output changes slightly.

### `src/serato/`

- `SeratoData::parse(&id3::Tag)` loops over the `Frame::Binary` frames with the `GEOB` ID
  and decodes them with `GeneralEncapsulatedObject::parse`, as `serato_data` does in AudioBatch.
- AudioBatch's `serato-tags` crate was extracted from this module, and takes plain description and payload pairs.
  Using it here would share the Serato parser between both projects.
  This works with either tag library, and is a separate decision.

### Tests And Benchmarks

- `tests/test_tag_roundtrip.rs` documents a rust-id3 bug.
  Rewrite it as lofty regression tests that keep the large `GEOB` and missing title cases.
- Add fixtures for a `UFID` and a `PRIV` frame without a terminator.
  They currently exist only as bytes built inside the unit tests.
- `tests/test_tag_read_write.rs` keeps its cases unchanged, which shows that behaviour is preserved.
- The test helpers in `src/serato/mod.rs`, `src/serato/markers.rs`, and `benches/serato.rs` read through lofty.
- Compare `benches/io.rs` before and after, with a saved Criterion baseline.

### Behaviour

- A `UFID` or `PRIV` frame without a terminator keeps all of its bytes and gains a terminator when written.
  Today the first content byte is overwritten instead.
- lofty merges duplicate frames when reading, keeping the one with content.
- lofty converts `ID3v2.2` and `ID3v2.3` frames to `ID3v2.4` when reading,
  for example `TYER` and `TDAT` become `TDRC`.
  track-rename already writes `ID3v2.4`, so files that are written end up the same.
- Fewer MP3 files should fail to read, so the conversion to AIFF should be needed less often.
  Keep it for files that lofty also rejects.

## New Formats

lofty makes the formats that track-rename now skips possible.
These come after the switch, as their own changes.

- **M4A** (AAC and ALAC in MP4):
  - Add `FileFormat::M4a` for `m4a`, and remove `m4a` from `OTHER_FILE_EXTENSIONS`.
  - Read and write artist, title, album, and genre through the `Ilst` tag with the `Accessor` trait.
    Genre can be stored as a numeric `gnre` atom or as `©gen` text, and `Accessor::genre` reads both.
  - Serato stores its data in M4A files as freeform `----` atoms, not as `GEOB` frames.
    Reading cue points and the overview from them needs a mapping like the native FLAC fields in `serato-tags`.
    Check the atom names on real files before writing that mapping.
- **WAV**:
  - Add `FileFormat::Wav`.
  - Serato and most DJ software store WAV tags in an `ID3 ` chunk, which `WavFile::id3v2` reads.
    lofty also reads RIFF `INFO`.
    Write `ID3v2` and leave `INFO` alone, so other software still sees its own fields.
- **Raw AAC** (`.aac` ADTS files) can carry `ID3v2` tags, and lofty reads them through `AacFile`.
  These are rare in DJ libraries, so add them only if real files need them.

## Evaluation

Run before switching, and again before each lofty minor version upgrade.

### Read Comparison

Add a cargo example such as `examples/tag_library_eval.rs`, with `lofty` as a dev-dependency.
It takes files or folders and reads every file with both stacks, without modifying any of them:

- rust-id3 with `Tag::read_from_path`, and metaflac for FLAC.
  Do not go through `FileTags::read`, because it repairs files in place.
- lofty through the concrete file types, with `read_properties(false)`.

For each file it compares artist, title, album, and genre,
the `GEOB` descriptions and payloads byte for byte, and the picture count.
It reports each stack's status (tagged, no tag, partial, error, or unsupported), lofty's warnings, and read time.
Run it on the whole DJ library, and treat every file where the two disagree as a finding to explain.

### Write Comparison

With a `--write` flag, each file is copied into a temporary directory and written three ways:

1. through track-rename's current `write_tags`,
2. as a single rust-id3 write without the two-phase workaround, to show whether that workaround is still needed,
3. with lofty.

Each written copy is read back with both stacks.
The check passes when the four fields match the written values,
the `GEOB` payloads and picture count match what that stack read before,
and the audio payload is unchanged byte for byte:
the data after the `ID3v2` tag in MP3, the `SSND` chunk in AIFF, the `data` chunk in WAV,
the frames after the metadata blocks in FLAC, and the `mdat` atom in M4A.

### Edge Cases

A `--generate` flag builds files that reproduce the known problems from the committed fixtures:

- `UFID` and `PRIV` frames without a terminator, with an album frame after them.
- A `COMM` frame with an invalid encoding byte, a case where rust-id3 returns a partial tag and lofty may fail.
- An MP3 and an AIFF with a 140 KB `APIC` frame and a 24 KB `GEOB` frame, the sizes that triggered the two-phase write.
- Numeric genre `(31)` in `ID3v2.3`, and an `ID3v2.2` tag.
- M4A files with AAC, with ALAC, and with cover art,
  plus an ADTS AAC file and a WAV with an `ID3 ` chunk, made with ffmpeg.

### Decision Criteria

Switch when:

- lofty reads every file that rust-id3 reads fully, with the same fields, `GEOB` payloads, and pictures.
- Files where rust-id3 returns a partial tag or fails are read at least as completely by lofty,
  or the difference is understood and accepted.
- Every lofty write passes the round trip check, including the large binary frame files.
- Read time on the library is not noticeably worse.

## Plan

1. Add the evaluation example and run it on the fixtures, the generated edge cases, and the DJ library.
   Record the results in this document.
2. Switch MP3, AIFF, and FLAC to lofty in `src/tags.rs` and `src/serato/`,
   with the logger for lofty warnings, and remove the workarounds, the fork, and metaflac.
3. Rewrite the tag round trip tests, add the malformed frame fixtures, and compare the I/O benchmarks.
4. Update `README.md`, `CLAUDE.md` (the project overview and `file_format.rs` notes),
   and the `convert_failed` description in `track-rename.toml`.
5. Add M4A support, then WAV support, each with fixtures and tests.
6. Optionally move `src/serato/` onto the shared `serato-tags` crate.

## Alternatives

- **Stay on rust-id3 and shrink the fork.**
  Offer the structured error changes upstream on Codeberg.
  A frame decoder that skips a bad frame instead of stopping would fix the cause for every user.
  This is the least churn, but depends on upstream accepting it, keeps metaflac, and adds no new formats.
- **TagLib through bindings.**
  It is the most battle tested, and the C++ AudioBatch uses it,
  but it adds a C++ build dependency to a pure Rust tool.
- **Not suitable:**
  `audiotags` is an unmaintained wrapper around rust-id3 and metaflac.
  Symphonia's metadata support is read only.
