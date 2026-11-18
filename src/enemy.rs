use bevy::asset::Assets;
use bevy::color::Color;
use bevy::math::Vec2;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::{Circle, ColorMaterial, Commands, Component, GlobalTransform, MeshMaterial2d, Query, ResMut, Transform, With};
use bevy_rapier2d::dynamics::{Ccd, GravityScale, LockedAxes, RigidBody, Velocity};
use bevy_rapier2d::geometry::Collider;
use crate::player::Player;

#[derive(Component)]
pub struct Enemy{
    pub velocity: Vec2,
    pub hitbox_radius: f32,
}

impl Default for Enemy{
    fn default() -> Self{
        Self{
            velocity: Vec2::default(),
            hitbox_radius: 25.0,
        }
    }
}

pub fn calc_direction(
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>) {

    let Ok(player_transform) = player_q.single() else {
        // No or Multiple players
        return;
    };

    let player_pos = player_transform.translation.truncate(); // Vec3 -> Vec2 (truncate)

    for enemy_transform in enemy_q.iter() {
        let enemy_pos = enemy_transform.translation.truncate();
        let to_enemy = enemy_pos - player_pos;

        if to_enemy.length_squared() > 0.0{
            let direction = to_enemy.normalize();
        }
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
        RigidBody::Fixed,
        GravityScale(0.0),
        Collider::ball(base_enemy.hitbox_radius),
        LockedAxes::ROTATION_LOCKED,
        Velocity::default(),
        Ccd::enabled(),
        base_enemy
    ));
}