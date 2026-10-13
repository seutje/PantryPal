# The private signing key is never printed or added to version control.
$ErrorActionPreference = 'Stop'
$releaseRoot = Split-Path $PSScriptRoot -Parent
Push-Location $releaseRoot
$previousKey = $env:TAURI_SIGNING_PRIVATE_KEY
$previousPassword = $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD
try {
    . (Join-Path $PSScriptRoot 'Enter-DevEnvironment.ps1')
    node scripts/check-version.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Release versions disagree.' }
    if (-not $env:TAURI_SIGNING_PRIVATE_KEY) {
        $releaseKey = Join-Path $releaseRoot '.release-secrets/pantrypal.key'
        if (-not (Test-Path -LiteralPath $releaseKey)) { throw 'Set TAURI_SIGNING_PRIVATE_KEY or supply the dedicated ignored PantryPal signing key.' }
        $env:TAURI_SIGNING_PRIVATE_KEY = $releaseKey
    }
    if (-not $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD) { $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = '' }
    npm run tauri build -- --bundles nsis -- --locked
    if ($LASTEXITCODE -ne 0) { throw 'Windows installer build failed.' }
} finally {
    $env:TAURI_SIGNING_PRIVATE_KEY = $previousKey
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $previousPassword
    Pop-Location
}
