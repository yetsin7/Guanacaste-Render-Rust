# Guanacaste Render (Rust)

Aplicación de escritorio para Windows, escrita en Rust con `eframe`/`egui`, para visualización arquitectónica en
tiempo real. Esta primera etapa contiene **solo las interfaces**: todas las pantallas del prototipo "Obsidian
Precision CAD" con navegación e interacciones básicas. No hay motor 3D, trazado de rayos ni formato de proyecto
todavía; los datos que se muestran salen de un único módulo de datos de muestra
(`src/services/sample_data/`) y las imágenes de `assets/placeholders/` son marcadores de posición que el
renderizador real reemplazará.

## Ejecutar

```powershell
cargo run --release
```

Opciones de desarrollo (para capturar pantallas sin hacer clic):

```powershell
cargo run --release -- --screen=library            # projects, build, library, inspector, materials, movie, render, settings, help
cargo run --release -- --screen=build --size=1333x820 --zoom=1
```

## Pruebas y calidad

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Ningún archivo `.rs` puede superar 400 líneas (lo comprueba `tests/line_limit.rs`).

## Estructura

Consulta [ARCHITECTURE.md](ARCHITECTURE.md) para el mapa de carpetas y dónde añadir cada cosa.

## Licencias de las fuentes

Las fuentes incluidas en `assets/fonts/` tienen licencias abiertas (detalle en `assets/fonts/NOTICE.md`):

- Inter: SIL Open Font License 1.1.
- JetBrains Mono: SIL Open Font License 1.1.
- Material Icons: Apache License 2.0.
