// Copies the offscreen 3D target into the egui pass. Colors are in gamma space; the linear entry point is used
// when the window surface is sRGB.

struct ScreenOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// One oversized triangle that covers the whole target.
fn fullscreen(index: u32) -> ScreenOut {
    var out: ScreenOut;
    let x = f32((index << 1u) & 2u);
    let y = f32(index & 2u);
    out.clip = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

@group(0) @binding(0) var blit_texture: texture_2d<f32>;
@group(0) @binding(1) var blit_sampler: sampler;

@vertex
fn vs_blit(@builtin(vertex_index) index: u32) -> ScreenOut {
    return fullscreen(index);
}

@fragment
fn fs_blit_gamma(input: ScreenOut) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(blit_texture, blit_sampler, input.uv).rgb, 1.0);
}

@fragment
fn fs_blit_linear(input: ScreenOut) -> @location(0) vec4<f32> {
    let rgb = textureSample(blit_texture, blit_sampler, input.uv).rgb;
    return vec4<f32>(pow(rgb, vec3<f32>(2.2)), 1.0);
}
