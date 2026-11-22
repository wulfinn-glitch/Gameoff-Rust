use bevy::prelude::*;
use crate::player::{duration, PlayerAction};
use crate::uibar::*;

#[derive(Component)]
pub struct Posture{
    pub current: i32,
    pub max: i32,
    pub min: i32,
    pub recover_rate: i32,
    pub recovery_timer: Timer
}

#[derive(Component)]
pub struct PostureBar;

impl Default for Posture {
    fn default() -> Self{
        Self {
            current: 1000,
            max: 1000,
            min: 0,
            recover_rate: 10,
            recovery_timer: Timer::from_seconds(duration(PlayerAction::Neutral), TimerMode::Repeating),
        }
    }
}

// TODO: Check for state
pub fn recover_posture(
    mut query: Query<&mut Posture>,
    time: Res<Time>,
){
    for mut posture in &mut query {
        if posture.current >= posture.max {
            continue;
        }
        if posture.recovery_timer.tick(time.delta()).just_finished() {
            posture.current = (posture.current + posture.recover_rate).min(posture.max);
        }
    }
}