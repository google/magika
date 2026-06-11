param(
    [Parameter(Mandatory = $true)]
    [string]$Source,

    [Parameter(Mandatory = $true)]
    [string]$Destination
)

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$archiveName = "backup-$timestamp.zip"
$archivePath = Join-Path -Path $Destination -ChildPath $archiveName

New-Item -ItemType Directory -Force -Path $Destination | Out-Null
Compress-Archive -Path $Source -DestinationPath $archivePath -Force
Write-Host "Created $archivePath"
