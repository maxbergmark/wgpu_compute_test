#[derive(Debug, Default, Clone, Copy)]
pub struct Uniforms {
    pub mouse_pos: (f32, f32),
    pub scroll_delta: f32,
    pub window_size: iced::Size<f32>,
    pub image_size: iced::Size<f32>,
}

impl Uniforms {
    pub fn to_raw(self) -> Raw {
        Raw {
            mouse_pos: [self.mouse_pos.0, self.mouse_pos.1],
            scroll_delta: self.scroll_delta,
            window_size: self.window_size.into(),
            image_size: self.image_size.into(),
            _padding: [0.0],
        }
    }
}

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Raw {
    pub mouse_pos: [f32; 2],
    pub window_size: [f32; 2],
    pub image_size: [f32; 2],
    pub scroll_delta: f32,
    pub _padding: [f32; 1],
}
