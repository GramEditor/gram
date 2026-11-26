#!/usr/bin/env bash
set -euo pipefail

# One-time cleanup when upgrading a Linux tarball installation from Gram 3.3 to 3.4.
# Install Gram 3.4 first, then preview: bash "Cleanup Script for old Linux Installer"
# Remove the listed obsolete files: bash "Cleanup Script for old Linux Installer" --apply
# Use the original --prefix if needed; add --channel dev for a development install.
# Removes verified old launchers and PNGs; keeps current files, preferences, and projects.
prefix="$HOME/.local"
channel=stable
apply=false

usage() {
    echo "Usage: bash \"${0##*/}\" [--prefix PREFIX] [--channel stable|dev] [--apply]"
    echo "Install the new release first. Without --apply, only list obsolete files."
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix|--channel)
            option="$1"
            [[ $# -ge 2 ]] || { usage >&2; exit 1; }
            if [[ "$option" == --prefix ]]; then prefix="$2"; else channel="$2"; fi
            shift 2
            ;;
        --apply) apply=true; shift ;;
        -h|--help) usage; exit 0 ;;
        *) usage >&2; exit 1 ;;
    esac
done

case "$channel" in
    stable) suffix=""; app_id=app.liten.Gram ;;
    dev) suffix=-dev; app_id=app.liten.Gram-Dev ;;
    *) usage >&2; exit 1 ;;
esac
prefix=$(cd "$prefix" && pwd -P)
app_dir="$prefix/gram$suffix.app"
applications="$prefix/share/applications"
icon_theme="$prefix/share/icons/hicolor"
if [[ ! -f "$applications/$app_id.desktop" || ! -f "$app_dir/share/applications/$app_id.desktop" ||
      ! -f "$icon_theme/scalable/apps/$app_id.svg" ]]; then
    echo "Install the new release with its modern desktop entry and SVG icon first." >&2
    exit 1
fi

changed_desktop=false
changed_icons=false
remove_obsolete() {
    if $apply; then
        rm -- "$1"
        printf 'Removed: %s\n' "$1"
    else
        printf 'Would remove: %s\n' "$1"
    fi
}

owns_launcher() {
    grep -Eq '^Name=Gram( Dev| Devel)?$' "$1" || return 1
    local executable line
    for executable in "$app_dir/bin/gram" "$prefix/bin/gram"; do
        while IFS= read -r line; do
            case "$line" in
                "TryExec=$executable"|"Exec=$executable"|"Exec=$executable "*|\
                "Exec=\"$executable\""|"Exec=\"$executable\" "*) return 0 ;;
            esac
        done < "$1"
    done
    return 1
}

for name in "gram$suffix.desktop" "gram-editor$suffix.desktop" "gram-$app_id.desktop"; do
    file="$applications/$name"
    if [[ -f "$file" ]] && owns_launcher "$file"; then
        remove_obsolete "$file"
        changed_desktop=true
    fi
    # Old archive contents can survive extraction over an existing app bundle.
    file="$app_dir/share/applications/$name"
    if [[ -f "$file" ]] && grep -Eq '^Name=Gram( Dev| Devel)?$' "$file"; then
        remove_obsolete "$file"
    fi
done

for size in 512x512 1024x1024; do
    source_icon="$app_dir/share/icons/hicolor/$size/apps/gram.png"
    installed_icon="$icon_theme/$size/apps/gram.png"
    if [[ -f "$source_icon" ]]; then
        # Remove a shared-directory icon only if its bytes match the old bundle.
        if [[ -f "$installed_icon" ]] && cmp -s -- "$source_icon" "$installed_icon"; then
            remove_obsolete "$installed_icon"
            changed_icons=true
        fi
        remove_obsolete "$source_icon"
    fi
done

if $apply; then
    if $changed_icons && [[ -d "$icon_theme" ]]; then touch "$icon_theme"; fi
    if $changed_desktop && command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$applications" ||\
            printf '%s\n' 'Warning: could not refresh the desktop database.' >&2
    fi
fi
