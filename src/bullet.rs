use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct Bullet{
    pub speed: f32,
    pub lifetime: f32,
    pub direction: Vec2,
}

#[derive(Component)]
pub struct EnemyBullet; // Tag for enemy bullet

#[derive(Component)]
pub struct LifeTime(pub Timer);

impl Default for Bullet{
    fn default() -> Self{
        Self{
            speed: 450.0,
            lifetime: 3.0,
            direction: Vec2::default(),
        }
    }
}

pub fn move_bullets(
    mut query: Query<(&Bullet, &mut Transform)>,
    time: Res<Time>,
){
    for (bullet, mut transform) in &mut query {
        transform.translation.x += bullet.direction.x * time.delta_secs() * bullet.speed;
        transform.translation.y += bullet.direction.y * time.delta_secs() * bullet.speed;
    }
}

pub fn despawn_expired_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LifeTime), With<Bullet>>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.0.tick(time.delta());
        if lifetime.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
