use crate::{layer, HitPoints, TaipoStage};
use bevy::prelude::*;

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // hack: catch goal healthbar spawn
        app.add_system_to_stage(TaipoStage::AfterUpdate, update.system());
        app.init_resource::<HealthBarMaterials>();
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

pub struct HealthBarMaterials {
    background: Handle<ColorMaterial>,
    healthy: Handle<ColorMaterial>,
    injured: Handle<ColorMaterial>,
    critical: Handle<ColorMaterial>,
    invisible: Handle<ColorMaterial>,
}

impl FromWorld for HealthBarMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        HealthBarMaterials {
            background: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
            healthy: materials.add(Color::GREEN.into()),
            injured: materials.add(Color::YELLOW.into()),
            critical: materials.add(Color::RED.into()),
            invisible: materials.add(Color::NONE.into()),
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
            material: materials.healthy.clone(),
            transform: Transform::from_translation(healthbar.offset.extend(layer::HEALTHBAR)),
            sprite: Sprite::new(Vec2::new(healthbar.size.x, healthbar.size.y)),
            ..Default::default()
        })
        .insert(HealthBarBar)
        .id();
    let background = commands
        .spawn_bundle(SpriteBundle {
            material: materials.background.clone(),
            transform: Transform::from_translation(healthbar.offset.extend(layer::HEALTHBAR_BG)),
            sprite: Sprite::new(Vec2::new(healthbar.size.x + 2.0, healthbar.size.y + 2.0)),
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
    mut query: Query<(&mut Transform, &mut Sprite, &mut Handle<ColorMaterial>), With<HealthBarBar>>,
    parent_query: Query<(&HealthBar, &HitPoints, &Children), (With<HealthBar>, Changed<HitPoints>)>,
    mut bg_query: Query<
        &mut Handle<ColorMaterial>,
        (With<HealthBarBackground>, Without<HealthBarBar>),
    >,
    materials: Res<HealthBarMaterials>,
) {
    for (healthbar, hp, children) in parent_query.iter() {
        let mut frac = hp.current as f32 / hp.max as f32;
        frac = frac.max(0.0).min(1.0);

        for child in children.iter() {
            // Update the bar itself

            if let Ok((mut transform, mut sprite, mut mat_handle)) = query.get_mut(*child) {
                if (hp.current == hp.max && !healthbar.show_full)
                    || (hp.current == 0 && !healthbar.show_empty)
                {
                    *mat_handle = materials.invisible.clone();
                } else if frac < 0.25 {
                    *mat_handle = materials.critical.clone();
                } else if frac < 0.75 {
                    *mat_handle = materials.injured.clone();
                } else {
                    *mat_handle = materials.healthy.clone();
                };

                let w = frac * healthbar.size.x;
                sprite.size.x = w;
                transform.translation.x = (healthbar.size.x - w) / -2.0;
            }

            // Update the bar background

            if let Ok(mut bg_material) = bg_query.get_mut(*child) {
                if (hp.current == hp.max && !healthbar.show_full)
                    || (hp.current == 0 && !healthbar.show_empty)
                {
                    *bg_material = materials.invisible.clone();
                } else {
                    *bg_material = materials.background.clone();
                }
            }
        }
    }
}
