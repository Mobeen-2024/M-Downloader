# Mock Bridge Client for Mdownloader
# This script connects to the named pipe to trigger the "CONNECTED" status in the UI.

$pipeName = "\\.\pipe\mdownloader_bridge"
Write-Host "Attempting to connect to $pipeName..."

try {
    $pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", "mdownloader_bridge", [System.IO.Pipes.PipeDirection]::InOut)
    $pipe.Connect(5000)
    Write-Host "Successfully connected to Bridge!"
    
    # Stay connected for 5 seconds to observe the UI status
    Start-Sleep -Seconds 5
    
    $pipe.Close()
    Write-Host "Disconnected from Bridge."
} catch {
    Write-Host "Failed to connect to Bridge: $($_.Exception.Message)"
}
