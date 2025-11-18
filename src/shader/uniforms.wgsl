struct Uniforms {
    cam_2_xyz: mat3x4<f32>,
    xyz_2_srgb: mat3x3<f32>,
    whitelevels: vec4<f32>,
    blacklevels: vec4<f32>,
    crops: vec4<u32>,
    mouse_pos: vec2<f32>,
    window_size: vec2<f32>,
    image_size: vec2<f32>,
    output_size: vec2<f32>,
    scroll_delta: f32,
    _padding: vec3<f32>,
};

@group(1)
@binding(0)
var<uniform> uniforms: Uniforms;