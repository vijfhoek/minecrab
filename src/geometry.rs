use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    RenderPass,
};

use crate::{render_context::RenderContext, vertex::Vertex};

/// Represents a set of triangles by its vertices and indices.
#[derive(Default)]
pub struct Geometry<T: Vertex> {
    pub vertices: Vec<T>,
    pub indices: Vec<u16>,
}

impl<T: Vertex> Geometry<T> {
    pub fn new(vertices: Vec<T>, indices: Vec<u16>) -> Self {
        Self { vertices, indices }
    }

    /// Moves all the vertices and indices of `other` into `Self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut Self) {
        self.vertices.append(&mut other.vertices);
        self.indices.append(&mut other.indices);
    }

    /// Returns the number of indices in the vertex.
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}

pub struct GeometryBuffers {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub index_count: usize,
}

impl GeometryBuffers {
    pub fn from_geometry<T: Vertex + bytemuck::Pod>(
        render_context: &RenderContext,
        geometry: &Geometry<T>,
        usage: wgpu::BufferUsage,
    ) -> Self {
        let vertices = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&geometry.vertices),
                usage: wgpu::BufferUsage::VERTEX | usage,
            });

        let indices = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&geometry.indices),
                usage: wgpu::BufferUsage::INDEX | usage,
            });

        Self {
            vertices,
            indices,
            index_count: geometry.index_count(),
        }
    }

    pub fn set_buffers<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
    }

    pub fn draw_indexed(&self, render_pass: &mut RenderPass) {
        render_pass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
