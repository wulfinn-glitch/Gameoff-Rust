use bevy::prelude::*;

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