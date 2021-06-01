use std::mem::size_of;

use wgpu::VertexAttribute;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texture_coordinates: [f32; 2],
    pub normal: [f32; 3],
}

const VERTEX_DESC: &[VertexAttribute] = &wgpu::vertex_attr_array![
    0 => Float32x3,
    1 => Float32x2,
    2 => Float32x3,
];

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: VERTEX_DESC,
        }
    }
}

/// Vertex used to represent block vertices.
///
/// Aside from the usual vertex position, texture coordinates and normal, this "vertex" also
/// contains whether the block is highlighted (i.e. the player is pointing at the block) and its
/// texture index (to address the texture arrays)
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockVertex {
    pub position: [f32; 3],
    pub texture_coordinates: [f32; 2],
    pub normal: [f32; 3],
    pub highlighted: i32,
    pub texture_id: i32,
}

const BLOCK_VERTEX_DESC: &[VertexAttribute] = &wgpu::vertex_attr_array![
    0 => Float32x3,
    1 => Float32x2,
    2 => Float32x3,
    3 => Sint32,
    4 => Sint32,
];

impl BlockVertex {
    pub fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: BLOCK_VERTEX_DESC,
        }
    }
}
