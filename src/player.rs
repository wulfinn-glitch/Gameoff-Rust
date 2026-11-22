use bevy::asset::Assets;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{GravityScale, LockedAxes, RigidBody};
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::prelude::{Ccd, Friction, Velocity};
use crate::config::DebugMode;
use crate::enemy::{Closest, Enemy};
use crate::health::Health;
use crate::uibar::*;
use crate::helpers::DoublePressTracker;
use crate::posture::Posture;

#[derive(Component)]
pub struct Player {
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub speed: f32,
    pub hitbox_radius: f32,
    pub locked_on: bool,
    pub turn_speed: f32,
}

#[derive(Component, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerAction {
    #[default]
    Neutral,
    NeutralPostureRecover,
    ParryActive,
    ParryRecover,
    AttackWindup,
    AttackActive,
    AttackRecover,
}

#[derive(Component)]
pub struct ActionTimer(pub Timer);

pub struct DashState {
    pub timer: Timer,
    pub direction: Vec2,
    pub dash_time: f32
}

impl Default for DashState {
    fn default() -> Self {
        Self {
            timer: Timer::default(),
            direction: Vec2::ZERO,
            dash_time: 0.3,
        }
    }
}

#[derive(Component)]
pub struct OrbitalMover {
    pub target: Vec2,
    pub distance: f32,
    pub speed_orbit: f32,
    pub step_amount: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self{
            velocity: Vec2::default(),
            acceleration: Vec2::default(),
            speed: 150.0,
            hitbox_radius: 20.0,
            locked_on: false,
            turn_speed: 40.0,
        }
    }
}

impl Default for OrbitalMover {
    fn default() -> Self {
        Self {
            target: Vec2::default(),
            distance: 0.0,
            speed_orbit: 120.0,
            step_amount: 500.0,
        }
    }
}

// Systems ----------------------------------------------------------------------------------------------------

pub fn debug_action(query: Query<&PlayerAction, Changed<PlayerAction>>){
    for action in &query{
        println!("PlayerAction: {action:?}");
    }
}

pub fn duration(action: PlayerAction) -> f32 {
    match action {
        PlayerAction::Neutral => 0.5,
        PlayerAction::NeutralPostureRecover => 0.5,
        PlayerAction::ParryActive => 0.15,
        PlayerAction::ParryRecover => 0.15,
        PlayerAction::AttackWindup => 0.10,
        PlayerAction::AttackActive => 0.10,
        PlayerAction::AttackRecover => 0.20,
    }
}

pub fn tick_action(time: Res<Time>, mut query: Query<(&mut PlayerAction, &mut ActionTimer)>) {
    for (mut action, mut timer) in &mut query{
        timer.0.tick(time.delta());

        if !timer.0.just_finished() {
            continue;
        }

        // Mappings to and from. This would need to be more flexible
        *action = match *action {
            PlayerAction::ParryActive => PlayerAction::ParryRecover,
            PlayerAction::ParryRecover => PlayerAction::Neutral,

            PlayerAction::AttackWindup => PlayerAction::AttackActive,
            PlayerAction::AttackActive => PlayerAction::AttackRecover,
            PlayerAction::AttackRecover => PlayerAction::Neutral,

            PlayerAction::Neutral => PlayerAction::NeutralPostureRecover,
            PlayerAction::NeutralPostureRecover => PlayerAction::Neutral
        };

        timer.0 = Timer::from_seconds(duration(*action), TimerMode::Once);
    }
}

pub fn start_action(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut PlayerAction, &mut ActionTimer)>)
{
    for(mut action, mut timer) in &mut query{
        let can_act = matches!(
            *action,
            PlayerAction::Neutral | PlayerAction::NeutralPostureRecover | PlayerAction::ParryRecover | PlayerAction::AttackRecover
        );

        // cannot act if not part of these states
        if !can_act {continue;}

        if keys.just_pressed(KeyCode::KeyQ){
            *action = PlayerAction::ParryActive;
            timer.0 = Timer::from_seconds(0.15, TimerMode::Once);
        }
        else if keys.just_pressed(KeyCode::KeyE){
            *action = PlayerAction::AttackWindup;
            timer.0 = Timer::from_seconds(0.15, TimerMode::Once);
        }
    }
}

pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    transform: Transform,
    ui_root: Entity
    ){
    // Store in var for getting defaults
    let player = Player::default();

    // Spawn Player (Lot of Components)
    let player = commands.spawn((
        transform,
        Mesh2d(meshes.add(Circle::new(player.hitbox_radius))),
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.3, 0.3))),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::ball(player.hitbox_radius),
        LockedAxes::ROTATION_LOCKED,
        Velocity::default(),
        Ccd::enabled(),
        Friction::coefficient(0.0),
        OrbitalMover::default(),
        player,
    ))
    .insert((
        // State Combat system
        PlayerAction::default(),
        ActionTimer(Timer::from_seconds(duration(PlayerAction::Neutral), TimerMode::Once)),

        // Bars
        // Health::default(),
        // Posture::default()

    )).id();

    let health = Health {current: 100, min: 0, max: 100};
    let posture = Posture {
        current: 1000,
        min: 0,
        max: 1000,
        recovery_timer: Timer::from_seconds(duration(PlayerAction::NeutralPostureRecover), TimerMode::Repeating), ..default()};

    // Add health bar
    commands.entity(ui_root).with_children(|mut parent| {
        spawn_stat_bar(
            &mut parent,
            player,
            &health,
            Color::srgb(0.15, 0.2, 0.18),
            Color::srgb(0.35, 0.9, 0.55),
            UIBarPlacement{
                bottom: Val::Percent(5.5),
                width: Val::Percent(80.0),
                justify_content: JustifyContent::Center,
                ..default()
            }
        );
    });
    // Add Posture bar
    commands.entity(ui_root).with_children(|mut parent| {
        spawn_stat_bar(
            &mut parent,
            player,
            &posture,
            Color::srgb(0.22, 0.19, 0.12),
            Color::srgb(0.95, 0.80, 0.25),
            UIBarPlacement{
                bottom: Val::Percent(3.0),
                width: Val::Percent(80.0),
                justify_content: JustifyContent::Center,
                ..default()
            }
        );
    });

    // Add to player
    commands.entity(player).insert((health, posture));
}

pub fn normal_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Velocity, &Player, &mut OrbitalMover)>,
    closest_enemy_query: Query<&Transform, With<Closest>>,
    time: Res<Time>,
){
    let dt = time.delta_secs();
    let Ok(target_enemy) = closest_enemy_query.single() else { return; };

    let (transform, mut velocity, player, mut mover) = query.iter_mut().next().unwrap();

    // Return if locked on
    if player.locked_on {return;}

    let mut direction = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) { direction.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
    if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }

    let desired = direction.normalize_or_zero();

    let target_vel = desired * player.speed;
    velocity.linvel = target_vel;

    if desired.length_squared() < 1e-6 {
        velocity.angvel = 0.0;
        return;
    }

    // Assuming sprite/collider faces +Y at rotation 0; use `transform.local_x().xy()` if +X
    let forward = transform.local_y().xy().normalize();
    let angle_diff = forward.angle_to(desired);

    velocity.angvel = (angle_diff / dt).clamp(-player.turn_speed, player.turn_speed);
}

pub fn orbital_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Velocity, &Player, &mut OrbitalMover)>,
    closest_enemy_query: Query<&Transform, With<Closest>>,
    time: Res<Time>,
    mut tracker: ResMut<DoublePressTracker>,
    mut dash: Local<DashState>,
){

    let dt = time.delta_secs();
    let Ok(target_enemy) = closest_enemy_query.single() else { return; };

    let (transform, mut velocity, player, mut mover) = query.iter_mut().next().unwrap();

    // Return if not locked on
    if !player.locked_on {return;}

    // Set target to closest enemy
    mover.target = target_enemy.translation.truncate();

    // If Locked on, combat controls
    let current_pos = transform.translation.truncate();
    let to_enemy = mover.target - current_pos;
    let current_distance = to_enemy.length();

    if current_distance < 0.001 {
        return;
    }

    // Normalized vec outward from player
    let dir_outward = -(to_enemy / current_distance);
    let tangent = Vec2::new(-dir_outward.y, dir_outward.x);

    let mut orbit_vel = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyA) {
        orbit_vel -= tangent * mover.speed_orbit;
    }
    if keys.pressed(KeyCode::KeyD) {
        orbit_vel += tangent * mover.speed_orbit;
    }

    // dashing
    let now = time.elapsed_secs();

    if tracker.check(KeyCode::KeyW, &keys, now) {
        dash.direction = -dir_outward;
        dash.timer = Timer::from_seconds(dash.dash_time, TimerMode::Once);
    } else if tracker.check(KeyCode::KeyS, &keys, now) {
        dash.direction = dir_outward;
        dash.timer = Timer::from_seconds(dash.dash_time, TimerMode::Once);
    }

    let mut zoom_vel = Vec2::ZERO;
    if !dash.timer.is_finished() {
        dash.timer.tick(time.delta());

        let remaining = 1.0 - dash.timer.fraction();
        let ease_out = remaining.powi(2);

        zoom_vel = dash.direction * mover.step_amount * ease_out;
    }

    velocity.linvel = orbit_vel + zoom_vel;

    let forward = transform.local_y().xy().normalize();
    let desired = to_enemy.normalize();

    let angle_diff = forward.angle_to(desired);
    let desired_angvel = (angle_diff / dt).clamp(-20.0, 20.0);
    velocity.angvel = desired_angvel;
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
    debug_enabled: Res<DebugMode>
) {
    if !debug_enabled.0 { return; }

    let Ok((player_transform, player)) = player_query.single() else { return; };

    if let Ok(enemy_transform) = closest_enemy_query.single() {
        let color = if player.locked_on { Color::srgb(1.0, 0.0, 0.0)} else {Color::srgb(1.0, 1.0, 1.0)};
        gizmos.line(
            player_transform.translation,
            enemy_transform.translation,
            color
        );

        if player.locked_on {
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

    if keys.just_pressed(KeyCode::Tab) {
        player.locked_on = !player.locked_on;
        println!("Locked on {} to {:?}", player.locked_on, enemy_transform.translation.truncate());
    }
}