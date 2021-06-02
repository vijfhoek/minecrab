use cgmath::SquareMatrix;

use crate::camera::{Camera, Projection};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct View {
    view_position: [f32; 4],
    view_projection: [[f32; 4]; 4],
}

impl View {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_projection: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_projection(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_projection = (projection.calculate_matrix() * camera.calculate_matrix()).into();
    }
}
