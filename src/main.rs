use bevy::prelude::*;
use rand::prelude::SliceRandom;
use typing::{TypingPlugin, TypingTarget, TypingTargetFinishedEvent, TypingTargetSpawnEvent};

#[macro_use]
extern crate anyhow;

mod data;
mod typing;

#[derive(Default)]
pub struct GameState {
    score: u32,
    possible_typing_targets: Vec<TypingTarget>,
}

struct ScoreDisplay;

struct BackgroundTile;

struct TowerSlot;

fn typing_target_finished(
    mut game_state: ResMut<GameState>,
    mut reader: Local<EventReader<TypingTargetFinishedEvent>>,
    typing_target_finished_events: Res<Events<TypingTargetFinishedEvent>>,
    mut typing_target_spawn_events: ResMut<Events<TypingTargetSpawnEvent>>,
    mut score_display_query: Query<&mut Text, With<ScoreDisplay>>,
) {
    for event in reader.iter(&typing_target_finished_events) {
        // Would prefer to reuse an rng. Can we do that?
        let mut rng = rand::thread_rng();
        let word = game_state.possible_typing_targets.choose(&mut rng).unwrap();

        typing_target_spawn_events.send(TypingTargetSpawnEvent(word.clone(), Some(event.entity)));

        game_state.score += 1;

        for mut target in score_display_query.iter_mut() {
            target.value = format!("{}", game_state.score);
        }
    }
}

fn startup_system(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
    mut typing_target_spawn_events: ResMut<Events<TypingTargetSpawnEvent>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
                index: 18,
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
                index: 19,
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
                index: 20,
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
                index: 21,
                ..Default::default()
            },
            texture_atlas: texture_atlas_handle.clone(),
            ..Default::default()
        })
        .with(TowerSlot);

    // TODO: load this from a file
    game_state.possible_typing_targets = data::parse_typing_targets(
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

    let word = game_state.possible_typing_targets.choose(&mut rng).unwrap();
    typing_target_spawn_events.send(TypingTargetSpawnEvent(word.clone(), None));
    let word = game_state.possible_typing_targets.choose(&mut rng).unwrap();
    typing_target_spawn_events.send(TypingTargetSpawnEvent(word.clone(), None));
    let word = game_state.possible_typing_targets.choose(&mut rng).unwrap();
    typing_target_spawn_events.send(TypingTargetSpawnEvent(word.clone(), None));
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
        .run();
}
