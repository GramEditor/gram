#!/bin/sh
set -eu

script_dir="$(cd "$(dirname "$0")" && pwd)"

usage() {
	cat <<'EOF'
Usage: ./script/dev.sh <target> [args...]

Targets:
	debug      Run Gram with GRAM_STATELESS=1
	debug-cli  Run Gram CLI
	fmt        Format all Rust files
	check      Check formatting
	lint       Run clippy
	test       Run the test suite
	clean      Remove cargo artifacts
EOF
}

target="${1:-}"
if [ "$target" = "" ] || [ "$target" = "-h" ] || [ "$target" = "--help" ]; then
	usage
	exit 0
fi
shift

case "$target" in
debug)
	GRAM_STATELESS=1 exec cargo run "$@"
	;;
debug-cli)
	cargo build -p gram
	exec cargo run -p cli -- --foreground "--gram=${CARGO_TARGET_DIR:-target}/debug/gram" "$@"
	;;
fmt)
	exec cargo fmt --all "$@"
	;;
check)
	cargo fmt --all -- --check
	cargo check --workspace --all-targets --locked
	exec "$script_dir/shellcheck.sh"
	exec "$script_dir/clippy.sh"
	;;
lint)
	exec "$script_dir/clippy.sh" "$@"
	;;
test)
	if command -v cargo-nextest >/dev/null 2>&1; then
		exec cargo nextest run --workspace --no-fail-fast --locked "$@"
	fi
	echo "cargo-nextest not found, using 'cargo test'." >&2
	exec cargo test --workspace --no-fail-fast --locked "$@"
	;;
clean)
	exec cargo clean "$@"
	;;
*)
	echo "Unknown target: $target" >&2
	usage >&2
	exit 2
	;;
esac
