#!/bin/sh
set -eu

: "${TOKEN:?TOKEN is not set}"
: "${DEB_FILE:?DEB_FILE is not set}"
: "${RPM_FILE:?RPM_FILE is not set}"

curl --fail-with-body --silent --show-error \
	--user "krig:$TOKEN" \
	--upload-file "$DEB_FILE" \
	'https://codeberg.org/api/packages/GramEditor/debian/pool/gram/release/upload'

curl --fail-with-body --silent --show-error \
	--user "krig:$TOKEN" \
	--upload-file "$RPM_FILE" \
	'https://codeberg.org/api/packages/GramEditor/rpm/upload?sign=true'
