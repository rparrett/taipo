use bevy::{
    input::{keyboard::KeyCode, keyboard::KeyboardInput},
    prelude::*,
};
use bevy_kira_audio::Audio;

use crate::{AppState, AudioHandles, AudioSettings, FontHandles, FONT_SIZE_INPUT, STAGE};

use std::collections::VecDeque;

pub struct TypingPlugin;

impl Plugin for TypingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // We need the font to have been loaded for this to work.
        app.on_state_enter(STAGE, AppState::Spawn, startup.system())
            .insert_resource(TypingCursorTimer(Timer::from_seconds(0.5, true)))
            .insert_resource(TypingState::default())
            .insert_resource(MatchState::default())
            .init_resource::<TypingTargets>()
            .add_system(
                typing_target_ascii_mode_event
                    .system()
                    .before("typing_system"),
            )
            .add_system(check_targets.system().before("typing_system"))
            .add_system(typing_system.system().label("typing_system"))
            .add_system(update_typing_targets.system().after("typing_system"))
            .add_system(update_typing_buffer.system().after("typing_system"))
            .add_system(typing_audio.system().after("typing_system"))
            .add_system(update_typing_cursor.system())
            .add_event::<AsciiModeEvent>()
            .add_event::<TypingTargetFinishedEvent>()
            .add_event::<TypingSubmitEvent>();
    }
}

pub struct TypingTargetContainer;

#[derive(Clone, Debug, Default)]
pub struct TypingTarget {
    pub render: Vec<String>,
    pub ascii: Vec<String>,
    pub fixed: bool,
    pub disabled: bool,
}
pub struct TypingTargetImage;
pub struct TypingTargetPriceContainer;
pub struct TypingTargetPriceText;
pub struct TypingTargetPriceImage;
pub struct TypingTargetText;

struct TypingBuffer;
struct TypingCursor;
struct TypingCursorTimer(Timer);

pub enum AsciiModeEvent {
    Disable,
    Enable,
    Toggle,
}

pub struct TypingSubmitEvent {
    pub text: String,
}

pub struct TypingTargetFinishedEvent {
    pub entity: Entity,
    pub target: TypingTarget,
}

#[derive(Default, Debug)]
pub struct TypingState {
    buf: String,
    pub ascii_mode: bool,
    just_typed_char: bool,
}
#[derive(Default, Debug)]
pub struct MatchState {
    longest: usize,
}

#[derive(Default)]
pub struct TypingTargets {
    pub possible: VecDeque<TypingTarget>,
    used_ascii: Vec<Vec<String>>,
}

impl TypingTargets {
    pub fn pop_front(&mut self) -> TypingTarget {
        let next_pos = self
            .possible
            .iter()
            .position(|v| !self.used_ascii.iter().any(|ascii| *ascii == v.ascii))
            .expect("no word found");

        let target = self.possible.remove(next_pos).expect("no words");

        self.used_ascii.push(target.ascii.clone());

        target
    }

    pub fn replace(&mut self, target: TypingTarget) -> TypingTarget {
        let next_pos = self
            .possible
            .iter()
            .position(|v| !self.used_ascii.iter().any(|ascii| *ascii == v.ascii))
            .expect("no word found");

        let next = self.possible.remove(next_pos).unwrap();

        self.possible.push_back(target.clone());

        if let Some(pos) = self.used_ascii.iter().position(|a| **a == target.ascii) {
            self.used_ascii.remove(pos);
        }

        self.used_ascii.push(next.ascii.clone());

        next
    }
}

fn check_targets(
    mut typing_submit_events: EventReader<TypingSubmitEvent>,
    mut typing_target_finished_events: ResMut<Events<TypingTargetFinishedEvent>>,
    //mut queries: QuerySet<(Query<(Entity, &TypingTarget)>, Query<&mut TypingTarget>)>,
    mut query: Query<(Entity, &mut TypingTarget)>,
    children_query: Query<&Children, With<TypingTarget>>,
    mut text_query: Query<&mut Text, With<TypingTargetText>>,
    mut typing_state: ResMut<TypingState>,
    mut typing_targets: ResMut<TypingTargets>,
) {
    for event in typing_submit_events.iter() {
        for (entity, mut target) in query.iter_mut() {
            if target.disabled {
                continue;
            }

            if target.ascii.join("") != event.text {
                continue;
            }

            typing_target_finished_events.send(TypingTargetFinishedEvent {
                entity,
                target: target.clone(),
            });

            if target.fixed {
                continue;
            }

            let new_target = typing_targets.replace(target.clone());

            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(*child) {
                        text.sections[0].value = "".to_string();
                        text.sections[1].value = if typing_state.ascii_mode {
                            new_target.ascii.join("")
                        } else {
                            new_target.render.join("")
                        };
                    }
                }
            }

            target.ascii = new_target.ascii.clone();
            target.render = new_target.render.clone();
        }
    }
}

fn typing_target_ascii_mode_event(
    mut typing_state: ResMut<TypingState>,
    mut toggle_events: EventReader<AsciiModeEvent>,
) {
    for event in toggle_events.iter() {
        typing_state.ascii_mode = match event {
            AsciiModeEvent::Toggle => !typing_state.ascii_mode,
            AsciiModeEvent::Disable => false,
            AsciiModeEvent::Enable => true,
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
                    text: Text::with_section(
                        ">".to_string(),
                        TextStyle {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE_INPUT,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "".to_string(),
                        TextStyle {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE_INPUT,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .with(TypingBuffer)
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "_".to_string(),
                        TextStyle {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE_INPUT,
                            color: Color::RED,
                            ..Default::default()
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .with(TypingCursor);
        });
}

fn typing_audio(
    state: ChangedRes<TypingState>,
    query: Query<&TypingTarget>,
    mut match_state: ResMut<MatchState>,
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
    audio_settings: Res<AudioSettings>,
) {
    let mut longest: usize = 0;

    for target in query.iter().filter(|t| !t.disabled) {
        let matched_length = target
            .ascii
            .join("")
            .chars()
            .zip(state.buf.chars())
            .position(|(a, b)| a != b)
            .unwrap_or(state.buf.len());

        info!("{} {}", target.ascii.join(""), matched_length);

        if matched_length > longest {
            longest = matched_length;
        }
    }

    info!(
        "{} {} {} {}",
        audio_settings.mute, longest, match_state.longest, state.just_typed_char
    );

    if !audio_settings.mute && longest <= match_state.longest && state.just_typed_char {
        audio.play(audio_handles.wrong_character.clone());
    }

    match_state.longest = longest;
}

fn update_typing_targets(
    state: ChangedRes<TypingState>,
    query: Query<(&TypingTarget, &Children)>,
    // accessing a mut text in a query seems to trigger recalculation / layout
    // even if the text.value did not actually change.
    // so we'll
    mut text_queries: QuerySet<(
        Query<&Text, With<TypingTargetText>>,
        Query<&mut Text, With<TypingTargetText>>,
    )>,
) {
    info!("changedres<typingstate>");

    for (target, target_children) in query.iter() {
        if target.disabled {
            continue;
        }

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
            if let Ok(text) = text_queries.q0().get(*child) {
                if text.sections[0].value != matched || text.sections[1].value != unmatched {
                    if let Ok(mut textmut) = text_queries.q1_mut().get_mut(*child) {
                        textmut.sections[0].value = matched.clone();
                        textmut.sections[1].value = unmatched.clone();
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
        target.sections[0].value = state.buf.clone();
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
        if target.sections[0].style.color != Color::NONE {
            target.sections[0].style.color = Color::NONE;
        } else {
            target.sections[0].style.color = Color::RED;
        }
    }
}

fn typing_system(
    mut typing_state: ResMut<TypingState>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
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

    for ev in keyboard_input_events.iter() {
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
                typing_state.just_typed_char = true;
            } else {
                typing_state.just_typed_char = false;
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
