struct Uniforms {
    mouse: vec2<f32>,
    window_size: vec2<f32>,
    image_size: vec2<f32>,
    scroll_delta: f32,
    _padding: f32,
};

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(1)
var image: texture_2d<f32>;

@group(0)
@binding(2)
var image_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), // bottom left
        vec2<f32>(1.0, -1.0), // bottom right
        vec2<f32>(-1.0, 1.0), // top left
        vec2<f32>(-1.0, 1.0), // top left
        vec2<f32>(1.0, -1.0), // bottom right
        vec2<f32>(1.0, 1.0)  // top right
    );

    let pos = positions[vertex_index];
    var out: VertexOutput;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + vec2<f32>(0.5, 0.5); // Map from [-1,1] to [0,1]
    out.uv.y = 1.0 - out.uv.y; // Flip Y for texture coordinates
    return out;
}

struct FragInput {
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(input: FragInput) -> @location(0) vec4<f32> {
    if uniforms.mouse.x >= 0.0 && uniforms.mouse.y >= 0.0 {
        let dist = distance(input.uv * uniforms.window_size, uniforms.mouse);
        let min_resolution = min(uniforms.window_size.x, uniforms.window_size.y);
        let radius = 0.1 * min_resolution + uniforms.scroll_delta;
        let sdf = circle_sdf(input.uv * uniforms.window_size, uniforms.mouse, radius);
        let glow = 1.0 - smoothstep(0.0, 1.0, 1.0 - sdf / radius);
        var coords = input.uv;
        // enlarge the area affected by the glow
        let mouse_uv = uniforms.mouse / uniforms.window_size;
        coords = mix(coords, mouse_uv, 1.0 - (0.5 + glow * 0.5));
        var color = textureSample(image, image_sampler, coords);
        // color.r = mix(1.0 - color.r, color.r, glow);
        // color.g = mix(1.0 - color.g, color.g, glow);
        // color.b = mix(1.0 - color.b, color.b, glow);
        return color;
    }
    return textureSample(image, image_sampler, input.uv);
}

fn circle_sdf(p: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return length(p - center) - radius;
}