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
    if coords.x >= i32(uniforms.image_size.x) || coords.y >= i32(uniforms.image_size.y) {
        return;
    }

    if uniforms.xyz_2_srgb[0].x == 1.0 {
        let color = textureLoad(image, coords, 0);
        textureStore(output, coords, color);
        return;
    }


    var color = demosaic(coords);
    textureStore(output, coords, color);
}


fn in_bounds(p: vec2<i32>, size: vec2<i32>) -> bool {
    return p.x >= 0 && p.y >= 0 && p.x < size.x && p.y < size.y;
}

fn load1(p: vec2<i32>) -> f32 {
    return textureLoad(image, p, 0).r;
}

fn avg_cross(p: vec2<i32>, size: vec2<i32>, center: f32) -> f32 {
    var sum: f32 = 0.0;
    var n: i32 = 0;
    let L = p + vec2<i32>(-1, 0);
    let R = p + vec2<i32>(1, 0);
    let U = p + vec2<i32>(0, -1);
    let D = p + vec2<i32>(0, 1);
    if in_bounds(L, size) { sum += load1(L); n += 1; }
    if in_bounds(R, size) { sum += load1(R); n += 1; }
    if in_bounds(U, size) { sum += load1(U); n += 1; }
    if in_bounds(D, size) { sum += load1(D); n += 1; }
    if n == 0 { return center; }
    return sum / f32(n);
}

fn avg_diag(p: vec2<i32>, size: vec2<i32>, center: f32) -> f32 {
    var sum: f32 = 0.0;
    var n: i32 = 0;
    let UL = p + vec2<i32>(-1, -1);
    let UR = p + vec2<i32>(1, -1);
    let DL = p + vec2<i32>(-1, 1);
    let DR = p + vec2<i32>(1, 1);
    if in_bounds(UL, size) { sum += load1(UL); n += 1; }
    if in_bounds(UR, size) { sum += load1(UR); n += 1; }
    if in_bounds(DL, size) { sum += load1(DL); n += 1; }
    if in_bounds(DR, size) { sum += load1(DR); n += 1; }
    if n == 0 { return center; }
    return sum / f32(n);
}

fn avg_lr(p: vec2<i32>, size: vec2<i32>, center: f32) -> f32 {
    var sum: f32 = 0.0;
    var n: i32 = 0;
    let L = p + vec2<i32>(-1, 0);
    let R = p + vec2<i32>(1, 0);
    if in_bounds(L, size) { sum += load1(L); n += 1; }
    if in_bounds(R, size) { sum += load1(R); n += 1; }
    if n == 0 { return center; }
    return sum / f32(n);
}

fn avg_ud(p: vec2<i32>, size: vec2<i32>, center: f32) -> f32 {
    var sum: f32 = 0.0;
    var n: i32 = 0;
    let U = p + vec2<i32>(0, -1);
    let D = p + vec2<i32>(0, 1);
    if in_bounds(U, size) { sum += load1(U); n += 1; }
    if in_bounds(D, size) { sum += load1(D); n += 1; }
    if n == 0 { return center; }
    return sum / f32(n);
}

fn demosaic(coords: vec2<i32>) -> vec4<f32> {
    let size = vec2<i32>(i32(uniforms.image_size.x), i32(uniforms.image_size.y));
    let x = coords.x;
    let y = coords.y;
    let p = vec2<i32>(x, y);

    let c = load1(p);

    var r: f32 = 0.0;
    var g: f32 = 0.0;
    var b: f32 = 0.0;

    if y % 2 == 0 {
        if x % 2 == 0 {
            // R location
            r = c;
            g = avg_cross(p, size, c);
            b = avg_diag(p, size, c);
        } else {
            // G on R row
            r = avg_lr(p, size, c);
            g = c;
            b = avg_ud(p, size, c);
        }
    } else {
        if x % 2 == 0 {
            // G on B row
            r = avg_ud(p, size, c);
            g = c;
            b = avg_lr(p, size, c);
        } else {
            // B location
            r = avg_diag(p, size, c);
            g = avg_cross(p, size, c);
            b = c;
        }
    }

    return vec4<f32>(r, g, b, 1.0);
}

/*
impl Rgb {
    pub fn to_rgba(self) -> [u8; 4] {
        [
            (self.r.clamp(0.0, 1.0) * 255.0) as u8,
            (self.g.clamp(0.0, 1.0) * 255.0) as u8,
            (self.b.clamp(0.0, 1.0) * 255.0) as u8,
            255, // Alpha channel
        ]
    }
}

impl Color for Rgb {}

impl From<Srgb> for Rgb {
    fn from(srgb: Srgb) -> Self {
        Self {
            r: gamma_correct(srgb.r),
            g: gamma_correct(srgb.g),
            b: gamma_correct(srgb.b),
        }
    }
}

fn gamma_correct(v: f32) -> f32 {
    if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}


impl From<Xyz> for Srgb {
    fn from(xyz: Xyz) -> Self {
        let r_lin = 3.2406 * xyz.x - 1.5372 * xyz.y - 0.4986 * xyz.z;
        let g_lin = -0.9689 * xyz.x + 1.8758 * xyz.y + 0.0415 * xyz.z;
        let b_lin = 0.0557 * xyz.x - 0.2040 * xyz.y + 1.0570 * xyz.z;
        Self {
            r: r_lin,
            g: g_lin,
            b: b_lin,
        }
    }
}

*/