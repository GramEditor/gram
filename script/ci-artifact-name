#!/bin/sh
set -eu

arch=$(uname -m)
version=-dev
debversion=dev
case "${FORGEJO_REF_TYPE:-}" in
tag)
	version="-$FORGEJO_REF_NAME"
	debversion="$FORGEJO_REF_NAME"
	;;
branch)
	version="-$FORGEJO_SHA"
	debversion="$FORGEJO_SHA"
	;;
esac

rpm_name=$(find target -name '*.rpm' -print -quit)
deb_name=$(find target -name '*.deb' -print -quit)
if [ -z "$rpm_name" ] || [ -z "$deb_name" ]; then
	echo "Error: no .deb/.rpm found under target/" >&2
	exit 1
fi

mv "target/release/gram-linux-$arch.tar.gz" "gram-linux-$arch$version.tar.gz"
mv "target/gram-remote-server-linux-$arch.gz" "gram-remote-server-linux-$arch$version.gz"
mv "$deb_name" "gram_${debversion}-1_$arch.deb"
mv "$rpm_name" "gram_${debversion}-1_$arch.rpm"

{
	echo "tarball_amd64=gram-linux-$arch$version.tar.gz"
	echo "remote_server_amd64=gram-remote-server-linux-$arch$version.gz"
	echo "deb_amd64=gram_${debversion}-1_$arch.deb"
	echo "rpm_amd64=gram_${debversion}-1_$arch.rpm"
} >>"${FORGEJO_OUTPUT:?FORGEJO_OUTPUT is not set}"
