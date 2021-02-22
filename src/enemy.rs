use crate::{ActionPanel, AnimationData, AnimationHandles, Currency, Goal, HitPoints};
use bevy::prelude::*;
use rand::{thread_rng, Rng};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(animate.system())
            .add_system(
                death
                    .system()
                    .label("enemy_death")
                    .before("update_currency_text"),
            )
            .add_system(movement.system())
            .add_system(deal_damage.system());
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Default, Debug)]
pub struct EnemyState {
    pub name: String,
    pub facing: Direction,
    pub state: AnimationState,
    pub tick: u32,
    pub path: Vec<Vec2>,
    pub path_index: usize,
}

pub struct EnemyAttackTimer(pub Timer);

fn death(
    mut query: Query<(&mut EnemyState, &mut Transform, &HitPoints), Changed<HitPoints>>,
    mut currency: ResMut<Currency>,
    mut action_panel: ResMut<ActionPanel>,
) {
    for (mut state, mut transform, hp) in query.iter_mut() {
        if hp.current == 0 {
            match state.state {
                AnimationState::Corpse => {}
                _ => {
                    state.state = AnimationState::Corpse;

                    let mut rng = thread_rng();
                    transform.rotate(Quat::from_rotation_z(rng.gen_range(-0.2..0.2)));

                    currency.current = currency.current.saturating_add(1);
                    currency.total_earned = currency.total_earned.saturating_add(1);

                    action_panel.update += 1;
                }
            }
        }
    }
}

fn deal_damage(
    time: Res<Time>,
    mut query: Query<(&mut EnemyAttackTimer, &EnemyState)>,
    mut goal_query: Query<&mut HitPoints, With<Goal>>,
) {
    // TODO this should really sync up with the animations somehow

    for (mut timer, state) in query.iter_mut() {
        if let AnimationState::Attacking = state.state {
            timer.0.tick(time.delta_seconds());
            if timer.0.finished() {
                for mut hp in goal_query.iter_mut() {
                    hp.current = hp.current.saturating_sub(1);
                }
            }
        }
    }
}

fn animate(
    time: Res<Time>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &mut EnemyState)>,
    anim_handles: Res<AnimationHandles>,
    anim_data_assets: Res<Assets<AnimationData>>,
) {
    for (mut timer, mut sprite, mut state) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            let anim_data = anim_data_assets
                .get(anim_handles.handles.get(&state.name).unwrap())
                .unwrap();

            // TODO there's really more to these animations than just cycling
            // through the frames at some fraction of the frame rate.

            let (start, length, modulus) = match (&state.state, &state.facing) {
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

            state.tick += 1;
            if state.tick % modulus == 0 {
                sprite.index += 1;
            }
            if sprite.index < start as u32 || sprite.index > (start + length - 1) as u32 {
                sprite.index = start as u32
            }
        }
    }
}

fn movement(time: Res<Time>, mut query: Query<(&mut EnemyState, &mut Transform)>) {
    for (mut state, mut transform) in query.iter_mut() {
        if state.path_index >= state.path.len() - 1 {
            continue;
        }

        if let AnimationState::Idle = state.state {
            state.state = AnimationState::Walking;
        }

        if let AnimationState::Corpse = state.state {
            continue;
        }

        let next_waypoint = state.path[state.path_index + 1];

        let dist = transform.translation.truncate().distance(next_waypoint);

        let speed = 20.0; // XXX
        let step = speed * time.delta_seconds();

        if step < dist {
            transform.translation.x += step / dist * (next_waypoint.x - transform.translation.x);
            transform.translation.y += step / dist * (next_waypoint.y - transform.translation.y);
        } else {
            transform.translation.x = next_waypoint.x;
            transform.translation.y = next_waypoint.y;
            state.path_index += 1;

            // check the next waypoint so we know which way we should be facing

            if let Some(next) = state.path.get(state.path_index + 1) {
                let dx = next.x - transform.translation.x;
                let dy = next.y - transform.translation.y;

                // this probably works fine while we're moving
                // orthogonally
                if dx > 0.1 {
                    state.facing = Direction::Right;
                } else if dx < -0.1 {
                    state.facing = Direction::Left;
                } else if dy > 0.1 {
                    state.facing = Direction::Up;
                } else if dy < -0.1 {
                    state.facing = Direction::Down;
                }
            } else {
                state.state = AnimationState::Attacking;
            }
        }
    }
}
