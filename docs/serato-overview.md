# Serato Overview Findings

## Payload Interpretation

On 2026-10-13, read-only comparisons of matching source tags and Serato overview screenshots
supported an indexed-color raster interpretation for the tested `01 05` payloads.
Each 16-byte block is a time column containing 16 vertical palette indices,
not sixteen independent frequency strengths or amplitude measurements.
The observed payloads had 240 columns, or 3,842 bytes including the two-byte header.
That observation does not establish a universal column-count requirement.

The foreground colors support a zero-based 6x6x6 RGB cube.
Decode an index using integer arithmetic:

```text
red   = (index / 36) * 51
green = ((index % 36) / 6) * 51
blue  = (index % 6) * 51
```

Examples include index 121 as RGB (153, 102, 51) and index 115 as RGB (153, 51, 51).
The zero-based RGB mapping fit the three compared AIFF overview references better
than the tested palette offsets, channel permutations and component reversals.
Timed color regions supported interpreting successive blocks as whole-track time columns.
This evidence supports the decoding hypothesis, not exact compatibility with every Serato version or color mode.

Index 0 is black. The ubiquitous index 1 was treated as background when deriving a combined waveform color.
Foreground aggregation in AudioBatch uses indices 2 through 215.
Indices 216 through 255 remain unverified and are skipped rather than assigned guessed colors.
An observed reserved entry coincided with neutral screenshot pixels,
but its exact gray level and the rest of the reserved palette are unknown.

## Current Rust Behavior

`src/serato/overview.rs` retains the raw column bytes.
Its parser skips two header bytes without checking their value,
accepts header-only input and ignores an incomplete trailing column.
These permissive behaviors are covered by existing tests and were not changed by this documentation correction.

The terminal renderer is a legacy heuristic.
It averages numeric indices into eight rows, downsamples columns, normalizes those values
and selects blue, cyan, green, yellow or red by thresholds.
Those values are not audio levels, and the output is not a palette-faithful Serato visualization.
The existing identifier names mentioning bands or frequencies do not establish the payload's semantics.
No terminal rendering behavior is changed here.

Any future palette-faithful rendering must decode indices before averaging or interpolation.
Adjacent numeric indices can cross RGB cube planes, so blending the numbers directly gives unrelated colors.
Background and unknown entries need an explicit policy instead of numeric level normalization.

## Scope And Limits

The small embedded overview cannot reproduce individual-hit color changes in Serato's detailed zoomed waveform.
The source and storage of that finer analysis have not been established.
No frequency boundaries, detailed-analysis format or Serato internal algorithm are inferred here.

The comparisons used local development audio and user-supplied screenshots.
No music, extracted payload dumps or screenshot assets are included in this documentation or required by portable tests.
The screenshots' Serato version, selected color mode and display scale were not supplied.
Reserved colors, additional payload versions and detailed-waveform fidelity remain open questions.
