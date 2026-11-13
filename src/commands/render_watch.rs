//! Use case: follows the render queue for the interface: the progress history of the running job (for the
//! performance card) and the notice that a render has just been completed.

use crate::features::rendering::models::{JobInfo, JobStatus};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Most progress samples kept.
const MAX_SAMPLES: usize = 90;
/// Seconds between two samples.
const SAMPLE_EVERY: Duration = Duration::from_millis(500);
/// How long the completion notice stays.
pub const NOTICE_TIME: Duration = Duration::from_secs(14);

/// A render that has just finished.
#[derive(Debug, Clone, PartialEq)]
pub struct Completed {
    /// The picture that was written, when known.
    pub output: Option<PathBuf>,
    /// When the notice was raised.
    pub at: Instant,
}

/// What the interface remembers about the queue.
#[derive(Default)]
pub struct RenderWatch {
    /// Progress (0..=1) of the running job, sampled over time.
    pub history: Vec<f32>,
    running: Option<u64>,
    last_sample: Option<Instant>,
    seen_done: HashSet<u64>,
    primed: bool,
    /// The latest completion, until it is dismissed or expires.
    pub completed: Option<Completed>,
}

impl RenderWatch {
    /// Takes a snapshot of the queue; call once per frame.
    pub fn update(&mut self, jobs: &[JobInfo]) {
        let running = jobs.iter().find(|job| job.status == JobStatus::Rendering);
        if running.map(|job| job.id) != self.running {
            self.running = running.map(|job| job.id);
            self.history.clear();
            self.last_sample = None;
        }
        if let Some(job) = running {
            if self
                .last_sample
                .is_none_or(|at| at.elapsed() >= SAMPLE_EVERY)
            {
                self.history.push(job.progress);
                if self.history.len() > MAX_SAMPLES {
                    self.history.remove(0);
                }
                self.last_sample = Some(Instant::now());
            }
        }
        for job in jobs.iter().filter(|job| job.status == JobStatus::Done) {
            if self.seen_done.insert(job.id) && self.primed {
                self.completed = Some(Completed {
                    output: job.output.clone(),
                    at: Instant::now(),
                });
            }
        }
        self.primed = true;
        if self
            .completed
            .as_ref()
            .is_some_and(|done| done.at.elapsed() > NOTICE_TIME)
        {
            self.completed = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(id: u64, status: JobStatus, progress: f32) -> JobInfo {
        JobInfo {
            id,
            name: "Vista".into(),
            preset: "Media".into(),
            resolution: "1280×720".into(),
            status,
            progress,
            elapsed: Duration::ZERO,
            remaining: None,
            error: None,
            output: Some(PathBuf::from("foto.png")),
        }
    }

    #[test]
    fn a_new_completion_raises_the_notice_once_and_old_ones_do_not() {
        let mut watch = RenderWatch::default();
        watch.update(&[job(1, JobStatus::Done, 1.0)]);
        assert!(
            watch.completed.is_none(),
            "jobs finished before we looked are not news"
        );
        watch.update(&[
            job(1, JobStatus::Done, 1.0),
            job(2, JobStatus::Rendering, 0.4),
        ]);
        assert!(watch.completed.is_none());
        assert_eq!(watch.history, vec![0.4]);
        watch.update(&[job(1, JobStatus::Done, 1.0), job(2, JobStatus::Done, 1.0)]);
        assert_eq!(
            watch.completed.as_ref().and_then(|c| c.output.clone()),
            Some(PathBuf::from("foto.png"))
        );
        watch.completed = None;
        watch.update(&[job(1, JobStatus::Done, 1.0), job(2, JobStatus::Done, 1.0)]);
        assert!(watch.completed.is_none());
    }
}
