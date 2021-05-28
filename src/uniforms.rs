use cgmath::SquareMatrix;

use crate::camera::Camera;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    view_position: [f32; 4],
    view_projection: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_projection: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_position = camera.eye.to_homogeneous().into();
        self.view_projection = camera.build_view_projection_matrix().into();
    }
}
