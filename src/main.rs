use bevy::prelude::*;
use rand::prelude::SliceRandom;
use typing::{
    TypingPlugin, TypingTarget, TypingTargetChangeEvent, TypingTargetContainer,
    TypingTargetFinishedEvent, TypingTargetSpawnEvent,
};

use std::collections::VecDeque;

#[macro_use]
extern crate anyhow;

mod data;
mod typing;

#[derive(Default)]
pub struct GameState {
    score: u32,
    selected: Option<Entity>,
    possible_typing_targets: VecDeque<TypingTarget>,
}

struct ScoreDisplay;

struct BackgroundTile;

struct TowerSlot;

struct Reticle;

struct UpdateActionsEvent;

#[derive(Clone)]
enum Action {
    SelectTower(Entity),
    GenerateMoney,
    Back,
}

fn update_actions(
    commands: &mut Commands,
    game_state: Res<GameState>,
    mut query: Query<(Entity, &mut Style), With<TypingTarget>>,
    container_query: Query<&Children, With<TypingTargetContainer>>,
    tower_query: Query<Entity, With<TowerSlot>>,
    events: Res<Events<UpdateActionsEvent>>,
    mut reader: Local<EventReader<UpdateActionsEvent>>,
) {
    for _ in reader.iter(&events) {
        info!("processing UpdateActionsEvent");

        let mut other = vec![];

        if game_state.selected.is_some() {
            other.push(Action::Back);
        } else {
            other.push(Action::GenerateMoney);
        }

        let other_iter = other.iter().cloned();

        let mut action_iter = other_iter.chain(
            tower_query
                .iter()
                .filter(|_| game_state.selected.is_none())
                .map(|e| Action::SelectTower(e.clone())),
        );

        for children in container_query.iter() {
            for child in children.iter() {
                for (entity, mut style) in query.get_mut(*child) {
                    let mut visible = false;
                    commands.remove_one::<Action>(entity);

                    if let Some(action) = action_iter.next() {
                        visible = true;
                        commands.insert_one(entity, action.clone());
                    }

                    if visible {
                        style.display = Display::Flex;
                    } else {
                        style.display = Display::None;
                    }
                }
            }
        }
    }
}

fn typing_target_finished(
    mut game_state: ResMut<GameState>,
    mut reader: Local<EventReader<TypingTargetFinishedEvent>>,
    typing_target_finished_events: Res<Events<TypingTargetFinishedEvent>>,
    mut typing_target_change_events: ResMut<Events<TypingTargetChangeEvent>>,
    mut update_actions_events: ResMut<Events<UpdateActionsEvent>>,
    mut score_display_query: Query<&mut Text, With<ScoreDisplay>>,
    action_query: Query<&Action>,
    mut reticle_query: Query<&mut Transform, With<Reticle>>,
    tower_transform_query: Query<&Transform, With<TowerSlot>>,
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
                game_state.score += 1;
            } else if let Action::SelectTower(tower) = *action {
                info!("processing a SelectTower action");
                game_state.selected = Some(tower);
            } else if let Action::Back = *action {
                info!("processing a Back action");
                game_state.selected = None;
            }
        }

        for mut target in score_display_query.iter_mut() {
            target.value = format!("{}", game_state.score);
        }

        for mut reticle_transform in reticle_query.iter_mut() {
            if let Some(tower) = game_state.selected {
                for transform in tower_transform_query.get(tower) {
                    reticle_transform.translation.x = transform.translation.x;
                    reticle_transform.translation.y = transform.translation.y;
                }
            } else {
                info!("hiding reticle");
                reticle_transform.translation.x = -3200.0;
                reticle_transform.translation.y = -3200.0;
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
            if sprite.index >= 63 {
                sprite.index = 48;
            }
        }
    }
}

fn startup_system(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
    mut typing_target_spawn_events: ResMut<Events<TypingTargetSpawnEvent>>,
    mut update_actions_events: ResMut<Events<UpdateActionsEvent>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    info!("startup");

    // Would prefer to reuse an rng. Can we do that?
    let mut rng = rand::thread_rng();

    let font = asset_server.load("fonts/Koruri-Regular.ttf");

    let texture_handle = asset_server.load("textures/main.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 16, 16);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        // 2d camera
        .spawn(CameraUiBundle::default())
        .spawn(Camera2dBundle::default());

    commands
        .spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                value: format!("{}", game_state.score),
                font: font.clone(),
                style: TextStyle {
                    font_size: 60.0,
                    color: Color::WHITE,
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(ScoreDisplay);

    let grass_indices = vec![32, 33, 34, 35, 36, 37, 38];
    for x in 0..32 {
        for y in 0..32 {
            commands
                .spawn(SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: *grass_indices.choose(&mut rng).unwrap(),
                        ..Default::default()
                    },
                    texture_atlas: texture_atlas_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        -32.0 * 16.0 + 32.0 * (x as f32),
                        -32.0 * 16.0 + 32.0 * (y as f32),
                        0.0,
                    )),
                    ..Default::default()
                })
                .with(BackgroundTile);
        }
    }

    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(-32.0, -64.0, 0.0)),
            sprite: TextureAtlasSprite {
                index: 21,
                ..Default::default()
            },
            texture_atlas: texture_atlas_handle.clone(),
            ..Default::default()
        })
        .with(TowerSlot);

    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(-64.0, 96.0, 0.0)),
            sprite: TextureAtlasSprite {
                index: 20,
                ..Default::default()
            },
            texture_atlas: texture_atlas_handle.clone(),
            ..Default::default()
        })
        .with(TowerSlot);

    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(96.0, 128.0, 0.0)),
            sprite: TextureAtlasSprite {
                index: 19,
                ..Default::default()
            },
            texture_atlas: texture_atlas_handle.clone(),
            ..Default::default()
        })
        .with(TowerSlot);

    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(-160.0, -128.0, 0.0)),
            sprite: TextureAtlasSprite {
                index: 18,
                ..Default::default()
            },
            texture_atlas: texture_atlas_handle.clone(),
            ..Default::default()
        })
        .with(TowerSlot);

    // I don't know how to make the reticle invisible so I will just put out somewhere out
    // of view
    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(-3200.0, -3200.0, 0.0)),
            sprite: TextureAtlasSprite {
                index: 48,
                ..Default::default()
            },
            texture_atlas: texture_atlas_handle.clone(),
            ..Default::default()
        })
        .with(Timer::from_seconds(0.01, true))
        .with(Reticle);

    // TODO: load this from a file
    let mut possible_typing_targets = data::parse_typing_targets(
        "ひ(hi)ら(ra)が(ga)な(na)
        カ(ka)タ(ta)カ(ka)ナ(na)
        1(juu)1(ichi):00(ji)
        大(oo)き(ki)い(i)
        大(dai)学(gaku)生(sei)
        あ(a)か(ka)い(i)ボ(bo)ー(-)ル(ru)
        ミ(mi)ル(ru)ク(ku)コ(ko)ー(-)ヒ(hi)ー(-)
        メ(me)ロ(ro)ン(nn)ソ(so)ー(-)ダ(da)
        た(ta)ま(ma)ご(go)
        か(ka)さ(sa)
        と(to)う(u)き(k)ょ(yo)う(u)
        カ(ka)ラ(ra)オ(o)ケ(ke)
        サ(sa)ン(nn)ド(do)イ(i)ッ(c)チ(chi)
        タ(ta)ク(ku)シ(shi)ー(-)
        カ(ka)レ(re)ー(-)ラ(ra)イ(i)ス(su)
        100(hyaku)パ(pa)ー(-)セ(se)ン(nn)ト(to)
        フ(fu)ラ(ra)ン(nn)ス(su)",
    )
    .unwrap();
    possible_typing_targets.shuffle(&mut rng);
    game_state.possible_typing_targets = possible_typing_targets.into();

    for _ in 0..8 {
        let target = game_state
            .possible_typing_targets
            .pop_front()
            .unwrap()
            .clone();
        typing_target_spawn_events.send(TypingTargetSpawnEvent(target.clone(), None));
    }

    update_actions_events.send(UpdateActionsEvent);
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            width: 720.,
            height: 480.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_webgl2::WebGL2Plugin)
        .add_plugin(TypingPlugin)
        .add_startup_system(startup_system.system())
        .add_resource(GameState::default())
        .add_system(typing_target_finished.system())
        .add_system(animate_reticle.system())
        .add_system_to_stage(stage::LAST, update_actions.system()) // this just needs to happen after TypingTargetSpawnEvent
        .add_event::<UpdateActionsEvent>()
        .run();
}
