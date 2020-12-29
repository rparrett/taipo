use crate::TypingTarget;
use nom::{
    bytes::complete::is_not,
    character::complete::{char, line_ending, space0},
    multi::{fold_many0, separated_list0},
    sequence::{delimited, pair},
    IResult,
};

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
