<#
.SYNOPSIS
  Instala (o desinstala) el add-in "Exportar a Guanacaste Render" para una version de Revit.
.DESCRIPTION
  Copia la DLL compilada a %APPDATA%\Autodesk\Revit\Addins\<anio>\GuanacasteRender\ y el manifiesto
  GuanacasteRender.addin a %APPDATA%\Autodesk\Revit\Addins\<anio>\. Solo se ejecuta cuando usted lo lanza.
  Nunca toca archivos que no sean de este add-in: si ya existe un GuanacasteRender.addin con otro AddInId, se
  detiene. Cierre Revit antes de instalar.
.EXAMPLE
  .\install.ps1 -Year 2026
  .\install.ps1 -Year 2026 -Uninstall
  .\install.ps1 -Year 2025 -SourceDir .\bin\Release\net8.0-windows -WhatIf
#>
[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [Parameter(Mandatory = $true)][ValidateRange(2021, 2030)][int]$Year,
    [string]$SourceDir,
    [switch]$Uninstall
)

$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
$addinId = '7b0f6d52-3c8a-4e0b-9a41-5d1e8f2c6a93'
$addinsRoot = Join-Path $env:APPDATA "Autodesk\Revit\Addins\$Year"
$manifest = Join-Path $addinsRoot 'GuanacasteRender.addin'
$folder = Join-Path $addinsRoot 'GuanacasteRender'

function Test-OurManifest([string]$path) {
    (Get-Content -LiteralPath $path -Raw) -match [regex]::Escape($addinId)
}

if ($Uninstall) {
    if ((Test-Path $manifest) -and -not (Test-OurManifest $manifest)) {
        throw "El archivo $manifest no es de Guanacaste Render; no se borra."
    }
    if ($PSCmdlet.ShouldProcess($addinsRoot, 'Desinstalar Guanacaste Render')) {
        if (Test-Path $manifest) { Remove-Item -LiteralPath $manifest }
        if (Test-Path $folder) { Remove-Item -LiteralPath $folder -Recurse }
        Write-Host "Desinstalado de Revit $Year."
    }
    return
}

# Marco de .NET segun la version de Revit: 2021-2024 .NET Framework 4.8, 2025-2026 .NET 8, 2027 en adelante .NET 10.
$framework = if ($Year -le 2024) { 'net48' } elseif ($Year -le 2026) { 'net8.0-windows' } else { 'net10.0-windows' }
if (-not $SourceDir) {
    $SourceDir = Join-Path $here "bin\Release\$framework"
    if (-not (Test-Path $SourceDir)) { $SourceDir = Join-Path $here "bin\Debug\$framework" }
}
$dll = Join-Path $SourceDir 'GuanacasteRender.Revit.dll'
if (-not (Test-Path $dll)) {
    throw "No se encontro $dll. Compile primero: .\build.ps1 (o indique -SourceDir)."
}
if ((Test-Path $manifest) -and -not (Test-OurManifest $manifest)) {
    throw "Ya existe $manifest y no es de Guanacaste Render (otro AddInId). No se sobrescribe."
}
if (Get-Process -Name Revit -ErrorAction SilentlyContinue) {
    Write-Warning 'Revit esta abierto: cierrelo y vuelva a abrirlo para que cargue el add-in (la DLL en uso no se podra reemplazar).'
}

if ($PSCmdlet.ShouldProcess($addinsRoot, "Instalar Guanacaste Render ($framework)")) {
    New-Item -ItemType Directory -Force -Path $folder | Out-Null
    Copy-Item -LiteralPath $dll -Destination $folder -Force
    $pdb = Join-Path $SourceDir 'GuanacasteRender.Revit.pdb'
    if (Test-Path $pdb) { Copy-Item -LiteralPath $pdb -Destination $folder -Force }
    Copy-Item -LiteralPath (Join-Path $here 'GuanacasteRender.addin') -Destination $manifest -Force
    Write-Host "Instalado en $addinsRoot. Abra Revit ${Year}: aparecera la pestana Guanacaste."
    Write-Host 'Revit pedira confirmar la carga del add-in la primera vez (elija "Cargar siempre").'
}
