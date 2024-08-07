use bevy::{
    color::palettes::css::{LIME, RED, YELLOW},
    prelude::*,
};

use crate::{layer, AfterUpdate, HitPoints, TaipoState};

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            AfterUpdate,
            (update, spawn).run_if(in_state(TaipoState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct HealthBar {
    pub size: Vec2,
    pub offset: Vec2,
    pub show_full: bool,
    pub show_empty: bool,
}
impl Default for HealthBar {
    fn default() -> Self {
        Self {
            size: Vec2::new(16.0, 2.0),
            offset: Vec2::ZERO,
            show_full: false,
            show_empty: false,
        }
    }
}
#[derive(Component)]
struct HealthBarBar;
#[derive(Component)]
struct HealthBarBackground;

const HEALTHBAR_BACKGROUND: Srgba = Srgba::rgb(0.2, 0.2, 0.2);
const HEALTHBAR_HEALTHY: Srgba = LIME;
const HEALTHBAR_INJURED: Srgba = YELLOW;
const HEALTHBAR_CRITICAL: Srgba = RED;
const HEALTHBAR_INVISIBLE: Srgba = Srgba::NONE;

pub fn spawn(mut commands: Commands, query: Query<(Entity, &HealthBar), Added<HealthBar>>) {
    for (entity, healthbar) in &query {
        let bar = commands
            .spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: healthbar.offset.extend(layer::HEALTHBAR),
                        scale: healthbar.size.extend(1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: HEALTHBAR_HEALTHY.into(),
                        ..default()
                    },
                    ..default()
                },
                HealthBarBar,
            ))
            .id();

        let background = commands
            .spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: healthbar.offset.extend(layer::HEALTHBAR_BG),
                        scale: Vec3::new(healthbar.size.x + 2.0, healthbar.size.y + 2.0, 1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: HEALTHBAR_BACKGROUND.into(),
                        ..default()
                    },
                    ..default()
                },
                HealthBarBackground,
            ))
            .id();

        commands.entity(entity).push_children(&[bar, background]);
    }
}

fn update(
    mut bar_query: Query<(&mut Transform, &mut Sprite), With<HealthBarBar>>,
    mut bg_query: Query<&mut Sprite, (With<HealthBarBackground>, Without<HealthBarBar>)>,
    health_query: Query<(&HealthBar, &HitPoints, &Children), Changed<HitPoints>>,
) {
    for (healthbar, hp, children) in health_query.iter() {
        let frac = (hp.current as f32 / hp.max as f32).clamp(0.0, 1.0);

        let invisible = (hp.current >= hp.max && !healthbar.show_full)
            || (hp.current == 0 && !healthbar.show_empty);

        for child in children {
            // Update the bar itself

            if let Ok((mut transform, mut sprite)) = bar_query.get_mut(*child) {
                if invisible {
                    sprite.color = HEALTHBAR_INVISIBLE.into();
                } else if frac < 0.25 {
                    sprite.color = HEALTHBAR_CRITICAL.into();
                } else if frac < 0.75 {
                    sprite.color = HEALTHBAR_INJURED.into();
                } else {
                    sprite.color = HEALTHBAR_HEALTHY.into();
                };

                let current_width = frac * healthbar.size.x;

                transform.translation.x = (healthbar.size.x - current_width) / -2.0;
                transform.scale.x = current_width;
            }

            // Update the bar background

            if let Ok(mut sprite) = bg_query.get_mut(*child) {
                if invisible {
                    sprite.color = HEALTHBAR_INVISIBLE.into();
                } else {
                    sprite.color = HEALTHBAR_BACKGROUND.into();
                }
            }
        }
    }
}
