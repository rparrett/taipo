use bevy::{
    input::{
        keyboard::KeyCode,
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    },
    prelude::*,
    window::ReceivedCharacter,
};

pub struct TypingPlugin;

impl Plugin for TypingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(typing_setup.system())
            .add_resource(TypingCursorTimer(Timer::from_seconds(0.5, true)))
            .add_startup_system(add_typing_targets.system())
            .add_system(typing_system.system())
            .add_system(update_typing_targets.system())
            .add_system(update_typing_buffer.system())
            .add_system(update_typing_cursor.system())
            .add_event::<TypingStateChangedEvent>();
    }
}

struct TypingTarget;
struct TypingPartA;
struct TypingPartB;

struct TypingBuffer;
struct TypingCursor;
struct TypingCursorTimer(Timer);

struct Ascii(String);
struct Japanese(String);

// Seems like ChangedRes isn't good enough for changing a bit of a struct,
// or I don't know how to trigger it or something.
struct TypingStateChangedEvent;

#[derive(Default)]
struct TypingState {
    buf: String,
    event_reader: EventReader<ReceivedCharacter>,
}

fn typing_setup(mut typing_state_events: ResMut<Events<TypingStateChangedEvent>>) {
    typing_state_events.send(TypingStateChangedEvent);
}

fn add_typing_targets(
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
            parent.spawn(TextBundle {
                style: Style {
                    ..Default::default()
                },
                text: Text {
                    value: "TEST".into(),
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
                        value: "TEST".into(),
                        font: font.clone(),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::RED,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(TypingPartA);
            parent
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text {
                        value: "TEST".into(),
                        font: font.clone(),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::BLUE,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(TypingPartB);
        })
        .with(TypingTarget)
        .with(Ascii("hiragana".to_string()))
        .with(Japanese("ひらがな".to_string()));
}

fn update_typing_targets(
    query: Query<(&Ascii, &Japanese, &Children), With<TypingTarget>>,
    mut texta_query: Query<&mut Text, With<TypingPartA>>,
    mut textb_query: Query<&mut Text, With<TypingPartB>>,
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
        for child in target.2.iter() {
            match target.0 .0.strip_prefix(&state.buf) {
                Some(postfix) if state.buf.len() > 0 => {
                    if let Ok(mut a) = texta_query.get_mut(*child) {
                        a.value = state.buf.clone();
                    }
                    if let Ok(mut b) = textb_query.get_mut(*child) {
                        b.value = postfix.to_string();
                    }
                }
                Some(_) | None => {
                    if let Ok(mut a) = texta_query.get_mut(*child) {
                        a.value = "".into();
                    }
                    if let Ok(mut b) = textb_query.get_mut(*child) {
                        b.value = target.0 .0.clone();
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
    time: Res<Time>, mut timer: ResMut<TypingCursorTimer>,
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

fn startup_system(commands: &mut Commands) {
    info!("startup");
    commands
        // 2d camera
        .spawn(CameraUiBundle::default());
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
) {
    let mut changed = false;

    for event in typing_state.event_reader.iter(&char_input_events) {
        typing_state.buf.push(event.char);
        changed = true;
    }

    for ev in input_state.keys.iter(&keyboard_input_events) {
        if ev.key_code == Some(KeyCode::Return) && !ev.state.is_pressed() {
            typing_state.buf.clear();
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
        .init_resource::<TrackInputState>()
        .add_system(track_input_events.system())
        .run();
}
