use std::cmp::min;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::math::Vec2;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{Ccd, GravityScale, LockedAxes, RigidBody, Velocity};
use bevy_rapier2d::geometry::Collider;
use crate::config::DebugMode;
use crate::player::Player;

#[derive(Component)]
pub struct Enemy{
    pub velocity: Vec2,
    pub hitbox_radius: f32,
    pub player_detect_dist: f32,
    pub turn_speed: f32
}

#[derive(Component)]
pub struct Closest;

impl Default for Enemy{
    fn default() -> Self{
        Self{
            velocity: Vec2::default(),
            hitbox_radius: 25.0,
            player_detect_dist: 200.0, // Also the dist player is at if inside
            turn_speed: 20.0,
        }
    }
}

pub fn draw_detect_dist(
    enemy_q: Query<(&Transform, &Enemy)>,
    player_q: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut gizmos: Gizmos,
    debug_mode: Res<DebugMode>,
) {
    if !debug_mode.0 { return;}

    let Ok(player_transform) = player_q.single() else {
        // No or Multiple players
        return;
    };

    for (enemy_transform, enemy) in enemy_q.iter() {
        // Get min dist of enemy.player_detect, and actual dist
        let dist = (enemy_transform.translation.truncate() - player_transform.translation.truncate()).length();
        let radius = f32::min(dist, enemy.player_detect_dist);
        gizmos.circle_2d(enemy_transform.translation.truncate(), radius, Color::srgb(0.7, 0.92, 0.95));
    }
}

pub fn face_player_in_range(
    player_q: Query<&Transform, With<Player>>,
    mut enemy_q: Query<(&mut Transform, &Enemy), Without<Player>>,
    time: Res<Time>
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let Some(player_transform) = player_q.iter().next() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let facing_axis = Vec2::Y;

    for (mut transform, enemy) in enemy_q.iter_mut() {
        let to_player = player_pos - transform.translation.truncate();
        let len_sq = to_player.length_squared();

        if len_sq < 1e-6 || len_sq > enemy.player_detect_dist.powi(2) {
            continue;
        }

        let target_rot = Quat::from_rotation_arc_2d(facing_axis, to_player.normalize());
        transform.rotation = transform.rotation.rotate_towards(target_rot, enemy.turn_speed * dt);
    }
}

pub fn spawn_enemy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    transform: Transform
) {

    let base_enemy = Enemy::default();

    // Spawn Enemy
    commands.spawn((
        transform,
        GlobalTransform::default(),
        Mesh2d(meshes.add(Circle::new(base_enemy.hitbox_radius))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.7, 0.7))),
        RigidBody::KinematicPositionBased,
        GravityScale(0.0),
        Collider::ball(base_enemy.hitbox_radius),
        LockedAxes::ROTATION_LOCKED,
        Velocity::default(),
        Ccd::enabled(),
        base_enemy
    ));
}