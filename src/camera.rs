use cgmath::InnerSpace;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub yaw: cgmath::Rad<f32>,
    pub pitch: cgmath::Rad<f32>,
}

impl Camera {
    pub fn new(
        position: cgmath::Point3<f32>,
        yaw: cgmath::Rad<f32>,
        pitch: cgmath::Rad<f32>,
    ) -> Self {
        Self {
            position,
            yaw,
            pitch,
        }
    }

    pub fn calculate_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_to_rh(
            self.position,
            cgmath::Vector3::new(self.yaw.0.cos(), self.pitch.0.sin(), self.yaw.0.sin())
                .normalize(),
            cgmath::Vector3::unit_y(),
        )
    }
}

pub struct Projection {
    pub aspect_ratio: f32,
    pub fov_y: cgmath::Rad<f32>,
    pub z_near: f32,
    pub z_far: f32,
}

impl Projection {
    pub fn new<Fov: Into<cgmath::Rad<f32>>>(
        width: u32,
        height: u32,
        fov_y: Fov,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            aspect_ratio: width as f32 / height as f32,
            fov_y: fov_y.into(),
            z_near,
            z_far,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }

    pub fn calculate_matrix(&self) -> cgmath::Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX
            * cgmath::perspective(self.fov_y, self.aspect_ratio, self.z_near, self.z_far)
    }
}
