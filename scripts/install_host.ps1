# Mdownloader — Native Messaging Host Installer (Windows)
# This script registers the helper.exe with Google Chrome and Microsoft Edge.

$HostManifestPath = "d:\WorkStation\IDM\src-tauri\com.mdownloader.app.json"

# ── Step 1: Register with Google Chrome ─────────────────────────────
$ChromeRegistryPath = "HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.mdownloader.app"
if (!(Test-Path $ChromeRegistryPath)) {
    New-Item -Path $ChromeRegistryPath -Force | Out-Null
}
Set-ItemProperty -Path $ChromeRegistryPath -Name "(Default)" -Value $HostManifestPath
Write-Host "Successfully registered Mdownloader Host with Google Chrome." -ForegroundColor Cyan

# ── Step 2: Register with Microsoft Edge ─────────────────────────────
$EdgeRegistryPath = "HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\com.mdownloader.app"
if (!(Test-Path $EdgeRegistryPath)) {
    New-Item -Path $EdgeRegistryPath -Force | Out-Null
}
Set-ItemProperty -Path $EdgeRegistryPath -Name "(Default)" -Value $HostManifestPath
Write-Host "Successfully registered Mdownloader Host with Microsoft Edge." -ForegroundColor Green

Write-Host "`nInstallation Complete. Restart your browser to apply changes." -ForegroundColor Yellow
