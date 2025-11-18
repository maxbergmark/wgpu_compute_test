@group(0)
@binding(0)
var image: texture_2d<f32>;

@group(0)
@binding(1)
var output: texture_storage_2d<rgba8unorm, write>;

struct Uniforms {
    mouse: vec2<f32>,
    window_size: vec2<f32>,
    image_size: vec2<f32>,
    output_size: vec2<f32>,
    scroll_delta: f32,
    _padding: f32,
};

@group(0)
@binding(2)
var<uniform> uniforms: Uniforms;


@compute
@workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);
    // Bounds check (important if image size isn’t a multiple of 16)
    if coords.x >= i32(uniforms.output_size.x) || coords.y >= i32(uniforms.output_size.y) {
        return;
    }

    let x = f32(coords.x) / f32(uniforms.output_size.x);
    let y = f32(coords.y) / f32(uniforms.output_size.y);
    let input_coords = vec2<i32>(i32(x * f32(uniforms.image_size.x)), i32(y * f32(uniforms.image_size.y)));
    let color = textureLoad(image, input_coords, 0);
    textureStore(output, coords, color);
}
