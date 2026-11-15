//! GPU resources of the 3D view, kept in egui's callback resources: pipelines, the offscreen target and the scene
//! buffers. Meshes are uploaded once per scene; per frame only a small uniform block is written.

use super::globals::{Globals, GLOBALS_SIZE};
use super::pipelines::{Pipelines, COLOR_FORMAT, DEPTH_FORMAT, SAMPLES};
use super::upload::SceneUpload;
use crate::features::import_export::models::Bounds;
use crate::features::viewport::logic::frustum;
use crate::features::viewport::logic::math::Mat4;
use crate::features::viewport::logic::scene_buffers::SceneBuffers;
use crate::features::viewport::models::ViewOptions;
use eframe::egui_wgpu::RenderState;
use eframe::wgpu::{self, util::DeviceExt};

/// Offscreen render target matching the size of the 3D view.
struct Target {
    size: [u32; 2],
    msaa_view: wgpu::TextureView,
    resolve_texture: wgpu::Texture,
    resolve_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    blit_group: wgpu::BindGroup,
}

struct GpuChunk {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
    bounds: Bounds,
    transparent: bool,
}

struct GpuScene {
    generation: u64,
    chunks: Vec<GpuChunk>,
    lines: wgpu::Buffer,
    grid_vertices: u32,
    axes_vertices: u32,
}

/// Everything the 3D view keeps on the GPU. Stored in `egui_wgpu::Renderer::callback_resources`.
pub struct ViewportGpu {
    pipelines: Pipelines,
    globals: wgpu::Buffer,
    globals_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    target: Option<Target>,
    scene: Option<GpuScene>,
}

/// Creates the GPU resources and registers them so paint callbacks can find them. Call once at start-up.
pub fn install(state: &RenderState) {
    let gpu = ViewportGpu::new(&state.device, state.target_format);
    state.renderer.write().callback_resources.insert(gpu);
}

fn texture(
    device: &wgpu::Device,
    size: [u32; 2],
    format: wgpu::TextureFormat,
    samples: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("viewport-target"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: samples,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

fn view(texture: &wgpu::Texture) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn buffer(device: &wgpu::Device, bytes: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("viewport-buffer"),
        contents: bytes,
        usage,
    })
}

impl ViewportGpu {
    fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let pipelines = Pipelines::new(device, surface_format);
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport-globals"),
            size: GLOBALS_SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewport-globals-group"),
            layout: &pipelines.globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            }],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            pipelines,
            globals,
            globals_group,
            sampler,
            target: None,
            scene: None,
        }
    }

    fn ensure_target(&mut self, device: &wgpu::Device, size: [u32; 2]) {
        if self.target.as_ref().is_some_and(|t| t.size == size) {
            return;
        }
        let attachment = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let resolve_texture = texture(
            device,
            size,
            COLOR_FORMAT,
            1,
            attachment | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
        );
        let resolve_view = view(&resolve_texture);
        let blit_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewport-blit-group"),
            layout: &self.pipelines.blit_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&resolve_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        self.target = Some(Target {
            size,
            msaa_view: view(&texture(device, size, COLOR_FORMAT, SAMPLES, attachment)),
            depth_view: view(&texture(device, size, DEPTH_FORMAT, SAMPLES, attachment)),
            resolve_texture,
            resolve_view,
            blit_group,
        });
    }

    fn upload(&mut self, device: &wgpu::Device, generation: u64, buffers: &SceneBuffers) {
        let chunks = buffers
            .chunks
            .iter()
            .map(|chunk| GpuChunk {
                vertices: buffer(device, &chunk.vertex_bytes, wgpu::BufferUsages::VERTEX),
                indices: buffer(device, &chunk.index_bytes, wgpu::BufferUsages::INDEX),
                index_count: chunk.index_count,
                bounds: chunk.bounds,
                transparent: chunk.transparent,
            })
            .collect();
        self.scene = Some(GpuScene {
            generation,
            chunks,
            lines: buffer(device, &buffers.overlay.bytes, wgpu::BufferUsages::VERTEX),
            grid_vertices: buffers.overlay.grid_vertices,
            axes_vertices: buffers.overlay.axes_vertices,
        });
    }

    /// Prepares the frame: resizes the target, uploads a new scene if needed and writes the uniforms.
    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
        globals: &Globals,
        scene: &SceneUpload,
    ) {
        let limit = device.limits().max_texture_dimension_2d;
        let size = [size[0].clamp(1, limit), size[1].clamp(1, limit)];
        self.ensure_target(device, size);
        if self.scene.as_ref().map(|s| s.generation) != Some(scene.generation) {
            if let Some(buffers) = scene.take() {
                self.upload(device, scene.generation, &buffers);
            }
        }
        queue.write_buffer(&self.globals, 0, &globals.to_bytes());
    }

    /// Records the 3D pass into `encoder`.
    pub(super) fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view_proj: &Mat4,
        options: ViewOptions,
    ) {
        let Some(target) = &self.target else {
            return;
        };
        let [r, g, b, a] = Globals::clear_color();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("viewport-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.msaa_view,
                depth_slice: None,
                resolve_target: Some(&target.resolve_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a }),
                    store: wgpu::StoreOp::Discard,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_bind_group(0, &self.globals_group, &[]);
        pass.set_pipeline(&self.pipelines.sky);
        pass.draw(0..3, 0..1);
        let Some(scene) = &self.scene else {
            return;
        };
        self.draw_chunks(&mut pass, scene, view_proj, false);
        self.draw_lines(&mut pass, scene, options);
        self.draw_chunks(&mut pass, scene, view_proj, true);
    }

    fn draw_chunks(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        scene: &GpuScene,
        view_proj: &Mat4,
        transparent: bool,
    ) {
        let pipeline = if transparent {
            &self.pipelines.blended
        } else {
            &self.pipelines.opaque
        };
        pass.set_pipeline(pipeline);
        for chunk in scene.chunks.iter().filter(|c| c.transparent == transparent) {
            if frustum::may_be_visible(view_proj, &chunk.bounds) {
                pass.set_vertex_buffer(0, chunk.vertices.slice(..));
                pass.set_index_buffer(chunk.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..chunk.index_count, 0, 0..1);
            }
        }
    }

    fn draw_lines(&self, pass: &mut wgpu::RenderPass<'_>, scene: &GpuScene, options: ViewOptions) {
        pass.set_pipeline(&self.pipelines.lines);
        pass.set_vertex_buffer(0, scene.lines.slice(..));
        if options.show_grid {
            pass.draw(0..scene.grid_vertices, 0..1);
        }
        if options.show_axes {
            let end = scene.grid_vertices + scene.axes_vertices;
            pass.draw(scene.grid_vertices..end, 0..1);
        }
    }

    /// Builds the GPU state on a bare device (no window), for the headless benchmark.
    pub(super) fn headless(device: &wgpu::Device) -> Self {
        Self::new(device, COLOR_FORMAT)
    }

    /// The finished color image of the last render (for reading it back).
    pub(super) fn resolved(&self) -> Option<&wgpu::Texture> {
        self.target.as_ref().map(|t| &t.resolve_texture)
    }

    /// Chunks held on the GPU.
    pub(super) fn chunk_count(&self) -> usize {
        self.scene.as_ref().map_or(0, |s| s.chunks.len())
    }

    /// Chunks and triangles the passes would draw for `view_proj` after frustum culling.
    pub(super) fn visible(&self, view_proj: &Mat4) -> (usize, usize) {
        let Some(scene) = &self.scene else {
            return (0, 0);
        };
        scene
            .chunks
            .iter()
            .filter(|c| frustum::may_be_visible(view_proj, &c.bounds))
            .fold((0, 0), |(n, t), c| {
                (n + 1, t + (c.index_count / 3) as usize)
            })
    }

    /// Copies the finished 3D target into egui's pass.
    pub(super) fn blit(&self, pass: &mut wgpu::RenderPass<'static>) {
        if let Some(target) = &self.target {
            pass.set_pipeline(&self.pipelines.blit);
            pass.set_bind_group(0, &target.blit_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
