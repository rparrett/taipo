use bevy::math::CompassOctant;
use bevy::prelude::*;

use bevy::input_focus::{
    directional_navigation::{DirectionalNavigationMap, DirectionalNavigationPlugin},
    InputDispatchPlugin, InputFocus, InputFocusVisible,
};

use rand::{prelude::SliceRandom, thread_rng};

use crate::ui::{button, checkbox, checkbox_click, Focusable, BORDER_RADIUS};
use crate::{
    data::{WordList, WordListMenuItem},
    loading::{FontHandles, GameDataHandles, LevelHandles},
    map::{TiledMapBundle, TiledMapHandle},
    typing::TypingTargets,
    ui_color, GameData, TaipoState, TypingTarget, FONT_SIZE_LABEL,
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

    commands.spawn(Camera2d);

    commands.spawn(TiledMapBundle {
        tiled_map: TiledMapHandle(level_handles.one.clone()),
        ..default()
    });

    let game_data = game_data_assets.get(&game_data_handles.game).unwrap();

    let mut focusables = Vec::new();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_self: AlignSelf::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(ui_color::OVERLAY.into()),
            StateScoped(TaipoState::MainMenu),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        padding: UiRect::all(Val::Px(20.)),
                        ..default()
                    },
                    BorderRadius::all(BORDER_RADIUS),
                    BackgroundColor(ui_color::DIALOG_BACKGROUND.into()),
                ))
                .with_children(|parent| {
                    parent.spawn((
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
                    ));

                    for selection in game_data.word_list_menu.iter() {
                        focusables.push(
                            parent
                                .spawn(checkbox(false, &selection.label, &font_handles))
                                // TODO how do we tidy this away?
                                .observe(checkbox_click)
                                .id(),
                        );
                    }
                    focusables.push(
                        parent
                            .spawn(button("Start Game", &font_handles))
                            .observe(start_game_click)
                            .id(),
                    );
                });
        });

    directional_nav_map.add_looping_edges(&focusables, CompassOctant::South);
    input_focus.set(focusables[0]);
}

fn main_menu() {}

fn start_game_click(
    trigger: Trigger<Pointer<Click>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &WordListMenuItem),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<TaipoState>>,
    game_data_handles: Res<GameDataHandles>,
    game_data_assets: Res<Assets<GameData>>,
    word_list_assets: Res<Assets<WordList>>,
    mut typing_targets: ResMut<TypingTargets>,
) {
    info!("start_game_click");
    // let game_data = game_data_assets.get(&game_data_handles.game).unwrap();

    // let mut rng = thread_rng();

    // let mut possible_typing_targets: Vec<TypingTarget> = vec![];
    // for list in &menu_item.word_lists {
    //     let word_list = word_list_assets.get(&game_data.word_lists[list]).unwrap();
    //     possible_typing_targets.extend(word_list.words.clone());
    // }

    // possible_typing_targets.shuffle(&mut rng);
    // typing_targets.possible = possible_typing_targets.into();

    // next_state.set(TaipoState::Spawn);
}
