use bevy::{platform::collections::HashMap, prelude::*};

use anyhow::anyhow;
use tiled::Object;

use crate::{
    atlas_loader::AtlasImage,
    enemy::{EnemyBundle, EnemyKind, EnemyPath},
    healthbar::HealthBar,
    layer,
    loading::EnemyAtlasHandles,
    map::{get_float_property, get_int_property, get_string_property},
    Armor, CleanupBeforeNewGame, HitPoints, Speed, TaipoState,
};

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Waves>().init_resource::<WaveState>();

        app.add_systems(Update, spawn_enemies.run_if(in_state(TaipoState::Playing)));

        app.add_systems(OnExit(TaipoState::GameOver), reset);
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

impl Wave {
    pub fn new(object: &Object, paths: &HashMap<i32, Vec<Vec2>>) -> anyhow::Result<Wave> {
        let enemy = get_string_property(object, "enemy")?;
        let num = get_int_property(object, "num")? as usize;
        let delay = get_float_property(object, "delay")?;
        let interval = get_float_property(object, "interval")?;
        let hp = get_int_property(object, "hp")? as u32;
        let armor = get_int_property(object, "armor")? as u32;
        let speed = get_float_property(object, "speed")?;
        let path_index = get_int_property(object, "path_index")?;

        let path = paths
            .get(&path_index)
            .ok_or_else(|| anyhow!("no path for path_index"))?
            .clone();

        Ok(Wave {
            path,
            enemy,
            num,
            hp,
            armor,
            speed,
            interval,
            delay,
        })
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
    atlas_images: Res<Assets<AtlasImage>>,
) {
    let Some(current_wave) = waves.current() else {
        return;
    };

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

    let atlas_image = atlas_images
        .get(&enemy_atlas_handles.by_key(&current_wave.enemy))
        .unwrap();

    commands.spawn((
        Sprite {
            image: atlas_image.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: atlas_image.layout.clone(),
                index: 0,
            }),
            ..default()
        },
        Transform::from_translation(Vec3::new(point.x, point.y, layer::ENEMY)),
        EnemyBundle {
            kind: EnemyKind(current_wave.enemy.to_string()),
            path: EnemyPath { path, ..default() },
            hit_points: HitPoints::full(current_wave.hp),
            armor: Armor(current_wave.armor),
            speed: Speed(current_wave.speed),
            health_bar: HealthBar {
                offset: Vec2::new(0.0, 14.0),
                ..default()
            },
            ..default()
        },
        CleanupBeforeNewGame,
    ));

    wave_state.remaining -= 1;

    if wave_state.remaining == 0 {
        if let Some(next) = waves.advance() {
            commands.insert_resource(WaveState::from(next));
        }
    }
}

fn reset(mut commands: Commands, mut waves: ResMut<Waves>) {
    commands.insert_resource(WaveState::default());
    waves.current = 0;
}
