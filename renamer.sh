#!/bin/bash
set -eo pipefail

DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

# Check platform:
case "$(uname -s)" in
    "Darwin")
        export PLATFORM="mac"
        ;;
    "MINGW"*)
        export PLATFORM="windows"
        ;;
    *)
        export PLATFORM="linux"
        ;;
esac

if [ "$PLATFORM" = windows ]; then
    music_path="/d/Dropbox/DJ MUSIC"
    python="python"
else
    music_path="$HOME/Dropbox/DJ MUSIC"
    python="python3"
fi

$python "$DIR/rename/renamer.py" "$music_path" "$@"
