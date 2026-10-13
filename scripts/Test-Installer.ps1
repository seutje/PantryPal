# Explicit isolated test identity: never install/uninstall the household product.
$ErrorActionPreference = 'Stop'
$testRoot = [IO.Path]::GetFullPath((Join-Path (Split-Path $PSScriptRoot -Parent) '.verification/phase8-installers'))
$installRoot = Join-Path $testRoot 'installed'
$testData = Join-Path $env:APPDATA 'ca.pantrypal.acceptance'
$testRegistry = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\PantryPalAcceptance'
if (Test-Path -LiteralPath $testRegistry) { throw 'An acceptance installation already exists; review it before running this test.' }
if (Test-Path -LiteralPath $testData) { throw 'Acceptance data already exists; preserve/review it before running this test.' }
$helper = Join-Path (Split-Path $PSScriptRoot -Parent) 'target/debug/examples/updater_install_spike.exe'
function Invoke-FixtureHelper($mode) {
    $log = Join-Path $testRoot "$mode.log"
    $process = Start-Process -FilePath $helper -ArgumentList $mode -WindowStyle Hidden -PassThru -Wait -RedirectStandardOutput $log -RedirectStandardError (Join-Path $testRoot "$mode-error.log")
    Get-Content -LiteralPath $log
    if ($process.ExitCode -ne 0) { throw "Acceptance helper failed: $mode (see ignored logs)." }
}
function Start-FixtureApp {
    $process = Start-Process -FilePath (Join-Path $installRoot 'pantrypal.exe') -WindowStyle Hidden -PassThru
    Start-Sleep -Seconds 5
    if ($process.HasExited) { throw 'Installed application did not stay running.' }
    # Only the process launched by this test; completed SQLite writes are durable.
    Stop-Process -Id $process.Id
    $process.WaitForExit()
}
New-Item -ItemType Directory -Force $installRoot | Out-Null
$installer = Join-Path $testRoot 'PantryPalAcceptance_0.1.0_x64-setup.exe'
$process = Start-Process -FilePath $installer -ArgumentList "/S /D=$installRoot" -WindowStyle Hidden -PassThru -Wait
if ($process.ExitCode -ne 0) { throw 'Initial fixture installation failed.' }
Start-FixtureApp
Invoke-FixtureHelper 'seed'
Start-FixtureApp
Invoke-FixtureHelper 'install'
$deadline = (Get-Date).AddSeconds(60)
do {
    Start-Sleep -Milliseconds 500
    $version = (Get-ItemProperty -LiteralPath $testRegistry -ErrorAction SilentlyContinue).DisplayVersion
} until ($version -eq '0.1.1' -or (Get-Date) -gt $deadline)
if ($version -ne '0.1.1') { throw 'Signed SDK installer handoff did not update the isolated installation.' }
if ((Get-Item -LiteralPath (Join-Path $installRoot 'pantrypal.exe')).VersionInfo.ProductVersion -ne '0.1.1') {
    throw 'The newer installer did not replace the executable in the original installation directory.'
}
$registeredLocation = (Get-ItemProperty -LiteralPath $testRegistry).InstallLocation.Trim('"')
if ([IO.Path]::GetFullPath($registeredLocation) -ne $installRoot) { throw 'The upgrade changed the fixture installation directory.' }
Start-FixtureApp
Invoke-FixtureHelper 'verify'
$uninstaller = Join-Path $installRoot 'uninstall.exe'
$process = Start-Process -FilePath $uninstaller -ArgumentList "/S _?=$installRoot" -WindowStyle Hidden -PassThru -Wait
if ($process.ExitCode -ne 0) { throw 'Fixture uninstall failed.' }
if (-not (Test-Path -LiteralPath (Join-Path $testData 'pantrypal.sqlite3'))) { throw 'Uninstall removed fixture household data.' }
Invoke-FixtureHelper 'verify'
'PASS: isolated NSIS install, startup/restart, signed old-to-new SDK upgrade and uninstall retained data' | Tee-Object -FilePath (Join-Path $testRoot 'acceptance.txt')
# Retain evidence/data for review. No recursive deletion of profiles or backups.
