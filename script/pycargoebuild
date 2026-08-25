#!/usr/bin/env bash
set -euo pipefail

get_priv_cmd() {
	if command -v doas >/dev/null 2>&1; then
		echo "doas"
	elif command -v sudo-rs >/dev/null 2>&1; then
		echo "sudo-rs"
	elif command -v sudo >/dev/null 2>&1; then
		echo "sudo"
	else
		echo ""
	fi
}

is_gentoo() {
	command -v emerge >/dev/null 2>&1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CURRENT_DIR="$PWD"
if [[ "$CURRENT_DIR" != "$SCRIPT_DIR" && "$CURRENT_DIR" != "$REPO_ROOT" ]]; then
	echo "Run this script from the repository root or the 'script' directory." >&2
	exit 1
fi

cd "$REPO_ROOT"

if [ -n "${1:-}" ]; then
	version="$1"
else
	if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
		tag=$(git describe --tags --exact-match 2>/dev/null || true)
		if [ -n "$tag" ]; then
			version="$tag"
		else
			version=$(git rev-parse --short HEAD)
		fi
	else
		version="dev"
	fi
fi

if ! command -v pycargoebuild >/dev/null 2>&1; then
	echo "pycargoebuild not found in PATH."
	if is_gentoo; then
		PRIV_CMD=$(get_priv_cmd)

		if [ -n "$PRIV_CMD" ]; then
			$PRIV_CMD emerge app-portage/pycargoebuild
		else
			emerge app-portage/pycargoebuild
		fi
	else
		PYTHON_CMD=""

		if command -v python3 >/dev/null 2>&1; then
			PYTHON_CMD="python3"
		elif command -v python >/dev/null 2>&1; then
			PYTHON_CMD="python"
		fi

		if [ -z "$PYTHON_CMD" ]; then
			echo "Python is not installed. Cannot proceed." >&2
			exit 1
		fi

		# TODO: uv?
		echo "Installing pycargoebuild via pip..."
		$PYTHON_CMD -m pip install --user --break-system-packages pycargoebuild 2>/dev/null ||
			$PYTHON_CMD -m pip install --user pycargoebuild

		if [[ -d "$HOME/.local/bin" ]] && [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
			export PATH="$HOME/.local/bin:$PATH"
		fi
	fi
fi

if ! is_gentoo; then
	TEMP_DIR=$(mktemp -d)
	trap 'rm -rf "$TEMP_DIR"' EXIT
	mkdir -p "$TEMP_DIR/portage/licenses" "$TEMP_DIR/portage/metadata"
	printf "[spdx-to-ebuild]\n" >"$TEMP_DIR/portage/metadata/license-mapping.conf"
	mkdir -p "$TEMP_DIR/mock_portage/portage"

	cat >"$TEMP_DIR/mock_portage/portage/__init__.py" <<'EOF'
import os

class Tree(dict):
    def __init__(self):
        super().__init__()
        self["porttree"] = self
        class _Repo:
            location = os.environ.get("MOCK_PORTAGE_DIR", "/tmp/portage")
        class _Repos(dict):
            def __getitem__(self, key):
                return _Repo()
        class _DBApi:
            repositories = _Repos()
        self.dbapi = _DBApi()
        self.settings = {}

def create_trees(*args, **kwargs):
    return {"porttree": Tree()}
EOF

	export MOCK_PORTAGE_DIR="$TEMP_DIR/portage"
	export PYTHONPATH="$TEMP_DIR/mock_portage"
fi

mkdir -p target/distdir
cd crates/gram
pycargoebuild -c -e --no-config --no-license --no-manifest --distdir ../../target/distdir --crate-tarball-path "../../gram-${version}-crates.tar.xz" --output "../../gram-${version}.ebuild" .
cd ../..

if command -v sha512sum >/dev/null 2>&1; then
	sha512sum -- "gram-${version}-crates.tar.xz"
elif command -v shasum >/dev/null 2>&1; then
	shasum -a 512 -- "gram-${version}-crates.tar.xz"
else
	echo "Neither sha512sum nor shasum found." >&2
	exit 1
fi
