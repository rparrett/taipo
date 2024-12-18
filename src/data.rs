use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::HashMap,
};

use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;

use crate::{japanese_parser, TypingTarget};

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

#[derive(Default, Asset, TypePath)]
pub struct WordList {
    pub words: Vec<TypingTarget>,
}

#[derive(Debug, Deserialize)]
pub enum InputKind {
    Japanese,
    Plain,
}

#[derive(Debug, Asset, TypePath, Default)]
pub struct GameData {
    pub word_list_menu: Vec<WordListMenuItem>,
    pub word_lists: HashMap<String, Handle<WordList>>,
}

#[derive(Debug, Asset, Deserialize, TypePath)]
#[allow(dead_code)]
pub struct AnimationData {
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
        app.init_asset::<GameData>()
            .init_asset::<WordList>()
            .register_asset_loader(GameDataLoader)
            .register_asset_loader(PlainWordListLoader)
            .register_asset_loader(JapaneseWordListLoader)
            .add_plugins(RonAssetPlugin::<AnimationData>::new(&["anim.ron"]));
    }
}
#[derive(Default)]
pub struct GameDataLoader;
#[derive(Default)]
pub struct PlainWordListLoader;
#[derive(Default)]
pub struct JapaneseWordListLoader;

impl AssetLoader for PlainWordListLoader {
    type Asset = WordList;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let words = parse_plain(std::str::from_utf8(&bytes)?)?;
        let list = WordList { words };
        Ok(list)
    }

    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
}

impl AssetLoader for JapaneseWordListLoader {
    type Asset = WordList;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let words = japanese_parser::parse(std::str::from_utf8(&bytes)?)?;
        let list = WordList { words };
        Ok(list)
    }

    fn extensions(&self) -> &[&str] {
        &["jp.txt"]
    }
}

impl AssetLoader for GameDataLoader {
    type Asset = GameData;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let raw_game_data = ron::de::from_bytes::<RawGameData>(&bytes)?;

        let mut word_list_handles: HashMap<String, Handle<WordList>> = HashMap::default();

        for file_name in raw_game_data
            .word_list_menu
            .iter()
            .cloned()
            .flat_map(|word_list| word_list.word_lists)
        {
            let handle = load_context.load(file_name.clone());

            word_list_handles.insert(file_name, handle);
        }

        let game_data = GameData {
            word_list_menu: raw_game_data.word_list_menu,
            word_lists: word_list_handles,
        };

        Ok(game_data)
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
            }
        })
        .collect::<Vec<_>>())
}
