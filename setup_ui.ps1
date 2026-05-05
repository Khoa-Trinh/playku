param (
    [string]$DownloadUrl = "https://github.com/tomasklaen/uosc/releases/latest/download/uosc.zip"
)

$ConfigDir = "mpv_config"
$ArchiveFile = "uosc_temp.zip"

Write-Host "--> Downloading uosc from GitHub..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $DownloadUrl -OutFile $ArchiveFile

Write-Host "--> Creating $ConfigDir directory..." -ForegroundColor Cyan
if (!(Test-Path -Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir | Out-Null
}

Write-Host "--> Extracting archive..." -ForegroundColor Cyan
Expand-Archive -Path $ArchiveFile -DestinationPath $ConfigDir -Force

Write-Host "--> Cleaning up temporary files..." -ForegroundColor Cyan
Remove-Item -Path $ArchiveFile -Force

Write-Host "--> DONE! uosc is ready in the '$ConfigDir' folder." -ForegroundColor Green
