#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
}
