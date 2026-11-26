#!/usr/bin/env bash
set -euo pipefail

build_flag="--release"
target_dir="release"
do_build=true
open_result=false
local_install=false
can_code_sign=false

CARGO_BUNDLE_VERSION="0.11.0"

# The following environment variables
# must be set:
#
# APPLE_ID
# APPLE_PASSWORD_GRAM
# APPLE_TEAM_ID
# APPLE_SIGNING_KEY
#
# APPLE_PASSWORD_GRAM must be an App-specific password.
# Create it by logging in to account.apple.com and
# click the "App-specific Passwords" button.
#
# Get the signing certificate via XCode, and view
# the signing key using:
#   security find-identity -p codesigning
if [[ -f "$HOME/.local.secrets" ]]; then
    # shellcheck disable=SC1091
    . "$HOME/.local.secrets"
    export APPLE_ID
    export APPLE_PASSWORD_GRAM
    export APPLE_TEAM_ID
    export APPLE_SIGNING_KEY
fi

help_info() {
    echo "
Usage: ${0##*/} [options] [architecture=host]

Build the application bundle for macOS.

Options:
  -b    Skip building binaries (assume they're already built).
  -d    Compile in debug mode.
  -o    Open dir with the resulting DMG or launch the app itself in local mode.
  -i    Install the resulting DMG into /Applications.
  -h    Display this help and exit.
"
}

while getopts 'bdloih' flag; do
    case "${flag}" in
        b) do_build=false ;;
        d)
            export CARGO_INCREMENTAL=true
            export CARGO_BUNDLE_SKIP_BUILD=true
            build_flag=""
            target_dir="debug"
            ;;
        o) open_result=true ;;
        i) local_install=true ;;
        h)
            help_info
            exit 0
            ;;
        *)
            echo "Error: Unexpected argument: $1" >&2
            help_info
            exit 1
            ;;
    esac
done
shift $((OPTIND - 1))

# Get release channel
pushd crates/gram >/dev/null || exit 1
channel=$(<RELEASE_CHANNEL)
export GRAM_RELEASE_CHANNEL="${channel}"
popd >/dev/null || exit 1

export GRAM_BUNDLE=true

if [[ "$do_build" = "true" ]]; then
    # `|| true` swallows the SIGPIPE that `head` causes under pipefail.
    cargo_bundle_version=$(cargo -q bundle --help 2>&1 | head -n 1 || true)
    if [[ "$cargo_bundle_version" != *"$CARGO_BUNDLE_VERSION"* ]]; then
        cargo install --version "$CARGO_BUNDLE_VERSION" cargo-bundle
    fi
fi

# Deal with versions of macOS that don't include libstdc++ headers
export CXXFLAGS="-stdlib=libc++"

version_info=$(rustc --version --verbose)
host_line=$(echo "$version_info" | grep host)
target_triple=${host_line#*: }

if [[ $# -gt 0 && -n "$1" ]]; then
    target_triple="$1"
fi

arch_suffix=""
if [[ "$target_triple" = "x86_64-apple-darwin" ]]; then
    arch_suffix="x86_64"
elif [[ "$target_triple" = "aarch64-apple-darwin" ]]; then
    arch_suffix="aarch64"
else
    echo "Unsupported architecture $target_triple" >&2
    exit 1
fi

if [[ "$do_build" = "true" ]]; then
    rustup target add "$target_triple"

    echo "Compiling gram binaries"
    # shellcheck disable=SC2086 # build_flag is intentionally word-split (empty in debug mode)
    cargo build ${build_flag} --package gram --package cli --target "$target_triple"

    # Build remote_server in separate invocation to prevent feature unification from other crates
    # from influencing dynamic libraries required by it.
    echo "Compiling remote_server for $target_triple"
    # shellcheck disable=SC2086
    cargo build ${build_flag} --package remote_server --target "$target_triple"
fi

echo "Creating application bundle"
pushd crates/gram >/dev/null || exit 1
cp Cargo.toml Cargo.toml.backup
sed \
    -i.backup \
    "s/package.metadata.bundle-${channel}/package.metadata.bundle/" \
    Cargo.toml
# shellcheck disable=SC2086
app_path=$(cargo bundle ${build_flag} --target "$target_triple" --format osx | xargs)
mv Cargo.toml.backup Cargo.toml
popd >/dev/null || exit 1

echo "Bundled ${app_path}"

if [[ -n "${APPLE_ID:-}" && -n "${APPLE_PASSWORD_GRAM:-}" && -n "${APPLE_TEAM_ID:-}" && -n "${APPLE_SIGNING_KEY:-}" ]]; then
    echo "Enabling code signing!"
    can_code_sign=true
fi

GIT_VERSION="v2.56.1"
GIT_VERSION_SHA="238326e"
GIT_ARM64_SHA256="a882ec6882416b57d212a22bac845ae27944f96cf73453a864f6a12c17863a79"
GIT_X64_SHA256="3fdb0c2f102fe0273d64b5d5fb77315c19f2b4648a154593723cb9ed98f2bc7d"

function download_and_unpack() {
    local url=$1
    local expected_sha256=$2
    local path_to_unpack=$3
    local target_path=$4
    local temp_dir
    temp_dir=$(mktemp -d)

    if ! command -v curl &>/dev/null; then
        echo "curl is not installed. Please install curl to continue." >&2
        exit 1
    fi

    # Verify the archive before unpacking: this binary ships inside the signed app.
    local archive="$temp_dir/archive.tar.gz"
    curl --silent --fail --location --output "$archive" "$url"
    local actual_sha256
    actual_sha256=$(shasum --algorithm 256 "$archive" | cut -d ' ' -f 1)
    if [[ "$actual_sha256" != "$expected_sha256" ]]; then
        echo "SHA-256 mismatch for $url: expected $expected_sha256, got $actual_sha256" >&2
        rm -rf "$temp_dir"
        exit 1
    fi

    tar -xvz -C "$temp_dir" -f "$archive" "$path_to_unpack"
    mv "$temp_dir/$path_to_unpack" "$target_path"
    rm -rf "$temp_dir"
}

function download_git() {
    local architecture=$1
    local target_binary=$2
    local tmp_dir
    tmp_dir=$(mktemp -d)
    pushd "$tmp_dir" >/dev/null || exit 1

    case "$architecture" in
        aarch64-apple-darwin)
            download_and_unpack "https://github.com/desktop/dugite-native/releases/download/${GIT_VERSION}/dugite-native-${GIT_VERSION}-${GIT_VERSION_SHA}-macOS-arm64.tar.gz" "$GIT_ARM64_SHA256" bin/git ./git
            ;;
        x86_64-apple-darwin)
            download_and_unpack "https://github.com/desktop/dugite-native/releases/download/${GIT_VERSION}/dugite-native-${GIT_VERSION}-${GIT_VERSION_SHA}-macOS-x64.tar.gz" "$GIT_X64_SHA256" bin/git ./git
            ;;
        *)
            echo "Unsupported architecture: $architecture" >&2
            exit 1
            ;;
    esac

    popd >/dev/null || exit 1
    mv "${tmp_dir}/git" "${target_binary}"
    rm -rf "$tmp_dir"
}

function sign_app_binaries() {
    rm -rf "${app_path}/Contents/Frameworks"
    mkdir -p "${app_path}/Contents/Frameworks"

    echo "Downloading git binary"
    download_git "${target_triple}" "${app_path}/Contents/MacOS/git"

    cp crates/gram/contents/embedded.provisionprofile "${app_path}/Contents/"

    if [[ $can_code_sign = true ]]; then
        echo "Code signing binaries"
        # sequence of codesign commands modeled after this example: https://developer.apple.com/forums/thread/701514
        /usr/bin/codesign --deep --force --timestamp --options runtime --sign "$APPLE_SIGNING_KEY" "${app_path}/Contents/MacOS/cli" -v
        /usr/bin/codesign --deep --force --timestamp --options runtime --sign "$APPLE_SIGNING_KEY" "${app_path}/Contents/MacOS/git" -v
        /usr/bin/codesign --deep --force --timestamp --options runtime --sign "$APPLE_SIGNING_KEY" "${app_path}/Contents/Resources/Assets.car" -v
        /usr/bin/codesign --deep --force --timestamp --options runtime --entitlements crates/gram/resources/gram.entitlements --sign "$APPLE_SIGNING_KEY" "${app_path}/Contents/MacOS/gram" -v
        /usr/bin/codesign --force --timestamp --options runtime --entitlements crates/gram/resources/gram.entitlements --sign "$APPLE_SIGNING_KEY" "${app_path}" -v
    elif [[ -n "${APPLE_SIGNING_KEY:-}" ]]; then
        echo "One or more of the following variables are missing: APPLE_ID, APPLE_PASSWORD_GRAM, APPLE_TEAM_ID"
        echo "The binary will be signed without all entitlements, so universal links won't work for example."
        sed '/com.apple.developer.associated-domains/,+1d' \
            crates/gram/resources/gram.entitlements >"${app_path}/Contents/Resources/gram.entitlements"
        codesign --force --deep --entitlements "${app_path}/Contents/Resources/gram.entitlements" --sign "${APPLE_SIGNING_KEY}" "${app_path}" -v
    fi

    bundle_name=$(basename "$app_path")

    if [ "$local_install" = true ]; then
        rm -rf "/Applications/$bundle_name"
        mv "$app_path" "/Applications/$bundle_name"
        echo "Installed application bundle: /Applications/$bundle_name"
        if [ "$open_result" = true ]; then
            echo "Opening /Applications/$bundle_name"
            open "/Applications/$bundle_name"
        fi
    elif [ "$open_result" = true ]; then
        open "$app_path"
    fi

    if [[ "$target_dir" = "debug" ]]; then
        echo "Debug build detected - skipping DMG creation and signing"
        if [ "$local_install" = false ]; then
            echo "Created application bundle:"
            echo "$app_path"
        fi
    else
        dmg_target_directory="target/${target_triple}/${target_dir}"
        dmg_source_directory="${dmg_target_directory}/dmg"
        dmg_file_path="${dmg_target_directory}/Gram-${arch_suffix}.dmg"
        xcode_bin_dir_path="$(xcode-select -p)/usr/bin"

        rm -rf "${dmg_source_directory}"
        mkdir -p "${dmg_source_directory}"
        mv "${app_path}" "${dmg_source_directory}"

        echo "Adding symlink to /Applications to ${dmg_source_directory}"
        ln -s /Applications "${dmg_source_directory}"

        echo "Creating final DMG at ${dmg_file_path} using ${dmg_source_directory}"
        diskutil image create from --volumeName Gram --format UDZO "${dmg_source_directory}" "${dmg_file_path}"

        # If someone runs this bundle script locally, a symlink will be placed in `dmg_source_directory`.
        # This symlink causes CPU issues with Gram if the Gram codebase is the project being worked on, so we simply remove it for now.
        echo "Removing symlink to /Applications from ${dmg_source_directory}"
        rm "${dmg_source_directory}/Applications"

        if [[ $can_code_sign = true ]]; then
            echo "Notarizing DMG with Apple"
            /usr/bin/codesign --deep --force --timestamp --options runtime --sign "$APPLE_SIGNING_KEY" "$(pwd)/${dmg_file_path}" -v
            "${xcode_bin_dir_path}/notarytool" submit --wait --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_PASSWORD_GRAM" "${dmg_file_path}"
            "${xcode_bin_dir_path}/stapler" staple "${dmg_file_path}"
        fi

        if [ "$open_result" = true ]; then
            open "$dmg_target_directory"
        fi
    fi
}

function sign_binary() {
    local binary_path=$1
    if [[ $can_code_sign = true ]]; then
        echo "Code signing executable $binary_path"
        /usr/bin/codesign --deep --force --timestamp --options runtime --entitlements crates/gram/resources/gram.entitlements --sign "$APPLE_SIGNING_KEY" "${binary_path}" -v
    fi
}

cp "target/${target_triple}/${target_dir}/gram" "${app_path}/Contents/MacOS/gram"
cp "target/${target_triple}/${target_dir}/cli" "${app_path}/Contents/MacOS/cli"

./script/mac-icon-to-car.sh
cp -r target/Assets.car "${app_path}/Contents/Resources/Assets.car"

plist_file="${app_path}/Contents/Info.plist"
sed 's|  <key>CFBundleIconFile</key>|  <key>CFBundleIconName</key>\
<string>AppIcon</string>\
<key>CFBundleIconFile</key>|' "$plist_file" >"${plist_file}.tmp"
mv "${plist_file}.tmp" "$plist_file"

sign_app_binaries

sign_binary "target/${target_triple}/${target_dir}/remote_server"
gzip -9 -f --to-stdout "target/${target_triple}/${target_dir}/remote_server" \
    >"target/gram-remote-server-macos-${arch_suffix}.gz"
