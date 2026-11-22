pub mod player;
pub mod enemy;
pub mod helpers;
pub mod config;
pub mod bullet;
pub mod health;
pub mod states;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{PowerPreference, WgpuSettings};
use bevy::window::{PresentMode, WindowMode, WindowResolution};

use bevy_egui::EguiPlugin;
use bevy_pretty_text::prelude::*;
use player::*;
use crate::bullet::*;
use crate::enemy::*;
use crate::config::{sync_rapier_debug, DebugMode, toggle_debug_mode};
use crate::health::monitor_health;
use crate::helpers::{setup_fps_counter, show_fps, spawn_text, DoublePressTracker};
use crate::states::{MyGamplaySet, MyPausedState};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Game Off Gamejam".into(),
                resolution: WindowResolution::new(800, 800),
                mode: WindowMode::Windowed,
                present_mode: PresentMode::AutoNoVsync,
                resizable: false,
                ..default()
            }),
            ..default()
        }).set(RenderPlugin {
            render_creation: WgpuSettings{
                power_preference: PowerPreference::HighPerformance,
                ..default()
            }.into(),
            ..default()
        }) )
        .add_plugins(FrameTimeDiagnosticsPlugin::default()) // <--- Required for FPS data
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default().disabled())
        .add_plugins(EguiPlugin::default())
        .add_plugins(PrettyTextPlugin)
        // .add_plugins(WorldInspectorPlugin::new())

        // Resources
        .insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.12)))
        .insert_resource(DoublePressTracker::default())
        .insert_resource(DebugMode(true))

        // States
        // .insert_state(MyPausedState::default())

        // Configure state sets
        // .configure_sets(Update, MyGamplaySet
        //         .run_if(in_state(MyPausedState::Running))
        // )

        // Systems
        .add_systems(
            Update,
            (sync_rapier_debug.run_if(resource_changed::<DebugMode>),
             toggle_debug_mode)
        )
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_fps_counter)

        // Helpers
        .add_systems(Update, show_fps)
        .add_systems(Update, monitor_health)

        // Player systems
        .add_systems(Update, normal_movement)
        .add_systems(Update, orbital_movement)
        .add_systems(Update, find_closest_enemy)
        .add_systems(Update, (start_action, tick_action).chain())
        .add_systems(Update, debug_action)

        // Enemy Systems
        .add_systems(Update, detect_player_in_range)
        .add_systems(Update, target_closest_enemy)
        .add_systems(Update, lock_unlock_closest_enemy)
        .add_systems(Update, move_to_preferred_dist)

        // Projectile System
        .add_systems(Update, shoot_player)
        .add_systems(Update, move_bullets)
        .add_systems(Update, despawn_expired_bullets)

        // .add_systems(Startup, spawn_text)
        .add_systems(Update, draw_detect_dist) // Run if debug on
        .run();
}

fn setup(mut commands: Commands,
                mut meshes: ResMut<Assets<Mesh>>,
                mut materials: ResMut<Assets<ColorMaterial>>)
{
    // Spawn Camera Bundle (Bundles can always be accessed by the Transform)
    commands.spawn((Camera2d, Msaa::Sample4));

    spawn_player(&mut commands, &mut meshes, &mut materials, Transform::from_xyz(0.0, -320.0, 0.0));

    // spawn_enemy(&mut commands, &mut meshes, &mut materials, Transform::from_xyz(-200.0, 320.0, 0.0));
    // spawn_enemy(&mut commands, &mut meshes, &mut materials, Transform::from_xyz(200.0, 320.0, 0.0));
    spawn_enemy(&mut commands, &mut meshes, &mut materials, Transform::from_xyz(0.0, 220.0, 0.0));
    // spawn_enemy(&mut commands, &mut meshes, &mut materials, Transform::from_xyz(-200.0, 120.0, 0.0));

}
