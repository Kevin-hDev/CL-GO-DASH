$ErrorActionPreference = "Stop"

$Repo = "Kevin-hDev/CL-GO-DASH"
$AppName = "CL-GO"
$ApiUrl = "https://api.github.com/repos/$Repo/releases/latest"

function Info($msg) { Write-Host "→ $msg" -ForegroundColor Blue }
function Ok($msg) { Write-Host "✓ $msg" -ForegroundColor Green }
function Fail($msg) { Write-Host "✗ $msg" -ForegroundColor Red; exit 1 }

Info "Détection : Windows $env:PROCESSOR_ARCHITECTURE"

Info "Récupération de la dernière version..."
try {
    $release = Invoke-RestMethod -Uri $ApiUrl -Headers @{ "User-Agent" = "CL-GO-Installer" }
} catch {
    Fail "Impossible de contacter GitHub."
}

$version = $release.tag_name -replace "^v", ""
if (-not $version) { Fail "Aucune release trouvée." }

$asset = $release.assets | Where-Object { $_.name -like "*.msi" } | Select-Object -First 1
if (-not $asset) {
    $asset = $release.assets | Where-Object { $_.name -like "*.exe" } | Select-Object -First 1
}
if (-not $asset) { Fail "Pas d'installeur Windows dans la release v$version." }

$url = $asset.browser_download_url
$tmpDir = Join-Path $env:TEMP "cl-go-update"
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
$tmpFile = Join-Path $tmpDir $asset.name

Info "Téléchargement de $AppName v$version..."
Invoke-WebRequest -Uri $url -OutFile $tmpFile -UseBasicParsing

$defaultDir = Join-Path $env:LOCALAPPDATA $AppName
Write-Host ""
Write-Host "📁 Répertoire d'installation : $defaultDir" -ForegroundColor Yellow
$customDir = Read-Host "   Appuie sur Entrée pour accepter, ou tape un autre chemin"
if ($customDir) {
    $installDir = $customDir
} else {
    $installDir = $defaultDir
}

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

Info "Installation dans $installDir..."
if ($tmpFile -like "*.msi") {
    Start-Process msiexec.exe -ArgumentList "/i", "`"$tmpFile`"", "/passive", "INSTALLDIR=`"$installDir`"" -Wait
} else {
    Start-Process $tmpFile -ArgumentList "/S", "/D=$installDir" -Wait
}

Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue

Ok "$AppName v$version installé."
