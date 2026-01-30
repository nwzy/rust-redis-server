# PowerShell script to test Redis server connections
# Usage: .\test_server.ps1 [<number_of_connections>]
# Default to 1 connection if no argument is provided

param(
    [int]$NumConns = 1
)

# Store background processes
$processes = @()

for ($i = 1; $i -le $NumConns; $i++) {
    # Create TCP connection using .NET classes
    $tcpClient = New-Object System.Net.Sockets.TcpClient
    $tcpClient.Connect("localhost", 6379)
    
    Write-Host "Started connection $i"
    
    # Keep connection open by running in background
    $process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Start-Sleep -Seconds 10" -PassThru
    
    # Store the process to wait later
    $processes += $process
}

Write-Host "Successfully initiated $NumConns connections."

# Wait for all background processes to complete
foreach ($process in $processes) {
    $process.WaitForExit()
}
