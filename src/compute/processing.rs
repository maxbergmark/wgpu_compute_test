use std::borrow::Cow;

use wgpu::PipelineCompilationOptions;

use crate::{
    compute::to_texture_view,
    renderer::{ComputeShaderData, Textures},
};

pub struct ProcessingShader;

impl ProcessingShader {
    pub fn compile(
        device: &wgpu::Device,
        uniforms: &wgpu::Buffer,
        textures: &Textures,
    ) -> ComputeShaderData {
        let processing_pipeline = Self::create_pipeline(device);
        let processing_bind_group =
            Self::create_bind_group(device, &processing_pipeline, uniforms, textures);
        ComputeShaderData {
            pipeline: processing_pipeline,
            bind_group: processing_bind_group,
        }
    }
    pub fn create_pipeline(device: &wgpu::Device) -> wgpu::ComputePipeline {
        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("compute_shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shader/processing.wgsl"
            ))),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("compute_pipeline_layout"),
            bind_group_layouts: &[&Self::create_bind_group_layout(device)],
            push_constant_ranges: &[],
        });

        // Instantiates the pipeline.
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&layout),
            module: &cs_module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        })
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("compute_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn create_bind_group(
        device: &wgpu::Device,
        compute_pipeline: &wgpu::ComputePipeline,
        uniforms: &wgpu::Buffer,
        textures: &Textures,
    ) -> wgpu::BindGroup {
        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let input_texture_view = to_texture_view(&textures.input_texture);
        let output_texture_view = to_texture_view(&textures.output_texture);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&output_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(uniforms.as_entire_buffer_binding()),
                },
            ],
        })
    }
}
