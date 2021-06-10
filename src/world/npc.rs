extern crate gltf;
extern crate wgpu;

use cgmath::Vector3;
use wgpu::{BufferUsage, RenderPass};

use crate::{
    geometry::{Geometry, GeometryBuffers},
    render_context::RenderContext,
    vertex::BlockVertex,
};

pub struct Npc {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub geometry: Geometry<BlockVertex, u32>,
    pub geometry_buffers: Option<GeometryBuffers<u32>>,
}

impl Npc {
    pub fn new() -> Self {
        let position: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        let scale: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        let rotation: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

        let (model, buffers, _) = gltf::import("assets/models/minecrab.glb").unwrap();

        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        for mesh in model.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                indices = reader.read_indices().unwrap().into_u32().collect();

                // loop over all primitives and get the normals, position and color
                let pos_iter = reader.read_positions().unwrap();
                let norm_iter = reader.read_normals().unwrap();
                let tex_iter = reader.read_tex_coords(0).unwrap().into_f32();

                for ((position, normal), texture_coordinates) in
                    pos_iter.zip(norm_iter).zip(tex_iter)
                {
                    let current_vert = BlockVertex {
                        position,
                        texture_coordinates,
                        normal,
                        highlighted: 0,
                        texture_id: 0,
                    };

                    vertices.push(current_vert);
                }
            }
        }

        Self {
            position,
            scale,
            rotation,
            geometry: Geometry::new(vertices, indices),
            geometry_buffers: None,
        }
    }

    pub fn load_geometry(&mut self, render_context: &RenderContext) {
        self.geometry_buffers = Some(GeometryBuffers::from_geometry(
            render_context,
            &self.geometry,
            BufferUsage::empty(),
        ));
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) -> usize {
        let buffers = self.geometry_buffers.as_ref().unwrap();
        buffers.apply_buffers(render_pass);
        buffers.draw_indexed(render_pass)
    }
}
