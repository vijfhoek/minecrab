use std::marker::PhantomData;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    RenderPass,
};

use crate::{geometry::Geometry, render_context::RenderContext, vertex::Vertex};

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
        usage: wgpu::BufferUsages,
    ) -> Self {
        let vertices = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("geometry vertex buffer"),
                contents: bytemuck::cast_slice(&geometry.vertices),
                usage: wgpu::BufferUsages::VERTEX | usage,
            });

        let indices = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("geometry index buffer"),
                contents: bytemuck::cast_slice(&geometry.indices),
                usage: wgpu::BufferUsages::INDEX | usage,
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
    pub fn apply_buffers<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
    }
}
