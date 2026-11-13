# Genera artifacts/guanacaste_render-<versión>.rbz (un zip) con guanacaste_render.rb y guanacaste_render/.
# Uso (desde cualquier carpeta):  powershell -File plugins\sketchup\build_rbz.ps1
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$registration = Join-Path $root 'guanacaste_render.rb'
$folder = Join-Path $root 'guanacaste_render'
$match = Select-String -Path $registration -Pattern "PLUGIN_VERSION = '([^']+)'" | Select-Object -First 1
if (-not $match) { throw 'No se encontró PLUGIN_VERSION en guanacaste_render.rb' }
$version = $match.Matches[0].Groups[1].Value

$artifacts = Join-Path $root 'artifacts'
New-Item -ItemType Directory -Force -Path $artifacts | Out-Null
$target = Join-Path $artifacts "guanacaste_render-$version.rbz"
if (Test-Path $target) { Remove-Item $target -Force }

$zip = [System.IO.Compression.ZipFile]::Open($target, 'Create')
try {
    # Los nombres de entrada usan "/" como exige el formato zip.
    [void][System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $registration, 'guanacaste_render.rb')
    Get-ChildItem -Path $folder -Recurse -File | ForEach-Object {
        $relative = $_.FullName.Substring($root.Length + 1).Replace('\', '/')
        [void][System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $_.FullName, $relative)
    }
}
finally { $zip.Dispose() }
Write-Host "Creado: $target"
