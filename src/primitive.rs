use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    Result,
    compute::{
        self, downsample::DownsampleShader, fragment::FragmentShader, processing::ProcessingShader,
    },
    renderer::{ComputeRenderer, Textures},
    uniforms::{self, Uniforms},
    util::{Resize, Tou32, timed},
};

#[derive(Debug)]
pub struct Primitive {
    pub uniforms: Uniforms,
    pub image_path: PathBuf,
    pub image: Arc<image::DynamicImage>,
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
                renderer.textures.window_size,
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
        let layout = renderer.fragment_shader.pipeline.get_bind_group_layout(0);
        let fragment_bind_group = FragmentShader::create_bind_group(
            device,
            &layout,
            &renderer.uniforms,
            &textures.output_texture,
        );
        let processing_bind_group = ProcessingShader::create_bind_group(
            device,
            &renderer.processing_shader.pipeline,
            &renderer.uniforms,
            &textures,
        );
        let downsample_bind_group = DownsampleShader::create_bind_group(
            device,
            &renderer.downsample_shader.pipeline,
            &renderer.uniforms,
            &textures,
        );
        renderer.image_path.clone_from(&self.image_path);
        renderer.textures = textures;
        renderer.fragment_shader.bind_group = fragment_bind_group;
        renderer.processing_shader.bind_group = processing_bind_group;
        renderer.downsample_shader.bind_group = downsample_bind_group;
    }

    fn create_image_textures(
        &self,
        image: &image::DynamicImage,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Textures {
        let image_size = iced::Size::new(image.width(), image.height());
        let window_size = self.uniforms.window_size.to_u32();
        let full_texture = compute::create_texture(device, image);
        let input_texture = compute::create_window_texture(device, window_size, image_size);
        let output_texture = compute::create_window_texture(device, window_size, image_size);
        let size = crate::util::calculate_image_size(window_size, image_size).resize(1.2);
        compute::write_texture(queue, &full_texture, image);

        Textures {
            full_texture,
            input_texture,
            output_texture,
            image_size,
            window_size: size,
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
        let processing_shader = ProcessingShader::compile(device, &uniforms, &textures);
        let downsample_shader = DownsampleShader::compile(device, &uniforms, &textures);

        ComputeRenderer {
            fragment_shader,
            uniforms,
            processing_shader,
            downsample_shader,
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
            bytemuck::bytes_of(&self.uniforms.to_raw()),
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
            &renderer.downsample_shader.pipeline,
            &renderer.downsample_shader.bind_group,
            renderer.textures.window_size.width,
            renderer.textures.window_size.height,
        );
        compute::enqueue_workload(
            encoder,
            &renderer.processing_shader.pipeline,
            &renderer.processing_shader.bind_group,
            renderer.textures.window_size.width,
            renderer.textures.window_size.height,
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

fn create_uniforms_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("halo.pipeline.uniforms"),
        size: std::mem::size_of::<uniforms::Raw>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
