use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use crate::{
    ansi::{DisplayIdent, GROUP, RANGE, RECOVER, RESET},
    error::{ErrorCode, SourceError, SourceRange},
    lexer::Token,
};

use super::{ArrayPattern, RecordPattern};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum Pattern<'a> {
    /// `(` pattern `)`
    ///
    /// Grouping pattern.
    Grouping(Box<Token<'a>>, Box<Pattern<'a>>, Box<Token<'a>>),
    /// ( `+` | `-` )? literal
    ///
    /// Matches against a constant value.
    Constant(Option<Box<Token<'a>>>, Box<Token<'a>>),
    /// ( `>` | `>=` | `<=` | `<` | `==` | `!=` | `~=` | `!~=` ) pattern_constant
    ///
    /// Matches against a relation with constant values.
    Relation(Box<Token<'a>>, Box<Pattern<'a>>),
    /// literal ( `..` | `..<` ) pattern_constant
    ///
    /// Matches against a range of constant values.
    Range(Box<Pattern<'a>>, Box<Token<'a>>, Box<Pattern<'a>>),
    /// `_`
    ///
    /// Matches and discards a value.
    Discard(Box<Token<'a>>),
    /// `mut`? identifier
    ///
    /// Matches and binds a value to a variable.
    Bind(Option<Box<Token<'a>>>, Box<Token<'a>>),
    /// ``````antlr
    /// pattern_record
    ///     : '(' sub_pattern* ')'
    ///     ;
    /// sub_pattern
    ///     : name colon pattern ','?
    ///     | colon pattern_bind ','?
    ///     | pattern ','?
    ///     | '..' pattern ','?
    ///     ;
    /// colon
    ///     : ':'
    ///     | '?:'
    ///     | '!:'
    ///     ;
    /// ``````
    /// Matches a record pattern.
    Record(Box<Token<'a>>, Vec<RecordPattern<'a>>, Box<Token<'a>>),
    /// ```antlr
    /// pattern_array
    ///     : '[' sub_pattern* ']'
    ///     ;
    /// sub_pattern
    ///     : pattern ','?
    ///     | '..' pattern? ','?
    ///     ;
    /// ```
    Array(Box<Token<'a>>, Vec<ArrayPattern<'a>>, Box<Token<'a>>),
    /// prefix<`..`>
    ///
    /// Contains no token.
    /// Used in [ArrayPattern]::Spread and [RecordPattern]::Spread
    /// as a placeholder since `_`
    /// must be omitted in these cases.
    SpreadDiscard,

    /// pattern `and` pattern
    ///
    /// Matches all of the patterns.
    And(Box<Pattern<'a>>, Box<Token<'a>>, Box<Pattern<'a>>),
    /// pattern `or` pattern
    ///
    /// Matches any of the patterns.
    Or(Box<Pattern<'a>>, Box<Token<'a>>, Box<Pattern<'a>>),
    /// `not` pattern
    ///
    /// Matches if the pattern does not match.
    Not(Box<Token<'a>>, Box<Pattern<'a>>),

    /// Unknown pattern.
    Unknown {
        pattern: Option<Box<Pattern<'a>>>,
        tokens: Vec<Token<'a>>,
        errors: Vec<SourceError>,
    },
}

impl<'a> Pattern<'a> {
    pub(crate) fn wrap_as_unknown<T: Into<Vec<Token<'a>>>>(
        self,
        tokens: T,
        error: ErrorCode,
    ) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Pattern::Unknown {
            pattern: Some(Box::new(self)),
            tokens,
            errors: vec![SourceError::new(range, error)],
        }
    }

    pub(crate) fn unknown<T: Into<Vec<Token<'a>>>>(tokens: T, error: ErrorCode) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Pattern::Unknown {
            pattern: None,
            tokens,
            errors: vec![SourceError::new(range, error)],
        }
    }
    pub(crate) fn unknown_range<T: Into<Vec<Token<'a>>>>(
        tokens: T,
        error_range: SourceRange,
        error: ErrorCode,
    ) -> Self {
        Pattern::Unknown {
            pattern: None,
            tokens: tokens.into(),
            errors: vec![SourceError::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<T: Into<Vec<Token<'a>>>, E: Into<Vec<SourceError>>>(
        tokens: T,
        errors: E,
    ) -> Self {
        Pattern::Unknown {
            pattern: None,
            tokens: tokens.into(),
            errors: errors.into(),
        }
    }
}

impl Display for Pattern<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Pattern<'_> {
    fn fmt_ident(&self, f: &mut Formatter<'_>, ident: usize) -> fmt::Result {
        use Pattern::*;
        match self {
            Grouping(op, p, cp) => {
                write!(f, "{GROUP}{op}{RESET}")?;
                p.fmt_ident(f, ident)?;
                write!(f, "{GROUP}{cp}{RESET}")?;
            }
            Constant(Some(prefix), token) => write!(f, "{prefix}{token}")?,
            Constant(None, token) => write!(f, "{token}")?,
            Relation(op, constant) => {
                op.fmt_ident(f, ident)?;
                write!(f, " ")?;
                constant.fmt_ident(f, ident)?;
            }
            Range(start, op, end) => {
                start.fmt_ident(f, ident)?;
                write!(f, "{RANGE}{op}{RESET}")?;
                end.fmt_ident(f, ident)?;
            }
            Discard(token) => write!(f, "{token}")?,
            Bind(None, token) => write!(f, "{token}")?,
            Bind(Some(kw_mut), token) => write!(f, "{kw_mut} {token}")?,
            Record(start, sub_patterns, end) => {
                write!(f, "{start}")?;
                for sub_pattern in sub_patterns.iter() {
                    sub_pattern.fmt_ident(f, ident)?;
                }
                write!(f, "{end}")?;
            }
            Array(start, sub_patterns, end) => {
                write!(f, "{start}")?;
                for sub_pattern in sub_patterns.iter() {
                    sub_pattern.fmt_ident(f, ident)?;
                }
                write!(f, "{end}")?;
            }
            SpreadDiscard => {}
            And(left, op, right) | Or(left, op, right) => {
                left.fmt_ident(f, ident)?;
                write!(f, " {op} ")?;
                right.fmt_ident(f, ident)?;
            }
            Not(op, pattern) => {
                write!(f, "{op} ")?;
                pattern.fmt_ident(f, ident)?;
            }
            Unknown {
                pattern: Some(p), ..
            } => {
                write!(f, "{RECOVER}<pattern{RESET}")?;
                p.fmt_ident(f, ident)?;
                write!(f, "{RECOVER}>{RESET}")?;
            }
            Unknown { tokens, .. } => {
                write!(f, "{RECOVER}<pattern{RESET}")?;
                for token in tokens {
                    write!(f, " {token}")?;
                }
                write!(f, "{RECOVER}>{RESET}")?;
            }
        }
        Ok(())
    }
}
