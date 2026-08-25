# Gram on Linux

See also [Repology](https://repology.org/project/gram/versions).

## Distro Packages

The preferred way to install is via adding a Gram repository file and installing
from it. This provides the user with automatic updates once new packages are
released. See instructions below. Alternatively, Gram provides prebuilt `.deb`
and `.rpm` packages as release assets which can be downloaded at
[Gram releases](https://codeberg.org/GramEditor/gram/releases).

### Debian/Ubuntu

```bash
# Add the repository key
sudo curl https://codeberg.org/api/packages/GramEditor/debian/repository.key -o /etc/apt/keyrings/forgejo-GramEditor.asc
# Add the repository file
echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/forgejo-GramEditor.asc] https://codeberg.org/api/packages/GramEditor/debian gram release" | sudo tee -a /etc/apt/sources.list.d/gram.list
# Update package cache and install
sudo apt update && sudo apt install gram
```

Requires Debian 12 (Bookworm)/Ubuntu 24.04 (noble) or later.

### Fedora/RHEL/Rocky/Alma

```bash
sudo dnf config-manager addrepo --from-repofile="https://codeberg.org/api/packages/GramEditor/rpm.repo"
sudo dnf install gram
```

**Note**: At the time of this writing RPM package signing does not work. In
order to be able to install Gram you need to set `gpgcheck=0` inside
`/etc/yum.repos.d/rpm.repo`.

Requires Fedora 42/RHEL 10.1/Rocky 10.1/Alma 10.1 or later.

### OpenSUSE

```bash
sudo zypper addrepo https://codeberg.org/api/packages/GramEditor/rpm.repo
sudo zypper install gram
```

**Note**: At the time of this writing RPM package signing does not work. In
order to be able to install Gram you need to set `gpgcheck=0` inside
`/etc/yum.repos.d/rpm.repo`.

Requires Leap 16 (or Tumbleweed/Slowroll) or later.

### Arch

Arch Linux publishes official packages for Gram in their
[extra](https://archlinux.org/packages/extra/x86_64/gram/) repository.

```bash
sudo pacman -S gram
```

Alternatively, to build and install a development snapshot from the latest Git
HEAD, build the VCS package from the aur:
[`gram-git`](https://aur.archlinux.org/packages/gram-git). If you install
packages from the AUR, it is your responsibility to verify their integrity
yourself.

### Alpine

Alpine Linux publishes official packages for Gram in their
[testing](https://pkgs.alpinelinux.org/package/edge/testing/x86_64/gram)
repository. Follow the
[instructions](https://wiki.alpinelinux.org/wiki/Repositories#Using_testing_repository)
to enable the testing repository, then run

```bash
doas apk add gram@testing
```

### Gentoo GNU/Linux

Gentoo provided by Gram's overlay.

```bash
# replace doas if you using sudo
doas eselect repository add gram git https://codeberg.org/GramEditor/gram-gentoo.git
doas emerge --sync gram
doas emerge -av app-editors/gram
```

**Note**: For more information about Gentoo package and installation ways go to
[Gram's overlay repo](https://codeberg.org/GramEditor/gram-gentoo).

### Solus

```bash
sudo eopkg install gram
```

## Flatpak

Gram provides a prebuilt flatpak as a release asset. It can be downloaded from
at [Gram releases](https://codeberg.org/GramEditor/gram/releases) and installed
by running:

```bash
flatpak install /path/to/app.liten.Gram-x86_64-${version}.flatpak
```

## From Tarball

If there is a tarball available for your architecture at the
[Gram Codeberg](https://codeberg.org/GramEditor/gram/releases) repository, you
can follow these instructions:

1. Download the
   [install.sh](https://codeberg.org/GramEditor/gram/raw/branch/main/script/install.sh)
   script.
2. Run the script.

```bash
./install.sh
```

This will download latest release of Gram and install Gram to `$HOME/.local`. To
install system-wide, use the `--prefix PREFIX` argument:

```bash
./install.sh --prefix /usr/local ./gram-linux-x86_64-1.1.0.tar.gz
```

## From Source

Gram is open source, and you can install from source. See
[developer notes](./development/linux.md) for instructions.

## Troubleshooting

### Graphics issues

#### Gram fails to open windows

Gram requires a GPU to run effectively. Under the hood, it uses
[Vulkan](https://www.vulkan.org/) to communicate with the GPU. If you are seeing
problems with performance or Gram fails to load, it is possible that Vulkan is
the culprit.

If you see a notification saying
`Gram failed to open a window: NoSupportedDeviceFound` this means that Vulkan
cannot find a compatible GPU. Try running
[vkcube](https://github.com/krh/vkcube) (usually available as part of the
`vulkaninfo` or `vulkan-tools` package on various distributions) to troubleshoot
where the issue is coming from like so:

```text
vkcube
```

> **_Note_**: Try running in both X11 and wayland modes by running
> `vkcube -m [x11|wayland]`. Some versions of `vkcube` use `vkcube` to run in
> X11 and `vkcube-wayland` to run in wayland.

This should output a line describing your current graphics setup and show a
rotating cube. If this does not work, you should be able to fix it by installing
Vulkan compatible GPU drivers, however in some cases there is no Vulkan support
yet.

You can find out which graphics card Gram is using by looking in the Gram log
(`~/.local/share/gram/logs/Gram.log`) for `Using GPU: ...`.

If you see errors like `ERROR_INITIALIZATION_FAILED` or `GPU Crashed` or
`ERROR_SURFACE_LOST_KHR` then you may be able to work around this by installing
different drivers for your GPU, or by selecting a different GPU to run on. (See
[#14225](https://github.com/zed-industries/zed/issues/14225))

On some systems the file `/etc/prime-discrete` can be used to enforce the use of
a discrete GPU using [PRIME](https://wiki.archlinux.org/title/PRIME). Depending
on the details of your setup, you may need to change the contents of this file
to "on" (to force discrete graphics) or "off" (to force integrated graphics).

On others, you may be able to the environment variable `DRI_PRIME=1` when
running Gram to force the use of the discrete GPU.

If you're using an AMD GPU and Gram crashes when selecting long lines, try
setting the `GRAM_PATH_SAMPLE_COUNT=0` environment variable. (See
[#26143](https://github.com/zed-industries/zed/issues/26143))

If you're using an AMD GPU, you might get a 'Broken Pipe' error. Try using the
RADV or Mesa drivers. (See
[#13880](https://github.com/zed-industries/zed/issues/13880))

If you are using `amdvlk`, the default open-source AMD graphics driver, you may
find that Gram consistently fails to launch. This is a known issue for some
users, for example on Omarchy (see issue
[#28851](https://github.com/zed-industries/zed/issues/28851)). To fix this, you
will need to use a different driver. We recommend removing the `amdvlk` and
`lib32-amdvlk` packages and installing `vulkan-radeon` instead (see issue
[#14141](https://github.com/zed-industries/zed/issues/14141)).

For more information, the
[Arch guide to Vulkan](https://wiki.archlinux.org/title/Vulkan) has some good
steps that translate well to most distributions.

#### Forcing Gram to use a specific GPU

There are a few different ways to force Gram to use a specific GPU:

##### Option A

You can use the `GRAM_DEVICE_ID={device_id}` environment variable to specify the
device ID of the GPU you wish to have Gram use.

You can obtain the device ID of your GPU by running `lspci -nn | grep VGA` which
will output each GPU on one line like:

```bash
08:00.0 VGA compatible controller [0300]: NVIDIA Corporation GA104 [GeForce RTX 3070] [10de:2484] (rev a1)
```

where the device ID here is `2484`. This value is in hexadecimal, so to force
Gram to use this specific GPU you would set the environment variable like so:

```bash
GRAM_DEVICE_ID=0x2484 gram
```

Make sure to export the variable if you choose to define it globally in a
`.bashrc` or similar.

##### Option B

If you are using Mesa, you can run
`MESA_VK_DEVICE_SELECT=list gram --foreground` to get a list of available GPUs
and then export `MESA_VK_DEVICE_SELECT=xxxx:yyyy` to choose a specific device.
Furthermore, you can fallback to xwayland with an additional export of
`WAYLAND_DISPLAY=""`.

##### Option C

Using [vkdevicechooser](https://github.com/jiriks74/vkdevicechooser).

#### Generating debug reports

Passing the `--system-specs` flag to Gram like

```bash
gram --system-specs
```

will print the system specs to the terminal.

The editor log is usually located at `~/.local/share/gram/logs/Gram.log`.

To generate a clean log file for debugging graphics issues, run:

```bash
truncate -s 0 ~/.local/share/gram/logs/Gram.log # Clear the log file
GRAM_LOG=wgpu=info gram .
cat ~/.local/share/gram/logs/Gram.log
# copy the output
```

Or, if you have the Gram cli setup, you can do

```bash
GRAM_LOG=wgpu=info /path/to/gram/cli --foreground .
# copy the output
```

### Forcing X11 scale factor

On X11 systems, Gram automatically detects the appropriate scale factor for
high-DPI displays. The scale factor is determined using the following priority
order:

1. `GPUI_X11_SCALE_FACTOR` environment variable (if set)
2. `Xft.dpi` from X resources database (xrdb)
3. Automatic detection via RandR based on monitor resolution and physical size

If you want to customize the scale factor beyond what Gram detects
automatically, you have several options:

#### Check your current scale factor

You can verify if you have `Xft.dpi` set:

```bash
xrdb -query | grep Xft.dpi
```

If this command returns no output, Gram is using RandR (X11's monitor management
extension) to automatically calculate the scale factor based on your monitor's
reported resolution and physical dimensions.

#### Option 1: Set Xft.dpi (X Resources Database)

`Xft.dpi` is a standard X11 setting that many applications use for consistent
font and UI scaling. Setting this ensures Gram scales the same way as other X11
applications that respect this setting.

Edit or create the `~/.Xresources` file:

```bash
vim ~/.Xresources
```

Add this line with your desired DPI:

```bash
Xft.dpi: 96
```

Common DPI values:

- `96` for standard 1x scaling
- `144` for 1.5x scaling
- `192` for 2x scaling
- `288` for 3x scaling

Load the configuration:

```bash
xrdb -merge ~/.Xresources
```

Restart Gram for the changes to take effect.

#### Option 2: Use the GPUI_X11_SCALE_FACTOR environment variable

This Gram-specific environment variable directly sets the scale factor,
bypassing all automatic detection.

```bash
GPUI_X11_SCALE_FACTOR=1.5 gram
```

You can use decimal values (e.g., `1.25`, `1.5`, `2.0`) or set
`GPUI_X11_SCALE_FACTOR=randr` to force RandR-based detection even when `Xft.dpi`
is set.

To make this permanent, add it to your shell profile or desktop entry.

#### Option 3: Adjust system-wide RandR DPI

This changes the reported DPI for your entire X11 session, affecting how RandR
calculates scaling for all applications that use it.

Add this to your `.xprofile` or `.xinitrc`:

```bash
xrandr --dpi 192
```

Replace `192` with your desired DPI value. This affects the system globally and
will be used by Gram's automatic RandR detection when `Xft.dpi` is not set.

### Font rendering parameters

On Linux, the `GRAM_FONTS_GAMMA` and `GRAM_FONTS_GRAYSCALE_ENHANCED_CONTRAST`
environment variables are read for the values to use for font rendering.

`GRAM_FONTS_GAMMA` corresponds to
[getgamma](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwriterenderingparams-getgamma)
values. Allowed range [1.0, 2.2], other values are clipped. Default: 1.8

`GRAM_FONTS_GRAYSCALE_ENHANCED_CONTRAST` corresponds to
[getgrayscaleenhancedcontrast](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_1/nf-dwrite_1-idwriterenderingparams1-getgrayscaleenhancedcontrast)
values. Allowed range: [0.0, ..), other values are clipped. Default: 1.0
