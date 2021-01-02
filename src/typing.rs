use bevy::{
    input::{keyboard::KeyCode, keyboard::KeyboardInput},
    prelude::*,
    window::ReceivedCharacter,
};

use crate::FontHandles;

pub struct TypingPlugin;

impl Plugin for TypingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(startup.system())
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
            .add_event::<TypingSubmitEvent>()
            .add_event::<TypingStateChangedEvent>();
    }
}

#[derive(Default)]
pub struct TrackInputState {
    pub keys: EventReader<KeyboardInput>,
}

pub struct TypingTargetContainer;

#[derive(Clone, Debug)]
pub struct TypingTarget {
    pub render: Vec<String>,
    pub ascii: Vec<String>,
}
pub struct TypingTargetImage;
struct TypingTargetMatchedText;
struct TypingTargetUnmatchedText;

struct TypingBuffer;
struct TypingCursor;
struct TypingCursorTimer(Timer);

// Seems like ChangedRes isn't good enough for changing a bit of a struct,
// or I don't know how to trigger it or something.
pub struct TypingStateChangedEvent;

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
    events: Res<Events<TypingTargetChangeEvent>>,
    mut reader: Local<EventReader<TypingTargetChangeEvent>>,
) {
    for event in reader.iter(&events) {
        info!("processing TypingTargetChangeEvent");
        for (mut target, children) in query.get_mut(event.entity) {
            for child in children.iter() {
                if let Ok(mut left) = left_query.get_mut(*child) {
                    left.value = "".to_string();
                }
                if let Ok(mut right) = right_query.get_mut(*child) {
                    right.value = event.target.render.join("");
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
                                font: font_handles.koruri.clone(),
                                style: TextStyle {
                                    font_size: 32.0,
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
                                font: font_handles.koruri.clone(),
                                style: TextStyle {
                                    font_size: 32.0,
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
        .with(TypingTargetContainer);

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
                        font: font_handles.koruri.clone(),
                        style: TextStyle {
                            font_size: 32.0,
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
                        font: font_handles.koruri.clone(),
                        style: TextStyle {
                            font_size: 32.0,
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
                        font: font_handles.koruri.clone(),
                        style: TextStyle {
                            font_size: 32.0,
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
    // TypingStateChangedEvent
    if reader.iter(&events).next().is_none() {
        return;
    }

    info!("update_typing_targets");

    for target in query.iter() {
        let mut matched = "".to_string();
        let mut unmatched = "".to_string();
        let mut buf = state.buf.clone();
        let mut fail = false;

        for (ascii, render) in target.0.ascii.iter().zip(target.0.render.iter()) {
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

        for child in target.1.iter() {
            if let Ok(mut left) = left_query.get_mut(*child) {
                left.value = matched.clone();
            }
            if let Ok(mut right) = right_query.get_mut(*child) {
                right.value = unmatched.clone();
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
    // TypingStateChangedEvent
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
