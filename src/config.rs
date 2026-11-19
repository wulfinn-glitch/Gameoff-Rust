use bevy::prelude::*;
use bevy_rapier2d::render::DebugRenderContext;

#[derive(Resource, Default)]
pub struct DebugMode(pub bool);

pub fn sync_rapier_debug(
    debug_mode: Res<DebugMode>,
    mut debug_context: ResMut<DebugRenderContext>,
) {
    debug_context.enabled = debug_mode.0;
}

pub fn toggle_debug_mode(
    mut debug_mode: ResMut<DebugMode>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        debug_mode.0 = !debug_mode.0;
    }
}