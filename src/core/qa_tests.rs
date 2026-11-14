//! QA of the whole workspace driven like a user: synthetic pointer and key events go through egui into the real
//! screens, so placing, selecting, deleting, undoing, saving cameras, rendering photos and a panorama and saving
//! and reopening are exercised end to end (without a window; the 3D view only paints callbacks).

use super::shortcuts;
use crate::commands::{camera_link, sync};
use crate::config::theme;
use crate::features::import_export::models::ImportOptions;
use crate::features::import_export::services::{obj, progress::Progress};
use crate::features::library::models::{AssetRef, ProceduralKind};
use crate::features::rendering::models::JobStatus;
use crate::features::viewport::models::Camera;
use crate::models::navigation::{BuildCategory, Mode, PhotoSection, Screen};
use crate::models::AppState;
use crate::screens::Screens;
use eframe::egui::{self, pos2, vec2, Event, Key, Modifiers, PointerButton, RawInput, Rect};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const CUBE_OBJ: &str = "v -1 0 -1\nv 1 0 -1\nv 1 0 1\nv -1 0 1\nv -1 2 -1\nv 1 2 -1\nv 1 2 1\nv -1 2 1\nf 1 2 3 4\nf 5 6 7 8\nf 1 2 6 5\nf 2 3 7 6\nf 3 4 8 7\nf 4 1 5 8\n";

struct Harness {
    ctx: egui::Context,
    screens: Screens,
    app: AppState,
    screen: Screen,
    time: f64,
    folder: PathBuf,
    last_repaint_delay: Duration,
}

impl Harness {
    fn new(label: &str) -> Self {
        let folder = std::env::temp_dir().join(format!(
            "gr-qa-{label}-{}",
            crate::features::projects::services::util::hex(
                &crate::features::projects::services::util::new_document_id()
            )
        ));
        std::fs::create_dir_all(&folder).unwrap();
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        let mut app = AppState::with_data_dir(folder.join("datos"));
        let outcome = obj::import_reader(
            CUBE_OBJ.as_bytes(),
            CUBE_OBJ.len() as u64,
            &ImportOptions::default(),
            &Progress::default(),
            &|_| None,
        )
        .unwrap();
        app.workspace
            .adopt_import(&outcome, &folder.join("Cubo.obj"));
        app.viewport.gpu_available = true;
        app.studio.state.output = folder.join("fotos").display().to_string();
        app.studio.state.resolution = 0;
        let mut harness = Self {
            ctx,
            screens: Screens::default(),
            app,
            screen: Screen::Workspace(Mode::Build),
            time: 0.0,
            folder,
            last_repaint_delay: Duration::ZERO,
        };
        harness.frame(vec![]);
        harness
    }

    /// One frame of the shell with `events`, like `StudioApp::ui`.
    fn frame(&mut self, events: Vec<Event>) {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0))),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        self.time += 0.02;
        let Self {
            ctx,
            screens,
            app,
            screen,
            ..
        } = self;
        let mut output = ctx.run_ui(input, |ui| {
            sync::step(app);
            let source = app.composer.compose(
                app.workspace.project.as_ref(),
                &app.landscape,
                &mut app.objects,
            );
            app.viewport.sync(source);
            egui::CentralPanel::default().show(ui, |ui| screens.show(ui, *screen, app));
            shortcuts::handle(ui.ctx(), app, *screen, false);
            let editing = ui.input(|i| i.pointer.any_down());
            crate::commands::undo::track(app, editing);
        });
        output.textures_delta.clear();
        self.last_repaint_delay = output
            .viewport_output
            .values()
            .map(|v| v.repaint_delay)
            .min()
            .unwrap_or(Duration::MAX);
        if let Some(target) = app.requests.navigate.take() {
            *screen = target;
            if let Some(mode) = target.mode() {
                app.mode = mode;
            }
        }
    }

    fn click(&mut self, at: egui::Pos2, modifiers: Modifiers) {
        self.frame(vec![Event::PointerMoved(at)]);
        let button = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers,
        };
        self.frame(vec![button(true)]);
        self.frame(vec![button(false)]);
        self.frame(vec![]);
    }

    fn key(&mut self, key: Key, modifiers: Modifiers) {
        let event = |pressed| Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers,
        };
        self.frame(vec![event(true)]);
        self.frame(vec![event(false)]);
    }

    /// Waits until the queue has no unfinished job.
    fn wait_for_renders(&mut self) {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(120) {
            self.frame(vec![]);
            let jobs = self.app.render.queue.snapshot();
            if !jobs.is_empty() && jobs.iter().all(|job| job.status.is_finished()) {
                return;
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        panic!("the render queue never finished");
    }

    fn center(&self) -> egui::Pos2 {
        pos2(640.0, 400.0)
    }

    fn look_at_origin(&mut self) {
        self.app.viewport.controller.go_to(
            Camera::looking_at([0.0, -30.0, 25.0], [0.0, 0.0, 3.0]),
            false,
        );
        self.frame(vec![]);
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.folder);
    }
}

fn raise_ground(h: &mut Harness, height: f32) {
    for value in h.app.landscape.landscape.terrain.heights.iter_mut() {
        *value = height;
    }
    h.app.landscape.touch();
    h.frame(vec![]);
}

#[test]
fn clicking_places_an_object_on_the_relief_then_select_delete_and_undo_work() {
    let mut h = Harness::new("objects");
    h.app.category = BuildCategory::Objects;
    h.look_at_origin();
    raise_ground(&mut h, 3.0);
    h.app.objects.browser.active = Some(AssetRef::Procedural {
        kind: ProceduralKind::Cube,
        seed: 0,
    });
    h.app.objects.placement.snap_to_surface = false;
    h.app.objects.placement.grid = 0.0;
    h.click(h.center(), Modifiers::NONE);
    assert_eq!(h.app.objects.instances().len(), 1, "a click places");
    let z = h.app.objects.instances()[0].transform.position[2];
    assert!((z - 3.0).abs() < 0.15, "rests on the sculpted ground: {z}");
    assert!(h.app.workspace.dirty());

    // Without an active asset a click selects.
    h.app.objects.browser.active = None;
    h.app.objects.clear_selection();
    h.look_at_origin();
    h.click(h.center(), Modifiers::NONE);
    assert_eq!(h.app.objects.selection.len(), 1, "a click selects");

    // Delete removes it and Ctrl+Z brings it back, Ctrl+Y removes it again.
    h.key(Key::Delete, Modifiers::NONE);
    assert!(h.app.objects.instances().is_empty());
    h.key(Key::Z, Modifiers::COMMAND);
    assert_eq!(h.app.objects.instances().len(), 1, "undo restores it");
    h.key(Key::Y, Modifiers::COMMAND);
    assert!(
        h.app.objects.instances().is_empty(),
        "redo removes it again"
    );
}

#[test]
fn photos_of_every_saved_camera_render_with_their_own_filters_and_a_panorama_follows() {
    let mut h = Harness::new("render");
    h.screen = Screen::Workspace(Mode::Photo);
    h.app.mode = Mode::Photo;
    h.look_at_origin();
    h.app.studio.state.resolution = 0;
    camera_link::save_current(&mut h.app);
    h.app.viewport.controller.go_to(
        Camera::looking_at([20.0, -20.0, 10.0], [0.0, 0.0, 1.0]),
        false,
    );
    h.frame(vec![]);
    h.app.camera_studio.stack =
        crate::features::camera::models::preset::FilterPreset::BlackWhite.stack();
    camera_link::save_current(&mut h.app);
    h.frame(vec![]);
    assert_eq!(h.app.cameras.len(), 2);
    assert!(h.app.cameras[0].filters.is_some() && h.app.cameras[1].filters.is_some());
    h.app.studio.state.quality = 0;

    crate::screens::workspace::render_photos(&mut h.app);
    h.wait_for_renders();
    let jobs = h.app.render.queue.snapshot();
    assert_eq!(jobs.len(), 2, "one job per saved camera");
    assert!(
        jobs.iter().all(|job| job.status == JobStatus::Done),
        "{jobs:?}"
    );
    for job in &jobs {
        let path = job.output.clone().expect("a file");
        let picture = image::open(&path).expect("a readable picture").to_rgba8();
        assert_eq!((picture.width(), picture.height()), (640, 360));
    }

    // The panorama tab: 360 x 180, saved as PNG, and the viewer opens on it.
    h.app.photo_section = PhotoSection::Panorama;
    h.app.render.queue.clear_finished();
    let camera = *h.app.viewport.camera();
    h.app
        .render
        .enqueue_panorama(
            &h.app.workspace,
            &crate::features::rendering::logic::plan::plan_from_studio(&h.app.studio.state),
            &camera,
            256,
        )
        .expect("queued");
    h.wait_for_renders();
    let job = h.app.render.queue.snapshot().pop().expect("a job");
    assert_eq!(job.status, JobStatus::Done);
    let picture = image::open(job.output.expect("file")).unwrap();
    assert_eq!((picture.width(), picture.height()), (256, 128));
}

#[test]
fn mode_keys_switch_modes_and_the_whole_project_survives_save_and_open() {
    let mut h = Harness::new("modes");
    for (key, mode) in [
        (Key::F2, Mode::Photo),
        (Key::F3, Mode::Movie),
        (Key::F4, Mode::Export),
        (Key::F1, Mode::Build),
    ] {
        h.key(key, Modifiers::NONE);
        assert_eq!(h.screen, Screen::Workspace(mode), "{key:?}");
        h.frame(vec![]);
    }
    h.app.environment.hour = 6.5;
    h.app
        .camera_studio
        .stack
        .push(crate::features::camera::models::filter::FilterKind::ALL[0]);
    camera_link::save_current(&mut h.app);
    h.frame(vec![]);
    let file = h.folder.join("Cubo.gr");
    assert!(h.app.workspace.save_to(&file, false));
    let mut other = AppState::with_data_dir(h.folder.join("otros"));
    assert!(other.workspace.open_file(&file));
    sync::step(&mut other);
    assert_eq!(other.environment.hour, 6.5);
    assert_eq!(other.cameras.len(), 1);
    assert_eq!(other.camera_studio.stack, h.app.camera_studio.stack);
    assert!(!other.workspace.project.as_ref().unwrap().meshes.is_empty());
}

#[test]
fn every_screen_and_every_photo_section_draws_without_panicking() {
    let mut h = Harness::new("screens");
    for screen in Screen::ALL {
        h.screen = screen;
        h.frame(vec![]);
        h.frame(vec![]);
    }
    h.screen = Screen::Workspace(Mode::Photo);
    for section in PhotoSection::ALL {
        h.app.photo_section = section;
        h.frame(vec![]);
    }
    h.screen = Screen::Workspace(Mode::Build);
    for category in BuildCategory::ALL {
        h.app.category = category;
        h.frame(vec![]);
    }
}

#[test]
fn an_idle_workspace_does_not_ask_to_repaint_continuously() {
    let mut h = Harness::new("idle");
    for mode in Mode::ALL {
        h.screen = Screen::Workspace(mode);
        h.app.mode = mode;
        for _ in 0..30 {
            h.frame(vec![]);
        }
        let delay = h.last_repaint_delay;
        assert!(
            delay > Duration::from_millis(15),
            "{mode:?} repaints every frame while idle ({delay:?})"
        );
    }
}
