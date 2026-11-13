//! Application use cases that coordinate several domains: requests that any screen can raise and the shell serves.

pub mod camera_link;
pub mod ground;
pub mod pointer;
pub mod rendering;
pub mod sample_model;
pub mod scene;
pub mod sync;
pub mod viewport;
pub mod workspace;

use crate::models::dialog::Dialog;
use eframe::egui::{Context, Id};
use std::path::PathBuf;

fn request_id() -> Id {
    Id::new("dialog-request")
}

/// Asks the shell to show `dialog` at the end of the current frame.
pub fn request_dialog(ctx: &Context, dialog: Dialog) {
    ctx.data_mut(|data| data.insert_temp(request_id(), dialog));
    ctx.request_repaint();
}

fn open_id() -> Id {
    Id::new("open-request")
}

/// Asks the shell to open the project at `path` (asking first about unsaved changes).
pub fn request_open(ctx: &Context, path: PathBuf) {
    ctx.data_mut(|data| data.insert_temp(open_id(), path));
    ctx.request_repaint();
}

/// Takes the pending open-file request, if any.
pub fn take_open_request(ctx: &Context) -> Option<PathBuf> {
    ctx.data_mut(|data| {
        let pending = data.get_temp::<PathBuf>(open_id());
        data.remove::<PathBuf>(open_id());
        pending
    })
}

/// Takes the pending dialog request, if any.
pub fn take_dialog_request(ctx: &Context) -> Option<Dialog> {
    ctx.data_mut(|data| {
        let pending = data.get_temp::<Dialog>(request_id());
        data.remove::<Dialog>(request_id());
        pending
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_is_served_once() {
        let ctx = Context::default();
        assert_eq!(take_dialog_request(&ctx), None);
        request_dialog(&ctx, Dialog::About);
        assert_eq!(take_dialog_request(&ctx), Some(Dialog::About));
        assert_eq!(take_dialog_request(&ctx), None);
    }

    #[test]
    fn an_open_request_is_served_once() {
        let ctx = Context::default();
        request_open(&ctx, PathBuf::from("C:/casa.gr"));
        assert_eq!(take_open_request(&ctx), Some(PathBuf::from("C:/casa.gr")));
        assert_eq!(take_open_request(&ctx), None);
    }
}
