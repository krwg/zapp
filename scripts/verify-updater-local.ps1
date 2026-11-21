# Local QA for tauri-plugin-updater artifacts (no GitHub release).
# Signs an existing NSIS setup with keys/zapp.key and writes latest.json.
param(
  [Parameter(Mandatory = $true)][string]$SetupExe,
  [string]$Version = "",
  [string]$KeyPath = "",
  [string]$Password = "zapp-local-dev",
  [string]$OutDir = "out-updater-qa"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
if (-not $KeyPath) { $KeyPath = Join-Path $root "keys\zapp.key" }
if (-not (Test-Path $KeyPath)) { throw "Missing private key: $KeyPath" }
$setup = Get-Item $SetupExe
if (-not $Version) {
  $Version = (Get-Content (Join-Path $root "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json).version
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$dest = Join-Path $OutDir $setup.Name
Copy-Item $setup.FullName $dest -Force

Write-Host "Signing $dest (version $Version)..."
# Prefer path flag; clear string env so CLI does not see both.
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PATH -ErrorAction SilentlyContinue
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $Password
Push-Location $root
try {
  npx tauri signer sign --private-key-path $KeyPath --password $Password --app-version $Version $dest
} finally {
  Pop-Location
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
}

if (-not (Test-Path "$dest.sig")) { throw "Signature not created: $dest.sig" }

$latest = Join-Path $OutDir "latest.json"
& (Join-Path $root "scripts\build-latest-json.ps1") `
  -SetupExe $dest `
  -Version $Version `
  -Tag "v$Version" `
  -Notes "Local updater QA — not a published release." `
  -Out $latest

$json = Get-Content $latest -Raw | ConvertFrom-Json
if (-not $json.platforms.'windows-x86_64'.signature) { throw "latest.json missing signature" }
if (-not $json.platforms.'windows-x86_64'.url) { throw "latest.json missing url" }

Write-Host ""
Write-Host "OK — updater QA artifacts in $OutDir"
Write-Host "  $($setup.Name)"
Write-Host "  $($setup.Name).sig"
Write-Host "  latest.json (version=$($json.version))"
Write-Host ""
Write-Host "Point plugins.updater.endpoints at a local/static host of latest.json to exercise the plugin, or keep GitHub fallback."
