use bevy::prelude::*;

use crate::{
    enemy::AnimationState, loading::FontHandles, ui_color, wave::Waves, AfterUpdate, Currency,
    Goal, HitPoints, TaipoState, FONT_SIZE,
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
        .get_single()
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
) {
    let lost = goal_query
        .get_single()
        .map(|hp| hp.current == 0)
        .unwrap_or(false);

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
            GlobalZIndex(1),
            StateScoped(TaipoState::GameOver),
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
                    parent.spawn((
                        Text::new(if lost {
                            format!("やってない!\n{}円", currency.total_earned)
                        } else {
                            format!("やった!\n{}円", currency.total_earned)
                        }),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextFont {
                            font: font_handles.jptext.clone(),
                            font_size: FONT_SIZE,
                            ..default()
                        },
                        TextColor(if lost {
                            ui_color::BAD_TEXT.into()
                        } else {
                            ui_color::NORMAL_TEXT.into()
                        }),
                    ));
                });
        });
}
