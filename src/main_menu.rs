use bevy::math::CompassOctant;
use bevy::prelude::*;

use bevy::input_focus::{directional_navigation::DirectionalNavigationMap, InputFocus};

use rand::{prelude::SliceRandom, thread_rng};

use crate::{
    data::{WordList, WordListMenuItem},
    loading::{AudioHandles, FontHandles, GameDataHandles, LevelHandles},
    map::{TiledMapBundle, TiledMapHandle},
    typing::PromptPool,
    ui::{button, checkbox, modal, Checkbox},
    ui_color, GameData, PromptChunks, SelectedWordLists, TaipoState, Volume, FONT_SIZE_LABEL,
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(TaipoState::MainMenu), setup);
        app.add_systems(
            Update,
            update_volume_text
                .run_if(in_state(TaipoState::MainMenu).and(resource_changed::<Volume>)),
        );
    }
}

#[derive(Component)]
struct VolumeButton;

fn setup(
    mut commands: Commands,
    font_handles: Res<FontHandles>,
    game_data_handles: Res<GameDataHandles>,
    game_data_assets: Res<Assets<GameData>>,
    level_handles: Res<LevelHandles>,
    mut directional_nav_map: ResMut<DirectionalNavigationMap>,
    mut input_focus: ResMut<InputFocus>,
    selected_word_lists: Res<SelectedWordLists>,
    volume: Res<Volume>,
) {
    info!("main_menu setup");

    commands.spawn(TiledMapBundle {
        tiled_map: TiledMapHandle(level_handles.one.clone()),
        ..default()
    });

    let game_data = game_data_assets.get(&game_data_handles.game).unwrap();

    let settings_label = commands
        .spawn((
            Text::new("Settings"),
            TextFont {
                font: font_handles.jp_text.clone(),
                font_size: FONT_SIZE_LABEL,
                ..default()
            },
            TextColor(ui_color::BUTTON_TEXT.into()),
            Node {
                margin: UiRect::bottom(Val::Px(10.)),
                ..default()
            },
        ))
        .id();

    let volume_button = commands
        .spawn((
            button(format!("Volume {}%", volume.0), &font_handles),
            VolumeButton,
        ))
        .observe(volume_click)
        .id();

    let word_list_label = commands
        .spawn((
            Text::new("Select Word Lists"),
            TextFont {
                font: font_handles.jp_text.clone(),
                font_size: FONT_SIZE_LABEL,
                ..default()
            },
            TextColor(ui_color::BUTTON_TEXT.into()),
            Node {
                margin: UiRect::vertical(Val::Px(10.)),
                ..default()
            },
        ))
        .id();

    let checkboxes = game_data
        .word_list_menu
        .iter()
        .map(|selection| {
            let id = commands
                .spawn((
                    checkbox(
                        selected_word_lists.0.contains(&selection.word_list),
                        &selection.label,
                        &font_handles,
                    ),
                    selection.clone(),
                ))
                .id();
            id
        })
        .collect::<Vec<_>>();

    let start_game_button = commands
        .spawn(button("Start Game", &font_handles))
        .observe(start_game_click)
        .id();

    let mut focusables = Vec::new();
    focusables.push(volume_button);
    focusables.extend(checkboxes.iter());
    focusables.push(start_game_button);

    let mut modal_children = Vec::new();
    modal_children.push(settings_label);
    modal_children.push(volume_button);
    modal_children.push(word_list_label);
    modal_children.extend(checkboxes.iter());
    modal_children.push(start_game_button);

    commands.spawn((modal(modal_children), StateScoped(TaipoState::MainMenu)));

    directional_nav_map.add_looping_edges(&focusables, CompassOctant::South);
    input_focus.set(focusables[1]);
}

fn start_game_click(
    mut trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    checkboxes: Query<(&Checkbox, &WordListMenuItem)>,
    mut next_state: ResMut<NextState<TaipoState>>,
    game_data_handles: Res<GameDataHandles>,
    game_data_assets: Res<Assets<GameData>>,
    word_list_assets: Res<Assets<WordList>>,
    mut prompt_pool: ResMut<PromptPool>,
    mut selected_word_lists: ResMut<SelectedWordLists>,
    audio_handles: Res<AudioHandles>,
) {
    trigger.propagate(false);

    let game_data = game_data_assets.get(&game_data_handles.game).unwrap();

    selected_word_lists.0.clear();

    let mut possible_prompts: Vec<PromptChunks> = vec![];

    for (_, menu_item) in checkboxes.iter().filter(|(checkbox, _)| checkbox.0) {
        let word_list = word_list_assets
            .get(&game_data.word_lists[&menu_item.word_list])
            .unwrap();

        possible_prompts.extend(word_list.words.clone());

        selected_word_lists.0.insert(menu_item.word_list.clone());
    }

    // TODO ensure that there are enough prompts to actually play a game.
    if possible_prompts.is_empty() {
        commands.spawn((
            AudioPlayer(audio_handles.wrong_character.clone()),
            PlaybackSettings::DESPAWN,
        ));

        return;
    }

    let mut rng = thread_rng();
    possible_prompts.shuffle(&mut rng);
    prompt_pool.possible = possible_prompts.into();

    next_state.set(TaipoState::Spawn);
}

fn volume_click(
    mut trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut volume: ResMut<Volume>,
    audio_handles: Res<AudioHandles>,
    mut global_volume: ResMut<GlobalVolume>,
) {
    volume.0 = volume.next();
    global_volume.volume = bevy::audio::Volume::Linear(volume.0 as f32 / 100.0);

    commands.spawn((
        AudioPlayer(audio_handles.wrong_character.clone()),
        PlaybackSettings::DESPAWN,
    ));

    trigger.propagate(false);
}

fn update_volume_text(
    volume: Res<Volume>,
    buttons: Query<&Children, With<VolumeButton>>,
    mut texts: Query<&mut Text>,
) {
    for children in &buttons {
        let mut texts_iter = texts.iter_many_mut(children);
        while let Some(mut text) = texts_iter.fetch_next() {
            text.0.clone_from(&format!("Volume {}%", volume.0));
        }
    }
}
