use std::time::Duration;

use bevy::{ecs::schedule::ReportExecutionOrderAmbiguities, utils::HashMap};
use bevy::{
    log::{Level, LogSettings},
    prelude::*,
    text::{Text2dSize, TextSection},
};
use bevy_kira_audio::{AudioPlugin, AudioSource};
use bevy_tiled_prototype::{
    tiled::{ObjectShape, PropertyValue},
    Object,
};
use bevy_tiled_prototype::{Map, TiledMapCenter};
use bullet::BulletPlugin;
use data::{AnimationData, GameData, GameDataPlugin};
use enemy::{AnimationState, EnemyBundle, EnemyKind, EnemyPath, EnemyPlugin};
use healthbar::HealthBarPlugin;
use loading::LoadingPlugin;
use main_menu::MainMenuPlugin;
use typing::{
    AsciiModeEvent, TypingPlugin, TypingTarget, TypingTargetContainer, TypingTargetFinishedEvent,
    TypingTargetImage, TypingTargetPriceContainer, TypingTargetPriceImage, TypingTargetPriceText,
    TypingTargetText, TypingTargets,
};

use util::set_visible_recursive;

extern crate anyhow;

mod bullet;
mod data;
mod enemy;
mod healthbar;
mod japanese_parser;
mod layer;
mod loading;
mod main_menu;
mod typing;
mod util;

static TOWER_PRICE: u32 = 20;
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

// This is getting quite bloated and probably contributing to a lot of
// noise in the ambiguity detector.
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

#[derive(Clone, Debug)]
enum Action {
    None,
    SelectTower(Entity),
    GenerateMoney,
    UnselectTower,
    BuildTower(TowerType),
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

struct CurrencyDisplay;
struct DelayTimerDisplay;
struct DelayTimerTimer(Timer);

struct TowerSprite;

#[derive(Debug, Copy, Clone)]
enum TowerType {
    Basic,
    Support,
    Debuff,
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
    pub tower_slot: Handle<Texture>,
    pub coin_ui: Handle<Texture>,
    pub upgrade_ui: Handle<Texture>,
    pub back_ui: Handle<Texture>,
    pub tower: Handle<Texture>,
    pub tower_two: Handle<Texture>,
    pub support_tower: Handle<Texture>,
    pub support_tower_two: Handle<Texture>,
    pub debuff_tower: Handle<Texture>,
    pub debuff_tower_two: Handle<Texture>,
    pub range_indicator: Handle<Texture>,
    pub status_up: Handle<Texture>,
    pub status_down: Handle<Texture>,
    pub shuriken_tower_ui: Handle<Texture>,
    pub support_tower_ui: Handle<Texture>,
    pub debuff_tower_ui: Handle<Texture>,
    pub timer_ui: Handle<Texture>,
    pub sell_ui: Handle<Texture>,
    pub bullet_shuriken: Handle<Texture>,
    pub bullet_debuff: Handle<Texture>,
    pub reticle: Handle<Texture>,
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

pub struct HitPoints {
    current: u32,
    max: u32,
}
impl Default for HitPoints {
    fn default() -> Self {
        HitPoints { current: 1, max: 1 }
    }
}
pub struct Speed(f32);
impl Default for Speed {
    fn default() -> Self {
        Self(20.0)
    }
}

#[derive(Clone, Debug)]
struct Wave {
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
struct Waves {
    waves: Vec<Wave>,
}

#[derive(Default)]
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
pub struct StatusUpSprite;
pub struct StatusDownSprite;

#[derive(Default)]
pub struct Armor(u32);

struct TowerChangedEvent;

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
            material: materials.add(Color::NONE.into()),
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
                    material: materials.add(item.icon.clone().into()),
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
                    material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.7).into()),
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
                            material: materials.add(texture_handles.coin_ui.clone().into()),
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

#[allow(clippy::too_many_arguments)]
fn update_action_panel(
    mut typing_target_query: Query<&mut TypingTarget>,
    mut visible_query: Query<&mut Visible>,
    mut style_query: Query<&mut Style>,
    mut text_query: Query<&mut Text, (With<TypingTargetText>, Without<TypingTargetPriceText>)>,
    mut price_text_query: Query<
        &mut Text,
        (With<TypingTargetPriceText>, Without<TypingTargetText>),
    >,
    target_children_query: Query<&Children, With<TypingTarget>>,
    children_query: Query<&Children>,
    tower_query: Query<(&TowerState, &TowerType, &TowerStats)>,
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
                TowerType::Basic => TOWER_PRICE,
                TowerType::Support => TOWER_PRICE,
                TowerType::Debuff => TOWER_PRICE,
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

        // Workaround for #838/#1135
        set_visible_recursive(visible, *entity, &mut visible_query, &children_query);

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

                    // Workaround for #838/#1135
                    set_visible_recursive(
                        price_visible,
                        price_entity,
                        &mut visible_query,
                        &children_query,
                    );

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
                    text.sections[1].style.color =
                        if disabled { Color::GRAY } else { Color::WHITE };
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

// This currently does not work properly for status effects with timers, but
// we don't have any of those in game yet.
fn update_tower_status_effect_appearance(
    mut commands: Commands,
    query: Query<(Entity, &StatusEffects, &Children), (With<TowerType>, Changed<StatusEffects>)>,
    up_query: Query<Entity, With<StatusUpSprite>>,
    down_query: Query<Entity, With<StatusDownSprite>>,
    tower_sprite_query: Query<&Sprite, With<TowerSprite>>,
    texture_handles: Res<TextureHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, status_effects, children) in query.iter() {
        let down = status_effects.get_max_sub_armor() > 0;
        let up = status_effects.get_total_add_damage() > 0;

        let sprite = children
            .iter()
            .filter_map(|child| tower_sprite_query.get(*child).ok())
            .next()
            .expect("no sprite for tower?");

        for child in children.iter() {
            match (down, down_query.get(*child)) {
                (true, Err(_)) => {
                    let down_ent = commands
                        .spawn_bundle(SpriteBundle {
                            material: materials.add(texture_handles.status_down.clone().into()),
                            transform: Transform::from_translation(Vec3::new(
                                sprite.size.x / 2.0 + 6.0,
                                -12.0,
                                layer::HEALTHBAR_BG,
                            )),
                            ..Default::default()
                        })
                        .insert(StatusDownSprite)
                        .id();
                    commands.entity(entity).push_children(&[down_ent]);
                }
                (false, Ok(down_ent)) => {
                    commands.entity(down_ent).despawn_recursive();
                }
                _ => {}
            }
            match (up, up_query.get(*child)) {
                (true, Err(_)) => {
                    let up_ent = commands
                        .spawn_bundle(SpriteBundle {
                            material: materials.add(texture_handles.status_up.clone().into()),
                            transform: Transform::from_translation(Vec3::new(
                                sprite.size.x / 2.0 + 6.0,
                                -12.0,
                                layer::HEALTHBAR_BG,
                            )),
                            ..Default::default()
                        })
                        .insert(StatusUpSprite)
                        .id();
                    commands.entity(entity).push_children(&[up_ent]);
                }
                (false, Ok(up_ent)) => {
                    commands.entity(up_ent).despawn_recursive();
                }
                _ => {}
            }
        }
    }
}

fn update_tower_status_effects(
    mut reader: EventReader<TowerChangedEvent>,
    query: Query<Entity, With<TowerState>>,
    kind_query: Query<&TowerType>,
    transform_query: Query<&Transform>,
    stats_query: Query<&TowerStats>,
    mut status_query: Query<&mut StatusEffects>,
) {
    if reader.iter().next().is_none() {
        return;
    }

    let towers: Vec<_> = query.iter().collect();

    for entity in towers.iter() {
        if let Ok(mut status) = status_query.get_mut(*entity) {
            status.0.clear();
        }
    }

    for support_entity in towers.iter() {
        if !matches!(kind_query.get(*support_entity), Ok(TowerType::Support)) {
            continue;
        }

        for entity in towers.iter() {
            if entity == support_entity {
                continue;
            }

            if let Ok(mut status) = status_query.get_mut(*entity) {
                let support_transform = transform_query.get(*support_entity).unwrap();
                let support_stats = stats_query.get(*support_entity).unwrap();
                let transform = transform_query.get(*entity).unwrap();

                let dist = transform
                    .translation
                    .truncate()
                    .distance(support_transform.translation.truncate());

                if dist < support_stats.range {
                    status.0.push(StatusEffect {
                        kind: StatusEffectKind::AddDamage(1),
                        timer: None,
                    });
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn typing_target_finished_event(
    mut commands: Commands,
    mut tower_state_query: Query<&mut TowerStats, With<TowerType>>,
    tower_children_query: Query<&Children, With<TowerSlot>>,
    tower_sprite_query: Query<Entity, With<TowerSprite>>,
    mut reticle_query: Query<(&mut Transform, &mut Visible), (With<Reticle>, Without<TowerSlot>)>,
    action_query: Query<&Action>,
    tower_transform_query: Query<&Transform, (With<TowerSlot>, Without<Reticle>)>,
    texture_handles: Res<TextureHandles>,
    (mut reader, mut toggle_events, mut tower_changed_events): (
        EventReader<TypingTargetFinishedEvent>,
        EventWriter<AsciiModeEvent>,
        EventWriter<TowerChangedEvent>,
    ),
    (mut currency, mut selection, mut materials, mut action_panel, mut sound_settings): (
        ResMut<Currency>,
        ResMut<TowerSelection>,
        ResMut<Assets<ColorMaterial>>,
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
            } else if let Action::BuildTower(tower_type) = *action {
                if currency.current < TOWER_PRICE {
                    continue;
                }
                currency.current -= TOWER_PRICE;

                if let Some(tower) = selection.selected {
                    let damage = match tower_type {
                        TowerType::Basic => 1,
                        _ => 0,
                    };

                    commands
                        .entity(tower)
                        .insert(TowerStats {
                            level: 1,
                            range: 128.0,
                            damage,
                            upgrade_price: 10,
                            speed: 1.0,
                        })
                        .insert(TowerState {
                            timer: Timer::from_seconds(1.0, true),
                        })
                        .insert(StatusEffects::default())
                        .insert(tower_type);

                    tower_changed_events.send(TowerChangedEvent);
                }
            } else if let Action::SellTower = *action {
                if let Some(tower) = selection.selected {
                    commands
                        .entity(tower)
                        .remove::<TowerType>()
                        .remove::<TowerStats>()
                        .remove::<TowerState>()
                        .remove::<StatusEffects>();

                    if let Ok(children) = tower_children_query.get(tower) {
                        for child in children.iter() {
                            if let Ok(ent) = tower_sprite_query.get(*child) {
                                commands.entity(ent).despawn();

                                let new_child = commands
                                    .spawn_bundle(SpriteBundle {
                                        material: materials
                                            .add(texture_handles.tower_slot.clone().into()),
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
    mut materials: ResMut<Assets<ColorMaterial>>,
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
            &mut materials,
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

fn shoot_enemies(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tower_query: Query<(
        &Transform,
        &mut TowerState,
        &TowerStats,
        &TowerType,
        &StatusEffects,
    )>,
    enemy_query: Query<(Entity, &HitPoints, &Transform), With<EnemyKind>>,
    texture_handles: Res<TextureHandles>,
    time: Res<Time>,
) {
    for (transform, mut tower_state, tower_stats, tower_type, status_effects) in
        tower_query.iter_mut()
    {
        if let TowerType::Support = *tower_type {
            continue;
        }

        tower_state.timer.tick(time.delta());
        if !tower_state.timer.finished() {
            continue;
        }

        // we are just naively iterating over every enemy right now. at some point we should
        // investigate whether some spatial data structure is useful here. but there is overhead
        // involved in maintaining one and I think it's unlikely that we'd break even with the
        // small amount of enemies and towers we're dealing with here.

        let mut in_range = enemy_query
            .iter()
            .filter(|(_, hp, _)| hp.current > 0)
            .filter(|(_, _, enemy_transform)| {
                let dist = enemy_transform
                    .translation
                    .truncate()
                    .distance(transform.translation.truncate());

                dist <= tower_stats.range
            });

        // right now, possibly coincidentally, this query seems to be iterating in the order that
        // the enemies were spawned.
        //
        // with all enemies current walking at the same speed, that is equivalent to the enemy
        // furthest along the path, which is the default behavior we probably want.
        //
        // other options might be to sort the in-range enemies and select
        // - closest to tower
        // - furthest along path
        // - highest health
        // - lowest health

        if let Some((enemy, _, _)) = in_range.next() {
            let mut bullet_translation = transform.translation;
            bullet_translation.y += 24.0; // XXX magic sprite offset

            let material = match tower_type {
                TowerType::Basic => materials.add(texture_handles.bullet_shuriken.clone().into()),
                TowerType::Debuff => materials.add(texture_handles.bullet_debuff.clone().into()),
                _ => panic!(),
            };

            let status = match tower_type {
                TowerType::Debuff => Some(StatusEffect {
                    kind: StatusEffectKind::SubArmor(2),
                    timer: None,
                }),
                _ => None,
            };

            let damage: u32 = tower_stats
                .damage
                .saturating_add(status_effects.get_total_add_damage());

            bullet::spawn(
                bullet_translation,
                enemy,
                damage,
                100.0,
                status,
                &mut commands,
                material,
            );
        }
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

fn update_tower_appearance(
    mut commands: Commands,
    sprite_query: Query<Entity, With<TowerSprite>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tower_query: Query<(Entity, &TowerStats, &TowerType, &Children), Changed<TowerStats>>,
    texture_handles: Res<TextureHandles>,
    textures: Res<Assets<Texture>>,
) {
    for (parent, stats, tower_type, children) in tower_query.iter_mut() {
        for child in children.iter() {
            if let Ok(ent) = sprite_query.get(*child) {
                commands.entity(ent).despawn();
            }
        }

        let texture_handle = match (tower_type, stats.level) {
            (TowerType::Basic, 1) => Some(texture_handles.tower.clone()),
            (TowerType::Basic, 2) => Some(texture_handles.tower_two.clone()),
            (TowerType::Support, 1) => Some(texture_handles.support_tower.clone()),
            (TowerType::Support, 2) => Some(texture_handles.support_tower_two.clone()),
            (TowerType::Debuff, 1) => Some(texture_handles.debuff_tower.clone()),
            (TowerType::Debuff, 2) => Some(texture_handles.debuff_tower_two.clone()),
            _ => None,
        };

        if let Some(texture_handle) = texture_handle {
            let texture = textures.get(texture_handle.clone()).unwrap();

            let new_child = commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(texture_handle.clone().into()),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        (texture.size.height / 2) as f32 - 16.0,
                        layer::TOWER,
                    )),
                    ..Default::default()
                })
                .insert(TowerSprite)
                .id();

            commands.entity(parent).push_children(&[new_child]);
        }
    }
}

// This only needs to run when TowerSelection is mutated or
// when TowerStats changes. It doesn't seem possible to accomplish
// that with bevy right now though. Keep an eye on Bevy #1313
fn update_range_indicator(
    selection: Res<TowerSelection>,
    mut query: Query<(&mut Transform, &mut Visible), (With<RangeIndicator>, Without<TowerStats>)>,
    tower_query: Query<(&Transform, &TowerStats), (With<TowerStats>, Without<RangeIndicator>)>,
) {
    if let Some(slot) = selection.selected {
        if let Ok((tower_t, stats)) = tower_query.get(slot) {
            if let Some((mut t, mut v)) = query.iter_mut().next() {
                t.translation.x = tower_t.translation.x;
                t.translation.y = tower_t.translation.y;

                // range is a radius, sprite width is diameter
                t.scale.x = stats.range * 2.0 / 722.0; // XXX magic sprite scaling factor
                t.scale.y = stats.range * 2.0 / 722.0; // XXX magic sprite scaling factor

                v.is_visible = true;
            }
        } else if let Some((_, mut v)) = query.iter_mut().next() {
            v.is_visible = false;
        }
    } else if let Some((_, mut v)) = query.iter_mut().next() {
        v.is_visible = false;
    }
}

#[allow(clippy::too_many_arguments)]
fn show_game_over(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    currency: Res<Currency>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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

    // Pretty sure this draws under the UI, so we'll just carefully avoid UI stuff.
    // A previous version of this used the UI, but it was causing JUST THE BACKGROUND
    // of the action pane to disappear.

    commands.spawn_bundle(SpriteBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, layer::OVERLAY_BG)),
        material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.7).into()),
        sprite: Sprite::new(Vec2::new(128.0, 74.0)),
        ..Default::default()
    });

    commands.spawn_bundle(Text2dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, layer::OVERLAY)),
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
    });
}

fn startup_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.7).into()),
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
                material: materials.add(texture_handles.coin_ui.clone().into()),
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
                material: materials.add(texture_handles.timer_ui.clone().into()),
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
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.7).into()),
            ..Default::default()
        })
        .insert(TypingTargetContainer)
        .id();

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, layer::RETICLE)),
            material: materials.add(texture_handles.reticle.clone().into()),
            visible: Visible {
                is_visible: false,
                is_transparent: true,
            },
            ..Default::default()
        })
        .insert(Reticle);

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, layer::RANGE_INDICATOR)),
            material: materials.add(texture_handles.range_indicator.clone().into()),
            visible: Visible {
                is_visible: false,
                is_transparent: true,
            },
            ..Default::default()
        })
        .insert(RangeIndicator);

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
        action: Action::BuildTower(TowerType::Basic),
        visible: false,
        disabled: false,
    });
    actions.push(ActionPanelItem {
        icon: texture_handles.support_tower_ui.clone(),
        target: typing_targets.pop_front(),
        action: Action::BuildTower(TowerType::Support),
        visible: false,
        disabled: false,
    });
    actions.push(ActionPanelItem {
        icon: texture_handles.debuff_tower_ui.clone(),
        target: typing_targets.pop_front(),
        action: Action::BuildTower(TowerType::Debuff),
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
        icon: texture_handles.sell_ui.clone(),
        target: typing_targets.pop_front(),
        action: Action::SellTower,
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
                &mut commands,
                &font_handles,
                &texture_handles,
                &mut materials,
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

#[allow(clippy::type_complexity)]
fn update_tower_slot_labels(
    mut bg_query: Query<&mut Sprite, With<TowerSlotLabelBg>>,
    query: Query<(&Text2dSize, &Parent), (With<TowerSlotLabel>, Changed<Text2dSize>)>,
) {
    for (size, parent) in query.iter() {
        if let Ok(mut bg_sprite) = bg_query.get_mut(**parent) {
            bg_sprite.size.x = size.size.width + 8.0;
        }
    }
}

fn start_game(mut game_state: ResMut<GameState>) {
    game_state.ready = true;
}

#[allow(clippy::too_many_arguments)]
fn spawn_map_objects(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut typing_targets: ResMut<TypingTargets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut waves: ResMut<Waves>,
    texture_handles: Res<TextureHandles>,
    font_handles: Res<FontHandles>,
    maps_query: Query<(&TiledMapCenter, &Handle<Map>)>,
    maps: Res<Assets<Map>>,
) {
    let (centered, map_handle) = maps_query
        .single()
        .expect("We can only do exactly one map at a time.");

    let map = match maps.get(map_handle) {
        Some(map) => map,
        None => panic!("Queried map not in assets?"),
    };

    for grp in map.groups.iter() {
        let mut tower_slots = grp
            .objects
            .iter()
            .filter(|o| o.obj_type == "tower_slot")
            .filter(|o| o.props.contains_key("index"))
            .filter_map(|o| match o.props.get(&"index".to_string()) {
                Some(PropertyValue::IntValue(index)) => Some((o, index)),
                _ => None,
            })
            .collect::<Vec<(&Object, &i32)>>();

        tower_slots.sort_by(|a, b| a.1.cmp(b.1));

        for (obj, _index) in tower_slots {
            // TODO can we use bevy_tiled_prototype::Object.transform_from_map for
            // this? I tried once, and things seemed way off.

            let transform = util::map_to_world(&map, obj.position, obj.size, 0.0, centered.0);

            let mut label_bg_transform = transform.clone();
            label_bg_transform.translation.y -= 32.0;
            label_bg_transform.translation.z = layer::TOWER_SLOT_LABEL_BG;

            // TODO we might be able to use ObjectReadyEvent for tower slots, but
            // it's a bit awkward because we're adding the graphic as a child to
            // make it easier to swap graphics.

            let tower = commands
                .spawn_bundle((transform, GlobalTransform::default()))
                .insert(TowerSlot)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(SpriteBundle {
                            material: materials.add(texture_handles.tower_slot.clone().into()),
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
                    material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.7).into()),
                    sprite: Sprite::new(Vec2::new(108.0, FONT_SIZE_LABEL)),
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

    for grp in map.groups.iter() {
        if let Some((transform, size, hp)) = grp
            .objects
            .iter()
            .filter(|o| o.obj_type == "goal")
            .map(|o| {
                let hp = match o.props.get(&"hp".to_string()) {
                    Some(PropertyValue::IntValue(hp)) => *hp as u32,
                    _ => 10,
                };

                let transform =
                    util::map_to_world(&map, o.position, o.size, layer::ENEMY, centered.0);

                (transform, o.size, hp)
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
                &mut materials,
            );
        }
    }

    let paths: HashMap<i32, Vec<Vec2>> = map
        .groups
        .iter()
        .flat_map(|grp| grp.objects.iter())
        .filter(|o| o.obj_type == "enemy_path")
        .filter_map(|o| match (&o.shape, o.props.get(&"index".to_string())) {
            (ObjectShape::Polyline { points }, Some(PropertyValue::IntValue(index))) => {
                Some((o, points, index))
            }
            (ObjectShape::Polygon { points }, Some(PropertyValue::IntValue(index))) => {
                Some((o, points, index))
            }
            _ => None,
        })
        .map(|(obj, points, index)| {
            let transformed: Vec<Vec2> = points
                .iter()
                .map(|(x, y)| {
                    let transform = util::map_to_world(
                        &map,
                        Vec2::new(*x, *y) + obj.position,
                        Vec2::new(0.0, 0.0),
                        0.0,
                        centered.0,
                    );
                    transform.translation.truncate()
                })
                .collect();

            (*index, transformed)
        })
        .collect();

    let mut map_waves = map
        .groups
        .iter()
        .flat_map(|grp| grp.objects.iter())
        .filter(|o| o.obj_type == "wave")
        .filter(|o| o.props.contains_key("index"))
        .filter_map(|o| match o.props.get(&"index".to_string()) {
            Some(PropertyValue::IntValue(index)) => Some((o, *index)),
            _ => None,
        })
        .collect::<Vec<(&Object, i32)>>();

    map_waves.sort_by(|a, b| a.1.cmp(&b.1));

    for (map_wave, _) in map_waves {
        let enemy = match map_wave.props.get(&"enemy".to_string()) {
            Some(PropertyValue::StringValue(v)) => v.to_string(),
            _ => continue,
        };

        let num = match map_wave.props.get(&"num".to_string()) {
            Some(PropertyValue::IntValue(v)) => *v as usize,
            _ => continue,
        };

        let delay = match map_wave.props.get(&"delay".to_string()) {
            Some(PropertyValue::FloatValue(v)) => *v,
            _ => continue,
        };

        let interval = match map_wave.props.get(&"interval".to_string()) {
            Some(PropertyValue::FloatValue(v)) => *v,
            _ => continue,
        };

        let hp = match map_wave.props.get(&"hp".to_string()) {
            Some(PropertyValue::IntValue(v)) => *v as u32,
            _ => continue,
        };

        let armor = match map_wave.props.get(&"armor".to_string()) {
            Some(PropertyValue::IntValue(v)) => *v as u32,
            _ => continue,
        };

        let speed = match map_wave.props.get(&"speed".to_string()) {
            Some(PropertyValue::FloatValue(v)) => *v,
            _ => continue,
        };

        let path_index = match map_wave.props.get(&"path_index".to_string()) {
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
        })
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
        .add_state(TaipoState::Preload)
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_webgl2::WebGL2Plugin)
        .add_plugin(bevy_tiled_prototype::TiledMapPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(GameDataPlugin)
        .add_plugin(TypingPlugin)
        .add_plugin(MainMenuPlugin)
        // also, AppState::MainMenu from MainMenuPlugin
        .add_plugin(LoadingPlugin)
        // also, AppState::Preload from LoadingPlugin
        // also, AppState::Load from LoadingPlugin
        .add_event::<TowerChangedEvent>()
        .add_system_set(
            SystemSet::on_enter(TaipoState::Spawn)
                .with_system(spawn_map_objects.system())
                .with_system(startup_system.system()),
        )
        .add_system_set(
            SystemSet::on_update(TaipoState::Spawn)
                .with_system(check_spawn.system())
                .with_system(update_action_panel.system()),
        )
        .add_system_set(SystemSet::on_enter(TaipoState::Ready).with_system(start_game.system()))
        .add_stage_after(
            CoreStage::Update,
            TaipoStage::AfterUpdate,
            SystemStage::parallel(),
        )
        .add_stage_after(
            CoreStage::PostUpdate,
            TaipoStage::AfterPostUpdate,
            SystemStage::parallel(),
        )
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
        .add_system(shoot_enemies.system())
        .add_system(animate_reticle.system())
        .add_system(update_timer_display.system())
        .add_system(
            typing_target_finished_event
                .system()
                .label("typing_target_finished_event"),
        )
        .add_system(
            update_tower_status_effects
                .system()
                .label("update_tower_status_effects")
                .before("typing_target_finished_event"),
        )
        .add_system(
            update_currency_text
                .system()
                .label("update_currency_text")
                .after("typing_target_finished_event"),
        )
        .add_system(spawn_enemies.system().label("spawn_enemies"))
        .add_system(show_game_over.system().after("spawn_enemies"))
        // update_actions_panel and update_range_indicator need to be aware of TowerStats components
        // that get queued to spawn in the update stage.)
        .add_system_to_stage(TaipoStage::AfterUpdate, update_action_panel.system())
        .add_system_to_stage(TaipoStage::AfterUpdate, update_range_indicator.system())
        // update_tower_appearance needs to detect added TowerStats components
        .add_system_to_stage(TaipoStage::AfterUpdate, update_tower_appearance.system())
        // update_tower_status_effect_appearance needs to detect an added or modified StatusEffects
        // component, so it must run in a later stage.
        .add_system_to_stage(
            TaipoStage::AfterUpdate,
            update_tower_status_effect_appearance.system(),
        )
        // update_tower_slot_labels uses Changed<CalculatedSize> which only works if we run after
        // POST_UPDATE.
        .add_system_to_stage(
            TaipoStage::AfterPostUpdate,
            update_tower_slot_labels.system(),
        )
        .run();
}
