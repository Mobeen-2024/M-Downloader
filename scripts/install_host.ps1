# ─────────────────────────────────────────────────────────────────────────────
# Mdownloader — Native Messaging Host Installer (Windows)
#
# Registers the native_host.exe with Chrome, Edge, and Brave by:
#   1. Locating the native_host.exe binary.
#   2. Patching the host manifest JSON with the absolute path.
#   3. Writing registry keys under HKCU for each browser.
#
# Usage:
#   .\install_host.ps1                   — auto-detect binary location
#   .\install_host.ps1 -BinaryPath "..." — specify a custom binary path
# ─────────────────────────────────────────────────────────────────────────────

param(
    [string]$BinaryPath = "",
    [switch]$Uninstall
)

$HostName = "com.mdownloader.host"

# The manifest lives next to the extension files.
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$ProjectRoot = Split-Path -Parent $ScriptDir
$ManifestSource = Join-Path $ProjectRoot "browser-extension\$HostName.json"

# ── Locate the native_host.exe ────────────────────────────────────────────
if (-not $BinaryPath) {
    # Try the release build first, then debug.
    $ReleasePath = Join-Path $ProjectRoot "src-tauri\target\release\native_host.exe"
    $DebugPath   = Join-Path $ProjectRoot "src-tauri\target\debug\native_host.exe"

    if (Test-Path $ReleasePath) {
        $BinaryPath = $ReleasePath
    } elseif (Test-Path $DebugPath) {
        $BinaryPath = $DebugPath
    } else {
        Write-Host "ERROR: Could not find native_host.exe." -ForegroundColor Red
        Write-Host "Build it first:  cargo build --bin native_host --release" -ForegroundColor Yellow
        exit 1
    }
}

$BinaryPath = (Resolve-Path $BinaryPath).Path
Write-Host "Binary: $BinaryPath" -ForegroundColor DarkGray

# ── Registry targets ─────────────────────────────────────────────────────
$Browsers = @(
    @{ Name = "Google Chrome"; Path = "HKCU:\Software\Google\Chrome\NativeMessagingHosts\$HostName" },
    @{ Name = "Microsoft Edge"; Path = "HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\$HostName" },
    @{ Name = "Brave";          Path = "HKCU:\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\$HostName" }
)

# ── Uninstall path ───────────────────────────────────────────────────────
if ($Uninstall) {
    foreach ($browser in $Browsers) {
        if (Test-Path $browser.Path) {
            Remove-Item -Path $browser.Path -Force
            Write-Host "Removed $($browser.Name) registration." -ForegroundColor Cyan
        }
    }
    Write-Host "`nUninstall complete." -ForegroundColor Green
    exit 0
}

# ── Patch the manifest with the real binary path ─────────────────────────
if (-not (Test-Path $ManifestSource)) {
    Write-Host "ERROR: Host manifest not found at $ManifestSource" -ForegroundColor Red
    exit 1
}

$ManifestJson = Get-Content $ManifestSource -Raw | ConvertFrom-Json
# Windows JSON requires escaped backslashes — but PowerShell handles this.
$ManifestJson.path = $BinaryPath
$ManifestDestDir = Join-Path $env:LOCALAPPDATA "Mdownloader"
if (-not (Test-Path $ManifestDestDir)) {
    New-Item -ItemType Directory -Path $ManifestDestDir -Force | Out-Null
}
$ManifestDest = Join-Path $ManifestDestDir "$HostName.json"
$ManifestJson | ConvertTo-Json -Depth 10 | Set-Content -Path $ManifestDest -Encoding UTF8
Write-Host "Manifest written to: $ManifestDest" -ForegroundColor DarkGray

# ── Register with each browser ───────────────────────────────────────────
foreach ($browser in $Browsers) {
    if (-not (Test-Path $browser.Path)) {
        New-Item -Path $browser.Path -Force | Out-Null
    }
    Set-ItemProperty -Path $browser.Path -Name "(Default)" -Value $ManifestDest
    Write-Host "Registered with $($browser.Name)." -ForegroundColor Cyan
}

Write-Host "`n Installation complete. Restart your browser(s) to apply." -ForegroundColor Green
Write-Host " To uninstall: .\install_host.ps1 -Uninstall" -ForegroundColor DarkGray
