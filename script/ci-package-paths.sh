#!/bin/sh
set -eu

output=${FORGEJO_OUTPUT:-${GITHUB_OUTPUT:-}}
if [ -z "$output" ]; then
	echo "Error: neither FORGEJO_OUTPUT nor GITHUB_OUTPUT is set" >&2
	exit 1
fi

deb_path=$(find target -name '*.deb' -print -quit)
rpm_path=$(find target -name '*.rpm' -print -quit)
if [ -z "$deb_path" ] || [ -z "$rpm_path" ]; then
	echo "Error: no .deb/.rpm found under target/" >&2
	exit 1
fi

echo "deb: $deb_path"
echo "rpm: $rpm_path"
{
	echo "deb=$deb_path"
	echo "rpm=$rpm_path"
} >>"$output"
