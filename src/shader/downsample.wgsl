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
    scroll_delta: f32,
    _padding: f32,
};

@group(0)
@binding(2)
var<uniform> uniforms: Uniforms;


@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);
    let x = f32(coords.x) / f32(num_workgroups.x);
    let y = f32(coords.y) / f32(num_workgroups.y);
    let input_coords = vec2<i32>(i32(x * f32(uniforms.image_size.x)), i32(y * f32(uniforms.image_size.y)));
    let color = textureLoad(image, input_coords, 0);
    textureStore(output, coords, color);
}
