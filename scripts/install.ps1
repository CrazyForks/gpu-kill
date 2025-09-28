$ErrorActionPreference = "Stop"

# gpukill Windows installer: prefer winget, fallback to zip from GitHub Releases

param(
  [string]$Version = "",
  [string]$BinDir = "$env:LOCALAPPDATA\Programs\gpukill",
  [switch]$Yes,
  [switch]$Insecure
)

function Get-Arch {
  if ([System.Environment]::Is64BitOperatingSystem) { return "x86_64" } else { return "x86" }
}

# Try winget first
try {
  if (Get-Command winget -ErrorAction SilentlyContinue) {
    winget install --id KageHQ.GPUKill --silent --accept-package-agreements --accept-source-agreements
    if ($LASTEXITCODE -eq 0) { Write-Host "✅ Installed via winget"; exit 0 }
  }
} catch {}

# Fallback to GitHub Releases
$Owner = "kagehq"
$Repo = "gpu-kill"
if ($Version -ne "") {
  $ApiUrl = "https://api.github.com/repos/$Owner/$Repo/releases/tags/$Version"
} else {
  $ApiUrl = "https://api.github.com/repos/$Owner/$Repo/releases/latest"
}

Write-Host "Resolving release…"
$resp = Invoke-RestMethod -Uri $ApiUrl -UseBasicParsing
$Tag = $resp.tag_name
if (-not $Tag) { throw "Failed to resolve release tag" }

$arch = Get-Arch
$assetName = "gpukill-$Tag-windows-$arch.zip"
$asset = $resp.assets | Where-Object { $_.name -eq $assetName }
if (-not $asset) { throw "No asset named $assetName in release $Tag" }

$tmp = New-Item -ItemType Directory -Path ([System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid())
$zipPath = Join-Path $tmp $assetName
$sumsAsset = $resp.assets | Where-Object { $_.name -eq 'SHA256SUMS' }
$sumsPath = Join-Path $tmp 'SHA256SUMS'

Write-Host "Downloading $assetName…"
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath -UseBasicParsing
if ($sumsAsset) {
  Invoke-WebRequest -Uri $sumsAsset.browser_download_url -OutFile $sumsPath -UseBasicParsing
}

if (Test-Path $sumsPath) {
  $hash = (Get-FileHash -Algorithm SHA256 $zipPath).Hash.ToLower()
  $sums = Get-Content $sumsPath
  if (-not ($sums -match $hash)) {
    if (-not $Insecure) { throw "Checksum verification failed" }
    Write-Warning "Checksum verification skipped (--Insecure)"
  }
}

Write-Host "Extracting…"
Expand-Archive -Path $zipPath -DestinationPath $tmp -Force

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Copy-Item -Path (Join-Path $tmp 'gpukill.exe') -Destination (Join-Path $BinDir 'gpukill.exe') -Force

# Add to PATH for current session
$env:PATH = "$BinDir;$env:PATH"
Write-Host "✅ Installed to $BinDir"
& (Join-Path $BinDir 'gpukill.exe') --version

