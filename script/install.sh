#!/usr/bin/env bash
# shellcheck shell=bash
set -eu

err() {
  echo "$1" >&2
  exit 1
}

has_command() {
  if ! command -v $1; then
    err "Required command not found: $1"
  fi
}

usage() {
  echo "
Usage: ${0##*/} [options] [BUNDLE]
Install Gram on Linux from a tar bundle.

Options:
  -h, --help          Display this help and exit.
  --build             Build the tar bundle before installation.
  --build-remote      Build the `remote_server` binary
  --prefix PREFIX     Install into PREFIX (default ~/.local).
  "
}

GRAM_BUILD_TARBALL=false
GRAM_BUILD_REMOTE=false
GRAM_INSTALL_PREFIX="$HOME/.local"
GRAM_BUNDLE_FILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        --build)
            GRAM_BUILD_TARBALL=true
            shift
            ;;
        --build-remote)
            GRAM_BUILD_REMOTE=true
            shift
            ;;
        --prefix)
            shift
            [[ $# -lt 1 ]] && err "Expected PREFIX"
            GRAM_INSTALL_PREFIX="$1"
            shift
            ;;
        --)
            shift
            break
            ;;
        -*)
            echo "Unknown option: $1" >&2
            help_info
            exit 1
            ;;
        *)
            if [[ $# -gt 1 ]]; then
              err "Too many arguments, expected [BUNDLE]"
            fi
            if [[ $# -eq 1 ]]; then
              GRAM_BUNDLE_FILE="$1"
              shift
            fi
            ;;
    esac
done

version=$(
  curl -s -X GET -H 'accept: application/json' https://codeberg.org/api/v1/repos/GramEditor/gram/releases/latest \
    | grep -o '"tag_name":"[^"]\+"' \
    | sed -e 's,"tag_name":"\([^"]\+\)",\1,g'
)

host_line="$(rustc --version --verbose | grep "host")"
target_triple=${host_line#*: }
arch="$(echo $target_triple | awk -F - '{print $1}')"

target_dir="${CARGO_TARGET_DIR:-target}"

if [[ "$GRAM_BUILD_TARBALL" = "true" ]]; then
  no_build_flag=""
  if [ "$GRAM_BUILD_REMOTE" = false ]; then
    no_build_flag="--no-build-remote"
  fi

  ./script/bundle-linux --tarball $no_build_flag
  GRAM_BUNDLE_FILE="${target_dir}/release/gram-linux-$arch.tar.gz"
elif [ "$GRAM_BUNDLE_FILE" = "" ]; then
  GRAM_BUNDLE_FILE="gram-linux-$arch-$version.tar.gz"
  curl --skip-existing -L -O https://codeberg.org/GramEditor/gram/releases/download/"$version"/"$GRAM_BUNDLE_FILE"
fi
[[ ! -f "$GRAM_BUNDLE_FILE" ]] && err "$GRAM_BUNDLE_FILE not found, exiting..."

channel=stable
if tar ztf "$GRAM_BUNDLE_FILE" | head -1 | grep -q "dev"; then
  channel=dev
fi
appid="app.liten.Gram"
suffix=""
if [ "$channel" != "stable" ]; then
  suffix="-$channel"
  appid="app.liten.Gram-Dev"
fi
mkdir -p "$GRAM_INSTALL_PREFIX/gram$suffix.app"
mkdir -p "$GRAM_INSTALL_PREFIX/bin" "$GRAM_INSTALL_PREFIX/share/applications"
tar -xzf "$GRAM_BUNDLE_FILE" -C "$GRAM_INSTALL_PREFIX/"

ln -sf "$GRAM_INSTALL_PREFIX/gram$suffix.app/bin/gram" "$HOME/.local/bin/gram"

desktop_file_path="$GRAM_INSTALL_PREFIX/share/applications/${appid}.desktop"
src_dir="$GRAM_INSTALL_PREFIX/gram$suffix.app/share/applications"
cp "$src_dir/gram${suffix}.desktop" "${desktop_file_path}"

sed -i -e "s|Icon=gram|Icon=$GRAM_INSTALL_PREFIX/gram$suffix.app/share/icons/hicolor/512x512/apps/gram.png|g" "${desktop_file_path}"
sed -i -e "s|Exec=gram|Exec=$GRAM_INSTALL_PREFIX/gram$suffix.app/bin/gram|g" "${desktop_file_path}"

echo "Installation to $GRAM_INSTALL_PREFIX complete."

