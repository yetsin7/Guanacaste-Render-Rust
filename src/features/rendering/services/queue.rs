//! The render queue: jobs run one after another on a background thread with progress, cancel and retry.

use super::output::{save_image, OutputError};
use crate::features::rendering::engine::World;
use crate::features::rendering::engine::{
    render_prepared, RenderControl, RenderError, RenderRequest, RenderScene, TileUpdate,
};
use crate::features::rendering::models::{JobInfo, JobStatus, OutputTarget};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// Everything needed to run one job.
#[derive(Debug, Clone)]
pub struct JobSpec {
    /// Name shown in the queue.
    pub name: String,
    /// Quality preset name shown in the queue.
    pub preset: String,
    /// The scene to render.
    pub scene: Arc<World>,
    /// Camera, light and quality.
    pub request: RenderRequest,
    /// Where the picture goes when finished.
    pub output: OutputTarget,
}

struct Entry {
    id: u64,
    spec: JobSpec,
    status: JobStatus,
    control: Arc<RenderControl>,
    progress: f32,
    started: Option<Instant>,
    elapsed: Duration,
    error: Option<String>,
    output: Option<PathBuf>,
}

/// Tiles of the picture being rendered, kept so the interface can paint them as they arrive.
#[derive(Default)]
struct Preview {
    job: Option<u64>,
    size: (u32, u32),
    tiles: Vec<Arc<TileUpdate>>,
}

#[derive(Default)]
struct State {
    jobs: Vec<Entry>,
    next_id: u64,
    shutdown: bool,
    preview: Preview,
    prepared: Option<(Arc<World>, Arc<RenderScene>)>,
}

struct Shared {
    state: Mutex<State>,
    wake: Condvar,
}

/// New tiles of the preview since the caller last looked.
#[derive(Debug, Clone, Default)]
pub struct PreviewUpdate {
    /// Job the picture belongs to.
    pub job: u64,
    /// Size of the whole picture.
    pub size: (u32, u32),
    /// True when the job changed: the caller must clear its picture and apply `tiles` from the start.
    pub reset: bool,
    /// Tiles to paint.
    pub tiles: Vec<Arc<TileUpdate>>,
    /// Pass this back as `cursor` next time.
    pub cursor: usize,
}

/// Handle of the queue and its worker thread. Dropping it cancels the running job and stops the worker.
pub struct RenderQueue {
    shared: Arc<Shared>,
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new()
    }
}

fn lock(shared: &Shared) -> MutexGuard<'_, State> {
    shared.state.lock().unwrap_or_else(|e| e.into_inner())
}

impl RenderQueue {
    /// Starts the worker thread with an empty queue.
    pub fn new() -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            wake: Condvar::new(),
        });
        let worker = Arc::clone(&shared);
        // If the thread cannot start, jobs stay queued forever; the interface still shows them.
        let _ = std::thread::Builder::new()
            .name("render-queue".into())
            .spawn(move || work(&worker));
        Self { shared }
    }

    /// Adds a job at the end of the queue and returns its id.
    pub fn enqueue(&self, spec: JobSpec) -> u64 {
        let mut state = lock(&self.shared);
        let id = state.next_id;
        state.next_id += 1;
        state.jobs.push(Entry {
            id,
            spec,
            status: JobStatus::Queued,
            control: Arc::new(RenderControl::new()),
            progress: 0.0,
            started: None,
            elapsed: Duration::ZERO,
            error: None,
            output: None,
        });
        self.shared.wake.notify_all();
        id
    }

    /// Cancels a queued or running job. Finished jobs are left alone.
    pub fn cancel(&self, id: u64) {
        let mut state = lock(&self.shared);
        if let Some(entry) = state.jobs.iter_mut().find(|e| e.id == id) {
            if !entry.status.is_finished() {
                entry.control.cancel();
                entry.progress = entry.control.progress();
                entry.elapsed = entry.started.map_or(Duration::ZERO, |s| s.elapsed());
                entry.status = JobStatus::Cancelled;
            }
        }
    }

    /// Puts a failed or cancelled job back at its place in the queue.
    pub fn retry(&self, id: u64) {
        let mut state = lock(&self.shared);
        if let Some(entry) = state.jobs.iter_mut().find(|e| e.id == id) {
            if matches!(entry.status, JobStatus::Failed | JobStatus::Cancelled) {
                entry.status = JobStatus::Queued;
                entry.control = Arc::new(RenderControl::new());
                entry.progress = 0.0;
                entry.started = None;
                entry.elapsed = Duration::ZERO;
                entry.error = None;
                entry.output = None;
                self.shared.wake.notify_all();
            }
        }
    }

    /// Removes a job from the list, cancelling it first when it runs.
    pub fn remove(&self, id: u64) {
        let mut state = lock(&self.shared);
        if let Some(entry) = state.jobs.iter().find(|e| e.id == id) {
            entry.control.cancel();
        }
        state.jobs.retain(|e| e.id != id);
    }

    /// Removes every finished job and returns how many were removed.
    pub fn clear_finished(&self) -> usize {
        let mut state = lock(&self.shared);
        let before = state.jobs.len();
        state.jobs.retain(|e| !e.status.is_finished());
        before - state.jobs.len()
    }

    /// A copy of the queue for drawing.
    pub fn snapshot(&self) -> Vec<JobInfo> {
        lock(&self.shared).jobs.iter().map(info).collect()
    }

    /// The preview tiles that arrived after `cursor`; `seen_job` is the job the caller last painted.
    pub fn preview_since(&self, seen_job: Option<u64>, cursor: usize) -> Option<PreviewUpdate> {
        let state = lock(&self.shared);
        let job = state.preview.job?;
        let reset = seen_job != Some(job);
        let from = if reset {
            0
        } else {
            cursor.min(state.preview.tiles.len())
        };
        if !reset && from == state.preview.tiles.len() {
            return None;
        }
        Some(PreviewUpdate {
            job,
            size: state.preview.size,
            reset,
            tiles: state.preview.tiles[from..].to_vec(),
            cursor: state.preview.tiles.len(),
        })
    }

    /// Whether any job is waiting or running.
    pub fn is_busy(&self) -> bool {
        lock(&self.shared)
            .jobs
            .iter()
            .any(|e| !e.status.is_finished())
    }
}

impl Drop for RenderQueue {
    fn drop(&mut self) {
        let mut state = lock(&self.shared);
        state.shutdown = true;
        for entry in &state.jobs {
            entry.control.cancel();
        }
        self.shared.wake.notify_all();
    }
}

fn info(entry: &Entry) -> JobInfo {
    let rendering = entry.status == JobStatus::Rendering;
    let progress = match entry.status {
        JobStatus::Done => 1.0,
        JobStatus::Rendering => entry.control.progress(),
        _ => entry.progress,
    };
    let elapsed = if rendering {
        entry.started.map_or(Duration::ZERO, |s| s.elapsed())
    } else {
        entry.elapsed
    };
    let remaining =
        (rendering && progress > 0.02).then(|| elapsed.mul_f32((1.0 - progress) / progress));
    let camera = &entry.spec.request.camera;
    JobInfo {
        id: entry.id,
        name: entry.spec.name.clone(),
        preset: entry.spec.preset.clone(),
        resolution: format!("{}×{}", camera.width, camera.height),
        status: entry.status,
        progress,
        elapsed,
        remaining,
        error: entry.error.clone(),
        output: entry.output.clone(),
    }
}

/// The worker loop: takes the first queued job, renders it, repeats.
fn work(shared: &Arc<Shared>) {
    loop {
        let (id, spec, control) = {
            let mut state = lock(shared);
            loop {
                if state.shutdown {
                    return;
                }
                if let Some(entry) = state
                    .jobs
                    .iter_mut()
                    .find(|e| e.status == JobStatus::Queued)
                {
                    entry.status = JobStatus::Rendering;
                    entry.started = Some(Instant::now());
                    let job = (entry.id, entry.spec.clone(), Arc::clone(&entry.control));
                    let camera = &job.1.request.camera;
                    state.preview = Preview {
                        job: Some(job.0),
                        size: (camera.width.max(1), camera.height.max(1)),
                        tiles: Vec::new(),
                    };
                    break job;
                }
                state = shared.wake.wait(state).unwrap_or_else(|e| e.into_inner());
            }
        };
        let outcome = catch_unwind(AssertUnwindSafe(|| run(shared, id, &spec, &control)))
            .unwrap_or_else(|_| Err(Failure::Message("panic".into())));
        let mut state = lock(shared);
        if let Some(entry) = state.jobs.iter_mut().find(|e| e.id == id) {
            if entry.status != JobStatus::Rendering {
                continue; // cancelled, removed or retried while it ran
            }
            entry.elapsed = entry.started.map_or(Duration::ZERO, |s| s.elapsed());
            entry.progress = control.progress();
            match outcome {
                Ok(path) => {
                    entry.status = JobStatus::Done;
                    entry.output = Some(path);
                }
                Err(Failure::Cancelled) => entry.status = JobStatus::Cancelled,
                Err(Failure::Message(message)) => {
                    entry.status = JobStatus::Failed;
                    entry.error = Some(message);
                }
            }
        }
    }
}

enum Failure {
    Cancelled,
    Message(String),
}

fn run(
    shared: &Arc<Shared>,
    id: u64,
    spec: &JobSpec,
    control: &Arc<RenderControl>,
) -> Result<PathBuf, Failure> {
    let prepared = {
        let cached = lock(shared)
            .prepared
            .as_ref()
            .filter(|(scene, _)| Arc::ptr_eq(scene, &spec.scene))
            .map(|(_, prepared)| Arc::clone(prepared));
        match cached {
            Some(prepared) => prepared,
            None => {
                let prepared = Arc::new(RenderScene::new(&spec.scene));
                lock(shared).prepared = Some((Arc::clone(&spec.scene), Arc::clone(&prepared)));
                prepared
            }
        }
    };
    if control.is_cancelled() {
        return Err(Failure::Cancelled);
    }
    let on_tile = |tile: &TileUpdate| {
        let mut state = lock(shared);
        if state.preview.job == Some(id) {
            state.preview.tiles.push(Arc::new(tile.clone()));
        }
    };
    let image =
        render_prepared(&prepared, &spec.request, control, &on_tile).map_err(|e| match e {
            RenderError::Cancelled => Failure::Cancelled,
            RenderError::EmptyImage => Failure::Message("empty".into()),
        })?;
    save_image(&image, &spec.output).map_err(|e| {
        Failure::Message(match e {
            OutputError::NoFolder => "folder".into(),
            OutputError::BadFolder(m) | OutputError::Write(m) => m,
        })
    })
}
