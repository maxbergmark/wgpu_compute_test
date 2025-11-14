struct Uniforms {
    mouse: vec2<f32>,
    resolution: vec2<f32>,
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
        let dist = distance(input.uv * uniforms.resolution, uniforms.mouse);
        let min_resolution = min(uniforms.resolution.x, uniforms.resolution.y);
        let radius = 0.06 * min_resolution;
        let glow = 1.0 - exp(-pow(dist - radius, 2.0) * 0.01);
        return textureSample(image, image_sampler, input.uv) * glow;
    }
    return textureSample(image, image_sampler, input.uv);
    // return vec4<f32>(input.uv.x, input.uv.y, 1.0 - input.uv.x, 1.0);
}