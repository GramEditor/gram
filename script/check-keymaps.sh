#!/bin/sh
set -eu

result=$(git grep --no-color --line-number --fixed-strings -e 'cmd-' -- \
	'assets/keymaps/' \
	':(exclude)assets/keymaps/storybook.jsonc' \
	':(exclude)assets/keymaps/default-macos.jsonc' \
	':(exclude)assets/keymaps/macos/*.jsonc' || true)
if [ -n "$result" ]; then
	printf '%s\n' "$result"
	echo "Error: Found 'cmd-' in non-macOS keymap files." >&2
	exit 1
fi

result=$(git grep --no-color --line-number --extended-regexp -e 'super-|win-|fn-' -- \
	'assets/keymaps/' || true)
if [ -n "$result" ]; then
	printf '%s\n' "$result"
	echo "Error: Found 'super-', 'win-', or 'fn-' in keymap files. Currently these aren't used." >&2
	exit 1
fi
