use cgmath::{EuclideanSpace, Matrix4, Point3, SquareMatrix, Vector4, Zero};

use crate::{
    aabb::Aabb,
    camera::{Camera, Projection, OPENGL_TO_WGPU_MATRIX},
};

pub struct View {
    view_position: Vector4<f32>,
    view_projection: Matrix4<f32>,
    pub aabb: Aabb,
}

impl View {
    pub fn to_raw(&self) -> ViewRaw {
        ViewRaw {
            view_position: self.view_position.into(),
            view_projection: self.view_projection.into(),
        }
    }

    pub fn new() -> Self {
        Self {
            view_position: Vector4::zero(),
            view_projection: Matrix4::identity(),
            aabb: Aabb::default(),
        }
    }

    pub fn update_view_projection(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous();
        self.view_projection = projection.calculate_matrix() * camera.calculate_matrix();
        self.aabb = self.frustrum_aabb();
    }

    fn frustrum_aabb(&self) -> Aabb {
        let projection = OPENGL_TO_WGPU_MATRIX.invert().unwrap() * self.view_projection;
        let inverse_matrix = projection.invert().unwrap();

        let corners = &[
            Vector4::new(-1.0, -1.0, 1.0, 1.0),
            Vector4::new(-1.0, -1.0, -1.0, 1.0),
            Vector4::new(-1.0, 1.0, 1.0, 1.0),
            Vector4::new(-1.0, 1.0, -1.0, 1.0),
            Vector4::new(1.0, -1.0, 1.0, 1.0),
            Vector4::new(1.0, -1.0, -1.0, 1.0),
            Vector4::new(1.0, 1.0, 1.0, 1.0),
            Vector4::new(1.0, 1.0, -1.0, 1.0),
        ];

        let mut min = Vector4::new(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0);
        let mut max = Vector4::new(0.0, 0.0, 0.0, 1.0);
        for &corner in corners {
            let corner = inverse_matrix * corner;
            let corner = corner / corner.w;

            min = min.zip(corner, f32::min);
            max = max.zip(corner, f32::max);
        }

        Aabb {
            min: Point3::from_vec(min.truncate()),
            max: Point3::from_vec(max.truncate()),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewRaw {
    view_position: [f32; 4],
    view_projection: [[f32; 4]; 4],
}
