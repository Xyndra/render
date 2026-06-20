// WARNING: AI GENERATED; UNDER REVIEW

use render_components::primitives::Rectangle;
use render_layout::InternalLayoutable;
use std::cell::Cell;
use wgpu::{include_wgsl, util::DeviceExt};

/// Rectangle renderer for wgpu.
///
/// Renders one rectangle at a time so that draw order matches the
/// primitive list produced by the layout engine.
pub struct RectangleRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    /// Next uniform slot to write into (reset each frame).
    next_rect: Cell<u32>,
    /// Aligned byte stride for one uniform entry.
    dynamic_alignment: u64,
}

/// Uniform data for rectangle rendering
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RectangleUniform {
    rect_position: [f32; 2],
    rect_size: [f32; 2],
    screen_size: [f32; 2],
    filler: [f32; 2], // Padding to align to 16 bytes
    rounding: [f32; 4],
    color: [f32; 4],
}

/// Vertex data for a quad
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// (0)XXXXXXXXXX(1)
///  X XX         X
///  X   XXX      X
///  X      XXX   X
///  X         XX X
/// (3)XXXXXXXXXX(2)
/// Quad vertices in local coordinates [0, 1]
const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0],
    }, // Top-left
    Vertex {
        position: [1.0, 0.0],
    }, // Top-right
    Vertex {
        position: [1.0, 1.0],
    }, // Bottom-right
    Vertex {
        position: [0.0, 1.0],
    }, // Bottom-left
];

/// Quad indices for two triangles
const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

impl RectangleRenderer {
    /// Create a new rectangle renderer
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        // Load shader
        let shader_source = include_wgsl!("shaders/rectangle.wgsl");
        let shader = device.create_shader_module(shader_source);

        let uniform_size = size_of::<RectangleUniform>() as u64;
        let min_alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let dynamic_alignment = (uniform_size + min_alignment - 1) & !(min_alignment - 1);

        // Uniform buffer — 128 slots, each dynamically offset
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rectangle Uniform Buffer"),
            size: dynamic_alignment * 128,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Rectangle Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group with explicit size matching one uniform entry
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rectangle Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: Some(
                        std::num::NonZeroU64::new(uniform_size)
                            .expect("uniform_size must be non-zero"),
                    ),
                }),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rectangle Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rectangle Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw, // Fed-in coordinates are CCW, but vertex shader makes them CW
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rectangle Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rectangle Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            next_rect: Cell::new(0),
            dynamic_alignment,
        }
    }

    /// Reset the per-frame uniform slot counter.  Call once at the
    /// start of every frame, before any `render_one` calls.
    pub fn begin_frame(&self) {
        self.next_rect.set(0);
    }

    /// Render a single rectangle primitive.
    ///
    /// Sets the pipeline, uploads uniforms, and issues the draw call.
    /// Safe to call interleaved with other renderers — each call is
    /// self-contained.
    pub fn render_one<'a>(
        &'a self,
        rect: &Rectangle,
        screen_size: (u32, u32),
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        let width = rect.get_width() as f32;
        let height = rect.get_height() as f32;

        let color_f32 = [
            rect.color.0 as f32 / 255.0,
            rect.color.1 as f32 / 255.0,
            rect.color.2 as f32 / 255.0,
            rect.color.3 as f32 / 255.0,
        ];

        let mut rounding_f32: [f32; 4] = rect.rounding.unwrap_or((0.0, 0.0, 0.0, 0.0)).into();
        rounding_f32 = rounding_f32.map(|a| a * (width.min(height) / 2.0));

        let position = [rect.get_x() as f32, rect.get_y() as f32];

        let uniforms = RectangleUniform {
            rect_position: position,
            rect_size: [width, height],
            screen_size: [screen_size.0 as f32, screen_size.1 as f32],
            filler: [0.0, 0.0],
            rounding: rounding_f32,
            color: color_f32,
        };

        let slot = self.next_rect.get();
        if slot >= 128 {
            return;
        }
        let byte_offset = slot as u64 * self.dynamic_alignment;

        queue.write_buffer(
            &self.uniform_buffer,
            byte_offset,
            bytemuck::cast_slice(&[uniforms]),
        );

        render_pass.set_bind_group(0, &self.bind_group, &[byte_offset as u32]);
        render_pass.draw_indexed(0..6, 0, 0..1);
        self.next_rect.set(slot + 1);
    }
}
