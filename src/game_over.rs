use bevy::{
    input_focus::{directional_navigation::DirectionalNavigationMap, InputFocus},
    prelude::*,
};

use crate::{
    enemy::AnimationState,
    loading::FontHandles,
    ui::{button, modal, Focusable},
    ui_color,
    wave::Waves,
    AfterUpdate, Currency, Goal, HitPoints, TaipoState, FONT_SIZE,
};
pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(TaipoState::GameOver), spawn_game_over);

        app.add_systems(
            AfterUpdate,
            check_game_over.run_if(in_state(TaipoState::Playing)),
        );

        // TODO maybe keep doing enemy movement and animations?
    }
}

fn check_game_over(
    query: Query<&AnimationState>,
    goal_query: Query<&HitPoints, With<Goal>>,
    waves: Res<Waves>,
    mut next_state: ResMut<NextState<TaipoState>>,
) {
    let lost = goal_query
        .single()
        .map(|hp| hp.current == 0)
        .unwrap_or(false);

    if lost {
        next_state.set(TaipoState::GameOver);
        return;
    }

    let won =
        waves.current().is_none() && query.iter().all(|x| matches!(x, AnimationState::Corpse));

    if won {
        next_state.set(TaipoState::GameOver);
    }
}

fn spawn_game_over(
    mut commands: Commands,
    font_handles: Res<FontHandles>,
    currency: Res<Currency>,
    goal_query: Query<&HitPoints, With<Goal>>,
    mut directional_nav_map: ResMut<DirectionalNavigationMap>,
    mut input_focus: ResMut<InputFocus>,
) {
    let lost = goal_query
        .single()
        .map(|hp| hp.current == 0)
        .unwrap_or(false);

    let font = TextFont {
        font: font_handles.jptext.clone(),
        font_size: FONT_SIZE,
        ..default()
    };

    let text = commands
        .spawn((
            Text::new(if lost {
                "やってない!"
            } else {
                "やった!"
            }),
            font.clone(),
            TextColor(if lost {
                ui_color::BAD_TEXT.into()
            } else {
                ui_color::NORMAL_TEXT.into()
            }),
            Node {
                margin: UiRect::bottom(Val::Px(10.)),
                ..default()
            },
        ))
        .id();

    let currency_text = commands
        .spawn((
            Text::new(format!("{}円 獲得", currency.total_earned)),
            font,
            TextColor(ui_color::NORMAL_TEXT.into()),
            Node {
                margin: UiRect::bottom(Val::Px(10.)),
                ..default()
            },
        ))
        .id();

    let button = commands
        .spawn(button("Back To Main Menu", &font_handles))
        .observe(back_button_click)
        .id();

    commands.spawn((
        modal(vec![text, currency_text, button]),
        StateScoped(TaipoState::GameOver),
    ));

    // Deliberately not setting InputFocus so the user doesn't accidentally exit the
    // Game Over screen while typing.
    input_focus.clear();
    let dummy = commands.spawn(Focusable).id();
    // Directional navigation does nothing if there is no focus, so create a one-way
    // edge from a dummy to our button.
    input_focus.set(dummy);
    directional_nav_map.add_edge(dummy, button, bevy::math::CompassOctant::South);
}

fn back_button_click(
    mut trigger: Trigger<Pointer<Click>>,
    mut next_state: ResMut<NextState<TaipoState>>,
) {
    next_state.set(TaipoState::MainMenu);
    trigger.propagate(false);
}
