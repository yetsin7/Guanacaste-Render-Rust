# Formato de proyecto `.gr` (Guanacaste Render)

Los proyectos de Guanacaste Render se guardan en archivos con extensión **`.gr`**. Guanacaste 3D también usa
`.gr` con su propio contenedor, así que **la extensión nunca decide el formato: lo deciden los 8 primeros bytes**.

| Firma (bytes 0–7) | Qué es | Qué hace Guanacaste Render |
|---|---|---|
| `GRRPROJ1` | Proyecto de Guanacaste Render | Lo abre como proyecto (lectura y escritura). |
| `G3DNATV1` | Proyecto de Guanacaste 3D | Lo importa en **solo lectura**; nunca lo sobrescribe. |
| cualquier otra | — | Error claro: «Firma desconocida». |

La detección vive en `features::import_export::logic::formats::{sniff, signature_of}` y la apertura en
`features::projects::services::open::open`.

## Regla de no sobrescritura de Guanacaste 3D

- `store::save` comprueba la firma del destino antes de escribir: si es `G3DNATV1` devuelve
  `StoreError::LegacyTarget` y no toca el archivo (ni crea `.tmp` ni `.bak`).
- Un proyecto que vino de un `.gr` de Guanacaste 3D no tiene ruta propia (`Project::path = None`,
  `Project::legacy_source = Some(original)`). «Guardar» actúa como «Guardar como» y propone
  `<nombre> - render.gr` en la misma carpeta, con la explicación «Se guarda con otro nombre para no estropear el
  archivo original de Guanacaste 3D».
- Pruebas: `features::projects::tests::legacy_tests::a_guanacaste_3d_file_is_never_overwritten` (bytes idénticos
  antes y después) y `core::dialogs::tests::save_on_a_guanacaste_3d_project_becomes_save_as_with_a_reason`.

## Contenedor `GRRPROJ1` (versión de contenedor 1)

Todos los enteros son little endian.

```text
CABECERA (96 bytes)
  0   8  firma "GRRPROJ1"
  8   2  versión de contenedor (u16 = 1)
 10   2  reservado (0)
 12   4  versión de esquema del documento (u32, hoy 1)
 16  16  identificador del documento (se conserva entre guardados)
 32   8  longitud del JSON sin comprimir (u64)
 40   8  longitud del JSON comprimido (u64)
 48   4  número de bloques (u32)
 52   4  reservado (0)
 56  32  SHA-256 del JSON sin comprimir
 88   8  posición del índice de bloques (u64)
JSON gzip (desde el byte 96)
BLOQUES gzip, uno detrás de otro
ÍNDICE: 64 bytes por bloque
  0   4  tipo (1 = malla, 2 = miniatura PNG)
  4   4  identificador dentro del tipo (número de malla; 0 para la miniatura)
  8   8  posición absoluta de los datos comprimidos
 16   8  longitud comprimida
 24   8  longitud sin comprimir
 32  32  SHA-256 de los datos sin comprimir
```

El índice termina exactamente al final del archivo; cualquier otra longitud se considera truncamiento.

### Bloque de malla (tipo 1)

`u32 vértices, u32 índices, f32×3 por vértice (metros, Z arriba, relativos a origin_shift), u32 por índice`.
Los recuentos deben coincidir con `meshes[i]` del documento y todo índice debe apuntar a un vértice existente.

### Documento JSON (esquema 1)

`ProjectDocument` (`src/features/projects/models/document.rs`): `meta` (nombre, fechas, versión de la aplicación),
`origin` (formato, versión, aplicación, nombre del archivo original —nunca la ruta—, tamaño, unidad y escala),
`origin_shift` (desplazamiento en metros que mantiene precisos los `f32`), `levels`, `materials`, `nodes`
(jerarquía con clase IFC, tipo, nivel y mallas), `meshes` (metadatos de cada bloque), `cameras`, `timeline`,
`render`, `sun_sky` y `vegetation`. Los campos ausentes toman su valor por defecto. Los números `f64` se
conservan bit a bit (serde_json con `float_roundtrip`; lo comprueba
`every_float_survives_the_json_document_bit_for_bit`).

## Guardado atómico

1. Se escribe `<archivo>.gr.tmp` en la misma carpeta y se sincroniza con el disco.
2. Se vuelve a abrir y se verifican cabecera, JSON y **todos** los bloques con sus SHA-256.
3. Si existía una versión anterior se copia a `<archivo>.gr.bak`.
4. Se renombra el temporal sobre el destino. Ante cualquier fallo el original queda intacto y el temporal se borra.

## Carga con límites

`Limits` (por defecto 256 MB de JSON, 1 GB por bloque, 8 GB en total) se comprueba con las longitudes declaradas
**antes** de reservar memoria; los bloques se leen de uno en uno. Un archivo dañado, truncado o con sumas que no
coinciden produce `ContainerError::Damaged` y nunca un pánico.

## Versiones y migraciones

- Esquema menor que el actual: se migra en memoria con la cadena `logic::migrations::MIGRATIONS` (un paso por
  versión, cada uno con su prueba) y se avisa con `LoadWarning::Migrated`.
- Esquema mayor que el actual: se abre en **solo lectura** (`LoadWarning::NewerSchema`); guardar devuelve
  `StoreError::ReadOnly` y el diálogo propone guardar una copia.
- Versión de contenedor desconocida: `ContainerError::Unsupported`.

## Recuperación y recientes

- Cada 60 s, si hay cambios sin guardar, se escribe una copia en `<datos>/recovery/<id>.gr` (mismo formato y
  mismo guardado atómico). Al guardar o descartar los cambios se borra. Al arrancar, si quedan copias, se ofrece
  restaurarlas.
- `<datos>/recent.json` guarda hasta 12 rutas; la lista muestra tamaño, antigüedad y la miniatura del archivo.
- `<datos>` es `%LOCALAPPDATA%\GuanacasteRender`, o la carpeta de la variable `GR_DATA_DIR` si existe.

## Importador de Guanacaste 3D (`G3DNATV1`, solo lectura)

Cabecera de 81 bytes: firma, `i32` versión de contenedor (1), `u8` tipo de documento (1 = proyecto; familias y
plantillas se rechazan), `i32` esquema, GUID de 16 bytes, `i64` longitud sin comprimir, `i64` longitud comprimida,
SHA-256 de 32 bytes y el JSON comprimido con gzip. Se verifica todo antes de leer.

Se convierten en sólidos simples: niveles, muros (con altura hasta el nivel superior si lo tienen), losas y
cubiertas (extruidas desde su contorno), columnas (prisma rectangular) y elementos estructurales. Avisos honestos:
huecos de puertas y ventanas no recortados, cubiertas inclinadas mostradas planas, columnas no rectangulares
aproximadas. Habitaciones, instalaciones y planos se cuentan como omitidos. Código:
`src/features/import_export/services/legacy.rs`.

Los proyectos `demo-house.gr` y `garden.gr` de Guanacaste 3D no estaban disponibles en el equipo cuando se escribió
este importador; las pruebas usan un proyecto sintético escrito con el formato documentado por Guanacaste 3D.

## Cifras con un modelo real

`cargo run --release --example project_roundtrip -- "<modelo.ifc>"` importa, guarda en una carpeta temporal,
reabre, compara todo y borra el temporal. Con una exportación IFC real de Revit de 119.6 MB (1875 elementos,
954 924 triángulos): proyecto `.gr` de 13.9 MB, guardado con verificación en ~0.58 s, apertura en ~0.22 s,
contenido idéntico tras reabrir.

## Enlace en vivo con Guanacaste 3D (`model_link`)

Un proyecto creado a partir de un `.g3d` guarda en su documento el campo opcional `model_link`
(`path`, `modified_ms`, `length`). Si el archivo trae al lado el `<archivo>.model-link` que escribe la orden
«Exportar a render» de Guanacaste 3D, el enlace apunta al modelo de origen (`SourcePath`, o `RelativeSourcePath`
si el modelo se movió junto con el proyecto).

Mientras el proyecto está abierto, Guanacaste Render mira ese archivo cada segundo. Cuando cambia y su tamaño y
fecha se repiten en dos lecturas seguidas (Guanacaste 3D terminó de escribir), vuelve a leer **solo la
geometría** (`origin`, `levels`, `nodes`, `meshes`) en segundo plano. Cámaras, animación, luces, paisaje,
objetos, entorno, filtros y los materiales editados (por nombre) se conservan. Los archivos sin `model_link`
se abren igual que antes. Código: `src/commands/model_link.rs`.
