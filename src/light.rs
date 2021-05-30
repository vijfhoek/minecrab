use cgmath::Vector3;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
}

impl Light {
    pub fn new(position: Vector3<f32>, color: Vector3<f32>) -> Self {
        Self {
            position: position.into(),
            _padding: 0,
            color: color.into(),
        }
    }
}
