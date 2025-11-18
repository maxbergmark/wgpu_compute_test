#[derive(Debug, Default, Clone, Copy)]
pub struct Uniforms {
    pub mouse_pos: (f32, f32),
    pub scroll_delta: f32,
    pub window_size: iced::Size<f32>,
    pub image_size: iced::Size<f32>,
    pub cam_2_xyz: [[f32; 4]; 3],
    pub xyz_2_srgb: [[f32; 3]; 3],
    pub whitelevels: [f32; 4],
    pub blacklevels: [f32; 4],
    pub crops: [u32; 4],
}

impl Uniforms {
    pub fn to_raw(self, output_size: iced::Size<f32>) -> Raw {
        Raw {
            cam_2_xyz: self.cam_2_xyz,
            xyz_2_srgb: pad_matrix(self.xyz_2_srgb),
            whitelevels: self.whitelevels,
            blacklevels: self.blacklevels,
            crops: self.crops,
            mouse_pos: [self.mouse_pos.0, self.mouse_pos.1],
            scroll_delta: self.scroll_delta,
            window_size: self.window_size.into(),
            image_size: self.image_size.into(),
            output_size: output_size.into(),
            _padding: [0.0; 3],
            _pad2: [0.0; 4],
        }
    }
}

const fn pad_matrix(matrix: [[f32; 3]; 3]) -> [[f32; 4]; 3] {
    [
        [matrix[0][0], matrix[0][1], matrix[0][2], 0.0],
        [matrix[1][0], matrix[1][1], matrix[1][2], 0.0],
        [matrix[2][0], matrix[2][1], matrix[2][2], 0.0],
    ]
}

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Raw {
    pub cam_2_xyz: [[f32; 4]; 3],
    pub xyz_2_srgb: [[f32; 4]; 3],
    pub whitelevels: [f32; 4],
    pub blacklevels: [f32; 4],
    pub crops: [u32; 4],
    pub mouse_pos: [f32; 2],
    pub window_size: [f32; 2],
    pub image_size: [f32; 2],
    pub output_size: [f32; 2],
    pub scroll_delta: f32,
    _padding: [f32; 3],
    _pad2: [f32; 4],
}
