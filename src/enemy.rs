use std::cmp::min;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::math::Vec2;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{Ccd, GravityScale, LockedAxes, RigidBody, Velocity};
use bevy_rapier2d::geometry::Collider;
use crate::bullet::*;
use crate::config::DebugMode;
use crate::health::Health;
use crate::player::Player;

#[derive(Component)]
pub struct Enemy{
    pub velocity: Vec2,
    pub hitbox_radius: f32,
    pub player_detect_dist: f32,
    pub turn_speed: f32,
    pub preferred_player_range: Vec2, // How far it prefers the player to be
    pub player_in_range: bool,
}

#[derive(Component)]
pub struct Closest;

impl Default for Enemy{
    fn default() -> Self{
        Self{
            velocity: Vec2::default(),
            hitbox_radius: 25.0,
            player_detect_dist: 500.0, // Also the dist player is at if inside
            turn_speed: 20.0,
            preferred_player_range: Vec2::new(200.0, 250.0),
            player_in_range: false,
        }
    }
}

pub fn shoot_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    player_query: Query<&Transform, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
){
    let Ok(player_transform) = player_query.single() else { return; };

    if !keys.just_pressed(KeyCode::Space) {return;}

    for transform in enemy_query.iter() {
        let shoot_point = transform.translation.truncate();
        let direction = (player_transform.translation.truncate() - shoot_point).normalize();

        let bullet = Bullet{
            direction,
            ..default()
        };
        commands.spawn((
            GlobalTransform::default(),
            EnemyBullet,
            Mesh2d(meshes.add(Circle::new(10.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.3, 0.3, 0.3))),
            LifeTime(Timer::from_seconds(bullet.lifetime, TimerMode::Once)),
            Transform::from_xyz(shoot_point.x, shoot_point.y, 0.0),
            Health::default(),
            bullet,
        ));
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

pub fn detect_player_in_range(
    player_q: Query<&Transform, With<Player>>,
    mut enemy_q: Query<(&mut Transform, &mut Enemy), Without<Player>>,
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

    for (mut transform, mut enemy) in enemy_q.iter_mut() {
        let to_player = player_pos - transform.translation.truncate();
        let len_sq = to_player.length_squared();

        if len_sq < 1e-6 || len_sq > enemy.player_detect_dist.powi(2) {
            enemy.player_in_range = false;
            continue;
        }

        // Player is in range
        enemy.player_in_range = true;

        // Face player
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