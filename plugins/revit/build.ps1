<#
.SYNOPSIS
  Compila el add-in para las versiones de Revit instaladas y arma el paquete de entrega en artifacts\.
.DESCRIPTION
  Cada marco se compila solo si existe la carpeta de Revit cuya API necesita (2024 -> net48, 2025 -> net8.0-windows,
  2027 -> net10.0-windows); el resto se omite con un aviso. Salidas: bin\ y obj\ (locales) y
  artifacts\GuanacasteRender.Revit-<version>.zip con las DLL, el manifiesto, install.ps1 y el README.
#>
[CmdletBinding()]
param([string]$Configuration = 'Release')

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$targets = @(
    @{ Framework = 'net48'; Revit = 'Revit 2024' },
    @{ Framework = 'net8.0-windows'; Revit = 'Revit 2025' },
    @{ Framework = 'net10.0-windows'; Revit = 'Revit 2027' }
)
$built = @()
Push-Location $root
try {
    foreach ($t in $targets) {
        $api = Join-Path ${env:ProgramFiles} "Autodesk\$($t.Revit)\RevitAPI.dll"
        if (-not (Test-Path $api)) { Write-Warning "Se omite $($t.Framework): no hay $api"; continue }
        dotnet build (Join-Path $root 'GuanacasteRender.Revit.csproj') -c $Configuration -f $t.Framework --nologo
        if ($LASTEXITCODE -ne 0) { throw "Fallo la compilacion de $($t.Framework)" }
        $built += $t.Framework
    }
    if ($built.Count -eq 0) { throw 'No se compilo ningun marco (no hay Revit instalado).' }
    $stage = Join-Path $root "obj\package"
    if (Test-Path $stage) { Remove-Item -LiteralPath $stage -Recurse }
    New-Item -ItemType Directory -Force -Path $stage | Out-Null
    foreach ($f in $built) {
        $dest = Join-Path $stage "bin\$Configuration\$f"
        New-Item -ItemType Directory -Force -Path $dest | Out-Null
        Copy-Item (Join-Path $root "bin\$Configuration\$f\GuanacasteRender.Revit.*") $dest -Include *.dll, *.pdb
    }
    Copy-Item (Join-Path $root 'GuanacasteRender.addin'), (Join-Path $root 'install.ps1'), (Join-Path $root 'README.md') $stage
    $artifacts = Join-Path $root 'artifacts'
    New-Item -ItemType Directory -Force -Path $artifacts | Out-Null
    $zip = Join-Path $artifacts 'GuanacasteRender.Revit-1.0.0.zip'
    if (Test-Path $zip) { Remove-Item -LiteralPath $zip }
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
    Write-Host "Paquete: $zip (marcos: $($built -join ', '))"
}
finally { Pop-Location }
