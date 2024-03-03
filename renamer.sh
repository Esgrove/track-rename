#!/bin/bash
set -eo pipefail

# Import common functions
DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=./common.sh
source "$DIR/common.sh"

if [ "$BASH_PLATFORM" = windows ]; then
    music_path="/d/Dropbox/DJ MUSIC"
    python="python"
else
    music_path="$HOME/Dropbox/DJ MUSIC"
    python="python3"
fi

$python "$DIR/rename/renamer.py" "$music_path" "$@"
