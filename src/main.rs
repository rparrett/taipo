#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::forget_non_drop)] // https://github.com/bevyengine/bevy/issues/4601

use std::time::Duration;

use bevy::{
    ecs::schedule::ReportExecutionOrderAmbiguities,
    prelude::*,
    text::{Text2dSize, TextSection},
    utils::HashMap,
};
use bevy_ecs_tilemap::TilemapPlugin;

use crate::{
    bullet::BulletPlugin,
    data::{AnimationData, GameData, GameDataPlugin},
    enemy::{AnimationState, EnemyBundle, EnemyKind, EnemyPath, EnemyPlugin},
    healthbar::HealthBarPlugin,
    loading::LoadingPlugin,
    main_menu::MainMenuPlugin,
    map::{TiledMap, TiledMapPlugin},
    tower::{
        TowerBundle, TowerChangedEvent, TowerKind, TowerPlugin, TowerSprite, TowerState,
        TowerStats, TOWER_PRICE,
    },
    typing::{
        AsciiModeEvent, TypingPlugin, TypingTarget, TypingTargetContainer,
        TypingTargetFinishedEvent, TypingTargetImage, TypingTargetPriceContainer,
        TypingTargetPriceImage, TypingTargetPriceText, TypingTargetText, TypingTargets,
    },
    ui_z::{UiZ, UiZPlugin},
};

use tiled::{Object, ObjectShape, PropertyValue};

extern crate anyhow;

mod bullet;
mod data;
mod enemy;
mod healthbar;
mod japanese_parser;
mod layer;
mod loading;
mod main_menu;
mod map;
mod tower;
mod typing;
mod ui_color;
mod ui_z;
mod util;

pub static FONT_SIZE: f32 = 32.0;
pub static FONT_SIZE_ACTION_PANEL: f32 = 32.0;
pub static FONT_SIZE_INPUT: f32 = 32.0;
pub static FONT_SIZE_LABEL: f32 = 24.0;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum TaipoStage {
    AfterUpdate,
    AfterPostUpdate,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum TaipoState {
    Preload,
    Load,
    Spawn,
    Ready,
    MainMenu,
}
#[derive(Default)]
pub struct GameState {
    // Just so we can keep these in the correct order
    tower_slots: Vec<Entity>,
    over: bool,
    ready: bool,
}

pub struct Currency {
    current: u32,
    total_earned: u32,
}
impl Default for Currency {
    fn default() -> Self {
        Currency {
            current: 10,
            total_earned: 0,
        }
    }
}

#[derive(Default)]
pub struct TowerSelection {
    selected: Option<Entity>,
}

#[derive(Default)]
struct ActionPanel {
    actions: Vec<ActionPanelItem>,
    entities: Vec<Entity>,
    update: u32,
}

struct ActionPanelItem {
    icon: Handle<Image>,
    target: TypingTarget,
    action: Action,
    visible: bool,
}

#[derive(Clone, Component, Debug)]
enum Action {
    None,
    SelectTower(Entity),
    GenerateMoney,
    UnselectTower,
    BuildTower(TowerKind),
    UpgradeTower,
    SellTower,
    SwitchLanguageMode,
    ToggleMute,
}
impl Default for Action {
    fn default() -> Self {
        Action::None
    }
}

#[derive(Component)]
struct CurrencyDisplay;
#[derive(Component)]
struct DelayTimerDisplay;
#[derive(Component)]
struct DelayTimerTimer(Timer);

#[derive(Component)]
struct Reticle;
#[derive(Component)]
struct RangeIndicator;

#[derive(Component)]
struct Goal;

#[derive(Component)]
struct TowerSlot;
#[derive(Component)]
struct TowerSlotLabel;
#[derive(Component)]
struct TowerSlotLabelBg;
#[derive(Default)]
struct AudioSettings {
    mute: bool,
}

// Map and GameData don't really belong. Consolidate into AssetHandles?
#[derive(Default)]
pub struct TextureHandles {
    pub tower_slot: Handle<Image>,
    pub coin_ui: Handle<Image>,
    pub upgrade_ui: Handle<Image>,
    pub back_ui: Handle<Image>,
    pub tower: Handle<Image>,
    pub tower_two: Handle<Image>,
    pub support_tower: Handle<Image>,
    pub support_tower_two: Handle<Image>,
    pub debuff_tower: Handle<Image>,
    pub debuff_tower_two: Handle<Image>,
    pub range_indicator: Handle<Image>,
    pub status_up: Handle<Image>,
    pub status_down: Handle<Image>,
    pub shuriken_tower_ui: Handle<Image>,
    pub support_tower_ui: Handle<Image>,
    pub debuff_tower_ui: Handle<Image>,
    pub timer_ui: Handle<Image>,
    pub sell_ui: Handle<Image>,
    pub bullet_shuriken: Handle<Image>,
    pub bullet_debuff: Handle<Image>,
    pub reticle: Handle<Image>,
    pub enemy_atlas: HashMap<String, Handle<TextureAtlas>>,
    pub enemy_atlas_texture: HashMap<String, Handle<Image>>,
    pub tiled_map: Handle<TiledMap>,
    pub game_data: Handle<GameData>,
}

#[derive(Default)]
pub struct AudioHandles {
    pub wrong_character: Handle<AudioSource>,
}

#[derive(Default)]
pub struct FontHandles {
    jptext: Handle<Font>,
    minimal: Handle<Font>,
}

#[derive(Default)]
struct AnimationHandles {
    handles: HashMap<String, Handle<AnimationData>>,
}

#[derive(Component)]
pub struct HitPoints {
    current: u32,
    max: u32,
}
impl Default for HitPoints {
    fn default() -> Self {
        HitPoints { current: 1, max: 1 }
    }
}
#[derive(Component)]
pub struct Speed(f32);
impl Default for Speed {
    fn default() -> Self {
        Self(20.0)
    }
}

#[derive(Clone, Debug)]
pub struct Wave {
    path: Vec<Vec2>,
    enemy: String,
    num: usize,
    hp: u32,
    armor: u32,
    speed: f32,
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
            armor: 0,
            speed: 20.0,
            interval: 3.0,
            delay: 30.0,
        }
    }
}

#[derive(Debug)]
struct WaveState {
    current: usize,
    spawn_timer: Timer,
    delay_timer: Timer,
    started: bool,
    spawned: usize,
    just_spawned: bool,
}

impl Default for WaveState {
    fn default() -> Self {
        WaveState {
            current: 0,
            spawn_timer: Timer::from_seconds(1.0, true), // arbitrary, overwritten by wave
            delay_timer: Timer::from_seconds(30.0, false), // arbitrary, overwritten by wave
            started: false,
            spawned: 0,
            just_spawned: false,
        }
    }
}
#[derive(Default)]
pub struct Waves {
    pub waves: Vec<Wave>,
}

#[derive(Component, Default)]
pub struct StatusEffects(Vec<StatusEffect>);
impl StatusEffects {
    pub fn get_max_sub_armor(&self) -> u32 {
        self.0
            .iter()
            .filter_map(|e| match e.kind {
                StatusEffectKind::SubArmor(amt) => Some(amt),
                _ => None,
            })
            .max()
            .unwrap_or(0)
    }

    pub fn get_total_add_damage(&self) -> u32 {
        self.0
            .iter()
            .filter_map(|e| match e.kind {
                StatusEffectKind::AddDamage(amt) => Some(amt),
                _ => None,
            })
            .sum::<u32>()
    }
}

#[derive(Clone, Debug)]
pub struct StatusEffect {
    pub kind: StatusEffectKind,
    pub timer: Option<Timer>,
}
#[derive(Clone, Debug)]
pub enum StatusEffectKind {
    SubArmor(u32),
    AddDamage(u32),
}
#[derive(Component)]
pub struct StatusUpSprite;
#[derive(Component)]
pub struct StatusDownSprite;

#[derive(Component, Default)]
pub struct Armor(u32);

fn spawn_action_panel_item(
    item: &ActionPanelItem,
    container: Entity,
    commands: &mut Commands,
    font_handles: &Res<FontHandles>,
    // just because we already had a resmut at the caller
    texture_handles: &ResMut<TextureHandles>,
) -> Entity {
    let child = commands
        .spawn_bundle(NodeBundle {
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
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(item.target.clone())
        .insert(item.action.clone())
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(5.0),
                            right: Val::Px(5.0),
                            ..Default::default()
                        },
                        size: Size::new(Val::Auto, Val::Px(32.0)),
                        ..Default::default()
                    },
                    image: item.icon.clone().into(),
                    ..Default::default()
                })
                .insert(TypingTargetImage);
            parent
                .spawn_bundle(NodeBundle {
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
                    color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
                    ..Default::default()
                })
                .insert(TypingTargetPriceContainer)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(ImageBundle {
                            style: Style {
                                margin: Rect {
                                    right: Val::Px(2.0),
                                    ..Default::default()
                                },
                                size: Size::new(Val::Px(12.0), Val::Px(12.0)),
                                ..Default::default()
                            },
                            image: texture_handles.coin_ui.clone().into(),
                            ..Default::default()
                        })
                        .insert(TypingTargetPriceImage);
                    parent
                        .spawn_bundle(TextBundle {
                            style: Style {
                                ..Default::default()
                            },
                            text: Text::with_section(
                                "0",
                                TextStyle {
                                    font: font_handles.jptext.clone(),
                                    font_size: 16.0, // 16px in this font is just not quite 16px is it?
                                    color: Color::WHITE,
                                },
                                TextAlignment::default(),
                            ),
                            ..Default::default()
                        })
                        .insert(TypingTargetPriceText);
                });
            parent
                .spawn_bundle(TextBundle {
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
                                },
                            },
                            TextSection {
                                value: item.target.displayed_chunks.join(""),
                                style: TextStyle {
                                    font: font_handles.jptext.clone(),
                                    font_size: FONT_SIZE_ACTION_PANEL,
                                    color: Color::WHITE,
                                },
                            },
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(TypingTargetText);
        })
        .id();

    commands.entity(container).push_children(&[child]);

    child
}

fn update_action_panel(
    mut typing_target_query: Query<&mut TypingTarget>,
    mut style_query: Query<&mut Style>,
    mut text_query: Query<&mut Text, (With<TypingTargetText>, Without<TypingTargetPriceText>)>,
    mut price_text_query: Query<
        &mut Text,
        (With<TypingTargetPriceText>, Without<TypingTargetText>),
    >,
    target_children_query: Query<&Children, With<TypingTarget>>,
    tower_query: Query<(&TowerState, &TowerKind, &TowerStats)>,
    price_query: Query<(Entity, &Children), With<TypingTargetPriceContainer>>,
    (actions, currency, selection): (Res<ActionPanel>, Res<Currency>, Res<TowerSelection>),
) {
    if !actions.is_changed() {
        return;
    }

    info!("update actions");

    for (item, entity) in actions.actions.iter().zip(actions.entities.iter()) {
        let visible = match item.action {
            Action::BuildTower(_) => match selection.selected {
                Some(tower_slot) => tower_query.get(tower_slot).is_err(),
                None => false,
            },
            Action::GenerateMoney => selection.selected.is_none(),
            Action::UnselectTower => selection.selected.is_some(),
            Action::UpgradeTower => match selection.selected {
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
            Action::SellTower => match selection.selected {
                Some(tower_slot) => tower_query.get(tower_slot).is_ok(),
                None => false,
            },
            _ => false,
        };

        let price = match item.action {
            Action::BuildTower(tower_type) => match tower_type {
                TowerKind::Basic => TOWER_PRICE,
                TowerKind::Support => TOWER_PRICE,
                TowerKind::Debuff => TOWER_PRICE,
            },
            Action::UpgradeTower => match selection.selected {
                Some(tower_slot) => match tower_query.get(tower_slot) {
                    Ok((_, _, stats)) => stats.upgrade_price,
                    Err(_) => 0,
                },
                None => 0,
            },
            _ => 0,
        };

        let disabled = price > currency.current;
        let price_visible = visible && price > 0;

        // visibility

        if let Ok(mut style) = style_query.get_mut(*entity) {
            style.display = if visible {
                Display::Flex
            } else {
                Display::None
            };
        }

        // price

        if let Ok(target_children) = target_children_query.get(*entity) {
            for target_child in target_children.iter() {
                if let Ok((price_entity, children)) = price_query.get(*target_child) {
                    if let Ok(mut style) = style_query.get_mut(price_entity) {
                        style.display = if price_visible {
                            Display::Flex
                        } else {
                            Display::None
                        };
                    }

                    for child in children.iter() {
                        if let Ok(mut text) = price_text_query.get_mut(*child) {
                            text.sections[0].value = format!("{}", price);
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
                    text.sections[1].style.color = if disabled { Color::RED } else { Color::WHITE };
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

fn typing_target_finished_event(
    mut commands: Commands,
    mut tower_state_query: Query<&mut TowerStats, With<TowerKind>>,
    tower_children_query: Query<&Children, With<TowerSlot>>,
    tower_sprite_query: Query<Entity, With<TowerSprite>>,
    mut reticle_query: Query<
        (&mut Transform, &mut Visibility),
        (With<Reticle>, Without<TowerSlot>),
    >,
    action_query: Query<&Action>,
    tower_transform_query: Query<&Transform, (With<TowerSlot>, Without<Reticle>)>,
    texture_handles: Res<TextureHandles>,
    (mut reader, mut toggle_events, mut tower_changed_events): (
        EventReader<TypingTargetFinishedEvent>,
        EventWriter<AsciiModeEvent>,
        EventWriter<TowerChangedEvent>,
    ),
    (mut currency, mut selection, mut action_panel, mut sound_settings): (
        ResMut<Currency>,
        ResMut<TowerSelection>,
        ResMut<ActionPanel>,
        ResMut<AudioSettings>,
    ),
) {
    for event in reader.iter() {
        info!("typing_target_finished");

        let mut toggled_ascii_mode = false;

        if let Ok(action) = action_query.get(event.entity) {
            info!("Processing action: {:?}", action);

            if let Action::GenerateMoney = *action {
                currency.current = currency.current.saturating_add(1);
                currency.total_earned = currency.total_earned.saturating_add(1);
            } else if let Action::SelectTower(tower) = *action {
                selection.selected = Some(tower);
                action_panel.update += 1;
            } else if let Action::UnselectTower = *action {
                selection.selected = None;
                action_panel.update += 1;
            } else if let Action::SwitchLanguageMode = *action {
                toggle_events.send(AsciiModeEvent::Toggle);
                toggled_ascii_mode = true;
                action_panel.update += 1;
            } else if let Action::ToggleMute = *action {
                sound_settings.mute = !sound_settings.mute;
            } else if let Action::UpgradeTower = *action {
                // TODO tower config from game.ron
                if let Some(tower) = selection.selected {
                    if let Ok(mut tower_state) = tower_state_query.get_mut(tower) {
                        // XXX
                        if tower_state.level < 2 && currency.current >= tower_state.upgrade_price {
                            tower_state.level += 1;
                            tower_state.range += 32.0;

                            currency.current -= tower_state.upgrade_price;

                            tower_changed_events.send(TowerChangedEvent);
                        }
                    }
                }

                action_panel.update += 1;
            } else if let Action::BuildTower(tower_kind) = *action {
                if currency.current < TOWER_PRICE {
                    continue;
                }
                currency.current -= TOWER_PRICE;

                if let Some(tower) = selection.selected {
                    commands
                        .entity(tower)
                        .insert_bundle(TowerBundle::new(tower_kind));

                    tower_changed_events.send(TowerChangedEvent);
                }
            } else if let Action::SellTower = *action {
                if let Some(tower) = selection.selected {
                    commands.entity(tower).remove_bundle::<TowerBundle>();

                    if let Ok(children) = tower_children_query.get(tower) {
                        for child in children.iter() {
                            if let Ok(ent) = tower_sprite_query.get(*child) {
                                commands.entity(ent).despawn();

                                let new_child = commands
                                    .spawn_bundle(SpriteBundle {
                                        texture: texture_handles.tower_slot.clone(),
                                        transform: Transform::from_translation(Vec3::new(
                                            0.0,
                                            0.0,
                                            layer::TOWER_SLOT,
                                        )),
                                        ..Default::default()
                                    })
                                    .insert(TowerSprite)
                                    .id();

                                commands.entity(tower).push_children(&[new_child]);
                            }
                        }
                    }

                    // TODO refund upgrade price too
                    currency.current = currency.current.saturating_add(TOWER_PRICE / 2);

                    tower_changed_events.send(TowerChangedEvent);
                }
            }

            action_panel.update += 1;
        }

        if !toggled_ascii_mode {
            toggle_events.send(AsciiModeEvent::Disable);
        }

        for (mut reticle_transform, mut reticle_visible) in reticle_query.iter_mut() {
            if let Some(tower) = selection.selected {
                if let Ok(transform) = tower_transform_query.get(tower) {
                    reticle_transform.translation.x = transform.translation.x;
                    reticle_transform.translation.y = transform.translation.y;
                }
                reticle_visible.is_visible = true;
            } else {
                reticle_visible.is_visible = false;
            }
        }
    }
}

fn animate_reticle(mut query: Query<&mut Transform, With<Reticle>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        let delta = time.delta_seconds();
        transform.rotate(Quat::from_rotation_z(-2.0 * delta));
    }
}

fn spawn_enemies(
    mut commands: Commands,
    waves: ResMut<Waves>,
    mut wave_state: ResMut<WaveState>,
    time: Res<Time>,
    texture_handles: Res<TextureHandles>,
    game_state: Res<GameState>,
) {
    if wave_state.just_spawned {
        wave_state.just_spawned = false;
    }

    if !game_state.ready || game_state.over {
        return;
    }

    let current_wave = match waves.waves.get(wave_state.current) {
        Some(wave) => wave,
        None => return,
    };

    // If we haven't started the delay timer for a new wave yet,
    // go ahead and do that.

    if !wave_state.started {
        wave_state.started = true;
        wave_state
            .delay_timer
            .set_duration(Duration::from_secs_f32(current_wave.delay));
        wave_state.delay_timer.reset();
        return;
    }

    // There's nothing to do until the delay timer is finished.

    wave_state.delay_timer.tick(time.delta());
    if !wave_state.delay_timer.finished() {
        return;
    }

    wave_state.spawn_timer.tick(time.delta());

    // immediately spawn the first enemy and start the timer
    let spawn = if wave_state.spawned == 0 {
        wave_state
            .spawn_timer
            .set_duration(Duration::from_secs_f32(current_wave.interval));
        wave_state.spawn_timer.reset();
        true
    } else {
        wave_state.spawn_timer.just_finished()
    };

    if spawn {
        let path = current_wave.path.clone();
        let point = path.get(0).unwrap();

        let entity = commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(point.x, point.y, layer::ENEMY)),
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..Default::default()
                },
                texture_atlas: texture_handles.enemy_atlas[&current_wave.enemy].clone(),
                ..Default::default()
            })
            .insert_bundle(EnemyBundle {
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
            })
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

        wave_state.spawned += 1;
        wave_state.just_spawned = true;
    }

    // that was the last enemy
    if wave_state.spawned == current_wave.num {
        wave_state.current += 1;
        wave_state.spawned = 0;
        wave_state.started = false;
    }
}

fn update_timer_display(
    mut query: Query<&mut Text, With<DelayTimerDisplay>>,
    mut timer: ResMut<DelayTimerTimer>,
    time: Res<Time>,
    wave_state: Res<WaveState>,
) {
    timer.0.tick(time.delta());
    if !timer.0.finished() {
        return;
    }

    for mut text in query.iter_mut() {
        let val = f32::max(
            0.0,
            (wave_state.delay_timer.duration() - wave_state.delay_timer.elapsed()).as_secs_f32(),
        );

        text.sections[0].value = format!("{:.1}", val);
    }
}

fn update_currency_text(
    currency: Res<Currency>,
    mut currency_display_query: Query<&mut Text, With<CurrencyDisplay>>,
) {
    if !currency.is_changed() {
        return;
    }

    for mut target in currency_display_query.iter_mut() {
        target.sections[0].value = format!("{}", currency.current);
    }
}

fn show_game_over(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    currency: Res<Currency>,
    query: Query<&AnimationState>,
    goal_query: Query<&HitPoints, With<Goal>>,
    waves: Res<Waves>,
    wave_state: Res<WaveState>,
    font_handles: Res<FontHandles>,
) {
    // Hm. This was triggering before the game started, so we'll just check
    // to see if there's at least one wave.

    if waves.waves.is_empty() {
        return;
    }

    if !game_state.ready || game_state.over {
        return;
    }

    // count the number of non-corpses on the screen if we're on the last wave.
    // it takes a frame for those enemies to appear in the query, so also check
    // that we didn't just spawn an enemy on this frame.

    let over_win = wave_state.current == waves.waves.len()
        && !wave_state.just_spawned
        && query.iter().all(|x| matches!(x, AnimationState::Corpse));

    let over_loss = if let Some(hp) = goal_query.iter().next() {
        hp.current == 0
    } else {
        false
    };

    game_state.over = over_win || over_loss;

    if !game_state.over {
        return;
    }

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                justify_content: JustifyContent::Center,
                align_self: AlignSelf::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: ui_color::OVERLAY.into(),
            ..Default::default()
        })
        .insert(UiZ(layer::UI_OVERLAY))
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        padding: Rect::all(Val::Px(20.)),
                        ..Default::default()
                    },
                    color: ui_color::BACKGROUND.into(),
                    ..Default::default()
                })
                .insert(UiZ(layer::UI_OVERLAY))
                .with_children(|parent| {
                    parent
                        .spawn_bundle(TextBundle {
                            text: Text::with_section(
                                if over_win {
                                    format!("やった!\n{}円", currency.total_earned)
                                } else {
                                    format!("やってない!\n{}円", currency.total_earned)
                                },
                                TextStyle {
                                    font: font_handles.jptext.clone(),
                                    font_size: FONT_SIZE,
                                    color: if over_win { Color::WHITE } else { Color::RED },
                                },
                                TextAlignment {
                                    vertical: VerticalAlign::Center,
                                    horizontal: HorizontalAlign::Center,
                                },
                            ),
                            ..Default::default()
                        })
                        .insert(UiZ(layer::UI_OVERLAY));
                });
        });
}

fn startup_system(
    mut commands: Commands,
    texture_handles: ResMut<TextureHandles>,
    mut action_panel: ResMut<ActionPanel>,
    mut typing_targets: ResMut<TypingTargets>,
    font_handles: Res<FontHandles>,
    currency: Res<Currency>,
) {
    info!("startup");

    commands
        .spawn_bundle(NodeBundle {
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
            color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(ImageBundle {
                style: Style {
                    margin: Rect {
                        left: Val::Px(5.0),
                        ..Default::default()
                    },
                    size: Size::new(Val::Auto, Val::Px(32.0)),
                    ..Default::default()
                },
                image: texture_handles.coin_ui.clone().into(),
                ..Default::default()
            });
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(5.0),
                            right: Val::Px(10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text::with_section(
                        format!("{}", currency.current),
                        TextStyle {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(CurrencyDisplay);
            parent.spawn_bundle(ImageBundle {
                style: Style {
                    margin: Rect {
                        left: Val::Px(5.0),
                        ..Default::default()
                    },
                    size: Size::new(Val::Auto, Val::Px(32.0)),
                    ..Default::default()
                },
                image: texture_handles.timer_ui.clone().into(),
                ..Default::default()
            });
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(5.0),
                            right: Val::Px(10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "30".to_string(),
                        TextStyle {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(DelayTimerDisplay);
        });

    let action_container = commands
        .spawn_bundle(NodeBundle {
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
            color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
            ..Default::default()
        })
        .insert(TypingTargetContainer)
        .id();

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, layer::RETICLE)),
            texture: texture_handles.reticle.clone(),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Reticle);

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, layer::RANGE_INDICATOR)),
            texture: texture_handles.range_indicator.clone(),
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(RangeIndicator);

    let actions = vec![
        ActionPanelItem {
            icon: texture_handles.coin_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::GenerateMoney,
            visible: true,
        },
        ActionPanelItem {
            icon: texture_handles.shuriken_tower_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::BuildTower(TowerKind::Basic),
            visible: false,
        },
        ActionPanelItem {
            icon: texture_handles.support_tower_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::BuildTower(TowerKind::Support),
            visible: false,
        },
        ActionPanelItem {
            icon: texture_handles.debuff_tower_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::BuildTower(TowerKind::Debuff),
            visible: false,
        },
        ActionPanelItem {
            icon: texture_handles.upgrade_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::UpgradeTower,
            visible: false,
        },
        ActionPanelItem {
            icon: texture_handles.sell_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::SellTower,
            visible: false,
        },
        ActionPanelItem {
            icon: texture_handles.back_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::UnselectTower,
            visible: false,
        },
    ];

    let entities: Vec<Entity> = actions
        .iter()
        .map(|action| {
            spawn_action_panel_item(
                action,
                action_container,
                &mut commands,
                &font_handles,
                &texture_handles,
            )
        })
        .collect();

    action_panel.actions = actions;
    action_panel.entities = entities;

    commands
        .spawn()
        .insert(TypingTarget {
            typed_chunks: "help".split("").map(|s| s.to_string()).collect(),
            displayed_chunks: "help".split("").map(|s| s.to_string()).collect(),
            fixed: true,
            disabled: false,
        })
        .insert(Action::SwitchLanguageMode);

    commands
        .spawn()
        .insert(TypingTarget {
            typed_chunks: "mute".split("").map(|s| s.to_string()).collect(),
            displayed_chunks: "mute".split("").map(|s| s.to_string()).collect(),
            fixed: true,
            disabled: false,
        })
        .insert(Action::ToggleMute);
}

fn update_tower_slot_labels(
    mut bg_query: Query<&mut Sprite, With<TowerSlotLabelBg>>,
    query: Query<(&Text2dSize, &Parent), (With<TowerSlotLabel>, Changed<Text2dSize>)>,
) {
    for (size, parent) in query.iter() {
        if let Ok(mut bg_sprite) = bg_query.get_mut(**parent) {
            if let Some(bg_sprite_size) = bg_sprite.custom_size {
                bg_sprite.custom_size = Some(Vec2::new(size.size.width + 8.0, bg_sprite_size.y));
            }
        }
    }
}

fn start_game(mut game_state: ResMut<GameState>) {
    game_state.ready = true;
}

fn spawn_map_objects(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut typing_targets: ResMut<TypingTargets>,
    mut waves: ResMut<Waves>,
    texture_handles: Res<TextureHandles>,
    font_handles: Res<FontHandles>,
    maps: Res<Assets<TiledMap>>,
) {
    let tiled_map = match maps.get(texture_handles.tiled_map.clone()) {
        Some(map) => map,
        None => panic!("Queried map not in assets?"),
    };

    info!("spawn_map_objects");

    // paths

    let paths: HashMap<i32, Vec<Vec2>> = tiled_map
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
                    let transform = crate::util::map_to_world(
                        tiled_map,
                        Vec2::new(*x, *y) + Vec2::new(obj.x, obj.y),
                        Vec2::new(0.0, 0.0),
                        0.0,
                    );
                    transform.translation.truncate()
                })
                .collect();

            (*index, transformed)
        })
        .collect();

    // waves

    let mut map_waves = tiled_map
        .map
        .object_groups
        .iter()
        .flat_map(|grp| grp.objects.iter())
        .filter(|o| o.obj_type == "wave")
        .collect::<Vec<&Object>>();

    map_waves.sort_by(|a, b| a.x.partial_cmp(&b.x).expect("sorting waves"));

    for map_wave in map_waves.iter() {
        info!("{:?}", map_wave.properties);
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

        let armor = match map_wave.properties.get(&"armor".to_string()) {
            Some(PropertyValue::IntValue(v)) => *v as u32,
            _ => continue,
        };

        let speed = match map_wave.properties.get(&"speed".to_string()) {
            Some(PropertyValue::FloatValue(v)) => *v,
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
            armor,
            speed,
            path,
        });
    }

    // goal

    for grp in tiled_map.map.object_groups.iter() {
        if let Some((transform, size, hp)) = grp
            .objects
            .iter()
            .filter(|o| o.obj_type == "goal")
            .map(|o| {
                let hp = match o.properties.get(&"hp".to_string()) {
                    Some(PropertyValue::IntValue(hp)) => *hp as u32,
                    _ => 10,
                };

                let pos = Vec2::new(o.x, o.y);
                let size = Vec2::new(o.width, o.height);

                let transform = crate::util::map_to_world(tiled_map, pos, size, layer::ENEMY);

                (transform, size, hp)
            })
            .next()
        {
            let entity = commands
                .spawn_bundle(SpriteBundle {
                    transform,
                    ..Default::default()
                })
                .insert(Goal)
                .insert(HitPoints {
                    current: hp,
                    max: hp,
                })
                .id();

            healthbar::spawn(
                entity,
                healthbar::HealthBar {
                    size: Vec2::new(size.x, size.y),
                    offset: Vec2::new(0.0, 0.0),
                    show_full: true,
                    show_empty: true,
                },
                &mut commands,
            );
        }
    }

    // tower slots

    for grp in tiled_map.map.object_groups.iter() {
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
            let pos = Vec2::new(obj.x, obj.y);
            let size = Vec2::new(obj.width, obj.height);

            let transform = util::map_to_world(tiled_map, pos, size, 0.0);

            let mut label_bg_transform = transform;
            label_bg_transform.translation.y -= 32.0;
            label_bg_transform.translation.z = layer::TOWER_SLOT_LABEL_BG;

            let tower = commands
                .spawn_bundle((transform, GlobalTransform::default()))
                .insert(TowerSlot)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(SpriteBundle {
                            texture: texture_handles.tower_slot.clone(),
                            transform: Transform::from_xyz(0.0, 0.0, layer::TOWER_SLOT),
                            ..Default::default()
                        })
                        .insert(TowerSprite);
                })
                .id();

            game_state.tower_slots.push(tower);

            let target = typing_targets.pop_front();

            commands
                .spawn_bundle(SpriteBundle {
                    transform: label_bg_transform,
                    sprite: Sprite {
                        color: Color::rgba(0.0, 0.0, 0.0, 0.7),
                        custom_size: Some(Vec2::new(108.0, FONT_SIZE_LABEL)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(TowerSlotLabelBg)
                .insert(target.clone())
                .insert(Action::SelectTower(tower))
                .with_children(|parent| {
                    parent
                        .spawn_bundle(Text2dBundle {
                            transform: Transform::from_xyz(0.0, 0.0, 0.1),
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
                                        },
                                    },
                                    TextSection {
                                        value: target.displayed_chunks.join(""),
                                        style: TextStyle {
                                            font: font_handles.jptext.clone(),
                                            font_size: FONT_SIZE_LABEL,
                                            color: Color::WHITE,
                                        },
                                    },
                                ],
                            },
                            ..Default::default()
                        })
                        .insert(TypingTargetText)
                        .insert(TowerSlotLabel);
                });
        }
    }
}

fn check_spawn(
    mut state: ResMut<State<TaipoState>>,
    mut actions: ResMut<ActionPanel>,
    typing_targets: Query<Entity, With<TypingTargetImage>>,
    waves: Res<Waves>,
) {
    // this whole phase is probably not actually doing anything, but it does serve as a
    // single place to put advance to the ready state from

    // typing targets are probably the last thing to spawn because they're spawned by an event
    // so maybe the game is ready if they are present.

    if typing_targets.iter().next().is_none() {
        return;
    }

    if waves.waves.is_empty() {
        return;
    }

    // We need to force the action panel to update now that it has spawned
    // because we didn't bother initializing it properly. Surprisingly this seems to work
    // every time, but we should probably be on the lookout for actions not getting
    // initialized

    actions.update += 1;

    state.replace(TaipoState::Ready).unwrap();
}

fn main() {
    let mut app = App::new();

    app.insert_resource(ReportExecutionOrderAmbiguities {});

    #[cfg(target_arch = "wasm32")]
    app.insert_resource(WindowDescriptor {
        width: 720.,
        height: 480.,
        ..Default::default()
    });
    #[cfg(not(target_arch = "wasm32"))]
    app.insert_resource(WindowDescriptor {
        width: 720.,
        height: 480.,
        ..Default::default()
    });

    app.add_state(TaipoState::Preload);

    app.add_stage_after(
        CoreStage::Update,
        TaipoStage::AfterUpdate,
        SystemStage::parallel(),
    )
    .add_stage_after(
        CoreStage::PostUpdate,
        TaipoStage::AfterPostUpdate,
        SystemStage::parallel(),
    );

    app.add_plugins(DefaultPlugins);

    app.add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_plugin(GameDataPlugin)
        .add_plugin(TypingPlugin)
        .add_plugin(MainMenuPlugin)
        // also, AppState::MainMenu from MainMenuPlugin
        .add_plugin(LoadingPlugin)
        // also, AppState::Preload from LoadingPlugin
        // also, AppState::Load from LoadingPlugin
        .add_plugin(TowerPlugin)
        .add_event::<TowerChangedEvent>()
        .add_system_set(
            SystemSet::on_enter(TaipoState::Spawn)
                .with_system(spawn_map_objects)
                .with_system(startup_system),
        )
        .add_system_set(
            SystemSet::on_update(TaipoState::Spawn)
                .with_system(check_spawn)
                .with_system(update_action_panel),
        )
        .add_system_set(SystemSet::on_enter(TaipoState::Ready).with_system(start_game))
        .add_plugin(UiZPlugin)
        .add_plugin(HealthBarPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(EnemyPlugin)
        .init_resource::<GameState>()
        .init_resource::<Currency>()
        .init_resource::<TowerSelection>()
        .init_resource::<ActionPanel>()
        .init_resource::<AudioSettings>()
        .insert_resource(Waves::default())
        .insert_resource(WaveState::default())
        .insert_resource(DelayTimerTimer(Timer::from_seconds(0.1, true)))
        .init_resource::<FontHandles>()
        .init_resource::<TextureHandles>()
        .init_resource::<AnimationHandles>()
        .init_resource::<AudioHandles>()
        .add_system(animate_reticle)
        .add_system(update_timer_display)
        .add_system(typing_target_finished_event.label("typing_target_finished_event"))
        .add_system(
            update_currency_text
                .label("update_currency_text")
                .after("typing_target_finished_event"),
        )
        .add_system(spawn_enemies.label("spawn_enemies"))
        .add_system(show_game_over.after("spawn_enemies"))
        // update_actions_panel and update_range_indicator need to be aware of TowerStats components
        // that get queued to spawn in the update stage.)
        .add_system_to_stage(TaipoStage::AfterUpdate, update_action_panel)
        // update_tower_slot_labels uses Changed<CalculatedSize> which only works if we run after
        // POST_UPDATE.
        .add_system_to_stage(TaipoStage::AfterPostUpdate, update_tower_slot_labels)
        .run();
}
