#!/bin/sh
set -eu

manifests=$(git ls-files -- '**/*/Cargo.toml')
if [ -z "$manifests" ]; then
	echo "Error: no crate manifests found" >&2
	exit 1
fi

while IFS= read -r cargo_toml; do
	dir=$(dirname "$cargo_toml")
	found=false
	for license in LICENSE-GPL LICENSE-APACHE; do
		if [ -L "$dir/$license" ]; then
			if [ ! -e "$dir/$license" ]; then
				echo "Error: $dir/$license is a broken symlink." >&2
				exit 1
			fi
			found=true
			break
		elif [ -e "$dir/$license" ]; then
			echo "Error: $dir/$license exists but is not a symlink." >&2
			exit 1
		fi
	done
	if [ "$found" = false ]; then
		echo "Error: $dir does not contain a LICENSE-GPL or LICENSE-APACHE symlink" >&2
		exit 1
	fi
done <<EOF
$manifests
EOF

echo "check-licenses succeeded"
