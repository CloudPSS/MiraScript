use std::fmt::Display;

use winnow::ascii::{line_ending, space0, till_line_ending};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, terminated};
use winnow::prelude::*;
use winnow::token::{any, take_until};

use crate::ansi::{COMMENT, DisplayIdent, RECOVER, RESET};

use super::Input;

#[derive(Debug, Clone, PartialEq)]
pub enum Trivia<'a> {
    LineComment(&'a str),
    BlockComment(&'a str),
    UnterminatedBlockComment(&'a str),
    EmptyLine,
}

impl Display for Trivia<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Trivia<'_> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
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
                        write!(f, "{COMMENT}/*{line}{RESET}")?;
                    } else {
                        writeln!(f)?;
                        Self::write_ident(f, ident, "</*>")?;
                        write!(f, "{COMMENT}{line}{RESET}")?;
                    }
                }
                write!(f, "{COMMENT}*/{RESET}")?;
            }
            UnterminatedBlockComment(s) => {
                for (i, line) in s.split('\n').enumerate() {
                    if i == 0 {
                        write!(f, "{COMMENT}/*{line}{RESET}")?;
                    } else {
                        writeln!(f)?;
                        Self::write_ident(f, ident, "</*?")?;
                        write!(f, "{COMMENT}{line}{RESET}")?;
                    }
                }
                write!(f, "{RECOVER}{COMMENT}*/{RESET}")?;
            }
            EmptyLine => {
                //  f.write_str("\n")?;
            }
        }
        Ok(())
    }
}

fn line_comment<'a>(i: &mut Input<'a>) -> ModalResult<Trivia<'a>> {
    delimited((space0, "//"), till_line_ending, opt(line_ending))
        .map(|s: &str| Trivia::LineComment(s))
        .parse_next(i)
}

fn block_comment<'a>(i: &mut Input<'a>) -> ModalResult<Trivia<'a>> {
    preceded(
        (space0, "/*"),
        alt((
            terminated(
                take_until(0.., "*/")
                    .take()
                    .map(|s: &str| Trivia::BlockComment(s)),
                "*/",
            ),
            repeat::<_, _, String, _, _>(0.., any)
                .take()
                .map(|s: &str| Trivia::UnterminatedBlockComment(s)),
        )),
    )
    .parse_next(i)
}

fn empty_line<'a>(i: &mut Input<'a>) -> ModalResult<Trivia<'a>> {
    (space0, line_ending)
        .map(|_| Trivia::EmptyLine)
        .parse_next(i)
}

pub(super) fn trivia<'a>(i: &mut Input<'a>) -> ModalResult<Trivia<'a>> {
    alt((line_comment, block_comment, empty_line)).parse_next(i)
}

pub(super) fn trivia_list<'a>(i: &mut Input<'a>) -> ModalResult<Vec<Trivia<'a>>> {
    repeat(0.., trivia).parse_next(i)
}
