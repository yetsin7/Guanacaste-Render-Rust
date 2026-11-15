//! Executable entry point; all wiring lives in `core`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result {
    guanacaste_render_rust::commands::model_link::record_application_path();
    guanacaste_render_rust::run()
}
