#!/bin/sh
set -eu

cd "$(dirname "$0")/.."
severity=${1:-style}
case "$severity" in
error | warning | info | style) ;;
*)
	echo "Usage: $0 [error|warning|info|style]" >&2
	exit 1
	;;
esac

files=$(git ls-files -z --cached --others --exclude-standard -- script |
	xargs -0 grep -lEs '^#!(/bin/|/usr/bin/env )(sh|bash|dash)' || true)
if [ -z "$files" ]; then
	echo "Error: no shell scripts found" >&2
	exit 1
fi

# shellcheck disable=SC2086
exec shellcheck -x -S "$severity" -C $files
