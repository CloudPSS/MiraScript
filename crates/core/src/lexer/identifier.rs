use std::borrow::Cow;
use std::str::FromStr;

use winnow::combinator::{alt, repeat, trace};
use winnow::prelude::*;
use winnow::token::{literal, one_of, take_while};

use super::{Input, Keyword, TokenKind};

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

pub(super) fn identifier<'s>(i: &mut Input<'s>) -> ModalResult<TokenKind<'s>> {
    trace(
        "identifier",
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
                if let Ok(kw) = Keyword::from_str(s) {
                    TokenKind::Keyword(kw)
                } else {
                    TokenKind::Identifier(Cow::Borrowed(s))
                }
            }),
    )
    .parse_next(i)
}
