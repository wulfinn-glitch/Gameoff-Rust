use std::collections::HashMap;
use bevy::diagnostic::{Diagnostics, DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_pretty_text::prelude::*;

#[derive(Component)]
pub struct FpsText;

#[derive(Resource)]
pub struct FpsUpdateTimer(Timer);

const DOUBLE_PRESS_WINDOW: f32 = 0.25;

#[derive(Resource, Default)]
pub struct DoublePressTracker {
    last_pressed: HashMap<KeyCode, f32>,
}

impl DoublePressTracker {
    pub fn check(&mut self, key: KeyCode, keys: &ButtonInput<KeyCode>, now: f32) -> bool {
        if !keys.just_pressed(key) { return false; }

        if let Some(&prev) = self.last_pressed.get(&key) {
            if now - prev <= DOUBLE_PRESS_WINDOW {
                self.last_pressed.remove(&key);
                return true;
            }
        }

        self.last_pressed.insert(key, now);

        false
    }
}

pub fn setup_fps_counter(mut commands: Commands) {
    commands.insert_resource(
        FpsUpdateTimer(Timer::from_seconds(0.25, TimerMode::Repeating))
    );
    commands.spawn((
        FpsText,
        Node{
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Text::new("FPS 0.0".to_string()),
        TextFont{
            font_size: 10.0,
            ..default()
        }
    ));
}

pub fn show_fps(
    time: Res<Time>,
    mut timer: ResMut<FpsUpdateTimer>,
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {

    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                **text = format!("FPS {:.1}", value);
            }
        }
    }
}

pub fn spawn_text(mut commands: Commands, mut materials: ResMut<Assets<Rainbow>>) {
    // Spawn wavy `Text`.
    commands.spawn((
        Text2d::new("Hello, World!"),
        Transform::from_xyz(200.0, 0., 0.),
        Wave::default(),
        TextFont::from_font_size(22.0),
    ));

    // Use the typewriter.
    commands.spawn((
        Typewriter::new(12.),
        Text2d::new("My text is revealed\none glyph at a time"),
        Transform::from_xyz(0., 200., 0.),
        TextFont::from_font_size(36.0),
    ));

    // Spawn a style entity.
    commands.spawn((
        PrettyStyle("my_style"),
        TextFont::from_font_size(48.0),
        effects![
            PrettyTextMaterial(materials.add(Rainbow::default())),
            Wave{
                frequency: 2.0,
                height: 0.1,
                ..default()
            }
        ],
    ));

    // Parse rich text and use custom style.
    commands.spawn((
        pretty!("I am |0.5|<0.8>*sniff*|1|<1.2> very [pretty](my_style)!|3|<1>"),
        Typewriter::new(10.0),
        TextFont::from_font_size(52.0),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..Default::default()
        },
    ));
}