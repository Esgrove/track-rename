#!/bin/bash
set -eo pipefail

USAGE="Usage: $0 [OPTIONS]

Build the Rust renamer tool.

OPTIONS: All options are optional
    --help
        Display these instructions.

    --verbose
        Display commands being executed."

while [ $# -gt 0 ]; do
    case "$1" in
        --help)
            echo "$USAGE"
            exit 1
            ;;
        --verbose)
            set -x
            ;;
    esac
    shift
done

REPO_ROOT=$(git rev-parse --show-toplevel || (cd "$(dirname "${BASH_SOURCE[0]}")" && pwd))

if [ -z "$(command -v cargo)" ]; then
    echo "Cargo not found in path. Maybe install rustup?"
    exit 1
fi

pushd "$REPO_ROOT" > /dev/null
cargo build --release

if [ "$PLATFORM" = windows ]; then
    executable="track-renamer.exe"
else
    executable="track-renamer"
fi

rm -f "$executable"
mv ./target/release/"$executable" "$executable"
./"$executable" --version
./"$executable" -h
popd > /dev/null
