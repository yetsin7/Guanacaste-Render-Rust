//! Headless frame-time benchmark of the 3D view: the same pipelines, buffers and frustum culling as the window,
//! drawn into an offscreen target with no display. Used by `examples/viewport_bench.rs` to measure real FPS.

use super::globals::Globals;
use super::gpu::ViewportGpu;
use super::upload::SceneUpload;
use crate::features::viewport::models::{Camera, ViewOptions};
use eframe::wgpu;
use std::future::Future;
use std::task::{Context, Poll, Waker};
use std::time::Instant;

/// Frame times of one camera position.
#[derive(Debug, Clone, PartialEq)]
pub struct ShotTiming {
    /// Name given to the camera.
    pub name: String,
    /// Chunks drawn after frustum culling, and the total number of chunks.
    pub chunks: (usize, usize),
    /// Triangles submitted after culling.
    pub triangles: usize,
    /// Milliseconds per frame: average, 95th percentile, worst.
    pub ms: (f64, f64, f64),
}

impl ShotTiming {
    /// Frames per second implied by the average frame time.
    pub fn fps(&self) -> f64 {
        1000.0 / self.ms.0.max(1e-6)
    }
}

fn block_on<T>(future: impl Future<Output = T>) -> T {
    let mut future = Box::pin(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// Opens the default adapter without a window; returns its name with the device and queue.
fn open_device() -> Option<(String, wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))
    .ok()?;
    let name = adapter.get_info().name;
    let (device, queue) =
        block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
    Some((name, device, queue))
}

/// Renders one frame of `scene` from each camera and returns the pictures as tightly packed RGBA8 (`size` pixels
/// each), so the look of the view (sky, horizon, haze) can be checked without a window.
pub fn capture(
    scene: &SceneUpload,
    cameras: &[Camera],
    size: [u32; 2],
    scene_radius: f32,
) -> Option<Vec<Vec<u8>>> {
    let (_, device, queue) = open_device()?;
    let mut gpu = ViewportGpu::headless(&device);
    let aspect = size[0] as f32 / size[1].max(1) as f32;
    let row = (size[0] * 4).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("viewport-capture"),
        size: u64::from(row) * u64::from(size[1]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut pictures = Vec::new();
    for camera in cameras {
        let globals = Globals::new(camera, aspect, scene_radius);
        gpu.prepare(&device, &queue, size, &globals, scene);
        let mut encoder = device.create_command_encoder(&Default::default());
        gpu.render(&mut encoder, &globals.view_proj, ViewOptions::default());
        encoder.copy_texture_to_buffer(
            gpu.resolved()?.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(size[1]),
                },
            },
            wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);
        buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let mut picture = Vec::with_capacity((size[0] * size[1] * 4) as usize);
        {
            let data = buffer.slice(..).get_mapped_range().ok()?;
            for line in 0..size[1] as usize {
                let start = line * row as usize;
                picture.extend_from_slice(&data[start..start + (size[0] * 4) as usize]);
            }
        }
        buffer.unmap();
        pictures.push(picture);
    }
    Some(pictures)
}

/// Renders `frames` frames from each camera into a `size` target and returns the adapter name and the timings.
/// `None` when no GPU is available. Every frame is submitted and waited for, so the time includes the GPU work.
pub fn run(
    scene: &SceneUpload,
    cameras: &[(String, Camera)],
    size: [u32; 2],
    frames: usize,
    scene_radius: f32,
) -> Option<(String, Vec<ShotTiming>)> {
    let (adapter, device, queue) = open_device()?;
    let mut gpu = ViewportGpu::headless(&device);
    let aspect = size[0] as f32 / size[1].max(1) as f32;
    let mut timings = Vec::new();
    for (name, camera) in cameras {
        let globals = Globals::new(camera, aspect, scene_radius);
        let mut samples = Vec::with_capacity(frames);
        for frame in 0..frames + 3 {
            let started = Instant::now();
            gpu.prepare(&device, &queue, size, &globals, scene);
            let mut encoder = device.create_command_encoder(&Default::default());
            gpu.render(&mut encoder, &globals.view_proj, ViewOptions::default());
            queue.submit([encoder.finish()]);
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            if frame >= 3 {
                samples.push(started.elapsed().as_secs_f64() * 1000.0);
            }
        }
        samples.sort_by(f64::total_cmp);
        let average = samples.iter().sum::<f64>() / samples.len().max(1) as f64;
        let p95 = samples[((samples.len() * 95) / 100).min(samples.len().saturating_sub(1))];
        let worst = samples.last().copied().unwrap_or(0.0);
        let (drawn, triangles) = gpu.visible(&globals.view_proj);
        timings.push(ShotTiming {
            name: name.clone(),
            chunks: (drawn, gpu.chunk_count()),
            triangles,
            ms: (average, p95, worst),
        });
    }
    Some((adapter, timings))
}
