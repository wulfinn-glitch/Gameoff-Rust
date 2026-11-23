#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct GridParams {
    time: f32,
    impact_count: u32,
    bullet_count: u32,
    _pad: f32,
    player: vec4<f32>,
    impacts: array<vec4<f32>, 32>,   // xy = pos, z = age, w = amplitude
    impact_dirs: array<vec4<f32>, 32>, // xy = dir, z = speed, w = half-angle
    bullets: array<vec4<f32>, 32>,   // xy = pos
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> grid: GridParams;

const CELL: f32 = 2.0;

fn hash21(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(123.34, 456.21));
    q = q + dot(q, q + 45.32);
    return fract(q.x * q.y);
}

fn hue2rgb(p: f32, q: f32, t_in: f32) -> f32 {
    var t = t_in;
    if (t < 0.0) { t = t + 1.0; }
    if (t > 1.0) { t = t - 1.0; }
    if (t < 1.0 / 6.0) { return p + (q - p) * 6.0 * t; }
    if (t < 0.5) { return q; }
    if (t < 2.0 / 3.0) { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    return p;
}

fn hsl(h: f32, s: f32, l: f32) -> vec3<f32> {
    let q = select(l * (1.0 + s), l + s - l * s, l < 0.5);
    let p = 2.0 * l - q;
    let hk = h / 360.0;
    return vec3<f32>(
        hue2rgb(p, q, hk + 1.0 / 3.0),
        hue2rgb(p, q, hk),
        hue2rgb(p, q, hk - 1.0 / 3.0),
    );
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let world = mesh.world_position.xy;
    let cell_id = floor(world / CELL);
    let center = (cell_id + 0.5) * CELL;

    // Base breathing.
    var energy = 0.04 * max(0.0, sin(cell_id.x * 0.3 + cell_id.y * 0.18 - grid.time * 1.3));
    var warp = vec2<f32>(0.0);

    // Shockwaves.
    for (var i = 0u; i < grid.impact_count; i = i + 1u) {
        let imp = grid.impacts[i];
        let wave = grid.impact_dirs[i];

        let age = imp.z;
        let amp = imp.w;
        let dir = wave.xy;
        let speed = wave.z;
        let half_angle = wave.w;

        let delta = center - imp.xy;

        // Split the offset into "forward along the attack" and "sideways".
        let along = dot(delta, dir);
        let lateral = length(delta - dir * along);

        // Cone that widens with distance from the impact.
        let cone_width = 12.0 + max(along, 0.0) * tan(half_angle);
        let angular = exp(-(lateral * lateral) / (2.0 * cone_width * cone_width))
                    * smoothstep(-8.0, 16.0, along);

        // Front travels forward along the direction.
        let ring = along - age * speed;
        let wd = 22.0 + age * 14.0;
        let w = exp(-(ring * ring) / (2.0 * wd * wd)) * exp(-age * 2.3) * amp * angular;

        // Small radial flash at the hit point.
        let dist = length(delta);
        let core = exp(-(dist * dist) / 900.0) * exp(-age * 10.0) * amp;

        energy = energy + w + core;

        // Push the grid forward, not outward.
        warp = warp + dir * w * 5.0;
    }

    // Bullet glow.
    for (var i = 0u; i < grid.bullet_count; i = i + 1u) {
        let d = center - grid.bullets[i].xy;
        energy = energy + exp(-dot(d, d) / 1500.0) * 0.95;
    }

    // Player glow.
    let pd = center - grid.player.xy;
    energy = energy + 0.22 * exp(-dot(pd, pd) / 9000.0);

    // Twinkle.
    let r = hash21(cell_id);
    if (r > 0.965) {
        energy = energy + 0.22 * (0.5 + 0.5 * sin(grid.time * 3.0 + r * 60.0));
    }

    // Cell mask with a gap, sampled at the warped position.
    let p = world + warp;
    let f = fract(p / CELL);
    let edge = max(abs(f.x - 0.5), abs(f.y - 0.5));
    let mask = 1.0 - smoothstep(0.4, 0.5, edge);

    // Vignette around the screen center (world origin for now).
    let v = clamp(1.25 - length(world / vec2<f32>(400.0, 400.0)), 0.0, 1.0);

    let e = min(energy, 1.4);
    let col = hsl(195.0 + e * 75.0, 0.9, 0.42 + min(e, 1.0) * 0.38);
    let glow = col * (0.09 + 0.9 * min(e, 1.0)) * mask * v;

    let base = vec3<f32>(0.02, 0.025, 0.05);
    return vec4<f32>(base + glow, 1.0);
}