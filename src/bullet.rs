use crate::{ActionPanel, AnimationState, EnemyState, GameState, HitPoints, TextureHandles};
use bevy::prelude::*;

pub struct BulletPlugin;

struct Bullet {
    target: Entity,
    damage: u32,
    speed: f32,
}

pub fn spawn(
    mut position: Vec3,
    target: Entity,
    damage: u32,
    speed: f32,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    texture_handles: &Res<TextureHandles>,
) {
    position.z = 10.0;

    commands
        .spawn(SpriteBundle {
            material: materials.add(texture_handles.bullet_shuriken.clone().into()),
            transform: Transform::from_translation(position),
            ..Default::default()
        })
        .with(Bullet {
            target,
            damage,
            speed,
        });
}

fn update(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Bullet)>,
    mut target_query: Query<(&mut Transform, &mut HitPoints, &mut EnemyState)>,
    mut game_state: ResMut<GameState>,
    mut action_panel: ResMut<ActionPanel>,
) {
    for (entity, mut transform, bullet) in query.iter_mut() {
        if let Ok((target_transform, mut hp, mut state)) = target_query.get_mut(bullet.target) {
            let d = transform
                .translation
                .truncate()
                .distance(target_transform.translation.truncate());

            let speed = bullet.speed;
            let delta = time.delta_seconds();
            let step = speed * delta;

            if step > d {
                hp.current = hp.current.saturating_sub(bullet.damage);

                // not sure how responsible I want bullet.rs to be for enemy animation.
                // should probably get this outta here when enemy.rs exists.
                if hp.current == 0 {
                    state.state = AnimationState::Corpse;

                    game_state.primary_currency = game_state.primary_currency.saturating_add(1);
                    game_state.score = game_state.score.saturating_add(1);
                    action_panel.update += 1;
                }

                commands.despawn_recursive(entity);
                continue;
            }

            transform.translation.x +=
                step / d * (target_transform.translation.x - transform.translation.x);
            transform.translation.y +=
                step / d * (target_transform.translation.y - transform.translation.y);

            // ten radians per second, clockwise
            transform.rotate(Quat::from_rotation_z(-10.0 * delta));
        } else {
            commands.despawn_recursive(entity);
            continue;
        }
    }
}

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(update.system().before("update_currency_display"));
    }
}
