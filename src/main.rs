use core::num;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;
use std::process::Command;

use unicode_ident::{is_xid_continue, is_xid_start};
use winnow::ascii::multispace1;
use winnow::combinator::{cut_err, repeat_till, seq};
use winnow::error::{ErrMode, ParserError, StrContext, StrContextValue};
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{none_of, take, take_until, take_while};
use winnow::{LocatingSlice, Result};
use winnow::{
    ascii::{digit1 as digits, multispace0},
    combinator::alt,
    combinator::dispatch,
    combinator::eof,
    combinator::fail,
    combinator::opt,
    combinator::peek,
    combinator::repeat,
    combinator::{delimited, preceded, terminated},
    error::ContextError,
    stream::TokenSlice,
    token::any,
    token::literal,
    token::one_of,
    token::take_till,
};

mod string_parser;
mod tokenizer;

type Input<'a> = &'a str;

#[derive(Debug, PartialEq)]
enum Whitespace<'a> {
    LineComment(&'a str),
    BlockComment(&'a str),
    Spaces(&'a str),
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F, O>(inner: F) -> impl Parser<Input<'a>, O, ErrMode<ContextError>>
where
    F: Parser<Input<'a>, O, ErrMode<ContextError>>,
{
    delimited(whitespace, inner, whitespace)
}

fn whitespace<'a>(i: &mut Input<'a>) -> ModalResult<Vec<Whitespace<'a>>> {
    repeat(
        0..,
        alt((
            multispace1.map(Whitespace::Spaces),
            line_comment,
            block_comment,
        )),
    )
    .parse_next(i)
    .map(|spaces: Vec<Whitespace<'a>>| spaces)
}

fn line_comment<'a>(i: &mut Input<'a>) -> ModalResult<Whitespace<'a>> {
    delimited("//", take_till(0.., '\n'), "\n")
        .parse_next(i)
        .map(|content| Whitespace::LineComment(content))
}

fn block_comment<'a>(i: &mut Input<'a>) -> ModalResult<Whitespace<'a>> {
    delimited("/*", take_until(0.., "*/"), "*/")
        .parse_next(i)
        .map(|content| Whitespace::BlockComment(content))
}

fn identifier<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    (
        one_of(|c: char| c == '_' || c == '$' || is_xid_start(c)),
        take_while(0.., |c: char| c == '$' || is_xid_continue(c)),
    )
        .take()
        .map(Expression::Identifier)
        .parse_next(i)
}

fn number<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    dispatch! {peek(opt(take(2usize)));
        Some("0x") => cut_err(take_while(1.., is_xid_continue).verify(|s: &str| s.chars().all(|c| c.is_ascii_hexdigit())))
            .context(StrContext::Label("digit"))
            .context(StrContext::Expected(StrContextValue::Description("hexadecimal"))),
        Some("0o") => cut_err(take_while(1.., is_xid_continue).verify(|s: &str| s.chars().all(|c| ('0'..='7').contains(&c))))
            .context(StrContext::Label("digit"))
            .context(StrContext::Expected(StrContextValue::Description("octal"))),
        Some("0b") => cut_err(take_while(1.., is_xid_continue).verify(|s: &str| s.chars().all(|c| c == '0' || c == '1')))
            .context(StrContext::Label("digit"))
            .context(StrContext::Expected(StrContextValue::Description("binary"))),
        _ => (
                 digits,
                 opt(('.', digits)),
                 opt((one_of(['e', 'E']), opt(one_of(['+', '-'])), digits)),
             )
             .take()
    }
    .take()
    .map(Expression::Number)
    .parse_next(i)
}

#[derive(Debug, PartialEq)]
enum Expression<'a> {
    // primary
    Number(&'a str),
    Identifier(&'a str),
    Grouping(Box<Expression<'a>>),
    Tuple(Vec<Expression<'a>>),
    NamedTuple(Vec<(&'a str, Expression<'a>)>),

    // function
    Call(Box<Expression<'a>>, Vec<Expression<'a>>),
    Access(Box<Expression<'a>>, &'a str),

    // unary
    Not(Box<Expression<'a>>),
    Negate(Box<Expression<'a>>),
    Plus(Box<Expression<'a>>),

    // exponent
    Exponent(Box<Expression<'a>>, Box<Expression<'a>>),

    // factor
    Multiply(Box<Expression<'a>>, Box<Expression<'a>>),
    Divide(Box<Expression<'a>>, Box<Expression<'a>>),
    Modulo(Box<Expression<'a>>, Box<Expression<'a>>),

    // term
    Add(Box<Expression<'a>>, Box<Expression<'a>>),
    Subtract(Box<Expression<'a>>, Box<Expression<'a>>),

    // and
    And(Box<Expression<'a>>, Box<Expression<'a>>),

    // or
    Or(Box<Expression<'a>>, Box<Expression<'a>>),
}

fn primary<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    ws(alt((
        identifier,
        number,
        delimited("(", ws(expression), ")"),
    )))
    .parse_next(i)
}

fn unary<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    preceded(
        whitespace,
        dispatch! {peek(any);
            '+' => seq!(Expression::Plus(_:"+", unary.map(Box::new))),
            '-' => seq!(Expression::Negate(_:"-", unary.map(Box::new))),
            'n' => alt((
                seq!(Expression::Not(_:"not", _: peek(none_of(is_xid_continue)), unary.map(Box::new))),
                primary
            )),
            _ => primary,
        },
    )
    .parse_next(i)
}

// fn and<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {}

// fn or<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {}

fn expression<'a>(i: &mut Input<'a>) -> ModalResult<Expression<'a>> {
    unary.parse_next(i)
}

fn main() {
    let text = r##"
let a = "dawsfd\0\x00\x\"";
fn x(a, b, c) {
    return a + b * c;
}
fn y(a, b, c) {
    a + b * c
}
fn z(p) {
    p.1.2 + 1.2
}
loop {
    break;
}
for x in y {
    continue;
}
let b = a + if x {
    1
} else {
    2
}
"##;

    let mut input = LocatingSlice::new(text);
    let mut result: Option<tokenizer::Token<'_>> = None;
    loop {
        let parsed = tokenizer::tokenizer(&mut input, &result);
        match parsed {
            Ok(token) => {
                result = Some(token);
            }
            Err(err) => {
                eprintln!("{:} {:?}", err, input);
                return;
            }
        }
        if !matches!(
            result.as_ref().unwrap().kind,
            tokenizer::TokenKind::Whitespace(_)
        ) {
            println!("{:?}", result.as_ref().unwrap());
        }
        if result.as_ref().unwrap().kind == tokenizer::TokenKind::Eof {
            break;
        }
    }
}
