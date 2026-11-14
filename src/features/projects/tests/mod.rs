//! Unit tests of the projects domain: repository filters, the `.gr` container, saving, recovery and the
//! read-only Guanacaste 3D path.

mod animation_tests;
mod container_tests;
mod legacy_tests;
mod plugin_fixture_tests;
mod revit_tests;
mod store_tests;

/// A fresh empty folder under the system temporary directory, unique per call.
pub(super) fn temp_folder(label: &str) -> std::path::PathBuf {
    let id = crate::features::projects::services::util::hex(
        &crate::features::projects::services::util::new_document_id(),
    );
    let folder = std::env::temp_dir().join(format!("gr-test-{label}-{id}"));
    std::fs::create_dir_all(&folder).expect("temp folder");
    folder
}
