use bevy::prelude::*;

use crate::{
    loading::{FontHandles, UiTextureHandles},
    tower::{TowerKind, TowerState, TowerStats, TOWER_PRICE},
    typing::{
        TypingTarget, TypingTargetBundle, TypingTargetSettings, TypingTargetText, TypingTargets,
    },
    ui_color::TRANSPARENT_BACKGROUND,
    Action, AfterUpdate, Currency, TaipoState, TowerSelection,
};

pub struct ActionPanelPlugin;

impl Plugin for ActionPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionPanel>();

        // `update_actions_panel` needs to be aware of `TowerStats` components that get queued to
        // spawn in `CoreSet::Update`
        app.add_systems(
            AfterUpdate,
            update_action_panel.run_if(in_state(TaipoState::Playing)),
        );

        app.add_systems(OnEnter(TaipoState::Spawn), setup_action_panel);
    }
}

pub static FONT_SIZE_ACTION_PANEL: f32 = 32.0;

#[derive(Resource, Default)]
pub struct ActionPanel {
    actions: Vec<ActionPanelItem>,
    entities: Vec<Entity>,
    /// Change this field's value to force an action panel update.
    /// TODO: It should be possible now to manually trigger change detection instead.
    pub update: u32,
}

struct ActionPanelItem {
    icon: Handle<Image>,
    target: TypingTarget,
    action: Action,
    visible: bool,
}

#[derive(Component)]
pub struct ActionPanelContainer;

#[derive(Component)]
pub struct ActionPanelItemImage;
#[derive(Component)]
pub struct ActionPanelItemPriceContainer;
#[derive(Component)]
pub struct ActionPanelItemPriceText;

fn setup_action_panel(
    mut commands: Commands,
    mut action_panel: ResMut<ActionPanel>,
    mut typing_targets: ResMut<TypingTargets>,
    ui_texture_handles: ResMut<UiTextureHandles>,
    font_handles: Res<FontHandles>,
) {
    let action_container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexEnd,
                    align_items: AlignItems::FlexEnd,
                    width: Val::Percent(30.0),
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.),
                    top: Val::Px(0.),
                    ..default()
                },
                background_color: TRANSPARENT_BACKGROUND.into(),
                ..default()
            },
            ActionPanelContainer,
        ))
        .id();

    let actions = vec![
        ActionPanelItem {
            icon: ui_texture_handles.coin_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::GenerateMoney,
            visible: true,
        },
        ActionPanelItem {
            icon: ui_texture_handles.shuriken_tower_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::BuildTower(TowerKind::Basic),
            visible: false,
        },
        ActionPanelItem {
            icon: ui_texture_handles.support_tower_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::BuildTower(TowerKind::Support),
            visible: false,
        },
        ActionPanelItem {
            icon: ui_texture_handles.debuff_tower_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::BuildTower(TowerKind::Debuff),
            visible: false,
        },
        ActionPanelItem {
            icon: ui_texture_handles.upgrade_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::UpgradeTower,
            visible: false,
        },
        ActionPanelItem {
            icon: ui_texture_handles.sell_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::SellTower,
            visible: false,
        },
        ActionPanelItem {
            icon: ui_texture_handles.back_ui.clone(),
            target: typing_targets.pop_front(),
            action: Action::UnselectTower,
            visible: false,
        },
    ];

    let entities: Vec<Entity> = actions
        .iter()
        .map(|action| {
            spawn_action_panel_item(
                action,
                action_container,
                &mut commands,
                &font_handles,
                &ui_texture_handles,
            )
        })
        .collect();

    action_panel.actions = actions;
    action_panel.entities = entities;
}

fn spawn_action_panel_item(
    item: &ActionPanelItem,
    container: Entity,
    commands: &mut Commands,
    font_handles: &FontHandles,
    texture_handles: &UiTextureHandles,
) -> Entity {
    let child = commands
        .spawn((
            NodeBundle {
                style: Style {
                    display: if item.visible {
                        Display::Flex
                    } else {
                        Display::None
                    },
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    height: Val::Px(42.0),
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            TypingTargetBundle {
                target: item.target.clone(),
                action: item.action.clone(),
                settings: TypingTargetSettings::default(),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageBundle {
                    style: Style {
                        margin: UiRect {
                            left: Val::Px(5.0),
                            right: Val::Px(5.0),
                            ..default()
                        },
                        height: Val::Px(32.0),
                        ..default()
                    },
                    image: item.icon.clone().into(),
                    ..default()
                },
                ActionPanelItemImage,
            ));
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(0.0),
                            left: Val::Px(2.0),
                            padding: UiRect {
                                left: Val::Px(2.0),
                                right: Val::Px(2.0),
                                ..default()
                            },
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: Val::Px(38.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        background_color: TRANSPARENT_BACKGROUND.into(),
                        ..default()
                    },
                    ActionPanelItemPriceContainer,
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: Style {
                            margin: UiRect {
                                right: Val::Px(2.0),
                                ..default()
                            },
                            width: Val::Px(12.0),
                            height: Val::Px(12.0),
                            ..default()
                        },
                        image: texture_handles.coin_ui.clone().into(),
                        ..default()
                    });
                    parent.spawn((
                        TextBundle {
                            style: Style { ..default() },
                            text: Text::from_section(
                                "0",
                                TextStyle {
                                    font: font_handles.jptext.clone(),
                                    font_size: 16.0, // 16px in this font is just not quite 16px is it?
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        },
                        ActionPanelItemPriceText,
                    ));
                });
            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "".into(),
                                style: TextStyle {
                                    font: font_handles.jptext.clone(),
                                    font_size: FONT_SIZE_ACTION_PANEL,
                                    color: Color::GREEN,
                                },
                            },
                            TextSection {
                                value: item.target.displayed_chunks.join(""),
                                style: TextStyle {
                                    font: font_handles.jptext.clone(),
                                    font_size: FONT_SIZE_ACTION_PANEL,
                                    color: Color::WHITE,
                                },
                            },
                        ],
                        ..default()
                    },
                    ..default()
                },
                TypingTargetText,
            ));
        })
        .id();

    commands.entity(container).add_child(child);

    child
}

fn update_action_panel(
    mut typing_target_query: Query<(&mut TypingTargetSettings, &Children)>,
    mut style_query: Query<&mut Style>,
    mut text_query: Query<&mut Text, (With<TypingTargetText>, Without<ActionPanelItemPriceText>)>,
    mut price_text_query: Query<
        &mut Text,
        (With<ActionPanelItemPriceText>, Without<TypingTargetText>),
    >,
    tower_query: Query<(&TowerState, &TowerKind, &TowerStats)>,
    price_query: Query<(Entity, &Children), With<ActionPanelItemPriceContainer>>,
    (actions, currency, selection): (Res<ActionPanel>, Res<Currency>, Res<TowerSelection>),
) {
    if !actions.is_changed() {
        return;
    }

    info!("update actions");

    for (item, entity) in actions.actions.iter().zip(actions.entities.iter()) {
        let visible = match item.action {
            Action::BuildTower(_) => match selection.selected {
                Some(tower_slot) => tower_query.get(tower_slot).is_err(),
                None => false,
            },
            Action::GenerateMoney => selection.selected.is_none(),
            Action::UnselectTower => selection.selected.is_some(),
            Action::UpgradeTower => match selection.selected {
                Some(tower_slot) => {
                    match tower_query.get(tower_slot) {
                        Ok((_, _, stats)) => {
                            // TODO
                            stats.level < 2
                        }
                        Err(_) => false,
                    }
                }
                None => false,
            },
            Action::SellTower => match selection.selected {
                Some(tower_slot) => tower_query.get(tower_slot).is_ok(),
                None => false,
            },
            _ => false,
        };

        let price = match item.action {
            Action::BuildTower(tower_type) => match tower_type {
                TowerKind::Basic | TowerKind::Support | TowerKind::Debuff => TOWER_PRICE,
            },
            Action::UpgradeTower => match selection.selected {
                Some(tower_slot) => match tower_query.get(tower_slot) {
                    Ok((_, _, stats)) => stats.upgrade_price,
                    Err(_) => 0,
                },
                None => 0,
            },
            _ => 0,
        };

        let disabled = price > currency.current;
        let price_visible = visible && price > 0;

        // visibility

        if let Ok(mut style) = style_query.get_mut(*entity) {
            style.display = if visible {
                Display::Flex
            } else {
                Display::None
            };
        }

        // price

        if let Ok((_, target_children)) = typing_target_query.get(*entity) {
            for target_child in target_children.iter() {
                if let Ok((price_entity, children)) = price_query.get(*target_child) {
                    if let Ok(mut style) = style_query.get_mut(price_entity) {
                        style.display = if price_visible {
                            Display::Flex
                        } else {
                            Display::None
                        };
                    }

                    for child in children.iter() {
                        if let Ok(mut text) = price_text_query.get_mut(*child) {
                            text.sections[0].value = format!("{}", price);
                            text.sections[0].style.color =
                                if disabled { Color::RED } else { Color::WHITE };
                        }
                    }
                }
            }
        }

        // disabledness
        // we could probably roll this into the vis queries at the expense of a headache

        if let Ok((_, target_children)) = typing_target_query.get(*entity) {
            for target_child in target_children.iter() {
                if let Ok(mut text) = text_query.get_mut(*target_child) {
                    text.sections[0].style.color = if disabled { Color::RED } else { Color::GREEN };
                    text.sections[1].style.color = if disabled { Color::RED } else { Color::WHITE };
                }
            }
        }

        // we don't want invisible typing targets to get updated or make
        // sounds or whatever
        if let Ok((mut settings, _)) = typing_target_query.get_mut(*entity) {
            settings.disabled = !visible;
        }
    }
}
