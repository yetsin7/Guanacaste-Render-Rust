//! Writing pictures: names are sanitized, files stay inside the chosen folder and nothing is overwritten.

use crate::features::rendering::models::{ImageFormat, OutputTarget};
use crate::features::rendering::services::{resolve_path, safe_stem, save_image, OutputError};
use image::{Rgba, RgbaImage};
use std::path::PathBuf;

/// A fresh empty folder for one test.
pub fn temp_folder(name: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "guanacaste-render-test-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&folder);
    folder
}

fn target(folder: &std::path::Path, name: &str, format: ImageFormat) -> OutputTarget {
    OutputTarget {
        folder: folder.to_path_buf(),
        file_name: name.into(),
        format,
        jpeg_quality: 0.9,
    }
}

#[test]
fn typed_names_lose_folders_and_forbidden_characters() {
    assert_eq!(safe_stem("../../evil"), "evil");
    assert_eq!(safe_stem("C:\\Windows\\System32\\x.png"), "x");
    assert_eq!(safe_stem("sala: <vista>?"), "sala vista");
    assert_eq!(safe_stem("foto.JPG"), "foto");
    assert_eq!(safe_stem("con"), "render");
    assert_eq!(safe_stem("NUL.txt"), "render");
    assert_eq!(safe_stem("  ..  "), "render");
    assert_eq!(safe_stem(""), "render");
    assert_eq!(safe_stem("villa.noche"), "villa.noche");
}

#[test]
fn the_path_always_sits_directly_inside_the_folder() {
    let folder = temp_folder("inside");
    for name in ["..\\..\\x", "/etc/passwd", "a/../../b", "..", "C:/x"] {
        let path = resolve_path(&target(&folder, name, ImageFormat::Png)).expect("path");
        assert_eq!(path.parent(), Some(folder.as_path()), "{name}");
    }
}

#[test]
fn an_empty_folder_is_refused() {
    let result = resolve_path(&target(std::path::Path::new(""), "a", ImageFormat::Png));
    assert_eq!(result, Err(OutputError::NoFolder));
}

#[test]
fn saving_creates_the_folder_and_never_overwrites() {
    let folder = temp_folder("save").join("nested");
    let image = RgbaImage::from_pixel(8, 6, Rgba([10, 200, 30, 255]));
    let first = save_image(&image, &target(&folder, "foto", ImageFormat::Png)).expect("saved");
    let second = save_image(&image, &target(&folder, "foto", ImageFormat::Png)).expect("saved");
    assert_eq!(first.file_name().and_then(|n| n.to_str()), Some("foto.png"));
    assert_eq!(
        second.file_name().and_then(|n| n.to_str()),
        Some("foto_2.png")
    );
    let decoded = image::open(&first).expect("decodes").to_rgba8();
    assert_eq!(decoded, image);
    let _ = std::fs::remove_dir_all(folder.parent().expect("parent"));
}

#[test]
fn jpeg_is_written_with_its_extension_and_decodes() {
    let folder = temp_folder("jpeg");
    let image = RgbaImage::from_pixel(16, 16, Rgba([200, 40, 40, 255]));
    let path = save_image(&image, &target(&folder, "foto.png", ImageFormat::Jpeg)).expect("saved");
    assert_eq!(path.extension().and_then(|e| e.to_str()), Some("jpg"));
    let decoded = image::open(&path).expect("decodes").to_rgb8();
    assert_eq!((decoded.width(), decoded.height()), (16, 16));
    assert!(decoded.get_pixel(8, 8)[0] > 150);
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn a_file_in_place_of_the_folder_is_an_error_not_a_panic() {
    let folder = temp_folder("clash");
    std::fs::create_dir_all(&folder).expect("folder");
    let blocker = folder.join("file");
    std::fs::write(&blocker, b"x").expect("file");
    let image = RgbaImage::new(2, 2);
    let result = save_image(&image, &target(&blocker, "a", ImageFormat::Png));
    assert!(matches!(result, Err(OutputError::BadFolder(_))));
    let _ = std::fs::remove_dir_all(&folder);
}
