use bevy::prelude::*;

/// Global game-feel state: camera-shake trauma and a hitstop timer.
#[derive(Resource, Default)]
pub struct CombatFeel {
    trauma: f32,
    hitstop: f32,
}

impl CombatFeel {
    /// Register an impact. `trauma` is roughly 0..1; `hitstop` is seconds of
    /// near-freeze (scaled by the real clock, not the slowed game clock).
    pub fn punch(&mut self, trauma: f32, hitstop: f32) {
        self.trauma = (self.trauma + trauma).min(1.0);
        self.hitstop = self.hitstop.max(hitstop);
    }
}

/// How slow the world runs during hitstop.
const HITSTOP_SPEED: f32 = 0.04;
/// Maximum camera offset in pixels at full trauma.
const MAX_SHAKE: f32 = 38.0;
/// How quickly trauma bleeds off per real second.
const TRAUMA_DECAY: f32 = 1.8;

pub fn apply_hitstop(
    real: Res<Time<Real>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut feel: ResMut<CombatFeel>,
) {
    if feel.hitstop > 0.0 {
        feel.hitstop = (feel.hitstop - real.delta_secs()).max(0.0);
    }

    if feel.hitstop > 0.0 {
        virtual_time.set_relative_speed(HITSTOP_SPEED);
    } else if virtual_time.relative_speed() != 1.0 {
        virtual_time.set_relative_speed(1.0);
    }
}

pub fn apply_camera_shake(
    real: Res<Time<Real>>,
    mut feel: ResMut<CombatFeel>,
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
    mut base: Local<Option<Vec3>>,
) {
    feel.trauma = (feel.trauma - real.delta_secs() * TRAUMA_DECAY).max(0.0);

    let Ok(mut transform) = camera_q.single_mut() else {
        return;
    };
    let base = *base.get_or_insert(transform.translation);

    if feel.trauma <= 0.0 {
        transform.translation = base;
        return;
    }

    // 1.5 curve: small hits stay subtle, big hits kick hard.
    let amp = MAX_SHAKE * feel.trauma.powf(1.5);

    // Cheap deterministic noise: two out-of-phase sines per axis.
    let t = real.elapsed_secs();
    let offset = Vec2::new(
        (t * 143.0).sin() + (t * 317.0).sin() * 0.6,
        (t * 191.0).cos() + (t * 271.0).cos() * 0.6,
    ) * amp;

    transform.translation = base + offset.extend(0.0);
}
