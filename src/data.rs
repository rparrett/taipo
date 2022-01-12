use crate::{japanese_parser, TypingTarget};

use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::{BoxedFuture, HashMap},
};
use bevy_asset_ron::*;

use serde::Deserialize;

// Tower stats, prices, etc should go in here eventually
#[derive(Debug, Deserialize)]
#[serde(rename = "GameData")]
pub struct RawGameData {
    pub word_list_menu: Vec<WordListMenuItem>,
}

#[derive(Component, Debug, Deserialize, Clone)]
pub struct WordListMenuItem {
    pub label: String,
    pub word_lists: Vec<String>,
}

#[derive(Default, TypeUuid)]
#[uuid = "c000f8e6-ecf2-4d6a-a865-c2065d8a429a"]
pub struct WordList {
    pub words: Vec<TypingTarget>,
}

#[derive(Debug, Deserialize)]
pub enum InputKind {
    Japanese,
    Plain,
}

#[derive(Debug, TypeUuid, Default)]
#[uuid = "fa116b6c-6c13-11eb-9439-0242ac130002"]
pub struct GameData {
    pub word_list_menu: Vec<WordListMenuItem>,
    pub word_lists: HashMap<String, Handle<WordList>>,
}

#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "8fa36319-786f-43f5-82fd-ab04124bd018"]
pub struct AnimationData {
    pub width: usize,
    pub height: usize,
    pub rows: usize,
    pub cols: usize,
    pub offset_x: f32,
    pub offset_y: f32,
    pub animations: HashMap<String, AnimationLocation>,
}

#[derive(Debug, Deserialize)]
pub struct AnimationLocation {
    pub length: usize,
    pub row: usize,
}

pub struct GameDataPlugin;

impl Plugin for GameDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<GameData>()
            .add_asset::<WordList>()
            .init_asset_loader::<GameDataLoader>()
            .init_asset_loader::<PlainWordListLoader>()
            .init_asset_loader::<JapaneseWordListLoader>()
            .add_plugin(RonAssetPlugin::<AnimationData>::new(&["anim.ron"]));
    }
}
#[derive(Default)]
pub struct GameDataLoader;
#[derive(Default)]
pub struct PlainWordListLoader;
#[derive(Default)]
pub struct JapaneseWordListLoader;

impl AssetLoader for PlainWordListLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let words = parse_plain(std::str::from_utf8(bytes)?)?;
            let list = WordList { words };
            load_context.set_default_asset(LoadedAsset::new(list));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
}

impl AssetLoader for JapaneseWordListLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let words = japanese_parser::parse(std::str::from_utf8(bytes)?)?;
            let list = WordList { words };
            load_context.set_default_asset(LoadedAsset::new(list));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["jp.txt"]
    }
}

impl AssetLoader for GameDataLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let raw_game_data = ron::de::from_bytes::<RawGameData>(bytes)?;

            let mut word_list_handles: HashMap<String, Handle<WordList>> = HashMap::default();
            let mut word_list_asset_paths = vec![];

            for file_name in raw_game_data
                .word_list_menu
                .iter()
                .cloned()
                .flat_map(|word_list| word_list.word_lists)
            {
                let path = AssetPath::new(file_name.clone().into(), None);
                let handle = load_context.get_handle(path.clone());

                word_list_handles.insert(file_name, handle);
                word_list_asset_paths.push(path);
            }

            let game_data = GameData {
                word_list_menu: raw_game_data.word_list_menu,
                word_lists: word_list_handles,
            };

            load_context.set_default_asset(
                LoadedAsset::new(game_data).with_dependencies(word_list_asset_paths),
            );
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

pub fn parse_plain(input: &str) -> Result<Vec<TypingTarget>, anyhow::Error> {
    Ok(input
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| {
            let chars = l.chars().map(|c| c.to_string()).collect::<Vec<_>>();
            TypingTarget {
                displayed_chunks: chars.clone(),
                typed_chunks: chars,
                fixed: false,
                disabled: false,
            }
        })
        .collect::<Vec<_>>())
}
