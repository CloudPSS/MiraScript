use winnow::{
    ModalResult, Parser,
    combinator::{Repeat, fail, peek, repeat},
    error::{ContextError, ErrMode},
    stream::Stream,
    token::{any, one_of},
};

use crate::lexer::{Keyword, Operator, Token, TokenKind};

use super::{
    Input,
    helper::{token_boxed, token_or_insert},
    record_elements::RecordElementBase,
};

fn record_name<'a>(i: &mut Input<'_, 'a>) -> ModalResult<Token<'a>> {
    one_of(|t: &Token<'a>| matches!(t.kind, TokenKind::Identifier(_) | TokenKind::Ordinal(_)))
        .map(ToOwned::to_owned)
        .parse_next(i)
}

struct RecordBaseParser<
    't,
    'a: 't,
    Named,
    OmitNamed,
    Unnamed,
    Spread,
    NamedParser: Parser<Input<'t, 'a>, Named, ErrMode<ContextError>>,
    OmitNamedParser: Parser<Input<'t, 'a>, OmitNamed, ErrMode<ContextError>>,
    UnnamedParser: Parser<Input<'t, 'a>, Unnamed, ErrMode<ContextError>>,
    SpreadParser: Parser<Input<'t, 'a>, Spread, ErrMode<ContextError>>,
> {
    named: NamedParser,
    omit_named: OmitNamedParser,
    unnamed: UnnamedParser,
    spread: SpreadParser,

    _phantom: std::marker::PhantomData<&'t &'a (Named, OmitNamed, Unnamed, Spread)>,
}
impl<
    't,
    'a: 't,
    Named: Clone + PartialEq,
    OmitNamed: Clone + PartialEq,
    Unnamed: Clone + PartialEq,
    Spread: Clone + PartialEq,
    NamedParser: Parser<Input<'t, 'a>, Named, ErrMode<ContextError>>,
    OmitNamedParser: Parser<Input<'t, 'a>, OmitNamed, ErrMode<ContextError>>,
    UnnamedParser: Parser<Input<'t, 'a>, Unnamed, ErrMode<ContextError>>,
    SpreadParser: Parser<Input<'t, 'a>, Spread, ErrMode<ContextError>>,
>
    Parser<
        Input<'t, 'a>,
        RecordElementBase<'a, Named, OmitNamed, Unnamed, Spread>,
        ErrMode<ContextError>,
    >
    for &mut RecordBaseParser<
        't,
        'a,
        Named,
        OmitNamed,
        Unnamed,
        Spread,
        NamedParser,
        OmitNamedParser,
        UnnamedParser,
        SpreadParser,
    >
{
    fn parse_next(
        &mut self,
        i: &mut Input<'t, 'a>,
    ) -> ModalResult<RecordElementBase<'a, Named, OmitNamed, Unnamed, Spread>> {
        let first = peek(any).parse_next(i)?;
        if *first == Operator::CloseParen {
            return fail.parse_next(i);
        }
        let mut result = if *first == Operator::SpreadRange {
            let s = token_boxed(Operator::SpreadRange).parse_next(i)?;
            let e = self.spread.parse_next(i)?;
            RecordElementBase::Spread(s, e, None)
        } else if *first == Operator::Colon {
            let c = token_boxed(Operator::Colon).parse_next(i)?;
            let o = self.omit_named.parse_next(i)?;
            RecordElementBase::OmitNamed(c, o, None)
        } else {
            let cp = i.checkpoint();
            let record_name = record_name.parse_next(i);
            let token_boxed = token_boxed(Operator::Colon).parse_next(i);
            let e = self.named.parse_next(i);
            if record_name.is_err() || token_boxed.is_err() || e.is_err() {
                i.reset(&cp);
                let o = self.unnamed.parse_next(i)?;
                RecordElementBase::Unnamed(o, None)
            } else {
                let record_name = record_name.unwrap();
                let token_boxed = token_boxed.unwrap();
                let e = e.unwrap();
                RecordElementBase::Named(Box::new(record_name), token_boxed, e, None)
            }
        };
        let last = peek(any).parse_next(i)?;
        if *last == Operator::CloseParen
            || *last == Operator::CloseBrace
            || *last == Operator::CloseBracket
            || *last == Operator::Semicolon
            || *last == TokenKind::Eof
            || *last == Keyword::Return
            || *last == Keyword::Break
            || *last == Keyword::Continue
            || *last == Keyword::Case
            || *last == Keyword::Else
            || *last == Keyword::In
            || *last == Keyword::Let
        {
            return Ok(result);
        }
        let comma = token_or_insert(Operator::Comma, "Missing comma").parse_next(i)?;
        result.set_tail_comma(Box::new(comma));
        Ok(result)
    }
}

type RecordBaseParserResult<'a, Named, OmitNamed, Unnamed, Spread> = (
    Box<Token<'a>>,
    Vec<RecordElementBase<'a, Named, OmitNamed, Unnamed, Spread>>,
    Box<Token<'a>>,
);

impl<
    't,
    'a: 't,
    Named: Clone + PartialEq,
    OmitNamed: Clone + PartialEq,
    Unnamed: Clone + PartialEq,
    Spread: Clone + PartialEq,
    NamedParser: Parser<Input<'t, 'a>, Named, ErrMode<ContextError>>,
    OmitNamedParser: Parser<Input<'t, 'a>, OmitNamed, ErrMode<ContextError>>,
    UnnamedParser: Parser<Input<'t, 'a>, Unnamed, ErrMode<ContextError>>,
    SpreadParser: Parser<Input<'t, 'a>, Spread, ErrMode<ContextError>>,
>
    Parser<
        Input<'t, 'a>,
        RecordBaseParserResult<'a, Named, OmitNamed, Unnamed, Spread>,
        ErrMode<ContextError>,
    >
    for RecordBaseParser<
        't,
        'a,
        Named,
        OmitNamed,
        Unnamed,
        Spread,
        NamedParser,
        OmitNamedParser,
        UnnamedParser,
        SpreadParser,
    >
{
    fn parse_next(
        &mut self,
        i: &mut Input<'t, 'a>,
    ) -> ModalResult<RecordBaseParserResult<'a, Named, OmitNamed, Unnamed, Spread>> {
        let open = token_boxed(Operator::OpenParen).parse_next(i)?;
        let mut repeat: Repeat<_, _, RecordElementBase<'a, _, _, _, _>, _, _> = repeat(0.., self);
        let parts: Vec<RecordElementBase<'a, Named, OmitNamed, Unnamed, Spread>> =
            repeat.parse_next(i)?;
        let close = token_or_insert(Operator::CloseParen, "Missing ')'")
            .map(Box::new)
            .parse_next(i)?;
        Ok((open, parts, close))
    }
}
pub(super) fn record_base<
    't,
    'a: 't,
    Named: Clone + PartialEq + 'a,
    OmitNamed: Clone + PartialEq + 'a,
    Unnamed: Clone + PartialEq + 'a,
    Spread: Clone + PartialEq + 'a,
>(
    named: impl Parser<Input<'t, 'a>, Named, ErrMode<ContextError>>,
    omit_named: impl Parser<Input<'t, 'a>, OmitNamed, ErrMode<ContextError>>,
    unnamed: impl Parser<Input<'t, 'a>, Unnamed, ErrMode<ContextError>>,
    spread: impl Parser<Input<'t, 'a>, Spread, ErrMode<ContextError>>,
) -> impl Parser<
    Input<'t, 'a>,
    RecordBaseParserResult<'a, Named, OmitNamed, Unnamed, Spread>,
    ErrMode<ContextError>,
> {
    RecordBaseParser {
        named,
        omit_named,
        unnamed,
        spread,
        _phantom: std::marker::PhantomData,
    }
}
