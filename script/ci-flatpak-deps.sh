#!/bin/sh
set -eu

sudo=
if [ "$(id -u)" -ne 0 ]; then
	sudo=$(command -v sudo || command -v sudo-rs || command -v doas || true)
fi

${sudo:+"$sudo"} apt-get install -y flatpak flatpak-builder

flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo

arch=$(uname -m)
fd_version=25.08
flatpak install -y --user "org.freedesktop.Platform/$arch/$fd_version"
flatpak install -y --user "org.freedesktop.Sdk/$arch/$fd_version"
flatpak install -y --user "org.freedesktop.Sdk.Extension.rust-stable/$arch/$fd_version"
