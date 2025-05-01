use std::time::Duration;

use bevy::{
    input_focus::{
        directional_navigation::{DirectionalNavigation, DirectionalNavigationPlugin},
        InputDispatchPlugin, InputFocus, InputFocusVisible,
    },
    math::{CompassOctant, FloatOrd},
    picking::{
        backend::HitData,
        pointer::{Location, PointerId},
    },
    platform::collections::HashSet,
    prelude::*,
    render::camera::NormalizedRenderTarget,
};

use crate::{loading::FontHandles, ui_color, with_related::WithRelated, FONT_SIZE_LABEL};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputDispatchPlugin, DirectionalNavigationPlugin));

        app.insert_resource(InputFocusVisible(true));
        app.init_resource::<ActionState>();
        app.init_resource::<DirectionalNavigationBindings>();

        app.add_systems(Update, button_interaction);

        app.add_systems(PreUpdate, (process_inputs, navigate).chain());

        app.add_systems(
            Update,
            (highlight_focused_element, interact_with_focused_button),
        );

        app.add_observer(checkbox_click);
    }
}

pub const BORDER_RADIUS: Val = Val::Px(5.);

#[derive(Component)]
pub struct Checkbox(pub bool);
#[derive(Component)]
pub struct Check;
#[derive(Component)]
pub struct Focusable;

// The indirection between inputs and actions allows us to easily remap inputs
// and handle multiple input sources (keyboard, gamepad, etc.) in our game
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum DirectionalNavigationAction {
    Up,
    Down,
    Left,
    Right,
    Select,
}

#[derive(Resource)]
struct DirectionalNavigationBindings(Vec<(DirectionalNavigationAction, Vec<KeyCode>)>);

impl Default for DirectionalNavigationBindings {
    fn default() -> Self {
        Self(vec![
            (DirectionalNavigationAction::Up, vec![KeyCode::ArrowUp]),
            (DirectionalNavigationAction::Down, vec![KeyCode::ArrowDown]),
            (DirectionalNavigationAction::Left, vec![KeyCode::ArrowLeft]),
            (
                DirectionalNavigationAction::Right,
                vec![KeyCode::ArrowRight],
            ),
            (
                DirectionalNavigationAction::Select,
                vec![KeyCode::Enter, KeyCode::Space],
            ),
        ])
    }
}

// This keeps track of the inputs that are currently being pressed
#[derive(Default, Resource)]
struct ActionState {
    pressed_actions: HashSet<DirectionalNavigationAction>,
}

fn process_inputs(
    mut action_state: ResMut<ActionState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bindings: Res<DirectionalNavigationBindings>,
) {
    // Reset the set of pressed actions each frame
    // to ensure that we only process each action once
    action_state.pressed_actions.clear();

    for (action, keycodes) in &bindings.0 {
        if keycodes
            .iter()
            .any(|keycode| keyboard_input.just_pressed(*keycode))
        {
            action_state.pressed_actions.insert(*action);
        }
    }
}

fn navigate(action_state: Res<ActionState>, mut directional_navigation: DirectionalNavigation) {
    // If the user is pressing both left and right, or up and down,
    // we should not move in either direction.
    let net_east_west = action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Right) as i8
        - action_state
            .pressed_actions
            .contains(&DirectionalNavigationAction::Left) as i8;

    let net_north_south = action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Up) as i8
        - action_state
            .pressed_actions
            .contains(&DirectionalNavigationAction::Down) as i8;

    // Compute the direction that the user is trying to navigate in
    let maybe_direction = match (net_east_west, net_north_south) {
        (0, 0) => None,
        (0, 1) => Some(CompassOctant::North),
        (1, 1) => Some(CompassOctant::NorthEast),
        (1, 0) => Some(CompassOctant::East),
        (1, -1) => Some(CompassOctant::SouthEast),
        (0, -1) => Some(CompassOctant::South),
        (-1, -1) => Some(CompassOctant::SouthWest),
        (-1, 0) => Some(CompassOctant::West),
        (-1, 1) => Some(CompassOctant::NorthWest),
        _ => None,
    };

    if let Some(direction) = maybe_direction {
        // TODO we could add audio/visual feedback here
        let _ = directional_navigation.navigate(direction);
    }
}

fn highlight_focused_element(
    input_focus: Res<InputFocus>,
    input_focus_visible: Res<InputFocusVisible>,
    mut query: Query<(Entity, &mut BorderColor), With<Focusable>>,
) {
    for (entity, mut border_color) in query.iter_mut() {
        if input_focus.0 == Some(entity) && input_focus_visible.0 {
            border_color.0 = ui_color::HOVERED_BUTTON.into();
        } else {
            border_color.0 = Color::NONE;
        }
    }
}

// By sending a Pointer<Click> trigger rather than directly handling button-like interactions,
// we can unify our handling of pointer and keyboard interactions
fn interact_with_focused_button(
    action_state: Res<ActionState>,
    input_focus: Res<InputFocus>,
    mut commands: Commands,
) {
    if action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Select)
    {
        if let Some(focused_entity) = input_focus.0 {
            commands.trigger_targets(
                Pointer::<Click> {
                    target: focused_entity,
                    // We're pretending that we're a mouse
                    pointer_id: PointerId::Mouse,
                    // This field isn't used, so we're just setting it to a placeholder value
                    pointer_location: Location {
                        target: NormalizedRenderTarget::Image(
                            bevy::render::camera::ImageRenderTarget {
                                handle: Handle::default(),
                                scale_factor: FloatOrd(1.0),
                            },
                        ),
                        position: Vec2::ZERO,
                    },
                    event: Click {
                        button: PointerButton::Primary,
                        // This field isn't used, so we're just setting it to a placeholder value
                        hit: HitData {
                            camera: Entity::PLACEHOLDER,
                            depth: 0.0,
                            position: None,
                            normal: None,
                        },
                        duration: Duration::from_secs_f32(0.1),
                    },
                },
                focused_entity,
            );
        }
    }
}

pub fn checkbox_click(
    mut trigger: Trigger<Pointer<Click>>,
    mut background_colors: Query<&mut BackgroundColor, With<Check>>,
    mut checkboxes: Query<&mut Checkbox>,
    children: Query<&Children>,
) {
    let Ok(mut checkbox) = checkboxes.get_mut(trigger.target()) else {
        return;
    };

    checkbox.0 = !checkbox.0;

    for child in children.iter_descendants(trigger.target()) {
        if let Ok(mut background_color) = background_colors.get_mut(child) {
            background_color.0 = if checkbox.0 {
                ui_color::NORMAL_BUTTON.into()
            } else {
                Color::NONE
            };
            break;
        }
    }

    trigger.propagate(false);
}

pub fn checkbox(checked: bool, text: &str, font_handles: &Res<FontHandles>) -> impl Bundle {
    (
        Name::new("checkbox"),
        Checkbox(checked),
        Focusable,
        Node {
            width: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            border: UiRect::all(Val::Px(2.)),
            padding: UiRect::all(Val::Px(5.)),
            ..default()
        },
        BorderRadius::all(BORDER_RADIUS),
        children![
            (
                Check,
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    margin: UiRect::right(Val::Px(5.)),
                    border: UiRect::all(Val::Px(2.)),
                    ..default()
                },
                BorderRadius::all(BORDER_RADIUS),
                BorderColor(ui_color::NORMAL_BUTTON.into()),
                BackgroundColor(if checked {
                    ui_color::NORMAL_BUTTON.into()
                } else {
                    Color::NONE
                })
            ),
            (
                Text::new(text),
                TextFont {
                    font: font_handles.jptext.clone(),
                    font_size: FONT_SIZE_LABEL,
                    ..default()
                },
                TextColor(ui_color::BUTTON_TEXT.into()),
            )
        ],
    )
}

pub fn button(text: &str, font_handles: &Res<FontHandles>) -> impl Bundle {
    (
        Button,
        Focusable,
        Node {
            width: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(2.)),
            padding: UiRect::all(Val::Px(5.)),
            ..default()
        },
        BorderRadius::all(BORDER_RADIUS),
        BackgroundColor(ui_color::NORMAL_BUTTON.into()),
        children![(
            Text::new(text),
            TextFont {
                font: font_handles.jptext.clone(),
                font_size: FONT_SIZE_LABEL,
                ..default()
            },
            TextColor(ui_color::BUTTON_TEXT.into()),
        )],
    )
}

fn button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background_color = ui_color::PRESSED_BUTTON.into();
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

pub fn modal(children: Vec<Entity>) -> impl Bundle {
    (
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
        Children::spawn(Spawn((
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
            Children::spawn(WithRelated(children.into_iter())),
        ))),
    )
}
