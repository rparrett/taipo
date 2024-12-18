use bevy::prelude::*;

use crate::{enemy::death, layer, Armor, HitPoints, StatusEffect, StatusEffects, TaipoState};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update.before(death).run_if(in_state(TaipoState::Playing)),
        );
    }
}

#[derive(Component)]
#[require(Sprite)]
pub struct Bullet {
    target: Entity,
    damage: u32,
    speed: f32,
    status_effect: Option<StatusEffect>,
}
impl Bullet {
    pub fn bundle(
        position: Vec2,
        image: Handle<Image>,
        target: Entity,
        damage: u32,
        speed: f32,
        status_effect: Option<StatusEffect>,
    ) -> impl Bundle {
        (
            Sprite { image, ..default() },
            Transform::from_translation(position.extend(layer::BULLET)),
            Bullet {
                target,
                damage,
                speed,
                status_effect,
            },
        )
    }
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Bullet)>,
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
        let Ok((target_transform, mut target_hp, target_armor, target_status)) =
            target_query.get_mut(bullet.target)
        else {
            commands.entity(entity).despawn_recursive();
            continue;
        };

        let target_pos = target_transform.translation.truncate();
        let bullet_pos = transform.translation.truncate();

        let dist = bullet_pos.distance(target_pos);

        let delta = time.delta_secs();
        let step = bullet.speed * delta;

        if step < dist {
            let dir = (target_pos - bullet_pos).normalize_or_zero();
            transform.translation += (dir * step).extend(0.);

            // ten radians per second, clockwise
            transform.rotate(Quat::from_rotation_z(-10.0 * delta));

            continue;
        }

        // bullet has hit its target

        let mut armor = target_armor.0;

        if let Some(mut target_status) = target_status {
            armor = armor.saturating_sub(target_status.get_max_sub_armor());

            if let Some(bullet_status) = bullet.status_effect.take() {
                target_status.0.push(bullet_status);
            }
        }

        let damage = bullet.damage.saturating_sub(armor);

        target_hp.current = target_hp.current.saturating_sub(damage);

        commands.entity(entity).despawn_recursive();
    }
}
