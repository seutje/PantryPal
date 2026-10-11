# Import the installed MSVC compiler/SDK environment into this PowerShell process.
# SQLite's bundled C source requires INCLUDE/LIB as well as cl.exe on PATH.
$ErrorActionPreference = 'Stop'
$pantryVsWhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path -LiteralPath $pantryVsWhere)) { throw 'Install Visual Studio Build Tools with Desktop development with C++.' }
$pantryVsInstall = & $pantryVsWhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $pantryVsInstall) { throw 'Visual Studio C++ tools were not found.' }
& (Join-Path $pantryVsInstall 'Common7\Tools\Launch-VsDevShell.ps1') -Arch amd64 -HostArch amd64 -SkipAutomaticLocation
