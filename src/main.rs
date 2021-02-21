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
    AsciiModeEvent, TypingPlugin, TypingTarget, TypingTargetContainer, TypingTargetFinishedEvent,
    TypingTargetImage, TypingTargetPriceContainer, TypingTargetPriceImage, TypingTargetPriceText,
    TypingTargetText, TypingTargets,
};
use util::set_visible_recursive;

#[macro_use]
extern crate anyhow;

mod bullet;
mod data;
mod enemy;
mod healthbar;
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
    State,
    AfterState,
    AfterUpdate,
    AfterPostUpdate,
}

#[derive(Clone)]
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

fn update_action_panel(
    actions: ChangedRes<ActionPanel>,
    mut typing_target_query: Query<&mut TypingTarget>,
    mut visible_query: Query<&mut Visible>,
    mut style_query: Query<&mut Style>,
    mut text_query: Query<&mut Text, With<TypingTargetText>>,
    mut price_text_query: Query<&mut Text, With<TypingTargetPriceText>>,
    target_children_query: Query<&Children, With<TypingTarget>>,
    children_query: Query<&Children>,
    tower_query: Query<(&TowerState, &TowerType, &TowerStats)>,
    price_query: Query<(Entity, &Children), With<TypingTargetPriceContainer>>,
    currency: Res<Currency>,
    selection: Res<TowerSelection>,
) {
    info!("update actions");

    for (item, entity) in actions.actions.iter().zip(actions.entities.iter()) {
        let visible = match item.action {
            Action::BuildBasicTower => match selection.selected {
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
            _ => false,
        };

        let price = match item.action {
            Action::BuildBasicTower => TOWER_PRICE,
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

fn typing_target_finished_event(
    mut reader: EventReader<TypingTargetFinishedEvent>,
    commands: &mut Commands,
    mut currency: ResMut<Currency>,
    mut selection: ResMut<TowerSelection>,
    mut toggle_events: ResMut<Events<AsciiModeEvent>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut action_panel: ResMut<ActionPanel>,
    mut sound_settings: ResMut<AudioSettings>,
    mut tower_state_query: Query<&mut TowerStats, With<TowerType>>,
    mut reticle_query: Query<(&mut Transform, &mut Visible), With<Reticle>>,
    action_query: Query<&Action>,
    tower_transform_query: Query<&Transform, With<TowerSlot>>,
    texture_handles: Res<TextureHandles>,
) {
    for event in reader.iter() {
        info!("typing_target_finished");

        let mut toggled_ascii_mode = false;

        for action in action_query.get(event.entity) {
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
                        }
                    }
                }

                action_panel.update += 1;
            } else if let Action::BuildBasicTower = *action {
                if currency.current < TOWER_PRICE {
                    continue;
                }
                currency.current -= TOWER_PRICE;

                if let Some(tower) = selection.selected {
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
                            transform: Transform::from_translation(Vec3::new(
                                0.0,
                                20.0, // XXX magic sprite offset
                                layer::TOWER,
                            )),
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

        if !toggled_ascii_mode {
            toggle_events.send(AsciiModeEvent::Disable);
        }

        for (mut reticle_transform, mut reticle_visible) in reticle_query.iter_mut() {
            if let Some(tower) = selection.selected {
                for transform in tower_transform_query.get(tower) {
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

fn animate_reticle(
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite), With<Reticle>>,
    time: Res<Time>,
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
    mut waves: ResMut<Waves>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
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
            .spawn(SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(point.x, point.y, layer::ENEMY)),
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
    mut query: Query<&mut Text, With<DelayTimerDisplay>>,
    mut timer: ResMut<DelayTimerTimer>,
    time: Res<Time>,
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
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tower_query: Query<(&Transform, &mut TowerState, &TowerStats, &TowerType)>,
    enemy_query: Query<(Entity, &HitPoints, &Transform), With<EnemyState>>,
    texture_handles: Res<TextureHandles>,
    time: Res<Time>,
) {
    for (transform, mut tower_state, tower_stats, _tower_type) in tower_query.iter_mut() {
        tower_state.timer.tick(time.delta_seconds());
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
            let mut bullet_translation = transform.translation.clone();
            bullet_translation.y += 24.0; // XXX magic sprite offset

            bullet::spawn(
                bullet_translation,
                enemy,
                tower_stats.damage,
                100.0,
                commands,
                &mut materials,
                &texture_handles,
            );
        }
    }
}

fn update_currency_text(
    currency: ChangedRes<Currency>,
    mut currency_display_query: Query<&mut Text, With<CurrencyDisplay>>,
) {
    for mut target in currency_display_query.iter_mut() {
        target.sections[0].value = format!("{}", currency.current);
    }
}

fn update_tower_appearance(
    mut material_query: Query<&mut Handle<ColorMaterial>, With<TowerSprite>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tower_query: Query<(&TowerStats, &Children), Changed<TowerStats>>,
    texture_handles: Res<TextureHandles>,
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

// This only needs to run when TowerSelection is mutated or
// when TowerStats changes. It doesn't seem possible to accomplish
// that with bevy right now though. Keep an eye on Bevy #1313
fn update_range_indicator(
    selection: Res<TowerSelection>,
    mut query: Query<(&mut Transform, &mut Visible), With<RangeIndicator>>,
    tower_query: Query<(&Transform, &TowerStats), With<TowerStats>>,
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
        }
    } else {
        if let Some((_, mut v)) = query.iter_mut().next() {
            v.is_visible = false;
        }
    }
}

fn show_game_over(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    currency: Res<Currency>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<&EnemyState>,
    goal_query: Query<&HitPoints, With<Goal>>,
    waves: Res<Waves>,
    font_handles: Res<FontHandles>,
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
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, layer::OVERLAY_BG)),
        material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.7).into()),
        sprite: Sprite::new(Vec2::new(128.0, 74.0)),
        ..Default::default()
    });

    commands.spawn(Text2dBundle {
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
    texture_handles: ResMut<TextureHandles>,
    mut action_panel: ResMut<ActionPanel>,
    mut typing_targets: ResMut<TypingTargets>,
    font_handles: Res<FontHandles>,
    currency: Res<Currency>,
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
                        format!("{}", currency.current),
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

    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, layer::RETICLE)),
            sprite: TextureAtlasSprite {
                index: 0,
                ..Default::default()
            },
            texture_atlas: texture_handles.reticle_atlas.clone(),
            visible: Visible {
                is_visible: false,
                is_transparent: true,
            },
            ..Default::default()
        })
        .with(Timer::from_seconds(0.01, true))
        .with(Reticle);

    commands
        .spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, layer::RANGE_INDICATOR)),
            material: materials.add(texture_handles.range_indicator.clone().into()),
            visible: Visible {
                is_visible: false,
                is_transparent: true,
            },
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
    mut bg_query: Query<&mut Sprite, With<TowerSlotLabelBg>>,
    query: Query<(&CalculatedSize, &Parent), (With<TowerSlotLabel>, Changed<CalculatedSize>)>,
) {
    for (size, parent) in query.iter() {
        if let Ok(mut bg_sprite) = bg_query.get_mut(**parent) {
            bg_sprite.size.x = size.size.width + 8.0;
        }
    }
}

fn init_audio(commands: &mut Commands) {
    info!("init_audio");
    commands.insert_resource(AudioInitialization {
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
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut waves: ResMut<Waves>,
    texture_handles: Res<TextureHandles>,
    font_handles: Res<FontHandles>,
    maps: Res<Assets<bevy_tiled_prototype::Map>>,
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
                            layer::TOWER_SLOT_LABEL_BG,
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
                                transform: Transform::from_translation(Vec3::new(
                                    0.0,
                                    0.0,
                                    layer::TOWER_SLOT_LABEL,
                                )),
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
                        transform: Transform::from_translation(pos.extend(layer::ENEMY)),
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

    if waves.waves.len() < 1 {
        return;
    }

    // We need to force the action panel to update now that it has spawned
    // because we didn't bother initializing it properly. Surprisingly this seems to work
    // every time, but we should probably be on the lookout for actions not getting
    // initialized

    actions.update += 1;

    state.set_next(TaipoState::Ready).unwrap();
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
        .insert_resource(State::new(TaipoState::Preload))
        .add_stage_after(
            CoreStage::Update,
            TaipoStage::State,
            StateStage::<TaipoState>::default(),
        )
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_webgl2::WebGL2Plugin)
        .add_plugin(bevy_tiled_prototype::TiledMapPlugin)
        .insert_resource(AudioInitialization {
            needed: false,
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
        .on_state_enter(
            TaipoStage::State,
            TaipoState::Spawn,
            spawn_map_objects.system(),
        )
        .on_state_enter(
            TaipoStage::State,
            TaipoState::Spawn,
            startup_system.system(),
        )
        .on_state_update(TaipoStage::State, TaipoState::Spawn, check_spawn.system())
        .on_state_update(
            TaipoStage::State,
            TaipoState::Spawn,
            update_action_panel.system(),
        )
        .on_state_enter(TaipoStage::State, TaipoState::Ready, start_game.system())
        .on_state_enter(TaipoStage::State, TaipoState::Ready, init_audio.system())
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
        .add_stage_after(
            TaipoStage::State,
            TaipoStage::AfterState,
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
            update_tower_appearance
                .system()
                .after("typing_target_finished_event"),
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
        // that get queued to spawn in the update stage.
        .add_system_to_stage(TaipoStage::AfterUpdate, update_action_panel.system())
        .add_system_to_stage(TaipoStage::AfterUpdate, update_range_indicator.system())
        // update_tower_slot_labels uses Changed<CalculatedSize> which only works if we run after
        // POST_UPDATE.
        .add_system_to_stage(
            TaipoStage::AfterPostUpdate,
            update_tower_slot_labels.system(),
        )
        .run();
}
