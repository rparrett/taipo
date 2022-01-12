use crate::TypingTarget;
use combine::{
    choice, many, many1, one_of, optional,
    parser::{sequence::between, token::token},
    satisfy, ParseError, Parser, Stream,
};

#[derive(Debug, Clone)]
struct DisplayedTypedPair(String, String);

static HIRAGANA: &str = "あいうえおかがきぎくぐけげこごさざしじすずせぜそぞただちぢつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもやゆよらりるれろわゐゑをんー";
static KATAKANA: &str = "アイウエオカガキギクグケゲコゴサザシジスズセゼソゾタダチヂツヅテデトドナニヌネノハバパヒビピフブプヘベペホボポマミムメモヤユヨラリルレロワヰヱヲンー";
static SUTEGANA: &str = "ァィゥェォャュョぁぃぅぇぉゃゅょ";
static SOKUON: &str = "っッ";

pub fn parse(input: &str) -> Result<Vec<TypingTarget>, anyhow::Error> {
    // TODO: This is a bit silly, but I could not make multi-line parsing
    // with `combine` work well.
    // TODO: We should really do some sort of error checking, even if just
    // to log to the console.
    Ok(input
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .filter_map(|line| japanese().parse(line).ok())
        .map(|result| result.0)
        .collect::<Vec<_>>())
}

fn kana_to_typed_chunk(kana: &str) -> Option<String> {
    match kana {
        // hiragana
        "あ" => Some("a".to_owned()),
        "い" => Some("i".to_owned()),
        "う" => Some("u".to_owned()),
        "え" => Some("e".to_owned()),
        "お" => Some("o".to_owned()),
        "か" => Some("ka".to_owned()),
        "が" => Some("ga".to_owned()),
        "き" => Some("ki".to_owned()),
        "ぎ" => Some("gi".to_owned()),
        "く" => Some("ku".to_owned()),
        "ぐ" => Some("gu".to_owned()),
        "け" => Some("ke".to_owned()),
        "げ" => Some("ge".to_owned()),
        "こ" => Some("ko".to_owned()),
        "ご" => Some("go".to_owned()),
        "さ" => Some("sa".to_owned()),
        "ざ" => Some("za".to_owned()),
        "し" => Some("shi".to_owned()),
        "じ" => Some("ji".to_owned()),
        "す" => Some("su".to_owned()),
        "ず" => Some("zu".to_owned()),
        "せ" => Some("se".to_owned()),
        "ぜ" => Some("ze".to_owned()),
        "そ" => Some("so".to_owned()),
        "ぞ" => Some("zo".to_owned()),
        "た" => Some("ta".to_owned()),
        "だ" => Some("da".to_owned()),
        "ち" => Some("chi".to_owned()),
        "ぢ" => Some("ji".to_owned()),
        "つ" => Some("tsu".to_owned()),
        "づ" => Some("du".to_owned()),
        "て" => Some("te".to_owned()),
        "で" => Some("de".to_owned()),
        "と" => Some("to".to_owned()),
        "ど" => Some("do".to_owned()),
        "な" => Some("na".to_owned()),
        "に" => Some("ni".to_owned()),
        "ぬ" => Some("nu".to_owned()),
        "ね" => Some("ne".to_owned()),
        "の" => Some("no".to_owned()),
        "は" => Some("ha".to_owned()),
        "ば" => Some("ba".to_owned()),
        "ぱ" => Some("pa".to_owned()),
        "ひ" => Some("hi".to_owned()),
        "び" => Some("bi".to_owned()),
        "ぴ" => Some("pi".to_owned()),
        "ふ" => Some("fu".to_owned()),
        "ぶ" => Some("bu".to_owned()),
        "ぷ" => Some("pu".to_owned()),
        "へ" => Some("he".to_owned()),
        "べ" => Some("be".to_owned()),
        "ぺ" => Some("pe".to_owned()),
        "ほ" => Some("ho".to_owned()),
        "ぼ" => Some("bo".to_owned()),
        "ぽ" => Some("po".to_owned()),
        "ま" => Some("ma".to_owned()),
        "み" => Some("mi".to_owned()),
        "む" => Some("mu".to_owned()),
        "め" => Some("me".to_owned()),
        "も" => Some("mo".to_owned()),
        "や" => Some("ya".to_owned()),
        "ゆ" => Some("yu".to_owned()),
        "よ" => Some("yo".to_owned()),
        "ら" => Some("ra".to_owned()),
        "り" => Some("ri".to_owned()),
        "る" => Some("ru".to_owned()),
        "れ" => Some("re".to_owned()),
        "ろ" => Some("ro".to_owned()),
        "わ" => Some("wa".to_owned()),
        "ゐ" => Some("wi".to_owned()),
        "ゑ" => Some("we".to_owned()),
        "を" => Some("wo".to_owned()),
        "ん" => Some("nn".to_owned()),
        // you-on
        "きゃ" => Some("kya".to_owned()),
        "きゅ" => Some("kyu".to_owned()),
        "きょ" => Some("kyo".to_owned()),
        "しゃ" => Some("sha".to_owned()),
        "しゅ" => Some("shu".to_owned()),
        "しょ" => Some("sho".to_owned()),
        "ちゃ" => Some("cha".to_owned()),
        "ちゅ" => Some("chu".to_owned()),
        "ちょ" => Some("cho".to_owned()),
        "にゃ" => Some("nya".to_owned()),
        "にゅ" => Some("nyu".to_owned()),
        "にょ" => Some("nyo".to_owned()),
        "ひゃ" => Some("hya".to_owned()),
        "ひゅ" => Some("hyu".to_owned()),
        "ひょ" => Some("hyo".to_owned()),
        "みゃ" => Some("mya".to_owned()),
        "みゅ" => Some("myu".to_owned()),
        "みょ" => Some("myo".to_owned()),
        "りゃ" => Some("rya".to_owned()),
        "りゅ" => Some("ryu".to_owned()),
        "りょ" => Some("ryo".to_owned()),
        "ぎゃ" => Some("gya".to_owned()),
        "ぎゅ" => Some("gyu".to_owned()),
        "ぎょ" => Some("gyo".to_owned()),
        "じゃ" => Some("ja".to_owned()),
        "じゅ" => Some("ju".to_owned()),
        "じょ" => Some("jo".to_owned()),
        "びゃ" => Some("bya".to_owned()),
        "びゅ" => Some("byu".to_owned()),
        "びょ" => Some("byo".to_owned()),
        "ぴゃ" => Some("pya".to_owned()),
        "ぴゅ" => Some("pyu".to_owned()),
        "ぴょ" => Some("pyo".to_owned()),
        // katakana
        "ア" => Some("a".to_owned()),
        "イ" => Some("i".to_owned()),
        "ウ" => Some("u".to_owned()),
        "エ" => Some("e".to_owned()),
        "オ" => Some("o".to_owned()),
        "カ" => Some("ka".to_owned()),
        "ガ" => Some("ga".to_owned()),
        "キ" => Some("ki".to_owned()),
        "ギ" => Some("gi".to_owned()),
        "ク" => Some("ku".to_owned()),
        "グ" => Some("gu".to_owned()),
        "ケ" => Some("ke".to_owned()),
        "ゲ" => Some("ge".to_owned()),
        "コ" => Some("ko".to_owned()),
        "ゴ" => Some("go".to_owned()),
        "サ" => Some("sa".to_owned()),
        "ザ" => Some("za".to_owned()),
        "シ" => Some("shi".to_owned()),
        "ジ" => Some("ji".to_owned()),
        "ス" => Some("su".to_owned()),
        "ズ" => Some("zu".to_owned()),
        "セ" => Some("se".to_owned()),
        "ゼ" => Some("ze".to_owned()),
        "ソ" => Some("so".to_owned()),
        "ゾ" => Some("zo".to_owned()),
        "タ" => Some("ta".to_owned()),
        "ダ" => Some("da".to_owned()),
        "チ" => Some("chi".to_owned()),
        "ヂ" => Some("ji".to_owned()),
        "ツ" => Some("tsu".to_owned()),
        "ヅ" => Some("du".to_owned()),
        "テ" => Some("te".to_owned()),
        "デ" => Some("de".to_owned()),
        "ト" => Some("to".to_owned()),
        "ド" => Some("do".to_owned()),
        "ナ" => Some("na".to_owned()),
        "ニ" => Some("ni".to_owned()),
        "ヌ" => Some("nu".to_owned()),
        "ネ" => Some("ne".to_owned()),
        "ノ" => Some("no".to_owned()),
        "ハ" => Some("ha".to_owned()),
        "バ" => Some("ba".to_owned()),
        "パ" => Some("pa".to_owned()),
        "ヒ" => Some("hi".to_owned()),
        "ビ" => Some("bi".to_owned()),
        "ピ" => Some("pi".to_owned()),
        "フ" => Some("fu".to_owned()),
        "ブ" => Some("bu".to_owned()),
        "プ" => Some("pu".to_owned()),
        "ヘ" => Some("he".to_owned()),
        "ベ" => Some("be".to_owned()),
        "ペ" => Some("pe".to_owned()),
        "ホ" => Some("ho".to_owned()),
        "ボ" => Some("bo".to_owned()),
        "ポ" => Some("po".to_owned()),
        "マ" => Some("ma".to_owned()),
        "ミ" => Some("mi".to_owned()),
        "ム" => Some("mu".to_owned()),
        "メ" => Some("me".to_owned()),
        "モ" => Some("mo".to_owned()),
        "ヤ" => Some("ya".to_owned()),
        "ユ" => Some("yu".to_owned()),
        "ヨ" => Some("yo".to_owned()),
        "ラ" => Some("ra".to_owned()),
        "リ" => Some("ri".to_owned()),
        "ル" => Some("ru".to_owned()),
        "レ" => Some("re".to_owned()),
        "ロ" => Some("ro".to_owned()),
        "ワ" => Some("wa".to_owned()),
        "ヰ" => Some("wi".to_owned()),
        "ヱ" => Some("we".to_owned()),
        "ヲ" => Some("wo".to_owned()),
        "ン" => Some("nn".to_owned()),
        "ー" => Some("-".to_owned()),
        // you-on
        "キャ" => Some("kya".to_owned()),
        "キュ" => Some("kyu".to_owned()),
        "キョ" => Some("kyo".to_owned()),
        "シャ" => Some("sha".to_owned()),
        "シュ" => Some("shu".to_owned()),
        "ショ" => Some("sho".to_owned()),
        "チャ" => Some("cha".to_owned()),
        "チュ" => Some("chu".to_owned()),
        "チョ" => Some("cho".to_owned()),
        "ニャ" => Some("nya".to_owned()),
        "ニュ" => Some("nyu".to_owned()),
        "ニョ" => Some("nyo".to_owned()),
        "ヒャ" => Some("hya".to_owned()),
        "ヒュ" => Some("hyu".to_owned()),
        "ヒョ" => Some("hyo".to_owned()),
        "ミャ" => Some("mya".to_owned()),
        "ミュ" => Some("myu".to_owned()),
        "ミョ" => Some("myo".to_owned()),
        "リャ" => Some("rya".to_owned()),
        "リュ" => Some("ryu".to_owned()),
        "リョ" => Some("ryo".to_owned()),
        "ギャ" => Some("gya".to_owned()),
        "ギュ" => Some("gyu".to_owned()),
        "ギョ" => Some("gyo".to_owned()),
        "ジャ" => Some("ja".to_owned()),
        "ジュ" => Some("ju".to_owned()),
        "ジョ" => Some("jo".to_owned()),
        "ビャ" => Some("bya".to_owned()),
        "ビュ" => Some("byu".to_owned()),
        "ビョ" => Some("byo".to_owned()),
        "ピャ" => Some("pya".to_owned()),
        "ピュ" => Some("pyu".to_owned()),
        "ピョ" => Some("pyo".to_owned()),
        // wacky katakan you-on
        "ウェ" => Some("we".to_owned()),
        _ => None,
    }
}

fn japanese<Input>() -> impl Parser<Input, Output = TypingTarget>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1::<Vec<Vec<DisplayedTypedPair>>, _, _>(choice((
        kana_chunk(),
        parenthetical().map(|x| vec![x]),
    )))
    .map(|part| {
        let mut typed_chunks = vec![];
        let mut displayed_chunks = vec![];

        for f in part.iter().cloned().flatten() {
            displayed_chunks.push(f.0);
            typed_chunks.push(f.1);
        }

        TypingTarget {
            typed_chunks,
            displayed_chunks,
            ..Default::default()
        }
    })
}

fn parenthetical<Input>() -> impl Parser<Input, Output = DisplayedTypedPair>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(satisfy(|c| c != '('))
        .and(between(
            token('('),
            token(')'),
            many::<Vec<Vec<DisplayedTypedPair>>, _, _>(kana_chunk()),
        ))
        .map(|(outside, inside): (String, _)| {
            // anything in a parenthetical has to be typed as one chunk, even
            // if it is composed of multiple kana.
            let typed = inside
                .iter()
                .flatten()
                .fold("".to_owned(), |mut acc, item| {
                    acc.push_str(&item.1);
                    acc
                });
            DisplayedTypedPair(outside, typed)
        })
}

fn kana_chunk<Input>() -> impl Parser<Input, Output = Vec<DisplayedTypedPair>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        optional(one_of(SOKUON.chars())),
        one_of(HIRAGANA.chars()).or(one_of(KATAKANA.chars())),
        optional(one_of(SUTEGANA.chars())),
    )
        .map(
            |(sokuon, hiragana, sutegana): (Option<char>, char, Option<char>)| {
                let mut pairs = vec![];

                let mut combined = String::from(hiragana);
                if let Some(sutegana) = sutegana {
                    combined.push(sutegana);
                }

                // this not being Some should probably be a parse error, but
                // I'm not sure how to do that from the middle of this .map
                if let Some(typed) = kana_to_typed_chunk(&combined) {
                    if let Some(sokuon) = sokuon {
                        // TODO does this work in all cases?
                        pairs.push(DisplayedTypedPair(
                            sokuon.into(),
                            typed.chars().next().unwrap().into(),
                        ));
                    }

                    pairs.push(DisplayedTypedPair(combined, typed));
                }

                pairs
            },
        )
}
