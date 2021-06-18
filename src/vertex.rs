use std::mem::size_of;

use wgpu::VertexAttribute;

pub trait Vertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static>;
}

/// Represents a vertex in HUD geometry.
///
/// Used to bind vertex information to shaders with a 2D position, texture
/// coordinates and index (for texture arrays) and a value (for dimming e.g.
/// the sides on blocks in inventories)
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HudVertex {
    pub position: [f32; 2],
    pub texture_coordinates: [f32; 2],
    pub texture_index: i32,
    pub color: [f32; 4],
}

const HUD_VERTEX_ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![
    0 => Float32x2,
    1 => Float32x2,
    2 => Sint32,
    3 => Float32x4,
];

impl Vertex for HudVertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: HUD_VERTEX_ATTRIBUTES,
        }
    }
}

/// Represents a vertex in world geometry.
///
/// Aside from the usual vertex position, texture coordinates and normal, this "vertex" also
/// contains whether the block is highlighted (i.e. the player is pointing at the block), its
/// texture index (to address the texture arrays) and a color multiplier.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockVertex {
    pub position: [f32; 3],
    pub texture_coordinates: [f32; 2],
    pub normal: [f32; 3],
    pub highlighted: i32,
    pub texture_id: i32,
    pub color: [f32; 4],
}

const BLOCK_VERTEX_ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![
    0 => Float32x3,
    1 => Float32x2,
    2 => Float32x3,
    3 => Sint32,
    4 => Sint32,
    5 => Float32x4,
];

impl Vertex for BlockVertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: BLOCK_VERTEX_ATTRIBUTES,
        }
    }
}
