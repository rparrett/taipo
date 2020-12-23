use bevy::{
    input::{
        keyboard::KeyCode,
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    },
    prelude::*,
    window::ReceivedCharacter,
};
use rand::prelude::SliceRandom;

#[derive(Default)]
pub struct GameState {
    score: u32,
    possible_words: Vec<String>,
}

struct ScoreDisplay;

pub struct TypingPlugin;

impl Plugin for TypingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(typing_setup.system())
            .add_startup_system(spawn_typing_buffer.system())
            .add_resource(TypingCursorTimer(Timer::from_seconds(0.5, true)))
            .add_system(typing_target_spawn_event.system())
            .add_system(typing_system.system())
            .add_system(update_typing_targets.system())
            .add_system(update_typing_buffer.system())
            .add_system(update_typing_cursor.system())
            .add_event::<TypingTargetSpawnEvent>()
            .add_event::<TypingTargetFinishedEvent>()
            .add_event::<TypingSubmitEvent>()
            .add_event::<TypingStateChangedEvent>();
    }
}

struct TypingTarget {
    render: String,
    ascii: String
}
struct TypingTargetMatchedText;
struct TypingTargetUnmatchedText;

struct TypingBuffer;
struct TypingCursor;
struct TypingCursorTimer(Timer);

// Seems like ChangedRes isn't good enough for changing a bit of a struct,
// or I don't know how to trigger it or something.
struct TypingStateChangedEvent;

struct TypingSubmitEvent {
    pub text: String,
}

struct TypingTargetSpawnEvent {
    pub text: String,
}

struct TypingTargetFinishedEvent {
    pub entity: Entity,
}

#[derive(Default)]
struct TypingState {
    buf: String,
    event_reader: EventReader<ReceivedCharacter>,
}

fn check_targets(
    mut reader: Local<EventReader<TypingSubmitEvent>>,
    typing_submit_events: Res<Events<TypingSubmitEvent>>,
    mut typing_target_finished_events: ResMut<Events<TypingTargetFinishedEvent>>,
    query: Query<(Entity, &TypingTarget)>,
) {
    for event in reader.iter(&typing_submit_events) {
        for target in query.iter() {
            if target.1.ascii == event.text {
                typing_target_finished_events.send(TypingTargetFinishedEvent { entity: target.0 });
            }
        }
    }
}

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
        let word = game_state.possible_words.choose(&mut rng).unwrap();

        typing_target_spawn_events.send(TypingTargetSpawnEvent {
            text: word.to_string(),
        });

        game_state.score += 1;

        for mut target in score_display_query.iter_mut() {
            target.value = format!("{}", game_state.score);
        }
    }
}

fn typing_setup(
    mut typing_target_spawn_events: ResMut<Events<TypingTargetSpawnEvent>>,
    game_state: Res<GameState>,
) {
    // Would prefer to reuse an rng. Can we do that?
    let mut rng = rand::thread_rng();
    let word = game_state.possible_words.choose(&mut rng).unwrap();

    typing_target_spawn_events.send(TypingTargetSpawnEvent {
        text: word.to_string(),
    });
}

fn typing_target_spawn_event(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    events: Res<Events<TypingTargetSpawnEvent>>,
    mut reader: Local<EventReader<TypingTargetSpawnEvent>>,
) {
    for event in reader.iter(&events) {
        let font = asset_server.load("fonts/Koruri-Regular.ttf");

        commands
            .spawn(NodeBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    position_type: PositionType::Absolute,
                    position: Rect {
                        left: Val::Px(0.),
                        top: Val::Px(0.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn(TextBundle {
                        style: Style {
                            ..Default::default()
                        },
                        text: Text {
                            value: "".into(),
                            font: font.clone(),
                            style: TextStyle {
                                font_size: 60.0,
                                color: Color::RED,
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
                            value: event.text.clone(),
                            font: font.clone(),
                            style: TextStyle {
                                font_size: 60.0,
                                color: Color::BLUE,
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    })
                    .with(TypingTargetUnmatchedText);
            })
            .with(TypingTarget {
                render: "ひらがな".to_string(),
                ascii: event.text.clone()
            });
    }
}

fn spawn_typing_buffer(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let font = asset_server.load("fonts/Koruri-Regular.ttf");

    commands
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                display: Display::Flex,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(0.),
                    top: Val::Px(0.),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text {
                        value: "".into(),
                        font: font.clone(),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::RED,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(TypingBuffer)
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text {
                        value: "_".into(),
                        font: font.clone(),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::RED,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(TypingCursor);
        });
}

fn update_typing_targets(
    query: Query<(&TypingTarget, &Children)>,
    mut left_query: Query<&mut Text, With<TypingTargetMatchedText>>,
    mut right_query: Query<&mut Text, With<TypingTargetUnmatchedText>>,
    state: Res<TypingState>,
    events: Res<Events<TypingStateChangedEvent>>,
    mut reader: Local<EventReader<TypingStateChangedEvent>>,
) {
    // Only need to update if we have actually received a
    // TypingStteChangedEvent
    if reader.iter(&events).next().is_none() {
        return;
    }

    info!("update_typing_targets");

    for target in query.iter() {
        for child in target.1.iter() {
            match target.0.ascii.strip_prefix(&state.buf) {
                Some(postfix) if state.buf.len() > 0 => {
                    if let Ok(mut left) = left_query.get_mut(*child) {
                        left.value = state.buf.clone();
                    }
                    if let Ok(mut right) = right_query.get_mut(*child) {
                        right.value = postfix.to_string();
                    }
                }
                Some(_) | None => {
                    if let Ok(mut left) = left_query.get_mut(*child) {
                        left.value = "".into();
                    }
                    if let Ok(mut right) = right_query.get_mut(*child) {
                        right.value = target.0.ascii.clone();
                    }
                }
            }
        }
    }
}

fn update_typing_buffer(
    mut query: Query<&mut Text, With<TypingBuffer>>,
    state: Res<TypingState>,
    events: Res<Events<TypingStateChangedEvent>>,
    mut reader: Local<EventReader<TypingStateChangedEvent>>,
) {
    // Only need to update if we have actually received a
    // TypingStteChangedEvent
    if reader.iter(&events).next().is_none() {
        return;
    }

    for mut target in query.iter_mut() {
        target.value = state.buf.clone();
    }
}

fn update_typing_cursor(
    time: Res<Time>,
    mut timer: ResMut<TypingCursorTimer>,
    mut query: Query<&mut Text, With<TypingCursor>>,
) {
    if !timer.0.tick(time.delta_seconds()).just_finished() {
        return;
    }

    for mut target in query.iter_mut() {
        if target.style.color != Color::NONE {
            target.style.color = Color::NONE;
        } else {
            target.style.color = Color::RED;
        }
    }
}

fn startup_system(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
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

    game_state.possible_words.push("hiragana".to_string());
    game_state.possible_words.push("katakana".to_string());
    game_state.possible_words.push("kanji".to_string());
    game_state.possible_words.push("typeme".to_string());
    game_state.possible_words.push("blargle".to_string());
    game_state.possible_words.push("malarkey".to_string());
}

fn counter(mut state: Local<CounterState>, time: Res<Time>) {
    if state.count % 60 == 0 {
        info!(
            "tick {} @ {:?} [d{}]",
            state.count,
            time.time_since_startup(),
            time.delta_seconds()
        );
    }
    state.count += 1;
}

#[derive(Default)]
struct CounterState {
    count: u32,
}

#[derive(Default)]
struct TrackInputState {
    keys: EventReader<KeyboardInput>,
    cursor: EventReader<CursorMoved>,
    motion: EventReader<MouseMotion>,
    mousebtn: EventReader<MouseButtonInput>,
    scroll: EventReader<MouseWheel>,
}

fn track_input_events(
    mut state: ResMut<TrackInputState>,
    ev_keys: Res<Events<KeyboardInput>>,
    ev_cursor: Res<Events<CursorMoved>>,
    ev_motion: Res<Events<MouseMotion>>,
    ev_mousebtn: Res<Events<MouseButtonInput>>,
    ev_scroll: Res<Events<MouseWheel>>,
) {
    // Keyboard input
    for ev in state.keys.iter(&ev_keys) {
        if ev.state.is_pressed() {
            info!("Just pressed key: {:?}", ev.key_code);
        } else {
            info!("Just released key: {:?}", ev.key_code);
        }
    }

    // Absolute cursor position (in window coordinates)
    for ev in state.cursor.iter(&ev_cursor) {
        info!("Cursor at: {}", ev.position);
    }

    // Relative mouse motion
    for ev in state.motion.iter(&ev_motion) {
        info!("Mouse moved {} pixels", ev.delta);
    }

    // Mouse buttons
    for ev in state.mousebtn.iter(&ev_mousebtn) {
        if ev.state.is_pressed() {
            info!("Just pressed mouse button: {:?}", ev.button);
        } else {
            info!("Just released mouse button: {:?}", ev.button);
        }
    }

    // scrolling (mouse wheel, touchpad, etc.)
    for ev in state.scroll.iter(&ev_scroll) {
        info!(
            "Scrolled vertically by {} and horizontally by {}.",
            ev.y, ev.x
        );
    }
}

fn typing_system(
    mut typing_state: ResMut<TypingState>,
    mut input_state: ResMut<TrackInputState>,
    char_input_events: Res<Events<ReceivedCharacter>>,
    keyboard_input_events: Res<Events<KeyboardInput>>,
    mut typing_state_events: ResMut<Events<TypingStateChangedEvent>>,
    mut typing_submit_events: ResMut<Events<TypingSubmitEvent>>,
) {
    let mut changed = false;

    for event in typing_state.event_reader.iter(&char_input_events) {
        typing_state.buf.push(event.char);
        changed = true;
    }

    for ev in input_state.keys.iter(&keyboard_input_events) {
        if ev.key_code == Some(KeyCode::Return) && !ev.state.is_pressed() {
            let text = typing_state.buf.clone();

            typing_state.buf.clear();
            typing_submit_events.send(TypingSubmitEvent { text });

            changed = true;
        }

        if ev.key_code == Some(KeyCode::Back) && !ev.state.is_pressed() {
            typing_state.buf.pop();
            changed = true;
        }
    }

    if changed {
        typing_state_events.send(TypingStateChangedEvent);
    }
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
        .add_startup_system(startup_system.system())
        .add_plugin(TypingPlugin)
        .add_system(counter.system())
        .add_resource(TypingState::default())
        .add_resource(GameState::default())
        .init_resource::<TrackInputState>()
        .add_system(track_input_events.system())
        .add_system(check_targets.system())
        .add_system(typing_target_finished.system())
        .run();
}
