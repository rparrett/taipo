use crate::{
    enemy::AnimationState, loading::FontHandles, ui_color, wave::Waves, AfterUpdate, Currency,
    Goal, HitPoints, TaipoState, FONT_SIZE,
};
use bevy::prelude::*;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_game_over.in_schedule(OnEnter(TaipoState::GameOver)));

        app.add_system(
            check_game_over
                .in_base_set(AfterUpdate)
                .run_if(in_state(TaipoState::Playing)),
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
        return;
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
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                justify_content: JustifyContent::Center,
                align_self: AlignSelf::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: ui_color::OVERLAY.into(),
            z_index: ZIndex::Global(1),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        padding: UiRect::all(Val::Px(20.)),
                        ..Default::default()
                    },
                    background_color: ui_color::DIALOG_BACKGROUND.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            if lost {
                                format!("やってない!\n{}円", currency.total_earned)
                            } else {
                                format!("やった!\n{}円", currency.total_earned)
                            },
                            TextStyle {
                                font: font_handles.jptext.clone(),
                                font_size: FONT_SIZE,
                                color: if lost { Color::RED } else { Color::WHITE },
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        ..Default::default()
                    });
                });
        });
}
