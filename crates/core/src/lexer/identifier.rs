use std::str::FromStr;

use winnow::{
    combinator::{alt, repeat},
    token::{literal, one_of, take_while},
};

use super::prelude::*;

pub(super) const IDENTIFIER_SPECIAL: &[char] = &['_', '$', '@'];

pub(super) fn is_identifier_special(ch: char) -> bool {
    IDENTIFIER_SPECIAL.contains(&ch)
}
pub(super) fn is_identifier_start(ch: char) -> bool {
    unicode_ident::is_xid_start(ch)
}
pub(super) fn is_identifier_continue(ch: char) -> bool {
    unicode_ident::is_xid_continue(ch)
}

pub(super) fn identifier<'s>(keyword: bool) -> impl Parser<'s, TokenKind<'s>> {
    move |i: &mut Input<'s>| {
        (
            alt((
                one_of(is_identifier_start).void(),
                repeat::<_, _, String, _, _>(1.., literal("_")).void(),
                repeat::<_, _, String, _, _>(1.., literal("$")).void(),
                repeat::<_, _, String, _, _>(1.., literal("@")).void(),
            )),
            take_while(0.., is_identifier_continue),
        )
            .take()
            .map(|s| {
                if keyword {
                    if let Ok(kw) = Keyword::from_str(s) {
                        return TokenKind::Keyword(kw, Some(s));
                    }
                }
                TokenKind::Identifier(s)
            })
            .parse_next(i)
    }
}
