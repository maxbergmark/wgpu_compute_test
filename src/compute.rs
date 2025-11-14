use std::{borrow::Cow, time::Instant};

use image::GenericImageView;
use wgpu::{Extent3d, PipelineCompilationOptions, TexelCopyBufferLayout};

pub const WIDTH: u32 = 1024;
pub const HEIGHT: u32 = 1024;

#[derive(Debug, Clone)]
pub struct ComputeShader {
    // device: wgpu::Device,
    // queue: wgpu::Queue,
    // compute_pipeline: wgpu::ComputePipeline,
    // storage_buffer: wgpu::Texture,
    // staging_buffer: wgpu::Buffer,
    // texture_view: wgpu::TextureView,
}

impl ComputeShader {
    // pub async fn new() -> Option<Self> {
    //     let (device, queue) = Self::create_queue().await?;
    //     // let compute_pipeline = Self::compile_shader(&device).await;
    //     let (staging_buffer, storage_buffer) = Self::create_buffers(&device);
    //     let texture_view = Self::create_texture_view(&storage_buffer);

    //     Some(Self {
    //         // device,
    //         // queue,
    //         // compute_pipeline,
    //         // storage_buffer,
    //         // staging_buffer,
    //         // texture_view,
    //     })
    // }

    fn create_texture_view(storage_buffer: &wgpu::Texture) -> wgpu::TextureView {
        storage_buffer.create_view(&wgpu::TextureViewDescriptor {
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

    pub fn compile_shader(
        device: &wgpu::Device,
        // layout: &wgpu::BindGroupLayout,
    ) -> wgpu::ComputePipeline {
        // Loads the shader from WGSL
        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("compute_shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
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
            ],
        })
    }

    async fn create_queue() -> Option<(wgpu::Device, wgpu::Queue)> {
        // Instantiates instance of WebGPU
        let instance = wgpu::Instance::default();

        // `request_adapter` instantiates the general connection to the GPU
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok()?;

        // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
        //  `features` being the available features.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("compute_device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .ok()?;

        Some((device, queue))
    }

    pub fn create_bind_group(
        // &self,
        device: &wgpu::Device,
        compute_pipeline: &wgpu::ComputePipeline,
        input_texture: &wgpu::TextureView,
        output_texture: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        // A bind group defines how buffers are accessed by shaders.
        // It is to WebGPU what a descriptor set is to Vulkan.
        // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).

        // A pipeline specifies the operation of a shader

        // Instantiates the bind group, once again specifying the binding of buffers.
        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(output_texture),
                },
            ],
        })
    }

    async fn execute_gpu_inner(
        // &self,
        device: &wgpu::Device,
        compute_pipeline: &wgpu::ComputePipeline,
        queue: &wgpu::Queue,
        storage_buffer: &wgpu::Texture,
        staging_buffer: &wgpu::Buffer,
        // numbers: &[u32],
    ) -> Option<Vec<u8>> {
        let texture_view = Self::create_texture_view(storage_buffer);
        let bind_group =
            Self::create_bind_group(device, compute_pipeline, &texture_view, &texture_view);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("compute_encoder"),
        });
        Self::enqueue_workload(&mut encoder, compute_pipeline, &bind_group, WIDTH, HEIGHT);
        Self::copy_data_to_cpu(&mut encoder, storage_buffer, staging_buffer);
        queue.submit(Some(encoder.finish()));

        Self::poll_queue(device, staging_buffer).await
    }

    pub fn create_image_texture_view(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &image::DynamicImage,
    ) -> wgpu::TextureView {
        let (width, height) = image.dimensions();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Storage Texture"),
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
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.to_rgba8(),
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

    pub fn create_output_texture_view(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        image: &image::DynamicImage,
    ) -> wgpu::TextureView {
        let (width, height) = image.dimensions();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
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
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

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

    fn create_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Texture) {
        // Gets the size in bytes of the buffer.
        // let size = std::mem::size_of_val(numbers) as wgpu::BufferAddress;
        let size = HEIGHT * WIDTH * 4; // width * height * RGBA8

        // Instantiates buffer without data.
        // `usage` of buffer specifies how it can be used:
        //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
        //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: size as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Storage Texture"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        (staging_buffer, storage_buffer)
    }

    pub fn enqueue_workload(
        // &self,
        // device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        // queue: &wgpu::Queue,
        compute_pipeline: &wgpu::ComputePipeline,
        // numbers: &[u32],
        bind_group: &wgpu::BindGroup,
        width: u32,
        height: u32,
        // size: wgpu::BufferAddress,
    ) {
        // A command encoder executes one or many pipelines.
        // It is to WebGPU what a command buffer is to Vulkan.
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(compute_pipeline);
            cpass.set_bind_group(0, bind_group, &[]);
            cpass.insert_debug_marker("compute collatz iterations");
            cpass.dispatch_workgroups(width, height, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }
    }

    fn copy_data_to_cpu(
        encoder: &mut wgpu::CommandEncoder,
        storage_buffer: &wgpu::Texture,
        staging_buffer: &wgpu::Buffer,
    ) {
        // Sets adds copy operation to command encoder.
        // Will copy data from storage buffer on GPU to staging buffer on CPU.
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: storage_buffer,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: staging_buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * WIDTH),
                    rows_per_image: Some(HEIGHT),
                },
            },
            Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
        );
    }

    async fn poll_queue(device: &wgpu::Device, staging_buffer: &wgpu::Buffer) -> Option<Vec<u8>> {
        // Note that we're not calling `.await` here.
        let buffer_slice = staging_buffer.slice(..);
        // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;

        // Awaits until `buffer_future` can be read from
        if let Ok(Ok(())) = receiver.recv_async().await {
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to u32
            let result = bytemuck::cast_slice(&data).to_vec();

            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            staging_buffer.unmap(); // Unmaps buffer from memory
            // If you are familiar with C++ these 2 lines can be thought of similarly to:
            //   delete myPointer;
            //   myPointer = NULL;
            // It effectively frees the memory

            // Returns data from buffer
            Some(result)
        } else {
            None
        }
    }
}

#[cfg_attr(test, allow(dead_code))]
pub async fn run() -> Vec<u8> {
    // let numbers = (0..10_000).collect::<Vec<_>>();
    let (device, queue) = ComputeShader::create_queue().await.unwrap();
    let compute_pipeline = ComputeShader::compile_shader(&device);
    let (staging_buffer, storage_buffer) = ComputeShader::create_buffers(&device);
    // let compute_kernel = ComputeShader::new().await.unwrap();
    let start = Instant::now();
    let result = ComputeShader::execute_gpu_inner(
        &device,
        &compute_pipeline,
        &queue,
        &storage_buffer,
        &staging_buffer,
    )
    .await
    .unwrap();
    let duration = start.elapsed();
    println!("GPU computation took {:?}", duration);
    result
    // execute_gpu(&numbers).await.unwrap()
}

// #[cfg_attr(test, allow(dead_code))]
// async fn execute_gpu(numbers: &[u32]) -> Option<Vec<u32>> {}
