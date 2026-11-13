//! Optional MP4 encoding with `ffmpeg`, started without a shell and with the arguments as a list.

use super::sequence::frame_pattern;
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Finds `ffmpeg` in the folders of the `PATH` variable.
pub fn find_ffmpeg() -> Option<PathBuf> {
    find_in(&std::env::var_os("PATH")?)
}

/// Finds `ffmpeg` (or `ffmpeg.exe`) in a `PATH`-style list of folders.
pub fn find_in(path_variable: &OsStr) -> Option<PathBuf> {
    std::env::split_paths(path_variable)
        .flat_map(|folder| [folder.join("ffmpeg"), folder.join("ffmpeg.exe")])
        .find(|candidate| candidate.is_file())
}

/// Arguments that turn the numbered PNG sequence in `folder` into an H.264 MP4 at `output`.
pub fn encode_args(folder: &Path, prefix: &str, fps: u32, output: &Path) -> Vec<OsString> {
    let mut input = folder.as_os_str().to_owned();
    input.push(std::path::MAIN_SEPARATOR_STR);
    input.push(frame_pattern(prefix));
    let mut args: Vec<OsString> = ["-y", "-hide_banner", "-loglevel", "error", "-framerate"]
        .iter()
        .map(OsString::from)
        .collect();
    args.push(fps.to_string().into());
    args.push("-i".into());
    args.push(input);
    for fixed in [
        "-vf",
        "scale=trunc(iw/2)*2:trunc(ih/2)*2",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-movflags",
        "+faststart",
    ] {
        args.push(fixed.into());
    }
    args.push(output.as_os_str().to_owned());
    args
}

/// Runs ffmpeg and waits for it, killing it when `cancel` is set. Returns the video path, or a description of
/// the failure (`None` inside `Err` means the user cancelled).
pub fn encode(
    ffmpeg: &Path,
    folder: &Path,
    prefix: &str,
    fps: u32,
    cancel: &AtomicBool,
) -> Result<PathBuf, Option<String>> {
    let output = folder.join(format!("{prefix}.mp4"));
    let mut child = Command::new(ffmpeg)
        .args(encode_args(folder, prefix, fps, &output))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Some(e.to_string()))?;
    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(None);
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(output),
            Ok(Some(status)) => {
                let mut message = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = stderr.read_to_string(&mut message);
                }
                let message = message.trim();
                return Err(Some(if message.is_empty() {
                    status.to_string()
                } else {
                    message.to_owned()
                }));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(error) => return Err(Some(error.to_string())),
        }
    }
}
