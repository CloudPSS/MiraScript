use winnow::{
    ascii::{line_ending, space0, till_line_ending},
    combinator::{alt, delimited, opt, preceded, repeat, terminated},
    token::{any, take_until},
};

use super::prelude::*;

#[cfg(feature = "formatter")]
#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum Trivia<'s> {
    LineComment(&'s str),
    BlockComment(&'s str),
    UnterminatedBlockComment(&'s str),
    NewLine,
}

#[cfg(not(feature = "formatter"))]
pub type Trivia<'s> = ();

/// 行注释，及末尾的换行
fn line_comment<'s>(i: &mut Input<'s>) -> Result<Trivia<'s>> {
    let parser = delimited((space0, "//"), till_line_ending, opt(line_ending));

    #[cfg(feature = "formatter")]
    {
        parser.map(|s: &str| Trivia::LineComment(s)).parse_next(i)
    }
    #[cfg(not(feature = "formatter"))]
    {
        parser.value(()).parse_next(i)
    }
}

/// 块注释
fn block_comment<'s>(i: &mut Input<'s>) -> Result<Trivia<'s>> {
    let (mapper1, mapper2);
    #[cfg(feature = "formatter")]
    {
        (mapper1, mapper2) = (
            |s: &'s str| Trivia::BlockComment(s),
            |s: &'s str| Trivia::UnterminatedBlockComment(s),
        );
    }
    #[cfg(not(feature = "formatter"))]
    {
        (mapper1, mapper2) = (|_: &'s str| (), |_: &'s str| ())
    }
    preceded(
        (space0, "/*"),
        alt((
            terminated(take_until(0.., "*/").map(mapper1), "*/"),
            repeat::<_, _, String, _, _>(0.., any).take().map(mapper2),
        )),
    )
    .parse_next(i)
}

/// 折行
fn new_line<'s>(i: &mut Input<'s>) -> Result<Trivia<'s>> {
    let result;
    #[cfg(feature = "formatter")]
    {
        result = Trivia::NewLine;
    }
    #[cfg(not(feature = "formatter"))]
    {
        result = ();
    }
    (space0, line_ending).value(result).parse_next(i)
}

#[cfg(feature = "formatter")]
pub type TriviaList<'s> = Box<[Trivia<'s>]>;
#[cfg(not(feature = "formatter"))]
pub type TriviaList<'s> = ();

#[cfg(feature = "formatter")]
fn to_trivia_list<'s>(v: Vec<Trivia<'s>>) -> TriviaList<'s> {
    v.into_boxed_slice()
}
#[cfg(not(feature = "formatter"))]
fn to_trivia_list<'s>(_: ()) -> TriviaList<'s> {
    ()
}

pub(super) fn leading_trivia<'s>(i: &mut Input<'s>) -> Result<TriviaList<'s>> {
    repeat(0.., alt((line_comment, block_comment, new_line)))
        .map(to_trivia_list)
        .parse_next(i)
}

pub(super) fn tailing_trivia<'s>(i: &mut Input<'s>) -> Result<TriviaList<'s>> {
    // 末尾的注释和换行
    alt((
        (repeat(1.., block_comment), alt((line_comment, new_line)))
            .map(|(b, e): (Vec<_>, _)| to_trivia_list(b.into_iter().chain([e]).collect())),
        repeat(0..=1, alt((line_comment, new_line))).map(to_trivia_list),
    ))
    .parse_next(i)
}
