#!/bin/sh
set -eu

cd "$(dirname "$0")/.."
severity=${1:-warning}
case "$severity" in
    error|warning|info|style) ;;
    *) echo "Usage: $0 [error|warning|info|style]" >&2; exit 1 ;;
esac

files=$(git grep -lE '^#!(/bin/|/usr/bin/env )(sh|bash|dash)' -- script) || exit 0

# shellcheck disable=SC2086
exec shellcheck -x -S "$severity" -C $files
