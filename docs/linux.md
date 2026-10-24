# Gram on Linux

See also [Repology](https://repology.org/project/gram/versions).

## Distro Packages

The preferred installation method is adding the Gram repository for automatic
updates. Alternatively, prebuilt `.deb` and `.rpm` packages are available as
release assets at
[Gram releases](https://codeberg.org/GramEditor/gram/releases).

### Alpine

Alpine Linux provides official packages in the
[testing](https://pkgs.alpinelinux.org/package/edge/testing/x86_64/gram)
repository. Enable the testing repository, then run:

```bash
doas apk add gram@testing
```

### Arch

Arch Linux provides official packages in the
[extra](https://archlinux.org/packages/extra/x86_64/gram) repository.

```bash
sudo pacman -S gram
```

Alternatively, build and install a development snapshot from the AUR:
[`gram-git`](https://aur.archlinux.org/packages/gram-git). Verify AUR package
integrity yourself.

### Debian/Ubuntu

Requires Debian 12 (Bookworm), Ubuntu 24.04 (Noble), or later.

```bash
# Add the repository key
sudo curl https://codeberg.org/api/packages/GramEditor/debian/repository.key -o /etc/apt/keyrings/forgejo-GramEditor.asc
# Add the repository
echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/forgejo-GramEditor.asc] https://codeberg.org/api/packages/GramEditor/debian gram release" | sudo tee -a /etc/apt/sources.list.d/gram.list
# Update package cache and install
sudo apt update && sudo apt install gram
```

### Fedora/RHEL/Rocky/Alma

Requires Fedora 42, RHEL 10.1, Rocky 10.1, Alma 10.1, or later.

#### Codeberg Repository

```bash
sudo dnf config-manager addrepo --from-repofile="https://codeberg.org/api/packages/GramEditor/rpm.repo"
sudo dnf install gram
```

> [!NOTE]
>
> RPM package signing is currently non-functional. To install Gram, set
> `gpgcheck=0` in `/etc/yum.repos.d/rpm.repo`.

#### Terra Repository

```bash
sudo dnf install --nogpgcheck --repofrompath 'terra,https://repos.fyralabs.com/terra$releasever' terra-release && sudo dnf install gram

# For immutable/atomic editions
curl -fsSL https://github.com/terrapkg/subatomic-repos/raw/main/terra.repo | pkexec tee /etc/yum.repos.d/terra.repo
sudo rpm-ostree install terra-release && sudo dnf install gram
```

### Gentoo

> [!NOTE]
>
> Gentoo packages are provided via Gram's overlay. For more information and
> installation methods, see
> [Gram's overlay repository](https://codeberg.org/GramEditor/gram-gentoo).

```bash
sudo eselect repository add gram git https://codeberg.org/GramEditor/gram-gentoo.git
sudo emerge --sync gram
sudo emerge -av app-editors/gram
```

### Nix

See: <https://github.com/NixOS/nixpkgs/tree/master/pkgs/by-name/gr/gram>.

#### Modern Nix (Recommended)

```bash
# Temporary environment
nix shell nixpkgs#gram

# User profile installation
nix profile add nixpkgs#gram
```

#### NixOS Configuration

```nix
environment.systemPackages = [
  pkgs.gram
];
```

#### Legacy Nix (Channels)

```bash
# On NixOS
nix-env -iA nixos.gram

# On non-NixOS Linux
nix-env -iA nixpkgs.gram
```

### OpenSUSE

Requires Leap 16 (or Tumbleweed/Slowroll) or later.

```bash
sudo zypper addrepo https://codeberg.org/api/packages/GramEditor/rpm.repo
sudo zypper install gram
```

> [!NOTE]
>
> RPM package signing is currently non-functional. To install Gram, set
> `gpgcheck=0` in `/etc/yum.repos.d/rpm.repo`.

### Solus

```bash
sudo eopkg install gram
```

## Flatpak

Prebuilt Flatpaks are available as release assets at
[Gram releases](https://codeberg.org/GramEditor/gram/releases). Install via:

```bash
flatpak install /path/to/app.liten.Gram-x86_64-${version}.flatpak
```

## From Tarball

If a tarball is available for your architecture at the
[Gram releases](https://codeberg.org/GramEditor/gram/releases) repository:

1. Download the
   [install.sh](https://codeberg.org/GramEditor/gram/raw/branch/main/script/install.sh)
   script.
2. Run the script.

```bash
./install.sh
```

This downloads and installs the latest Gram release to `$HOME/.local`. Use
`--prefix` to install to a different location:

```bash
./install.sh --prefix /usr/local ./gram-linux-x86_64-VERSION.tar.gz
```

Replace `VERSION` with the release version. A nonstandard prefix must be included
in the desktop's data search path for its launcher and icons to be discoverable.

## From Source

Gram is open source. See [developer notes](./development/linux.md) for build
instructions.

## Troubleshooting

### Graphics Issues

#### Gram fails to open a window

Gram requires a GPU and uses [Vulkan](https://www.vulkan.org) for rendering. If
Gram fails to load or perform well, Vulkan may be the cause.

A `Gram failed to open a window: NoSupportedDeviceFound` error indicates Vulkan
cannot find a compatible GPU. Run [`vkcube`](https://github.com/krh/vkcube)
command (usually in the `vulkaninfo` or `vulkan-tools` package) to troubleshoot.

> [!NOTE]
>
> Test both X11 and Wayland modes by running `vkcube -m x11` or `vkcube -m
> wayland`. Some distributions use `vkcube-wayland` for Wayland.

This outputs your graphics setup and displays a rotating cube. If it fails,
install Vulkan-compatible GPU drivers. Note that some hardware lacks Vulkan
support.

Check which GPU Gram uses by searching for `Using GPU: ...` in
`~/.local/share/gram/logs/Gram.log`.

Errors like `ERROR_INITIALIZATION_FAILED`, `GPU Crashed`, or
`ERROR_SURFACE_LOST_KHR` may be resolved by installing different GPU drivers or
selecting a different GPU (see
[#14225](https://github.com/zed-industries/zed/issues/14225)).

- **PRIME**: On some systems, set `/etc/prime-discrete` to `on` (force discrete)
  or `off` (force integrated).
- **DRI_PRIME**: Set the environment variable `DRI_PRIME=1` when running Gram to
  force the discrete GPU.
- **AMD Long Lines**: If Gram crashes when selecting long lines on an AMD GPU,
  set `GRAM_PATH_SAMPLE_COUNT=0` (see
  [#26143](https://github.com/zed-industries/zed/issues/26143)).
- **AMD Broken Pipe**: If you encounter a 'Broken Pipe' error on AMD, try using
  RADV or Mesa drivers (see
  [#13880](https://github.com/zed-industries/zed/issues/13880)).
- **AMDVLK**: If Gram consistently fails to launch with the default `amdvlk`
  driver (e.g., on Omarchy, see
  [#28851](https://github.com/zed-industries/zed/issues/28851)), remove `amdvlk`
  and `lib32-amdvlk`, and install `vulkan-radeon` instead (see
  [#14141](https://github.com/zed-industries/zed/issues/14141)).

For more details, refer to the
[Arch Linux Vulkan guide](https://wiki.archlinux.org/title/Vulkan).

#### Forcing Gram to use a specific GPU

**Option A: Environment Variable** Use `GRAM_DEVICE_ID={device_id}` to specify
the GPU. Find the device ID via `lspci -nn | grep VGA`:

```bash
08:00.0 VGA compatible controller [0300]: NVIDIA Corporation GA104 [GeForce RTX 3070] [10de:2484] (rev a1)
```

The device ID is `2484` (hexadecimal). Force Gram to use it:

```bash
GRAM_DEVICE_ID=0x2484 gram
```

Export this variable in `.bashrc` or similar if defining it globally.

**Option B: Mesa Device Select** If using Mesa, list available GPUs:

```bash
MESA_VK_DEVICE_SELECT=list gram --foreground
```

Then export `MESA_VK_DEVICE_SELECT=xxxx:yyyy` to choose a specific device. To
fallback to Xwayland, additionally export `WAYLAND_DISPLAY=""`.

**Option C: VKDeviceChooser** Use
[vkdevicechooser](https://github.com/jiriks74/vkdevicechooser).

#### Generating Debug Reports

Pass `--system-specs` to print system specifications:

```bash
gram --system-specs
```

The editor log is located at `~/.local/share/gram/logs/Gram.log`. To generate a
clean log for debugging graphics issues:

```bash
truncate -s 0 ~/.local/share/gram/logs/Gram.log # Clear the log file
GRAM_LOG=wgpu=info gram .
cat ~/.local/share/gram/logs/Gram.log
# Copy the output
```

Or, if the Gram CLI is set up:

```bash
GRAM_LOG=wgpu=info /path/to/gram/cli --foreground .
# Copy the output
```

### Forcing X11 Scale Factor

On X11, Gram automatically detects the scale factor for high-DPI displays using
this priority:

1. `GPUI_X11_SCALE_FACTOR` environment variable (if set)
2. `Xft.dpi` from the X resources database (xrdb)
3. Automatic detection via RandR (monitor resolution and physical size)

To customize the scale factor:

#### Check Current Scale Factor

Verify if `Xft.dpi` is set:

```bash
xrdb -query | grep Xft.dpi
```

If empty, Gram uses RandR for automatic calculation.

#### Option 1: Set Xft.dpi (X Resources Database)

`Xft.dpi` ensures consistent scaling across X11 applications. Edit or create
`~/.Xresources`:

```bash
vim ~/.Xresources
```

Add your desired DPI (e.g., `96` for 1x, `144` for 1.5x, `192` for 2x, `288` for
3x):

```ini
Xft.dpi: 96
```

Load the configuration:

```bash
xrdb -merge ~/.Xresources
```

Restart Gram.

#### Option 2: Use GPUI_X11_SCALE_FACTOR

This Gram-specific variable bypasses automatic detection.

```bash
GPUI_X11_SCALE_FACTOR=1.5 gram
```

Accepts decimal values (e.g., `1.25`, `1.5`, `2.0`) or `randr` to force RandR
detection even if `Xft.dpi` is set. Add to your shell profile or desktop entry
to make it permanent.

#### Option 3: Adjust System-Wide RandR DPI

This changes the reported DPI for the entire X11 session. Add to `.xprofile` or
`.xinitrc`:

```bash
xrandr --dpi 192
```

Replace `192` with your desired DPI.

### Font Rendering Parameters

On Linux, Gram reads the following environment variables for font rendering:

- `GRAM_FONTS_GAMMA`: Corresponds to
  [GetGamma](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwriterenderingparams-getgamma)
  values. Allowed range: `[1.0, 2.2]` (others are clipped). Default: `1.8`.
- `GRAM_FONTS_GRAYSCALE_ENHANCED_CONTRAST`: Corresponds to
  [GetGrayscaleEnhancedContrast](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_1/nf-dwrite_1-idwriterenderingparams1-getgrayscaleenhancedcontrast)
  values. Allowed range: `[0.0, ∞)` (others are clipped). Default: `1.0`.
