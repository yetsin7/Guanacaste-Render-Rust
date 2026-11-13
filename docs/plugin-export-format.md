# Formato de exportación de los plugins (SketchUp, Revit)

Los plugins escriben directamente un proyecto **`.gr`** de Guanacaste Render (firma `GRRPROJ1`, contenedor versión 1,
esquema 1). Es el mismo contenedor que guarda la aplicación (`src/features/projects/services/container.rs`), así que
Guanacaste Render lo abre con "Abrir" sin pasar por DAE/OBJ y sin ningún cambio en el lado Rust. No se usa el formato
legado `G3DNATV1` (solo lectura, sin mallas arbitrarias).

Todo es **little endian**. Longitudes en bytes. Unidades: **metros**, eje **Z arriba** (los plugins convierten).

## 1. Contenedor

```
CABECERA (96 bytes)
  off  tam  campo
   0    8   firma "GRRPROJ1" (ASCII)
   8    2   versión del contenedor = 1            (u16)
  10    2   reservado = 0
  12    4   versión del esquema JSON = 1          (u32)
  16   16   id del documento (16 bytes aleatorios)
  32    8   longitud del JSON sin comprimir       (u64)
  40    8   longitud del JSON comprimido (gzip)   (u64)
  48    4   número de bloques                     (u32)
  52    4   reservado = 0
  56   32   SHA-256 del JSON sin comprimir
  88    8   offset absoluto del índice de bloques (u64)
JSON comprimido con gzip            empieza en 96
BLOQUES, cada uno comprimido con gzip, uno tras otro
ÍNDICE: 64 bytes por bloque, empieza en el offset de la cabecera y llega al final del archivo
  0   4  tipo (u32): 1 = malla, 2 = miniatura PNG
  4   4  id (u32): para mallas, el índice de la malla en document.meshes (0, 1, 2...)
  8   8  offset absoluto del dato comprimido (u64)
 16   8  longitud comprimida (u64)
 24   8  longitud sin comprimir (u64)
 32  32  SHA-256 de los datos sin comprimir
```

Reglas que el lector comprueba: `96 + json_gz <= index_offset`; `index_offset + 64 * bloques == tamaño del archivo`;
cada bloque cae entre el fin del JSON y el índice; las sumas SHA-256 coinciden; el gzip descomprime exactamente a la
longitud declarada. El JSON debe ser no vacío.

`pack` de Ruby: cabecera `a8 v v V a16 Q< Q< V V a32 Q<`; entrada de índice `V V Q< Q< Q< a32`.

## 2. Bloque de malla (tipo 1, id = índice de malla)

```
u32  nv                    número de vértices
u32  ni                    número de índices (múltiplo de 3)
f32  x, y, z   * nv        posiciones en metros, relativas a origin_shift
u32  i         * ni        índices de triángulo (< nv), sentido antihorario = cara frontal
```

Ruby: `[nv, ni].pack("VV") + posiciones.pack("e*") + indices.pack("V*")`.

## 3. Documento JSON (UTF-8)

Los campos desconocidos se ignoran; los ausentes toman valores por defecto, salvo los marcados.

```json
{
  "meta":   { "name": "Casa", "created": 1700000000, "modified": 1700000000, "app_version": "0.1.0" },
  "origin": { "format": "SketchUp", "version": "SketchUp 2024", "application": "SketchUp",
              "file_name": "casa.skp", "file_size": 0, "length_unit": "in", "unit_scale": 0.0254,
              "imported": 1700000000 },
  "origin_shift": [0.0, 0.0, 0.0],
  "levels":    [ { "name": "Nivel 1", "elevation": 0.0, "node": 3 } ],
  "materials": [ { "name": "Madera", "color": [0.5, 0.3, 0.1, 1.0] } ],
  "nodes":     [ { "name": "Muro", "class": "Wall", "kind": "element", "type_name": null,
                   "parent": 0, "meshes": [0, 1], "level": null } ],
  "meshes":    [ { "material": 0, "vertices": 24, "indices": 36 } ]
}
```

* `origin`: si está, todos sus campos son obligatorios. `origin_shift`: metros, doble precisión; posición real =
  vértice + shift.
* `materials[].color`: RGBA 0..1, `a` = opacidad.
* `meshes`: una entrada por bloque de malla; `vertices` e `indices` deben coincidir con el bloque.
* `nodes[0]` es la raíz (`kind` `"project"`, sin `parent`). Cada otro nodo apunta a su `parent`; las listas de hijos
  se derivan de `parent`.
* `kind`: `project`, `site`, `building`, `level`, `element` (objeto con geometría) o `group` (agrupación). Cualquier
  otro texto se lee como `group`.
* `class`: categoría de origen (Revit: categoría; SketchUp: etiqueta). `type_name`: tipo (Revit) o definición del
  componente (SketchUp).
* `meshes[i].material` indexa `materials`; una malla tiene un solo material. `nodes[].meshes` indexa `meshes`.
* `levels` ordenados por elevación; `node` es el índice del nodo de tipo `level`. Opcional.
* No hay texturas ni instancias: la geometría instanciada se escribe expandida, con su transformación aplicada.
* `origin.format` desconocido para la aplicación se muestra como un origen genérico; `unit_scale` y `length_unit` son
  informativos (los vértices ya van en metros).

Ruby: `JSON.generate(doc)`; los `Float` se serializan con ida y vuelta exacta.

## 4. Cómo lo escribe SketchUp

* 1 pulgada = 0,0254 m; SketchUp ya es Z arriba, sin giro de ejes.
* Cada grupo o componente instanciado es un nodo (`element` si tiene geometría propia, `group` si no); `class` =
  etiqueta (tag) efectiva, `type_name` = nombre de la definición del componente.
* Las caras se triangulan con `Face#mesh` (admite huecos). Cara frontal con material frontal; si solo la posterior
  tiene material, se emite con el sentido invertido y ese material.
* `origin_shift` = centro XY del cuadro delimitador y mínimo Z, para conservar precisión en `f32`.
* Las texturas no existen en el formato: el material conserva su color (el promedio que da SketchUp) y se avisa.

## 5. Prueba

`src/features/projects/tests/plugin_fixture_tests.rs` construye byte a byte (sin usar el escritor de Rust) un archivo
con exactamente este diseño y comprueba que `open()` lo lee: jerarquía, materiales, mallas y `origin_shift`.
