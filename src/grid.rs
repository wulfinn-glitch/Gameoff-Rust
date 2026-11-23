use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{Material2d, Material2dPlugin};

use crate::bullet::Bullet;
use crate::player::Player;

pub const MAX_IMPACTS: usize = 32;
pub const MAX_BULLETS: usize = 32;
const IMPACT_LIFETIME: f32 = 2.6;

#[derive(ShaderType, Debug, Clone)]
pub struct GridParams {
    pub time: f32,
    pub impact_count: u32,
    pub bullet_count: u32,
    pub _pad: f32,
    pub player: Vec4,
    pub impacts: [Vec4; MAX_IMPACTS],
    pub impact_dirs: [Vec4; MAX_IMPACTS],
    pub bullets: [Vec4; MAX_BULLETS],
}

impl Default for GridParams {
    fn default() -> Self {
        Self {
            time: 0.0,
            impact_count: 0,
            bullet_count: 0,
            _pad: 0.0,
            player: Vec4::ZERO,
            impacts: [Vec4::ZERO; MAX_IMPACTS],
            impact_dirs: [Vec4::ZERO; MAX_IMPACTS],
            bullets: [Vec4::ZERO; MAX_BULLETS],
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(0)]
    pub params: GridParams,
}

impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid.wgsl".into()
    }
}

#[derive(Resource)]
pub struct GridHandle(pub Handle<GridMaterial>);

#[derive(Clone, Copy)]
pub struct GridImpact {
    pub pos: Vec2,
    pub dir: Vec2,
    pub start: f32,
    pub amp: f32,
    pub speed: f32,
    pub half_angle: f32,
}

#[derive(Resource, Default)]
pub struct GridFx {
    pub now: f32,
    pub impacts: Vec<GridImpact>,
}

impl GridFx {
    /// `amp`: ~0.3 tap, ~0.8 bullet, ~1.2 parry, ~1.7 blast.
    /// `dir` is the direction the wave travels; `half_angle` is the cone half-width in radians.
    pub fn flash(&mut self, pos: Vec2, dir: Vec2, amp: f32, speed: f32, half_angle: f32) {
        self.impacts.push(GridImpact {
            pos,
            dir: dir.normalize_or_zero(),
            start: self.now,
            amp,
            speed,
            half_angle: half_angle.clamp(0.1, 1.2),
        });
        if self.impacts.len() > MAX_IMPACTS {
            self.impacts.remove(0);
        }
    }
}

pub fn spawn_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GridMaterial>>,
) {
    let handle = materials.add(GridMaterial {
        params: GridParams::default(),
    });

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(4000.0, 4000.0))),
        MeshMaterial2d(handle.clone()),
        Transform::from_xyz(0.0, 0.0, -100.0),
    ));

    commands.insert_resource(GridHandle(handle));
}

pub fn update_grid(
    time: Res<Time>,
    mut fx: ResMut<GridFx>,
    handle: Res<GridHandle>,
    mut materials: ResMut<Assets<GridMaterial>>,
    player_q: Query<&Transform, With<Player>>,
    bullet_q: Query<&Transform, With<Bullet>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let now = time.elapsed_secs();
    fx.now = now;
    fx.impacts.retain(|i| now - i.start <= IMPACT_LIFETIME);

    let Some(material) = materials.get_mut(&handle.0) else {
        return;
    };

    material.params.time = now;
    material.params.impact_count = fx.impacts.len() as u32;

    for (i, impact) in fx.impacts.iter().enumerate() {
        material.params.impacts[i] = Vec4::new(
            impact.pos.x,
            impact.pos.y,
            now - impact.start,
            impact.amp,
        );
        material.params.impact_dirs[i] = Vec4::new(
            impact.dir.x,
            impact.dir.y,
            impact.speed,
            impact.half_angle,
        );
    }

    material.params.bullet_count = 0;
    for transform in &bullet_q {
        let idx = material.params.bullet_count as usize;
        if idx >= MAX_BULLETS {
            break;
        }
        let p = transform.translation.truncate();
        material.params.bullets[idx] = Vec4::new(p.x, p.y, 0.0, 0.0);
        material.params.bullet_count += 1;
    }

    if let Ok(transform) = player_q.single() {
        let p = transform.translation.truncate();
        material.params.player = Vec4::new(p.x, p.y, 0.0, 0.0);
    }

    if keys.just_pressed(KeyCode::KeyG) {
        fx.flash(Vec2::ZERO, Vec2::X, 1.7, 820.0, 0.6);
    }
    if keys.just_pressed(KeyCode::KeyJ) {
        fx.flash(Vec2::ZERO, Vec2::Y, 1.7, 820.0, 0.6);
    }
}