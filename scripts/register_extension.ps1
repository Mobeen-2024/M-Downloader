# Mdownloader Browser Integration Registry Setup
# Run this to link Mdownloader to Chrome/Edge for link capturing.

$ManifestName = "com.mobeen.mdownloader"
$ProjectRoot = Get-Location
$NativeHostExe = "$ProjectRoot\src-tauri\target\debug\native_host.exe" # Change to release for production
$ManifestFile = "$ProjectRoot\com.mobeen.mdownloader.json"

if (-Not (Test-Path $NativeHostExe)) {
    Write-Host "Error: native_host.exe not found at $NativeHostExe" -ForegroundColor Red
    Write-Host "Please build it first: cargo build --bin native_host"
    exit 1
}

# 1. Update manifest with full path (JSON escape backslashes)
$EscapedPath = $NativeHostExe.Replace("\", "\\")
$Content = Get-Content $ManifestFile | ConvertFrom-Json
$Content.path = $NativeHostExe
$Content | ConvertTo-Json | Set-Content $ManifestFile

Write-Host "[1/3] Manifest updated with path: $NativeHostExe"

# 2. Register for Chrome
$ChromeRegistryPath = "HKCU:\Software\Google\Chrome\NativeMessagingHosts\$ManifestName"
if (-Not (Test-Path $ChromeRegistryPath)) { New-Item -Path $ChromeRegistryPath -Force }
Set-ItemProperty -Path $ChromeRegistryPath -Name "(Default)" -Value $ManifestFile
Write-Host "[2/3] Registered for Google Chrome."

# 3. Register for Microsoft Edge
$EdgeRegistryPath = "HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\$ManifestName"
if (-Not (Test-Path $EdgeRegistryPath)) { New-Item -Path $EdgeRegistryPath -Force }
Set-ItemProperty -Path $EdgeRegistryPath -Name "(Default)" -Value $ManifestFile
Write-Host "[3/3] Registered for Microsoft Edge."

Write-Host "`nSUCCESS: Mdownloader is now linked to your browser." -ForegroundColor Green
Write-Host "Restart your browser to apply changes."
