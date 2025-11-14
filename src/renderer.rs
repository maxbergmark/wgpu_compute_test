pub struct ComputeRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub compute_bind_group: wgpu::BindGroup,
    pub uniforms: wgpu::Buffer,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub image_size: [u32; 2],
}
