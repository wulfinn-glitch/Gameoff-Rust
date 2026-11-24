use std::collections::HashSet;
use std::f32::consts::FRAC_PI_2;
use bevy::asset::Assets;
use bevy::camera::primitives::HalfSpace;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{GravityScale, LockedAxes, RigidBody};
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::plugin::ReadRapierContext;
use bevy_rapier2d::prelude::{Ccd, ColliderMassProperties, CollisionGroups, Friction, Group, Sensor, Velocity};
use bevy_rapier2d::render::ColliderDebugColor;
use crate::bullet::{Bullet, EnemyBullet, LifeTime};
use crate::config::DebugMode;
use crate::enemy::{Closest, Enemy, HitFlash, ENEMY_COLOR};
use crate::grid::GridFx;
use crate::health::Health;
use crate::uibar::*;
use crate::helpers::{DoublePressTracker, GROUP_ENEMY, GROUP_HITBOX, GROUP_PLAYER};
use crate::posture::Posture;
use crate::slash::{spawn_slash, spawn_slash_cluster, SlashAssets, SlashShape};
use crate::juice::CombatFeel;
use crate::fx::FxPalette;

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
    Block,
    ParryRecover,
    BlockRecover,
    AttackWindup,
    AttackActive,
    AttackRecover,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum HitboxKind {
    Attack,
    Parry
}

#[derive(Component)]
pub struct ActionTimer(pub Timer);

/// How far through the attack chain the player is.
#[derive(Component)]
pub struct Combo {
    pub step: u8,
    pub max: u8,
    /// An attack press made during `AttackActive`, to be consumed when it ends.
    pub queued: bool,
}

impl Default for Combo {
    fn default() -> Self {
        Self { step: 0, max: 3, queued: false }
    }
}

/// Post-action lockouts so attack/parry can't be spammed.
#[derive(Component, Default)]
pub struct Cooldowns {
    attack: f32,
    parry: f32,
}

const ATTACK_COOLDOWN: f32 = 0.18;
const PARRY_COOLDOWN: f32 = 0.35;

impl Cooldowns {
    fn tick(&mut self, dt: f32) {
        self.attack = (self.attack - dt).max(0.0);
        self.parry = (self.parry - dt).max(0.0);
    }

    fn attack_ready(&self) -> bool {
        self.attack <= 0.0
    }

    fn parry_ready(&self) -> bool {
        self.parry <= 0.0
    }
}

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
        PlayerAction::Block => 0.0,
        PlayerAction::ParryRecover => 0.15,
        PlayerAction::BlockRecover => 0.15,
        PlayerAction::AttackWindup => 0.10,
        PlayerAction::AttackActive => 0.10,
        PlayerAction::AttackRecover => 0.20,
    }
}

pub fn tick_action(
    time: Res<Time>,
    mut query: Query<(&mut PlayerAction, &mut ActionTimer, &mut Combo, &mut Cooldowns)>,
    keys: Res<ButtonInput<KeyCode>>
) {
    for (mut action, mut timer, mut combo, mut cooldowns) in &mut query{

        cooldowns.tick(time.delta_secs());

        // Block has no timer
        if *action == PlayerAction::Block{
            continue;
        }

        timer.0.tick(time.delta());

        if !timer.0.just_finished() {
            continue;
        }

        // Mappings to and from. This would need to be more flexible
        *action = match *action {
            PlayerAction::ParryActive => {
                if keys.pressed(KeyCode::KeyQ){
                    PlayerAction::Block
                } else {
                    PlayerAction::ParryRecover
                }
            }
            PlayerAction::ParryRecover => {
                cooldowns.parry = PARRY_COOLDOWN;
                PlayerAction::Neutral
            }
            PlayerAction::BlockRecover => {
                cooldowns.parry = PARRY_COOLDOWN;
                PlayerAction::Neutral
            }

            PlayerAction::AttackWindup => PlayerAction::AttackActive,
            PlayerAction::AttackActive => {
                // Consume a buffered press: chain straight into the next windup.
                if combo.queued && combo.step + 1 < combo.max {
                    combo.queued = false;
                    combo.step += 1;
                    PlayerAction::AttackWindup
                } else {
                    combo.queued = false;
                    PlayerAction::AttackRecover
                }
            }
            PlayerAction::AttackRecover => {
                combo.step = 0;
                combo.queued = false;
                cooldowns.attack = ATTACK_COOLDOWN;
                PlayerAction::Neutral
            }

            PlayerAction::Neutral => PlayerAction::NeutralPostureRecover,
            PlayerAction::NeutralPostureRecover => PlayerAction::Neutral,

            PlayerAction::Block => PlayerAction::Block,
        };

        if *action != PlayerAction::Block {
            timer.0 = Timer::from_seconds(duration(*action), TimerMode::Once);
        }
    }
}

pub fn sector_collider(
    range: f32,
    half_angle: f32,
    segments: usize
) -> Collider {
    let mut points = Vec::with_capacity(segments + 2);
    points.push(Vec2::ZERO);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = FRAC_PI_2 - half_angle + 2.0 * half_angle * t;
        points.push(Vec2::from_angle(angle) * range);
    }

    Collider::convex_hull(&points).expect("sector hull")
}

pub fn start_action(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut PlayerAction, &mut ActionTimer, &mut Combo, &Cooldowns)>)
{
    for(mut action, mut timer, mut combo, cooldowns) in &mut query{

        if *action == PlayerAction::Block {
            if !keys.pressed(KeyCode::KeyQ) {
                *action = PlayerAction::BlockRecover;
                timer.0 = Timer::from_seconds(duration(PlayerAction::BlockRecover), TimerMode::Once);
            }
            continue;
        }

        // Buffer an attack press made during the active frames.
        if *action == PlayerAction::AttackActive && keys.just_pressed(KeyCode::KeyE) {
            combo.queued = true;
            continue;
        }

        let can_act = matches!(
            *action,
            PlayerAction::Neutral
            | PlayerAction::NeutralPostureRecover
            | PlayerAction::ParryRecover
            | PlayerAction::BlockRecover
            | PlayerAction::AttackRecover
        );

        // cannot act if not part of these states
        if !can_act {continue;}

        if keys.just_pressed(KeyCode::KeyQ) && cooldowns.parry_ready() {
            combo.step = 0;
            combo.queued = false;
            *action = PlayerAction::ParryActive;
            timer.0 = Timer::from_seconds(duration(PlayerAction::ParryActive), TimerMode::Once);
        }
        else if keys.just_pressed(KeyCode::KeyE){
            // Chain into the next hit during recovery, otherwise start fresh.
            if *action == PlayerAction::AttackRecover && combo.step + 1 < combo.max {
                combo.step += 1;
            } else if cooldowns.attack_ready() {
                combo.step = 0;
            } else {
                continue;
            }
            combo.queued = false;
            *action = PlayerAction::AttackWindup;
            timer.0 = Timer::from_seconds(duration(PlayerAction::AttackWindup), TimerMode::Once);
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
        Combo::default(),
        Cooldowns::default(),

        CollisionGroups::new(GROUP_PLAYER, Group::ALL),
    )).id();

    // Spawn hitboxes
    spawn_hitbox(commands, player, HitboxKind::Attack, 95.0, 75.0);
    spawn_hitbox(commands, player, HitboxKind::Parry, 60.0, 60.0);

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

fn spawn_hitbox(
    commands: &mut Commands,
    player: Entity,
    kind: HitboxKind,
    range: f32,
    half_angle_deg: f32,
){
    commands.spawn((
        ChildOf(player),
        kind,
        sector_collider(range, half_angle_deg.to_radians(), 12),
        Sensor,
        ColliderMassProperties::Density(0.0),
        // Is in hitbox, can hit enemy
        CollisionGroups::new(GROUP_HITBOX, GROUP_ENEMY),
        ColliderDebugColor(Hsla::new(0.0, 0.0, 0.2, 0.15)),
        Transform::default(),
        ));
}

pub fn color_hitboxes(
    player_q: Query<&PlayerAction, With<Player>>,
    mut hitbox_q: Query<(&HitboxKind, &mut ColliderDebugColor)>
) {
    let Ok(action) = player_q.single() else { return; };

    for (kind, mut color) in hitbox_q.iter_mut() {
        color.0 = hitbox_color(*kind, *action);
    }
}

fn hitbox_color(kind: HitboxKind, action: PlayerAction ) -> Hsla {
    match (kind, action) {
        (HitboxKind::Attack, PlayerAction::AttackWindup) => Hsla::new(45.0, 1.0, 0.6, 0.9),   // orange
        (HitboxKind::Attack, PlayerAction::AttackActive) => Hsla::new(0.0, 1.0, 0.55, 1.0),   // red
        (HitboxKind::Attack, PlayerAction::AttackRecover) => Hsla::new(220.0, 0.6, 0.4, 0.6), // dim blue

        (HitboxKind::Parry, PlayerAction::ParryActive) => Hsla::new(180.0, 1.0, 0.6, 1.0),    // cyan
        (HitboxKind::Parry, PlayerAction::Block) => Hsla::new(205.0, 0.9, 0.5, 0.95),
        (HitboxKind::Parry, PlayerAction::ParryRecover) => Hsla::new(220.0, 0.5, 0.4, 0.6),   // dim blue
        (HitboxKind::Parry, PlayerAction::BlockRecover) => Hsla::new(220.0, 0.5, 0.4, 0.6),
        _ => Hsla::new(0.0, 0.0, 0.2, 0.15),
    }
}

pub fn resolve_attack_hits(
    mut commands: Commands,
    rapier: ReadRapierContext,
    player_q: Query<(&PlayerAction, &Transform, &Combo), With<Player>>,
    hitbox_q: Query<(Entity, &HitboxKind)>,
    mut enemy_q: Query<(&Transform, &mut Health), With<Enemy>>,
    mut already_hit: Local<HashSet<Entity>>,
    mut fx: ResMut<GridFx>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<SlashAssets>,
    palette: Res<FxPalette>,
    mut feel: ResMut<CombatFeel>,
) {
    let Ok((action, player_transform, combo)) = player_q.single() else { return; };
    let player_pos = player_transform.translation.truncate();

    if *action != PlayerAction::AttackActive {
        already_hit.clear();
        return;
    }

    let Ok(context) = rapier.single() else { return; };

    // Check for intersection
    for (sensor, kind) in hitbox_q.iter() {
        if *kind != HitboxKind::Attack { continue; }

        for (a, b, intersecting) in context.intersection_pairs_with(sensor){
            if !intersecting { continue; }

            let other = if a == sensor { b } else { a };

            let Ok((enemy_transform, mut health)) = enemy_q.get_mut(other) else { continue; };

            // One hit per swing per enemy
            if !already_hit.insert(other){
                continue;
            }

            let damage = match combo.step {
                0 => 10,
                1 => 12,
                _ => 16,
            };
            health.current -= damage;
            println!("Enemy Health: {:?}", health.current);

            let enemy_pos = enemy_transform.translation.truncate();
            let dir = (enemy_pos - player_pos).normalize_or_zero();
            fx.flash(enemy_pos, dir, 1.0 + combo.step as f32 * 0.2, 430.0, 0.7);

            // Impact spark + hitstop/shake.
            let mut spark = SlashShape::hit_spark();
            spark.color = palette.spark_color();
            spawn_slash(
                &mut commands,
                &mut meshes,
                &assets,
                enemy_pos,
                dir,
                spark,
                0.16,
            );
            feel.punch(
                0.35 + combo.step as f32 * 0.05,
                0.035 + combo.step as f32 * 0.012,
            );

            // White flash on the enemy.
            commands.entity(other).insert(HitFlash {
                timer: Timer::from_seconds(0.08, TimerMode::Once),
                base: ENEMY_COLOR,
            });
        }
    }
}

pub fn receive_bullet_hits(
    mut commands: Commands,
    rapier: ReadRapierContext,
    bullet_q: Query<(Entity, &Bullet, &Transform), With<EnemyBullet>>,
    mut player_q: Query<(&mut Health, &Transform), With<Player>>,
    mut fx: ResMut<GridFx>,
    mut feel: ResMut<CombatFeel>,
) {
    let Ok(context) = rapier.single() else { return; };

    for (bullet_entity, bullet, bullet_transform) in bullet_q.iter() {
        for (a, b, intersecting) in context.intersection_pairs_with(bullet_entity){
            if !intersecting { continue; }

            let other = if a == bullet_entity { b } else { a };

            if let Ok((mut health, _transform)) = player_q.get_mut(other){
                health.current -= bullet.damage;
                commands.entity(bullet_entity).despawn();
                let pos = bullet_transform.translation.truncate();
                let dir = bullet.direction.normalize_or_zero(); // back toward the shooter
                fx.flash(pos, dir, 0.8, 380.0, 0.5);
                feel.punch(0.6, 0.05);
                break;
            }
        }
    }
}

pub fn flash_attack_swing(
    player_q: Query<(&Transform, &PlayerAction), (Changed<PlayerAction>, With<Player>)>,
    mut fx: ResMut<GridFx>,
) {
    for (transform, action) in player_q.iter() {
        if *action == PlayerAction::AttackActive{
            let pos = transform.translation.truncate();
            // Sprite/hitbox face local +Y. Locked-on rotation already aims it at the enemy,
            // so forward is correct whether locked on or moving.
            let dir = transform.local_y().xy().normalize_or_zero();
            fx.flash(pos, dir, 0.5, 300.0, 0.8);
        }
    }
}

/// Spawn the procedural slash/parry visuals when the action state changes.
pub fn spawn_action_effects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<SlashAssets>,
    palette: Res<FxPalette>,
    mut feel: ResMut<CombatFeel>,
    mut seed: Local<u64>,
    player_q: Query<(&Transform, &PlayerAction, &Combo), (Changed<PlayerAction>, With<Player>)>,
) {
    for (transform, action, combo) in &player_q {
        let pos = transform.translation.truncate();
        let facing = transform.local_y().xy().normalize_or_zero();

        match action {
            PlayerAction::AttackActive => {
                *seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let count = 3 + combo.step as usize;
                let mut shape = SlashShape::for_step(combo.step);
                shape.color = palette.attack_color();
                spawn_slash_cluster(
                    &mut commands,
                    &mut meshes,
                    &assets,
                    pos,
                    facing,
                    shape,
                    0.16,
                    count,
                    *seed ^ (combo.step as u64).wrapping_mul(0x2545_F491_4F6C_DD1D),
                );
                // Tiny kick on every swing; landing a hit adds more.
                feel.punch(0.12, 0.0);
            }
            PlayerAction::ParryActive => {
                *seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
                // A couple of overlapping shield arcs.
                let mut shape = SlashShape::parry_arc();
                shape.color = palette.parry_color();
                spawn_slash_cluster(
                    &mut commands,
                    &mut meshes,
                    &assets,
                    pos,
                    facing,
                    shape,
                    0.16,
                    2,
                    *seed,
                );
                feel.punch(0.08, 0.0);
            }
            _ => {}
        }
    }
}

pub fn resolve_parry(
    mut commands: Commands,
    rapier: ReadRapierContext,
    mut player_q: Query<(&PlayerAction, &mut Health, &mut Posture, &Transform), With<Player>>,
    hitbox_q: Query<(Entity, &HitboxKind)>,
    bullet_q: Query<(&Bullet, &Transform), With<EnemyBullet>>,
    mut fx: ResMut<GridFx>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<SlashAssets>,
    palette: Res<FxPalette>,
    mut feel: ResMut<CombatFeel>,
    mut seed: Local<u64>,
) {
    let Ok((action, mut health, mut posture, player_transform)) = player_q.single_mut() else {
        return;
    };

    let parrying = *action == PlayerAction::ParryActive;
    let blocking = *action == PlayerAction::Block;

    if !parrying && !blocking {return;}

    let Ok(context) = rapier.single() else {
        return;
    };

    for (sensor, kind) in &hitbox_q {
        if *kind != HitboxKind::Parry {
            continue;
        }

        for (a, b, intersecting) in context.intersection_pairs_with(sensor) {
            if !intersecting {
                continue;
            }

            let other = if a == sensor { b } else { a };
            let Ok((bullet, bullet_transform)) = bullet_q.get(other) else {
                continue;
            };

            if parrying {
                // Perfect parry
                posture.current = (posture.current + posture.parry_gain).min(posture.max);
            }
            else{
                // Block Spend guard
                let after = posture.current - posture.block_cost;
                if after <= posture.min {
                    posture.current = posture.min;
                    health.current -= bullet.damage;
                }
                else{
                    posture.current = after;
                }
            }

            let player_pos = player_transform.translation.truncate();
            let facing = player_transform.local_y().xy().normalize_or_zero();
            let pos = bullet_transform.translation.truncate();
            let dir = (pos - player_pos).normalize_or_zero(); // toward the incoming bullet
            let (amp, speed) = if parrying { (1.2, 500.0) } else { (0.6, 350.0) };
            fx.flash(pos, dir, amp, speed, 0.5);

            let mut parry_pop = SlashShape::parry_pop();
            parry_pop.color = palette.parry_color();
            let mut spark = SlashShape::hit_spark();
            spark.color = palette.spark_color();
            let mut arc = SlashShape::parry_arc();
            arc.color = palette.parry_color();

            if parrying {
                *seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);

                // Confident burst at the point of contact.
                spawn_slash_cluster(
                    &mut commands,
                    &mut meshes,
                    &assets,
                    pos,
                    dir,
                    parry_pop,
                    0.24,
                    3,
                    *seed,
                );
                spawn_slash_cluster(
                    &mut commands,
                    &mut meshes,
                    &assets,
                    pos,
                    dir,
                    spark,
                    0.18,
                    2,
                    *seed ^ 0x00A5_5A5A,
                );
                // Shield arcs flaring out from the player.
                spawn_slash_cluster(
                    &mut commands,
                    &mut meshes,
                    &assets,
                    player_pos,
                    facing,
                    arc,
                    0.20,
                    2,
                    *seed ^ 0x1234_5678,
                );

                // Freeze the world for a beat so the parry reads.
                feel.punch(0.7, 0.13);

                // Deflect the bullet cleanly: facing - incoming.
                let incoming = bullet.direction.normalize_or_zero();
                let mut deflected = (facing - incoming).normalize_or_zero();
                if deflected.length_squared() < 1e-6 {
                    deflected = -incoming;
                }
                if deflected.length_squared() < 1e-6 {
                    deflected = Vec2::Y;
                }

                commands.spawn((
                    GlobalTransform::default(),
                    Mesh2d(meshes.add(Circle::new(9.0))),
                    MeshMaterial2d(materials.add(palette.deflect_color())),
                    LifeTime(Timer::from_seconds(2.0, TimerMode::Once)),
                    Transform::from_xyz(pos.x, pos.y, 0.0),
                    Bullet {
                        direction: deflected,
                        speed: 900.0,
                        ..default()
                    },
                ));
            } else {
                // Block: stop the bullet, smaller reaction.
                spawn_slash(
                    &mut commands,
                    &mut meshes,
                    &assets,
                    pos,
                    dir,
                    parry_pop,
                    0.16,
                );
                feel.punch(0.3, 0.035);
            }

            commands.entity(other).despawn();
        }
    }
}

pub fn normal_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity, &Player, &mut OrbitalMover)>,
    time: Res<Time>,
){
    let dt = time.delta_secs();
    let Ok((mut transform, mut velocity, player, _mover)) = query.single_mut() else { return; };

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
        return;
    }

    // Sprite/collider/hitboxes face local +Y, so rotate the transform toward the
    // movement direction. Rapier rotation is locked; we drive it manually.
    let target = Quat::from_rotation_arc_2d(Vec2::Y, desired);
    transform.rotation = transform.rotation.rotate_towards(target, player.turn_speed * dt);
}

pub fn orbital_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity, &Player, &mut OrbitalMover), Without<Closest>>,
    closest_enemy_query: Query<&Transform, With<Closest>>,
    time: Res<Time>,
    mut tracker: ResMut<DoublePressTracker>,
    mut dash: Local<DashState>,
){

    let dt = time.delta_secs();
    let Some(target_enemy) = closest_enemy_query.iter().next() else { return; };

    let Ok((mut transform, mut velocity, player, mut mover)) = query.single_mut() else { return; };

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

    // Always face the locked-on enemy (local +Y is forward).
    let target = Quat::from_rotation_arc_2d(Vec2::Y, to_enemy.normalize());
    transform.rotation = transform.rotation.rotate_towards(target, player.turn_speed * dt);
}

pub fn find_closest_enemy(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Player)>,
    enemies_query: Query<(Entity, &Transform), With<Enemy>>,
    current_closest_query: Query<Entity, With<Closest>>,
){
    let Ok((player_transform, mut player)) = player_query.single_mut() else {
        return;
    };

    // Keep the current lock target while it still exists. If it died, release
    // the lock so we fall back to the nearest enemy (or free movement).
    if player.locked_on {
        if current_closest_query.iter().next().is_some() {
            return;
        }
        player.locked_on = false;
    }

    let player_pos = player_transform.translation.truncate();

    let nearest = enemies_query
        .iter()
        .min_by(|(_, transform_a), (_, transform_b)| {
            let dist_a = player_pos.distance_squared(transform_a.translation.truncate());
            let dist_b = player_pos.distance_squared(transform_b.translation.truncate());

            dist_a.total_cmp(&dist_b)
        })
        .map(|(entity, _)| entity);

    // Guarantee at most one `Closest`, and that it is the nearest enemy.
    for entity in current_closest_query.iter() {
        if Some(entity) != nearest {
            commands.entity(entity).remove::<Closest>();
        }
    }

    if let Some(nearest) = nearest {
        if !current_closest_query.contains(nearest) {
            commands.entity(nearest).insert(Closest);
        }
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

    if let Some(enemy_transform) = closest_enemy_query.iter().next() {
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

    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }

    if player.locked_on {
        // Always allow releasing, even if the target disappeared.
        player.locked_on = false;
    } else if closest_enemy_query.iter().next().is_some() {
        player.locked_on = true;
    }
}