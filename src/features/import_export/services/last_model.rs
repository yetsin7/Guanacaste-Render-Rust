//! Remembers the last imported model so it can be reopened in one click.

use std::path::{Path, PathBuf};

const FILE: &str = "ultimo_modelo.txt";

/// The last model recorded in `dir`, if the note exists and is not empty.
pub fn load(dir: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(dir.join(FILE)).ok()?;
    let line = text.lines().next()?.trim();
    (!line.is_empty()).then(|| PathBuf::from(line))
}

/// Records `model` as the last imported one.
pub fn save(dir: &Path, model: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(FILE), model.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_model_round_trips() {
        let dir = std::env::temp_dir().join(format!("gr-last-model-{}", std::process::id()));
        assert_eq!(load(&dir), None);
        save(&dir, Path::new("C:/modelos/casa.dae")).unwrap();
        assert_eq!(load(&dir), Some(PathBuf::from("C:/modelos/casa.dae")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
