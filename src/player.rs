use bevy::asset::Assets;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::{Circle, ColorMaterial, Commands, Component, GlobalTransform, KeyCode, MeshMaterial2d, Query, Res, ResMut, Time, Transform};
use bevy_rapier2d::dynamics::{GravityScale, LockedAxes, RigidBody};
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::prelude::{Ccd, Friction, Velocity};

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
