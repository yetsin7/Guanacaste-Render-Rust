# Arquitectura de Guanacaste Render

Este proyecto usa una organización modular por capas y por dominio, igual que el esqueleto de Guanacaste 3D. El
código Rust ejecutable vive en `src/`; `assets/` contiene recursos estáticos y `tests/` contiene pruebas de
integración. `Cargo.toml` define el paquete y sus puntos de entrada (`src/lib.rs` y `src/main.rs`).

Estado actual: software funcional por modos (Construir, Foto, Película, Exportar) con vista 3D real, render por CPU, proyectos `.gr` e importadores. Ya no hay datos de muestra: `services/sample_data` se eliminó; solo la imagen ilustrativa de la pantalla de inicio es de ejemplo y está rotulada como tal.

## Mapa del proyecto

- `src/core/`: ciclo de vida de la aplicación (ventana sin marco, atajos de arranque, composición del shell).
- `src/features/`: dominios del producto, independientes entre sí.
- `src/screens/`: composición de pantallas y navegación (`src/screens/<flujo>/`).
- `src/components/`: componentes de interfaz compartidos (botones, chips, controles, barra de título, riel...).
- `src/services/`: servicios transversales: localización en español, caché de texturas y datos de muestra.
- `src/models/`: tipos compartidos que no pertenecen a un dominio (navegación, arte de marcador de posición).
- `src/commands/`: casos de uso que coordinan varios dominios (aún sin contenido).
- `src/config/`: configuración tipada: tokens del tema "Obsidian Precision CAD" y opciones de arranque.
- `src/utils/`: utilidades pequeñas y puras (matemática de cuadrículas).
- `assets/`: fuentes (`fonts/`, con sus licencias) e imágenes de marcador de posición (`placeholders/`).
- `tests/`: pruebas de integración (límite de 400 líneas, cobertura de claves de localización, datos de muestra).

## Dominios

`src/features/` contiene `projects/`, `build/`, `viewport/`, `library/`, `materials/`, `movie/`, `rendering/`,
`settings/` y `help/`. Cada dominio crea solo las carpetas que necesita:

- `ui/`: presentadores propios del dominio; no aloja navegación global.
- `components/`: controles de interfaz exclusivos de ese dominio.
- `logic/`: reglas y casos de uso, independientes de la interfaz.
- `models/`: entidades, tipos de valor y estado de interfaz del dominio.
- `services/`: persistencia, motores o integraciones detrás de interfaces explícitas.
- `tests/`: pruebas unitarias del dominio.

`build/` es la colocación de objetos sobre la vista 3D; `viewport/` es la envoltura de la vista 3D (fondo y retícula);
`rendering/` contiene el vocabulario de calidad, la cola de render, la exportación de imágenes y el estudio de render,
y es el lugar de las futuras funciones del motor.

## Pantallas y flujo por modos

La app tiene una pantalla de inicio (`home`: nuevo, abrir, importar, proyectos recientes) y un **espacio de trabajo**
con la vista 3D a pantalla completa y cuatro modos siempre visibles en la barra de modos (`components/mode_bar.rs`,
teclas F1-F4): Construir, Foto, Película y Exportar. Las pantallas secundarias (biblioteca, inspector de activos, editor
de materiales, ajustes, ayuda) se abren desde la barra de modos o desde el contenido.

| Pantalla | Código | `--screen=` |
|---|---|---|
| Inicio | `src/screens/home.rs` | `home` (`projects`) |
| Construir / Foto / Película / Exportar | `src/screens/workspace/` | `build`, `photo`, `movie`, `render` (`export`) |
| Biblioteca y inspector de activos | `src/screens/library/` | `library`, `inspector` |
| Editor de materiales | `src/screens/materials/` | `materials` |
| Ajustes / Ayuda | `src/screens/settings/`, `src/screens/help/` | `settings`, `help` |

**Estado compartido:** `models::AppState` (`src/models/app_state.rs`) guarda el proyecto abierto (`workspace`), el modo
y la categoría activos, la cámara actual y las cámaras guardadas (`models::camera`), los efectos de foto, los paneles
abiertos, las solicitudes al shell (`requests`: encuadrar, navegar) y el estado de cada panel de dominio.

**Vista 3D y herramientas:** la vista 3D real (`features/viewport`) es la misma en todos los modos
(`workspace/viewport.rs`); sobre ella flotan la paleta de herramientas (`components/tool_palette.rs`,
`workspace/palette.rs`), los avisos de render (`workspace/render_overlay.rs`) y los paneles. Los rayos del ratón
(`commands/pointer.rs`) mueven los pinceles de paisaje y la colocación de objetos. `commands/scene.rs` compone la
escena (modelo + objetos + suelo base), `commands/sync.rs` guarda y carga cada dominio en el `.gr`, y
`commands/thumbnails.rs` captura las miniaturas de las cámaras guardadas. Todo proyecto tiene un suelo base plano
de 200 m en Z = 0 (`Landscape::default`), que se puede ocultar; los modelos importados se posan sobre él.

**Ranuras de contenido:** cada modo es un `ModeSlots` (`src/screens/workspace/mod.rs`): un panel derecho, un cajón
inferior y una superposición, todos `fn(&mut Ui, &mut AppState)`. Para llenar una ranura se sustituye la función en
`workspace/{build,photo,movie,export}.rs`; la vista 3D es `workspace/viewport.rs`. Los paneles son colapsables
(`components/dock_panel.rs`) y los atajos visibles viven en `components/tool_hints.rs`. Atajos: `ASDW`/`QE` (vista 3D,
pendiente de la vista real), `F` encuadrar, `Espacio` reproducir en Película, `F1-F4` modos, `Ctrl+S` guardar.

## Diálogos y avisos

Los diálogos (abrir, guardar, nueva escena, exportar imagen, preferencias, atajos, acerca de, errores) y el panel de
notificaciones se alojan en `src/core/dialogs.rs`. Cualquier pantalla pide uno con
`commands::request_dialog(ctx, Dialog::...)`; el menú de la aplicación se abre desde el avatar de la barra de título y
los atajos `Ctrl+N/O/S/E/,/` los abren directamente. Los controles que requieren un motor o un formato de proyecto que
aún no existen se muestran desactivados con su motivo (`Button::disabled`, `form::disabled`). Opción de desarrollo:
`--dialog=<new|open|save|export|preferences|shortcuts|about|notifications|error>`.

## Dónde añadir cada cosa

1. **Pantalla nueva:** crea un módulo en `src/screens/<flujo>/`, añade su variante a `models::navigation::Screen` y
   regístrala en `src/screens/mod.rs`. La pantalla compone componentes y delega casos de uso; no implementa reglas
   del dominio.
2. **Botón o control reutilizable:** añádelo a `src/components/`. Si solo sirve a un dominio, colócalo en
   `src/features/<dominio>/components/`. Si el mismo aspecto visual aparece dos veces, debe ser un único componente.
3. **Regla del dominio:** ubica su caso de uso en `src/features/<dominio>/logic/`; coordina varios dominios desde
   `src/commands/`.
4. **Modelo:** usa `src/features/<dominio>/models/` para entidades específicas y `src/models/` solo para tipos
   realmente compartidos.
5. **Servicio o integración:** adaptadores de dominio en `src/features/<dominio>/services/`; reserva `src/services/`
   para capacidades transversales.
6. **Texto visible:** todo texto de interfaz va en `src/services/localization/<dominio>.rs` y se consulta con
   `tr("clave")`; no escribas literales en pantallas ni componentes. Los textos de los datos de muestra viven en
   `src/services/sample_data/`.
7. **Datos de muestra:** `src/services/sample_data/` es el único lugar con datos de relleno; al conectar datos reales
   se reemplaza solo ese módulo.
8. **Configuración o recurso:** configuración tipada en `src/config/`; fuentes, texturas e imágenes en `assets/`.
9. **Pruebas:** pruebas unitarias junto al código (`#[cfg(test)]` o `tests/` del dominio) y pruebas que atraviesan
   módulos en `tests/`.
10. **Utilidad:** `src/utils/` solo si la función es pura, genérica y útil en más de un dominio.

## Reglas de crecimiento

- Ningún archivo `.rs` supera 400 líneas (`tests/line_limit.rs` lo verifica); divide por responsabilidad.
- Nombres de módulos y archivos en `snake_case`; tipos en `UpperCamelCase`.
- `mod.rs` es solo punto de composición cuando el módulo crece; las implementaciones viven en archivos propios.
- Toda función y tipo público lleva comentario de documentación; sin código muerto.
- `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings` deben pasar.
- La interfaz no repinta de forma continua: solo se solicita repintado ante interacción o animación.

## Puntos de entrada

`src/main.rs` inicia el ejecutable y `src/lib.rs` expone la biblioteca. El arranque y el cableado global viven en
`core`; no pongas lógica de dominio en `main.rs`.

## Control de calidad, deshacer y panorama

- `commands/undo.rs`: deshacer/rehacer global (Ctrl+Z, Ctrl+Y) de objetos y paisaje en el orden en que se hicieron.
- `commands/sync/photo.rs`: filtros, cámara fotográfica y calidad del modo Foto viajan en el `.gr`; cada cámara guardada
  lleva sus filtros y su miniatura (base64 PNG), todo retrocompatible.
- Los objetos se apoyan en el relieve (`ObjectsState::ground`, enlazado en `sync::link_ground`) y lo siguen al esculpir.
- Panorama 360° (pestaña de Foto): `RenderCamera::panorama` (equirrectangular) y visor en `rendering/components/panorama_viewer.rs`.
- `core/qa_tests.rs`: pruebas de extremo a extremo con eventos sintéticos de egui (colocar, seleccionar, borrar,
  deshacer, renderizar cada cámara guardada, panorama, guardar y reabrir, reposo sin repintado continuo).