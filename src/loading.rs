use crate::{data::AnimationData, map::TiledMap, GameData, TaipoState};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(TaipoState::Load)
                .continue_to_state(TaipoState::MainMenu)
                .with_collection::<TextureHandles>()
                .with_collection::<UiTextureHandles>()
                .with_collection::<EnemyAtlasHandles>()
                .with_collection::<EnemyAnimationHandles>()
                .with_collection::<GameDataHandles>()
                .with_collection::<FontHandles>()
                .with_collection::<LevelHandles>()
                .with_collection::<AudioHandles>()
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "enemy_atlas.assets",
                ]),
        );
    }
}

#[derive(AssetCollection)]
pub struct UiTextureHandles {
    #[asset(path = "textures/ui/coin.png")]
    pub coin_ui: Handle<Image>,
    #[asset(path = "textures/ui/upgrade.png")]
    pub upgrade_ui: Handle<Image>,
    #[asset(path = "textures/ui/back.png")]
    pub back_ui: Handle<Image>,
    #[asset(path = "textures/ui/shuriken_tower.png")]
    pub shuriken_tower_ui: Handle<Image>,
    #[asset(path = "textures/ui/pupper_tower.png")]
    pub support_tower_ui: Handle<Image>,
    #[asset(path = "textures/ui/boss_tower.png")]
    pub debuff_tower_ui: Handle<Image>,
    #[asset(path = "textures/ui/timer.png")]
    pub timer_ui: Handle<Image>,
    #[asset(path = "textures/ui/sell.png")]
    pub sell_ui: Handle<Image>,
}
#[derive(AssetCollection)]
pub struct TextureHandles {
    #[asset(path = "textures/shuriken.png")]
    pub bullet_shuriken: Handle<Image>,
    #[asset(path = "textures/boss_bullet.png")]
    pub bullet_debuff: Handle<Image>,
    #[asset(path = "textures/reticle.png")]
    pub reticle: Handle<Image>,
    #[asset(path = "textures/range_indicator.png")]
    pub range_indicator: Handle<Image>,
    #[asset(path = "textures/status_up.png")]
    pub status_up: Handle<Image>,
    #[asset(path = "textures/status_down.png")]
    pub status_down: Handle<Image>,
    #[asset(path = "textures/tower_slot.png")]
    pub tower_slot: Handle<Image>,
    #[asset(path = "textures/towers/shuriken.png")]
    pub tower: Handle<Image>,
    #[asset(path = "textures/towers/shuriken2.png")]
    pub tower_two: Handle<Image>,
    #[asset(path = "textures/towers/pupper.png")]
    pub support_tower: Handle<Image>,
    #[asset(path = "textures/towers/pupper2.png")]
    pub support_tower_two: Handle<Image>,
    #[asset(path = "textures/towers/boss.png")]
    pub debuff_tower: Handle<Image>,
    #[asset(path = "textures/towers/boss2.png")]
    pub debuff_tower_two: Handle<Image>,
}
#[derive(AssetCollection)]
pub struct LevelHandles {
    #[asset(path = "textures/level1.tmx")]
    pub one: Handle<TiledMap>,
}

#[derive(AssetCollection)]
pub struct EnemyAtlasHandles {
    #[asset(key = "crab")]
    crab: Handle<TextureAtlas>,
    #[asset(key = "deathknight")]
    deathknight: Handle<TextureAtlas>,
    #[asset(key = "skeleton")]
    skeleton: Handle<TextureAtlas>,
    #[asset(key = "skeleton2")]
    skeleton2: Handle<TextureAtlas>,
    #[asset(key = "snake")]
    snake: Handle<TextureAtlas>,
}
impl EnemyAtlasHandles {
    pub fn by_key(&self, key: &str) -> Handle<TextureAtlas> {
        match key {
            "crab" => self.crab.clone(),
            "deathknight" => self.deathknight.clone(),
            "skeleton" => self.skeleton.clone(),
            "skeleton2" => self.skeleton2.clone(),
            "snake" => self.snake.clone(),
            _ => panic!("enemy atlas does not exist"),
        }
    }
}

#[derive(AssetCollection)]
pub struct EnemyAnimationHandles {
    #[asset(path = "data/anim/crab.anim.ron")]
    pub crab: Handle<AnimationData>,
    #[asset(path = "data/anim/deathknight.anim.ron")]
    pub deathknight: Handle<AnimationData>,
    #[asset(path = "data/anim/skeleton.anim.ron")]
    pub skeleton: Handle<AnimationData>,
    #[asset(path = "data/anim/skeleton2.anim.ron")]
    pub skeleton2: Handle<AnimationData>,
    #[asset(path = "data/anim/snake.anim.ron")]
    pub snake: Handle<AnimationData>,
}
impl EnemyAnimationHandles {
    pub fn by_key(&self, key: &str) -> Handle<AnimationData> {
        match key {
            "crab" => self.crab.clone(),
            "deathknight" => self.deathknight.clone(),
            "skeleton" => self.skeleton.clone(),
            "skeleton2" => self.skeleton2.clone(),
            "snake" => self.snake.clone(),
            _ => panic!("enemy atlas does not exist"),
        }
    }
}

#[derive(AssetCollection)]
pub struct GameDataHandles {
    #[asset(path = "data/game.ron")]
    pub game: Handle<GameData>,
}

#[derive(AssetCollection)]
pub struct FontHandles {
    #[asset(path = "fonts/NotoSansJP-Light.otf")]
    pub jptext: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct AudioHandles {
    #[asset(path = "sounds/wrong_character.ogg")]
    pub wrong_character: Handle<AudioSource>,
}
