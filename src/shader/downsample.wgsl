@group(0)
@binding(0)
var image: texture_2d<f32>;

@group(0)
@binding(1)
var output: texture_storage_2d<rgba32float, write>;

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

    let normalized = vec2<f32>(coords) / vec2<f32>(uniforms.output_size);
    let input_coords = cropped_coords(normalized);

    let color = textureLoad(image, input_coords, 0);
    textureStore(output, coords, color);
}

fn cropped_coords(normalized: vec2<f32>) -> vec2<i32> {
    let top = f32(uniforms.crops.x);
    let right = f32(uniforms.crops.y);
    let bottom = f32(uniforms.crops.z);
    let left = f32(uniforms.crops.w);

    let sample_x = normalized.x * (uniforms.image_size.x - right - left) + left;
    let sample_y = normalized.y * (uniforms.image_size.y - top - bottom) + top;
    return vec2<i32>(i32(sample_x), i32(sample_y));
}