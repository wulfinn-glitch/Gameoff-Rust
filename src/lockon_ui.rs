//! Lock-on reticle drawn in screen space over the locked enemy.
//!
//! No text, no panels: just a bracket ring that snaps on when the lock is made.

use bevy::prelude::*;

use crate::enemy::Closest;
use crate::fx::FxPalette;
use crate::player::Player;

const CYAN: Color = Color::srgb(0.35, 0.95, 1.0);
const RETICLE_SIZE: f32 = 118.0;

/// Root of the screen-space reticle that follows the locked enemy.
#[derive(Component)]
pub struct LockReticle;

/// Rotating bracket ring inside the reticle.
#[derive(Component)]
pub struct ReticleSpin;

/// Pulsing diamond at the reticle centre.
#[derive(Component)]
pub struct ReticlePulse;

/// Marks a static reticle element so it follows the palette colour.
#[derive(Component)]
pub struct ReticleTint {
    pub alpha: f32,
}

#[derive(Resource, Default)]
pub struct LockUi {
    /// 1.0 right after locking on, decays to 0.0; drives the reticle snap.
    pop: f32,
    was_locked: bool,
}

pub fn setup_lock_ui(mut commands: Commands) {
    spawn_reticle(&mut commands);
}

fn spawn_reticle(commands: &mut Commands) {
    commands
        .spawn((
            LockReticle,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(RETICLE_SIZE),
                height: Val::Px(RETICLE_SIZE),
                left: Val::Px(-500.0),
                top: Val::Px(-500.0),
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|root| {
            // Rotating brackets and edge ticks.
            root.spawn((
                ReticleSpin,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                UiTransform::default(),
            ))
            .with_children(|spin| {
                spawn_corner(spin, false, false);
                spawn_corner(spin, true, false);
                spawn_corner(spin, false, true);
                spawn_corner(spin, true, true);

                spawn_edge_tick(spin, TickSide::Top);
                spawn_edge_tick(spin, TickSide::Bottom);
                spawn_edge_tick(spin, TickSide::Left);
                spawn_edge_tick(spin, TickSide::Right);
            });

            // Pulsing centre diamond.
            root.spawn((
                ReticlePulse,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(RETICLE_SIZE * 0.5 - 16.0),
                    top: Val::Px(RETICLE_SIZE * 0.5 - 16.0),
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                Outline::new(Val::Px(2.0), Val::Px(0.0), CYAN),
                UiTransform::from_rotation(Rot2::radians(std::f32::consts::FRAC_PI_4)),
            ));

            // Centre dot.
            root.spawn((
                ReticleTint { alpha: 1.0 },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(RETICLE_SIZE * 0.5 - 2.0),
                    top: Val::Px(RETICLE_SIZE * 0.5 - 2.0),
                    width: Val::Px(4.0),
                    height: Val::Px(4.0),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(CYAN),
                BoxShadow::new(
                    CYAN.with_alpha(0.85),
                    Val::Px(0.0),
                    Val::Px(0.0),
                    Val::Px(0.0),
                    Val::Px(7.0),
                ),
            ));
        });
}

fn spawn_corner(parent: &mut ChildSpawnerCommands, right: bool, bottom: bool) {
    let mut corner = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(34.0),
        height: Val::Px(34.0),
        ..default()
    };
    if right {
        corner.right = Val::Px(0.0);
    } else {
        corner.left = Val::Px(0.0);
    }
    if bottom {
        corner.bottom = Val::Px(0.0);
    } else {
        corner.top = Val::Px(0.0);
    }

    parent.spawn((corner, UiTransform::default())).with_children(|c| {
        // Horizontal arm.
        let mut h = Node {
            position_type: PositionType::Absolute,
            width: Val::Px(34.0),
            height: Val::Px(3.0),
            ..default()
        };
        if right {
            h.right = Val::Px(0.0);
        } else {
            h.left = Val::Px(0.0);
        }
        if bottom {
            h.bottom = Val::Px(0.0);
        } else {
            h.top = Val::Px(0.0);
        }
        c.spawn((
            ReticleTint { alpha: 1.0 },
            h,
            BackgroundColor(CYAN),
            BoxShadow::new(
                CYAN.with_alpha(0.55),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(6.0),
            ),
        ));

        // Vertical arm.
        let mut v = Node {
            position_type: PositionType::Absolute,
            width: Val::Px(3.0),
            height: Val::Px(34.0),
            ..default()
        };
        if right {
            v.right = Val::Px(0.0);
        } else {
            v.left = Val::Px(0.0);
        }
        if bottom {
            v.bottom = Val::Px(0.0);
        } else {
            v.top = Val::Px(0.0);
        }
        c.spawn((
            ReticleTint { alpha: 1.0 },
            v,
            BackgroundColor(CYAN),
            BoxShadow::new(
                CYAN.with_alpha(0.55),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(6.0),
            ),
        ));
    });
}

enum TickSide {
    Top,
    Bottom,
    Left,
    Right,
}

fn spawn_edge_tick(parent: &mut ChildSpawnerCommands, side: TickSide) {
    let mut node = Node {
        position_type: PositionType::Absolute,
        ..default()
    };
    match side {
        TickSide::Top => {
            node.width = Val::Px(12.0);
            node.height = Val::Px(3.0);
            node.left = Val::Px((RETICLE_SIZE - 12.0) * 0.5);
            node.top = Val::Px(0.0);
        }
        TickSide::Bottom => {
            node.width = Val::Px(12.0);
            node.height = Val::Px(3.0);
            node.left = Val::Px((RETICLE_SIZE - 12.0) * 0.5);
            node.bottom = Val::Px(0.0);
        }
        TickSide::Left => {
            node.width = Val::Px(3.0);
            node.height = Val::Px(12.0);
            node.left = Val::Px(0.0);
            node.top = Val::Px((RETICLE_SIZE - 12.0) * 0.5);
        }
        TickSide::Right => {
            node.width = Val::Px(3.0);
            node.height = Val::Px(12.0);
            node.right = Val::Px(0.0);
            node.top = Val::Px((RETICLE_SIZE - 12.0) * 0.5);
        }
    }

    parent.spawn((
        ReticleTint { alpha: 0.65 },
        node,
        BackgroundColor(CYAN.with_alpha(0.65)),
    ));
}

pub fn tint_reticle(
    palette: Res<FxPalette>,
    mut q: Query<(&ReticleTint, Option<&mut BackgroundColor>, Option<&mut BoxShadow>)>,
) {
    let color = palette.reticle_color();

    for (tint, bg, shadow) in &mut q {
        if let Some(mut bg) = bg {
            bg.0 = color.with_alpha(tint.alpha);
        }
        if let Some(mut shadow) = shadow {
            if let Some(style) = shadow.0.first_mut() {
                style.color = color.with_alpha(tint.alpha * 0.7);
            }
        }
    }
}

pub fn update_lock_reticle(
    time: Res<Time>,
    mut state: ResMut<LockUi>,
    palette: Res<FxPalette>,
    player_q: Query<&Player>,
    target_q: Query<&Transform, With<Closest>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut reticle_q: Query<(&mut Node, &mut Visibility), With<LockReticle>>,
    mut spin_q: Query<&mut UiTransform, (With<ReticleSpin>, Without<ReticlePulse>)>,
    mut pulse_q: Query<(&mut UiTransform, &mut Outline), (With<ReticlePulse>, Without<ReticleSpin>)>,
) {
    let locked = player_q.iter().next().map(|p| p.locked_on).unwrap_or(false);

    if locked != state.was_locked {
        state.was_locked = locked;
        if locked {
            state.pop = 1.0;
        }
    }
    state.pop = (state.pop - time.delta_secs() * 4.0).max(0.0);

    let Ok((mut node, mut visibility)) = reticle_q.single_mut() else {
        return;
    };

    let screen = if locked {
        match (target_q.iter().next(), camera_q.single()) {
            (Some(target), Ok((camera, cam_tf))) => {
                camera.world_to_viewport(cam_tf, target.translation).ok()
            }
            _ => None,
        }
    } else {
        None
    };

    let Some(pos) = screen else {
        *visibility = Visibility::Hidden;
        return;
    };

    // Hide if the target is outside the viewport.
    if let Ok((camera, _)) = camera_q.single() {
        if let Some(size) = camera.logical_viewport_size() {
            let margin = RETICLE_SIZE;
            if pos.x < -margin
                || pos.y < -margin
                || pos.x > size.x + margin
                || pos.y > size.y + margin
            {
                *visibility = Visibility::Hidden;
                return;
            }
        }
    }

    *visibility = Visibility::Inherited;
    let half = RETICLE_SIZE * 0.5;
    node.left = Val::Px(pos.x - half);
    node.top = Val::Px(pos.y - half);

    let t = time.elapsed_secs();
    let pop = state.pop;
    let snap = 1.0 + 0.55 * pop * pop;

    if let Ok(mut spin) = spin_q.single_mut() {
        spin.rotation = Rot2::radians(t * 1.7 + pop * 2.4);
        spin.scale = Vec2::splat(snap);
    }

    if let Ok((mut pulse, mut outline)) = pulse_q.single_mut() {
        let breathe = 1.0 + 0.10 * (t * 6.0).sin();
        pulse.rotation = Rot2::radians(std::f32::consts::FRAC_PI_4 + t * 0.9);
        pulse.scale = Vec2::splat(snap * breathe);

        let glow = 0.55 + 0.45 * (t * 6.0).sin().abs();
        outline.color = palette.reticle_color().with_alpha(glow);
    }
}
