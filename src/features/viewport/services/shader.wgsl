// Shaders of the 3D view. Colors are authored and written in gamma (display) space; the blit converts when the
// window surface is sRGB.

struct Globals {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    light_dir: vec4<f32>,   // unit vector pointing towards the sun
    sky_top: vec4<f32>,
    sky_bottom: vec4<f32>,
    params: vec4<f32>,      // x: distance where the grid has faded out
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_scene(input: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip = globals.view_proj * vec4<f32>(input.position, 1.0);
    out.world = input.position;
    out.color = input.color;
    return out;
}

@fragment
fn fs_scene(input: VertexOut) -> @location(0) vec4<f32> {
    // Flat normal from screen-space derivatives (no normals are stored), facing the viewer so
    // inconsistent triangle winding never shows a black back face.
    let c = cross(dpdx(input.world), dpdy(input.world));
    var n = c / max(length(c), 1e-12);
    let to_eye = normalize(globals.eye.xyz - input.world);
    if (dot(n, to_eye) < 0.0) {
        n = -n;
    }
    let sun = max(dot(n, globals.light_dir.xyz), 0.0);
    let fill = max(dot(n, -globals.light_dir.xyz * vec3<f32>(1.0, 1.0, 0.0)), 0.0) * 0.12;
    let hemisphere = mix(vec3<f32>(0.30, 0.29, 0.28), vec3<f32>(0.52, 0.58, 0.68), n.z * 0.5 + 0.5);
    let light = hemisphere + vec3<f32>(1.0, 0.97, 0.90) * (sun * 0.72 + fill);
    return vec4<f32>(input.color.rgb * light, input.color.a);
}

@fragment
fn fs_line(input: VertexOut) -> @location(0) vec4<f32> {
    let fade = 1.0 - smoothstep(0.35, 1.0, distance(globals.eye.xyz, input.world) / globals.params.x);
    return vec4<f32>(input.color.rgb, input.color.a * fade);
}

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

@vertex
fn vs_sky(@builtin(vertex_index) index: u32) -> ScreenOut {
    var out = fullscreen(index);
    out.clip.z = 1.0;
    return out;
}

@fragment
fn fs_sky(input: ScreenOut) -> @location(0) vec4<f32> {
    return mix(globals.sky_top, globals.sky_bottom, clamp(input.uv.y, 0.0, 1.0));
}
