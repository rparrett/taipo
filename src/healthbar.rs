use crate::{layer, AfterUpdate, HitPoints, TaipoState};
use bevy::prelude::*;

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        // hack: catch goal healthbar spawn
        app.add_system(
            update
                .in_base_set(AfterUpdate)
                .run_if(in_state(TaipoState::Playing)), // TODO this is new, is it ok?
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
#[derive(Component)]
struct HealthBarBar;
#[derive(Component)]
struct HealthBarBackground;

const HEALTHBAR_BACKGROUND: Color = Color::rgb(0.2, 0.2, 0.2);
const HEALTHBAR_HEALTHY: Color = Color::GREEN;
const HEALTHBAR_INJURED: Color = Color::YELLOW;
const HEALTHBAR_CRITICAL: Color = Color::RED;
const HEALTHBAR_INVISIBLE: Color = Color::NONE;

pub fn spawn(parent: Entity, healthbar: HealthBar, commands: &mut Commands) {
    let bar = commands
        .spawn((
            SpriteBundle {
                transform: Transform {
                    translation: healthbar.offset.extend(layer::HEALTHBAR),
                    scale: Vec3::new(healthbar.size.x, healthbar.size.y, 0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: HEALTHBAR_HEALTHY,
                    ..Default::default()
                },
                ..Default::default()
            },
            HealthBarBar,
        ))
        .id();
    let background = commands
        .spawn((
            SpriteBundle {
                transform: Transform {
                    translation: healthbar.offset.extend(layer::HEALTHBAR_BG),
                    scale: Vec3::new(healthbar.size.x + 2.0, healthbar.size.y + 2.0, 0.0),
                    ..Default::default()
                },
                sprite: Sprite {
                    color: HEALTHBAR_BACKGROUND,
                    ..Default::default()
                },
                ..Default::default()
            },
            HealthBarBackground,
        ))
        .id();

    commands
        .entity(parent)
        .insert(healthbar)
        .push_children(&[bar, background]);
}

fn update(
    mut bar_query: Query<(&mut Transform, &mut Sprite), With<HealthBarBar>>,
    mut bg_query: Query<&mut Sprite, (With<HealthBarBackground>, Without<HealthBarBar>)>,
    health_query: Query<
        (&HealthBar, &HitPoints, &Children),
        (Changed<HitPoints>, Without<HealthBarBar>),
    >,
) {
    for (healthbar, hp, children) in health_query.iter() {
        let mut frac = hp.current as f32 / hp.max as f32;
        frac = frac.clamp(0.0, 1.0);

        for child in children.iter() {
            // Update the bar itself

            if let Ok((mut transform, mut sprite)) = bar_query.get_mut(*child) {
                if (hp.current == hp.max && !healthbar.show_full)
                    || (hp.current == 0 && !healthbar.show_empty)
                {
                    sprite.color = HEALTHBAR_INVISIBLE;
                } else if frac < 0.25 {
                    sprite.color = HEALTHBAR_CRITICAL;
                } else if frac < 0.75 {
                    sprite.color = HEALTHBAR_INJURED;
                } else {
                    sprite.color = HEALTHBAR_HEALTHY;
                };

                let w = frac * healthbar.size.x;
                transform.scale.x = w;
                transform.translation.x = (healthbar.size.x - transform.scale.x) / -2.0;
            }

            // Update the bar background

            if let Ok(mut sprite) = bg_query.get_mut(*child) {
                if (hp.current == hp.max && !healthbar.show_full)
                    || (hp.current == 0 && !healthbar.show_empty)
                {
                    sprite.color = HEALTHBAR_INVISIBLE;
                } else {
                    sprite.color = HEALTHBAR_BACKGROUND;
                }
            }
        }
    }
}
