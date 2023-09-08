use bevy::{prelude::*, utils::HashMap};

use anyhow::anyhow;
use tiled::{Object, PropertyValue};

use crate::{
    enemy::{EnemyBundle, EnemyKind, EnemyPath},
    healthbar::HealthBar,
    layer,
    loading::EnemyAtlasHandles,
    Armor, HitPoints, Speed, TaipoState,
};

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Waves>().init_resource::<WaveState>();

        app.add_systems(Update, spawn_enemies.run_if(in_state(TaipoState::Playing)));
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
        let enemy = object
            .properties
            .get(&"enemy".to_string())
            .ok_or_else(|| anyhow!("required enemy property not found"))
            .and_then(|v| {
                if let PropertyValue::StringValue(v) = v {
                    Ok(v.to_string())
                } else {
                    Err(anyhow!("enemy property should be a string"))
                }
            })?;

        let num = object
            .properties
            .get(&"num".to_string())
            .ok_or_else(|| anyhow!("required num property not found"))
            .and_then(|v| {
                if let PropertyValue::IntValue(v) = v {
                    Ok(*v as usize)
                } else {
                    Err(anyhow!("num property should be an int"))
                }
            })?;

        let delay = object
            .properties
            .get(&"delay".to_string())
            .ok_or_else(|| anyhow!("required delay property not found"))
            .and_then(|v| {
                if let PropertyValue::FloatValue(v) = v {
                    Ok(*v)
                } else {
                    Err(anyhow!("delay property should be a float"))
                }
            })?;

        let interval = object
            .properties
            .get(&"interval".to_string())
            .ok_or_else(|| anyhow!("required interval property not found"))
            .and_then(|v| {
                if let PropertyValue::FloatValue(v) = v {
                    Ok(*v)
                } else {
                    Err(anyhow!("interval property should be a float"))
                }
            })?;

        let hp = object
            .properties
            .get(&"hp".to_string())
            .ok_or_else(|| anyhow!("required hp property not found"))
            .and_then(|v| {
                if let PropertyValue::IntValue(v) = v {
                    Ok(*v as u32)
                } else {
                    Err(anyhow!("hp property should be an int"))
                }
            })?;

        let armor = object
            .properties
            .get(&"armor".to_string())
            .ok_or_else(|| anyhow!("required armor property not found"))
            .and_then(|v| {
                if let PropertyValue::IntValue(v) = v {
                    Ok(*v as u32)
                } else {
                    Err(anyhow!("armor property should be an int"))
                }
            })?;

        let speed = object
            .properties
            .get(&"speed".to_string())
            .ok_or_else(|| anyhow!("required speed property not found"))
            .and_then(|v| {
                if let PropertyValue::FloatValue(v) = v {
                    Ok(*v)
                } else {
                    Err(anyhow!("speed property should be a float"))
                }
            })?;

        let path_index = object
            .properties
            .get(&"path_index".to_string())
            .ok_or_else(|| anyhow!("required path_index property not found"))
            .and_then(|v| {
                if let PropertyValue::IntValue(v) = v {
                    Ok(*v)
                } else {
                    Err(anyhow!("path_index property should be an int"))
                }
            })?;

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

    commands.spawn((
        SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(point.x, point.y, layer::ENEMY)),
            texture_atlas: enemy_atlas_handles.by_key(&current_wave.enemy),
            ..default()
        },
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
    ));

    wave_state.remaining -= 1;

    if wave_state.remaining == 0 {
        if let Some(next) = waves.advance() {
            commands.insert_resource(WaveState::from(next));
        }
    }
}
