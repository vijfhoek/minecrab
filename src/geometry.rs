use std::marker::PhantomData;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    RenderPass,
};

use crate::{render_context::RenderContext, vertex::Vertex};

/// Represents a set of triangles by its vertices and indices.
#[derive(Default)]
pub struct Geometry<V: Vertex, I> {
    pub vertices: Vec<V>,
    pub indices: Vec<I>,
}

impl<T: Vertex, I> Geometry<T, I> {
    pub fn new(vertices: Vec<T>, indices: Vec<I>) -> Self {
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

pub struct GeometryBuffers<I> {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub index_count: usize,

    // Phantom data to store the index type
    _phantom: PhantomData<I>,
}

impl<I: bytemuck::Pod> GeometryBuffers<I> {
    pub fn from_geometry<V: Vertex + bytemuck::Pod>(
        render_context: &RenderContext,
        geometry: &Geometry<V, I>,
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
            _phantom: PhantomData,
        }
    }

    pub fn draw_indexed(&self, render_pass: &mut RenderPass) -> usize {
        render_pass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        self.index_count / 3
    }
}

impl GeometryBuffers<u16> {
    pub fn apply_buffers<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
    }
}

impl GeometryBuffers<u32> {
    pub fn set_buffers<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
    }
}
