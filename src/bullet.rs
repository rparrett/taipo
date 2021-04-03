use crate::{layer, Armor, HitPoints, StatusEffect, StatusEffects};
use bevy::prelude::*;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(update.system().before("enemy_death"));
    }
}

struct Bullet {
    target: Entity,
    damage: u32,
    speed: f32,
    status_effect: Option<StatusEffect>,
}

pub fn spawn(
    mut position: Vec3,
    target: Entity,
    damage: u32,
    speed: f32,
    status_effect: Option<StatusEffect>,
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
) {
    position.z = layer::BULLET;

    commands
        .spawn_bundle(SpriteBundle {
            material,
            transform: Transform::from_translation(position),
            ..Default::default()
        })
        .insert(Bullet {
            target,
            damage,
            speed,
            status_effect,
        });
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Bullet), With<Bullet>>,
    mut target_query: Query<
        (
            &Transform,
            &mut HitPoints,
            &Armor,
            Option<&mut StatusEffects>,
        ),
        Without<Bullet>,
    >,
) {
    for (entity, mut transform, mut bullet) in query.iter_mut() {
        if let Ok((target_transform, mut hp, target_armor, target_status)) =
            target_query.get_mut(bullet.target)
        {
            let dist = transform
                .translation
                .truncate()
                .distance(target_transform.translation.truncate());

            let delta = time.delta_seconds();
            let step = bullet.speed * delta;

            if step < dist {
                transform.translation.x +=
                    step / dist * (target_transform.translation.x - transform.translation.x);
                transform.translation.y +=
                    step / dist * (target_transform.translation.y - transform.translation.y);

                // ten radians per second, clockwise
                transform.rotate(Quat::from_rotation_z(-10.0 * delta));
            } else {
                let mut armor = target_armor.0;

                if let Some(mut target_status) = target_status {
                    armor = armor.saturating_sub(target_status.get_max_sub_armor());

                    if let Some(bullet_status) = bullet.status_effect.take() {
                        target_status.0.push(bullet_status);
                    }
                }

                let damage = bullet.damage.saturating_sub(armor);

                hp.current = hp.current.saturating_sub(damage);

                commands.entity(entity).despawn_recursive();
            }
        } else {
            commands.entity(entity).despawn_recursive();
        }
    }
}
