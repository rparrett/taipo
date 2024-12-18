// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use action_panel::{ActionPanel, ActionPanelItemImage, ActionPanelPlugin};
use atlas_loader::{AtlasImage, AtlasImageLoader};
use bevy::{
    app::MainScheduleOrder,
    asset::AssetMetaCheck,
    ecs::schedule::ScheduleLabel,
    prelude::*,
    text::{update_text2d_layout, TextLayoutInfo},
    utils::HashMap,
};

use bevy_ecs_tilemap::TilemapPlugin;
use tiled::{ObjectShape, PropertyValue};

use crate::{
    bullet::BulletPlugin,
    data::{AnimationData, GameData, GameDataPlugin},
    enemy::EnemyPlugin,
    game_over::GameOverPlugin,
    healthbar::{HealthBar, HealthBarPlugin},
    loading::{FontHandles, LevelHandles, LoadingPlugin, TextureHandles, UiTextureHandles},
    main_menu::MainMenuPlugin,
    map::{find_objects, get_int_property, map_to_world, TiledMap, TiledMapPlugin},
    reticle::ReticlePlugin,
    tower::{
        TowerBundle, TowerChangedEvent, TowerKind, TowerPlugin, TowerSprite, TowerStats,
        TOWER_PRICE,
    },
    typing::{
        AsciiModeEvent, TypingPlugin, TypingTarget, TypingTargetBundle, TypingTargetFinishedEvent,
        TypingTargetSettings, TypingTargetText, TypingTargets,
    },
    wave::{Wave, WavePlugin, WaveState, Waves},
};

extern crate anyhow;

mod action_panel;
mod atlas_loader;
mod bullet;
mod data;
mod enemy;
mod game_over;
mod healthbar;
mod japanese_parser;
mod layer;
mod loading;
mod main_menu;
mod map;
mod reticle;
mod tower;
mod typing;
mod ui_color;
mod wave;

pub static FONT_SIZE: f32 = 22.0;
pub static FONT_SIZE_INPUT: f32 = 22.0;
pub static FONT_SIZE_LABEL: f32 = 16.0;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
struct AfterUpdate;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum TaipoState {
    #[default]
    Load,
    Spawn,
    MainMenu,
    Playing,
    GameOver,
}
#[derive(Resource, Default)]
pub struct GameState {
    // Just so we can keep these in the correct order
    tower_slots: Vec<Entity>,
}
#[derive(Resource)]
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

#[derive(Resource, Default)]
pub struct TowerSelection {
    selected: Option<Entity>,
}

#[derive(Clone, Component, Debug, Default)]
pub enum Action {
    #[default]
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

#[derive(Component)]
struct CurrencyDisplay;
#[derive(Component)]
struct DelayTimerDisplay;

#[derive(Component)]
struct Goal;

#[derive(Component)]
struct TowerSlot;
#[derive(Component)]
struct TowerSlotLabel;
#[derive(Component)]
struct TowerSlotLabelBg;
#[derive(Resource, Default)]
struct AudioSettings {
    mute: bool,
}
#[derive(Component)]
pub struct HitPoints {
    current: u32,
    max: u32,
}
impl Default for HitPoints {
    fn default() -> Self {
        Self::full(1)
    }
}
impl HitPoints {
    fn full(val: u32) -> Self {
        Self {
            current: val,
            max: val,
        }
    }
}
#[derive(Component)]
pub struct Speed(f32);
impl Default for Speed {
    fn default() -> Self {
        Self(20.0)
    }
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

fn typing_target_finished_event(
    mut commands: Commands,
    mut tower_state_query: Query<&mut TowerStats, With<TowerKind>>,
    tower_children_query: Query<&Children, With<TowerSlot>>,
    tower_sprite_query: Query<Entity, With<TowerSprite>>,
    action_query: Query<&Action>,
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
    for event in reader.read() {
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
                    commands.entity(tower).insert(TowerBundle::new(tower_kind));

                    tower_changed_events.send(TowerChangedEvent);
                }
            } else if let Action::SellTower = *action {
                if let Some(tower) = selection.selected {
                    commands.entity(tower).remove::<TowerBundle>();

                    if let Ok(children) = tower_children_query.get(tower) {
                        for child in children.iter() {
                            if let Ok(ent) = tower_sprite_query.get(*child) {
                                commands.entity(ent).despawn();

                                let new_child = commands
                                    .spawn((
                                        Sprite {
                                            image: texture_handles.tower_slot.clone(),
                                            ..default()
                                        },
                                        Transform::from_translation(Vec3::new(
                                            0.0,
                                            0.0,
                                            layer::TOWER_SLOT,
                                        )),
                                        TowerSprite,
                                    ))
                                    .id();

                                commands.entity(tower).add_child(new_child);
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

        // Any action except for toggling ascii "help" mode should disable ascii mode.
        if !toggled_ascii_mode {
            toggle_events.send(AsciiModeEvent::Disable);
        }
    }
}

fn update_timer_display(
    mut query: Query<&mut Text, With<DelayTimerDisplay>>,
    wave_state: Res<WaveState>,
) {
    if !wave_state.is_changed() {
        return;
    }

    for mut text in query.iter_mut() {
        text.0 = format!("{:.1}", wave_state.delay_timer.remaining_secs());
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
        target.0 = format!("{}", currency.current);
    }
}

fn startup_system(
    mut commands: Commands,
    ui_texture_handles: ResMut<UiTextureHandles>,
    font_handles: Res<FontHandles>,
    currency: Res<Currency>,
) {
    info!("startup");

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.),
                top: Val::Px(0.),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                height: Val::Px(42.0),
                ..default()
            },
            BackgroundColor(ui_color::TRANSPARENT_BACKGROUND.into()),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image: ui_texture_handles.coin_ui.clone(),
                    ..default()
                },
                Node {
                    margin: UiRect {
                        left: Val::Px(5.0),
                        ..default()
                    },
                    height: Val::Px(32.0),
                    ..default()
                },
            ));
            parent.spawn((
                Text::new(format!("{}", currency.current)),
                Node {
                    margin: UiRect {
                        left: Val::Px(5.0),
                        right: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                },
                TextFont {
                    font: font_handles.jptext.clone(),
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(ui_color::NORMAL_TEXT.into()),
                CurrencyDisplay,
            ));
            parent.spawn((
                ImageNode {
                    image: ui_texture_handles.timer_ui.clone().into(),
                    ..default()
                },
                Node {
                    margin: UiRect {
                        left: Val::Px(5.0),
                        ..default()
                    },
                    height: Val::Px(32.0),
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("30"),
                Node {
                    margin: UiRect {
                        left: Val::Px(5.0),
                        right: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                },
                TextFont {
                    font: font_handles.jptext.clone(),
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(ui_color::NORMAL_TEXT.into()),
                DelayTimerDisplay,
            ));
        });

    commands.spawn(TypingTargetBundle {
        target: TypingTarget::new("help"),
        settings: TypingTargetSettings {
            fixed: true,
            disabled: false,
        },
        action: Action::SwitchLanguageMode,
    });

    commands.spawn(TypingTargetBundle {
        target: TypingTarget::new("mute"),
        settings: TypingTargetSettings {
            fixed: true,
            disabled: false,
        },
        action: Action::ToggleMute,
    });
}

fn update_tower_slot_labels(
    mut bg_query: Query<&mut Sprite, With<TowerSlotLabelBg>>,
    query: Query<(&TextLayoutInfo, &Parent), (With<TowerSlotLabel>, Changed<TextLayoutInfo>)>,
) {
    for (info, parent) in query.iter() {
        if let Ok(mut bg_sprite) = bg_query.get_mut(**parent) {
            if let Some(bg_sprite_size) = bg_sprite.custom_size {
                bg_sprite.custom_size = Some(Vec2::new(info.size.x + 8.0, bg_sprite_size.y));
            }
        }
    }
}

fn spawn_map_objects(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut typing_targets: ResMut<TypingTargets>,
    mut waves: ResMut<Waves>,
    level_handles: Res<LevelHandles>,
    font_handles: Res<FontHandles>,
    texture_handles: Res<TextureHandles>,
    maps: Res<Assets<TiledMap>>,
) {
    let Some(tiled_map) = maps.get(&level_handles.one) else {
        panic!("Queried map not in assets?");
    };

    info!("spawn_map_objects");

    // paths

    let paths: HashMap<i32, Vec<Vec2>> = find_objects(tiled_map, "enemy_path")
        .filter_map(|o| {
            let Some(PropertyValue::IntValue(index)) = o.properties.get("index") else {
                return None;
            };

            let (ObjectShape::Polyline { points } | ObjectShape::Polygon { points }) = &o.shape
            else {
                return None;
            };

            let transformed: Vec<Vec2> = points
                .iter()
                .map(|(x, y)| {
                    let transform = map_to_world(
                        tiled_map,
                        Vec2::new(*x, *y) + Vec2::new(o.x, o.y),
                        Vec2::ZERO,
                        0.0,
                    );
                    transform.translation.truncate()
                })
                .collect();

            Some((*index, transformed))
        })
        .collect();

    // waves

    let mut map_waves = find_objects(tiled_map, "wave").collect::<Vec<_>>();

    map_waves.sort_by(|a, b| a.x.partial_cmp(&b.x).expect("sorting waves"));

    for map_wave in map_waves.iter() {
        let Ok(wave) = Wave::new(map_wave, &paths) else {
            warn!("skipped invalid wave object");
            continue;
        };

        waves.waves.push(wave);
    }

    commands.insert_resource(WaveState::from(waves.current().unwrap()));

    // goal

    find_objects(tiled_map, "goal").for_each(|o| {
        let hp = match get_int_property(&o, "hp") {
            Ok(hp) => hp as u32,
            Err(err) => {
                warn!("goal: {}", err);
                10
            }
        };

        let pos = Vec2::new(o.x, o.y);
        let size = match o.shape {
            ObjectShape::Rect { width, height } => Vec2::new(width, height),
            _ => {
                warn!("goal is not a rectangle");
                return;
            }
        };

        let transform = map_to_world(tiled_map, pos, size, layer::ENEMY);

        commands.spawn((
            Goal,
            // TODO does this actually need a Sprite?
            Sprite::default(),
            transform,
            HitPoints::full(hp),
            HealthBar {
                size,
                show_full: true,
                show_empty: true,
                ..default()
            },
        ));
    });

    // tower slots

    let mut tower_slots = find_objects(tiled_map, "tower_slot")
        .filter_map(|o| match get_int_property(&o, "index") {
            Ok(index) => Some((o, index)),
            Err(err) => {
                warn!("tower_slot: {}", err);
                None
            }
        })
        .collect::<Vec<_>>();

    tower_slots.sort_by(|a, b| a.1.cmp(&b.1));

    for (obj, _index) in tower_slots {
        let pos = Vec2::new(obj.x, obj.y);
        let size = match obj.shape {
            ObjectShape::Rect { width, height } => Vec2::new(width, height),
            _ => continue,
        };

        let transform = map_to_world(tiled_map, pos, size, 0.0);

        let mut label_bg_transform = transform;
        label_bg_transform.translation.y -= 32.0;
        label_bg_transform.translation.z = layer::TOWER_SLOT_LABEL_BG;

        let tower = commands
            .spawn((TowerSlot, transform))
            .with_children(|parent| {
                parent.spawn((
                    Sprite {
                        image: texture_handles.tower_slot.clone(),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, layer::TOWER_SLOT),
                    TowerSprite,
                ));
            })
            .id();

        game_state.tower_slots.push(tower);

        let target = typing_targets.pop_front();

        commands
            .spawn((
                Sprite {
                    color: ui_color::TRANSPARENT_BACKGROUND.into(),
                    custom_size: Some(Vec2::new(108.0, FONT_SIZE_LABEL + 8.0)),
                    ..default()
                },
                label_bg_transform,
                TowerSlotLabelBg,
                TypingTargetBundle {
                    target: target.clone(),
                    action: Action::SelectTower(tower),
                    settings: TypingTargetSettings::default(),
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Text2d::new(""),
                        TextFont {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE_LABEL,
                            ..default()
                        },
                        TextColor(ui_color::GOOD_TEXT.into()),
                        Transform::from_xyz(0.0, 0.0, 0.1),
                        TypingTargetText,
                        TowerSlotLabel,
                    ))
                    .with_child((
                        TextSpan::new(target.displayed_chunks.join("")),
                        TextFont {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE_LABEL,
                            ..default()
                        },
                        TextColor(ui_color::NORMAL_TEXT.into()),
                    ));
            });
    }
}

fn check_spawn(
    mut next_state: ResMut<NextState<TaipoState>>,
    mut actions: ResMut<ActionPanel>,
    typing_targets: Query<Entity, With<ActionPanelItemImage>>,
    waves: Res<Waves>,
) {
    // this whole phase is probably not actually doing anything, but it does serve as a
    // single place to put advance to the ready state from

    // typing targets are probably the last thing to spawn because they're spawned by an event
    // so maybe the game is ready if they are present.

    if typing_targets.is_empty() {
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

    next_state.set(TaipoState::Playing);
}

fn main() {
    let mut app = App::new();

    let mut order = app.world_mut().resource_mut::<MainScheduleOrder>();
    order.insert_after(Update, AfterUpdate);

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: [720., 480.].into(),
                    canvas: Some("#bevy-canvas".to_string()),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                // Workaround for Bevy attempting to load .meta files in wasm builds. On itch,
                // the CDN serves HTTP 403 errors instead of 404 when files don't exist, which
                // causes Bevy to break.
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    );

    app.init_state::<TaipoState>();

    app.init_asset::<AtlasImage>()
        .register_asset_loader(AtlasImageLoader);

    app.add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(GameDataPlugin)
        .add_plugins(TypingPlugin)
        .add_plugins(MainMenuPlugin)
        .add_plugins(LoadingPlugin)
        .add_plugins(TowerPlugin)
        .add_plugins(HealthBarPlugin)
        .add_plugins(BulletPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(WavePlugin)
        .add_plugins(ReticlePlugin)
        .add_plugins(GameOverPlugin)
        .add_plugins(ActionPanelPlugin);

    app.init_resource::<GameState>()
        .init_resource::<Currency>()
        .init_resource::<TowerSelection>()
        .init_resource::<AudioSettings>();

    app.add_event::<TowerChangedEvent>();

    app.add_systems(
        OnEnter(TaipoState::Spawn),
        (spawn_map_objects, startup_system),
    );

    app.add_systems(Update, check_spawn.run_if(in_state(TaipoState::Spawn)));

    app.add_systems(
        Update,
        (
            update_timer_display,
            typing_target_finished_event,
            update_currency_text.after(typing_target_finished_event),
        )
            .run_if(in_state(TaipoState::Playing)),
    );

    // `update_tower_slot_labels` uses `Changed<CalculatedSize>` which only works if we run in
    // after Bevy's `update_text2d_layout`.
    app.add_systems(
        PostUpdate,
        update_tower_slot_labels
            .after(update_text2d_layout)
            .run_if(in_state(TaipoState::Playing)),
    );

    app.run();
}
