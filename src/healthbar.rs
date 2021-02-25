use crate::{layer, HitPoints, TaipoStage};
use bevy::prelude::*;

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // hack: catch goal healthbar spawn
        app.add_system_to_stage(TaipoStage::AfterState, update.system());
    }
}

pub struct HealthBar {
    pub size: Vec2,
    pub offset: Vec2,
    pub show_full: bool,
    pub show_empty: bool,
}
struct HealthBarBar;
struct HealthBarBackground;

pub fn spawn(
    parent: Entity,
    healthbar: HealthBar,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let current = commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
            transform: Transform::from_translation(healthbar.offset.extend(layer::HEALTHBAR)),
            sprite: Sprite::new(Vec2::new(healthbar.size.x, healthbar.size.y)),
            ..Default::default()
        })
        .with(HealthBarBar)
        .current_entity()
        .unwrap();
    let total = commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
            transform: Transform::from_translation(healthbar.offset.extend(layer::HEALTHBAR_BG)),
            sprite: Sprite::new(Vec2::new(healthbar.size.x + 2.0, healthbar.size.y + 2.0)),
            ..Default::default()
        })
        .with(HealthBarBackground)
        .current_entity()
        .unwrap();

    commands.insert_one(parent, healthbar);

    commands.push_children(parent, &[current, total]);
}

#[allow(clippy::type_complexity)]
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

            if let Ok((mut transform, mut sprite, mat_handle)) = query.get_mut(*child) {
                if let Some(material) = materials.get_mut(mat_handle) {
                    if (hp.current == hp.max && !healthbar.show_full)
                        || (hp.current == 0 && !healthbar.show_empty)
                    {
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

            if let Ok(total_mat_handle) = bg_query.get_mut(*child) {
                if let Some(total_material) = materials.get_mut(total_mat_handle) {
                    if (hp.current == hp.max && !healthbar.show_full)
                        || (hp.current == 0 && !healthbar.show_empty)
                    {
                        total_material.color = Color::NONE;
                    } else {
                        total_material.color = Color::rgb(0.2, 0.2, 0.2);
                    }
                }
            }
        }
    }
}
