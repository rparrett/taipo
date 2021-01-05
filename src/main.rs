use bevy::{
    asset::LoadState,
    log::{Level, LogSettings},
    prelude::*,
};
use bevy_tiled_prototype::{Map, TiledMapCenter};
use bullet::BulletPlugin;
use data::{GameData, GameDataPlugin};
use enemy::{AnimationState, EnemyPlugin, EnemyState, Skeleton};
use healthbar::HealthBarPlugin;
use rand::{prelude::SliceRandom, thread_rng};
use typing::{
    TypingPlugin, TypingState, TypingStateChangedEvent, TypingTarget, TypingTargetChangeEvent,
    TypingTargetContainer, TypingTargetFinishedEvent, TypingTargetImage, TypingTargetSpawnEvent,
};

use std::collections::VecDeque;

#[macro_use]
extern crate anyhow;

mod bullet;
mod data;
mod enemy;
mod healthbar;
mod typing;

static TOWER_PRICE: u32 = 10;
pub static FONT_SIZE: f32 = 32.0;

const STAGE: &str = "app_state";

#[derive(Clone)]
enum AppState {
    Preload,
    Load,
    Spawn,
    Ready,
}

#[derive(Default)]
pub struct GameState {
    primary_currency: u32,
    score: u32,
    selected: Option<Entity>,
    possible_typing_targets: VecDeque<TypingTarget>,
    // Just so we can keep these in the correct order
    tower_slots: Vec<Entity>,
    over: bool,
    ready: bool,
}

struct CurrencyDisplay;
struct CooldownTimerDisplay;
struct CooldownTimerTimer(Timer);

struct TowerSlot {
    texture_ui: Handle<Texture>,
}

#[derive(Debug)]
enum TowerType {
    Basic,
}

#[derive(Default)]
struct TowerState {
    level: u32,
    range: f32,
    timer: Timer,
}

struct Reticle;

struct UpdateActionsEvent;

// Map and GameData don't really belong. Consolidate into AssetHandles?
#[derive(Default)]
pub struct TextureHandles {
    pub tower_slot_ui: Vec<Handle<Texture>>,
    pub coin_ui: Handle<Texture>,
    pub back_ui: Handle<Texture>,
    pub tower: Handle<Texture>,
    pub tower_ui: Handle<Texture>,
    pub timer_ui: Handle<Texture>,
    pub bullet_shuriken: Handle<Texture>,
    pub main_atlas: Handle<TextureAtlas>,
    pub main_atlas_texture: Handle<Texture>,
    pub skel_atlas: Handle<TextureAtlas>,
    pub skel_atlas_texture: Handle<Texture>,
    pub tiled_map: Handle<Map>,
    pub game_data: Handle<GameData>,
}

#[derive(Default)]
struct FontHandles {
    jptext: Handle<Font>,
    minimal: Handle<Font>,
}

#[derive(Clone)]
enum Action {
    SelectTower(Entity),
    GenerateMoney,
    Back,
    BuildBasicTower,
    SwitchLanguageMode,
}
struct HitPoints {
    current: u32,
    max: u32,
}
impl Default for HitPoints {
    fn default() -> Self {
        HitPoints { current: 1, max: 1 }
    }
}

#[derive(Clone, Debug)]
struct Wave {
    path: Vec<Vec2>,
    enemy: String,
    num: usize,
    hp: u32,
    interval: f32,
    delay: f32,
}
impl Default for Wave {
    fn default() -> Self {
        Wave {
            path: vec![],
            enemy: "Skeleton".to_string(),
            hp: 5,
            num: 10,
            interval: 3.0,
            delay: 30.0,
        }
    }
}

#[derive(Debug)]
struct Waves {
    current: usize,
    spawn_timer: Timer,
    cooldown_timer: Timer,
    started: bool,
    spawned: usize,
    waves: Vec<Wave>,
}
impl Default for Waves {
    fn default() -> Self {
        Waves {
            current: 0,
            spawn_timer: Timer::from_seconds(1.0, true), // arbitrary, overwritten by wave
            cooldown_timer: Timer::from_seconds(30.0, false), // arbitrary, overwritten by wave
            started: false,
            spawned: 0,
            waves: vec![],
        }
    }
}

struct LoadingScreen;
struct LoadingScreenText;

fn update_actions(
    commands: &mut Commands,
    game_state: Res<GameState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(Entity, &Children), With<TypingTarget>>,
    container_query: Query<&Children, With<TypingTargetContainer>>,
    tower_slot_query: Query<&TowerSlot>,
    tower_type_query: Query<&TowerType>,
    image_query: Query<Entity, With<TypingTargetImage>>,
    mut style_query: Query<&mut Style>,
    mut visible_query: Query<&mut Visible>,
    events: Res<Events<UpdateActionsEvent>>,
    mut reader: Local<EventReader<UpdateActionsEvent>>,
    texture_handles: Res<TextureHandles>,
) {
    // multiple update action events may come in, but it's enough to just do
    // this update once.

    if reader.iter(&events).next().is_none() {
        return;
    }

    info!("processing UpdateActionsEvent");

    let mut other = vec![];

    if let Some(selected) = game_state.selected {
        if tower_type_query.get(selected).is_err() {
            other.push((texture_handles.tower_ui.clone(), Action::BuildBasicTower));
        }

        other.push((texture_handles.back_ui.clone(), Action::Back));
    } else {
        other.push((texture_handles.coin_ui.clone(), Action::GenerateMoney));
    }

    let other_iter = other.iter().cloned();

    let mut action_iter = game_state
        .tower_slots
        .iter()
        .cloned()
        .filter(|_| game_state.selected.is_none())
        .map(|ent| {
            (
                tower_slot_query.get(ent).unwrap().texture_ui.clone(),
                Action::SelectTower(ent.clone()),
            )
        })
        .chain(other_iter);

    for container_children in container_query.iter() {
        for container in container_children.iter() {
            for (entity, target_children) in query.get_mut(*container) {
                commands.remove_one::<Action>(entity);

                for mut style in style_query.get_mut(entity) {
                    style.display = Display::None;
                }

                // find any TypingTargetImages inside this particular
                // target and destroy them.

                for target_child in target_children.iter() {
                    for image in image_query.get(*target_child) {
                        commands.despawn_recursive(image);
                    }

                    // Workaround for #838/#1135
                    for mut child_visible in visible_query.get_mut(*target_child) {
                        child_visible.is_visible = false;
                    }
                }

                if let Some((texture, action)) = action_iter.next() {
                    for mut style in style_query.get_mut(entity) {
                        style.display = Display::Flex;
                    }

                    // Workaround for #838/#1135
                    for target_child in target_children.iter() {
                        for mut child_visible in visible_query.get_mut(*target_child) {
                            child_visible.is_visible = true;
                        }
                    }

                    commands.insert_one(entity, action.clone());

                    // add an image back

                    let child = commands
                        .spawn(ImageBundle {
                            style: Style {
                                margin: Rect {
                                    left: Val::Px(5.0),
                                    right: Val::Px(5.0),
                                    ..Default::default()
                                },
                                size: Size::new(Val::Auto, Val::Px(32.0)),
                                ..Default::default()
                            },
                            // can I somehow get this from the sprite sheet? naively tossing a
                            // spritesheetbundle here instead of an imagebundle seems to panic.
                            material: materials.add(texture.into()),
                            ..Default::default()
                        })
                        .with(TypingTargetImage)
                        .current_entity()
                        .unwrap();

                    commands.insert_children(entity, 0, &[child]);
                }
            }
        }
    }
}

fn typing_target_finished(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    mut reader: Local<EventReader<TypingTargetFinishedEvent>>,
    typing_target_finished_events: Res<Events<TypingTargetFinishedEvent>>,
    mut typing_target_change_events: ResMut<Events<TypingTargetChangeEvent>>,
    mut update_actions_events: ResMut<Events<UpdateActionsEvent>>,
    action_query: Query<&Action>,
    mut reticle_query: Query<&mut Transform, With<Reticle>>,
    tower_transform_query: Query<&Transform, With<TowerSlot>>,
    texture_handles: Res<TextureHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut typing_state_changed_events: ResMut<Events<TypingStateChangedEvent>>,
    mut typing_state: ResMut<TypingState>,
) {
    for event in reader.iter(&typing_target_finished_events) {
        game_state
            .possible_typing_targets
            .push_back(event.target.clone());
        let target = game_state
            .possible_typing_targets
            .pop_front()
            .unwrap()
            .clone();
        typing_target_change_events.send(TypingTargetChangeEvent {
            entity: event.entity,
            target: target.clone(),
        });
        info!("new target: {}", target.ascii.join(""));

        for action in action_query.get(event.entity) {
            info!("there is some sort of action");
            if let Action::GenerateMoney = *action {
                info!("processing a GenerateMoney action");
                game_state.primary_currency = game_state.primary_currency.saturating_add(1);
                game_state.score = game_state.score.saturating_add(1);
            } else if let Action::SelectTower(tower) = *action {
                info!("processing a SelectTower action");
                game_state.selected = Some(tower);
            } else if let Action::Back = *action {
                info!("processing a Back action");
                game_state.selected = None;
            } else if let Action::SwitchLanguageMode = *action {
                info!("switching language mode!");
                typing_state.ascii_mode = !typing_state.ascii_mode;
                typing_state_changed_events.send(TypingStateChangedEvent);
            } else if let Action::BuildBasicTower = *action {
                if game_state.primary_currency < TOWER_PRICE {
                    continue;
                }
                game_state.primary_currency -= TOWER_PRICE;

                if let Some(tower) = game_state.selected {
                    for tower_transform in tower_transform_query.get(tower) {
                        info!(
                            "sending tower off to {} {} {}",
                            tower_transform.translation.x,
                            tower_transform.translation.y + 16.0,
                            20.0
                        );
                        commands.insert_one(
                            tower,
                            TowerState {
                                level: 1,
                                range: 128.0,
                                timer: Timer::from_seconds(1.0, true),
                            },
                        );
                        commands.insert_one(tower, TowerType::Basic);

                        let child = commands
                            .spawn(SpriteBundle {
                                material: materials.add(texture_handles.tower.clone().into()),
                                // Odd y value because the bottom of the sprite is not correctly
                                // positioned. Odd z value because we want to be above tiles but
                                // below the reticle.
                                transform: Transform::from_translation(Vec3::new(0.0, 20.0, 10.0)),
                                ..Default::default()
                            })
                            .current_entity()
                            .unwrap();

                        commands.insert_children(tower, 0, &[child]);
                    }
                }
            }
        }

        for mut reticle_transform in reticle_query.iter_mut() {
            if let Some(tower) = game_state.selected {
                for transform in tower_transform_query.get(tower) {
                    info!(
                        "sending reticle off to {} {} {}",
                        transform.translation.x, transform.translation.y, 20.0
                    );
                    reticle_transform.translation.x = transform.translation.x;
                    reticle_transform.translation.y = transform.translation.y;
                    reticle_transform.translation.z = 20.0;
                }
            } else {
                info!("hiding reticle");
                reticle_transform.translation.z = -1.0;
            }
        }

        update_actions_events.send(UpdateActionsEvent);
    }
}

fn animate_reticle(
    time: Res<Time>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite), With<Reticle>>,
) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            sprite.index += 1;
            if sprite.index >= 30 {
                sprite.index = 16;
            }
        }
    }
}

fn spawn_enemies(
    commands: &mut Commands,
    time: Res<Time>,
    mut waves: ResMut<Waves>,
    materials: ResMut<Assets<ColorMaterial>>,
    texture_handles: Res<TextureHandles>,
    game_state: Res<GameState>,
) {
    if !game_state.ready {
        return;
    }

    if waves.waves.len() <= waves.current {
        return;
    }

    // If we haven't started the delay timer for a new wave yet,
    // go ahead and do that.

    if !waves.started {
        // what did I ever do to you, borrow checker?
        let wave_delay = {
            let wave = waves.waves.get(waves.current).unwrap();
            wave.delay.clone()
        };

        waves.started = true;
        waves.cooldown_timer.set_duration(wave_delay);
        waves.cooldown_timer.reset();
        return;
    }

    // There's nothing to do until the delay timer is finished.

    waves.cooldown_timer.tick(time.delta_seconds());
    if !waves.cooldown_timer.finished() {
        return;
    }

    waves.spawn_timer.tick(time.delta_seconds());

    let (wave_time, wave_num, wave_hp) = {
        let wave = waves.waves.get(waves.current).unwrap();
        (wave.interval.clone(), wave.num.clone(), wave.hp.clone())
    };

    // immediately spawn the first enemy and start the timer
    let spawn = if waves.spawned == 0 {
        waves.spawn_timer.set_duration(wave_time);
        waves.spawn_timer.reset();
        true
    } else if waves.spawn_timer.just_finished() {
        true
    } else {
        false
    };

    if spawn {
        let path = waves.waves.get(waves.current).unwrap().path.clone();
        let point = path.get(0).unwrap();

        let entity = commands
            .spawn(SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(point.x, point.y, 10.0)),
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..Default::default()
                },
                texture_atlas: texture_handles.skel_atlas.clone(),
                ..Default::default()
            })
            .with(Timer::from_seconds(0.1, true))
            .with(Skeleton)
            .with(EnemyState {
                path,
                ..Default::default()
            })
            .with(HitPoints {
                current: wave_hp,
                max: wave_hp,
            })
            .current_entity()
            .unwrap();

        healthbar::spawn(entity, commands, materials, Vec2::new(16.0, 2.0));

        waves.spawned += 1
    }

    // that was the last enemy
    if waves.spawned == wave_num {
        waves.current += 1;
        waves.spawned = 0;
        waves.started = false;
    }
}

fn update_timer_display(
    time: Res<Time>,
    mut timer: ResMut<CooldownTimerTimer>,
    mut query: Query<&mut Text, With<CooldownTimerDisplay>>,
    waves: Res<Waves>,
) {
    timer.0.tick(time.delta_seconds());
    if !timer.0.finished() {
        return;
    }

    for mut text in query.iter_mut() {
        let val = f32::max(
            0.0,
            waves.cooldown_timer.duration() - waves.cooldown_timer.elapsed(),
        );

        text.value = format!("{:.1}", val);
    }
}

fn shoot_enemies(
    time: Res<Time>,
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tower_query: Query<(&Transform, &mut TowerState, &TowerType)>,
    enemy_query: Query<(Entity, &HitPoints, &Transform), With<EnemyState>>,
    texture_handles: Res<TextureHandles>,
) {
    for (transform, mut tower_state, _tower_type) in tower_query.iter_mut() {
        tower_state.timer.tick(time.delta_seconds());
        if !tower_state.timer.finished() {
            continue;
        }

        // TODO any ol' enemy is good enough for now, but we'll probably want targetting modes
        // - "enemy least far/furthest far along the path that is in range"
        // - "enemy with least/most hp that is in range"
        //
        // With the amount of enemies and tower we'll be dealing with, some fancy spatial data
        // structure probably isn't super impactful though.

        for (enemy, hp, enemy_transform) in enemy_query.iter() {
            if hp.current <= 0 {
                continue;
            }

            let d = enemy_transform
                .translation
                .truncate()
                .distance(transform.translation.truncate());

            if d > tower_state.range {
                continue;
            }

            bullet::spawn(
                transform.translation,
                enemy,
                1,
                100.0,
                commands,
                &mut materials,
                &texture_handles,
            );
            break;
        }
    }
}

fn update_currency_display(
    mut currency_display_query: Query<&mut Text, With<CurrencyDisplay>>,
    game_state: ChangedRes<GameState>,
) {
    for mut target in currency_display_query.iter_mut() {
        target.value = format!("{}", game_state.primary_currency);
    }
}

fn show_game_over(
    commands: &mut Commands,
    query: Query<&EnemyState>,
    waves: Res<Waves>,
    mut game_state: ResMut<GameState>,
    font_handles: Res<FontHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Hm. This was triggering before the game started, so we'll just check
    // to see if there's at least one wave.

    if waves.waves.len() < 1 {
        return;
    }

    if waves.current != waves.waves.len() {
        return;
    }

    if query.iter().any(|x| match x.state {
        AnimationState::Corpse => false,
        _ => true,
    }) {
        return;
    }

    if game_state.over {
        return;
    }

    game_state.over = true;

    // Pretty sure this draws under the UI, so we'll just carefully avoid UI stuff.
    // A previous version of this used the UI, but it was causing JUST THE BACKGROUND
    // of the action pane to disappear.

    commands.spawn(SpriteBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
        material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.5).into()),
        sprite: Sprite::new(Vec2::new(108.0, 74.0)),
        ..Default::default()
    });

    commands.spawn(Text2dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
        text: Text {
            value: format!("やった!\n{}円", game_state.score),
            font: font_handles.jptext.clone(),
            style: TextStyle {
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
                font_size: FONT_SIZE,
                color: Color::WHITE,
                ..Default::default()
            },
        },
        ..Default::default()
    });
}

fn startup_system(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_state: ResMut<GameState>,
    mut typing_target_spawn_events: ResMut<Events<TypingTargetSpawnEvent>>,
    texture_handles: ResMut<TextureHandles>,
    font_handles: ResMut<FontHandles>,
    mut update_actions_events: ResMut<Events<UpdateActionsEvent>>,
) {
    info!("startup");

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(0.),
                    top: Val::Px(0.),
                    ..Default::default()
                },
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                size: Size::new(Val::Auto, Val::Px(42.0)),
                ..Default::default()
            },
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.50).into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    margin: Rect {
                        left: Val::Px(5.0),
                        ..Default::default()
                    },
                    size: Size::new(Val::Auto, Val::Px(32.0)),
                    ..Default::default()
                },
                // can I somehow get this from the sprite sheet? naively tossing a
                // spritesheetbundle here instead of an imagebundle seems to panic.
                material: materials.add(texture_handles.coin_ui.clone().into()),
                ..Default::default()
            });
            parent
                .spawn(TextBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(5.0),
                            right: Val::Px(10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: format!("{}", game_state.primary_currency),
                        font: font_handles.jptext.clone(),
                        style: TextStyle {
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(CurrencyDisplay);
            parent.spawn(ImageBundle {
                style: Style {
                    margin: Rect {
                        left: Val::Px(5.0),
                        ..Default::default()
                    },
                    size: Size::new(Val::Auto, Val::Px(32.0)),
                    ..Default::default()
                },
                material: materials.add(texture_handles.timer_ui.clone().into()),
                ..Default::default()
            });
            parent
                .spawn(TextBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(5.0),
                            right: Val::Px(10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: format!("{}", "30"),
                        font: font_handles.jptext.clone(),
                        style: TextStyle {
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(CooldownTimerDisplay);
        });

    // I don't know how to make the reticle invisible so I will just put out somewhere out
    // of view
    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
            sprite: TextureAtlasSprite {
                index: 16,
                ..Default::default()
            },
            texture_atlas: texture_handles.main_atlas.clone(),
            ..Default::default()
        })
        .with(Timer::from_seconds(0.01, true))
        .with(Reticle);

    for _ in 0..8 {
        let target = game_state
            .possible_typing_targets
            .pop_front()
            .unwrap()
            .clone();
        typing_target_spawn_events.send(TypingTargetSpawnEvent(target.clone(), None));
    }

    // Pretty sure this is duplicating the action update unnecessarily
    update_actions_events.send(UpdateActionsEvent);

    commands.spawn((
        TypingTarget {
            ascii: vec!["help".to_string()],
            render: vec!["help".to_string()],
        },
        Action::SwitchLanguageMode,
    ));
}

fn start_game(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    loading_screen_query: Query<Entity, With<LoadingScreen>>,
    loading_screen_text_query: Query<Entity, With<LoadingScreenText>>,
) {
    // TODO why did I not just make loading_screen_text a child?
    for loading_screen in loading_screen_query.iter() {
        commands.despawn(loading_screen);
    }
    for loading_screen_text in loading_screen_text_query.iter() {
        commands.despawn_recursive(loading_screen_text);
    }

    game_state.ready = true;
}

fn spawn_map_objects(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    texture_handles: Res<TextureHandles>,
    maps: Res<Assets<bevy_tiled_prototype::Map>>,
    mut update_actions_events: ResMut<Events<UpdateActionsEvent>>,
    mut waves: ResMut<Waves>,
) {
    // This seems pretty wild. Not remotely clear if this is the correct way to go about this,
    // but it seems to do the job.
    //
    // Because we're just worried about object data (and not placing sprites) from bevy_tiled
    // right now, it seems okay to potentially do this stuff before bevy_tiled is done processing
    // the asset event iself.

    use bevy_tiled_prototype::tiled::{Object, ObjectShape, PropertyValue};

    if let Some(map) = maps.get(texture_handles.tiled_map.clone()) {
        for grp in map.map.object_groups.iter() {
            let mut sorted = grp
                .objects
                .iter()
                .filter(|o| o.obj_type == "tile_slot")
                .filter(|o| o.properties.contains_key("index"))
                .filter_map(|o| match o.properties.get(&"index".to_string()) {
                    Some(PropertyValue::IntValue(index)) => Some((o, index)),
                    _ => None,
                })
                .collect::<Vec<(&Object, &i32)>>();

            sorted.sort_by(|a, b| a.1.cmp(b.1));

            for (obj, index) in sorted {
                // TODO We're just using centered maps right now, but we should be
                // able to find out if we should be centering these or not.
                //
                // Or better yet, bevy_tiled should provide this data to us
                // transformed somehow.
                let mut transform = map.center(Transform::default());

                // Y axis in bevy/tiled are reverse?
                transform.translation.x += obj.x + obj.width / 2.0;
                transform.translation.y -= obj.y - obj.height / 2.0;

                // I am just using these objects as markers right now, despite them
                // being associated with the correct tile. So there's no need to
                // draw these objects.

                game_state.tower_slots.push(
                    commands
                        .spawn(SpriteBundle {
                            transform,
                            ..Default::default()
                        })
                        .with(TowerSlot {
                            texture_ui: texture_handles.tower_slot_ui[*index as usize].clone(),
                        })
                        .current_entity()
                        .unwrap(),
                );
            }
        }

        // Pretty sure this is duplicating the action update unnecessarily
        update_actions_events.send(UpdateActionsEvent);

        // Try to grab the enemy path defined in the map
        for grp in map.map.object_groups.iter() {
            for (obj, points, _index) in grp
                .objects
                .iter()
                .filter(|o| o.obj_type == "enemy_path")
                .filter_map(
                    |o| match (&o.shape, o.properties.get(&"index".to_string())) {
                        (
                            ObjectShape::Polyline { points },
                            Some(PropertyValue::IntValue(index)),
                        ) => Some((o, points, index)),
                        (ObjectShape::Polygon { points }, Some(PropertyValue::IntValue(index))) => {
                            Some((o, points, index))
                        }
                        _ => None,
                    },
                )
            {
                let transformed: Vec<Vec2> = points
                    .iter()
                    .map(|(x, y)| {
                        let transform = map.center(Transform::default());

                        // Y axis in bevy/tiled are reverse?
                        Vec2::new(
                            transform.translation.x + obj.x + x,
                            transform.translation.y - obj.y - y,
                        )
                    })
                    .collect();

                // Temporary. We want to collect paths and reference them later when
                // collecting "wave objects."
                waves.waves.push(Wave {
                    path: transformed.clone(),
                    hp: 5,
                    ..Default::default()
                });
                waves.waves.push(Wave {
                    path: transformed.clone(),
                    hp: 9,
                    ..Default::default()
                });
                waves.waves.push(Wave {
                    path: transformed.clone(),
                    hp: 13,
                    ..Default::default()
                });
                waves.waves.push(Wave {
                    path: transformed.clone(),
                    hp: 17,
                    ..Default::default()
                })
            }
        }
    }
}

// Our main font is gigantic, but I'd like to use some text on the loading screen. So lets load
// a stripped down version.
//
// It probably makes way more sense to preload these things in JS or something, because the
// wasm bundle is also gigantic, so we'll want some sort of loading indicator there too.
//
// Or wasn't there some way to bundle the assets into the binary?
fn preload_assets_startup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut font_handles: ResMut<FontHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    font_handles.minimal = asset_server.load("fonts/NotoSans-Light-Min.ttf");

    commands
        // 2d camera
        .spawn(CameraUiBundle::default())
        .spawn(Camera2dBundle::default());

    commands
        .spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.5).into()),
            sprite: Sprite::new(Vec2::new(108.0, 42.0)),
            ..Default::default()
        })
        .with(LoadingScreen);

    commands
        .spawn(Text2dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
            text: Text {
                value: format!("Loading"),
                font: font_handles.minimal.clone(),
                style: TextStyle {
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                    font_size: FONT_SIZE,
                    color: Color::WHITE,
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(LoadingScreenText);
}

// TODO Show that loading screen
fn check_preload_assets(
    font_handles: Res<FontHandles>,
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    if let LoadState::Loaded = asset_server.get_load_state(font_handles.minimal.id) {
        state.set_next(AppState::Load).unwrap()
    }
}

fn load_assets_startup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut font_handles: ResMut<FontHandles>,
    mut texture_handles: ResMut<TextureHandles>,
) {
    font_handles.jptext = asset_server.load("fonts/NotoSansJP-Light.otf");

    texture_handles.main_atlas_texture = asset_server.load("textures/main.png");

    texture_handles.skel_atlas_texture = asset_server.load("textures/skeleton.png");

    // Also we need all these loose textures because UI doesn't speak TextureAtlas

    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_a.png"));
    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_b.png"));
    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_c.png"));
    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_d.png"));
    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_e.png"));
    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_f.png"));
    texture_handles.coin_ui = asset_server.load("textures/coin.png");
    texture_handles.back_ui = asset_server.load("textures/back_ui.png");
    texture_handles.tower_ui = asset_server.load("textures/tower_ui.png");
    texture_handles.timer_ui = asset_server.load("textures/timer.png");

    // And these because they don't fit on the grid...

    texture_handles.tower = asset_server.load("textures/shuriken_tower.png");
    texture_handles.bullet_shuriken = asset_server.load("textures/shuriken.png");

    //

    texture_handles.game_data = asset_server.load("data/game.ron");
    texture_handles.tiled_map = asset_server.load("textures/tiled-test.tmx");

    commands.spawn(bevy_tiled_prototype::TiledMapBundle {
        map_asset: texture_handles.tiled_map.clone(),
        center: TiledMapCenter(true),
        origin: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
        ..Default::default()
    });
}

fn check_load_assets(
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<AppState>>,
    font_handles: Res<FontHandles>,
    mut texture_handles: ResMut<TextureHandles>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut game_state: ResMut<GameState>,
    game_data_assets: Res<Assets<GameData>>,
    chunks: Query<&bevy_tiled_prototype::TileMapChunk>,
) {
    let ids = &[
        font_handles.jptext.id,
        texture_handles.coin_ui.id,
        texture_handles.back_ui.id,
        texture_handles.tower_ui.id,
        texture_handles.timer_ui.id,
        texture_handles.tower.id,
        texture_handles.bullet_shuriken.id,
        texture_handles.main_atlas_texture.id,
    ];

    // Surely there's a better way
    if ids.iter().any(|id| {
        if let LoadState::NotLoaded = asset_server.get_load_state(*id) {
            true
        } else {
            false
        }
    }) {
        return;
    }

    if texture_handles.tower_slot_ui.iter().any(|id| {
        if let LoadState::NotLoaded = asset_server.get_load_state(id) {
            true
        } else {
            false
        }
    }) {
        return;
    }

    if let LoadState::NotLoaded = asset_server.get_load_state(texture_handles.game_data.id) {
        return;
    }

    if chunks.iter().next().is_none() {
        return;
    }

    // Uh, why is the thing above not enough for custom assets?
    let game_data = game_data_assets.get(&texture_handles.game_data);
    // so I added these 4 lines and it broke everything
    if game_data.is_none() {
        return;
    }
    let game_data = game_data.unwrap();

    let mut rng = thread_rng();
    let mut possible_typing_targets =
        if let Ok(targets) = data::parse_typing_targets(game_data.lexicon.as_str()) {
            targets
        } else {
            vec![
                TypingTarget {
                    ascii: vec!["uhoh".to_string()],
                    render: vec!["uhoh".to_string()],
                },
                TypingTarget {
                    ascii: vec!["wehave".to_string()],
                    render: vec!["wehave".to_string()],
                },
                TypingTarget {
                    ascii: vec!["nodata".to_string()],
                    render: vec!["nodata".to_string()],
                },
            ]
        };

    possible_typing_targets.shuffle(&mut rng);
    game_state.possible_typing_targets = possible_typing_targets.into();

    let texture_atlas = TextureAtlas::from_grid(
        texture_handles.main_atlas_texture.clone(),
        Vec2::new(32.0, 32.0),
        16,
        16,
    );

    texture_handles.main_atlas = texture_atlases.add(texture_atlas);

    let skel_texture_atlas = TextureAtlas::from_grid(
        texture_handles.skel_atlas_texture.clone(),
        Vec2::new(32.0, 32.0),
        4,
        9,
    );

    texture_handles.skel_atlas = texture_atlases.add(skel_texture_atlas);

    state.set_next(AppState::Spawn).unwrap();
}

fn check_spawn(
    typing_targets: Query<Entity, With<TypingTargetImage>>,
    mut state: ResMut<State<AppState>>,
    waves: Res<Waves>,
) {
    // this whole phase is probably not actually doing anything, but it does serve as a
    // single place to put advance to the ready state from

    // typing targets are probably the last thing to spawn because they're spawned by an event
    // so maybe the game is ready if they are present.

    if typing_targets.iter().next().is_none() {
        return;
    }

    if waves.waves.len() < 1 {
        return;
    }

    state.set_next(AppState::Ready).unwrap();
}

fn main() {
    App::build()
        // Make bevy_webgl2 shut up
        .add_resource(LogSettings {
            filter: "bevy_webgl2=warn".into(),
            level: Level::INFO,
        })
        .add_resource(WindowDescriptor {
            width: 720.,
            height: 480.,
            canvas: Some("#bevy-canvas".to_string()),
            ..Default::default()
        })
        .add_resource(State::new(AppState::Preload))
        .add_stage_after(stage::UPDATE, STAGE, StateStage::<AppState>::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_webgl2::WebGL2Plugin)
        .add_plugin(bevy_tiled_prototype::TiledMapPlugin)
        .add_plugin(GameDataPlugin)
        .add_plugin(TypingPlugin)
        .on_state_enter(STAGE, AppState::Preload, preload_assets_startup.system())
        .on_state_update(STAGE, AppState::Preload, check_preload_assets.system())
        .on_state_enter(STAGE, AppState::Load, load_assets_startup.system())
        .on_state_update(STAGE, AppState::Load, check_load_assets.system())
        .on_state_enter(STAGE, AppState::Spawn, startup_system.system())
        .on_state_enter(STAGE, AppState::Spawn, spawn_map_objects.system())
        .on_state_update(STAGE, AppState::Spawn, check_spawn.system())
        .on_state_update(STAGE, AppState::Ready, start_game.system())
        .add_plugin(HealthBarPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(EnemyPlugin)
        .add_resource(GameState::default())
        .add_resource(Waves::default())
        .add_resource(CooldownTimerTimer(Timer::from_seconds(0.1, true)))
        .init_resource::<FontHandles>()
        .init_resource::<TextureHandles>()
        .add_system(typing_target_finished.system())
        .add_system(animate_reticle.system())
        .add_system(spawn_enemies.system())
        .add_system(shoot_enemies.system())
        .add_system(update_timer_display.system())
        .add_system(show_game_over.system())
        // this just needs to happen after TypingTargetSpawnEvent gets processed
        .add_stage_after(stage::UPDATE, "test1", SystemStage::parallel())
        .add_system_to_stage("test1", update_actions.system())
        // .. and this needs to happen after update_actions
        .add_stage_after(stage::UPDATE, "test2", SystemStage::parallel())
        .add_system_to_stage("test2", update_currency_display.system())
        .add_event::<UpdateActionsEvent>()
        .run();
}
