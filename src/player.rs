use std::ptr::null;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::{Circle, ColorMaterial, Commands, Component, ContainsEntity, Entity, Gizmo, Gizmos, GlobalTransform, KeyCode, MeshMaterial2d, Query, Res, ResMut, Time, Transform, With};
use bevy::tasks::futures_lite::future::or;
use bevy_rapier2d::dynamics::{GravityScale, LockedAxes, RigidBody};
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::prelude::{Ccd, Friction, Velocity};
use crate::enemy::{Closest, Enemy};

#[derive(Component)]
pub struct Player {
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub speed: f32,
    pub hitbox_radius: f32,
    pub locked_on: bool
}

#[derive(Component)]
pub struct OrbitalMover {
    pub target: Vec2,
    pub distance: f32,
    pub speed_orbit: f32,
    pub speed_zoom: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self{
            velocity: Vec2::default(),
            acceleration: Vec2::default(),
            speed: 450.0,
            hitbox_radius: 20.0,
            locked_on: false
        }
    }
}

impl Default for OrbitalMover {
    fn default() -> Self {
        Self {
            target: Vec2::default(),
            distance: 0.0,
            speed_orbit: 200.0,
            speed_zoom: 200.0,
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
        OrbitalMover::default(),
        player,
    ));
}

pub fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Velocity, &Player, &mut OrbitalMover)>,
    closest_enemy_query: Query<&Transform, With<Closest>>,
) {
    let Ok(target_enemy) = closest_enemy_query.single() else { return; };

    for (transform, mut velocity, player, mut mover) in &mut query {

        // Set target to closest enemy
        mover.target = target_enemy.translation.truncate();

        if player.locked_on {
            let current_pos = transform.translation.truncate();
            let to_player = current_pos - mover.target;
            let current_distance = to_player.length();

            if current_distance < 0.001 {
                continue;
            }

            // Normalized vec outward from player
            let dir_outward = to_player / current_distance;
            let tangent = Vec2::new(-dir_outward.y, dir_outward.x);

            let mut orbit_vel = Vec2::ZERO;
            if keys.pressed(KeyCode::KeyA){
                orbit_vel -= tangent * mover.speed_orbit;
            }
            if keys.pressed(KeyCode::KeyD){
                orbit_vel += tangent * mover.speed_orbit;
            }

            let mut zoom_vel = Vec2::ZERO;
            if keys.pressed(KeyCode::KeyW) {
                zoom_vel -= dir_outward * mover.speed_zoom;
            }
            if keys.pressed(KeyCode::KeyS) {
                zoom_vel += dir_outward * mover.speed_zoom;
            }

            velocity.linvel = orbit_vel + zoom_vel;
        }
        else {
            let mut direction = Vec2::ZERO;

            if keys.pressed(KeyCode::KeyW) { direction.y += 1.0; }
            if keys.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
            if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
            if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }

            let target_vel = direction.normalize_or_zero() * player.speed;
            velocity.linvel = target_vel;
        }
    }
}
pub fn find_closest_enemy(
    mut commands: Commands,
    player_query: Query<(&Transform, &Player)>,
    enemies_query: Query<(Entity, &Transform), With<Enemy>>,
    current_closest_query: Query<Entity, With<Closest>>,
){
    let Ok((player_transform, player)) = player_query.single() else {
         return;
    };

    // Return if player is already locked on to some enemy
    if player.locked_on {
        return;
    }

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
}

pub fn target_closest_enemy(
    closest_enemy_query: Query<&Transform, With<Closest>>,
    player_query: Query<(&Transform, &Player)>,
    mut gizmos: Gizmos,
) {
    let Ok((player_transform, player)) = player_query.single() else { return; };

    if let Ok(enemy_transform) = closest_enemy_query.single() {
        let color = if player.locked_on { Color::srgb(1.0, 0.0, 0.0)} else {Color::srgb(1.0, 1.0, 1.0)};
        gizmos.line(
            player_transform.translation,
            enemy_transform.translation,
            color
        );

        if player.locked_on{
            gizmos.circle_2d(
                enemy_transform.translation.truncate(),
                (enemy_transform.translation.truncate() - player_transform.translation.truncate()).length(),
                Color::srgb(0.0, 1.0, 0.0));
        }
    }
}

pub fn lock_unlock_closest_enemy(
    closest_enemy_query: Query<&Transform, With<Closest>>,
    mut player_query: Query<(&Transform, &mut Player)>,
    keys: Res<ButtonInput<KeyCode>>,
){

    let Ok((_, mut player)) = player_query.single_mut() else { return; };
    let Ok(enemy_transform) = closest_enemy_query.single() else { return; };

    if keys.just_pressed(KeyCode::KeyE) {
        player.locked_on = !player.locked_on;
        println!("Locked on {} to {:?}", player.locked_on, enemy_transform.translation.truncate());
    }
}