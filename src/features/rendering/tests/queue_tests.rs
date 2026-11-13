//! The job queue: sequential execution, progress, cancel, retry, remove and preview tiles.

use super::output_tests::temp_folder;
use super::scenes::cube_on_ground;
use crate::features::rendering::engine::{frame_bounds, RenderRequest, ViewPreset};
use crate::features::rendering::models::{
    ImageFormat, JobInfo, JobStatus, OutputTarget, RayTracingQuality,
};
use crate::features::rendering::services::{JobSpec, RenderQueue};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn spec(folder: &Path, name: &str, size: (u32, u32), quality: RayTracingQuality) -> JobSpec {
    let scene = Arc::new(cube_on_ground());
    let bounds = scene.bounds().expect("bounds");
    let camera = frame_bounds(&bounds, ViewPreset::Corner, size.0, size.1);
    JobSpec {
        name: name.into(),
        preset: "test".into(),
        scene,
        request: RenderRequest::new(camera, quality),
        output: OutputTarget {
            folder: folder.to_path_buf(),
            file_name: name.into(),
            format: ImageFormat::Png,
            jpeg_quality: 0.9,
        },
    }
}

fn job(queue: &RenderQueue, id: u64) -> JobInfo {
    queue
        .snapshot()
        .into_iter()
        .find(|j| j.id == id)
        .expect("job exists")
}

fn wait_for(queue: &RenderQueue, id: u64, done: impl Fn(&JobInfo) -> bool) -> JobInfo {
    let limit = Instant::now() + Duration::from_secs(60);
    loop {
        let info = job(queue, id);
        if done(&info) {
            return info;
        }
        assert!(Instant::now() < limit, "timed out waiting: {info:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn jobs_run_one_after_another_and_write_their_files() {
    let folder = temp_folder("queue");
    let queue = RenderQueue::new();
    let a = queue.enqueue(spec(&folder, "uno", (64, 48), RayTracingQuality::Low));
    let b = queue.enqueue(spec(&folder, "dos", (48, 32), RayTracingQuality::Low));
    let first = wait_for(&queue, a, |j| j.status.is_finished());
    assert_eq!(first.status, JobStatus::Done);
    assert!((first.progress - 1.0).abs() < 1e-6);
    let first_file = first.output.expect("output path");
    assert!(first_file.starts_with(&folder));
    let second = wait_for(&queue, b, |j| j.status.is_finished());
    assert_eq!(second.status, JobStatus::Done);
    let size = image::image_dimensions(second.output.expect("output path")).expect("png");
    assert_eq!(size, (48, 32));
    assert_eq!(image::image_dimensions(first_file).expect("png"), (64, 48));
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn the_second_job_waits_while_the_first_renders_and_can_be_cancelled() {
    let folder = temp_folder("cancel");
    let queue = RenderQueue::new();
    let slow = queue.enqueue(spec(&folder, "lento", (400, 300), RayTracingQuality::Ultra));
    let waiting = queue.enqueue(spec(&folder, "espera", (32, 24), RayTracingQuality::Low));
    wait_for(&queue, slow, |j| j.status == JobStatus::Rendering);
    assert_eq!(job(&queue, waiting).status, JobStatus::Queued);
    assert!(queue.is_busy());
    queue.cancel(slow);
    assert_eq!(job(&queue, slow).status, JobStatus::Cancelled);
    // The worker moves on to the next job.
    let next = wait_for(&queue, waiting, |j| j.status.is_finished());
    assert_eq!(next.status, JobStatus::Done);
    assert!(
        !folder.join("lento.png").exists(),
        "a cancelled job writes no file"
    );
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn a_cancelled_job_can_be_retried_and_removed_jobs_disappear() {
    let folder = temp_folder("retry");
    let queue = RenderQueue::new();
    let slow = queue.enqueue(spec(&folder, "otra", (400, 300), RayTracingQuality::Ultra));
    wait_for(&queue, slow, |j| j.status == JobStatus::Rendering);
    queue.cancel(slow);
    queue.retry(slow);
    assert_ne!(job(&queue, slow).status, JobStatus::Cancelled);
    // Retrying a running or queued job does nothing special; cancel again to stop the heavy render.
    queue.cancel(slow);
    queue.retry(slow);
    queue.cancel(slow);
    wait_for(&queue, slow, |j| j.status == JobStatus::Cancelled);
    queue.remove(slow);
    assert!(queue.snapshot().is_empty());
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn a_failing_output_marks_the_job_failed_and_it_can_be_cleared() {
    let folder = temp_folder("fail");
    std::fs::create_dir_all(&folder).expect("folder");
    let blocker = folder.join("blocker");
    std::fs::write(&blocker, b"x").expect("file");
    let queue = RenderQueue::new();
    let id = queue.enqueue(spec(&blocker, "x", (16, 12), RayTracingQuality::Low));
    let failed = wait_for(&queue, id, |j| j.status.is_finished());
    assert_eq!(failed.status, JobStatus::Failed);
    assert!(failed.error.is_some());
    assert_eq!(queue.clear_finished(), 1);
    assert!(!queue.is_busy());
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn preview_tiles_arrive_for_the_running_job_and_reset_for_the_next() {
    let folder = temp_folder("preview");
    let queue = RenderQueue::new();
    assert!(queue.preview_since(None, 0).is_none());
    let a = queue.enqueue(spec(&folder, "a", (70, 40), RayTracingQuality::Low));
    wait_for(&queue, a, |j| j.status.is_finished());
    let update = queue.preview_since(None, 0).expect("preview of the job");
    assert!(update.reset);
    assert_eq!(update.job, a);
    assert_eq!(update.size, (70, 40));
    assert_eq!(update.tiles.len(), 6);
    assert!(queue.preview_since(Some(a), update.cursor).is_none());
    let b = queue.enqueue(spec(&folder, "b", (32, 32), RayTracingQuality::Low));
    wait_for(&queue, b, |j| j.status.is_finished());
    let next = queue
        .preview_since(Some(a), update.cursor)
        .expect("new job");
    assert!(next.reset);
    assert_eq!(next.job, b);
    let _ = std::fs::remove_dir_all(&folder);
}
