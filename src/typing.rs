use bevy::{
    input::keyboard::{Key, KeyCode, KeyboardInput},
    prelude::*,
    text::{TextReader, TextRoot, TextWriter},
};

use std::collections::VecDeque;

use crate::{
    loading::AudioHandles, ui_color, Action, AudioSettings, CleanupBeforeNewGame, FontHandles,
    TaipoState, FONT_SIZE_INPUT,
};

pub struct TypingPlugin;

impl Plugin for TypingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TypingCursorTimer(Timer::from_seconds(
            0.5,
            TimerMode::Repeating,
        )))
        .init_resource::<TypingState>()
        .init_resource::<PromptPool>();

        app.add_event::<HelpModeEvent>()
            .add_event::<PromptCompletedEvent>()
            .add_event::<TypingSubmitEvent>();

        // We need the font to have been loaded for this to work.
        app.add_systems(OnEnter(TaipoState::Spawn), startup);
        app.add_systems(
            Update,
            (handle_help_mode, handle_submit)
                .before(keyboard)
                .run_if(in_state(TaipoState::Playing)),
        );
        app.add_systems(Update, keyboard.run_if(in_state(TaipoState::Playing)));
        app.add_systems(
            Update,
            (
                update_prompt_text::<Text>,
                update_prompt_text::<Text2d>,
                update_buffer_text,
                audio,
            )
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
pub struct PromptChunks {
    pub displayed: Vec<String>,
    pub typed: Vec<String>,
}
impl PromptChunks {
    /// Create a new `PromptChunks` from an ascii string. The "displayed" and "typed"
    /// chunks will be the same.
    pub fn new(word: &str) -> Self {
        let chunks: Vec<String> = word.split("").map(|s| s.to_string()).collect();

        Self {
            displayed: chunks.clone(),
            typed: chunks,
        }
    }
}
#[derive(Component, Default)]
pub struct PromptSettings {
    /// If true, do not replace the `Prompt` with another from the word list after it is typed.
    pub fixed: bool,
    /// If true, does not perform its action or make sounds when typed.
    pub disabled: bool,
}
#[derive(Bundle)]
pub struct Prompt {
    pub chunks: PromptChunks,
    pub settings: PromptSettings,
    pub action: Action,
}
/// A marker component for the `Text` representing a `Prompt`.
#[derive(Component)]
pub struct PromptText;

#[derive(Component)]
struct TypingBuffer;
/// A marker component for the `Text` representing the cursor.
#[derive(Component)]
struct TypingCursor;
#[derive(Resource)]
struct TypingCursorTimer(Timer);

#[derive(Event)]
pub enum HelpModeEvent {
    Disable,
    Toggle,
}

#[derive(Event)]
pub struct TypingSubmitEvent {
    pub text: String,
}

#[derive(Event)]
pub struct PromptCompletedEvent {
    pub entity: Entity,
}

#[derive(Resource, Default, Debug)]
pub struct TypingState {
    buffer: String,
    pub help_mode: bool,
    just_typed_char: bool,
}

#[derive(Resource, Default)]
pub struct PromptPool {
    pub possible: VecDeque<PromptChunks>,
    used_ascii: Vec<Vec<String>>,
}

impl PromptPool {
    /// Returns the next `Prompts`, removing it from the list of possible
    /// prompts and ensuring that it is not ambiguous with another prompt that
    /// was previously removed from the stack.
    pub fn pop_front(&mut self) -> PromptChunks {
        let next_pos = self
            .possible
            .iter()
            .position(|v| {
                !self
                    .used_ascii
                    .iter()
                    .any(|ascii| *ascii.join("") == v.typed.join(""))
            })
            .expect("no word found");

        let next = self.possible.remove(next_pos).unwrap();

        self.used_ascii.push(next.typed.clone());

        next
    }

    /// Puts a `PromptChunks` back into the list of possible prompts and returns
    /// the next prompt, ensuring that it is not ambiguous with another prompt
    /// that was previously removed from the stack or the prompt that was put
    /// back.
    pub fn push_back_pop_front(&mut self, prompt: PromptChunks) -> PromptChunks {
        self.possible.push_back(prompt.clone());

        let next = self.pop_front();

        if next.typed != prompt.typed {
            self.used_ascii.retain(|ascii| *ascii != prompt.typed);
        }

        next
    }
}

fn handle_submit(
    mut typing_submit_events: EventReader<TypingSubmitEvent>,
    mut prompt_completed_events: EventWriter<PromptCompletedEvent>,
    mut prompts: Query<(Entity, &mut PromptChunks, &PromptSettings)>,
    prompt_children: Query<&Children, With<PromptChunks>>,
    prompt_texts: Query<(), With<PromptText>>,
    typing_state: Res<TypingState>,
    mut prompt_pool: ResMut<PromptPool>,
    mut text_set: ParamSet<(TextUiWriter, Text2dWriter)>,
) {
    for event in typing_submit_events.read() {
        for (entity, mut prompt, settings) in prompts.iter_mut() {
            if settings.disabled {
                continue;
            }

            if prompt.typed.join("") != event.text {
                continue;
            }

            prompt_completed_events.write(PromptCompletedEvent { entity });

            if settings.fixed {
                continue;
            }

            let new_target = prompt_pool.push_back_pop_front(prompt.clone());

            if let Ok(children) = prompt_children.get(entity) {
                for child in children.iter() {
                    if prompt_texts.get(child).is_ok() {
                        let new_val = if typing_state.help_mode {
                            new_target.typed.join("")
                        } else {
                            new_target.displayed.join("")
                        };

                        // TODO yikes. Is there a better way? Maybe this system should
                        // be split so it can be generic like `update_target_text`.
                        let writer = text_set.p0();
                        reset_target_text(writer, child, &new_val);
                        let writer = text_set.p1();
                        reset_target_text(writer, child, &new_val);
                    }
                }
            }

            prompt.typed.clone_from(&new_target.typed);
            prompt.displayed.clone_from(&new_target.displayed);
        }
    }
}

fn handle_help_mode(
    mut typing_state: ResMut<TypingState>,
    mut help_mode_events: EventReader<HelpModeEvent>,
) {
    for event in help_mode_events.read() {
        typing_state.help_mode = match event {
            HelpModeEvent::Toggle => !typing_state.help_mode,
            HelpModeEvent::Disable => false,
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
            CleanupBeforeNewGame,
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
    query: Query<(&PromptChunks, &PromptSettings)>,
    audio_handles: Res<AudioHandles>,
    audio_settings: Res<AudioSettings>,
) {
    if !state.is_changed() {
        return;
    }

    let mut longest: usize = 0;

    for (target, _) in query.iter().filter(|(_t, s)| !s.disabled) {
        let matched_length = if target.typed.join("").starts_with(&state.buffer) {
            state.buffer.len()
        } else {
            0
        };

        if matched_length > longest {
            longest = matched_length;
        }
    }

    if !audio_settings.mute && state.just_typed_char && longest < state.buffer.len() {
        commands.spawn((
            AudioPlayer(audio_handles.wrong_character.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn update_prompt_text<R: TextRoot>(
    state: Res<TypingState>,
    text_query: Query<(), (With<R>, With<PromptText>)>,
    query: Query<(&PromptChunks, &PromptSettings, &Children)>,
    mut text_set: ParamSet<(TextReader<R>, TextWriter<R>)>,
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
        let mut buf = state.buffer.clone();
        let mut fail = false;

        let render_iter = if state.help_mode {
            target.typed.iter()
        } else {
            target.displayed.iter()
        };

        for (ascii, render) in target.typed.iter().zip(render_iter) {
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
            if text_query.get(child).is_ok() {
                let changed = {
                    let mut reader = text_set.p0();
                    reader.text(child, 0) != matched || reader.text(child, 1) != unmatched
                };

                if changed {
                    let mut writer = text_set.p1();
                    writer.text(child, 0).clone_from(&matched);
                    writer.text(child, 1).clone_from(&unmatched);
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
        target.0.clone_from(&state.buffer);
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
                typing_state.buffer.push_str(s.as_str());
                typing_state.just_typed_char = true;
            } else {
                typing_state.just_typed_char = false;
            }

            match ev.key_code {
                KeyCode::Enter => {
                    let text = typing_state.buffer.clone();

                    typing_state.buffer.clear();
                    typing_submit_events.write(TypingSubmitEvent { text });
                }
                KeyCode::Backspace => {
                    typing_state.buffer.pop();
                }
                KeyCode::Escape => {
                    typing_state.buffer.clear();
                }
                _ => {}
            }
        }
    }
}

fn reset_target_text<R: TextRoot>(mut writer: TextWriter<R>, entity: Entity, val: &String) {
    if let Some(mut section_0) = writer.get_text(entity, 0) {
        section_0.clear();
    }
    if let Some(mut section_1) = writer.get_text(entity, 1) {
        section_1.clone_from(val);
    }
}
