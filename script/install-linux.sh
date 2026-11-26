#!/bin/sh
set -eu

if [ $# -gt 0 ]; then
	cat <<EOF

Usage: ${0##*/}

Builds and installs gram onto your system into ~/.local, making it available as ~/.local/bin/gram.
Before running this you should ensure you have all the build dependencies installed with './script/linux.sh'.
EOF
	exit 1
fi

exec ./script/install.sh --build
