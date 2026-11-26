#!/bin/sh
set -eu

case " $* " in
*" -p "* | *" --package "*) ;;
*) set -- "$@" --workspace ;;
esac

set -x
exec "${CARGO:-cargo}" clippy "$@" --release --all-targets --all-features -- --deny warnings
