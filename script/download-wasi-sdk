#!/bin/sh
set -eu

WASI_SDK_VERSION=${WASI_SDK_VERSION:-25}
WASI_SDK_DIR=./target/wasi-sdk

installed_version=
if [ -f "$WASI_SDK_DIR/VERSION" ]; then
	installed_version=$(head -n 1 "$WASI_SDK_DIR/VERSION" | tr -d '\r')
fi
if [ "$installed_version" = "$WASI_SDK_VERSION.0" ]; then
	echo "WASI SDK v$WASI_SDK_VERSION already exists at $WASI_SDK_DIR"
	exit 0
fi

arch=$(uname -m)
case "$arch" in
x86_64) arch=x86_64 ;;
arm64 | aarch64) arch=arm64 ;;
*)
	echo "Unsupported architecture: $arch" >&2
	exit 1
	;;
esac

os=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$os" in
darwin) os=macos ;;
linux) os=linux ;;
mingw* | msys* | cygwin*) os=mingw ;;
*)
	echo "Unsupported OS: $os" >&2
	exit 1
	;;
esac

name="wasi-sdk-$WASI_SDK_VERSION.0-$arch-$os"
url="https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-$WASI_SDK_VERSION/$name.tar.gz"
echo "Downloading from: $url"

mkdir -p ./target
archive=$(mktemp)
trap 'rm -f "$archive"' EXIT
curl -fLsS --output "$archive" "$url"
rm -rf "./target/$name"
tar -xzf "$archive" -C ./target
rm -rf "$WASI_SDK_DIR"
mv "./target/$name" "$WASI_SDK_DIR"
echo "WASI SDK v$WASI_SDK_VERSION installed successfully"
