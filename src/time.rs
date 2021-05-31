#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Time {
    pub time: f32,
}

impl Time {
    pub fn new() -> Self {
        Self { time: 0.0 }
    }
}
