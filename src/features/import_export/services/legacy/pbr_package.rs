//! Portable Guanacaste 3D PBR packages: validate local map paths before loading independent material channels.

use crate::features::materials::models::{Material, TextureRef};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Finds a manifest field without depending on its JSON property casing.
fn field<'a>(document: &'a Value, name: &str) -> Option<&'a Value> {
    document
        .as_object()?
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value)
}

/// Resolves an image inside the package directory, rejecting absolute paths and symlink escapes.
fn image(directory: &Path, name: &str) -> Option<TextureRef> {
    let relative = Path::new(name);
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return None;
    }
    let extension = relative.extension()?.to_str()?.to_ascii_lowercase();
    if !matches!(extension.as_str(), "png" | "jpg" | "jpeg") {
        return None;
    }
    let root = directory.canonicalize().ok()?;
    let resolved = root.join(relative).canonicalize().ok()?;
    if !resolved.starts_with(&root) || !resolved.is_file() {
        return None;
    }
    Some(TextureRef::file(resolved.to_string_lossy().into_owned()))
}

/// Loads supported PBR maps and scalar properties without changing the package or project.
pub fn read(assignment: &str, source_folder: Option<&Path>) -> Option<Material> {
    let raw = assignment.strip_prefix("pbr:")?;
    let location = Path::new(raw);
    let manifest: PathBuf = if location.is_absolute() {
        location.to_owned()
    } else {
        source_folder?.join(location)
    };
    if !manifest.extension()?.to_str()?.eq_ignore_ascii_case("g3dm")
        || std::fs::metadata(&manifest).ok()?.len() > 65_536
    {
        return None;
    }
    let document: Value = serde_json::from_slice(&std::fs::read(&manifest).ok()?).ok()?;
    if field(&document, "version")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        != 1
    {
        return None;
    }
    let folder = manifest.parent()?;
    let map = |name: &str| {
        field(&document, name)
            .and_then(Value::as_str)
            .and_then(|name| image(folder, name))
    };
    let albedo = map("albedo")?;
    let number = |name: &str, fallback: f32| {
        field(&document, name)
            .and_then(Value::as_f64)
            .map_or(fallback, |value| value as f32)
    };
    let mut material = Material::new(
        field(&document, "name")
            .and_then(Value::as_str)
            .unwrap_or("PBR"),
    );
    material.base_color = [1.0; 4];
    material.maps.albedo = Some(albedo);
    material.maps.normal = map("normal").or_else(|| {
        map("heightMap").map(|mut texture| {
            texture.is_height = true;
            texture
        })
    });
    material.maps.roughness = map("roughnessMap");
    material.maps.metallic = map("metallicMap");
    // Guanacaste 3D treats scalar maps as absolute values; Render multiplies maps by these factors.
    material.roughness = if material.maps.roughness.is_some() {
        1.0
    } else {
        number("roughness", 0.7)
    };
    material.metallic = if material.maps.metallic.is_some() {
        1.0
    } else {
        number("metallic", 0.0)
    };
    material.ior = number("ior", 1.5);
    let size = number("textureSize", 1.0).clamp(0.01, 1000.0);
    material.uv.scale = [1.0 / size; 2];
    material.normal_strength = number("relief", 0.2).clamp(0.0, 1.0);
    Some(material.clamped())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent PBR channels survive import, and a map cannot escape the material package.
    #[test]
    fn package_loads_maps_and_rejects_escaping_paths() {
        let directory = std::env::temp_dir().join("guanacaste-pbr-package-test");
        std::fs::create_dir_all(&directory).unwrap();
        for name in ["color.png", "normal.png", "rough.jpg"] {
            std::fs::write(directory.join(name), b"fixture").unwrap();
        }
        let manifest = directory.join("surface.g3dm");
        std::fs::write(&manifest, r#"{"version":1,"name":"Stone","albedo":"color.png","normal":"normal.png","roughnessMap":"rough.jpg","metallicMap":"../outside.png","roughness":0.8,"textureSize":2.0}"#).unwrap();
        let material = read("pbr:surface.g3dm", Some(&directory)).unwrap();
        assert!(material.maps.albedo.is_some());
        assert!(material.maps.normal.is_some());
        assert!(material.maps.roughness.is_some());
        assert!(material.maps.metallic.is_none());
        assert_eq!(material.uv.scale, [0.5, 0.5]);
        std::fs::write(&manifest, r#"{"version":1,"albedo":"../outside.png"}"#).unwrap();
        assert!(read("pbr:surface.g3dm", Some(&directory)).is_none());
    }
}
