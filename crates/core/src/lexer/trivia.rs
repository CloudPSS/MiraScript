use winnow::{
    ascii::{line_ending, space0, till_line_ending},
    combinator::{alt, delimited, opt, preceded, repeat, terminated},
    token::{any, take_until},
};

use super::prelude::*;

#[cfg(feature = "formatter")]
#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum Trivia<'s> {
    LineComment(&'s str, SourceRange),
    BlockComment(&'s str, SourceRange),
    UnterminatedBlockComment(&'s str, SourceRange),
    NewLine(SourceRange),
}

#[cfg(feature = "formatter")]
impl Trivia<'_> {
    pub fn range(&self) -> &SourceRange {
        match self {
            Self::LineComment(_, range)
            | Self::BlockComment(_, range)
            | Self::UnterminatedBlockComment(_, range)
            | Self::NewLine(range) => range,
        }
    }
}

#[cfg(not(feature = "formatter"))]
pub type Trivia<'s> = ();

/// 行注释，及末尾的换行
fn line_comment<'s>(i: &mut Input<'s>) -> Result<Trivia<'s>> {
    let parser = delimited((space0, "//"), till_line_ending, opt(line_ending));

    #[cfg(feature = "formatter")]
    {
        parser
            .with_span()
            .map(|(s, range): (&str, SourceRange)| Trivia::LineComment(s, range))
            .parse_next(i)
    }
    #[cfg(not(feature = "formatter"))]
    {
        parser.value(()).parse_next(i)
    }
}

/// 块注释
fn block_comment<'s>(i: &mut Input<'s>) -> Result<Trivia<'s>> {
    let parser = preceded(
        (space0, "/*"),
        alt((
            terminated(take_until(0.., "*/"), "*/").map(|s: &str| (s, true)),
            repeat::<_, _, String, _, _>(0.., any)
                .take()
                .map(|s: &str| (s, false)),
        )),
    );
    #[cfg(feature = "formatter")]
    {
        parser
            .with_span()
            .map(|((s, terminated), range): ((&str, bool), SourceRange)| {
                if terminated {
                    Trivia::BlockComment(s, range)
                } else {
                    Trivia::UnterminatedBlockComment(s, range)
                }
            })
            .parse_next(i)
    }
    #[cfg(not(feature = "formatter"))]
    {
        parser.value(()).parse_next(i)
    }
}

/// 折行
#[cfg(feature = "formatter")]
fn new_line<'s>(i: &mut Input<'s>) -> Result<Trivia<'s>> {
    (space0, line_ending)
        .with_span()
        .map(|(_, range)| Trivia::NewLine(range))
        .parse_next(i)
}

#[cfg(not(feature = "formatter"))]
fn new_line<'s>(i: &mut Input<'s>) -> Result<Trivia<'s>> {
    (space0, line_ending).value(()).parse_next(i)
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
fn to_trivia_list<'s>(_: ()) -> TriviaList<'s> {}

pub(super) fn leading_trivia<'s>(i: &mut Input<'s>) -> Result<TriviaList<'s>> {
    repeat(0.., alt((line_comment, block_comment, new_line)))
        .map(to_trivia_list)
        .parse_next(i)
}

pub(super) fn tailing_trivia<'s>(i: &mut Input<'s>) -> Result<TriviaList<'s>> {
    #[cfg(feature = "formatter")]
    fn block_comment_verifier(s: &Trivia<'_>) -> bool {
        let Trivia::BlockComment(s, _) = s else {
            return false;
        };
        // 不包含换行，不是文档注释
        !s.contains('\n') && !s.starts_with("*")
    }
    #[cfg(not(feature = "formatter"))]
    fn block_comment_verifier(_: &Trivia<'_>) -> bool {
        true
    }
    // 末尾的注释和换行
    alt((
        (
            repeat(1.., block_comment.verify(block_comment_verifier)),
            alt((line_comment, new_line)),
        )
            .map(|(b, e): (Vec<_>, _)| {
                #[cfg_attr(not(feature = "formatter"), allow(clippy::unit_arg))]
                to_trivia_list(b.into_iter().chain([e]).collect())
            }),
        repeat(0..=1, alt((line_comment, new_line))).map(to_trivia_list),
    ))
    .parse_next(i)
}
