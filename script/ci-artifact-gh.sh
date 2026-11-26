#!/bin/sh
set -eu

arch=${ARCH:?ARCH environment variable is not set}
if [ "${GITHUB_REF_TYPE:-}" = tag ]; then
	version="-$GITHUB_REF_NAME"
else
	version="-$GITHUB_SHA"
fi

flatpak_path=$(find target/release -maxdepth 1 -name '*.flatpak' -print -quit)
if [ -z "$flatpak_path" ]; then
	echo "Error: no .flatpak found under target/release" >&2
	exit 1
fi
flatpak_id=$(basename "$flatpak_path" .flatpak)

mkdir -p artifacts
cp "target/release/gram-linux-$arch.tar.gz" "artifacts/gram-linux-$arch$version.tar.gz"
cp "target/gram-remote-server-linux-$arch.gz" "artifacts/gram-remote-server-linux-$arch$version.gz"
cp "$flatpak_path" "artifacts/$flatpak_id-$arch$version.flatpak"
