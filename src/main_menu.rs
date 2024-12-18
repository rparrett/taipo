use bevy::prelude::*;

use rand::{prelude::SliceRandom, thread_rng};

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

        app.add_systems(
            Update,
            (main_menu, button_system).run_if(in_state(TaipoState::MainMenu)),
        );

        app.add_systems(OnExit(TaipoState::MainMenu), main_menu_cleanup);
    }
}

#[derive(Component)]
pub struct MainMenuMarker;

fn main_menu_startup(
    mut commands: Commands,
    font_handles: Res<FontHandles>,
    game_data_handles: Res<GameDataHandles>,
    game_data_assets: Res<Assets<GameData>>,
    level_handles: Res<LevelHandles>,
) {
    info!("main_menu_startup");

    commands.spawn(Camera2d);

    commands.spawn(TiledMapBundle {
        tiled_map: TiledMapHandle(level_handles.one.clone()),
        ..default()
    });

    let game_data = game_data_assets.get(&game_data_handles.game).unwrap();

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
            MainMenuMarker,
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
                    BackgroundColor(ui_color::DIALOG_BACKGROUND.into()),
                ))
                .with_children(|parent| {
                    for selection in game_data.word_list_menu.iter() {
                        parent
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(48.0),
                                    margin: UiRect::all(Val::Px(5.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(ui_color::NORMAL_BUTTON.into()),
                                selection.clone(),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(&selection.label),
                                    TextFont {
                                        font: font_handles.jptext.clone(),
                                        font_size: FONT_SIZE_LABEL,
                                        ..default()
                                    },
                                    TextColor(ui_color::BUTTON_TEXT.into()),
                                ));
                            });
                    }
                });
        });
}

fn main_menu() {}

fn main_menu_cleanup(mut commands: Commands, main_menu_query: Query<Entity, With<MainMenuMarker>>) {
    for ent in main_menu_query.iter() {
        commands.entity(ent).despawn_recursive();
    }
}

fn button_system(
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
    for (interaction, mut background_color, menu_item) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background_color = ui_color::PRESSED_BUTTON.into();

                let game_data = game_data_assets.get(&game_data_handles.game).unwrap();

                let mut rng = thread_rng();

                let mut possible_typing_targets: Vec<TypingTarget> = vec![];
                for list in &menu_item.word_lists {
                    let word_list = word_list_assets.get(&game_data.word_lists[list]).unwrap();
                    possible_typing_targets.extend(word_list.words.clone());
                }

                possible_typing_targets.shuffle(&mut rng);
                typing_targets.possible = possible_typing_targets.into();

                next_state.set(TaipoState::Spawn);
            }
            Interaction::Hovered => {
                *background_color = ui_color::HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *background_color = ui_color::NORMAL_BUTTON.into();
            }
        }
    }
}
