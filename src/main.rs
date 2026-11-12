//! Executable entry point; all wiring lives in `core`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result {
    guanacaste_render_rust::run()
}
