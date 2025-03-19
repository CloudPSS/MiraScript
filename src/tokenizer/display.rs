use std::fmt::{Display, Write};

use super::{Keyword, Operator, Token, TokenKind, Whitespace};

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = format!("{:?}", self);
        d.make_ascii_lowercase();
        f.write_str(&d)
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = *self as u32;
        if v < 0x80 {
            let c = v as u8 as char;
            if c.is_ascii_graphic() {
                return f.write_char(c);
            }
        }
        if v < 0x8000 {
            let c1 = (v >> 8) as u8 as char;
            let c2 = v as u8 as char;
            return write!(f, "{}{}", c1, c2);
        }
        if v < 0x800000 {
            let c1 = (v >> 16) as u8 as char;
            let c2 = (v >> 8) as u8 as char;
            let c3 = v as u8 as char;
            return write!(f, "{}{}{}", c1, c2, c3);
        }
        let c1 = (v >> 24) as u8 as char;
        let c2 = (v >> 16) as u8 as char;
        let c3 = (v >> 8) as u8 as char;
        let c4 = v as u8 as char;
        write!(f, "{}{}{}{}", c1, c2, c3, c4)
    }
}

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
            Self::String(s) => write!(f, "{:?}", s),
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
            Self::Keyword(kw) => write!(f, "{}", kw),
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
        <TokenKind as Display>::fmt(&self.kind, f)
    }
}
