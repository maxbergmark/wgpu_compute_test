@group(0)
@binding(0)
var image: texture_2d<f32>;

@group(0)
@binding(1)
var output: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);
    let offset_top_left = vec2<f32>(coords);
    let offset_bottom_right = vec2<f32>(num_workgroups.xy) - offset_top_left;
    let radius = 50.0;
    // let min_offset = calculate_min_offset(offset_top_left, offset_bottom_right);
    let picture_half_size = vec2<f32>(num_workgroups.xy) * 0.5;
    let min_offset = calculate_rounded_offset((offset_top_left - picture_half_size), picture_half_size - radius, radius);
    let opacity = smoothstep(0.0, radius, min_offset);
    var color = textureLoad(image, coords, 0);
    color.a = 1.0 - opacity;
    textureStore(output, coords, color);
}

fn calculate_min_offset(offset_top_left: vec2<f32>, offset_bottom_right: vec2<f32>) -> f32 {
    let offset_x = min(offset_top_left.x, offset_bottom_right.x);
    let offset_y = min(offset_top_left.y, offset_bottom_right.y);
    return min(offset_x, offset_y);
}

fn calculate_rounded_offset(
    p: vec2<f32>,
    picture_half_size: vec2<f32>,
    corner_radius: f32
) -> f32 {
    let d = abs(p) - (picture_half_size - vec2<f32>(corner_radius));
    return length(max(d, vec2<f32>(0.0))) - corner_radius;
}