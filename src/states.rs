use bevy::prelude::*;

// Not using these --------------------------
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MyPausedState {
    #[default]
    Paused,
    Running,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MyGamplaySet;

// ------------------------------------------

