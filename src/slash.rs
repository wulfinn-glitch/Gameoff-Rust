use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, Mesh2d, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;

/// A single, data-driven slash/arc. Change these fields to change the shape.
#[derive(Clone, Copy, Debug)]
pub struct SlashShape {
    /// Distance from the player to the middle of the arc.
    pub radius: f32,
    /// Core band width at the middle (the glow is a multiple of this).
    pub thickness: f32,
    /// 0 = constant width, 1 = pointed tips.
    pub taper: f32,
    /// Total sweep in radians. `TAU` makes a full ring.
    pub arc: f32,
    /// How far the middle bulges out from the tips (the "parabola").
    pub bow: f32,
    /// Extra rotation applied to the whole arc.
    pub tilt: f32,
    /// +1 or -1: which way the arc sweeps.
    pub flip: f32,
    /// Radius added over the effect's lifetime (expanding rings).
    pub grow: f32,
    /// Rotation added over the effect's lifetime.
    pub spin: f32,
    /// 0 = evenly lit ring, 1 = white-hot leading edge with a fading trail.
    pub hot: f32,
    pub color: Color,
    pub segments: usize,
}

impl SlashShape {
    /// First combo hit: a fast horizontal swipe.
    pub fn slash() -> Self {
        Self {
            radius: 70.0,
            thickness: 6.0,
            taper: 0.9,
            arc: 2.1,
            bow: 16.0,
            tilt: 0.0,
            flip: 1.0,
            grow: 14.0,
            spin: 0.0,
            hot: 1.0,
            color: Color::srgb(0.35, 0.8, 1.0),
            segments: 32,
        }
    }

    /// Second combo hit: mirrored, slightly angled.
    pub fn backhand() -> Self {
        Self {
            tilt: -0.3,
            flip: -1.0,
            color: Color::srgb(0.6, 0.9, 1.0),
            ..Self::slash()
        }
    }

    /// Third combo hit: wider, heavier, brighter.
    pub fn heavy() -> Self {
        Self {
            radius: 82.0,
            thickness: 11.0,
            taper: 0.72,
            arc: 2.7,
            bow: 28.0,
            grow: 20.0,
            hot: 1.0,
            color: Color::srgb(0.75, 0.92, 1.0),
            segments: 40,
            ..Self::slash()
        }
    }

    /// Shield arc shown while the parry window is active.
    pub fn parry_arc() -> Self {
        Self {
            radius: 52.0,
            thickness: 3.5,
            taper: 0.5,
            arc: 2.4,
            bow: 6.0,
            tilt: 0.0,
            flip: 1.0,
            grow: 12.0,
            spin: 0.0,
            hot: 1.0,
            color: Color::srgb(0.25, 1.0, 0.9),
            segments: 32,
        }
    }

    /// Ring pop when a bullet is actually parried/blocked.
    pub fn parry_pop() -> Self {
        Self {
            radius: 22.0,
            thickness: 3.0,
            taper: 0.0,
            arc: TAU,
            bow: 0.0,
            tilt: 0.0,
            flip: 1.0,
            grow: 48.0,
            spin: 0.0,
            hot: 0.0,
            color: Color::srgb(0.45, 1.0, 0.95),
            segments: 56,
        }
    }

    /// Clean, full-circle pressure wave for perfect parries and other big
    /// impacts. Unlike [`Self::parry_pop`] it doesn't sweep: it is a whole ring
    /// from the first frame and just expands, thins and fades.
    pub fn shockwave() -> Self {
        Self {
            radius: 28.0,
            thickness: 8.0,
            taper: 0.0,
            arc: TAU,
            bow: 0.0,
            tilt: 0.0,
            flip: 1.0,
            grow: 260.0,
            spin: 0.0,
            hot: 1.0,
            color: Color::srgb(0.45, 1.0, 0.95),
            segments: 96,
        }
    }

    /// Small expanding ring at the point an attack lands.
    pub fn hit_spark() -> Self {
        Self {
            radius: 6.0,
            thickness: 2.5,
            taper: 0.0,
            arc: TAU,
            bow: 0.0,
            tilt: 0.0,
            flip: 1.0,
            grow: 30.0,
            spin: 0.0,
            hot: 0.0,
            color: Color::srgb(1.0, 0.95, 0.8),
            segments: 32,
        }
    }

    /// Pick a shape for a combo step (0-based).
    pub fn for_step(step: u8) -> Self {
        match step {
            0 => Self::slash(),
            1 => Self::backhand(),
            _ => Self::heavy(),
        }
    }
}

/// Shared material for every slash. All colour/alpha lives in vertex colours.
#[derive(Resource)]
pub struct SlashAssets {
    pub material: Handle<ColorMaterial>,
}

pub fn setup_slash_assets(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(SlashAssets {
        material: materials.add(Color::WHITE),
    });
}

#[derive(Component)]
pub struct Slash {
    pub shape: SlashShape,
    pub timer: Timer,
    pub mesh: Handle<Mesh>,
}

/// A whole-circle pressure wave. It is complete on the first frame, so it needs
/// its own timing curve instead of [`Slash`]'s tip-to-tip reveal.
#[derive(Component)]
pub struct Shockwave {
    pub shape: SlashShape,
    pub timer: Timer,
    pub mesh: Handle<Mesh>,
    /// Fraction of the lifetime spent at full brightness before the fade starts.
    pub hold: f32,
}

/// Spawn a slash at `pos`, aimed along `facing` (world space, so it stays put).
pub fn spawn_slash(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &SlashAssets,
    pos: Vec2,
    facing: Vec2,
    shape: SlashShape,
    lifetime: f32,
) {
    let facing = if facing.length_squared() > 1e-6 {
        facing.normalize()
    } else {
        Vec2::Y
    };

    let mesh = meshes.add(Mesh::new(
        PrimitiveTopology::TriangleList,
        // Keep the mesh in the main world so we can rewrite it every frame.
        RenderAssetUsages::default(),
    ));

    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(assets.material.clone()),
        Transform::from_xyz(pos.x, pos.y, 5.0)
            .with_rotation(Quat::from_rotation_arc_2d(Vec2::Y, facing)),
        Slash {
            shape,
            timer: Timer::from_seconds(lifetime, TimerMode::Once),
            mesh,
        },
    ));
}

/// Spawn a full-circle shockwave centred on `pos`. `hold` is the fraction of
/// `lifetime` the ring stays bright before fading (0.3 is a good default).
pub fn spawn_shockwave(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &SlashAssets,
    pos: Vec2,
    shape: SlashShape,
    lifetime: f32,
    hold: f32,
) {
    let mesh = meshes.add(Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    ));

    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(assets.material.clone()),
        // Behind the contact burst but in front of the player sprite.
        Transform::from_xyz(pos.x, pos.y, 4.0),
        Shockwave {
            shape,
            timer: Timer::from_seconds(lifetime, TimerMode::Once),
            mesh,
            hold: hold.clamp(0.0, 0.95),
        },
    ));
}

/// Deterministic per-cluster randomness, so we don't need an RNG dependency.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        // Avoid the all-zero state for xorshift.
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next_unit(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        // Top 24 bits -> [0, 1), exactly representable as f32.
        ((x >> 40) as f32) / ((1u32 << 24) as f32)
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next_unit()
    }
}

/// Spawn several overlapping arcs in slightly different shapes so one swing
/// reads as a messy, layered slash instead of a single clean arc.
pub fn spawn_slash_cluster(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &SlashAssets,
    pos: Vec2,
    facing: Vec2,
    base: SlashShape,
    lifetime: f32,
    count: usize,
    seed: u64,
) {
    let facing = if facing.length_squared() > 1e-6 {
        facing.normalize()
    } else {
        Vec2::Y
    };

    let mut rng = Rng::new(seed);
    let count = count.max(1);

    for i in 0..count {
        // Later arcs fan out a little further and thinner.
        let spread = i as f32 / count as f32;

        let mut shape = base;

        shape.radius += rng.range(-10.0, 12.0) + spread * 10.0;
        shape.thickness =
            (shape.thickness * rng.range(0.55, 1.15) * (1.0 - spread * 0.4)).max(1.0);
        shape.bow += rng.range(-12.0, 20.0);
        shape.tilt += rng.range(-0.22, 0.22);
        shape.arc += rng.range(-0.4, 0.4);
        shape.grow *= rng.range(0.6, 1.5);
        shape.spin += rng.range(-0.3, 0.3);

        // Occasionally mirror the sweep for a cross-cut look.
        if rng.next_unit() < 0.3 {
            shape.flip = -shape.flip;
        }

        // Vary brightness so the layers separate.
        let bright = rng.range(0.7, 1.25);
        let lin = shape.color.to_linear();
        shape.color = Color::linear_rgba(
            (lin.red * bright).min(1.0),
            (lin.green * bright).min(1.0),
            (lin.blue * bright).min(1.0),
            lin.alpha,
        );

        // Rotate and jitter the whole arc around the facing axis.
        let angle = rng.range(-0.14, 0.14);
        let (sin, cos) = angle.sin_cos();
        let dir = Vec2::new(
            facing.x * cos - facing.y * sin,
            facing.x * sin + facing.y * cos,
        );
        let side = Vec2::new(-facing.y, facing.x);
        let offset = side * rng.range(-7.0, 7.0);

        // Slightly different lifetimes desync the fades.
        let life = lifetime * rng.range(0.75, 1.15);
        spawn_slash(commands, meshes, assets, pos + offset, dir, shape, life);
    }
}

pub fn animate_slashes(
    time: Res<Time>,
    mut commands: Commands,
    mut slashes: Query<(Entity, &mut Slash)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut slash) in &mut slashes {
        slash.timer.tick(time.delta());
        let t = slash.timer.fraction();

        // Sweep fast, hold for a beat, then fade.
        let reveal = smoothstep(0.0, 0.40, t);
        let fade = 1.0 - smoothstep(0.52, 1.0, t);
        // Snap: briefly fatter at spawn, then settle to the thin blade.
        let punch = 1.0 + 0.6 * (1.0 - smoothstep(0.0, 0.18, t));

        let shape = SlashShape {
            radius: slash.shape.radius + slash.shape.grow * t,
            tilt: slash.shape.tilt + slash.shape.spin * t,
            ..slash.shape
        };

        if let Some(mesh) = meshes.get_mut(&slash.mesh) {
            *mesh = build_slash_mesh(&shape, reveal, fade, punch);
        }

        if slash.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Fast at first, easing out as the front loses energy.
fn shockwave_expand(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Drive the pressure wave: expand, thin, cool from white-hot to the base
/// colour, then fade out.
pub fn animate_shockwaves(
    time: Res<Time>,
    mut commands: Commands,
    mut waves: Query<(Entity, &mut Shockwave)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut wave) in &mut waves {
        wave.timer.tick(time.delta());
        let t = wave.timer.fraction();

        let shape = SlashShape {
            radius: wave.shape.radius + wave.shape.grow * shockwave_expand(t),
            // The band thins as it spreads, like a real pressure front.
            thickness: (wave.shape.thickness * (1.0 - 0.7 * t)).max(1.5),
            // White-hot on spawn, settling into the palette colour.
            hot: 1.0 - smoothstep(0.0, 0.5, t),
            ..wave.shape
        };

        // No reveal: the ring is complete immediately.
        let fade = 1.0 - smoothstep(wave.hold, 1.0, t);

        if let Some(mesh) = meshes.get_mut(&wave.mesh) {
            *mesh = build_slash_mesh(&shape, 1.0, fade, 1.0);
        }

        if wave.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if (edge1 - edge0).abs() < 1e-6 {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Build the mesh: a wide soft glow ribbon plus a thin hot core ribbon.
///
/// `reveal` (0..1) trims the arc from one tip toward the other, `fade` scales
/// the overall alpha, and `punch` scales the band width for the spawn snap.
fn build_slash_mesh(shape: &SlashShape, reveal: f32, fade: f32, punch: f32) -> Mesh {
    let segments = shape.segments.max(2);
    let reveal = reveal.clamp(0.0, 1.0);
    let n = ((segments as f32 * reveal).ceil() as usize).max(1);

    // Five samples across each ribbon (soft falloff toward the edges).
    const ACROSS: usize = 5;
    const RIBBONS: usize = 2;
    let stride = ACROSS * RIBBONS;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((n + 1) * stride);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity((n + 1) * stride);
    let mut indices: Vec<u32> = Vec::with_capacity(n * (ACROSS - 1) * RIBBONS * 6);

    let lin = shape.color.to_linear();
    let base = [lin.red, lin.green, lin.blue];

    let across = [-1.0_f32, -0.5, 0.0, 0.5, 1.0];
    let glow_falloff = [0.0_f32, 0.35, 1.0, 0.35, 0.0];
    let core_falloff = [0.0_f32, 0.85, 1.0, 0.85, 0.0];

    let hot = shape.hot.clamp(0.0, 1.0);

    for j in 0..=n {
        let s = (j as f32 / n as f32) * reveal;
        let k = s * 2.0 - 1.0; // -1 at one tip, +1 at the other

        let theta = (-shape.arc * 0.5 + shape.arc * s) * shape.flip + shape.tilt;
        let dir = Vec2::new(-theta.sin(), theta.cos());

        // Parabolic bulge: middle at `radius`, tips pulled back by `bow`.
        let r = shape.radius - shape.bow * k * k;
        let center = dir * r;

        // Taper the band toward the tips, plus the spawn-width snap.
        let half_w = (0.5 * shape.thickness * (1.0 - shape.taper * k * k)).max(0.0) * punch;

        // Fade the trail behind the leading edge so it reads as a swipe.
        let from_head = (reveal - s).max(0.0);
        let trail = 1.0 - smoothstep(0.25, 1.6, from_head) * hot;
        let head = (1.0 - smoothstep(0.0, 0.15, from_head)) * hot;

        // Bright white-hot leading edge, base colour in the body.
        let core = [
            base[0] + (1.0 - base[0]) * head,
            base[1] + (1.0 - base[1]) * head,
            base[2] + (1.0 - base[2]) * head,
        ];

        let glow_alpha = 0.22 * trail * fade;
        let core_alpha = (0.85 + 0.15 * head) * trail * fade;

        // Glow ribbon: wide and soft, slightly behind the core.
        for (v, f) in across.iter().zip(glow_falloff.iter()) {
            let p = center + dir * (half_w * 2.8 * v);
            positions.push([p.x, p.y, 0.0]);
            colors.push([base[0], base[1], base[2], glow_alpha * f]);
        }
        // Core ribbon: thin and bright, nudged toward the camera.
        for (v, f) in across.iter().zip(core_falloff.iter()) {
            let p = center + dir * (half_w * v);
            positions.push([p.x, p.y, 0.02]);
            colors.push([core[0], core[1], core[2], core_alpha * f]);
        }
    }

    for j in 0..n {
        for ribbon in 0..RIBBONS {
            let offset = ribbon * ACROSS;
            for row in 0..ACROSS - 1 {
                let a = (j * stride + offset + row) as u32;
                let b = (j * stride + offset + row + 1) as u32;
                let c = ((j + 1) * stride + offset + row) as u32;
                let d = ((j + 1) * stride + offset + row + 1) as u32;
                indices.extend_from_slice(&[a, b, c, b, d, c]);
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(colors),
    );
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_mesh(shape: &SlashShape, reveal: f32, fade: f32, punch: f32) {
        let mesh = build_slash_mesh(shape, reveal, fade, punch);

        let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            VertexAttributeValues::Float32x3(p) => p,
            _ => panic!("positions should be Float32x3"),
        };
        let colors = match mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap() {
            VertexAttributeValues::Float32x4(c) => c,
            _ => panic!("colors should be Float32x4"),
        };
        assert_eq!(positions.len(), colors.len());

        let count = positions.len();
        match mesh.indices().unwrap() {
            Indices::U32(idx) => {
                assert!(idx.iter().all(|&i| (i as usize) < count), "index out of bounds");
                assert_eq!(idx.len() % 3, 0, "triangle list needs a multiple of 3");
            }
            Indices::U16(_) => panic!("expected U32 indices"),
        }
    }

    #[test]
    fn shockwave_front_only_moves_outward() {
        assert_eq!(shockwave_expand(0.0), 0.0);
        assert!((shockwave_expand(1.0) - 1.0).abs() < 1e-6);

        let mut previous = 0.0;
        for step in 0..=10 {
            let v = shockwave_expand(step as f32 / 10.0);
            assert!(v >= previous, "front must never move back inward");
            previous = v;
        }

        // Ease-out: past halfway in time, most of the distance is covered.
        assert!(shockwave_expand(0.5) > 0.8);
    }

    #[test]
    fn mesh_is_valid_across_the_lifecycle() {
        let shapes = [
            SlashShape::slash(),
            SlashShape::backhand(),
            SlashShape::heavy(),
            SlashShape::parry_arc(),
            SlashShape::parry_pop(),
            SlashShape::shockwave(),
            SlashShape::hit_spark(),
        ];

        for shape in shapes {
            for t in [0.0_f32, 0.1, 0.25, 0.5, 0.75, 1.0] {
                let reveal = smoothstep(0.0, 0.40, t);
                let fade = 1.0 - smoothstep(0.52, 1.0, t);
                let punch = 1.0 + 0.6 * (1.0 - smoothstep(0.0, 0.18, t));
                check_mesh(&shape, reveal, fade, punch);
            }
        }
    }
}
