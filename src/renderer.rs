use std::path::PathBuf;

use crate::compute::{
    demosaic::DemosaicShader, downsample::DownsampleShader, fragment::FragmentShader,
    processing::ProcessingShader,
};

pub struct ComputeRenderer {
    pub fragment_shader: RenderShaderData,
    pub uniforms: wgpu::Buffer,
    pub demosaic_shader: ComputeShaderData,
    pub downsample_shader: ComputeShaderData,
    pub processing_shader: ComputeShaderData,
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
    pub full_output_texture: wgpu::Texture,
    pub input_texture: wgpu::Texture,
    pub output_texture: wgpu::Texture,
    #[allow(dead_code)]
    pub image_size: iced::Size<u32>,
    pub output_size: iced::Size<u32>,
}

impl ComputeRenderer {
    pub fn replace_bind_groups(&mut self, device: &wgpu::Device) {
        let (fragment_bind_group, fragment_uniform_bind_group) = FragmentShader::create_bind_group(
            device,
            &self.fragment_shader.pipeline,
            &self.uniforms,
            &self.textures.output_texture,
        );
        let (processing_bind_group, processing_uniform_bind_group) =
            ProcessingShader::create_bind_group(
                device,
                &self.processing_shader.pipeline,
                &self.uniforms,
                &self.textures,
            );
        let (downsample_bind_group, downsample_uniform_bind_group) =
            DownsampleShader::create_bind_group(
                device,
                &self.downsample_shader.pipeline,
                &self.uniforms,
                &self.textures,
            );
        let (demosaic_bind_group, demosaic_uniform_bind_group) = DemosaicShader::create_bind_group(
            device,
            &self.demosaic_shader.pipeline,
            &self.uniforms,
            &self.textures,
        );
        self.fragment_shader.bind_group = fragment_bind_group;
        self.fragment_shader.uniform_bind_group = fragment_uniform_bind_group;
        self.processing_shader.bind_group = processing_bind_group;
        self.downsample_shader.bind_group = downsample_bind_group;
        self.demosaic_shader.bind_group = demosaic_bind_group;
        self.processing_shader.uniform_bind_group = processing_uniform_bind_group;
        self.downsample_shader.uniform_bind_group = downsample_uniform_bind_group;
        self.demosaic_shader.uniform_bind_group = demosaic_uniform_bind_group;
    }
}
