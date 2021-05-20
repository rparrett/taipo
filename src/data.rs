use crate::TypingTarget;
use bevy::utils::HashMap;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use bevy_asset_ron::*;
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    combinator::{map, opt},
    multi::{fold_many0, many1},
    sequence::{delimited, pair, tuple},
};
use serde::Deserialize;

// Tower stats, prices, etc should go in here eventually
#[serde(rename = "GameData")]
#[derive(Debug, Deserialize)]
pub struct RawGameData {
    pub word_lists: HashMap<String, WordList>,
}

#[derive(Debug, Deserialize)]
pub struct WordList {
    kind: WordListKind,
    string: String,
}

#[derive(Debug, Deserialize)]
pub enum WordListKind {
    Parenthesized,
    UniformChars,
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
                let targets = match word_list.kind {
                    WordListKind::Parenthesized => parse_japanese(&word_list.string)?,
                    WordListKind::UniformChars => parse_uniform_chars(&word_list.string)?,
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

#[derive(Debug, Clone)]
struct IntermediateTypingTarget {
    displayed_chunks: Vec<String>,
    typed_chunks: Vec<Vec<String>>,
}

static HIRAGANA: &str = "あいうえおかがきぎくぐけげこごさざしじすずせぜそぞただちぢつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもやゆよらりるれろわゐゑをんー";
static KATAKANA: &str = "アイウエオカガキギクグケゲコゴサザシジスズセゼソゾタダチヂツヅテデトドナニヌネノハバパヒビピフブプヘベペホボポマミムメモヤユヨラリルレロワヰヱヲンー";
static SUTEGANA: &str = "ァィゥェォャュョぁぃぅぇぉゃゅょ";
static SOKUON: &str = "っッ";

fn kana_to_typed_chunks(kana: &str) -> Option<Vec<String>> {
    match kana {
        // hiragana
        "あ" => Some(vec!["a".to_owned()]),
        "い" => Some(vec!["i".to_owned()]),
        "う" => Some(vec!["u".to_owned()]),
        "え" => Some(vec!["e".to_owned()]),
        "お" => Some(vec!["o".to_owned()]),
        "か" => Some(vec!["ka".to_owned()]),
        "が" => Some(vec!["ga".to_owned()]),
        "き" => Some(vec!["ki".to_owned()]),
        "ぎ" => Some(vec!["gi".to_owned()]),
        "く" => Some(vec!["ku".to_owned()]),
        "ぐ" => Some(vec!["gu".to_owned()]),
        "け" => Some(vec!["ke".to_owned()]),
        "げ" => Some(vec!["ge".to_owned()]),
        "こ" => Some(vec!["ko".to_owned()]),
        "ご" => Some(vec!["go".to_owned()]),
        "さ" => Some(vec!["sa".to_owned()]),
        "ざ" => Some(vec!["za".to_owned()]),
        "し" => Some(vec!["shi".to_owned(), "si".to_owned()]),
        "じ" => Some(vec!["ji".to_owned()]),
        "す" => Some(vec!["su".to_owned()]),
        "ず" => Some(vec!["zu".to_owned()]),
        "せ" => Some(vec!["se".to_owned()]),
        "ぜ" => Some(vec!["ze".to_owned()]),
        "そ" => Some(vec!["so".to_owned()]),
        "ぞ" => Some(vec!["zo".to_owned()]),
        "た" => Some(vec!["ta".to_owned()]),
        "だ" => Some(vec!["da".to_owned()]),
        "ち" => Some(vec!["chi".to_owned(), "ti".to_owned()]),
        "ぢ" => Some(vec!["ji".to_owned()]),
        "つ" => Some(vec!["tsu".to_owned(), "tu".to_owned()]),
        "づ" => Some(vec!["dzu".to_owned(), "du".to_owned()]),
        "て" => Some(vec!["te".to_owned()]),
        "で" => Some(vec!["de".to_owned()]),
        "と" => Some(vec!["to".to_owned()]),
        "ど" => Some(vec!["do".to_owned()]),
        "な" => Some(vec!["na".to_owned()]),
        "に" => Some(vec!["ni".to_owned()]),
        "ぬ" => Some(vec!["nu".to_owned()]),
        "ね" => Some(vec!["ne".to_owned()]),
        "の" => Some(vec!["no".to_owned()]),
        "は" => Some(vec!["ha".to_owned()]),
        "ば" => Some(vec!["ba".to_owned()]),
        "ぱ" => Some(vec!["pa".to_owned()]),
        "ひ" => Some(vec!["hi".to_owned()]),
        "び" => Some(vec!["bi".to_owned()]),
        "ぴ" => Some(vec!["pi".to_owned()]),
        "ふ" => Some(vec!["fu".to_owned()]),
        "ぶ" => Some(vec!["bu".to_owned()]),
        "ぷ" => Some(vec!["pu".to_owned()]),
        "へ" => Some(vec!["he".to_owned()]),
        "べ" => Some(vec!["be".to_owned()]),
        "ぺ" => Some(vec!["pe".to_owned()]),
        "ほ" => Some(vec!["ho".to_owned()]),
        "ぼ" => Some(vec!["bo".to_owned()]),
        "ぽ" => Some(vec!["po".to_owned()]),
        "ま" => Some(vec!["ma".to_owned()]),
        "み" => Some(vec!["mi".to_owned()]),
        "む" => Some(vec!["mu".to_owned()]),
        "め" => Some(vec!["me".to_owned()]),
        "も" => Some(vec!["mo".to_owned()]),
        "や" => Some(vec!["ya".to_owned()]),
        "ゆ" => Some(vec!["yu".to_owned()]),
        "よ" => Some(vec!["yo".to_owned()]),
        "ら" => Some(vec!["ra".to_owned()]),
        "り" => Some(vec!["ri".to_owned()]),
        "る" => Some(vec!["ru".to_owned()]),
        "れ" => Some(vec!["re".to_owned()]),
        "ろ" => Some(vec!["ro".to_owned()]),
        "わ" => Some(vec!["wa".to_owned()]),
        "ゐ" => Some(vec!["wi".to_owned()]),
        "ゑ" => Some(vec!["we".to_owned()]),
        "を" => Some(vec!["wo".to_owned()]),
        "ん" => Some(vec!["n".to_owned(), "nn".to_owned()]),
        // you-on
        "きゃ" => Some(vec!["kya".to_owned()]),
        "きゅ" => Some(vec!["kyu".to_owned()]),
        "きょ" => Some(vec!["kyo".to_owned()]),
        "しゃ" => Some(vec!["sha".to_owned()]),
        "しゅ" => Some(vec!["shu".to_owned()]),
        "しょ" => Some(vec!["sho".to_owned()]),
        "ちゃ" => Some(vec!["cha".to_owned()]),
        "ちゅ" => Some(vec!["chu".to_owned()]),
        "ちょ" => Some(vec!["cho".to_owned()]),
        "にゃ" => Some(vec!["nya".to_owned()]),
        "にゅ" => Some(vec!["nyu".to_owned()]),
        "にょ" => Some(vec!["nyo".to_owned()]),
        "ひゃ" => Some(vec!["hya".to_owned()]),
        "ひゅ" => Some(vec!["hyu".to_owned()]),
        "ひょ" => Some(vec!["hyo".to_owned()]),
        "みゃ" => Some(vec!["mya".to_owned()]),
        "みゅ" => Some(vec!["myu".to_owned()]),
        "みょ" => Some(vec!["myo".to_owned()]),
        "りゃ" => Some(vec!["rya".to_owned()]),
        "りゅ" => Some(vec!["ryu".to_owned()]),
        "りょ" => Some(vec!["ryo".to_owned()]),
        "ぎゃ" => Some(vec!["gya".to_owned()]),
        "ぎゅ" => Some(vec!["gyu".to_owned()]),
        "ぎょ" => Some(vec!["gyo".to_owned()]),
        "じゃ" => Some(vec!["ja".to_owned()]),
        "じゅ" => Some(vec!["ju".to_owned()]),
        "じょ" => Some(vec!["jo".to_owned()]),
        "びゃ" => Some(vec!["bya".to_owned()]),
        "びゅ" => Some(vec!["byu".to_owned()]),
        "びょ" => Some(vec!["byo".to_owned()]),
        "ぴゃ" => Some(vec!["pya".to_owned()]),
        "ぴゅ" => Some(vec!["pyu".to_owned()]),
        "ぴょ" => Some(vec!["pyo".to_owned()]),
        // katakana
        "ア" => Some(vec!["a".to_owned()]),
        "イ" => Some(vec!["i".to_owned()]),
        "ウ" => Some(vec!["u".to_owned()]),
        "エ" => Some(vec!["e".to_owned()]),
        "オ" => Some(vec!["o".to_owned()]),
        "カ" => Some(vec!["ka".to_owned()]),
        "ガ" => Some(vec!["ga".to_owned()]),
        "キ" => Some(vec!["ki".to_owned()]),
        "ギ" => Some(vec!["gi".to_owned()]),
        "ク" => Some(vec!["ku".to_owned()]),
        "グ" => Some(vec!["gu".to_owned()]),
        "ケ" => Some(vec!["ke".to_owned()]),
        "ゲ" => Some(vec!["ge".to_owned()]),
        "コ" => Some(vec!["ko".to_owned()]),
        "ゴ" => Some(vec!["go".to_owned()]),
        "サ" => Some(vec!["sa".to_owned()]),
        "ザ" => Some(vec!["za".to_owned()]),
        "シ" => Some(vec!["shi".to_owned(), "si".to_owned()]),
        "ジ" => Some(vec!["ji".to_owned()]),
        "ス" => Some(vec!["su".to_owned()]),
        "ズ" => Some(vec!["zu".to_owned()]),
        "セ" => Some(vec!["se".to_owned()]),
        "ゼ" => Some(vec!["ze".to_owned()]),
        "ソ" => Some(vec!["so".to_owned()]),
        "ゾ" => Some(vec!["zo".to_owned()]),
        "タ" => Some(vec!["ta".to_owned()]),
        "ダ" => Some(vec!["da".to_owned()]),
        "チ" => Some(vec!["chi".to_owned(), "ti".to_owned()]),
        "ヂ" => Some(vec!["ji".to_owned()]),
        "ツ" => Some(vec!["tsu".to_owned(), "tu".to_owned()]),
        "ヅ" => Some(vec!["dzu".to_owned(), "du".to_owned()]),
        "テ" => Some(vec!["te".to_owned()]),
        "デ" => Some(vec!["de".to_owned()]),
        "ト" => Some(vec!["to".to_owned()]),
        "ド" => Some(vec!["do".to_owned()]),
        "ナ" => Some(vec!["na".to_owned()]),
        "ニ" => Some(vec!["ni".to_owned()]),
        "ヌ" => Some(vec!["nu".to_owned()]),
        "ネ" => Some(vec!["ne".to_owned()]),
        "ノ" => Some(vec!["no".to_owned()]),
        "ハ" => Some(vec!["ha".to_owned()]),
        "バ" => Some(vec!["ba".to_owned()]),
        "パ" => Some(vec!["pa".to_owned()]),
        "ヒ" => Some(vec!["hi".to_owned()]),
        "ビ" => Some(vec!["bi".to_owned()]),
        "ピ" => Some(vec!["pi".to_owned()]),
        "フ" => Some(vec!["fu".to_owned()]),
        "ブ" => Some(vec!["bu".to_owned()]),
        "プ" => Some(vec!["pu".to_owned()]),
        "ヘ" => Some(vec!["he".to_owned()]),
        "ベ" => Some(vec!["be".to_owned()]),
        "ペ" => Some(vec!["pe".to_owned()]),
        "ホ" => Some(vec!["ho".to_owned()]),
        "ボ" => Some(vec!["bo".to_owned()]),
        "ポ" => Some(vec!["po".to_owned()]),
        "マ" => Some(vec!["ma".to_owned()]),
        "ミ" => Some(vec!["mi".to_owned()]),
        "ム" => Some(vec!["mu".to_owned()]),
        "メ" => Some(vec!["me".to_owned()]),
        "モ" => Some(vec!["mo".to_owned()]),
        "ヤ" => Some(vec!["ya".to_owned()]),
        "ユ" => Some(vec!["yu".to_owned()]),
        "ヨ" => Some(vec!["yo".to_owned()]),
        "ラ" => Some(vec!["ra".to_owned()]),
        "リ" => Some(vec!["ri".to_owned()]),
        "ル" => Some(vec!["ru".to_owned()]),
        "レ" => Some(vec!["re".to_owned()]),
        "ロ" => Some(vec!["ro".to_owned()]),
        "ワ" => Some(vec!["wa".to_owned()]),
        "ヰ" => Some(vec!["wi".to_owned()]),
        "ヱ" => Some(vec!["we".to_owned()]),
        "ヲ" => Some(vec!["wo".to_owned()]),
        "ン" => Some(vec!["nn".to_owned(), "n".to_owned()]),
        "ー" => Some(vec!["-".to_owned()]),
        // you-on
        "キャ" => Some(vec!["kya".to_owned()]),
        "キュ" => Some(vec!["kyu".to_owned()]),
        "キョ" => Some(vec!["kyo".to_owned()]),
        "シャ" => Some(vec!["sha".to_owned()]),
        "シュ" => Some(vec!["shu".to_owned()]),
        "ショ" => Some(vec!["sho".to_owned()]),
        "チャ" => Some(vec!["cha".to_owned()]),
        "チュ" => Some(vec!["chu".to_owned()]),
        "チョ" => Some(vec!["cho".to_owned()]),
        "ニャ" => Some(vec!["nya".to_owned()]),
        "ニュ" => Some(vec!["nyu".to_owned()]),
        "ニョ" => Some(vec!["nyo".to_owned()]),
        "ヒャ" => Some(vec!["hya".to_owned()]),
        "ヒュ" => Some(vec!["hyu".to_owned()]),
        "ヒョ" => Some(vec!["hyo".to_owned()]),
        "ミャ" => Some(vec!["mya".to_owned()]),
        "ミュ" => Some(vec!["myu".to_owned()]),
        "ミョ" => Some(vec!["myo".to_owned()]),
        "リャ" => Some(vec!["rya".to_owned()]),
        "リュ" => Some(vec!["ryu".to_owned()]),
        "リョ" => Some(vec!["ryo".to_owned()]),
        "ギャ" => Some(vec!["gya".to_owned()]),
        "ギュ" => Some(vec!["gyu".to_owned()]),
        "ギョ" => Some(vec!["gyo".to_owned()]),
        "ジャ" => Some(vec!["ja".to_owned()]),
        "ジュ" => Some(vec!["ju".to_owned()]),
        "ジョ" => Some(vec!["jo".to_owned()]),
        "ビャ" => Some(vec!["bya".to_owned()]),
        "ビュ" => Some(vec!["byu".to_owned()]),
        "ビョ" => Some(vec!["byo".to_owned()]),
        "ピャ" => Some(vec!["pya".to_owned()]),
        "ピュ" => Some(vec!["pyu".to_owned()]),
        "ピョ" => Some(vec!["pyo".to_owned()]),
        // wacky katakan you-on
        "ウェ" => Some(vec!["we".to_owned()]),
        _ => None,
    }
}

fn is_not_kana_or_open_paren(c: char) -> bool {
    c != '('
        && !HIRAGANA.contains(c)
        && !KATAKANA.contains(c)
        && !SUTEGANA.contains(c)
        && !SOKUON.contains(c)
}

fn is_katakana(i: &str) -> nom::IResult<&str, char> {
    nom::character::complete::one_of(KATAKANA)(i)
}

fn is_hiragana(i: &str) -> nom::IResult<&str, char> {
    nom::character::complete::one_of(HIRAGANA)(i)
}

fn is_sutegana(i: &str) -> nom::IResult<&str, char> {
    nom::character::complete::one_of(SUTEGANA)(i)
}

fn is_sokuon(i: &str) -> nom::IResult<&str, char> {
    nom::character::complete::one_of(SOKUON)(i)
}

fn parenthesized(i: &str) -> nom::IResult<&str, IntermediateTypingTarget> {
    map(
        many1(pair(
            take_while(is_not_kana_or_open_paren),
            delimited(tag("("), take_while(|c| c != ')'), tag(")")),
        )),
        |things: Vec<(&str, &str)>| {
            let mut typed_chunks = vec![];
            let mut displayed_chunks = vec![];
            for (displayed, typed) in things {
                typed_chunks.push(vec![typed.into()]);
                displayed_chunks.push(displayed.into());
            }
            IntermediateTypingTarget {
                typed_chunks,
                displayed_chunks,
            }
        },
    )(i)
}

fn japanese(i: &str) -> nom::IResult<&str, IntermediateTypingTarget> {
    // empty should be a parse error

    fold_many0(
        alt((kana_chunk, parenthesized)),
        IntermediateTypingTarget {
            typed_chunks: vec![],
            displayed_chunks: vec![],
        },
        |mut acc, thing| {
            acc.typed_chunks.extend(thing.typed_chunks);
            acc.displayed_chunks.extend(thing.displayed_chunks);
            acc
        },
    )(i)
}

fn kana_chunk(i: &str) -> nom::IResult<&str, IntermediateTypingTarget> {
    map(
        many1(tuple((
            opt(is_sokuon),
            alt((is_hiragana, is_katakana)),
            opt(is_sutegana),
        ))),
        |things| {
            let mut typed_chunks = vec![];
            let mut displayed_chunks = vec![];

            for (sokuon, kana, sutegana) in things {
                let mut combined = String::from(kana);
                if let Some(sutegana) = sutegana {
                    combined.push(sutegana);
                }

                // maybe this should be a parse error.
                if let Some(typed) = kana_to_typed_chunks(&combined) {
                    if let Some(sokuon) = sokuon {
                        // TODO does this work in all cases?
                        typed_chunks.push(vec![typed
                            .get(0)
                            .unwrap()
                            .chars()
                            .next()
                            .unwrap()
                            .into()]);
                        displayed_chunks.push(sokuon.into());
                    }
                    typed_chunks.push(typed);
                    displayed_chunks.push(combined.into());
                }
            }

            IntermediateTypingTarget {
                typed_chunks,
                displayed_chunks,
            }
        },
    )(i)
}

pub fn parse_japanese(input: &str) -> Result<Vec<TypingTarget>, anyhow::Error> {
    Ok(input
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| {
            // XXX
            let itt = japanese(l).unwrap().1;

            TypingTarget {
                render: itt.displayed_chunks.clone(),
                ascii: itt
                    .typed_chunks
                    .iter()
                    .cloned()
                    .multi_cartesian_product()
                    .collect::<Vec<_>>(),
                fixed: false,
                disabled: false,
            }
        })
        .filter(|tt| !tt.ascii.is_empty())
        .collect::<Vec<_>>())
}

pub fn parse_uniform_chars(input: &str) -> Result<Vec<TypingTarget>, anyhow::Error> {
    Ok(input
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| {
            let chars = l.chars().map(|c| c.to_string()).collect::<Vec<_>>();
            TypingTarget {
                render: chars.clone(),
                ascii: vec![chars],
                fixed: false,
                disabled: false,
            }
        })
        .collect::<Vec<_>>())
}
