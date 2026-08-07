# Serial CLI installer — Windows
#
# Detects the platform, downloads the matching release binary from GitHub,
# verifies its SHA-256 checksum, installs it, and adds it to the user PATH.
# Optionally registers the daemon to auto-start on boot (`--service`).
#
# Usage:
#   .\install.ps1 [-Version v0.6.0] [-Prefix C:\tools\serial-cli] [-Service] [-Uninstall]
#
# Examples:
#   .\install.ps1                         # latest release, no service
#   .\install.ps1 -Service                # latest release + auto-start on boot
#   .\install.ps1 -Version v0.6.0         # pinned version
#
# NOTE: `irm ... | iex` runs the script with your privileges before you see
# it. Prefer downloading the script, reviewing it, then running it.

param(
    [string]$Version = "latest",
    [string]$Prefix = "",
    [switch]$Service,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$Repo = "transmit-bug/serial_cli"
$TaskName = "SerialCLIDaemon"

# --- Uninstall --------------------------------------------------------------
if ($Uninstall) {
    $existing = Get-Command serial-cli -ErrorAction SilentlyContinue
    if ($existing) {
        & schtasks /Delete /TN $TaskName /F 2>$null | Out-Null
        Remove-Item $existing.Source -Force -ErrorAction SilentlyContinue
        Write-Host "Removed $($existing.Source)"
    } else {
        Write-Host "serial-cli is not installed."
    }
    exit 0
}

# --- Platform detection -----------------------------------------------------
$Arch = $env:PROCESSOR_ARCHITECTURE
if ($Arch -eq "AMD64") {
    $AssetName = "serial-cli-windows-x86_64.exe"
} elseif ($Arch -eq "ARM64") {
    $AssetName = "serial-cli-windows-x86_64.exe"  # prebuilt x64 runs under emulation
    Write-Host "Note: installing the x86_64 build (ARM64 emulation)." -ForegroundColor Yellow
} else {
    throw "Unsupported architecture: $Arch"
}

# --- Resolve release + asset URL -------------------------------------------
if ($Version -eq "latest") {
    $ApiUrl = "$env:SERIAL_CLI_RELEASE_URL"
    if (-not $ApiUrl) { $ApiUrl = "https://api.github.com/repos/$Repo/releases/latest" }
} else {
    $ApiUrl = "$env:SERIAL_CLI_RELEASE_URL"
    if (-not $ApiUrl) { $ApiUrl = "https://api.github.com/repos/$Repo/releases/tags/$Version" }
}
Write-Host "Fetching release info from $ApiUrl"
$Release = Invoke-RestMethod -Uri $ApiUrl -Headers @{ "User-Agent" = "serial-cli-installer" }
$Asset = $Release.assets | Where-Object { $_.name -eq $AssetName } | Select-Object -First 1
if (-not $Asset) {
    throw "Could not find asset '$AssetName' in $Version release."
}

# --- Download + verify ------------------------------------------------------
$TmpDir = Join-Path $env:TEMP "serial-cli-install-$([guid]::NewGuid())"
New-Item -ItemType Directory -Path $TmpDir | Out-Null
try {
    $BinPath = Join-Path $TmpDir $AssetName
    Write-Host "Downloading $($Asset.browser_download_url)"
    Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $BinPath

    $ShaPath = Join-Path $TmpDir "$AssetName.sha256"
    Invoke-WebRequest -Uri "$($Asset.browser_download_url).sha256" -OutFile $ShaPath
    $Expected = (Get-Content $ShaPath -Raw).Split(" ")[0].Trim()
    $Actual = (Get-FileHash -Path $BinPath -Algorithm SHA256).Hash.ToLower()
    if ($Expected -ne $Actual) {
        throw "Checksum verification FAILED.`n  expected: $Expected`n  actual:   $Actual"
    }
    Write-Host "Checksum verified."

    # --- Install ------------------------------------------------------------
    if (-not $Prefix) {
        $Prefix = Join-Path $env:LOCALAPPDATA "serial-cli\bin"
    }
    New-Item -ItemType Directory -Path $Prefix -Force | Out-Null
    $Dest = Join-Path $Prefix "serial-cli.exe"
    Copy-Item $BinPath $Dest -Force
    Write-Host "Installed to $Dest"

    # Add to user PATH if not already present
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$Prefix*") {
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$Prefix", "User")
        Write-Host "Added $Prefix to your user PATH (new terminals will see it)."
    }

    # --- Optional auto-start ------------------------------------------------
    if ($Service) {
        Write-Host "Registering daemon auto-start..."
        & $Dest server service install
        if ($LASTEXITCODE -ne 0) { throw "Failed to register auto-start." }
    }

    Write-Host "`nDone. Run 'serial-cli --help' to get started."
}
finally {
    Remove-Item $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
}
