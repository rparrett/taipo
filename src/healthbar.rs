use crate::{layer, HitPoints, TaipoStage};
use bevy::prelude::*;

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        // hack: catch goal healthbar spawn
        app.add_system_to_stage(TaipoStage::AfterUpdate, update.system());
        app.init_resource::<HealthBarMaterials>();
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

pub struct HealthBarMaterials {
    background: Color,
    healthy: Color,
    injured: Color,
    critical: Color,
    invisible: Color,
}

impl FromWorld for HealthBarMaterials {
    fn from_world(world: &mut World) -> Self {
        // TODO this very much does not need to be from_world anymore

        HealthBarMaterials {
            background: Color::rgb(0.2, 0.2, 0.2).into(),
            healthy: Color::GREEN.into(),
            injured: Color::YELLOW.into(),
            critical: Color::RED.into(),
            invisible: Color::NONE.into(),
        }
    }
}

pub fn spawn(
    parent: Entity,
    healthbar: HealthBar,
    commands: &mut Commands,
    materials: &Res<HealthBarMaterials>,
) {
    let bar = commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: healthbar.offset.extend(layer::HEALTHBAR),
                scale: Vec3::new(healthbar.size.x, healthbar.size.y, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: materials.healthy.clone(),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(HealthBarBar)
        .id();
    let background = commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: healthbar.offset.extend(layer::HEALTHBAR_BG),
                scale: Vec3::new(healthbar.size.x + 2.0, healthbar.size.y + 2.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: materials.background.clone(),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(HealthBarBackground)
        .id();

    commands
        .entity(parent)
        .insert(healthbar)
        .push_children(&[bar, background]);
}

#[allow(clippy::type_complexity)]
fn update(
    mut query: Query<(&mut Transform, &mut Sprite), With<HealthBarBar>>,
    parent_query: Query<
        (&HealthBar, &HitPoints, &Children, &Transform),
        (With<HealthBar>, Changed<HitPoints>, Without<HealthBarBar>),
    >,
    mut bg_query: Query<&mut Sprite, (With<HealthBarBackground>, Without<HealthBarBar>)>,
    materials: Res<HealthBarMaterials>,
) {
    for (healthbar, hp, children, parent_transform) in parent_query.iter() {
        let mut frac = hp.current as f32 / hp.max as f32;
        frac = frac.max(0.0).min(1.0);

        for child in children.iter() {
            // Update the bar itself

            if let Ok((mut transform, mut sprite)) = query.get_mut(*child) {
                if (hp.current == hp.max && !healthbar.show_full)
                    || (hp.current == 0 && !healthbar.show_empty)
                {
                    sprite.color = materials.invisible.clone();
                } else if frac < 0.25 {
                    sprite.color = materials.critical.clone();
                } else if frac < 0.75 {
                    sprite.color = materials.injured.clone();
                } else {
                    sprite.color = materials.healthy.clone();
                };

                let w = frac * healthbar.size.x;
                transform.scale.x = w;
                transform.translation.x = (parent_transform.scale.x - w) / -2.0;
            }

            // Update the bar background

            if let Ok(mut sprite) = bg_query.get_mut(*child) {
                if (hp.current == hp.max && !healthbar.show_full)
                    || (hp.current == 0 && !healthbar.show_empty)
                {
                    sprite.color = materials.invisible.clone();
                } else {
                    sprite.color = materials.background.clone();
                }
            }
        }
    }
}
