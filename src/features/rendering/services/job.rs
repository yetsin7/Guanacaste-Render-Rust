//! Video export jobs: one background thread per job and a queue that runs them one after another.

use super::ffmpeg;
use super::frame_source::FrameSource;
use super::sequence::{export_sequence, ExportError, ExportSettings};
use crate::features::movie::models::Animation;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// What happened with the optional MP4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoResult {
    /// The user only asked for the frame sequence.
    NotRequested,
    /// A video was requested but ffmpeg is not installed: only the sequence exists.
    FfmpegMissing,
    /// The video file was written.
    Encoded(PathBuf),
    /// ffmpeg ran and failed; the sequence is complete.
    Failed(String),
}

/// A finished export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportReport {
    /// Name used for the files.
    pub name: String,
    /// Folder holding the frames.
    pub folder: PathBuf,
    /// Number of frames written.
    pub frames: u32,
    /// MP4 result.
    pub video: VideoResult,
}

/// How a job ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobEnd {
    /// All frames were written.
    Finished(ExportReport),
    /// A frame or file failed (detail of the cause).
    Failed(String),
    /// The user cancelled.
    Cancelled,
}

/// Everything a job needs; the source moves to the job's thread.
pub struct ExportRequest {
    /// The animation to render (a copy, so later edits do not change a running job).
    pub animation: Animation,
    /// Where the frames come from.
    pub source: Box<dyn FrameSource>,
    /// Output settings.
    pub settings: ExportSettings,
    /// Path of ffmpeg, when installed (see [`ffmpeg::find_ffmpeg`]).
    pub ffmpeg: Option<PathBuf>,
}

#[derive(Default)]
struct Shared {
    done: AtomicU32,
    total: AtomicU32,
    cancel: AtomicBool,
    encoding: AtomicBool,
    result: Mutex<Option<JobEnd>>,
}

/// A running export.
pub struct ExportJob {
    shared: Arc<Shared>,
    name: String,
    handle: Option<JoinHandle<()>>,
}

fn run(mut request: ExportRequest, shared: &Shared) -> JobEnd {
    let settings = request.settings.clone();
    let mut progress = |done: u32, total: u32| {
        shared.done.store(done, Ordering::Relaxed);
        shared.total.store(total, Ordering::Relaxed);
    };
    let frames = match export_sequence(
        &request.animation,
        request.source.as_mut(),
        &settings,
        &shared.cancel,
        &mut progress,
    ) {
        Ok(frames) => frames,
        Err(ExportError::Cancelled) => return JobEnd::Cancelled,
        Err(error) => return JobEnd::Failed(error.to_string()),
    };
    let video = match (&request.ffmpeg, settings.encode_video) {
        (_, false) => VideoResult::NotRequested,
        (None, true) => VideoResult::FfmpegMissing,
        (Some(program), true) => {
            shared.encoding.store(true, Ordering::Relaxed);
            match ffmpeg::encode(
                program,
                &settings.folder,
                &settings.prefix,
                request.animation.fps,
                &shared.cancel,
            ) {
                Ok(path) => VideoResult::Encoded(path),
                Err(None) => return JobEnd::Cancelled,
                Err(Some(detail)) => VideoResult::Failed(detail),
            }
        }
    };
    JobEnd::Finished(ExportReport {
        name: settings.prefix,
        folder: settings.folder,
        frames,
        video,
    })
}

impl ExportJob {
    /// Starts the job on its own thread.
    pub fn start(request: ExportRequest) -> Self {
        let shared = Arc::new(Shared::default());
        shared
            .total
            .store(request.animation.frame_count(), Ordering::Relaxed);
        let name = request.settings.prefix.clone();
        let worker = Arc::clone(&shared);
        let spawned = std::thread::Builder::new()
            .name("exportar-video".into())
            .spawn(move || {
                let end = run(request, &worker);
                if let Ok(mut slot) = worker.result.lock() {
                    *slot = Some(end);
                }
            });
        let handle = match spawned {
            Ok(handle) => Some(handle),
            Err(error) => {
                if let Ok(mut slot) = shared.result.lock() {
                    *slot = Some(JobEnd::Failed(error.to_string()));
                }
                None
            }
        };
        Self {
            shared,
            name,
            handle,
        }
    }

    /// Name used for the files of this job.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Frames written so far and the total.
    pub fn progress(&self) -> (u32, u32) {
        (
            self.shared.done.load(Ordering::Relaxed),
            self.shared.total.load(Ordering::Relaxed),
        )
    }

    /// Whether the frames are done and ffmpeg is encoding.
    pub fn is_encoding(&self) -> bool {
        self.shared.encoding.load(Ordering::Relaxed)
    }

    /// Asks the job to stop as soon as possible.
    pub fn cancel(&self) {
        self.shared.cancel.store(true, Ordering::Relaxed);
    }

    /// The outcome once the thread has finished, `None` while it runs.
    pub fn poll(&mut self) -> Option<JobEnd> {
        let end = self.shared.result.lock().ok()?.take()?;
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        Some(end)
    }
}

/// Jobs waiting their turn, the one running and the outcomes so far.
#[derive(Default)]
pub struct ExportQueue {
    waiting: VecDeque<ExportRequest>,
    current: Option<ExportJob>,
    /// Finished jobs (name, outcome), oldest first.
    pub history: Vec<(String, JobEnd)>,
}

impl ExportQueue {
    /// Adds a job; it starts when the ones before it finish.
    pub fn enqueue(&mut self, request: ExportRequest) {
        self.waiting.push_back(request);
    }

    /// Collects the outcome of the running job and starts the next one. Call it every frame while busy.
    pub fn poll(&mut self) {
        if let Some(job) = &mut self.current {
            if let Some(end) = job.poll() {
                self.history.push((job.name().to_owned(), end));
                self.current = None;
            }
        }
        if self.current.is_none() {
            self.current = self.waiting.pop_front().map(ExportJob::start);
        }
    }

    /// Whether a job runs or waits.
    pub fn is_busy(&self) -> bool {
        self.current.is_some() || !self.waiting.is_empty()
    }

    /// The running job.
    pub fn current(&self) -> Option<&ExportJob> {
        self.current.as_ref()
    }

    /// Number of jobs waiting behind the running one.
    pub fn waiting(&self) -> usize {
        self.waiting.len()
    }

    /// Cancels the running job and drops the waiting ones.
    pub fn cancel_all(&mut self) {
        self.waiting.clear();
        if let Some(job) = &self.current {
            job.cancel();
        }
    }
}
