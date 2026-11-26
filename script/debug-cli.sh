#!/bin/sh
set -eu

cargo build -p gram
exec cargo run -p cli -- --foreground "--gram=${CARGO_TARGET_DIR:-target}/debug/gram" "$@"
