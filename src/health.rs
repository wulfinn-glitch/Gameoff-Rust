use bevy::prelude::*;
use crate::uibar::*;

#[derive(Component)]
pub struct Health{
    pub current: i32,
    pub max: i32,
    pub min: i32
}
impl Default for Health {
    fn default() -> Self{
        Self {
            current: 100,
            max: 100,
            min: 0
        }
    }
}

#[derive(Component)]
pub struct HealthBar;

pub fn spawn_health_bar(
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
            BackgroundColor(Color::srgb(0.15, 0.2, 0.18))
            ))
            .with_children(|track| {
                track.spawn((
                    UIBarFill(target_entity),
                    HealthBar,
                    Node{
                        width: Val::Percent(75.0),
                        height: Val::Px(10.0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.35, 0.9, 0.55))
                ));
            });
}

pub fn update_health_bar(
    health_query: Query<Ref<Health>>,
    mut fill_query: Query<(&UIBarFill, &mut Node), With<HealthBar>>,
){
    for (fill, mut node) in &mut fill_query {
        let Ok(health) = health_query.get(fill.0) else { continue; };

        if !health.is_changed() { continue; }

        let range = (health.max - health.min) as f32;

        let percent = if range > 0.0 {
            ((health.current - health.min) as f32 / range).clamp(0.0, 1.0) * 100.0
        } else{
            0.0
        };

        node.width = Val::Percent(percent);
    }
}

pub fn monitor_health(
    mut query: Query<(Entity, &mut Health)>,
){
    for (entity, mut health) in query.iter_mut() {
        if health.current > health.max {
            println!("Entity {:?} already is full health", entity);
            health.current = health.max;
        }
        if health.current < health.min {
            println!("Entity {:?} is dead", entity);
            health.current = health.min;
        }
    }
}