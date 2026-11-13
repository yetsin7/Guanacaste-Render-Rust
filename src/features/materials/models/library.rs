//! The list of materials of a project or session, with unique names.

use super::material::Material;
use crate::features::materials::logic::presets;
use crate::services::localization::trf;

/// An ordered list of materials. Indices are stable identifiers (meshes refer to materials by index), so removing
/// one shifts the following ones; callers that keep indices must remap them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MaterialLibrary {
    materials: Vec<Material>,
}

impl MaterialLibrary {
    /// An empty library.
    pub fn new() -> Self {
        Self::default()
    }

    /// A library holding the built-in architectural presets.
    pub fn with_presets() -> Self {
        Self::from_materials(presets::all())
    }

    /// Builds a library from `materials`, clamping every value and making names unique and non-empty. The order
    /// and count are kept.
    pub fn from_materials(materials: Vec<Material>) -> Self {
        let mut library = Self::new();
        for material in materials {
            let mut material = material.clamped();
            material.name = library.unique_name(&material.name, None);
            library.materials.push(material);
        }
        library
    }

    /// The materials, in order.
    pub fn as_slice(&self) -> &[Material] {
        &self.materials
    }

    /// Iterates the materials.
    pub fn iter(&self) -> std::slice::Iter<'_, Material> {
        self.materials.iter()
    }

    /// Number of materials.
    pub fn len(&self) -> usize {
        self.materials.len()
    }

    /// Whether there are no materials.
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }

    /// The material at `index`.
    pub fn get(&self, index: usize) -> Option<&Material> {
        self.materials.get(index)
    }

    /// Mutable access to the material at `index`. Rename through [`rename`](Self::rename) to keep names unique.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Material> {
        self.materials.get_mut(index)
    }

    /// Index of the material called `name`.
    pub fn find(&self, name: &str) -> Option<usize> {
        self.materials.iter().position(|m| m.name == name)
    }

    /// `name` (or "Material" when blank) made unique by appending " 2", " 3"...; `skip` is an index whose own
    /// name does not count as taken.
    pub fn unique_name(&self, name: &str, skip: Option<usize>) -> String {
        let base = match name.trim() {
            "" => "Material",
            trimmed => trimmed,
        };
        let taken = |candidate: &str| {
            self.materials
                .iter()
                .enumerate()
                .any(|(i, m)| Some(i) != skip && m.name == candidate)
        };
        let mut candidate = base.to_owned();
        let mut counter = 2;
        while taken(&candidate) {
            candidate = format!("{base} {counter}");
            counter += 1;
        }
        candidate
    }

    /// Appends `material` (clamped, with a unique name) and returns its index.
    pub fn push(&mut self, material: Material) -> usize {
        let mut material = material.clamped();
        material.name = self.unique_name(&material.name, None);
        self.materials.push(material);
        self.materials.len() - 1
    }

    /// Appends a copy of the material at `index` (so no existing index moves) and returns the copy's index.
    pub fn duplicate(&mut self, index: usize) -> Option<usize> {
        let mut copy = self.materials.get(index)?.clone();
        copy.name = trf("material.copy_name", &[&copy.name]);
        Some(self.push(copy))
    }

    /// Renames the material at `index`; a name already used by another material gets a numeric suffix. Returns
    /// false when `index` is invalid or the name is blank.
    pub fn rename(&mut self, index: usize, name: &str) -> bool {
        if index >= self.materials.len() || name.trim().is_empty() {
            return false;
        }
        let unique = self.unique_name(name, Some(index));
        self.materials[index].name = unique;
        true
    }

    /// Removes and returns the material at `index`; later materials shift down by one.
    pub fn remove(&mut self, index: usize) -> Option<Material> {
        (index < self.materials.len()).then(|| self.materials.remove(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library(names: &[&str]) -> MaterialLibrary {
        MaterialLibrary::from_materials(names.iter().map(|n| Material::new(*n)).collect())
    }

    #[test]
    fn names_are_made_unique_on_load_and_push() {
        let mut lib = library(&["A", "A", "A", ""]);
        let names: Vec<_> = lib.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, ["A", "A 2", "A 3", "Material"]);
        let index = lib.push(Material::new("A"));
        assert_eq!(lib.get(index).unwrap().name, "A 4");
    }

    #[test]
    fn rename_keeps_names_unique_and_rejects_blanks() {
        let mut lib = library(&["A", "B"]);
        assert!(lib.rename(1, "A"));
        assert_eq!(lib.get(1).unwrap().name, "A 2");
        assert!(lib.rename(0, "A"), "renaming to its own name is fine");
        assert_eq!(lib.get(0).unwrap().name, "A");
        assert!(!lib.rename(0, "   "));
        assert!(!lib.rename(9, "Z"));
    }

    #[test]
    fn duplicate_appends_a_named_copy_without_moving_others() {
        let mut lib = library(&["Concreto", "Vidrio"]);
        lib.get_mut(0).unwrap().roughness = 0.9;
        let copy = lib.duplicate(0).expect("copy");
        assert_eq!(copy, 2);
        assert_eq!(lib.get(1).unwrap().name, "Vidrio");
        assert_eq!(lib.get(2).unwrap().name, "Concreto (copia)");
        assert_eq!(lib.get(2).unwrap().roughness, 0.9);
        let again = lib.duplicate(0).unwrap();
        assert_eq!(lib.get(again).unwrap().name, "Concreto (copia) 2");
        assert_eq!(lib.len(), 4);
        assert!(lib.duplicate(99).is_none());
    }

    #[test]
    fn remove_shifts_and_reports_missing_indices() {
        let mut lib = library(&["A", "B", "C"]);
        assert_eq!(lib.remove(1).unwrap().name, "B");
        assert_eq!(lib.find("C"), Some(1));
        assert!(lib.remove(5).is_none());
        assert_eq!(lib.len(), 2);
    }

    #[test]
    fn loading_clamps_damaged_values() {
        let mut bad = Material::new("Malo");
        bad.roughness = f32::NAN;
        bad.metallic = 7.0;
        let lib = MaterialLibrary::from_materials(vec![bad]);
        let m = lib.get(0).unwrap();
        assert_eq!(m.roughness, Material::default().roughness);
        assert_eq!(m.metallic, 1.0);
    }
}
