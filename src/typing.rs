use bevy::{
    input::{keyboard::KeyCode, keyboard::KeyboardInput},
    prelude::*,
};
use bevy_kira_audio::Audio;

use crate::{AudioHandles, AudioSettings, FontHandles, TaipoState, FONT_SIZE_INPUT};

use std::collections::VecDeque;

pub struct TypingPlugin;

impl Plugin for TypingPlugin {
    fn build(&self, app: &mut App) {
        // We need the font to have been loaded for this to work.
        app.add_system_set(SystemSet::on_enter(TaipoState::Spawn).with_system(startup.system()))
            .insert_resource(TypingCursorTimer(Timer::from_seconds(0.5, true)))
            .insert_resource(TypingState::default())
            .init_resource::<TypingTargets>()
            .add_event::<AsciiModeEvent>()
            .add_event::<TypingTargetFinishedEvent>()
            .add_event::<TypingSubmitEvent>()
            .add_system(ascii_mode_event.system().before("keyboard"))
            .add_system(submit_event.system().before("keyboard"))
            .add_system(keyboard.system().label("keyboard"))
            .add_system(update_target_text.system().after("keyboard"))
            .add_system(update_buffer_text.system().after("keyboard"))
            .add_system(audio.system().after("keyboard"))
            .add_system(update_cursor_text.system());
    }
}

#[derive(Component)]
pub struct TypingTargetContainer;

#[derive(Clone, Component, Debug, Default)]
pub struct TypingTarget {
    pub displayed_chunks: Vec<String>,
    pub typed_chunks: Vec<String>,
    /// If true, do not replace the `TypingTarget` with another from the word list after it is typed.
    pub fixed: bool,
    /// If true, does not perform its action or make sounds when typed.
    pub disabled: bool,
}

#[derive(Component)]
pub struct TypingTargetImage;
#[derive(Component)]
pub struct TypingTargetPriceContainer;
#[derive(Component)]
pub struct TypingTargetPriceText;
#[derive(Component)]
pub struct TypingTargetPriceImage;
#[derive(Component)]
pub struct TypingTargetText;

#[derive(Component)]
struct TypingBuffer;
#[derive(Component)]
struct TypingCursor;
#[derive(Component)]
struct TypingCursorTimer(Timer);

pub enum AsciiModeEvent {
    Disable,
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

#[derive(Default)]
pub struct TypingTargets {
    pub possible: VecDeque<TypingTarget>,
    used_ascii: Vec<Vec<String>>,
}

impl TypingTargets {
    /// Returns the next `TypingTarget`, removing it from the list of possible
    /// targets and ensuring that it is not ambiguous with another target that
    /// was previous removed from the stack.
    pub fn pop_front(&mut self) -> TypingTarget {
        let next_pos = self
            .possible
            .iter()
            .position(|v| !self.used_ascii.iter().any(|ascii| *ascii == v.typed_chunks))
            .expect("no word found");

        let next = self.possible.remove(next_pos).unwrap();

        self.used_ascii.push(next.typed_chunks.clone());

        next
    }

    /// Puts a `TypingTarget` back into the list of possible targets and returns
    /// the next target, ensuring that it is not ambiguous with another target
    /// that was previously removed from the stack or the target that was put
    /// back.
    pub fn push_back_pop_front(&mut self, target: TypingTarget) -> TypingTarget {
        self.possible.push_back(target.clone());

        let next = self.pop_front();

        if next.typed_chunks != target.typed_chunks {
            self.used_ascii
                .retain(|ascii| *ascii != target.typed_chunks);
        }

        next
    }
}

fn submit_event(
    mut typing_submit_events: EventReader<TypingSubmitEvent>,
    mut typing_target_finished_events: EventWriter<TypingTargetFinishedEvent>,
    mut query: Query<(Entity, &mut TypingTarget)>,
    children_query: Query<&Children, With<TypingTarget>>,
    mut text_query: Query<&mut Text, With<TypingTargetText>>,
    typing_state: Res<TypingState>,
    mut typing_targets: ResMut<TypingTargets>,
) {
    for event in typing_submit_events.iter() {
        for (entity, mut target) in query.iter_mut() {
            if target.disabled {
                continue;
            }

            if target.typed_chunks.join("") != event.text {
                continue;
            }

            typing_target_finished_events.send(TypingTargetFinishedEvent {
                entity,
                target: target.clone(),
            });

            if target.fixed {
                continue;
            }

            let new_target = typing_targets.push_back_pop_front(target.clone());

            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(*child) {
                        text.sections[0].value = "".to_string();
                        text.sections[1].value = if typing_state.ascii_mode {
                            new_target.typed_chunks.join("")
                        } else {
                            new_target.displayed_chunks.join("")
                        };
                    }
                }
            }

            target.typed_chunks = new_target.typed_chunks.clone();
            target.displayed_chunks = new_target.displayed_chunks.clone();
        }
    }
}

fn ascii_mode_event(
    mut typing_state: ResMut<TypingState>,
    mut toggle_events: EventReader<AsciiModeEvent>,
) {
    for event in toggle_events.iter() {
        typing_state.ascii_mode = match event {
            AsciiModeEvent::Toggle => !typing_state.ascii_mode,
            AsciiModeEvent::Disable => false,
        }
    }
}

fn startup(mut commands: Commands, font_handles: Res<FontHandles>) {
    commands
        .spawn_bundle(NodeBundle {
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
            color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
            ..Default::default()
        })
        .insert(TypingTargetPriceContainer)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
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
                    },
                    TextAlignment::default(),
                ),
                ..Default::default()
            });
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "".to_string(),
                        TextStyle {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE_INPUT,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(TypingBuffer);
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "_".to_string(),
                        TextStyle {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE_INPUT,
                            color: Color::RED,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(TypingCursor);
        });
}

fn audio(
    state: Res<TypingState>,
    query: Query<&TypingTarget>,
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
    audio_settings: Res<AudioSettings>,
) {
    if !state.is_changed() {
        return;
    }

    let mut longest: usize = 0;

    for target in query.iter().filter(|t| !t.disabled) {
        let matched_length = if target.typed_chunks.join("").starts_with(&state.buf) {
            state.buf.len()
        } else {
            0
        };

        if matched_length > longest {
            longest = matched_length;
        }
    }

    if !audio_settings.mute && state.just_typed_char && longest < state.buf.len() {
        audio.play(audio_handles.wrong_character.clone());
    }
}

#[allow(clippy::type_complexity)]
fn update_target_text(
    state: Res<TypingState>,
    // accessing a mut text in a query seems to trigger recalculation / layout
    // even if the text.value did not actually change.
    // so we'll
    mut text_queries: QuerySet<(
        QueryState<&Text, With<TypingTargetText>>,
        QueryState<&mut Text, With<TypingTargetText>>,
    )>,
    query: Query<(&TypingTarget, &Children)>,
) {
    if !state.is_changed() {
        return;
    }

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
            target.typed_chunks.iter()
        } else {
            target.displayed_chunks.iter()
        };

        for (ascii, render) in target.typed_chunks.iter().zip(render_iter) {
            match (fail, buf.strip_prefix(ascii)) {
                (false, Some(leftover)) => {
                    matched.push_str(render);
                    buf = leftover.to_string().clone();
                }
                (true, _) | (_, None) => {
                    fail = true;
                    unmatched.push_str(render);
                }
            }
        }

        for child in target_children.iter() {
            if let Ok(text) = text_queries.q0().get(*child) {
                if text.sections[0].value != matched || text.sections[1].value != unmatched {
                    if let Ok(mut textmut) = text_queries.q1().get_mut(*child) {
                        textmut.sections[0].value = matched.clone();
                        textmut.sections[1].value = unmatched.clone();
                    }
                }
            }
        }
    }
}

fn update_buffer_text(state: Res<TypingState>, mut query: Query<&mut Text, With<TypingBuffer>>) {
    if !state.is_changed() {
        return;
    }

    for mut target in query.iter_mut() {
        target.sections[0].value = state.buf.clone();
    }
}

fn update_cursor_text(
    mut timer: ResMut<TypingCursorTimer>,
    mut query: Query<&mut Text, With<TypingCursor>>,
    time: Res<Time>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
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

fn keyboard(
    mut typing_state: ResMut<TypingState>,
    mut typing_submit_events: EventWriter<TypingSubmitEvent>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
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
