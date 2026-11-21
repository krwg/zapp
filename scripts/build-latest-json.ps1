# Build latest.json for tauri-plugin-updater from a signed NSIS artifact.
param(
  [Parameter(Mandatory = $true)][string]$SetupExe,
  [Parameter(Mandatory = $true)][string]$Version,
  [string]$Repo = "krwg/zapp",
  [string]$Tag = "",
  [string]$Notes = "See RELEASE notes on GitHub.",
  [string]$Out = "latest.json"
)

$ErrorActionPreference = "Stop"
if (-not $Tag) { $Tag = "v$Version" }
$setup = Get-Item $SetupExe
$sigPath = "$($setup.FullName).sig"
if (-not (Test-Path $sigPath)) {
  throw "Missing signature file: $sigPath (run tauri signer / createUpdaterArtifacts build)"
}
$sig = (Get-Content $sigPath -Raw).Trim()
$url = "https://github.com/$Repo/releases/download/$Tag/$($setup.Name)"
$obj = [ordered]@{
  version  = $Version
  notes    = $Notes
  pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
  platforms = [ordered]@{
    "windows-x86_64" = [ordered]@{
      signature = $sig
      url       = $url
    }
  }
}
$json = ($obj | ConvertTo-Json -Depth 6) + "`n"
$fullOut = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Out)
$dir = Split-Path -Parent $fullOut
if ($dir -and -not (Test-Path $dir)) {
  New-Item -ItemType Directory -Force -Path $dir | Out-Null
}
# UTF-8 without BOM (PS 5.1 compatible)
[IO.File]::WriteAllText($fullOut, $json)
Write-Host "Wrote $fullOut -> $url"
