#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    pub mouse_pos: [f32; 2],
    pub window_size: [f32; 2],
}
