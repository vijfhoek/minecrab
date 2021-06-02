extern crate gltf;
extern crate wgpu;

use cgmath::{Vector3};

use crate::{
    vertex::Vertex,
};

pub struct Npc {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Option<wgpu::Buffer>, 
    pub index_buffer: Option<wgpu::Buffer>,
}


impl Npc {
    pub fn load() -> Self {
        let position: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        let scale: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        let rotation: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

        let (model, buffers, _) = gltf::import("assets/models/minecrab.glb").unwrap();
        
        let mut indices: Vec<u32> = Vec::new();
        let mut vertices: Vec<Vertex> = Vec::new();

        for mesh in model.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                indices = reader.read_indices().unwrap().into_u32().collect();
                // loop over all primitives and get the normals, position and color

                let pos_iter = reader.read_positions().unwrap();
                let norm_iter = reader.read_normals().unwrap();
                let tex_iter = reader.read_tex_coords(0).unwrap().into_f32();

                for it in pos_iter.zip(norm_iter).zip(tex_iter) {
                    let ((position, normal), [tex_x, tex_y]) = it;

                    let current_vert: Vertex = Vertex {
                        position, 
                        texture_coordinates: [tex_x, tex_y, 0.0], 
                        normal, 
                        highlighted: 0
                    };

                    vertices.push(current_vert);
                }
            }
         }

        return Self {
            position,
            scale,
            rotation,
            indices,
            vertices,
            vertex_buffer: None,
            index_buffer: None
        };
    }
}
