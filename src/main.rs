use bevy::{
    asset::LoadState,
    log::{Level, LogSettings},
    prelude::*,
    text::CalculatedSize,
};
use bevy_tiled_prototype::{Map, TiledMapCenter};
use bullet::BulletPlugin;
use data::{GameData, GameDataPlugin};
use enemy::{AnimationState, EnemyAttackTimer, EnemyPlugin, EnemyState, Skeleton};
use healthbar::HealthBarPlugin;
use rand::{prelude::SliceRandom, thread_rng, Rng};
use typing::{
    TypingPlugin, TypingState, TypingTarget, TypingTargetChangeEvent, TypingTargetContainer,
    TypingTargetFinishedEvent, TypingTargetFullText, TypingTargetImage, TypingTargetMatchedText,
    TypingTargetPriceContainer, TypingTargetPriceImage, TypingTargetPriceText,
    TypingTargetToggleModeEvent, TypingTargetUnmatchedText,
};

use std::collections::VecDeque;

#[macro_use]
extern crate anyhow;

mod app_stages;
mod bullet;
mod data;
mod enemy;
mod healthbar;
mod typing;

static TOWER_PRICE: u32 = 20;
pub static FONT_SIZE: f32 = 32.0;
pub static FONT_SIZE_ACTION_PANEL: f32 = 32.0;
pub static FONT_SIZE_INPUT: f32 = 32.0;
pub static FONT_SIZE_LABEL: f32 = 24.0;

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

struct ActionPanel {
    actions: Vec<ActionPanelItem>,
    entities: Vec<Entity>,
    update: u32,
}
impl Default for ActionPanel {
    fn default() -> Self {
        ActionPanel {
            actions: vec![],
            entities: vec![],
            update: 0,
        }
    }
}
struct ActionPanelItem {
    icon: Handle<Texture>,
    target: TypingTarget,
    action: Action,
    visible: bool,
    disabled: bool,
}

#[derive(Clone)]
enum Action {
    NoAction,
    SelectTower(Entity),
    GenerateMoney,
    UnselectTower,
    BuildBasicTower,
    UpgradeTower,
    SwitchLanguageMode,
}
impl Default for Action {
    fn default() -> Self {
        Action::NoAction
    }
}

struct CurrencyDisplay;
struct DelayTimerDisplay;
struct DelayTimerTimer(Timer);

#[derive(Debug)]
enum TowerType {
    Basic,
}

#[derive(Default, Debug)]
struct TowerStats {
    level: u32,
    range: f32,
    damage: u32,
    upgrade_price: u32,
    speed: f32,
}

#[derive(Default)]
struct TowerState {
    timer: Timer,
}

struct Reticle;
struct RangeIndicator;

struct Goal;

struct TowerSlot;
struct TowerSlotLabel;
struct TowerSlotLabelMatched;
struct TowerSlotLabelUnmatched;
struct TowerSlotLabelBg;

// Map and GameData don't really belong. Consolidate into AssetHandles?
#[derive(Default)]
pub struct TextureHandles {
    pub tower_slot_ui: Vec<Handle<Texture>>,
    pub coin_ui: Handle<Texture>,
    pub upgrade_ui: Handle<Texture>,
    pub back_ui: Handle<Texture>,
    pub tower: Handle<Texture>,
    pub tower_two: Handle<Texture>,
    pub range_indicator: Handle<Texture>,
    pub shuriken_tower_ui: Handle<Texture>,
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
    delay_timer: Timer,
    started: bool,
    spawned: usize,
    waves: Vec<Wave>,
}
impl Default for Waves {
    fn default() -> Self {
        Waves {
            current: 0,
            spawn_timer: Timer::from_seconds(1.0, true), // arbitrary, overwritten by wave
            delay_timer: Timer::from_seconds(30.0, false), // arbitrary, overwritten by wave
            started: false,
            spawned: 0,
            waves: vec![],
        }
    }
}

struct LoadingScreen;
struct LoadingScreenText;

fn spawn_action_panel_item(
    item: &ActionPanelItem,
    container: Entity,
    commands: &mut Commands,
    font_handles: &Res<FontHandles>,
    // just because we already had a resmut at the caller
    texture_handles: &ResMut<TextureHandles>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
) -> Entity {
    let mut rng = thread_rng();
    let price: u32 = rng.gen_range(1..300);

    let child = commands
        .spawn(NodeBundle {
            style: Style {
                display: if item.visible {
                    Display::Flex
                } else {
                    Display::None
                },
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                size: Size::new(Val::Percent(100.0), Val::Px(42.0)),
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with(item.target.clone())
        .with(item.action.clone())
        .with_children(|parent| {
            parent
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
                    material: materials.add(item.icon.clone().into()),
                    ..Default::default()
                })
                .with(TypingTargetImage);
            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            bottom: Val::Px(0.0),
                            left: Val::Px(2.0),
                            ..Default::default()
                        },
                        padding: Rect {
                            left: Val::Px(2.0),
                            right: Val::Px(2.0),
                            ..Default::default()
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        size: Size::new(Val::Px(38.0), Val::Px(16.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.5).into()),
                    ..Default::default()
                })
                .with(TypingTargetPriceContainer)
                .with_children(|parent| {
                    parent
                        .spawn(ImageBundle {
                            style: Style {
                                margin: Rect {
                                    right: Val::Px(2.0),
                                    ..Default::default()
                                },
                                size: Size::new(Val::Px(12.0), Val::Px(12.0)),
                                ..Default::default()
                            },
                            material: materials.add(texture_handles.coin_ui.clone().into()),
                            ..Default::default()
                        })
                        .with(TypingTargetPriceImage);
                    parent
                        .spawn(TextBundle {
                            style: Style {
                                ..Default::default()
                            },
                            text: Text {
                                value: format!("{}", price).into(),
                                font: font_handles.jptext.clone(),
                                style: TextStyle {
                                    font_size: 16.0, // 16px in this font is just not quite 16px is it?
                                    color: Color::WHITE,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        })
                        .with(TypingTargetPriceText);
                });
            parent
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text {
                        value: "".into(),
                        font: font_handles.jptext.clone(),
                        style: TextStyle {
                            font_size: FONT_SIZE_ACTION_PANEL,
                            color: Color::GREEN,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(TypingTargetMatchedText);
            parent
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text {
                        value: item.target.render.join(""),
                        font: font_handles.jptext.clone(),
                        style: TextStyle {
                            font_size: FONT_SIZE_ACTION_PANEL,
                            color: if item.disabled {
                                Color::GRAY
                            } else {
                                Color::WHITE
                            },
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(TypingTargetUnmatchedText);
        })
        .current_entity()
        .unwrap();

    commands.push_children(container, &[child]);

    child.clone()
}

// We should really store references to the various bits and pieces here Because
// this is sort of out of control.
fn update_actions(
    actions: ChangedRes<ActionPanel>,
    target_children_query: Query<&Children, With<TypingTarget>>,
    mut visible_query: Query<&mut Visible>,
    mut style_query: Query<&mut Style>,
    tower_query: Query<(&TowerState, &TowerType, &TowerStats)>,
    price_query: Query<(Entity, &Children), With<TypingTargetPriceContainer>>,
    mut price_text_query: Query<&mut Text, With<TypingTargetPriceText>>,
    mut matched_text_query: Query<&mut Text, With<TypingTargetMatchedText>>,
    mut unmatched_text_query: Query<&mut Text, With<TypingTargetUnmatchedText>>,
    game_state: Res<GameState>,
) {
    info!("update actions");

    for (item, entity) in actions.actions.iter().zip(actions.entities.iter()) {
        let visible = match item.action {
            Action::BuildBasicTower => match game_state.selected {
                Some(tower_slot) => tower_query.get(tower_slot).is_err(),
                None => false,
            },
            Action::GenerateMoney => game_state.selected.is_none(),
            Action::UnselectTower => game_state.selected.is_some(),
            Action::UpgradeTower => match game_state.selected {
                Some(tower_slot) => {
                    match tower_query.get(tower_slot) {
                        Ok((_, _, stats)) => {
                            // TODO
                            stats.level < 2
                        }
                        Err(_) => false,
                    }
                }
                None => false,
            },
            _ => false,
        };

        let price = match item.action {
            Action::BuildBasicTower => TOWER_PRICE,
            Action::UpgradeTower => match game_state.selected {
                Some(tower_slot) => match tower_query.get(tower_slot) {
                    Ok((_, _, stats)) => stats.upgrade_price,
                    Err(_) => 0,
                },
                None => 0,
            },
            _ => 0,
        };

        let disabled = price > game_state.primary_currency;
        let price_visible = visible && price > 0;

        // visibility

        if let Ok(mut style) = style_query.get_mut(*entity) {
            style.display = if visible {
                Display::Flex
            } else {
                Display::None
            };
        }
        // Workaround for #838/#1135
        if let Ok(children) = target_children_query.get(*entity) {
            for child in children.iter() {
                if let Ok(mut vis) = visible_query.get_mut(*child) {
                    vis.is_visible = visible;
                }
            }
        }

        // price

        if let Ok(target_children) = target_children_query.get(*entity) {
            for target_child in target_children.iter() {
                for (price_entity, children) in price_query.get(*target_child) {
                    if let Ok(mut style) = style_query.get_mut(price_entity) {
                        style.display = if price_visible {
                            Display::Flex
                        } else {
                            Display::None
                        };
                    }

                    for child in children.iter() {
                        if let Ok(mut vis) = visible_query.get_mut(*child) {
                            vis.is_visible = price_visible;
                        }
                    }

                    for child in children.iter() {
                        if let Ok(mut text) = price_text_query.get_mut(*child) {
                            text.value = format!("{}", price).into();
                        }
                    }
                }
            }
        }

        // disabledness
        // we could probably roll this into the vis queries at the expense of a headache

        if let Ok(target_children) = target_children_query.get(*entity) {
            for target_child in target_children.iter() {
                if let Ok(mut text) = unmatched_text_query.get_mut(*target_child) {
                    text.style.color = if disabled { Color::GRAY } else { Color::WHITE }
                }
                if let Ok(mut text) = matched_text_query.get_mut(*target_child) {
                    text.style.color = if disabled { Color::RED } else { Color::GREEN }
                }
            }
        }
    }
}

fn typing_target_finished(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    typing_state: ResMut<TypingState>,
    mut reader: Local<EventReader<TypingTargetFinishedEvent>>,
    typing_target_finished_events: Res<Events<TypingTargetFinishedEvent>>,
    mut typing_target_change_events: ResMut<Events<TypingTargetChangeEvent>>,
    mut toggle_events: ResMut<Events<TypingTargetToggleModeEvent>>,
    action_query: Query<&Action>,
    mut reticle_query: Query<&mut Transform, With<Reticle>>,
    tower_transform_query: Query<&Transform, With<TowerSlot>>,
    mut tower_state_query: Query<&mut TowerStats, With<TowerType>>,
    texture_handles: Res<TextureHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut action_panel: ResMut<ActionPanel>,
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

        let mut toggled_ascii_mode = false;

        for action in action_query.get(event.entity) {
            info!("there is some sort of action");
            if let Action::GenerateMoney = *action {
                info!("processing a GenerateMoney action");
                game_state.primary_currency = game_state.primary_currency.saturating_add(1);
                game_state.score = game_state.score.saturating_add(1);
            } else if let Action::SelectTower(tower) = *action {
                info!("processing a SelectTower action");
                game_state.selected = Some(tower);
                action_panel.update += 1;
            } else if let Action::UnselectTower = *action {
                info!("processing a UnselectTower action");
                game_state.selected = None;
                action_panel.update += 1;
            } else if let Action::SwitchLanguageMode = *action {
                info!("switching language mode!");
                toggled_ascii_mode = true;
                toggle_events.send(TypingTargetToggleModeEvent {});
                action_panel.update += 1;
            } else if let Action::UpgradeTower = *action {
                info!("upgrading tower!");

                // TODO tower config from game.ron
                if let Some(tower) = game_state.selected {
                    if let Ok(mut tower_state) = tower_state_query.get_mut(tower) {
                        // XXX
                        if tower_state.level < 2
                            && game_state.primary_currency > tower_state.upgrade_price
                        {
                            tower_state.level += 1;
                            tower_state.range += 32.0;
                            game_state.primary_currency -= tower_state.upgrade_price;
                        }
                    }
                }

                action_panel.update += 1;
            } else if let Action::BuildBasicTower = *action {
                if game_state.primary_currency < TOWER_PRICE {
                    continue;
                }
                game_state.primary_currency -= TOWER_PRICE;

                if let Some(tower) = game_state.selected {
                    // Should I.... bundle these... somehow?
                    commands.insert_one(
                        tower,
                        TowerStats {
                            level: 1,
                            range: 128.0,
                            damage: 1,
                            upgrade_price: 10,
                            speed: 1.0,
                            ..Default::default()
                        },
                    );
                    commands.insert_one(
                        tower,
                        TowerState {
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

            action_panel.update += 1;
        }

        // automatically switch out of ascii mode if we completed a word in ascii mode
        if !toggled_ascii_mode && typing_state.ascii_mode {
            toggle_events.send(TypingTargetToggleModeEvent {});
        }

        for mut reticle_transform in reticle_query.iter_mut() {
            if let Some(tower) = game_state.selected {
                for transform in tower_transform_query.get(tower) {
                    reticle_transform.translation.x = transform.translation.x;
                    reticle_transform.translation.y = transform.translation.y;
                    reticle_transform.translation.z = 20.0;
                }
            } else {
                reticle_transform.translation.z = -1.0;
            }
        }
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
    mut materials: ResMut<Assets<ColorMaterial>>,
    texture_handles: Res<TextureHandles>,
    game_state: Res<GameState>,
) {
    if !game_state.ready || game_state.over {
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
        waves.delay_timer.set_duration(wave_delay);
        waves.delay_timer.reset();
        return;
    }

    // There's nothing to do until the delay timer is finished.

    waves.delay_timer.tick(time.delta_seconds());
    if !waves.delay_timer.finished() {
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
            // enemies currently just "below" towers in z axis. This is okay because the
            // current map never shows an enemy in front of a tower.
            //
            // the z axis situation is hard to reason about because there are not really
            // "layers" so the background tiles are given z values based on their Tiled
            // layer id.
            //
            // we could probably hack something together where we do z = 100 + y, but
            // the camera is at 1000, so we may need to scale that. and everything's all
            // floaty, so that may lead to glitchy behavior when things are close together.
            .spawn(SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(point.x, point.y, 9.0)), // XXX magic z
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
            .with(EnemyAttackTimer(Timer::from_seconds(1.0, true)))
            .with(HitPoints {
                current: wave_hp,
                max: wave_hp,
            })
            .current_entity()
            .unwrap();

        healthbar::spawn(
            entity,
            commands,
            &mut materials,
            Vec2::new(16.0, 2.0),
            Vec2::new(0.0, 14.0),
            false,
            false,
        );

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
    mut timer: ResMut<DelayTimerTimer>,
    mut query: Query<&mut Text, With<DelayTimerDisplay>>,
    waves: Res<Waves>,
) {
    timer.0.tick(time.delta_seconds());
    if !timer.0.finished() {
        return;
    }

    for mut text in query.iter_mut() {
        let val = f32::max(
            0.0,
            waves.delay_timer.duration() - waves.delay_timer.elapsed(),
        );

        text.value = format!("{:.1}", val);
    }
}

fn shoot_enemies(
    time: Res<Time>,
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tower_query: Query<(&Transform, &mut TowerState, &TowerStats, &TowerType)>,
    enemy_query: Query<(Entity, &HitPoints, &Transform), With<EnemyState>>,
    texture_handles: Res<TextureHandles>,
) {
    for (transform, mut tower_state, tower_stats, _tower_type) in tower_query.iter_mut() {
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

            if d > tower_stats.range {
                continue;
            }

            // XXX
            let mut bullet_translation = transform.translation.clone();
            bullet_translation.y += 24.0;

            bullet::spawn(
                bullet_translation,
                enemy,
                tower_stats.damage,
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

fn update_tower_appearance(
    commands: &mut Commands,
    tower_query: Query<(Entity, &TowerStats, &Children), Changed<TowerStats>>,
    texture_handles: Res<TextureHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, stats, children) in tower_query.iter() {
        if stats.level == 2 {
            // Surely there's an easier way to swap out a single sprite when the sprite
            // replacing it has the same dimensions? I'm sure the answer is to use a texture
            // atlas.
            for child in children.iter() {
                commands.despawn(*child);
            }

            let new_child = commands
                .spawn(SpriteBundle {
                    material: materials.add(texture_handles.tower_two.clone().into()),
                    // Odd y value because the bottom of the sprite is not correctly
                    // positioned. Odd z value because we want to be above tiles but
                    // below the reticle.
                    transform: Transform::from_translation(Vec3::new(0.0, 20.0, 10.0)),
                    ..Default::default()
                })
                .current_entity()
                .unwrap();

            commands.push_children(entity, &[new_child]);
        }
    }
}

// Maybe we should break "selected" out of gamestate
fn update_range_indicator(
    mut query: Query<&mut Transform, With<RangeIndicator>>,
    game_state: ChangedRes<GameState>,
    tower_query: Query<(&Transform, &TowerStats), With<TowerStats>>,
) {
    if let Some(slot) = game_state.selected {
        if let Ok((tower_t, stats)) = tower_query.get(slot) {
            if let Some(mut t) = query.iter_mut().next() {
                t.translation.x = tower_t.translation.x;
                t.translation.y = tower_t.translation.y;

                // range is a radius, sprite width is diameter
                t.scale.x = stats.range * 2.0 / 722.0; // XXX magic sprite scaling factor
                t.scale.y = stats.range * 2.0 / 722.0; // XXX magic sprite scaling factor

                t.translation.z = 8.0; // XXX magic z, hope we don't have more than 8 tile layers
            }
        }
    } else {
        if let Some(mut t) = query.iter_mut().next() {
            t.translation.z = -1.0;
        }
    }
}

fn show_game_over(
    commands: &mut Commands,
    query: Query<&EnemyState>,
    goal_query: Query<&HitPoints, With<Goal>>,
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

    if !game_state.ready || game_state.over {
        return;
    }

    let over_win = if waves.current == waves.waves.len()
        && !query.iter().any(|x| match x.state {
            AnimationState::Corpse => false,
            _ => true,
        }) {
        true
    } else {
        false
    };

    let over_loss = if let Some(hp) = goal_query.iter().next() {
        hp.current <= 0
    } else {
        false
    };

    game_state.over = over_win || over_loss;

    if !game_state.over {
        return;
    }

    // Pretty sure this draws under the UI, so we'll just carefully avoid UI stuff.
    // A previous version of this used the UI, but it was causing JUST THE BACKGROUND
    // of the action pane to disappear.

    commands.spawn(SpriteBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 200.0)),
        material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.7).into()),
        sprite: Sprite::new(Vec2::new(128.0, 74.0)),
        ..Default::default()
    });

    commands.spawn(Text2dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 201.0)),
        text: Text {
            value: if over_win {
                format!("やった!\n{}円", game_state.score)
            } else {
                format!("やってない!\n{}円", game_state.score)
            },
            font: font_handles.jptext.clone(),
            style: TextStyle {
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
                font_size: FONT_SIZE,
                color: if over_win { Color::WHITE } else { Color::RED },
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
    texture_handles: ResMut<TextureHandles>,
    font_handles: Res<FontHandles>,
    mut action_panel: ResMut<ActionPanel>,
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
                .with(DelayTimerDisplay);
        });

    let action_container = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexEnd,
                size: Size::new(Val::Percent(30.0), Val::Auto),
                position_type: PositionType::Absolute,
                position: Rect {
                    right: Val::Px(0.),
                    top: Val::Px(0.),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.50).into()),
            ..Default::default()
        })
        .with(TypingTargetContainer)
        .current_entity()
        .unwrap();

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

    commands
        .spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
            material: materials.add(texture_handles.range_indicator.clone().into()),
            ..Default::default()
        })
        .with(Timer::from_seconds(0.01, true))
        .with(RangeIndicator);

    let mut actions = vec![];

    actions.push(ActionPanelItem {
        icon: texture_handles.coin_ui.clone(),
        target: game_state
            .possible_typing_targets
            .pop_front()
            .unwrap()
            .clone(),
        action: Action::GenerateMoney,
        visible: true,
        disabled: false,
    });
    actions.push(ActionPanelItem {
        icon: texture_handles.shuriken_tower_ui.clone(),
        target: game_state
            .possible_typing_targets
            .pop_front()
            .unwrap()
            .clone(),
        action: Action::BuildBasicTower,
        visible: false,
        disabled: false,
    });
    actions.push(ActionPanelItem {
        icon: texture_handles.upgrade_ui.clone(),
        target: game_state
            .possible_typing_targets
            .pop_front()
            .unwrap()
            .clone(),
        action: Action::UpgradeTower,
        visible: false,
        disabled: false,
    });
    actions.push(ActionPanelItem {
        icon: texture_handles.back_ui.clone(),
        target: game_state
            .possible_typing_targets
            .pop_front()
            .unwrap()
            .clone(),
        action: Action::UnselectTower,
        visible: false,
        disabled: false,
    });

    let entities: Vec<Entity> = actions
        .iter()
        .map(|action| {
            spawn_action_panel_item(
                &action,
                action_container,
                commands,
                &font_handles,
                &texture_handles,
                &mut materials,
            )
        })
        .collect();

    action_panel.actions = actions;
    action_panel.entities = entities;

    commands.spawn((
        TypingTarget {
            ascii: vec!["help".to_string()],
            render: vec!["help".to_string()],
        },
        Action::SwitchLanguageMode,
    ));
}

fn update_tower_slot_labels(
    mut left_query: Query<
        (
            &mut Transform,
            &mut GlobalTransform,
            &CalculatedSize,
            &Parent,
        ),
        (With<TowerSlotLabelUnmatched>, Changed<CalculatedSize>),
    >,
    mut right_query: Query<
        (&mut Transform, &mut GlobalTransform, &CalculatedSize),
        With<TowerSlotLabelMatched>,
    >,
    full_query: Query<&CalculatedSize, With<TowerSlotLabel>>,
    mut bg_query: Query<(&mut Sprite, &GlobalTransform), With<TowerSlotLabelBg>>,
    children_query: Query<&Children>,
) {
    for (mut left_t, mut left_gt, left_size, parent) in left_query.iter_mut() {
        // can probably just add Children to bg_query and use that here.
        if let Ok(children) = children_query.get(**parent) {
            // My iterator/result-fu is not enough for this.
            let mut full_width = 0.0;
            let mut global_x = 0.0;

            for child in children.iter() {
                if let Ok(full_size) = full_query.get(*child) {
                    full_width = full_size.size.width;
                }
            }

            if let Ok((mut bg_sprite, gt)) = bg_query.get_mut(**parent) {
                bg_sprite.size.x = full_width + 8.0;
                global_x = gt.translation.x;
            }

            // Muckign around with GlobalTransform seems completely necessary to prevent weird
            // positioning judder, but it seems to mess up heirarchical positioning. So we'll
            // just grab the parent's position and do that ourselves.

            left_t.translation.x = global_x + full_width / 2.0 - left_size.size.width / 2.0;
            left_gt.translation.x = global_x + full_width / 2.0 - left_size.size.width / 2.0;

            for child in children.iter() {
                if let Ok((mut right_t, mut right_gt, right_size)) = right_query.get_mut(*child) {
                    right_t.translation.x =
                        global_x - full_width / 2.0 + right_size.size.width / 2.0;
                    right_gt.translation.x =
                        global_x - full_width / 2.0 + right_size.size.width / 2.0;
                }
            }
        }
    }
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
    font_handles: Res<FontHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    maps: Res<Assets<bevy_tiled_prototype::Map>>,
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

            for (obj, _index) in sorted {
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
                let tower = commands
                    .spawn(SpriteBundle {
                        transform,
                        ..Default::default()
                    })
                    .with(TowerSlot)
                    .current_entity()
                    .unwrap();
                game_state.tower_slots.push(tower);

                let target = game_state
                    .possible_typing_targets
                    .pop_front()
                    .unwrap()
                    .clone();

                commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(
                            transform.translation.x,
                            transform.translation.y - 32.0,
                            99.0,
                        )),
                        material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.5).into()),
                        sprite: Sprite::new(Vec2::new(108.0, FONT_SIZE_LABEL)),
                        ..Default::default()
                    })
                    .with(TowerSlotLabelBg)
                    .with(target.clone())
                    .with(Action::SelectTower(tower))
                    .with_children(|parent| {
                        parent
                            .spawn(Text2dBundle {
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
                                text: Text {
                                    value: "".to_string(),
                                    font: font_handles.jptext.clone(),
                                    style: TextStyle {
                                        alignment: TextAlignment {
                                            vertical: VerticalAlign::Center,
                                            horizontal: HorizontalAlign::Center,
                                        },
                                        font_size: FONT_SIZE_LABEL,
                                        color: Color::GREEN,
                                        ..Default::default()
                                    },
                                },
                                ..Default::default()
                            })
                            .with(typing::TypingTargetMatchedText)
                            .with(TowerSlotLabelMatched);
                        parent
                            .spawn(Text2dBundle {
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
                                text: Text {
                                    value: target.render.join(""),
                                    font: font_handles.jptext.clone(),
                                    style: TextStyle {
                                        alignment: TextAlignment {
                                            vertical: VerticalAlign::Center,
                                            horizontal: HorizontalAlign::Center,
                                        },
                                        font_size: FONT_SIZE_LABEL,
                                        color: Color::WHITE,
                                        ..Default::default()
                                    },
                                },
                                ..Default::default()
                            })
                            .with(typing::TypingTargetUnmatchedText)
                            .with(TowerSlotLabelUnmatched);
                        parent
                            .spawn(Text2dBundle {
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 98.0)),
                                text: Text {
                                    value: target.render.join(""),
                                    font: font_handles.jptext.clone(),
                                    style: TextStyle {
                                        alignment: TextAlignment {
                                            vertical: VerticalAlign::Center,
                                            horizontal: HorizontalAlign::Center,
                                        },
                                        font_size: FONT_SIZE_LABEL,
                                        color: Color::NONE,
                                        ..Default::default()
                                    },
                                },
                                ..Default::default()
                            })
                            .with(TypingTargetFullText)
                            .with(TowerSlotLabel);
                    });
            }
        }

        for grp in map.map.object_groups.iter() {
            if let Some((pos, size, hp)) = grp
                .objects
                .iter()
                .filter(|o| o.obj_type == "goal")
                .map(|o| {
                    let hp = match o.properties.get(&"hp".to_string()) {
                        Some(PropertyValue::IntValue(hp)) => *hp as u32,
                        _ => 10 as u32,
                    };

                    let transform = map.center(Transform::default());

                    (
                        // Y axis in bevy/tiled are reverse?
                        Vec2::new(
                            transform.translation.x + o.x + o.width / 2.0,
                            transform.translation.y - o.y + o.height / 2.0,
                        ),
                        Vec2::new(o.width, o.height),
                        hp,
                    )
                })
                .next()
            {
                let entity = commands
                    .spawn(SpriteBundle {
                        transform: Transform::from_translation(pos.extend(10.0)), // XXX magic z
                        ..Default::default()
                    })
                    .with(Goal)
                    .with(HitPoints {
                        current: hp,
                        max: hp,
                    })
                    .current_entity()
                    .unwrap();
                healthbar::spawn(
                    entity,
                    commands,
                    &mut materials,
                    Vec2::new(size.x, size.y),
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                );
            }
        }

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
                    num: 8,
                    delay: 20.0, // XXX
                    hp: 5,
                    ..Default::default()
                });
                waves.waves.push(Wave {
                    path: transformed.clone(),
                    delay: 45.0,
                    hp: 9,
                    ..Default::default()
                });
                waves.waves.push(Wave {
                    path: transformed.clone(),
                    delay: 45.0,
                    hp: 13,
                    ..Default::default()
                });
                waves.waves.push(Wave {
                    path: transformed.clone(),
                    delay: 45.0,
                    hp: 17,
                    ..Default::default()
                });
                waves.waves.push(Wave {
                    path: transformed.clone(),
                    delay: 45.0,
                    hp: 21,
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
                    font_size: FONT_SIZE_ACTION_PANEL,
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

    texture_handles.coin_ui = asset_server.load("textures/coin.png");
    texture_handles.upgrade_ui = asset_server.load("textures/upgrade.png");
    texture_handles.back_ui = asset_server.load("textures/back_ui.png");
    texture_handles.shuriken_tower_ui = asset_server.load("textures/shuriken_tower_ui.png");
    texture_handles.timer_ui = asset_server.load("textures/timer.png");

    // And these because they don't fit on the grid...

    texture_handles.range_indicator = asset_server.load("textures/range_indicator.png");
    texture_handles.tower = asset_server.load("textures/shuriken_tower.png");
    texture_handles.tower_two = asset_server.load("textures/shuriken_tower_two.png");
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
        texture_handles.shuriken_tower_ui.id,
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
    mut actions: ResMut<ActionPanel>,
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

    // We need to force the action panel to update now that it has spawned
    // because we didn't bother initializing it properly. Surprisingly this seems to work
    // every time, but we should probably be on the lookout for actions not getting
    // initialized

    actions.update += 1;

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
        .on_state_enter(STAGE, AppState::Spawn, spawn_map_objects.system())
        .on_state_update(STAGE, AppState::Spawn, check_spawn.system())
        .on_state_update(STAGE, AppState::Spawn, update_actions.system())
        .on_state_enter(STAGE, AppState::Spawn, startup_system.system())
        .on_state_update(STAGE, AppState::Ready, start_game.system())
        .add_stage_after(
            stage::POST_UPDATE,
            app_stages::AFTER_POST_UPDATE,
            SystemStage::parallel(),
        )
        .add_stage_after(
            stage::UPDATE,
            app_stages::AFTER_UPDATE,
            SystemStage::parallel(),
        )
        .add_stage_after(
            app_stages::AFTER_UPDATE,
            app_stages::AFTER_UPDATE_2,
            SystemStage::parallel(),
        )
        .add_stage_after(
            STAGE,
            app_stages::AFTER_STATE_STAGE,
            SystemStage::parallel(),
        )
        .add_plugin(HealthBarPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(EnemyPlugin)
        .add_resource(GameState {
            primary_currency: 10,
            ..Default::default()
        })
        .init_resource::<ActionPanel>()
        .add_resource(Waves::default())
        .add_resource(DelayTimerTimer(Timer::from_seconds(0.1, true)))
        .init_resource::<FontHandles>()
        .init_resource::<TextureHandles>()
        .add_system(typing_target_finished.system())
        .add_system(animate_reticle.system())
        .add_system(spawn_enemies.system())
        .add_system(shoot_enemies.system())
        .add_system(update_timer_display.system())
        .add_system(update_tower_appearance.system())
        .add_system(show_game_over.system())
        // this just needs to happen after TypingTargetSpawnEvent gets processed
        .add_system_to_stage(app_stages::AFTER_UPDATE, update_actions.system())
        // .. and this needs to happen after update_actions
        .add_system_to_stage(app_stages::AFTER_UPDATE_2, update_currency_display.system())
        .add_system_to_stage(app_stages::AFTER_UPDATE_2, update_range_indicator.system())
        // Changed<CalculatedSize> works if we run after POST_UPDATE.
        .add_system_to_stage(
            app_stages::AFTER_POST_UPDATE,
            update_tower_slot_labels.system(),
        )
        .run();
}
