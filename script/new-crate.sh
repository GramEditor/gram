#!/bin/sh
set -eu

if [ ! -d crates ] || [ ! -d script ] || [ ! -f Cargo.toml ]; then
	echo "Error: Run from the \`gram\` repo root" >&2
	exit 1
fi

if [ $# -eq 0 ]; then
	echo "Usage: $0 <crate_name> [optional_license_flag]" >&2
	exit 1
fi

CRATE_NAME=$1
case "$CRATE_NAME" in
'' | *[!a-z0-9_]*)
	echo "Error: Crate name must be lowercase and contain only alphanumeric characters and underscores" >&2
	exit 1
	;;
esac

LICENSE_FLAG=$(printf '%s' "${2:-}" | tr '[:upper:]' '[:lower:]')
case "$LICENSE_FLAG" in
*apache*)
	LICENSE_MODE=Apache-2.0
	LICENSE_FILE=LICENSE-APACHE
	;;
*)
	LICENSE_MODE=GPL-3.0-or-later
	LICENSE_FILE=LICENSE-GPL
	;;
esac

CRATE_PATH="crates/$CRATE_NAME"
mkdir -p "$CRATE_PATH/src"
ln -sf "../../$LICENSE_FILE" "$CRATE_PATH/$LICENSE_FILE"

sed -e "s/\$CRATE_NAME/$CRATE_NAME/g" -e "s/\$LICENSE_MODE/$LICENSE_MODE/g" >"$CRATE_PATH/Cargo.toml" <<'EOF'
[package]
name = "$CRATE_NAME"
version.workspace = true
edition.workspace = true
publish.workspace = true
repository.workspace = true
license = "$LICENSE_MODE"

[lints]
workspace = true

[lib]
path = "src/$CRATE_NAME.rs"

[features]
default = []

[dependencies]
anyhow.workspace = true
gpui.workspace = true
ui.workspace = true
util.workspace = true

# Uncomment other workspace dependencies as needed
# app_actions.workspace = true
# fs.workspace = true
# project.workspace = true
# settings.workspace = true
EOF

printf '//! # %s\n' "$CRATE_NAME" >"$CRATE_PATH/src/$CRATE_NAME.rs"

echo "Created new crate: $CRATE_NAME in $CRATE_PATH"
echo "License: $LICENSE_MODE (symlinked from $LICENSE_FILE)"
echo "Don't forget to add the new crate to the workspace!"
