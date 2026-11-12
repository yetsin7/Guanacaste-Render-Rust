//! Fonts of the design system: Inter for interface text, JetBrains Mono for numbers and technical labels, and
//! Material Icons for glyphs. All three are openly licensed (see `assets/fonts/NOTICE.md`).

use eframe::egui::{self, FontDefinitions, FontFamily, FontId};
use std::sync::Arc;

const SEMIBOLD: &str = "inter-semibold";
const MONO_MEDIUM: &str = "mono-medium";
const ICONS: &str = "material-icons";

/// Interface text (Inter Regular).
pub fn body(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

/// Headings (Inter SemiBold).
pub fn heading(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

/// Numbers and technical labels (JetBrains Mono Regular).
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

/// Emphasized technical labels (JetBrains Mono Medium).
pub fn mono_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(MONO_MEDIUM.into()))
}

/// Material icon glyphs.
pub fn icon(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(ICONS.into()))
}

/// `headline-lg`: 24 px semibold.
pub fn headline_lg() -> FontId {
    heading(24.0)
}
/// `headline-md`: 20 px semibold.
pub fn headline_md() -> FontId {
    heading(20.0)
}
/// `headline-sm`: 16 px semibold.
pub fn headline_sm() -> FontId {
    heading(16.0)
}
/// `body-lg`: 14 px.
pub fn body_lg() -> FontId {
    body(14.0)
}
/// `body-md`: 13 px.
pub fn body_md() -> FontId {
    body(13.0)
}
/// `body-sm`: 12 px.
pub fn body_sm() -> FontId {
    body(12.0)
}
/// `label-lg`: 13 px mono medium.
pub fn label_lg() -> FontId {
    mono_medium(13.0)
}
/// `label-md`: 12 px mono medium.
pub fn label_md() -> FontId {
    mono_medium(12.0)
}
/// `label-sm`: 11 px mono regular.
pub fn label_sm() -> FontId {
    mono(11.0)
}

/// Registers every bundled font with egui.
pub fn install_fonts(context: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut add = |name: &str, bytes: &'static [u8], family: FontFamily| {
        fonts
            .font_data
            .insert(name.into(), Arc::new(egui::FontData::from_static(bytes)));
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, name.into());
    };
    add(
        "inter",
        include_bytes!("../../../assets/fonts/Inter-Regular.otf"),
        FontFamily::Proportional,
    );
    add(
        "jetbrains-mono",
        include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf"),
        FontFamily::Monospace,
    );
    add(
        SEMIBOLD,
        include_bytes!("../../../assets/fonts/Inter-SemiBold.otf"),
        FontFamily::Name(SEMIBOLD.into()),
    );
    add(
        MONO_MEDIUM,
        include_bytes!("../../../assets/fonts/JetBrainsMono-Medium.ttf"),
        FontFamily::Name(MONO_MEDIUM.into()),
    );
    add(
        ICONS,
        include_bytes!("../../../assets/fonts/MaterialIcons-Regular.ttf"),
        FontFamily::Name(ICONS.into()),
    );
    // Named families fall back to the base fonts for characters they lack (accents, symbols).
    for (family, fallback) in [(SEMIBOLD, "inter"), (MONO_MEDIUM, "jetbrains-mono")] {
        fonts
            .families
            .entry(FontFamily::Name(family.into()))
            .or_default()
            .push(fallback.into());
    }
    context.set_fonts(fonts);
}
