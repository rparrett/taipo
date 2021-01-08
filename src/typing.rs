use bevy::{
    input::{keyboard::KeyCode, keyboard::KeyboardInput},
    prelude::*,
};

use crate::{AppState, FontHandles, FONT_SIZE, STAGE};

pub struct TypingPlugin;

impl Plugin for TypingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // We need the font to have been loaded for this to work.
        app.on_state_enter(STAGE, AppState::Spawn, startup.system())
            .add_resource(TypingCursorTimer(Timer::from_seconds(0.5, true)))
            .add_resource(TypingState::default())
            .add_resource(TrackInputState::default())
            .add_system(typing_target_spawn_event.system())
            .add_system(typing_target_change_event.system())
            .add_system(typing_system.system())
            .add_system(update_typing_targets.system())
            .add_system(update_typing_buffer.system())
            .add_system(update_typing_cursor.system())
            .add_system(check_targets.system())
            .add_event::<TypingTargetSpawnEvent>()
            .add_event::<TypingTargetChangeEvent>()
            .add_event::<TypingTargetFinishedEvent>()
            .add_event::<TypingSubmitEvent>();
    }
}

#[derive(Default)]
pub struct TrackInputState {
    pub keys: EventReader<KeyboardInput>,
}

pub struct TypingTargetContainer;

#[derive(Clone, Debug, Default)]
pub struct TypingTarget {
    pub render: Vec<String>,
    pub ascii: Vec<String>,
}
pub struct TypingTargetImage;
pub struct TypingTargetPriceContainer;
pub struct TypingTargetPriceText;
pub struct TypingTargetPriceImage;
pub struct TypingTargetMatchedText;
pub struct TypingTargetUnmatchedText;
pub struct TypingTargetFullText;

struct TypingBuffer;
struct TypingCursor;
struct TypingCursorTimer(Timer);

pub struct TypingSubmitEvent {
    pub text: String,
}

pub struct TypingTargetSpawnEvent(pub TypingTarget, pub Option<Entity>);

pub struct TypingTargetFinishedEvent {
    pub entity: Entity,
    pub target: TypingTarget,
}

pub struct TypingTargetChangeEvent {
    pub entity: Entity,
    pub target: TypingTarget,
}

#[derive(Default, Debug)]
pub struct TypingState {
    buf: String,
    pub ascii_mode: bool,
}

fn check_targets(
    mut reader: Local<EventReader<TypingSubmitEvent>>,
    typing_submit_events: Res<Events<TypingSubmitEvent>>,
    mut typing_target_finished_events: ResMut<Events<TypingTargetFinishedEvent>>,
    query: Query<(Entity, &TypingTarget)>,
) {
    for event in reader.iter(&typing_submit_events) {
        for target in query.iter() {
            if target.1.ascii.join("") == event.text {
                typing_target_finished_events.send(TypingTargetFinishedEvent {
                    entity: target.0,
                    target: target.1.clone(),
                });
            }
        }
    }
}

fn typing_target_change_event(
    mut query: Query<(&mut TypingTarget, &Children)>,
    mut left_query: Query<&mut Text, With<TypingTargetMatchedText>>,
    mut right_query: Query<&mut Text, With<TypingTargetUnmatchedText>>,
    mut full_query: Query<&mut Text, With<TypingTargetFullText>>,
    events: Res<Events<TypingTargetChangeEvent>>,
    mut reader: Local<EventReader<TypingTargetChangeEvent>>,
    typing_state: Res<TypingState>,
) {
    for event in reader.iter(&events) {
        info!("processing TypingTargetChangeEvent");
        for (mut target, children) in query.get_mut(event.entity) {
            for child in children.iter() {
                if let Ok(mut left) = left_query.get_mut(*child) {
                    left.value = "".to_string();
                }
                if let Ok(mut right) = right_query.get_mut(*child) {
                    right.value = if typing_state.ascii_mode {
                        event.target.ascii.join("")
                    } else {
                        event.target.render.join("")
                    };
                }
                if let Ok(mut full) = full_query.get_mut(*child) {
                    full.value = if typing_state.ascii_mode {
                        event.target.ascii.join("")
                    } else {
                        event.target.render.join("")
                    };
                }
            }

            target.ascii = event.target.ascii.clone();
            target.render = event.target.render.clone();
        }
    }
}

fn typing_target_spawn_event(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    events: Res<Events<TypingTargetSpawnEvent>>,
    mut reader: Local<EventReader<TypingTargetSpawnEvent>>,
    container_query: Query<(Entity, Option<&Children>), With<TypingTargetContainer>>,
    font_handles: Res<FontHandles>,
) {
    for event in reader.iter(&events) {
        info!("processing TypingTargetSpawnEvent");

        for (container, children) in container_query.iter() {
            let child = commands
                .spawn(NodeBundle {
                    style: Style {
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        size: Size::new(Val::Percent(100.0), Val::Px(42.0)),
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
                                font: font_handles.jptext.clone(),
                                style: TextStyle {
                                    font_size: FONT_SIZE,
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
                                value: event.0.render.join(""),
                                font: font_handles.jptext.clone(),
                                style: TextStyle {
                                    font_size: FONT_SIZE,
                                    color: Color::WHITE,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        })
                        .with(TypingTargetUnmatchedText);
                })
                .with(event.0.clone())
                .current_entity()
                .unwrap();

            // If we're replacing another target, make sure we end up in the same
            // position.
            let mut insert_index = 0;
            if let Some(replaced) = event.1 {
                if let Some(children) = children {
                    if let Some(index) = children.iter().position(|c| *c == replaced) {
                        insert_index = index;
                    }
                }
                commands.despawn_recursive(replaced);
            }

            commands.insert_children(container, insert_index, &[child]);
        }
    }
}

fn startup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    font_handles: Res<FontHandles>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                size: Size::new(Val::Percent(100.0), Val::Px(42.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(0.),
                    bottom: Val::Px(0.),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.50).into()),
            ..Default::default()
        })
        .with(TypingTargetPriceContainer)
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(10.0),
                            right: Val::Px(5.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: ">".into(),
                        font: font_handles.jptext.clone(),
                        style: TextStyle {
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text {
                        value: "".into(),
                        font: font_handles.jptext.clone(),
                        style: TextStyle {
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
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
                        font: font_handles.jptext.clone(),
                        style: TextStyle {
                            font_size: FONT_SIZE,
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
    state: ChangedRes<TypingState>,
    query: Query<(&TypingTarget, &Children)>,
    // accessing a mut text in a query seems to trigger recalculation / layout
    // even if the text.value did not actually change.
    // so we'll
    mut left_queries: QuerySet<(
        Query<&Text, With<TypingTargetMatchedText>>,
        Query<&mut Text, With<TypingTargetMatchedText>>,
    )>,
    mut right_queries: QuerySet<(
        Query<&Text, With<TypingTargetUnmatchedText>>,
        Query<&mut Text, With<TypingTargetUnmatchedText>>,
    )>,
    mut full_queries: QuerySet<(
        Query<&Text, With<TypingTargetFullText>>,
        Query<&mut Text, With<TypingTargetFullText>>,
    )>,
) {
    info!("changedres<typingstate>");
    for (target, target_children) in query.iter() {
        let mut matched = "".to_string();
        let mut unmatched = "".to_string();
        let mut buf = state.buf.clone();
        let mut fail = false;

        let render_iter = if state.ascii_mode {
            target.ascii.iter()
        } else {
            target.render.iter()
        };

        for (ascii, render) in target.ascii.iter().zip(render_iter) {
            match (fail, buf.strip_prefix(ascii)) {
                (false, Some(leftover)) => {
                    matched.push_str(&render);
                    buf = leftover.to_string().clone();
                }
                (true, _) | (_, None) => {
                    fail = true;
                    unmatched.push_str(&render);
                }
            }
        }

        for child in target_children.iter() {
            if let Ok(left) = left_queries.q0().get(*child) {
                if left.value != matched {
                    if let Ok(mut leftmut) = left_queries.q1_mut().get_mut(*child) {
                        leftmut.value = matched.clone();
                    }
                }
            }

            if let Ok(right) = right_queries.q0().get(*child) {
                if right.value != unmatched {
                    if let Ok(mut rightmut) = right_queries.q1_mut().get_mut(*child) {
                        rightmut.value = unmatched.clone();
                    }
                }
            }

            // This needs to happen in case we just switched to
            // ascii mode and various sizes need to be recalculated
            if let Ok(full) = full_queries.q0().get(*child) {
                if full.value != target.render.join("") {
                    if let Ok(mut fullmut) = full_queries.q1_mut().get_mut(*child) {
                        fullmut.value = target.render.join("")
                    }
                }
            }
        }
    }
}

fn update_typing_buffer(
    mut query: Query<&mut Text, With<TypingBuffer>>,
    state: ChangedRes<TypingState>,
) {
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

fn typing_system(
    mut typing_state: ResMut<TypingState>,
    mut input_state: ResMut<TrackInputState>,
    keyboard_input_events: Res<Events<KeyboardInput>>,
    mut typing_submit_events: ResMut<Events<TypingSubmitEvent>>,
) {
    // We were previously using Res<Events<ReceivedCharacter>> to handle the ascii bits,
    // and Res<Events<KeyboardInput>> to handle backspace/enter, but there was something
    // wacky going on where backspace could end up coming in out of order.
    //
    // After testing using puppeteer to shove various keyboard inputs in, it seems like
    // this solution, though ugly, results in a better typing experience.
    //
    // I had also attempted to get ReceivedCharacter to give me backspace/enter, but that
    // was not working, despite winit docs seeming to suggest that it should. But I found
    // that I received no ReceivedCharacter events at all when typing backspace/enter.
    //
    // I'm guessing that the ReceivedCharacter approach would be ideal though if this
    // solution doesn't work for people with non-english keyboards or dvorak layouts or
    // whatever.

    for ev in input_state.keys.iter(&keyboard_input_events) {
        if ev.state.is_pressed() {
            let maybe_char = match ev.key_code {
                Some(KeyCode::A) => Some('a'),
                Some(KeyCode::B) => Some('b'),
                Some(KeyCode::C) => Some('c'),
                Some(KeyCode::D) => Some('d'),
                Some(KeyCode::E) => Some('e'),
                Some(KeyCode::F) => Some('f'),
                Some(KeyCode::G) => Some('g'),
                Some(KeyCode::H) => Some('h'),
                Some(KeyCode::I) => Some('i'),
                Some(KeyCode::J) => Some('j'),
                Some(KeyCode::K) => Some('k'),
                Some(KeyCode::L) => Some('l'),
                Some(KeyCode::M) => Some('m'),
                Some(KeyCode::N) => Some('n'),
                Some(KeyCode::O) => Some('o'),
                Some(KeyCode::P) => Some('p'),
                Some(KeyCode::Q) => Some('q'),
                Some(KeyCode::R) => Some('r'),
                Some(KeyCode::S) => Some('s'),
                Some(KeyCode::T) => Some('t'),
                Some(KeyCode::U) => Some('u'),
                Some(KeyCode::V) => Some('v'),
                Some(KeyCode::W) => Some('w'),
                Some(KeyCode::X) => Some('x'),
                Some(KeyCode::Y) => Some('y'),
                Some(KeyCode::Z) => Some('z'),
                Some(KeyCode::Minus) => Some('-'),
                Some(KeyCode::Slash) => Some('?'), // should check for shift
                Some(KeyCode::Key1) => Some('!'),  // should check for shift
                _ => None,
            };

            if let Some(char) = maybe_char {
                typing_state.buf.push(char);
            }

            if ev.key_code == Some(KeyCode::Return) {
                let text = typing_state.buf.clone();

                typing_state.buf.clear();
                typing_submit_events.send(TypingSubmitEvent { text });
            }

            if ev.key_code == Some(KeyCode::Back) {
                typing_state.buf.pop();
            }
        }
    }
}
