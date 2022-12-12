use bevy::prelude::*;

use crate::{
    enemy::{EnemyBundle, EnemyKind, EnemyPath},
    healthbar, layer,
    loading::EnemyAtlasHandles,
    Armor, GameState, HitPoints, Speed, TaipoState,
};

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(TaipoState::Playing).with_system(spawn_enemies));
    }
}

#[derive(Resource, Default)]
pub struct Waves {
    pub waves: Vec<Wave>,
    pub current: usize,
}
impl Waves {
    pub fn current(&self) -> Option<&Wave> {
        self.waves.get(self.current)
    }
    pub fn advance(&mut self) -> Option<&Wave> {
        self.current += 1;
        self.current()
    }
}

#[derive(Clone, Debug)]
pub struct Wave {
    pub path: Vec<Vec2>,
    pub enemy: String,
    pub num: usize,
    pub hp: u32,
    pub armor: u32,
    pub speed: f32,
    pub interval: f32,
    pub delay: f32,
}
impl Default for Wave {
    fn default() -> Self {
        Wave {
            path: vec![],
            enemy: "skeleton".to_string(),
            hp: 5,
            num: 10,
            armor: 0,
            speed: 20.0,
            interval: 3.0,
            delay: 30.0,
        }
    }
}

#[derive(Resource)]
pub struct WaveState {
    pub delay_timer: Timer,
    pub spawn_timer: Timer,
    pub remaining: usize,
}
impl Default for WaveState {
    fn default() -> Self {
        Self {
            delay_timer: Timer::from_seconds(1., TimerMode::Once),
            spawn_timer: Timer::from_seconds(1., TimerMode::Repeating),
            remaining: 0,
        }
    }
}

impl From<&Wave> for WaveState {
    fn from(value: &Wave) -> Self {
        Self {
            delay_timer: Timer::from_seconds(value.delay, TimerMode::Once),
            spawn_timer: Timer::from_seconds(value.interval, TimerMode::Repeating),
            remaining: value.num,
        }
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    mut waves: ResMut<Waves>,
    mut wave_state: ResMut<WaveState>,
    time: Res<Time>,
    enemy_atlas_handles: Res<EnemyAtlasHandles>,
    game_state: Res<GameState>,
) {
    if !game_state.ready || game_state.over {
        return;
    }

    let Some(current_wave) = waves.current() else {
        return;
    };

    // If we haven't started the delay timer for a new wave yet,
    // go ahead and do that.

    wave_state.delay_timer.tick(time.delta());
    if !wave_state.delay_timer.finished() {
        return;
    }

    wave_state.spawn_timer.tick(time.delta());
    if !wave_state.spawn_timer.just_finished() {
        return;
    }

    let path = current_wave.path.clone();
    let point = path[0];

    let entity = commands
        .spawn((
            SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(point.x, point.y, layer::ENEMY)),
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..Default::default()
                },
                texture_atlas: enemy_atlas_handles.by_key(&current_wave.enemy),
                ..Default::default()
            },
            EnemyBundle {
                kind: EnemyKind(current_wave.enemy.to_string()),
                path: EnemyPath {
                    path,
                    ..Default::default()
                },
                hit_points: HitPoints {
                    current: current_wave.hp,
                    max: current_wave.hp,
                },
                armor: Armor(current_wave.armor),
                speed: Speed(current_wave.speed),
                ..Default::default()
            },
        ))
        .id();

    healthbar::spawn(
        entity,
        healthbar::HealthBar {
            size: Vec2::new(16.0, 2.0),
            offset: Vec2::new(0.0, 14.0),
            show_full: false,
            show_empty: false,
        },
        &mut commands,
    );

    wave_state.remaining -= 1;

    if wave_state.remaining == 0 {
        if let Some(next) = waves.advance() {
            commands.insert_resource(WaveState::from(next));
        }
    }
}
