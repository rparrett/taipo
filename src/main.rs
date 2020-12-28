use bevy::prelude::*;
use rand::prelude::SliceRandom;
use typing::{TypingPlugin, TypingTarget, TypingTargetFinishedEvent, TypingTargetSpawnEvent};

mod typing;

#[derive(Default)]
pub struct GameState {
    score: u32,
    possible_typing_targets: Vec<TypingTarget>,
}

struct ScoreDisplay;

fn typing_target_finished(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    mut reader: Local<EventReader<TypingTargetFinishedEvent>>,
    typing_target_finished_events: Res<Events<TypingTargetFinishedEvent>>,
    mut typing_target_spawn_events: ResMut<Events<TypingTargetSpawnEvent>>,
    mut score_display_query: Query<&mut Text, With<ScoreDisplay>>,
) {
    for event in reader.iter(&typing_target_finished_events) {
        commands.despawn_recursive(event.entity);

        // Would prefer to reuse an rng. Can we do that?
        let mut rng = rand::thread_rng();
        let word = game_state.possible_typing_targets.choose(&mut rng).unwrap();

        typing_target_spawn_events.send(TypingTargetSpawnEvent(word.clone()));

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
) {
    info!("startup");

    let font = asset_server.load("fonts/Koruri-Regular.ttf");

    commands
        // 2d camera
        .spawn(CameraUiBundle::default());

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

    game_state.possible_typing_targets.push(TypingTarget {
        ascii: vec![
            "hi".to_string(),
            "ra".to_string(),
            "ga".to_string(),
            "na".to_string(),
        ],
        render: vec![
            "ひ".to_string(),
            "ら".to_string(),
            "が".to_string(),
            "な".to_string(),
        ],
    });
    game_state.possible_typing_targets.push(TypingTarget {
        ascii: vec![
            "ka".to_string(),
            "ta".to_string(),
            "ka".to_string(),
            "na".to_string(),
        ],
        render: vec![
            "カ".to_string(),
            "タ".to_string(),
            "カ".to_string(),
            "ナ".to_string(),
        ],
    });
    game_state.possible_typing_targets.push(TypingTarget {
        ascii: vec!["oo".to_string(), "ki".to_string(), "i".to_string()],
        render: vec!["大".to_string(), "き".to_string(), "い".to_string()],
    });

    // Would prefer to reuse an rng. Can we do that?
    let mut rng = rand::thread_rng();
    let word = game_state.possible_typing_targets.choose(&mut rng).unwrap();

    typing_target_spawn_events.send(TypingTargetSpawnEvent(word.clone()));
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            width: 300.,
            height: 300.,
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
