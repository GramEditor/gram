[CmdletBinding()]
Param(
    [Parameter()][Alias('i')][switch]$Install,
    [Parameter()][Alias('h')][switch]$Help,
    [Parameter()][Alias('a')][string]$Architecture,
    [Parameter()][string]$Name
)

. "$PSScriptRoot/workspace.ps1"

# https://stackoverflow.com/questions/57949031/powershell-script-stops-if-program-fails-like-bash-set-o-errexit
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$buildSuccess = $false

$OSArchitecture = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
    "X64" { "x86_64" }
    "Arm64" { "aarch64" }
    default { throw "Unsupported architecture" }
}

$Architecture = if ($Architecture) {
    $Architecture
} else {
    $OSArchitecture
}

$CargoOutDir = "./target/$Architecture-pc-windows-msvc/release"

function Get-VSArch {
    param(
        [string]$Arch
    )

    switch ($Arch) {
        "x86_64" { "amd64" }
        "aarch64" { "arm64" }
    }
}

$VSDevPath = Join-Path -Path ((& 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe' -latest -format json | ConvertFrom-Json).installationPath) -ChildPath "\Common7\Tools\Launch-VsDevShell.ps1"

Push-Location
& $VSDevPath -Arch (Get-VSArch -Arch $Architecture) -HostArch (Get-VSArch -Arch $OSArchitecture)
Pop-Location

$target = "$Architecture-pc-windows-msvc"

if ($Help) {
    Write-Output "Usage: test.ps1 [-Install] [-Help]"
    Write-Output "Build the installer for Windows.\n"
    Write-Output "Options:"
    Write-Output "  -Architecture, -a Which architecture to build (x86_64 or aarch64)"
    Write-Output "  -Install, -i      Run the installer after building."
    Write-Output "  -Help, -h         Show this help message."
    exit 0
}

Push-Location -Path crates/gram
$channel = Get-Content "RELEASE_CHANNEL"
$env:GRAM_RELEASE_CHANNEL = $channel
$env:RELEASE_CHANNEL = $channel
Pop-Location

function CheckEnvironmentVariables {
    $requiredVars = @(
        'GRAM_WORKSPACE', 'RELEASE_VERSION', 'GRAM_RELEASE_CHANNEL'
    )

    foreach ($var in $requiredVars) {
        if (-not (Test-Path "env:$var")) {
            Write-Error "$var is not set"
            exit 1
        }
    }
}

function PrepareForBundle {
    if (Test-Path "$innoDir") {
        Remove-Item -Path "$innoDir" -Recurse -Force
    }
    New-Item -Path "$innoDir" -ItemType Directory -Force
    Copy-Item -Path "$env:GRAM_WORKSPACE\crates\gram\resources\windows\*" -Destination "$innoDir" -Recurse -Force
    New-Item -Path "$innoDir\bin" -ItemType Directory -Force
    New-Item -Path "$innoDir\tools" -ItemType Directory -Force

    rustup target add $target
}

function BuildGramAndItsFriends {
    Write-Output "Building Gram and its friends, for channel: $channel"
    # Build gram.exe, cli.exe
    cargo build --release --package gram --package cli --target $target
    Copy-Item -Path ".\$CargoOutDir\gram.exe" -Destination "$innoDir\Gram.exe" -Force
    Copy-Item -Path ".\$CargoOutDir\cli.exe" -Destination "$innoDir\cli.exe" -Force
}

function ZipGramAndItsFriendsDebug {
    $items = @(
        ".\$CargoOutDir\gram.pdb",
        ".\$CargoOutDir\cli.pdb"
    )

    Compress-Archive -Path $items -DestinationPath ".\$CargoOutDir\gram-$env:RELEASE_VERSION-$env:GRAM_RELEASE_CHANNEL.dbg.zip" -Force
}

function CollectFiles {
    Move-Item -Path "$innoDir\cli.exe" -Destination "$innoDir\bin\gram.exe" -Force
    Move-Item -Path "$innoDir\gram.sh" -Destination "$innoDir\bin\gram" -Force
}

function BuildInstaller {
    $issFilePath = "$innoDir\gram.iss"
    switch ($channel) {
        "stable" {
            $appId = "{{E62BA84E-40DF-471F-97EF-B85924F488FB}"
            $appIconName = "app-icon"
            $appName = "Gram"
            $appDisplayName = "Gram"
            $appSetupName = "Gram-$Architecture"
            # The mutex name here should match the mutex name in crates\gram\src\gram\windows_only_instance.rs
            $appMutex = "Gram-Stable-Instance-Mutex"
            $appExeName = "Gram"
            $regValueName = "Gram"
            $appUserId = "Gram.Gram"
            $appShellNameShort = "G&ram"
        }
        "dev" {
            $appId = "{{4FEF353A-EA46-468C-95DD-2B343A71416F}"
            $appIconName = "app-icon-dev"
            $appName = "Gram Dev"
            $appDisplayName = "Gram Dev"
            $appSetupName = "Gram-$Architecture"
            # The mutex name here should match the mutex name in crates\gram\src\gram\windows_only_instance.rs
            $appMutex = "Gram-Dev-Instance-Mutex"
            $appExeName = "Gram"
            $regValueName = "GramDev"
            $appUserId = "Gram.Gram.Dev"
            $appShellNameShort = "G&ram Dev"
        }
        default {
            Write-Error "can't bundle installer for $channel."
            exit 1
        }
    }

    # Windows runner 2022 default has iscc in PATH, https://github.com/actions/runner-images/blob/main/images/windows/Windows2022-Readme.md
    # Currently, we are using Windows 2022 runner.
    # Windows runner 2025 doesn't have iscc in PATH for now, https://github.com/actions/runner-images/issues/11228
    $innoSetupPath = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"

    $definitions = @{
        "AppId"          = $appId
        "AppIconName"    = $appIconName
        "OutputDir"      = "$env:GRAM_WORKSPACE\target"
        "AppSetupName"   = $appSetupName
        "AppName"        = $appName
        "AppDisplayName" = $appDisplayName
        "RegValueName"   = $regValueName
        "AppMutex"       = $appMutex
        "AppExeName"     = $appExeName
        "ResourcesDir"   = "$innoDir"
        "ShellNameShort" = $appShellNameShort
        "AppUserId"      = $appUserId
        "Version"        = "$env:RELEASE_VERSION"
        "SourceDir"      = "$env:GRAM_WORKSPACE"
    }

    $defs = @()
    foreach ($key in $definitions.Keys) {
        $defs += "/d$key=`"$($definitions[$key])`""
    }

    $innoArgs = @($issFilePath) + $defs

    # Execute Inno Setup
    Write-Host "🚀 Running Inno Setup: $innoSetupPath $innoArgs"
    $process = Start-Process -FilePath $innoSetupPath -ArgumentList $innoArgs -NoNewWindow -Wait -PassThru

    if ($process.ExitCode -eq 0) {
        Write-Host "✅ Inno Setup successfully compiled the installer"
        Write-Output "SETUP_PATH=target/$appSetupName.exe" >> $env:GITHUB_ENV
        $script:buildSuccess = $true
    }
    else {
        Write-Host "❌ Inno Setup failed: $($process.ExitCode)"
        $script:buildSuccess = $false
    }
}

ParseGramWorkspace
$innoDir = "$env:GRAM_WORKSPACE\inno\$Architecture"
$debugArchive = "$CargoOutDir\gram-$env:RELEASE_VERSION-$env:GRAM_RELEASE_CHANNEL.dbg.zip"
$debugStoreKey = "$env:GRAM_RELEASE_CHANNEL/gram-$env:RELEASE_VERSION-$env:GRAM_RELEASE_CHANNEL.dbg.zip"

CheckEnvironmentVariables
PrepareForBundle
GenerateLicenses
BuildGramAndItsFriends
ZipGramAndItsFriendsDebug
CollectFiles
BuildInstaller

if ($buildSuccess) {
    Write-Output "Build successful"
    if ($Install) {
        Write-Output "Installing Gram..."
        Start-Process -FilePath "$env:GRAM_WORKSPACE/target/GramEditorUserSetup-x64-$env:RELEASE_VERSION.exe"
    }
    exit 0
}
else {
    Write-Output "Build failed"
    exit 1
}
