use std::ptr::null;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::{Circle, ColorMaterial, Commands, Component, ContainsEntity, Entity, Gizmo, Gizmos, GlobalTransform, KeyCode, MeshMaterial2d, Query, Res, ResMut, Time, Transform, With};
use bevy_rapier2d::dynamics::{GravityScale, LockedAxes, RigidBody};
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::parry::transformation::utils::transform;
use bevy_rapier2d::prelude::{Ccd, Friction, Velocity};
use crate::enemy::{Closest, Enemy};

#[derive(Component)]
pub struct Player {
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub speed: f32,
    pub hitbox_radius: f32
}

impl Default for Player {
    fn default() -> Self {
        Self{
            velocity: Vec2::default(),
            acceleration: Vec2::default(),
            speed: 450.0,
            hitbox_radius: 20.0,
        }
    }
}

// Systems ----------------------------------------------------------------------------------------------------

pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    transform: Transform){
    // Store in var for getting defaults
    let player = Player::default();

    // Spawn Player (Lot of Components)
    commands.spawn((
        GlobalTransform::default(),
        Mesh2d(meshes.add(Circle::new(player.hitbox_radius))),
        MeshMaterial2d(materials.add(Color::srgb(0.9, 0.9, 0.9))),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::ball(player.hitbox_radius),
        LockedAxes::ROTATION_LOCKED,
        Velocity::default(),
        Ccd::enabled(),
        Friction::coefficient(0.0),
        player,
    ));
}

pub fn move_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Player)>,
) {
    for (mut velocity, player) in &mut query {
        let mut direction = Vec2::ZERO;

        if keys.pressed(KeyCode::KeyW) { direction.y += 1.0; }
        if keys.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
        if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }

        let target_vel = direction.normalize_or_zero() * player.speed;

        velocity.linvel = target_vel;
    }
}
pub fn find_closest_enemy(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    enemies_query: Query<(Entity, &Transform), With<Enemy>>,
    mut gizmos: Gizmos,
    current_closest_query: Query<Entity, With<Closest>>,
){
    let Ok(player_transform) = player_query.single() else {
         return;
    };

    let player_pos = player_transform.translation.truncate();

    let Some((new_closest, _)) = enemies_query
        .iter()
        .min_by(|(_, transform_a), (_, transform_b)| {
            let dist_a = player_pos.distance_squared(transform_a.translation.truncate());
            let dist_b = player_pos.distance_squared(transform_b.translation.truncate());

            dist_a.total_cmp(&dist_b)
        }) else {
            for entity in &current_closest_query {
                commands.entity(entity).remove::<Closest>();
            }
        return;
    };
    let current_closest = current_closest_query.iter().next();

    if current_closest != Some(new_closest){
        if let Some(old_entity) = current_closest {
            commands.entity(old_entity).remove::<Closest>();
        }

        commands.entity(new_closest).insert(Closest);
    }

    // if let Ok((_, transform)) = enemies_query.get(new_closest) {
    //     let actual_distance = player_pos.distance(transform.translation.truncate());
    //     gizmos.line_2d(player_pos, transform.translation.truncate(), Color::srgb(1.0, 1.0, 1.0));
    // }
}

pub fn target_closest_enemy(
    closest_enemy_query: Query<&Transform, With<Closest>>,
    player_query: Query<&Transform, With<Player>>,
    mut gizmos: Gizmos,
) {
    let Ok(player_transform) = player_query.single() else { return; };

    if let Ok(enemy_transform) = closest_enemy_query.single() {
        gizmos.line(
            player_transform.translation,
            enemy_transform.translation,
            Color::srgb(1.0, 0.0, 0.0)
        );
    }
}
