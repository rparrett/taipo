use bevy::{ecs::query::Or, prelude::*};

use rand::{thread_rng, Rng};

use crate::{
    action_panel::ActionPanel,
    healthbar::HealthBar,
    layer,
    loading::{EnemyAnimationHandles, TextureHandles},
    update_currency_text, AfterUpdate, AnimationData, Armor, Currency, Goal, HitPoints, Speed,
    StatusDownSprite, StatusEffects, StatusUpSprite, TaipoState,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate,
                movement,
                deal_damage,
                death.before(update_currency_text),
            )
                .run_if(in_state(TaipoState::Playing)),
        );

        app.add_systems(
            AfterUpdate,
            status_effect_appearance.run_if(in_state(TaipoState::Playing)),
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
    pub health_bar: HealthBar,
}

#[derive(Component, Debug, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Walking,
    Attacking,
    Corpse,
}

#[derive(Component, Debug, Default, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    #[default]
    Right,
}
impl From<Vec2> for Direction {
    fn from(value: Vec2) -> Self {
        const DIRECTIONS: [(Direction, Vec2); 4] = [
            (Direction::Left, Vec2::NEG_X),
            (Direction::Right, Vec2::X),
            (Direction::Up, Vec2::Y),
            (Direction::Down, Vec2::NEG_Y),
        ];

        let max = DIRECTIONS
            .iter()
            .max_by(|a, b| a.1.dot(value).partial_cmp(&b.1.dot(value)).unwrap())
            .unwrap();

        max.0
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
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}
#[derive(Component)]
pub struct AttackTimer(pub Timer);
impl Default for AttackTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

pub fn death(
    mut query: Query<(&mut AnimationState, &mut Transform, &HitPoints), Changed<HitPoints>>,
    mut currency: ResMut<Currency>,
    mut action_panel: ResMut<ActionPanel>,
) {
    for (mut state, mut transform, hp) in query.iter_mut() {
        if hp.current == 0 && !matches!(*state, AnimationState::Corpse) {
            *state = AnimationState::Corpse;

            let mut rng = thread_rng();
            transform.rotate(Quat::from_rotation_z(rng.gen_range(-0.2..0.2)));
            transform.translation.z = layer::CORPSE;

            currency.current = currency.current.saturating_add(2);
            currency.total_earned = currency.total_earned.saturating_add(2);

            // Force an action panel update
            action_panel.set_changed();
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
                if let Ok(ent) = down_query.get(child) {
                    down_sprite = Some(ent);
                }
                if let Ok(ent) = up_query.get(child) {
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
                    .spawn((
                        Sprite {
                            image: texture_handles.status_down.clone(),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(
                            healthbar.size.x / 2.0 + 6.0,
                            healthbar.offset.y,
                            layer::HEALTHBAR_BG,
                        )),
                        StatusDownSprite,
                    ))
                    .id();

                commands.entity(entity).add_child(down_ent);
            }
            (false, Some(down_ent)) => {
                commands.entity(down_ent).despawn();
            }
            _ => {}
        }

        match (up, up_sprite) {
            (true, None) => {
                let up_ent = commands
                    .spawn((
                        Sprite {
                            image: texture_handles.status_up.clone(),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(
                            healthbar.size.x / 2.0 + 6.0,
                            healthbar.offset.y,
                            layer::HEALTHBAR_BG,
                        )),
                        StatusUpSprite,
                    ))
                    .id();
                commands.entity(entity).add_child(up_ent);
            }
            (false, Some(up_ent)) => {
                commands.entity(up_ent).despawn();
            }
            _ => {}
        }
    }
}

fn animate(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut Sprite,
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
        if !timer.0.just_finished() {
            continue;
        }

        let anim_data = anim_data_assets.get(&anim_handles.by_key(&kind.0)).unwrap();

        // TODO there's really more to these animations than just cycling
        // through the frames at some fraction of the frame rate.

        let (start, length, modulus, flip_x) = match (&anim_state, &direction) {
            (AnimationState::Walking, Direction::Up) => {
                let anim = &anim_data.animations["walk_up"];
                (anim.row * anim_data.cols, anim.length, 1, false)
            }
            (AnimationState::Walking, Direction::Down) => {
                let anim = &anim_data.animations["walk_down"];
                (anim.row * anim_data.cols, anim.length, 1, false)
            }
            (AnimationState::Walking, Direction::Right) => {
                let anim = &anim_data.animations["walk_right"];
                (anim.row * anim_data.cols, anim.length, 1, false)
            }
            (AnimationState::Walking, Direction::Left) => {
                let anim = &anim_data.animations["walk_right"];
                (anim.row * anim_data.cols, anim.length, 1, true)
            }
            (AnimationState::Idle, Direction::Up) => {
                let anim = &anim_data.animations["idle_up"];
                (anim.row * anim_data.cols, anim.length, 20, false)
            }
            (AnimationState::Idle, Direction::Down) => {
                let anim = &anim_data.animations["idle_down"];
                (anim.row * anim_data.cols, anim.length, 20, false)
            }
            (AnimationState::Idle, Direction::Right) => {
                let anim = &anim_data.animations["idle_right"];
                (anim.row * anim_data.cols, anim.length, 20, false)
            }
            (AnimationState::Idle, Direction::Left) => {
                let anim = &anim_data.animations["idle_right"];
                (anim.row * anim_data.cols, anim.length, 20, true)
            }
            (AnimationState::Attacking, Direction::Up) => {
                let anim = &anim_data.animations["atk_up"];
                (anim.row * anim_data.cols, anim.length, 2, false)
            }
            (AnimationState::Attacking, Direction::Down) => {
                let anim = &anim_data.animations["atk_down"];
                (anim.row * anim_data.cols, anim.length, 2, false)
            }
            (AnimationState::Attacking, Direction::Right) => {
                let anim = &anim_data.animations["atk_right"];
                (anim.row * anim_data.cols, anim.length, 2, false)
            }
            (AnimationState::Attacking, Direction::Left) => {
                let anim = &anim_data.animations["atk_right"];
                (anim.row * anim_data.cols, anim.length, 2, true)
            }
            // I think browserquest just poofs the enemies with a generic death animation,
            // but I think it would be nice to litter the path with the fallen. We can
            // just use one of the idle frames for now.
            (AnimationState::Corpse, _) => {
                let anim = &anim_data.animations["idle_up"];
                (anim.row * anim_data.cols, 1, 2, false)
            }
        };

        sprite.flip_x = flip_x;

        let Some(ref mut atlas) = sprite.texture_atlas else {
            continue;
        };

        tick.0 += 1;
        if tick.0 % modulus == 0 {
            atlas.index += 1;
        }

        let end = start + length - 1;

        if !(start..=end).contains(&atlas.index) {
            atlas.index = start;
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
        if let AnimationState::Corpse = *anim_state {
            continue;
        }

        if path.path_index >= path.path.len() - 1 {
            *anim_state = AnimationState::Attacking;
            continue;
        }

        if let AnimationState::Idle = *anim_state {
            *anim_state = AnimationState::Walking;
        }

        let next_waypoint = path.path[path.path_index + 1];

        let diff = next_waypoint - transform.translation.truncate();
        let dist = diff.length();

        let step = speed.0 * time.delta_secs();

        if step < dist {
            transform.translation += (diff.normalize_or_zero() * step).extend(0.);
        } else {
            transform.translation.x = next_waypoint.x;
            transform.translation.y = next_waypoint.y;
            path.path_index += 1;
        }

        *direction = diff.into();
    }
}
