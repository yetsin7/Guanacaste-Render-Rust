//! Video export: PNG sequence, cancellation, the queue and the optional ffmpeg step.

use crate::features::movie::models::{Animation, CameraPose, PathMode};
use crate::features::rendering::services::ffmpeg;
use crate::features::rendering::services::frame_source::{
    ColorFrameSource, FrameRequest, FrameSource,
};
use crate::features::rendering::services::job::{
    ExportJob, ExportQueue, ExportRequest, JobEnd, VideoResult,
};
use crate::features::rendering::services::sequence::{
    clean_prefix, export_sequence, frame_file_name, frame_pattern, ExportError, ExportSettings,
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

fn temp_folder(label: &str) -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "gr-export-{label}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&folder);
    folder
}

/// A one-second animation at 4 fps (5 frames) with the camera moving along X.
fn short_animation() -> Animation {
    let mut animation = Animation {
        fps: 4,
        duration: 1.0,
        ..Animation::default()
    };
    animation.camera.path = PathMode::Linear;
    animation.add_camera_key(
        0.0,
        CameraPose::from_euler([0.0, 0.0, 1.5], 0.0, 0.0, 0.0, 60.0),
    );
    animation.add_camera_key(
        1.0,
        CameraPose::from_euler([20.0, 0.0, 1.5], 0.0, 0.0, 0.0, 60.0),
    );
    let channel = animation.add_channel("hora_del_dia");
    animation.add_value_key(channel, 0.0, 6.0);
    animation.add_value_key(channel, 1.0, 18.0);
    animation
}

fn settings(folder: &Path, encode_video: bool) -> ExportSettings {
    ExportSettings {
        folder: folder.to_path_buf(),
        prefix: "prueba".into(),
        width: 16,
        height: 9,
        encode_video,
    }
}

fn wait(job: &mut ExportJob) -> JobEnd {
    let started = Instant::now();
    loop {
        if let Some(end) = job.poll() {
            return end;
        }
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "export did not finish"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn request(folder: &Path, encode_video: bool, ffmpeg: Option<PathBuf>) -> ExportRequest {
    ExportRequest {
        animation: short_animation(),
        source: Box::new(ColorFrameSource),
        settings: settings(folder, encode_video),
        ffmpeg,
    }
}

#[test]
fn names_and_patterns_are_numbered_from_one() {
    assert_eq!(frame_file_name("a", 0), "a_000001.png");
    assert_eq!(frame_file_name("a", 123), "a_000124.png");
    assert_eq!(frame_pattern("a"), "a_%06d.png");
    assert_eq!(clean_prefix("  mi/ruta:1 "), "mi_ruta_1");
    assert_eq!(clean_prefix("   "), "recorrido");
}

#[test]
fn the_sequence_is_written_with_one_png_per_frame_and_the_camera_shows() {
    let folder = temp_folder("sequence");
    let animation = short_animation();
    let mut seen = Vec::new();
    let result = export_sequence(
        &animation,
        &mut ColorFrameSource,
        &settings(&folder, false),
        &AtomicBool::new(false),
        &mut |done, total| seen.push((done, total)),
    );
    assert_eq!(result, Ok(5));
    assert_eq!(seen.first(), Some(&(0, 5)));
    assert_eq!(seen.last(), Some(&(5, 5)));
    assert!(seen.windows(2).all(|pair| pair[0].0 <= pair[1].0));
    let mut colors = Vec::new();
    for index in 0..5 {
        let path = folder.join(frame_file_name("prueba", index));
        let image = image::open(&path).expect("a decodable PNG").to_rgba8();
        assert_eq!(image.dimensions(), (16, 9));
        colors.push(*image.get_pixel(0, 0));
    }
    colors.dedup();
    assert_eq!(colors.len(), 5, "every frame has its own color");
    assert!(!folder.join(frame_file_name("prueba", 5)).exists());
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn cancelling_stops_before_writing_anything() {
    let folder = temp_folder("cancelled");
    let result = export_sequence(
        &short_animation(),
        &mut ColorFrameSource,
        &settings(&folder, false),
        &AtomicBool::new(true),
        &mut |_, _| {},
    );
    assert_eq!(result, Err(ExportError::Cancelled));
    assert!(!folder.join(frame_file_name("prueba", 0)).exists());
    let _ = std::fs::remove_dir_all(&folder);
}

struct Failing;

impl FrameSource for Failing {
    fn render(&mut self, request: &FrameRequest) -> Result<image::RgbaImage, String> {
        Err(format!("sin memoria en el fotograma {}", request.index))
    }
}

#[test]
fn a_failing_source_reports_the_cause() {
    let folder = temp_folder("failing");
    let result = export_sequence(
        &short_animation(),
        &mut Failing,
        &settings(&folder, false),
        &AtomicBool::new(false),
        &mut |_, _| {},
    );
    assert_eq!(
        result,
        Err(ExportError::Render("sin memoria en el fotograma 0".into()))
    );
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn ffmpeg_is_looked_up_in_the_path_list_and_called_with_a_plain_argument_list() {
    let bin = temp_folder("bin");
    std::fs::create_dir_all(&bin).expect("folder");
    let empty = OsString::from(bin.as_os_str());
    assert_eq!(ffmpeg::find_in(&empty), None);
    std::fs::write(bin.join("ffmpeg"), b"").expect("file");
    assert_eq!(ffmpeg::find_in(&empty), Some(bin.join("ffmpeg")));
    let _ = std::fs::remove_dir_all(&bin);
    let args = ffmpeg::encode_args(
        Path::new("salida"),
        "clip",
        24,
        Path::new("salida/clip.mp4"),
    );
    let args: Vec<String> = args
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let at = args
        .iter()
        .position(|a| a == "-framerate")
        .expect("framerate");
    assert_eq!(args[at + 1], "24");
    assert!(args.iter().any(|a| a.ends_with("clip_%06d.png")));
    assert_eq!(args.last().map(String::as_str), Some("salida/clip.mp4"));
    assert!(args.iter().all(|a| !a.contains('"')), "no shell quoting");
}

#[test]
fn a_job_without_ffmpeg_still_produces_the_sequence_and_says_so() {
    let folder = temp_folder("noffmpeg");
    let mut job = ExportJob::start(request(&folder, true, None));
    match wait(&mut job) {
        JobEnd::Finished(report) => {
            assert_eq!(report.frames, 5);
            assert_eq!(report.video, VideoResult::FfmpegMissing);
        }
        other => panic!("unexpected end {other:?}"),
    }
    assert_eq!(job.progress(), (5, 5));
    assert!(folder.join(frame_file_name("prueba", 4)).exists());
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn a_broken_ffmpeg_does_not_lose_the_sequence() {
    let folder = temp_folder("brokenffmpeg");
    let missing = folder.join("no-existe").join("ffmpeg");
    let mut job = ExportJob::start(request(&folder, true, Some(missing)));
    match wait(&mut job) {
        JobEnd::Finished(report) => assert!(matches!(report.video, VideoResult::Failed(_))),
        other => panic!("unexpected end {other:?}"),
    }
    assert!(folder.join(frame_file_name("prueba", 0)).exists());
    let _ = std::fs::remove_dir_all(&folder);
}

struct Slow;

impl FrameSource for Slow {
    fn render(&mut self, request: &FrameRequest) -> Result<image::RgbaImage, String> {
        std::thread::sleep(Duration::from_millis(40));
        ColorFrameSource.render(request)
    }
}

#[test]
fn a_running_job_can_be_cancelled() {
    let folder = temp_folder("running");
    let mut job = ExportJob::start(ExportRequest {
        source: Box::new(Slow),
        ..request(&folder, false, None)
    });
    std::thread::sleep(Duration::from_millis(60));
    job.cancel();
    assert_eq!(wait(&mut job), JobEnd::Cancelled);
    assert!(!folder.join(frame_file_name("prueba", 4)).exists());
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn the_queue_runs_jobs_one_after_another_and_keeps_the_history() {
    let (first, second) = (temp_folder("queue-a"), temp_folder("queue-b"));
    let mut queue = ExportQueue::default();
    queue.enqueue(request(&first, false, None));
    queue.enqueue(request(&second, false, None));
    assert!(queue.is_busy());
    queue.poll();
    assert_eq!(queue.waiting(), 1);
    let started = Instant::now();
    while queue.is_busy() {
        queue.poll();
        assert!(started.elapsed() < Duration::from_secs(20), "queue stuck");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(queue.history.len(), 2);
    assert!(queue
        .history
        .iter()
        .all(|(_, end)| matches!(end, JobEnd::Finished(_))));
    assert!(first.join(frame_file_name("prueba", 4)).exists());
    assert!(second.join(frame_file_name("prueba", 4)).exists());
    let _ = std::fs::remove_dir_all(&first);
    let _ = std::fs::remove_dir_all(&second);
}
