use std::path::PathBuf;

pub struct ComputeRenderer {
    pub fragment_shader: RenderShaderData,
    pub uniforms: wgpu::Buffer,
    pub processing_shader: ComputeShaderData,
    pub downsample_shader: ComputeShaderData,
    pub image_path: PathBuf,
    pub textures: Textures,
}

pub struct ComputeShaderData {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub uniform_bind_group: wgpu::BindGroup,
}

pub struct RenderShaderData {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub uniform_bind_group: wgpu::BindGroup,
}

pub struct Textures {
    pub full_texture: wgpu::Texture,
    pub input_texture: wgpu::Texture,
    pub output_texture: wgpu::Texture,
    #[allow(dead_code)]
    pub image_size: iced::Size<u32>,
    pub output_size: iced::Size<u32>,
}
