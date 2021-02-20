use crate::{layer, HitPoints, TaipoStage};
use bevy::prelude::*;

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // hack: catch goal healthbar spawn
        app.add_system_to_stage(TaipoStage::AfterState, update.system());
    }
}

struct HealthBar {
    size: Vec2,
    show_full: bool,
    show_empty: bool,
}
struct HealthBarBar;
struct HealthBarBackground;

pub fn spawn(
    entity: Entity,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    size: Vec2,
    offset: Vec2,
    show_full: bool,
    show_empty: bool,
) {
    commands.insert_one(
        entity,
        HealthBar {
            size,
            show_full,
            show_empty,
        },
    );

    let current = commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
            transform: Transform::from_translation(offset.extend(layer::HEALTHBAR)),
            sprite: Sprite::new(Vec2::new(size.x, size.y)),
            ..Default::default()
        })
        .with(HealthBarBar)
        .current_entity()
        .unwrap();
    let total = commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
            transform: Transform::from_translation(offset.extend(layer::HEALTHBAR_BG)),
            sprite: Sprite::new(Vec2::new(size.x + 2.0, size.y + 2.0)),
            ..Default::default()
        })
        .with(HealthBarBackground)
        .current_entity()
        .unwrap();

    commands.push_children(entity, &[current, total]);
}

fn update(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut Transform, &mut Sprite, &Handle<ColorMaterial>), With<HealthBarBar>>,
    parent_query: Query<(&HealthBar, &HitPoints, &Children), (With<HealthBar>, Changed<HitPoints>)>,
    mut bg_query: Query<&Handle<ColorMaterial>, With<HealthBarBackground>>,
) {
    for (healthbar, hp, children) in parent_query.iter() {
        let mut frac = hp.current as f32 / hp.max as f32;
        frac = frac.max(0.0).min(1.0);

        for child in children.iter() {
            // Update the bar itself

            for (mut transform, mut sprite, mat_handle) in query.get_mut(*child) {
                if let Some(material) = materials.get_mut(mat_handle) {
                    if hp.current == hp.max && !healthbar.show_full {
                        material.color = Color::NONE;
                    } else if hp.current == 0 && !healthbar.show_empty {
                        material.color = Color::NONE;
                    } else if frac < 0.25 {
                        material.color = Color::RED;
                    } else if frac < 0.75 {
                        material.color = Color::YELLOW;
                    } else {
                        material.color = Color::GREEN;
                    };
                }

                let w = frac * healthbar.size.x;
                sprite.size.x = w;
                transform.translation.x = (healthbar.size.x - w) / -2.0;
            }

            // Update the bar background

            for total_mat_handle in bg_query.get_mut(*child) {
                if let Some(total_material) = materials.get_mut(total_mat_handle) {
                    if hp.current == hp.max && !healthbar.show_full {
                        total_material.color = Color::NONE;
                    } else if hp.current == 0 && !healthbar.show_empty {
                        total_material.color = Color::NONE;
                    } else {
                        total_material.color = Color::rgb(0.2, 0.2, 0.2);
                    }
                }
            }
        }
    }
}
