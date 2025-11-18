@group(0)
@binding(0)
var image: texture_2d<f32>;

@group(0)
@binding(1)
var output: texture_storage_2d<rgba8unorm, write>;

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
    exposure: f32,
    contrast: f32,
    // _padding: vec3<f32>,
};

@group(1)
@binding(0)
var<uniform> uniforms: Uniforms;


@compute
@workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);

    // Bounds check (important if image size isn’t a multiple of 16)
    if coords.x >= i32(uniforms.output_size.x) || coords.y >= i32(uniforms.output_size.y) {
        return;
    }

    var color = textureLoad(image, coords, 0);
    color.a = calculate_opacity(coords);
    textureStore(output, coords, color);
}

fn calculate_opacity(coords: vec2<i32>) -> f32 {
    let p = vec2<f32>(coords);
    let radius = 50.0;
    let picture_half_size = vec2<f32>(uniforms.output_size.xy) * 0.5;
    let min_offset = calculate_rounded_offset((p - picture_half_size), picture_half_size - radius, radius);
    return 1.0 - smoothstep(0.0, radius, min_offset);
}

fn calculate_rounded_offset(
    p: vec2<f32>,
    picture_half_size: vec2<f32>,
    corner_radius: f32
) -> f32 {
    let d = abs(p) - (picture_half_size - vec2<f32>(corner_radius));
    return length(max(d, vec2<f32>(0.0))) - corner_radius;
}