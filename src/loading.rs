use crate::{
    layer,
    map::{TiledMap, TiledMapBundle, TiledMapLoadedEvent},
    AnimationData, AnimationHandles, AudioHandles, FontHandles, GameData, TaipoState,
    TextureHandles, FONT_SIZE_ACTION_PANEL,
};
use bevy::{
    asset::{HandleId, LoadState},
    prelude::*,
};
use bevy_ecs_tilemap::Map;

pub struct LoadingPlugin;

#[derive(Default)]
struct MapReady {
    ready: bool,
}

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(TaipoState::Preload).with_system(preload_assets_startup.system()),
        )
        .add_system_set(
            SystemSet::on_update(TaipoState::Preload).with_system(check_preload_assets.system()),
        )
        .add_system_set(
            SystemSet::on_enter(TaipoState::Load).with_system(load_assets_startup.system()),
        )
        .add_system_set(
            SystemSet::on_update(TaipoState::Load).with_system(check_load_assets.system()),
        )
        .add_system_set(SystemSet::on_exit(TaipoState::Load).with_system(load_cleanup.system()));
    }
}
#[derive(Component)]
struct LoadingScreenMarker;

// Our main font is gigantic, but I'd like to use some text on the loading screen. So let's load
// a stripped down version.
//
// It probably makes way more sense to preload these things in JS or something, because the
// wasm bundle is also gigantic, so we'll want some sort of loading indicator there too.
//
// Or wasn't there some way to bundle the assets into the binary?
fn preload_assets_startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut font_handles: ResMut<FontHandles>,
) {
    font_handles.minimal = asset_server.load("fonts/NotoSans-Light-Min.ttf");

    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, layer::OVERLAY_BG),
                scale: Vec3::new(108.0, 42.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgba(0.0, 0.0, 0.0, 0.7),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(LoadingScreenMarker);

    commands
        .spawn_bundle(Text2dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, layer::OVERLAY)),
            text: Text::with_section(
                "Loading".to_string(),
                TextStyle {
                    font: font_handles.minimal.clone(),
                    font_size: FONT_SIZE_ACTION_PANEL,
                    color: Color::WHITE,
                },
                TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            ),
            ..Default::default()
        })
        .insert(LoadingScreenMarker);
}

// TODO Show that loading screen
fn check_preload_assets(
    font_handles: Res<FontHandles>,
    mut state: ResMut<State<TaipoState>>,
    asset_server: Res<AssetServer>,
) {
    if let LoadState::Loaded = asset_server.get_load_state(font_handles.minimal.id) {
        state.replace(TaipoState::Load).unwrap()
    }
}

fn load_assets_startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut font_handles: ResMut<FontHandles>,
    mut texture_handles: ResMut<TextureHandles>,
    mut animation_handles: ResMut<AnimationHandles>,
    mut audio_handles: ResMut<AudioHandles>,
) {
    font_handles.jptext = asset_server.load("fonts/NotoSansJP-Light.otf");

    let enemies = &["skeleton", "crab", "snake", "skeleton2", "deathknight"];

    for enemy in enemies {
        texture_handles.enemy_atlas_texture.insert(
            enemy.to_string(),
            asset_server.load(format!("textures/enemies/{}.png", enemy).as_str()),
        );
        animation_handles.handles.insert(
            enemy.to_string(),
            asset_server.load(format!("data/anim/{}.anim.ron", enemy).as_str()),
        );
    }

    // Also we need all these loose textures because UI doesn't speak TextureAtlas

    texture_handles.coin_ui = asset_server.load("textures/ui/coin.png");
    texture_handles.upgrade_ui = asset_server.load("textures/ui/upgrade.png");
    texture_handles.back_ui = asset_server.load("textures/ui/back.png");
    texture_handles.shuriken_tower_ui = asset_server.load("textures/ui/shuriken_tower.png");
    texture_handles.support_tower_ui = asset_server.load("textures/ui/pupper_tower.png");
    texture_handles.debuff_tower_ui = asset_server.load("textures/ui/boss_tower.png");
    texture_handles.timer_ui = asset_server.load("textures/ui/timer.png");
    texture_handles.sell_ui = asset_server.load("textures/ui/sell.png");

    // And these because they don't fit on the grid...

    texture_handles.reticle = asset_server.load("textures/reticle.png");
    texture_handles.range_indicator = asset_server.load("textures/range_indicator.png");
    texture_handles.status_up = asset_server.load("textures/status_up.png");
    texture_handles.status_down = asset_server.load("textures/status_down.png");
    texture_handles.tower = asset_server.load("textures/towers/shuriken.png");
    texture_handles.tower_two = asset_server.load("textures/towers/shuriken2.png");
    texture_handles.bullet_shuriken = asset_server.load("textures/shuriken.png");
    texture_handles.bullet_debuff = asset_server.load("textures/boss_bullet.png");
    texture_handles.debuff_tower = asset_server.load("textures/towers/boss.png");
    texture_handles.debuff_tower_two = asset_server.load("textures/towers/boss2.png");
    texture_handles.support_tower = asset_server.load("textures/towers/pupper.png");
    texture_handles.support_tower_two = asset_server.load("textures/towers/pupper2.png");

    // And this because I don't want to create an atlas for one sprite...

    texture_handles.tower_slot = asset_server.load("textures/tower_slot.png");

    //

    texture_handles.game_data = asset_server.load("data/game.ron");
    texture_handles.tiled_map = asset_server.load("textures/level1.tmx");

    //

    audio_handles.wrong_character = asset_server.load("sounds/wrong_character.wav");

    let map_entity = commands.spawn().id();
    commands.entity(map_entity).insert_bundle(TiledMapBundle {
        tiled_map: texture_handles.tiled_map.clone(),
        map: Map::new(0u16, map_entity),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

#[allow(clippy::too_many_arguments)]
fn check_load_assets(
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<TaipoState>>,
    font_handles: Res<FontHandles>,
    mut texture_handles: ResMut<TextureHandles>,
    anim_handles: Res<AnimationHandles>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    game_data_assets: Res<Assets<GameData>>,
    anim_assets: Res<Assets<AnimationData>>,
    tiled_maps: Res<Assets<TiledMap>>,
    mut map_ready: Local<MapReady>,
    mut map_ready_events: EventReader<TiledMapLoadedEvent>,
) {
    for _event in map_ready_events.iter() {
        map_ready.ready = true;
    }

    if !map_ready.ready {
        return;
    }

    match tiled_maps.get(texture_handles.tiled_map.clone()) {
        Some(tiled_map) => {
            let ids = tiled_map
                .tilesets
                .iter()
                .map(|ts| ts.1.id)
                .collect::<Vec<HandleId>>();

            if !matches!(asset_server.get_group_load_state(ids), LoadState::Loaded) {
                info!("declining to load due to tileset");
                return;
            }
        }
        None => return,
    }

    let ids = &[
        font_handles.jptext.id,
        texture_handles.coin_ui.id,
        texture_handles.back_ui.id,
        texture_handles.shuriken_tower_ui.id,
        texture_handles.timer_ui.id,
        texture_handles.tower.id,
        texture_handles.bullet_shuriken.id,
        texture_handles.game_data.id,
    ];

    if !matches!(
        asset_server.get_group_load_state(ids.iter().cloned()),
        LoadState::Loaded
    ) {
        return;
    }

    if !matches!(
        asset_server.get_group_load_state(
            texture_handles
                .enemy_atlas_texture
                .iter()
                .map(|(_, v)| v.id)
        ),
        LoadState::Loaded
    ) {
        return;
    }

    // Uh, why is the thing above not enough for custom assets?
    let game_data = game_data_assets.get(&texture_handles.game_data);
    if game_data.is_none() {
        return;
    }

    // do these take an extra frame to make it into the assets resource after they stop being
    // NotLoaded or something?
    if anim_handles
        .handles
        .iter()
        .map(|(_, v)| v.id)
        .any(|id| anim_assets.get(id).is_none())
    {
        return;
    }

    let names: Vec<String> = texture_handles
        .enemy_atlas_texture
        .keys()
        .cloned()
        .collect();

    for name in names {
        let anim_data = anim_assets
            .get(anim_handles.handles.get(&name.to_string()).unwrap())
            .unwrap();

        let atlas_handle = texture_atlases.add(TextureAtlas::from_grid(
            texture_handles.enemy_atlas_texture[&name].clone(),
            Vec2::new(anim_data.width as f32, anim_data.height as f32),
            anim_data.cols,
            anim_data.rows,
        ));

        texture_handles
            .enemy_atlas
            .insert(name.to_string(), atlas_handle);
    }

    state.replace(TaipoState::MainMenu).unwrap();
}

fn load_cleanup(mut commands: Commands, loading_query: Query<Entity, With<LoadingScreenMarker>>) {
    for ent in loading_query.iter() {
        commands.entity(ent).despawn_recursive();
    }
}
