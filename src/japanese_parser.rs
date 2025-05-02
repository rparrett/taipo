use anyhow::anyhow;
use chumsky::{
    error::Cheap,
    prelude::end,
    primitive::{choice, just, none_of, one_of},
    text::whitespace,
    Error, Parser,
};

use crate::typing::PromptChunks;

#[derive(Debug, Clone)]
struct DisplayedTypedPair(String, String);

static HIRAGANA: &str = "あいうえおかがきぎくぐけげこごさざしじすずせぜそぞただちぢつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもやゆよらりるれろわゐゑをんー";
static KATAKANA: &str = "アイウエオカガキギクグケゲコゴサザシジスズセゼソゾタダチヂツヅテデトドナニヌネノハバパヒビピフブプヘベペホボポマミムメモヤユヨラリルレロワヰヱヲンー";
static SUTEGANA: &str = "ァィゥェォャュョぁぃぅぇぉゃゅょ";
static SOKUON: &str = "っッ";

fn kana_to_typed_chunk(kana: &str) -> Option<&'static str> {
    #![allow(clippy::match_same_arms)]
    match kana {
        // hiragana
        "あ" => Some("a"),
        "い" => Some("i"),
        "う" => Some("u"),
        "え" => Some("e"),
        "お" => Some("o"),
        "か" => Some("ka"),
        "が" => Some("ga"),
        "き" => Some("ki"),
        "ぎ" => Some("gi"),
        "く" => Some("ku"),
        "ぐ" => Some("gu"),
        "け" => Some("ke"),
        "げ" => Some("ge"),
        "こ" => Some("ko"),
        "ご" => Some("go"),
        "さ" => Some("sa"),
        "ざ" => Some("za"),
        "し" => Some("shi"),
        "じ" => Some("ji"),
        "す" => Some("su"),
        "ず" => Some("zu"),
        "せ" => Some("se"),
        "ぜ" => Some("ze"),
        "そ" => Some("so"),
        "ぞ" => Some("zo"),
        "た" => Some("ta"),
        "だ" => Some("da"),
        "ち" => Some("chi"),
        "ぢ" => Some("ji"),
        "つ" => Some("tsu"),
        "づ" => Some("du"),
        "て" => Some("te"),
        "で" => Some("de"),
        "と" => Some("to"),
        "ど" => Some("do"),
        "な" => Some("na"),
        "に" => Some("ni"),
        "ぬ" => Some("nu"),
        "ね" => Some("ne"),
        "の" => Some("no"),
        "は" => Some("ha"),
        "ば" => Some("ba"),
        "ぱ" => Some("pa"),
        "ひ" => Some("hi"),
        "び" => Some("bi"),
        "ぴ" => Some("pi"),
        "ふ" => Some("fu"),
        "ぶ" => Some("bu"),
        "ぷ" => Some("pu"),
        "へ" => Some("he"),
        "べ" => Some("be"),
        "ぺ" => Some("pe"),
        "ほ" => Some("ho"),
        "ぼ" => Some("bo"),
        "ぽ" => Some("po"),
        "ま" => Some("ma"),
        "み" => Some("mi"),
        "む" => Some("mu"),
        "め" => Some("me"),
        "も" => Some("mo"),
        "や" => Some("ya"),
        "ゆ" => Some("yu"),
        "よ" => Some("yo"),
        "ら" => Some("ra"),
        "り" => Some("ri"),
        "る" => Some("ru"),
        "れ" => Some("re"),
        "ろ" => Some("ro"),
        "わ" => Some("wa"),
        "ゐ" => Some("wi"),
        "ゑ" => Some("we"),
        "を" => Some("wo"),
        "ん" => Some("nn"),
        // you-on
        "きゃ" => Some("kya"),
        "きゅ" => Some("kyu"),
        "きょ" => Some("kyo"),
        "しゃ" => Some("sha"),
        "しゅ" => Some("shu"),
        "しょ" => Some("sho"),
        "ちゃ" => Some("cha"),
        "ちゅ" => Some("chu"),
        "ちょ" => Some("cho"),
        "にゃ" => Some("nya"),
        "にゅ" => Some("nyu"),
        "にょ" => Some("nyo"),
        "ひゃ" => Some("hya"),
        "ひゅ" => Some("hyu"),
        "ひょ" => Some("hyo"),
        "みゃ" => Some("mya"),
        "みゅ" => Some("myu"),
        "みょ" => Some("myo"),
        "りゃ" => Some("rya"),
        "りゅ" => Some("ryu"),
        "りょ" => Some("ryo"),
        "ぎゃ" => Some("gya"),
        "ぎゅ" => Some("gyu"),
        "ぎょ" => Some("gyo"),
        "じゃ" => Some("ja"),
        "じゅ" => Some("ju"),
        "じょ" => Some("jo"),
        "びゃ" => Some("bya"),
        "びゅ" => Some("byu"),
        "びょ" => Some("byo"),
        "ぴゃ" => Some("pya"),
        "ぴゅ" => Some("pyu"),
        "ぴょ" => Some("pyo"),
        // katakana
        "ア" => Some("a"),
        "イ" => Some("i"),
        "ウ" => Some("u"),
        "エ" => Some("e"),
        "オ" => Some("o"),
        "カ" => Some("ka"),
        "ガ" => Some("ga"),
        "キ" => Some("ki"),
        "ギ" => Some("gi"),
        "ク" => Some("ku"),
        "グ" => Some("gu"),
        "ケ" => Some("ke"),
        "ゲ" => Some("ge"),
        "コ" => Some("ko"),
        "ゴ" => Some("go"),
        "サ" => Some("sa"),
        "ザ" => Some("za"),
        "シ" => Some("shi"),
        "ジ" => Some("ji"),
        "ス" => Some("su"),
        "ズ" => Some("zu"),
        "セ" => Some("se"),
        "ゼ" => Some("ze"),
        "ソ" => Some("so"),
        "ゾ" => Some("zo"),
        "タ" => Some("ta"),
        "ダ" => Some("da"),
        "チ" => Some("chi"),
        "ヂ" => Some("ji"),
        "ツ" => Some("tsu"),
        "ヅ" => Some("du"),
        "テ" => Some("te"),
        "デ" => Some("de"),
        "ト" => Some("to"),
        "ド" => Some("do"),
        "ナ" => Some("na"),
        "ニ" => Some("ni"),
        "ヌ" => Some("nu"),
        "ネ" => Some("ne"),
        "ノ" => Some("no"),
        "ハ" => Some("ha"),
        "バ" => Some("ba"),
        "パ" => Some("pa"),
        "ヒ" => Some("hi"),
        "ビ" => Some("bi"),
        "ピ" => Some("pi"),
        "フ" => Some("fu"),
        "ブ" => Some("bu"),
        "プ" => Some("pu"),
        "ヘ" => Some("he"),
        "ベ" => Some("be"),
        "ペ" => Some("pe"),
        "ホ" => Some("ho"),
        "ボ" => Some("bo"),
        "ポ" => Some("po"),
        "マ" => Some("ma"),
        "ミ" => Some("mi"),
        "ム" => Some("mu"),
        "メ" => Some("me"),
        "モ" => Some("mo"),
        "ヤ" => Some("ya"),
        "ユ" => Some("yu"),
        "ヨ" => Some("yo"),
        "ラ" => Some("ra"),
        "リ" => Some("ri"),
        "ル" => Some("ru"),
        "レ" => Some("re"),
        "ロ" => Some("ro"),
        "ワ" => Some("wa"),
        "ヰ" => Some("wi"),
        "ヱ" => Some("we"),
        "ヲ" => Some("wo"),
        "ン" => Some("nn"),
        "ー" => Some("-"),
        // you-on
        "キャ" => Some("kya"),
        "キュ" => Some("kyu"),
        "キョ" => Some("kyo"),
        "シャ" => Some("sha"),
        "シュ" => Some("shu"),
        "ショ" => Some("sho"),
        "チャ" => Some("cha"),
        "チュ" => Some("chu"),
        "チョ" => Some("cho"),
        "ニャ" => Some("nya"),
        "ニュ" => Some("nyu"),
        "ニョ" => Some("nyo"),
        "ヒャ" => Some("hya"),
        "ヒュ" => Some("hyu"),
        "ヒョ" => Some("hyo"),
        "ミャ" => Some("mya"),
        "ミュ" => Some("myu"),
        "ミョ" => Some("myo"),
        "リャ" => Some("rya"),
        "リュ" => Some("ryu"),
        "リョ" => Some("ryo"),
        "ギャ" => Some("gya"),
        "ギュ" => Some("gyu"),
        "ギョ" => Some("gyo"),
        "ジャ" => Some("ja"),
        "ジュ" => Some("ju"),
        "ジョ" => Some("jo"),
        "ビャ" => Some("bya"),
        "ビュ" => Some("byu"),
        "ビョ" => Some("byo"),
        "ピャ" => Some("pya"),
        "ピュ" => Some("pyu"),
        "ピョ" => Some("pyo"),
        // wacky katakan you-on
        "ウェ" => Some("we"),
        "ジェ" => Some("je"),
        "チェ" => Some("che"),
        "フェ" => Some("fe"),
        "フィ" => Some("fi"),
        "ティ" => Some("texi"),
        _ => None,
    }
}

fn line() -> impl Parser<char, Vec<DisplayedTypedPair>, Error = Cheap<char>> {
    kana()
        .or(parenthetical())
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .flatten()
        .labelled("line")
}

fn parenthetical() -> impl Parser<char, Vec<DisplayedTypedPair>, Error = Cheap<char>> {
    none_of("\n()")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .then(kana().delimited_by(just('('), just(')')))
        .map(|(outside, inside)| {
            let inside_string = inside.iter().cloned().map(|i| i.1).collect::<String>();
            vec![DisplayedTypedPair(outside, inside_string)]
        })
}

fn kana() -> impl Parser<char, Vec<DisplayedTypedPair>, Error = Cheap<char>> {
    one_of(SOKUON)
        .or_not()
        .then(choice((one_of(HIRAGANA), one_of(KATAKANA))).labelled("kana"))
        .then(one_of(SUTEGANA).or_not())
        .try_map(|((sokuon, hiragana), sutegana), span| {
            let mut combined = String::from(hiragana);
            if let Some(sutegana) = sutegana {
                combined.push(sutegana);
            }

            let typed = kana_to_typed_chunk(&combined)
                .ok_or_else(|| Cheap::<char>::expected_input_found(span, [], None))?;

            let mut pairs = vec![];

            if let Some(sokuon) = sokuon {
                // TODO does this work in all cases?
                // If there's a sokuon, repeat the first character of the typed output
                pairs.push(DisplayedTypedPair(
                    sokuon.into(),
                    typed.chars().next().unwrap().into(),
                ));
            }

            pairs.push(DisplayedTypedPair(combined, typed.to_owned()));

            Ok(pairs)
        })
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .flatten()
}

pub fn parser() -> impl Parser<char, Vec<PromptChunks>, Error = Cheap<char>> {
    whitespace()
        .ignore_then(
            line()
                .map(|l| {
                    let mut typed_chunks = vec![];
                    let mut displayed_chunks = vec![];

                    for f in l.iter().cloned() {
                        displayed_chunks.push(f.0);
                        typed_chunks.push(f.1);
                    }

                    PromptChunks {
                        typed: typed_chunks,
                        displayed: displayed_chunks,
                    }
                })
                .separated_by(whitespace()),
        )
        .then_ignore(whitespace())
        .then_ignore(end())
}

pub fn parse(input: &str) -> anyhow::Result<Vec<PromptChunks>> {
    parser().parse(input).map_err(|errs| {
        let err = &errs[0];
        let (line, col) = get_line_and_column(err.span().start, input);
        anyhow!(format!("Parsing failed at line {}, column {}", line, col))
    })
}

fn get_line_and_column(char_index: usize, input: &str) -> (usize, usize) {
    let mut last: usize = 0;
    let mut count: usize = 0;

    input
        .chars()
        .enumerate()
        .take(char_index)
        .filter(|(_, c)| *c == '\n')
        .for_each(|(i, _)| {
            count += 1;
            last = i;
        });

    (count + 1, char_index - last)
}
