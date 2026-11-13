//! Heightmap file picker on its own thread, so the interface keeps drawing while the dialog is open.

use eframe::egui::Context;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Shared slot where the picker thread leaves its answer; cloning shares the slot.
#[derive(Debug, Clone, Default)]
pub struct PickSlot(Arc<Mutex<Pending>>);

#[derive(Debug, Default)]
struct Pending {
    open: bool,
    answer: Option<PathBuf>,
}

impl PickSlot {
    /// Opens a PNG picker unless one is already open; `ctx` is repainted when it closes.
    pub fn open(&self, ctx: &Context) {
        let Ok(mut pending) = self.0.lock() else {
            return;
        };
        if pending.open {
            return;
        }
        pending.open = true;
        drop(pending);
        let slot = self.0.clone();
        let ctx = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("landscape-picker".into())
            .spawn(move || {
                let picked = rfd::FileDialog::new()
                    .add_filter("PNG", &["png"])
                    .pick_file();
                if let Ok(mut pending) = slot.lock() {
                    pending.open = false;
                    pending.answer = picked;
                }
                ctx.request_repaint();
            });
        if spawned.is_err() {
            if let Ok(mut pending) = self.0.lock() {
                pending.open = false;
            }
        }
    }

    /// The chosen file, once, after the dialog closed.
    pub fn take(&self) -> Option<PathBuf> {
        self.0.lock().ok()?.answer.take()
    }
}
