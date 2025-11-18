use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    Result,
    compute::{
        self, demosaic::DemosaicShader, downsample::DownsampleShader, fragment::FragmentShader,
        processing::ProcessingShader,
    },
    program,
    renderer::{ComputeRenderer, Textures},
    uniforms::{self, Uniforms},
    util::{Resize, Tof32, Tou32, timed},
};

#[derive(Debug)]
pub struct Primitive {
    pub uniforms: Uniforms,
    pub image_path: PathBuf,
    pub image: Arc<program::Image>,
}

impl Primitive {
    fn check_resize(
        &self,
        renderer: &mut ComputeRenderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if renderer.image_path != self.image_path
            || should_resize(
                self.uniforms.window_size.to_u32(),
                renderer.textures.output_size,
            )
        {
            timed("Recreating buffers", || {
                self.recreate_buffers(renderer, device, queue);
            });
        }
    }

    fn recreate_buffers(
        &self,
        renderer: &mut ComputeRenderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let image = self.image.as_ref();
        // TODO: No need to recreate the full size texture if the image hasn't changed
        let textures = self.create_image_textures(image, device, queue);
        renderer.image_path.clone_from(&self.image_path);
        renderer.textures = textures;
        renderer.replace_bind_groups(device);
    }

    fn create_image_textures(
        &self,
        image: &program::Image,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Textures {
        let image_size = iced::Size::new(image.width(), image.height());
        let window_size = self.uniforms.window_size.to_u32();
        let full_texture = match image {
            program::Image::DynamicImage(dynamic_image) => {
                compute::create_texture(device, dynamic_image)
            }
            program::Image::RawImage(_) => {
                compute::create_float_texture(device, image_size, wgpu::TextureFormat::R32Float)
            }
        };
        let full_output_texture =
            compute::create_float_texture(device, image_size, wgpu::TextureFormat::Rgba32Float);
        let input_texture = compute::create_window_texture(device, window_size, image_size);
        let output_texture = compute::create_window_texture(device, window_size, image_size);
        let output_size = crate::util::calculate_image_size(window_size, image_size).resize(1.2);
        compute::write_texture(queue, &full_texture, image);

        Textures {
            full_texture,
            full_output_texture,
            input_texture,
            output_texture,
            image_size,
            output_size,
        }
    }
}

impl iced::widget::shader::Primitive for Primitive {
    type Renderer = ComputeRenderer;

    fn initialize(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Renderer {
        let image = self.image.as_ref();
        let uniforms = create_uniforms_buffer(device);
        let textures = self.create_image_textures(image, device, queue);
        let fragment_shader =
            FragmentShader::compile(device, format, &uniforms, &textures.output_texture);
        let demosaic_shader = DemosaicShader::compile(device, &uniforms, &textures);
        let downsample_shader = DownsampleShader::compile(device, &uniforms, &textures);
        let processing_shader = ProcessingShader::compile(device, &uniforms, &textures);

        ComputeRenderer {
            fragment_shader,
            uniforms,
            demosaic_shader,
            downsample_shader,
            processing_shader,
            image_path: self.image_path.clone(),
            textures,
        }
    }

    fn prepare(
        &self,
        renderer: &mut Self::Renderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &iced::Rectangle,
        _viewport: &iced::widget::shader::Viewport,
    ) {
        self.check_resize(renderer, device, queue);
        queue.write_buffer(
            &renderer.uniforms,
            0,
            bytemuck::bytes_of(&self.uniforms.to_raw(renderer.textures.output_size.to_f32())),
        );
    }

    fn render(
        &self,
        renderer: &Self::Renderer,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        bounds: &iced::Rectangle<u32>,
    ) {
        compute::enqueue_workload(
            encoder,
            &renderer.demosaic_shader.pipeline,
            &renderer.demosaic_shader.bind_group,
            &renderer.demosaic_shader.uniform_bind_group,
            renderer.textures.image_size,
        );

        compute::enqueue_workload(
            encoder,
            &renderer.downsample_shader.pipeline,
            &renderer.downsample_shader.bind_group,
            &renderer.downsample_shader.uniform_bind_group,
            renderer.textures.output_size,
        );
        compute::enqueue_workload(
            encoder,
            &renderer.processing_shader.pipeline,
            &renderer.processing_shader.bind_group,
            &renderer.processing_shader.uniform_bind_group,
            renderer.textures.output_size,
        );
        enqueue_draw(renderer, encoder, target, bounds);
    }
}

fn enqueue_draw(
    renderer: &ComputeRenderer,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    bounds: &iced::Rectangle<u32>,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("primitive.render_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    pass.set_scissor_rect(bounds.x, bounds.y, bounds.width, bounds.height);
    pass.set_viewport(
        bounds.x as f32,
        bounds.y as f32,
        bounds.width as f32,
        bounds.height as f32,
        0.0,
        1.0,
    );

    pass.set_pipeline(&renderer.fragment_shader.pipeline);
    pass.set_bind_group(0, &renderer.fragment_shader.bind_group, &[]);
    pass.set_bind_group(1, &renderer.fragment_shader.uniform_bind_group, &[]);
    pass.draw(0..6, 0..1);
}

// resize if new_size is larger than current_size, or smaller by a significant amount
fn should_resize(new_size: iced::Size<u32>, current_size: iced::Size<u32>) -> bool {
    if new_size.width > current_size.width || new_size.height > current_size.height {
        return true;
    }
    let width_diff = current_size.width as i32 - new_size.width as i32;
    let height_diff = current_size.height as i32 - new_size.height as i32;
    let width_threshold = (current_size.width as f32 * 0.5) as i32;
    let height_threshold = (current_size.height as f32 * 0.5) as i32;

    width_diff.abs() > width_threshold || height_diff.abs() > height_threshold
}

pub fn load_image(path: &Path) -> Result<image::DynamicImage> {
    let image = image::ImageReader::open(path)?.decode()?;
    Ok(image)
}

pub fn load_cr2_image(path: &Path) -> Result<rawloader::RawImage> {
    let image = rawloader::decode_file(path)?;
    Ok(image)
}

fn create_uniforms_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("halo.pipeline.uniforms"),
        size: std::mem::size_of::<uniforms::Raw>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
