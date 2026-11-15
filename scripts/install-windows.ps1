<#
.SYNOPSIS
Builds and installs Guanacaste Render for the current Windows user.

.DESCRIPTION
Builds the release executable inside the repository (target\release), copies it to
%LOCALAPPDATA%\Programs\Guanacaste Render\<version>\GuanacasteRender.exe, records that path in
%LOCALAPPDATA%\GuanacasteRender\application-path.txt and creates a Start Menu shortcut. That folder layout and
the recorded path are what Guanacaste 3D uses to find the render application. Only a running copy of this same
installed version is stopped.
#>
[CmdletBinding()]
param(
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$startingLocation = Get-Location
$projectRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)

try {
    Set-Location -LiteralPath $projectRoot
    $version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"' |
        Select-Object -First 1).Matches[0].Groups[1].Value
    if (-not $SkipBuild) {
        # Cargo reports progress on stderr, which Windows PowerShell would turn into a terminating error.
        $ErrorActionPreference = 'Continue'
        cargo build --release --bin GuanacasteRender 2>&1 | ForEach-Object { "$_" }
        $ErrorActionPreference = 'Stop'
        if ($LASTEXITCODE -ne 0) { throw "La compilación falló con código $LASTEXITCODE." }
    }
    $built = Join-Path $projectRoot 'target\release\GuanacasteRender.exe'
    if (-not (Test-Path -LiteralPath $built)) { throw "No existe $built; compila primero." }

    $installRoot = Join-Path $env:LOCALAPPDATA "Programs\Guanacaste Render\$version"
    $installed = Join-Path $installRoot 'GuanacasteRender.exe'
    Get-Process GuanacasteRender -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -eq $installed } |
        ForEach-Object { $_.CloseMainWindow() | Out-Null; if (-not $_.WaitForExit(5000)) { $_.Kill() } }

    New-Item -ItemType Directory -Force -Path $installRoot | Out-Null
    Copy-Item -LiteralPath $built -Destination $installed -Force

    $dataDir = Join-Path $env:LOCALAPPDATA 'GuanacasteRender'
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null
    Set-Content -LiteralPath (Join-Path $dataDir 'application-path.txt') -Value $installed -Encoding utf8 -NoNewline

    $programs = [Environment]::GetFolderPath('Programs')
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut((Join-Path $programs 'Guanacaste Render.lnk'))
    $shortcut.TargetPath = $installed
    $shortcut.WorkingDirectory = $installRoot
    $shortcut.Description = 'Guanacaste Render: visualización arquitectónica'
    $shortcut.Save()

    Write-Host "Guanacaste Render $version instalado en $installed"
}
finally {
    Set-Location -LiteralPath $startingLocation
}
