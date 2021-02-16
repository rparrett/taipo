use crate::TypingTarget;
use bevy::utils::HashMap;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use bevy_asset_ron::*;
use nom::{
    bytes::complete::is_not,
    character::complete::{char, line_ending, space0},
    multi::{fold_many0, separated_list0},
    sequence::{delimited, pair},
    IResult,
};
use serde::Deserialize;

// Tower stats, prices, etc should go in here eventually
#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "14b5fdb6-8272-42c2-b337-5fd258dcebb1"]
pub struct RawGameData {
    pub word_lists: HashMap<String, String>,
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

            for (k, v) in raw_game_data.word_lists.iter() {
                game_data
                    .word_lists
                    .insert(k.clone(), parse_typing_targets(&v)?);
            }

            load_context.set_default_asset(LoadedAsset::new(game_data));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

impl Plugin for GameDataPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<GameData>()
            .init_asset_loader::<GameDataLoader>()
            .add_plugin(RonAssetPlugin::<AnimationData>::new(&["anim.ron"]));
    }
}

// I attempted to use map_err to get some sort of useful error out of this thing,
// but then Rust demanded that input be 'static and I gave up.
pub fn parse_typing_targets(input: &str) -> Result<Vec<TypingTarget>, anyhow::Error> {
    if let Ok((_, targets)) = separated_list0(line_ending, delimited(space0, line, space0))(input) {
        Ok(targets
            .iter()
            .cloned()
            .filter(|i| !i.render.is_empty() && !i.ascii.is_empty())
            .collect())
    } else {
        Err(anyhow!("Frustratingly Generic Parser Error"))
    }
}

fn line(input: &str) -> IResult<&str, TypingTarget> {
    fold_many0(
        render_ascii_pair,
        TypingTarget {
            render: vec![],
            ascii: vec![],
            fixed: false,
            disabled: false,
        },
        |mut t, item| {
            t.render.push(item.0.to_string());
            t.ascii.push(item.1.to_string());
            t
        },
    )(input)
}

fn render_ascii_pair(input: &str) -> IResult<&str, (&str, &str)> {
    pair(is_not("()\r\n"), parens)(input)
}

fn parens(input: &str) -> IResult<&str, &str> {
    delimited(char('('), is_not(")\r\n"), char(')'))(input)
}
