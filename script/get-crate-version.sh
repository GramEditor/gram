#!/bin/sh
set -eu

if [ $# -ne 1 ]; then
	echo "Usage: $0 <crate_name>" >&2
	exit 1
fi

metadata=$(cargo metadata --no-deps --format-version=1)
version=$(printf '%s' "$metadata" |
	jq --raw-output --arg name "$1" '.packages[] | select(.name == $name) | .version')
if [ -z "$version" ]; then
	echo "Error: crate '$1' not found in the workspace" >&2
	exit 1
fi
printf '%s\n' "$version"
