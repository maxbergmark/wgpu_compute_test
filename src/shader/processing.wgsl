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

    var color = textureLoad(image, coords, 0);
    if uniforms.xyz_2_srgb[0].x == 1.0 {
        textureStore(output, coords, color);
        return;
    }

    color = clamp(color, vec4<f32>(0.0), uniforms.whitelevels);
    color -= uniforms.blacklevels;
    color = max(color, vec4<f32>(0.0));
    var xyz = color.rgba * uniforms.cam_2_xyz;
    xyz *= pow(2.0, uniforms.exposure);
    xyz = contrast(xyz, uniforms.contrast);

    var srgb_linear = uniforms.xyz_2_srgb * xyz.rgb;
    let srgb_gamma = gamma(srgb_linear);

    textureStore(output, coords, vec4<f32>(srgb_gamma, 1.0));
    // textureStore(output, coords, color);
}

fn contrast(v: vec3<f32>, value: f32) -> vec3<f32> {
    return vec3<f32>(
        map_contrast(v.r, value),
        map_contrast(v.g, value),
        map_contrast(v.b, value)
    );
}

fn map_contrast(channel: f32, value: f32) -> f32 {
    let factor = (259.0 * (value + 255.0)) / (255.0 * (259.0 - value));
    return factor * (channel - 0.5) + 0.5;
}


fn gamma(v: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        gamma_correct(v.r),
        gamma_correct(v.g),
        gamma_correct(v.b)
    );
}

fn gamma_correct(v: f32) -> f32 {
    if v <= 0.0031308 {
        return 12.92 * v;
    } else {
        return 1.055 * pow(v, 1.0 / 2.4) - 0.055;
    }
}