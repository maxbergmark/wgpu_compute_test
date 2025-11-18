use std::borrow::Cow;

use crate::{
    compute::{self, uniforms_bind_group_layout},
    renderer::RenderShaderData,
};

pub struct FragmentShader;

impl FragmentShader {
    pub fn compile(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        uniforms: &wgpu::Buffer,
        output_texture: &wgpu::Texture,
    ) -> RenderShaderData {
        let pipeline = Self::create_pipeline(device, format);
        let (bind_group, uniform_bind_group) =
            Self::create_bind_group(device, &pipeline, uniforms, output_texture);
        RenderShaderData {
            pipeline,
            bind_group,
            uniform_bind_group,
        }
    }
    pub fn create_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("primitive.create_pipeline.layout"),
            bind_group_layouts: &[
                &Self::create_bind_group_layout(device),
                &uniforms_bind_group_layout(device),
            ],
            push_constant_ranges: &[],
        });

        let module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fragment.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(concat!(include_str!(
                "../shader/fragment.wgsl"
            ),))),
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("primitive.create_pipeline.render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        })
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("primitive.bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    pub fn create_bind_group(
        device: &wgpu::Device,
        pipeline: &wgpu::RenderPipeline,
        uniforms: &wgpu::Buffer,
        texture: &wgpu::Texture,
    ) -> (wgpu::BindGroup, wgpu::BindGroup) {
        let sampler = compute::create_sampler(device);
        let texture_view = compute::to_texture_view(texture);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("primitive.bind_group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let uniform_bind_group_layout = pipeline.get_bind_group_layout(1);
        let uniform_bind_group =
            compute::uniforms_bind_group(device, &uniform_bind_group_layout, uniforms);
        (bind_group, uniform_bind_group)
    }
}
