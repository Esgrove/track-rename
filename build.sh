#!/bin/bash
set -eo pipefail

# Import common functions
DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=./common.sh
source "$DIR/common.sh"

USAGE="Usage: $0 [OPTIONS]

Build the Rust renamer tool.

OPTIONS: All options are optional
    -h | --help       Display these instructions.
    -v | --verbose    Display commands being executed.
"

while [ $# -gt 0 ]; do
    case "$1" in
        -h | --help)
            print_usage_and_exit
            ;;
        -v | --verbose)
            set -x
            ;;
    esac
    shift
done

if [ -z "$(command -v cargo)" ]; then
    print_error_and_exit "Cargo not found in path. Maybe install rustup?"
fi

cd "$REPO_ROOT"

print_magenta "Building binaries..."
cargo build --release --bins

executables=$(get_rust_executable_names)
for executable in $executables; do
    print_green "${executable%.exe}"
    rm -f "$executable"
    mv ./target/release/"$executable" "$executable"
    ./"$executable" --version
    ./"$executable" -h || :
    echo ""
done
