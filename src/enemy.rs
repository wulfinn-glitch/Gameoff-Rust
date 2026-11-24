use std::cmp::min;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::math::Vec2;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{Ccd, GravityScale, LockedAxes, RigidBody, Velocity};
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::prelude::{CollisionGroups, Group, Sensor};
use crate::bullet::*;
use crate::config::DebugMode;
use crate::health::Health;
use crate::helpers::GROUP_ENEMY;
use crate::player::Player;
use crate::fx::FxPalette;

#[derive(Component)]
pub struct Enemy{
    pub velocity: Vec2,
    pub hitbox_radius: f32,
    pub player_detect_dist: f32,
    pub move_speed: f32,
    pub turn_speed: f32,
    pub preferred_player_range: Vec2, // How far it prefers the player to be
    pub player_in_range: bool,
}

#[derive(Component)]
pub struct Closest;

/// Base body colour of an enemy.
pub const ENEMY_COLOR: Color = Color::srgb(1.0, 0.7, 0.7);

/// Brief white flash applied when an attack lands.
#[derive(Component)]
pub struct HitFlash {
    pub timer: Timer,
    pub base: Color,
}

pub fn animate_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    palette: Res<FxPalette>,
    mut flashes: Query<(Entity, &mut HitFlash, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut flash, material_handle) in &mut flashes {
        flash.timer.tick(time.delta());

        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        if flash.timer.is_finished() {
            material.color = flash.base;
            commands.entity(entity).remove::<HitFlash>();
            continue;
        }

        // Flash toward the configured impact colour, easing back to the base.
        let f = 1.0 - flash.timer.fraction();
        let base = flash.base.to_srgba();
        let hot = palette.hit_flash_color().to_srgba();
        material.color = Color::srgb(
            base.red + (hot.red - base.red) * f,
            base.green + (hot.green - base.green) * f,
            base.blue + (hot.blue - base.blue) * f,
        );
    }
}

impl Default for Enemy{
    fn default() -> Self{
        Self{
            velocity: Vec2::default(),
            hitbox_radius: 40.0,
            player_detect_dist: 250.0, // Also the dist player is at if inside
            move_speed: 75.0,
            turn_speed: 20.0,
            preferred_player_range: Vec2::new(100.0, 150.0),
            player_in_range: false,
        }
    }
}

pub fn move_to_preferred_dist(
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&Transform, &mut Enemy, &mut Velocity)>
){
    let Ok(player_transform) = player_query.single() else { return; };

    for (enemy_transform, mut enemy, mut vel) in enemy_query.iter_mut() {
        let to_player = (enemy_transform.translation.truncate() - player_transform.translation.truncate());
        let dist = to_player.length();
        let direction = to_player.normalize();

        // Not in range
        if dist < enemy.preferred_player_range.x {
            // Move towards player
            vel.linvel = direction * enemy.move_speed;
            enemy.player_in_range = true;
            continue;
        }

        if dist > enemy.preferred_player_range.y && dist <= enemy.player_detect_dist{
            vel.linvel = -direction * enemy.move_speed;
            enemy.player_in_range = true;
            continue;
        }

        enemy.player_in_range = false;
        vel.linvel = Vec2::ZERO;
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
            Collider::ball(10.0),
            Sensor,
            Health::default(),
            CollisionGroups::new(GROUP_ENEMY, Group::ALL),
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
        let pos = enemy_transform.translation.truncate();
        let dist = (pos - player_transform.translation.truncate()).length();
        let radius = f32::min(dist, enemy.player_detect_dist);

        gizmos.circle_2d(pos, radius, Color::srgb(0.7, 0.92, 0.95));

        gizmos.circle_2d(pos, enemy.preferred_player_range.x, Color::srgb(0.1, 0.1, 0.1));
        gizmos.circle_2d(pos, enemy.preferred_player_range.y, Color::srgb(0.1, 0.1, 0.1));
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
        MeshMaterial2d(materials.add(ENEMY_COLOR)),
        RigidBody::KinematicVelocityBased,
        GravityScale(0.0),
        Collider::ball(base_enemy.hitbox_radius),
        LockedAxes::ROTATION_LOCKED,
        Velocity::default(),
        Ccd::enabled(),
        Health::default(),
        CollisionGroups::new(GROUP_ENEMY, Group::ALL),
        base_enemy
    ));
}