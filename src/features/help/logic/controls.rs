//! The real controls of the application, grouped for the help screen and the shortcuts dialog. Every row is a
//! feature that exists; both the action and the key names come from the catalog.

use crate::features::help::models::Shortcut;
use crate::services::localization::tr;

fn row(action: &'static str, keys: &'static str) -> Shortcut {
    Shortcut {
        action: tr(action),
        keys: tr(keys),
    }
}

/// The navigation controls.
pub fn navigation() -> Vec<Shortcut> {
    vec![
        row("ctl.fly", "ctk.fly"),
        row("ctl.up_down", "ctk.up_down"),
        row("ctl.look", "ctk.look"),
        row("ctl.fast", "ctk.fast"),
        row("ctl.slow", "ctk.slow"),
        row("ctl.speed_wheel", "ctk.speed_wheel"),
        row("ctl.zoom", "ctk.zoom"),
        row("ctl.orbit", "ctk.orbit"),
        row("ctl.pan", "ctk.pan"),
        row("ctl.frame", "ctk.frame"),
    ]
}

/// The cameras controls.
pub fn cameras() -> Vec<Shortcut> {
    vec![
        row("ctl.recall_camera", "ctk.recall_camera"),
        row("ctl.save_camera", "ctk.save_camera"),
    ]
}

/// The objects controls.
pub fn objects() -> Vec<Shortcut> {
    vec![
        row("ctl.place", "ctk.place"),
        row("ctl.select", "ctk.select"),
        row("ctl.add_select", "ctk.add_select"),
        row("ctl.drag", "ctk.drag"),
        row("ctl.delete", "ctk.delete"),
        row("ctl.deselect", "ctk.deselect"),
        row("ctl.undo", "ctk.undo"),
        row("ctl.redo", "ctk.redo"),
    ]
}

/// The application controls.
pub fn application() -> Vec<Shortcut> {
    vec![
        row("ctl.mode_build", "ctk.mode_build"),
        row("ctl.mode_photo", "ctk.mode_photo"),
        row("ctl.mode_movie", "ctk.mode_movie"),
        row("ctl.mode_export", "ctk.mode_export"),
        row("ctl.play", "ctk.play"),
        row("ctl.new_project", "ctk.new_project"),
        row("ctl.open_project", "ctk.open_project"),
        row("ctl.import", "ctk.import"),
        row("ctl.save", "ctk.save"),
        row("ctl.save_as", "ctk.save_as"),
        row("ctl.export", "ctk.export"),
        row("ctl.preferences", "ctk.preferences"),
        row("ctl.shortcuts", "ctk.shortcuts"),
        row("ctl.close", "ctk.close"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_has_text_and_keys() {
        for group in [navigation(), cameras(), objects(), application()] {
            assert!(!group.is_empty());
            for shortcut in group {
                assert!(!shortcut.keys.starts_with("ctk."), "{}", shortcut.keys);
                assert!(!shortcut.action.starts_with("ctl."), "{}", shortcut.action);
            }
        }
    }
}
