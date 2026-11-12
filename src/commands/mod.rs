//! Application use cases that coordinate several domains: requests that any screen can raise and the shell serves.

use crate::models::dialog::Dialog;
use eframe::egui::{Context, Id};

fn request_id() -> Id {
    Id::new("dialog-request")
}

/// Asks the shell to show `dialog` at the end of the current frame.
pub fn request_dialog(ctx: &Context, dialog: Dialog) {
    ctx.data_mut(|data| data.insert_temp(request_id(), dialog));
    ctx.request_repaint();
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
}
