#!/bin/bash
set -eo pipefail

# Install the Rust renamer tool to path.

# Import common functions
DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=./common.sh
source "$DIR/common.sh"

if [ -z "$(command -v cargo)" ]; then
    print_error_and_exit "Cargo not found in path. Maybe install rustup?"
fi

print_magenta "Installing binaries..."
cargo install --force --path "$REPO_ROOT"
echo ""

for executable in $(get_rust_executable_names); do
    if [ -z "$(command -v "$executable")" ]; then
        print_error_and_exit "Binary not found. Is the Cargo install directory in path?"
    fi
    echo "$($executable --version) from $(which "$executable")"
done
