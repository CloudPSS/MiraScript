use std::fmt::Display;

use super::{Keyword, Operator, Token, TokenKind, Whitespace};

impl Display for TokenKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => write!(f, "␀"),
            Self::Whitespace(Whitespace::LineComment) => write!(f, "//"),
            Self::Whitespace(Whitespace::BlockComment) => write!(f, "/*"),
            Self::Whitespace(Whitespace::Spaces) => write!(f, " "),
            Self::Identifier(s) => write!(f, "{}", s),
            Self::Ordinal(n) => write!(f, "{}", n),
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Operator(op) => write!(
                f,
                "{}",
                match op {
                    Operator::OpenParen => "(",
                    Operator::CloseParen => ")",
                    Operator::Colon => ":",
                    Operator::Comma => ",",
                    Operator::Dot => ".",
                    Operator::Plus => "+",
                    Operator::Minus => "-",
                    Operator::Caret => "^",
                    Operator::Asterisk => "*",
                    Operator::Slash => "/",
                    Operator::Percent => "%",
                    Operator::Equal => "=",
                    Operator::EqualEqual => "==",
                    Operator::NotEqual => "!=",
                    Operator::Greater => ">",
                    Operator::GreaterEqual => ">=",
                    Operator::Less => "<",
                    Operator::LessEqual => "<=",
                    Operator::Semicolon => ";",
                    Operator::OpenBrace => "{",
                    Operator::CloseBrace => "}",
                }
            ),
            Self::Keyword(kw) => write!(
                f,
                "{}",
                match kw {
                    Keyword::and => "and",
                    Keyword::or => "or",
                    Keyword::not => "not",
                    Keyword::r#if => "if",
                    Keyword::r#else => "else",
                    Keyword::r#match => "match",
                    Keyword::r#for => "for",
                    Keyword::r#in => "in",
                    Keyword::r#while => "while",
                    Keyword::r#loop => "loop",
                    Keyword::r#break => "break",
                    Keyword::r#continue => "continue",
                    Keyword::r#return => "return",
                    Keyword::r#fn => "fn",
                    Keyword::op => "op",
                }
            ),
            Self::Unknown { recovered, .. } => {
                if let Some(recovered) = recovered {
                    write!(f, "<{}>", recovered)
                } else {
                    write!(f, "<?>")
                }
            }
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}
