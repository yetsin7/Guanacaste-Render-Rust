//! Color tokens of the "Obsidian Precision CAD" design system, named after the Material roles the prototype uses.

use eframe::egui::Color32;

/// Builds an opaque color from a `0xRRGGBB` value.
pub const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// Page and 3D canvas background (`surface`).
pub const SURFACE: Color32 = hex(0x121316);
/// Deepest recessed slots (`surface-container-lowest`).
pub const SURFACE_LOWEST: Color32 = hex(0x0d0e11);
/// Docked bars and the tool rail (`surface-container-low`).
pub const SURFACE_LOW: Color32 = hex(0x1a1b1e);
/// Containers and inputs (`surface-container`).
pub const SURFACE_CONTAINER: Color32 = hex(0x1f1f23);
/// Raised controls and hovered rows (`surface-container-high`).
pub const SURFACE_HIGH: Color32 = hex(0x292a2d);
/// Highest surface step: hairlines on dark panels and pressed chips (`surface-container-highest`).
pub const SURFACE_HIGHEST: Color32 = hex(0x343538);
/// Primary text (`on-surface`).
pub const ON_SURFACE: Color32 = hex(0xe3e2e6);
/// Secondary text (`on-surface-variant`).
pub const ON_SURFACE_VARIANT: Color32 = hex(0xc3c6d4);
/// Muted text and icons (`outline`).
pub const OUTLINE: Color32 = hex(0x8d909d);
/// Structural borders (`outline-variant`).
pub const OUTLINE_VARIANT: Color32 = hex(0x424752);
/// Focal accent (`primary`).
pub const PRIMARY: Color32 = hex(0xadc6ff);
/// Text drawn on a `PRIMARY` fill (`on-primary`).
pub const ON_PRIMARY: Color32 = hex(0x002e69);
/// Bright cerulean for the active strip and selection (`primary-container`).
pub const PRIMARY_CONTAINER: Color32 = hex(0x75a4ff);
/// Text drawn on a `PRIMARY_CONTAINER` fill (`on-primary-container`).
pub const ON_PRIMARY_CONTAINER: Color32 = hex(0x00387d);
/// Electric blue of pressed toggles and primary buttons (`secondary-container`).
pub const SECONDARY_CONTAINER: Color32 = hex(0x0671d7);
/// Text drawn on a `SECONDARY_CONTAINER` fill (`on-secondary-container`).
pub const ON_SECONDARY_CONTAINER: Color32 = hex(0xf7f8ff);
/// Lighter blue for secondary emphasis (`secondary`).
pub const SECONDARY: Color32 = hex(0xa8c8ff);
/// Telemetry green for live readouts (`tertiary`).
pub const TERTIARY: Color32 = hex(0x5cdbc0);
/// Dark green drawn on a `TERTIARY` fill (`on-tertiary`).
pub const ON_TERTIARY: Color32 = hex(0x00382e);
/// Error text and icons (`error`).
pub const ERROR: Color32 = hex(0xffb4ab);
/// Error fills, e.g. the close button hover (`error-container`).
pub const ERROR_CONTAINER: Color32 = hex(0x93000a);
/// Warning amber for thresholds.
pub const WARNING: Color32 = hex(0xf5a623);
/// Fixed light primary (`primary-fixed`).
pub const PRIMARY_FIXED: Color32 = hex(0xd8e2ff);
/// Floating-panel border.
pub const FLOATING_BORDER: Color32 = hex(0x45484e);

/// Returns `color` with the given alpha (0..=255) and the same hue.
pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}
