use crate::{
    healthbar::HealthBar,
    layer,
    loading::{EnemyAnimationHandles, TextureHandles},
    update_currency_text, ActionPanel, AnimationData, Armor, Currency, Goal, HitPoints, Speed,
    StatusDownSprite, StatusEffects, StatusUpSprite, TaipoStage, TaipoState,
};
use bevy::{ecs::query::Or, prelude::*};
use rand::{thread_rng, Rng};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(TaipoState::Playing)
                .with_system(animate)
                .with_system(movement)
                .with_system(deal_damage)
                .with_system(death.before(update_currency_text)),
        )
        .add_system_set_to_stage(
            TaipoStage::AfterUpdate,
            SystemSet::on_update(TaipoState::Playing).with_system(status_effect_appearance),
        );
    }
}
#[derive(Bundle, Default)]
pub struct EnemyBundle {
    pub kind: EnemyKind,
    pub path: EnemyPath,
    pub animation_tick: AnimationTick,
    pub animation_timer: AnimationTimer,
    pub animation_state: AnimationState,
    pub direction: Direction,
    pub attack_timer: AttackTimer,
    pub hit_points: HitPoints,
    pub status_effects: StatusEffects,
    pub armor: Armor,
    pub speed: Speed,
}

#[derive(Component, Debug)]
pub enum AnimationState {
    Idle,
    Walking,
    Attacking,
    Corpse,
}
impl Default for AnimationState {
    fn default() -> Self {
        AnimationState::Idle
    }
}

#[derive(Component, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Default for Direction {
    fn default() -> Self {
        Direction::Right
    }
}
#[derive(Component, Default, Debug)]
pub struct EnemyKind(pub String);

#[derive(Component, Default, Debug)]
pub struct EnemyPath {
    pub path: Vec<Vec2>,
    pub path_index: usize,
}

#[derive(Component, Default)]
pub struct AnimationTick(pub u32);
#[derive(Component)]
pub struct AnimationTimer(pub Timer);
impl Default for AnimationTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, true))
    }
}
#[derive(Component)]
pub struct AttackTimer(pub Timer);
impl Default for AttackTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, true))
    }
}
pub fn death(
    mut query: Query<(&mut AnimationState, &mut Transform, &HitPoints), Changed<HitPoints>>,
    mut currency: ResMut<Currency>,
    mut action_panel: ResMut<ActionPanel>,
) {
    for (mut state, mut transform, hp) in query.iter_mut() {
        if hp.current == 0 {
            match *state {
                AnimationState::Corpse => {}
                _ => {
                    *state = AnimationState::Corpse;

                    let mut rng = thread_rng();
                    transform.rotate(Quat::from_rotation_z(rng.gen_range(-0.2..0.2)));
                    transform.translation.z = layer::CORPSE;

                    currency.current = currency.current.saturating_add(2);
                    currency.total_earned = currency.total_earned.saturating_add(2);

                    action_panel.update += 1;
                }
            }
        }
    }
}

fn deal_damage(
    time: Res<Time>,
    mut query: Query<(&mut AttackTimer, &AnimationState)>,
    mut goal_query: Query<&mut HitPoints, With<Goal>>,
) {
    // TODO this should really sync up with the animations somehow

    for (mut timer, state) in query.iter_mut() {
        if let AnimationState::Attacking = state {
            timer.0.tick(time.delta());
            if timer.0.finished() {
                for mut hp in goal_query.iter_mut() {
                    hp.current = hp.current.saturating_sub(1);
                }
            }
        }
    }
}

fn status_effect_appearance(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &StatusEffects,
            &AnimationState,
            &HealthBar,
            Option<&Children>,
        ),
        Or<(Changed<AnimationState>, Changed<StatusEffects>)>,
    >,
    up_query: Query<Entity, With<StatusUpSprite>>,
    down_query: Query<Entity, With<StatusDownSprite>>,
    texture_handles: Res<TextureHandles>,
) {
    for (entity, status_effects, state, healthbar, children) in query.iter() {
        let dead = matches!(state, AnimationState::Corpse);

        let down = status_effects.get_max_sub_armor() > 0;
        let up = status_effects.get_total_add_damage() > 0;

        let mut down_sprite = None;
        let mut up_sprite = None;

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(ent) = down_query.get(*child) {
                    down_sprite = Some(ent);
                }
                if let Ok(ent) = up_query.get(*child) {
                    up_sprite = Some(ent);
                }
            }
        }

        if dead {
            if let Some(down_ent) = down_sprite {
                commands.entity(down_ent).despawn();
            }
            if let Some(up_ent) = up_sprite {
                commands.entity(up_ent).despawn();
            }
            break;
        }

        match (down, down_sprite) {
            (true, None) => {
                let down_ent = commands
                    .spawn_bundle(SpriteBundle {
                        texture: texture_handles.status_down.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            healthbar.size.x / 2.0 + 6.0,
                            healthbar.offset.y,
                            layer::HEALTHBAR_BG,
                        )),
                        ..Default::default()
                    })
                    .insert(StatusDownSprite)
                    .id();

                commands.entity(entity).push_children(&[down_ent]);
            }
            (false, Some(down_ent)) => {
                commands.entity(down_ent).despawn_recursive();
            }
            _ => {}
        }
        match (up, up_sprite) {
            (true, None) => {
                let up_ent = commands
                    .spawn_bundle(SpriteBundle {
                        texture: texture_handles.status_up.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            healthbar.size.x / 2.0 + 6.0,
                            healthbar.offset.y,
                            layer::HEALTHBAR_BG,
                        )),
                        ..Default::default()
                    })
                    .insert(StatusUpSprite)
                    .id();
                commands.entity(entity).push_children(&[up_ent]);
            }
            (false, Some(up_ent)) => {
                commands.entity(up_ent).despawn_recursive();
            }
            _ => {}
        }
    }
}

fn animate(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &EnemyKind,
        &Direction,
        &AnimationState,
        &mut AnimationTick,
    )>,
    anim_handles: Res<EnemyAnimationHandles>,
    anim_data_assets: Res<Assets<AnimationData>>,
) {
    for (mut timer, mut sprite, kind, direction, anim_state, mut tick) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let anim_data = anim_data_assets.get(&anim_handles.by_key(&kind.0)).unwrap();

            // TODO there's really more to these animations than just cycling
            // through the frames at some fraction of the frame rate.

            let (start, length, modulus) = match (&anim_state, &direction) {
                (AnimationState::Walking, Direction::Up) => {
                    let anim = &anim_data.animations["walk_up"];
                    (anim.row * anim_data.cols, anim.length, 1)
                }
                (AnimationState::Walking, Direction::Down) => {
                    let anim = &anim_data.animations["walk_down"];
                    (anim.row * anim_data.cols, anim.length, 1)
                }
                (AnimationState::Walking, Direction::Right) => {
                    let anim = &anim_data.animations["walk_right"];
                    (anim.row * anim_data.cols, anim.length, 1)
                }
                (AnimationState::Walking, Direction::Left) => {
                    let anim = &anim_data.animations["walk_left"];
                    (anim.row * anim_data.cols, anim.length, 1)
                }
                (AnimationState::Idle, Direction::Up) => {
                    let anim = &anim_data.animations["idle_up"];
                    (anim.row * anim_data.cols, anim.length, 20)
                }
                (AnimationState::Idle, Direction::Down) => {
                    let anim = &anim_data.animations["idle_down"];
                    (anim.row * anim_data.cols, anim.length, 20)
                }
                (AnimationState::Idle, Direction::Right) => {
                    let anim = &anim_data.animations["idle_right"];
                    (anim.row * anim_data.cols, anim.length, 20)
                }
                (AnimationState::Idle, Direction::Left) => {
                    let anim = &anim_data.animations["idle_left"];
                    (anim.row * anim_data.cols, anim.length, 20)
                }
                (AnimationState::Attacking, Direction::Up) => {
                    let anim = &anim_data.animations["atk_up"];
                    (anim.row * anim_data.cols, anim.length, 2)
                }
                (AnimationState::Attacking, Direction::Down) => {
                    let anim = &anim_data.animations["atk_down"];
                    (anim.row * anim_data.cols, anim.length, 2)
                }
                (AnimationState::Attacking, Direction::Right) => {
                    let anim = &anim_data.animations["atk_right"];
                    (anim.row * anim_data.cols, anim.length, 2)
                }
                (AnimationState::Attacking, Direction::Left) => {
                    let anim = &anim_data.animations["atk_left"];
                    (anim.row * anim_data.cols, anim.length, 2)
                }
                // I think browserquest just poofs the enemies with a generic death animation,
                // but I think it would be nice to litter the path with the fallen. We can
                // just use one of the idle frames for now.
                (AnimationState::Corpse, _) => {
                    let anim = &anim_data.animations["idle_up"];
                    (anim.row * anim_data.cols, 1, 2)
                }
            };

            tick.0 += 1;
            if tick.0 % modulus == 0 {
                sprite.index += 1;
            }
            if sprite.index < start || sprite.index > (start + length - 1) {
                sprite.index = start
            }
        }
    }
}

fn movement(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationState,
        &mut Direction,
        &mut EnemyPath,
        &mut Transform,
        &Speed,
    )>,
) {
    for (mut anim_state, mut direction, mut path, mut transform, speed) in query.iter_mut() {
        if path.path_index >= path.path.len() - 1 {
            continue;
        }

        if let AnimationState::Idle = *anim_state {
            *anim_state = AnimationState::Walking;
        }

        if let AnimationState::Corpse = *anim_state {
            continue;
        }

        let next_waypoint = path.path[path.path_index + 1];

        let dist = transform.translation.truncate().distance(next_waypoint);

        let step = speed.0 * time.delta_seconds();

        if step < dist {
            transform.translation.x += step / dist * (next_waypoint.x - transform.translation.x);
            transform.translation.y += step / dist * (next_waypoint.y - transform.translation.y);
        } else {
            transform.translation.x = next_waypoint.x;
            transform.translation.y = next_waypoint.y;
            path.path_index += 1;

            // check the next waypoint so we know which way we should be facing

            if let Some(next) = path.path.get(path.path_index + 1) {
                let dx = next.x - transform.translation.x;
                let dy = next.y - transform.translation.y;

                // this probably works fine while we're moving
                // orthogonally
                if dx > 0.1 {
                    *direction = Direction::Right;
                } else if dx < -0.1 {
                    *direction = Direction::Left;
                } else if dy > 0.1 {
                    *direction = Direction::Up;
                } else if dy < -0.1 {
                    *direction = Direction::Down;
                }
            } else {
                *anim_state = AnimationState::Attacking;
            }
        }
    }
}
