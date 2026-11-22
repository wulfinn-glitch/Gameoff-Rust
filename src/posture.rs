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

pub fn spawn_posture_bar(
    parent: &mut ChildSpawnerCommands,
    target_entity: Entity,
    ui_bar_placement: UIBarPlacement,
){
    parent
        .spawn((
            Node {
                left: ui_bar_placement.left,
                top: ui_bar_placement.top,
                right: ui_bar_placement.right,
                bottom: ui_bar_placement.bottom,
                width: ui_bar_placement.width,
                height: Val::Px(4.0),
                align_items: AlignItems::Center,
                justify_content: ui_bar_placement.justify_content,
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgb(0.22, 0.19, 0.12)),
            ))
            .with_children(|track| {
                track.spawn((
                    UIBarFill(target_entity),
                    PostureBar,
                    Node{
                        width: Val::Percent(100.0),
                        height: Val::Px(15.0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.8745, 0.5569, 0.1137))
                ));
            });
}

pub fn update_posture_bar(
    posture_query: Query<Ref<Posture>>,
    mut fill_query: Query<(&UIBarFill, &mut Node), With<PostureBar>>,
    time: Res<Time>,
){
    let lerp_speed = 12.0;
    let t = (lerp_speed * time.delta_secs()).min(1.0);

    for (fill, mut node) in &mut fill_query {
        let Ok(posture) = posture_query.get(fill.0) else { continue; };

        if !posture.is_changed() { continue; }

        let range = (posture.max - posture.min) as f32;

        let target_percent = if range > 0.0 {
            ((posture.current - posture.min) as f32 / range).clamp(0.0, 1.0) * 100.0
        } else{
            0.0
        };

        // Smooth
        // Read the current displayed width percentage
        let current_percent = match node.width {
            Val::Percent(val) => val,
            _ => target_percent,
        };

        // Skip writing to `node.width` if already within 0.05% (avoids unnecessary UI relayout)
        if (target_percent - current_percent).abs() < 0.05 {
            if current_percent != target_percent {
                node.width = Val::Percent(target_percent);
            }
            continue;
        }

        let next_percent = current_percent.lerp(target_percent, t);
        node.width = Val::Percent(next_percent);
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