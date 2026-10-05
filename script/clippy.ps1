$ErrorActionPreference = "Stop"

Write-Host "Your PATH entries:"
$env:Path -split ";" | ForEach-Object { Write-Host "  $_" }

$clippyArgs = @($args)
$argsStr = " " + ($clippyArgs -join " ") + " "
if ($argsStr -notmatch "-p" -and $argsStr -notmatch "--package")
{
    $clippyArgs += "--workspace"
}

# https://stackoverflow.com/questions/41324882/how-to-run-a-powershell-script-with-verbose-output/70020655#70020655
# Set-PSDebug -Trace 2

$Cargo = if ($env:CARGO)
{
    $env:CARGO
} elseif (Get-Command "cargo" -ErrorAction SilentlyContinue)
{
    "cargo"
} else
{
    Write-Error "Could not find cargo in path." -ErrorAction Stop
}

& $Cargo clippy @clippyArgs --release --all-targets --all-features -- --deny warnings