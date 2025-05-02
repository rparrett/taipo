use bevy::math::CompassOctant;
use bevy::prelude::*;

use bevy::input_focus::{directional_navigation::DirectionalNavigationMap, InputFocus};

use rand::{prelude::SliceRandom, thread_rng};

use crate::ui::modal;
use crate::{
    data::{WordList, WordListMenuItem},
    loading::{FontHandles, GameDataHandles, LevelHandles},
    map::{TiledMapBundle, TiledMapHandle},
    typing::PromptPool,
    ui::{button, checkbox, Checkbox},
    ui_color, GameData, PromptChunks, TaipoState, FONT_SIZE_LABEL,
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(TaipoState::MainMenu), main_menu_startup);

        app.add_systems(Update, main_menu.run_if(in_state(TaipoState::MainMenu)));
    }
}

fn main_menu_startup(
    mut commands: Commands,
    font_handles: Res<FontHandles>,
    game_data_handles: Res<GameDataHandles>,
    game_data_assets: Res<Assets<GameData>>,
    level_handles: Res<LevelHandles>,
    mut directional_nav_map: ResMut<DirectionalNavigationMap>,
    mut input_focus: ResMut<InputFocus>,
) {
    info!("main_menu_startup");

    commands.spawn(TiledMapBundle {
        tiled_map: TiledMapHandle(level_handles.one.clone()),
        ..default()
    });

    let game_data = game_data_assets.get(&game_data_handles.game).unwrap();

    let label = commands
        .spawn((
            Text::new("Select Word Lists"),
            TextFont {
                font: font_handles.jptext.clone(),
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

    let checkboxes = game_data
        .word_list_menu
        .iter()
        .map(|selection| {
            let id = commands
                .spawn((
                    checkbox(false, &selection.label, &font_handles),
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
    focusables.extend(checkboxes);
    focusables.push(start_game_button);

    let mut modal_children = Vec::new();
    modal_children.push(label);
    modal_children.extend(focusables.iter());

    commands.spawn((modal(modal_children), StateScoped(TaipoState::MainMenu)));

    directional_nav_map.add_looping_edges(&focusables, CompassOctant::South);
    input_focus.set(focusables[0]);
}

fn main_menu() {}

fn start_game_click(
    mut trigger: Trigger<Pointer<Click>>,
    checkboxes: Query<(&Checkbox, &WordListMenuItem)>,
    mut next_state: ResMut<NextState<TaipoState>>,
    game_data_handles: Res<GameDataHandles>,
    game_data_assets: Res<Assets<GameData>>,
    word_list_assets: Res<Assets<WordList>>,
    mut prompt_pool: ResMut<PromptPool>,
) {
    trigger.propagate(false);

    let game_data = game_data_assets.get(&game_data_handles.game).unwrap();

    let mut possible_prompts: Vec<PromptChunks> = vec![];

    for (_, menu_item) in checkboxes.iter().filter(|(checkbox, _)| checkbox.0) {
        for list in &menu_item.word_lists {
            let word_list = word_list_assets.get(&game_data.word_lists[list]).unwrap();

            possible_prompts.extend(word_list.words.clone());
        }
    }

    // TODO ensure that there are enough prompts to actually play a game.
    // TODO provide some sort of feedback to the user.
    if possible_prompts.is_empty() {
        return;
    }

    let mut rng = thread_rng();
    possible_prompts.shuffle(&mut rng);
    prompt_pool.possible = possible_prompts.into();

    next_state.set(TaipoState::Spawn);
}
