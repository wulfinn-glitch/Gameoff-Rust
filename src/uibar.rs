use bevy::prelude::{Component, Entity, JustifyContent, Val};

#[derive(Component)]
pub struct UIBarFill(pub Entity);

pub struct UIBarPlacement {
    pub left: Val,
    pub top: Val,
    pub right: Val,
    pub bottom: Val,
    pub width: Val,
    pub justify_content: JustifyContent
}

impl Default for UIBarPlacement{
    fn default() -> Self {
        Self{
            left: Val::Auto,
            top: Val::Auto,
            right: Val::Auto,
            bottom: Val::Auto,
            width: Val::Px(240.0),
            justify_content: JustifyContent::Center,
        }
    }
}