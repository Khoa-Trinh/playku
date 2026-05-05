param (
    [string]$DownloadUrl = "https://sourceforge.net/projects/mpv-player-windows/files/libmpv/mpv-dev-x86_64-v3-20260419-git-06f4ce7.7z/download"
)

$LibDir = "mpv_lib"
$ArchiveFile = "mpv_temp.7z"
$ExtractDir = "mpv_extracted"
$SevenZipPath = "C:\Program Files\7-Zip\7z.exe"

Write-Host "--> Checking for 7-Zip..." -ForegroundColor Cyan
if (!(Test-Path $SevenZipPath)) {
    Write-Host "ERROR: 7-Zip not found at $SevenZipPath. Please install it or update the path in this script." -ForegroundColor Red
    exit
}

Write-Host "--> Creating $LibDir directory..." -ForegroundColor Cyan
if (!(Test-Path -Path $LibDir)) {
    New-Item -ItemType Directory -Path $LibDir | Out-Null
}

Write-Host "--> Downloading mpv from SourceForge..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $DownloadUrl -OutFile $ArchiveFile -UserAgent "Wget/1.9"

Write-Host "--> Extracting archive using 7-Zip..." -ForegroundColor Cyan
& $SevenZipPath x $ArchiveFile "-o$ExtractDir" -y | Out-Null

Write-Host "--> Locating files and fixing names..." -ForegroundColor Cyan

# 1. Copy any .dll files (like libmpv-2.dll)
Get-ChildItem -Path $ExtractDir -Recurse -Filter "*.dll" | Copy-Item -Destination $LibDir -Force

# 2. Find libmpv.dll.a, copy it, and rename it to mpv.lib automatically
$dllA_file = Get-ChildItem -Path $ExtractDir -Recurse -Filter "libmpv.dll.a" | Select-Object -First 1

if ($dllA_file) {
    Write-Host "    Found libmpv.dll.a! Renaming to mpv.lib..." -ForegroundColor Yellow
    $destinationPath = Join-Path $LibDir "mpv.lib"
    Copy-Item -Path $dllA_file.FullName -Destination $destinationPath -Force
} else {
    # Fallback just in case standard .lib files exist in future builds
    Get-ChildItem -Path $ExtractDir -Recurse -Filter "*.lib" | Copy-Item -Destination $LibDir -Force
}

Write-Host "--> Cleaning up temporary files..." -ForegroundColor Cyan
Remove-Item -Path $ArchiveFile -Force
Remove-Item -Path $ExtractDir -Recurse -Force

Write-Host "--> DONE! Your mpv.lib and .dll are ready in the '$LibDir' folder." -ForegroundColor Green