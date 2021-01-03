use bevy::prelude::*;
use bevy_tiled_prototype::TiledMapCenter;
use bullet::BulletPlugin;
use healthbar::HealthBarPlugin;
use rand::{prelude::SliceRandom, thread_rng, Rng};
use typing::{
    TypingPlugin, TypingTarget, TypingTargetChangeEvent, TypingTargetContainer,
    TypingTargetFinishedEvent, TypingTargetImage, TypingTargetSpawnEvent,
};

use std::collections::VecDeque;

#[macro_use]
extern crate anyhow;

mod bullet;
mod data;
mod healthbar;
mod typing;

static TOWER_PRICE: u32 = 10;
pub static FONT_SIZE: f32 = 32.0;

#[derive(Default)]
pub struct GameState {
    primary_currency: u32,
    selected: Option<Entity>,
    possible_typing_targets: VecDeque<TypingTarget>,
    // Just so we can keep these in the correct order
    tower_slots: Vec<Entity>,
}

struct CurrencyDisplay;
struct CooldownTimerDisplay;
struct CooldownTimerTimer(Timer);

struct TowerSlot {
    texture_ui: Handle<Texture>,
}

#[derive(Debug)]
enum TowerType {
    Basic,
}

#[derive(Default)]
struct TowerState {
    level: u32,
    range: f32,
    timer: Timer,
}

struct Reticle;

struct UpdateActionsEvent;

#[derive(Default)]
struct TextureHandles {
    tower_slot_ui: Vec<Handle<Texture>>,
    coin_ui: Handle<Texture>,
    back_ui: Handle<Texture>,
    tower: Handle<Texture>,
    tower_ui: Handle<Texture>,
}

#[derive(Default)]
struct FontHandles {
    jptext: Handle<Font>,
}

#[derive(Clone)]
enum Action {
    SelectTower(Entity),
    GenerateMoney,
    Back,
    BuildBasicTower,
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Default for Direction {
    fn default() -> Self {
        Direction::Right
    }
}

#[derive(Debug)]
enum AnimationState {
    Idle,
    Walking,
    Attacking,
    Corpse,
}
impl Default for AnimationState {
    fn default() -> Self {
        AnimationState::Idle
    }
}

#[derive(Default, Debug)]
struct EnemyState {
    facing: Direction,
    state: AnimationState,
    tick: u32,
    path: Vec<Vec2>,
    path_index: usize,
}

struct Skeleton;

struct Wave {
    path: Vec<Vec2>,
    enemy: String,
    num: usize,
    time: f32,
}
impl Default for Wave {
    fn default() -> Self {
        Wave {
            path: vec![],
            enemy: "Skeleton".to_string(),
            num: 10,
            time: 3.0,
        }
    }
}

struct HitPoints {
    current: u32,
    max: u32,
}
impl Default for HitPoints {
    fn default() -> Self {
        HitPoints { current: 1, max: 1 }
    }
}

struct Waves {
    current: usize,
    spawn_timer: Timer,
    cooldown_timer: Timer,
    spawned: usize,
    waves: Vec<Wave>,
}
impl Default for Waves {
    fn default() -> Self {
        Waves {
            current: 0,
            spawn_timer: Timer::from_seconds(1.0, true),
            cooldown_timer: Timer::from_seconds(30.0, false),
            spawned: 0,
            waves: vec![],
        }
    }
}

fn update_actions(
    commands: &mut Commands,
    game_state: Res<GameState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(Entity, &Children), With<TypingTarget>>,
    container_query: Query<&Children, With<TypingTargetContainer>>,
    tower_slot_query: Query<&TowerSlot>,
    tower_type_query: Query<&TowerType>,
    image_query: Query<Entity, With<TypingTargetImage>>,
    mut style_query: Query<&mut Style>,
    mut visible_query: Query<&mut Visible>,
    events: Res<Events<UpdateActionsEvent>>,
    mut reader: Local<EventReader<UpdateActionsEvent>>,
    texture_handles: Res<TextureHandles>,
) {
    for _ in reader.iter(&events) {
        info!("processing UpdateActionsEvent");

        let mut other = vec![];

        if let Some(selected) = game_state.selected {
            if tower_type_query.get(selected).is_err() {
                other.push((texture_handles.tower_ui.clone(), Action::BuildBasicTower));
            }

            other.push((texture_handles.back_ui.clone(), Action::Back));
        } else {
            other.push((texture_handles.coin_ui.clone(), Action::GenerateMoney));
        }

        let other_iter = other.iter().cloned();

        let mut action_iter = game_state
            .tower_slots
            .iter()
            .cloned()
            .filter(|_| game_state.selected.is_none())
            .map(|ent| {
                (
                    tower_slot_query.get(ent).unwrap().texture_ui.clone(),
                    Action::SelectTower(ent.clone()),
                )
            })
            .chain(other_iter);

        for container_children in container_query.iter() {
            for container in container_children.iter() {
                for (entity, target_children) in query.get_mut(*container) {
                    commands.remove_one::<Action>(entity);

                    for mut style in style_query.get_mut(entity) {
                        style.display = Display::None;
                    }

                    // find any TypingTargetImages inside this particular
                    // target and destroy them.

                    for target_child in target_children.iter() {
                        for image in image_query.get(*target_child) {
                            commands.despawn_recursive(image);
                        }

                        // Workaround for #838/#1135
                        for mut child_visible in visible_query.get_mut(*target_child) {
                            child_visible.is_visible = false;
                        }
                    }

                    if let Some((texture, action)) = action_iter.next() {
                        for mut style in style_query.get_mut(entity) {
                            style.display = Display::Flex;
                        }

                        // Workaround for #838/#1135
                        for target_child in target_children.iter() {
                            for mut child_visible in visible_query.get_mut(*target_child) {
                                child_visible.is_visible = true;
                            }
                        }

                        commands.insert_one(entity, action.clone());

                        // add an image back

                        let child = commands
                            .spawn(ImageBundle {
                                style: Style {
                                    margin: Rect {
                                        left: Val::Px(5.0),
                                        right: Val::Px(5.0),
                                        ..Default::default()
                                    },
                                    size: Size::new(Val::Auto, Val::Px(32.0)),
                                    ..Default::default()
                                },
                                // can I somehow get this from the sprite sheet? naively tossing a
                                // spritesheetbundle here instead of an imagebundle seems to panic.
                                material: materials.add(texture.into()),
                                ..Default::default()
                            })
                            .with(TypingTargetImage)
                            .current_entity()
                            .unwrap();

                        commands.insert_children(entity, 0, &[child]);
                    }
                }
            }
        }
    }
}

fn typing_target_finished(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    mut reader: Local<EventReader<TypingTargetFinishedEvent>>,
    typing_target_finished_events: Res<Events<TypingTargetFinishedEvent>>,
    mut typing_target_change_events: ResMut<Events<TypingTargetChangeEvent>>,
    mut update_actions_events: ResMut<Events<UpdateActionsEvent>>,
    action_query: Query<&Action>,
    mut reticle_query: Query<&mut Transform, With<Reticle>>,
    tower_transform_query: Query<&Transform, With<TowerSlot>>,
    texture_handles: Res<TextureHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in reader.iter(&typing_target_finished_events) {
        game_state
            .possible_typing_targets
            .push_back(event.target.clone());
        let target = game_state
            .possible_typing_targets
            .pop_front()
            .unwrap()
            .clone();
        typing_target_change_events.send(TypingTargetChangeEvent {
            entity: event.entity,
            target: target.clone(),
        });
        info!("new target: {}", target.ascii.join(""));

        for action in action_query.get(event.entity) {
            info!("there is some sort of action");
            if let Action::GenerateMoney = *action {
                info!("processing a GenerateMoney action");
                game_state.primary_currency += 1;
            } else if let Action::SelectTower(tower) = *action {
                info!("processing a SelectTower action");
                game_state.selected = Some(tower);
            } else if let Action::Back = *action {
                info!("processing a Back action");
                game_state.selected = None;
            } else if let Action::BuildBasicTower = *action {
                if game_state.primary_currency < TOWER_PRICE {
                    continue;
                }
                game_state.primary_currency -= TOWER_PRICE;

                if let Some(tower) = game_state.selected {
                    for tower_transform in tower_transform_query.get(tower) {
                        info!(
                            "sending tower off to {} {} {}",
                            tower_transform.translation.x,
                            tower_transform.translation.y + 16.0,
                            20.0
                        );
                        commands.insert_one(
                            tower,
                            TowerState {
                                level: 1,
                                range: 200.0,
                                timer: Timer::from_seconds(1.0, true),
                            },
                        );
                        commands.insert_one(tower, TowerType::Basic);

                        let child = commands
                            .spawn(SpriteBundle {
                                material: materials.add(texture_handles.tower.clone().into()),
                                // Odd y value because the bottom of the sprite is not correctly
                                // positioned. Odd z value because we want to be above tiles but
                                // below the reticle.
                                transform: Transform::from_translation(Vec3::new(0.0, 20.0, 10.0)),
                                ..Default::default()
                            })
                            .current_entity()
                            .unwrap();

                        commands.insert_children(tower, 0, &[child]);
                    }
                }
            }
        }

        for mut reticle_transform in reticle_query.iter_mut() {
            if let Some(tower) = game_state.selected {
                for transform in tower_transform_query.get(tower) {
                    info!(
                        "sending reticle off to {} {} {}",
                        transform.translation.x, transform.translation.y, 20.0
                    );
                    reticle_transform.translation.x = transform.translation.x;
                    reticle_transform.translation.y = transform.translation.y;
                    reticle_transform.translation.z = 20.0;
                }
            } else {
                info!("hiding reticle");
                reticle_transform.translation.z = -1.0;
            }
        }

        update_actions_events.send(UpdateActionsEvent);
    }
}

fn animate_reticle(
    time: Res<Time>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite), With<Reticle>>,
) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            sprite.index += 1;
            if sprite.index >= 30 {
                sprite.index = 16;
            }
        }
    }
}

fn animate_skeleton(
    time: Res<Time>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &mut EnemyState), With<Skeleton>>,
) {
    for (mut timer, mut sprite, mut state) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            // TODO there's really more to these animations than just cycling
            // through the frames at some paticular rate.
            let (start, end, modulus) = match (&state.state, &state.facing) {
                (AnimationState::Walking, Direction::Up) => (17, 20, 1),
                (AnimationState::Walking, Direction::Down) => (29, 32, 1),
                // oh god how do I flip things? seems like I have to
                // rotate 180 over y?
                (AnimationState::Walking, Direction::Left) => (4, 7, 1),
                (AnimationState::Walking, Direction::Right) => (4, 7, 1),
                (AnimationState::Idle, Direction::Up) => (20, 22, 20),
                (AnimationState::Idle, Direction::Down) => (30, 32, 20),
                // TODO flip
                (AnimationState::Idle, Direction::Left) => (8, 9, 20),
                (AnimationState::Idle, Direction::Right) => (8, 9, 20),
                (AnimationState::Attacking, Direction::Up) => (12, 14, 2),
                (AnimationState::Attacking, Direction::Down) => (24, 26, 21),
                // TODO flip
                (AnimationState::Attacking, Direction::Left) => (0, 2, 2),
                (AnimationState::Attacking, Direction::Right) => (0, 2, 2),
                // TODO there is no corpse? wasn't there one in the tilemap?
                // We can pretend with one of the idle-up frames
                (AnimationState::Corpse, _) => (21, 21, 1),
            };

            state.tick += 1;
            if state.tick % modulus == 0 {
                sprite.index += 1;
            }
            if sprite.index < start || sprite.index > end {
                sprite.index = start
            }
        }
    }
}

fn move_enemies(time: Res<Time>, mut query: Query<(&mut EnemyState, &mut Transform)>) {
    for (mut state, mut transform) in query.iter_mut() {
        if state.path_index >= state.path.len() - 1 {
            continue;
        }

        if let AnimationState::Idle = state.state {
            state.state = AnimationState::Walking;
        }

        if let AnimationState::Corpse = state.state {
            continue;
        }

        let next = Vec2::extend(
            state.path.get(state.path_index + 1).unwrap().clone(),
            transform.translation.z,
        );
        let d = transform.translation.distance(next);

        let speed = 20.0;
        let step = speed * time.delta_seconds();

        if step > d {
            transform.translation.x = next.x;
            transform.translation.y = next.y;
            state.path_index += 1;

            if let Some(next) = state.path.get(state.path_index + 1) {
                let dx = next.x - transform.translation.x;
                let dy = next.y - transform.translation.y;

                // this probably works fine while we're moving
                // orthogonally
                if dx > 0.1 {
                    state.facing = Direction::Right;
                } else if dx < -0.1 {
                    state.facing = Direction::Left;
                } else if dy > 0.1 {
                    state.facing = Direction::Up;
                } else if dy < -0.1 {
                    state.facing = Direction::Down;
                }
            } else {
                state.state = AnimationState::Attacking;
            }

            continue;
        }

        transform.translation.x += step / d * (next.x - transform.translation.x);
        transform.translation.y += step / d * (next.y - transform.translation.y);
    }
}

fn spawn_enemies(
    commands: &mut Commands,
    time: Res<Time>,
    mut waves: ResMut<Waves>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    if waves.waves.len() <= waves.current {
        return;
    }

    waves.cooldown_timer.tick(time.delta_seconds());
    if !waves.cooldown_timer.finished() {
        return;
    }

    waves.spawn_timer.tick(time.delta_seconds());

    let (wave_time, wave_num) = {
        let wave = waves.waves.get(waves.current).unwrap();
        (wave.time.clone(), wave.num.clone())
    };

    // immediately spawn the first enemy and start the timer
    let spawn = if waves.spawned == 0 {
        waves.spawn_timer.set_duration(wave_time);
        waves.spawn_timer.reset();
        true
    } else if waves.spawn_timer.just_finished() {
        true
    } else {
        false
    };

    if spawn {
        let skel_texture_handle = asset_server.load("textures/skeleton.png");
        let skel_texture_atlas =
            TextureAtlas::from_grid(skel_texture_handle, Vec2::new(32.0, 32.0), 4, 9);
        let skel_texture_atlas_handle = texture_atlases.add(skel_texture_atlas);

        let path = waves.waves.get(waves.current).unwrap().path.clone();
        let point = path.get(0).unwrap();

        let entity = commands
            .spawn(SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(point.x, point.y, 10.0)),
                sprite: TextureAtlasSprite {
                    index: 0,
                    ..Default::default()
                },
                texture_atlas: skel_texture_atlas_handle,
                ..Default::default()
            })
            .with(Timer::from_seconds(0.1, true))
            .with(Skeleton)
            .with(EnemyState {
                path,
                ..Default::default()
            })
            .with(HitPoints { current: 5, max: 5 })
            .current_entity()
            .unwrap();

        healthbar::spawn(entity, commands, materials, Vec2::new(16.0, 2.0));

        waves.spawned += 1
    }

    // that was the last enemy
    if waves.spawned == wave_num {
        waves.current += 1;
        waves.spawned = 0;
    }
}

fn update_timer_display(
    time: Res<Time>,
    mut timer: ResMut<CooldownTimerTimer>,
    mut query: Query<&mut Text, With<CooldownTimerDisplay>>,
    waves: Res<Waves>,
) {
    timer.0.tick(time.delta_seconds());
    if !timer.0.finished() {
        return;
    }

    for mut text in query.iter_mut() {
        let val = f32::max(
            0.0,
            waves.cooldown_timer.duration() - waves.cooldown_timer.elapsed(),
        );

        text.value = format!("{:.1}", val);
    }
}

fn shoot_enemies(
    time: Res<Time>,
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tower_query: Query<(&Transform, &mut TowerState, &TowerType)>,
    enemy_query: Query<(Entity, &HitPoints, &Transform), With<EnemyState>>,
) {
    for (transform, mut tower_state, _tower_type) in tower_query.iter_mut() {
        tower_state.timer.tick(time.delta_seconds());
        if !tower_state.timer.finished() {
            continue;
        }

        // TODO any ol' enemy is good enough for now, but we'll probably want targetting modes
        // - "enemy least far/furthest far along the path that is in range"
        // - "enemy with least/most hp that is in range"
        //
        // With the amount of enemies and tower we'll be dealing with, some fancy spatial data
        // structure probably isn't super impactful though.

        for (enemy, hp, enemy_transform) in enemy_query.iter() {
            if hp.current <= 0 {
                continue;
            }

            let d = enemy_transform
                .translation
                .truncate()
                .distance(transform.translation.truncate());

            if d > tower_state.range {
                continue;
            }

            bullet::spawn(
                transform.translation,
                enemy,
                1,
                100.0,
                commands,
                &mut materials,
            );
            break;
        }
    }
}

fn update_currency_display(
    mut currency_display_query: Query<&mut Text, With<CurrencyDisplay>>,
    game_state: ChangedRes<GameState>,
) {
    for mut target in currency_display_query.iter_mut() {
        target.value = format!("{}", game_state.primary_currency);
    }
}

fn startup_system(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_state: ResMut<GameState>,
    mut typing_target_spawn_events: ResMut<Events<TypingTargetSpawnEvent>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut texture_handles: ResMut<TextureHandles>,
    mut font_handles: ResMut<FontHandles>,
) {
    info!("startup");

    // Would prefer to reuse an rng. Can we do that?
    let mut rng = thread_rng();

    let texture_handle = asset_server.load("textures/main.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 16, 16);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    font_handles.jptext = asset_server.load("fonts/NotoSansJP-Light.otf");

    // Also we need all these loose textures because UI doesn't speak TextureAtlas

    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_a.png"));
    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_b.png"));
    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_c.png"));
    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_d.png"));
    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_e.png"));
    texture_handles
        .tower_slot_ui
        .push(asset_server.load("textures/tower_slot_ui_f.png"));
    texture_handles.coin_ui = asset_server.load("textures/coin.png");
    texture_handles.back_ui = asset_server.load("textures/back_ui.png");
    texture_handles.tower_ui = asset_server.load("textures/tower_ui.png");

    // And these because they don't fit on the grid...
    texture_handles.tower = asset_server.load("textures/tower.png");

    commands
        // 2d camera
        .spawn(CameraUiBundle::default())
        .spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(0.),
                    top: Val::Px(0.),
                    ..Default::default()
                },
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                size: Size::new(Val::Auto, Val::Px(42.0)),
                ..Default::default()
            },
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.50).into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    margin: Rect {
                        left: Val::Px(5.0),
                        ..Default::default()
                    },
                    size: Size::new(Val::Auto, Val::Px(32.0)),
                    ..Default::default()
                },
                // can I somehow get this from the sprite sheet? naively tossing a
                // spritesheetbundle here instead of an imagebundle seems to panic.
                material: materials.add(asset_server.load("textures/coin.png").into()),
                ..Default::default()
            });
            parent
                .spawn(TextBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(5.0),
                            right: Val::Px(10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: format!("{}", game_state.primary_currency),
                        font: font_handles.jptext.clone(),
                        style: TextStyle {
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(CurrencyDisplay);
            parent.spawn(ImageBundle {
                style: Style {
                    margin: Rect {
                        left: Val::Px(5.0),
                        ..Default::default()
                    },
                    size: Size::new(Val::Auto, Val::Px(32.0)),
                    ..Default::default()
                },
                // can I somehow get this from the sprite sheet? naively tossing a
                // spritesheetbundle here instead of an imagebundle seems to panic.
                material: materials.add(asset_server.load("textures/timer.png").into()),
                ..Default::default()
            });
            parent
                .spawn(TextBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(5.0),
                            right: Val::Px(10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: format!("{}", "30"),
                        font: font_handles.jptext.clone(),
                        style: TextStyle {
                            font_size: FONT_SIZE,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(CooldownTimerDisplay);
        });

    commands.spawn(bevy_tiled_prototype::TiledMapBundle {
        map_asset: asset_server.load("textures/tiled-test.tmx"),
        center: TiledMapCenter(true),
        origin: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
        ..Default::default()
    });

    // I don't know how to make the reticle invisible so I will just put out somewhere out
    // of view
    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
            sprite: TextureAtlasSprite {
                index: 16,
                ..Default::default()
            },
            texture_atlas: texture_atlas_handle.clone(),
            ..Default::default()
        })
        .with(Timer::from_seconds(0.01, true))
        .with(Reticle);

    // TODO: load this from a file
    let mut possible_typing_targets = data::parse_typing_targets(
        "ひ(hi)ら(ra)が(ga)な(na)
        カ(ka)タ(ta)カ(ka)ナ(na)
        1(juu)1(ichi):00(ji)
        大(oo)き(ki)い(i)
        大(dai)学(gaku)生(sei)
        あ(a)か(ka)い(i)ボ(bo)ー(-)ル(ru)
        ミ(mi)ル(ru)ク(ku)コ(ko)ー(-)ヒ(hi)ー(-)
        メ(me)ロ(ro)ン(nn)ソ(so)ー(-)ダ(da)
        た(ta)ま(ma)ご(go)
        か(ka)さ(sa)
        と(to)う(u)き(k)ょ(yo)う(u)
        カ(ka)ラ(ra)オ(o)ケ(ke)
        サ(sa)ン(nn)ド(do)イ(i)ッ(c)チ(chi)
        タ(ta)ク(ku)シ(shi)ー(-)
        カ(ka)レ(re)ー(-)ラ(ra)イ(i)ス(su)
        100(hyaku)パ(pa)ー(-)セ(se)ン(nn)ト(to)
        フ(fu)ラ(ra)ン(nn)ス(su)
        一(hito)つ(tsu)
        二(futa)つ(tsu)
        三(mit)つ(tsu)
        四(yot)つ(tsu)
        五(itsu)つ(tsu)
        六(mut)つ(tsu)
        七(nana)つ(tsu)
        八(yat)つ(tsu)
        九(kokono)つ(tsu)
        1000(senn)円(enn)
        ま(ma)い(i)に(ni)ち(chi)
        か(ka)ん(nn)じ(ji)
        コ(ko)コ(ko)ナ(na)ツ(tsu)
        が(ga)ん(nn)ば(ba)っ(t)て(te)
        ま(ma)も(mo)な(na)く(ku)
        あ(a)り(ri)が(ga)と(to)う(u)
        ご(go)ざ(za)い(i)ま(ma)す(su)
        日(nichi)曜(you)日(bi)
        月(getsu)曜(you)日(bi)
        火(ka)曜(you)日(bi)
        水(sui)曜(you)日(bi)
        木(moku)曜(you)日(bi)
        金(kinn)曜(you)日(bi)
        土(do)曜(you)日(bi)
        ３(sann)０００(zenn)円(enn)
        1(ichi)月(gatsu)
        2(ni)月(gatsu)
        3(sann)月(gatsu)
        4(shi)月(gatsu)
        5(go)月(gatsu)
        6(roku)月(gatsu)
        7(shichi)月(gatsu)
        8(hachi)月(gatsu)
        9(ku)月(gatsu)
        10(juu)月(gatsu)
        1(juu)1(ichi)月(gatsu)
        1(juu)2(ni)月(gatsu)
        ひ(hi)だ(da)り(ri)手(te)
        み(mi)ぎ(gi)手(te)
        あ(a)し(shi)く(ku)び(bi)
        く(ku)つ(tsu)し(shi)た(ta)
        1(ichi)0000(mann)円(enn)
        ワ(wa)イ(i)ン(nn)
        カ(ka)メ(me)ラ(ra)
        ア(a)メ(me)リ(ri)カ(ka)
        ホ(ho)テ(te)ル(ru)
        エ(e)ス(su)カ(ka)レ(re)ー(-)タ(ta)ー(-)
        ロ(ro)ボ(bo)ッ(t)ト(to)
        カ(ka)ヤ(ya)ッ(k)ク(ku)
        ユ(yu)ニ(ni)ー(-)ク(ku)
        マ(ma)ヨ(yo)ネ(ne)ー(-)ズ(zu)
        ア(a)イ(i)ス(su)ク(ku)リ(ri)ー(-)ム(mu)
        レ(re)モ(mo)ン(nn)
        ハ(ha)イ(i)キ(ki)ン(nn)グ(gu)
        ゴ(go)ル(ru)フ(fu)
        ヘ(he)リ(ri)コ(ko)プ(pu)タ(ta)ー(-)
        シ(sh)ャ(a)ツ(tsu)
        ポ(po)ケ(ke)ッ(t)ト(to)
        ダ(da)ウ(u)ン(nn)ロ(ro)ー(-)ド(do)",
    )
    .unwrap();
    possible_typing_targets.shuffle(&mut rng);
    game_state.possible_typing_targets = possible_typing_targets.into();

    for _ in 0..8 {
        let target = game_state
            .possible_typing_targets
            .pop_front()
            .unwrap()
            .clone();
        typing_target_spawn_events.send(TypingTargetSpawnEvent(target.clone(), None));
    }
}

fn spawn_map_objects(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    texture_handles: Res<TextureHandles>,
    maps: Res<Assets<bevy_tiled_prototype::Map>>,
    map_events: Res<Events<AssetEvent<bevy_tiled_prototype::Map>>>,
    mut map_event_reader: Local<EventReader<AssetEvent<bevy_tiled_prototype::Map>>>,
    mut update_actions_events: ResMut<Events<UpdateActionsEvent>>,
    mut waves: ResMut<Waves>,
) {
    // This seems pretty wild. Not remotely clear if this is the correct way to go about this,
    // but it seems to do the job.
    //
    // Because we're just worried about object data (and not placing sprites) from bevy_tiled
    // right now, it seems okay to potentially do this stuff before bevy_tiled is done processing
    // the asset event iself.

    use bevy_tiled_prototype::tiled::{Object, ObjectShape, PropertyValue};

    for event in map_event_reader.iter(&map_events) {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(map_asset) = maps.get(handle) {
                    // So we've loaded in a new bevy_tiled_prototype::Map and can do things
                    // to it now.

                    for grp in map_asset.map.object_groups.iter() {
                        let mut sorted = grp
                            .objects
                            .iter()
                            .filter(|o| o.obj_type == "tile_slot")
                            .filter(|o| o.properties.contains_key("index"))
                            .filter_map(|o| match o.properties.get(&"index".to_string()) {
                                Some(PropertyValue::IntValue(index)) => Some((o, index)),
                                _ => None,
                            })
                            .collect::<Vec<(&Object, &i32)>>();

                        sorted.sort_by(|a, b| a.1.cmp(b.1));

                        for (obj, index) in sorted {
                            // TODO We're just using centered maps right now, but we should be
                            // able to find out if we should be centering these or not.
                            //
                            // Or better yet, bevy_tiled should provide this data to us
                            // transformed somehow.
                            let mut transform = map_asset.center(Transform::default());

                            // Y axis in bevy/tiled are reverse?
                            transform.translation.x += obj.x + obj.width / 2.0;
                            transform.translation.y -= obj.y - obj.height / 2.0;

                            // I am just using these objects as markers right now, despite them
                            // being associated with the correct tile. So there's no need to
                            // draw these objects.

                            game_state.tower_slots.push(
                                commands
                                    .spawn(SpriteBundle {
                                        transform,
                                        ..Default::default()
                                    })
                                    .with(TowerSlot {
                                        texture_ui: texture_handles.tower_slot_ui[*index as usize]
                                            .clone(),
                                    })
                                    .current_entity()
                                    .unwrap(),
                            );
                        }
                    }

                    // Pretty sure this is duplicating the action update unnecessarily
                    update_actions_events.send(UpdateActionsEvent);

                    // Try to grab the enemy path defined in the map
                    for grp in map_asset.map.object_groups.iter() {
                        for (obj, points, _index) in grp
                            .objects
                            .iter()
                            .filter(|o| o.obj_type == "enemy_path")
                            .filter_map(|o| {
                                match (&o.shape, o.properties.get(&"index".to_string())) {
                                    (
                                        ObjectShape::Polyline { points },
                                        Some(PropertyValue::IntValue(index)),
                                    ) => Some((o, points, index)),
                                    (
                                        ObjectShape::Polygon { points },
                                        Some(PropertyValue::IntValue(index)),
                                    ) => Some((o, points, index)),
                                    _ => None,
                                }
                            })
                        {
                            let transformed = points
                                .iter()
                                .map(|(x, y)| {
                                    let transform = map_asset.center(Transform::default());

                                    // Y axis in bevy/tiled are reverse?
                                    Vec2::new(
                                        transform.translation.x + obj.x + x,
                                        transform.translation.y - obj.y - y,
                                    )
                                })
                                .collect();

                            // Temporary. We want to collect paths and reference them later when
                            // collecting "wave objects."
                            waves.waves.push(Wave {
                                path: transformed,
                                ..Default::default()
                            })
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            width: 720.,
            height: 480.,
            canvas: Some("#bevy-canvas".to_string()),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_webgl2::WebGL2Plugin)
        .add_plugin(bevy_tiled_prototype::TiledMapPlugin)
        .add_startup_system(startup_system.system())
        .add_plugin(TypingPlugin)
        .add_plugin(HealthBarPlugin)
        .add_plugin(BulletPlugin)
        .add_resource(GameState::default())
        .add_resource(Waves::default())
        .add_resource(CooldownTimerTimer(Timer::from_seconds(0.1, true)))
        .init_resource::<FontHandles>()
        .init_resource::<TextureHandles>()
        .add_system(typing_target_finished.system())
        .add_system(animate_reticle.system())
        .add_system(animate_skeleton.system())
        .add_system(spawn_map_objects.system())
        .add_system(spawn_enemies.system())
        .add_system(move_enemies.system())
        .add_system(shoot_enemies.system())
        .add_system(update_timer_display.system())
        // this just needs to happen after TypingTargetSpawnEvent gets processed
        .add_system_to_stage(stage::LAST, update_actions.system())
        // .. and this needs to happen after update_actions
        .add_stage_after(stage::UPDATE, "test2", SystemStage::parallel())
        .add_system_to_stage("test2", update_currency_display.system())
        .add_event::<UpdateActionsEvent>()
        .run();
}
