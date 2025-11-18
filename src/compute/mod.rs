use image::GenericImageView;

use crate::{program, renderer::ComputeShaderData, util::Resize};

pub mod demosaic;
pub mod downsample;
pub mod fragment;
pub mod processing;

pub fn enqueue_workload(encoder: &mut wgpu::CommandEncoder, shader: &ComputeShaderData) {
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("compute_pass"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&shader.pipeline);
        cpass.set_bind_group(0, &shader.bind_group, &[]);
        cpass.set_bind_group(1, &shader.uniform_bind_group, &[]);
        cpass.insert_debug_marker("Dispatching compute shader");
        let workgroup_size = 16;
        let dispatch_x = shader.size.width.div_ceil(workgroup_size);
        let dispatch_y = shader.size.height.div_ceil(workgroup_size);
        cpass.dispatch_workgroups(dispatch_x, dispatch_y, 1); // Number of cells to run, the (x,y,z) size of item being processed
    }
}

pub fn uniforms_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("uniforms_bind_group_layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

pub fn uniforms_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniforms: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("uniforms_bind_group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(uniforms.as_entire_buffer_binding()),
        }],
    })
}

pub fn create_texture(device: &wgpu::Device, image: &image::DynamicImage) -> wgpu::Texture {
    let (width, height) = image.dimensions();
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Storage JPG Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
    })
}

pub fn create_float_texture(
    device: &wgpu::Device,
    size: iced::Size<u32>,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Storage Float Texture"),
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[format],
    })
}

pub fn create_window_texture(
    device: &wgpu::Device,
    window_size: iced::Size<u32>,
    image_size: iced::Size<u32>,
) -> wgpu::Texture {
    let size = crate::util::calculate_image_size(window_size, image_size).resize(1.2);
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Storage Texture"),
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
    })
}

pub fn to_texture_view(texture: &wgpu::Texture) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("compute_image_texture_view"),
        format: None,
        dimension: Some(wgpu::TextureViewDimension::D2),
        // usage: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
        usage: None,
    })
}

pub fn write_texture(queue: &wgpu::Queue, texture: &wgpu::Texture, image: &program::Image) {
    let (width, height) = image.dimensions();
    let data: Vec<u8> = match image {
        program::Image::DynamicImage(img) => img.to_rgba8().into_raw(),
        program::Image::RawImage(raw) => match &raw.data {
            rawloader::RawImageData::Integer(items) => bytemuck::cast_slice(
                &items
                    .iter()
                    .copied()
                    .map(f32::from)
                    // .flat_map(|v| [v, v, v, v])
                    .collect::<Vec<f32>>(),
            )
            .to_vec(),
            #[allow(clippy::panic)]
            rawloader::RawImageData::Float(_) => panic!("Not supported"),
        },
    };

    // info!("Writing texture of size {}x{}", width, height);
    // info!("Data length: {}", data.len());
    // info!("Bytes per row: {}", 4 * width);
    // info!("Rows per image: {}", data.len() as u32 / (4 * width));
    // info!("Texture: {:?}", texture.format());
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

pub fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("my_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
