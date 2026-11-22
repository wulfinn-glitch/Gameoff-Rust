use bevy::prelude::*;
use crate::health::Health;
use crate::posture::Posture;

pub trait StatBar: Component {
    fn percent(&self) -> f32;
}

impl StatBar for Health {
    fn percent(&self) -> f32 {
        calc_percent(self.current, self.min, self.max)
    }
}
impl StatBar for Posture {
    fn percent(&self) -> f32 {
        calc_percent(self.current, self.min, self.max)
    }
}

fn calc_percent(current: i32, min: i32, max: i32) -> f32 {
    let range = (max - min) as f32;
    if range > 0.0 {
        ((current - min) as f32 / range).clamp(0.0, 1.0) * 100.0
    } else {
        0.0
    }
}

// Generic
#[derive(Component)]
pub struct UIBarFill<T: StatBar> {
    pub target: Entity,
    _marker: std::marker::PhantomData<T>,
}
impl<T: StatBar> UIBarFill<T> {
    pub fn new(target: Entity) -> Self {
        Self{
            target,
            _marker: std::marker::PhantomData,
        }
    }
}

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

pub fn spawn_stat_bar<T: StatBar>(
    parent: &mut ChildSpawnerCommands,
    target_entity: Entity,
    stat: &T,
    track_color: Color,
    fill_color: Color,
    placement: UIBarPlacement,
) {
parent
    .spawn((
        Node {
            position_type: PositionType::Absolute,
            left: placement.left,
            top: placement.top,
            right: placement.right,
            bottom: placement.bottom,
            width: placement.width,
            height: Val::Px(4.0),
            align_items: AlignItems::Center,
            justify_content: placement.justify_content,
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(track_color),
    ))
    .with_children(|track| {
        track.spawn((
            UIBarFill::<T>::new(target_entity),
            Node {
                width: Val::Percent(stat.percent()),
                height: Val::Px(10.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(fill_color),
        ));
    });
}

pub fn update_stat_bar<T: StatBar>(
    stat_query: Query<&T>,
    mut fill_query: Query<(&UIBarFill<T>, &mut Node)>,
    time: Res<Time>,
) {
    let lerp_speed = 12.0;
    let t = (lerp_speed * time.delta_secs()).min(1.0);

    for (fill, mut node) in &mut fill_query {
        let Ok(stat) = stat_query.get(fill.target) else {
            continue;
        };

        let target_percent = stat.percent();
        let current_percent = match node.width {
            Val::Percent(val) => val,
            _ => target_percent,
        };

        if (target_percent - current_percent).abs() < 0.05 {
            if current_percent != target_percent {
                node.width = Val::Percent(target_percent);
            }
            continue;
        }

        node.width = Val::Percent(current_percent.lerp(target_percent, t));
    }
}