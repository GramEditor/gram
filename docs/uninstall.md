# Uninstall

This guide covers how to uninstall Gram on different operating systems.

## macOS

### Bundle compilation

If you installed Gram by compiling it using the `script/bundle-mac` command:

1. Quit Gram if it's running
2. Open Finder and go to your Applications folder
3. Drag Gram to the Trash (or right-click and select "Move to Trash")
4. Empty the Trash

### Removing User Data (Optional)

To completely remove all Gram configuration files and data:

1. Open Finder
2. Press `Cmd + Shift + G` to open "Go to Folder"
3. Delete the following directories if they exist:
   - `~/Library/Application Support/Gram`
   - `~/Library/Saved Application State/app.liten.Gram.savedState`
   - `~/Library/Logs/Gram`
   - `~/Library/Caches/app.liten.Gram`

## Linux

### Standard Uninstall

If Gram was installed using the default installation script, run:

```sh
gram --uninstall
```

You'll be prompted whether to keep or delete your preferences. After making a
choice, you should see a message that Gram was successfully uninstalled.

If the `gram` command is not found in your PATH, try:

```sh
$HOME/.local/bin/gram --uninstall
```

or:

```sh
$HOME/.local/gram.app/bin/gram --uninstall
```

### Package Manager

If you installed Gram using a package manager (such as Flatpak or a
distribution-specific package manager), consult that package manager's
documentation for uninstallation instructions.

### Manual Removal

If the uninstall command fails or Gram was installed to a custom location, you
can manually remove:

- Installation directory: `~/.local/gram.app` (or your custom installation path)
- Binary symlink: `~/.local/bin/gram`
- Configuration and data: `~/.config/gram`

## Windows

### Standard Installation

1. Quit Gram if it's running
2. Open Settings (Windows key + I)
3. Go to "Apps" > "Installed apps" (or "Apps & features" on Windows 10)
4. Search for "Gram"
5. Click the three dots menu next to Gram and select "Uninstall"
6. Follow the prompts to complete the uninstallation

Alternatively, you can:

1. Open the Start menu
2. Right-click on Gram
3. Select "Uninstall"

### Removing User Data (Optional)

To completely remove all Gram configuration files and data:

1. Press `Windows key + R` to open Run
2. Type `%APPDATA%` and press Enter
3. Delete the `Gram` folder if it exists
4. Press `Windows key + R` again, type `%LOCALAPPDATA%` and press Enter
5. Delete the `Gram` folder if it exists

## Troubleshooting

If you encounter issues during uninstallation:

- **macOS/Windows**: Ensure Gram is completely quit before attempting to
  uninstall. Check Activity Manager (macOS) or Task Manager (Windows) for any
  running Gram processes.
- **Linux**: If the uninstall script fails, check the error message and consider
  manual removal of the directories listed above.
- **All platforms**: If you want to start fresh while keeping Gram installed,
  you can delete the configuration directories instead of uninstalling the
  application entirely.

For additional help, see the [Linux-specific documentation](./linux.md).
