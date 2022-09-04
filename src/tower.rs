use crate::{
    bullet, enemy::EnemyKind, layer, typing_target_finished_event, HitPoints, RangeIndicator,
    StatusDownSprite, StatusEffect, StatusEffectKind, StatusEffects, StatusUpSprite, TaipoStage,
    TaipoState, TextureHandles, TowerSelection,
};
use bevy::prelude::*;
pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(TaipoState::Playing)
                .with_system(shoot_enemies)
                // ensure that we process the TowerChanged event in the frame *after*. This adds
                // a one frame delay but prevents us from needing yet another stage.
                .with_system(update_tower_status_effects.before(typing_target_finished_event)),
        );

        app.add_system_set_to_stage(
            TaipoStage::AfterUpdate,
            SystemSet::on_update(TaipoState::Playing)
                // update_actions_panel and update_range_indicator need to be aware of TowerStats components
                // that get queued to spawn in the update stage.)
                .with_system(update_range_indicator)
                // update_tower_appearance needs to detect added TowerStats components
                .with_system(update_tower_appearance)
                // update_tower_status_effect_appearance needs to detect an added or modified StatusEffects
                // component, so it must run in a later stage.
                .with_system(update_tower_status_effect_appearance),
        );
    }
}

pub static TOWER_PRICE: u32 = 20;

#[derive(Bundle, Default)]
pub struct TowerBundle {
    pub kind: TowerKind,
    pub stats: TowerStats,
    pub state: TowerState,
    pub status_effects: StatusEffects,
}
impl TowerBundle {
    pub fn new(kind: TowerKind) -> Self {
        let damage = match kind {
            TowerKind::Basic => 1,
            _ => 0,
        };
        Self {
            stats: TowerStats {
                level: 1,
                range: 128.0,
                damage,
                upgrade_price: 10,
                speed: 1.0,
            },
            state: TowerState {
                timer: Timer::from_seconds(1.0, true),
            },
            kind,
            ..Default::default()
        }
    }
}

#[derive(Component)]
pub struct TowerSprite;
#[derive(Component, Debug, Copy, Clone)]
pub enum TowerKind {
    Basic,
    Support,
    Debuff,
}
impl Default for TowerKind {
    fn default() -> Self {
        Self::Basic
    }
}
#[derive(Component, Default, Debug)]
pub struct TowerStats {
    pub level: u32,
    pub range: f32,
    pub damage: u32,
    pub upgrade_price: u32,
    pub speed: f32,
}
#[derive(Component, Default)]
pub struct TowerState {
    pub timer: Timer,
}

pub struct TowerChangedEvent;

// This currently does not work properly for status effects with timers, but
// we don't have any of those in game yet.
fn update_tower_status_effect_appearance(
    mut commands: Commands,
    query: Query<(Entity, &StatusEffects, &Children), (With<TowerKind>, Changed<StatusEffects>)>,
    up_query: Query<Entity, With<StatusUpSprite>>,
    down_query: Query<Entity, With<StatusDownSprite>>,
    tower_sprite_query: Query<&Transform, With<TowerSprite>>,
    texture_handles: Res<TextureHandles>,
) {
    for (entity, status_effects, children) in query.iter() {
        let down = status_effects.get_max_sub_armor() > 0;
        let up = status_effects.get_total_add_damage() > 0;

        let sprite_transform = children
            .iter()
            .filter_map(|child| tower_sprite_query.get(*child).ok())
            .next()
            .expect("no sprite for tower?");
        let sprite_size = sprite_transform.scale.truncate();

        for child in children.iter() {
            match (down, down_query.get(*child)) {
                (true, Err(_)) => {
                    let down_ent = commands
                        .spawn_bundle(SpriteBundle {
                            texture: texture_handles.status_down.clone(),
                            transform: Transform::from_translation(Vec3::new(
                                sprite_size.x / 2.0 + 6.0,
                                -12.0,
                                layer::HEALTHBAR_BG,
                            )),
                            ..Default::default()
                        })
                        .insert(StatusDownSprite)
                        .id();
                    commands.entity(entity).push_children(&[down_ent]);
                }
                (false, Ok(down_ent)) => {
                    commands.entity(down_ent).despawn_recursive();
                }
                _ => {}
            }
            match (up, up_query.get(*child)) {
                (true, Err(_)) => {
                    let up_ent = commands
                        .spawn_bundle(SpriteBundle {
                            texture: texture_handles.status_up.clone(),
                            transform: Transform::from_translation(Vec3::new(
                                sprite_size.x / 2.0 + 6.0,
                                -12.0,
                                layer::HEALTHBAR_BG,
                            )),
                            ..Default::default()
                        })
                        .insert(StatusUpSprite)
                        .id();
                    commands.entity(entity).push_children(&[up_ent]);
                }
                (false, Ok(up_ent)) => {
                    commands.entity(up_ent).despawn_recursive();
                }
                _ => {}
            }
        }
    }
}

fn update_tower_status_effects(
    mut reader: EventReader<TowerChangedEvent>,
    query: Query<Entity, With<TowerState>>,
    kind_query: Query<&TowerKind>,
    transform_query: Query<&Transform>,
    stats_query: Query<&TowerStats>,
    mut status_query: Query<&mut StatusEffects, With<TowerKind>>,
) {
    // consumes all TowerChangedEvents, which is okay because this is the only
    // system currently making use of those.
    if reader.iter().last().is_none() {
        return;
    }

    let support_towers: Vec<_> = query
        .iter()
        .filter(|entity| matches!(kind_query.get(*entity), Ok(TowerKind::Support)))
        .collect();

    for mut status in status_query.iter_mut() {
        status.0.clear();
    }

    for support_entity in support_towers.iter() {
        for affected_entity in query.iter().filter(|entity| *entity != *support_entity) {
            if let Ok(mut status) = status_query.get_mut(affected_entity) {
                let support_transform = transform_query.get(*support_entity).unwrap();
                let support_stats = stats_query.get(*support_entity).unwrap();
                let transform = transform_query.get(affected_entity).unwrap();

                let dist = transform
                    .translation
                    .truncate()
                    .distance(support_transform.translation.truncate());

                if dist < support_stats.range {
                    status.0.push(StatusEffect {
                        kind: StatusEffectKind::AddDamage(1),
                        timer: None,
                    });
                }
            }
        }
    }
}

fn update_tower_appearance(
    mut commands: Commands,
    sprite_query: Query<Entity, With<TowerSprite>>,
    mut tower_query: Query<(Entity, &TowerStats, &TowerKind, &Children), Changed<TowerStats>>,
    texture_handles: Res<TextureHandles>,
    textures: Res<Assets<Image>>,
) {
    for (parent, stats, tower_type, children) in tower_query.iter_mut() {
        info!("picked up a changed<towerstats>");
        for child in children.iter() {
            if let Ok(ent) = sprite_query.get(*child) {
                commands.entity(ent).despawn();
            }
        }

        let texture_handle = match (tower_type, stats.level) {
            (TowerKind::Basic, 1) => Some(&texture_handles.tower),
            (TowerKind::Basic, 2) => Some(&texture_handles.tower_two),
            (TowerKind::Support, 1) => Some(&texture_handles.support_tower),
            (TowerKind::Support, 2) => Some(&texture_handles.support_tower_two),
            (TowerKind::Debuff, 1) => Some(&texture_handles.debuff_tower),
            (TowerKind::Debuff, 2) => Some(&texture_handles.debuff_tower_two),
            _ => None,
        };

        if let Some(texture_handle) = texture_handle {
            let texture = textures.get(texture_handle).unwrap();

            let new_child = commands
                .spawn_bundle(SpriteBundle {
                    texture: texture_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        (texture.texture_descriptor.size.height / 2) as f32 - 16.0,
                        layer::TOWER,
                    )),
                    ..Default::default()
                })
                .insert(TowerSprite)
                .id();

            commands.entity(parent).push_children(&[new_child]);
        }
    }
}

// Update the range indicator when the tower selection is changed, or when the selected tower's range changes
fn update_range_indicator(
    selection: Res<TowerSelection>,
    mut indicator_query: Query<
        (&mut Transform, &mut Visibility),
        (With<RangeIndicator>, Without<TowerStats>),
    >,
    changed_tower_query: Query<Entity, Changed<TowerStats>>,
    tower_query: Query<(&Transform, &TowerStats), (With<TowerStats>, Without<RangeIndicator>)>,
) {
    if selection.is_changed() && selection.selected.is_none() {
        if let Some((_, mut v)) = indicator_query.iter_mut().next() {
            v.is_visible = false;
        }
    }

    for slot in selection
        .selected
        .into_iter()
        .chain(changed_tower_query.iter())
    {
        if let Ok((tower_t, stats)) = tower_query.get(slot) {
            if let Ok((mut indicator_t, mut indicator_v)) = indicator_query.get_single_mut() {
                indicator_t.translation.x = tower_t.translation.x;
                indicator_t.translation.y = tower_t.translation.y;

                // range is a radius, sprite width is diameter
                indicator_t.scale.x = stats.range * 2.0 / 722.0; // XXX magic sprite scaling factor
                indicator_t.scale.y = stats.range * 2.0 / 722.0; // XXX magic sprite scaling factor

                indicator_v.is_visible = true;
            }
        } else if let Ok((_, mut indicator_v)) = indicator_query.get_single_mut() {
            indicator_v.is_visible = false;
        }
    }
}

fn shoot_enemies(
    mut commands: Commands,
    mut tower_query: Query<(
        &Transform,
        &mut TowerState,
        &TowerStats,
        &TowerKind,
        &StatusEffects,
    )>,
    enemy_query: Query<(Entity, &HitPoints, &Transform), With<EnemyKind>>,
    texture_handles: Res<TextureHandles>,
    time: Res<Time>,
) {
    for (transform, mut tower_state, tower_stats, tower_type, status_effects) in
        tower_query.iter_mut()
    {
        if let TowerKind::Support = *tower_type {
            continue;
        }

        tower_state.timer.tick(time.delta());
        if !tower_state.timer.finished() {
            continue;
        }

        // we are just naively iterating over every enemy right now. at some point we should
        // investigate whether some spatial data structure is useful here. but there is overhead
        // involved in maintaining one and I think it's unlikely that we'd break even with the
        // small amount of enemies and towers we're dealing with here.

        let mut in_range = enemy_query
            .iter()
            .filter(|(_, hp, _)| hp.current > 0)
            .filter(|(_, _, enemy_transform)| {
                let dist = enemy_transform
                    .translation
                    .truncate()
                    .distance(transform.translation.truncate());

                dist <= tower_stats.range
            });

        // right now, possibly coincidentally, this query seems to be iterating in the order that
        // the enemies were spawned.
        //
        // with all enemies current walking at the same speed, that is equivalent to the enemy
        // furthest along the path, which is the default behavior we probably want.
        //
        // other options might be to sort the in-range enemies and select
        // - closest to tower
        // - furthest along path
        // - highest health
        // - lowest health

        if let Some((enemy, _, _)) = in_range.next() {
            let mut bullet_translation = transform.translation;
            bullet_translation.y += 24.0; // XXX magic sprite offset

            let texture = match tower_type {
                TowerKind::Basic => texture_handles.bullet_shuriken.clone(),
                TowerKind::Debuff => texture_handles.bullet_debuff.clone(),
                _ => panic!(),
            };

            let status = match tower_type {
                TowerKind::Debuff => Some(StatusEffect {
                    kind: StatusEffectKind::SubArmor(2),
                    timer: None,
                }),
                _ => None,
            };

            let damage: u32 = tower_stats
                .damage
                .saturating_add(status_effects.get_total_add_damage());

            bullet::spawn(
                bullet_translation,
                enemy,
                damage,
                100.0,
                status,
                &mut commands,
                texture,
            );
        }
    }
}
