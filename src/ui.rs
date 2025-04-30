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

use crate::{loading::FontHandles, ui_color, FONT_SIZE_LABEL};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputDispatchPlugin, DirectionalNavigationPlugin));

        app.insert_resource(InputFocusVisible(true));
        app.init_resource::<ActionState>();

        app.add_systems(Update, button_interaction);

        app.add_systems(PreUpdate, (process_inputs, navigate).chain());

        app.add_systems(
            Update,
            (
                // We need to show which button is currently focused
                highlight_focused_element,
                // Pressing the "Interact" button while we have a focused element should simulate a click
                interact_with_focused_button,
                // We're doing a tiny animation when the button is interacted with,
                // so we need a timer and a polling mechanism to reset it
                //reset_button_after_interaction,
            ),
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
#[derive(Debug, PartialEq, Eq, Hash)]
enum DirectionalNavigationAction {
    Up,
    Down,
    Left,
    Right,
    Select,
}

impl DirectionalNavigationAction {
    fn variants() -> Vec<Self> {
        vec![
            DirectionalNavigationAction::Up,
            DirectionalNavigationAction::Down,
            DirectionalNavigationAction::Left,
            DirectionalNavigationAction::Right,
            DirectionalNavigationAction::Select,
        ]
    }

    fn keycode(&self) -> KeyCode {
        match self {
            DirectionalNavigationAction::Up => KeyCode::ArrowUp,
            DirectionalNavigationAction::Down => KeyCode::ArrowDown,
            DirectionalNavigationAction::Left => KeyCode::ArrowLeft,
            DirectionalNavigationAction::Right => KeyCode::ArrowRight,
            DirectionalNavigationAction::Select => KeyCode::Enter,
        }
    }

    fn gamepad_button(&self) -> GamepadButton {
        match self {
            DirectionalNavigationAction::Up => GamepadButton::DPadUp,
            DirectionalNavigationAction::Down => GamepadButton::DPadDown,
            DirectionalNavigationAction::Left => GamepadButton::DPadLeft,
            DirectionalNavigationAction::Right => GamepadButton::DPadRight,
            // This is the "A" button on an Xbox controller,
            // and is conventionally used as the "Select" / "Interact" button in many games
            DirectionalNavigationAction::Select => GamepadButton::South,
        }
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
    gamepad_input: Query<&Gamepad>,
) {
    // Reset the set of pressed actions each frame
    // to ensure that we only process each action once
    action_state.pressed_actions.clear();

    for action in DirectionalNavigationAction::variants() {
        // Use just_pressed to ensure that we only process each action once
        // for each time it is pressed
        if keyboard_input.just_pressed(action.keycode()) {
            action_state.pressed_actions.insert(action);
        }
    }

    // We're treating this like a single-player game:
    // if multiple gamepads are connected, we don't care which one is being used
    for gamepad in gamepad_input.iter() {
        for action in DirectionalNavigationAction::variants() {
            // Unlike keyboard input, gamepads are bound to a specific controller
            if gamepad.just_pressed(action.gamepad_button()) {
                action_state.pressed_actions.insert(action);
            }
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
        match directional_navigation.navigate(direction) {
            // In a real game, you would likely want to play a sound or show a visual effect
            // on both successful and unsuccessful navigation attempts
            Ok(entity) => {
                println!("Navigated {direction:?} successfully. {entity} is now focused.");
            }
            Err(e) => println!("Navigation failed: {e}"),
        }
    }
}

fn highlight_focused_element(
    input_focus: Res<InputFocus>,
    // While this isn't strictly needed for the example,
    // we're demonstrating how to be a good citizen by respecting the `InputFocusVisible` resource.
    input_focus_visible: Res<InputFocusVisible>,
    mut query: Query<(Entity, &mut BorderColor), With<Focusable>>,
) {
    for (entity, mut border_color) in query.iter_mut() {
        if input_focus.0 == Some(entity) && input_focus_visible.0 {
            // Don't change the border size / radius here,
            // as it would result in wiggling buttons when they are focused
            border_color.0 = ui_color::HOVERED_BUTTON.into();
        } else {
            border_color.0 = Color::NONE;
        }
    }
}

// By sending a Pointer<Click> trigger rather than directly handling button-like interactions,
// we can unify our handling of pointer and keyboard/gamepad interactions
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
