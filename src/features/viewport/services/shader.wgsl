// Shaders of the 3D view. Colors are authored and written in gamma (display) space; the blit converts when the
// window surface is sRGB.

struct Globals {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    light_dir: vec4<f32>,   // unit vector pointing towards the sun
    sky_top: vec4<f32>,
    horizon: vec4<f32>,     // sky color at the horizon, also the color of the haze
    params: vec4<f32>,      // x: distance where the grid has faded out
    cam_right: vec4<f32>,
    cam_up: vec4<f32>,
    cam_forward: vec4<f32>,
    lens: vec4<f32>,        // x: tan(fov_y / 2), y: aspect, z: haze start, w: haze end (meters)
    ground_far: vec4<f32>,  // color of the far ground below the horizon
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
    // Distant geometry (the edge of the ground plane above all) melts into the horizon color, so there is no
    // visible seam where the ground ends.
    let haze = smoothstep(globals.lens.z, globals.lens.w, distance(globals.eye.xyz, input.world));
    let lit = input.color.rgb * light;
    return vec4<f32>(mix(lit, globals.horizon.rgb, haze), input.color.a);
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
    // The gradient follows the real horizon: each pixel's view ray decides its color, so tilting the camera
    // moves the horizon instead of leaving a fixed grey band in the middle of the screen.
    let ndc = vec2<f32>(input.uv.x * 2.0 - 1.0, 1.0 - input.uv.y * 2.0);
    let ray = normalize(
        globals.cam_forward.xyz
            + globals.cam_right.xyz * (ndc.x * globals.lens.x * globals.lens.y)
            + globals.cam_up.xyz * (ndc.y * globals.lens.x)
    );
    if (ray.z >= 0.0) {
        return mix(globals.horizon, globals.sky_top, smoothstep(0.0, 0.7, ray.z));
    }
    // Below the horizon the ground goes on beyond the edge of the ground plane: it is the same grass, hazed by
    // the same distance rule as the plane, so the plane's edge never shows as a band.
    let distance_to_ground = abs(globals.eye.z) / max(-ray.z, 1e-4);
    let haze = smoothstep(globals.lens.z, globals.lens.w, distance_to_ground);
    return mix(globals.ground_far, globals.horizon, haze);
}
