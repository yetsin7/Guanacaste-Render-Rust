# Copia SOLO la extensión a la carpeta Plugins de SketchUp. No se ejecuta solo: lo lanza el usuario.
# Uso:  powershell -File plugins\sketchup\install.ps1 -Version 2024
#       (sin -Version se listan las versiones de SketchUp encontradas y no se copia nada)
param([string]$Version)
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$base = Join-Path $env:APPDATA 'SketchUp'

if (-not $Version) {
    Write-Host 'Indique la versión de SketchUp con -Version. Versiones encontradas:'
    if (Test-Path $base) { Get-ChildItem $base -Directory | ForEach-Object { Write-Host "  $($_.Name)" } }
    else { Write-Host '  (ninguna: no existe ' $base ')' }
    exit 1
}

$plugins = Join-Path $base "SketchUp $Version\SketchUp\Plugins"
if (-not (Test-Path $plugins)) { throw "No existe la carpeta Plugins: $plugins" }

Copy-Item (Join-Path $root 'guanacaste_render.rb') $plugins -Force
$destination = Join-Path $plugins 'guanacaste_render'
if (Test-Path $destination) { Remove-Item $destination -Recurse -Force }
Copy-Item (Join-Path $root 'guanacaste_render') $destination -Recurse
Write-Host "Instalado en $plugins. Reinicie SketchUp y busque Extensiones > Guanacaste Render."
