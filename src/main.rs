use bevy::utils::HashMap;
use bevy::{
    log::{Level, LogSettings},
    prelude::*,
    text::{CalculatedSize, TextSection},
};
use bevy_kira_audio::{AudioInitialization, AudioPlugin, AudioSource};
use bevy_tiled_prototype::{Map, TiledMapCenter};
use bullet::BulletPlugin;
use data::{AnimationData, GameData, GameDataPlugin};
use enemy::{AnimationState, EnemyAttackTimer, EnemyPlugin, EnemyState};
use healthbar::HealthBarPlugin;
use loading::LoadingPlugin;
use main_menu::MainMenuPlugin;
use typing::{
    TypingPlugin, TypingTarget, TypingTargetAsciiModeEvent, TypingTargetContainer,
    TypingTargetFinishedEvent, TypingTargetImage, TypingTargetPriceContainer,
    TypingTargetPriceImage, TypingTargetPriceText, TypingTargetText, TypingTargets,
};
use util::set_visible_recursive;

#[macro_use]
extern crate anyhow;

mod app_stages;
mod bullet;
mod data;
mod enemy;
mod healthbar;
mod loading;
mod main_menu;
mod typing;
mod util;

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
    MainMenu,
}

#[derive(Default)]
pub struct GameState {
    primary_currency: u32,
    score: u32,
    selected: Option<Entity>,
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
    ToggleMute,
}
impl Default for Action {
    fn default() -> Self {
        Action::NoAction
    }
}

struct CurrencyDisplay;
struct DelayTimerDisplay;
struct DelayTimerTimer(Timer);

struct TowerSprite;

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
struct TowerSlotLabelBg;
#[derive(Default)]
struct AudioSettings {
    mute: bool,
}

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
    pub reticle_atlas: Handle<TextureAtlas>,
    pub reticle_atlas_texture: Handle<Texture>,
    pub enemy_atlas: HashMap<String, Handle<TextureAtlas>>,
    pub enemy_atlas_texture: HashMap<String, Handle<Texture>>,
    pub tiled_map: Handle<Map>,
    pub game_data: Handle<GameData>,
}

#[derive(Default)]
pub struct AudioHandles {
    pub wrong_character: Handle<AudioSource>,
}

#[derive(Default)]
struct FontHandles {
    jptext: Handle<Font>,
    minimal: Handle<Font>,
}

#[derive(Default)]
struct AnimationHandles {
    handles: HashMap<String, Handle<AnimationData>>,
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
            enemy: "skeleton".to_string(),
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
    just_spawned: bool,
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
            just_spawned: false,
            waves: vec![],
        }
    }
}

fn spawn_action_panel_item(
    item: &ActionPanelItem,
    container: Entity,
    commands: &mut Commands,
    font_handles: &Res<FontHandles>,
    // just because we already had a resmut at the caller
    texture_handles: &ResMut<TextureHandles>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> Entity {
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
                            text: Text::with_section(
                                "0",
                                TextStyle {
                                    font: font_handles.jptext.clone(),
                                    font_size: 16.0, // 16px in this font is just not quite 16px is it?
                                    color: Color::WHITE,
                                    ..Default::default()
                                },
                                TextAlignment::default(),
                            ),
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
                        sections: vec![
                            TextSection {
                                value: "".into(),
                                style: TextStyle {
                                    font: font_handles.jptext.clone(),
                                    font_size: FONT_SIZE_ACTION_PANEL,
                                    color: Color::GREEN,
                                    ..Default::default()
                                },
                            },
                            TextSection {
                                value: item.target.render.join("").into(),
                                style: TextStyle {
                                    font: font_handles.jptext.clone(),
                                    font_size: FONT_SIZE_ACTION_PANEL,
                                    color: Color::WHITE,
                                    ..Default::default()
                                },
                            },
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(TypingTargetText);
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
    mut typing_target_query: Query<&mut TypingTarget>,
    children_query: Query<&Children>,
    mut visible_query: Query<&mut Visible>,
    mut style_query: Query<&mut Style>,
    tower_query: Query<(&TowerState, &TowerType, &TowerStats)>,
    price_query: Query<(Entity, &Children), With<TypingTargetPriceContainer>>,
    mut text_query: Query<&mut Text, With<TypingTargetText>>,
    mut price_text_query: Query<&mut Text, With<TypingTargetPriceText>>,
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
        set_visible_recursive(visible, *entity, &mut visible_query, &children_query);

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

                    // Workaround for #838/#1135
                    set_visible_recursive(
                        price_visible,
                        price_entity,
                        &mut visible_query,
                        &children_query,
                    );

                    for child in children.iter() {
                        if let Ok(mut text) = price_text_query.get_mut(*child) {
                            text.sections[0].value = format!("{}", price).into();
                        }
                    }
                    for child in children.iter() {
                        if let Ok(mut text) = price_text_query.get_mut(*child) {
                            text.sections[0].style.color =
                                if disabled { Color::RED } else { Color::WHITE };
                        }
                    }
                }
            }
        }

        // disabledness
        // we could probably roll this into the vis queries at the expense of a headache

        if let Ok(target_children) = target_children_query.get(*entity) {
            for target_child in target_children.iter() {
                if let Ok(mut text) = text_query.get_mut(*target_child) {
                    text.sections[0].style.color = if disabled { Color::RED } else { Color::GREEN };
                    text.sections[1].style.color = if disabled {
                        Color::DARK_GRAY
                    } else {
                        Color::WHITE
                    };
                }
            }
        }

        // we don't want invisible typing targets to get updated or make
        // sounds or whatever
        if let Ok(mut target) = typing_target_query.get_mut(*entity) {
            target.disabled = !visible;
        }
    }
}

fn typing_target_finished(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    mut reader: EventReader<TypingTargetFinishedEvent>,
    mut toggle_events: ResMut<Events<TypingTargetAsciiModeEvent>>,
    action_query: Query<&Action>,
    mut reticle_query: Query<&mut Transform, With<Reticle>>,
    tower_transform_query: Query<&Transform, With<TowerSlot>>,
    mut tower_state_query: Query<&mut TowerStats, With<TowerType>>,
    texture_handles: Res<TextureHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut action_panel: ResMut<ActionPanel>,
    mut sound_settings: ResMut<AudioSettings>,
) {
    for event in reader.iter() {
        info!("typing_target_finished");

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
                toggle_events.send(TypingTargetAsciiModeEvent {});
                action_panel.update += 1;
            } else if let Action::ToggleMute = *action {
                info!("toggling mute!");
                sound_settings.mute = !sound_settings.mute;
            } else if let Action::UpgradeTower = *action {
                info!("upgrading tower!");

                // TODO tower config from game.ron
                if let Some(tower) = game_state.selected {
                    if let Ok(mut tower_state) = tower_state_query.get_mut(tower) {
                        // XXX
                        if tower_state.level < 2
                            && game_state.primary_currency >= tower_state.upgrade_price
                        {
                            tower_state.level += 1;
                            tower_state.range += 32.0;
                            game_state.primary_currency -= tower_state.upgrade_price;
                        }
                    }
                }

                action_panel.update += 1;
            } else if let Action::BuildBasicTower = *action {
                info!("building tower!");

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
                        .with(TowerSprite)
                        .current_entity()
                        .unwrap();

                    commands.insert_children(tower, 0, &[child]);
                }
            }

            action_panel.update += 1;
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
            if sprite.index >= 15 {
                sprite.index = 0;
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
    if waves.just_spawned {
        waves.just_spawned = false;
    }

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

    let (wave_time, wave_num, wave_hp, wave_enemy) = {
        let wave = waves.waves.get(waves.current).unwrap();
        (
            wave.interval.clone(),
            wave.num.clone(),
            wave.hp.clone(),
            wave.enemy.clone(),
        )
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

        info!("spawn {:?}", wave_enemy);

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
                texture_atlas: texture_handles.enemy_atlas[&wave_enemy].clone(),
                ..Default::default()
            })
            .with(Timer::from_seconds(0.1, true))
            .with(EnemyState {
                path,
                name: wave_enemy.to_string(),
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

        waves.spawned += 1;
        waves.just_spawned = true;
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

        text.sections[0].value = format!("{:.1}", val);
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
        target.sections[0].value = format!("{}", game_state.primary_currency);
    }
}

fn update_tower_appearance(
    tower_query: Query<(&TowerStats, &Children), Changed<TowerStats>>,
    mut material_query: Query<&mut Handle<ColorMaterial>, With<TowerSprite>>,
    texture_handles: Res<TextureHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (stats, children) in tower_query.iter() {
        if stats.level == 2 {
            // Surely there's an easier way to swap out a single sprite when the sprite
            // replacing it has the same dimensions? I'm sure the answer is to use a texture
            // atlas.
            for child in children.iter() {
                if let Ok(mut material) = material_query.get_mut(*child) {
                    *material = materials.add(texture_handles.tower_two.clone().into());
                }
            }
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

    // count the number of non-corpses on the screen if we're on the last wave.
    // it takes a frame for those enemies to appear in the query, so also check
    // that we didn't just spawn an enemy on this frame.

    let over_win = if waves.current == waves.waves.len()
        && !waves.just_spawned
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
        text: Text::with_section(
            if over_win {
                format!("やった!\n{}円", game_state.score)
            } else {
                format!("やってない!\n{}円", game_state.score)
            },
            TextStyle {
                font: font_handles.jptext.clone(),
                font_size: FONT_SIZE,
                color: if over_win { Color::WHITE } else { Color::RED },
                ..Default::default()
            },
            TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        ),
        ..Default::default()
    });
}

fn startup_system(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_state: Res<GameState>,
    texture_handles: ResMut<TextureHandles>,
    font_handles: Res<FontHandles>,
    mut action_panel: ResMut<ActionPanel>,
    mut typing_targets: ResMut<TypingTargets>,
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
                    text: Text::with_section(
                        format!("{}", game_state.primary_currency),
                        TextStyle {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                        TextAlignment::default(),
                    ),
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
                    text: Text::with_section(
                        format!("{}", "30"),
                        TextStyle {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                        TextAlignment::default(),
                    ),
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
                index: 0,
                ..Default::default()
            },
            texture_atlas: texture_handles.reticle_atlas.clone(),
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
        target: typing_targets.pop_front(),
        action: Action::GenerateMoney,
        visible: true,
        disabled: false,
    });
    actions.push(ActionPanelItem {
        icon: texture_handles.shuriken_tower_ui.clone(),
        target: typing_targets.pop_front(),
        action: Action::BuildBasicTower,
        visible: false,
        disabled: false,
    });
    actions.push(ActionPanelItem {
        icon: texture_handles.upgrade_ui.clone(),
        target: typing_targets.pop_front(),
        action: Action::UpgradeTower,
        visible: false,
        disabled: false,
    });
    actions.push(ActionPanelItem {
        icon: texture_handles.back_ui.clone(),
        target: typing_targets.pop_front(),
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
            ascii: "help".split("").map(|s| s.to_string()).collect(),
            render: "help".split("").map(|s| s.to_string()).collect(),
            fixed: true,
            disabled: false,
        },
        Action::SwitchLanguageMode,
    ));

    commands.spawn((
        TypingTarget {
            ascii: "mute".split("").map(|s| s.to_string()).collect(),
            render: "mute".split("").map(|s| s.to_string()).collect(),
            fixed: true,
            disabled: false,
        },
        Action::ToggleMute,
    ));
}

fn update_tower_slot_labels(
    query: Query<(&CalculatedSize, &Parent), (With<TowerSlotLabel>, Changed<CalculatedSize>)>,
    mut bg_query: Query<&mut Sprite, With<TowerSlotLabelBg>>,
) {
    for (size, parent) in query.iter() {
        if let Ok(mut bg_sprite) = bg_query.get_mut(**parent) {
            bg_sprite.size.x = size.size.width + 8.0;
        }
    }
}

fn init_audio(_world: &mut World, resources: &mut Resources) {
    info!("init_audio");
    resources.insert(AudioInitialization {
        needed: true,
        ..Default::default()
    });
}

fn start_game(mut game_state: ResMut<GameState>) {
    game_state.ready = true;
}

fn spawn_map_objects(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    mut typing_targets: ResMut<TypingTargets>,
    texture_handles: Res<TextureHandles>,
    font_handles: Res<FontHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    maps: Res<Assets<bevy_tiled_prototype::Map>>,
    mut waves: ResMut<Waves>,
) {
    use bevy_tiled_prototype::tiled::{Object, ObjectShape, PropertyValue};

    if let Some(map) = maps.get(texture_handles.tiled_map.clone()) {
        for grp in map.map.object_groups.iter() {
            let mut tower_slots = grp
                .objects
                .iter()
                .filter(|o| o.obj_type == "tower_slot")
                .filter(|o| o.properties.contains_key("index"))
                .filter_map(|o| match o.properties.get(&"index".to_string()) {
                    Some(PropertyValue::IntValue(index)) => Some((o, index)),
                    _ => None,
                })
                .collect::<Vec<(&Object, &i32)>>();

            tower_slots.sort_by(|a, b| a.1.cmp(b.1));

            for (obj, _index) in tower_slots {
                // TODO We're just using centered maps right now, but we should be
                // able to find out if we should be centering these or not.
                //
                // Or better yet, bevy_tiled should provide this data to us
                // transformed somehow.
                let mut transform = map.center(Transform::default());

                // Y axis in bevy/tiled are reverse?
                transform.translation.x += obj.x + obj.width / 2.0;
                transform.translation.y -= obj.y - obj.height / 2.0;

                // These Tiled objects are just markers. The "tower slot" graphics are just a
                // a background tile, so this thing doesn't need be drawn. We'll add tower graphics
                // as a child later.
                let tower = commands
                    .spawn((transform, GlobalTransform::default()))
                    .with(TowerSlot)
                    .current_entity()
                    .unwrap();
                game_state.tower_slots.push(tower);

                let target = typing_targets.pop_front();

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
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 98.0)),
                                text: Text {
                                    alignment: TextAlignment {
                                        vertical: VerticalAlign::Center,
                                        horizontal: HorizontalAlign::Center,
                                    },
                                    sections: vec![
                                        TextSection {
                                            value: "".into(),
                                            style: TextStyle {
                                                font: font_handles.jptext.clone(),
                                                font_size: FONT_SIZE_LABEL,
                                                color: Color::GREEN,
                                                ..Default::default()
                                            },
                                        },
                                        TextSection {
                                            value: target.render.join(""),
                                            style: TextStyle {
                                                font: font_handles.jptext.clone(),
                                                font_size: FONT_SIZE_LABEL,
                                                color: Color::WHITE,
                                                ..Default::default()
                                            },
                                        },
                                    ],
                                },
                                ..Default::default()
                            })
                            .with(TypingTargetText)
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

        let paths: HashMap<i32, Vec<Vec2>> = map
            .map
            .object_groups
            .iter()
            .flat_map(|grp| grp.objects.iter())
            .filter(|o| o.obj_type == "enemy_path")
            .filter_map(
                |o| match (&o.shape, o.properties.get(&"index".to_string())) {
                    (ObjectShape::Polyline { points }, Some(PropertyValue::IntValue(index))) => {
                        Some((o, points, index))
                    }
                    (ObjectShape::Polygon { points }, Some(PropertyValue::IntValue(index))) => {
                        Some((o, points, index))
                    }
                    _ => None,
                },
            )
            .map(|(obj, points, index)| {
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

                (*index, transformed)
            })
            .collect();

        let mut map_waves = map
            .map
            .object_groups
            .iter()
            .flat_map(|grp| grp.objects.iter())
            .filter(|o| o.obj_type == "wave")
            .filter(|o| o.properties.contains_key("index"))
            .filter_map(|o| match o.properties.get(&"index".to_string()) {
                Some(PropertyValue::IntValue(index)) => Some((o, *index)),
                _ => None,
            })
            .collect::<Vec<(&Object, i32)>>();

        map_waves.sort_by(|a, b| a.1.cmp(&b.1));

        for (map_wave, _) in map_waves {
            let enemy = match map_wave.properties.get(&"enemy".to_string()) {
                Some(PropertyValue::StringValue(v)) => v.to_string(),
                _ => continue,
            };

            let num = match map_wave.properties.get(&"num".to_string()) {
                Some(PropertyValue::IntValue(v)) => *v as usize,
                _ => continue,
            };

            let delay = match map_wave.properties.get(&"delay".to_string()) {
                Some(PropertyValue::FloatValue(v)) => *v,
                _ => continue,
            };

            let interval = match map_wave.properties.get(&"interval".to_string()) {
                Some(PropertyValue::FloatValue(v)) => *v,
                _ => continue,
            };

            let hp = match map_wave.properties.get(&"hp".to_string()) {
                Some(PropertyValue::IntValue(v)) => *v as u32,
                _ => continue,
            };

            let path_index = match map_wave.properties.get(&"path_index".to_string()) {
                Some(PropertyValue::IntValue(v)) => *v as i32,
                _ => continue,
            };

            let path = match paths.get(&path_index) {
                Some(p) => p.clone(),
                None => {
                    warn!("Invalid path index");
                    continue;
                }
            };

            waves.waves.push(Wave {
                enemy,
                num,
                delay,
                interval,
                hp,
                path,
                ..Default::default()
            })
        }
    }
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
        .insert_resource(ReportExecutionOrderAmbiguities {})
        // Make bevy_webgl2 shut up
        .insert_resource(LogSettings {
            filter: "bevy_webgl2=warn".into(),
            level: Level::INFO,
        })
        .insert_resource(WindowDescriptor {
            width: 720.,
            height: 480.,
            canvas: Some("#bevy-canvas".to_string()),
            ..Default::default()
        })
        .insert_resource(State::new(AppState::Preload))
        .add_stage_after(stage::UPDATE, STAGE, StateStage::<AppState>::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_webgl2::WebGL2Plugin)
        .add_plugin(bevy_tiled_prototype::TiledMapPlugin)
        .insert_resource(AudioInitialization {
            needed: true,
            ..Default::default()
        })
        .add_plugin(AudioPlugin)
        .add_plugin(GameDataPlugin)
        .add_plugin(TypingPlugin)
        .add_plugin(MainMenuPlugin)
        // also, AppState::MainMenu from MainMenuPlugin
        .add_plugin(LoadingPlugin)
        // also, AppState::Preload from LoadingPlugin
        // also, AppState::Load from LoadingPlugin
        .on_state_enter(STAGE, AppState::Spawn, spawn_map_objects.system())
        .on_state_enter(STAGE, AppState::Spawn, startup_system.system())
        .on_state_update(STAGE, AppState::Spawn, check_spawn.system())
        .on_state_update(STAGE, AppState::Spawn, update_actions.system())
        .on_state_enter(STAGE, AppState::Ready, start_game.system())
        .on_state_enter(STAGE, AppState::Ready, init_audio.exclusive_system())
        .add_stage_after(
            stage::UPDATE,
            app_stages::AFTER_UPDATE,
            SystemStage::parallel(),
        )
        .add_stage_after(
            stage::POST_UPDATE,
            app_stages::AFTER_POST_UPDATE,
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
        .insert_resource(GameState {
            primary_currency: 10,
            ..Default::default()
        })
        .init_resource::<ActionPanel>()
        .init_resource::<AudioSettings>()
        .insert_resource(Waves::default())
        .insert_resource(DelayTimerTimer(Timer::from_seconds(0.1, true)))
        .init_resource::<FontHandles>()
        .init_resource::<TextureHandles>()
        .init_resource::<AnimationHandles>()
        .init_resource::<AudioHandles>()
        .add_system(animate_reticle.system())
        .add_system(spawn_enemies.system())
        .add_system(shoot_enemies.system())
        .add_system(update_timer_display.system())
        .add_system(
            typing_target_finished
                .system()
                .label("typing_target_finished"),
        )
        .add_system(
            update_tower_appearance
                .system()
                .after("typing_target_finished"),
        )
        .add_system(
            update_currency_display
                .system()
                .after("typing_target_finished"),
        )
        // update_actions and update_range_indicator need to be aware of TowerStats components
        // that get queued to spawn in the update stage.
        .add_system_to_stage(app_stages::AFTER_UPDATE, update_actions.system())
        .add_system_to_stage(app_stages::AFTER_UPDATE, update_range_indicator.system())
        // update_tower_slot_labels uses Changed<CalculatedSize> which only works if we run after
        // POST_UPDATE.
        .add_system_to_stage(
            app_stages::AFTER_POST_UPDATE,
            update_tower_slot_labels.system(),
        )
        .add_system(show_game_over.system())
        .run();
}
