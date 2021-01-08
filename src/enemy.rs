use crate::{Goal, HitPoints};
use bevy::prelude::*;

pub struct EnemyPlugin;

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
    pub facing: Direction,
    pub state: AnimationState,
    pub tick: u32,
    pub path: Vec<Vec2>,
    pub path_index: usize,
}

pub struct EnemyAttackTimer(pub Timer);

pub struct Skeleton;

fn deal_damage(
    time: Res<Time>,
    mut query: Query<(&mut EnemyAttackTimer, &EnemyState), With<Skeleton>>,
    mut goal_query: Query<&mut HitPoints, With<Goal>>,
) {
    // TODO this should really sync up with the animations somehow

    for (mut timer, state) in query.iter_mut() {
        if let AnimationState::Attacking = state.state {
            timer.0.tick(time.delta_seconds());
            if timer.0.finished() {
                for mut hp in goal_query.iter_mut() {
                    hp.current = hp.current.saturating_sub(1);
                    info!("attacking goal {}", hp.current);
                }
            }
        }
    }
}

fn animate_skeleton(
    time: Res<Time>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &mut EnemyState), With<Skeleton>>,
) {
    for (mut timer, mut sprite, mut state) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            // TODO there's really more to these animations than just cycling
            // through the frames at some paticular rate.
            let (start, end, modulus) = match (&state.state, &state.facing) {
                (AnimationState::Walking, Direction::Up) => (17, 20, 1),
                (AnimationState::Walking, Direction::Down) => (29, 32, 1),
                // oh god how do I flip things? seems like I have to
                // rotate 180 over y?
                (AnimationState::Walking, Direction::Left) => (4, 7, 1),
                (AnimationState::Walking, Direction::Right) => (4, 7, 1),
                (AnimationState::Idle, Direction::Up) => (20, 22, 20),
                (AnimationState::Idle, Direction::Down) => (30, 32, 20),
                // TODO flip
                (AnimationState::Idle, Direction::Left) => (8, 9, 20),
                (AnimationState::Idle, Direction::Right) => (8, 9, 20),
                (AnimationState::Attacking, Direction::Up) => (12, 14, 2),
                (AnimationState::Attacking, Direction::Down) => (24, 26, 21),
                // TODO flip
                (AnimationState::Attacking, Direction::Left) => (0, 2, 2),
                (AnimationState::Attacking, Direction::Right) => (0, 2, 2),
                // TODO there is no corpse? wasn't there one in the tilemap?
                // We can pretend with one of the idle-up frames
                (AnimationState::Corpse, _) => (21, 21, 1),
            };

            state.tick += 1;
            if state.tick % modulus == 0 {
                sprite.index += 1;
            }
            if sprite.index < start || sprite.index > end {
                sprite.index = start
            }
        }
    }
}

fn move_enemies(time: Res<Time>, mut query: Query<(&mut EnemyState, &mut Transform)>) {
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

        let next = Vec2::extend(
            state.path.get(state.path_index + 1).unwrap().clone(),
            transform.translation.z,
        );
        let d = transform.translation.distance(next);

        let speed = 20.0;
        let step = speed * time.delta_seconds();

        if step > d {
            transform.translation.x = next.x;
            transform.translation.y = next.y;
            state.path_index += 1;

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

            continue;
        }

        transform.translation.x += step / d * (next.x - transform.translation.x);
        transform.translation.y += step / d * (next.y - transform.translation.y);
    }
}

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(animate_skeleton.system())
            .add_system(move_enemies.system())
            .add_system(deal_damage.system());
    }
}
