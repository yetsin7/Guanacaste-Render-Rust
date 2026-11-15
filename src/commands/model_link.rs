//! Live link with Guanacaste 3D. A project made from a Guanacaste 3D model remembers that model's file; when the
//! file changes (the person saved or exported again in Guanacaste 3D) its geometry is read again on a background
//! thread and swapped in. Everything made in the render application — cameras, animation, lighting, landscape,
//! placed objects, environment, filters and edited materials — is kept.
//!
//! When the opened file is an export made by Guanacaste 3D's "Export to render" command, a `<file>.model-link`
//! sidecar names the model it was exported from; the link then follows that model, so saving in Guanacaste 3D
//! is enough and no second export is needed.

use crate::features::import_export::models::{ImportOptions, ImportOutcome};
use crate::features::import_export::services::{job, progress::Progress};
use crate::features::projects::logic::convert;
use crate::features::projects::models::{ModelLinkDoc, Project};
use crate::models::AppState;
use crate::services::localization::trf;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::{Duration, Instant, UNIX_EPOCH};

/// How often the linked file is looked at.
pub const CHECK_EVERY: Duration = Duration::from_millis(1000);
/// Largest sidecar read (it holds a whole baseline model as JSON, gzip-compressed).
const MAX_SIDECAR: u64 = 128 << 20;

/// Writes this executable's path to `application-path.txt` in the data folder, where Guanacaste 3D looks for the
/// render application when the person asks to open a model in it. Failures are ignored: the link is a convenience.
pub fn record_application_path() {
    if let Ok(exe) = std::env::current_exe() {
        let dir = crate::features::projects::services::recovery::data_dir();
        if std::fs::create_dir_all(&dir).is_ok() {
            let _ = std::fs::write(
                dir.join("application-path.txt"),
                exe.to_string_lossy().as_bytes(),
            );
        }
    }
}

/// Last write time (ms since the Unix epoch) and length of `path`, or `None` when it cannot be read.
pub fn stamp(path: &Path) -> Option<(u64, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    Some((modified.as_millis() as u64, meta.len()))
}

/// The model a Guanacaste 3D file stands for: the source named by its `.model-link` sidecar when that file
/// still exists, otherwise the file itself.
pub fn resolve_source(path: &Path) -> PathBuf {
    sidecar_source(path).unwrap_or_else(|| path.to_path_buf())
}

fn sidecar_source(path: &Path) -> Option<PathBuf> {
    let mut sidecar = path.as_os_str().to_owned();
    sidecar.push(".model-link");
    let file = std::fs::File::open(PathBuf::from(sidecar)).ok()?;
    let mut json = Vec::new();
    flate2::read::GzDecoder::new(file.take(MAX_SIDECAR))
        .take(MAX_SIDECAR)
        .read_to_end(&mut json)
        .ok()?;
    let value: serde_json::Value = serde_json::from_slice(&json).ok()?;
    let text = |key: &str| {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
    };
    if let Some(absolute) = text("SourcePath")
        .map(PathBuf::from)
        .filter(|p| p.is_file())
    {
        return Some(absolute);
    }
    let relative = path.parent()?.join(text("RelativeSourcePath")?);
    relative.is_file().then_some(relative)
}

/// A link to the model at `path` (or the source its sidecar names), stamped as read now.
pub fn link_for(path: &Path) -> Option<ModelLinkDoc> {
    let source = resolve_source(path);
    let source = std::fs::canonicalize(&source).unwrap_or(source);
    let (modified_ms, length) = stamp(&source)?;
    Some(ModelLinkDoc {
        path: source
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_owned(),
        modified_ms,
        length,
        offset: [0.0; 3],
    })
}

/// Replaces the model geometry of `project` with `outcome`, keeping the render work. Materials the person edited
/// survive by name; new materials of the model take their imported look.
pub fn adopt_geometry(project: &mut Project, outcome: &ImportOutcome) {
    let rested = super::ground::lowest_z(&project.meshes).is_some_and(|z| z.abs() <= 0.005);
    let doc = &mut project.document;
    let file_name = doc
        .origin
        .as_ref()
        .map(|o| o.file_name.clone())
        .unwrap_or_default();
    let (fresh, meshes) = convert::from_scene(
        &outcome.scene,
        &doc.meta.name,
        &file_name,
        doc.meta.modified,
    );
    let mut materials = fresh.materials;
    for material in &mut materials {
        if let Some(kept) = doc.materials.iter().find(|old| old.name == material.name) {
            *material = kept.clone();
        }
    }
    doc.origin = fresh.origin;
    doc.origin_shift = fresh.origin_shift;
    doc.levels = fresh.levels;
    doc.nodes = fresh.nodes;
    doc.meshes = fresh.meshes;
    doc.materials = materials;
    let offset = doc.model_link.as_ref().map_or([0.0; 3], |l| l.offset);
    project.meshes = meshes;
    for mesh in &mut project.meshes {
        for position in &mut mesh.positions {
            for axis in 0..3 {
                position[axis] += offset[axis];
            }
        }
    }
    // Projects saved before the offset was recorded: a model that stood on the ground keeps standing on it.
    if rested {
        super::ground::settle(project);
    }
    project.dirty = true;
}

/// A geometry read in flight: the stamp of the file it started from and where its result arrives.
struct Reading {
    stamp: (u64, u64),
    result: Receiver<job::JobResult>,
}

/// Watches the linked model of the open project.
#[derive(Default)]
pub struct ModelWatch {
    last_check: Option<Instant>,
    /// A changed stamp seen once; the read starts when the next check sees the same stamp (the writer finished).
    settling: Option<(u64, u64)>,
    reading: Option<Reading>,
}

impl ModelWatch {
    /// Whether the open project follows a Guanacaste 3D model (or a reload is in flight), so frames are needed.
    pub fn is_watching(&self, app: &AppState) -> bool {
        self.reading.is_some()
            || app
                .workspace
                .project
                .as_ref()
                .is_some_and(|p| !p.read_only && p.document.model_link.is_some())
    }

    /// Call once per frame: looks at the linked file every second and swaps in a new geometry when one is read.
    pub fn step(&mut self, app: &mut AppState) {
        self.finish(app);
        let now = Instant::now();
        if self.reading.is_some() || self.last_check.is_some_and(|t| now - t < CHECK_EVERY) {
            return;
        }
        self.last_check = Some(now);
        let Some(link) = app
            .workspace
            .project
            .as_ref()
            .filter(|p| !p.read_only)
            .and_then(|p| p.document.model_link.clone())
        else {
            self.settling = None;
            return;
        };
        let path = PathBuf::from(&link.path);
        let Some(current) = stamp(&path) else {
            return;
        };
        if current == (link.modified_ms, link.length) {
            self.settling = None;
            return;
        }
        if self.settling != Some(current) {
            self.settling = Some(current);
            return;
        }
        self.settling = None;
        let (send, result) = channel();
        std::thread::spawn(move || {
            let _ = send.send(job::run(
                &path,
                &ImportOptions::default(),
                &Progress::default(),
            ));
        });
        self.reading = Some(Reading {
            stamp: current,
            result,
        });
    }

    fn finish(&mut self, app: &mut AppState) {
        let Some(reading) = &self.reading else {
            return;
        };
        let result = match reading.result.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                self.reading = None;
                return;
            }
        };
        let stamp = reading.stamp;
        self.reading = None;
        let Some(project) = app.workspace.project.as_mut() else {
            return;
        };
        let Some(link) = project.document.model_link.as_mut() else {
            return;
        };
        // The stamp is recorded even when the read failed, so a broken file is not read again every second.
        (link.modified_ms, link.length) = stamp;
        let name = Path::new(&link.path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match result {
            Ok(outcome) => {
                adopt_geometry(project, &outcome);
                app.workspace.generation += 1;
                app.composer.invalidate();
                app.materials.project_id = None;
                app.workspace
                    .notify(trf("workspace.model_updated", &[&name]), false);
            }
            Err(error) => {
                project.dirty = true;
                app.workspace.notify(
                    trf("workspace.model_update_failed", &[&name, error.detail()]),
                    true,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::projects::models::CameraDoc;
    use std::io::Write;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gr-link-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn import(path: &Path) -> ImportOutcome {
        job::run(path, &ImportOptions::default(), &Progress::default()).unwrap()
    }

    #[test]
    fn new_geometry_keeps_cameras_and_edited_materials() {
        let dir = temp_dir("adopt");
        let model = dir.join("casa.obj");
        std::fs::write(dir.join("casa.mtl"), "newmtl Madera\nKd 0.5 0.4 0.3\n").unwrap();
        let head = "mtllib casa.mtl\nusemtl Madera\n";
        std::fs::write(
            &model,
            format!("{head}v 0 0 0\nv 1 0 0\nv 1 1 0\nf 1 2 3\n"),
        )
        .unwrap();
        let first = import(&model);
        let (document, meshes) = convert::from_scene(&first.scene, "Casa", "casa.obj", 0);
        let mut project = Project {
            id: [3; 16],
            document,
            meshes,
            path: None,
            legacy_source: None,
            read_only: false,
            dirty: false,
        };
        project.document.cameras.push(CameraDoc {
            name: "Fachada".into(),
            position: [0.0, -10.0, 1.6],
            target: [0.0, 0.0, 1.6],
            focal_mm: 24.0,
            aperture: 8.0,
            filters: None,
            thumbnail: None,
        });
        project.document.materials[0].color = [0.9, 0.1, 0.1, 1.0];
        let triangles_before: usize = project.meshes.iter().map(|m| m.indices.len()).sum();

        std::fs::write(
            &model,
            format!(
                "{head}v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nv 0 0 3\nf 1 2 3\nf 1 3 4\nf 1 2 5\n"
            ),
        )
        .unwrap();
        adopt_geometry(&mut project, &import(&model));

        let triangles_after: usize = project.meshes.iter().map(|m| m.indices.len()).sum();
        assert!(triangles_after > triangles_before, "the new geometry is in");
        assert_eq!(project.document.meshes.len(), project.meshes.len());
        assert_eq!(project.document.cameras.len(), 1, "render cameras survive");
        assert_eq!(project.document.materials[0].color, [0.9, 0.1, 0.1, 1.0]);
        assert!(project.dirty);

        // A model moved in Render (lifted 0.25 m to rest on the ground) lands in the same place after a reload.
        project.document.model_link = Some(ModelLinkDoc::default());
        super::super::ground::shift(&mut project, [0.0, 0.0, 0.25]);
        assert_eq!(
            project.document.model_link.as_ref().unwrap().offset,
            [0.0, 0.0, 0.25]
        );
        adopt_geometry(&mut project, &import(&model));
        assert_eq!(super::super::ground::lowest_z(&project.meshes), Some(0.25));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_sidecar_points_at_the_source_model() {
        let dir = temp_dir("sidecar");
        let export = dir.join("casa-render.g3d");
        let source = dir.join("casa.g3d");
        std::fs::write(&export, b"x").unwrap();
        std::fs::write(&source, b"y").unwrap();
        assert_eq!(
            resolve_source(&export),
            export,
            "no sidecar: the file itself"
        );

        let write_sidecar = |json: &str| {
            let file = std::fs::File::create(dir.join("casa-render.g3d.model-link")).unwrap();
            let mut gz = flate2::write::GzEncoder::new(file, flate2::Compression::fast());
            gz.write_all(json.as_bytes()).unwrap();
            gz.finish().unwrap();
        };
        let absolute = serde_json::json!({ "FormatVersion": 1, "SourcePath": source }).to_string();
        write_sidecar(&absolute);
        assert_eq!(resolve_source(&export), source);

        write_sidecar(r#"{"SourcePath":"Z:/movido/casa.g3d","RelativeSourcePath":"casa.g3d"}"#);
        assert_eq!(
            resolve_source(&export),
            dir.join("casa.g3d"),
            "falls back to the relative path"
        );

        let link = link_for(&export).unwrap();
        assert_eq!(link.length, 1);
        assert!(link.path.ends_with("casa.g3d"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
