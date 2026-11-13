//! Render pipelines and shared GPU objects of the 3D view.

use super::globals::GLOBALS_SIZE;
use crate::features::viewport::logic::scene_buffers::VERTEX_STRIDE;
use eframe::wgpu;

/// Format of the offscreen color target. Colors are written in gamma space.
pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// Format of the offscreen depth target.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// Multisampling of the offscreen target.
pub const SAMPLES: u32 = 4;

/// Pipelines for the sky, opaque and blended meshes, lines and the final copy into egui.
pub(super) struct Pipelines {
    pub(super) sky: wgpu::RenderPipeline,
    pub(super) opaque: wgpu::RenderPipeline,
    pub(super) blended: wgpu::RenderPipeline,
    pub(super) lines: wgpu::RenderPipeline,
    pub(super) blit: wgpu::RenderPipeline,
    pub(super) globals_layout: wgpu::BindGroupLayout,
    pub(super) blit_layout: wgpu::BindGroupLayout,
}

fn depth_state(write: bool, compare: wgpu::CompareFunction) -> Option<wgpu::DepthStencilState> {
    Some(wgpu::DepthStencilState {
        format: DEPTH_FORMAT,
        depth_write_enabled: Some(write),
        depth_compare: Some(compare),
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    })
}

struct Spec<'a> {
    label: &'a str,
    layout: &'a wgpu::PipelineLayout,
    module: &'a wgpu::ShaderModule,
    vertex: &'a str,
    fragment: &'a str,
    format: wgpu::TextureFormat,
    topology: wgpu::PrimitiveTopology,
    buffers: &'a [Option<wgpu::VertexBufferLayout<'a>>],
    blend: Option<wgpu::BlendState>,
    depth: Option<wgpu::DepthStencilState>,
    samples: u32,
}

fn pipeline(device: &wgpu::Device, spec: Spec) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(spec.label),
        layout: Some(spec.layout),
        vertex: wgpu::VertexState {
            module: spec.module,
            entry_point: Some(spec.vertex),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: spec.buffers,
        },
        primitive: wgpu::PrimitiveState {
            topology: spec.topology,
            ..Default::default()
        },
        depth_stencil: spec.depth,
        multisample: wgpu::MultisampleState {
            count: spec.samples,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: spec.module,
            entry_point: Some(spec.fragment),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: spec.format,
                blend: spec.blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

impl Pipelines {
    /// Compiles the shaders and builds every pipeline. `surface_format` is the window's format.
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let scene_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewport-scene"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let blit_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewport-blit"),
            source: wgpu::ShaderSource::Wgsl(include_str!("blit.wgsl").into()),
        });
        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport-globals-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(GLOBALS_SIZE as u64),
                },
                count: None,
            }],
        });
        let blit_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport-blit-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let scene_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("viewport-scene-layout"),
            bind_group_layouts: &[Some(&globals_layout)],
            immediate_size: 0,
        });
        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("viewport-blit-pipeline-layout"),
            bind_group_layouts: &[Some(&blit_layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Unorm8x4];
        let vertex_layout = [Some(wgpu::VertexBufferLayout {
            array_stride: VERTEX_STRIDE as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attributes,
        })];
        let scene = |label, vertex, fragment, topology, blend, depth| {
            pipeline(
                device,
                Spec {
                    label,
                    layout: &scene_layout,
                    module: &scene_module,
                    vertex,
                    fragment,
                    format: COLOR_FORMAT,
                    topology,
                    buffers: if vertex == "vs_sky" {
                        &[]
                    } else {
                        &vertex_layout
                    },
                    blend,
                    depth,
                    samples: SAMPLES,
                },
            )
        };
        let triangles = wgpu::PrimitiveTopology::TriangleList;
        let alpha = Some(wgpu::BlendState::ALPHA_BLENDING);
        let sky = scene(
            "viewport-sky",
            "vs_sky",
            "fs_sky",
            triangles,
            None,
            depth_state(false, wgpu::CompareFunction::Always),
        );
        let opaque = scene(
            "viewport-opaque",
            "vs_scene",
            "fs_scene",
            triangles,
            None,
            depth_state(true, wgpu::CompareFunction::Less),
        );
        let blended = scene(
            "viewport-blended",
            "vs_scene",
            "fs_scene",
            triangles,
            alpha,
            depth_state(false, wgpu::CompareFunction::Less),
        );
        let lines = scene(
            "viewport-lines",
            "vs_scene",
            "fs_line",
            wgpu::PrimitiveTopology::LineList,
            alpha,
            depth_state(false, wgpu::CompareFunction::LessEqual),
        );
        let blit = pipeline(
            device,
            Spec {
                label: "viewport-blit",
                layout: &blit_pipeline_layout,
                module: &blit_module,
                vertex: "vs_blit",
                fragment: if surface_format.is_srgb() {
                    "fs_blit_linear"
                } else {
                    "fs_blit_gamma"
                },
                format: surface_format,
                topology: triangles,
                buffers: &[],
                blend: None,
                depth: None,
                samples: 1,
            },
        );
        Self {
            sky,
            opaque,
            blended,
            lines,
            blit,
            globals_layout,
            blit_layout,
        }
    }
}
