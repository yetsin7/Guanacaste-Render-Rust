//! A row of mutually exclusive buttons with their own gaps (tab strips, category pills, tool buttons).

use super::button::{Button, ButtonStyle};
use crate::config::theme::colors;
use eframe::egui::{Color32, FontId, Ui};

/// One option of a choice row.
pub struct Choice<'a> {
    /// Leading icon, if any.
    pub icon: Option<char>,
    /// Label.
    pub label: &'a str,
}

/// Look of a choice row.
pub struct ChoiceStyle {
    /// Fill of the selected option.
    pub selected_fill: Color32,
    /// Text color of the selected option.
    pub selected_ink: Color32,
    /// Fill of the other options.
    pub idle_fill: Color32,
    /// Text color of the other options.
    pub idle_ink: Color32,
    /// Label font.
    pub font: FontId,
    /// Label font of the selected option, when it differs.
    pub selected_font: Option<FontId>,
    /// Icon size.
    pub icon_size: f32,
    /// Horizontal and vertical padding.
    pub padding: (f32, f32),
}

impl ChoiceStyle {
    /// Applies the style to a button for the given selection state.
    fn button(&self, label: &str, icon: Option<char>, selected: bool) -> Button {
        let (fill, ink) = if selected {
            (self.selected_fill, self.selected_ink)
        } else {
            (self.idle_fill, self.idle_ink)
        };
        let font = match (&self.selected_font, selected) {
            (Some(font), true) => font.clone(),
            _ => self.font.clone(),
        };
        let mut button = Button::new(label, ButtonStyle::Quiet)
            .font(font)
            .padding(self.padding.0, self.padding.1)
            .colors(fill, ink);
        if let Some(glyph) = icon {
            button = button.icon(glyph, self.icon_size);
        }
        button
    }
}

/// Draws the options left to right with `gap` between them; returns true when the selection changed.
pub fn show(
    ui: &mut Ui,
    choices: &[Choice],
    selected: &mut usize,
    style: &ChoiceStyle,
    gap: f32,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (index, choice) in choices.iter().enumerate() {
            if style
                .button(choice.label, choice.icon, index == *selected)
                .show(ui)
                .clicked()
                && index != *selected
            {
                *selected = index;
                changed = true;
            }
        }
    });
    changed
}

/// Transparent tabs with a solid blue selected tab (build mode tab strip).
pub fn tab_style(font: FontId, icon_size: f32) -> ChoiceStyle {
    ChoiceStyle {
        selected_fill: colors::SECONDARY_CONTAINER,
        selected_ink: colors::ON_SECONDARY_CONTAINER,
        idle_fill: Color32::TRANSPARENT,
        idle_ink: colors::ON_SURFACE_VARIANT,
        font,
        selected_font: None,
        icon_size,
        padding: (12.0, 6.0),
    }
}

/// Small pills with a cerulean selected pill (category row).
pub fn pill_style(font: FontId) -> ChoiceStyle {
    ChoiceStyle {
        selected_fill: colors::PRIMARY_CONTAINER,
        selected_ink: colors::ON_PRIMARY_CONTAINER,
        idle_fill: colors::SURFACE_CONTAINER,
        idle_ink: colors::OUTLINE,
        font,
        selected_font: None,
        icon_size: 14.0,
        padding: (10.0, 4.0),
    }
}

/// Tool buttons with a pale selected button.
pub fn tool_style(font: FontId) -> ChoiceStyle {
    ChoiceStyle {
        selected_fill: colors::PRIMARY,
        selected_ink: colors::ON_PRIMARY,
        idle_fill: colors::SURFACE_CONTAINER,
        idle_ink: colors::ON_SURFACE_VARIANT,
        font,
        selected_font: None,
        icon_size: 16.0,
        padding: (12.0, 6.0),
    }
}
