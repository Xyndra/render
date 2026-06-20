// WARNING: AI GENERATED; UNDER REVIEW

use bytemuck::{Pod, Zeroable};
use std::mem::size_of;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexStepMode, vertex_attr_array};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(super) struct TextUniform {
    pub(super) screen_size: [f32; 2],
    pub(super) _pad: [f32; 2],
    pub(super) color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(super) struct TextVertex {
    pub(super) position: [f32; 2],
    pub(super) uv: [f32; 2],
    /// 0 = text atlas (R8, uniform colour), 1 = color atlas (RGBA, direct).
    pub(super) tex_index: u32,
}

impl TextVertex {
    const ATTRIBS: [VertexAttribute; 3] = vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Uint32,
    ];

    pub(super) fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
