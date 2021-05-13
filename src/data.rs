use crate::TypingTarget;
use bevy::utils::HashMap;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use bevy_asset_ron::*;
use serde::Deserialize;

// Tower stats, prices, etc should go in here eventually
#[derive(Debug, Deserialize)]
#[serde(rename = "GameData")]
pub struct RawGameData {
    pub word_lists: HashMap<String, WordList>,
}

#[derive(Debug, Deserialize)]
pub struct WordList {
    input_method: InputMethod,
    list: WordListKind,
}
#[derive(Debug, Deserialize)]
pub enum InputMethod {
    Ascii,
    Kana, // TODO
}

pub type Word = String;
pub type Annotation = String;

#[derive(Debug, Deserialize)]
pub enum WordListKind {
    Plain(Vec<String>),
    Annotated(Vec<(Word, Annotation)>),
}

#[derive(Debug, TypeUuid, Default)]
#[uuid = "fa116b6c-6c13-11eb-9439-0242ac130002"]
pub struct GameData {
    pub word_lists: HashMap<String, Vec<TypingTarget>>,
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
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<GameData>()
            .init_asset_loader::<GameDataLoader>()
            .add_plugin(RonAssetPlugin::<AnimationData>::new(&["anim.ron"]));
    }
}
#[derive(Default)]
pub struct GameDataLoader;

impl AssetLoader for GameDataLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let raw_game_data = ron::de::from_bytes::<RawGameData>(bytes)?;

            let mut game_data = GameData::default();

            for (key, word_list) in raw_game_data.word_lists.iter() {
                let targets = match &word_list.list {
                    WordListKind::Plain(words) => words
                        .into_iter()
                        .map(|word| {
                            let chars: Vec<String> = word.chars().map(|c| c.to_string()).collect();
                            TypingTarget {
                                render: chars.clone(),
                                ascii: chars,
                                fixed: false,
                                disabled: false,
                            }
                        })
                        .collect(),
                    WordListKind::Annotated(words) => words
                        .into_iter()
                        .map(|(word, annotation)| {
                            let delimeters = &['|', '｜'][..];
                            TypingTarget {
                                render: word.split(delimeters).map(ToString::to_string).collect(),
                                ascii: annotation
                                    .split(delimeters)
                                    .map(ToString::to_string)
                                    .collect(),
                                fixed: false,
                                disabled: false,
                            }
                        })
                        .collect(),
                };

                game_data.word_lists.insert(key.clone(), targets);
            }

            load_context.set_default_asset(LoadedAsset::new(game_data));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
