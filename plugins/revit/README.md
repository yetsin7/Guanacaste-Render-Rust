# Exportar a Guanacaste Render (add-in de Revit)

Add-in de Revit que exporta la vista 3D directamente a un proyecto **`.gr`** de Guanacaste Render, sin pasar por IFC.
Guanacaste Render lo abre con "Abrir" como cualquier otro proyecto.

Formato escrito: contenedor `GRRPROJ1` (versión 1, esquema 1), el mismo que guarda la aplicación. La especificación
byte a byte está en [`docs/plugin-export-format.md`](../../docs/plugin-export-format.md), compartida con el plugin de
SketchUp.

## Qué exporta

* La geometría **visible en la vista 3D activa** (si la vista activa no es 3D, usa la vista `{3D}` o la primera vista 3D
  del proyecto). Lo que oculte, filtre o recorte con caja de sección en esa vista no se exporta.
* **Unidades**: pies de Revit a metros; Z arriba (ya es el eje de Revit). Los vértices se guardan relativos a un
  `origin_shift` (centro XY del modelo) para conservar precisión de `f32`.
* **Instancias** (familias, grupos, componentes anidados): se escriben expandidas con su transformación aplicada; las
  transformaciones con espejo invierten el orden de los triángulos para mantener las caras hacia fuera.
* **Jerarquía**: Proyecto > Nivel > Categoría > Elemento. Cada elemento lleva su categoría (`class`) y su tipo
  (`type_name`, "Familia: Tipo"). Los elementos sin nivel cuelgan de "Sin nivel". Los niveles se guardan con su elevación.
* **Materiales**: nombre, color y transparencia (alfa = 1 - transparencia). Revit no entrega texturas por este camino:
  se exporta el color. Una malla por elemento y material.
* **Vínculos de Revit** (RVT vinculados): se omiten y se avisa; expórtelos desde su propio proyecto.

El comando es de solo lectura: no abre transacciones ni modifica el modelo.

## Uso

1. Instale el add-in (abajo) y abra Revit.
2. Abra una vista 3D, ajuste nivel de detalle ("Fino" da mejor geometría), visibilidad y caja de sección.
3. Pestaña **Guanacaste** > panel **Guanacaste Render** > **Exportar a Guanacaste Render**.
4. Elija dónde guardar el `.gr`. Una barra de progreso permite **Cancelar** (no queda ningún archivo a medias: se
   escribe a un temporal y se renombra al final).
5. En Guanacaste Render: Abrir > el `.gr`.

Los errores (sin vista 3D, vista vacía, disco lleno, falta de permisos, falta de memoria) se muestran en español.

## Versiones de Revit y compilación

| Revit        | .NET              | Marco del proyecto |
|--------------|-------------------|--------------------|
| 2021 - 2024  | .NET Framework 4.8| `net48`            |
| 2025 - 2026  | .NET 8            | `net8.0-windows`   |
| 2027         | .NET 10           | `net10.0-windows`  |

Sin paquetes NuGet: se compila contra los `RevitAPI.dll` y `RevitAPIUI.dll` de la instalación de Revit
(`C:\Program Files\Autodesk\Revit <año>`). Requisitos: SDK de .NET 10 (o uno que soporte los marcos que necesite) y la
versión de Revit correspondiente.

```powershell
cd plugins\revit
.\build.ps1                      # compila los marcos cuya versión de Revit está instalada y arma artifacts\*.zip
dotnet build -c Release -f net8.0-windows -p:RevitApiDir="C:\Program Files\Autodesk\Revit 2026"   # una sola versión
```

Salidas: `bin\` y `obj\` locales (ignoradas por git) y el paquete en `artifacts\`.

## Instalación

Cierre Revit y ejecute (solo cuando usted lo decida):

```powershell
.\install.ps1 -Year 2026            # copia a %APPDATA%\Autodesk\Revit\Addins\2026
.\install.ps1 -Year 2026 -WhatIf    # muestra qué haría sin copiar nada
.\install.ps1 -Year 2026 -Uninstall
```

El script solo escribe `GuanacasteRender.addin` y la carpeta `GuanacasteRender\`; si ya existe un manifiesto con ese
nombre pero otro `AddInId`, se detiene sin tocarlo. La primera vez Revit pregunta si cargar el add-in: elija
"Cargar siempre".

## Prueba del formato sin Revit

`tests/GrFormatCheck` usa el mismo escritor (`src/Format/GrFile.cs`, sin tipos de Revit) para generar
`tests/fixtures/sample.gr`:

```powershell
dotnet run --project tests\GrFormatCheck -- tests\fixtures\sample.gr
```

El test Rust `src/features/projects/tests/revit_tests.rs` lee ese archivo, comprueba el texto JSON exacto, los bloques
de malla byte a byte y que `open()` lo carga con jerarquía, niveles y materiales.

## Estado de verificación

* Compilado con 0 advertencias para `net10.0-windows` contra la API de **Revit 2027** instalada.
* `net48` compila también (contra la API de 2027, como comprobación de sintaxis); `net8.0-windows` no se pudo
  compilar aquí (no hay Revit 2025/2026 instalado y la API de 2027 exige .NET 10).
* El comportamiento dentro de Revit (ribbon, `CustomExporter`, barra de progreso) **no se ha ejecutado**: está escrito
  contra la API pública, pero necesita una prueba manual con un modelo real.
* Limitaciones conocidas: sin texturas; origen mostrado como genérico en la aplicación (`origin.format` = "Revit");
  `OnMaterial` conserva el último material (comportamiento habitual de `CustomExporter`).
