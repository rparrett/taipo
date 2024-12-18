use bevy::{
    input::keyboard::{Key, KeyCode, KeyboardInput},
    prelude::*,
};

use std::collections::VecDeque;

use crate::{
    loading::AudioHandles, ui_color, Action, AudioSettings, FontHandles, TaipoState,
    FONT_SIZE_INPUT,
};

pub struct TypingPlugin;

impl Plugin for TypingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TypingCursorTimer(Timer::from_seconds(
            0.5,
            TimerMode::Repeating,
        )))
        .init_resource::<TypingState>()
        .init_resource::<TypingTargets>();

        app.add_event::<AsciiModeEvent>()
            .add_event::<TypingTargetFinishedEvent>()
            .add_event::<TypingSubmitEvent>();

        // We need the font to have been loaded for this to work.
        app.add_systems(OnEnter(TaipoState::Spawn), startup);
        app.add_systems(
            Update,
            (ascii_mode_event, submit_event)
                .before(keyboard)
                .run_if(in_state(TaipoState::Playing)),
        );
        app.add_systems(Update, keyboard.run_if(in_state(TaipoState::Playing)));
        app.add_systems(
            Update,
            (update_target_text, update_buffer_text, audio)
                .after(keyboard)
                .run_if(in_state(TaipoState::Playing)),
        );
        app.add_systems(
            Update,
            update_cursor_text.run_if(in_state(TaipoState::Playing)),
        );
    }
}

#[derive(Clone, Component, Debug)]
pub struct TypingTarget {
    pub displayed_chunks: Vec<String>,
    pub typed_chunks: Vec<String>,
}
impl TypingTarget {
    pub fn new(word: &str) -> Self {
        let chunks: Vec<String> = word.split("").map(|s| s.to_string()).collect();

        Self {
            displayed_chunks: chunks.clone(),
            typed_chunks: chunks,
        }
    }
}
#[derive(Component, Default)]
pub struct TypingTargetSettings {
    /// If true, do not replace the `TypingTarget` with another from the word list after it is typed.
    pub fixed: bool,
    /// If true, does not perform its action or make sounds when typed.
    pub disabled: bool,
}
#[derive(Bundle)]
pub struct TypingTargetBundle {
    pub target: TypingTarget,
    pub settings: TypingTargetSettings,
    pub action: Action,
}
#[derive(Component)]
pub struct TypingTargetText;

#[derive(Component)]
struct TypingBuffer;
#[derive(Component)]
struct TypingCursor;
#[derive(Resource)]
struct TypingCursorTimer(Timer);

#[derive(Event)]
pub enum AsciiModeEvent {
    Disable,
    Toggle,
}

#[derive(Event)]
pub struct TypingSubmitEvent {
    pub text: String,
}

#[derive(Event)]
pub struct TypingTargetFinishedEvent {
    pub entity: Entity,
}

#[derive(Resource, Default, Debug)]
pub struct TypingState {
    buf: String,
    pub ascii_mode: bool,
    just_typed_char: bool,
}

#[derive(Resource, Default)]
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
            .position(|v| {
                !self
                    .used_ascii
                    .iter()
                    .any(|ascii| *ascii.join("") == v.typed_chunks.join(""))
            })
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
    mut query: Query<(Entity, &mut TypingTarget, &TypingTargetSettings)>,
    children_query: Query<&Children, With<TypingTarget>>,
    text_query: Query<(), With<TypingTargetText>>,
    typing_state: Res<TypingState>,
    mut typing_targets: ResMut<TypingTargets>,
    mut writer: TextUiWriter,
) {
    for event in typing_submit_events.read() {
        for (entity, mut target, settings) in query.iter_mut() {
            if settings.disabled {
                continue;
            }

            if target.typed_chunks.join("") != event.text {
                continue;
            }

            typing_target_finished_events.send(TypingTargetFinishedEvent { entity });

            if settings.fixed {
                continue;
            }

            let new_target = typing_targets.push_back_pop_front(target.clone());

            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    if let Ok(_) = text_query.get(*child) {
                        writer.text(*child, 0).clear();
                        *writer.text(*child, 1) = if typing_state.ascii_mode {
                            new_target.typed_chunks.join("")
                        } else {
                            new_target.displayed_chunks.join("")
                        };
                    }
                }
            }

            target.typed_chunks.clone_from(&new_target.typed_chunks);
            target
                .displayed_chunks
                .clone_from(&new_target.displayed_chunks);
        }
    }
}

fn ascii_mode_event(
    mut typing_state: ResMut<TypingState>,
    mut toggle_events: EventReader<AsciiModeEvent>,
) {
    for event in toggle_events.read() {
        typing_state.ascii_mode = match event {
            AsciiModeEvent::Toggle => !typing_state.ascii_mode,
            AsciiModeEvent::Disable => false,
        }
    }
}

fn startup(mut commands: Commands, font_handles: Res<FontHandles>) {
    commands
        .spawn((
            Node {
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                height: Val::Px(42.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.),
                bottom: Val::Px(0.),
                ..default()
            },
            BackgroundColor(ui_color::TRANSPARENT_BACKGROUND.into()),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(">"),
                TextFont {
                    font: font_handles.jptext.clone(),
                    font_size: FONT_SIZE_INPUT,
                    ..default()
                },
                TextColor(ui_color::NORMAL_TEXT.into()),
                Node {
                    margin: UiRect {
                        left: Val::Px(10.0),
                        right: Val::Px(5.0),
                        ..default()
                    },
                    ..default()
                },
            ));
            parent.spawn((
                Text::default(),
                TextFont {
                    font: font_handles.jptext.clone(),
                    font_size: FONT_SIZE_INPUT,
                    ..default()
                },
                TextColor(ui_color::NORMAL_TEXT.into()),
                TypingBuffer,
            ));
            parent.spawn((
                Text::new("_"),
                TextFont {
                    font: font_handles.jptext.clone(),
                    font_size: FONT_SIZE_INPUT,
                    ..default()
                },
                TextColor(ui_color::CURSOR_TEXT.into()),
                TypingCursor,
            ));
        });
}

fn audio(
    mut commands: Commands,
    state: Res<TypingState>,
    query: Query<(&TypingTarget, &TypingTargetSettings)>,
    audio_handles: Res<AudioHandles>,
    audio_settings: Res<AudioSettings>,
) {
    if !state.is_changed() {
        return;
    }

    let mut longest: usize = 0;

    for (target, _) in query.iter().filter(|(_t, s)| !s.disabled) {
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
        commands.spawn((
            AudioPlayer(audio_handles.wrong_character.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn update_target_text(
    state: Res<TypingState>,
    text_query: Query<(), With<TypingTargetText>>,
    query: Query<(&TypingTarget, &TypingTargetSettings, &Children)>,
    mut text_set: ParamSet<(TextUiReader, TextUiWriter)>,
) {
    if !state.is_changed() {
        return;
    }

    for (target, settings, target_children) in query.iter() {
        if settings.disabled {
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
                    buf.clone_from(&leftover.to_string());
                }
                (true, _) | (_, None) => {
                    fail = true;
                    unmatched.push_str(render);
                }
            }
        }

        for child in target_children.iter() {
            if let Ok(_) = text_query.get(*child) {
                let changed = {
                    let mut reader = text_set.p0();
                    reader.text(*child, 0) != matched || reader.text(*child, 1) != unmatched
                };

                if changed {
                    let mut writer = text_set.p1();
                    writer.text(*child, 0).clone_from(&matched);
                    writer.text(*child, 1).clone_from(&unmatched);
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
        target.0.clone_from(&state.buf);
    }
}

fn update_cursor_text(
    mut timer: ResMut<TypingCursorTimer>,
    mut query: Query<&mut TextColor, With<TypingCursor>>,
    time: Res<Time>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for mut color in query.iter_mut() {
        if color.0 != Color::NONE {
            color.0 = Color::NONE;
        } else {
            color.0 = ui_color::CURSOR_TEXT.into();
        }
    }
}

fn keyboard(
    mut typing_state: ResMut<TypingState>,
    mut typing_submit_events: EventWriter<TypingSubmitEvent>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
) {
    for ev in keyboard_input_events.read() {
        if ev.state.is_pressed() {
            if let Key::Character(ref s) = ev.logical_key {
                typing_state.buf.push_str(s.as_str());
                typing_state.just_typed_char = true;
            } else {
                typing_state.just_typed_char = false;
            }

            match ev.key_code {
                KeyCode::Enter => {
                    let text = typing_state.buf.clone();

                    typing_state.buf.clear();
                    typing_submit_events.send(TypingSubmitEvent { text });
                }
                KeyCode::Backspace => {
                    typing_state.buf.pop();
                }
                KeyCode::Escape => {
                    typing_state.buf.clear();
                }
                _ => {}
            }
        }
    }
}
