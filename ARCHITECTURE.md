# Arquitectura de Guanacaste Render

Este proyecto usa una organización modular por capas y por dominio, igual que el esqueleto de Guanacaste 3D. El
código Rust ejecutable vive en `src/`; `assets/` contiene recursos estáticos y `tests/` contiene pruebas de
integración. `Cargo.toml` define el paquete y sus puntos de entrada (`src/lib.rs` y `src/main.rs`).

Estado actual: **solo interfaces**. No existe motor 3D, trazado de rayos ni formato de proyecto. Todo dato que se
muestra sale de `src/services/sample_data/` y las imágenes de `assets/placeholders/` son marcadores de posición.

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

`src/features/` contiene `projects/`, `viewport/`, `library/`, `materials/`, `movie/`, `rendering/`, `settings/` y
`help/`. Cada dominio crea solo las carpetas que necesita:

- `ui/`: presentadores propios del dominio; no aloja navegación global.
- `components/`: controles de interfaz exclusivos de ese dominio.
- `logic/`: reglas y casos de uso, independientes de la interfaz.
- `models/`: entidades, tipos de valor y estado de interfaz del dominio.
- `services/`: persistencia, motores o integraciones detrás de interfaces explícitas.
- `tests/`: pruebas unitarias del dominio.

`viewport/` es la envoltura de la vista 3D (fondo, brújula, superposiciones de navegación) y `rendering/` reserva el
lugar de las futuras funciones del motor (vista previa y render).

## Pantallas

| Pantalla | Carpeta | Parámetro `--screen=` |
|---|---|---|
| Proyectos | `src/screens/projects/` | `projects` |
| Construcción | `src/screens/build/` | `build` |
| Biblioteca y inspector de activos | `src/screens/library/` | `library`, `inspector` |
| Editor de materiales | `src/screens/materials/` | `materials` |
| Estudio de película | `src/screens/movie/` | `movie` |
| Ajustes | `src/screens/settings/` | `settings` |
| Ayuda y diagnóstico | `src/screens/help/` | `help` |
| Estudio de render (vacío) | `src/screens/render/` | `render` |

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
