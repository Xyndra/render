// WARNING: AI GENERATED; UNDER REVIEW

use render_components::shapes::Shapes;
use wgpu::{include_wgsl, util::DeviceExt};

/// Rectangle renderer for wgpu
pub struct RectangleRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

/// Uniform data for rectangle rendering
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RectangleUniform {
    rect_position: [f32; 2],
    rect_size: [f32; 2],
    screen_size: [f32; 2],
    rounding: f32,
    _padding1: f32, // Padding for vec4 alignment
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
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
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

        // Create uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rectangle Uniform Buffer"),
            contents: bytemuck::cast_slice(&[RectangleUniform {
                rect_position: [0.0, 0.0],
                rect_size: [100.0, 100.0],
                screen_size: [800.0, 600.0],
                _padding1: 0.0,
                color: [1.0, 0.0, 0.0, 1.0], // Red
                rounding: 0.0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Rectangle Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rectangle Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
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
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
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
        }
    }

    /// Render rectangles
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        shapes: &[Shapes<'_>],
        screen_size: (u32, u32),
        queue: &wgpu::Queue,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for shape in shapes {
            if let Shapes::Rectangle {
                sizing,
                position,
                color,
                rounding,
            } = shape
            {
                // Get resolved dimensions or use defaults
                let width = sizing.resolved_width.expect("width was not resolved") as f32;
                let height = sizing.resolved_height.expect("height was not resolved") as f32;

                // Convert color from (u8, u8, u8) to [f32; 4]
                let color_f32 = [
                    color.0 as f32 / 255.0,
                    color.1 as f32 / 255.0,
                    color.2 as f32 / 255.0,
                    1.0, // Alpha
                ];

                // Get rounding value
                let rounding_f32 = rounding.unwrap_or(0) as f32;

                let position = [
                    position.resolved_x.expect("x was not resolved") as f32,
                    position.resolved_y.expect("y was not resolved") as f32,
                ];

                // Update uniform buffer
                let uniforms = RectangleUniform {
                    rect_position: position,
                    rect_size: [width, height],
                    screen_size: [screen_size.0 as f32, screen_size.1 as f32],
                    _padding1: 0.0,
                    color: color_f32,
                    rounding: rounding_f32,
                };

                queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

                // Draw the rectangle (quad with 6 indices)
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }
    }
}
