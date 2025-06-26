use winnow::{
    ascii::{line_ending, space0, till_line_ending},
    combinator::{alt, delimited, opt, preceded, repeat, terminated},
    token::{any, take_until},
};

use super::prelude::*;

#[cfg(feature = "trivia")]
#[derive(Debug, Clone, PartialEq)]
pub enum Trivia<'s> {
    LineComment(&'s str),
    BlockComment(&'s str),
    UnterminatedBlockComment(&'s str),
    NewLine,
}

#[cfg(feature = "trivia")]
impl std::fmt::Display for Trivia<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Trivia::*;
        match self {
            LineComment(s) => {
                // f.write_str("//")?;
                // f.write_str(s)?;
                // f.write_str("\n")?;
            }
            BlockComment(s) => {
                for (i, line) in s.split('\n').enumerate() {
                    if i == 0 {
                        write!(f, "/*{line}")?;
                    } else {
                        writeln!(f)?;
                        write!(f, "{line}")?;
                    }
                }
                write!(f, "*/")?;
            }
            UnterminatedBlockComment(s) => {
                for (i, line) in s.split('\n').enumerate() {
                    if i == 0 {
                        write!(f, "/*{line}")?;
                    } else {
                        writeln!(f)?;
                        write!(f, "{line}")?;
                    }
                }
                write!(f, "*/")?;
            }
            NewLine => {
                //  f.write_str("\n")?;
            }
        }
        Ok(())
    }
}

#[cfg(not(feature = "trivia"))]
pub type Trivia<'s> = ();

/// 行注释，及末尾的换行
fn line_comment<'s>(i: &mut Input<'s>) -> Result<Trivia<'s>> {
    let parser = delimited((space0, "//"), till_line_ending, opt(line_ending));

    #[cfg(feature = "trivia")]
    {
        parser.map(|s: &str| Trivia::LineComment(s)).parse_next(i)
    }
    #[cfg(not(feature = "trivia"))]
    {
        parser.value(()).parse_next(i)
    }
}

/// 块注释
fn block_comment<'s>(i: &mut Input<'s>) -> Result<Trivia<'s>> {
    let (mapper1, mapper2);
    #[cfg(feature = "trivia")]
    {
        (mapper1, mapper2) = (
            |s: &'s str| Trivia::BlockComment(s),
            |s: &'s str| Trivia::UnterminatedBlockComment(s),
        );
    }
    #[cfg(not(feature = "trivia"))]
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
    #[cfg(feature = "trivia")]
    {
        result = Trivia::NewLine;
    }
    #[cfg(not(feature = "trivia"))]
    {
        result = ();
    }
    (space0, line_ending).value(result).parse_next(i)
}

#[cfg(feature = "trivia")]
type TriviaListResult<'s> = Vec<Trivia<'s>>;
#[cfg(not(feature = "trivia"))]
type TriviaListResult<'s> = ();

pub(super) fn leading_trivia<'s>(i: &mut Input<'s>) -> Result<TriviaListResult<'s>> {
    repeat(0.., alt((line_comment, block_comment, new_line))).parse_next(i)
}

pub(super) fn tailing_trivia<'s>(i: &mut Input<'s>) -> Result<TriviaListResult<'s>> {
    // 目前仅解析行末的注释，后续可以增加更复杂的规则
    repeat(0..1, line_comment).parse_next(i)
}
