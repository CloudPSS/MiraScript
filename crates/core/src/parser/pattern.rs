use std::fmt::{self, Display, Formatter};

use crate::{
    ansi::{DisplayIdent, GROUP, RANGE, RECOVER, RESET},
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    lexer::Token,
};

use super::{ArrayPattern, AstVisitor, AstVisitorMut, AstWalker, RecordPattern};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum Pattern<'s> {
    /// `(` pattern `)`
    ///
    /// Grouping pattern.
    Grouping(Box<Token<'s>>, Box<Pattern<'s>>, Box<Token<'s>>),
    /// ( `+` | `-` )? literal
    ///
    /// Matches against a constant value.
    Constant(Option<Box<Token<'s>>>, Box<Token<'s>>),
    /// ( `>` | `>=` | `<=` | `<` | `==` | `!=` | `~=` | `!~=` ) pattern_constant
    ///
    /// Matches against a relation with constant values.
    Relation(Box<Token<'s>>, Box<Pattern<'s>>),
    /// literal ( `..` | `..<` ) pattern_constant
    ///
    /// Matches against a range of constant values.
    Range(Box<Pattern<'s>>, Box<Token<'s>>, Box<Pattern<'s>>),
    /// `_`
    ///
    /// Matches and discards a value.
    Discard(Box<Token<'s>>),
    /// `mut`? identifier
    ///
    /// Matches and binds a value to a variable.
    Bind(Option<Box<Token<'s>>>, Box<Token<'s>>),
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
    ///     ;
    /// ``````
    /// Matches a record pattern.
    Record(Box<Token<'s>>, Vec<RecordPattern<'s>>, Box<Token<'s>>),
    /// ```antlr
    /// pattern_array
    ///     : '[' sub_pattern* ']'
    ///     ;
    /// sub_pattern
    ///     : pattern ','?
    ///     | '..' pattern? ','?
    ///     ;
    /// ```
    Array(Box<Token<'s>>, Vec<ArrayPattern<'s>>, Box<Token<'s>>),
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
    And(Box<Pattern<'s>>, Box<Token<'s>>, Box<Pattern<'s>>),
    /// pattern `or` pattern
    ///
    /// Matches any of the patterns.
    Or(Box<Pattern<'s>>, Box<Token<'s>>, Box<Pattern<'s>>),
    /// `not` pattern
    ///
    /// Matches if the pattern does not match.
    Not(Box<Token<'s>>, Box<Pattern<'s>>),

    /// Unknown pattern.
    Unknown {
        pattern: Option<Box<Pattern<'s>>>,
        tokens: Vec<Token<'s>>,
        errors: Vec<SourceDiagnostic>,
    },
}

impl<'s> Pattern<'s> {
    pub(crate) fn wrap_as_unknown<T: Into<Vec<Token<'s>>>>(
        self,
        tokens: T,
        error: DiagnosticCode,
    ) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Pattern::Unknown {
            pattern: Some(Box::new(self)),
            tokens,
            errors: vec![SourceDiagnostic::new(range, error)],
        }
    }

    pub(crate) fn unknown<T: Into<Vec<Token<'s>>>>(tokens: T, error: DiagnosticCode) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Pattern::Unknown {
            pattern: None,
            tokens,
            errors: vec![SourceDiagnostic::new(range, error)],
        }
    }
    pub(crate) fn unknown_range<T: Into<Vec<Token<'s>>>>(
        tokens: T,
        error_range: SourceRange,
        error: DiagnosticCode,
    ) -> Self {
        Pattern::Unknown {
            pattern: None,
            tokens: tokens.into(),
            errors: vec![SourceDiagnostic::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<T: Into<Vec<Token<'s>>>, E: Into<Vec<SourceDiagnostic>>>(
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

impl<'s> AstWalker<'s> for Pattern<'s> {
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        use Pattern::*;
        visitor.visit_pattern(self);
        match self {
            Grouping(o, p, e) => {
                o.walk_mut(visitor);
                p.walk_mut(visitor);
                e.walk_mut(visitor);
            }
            Constant(p, t) => {
                p.walk_mut(visitor);
                t.walk_mut(visitor);
            }
            Relation(o, p) => {
                o.walk_mut(visitor);
                p.walk_mut(visitor);
            }
            Range(s, o, e) => {
                s.walk_mut(visitor);
                o.walk_mut(visitor);
                e.walk_mut(visitor);
            }
            Discard(t) => t.walk_mut(visitor),
            Bind(p, t) => {
                p.walk_mut(visitor);
                t.walk_mut(visitor);
            }
            Record(s, p, e) => {
                s.walk_mut(visitor);
                for sub_pattern in p.iter_mut() {
                    sub_pattern.walk_mut(visitor);
                }
                e.walk_mut(visitor);
            }
            Array(s, p, e) => {
                s.walk_mut(visitor);
                for sub_pattern in p.iter_mut() {
                    sub_pattern.walk_mut(visitor);
                }
                e.walk_mut(visitor);
            }
            SpreadDiscard => {}
            And(l, o, r) => {
                l.walk_mut(visitor);
                o.walk_mut(visitor);
                r.walk_mut(visitor);
            }
            Or(l, o, r) => {
                l.walk_mut(visitor);
                o.walk_mut(visitor);
                r.walk_mut(visitor);
            }
            Not(o, p) => {
                o.walk_mut(visitor);
                p.walk_mut(visitor);
            }
            Unknown {
                pattern,
                tokens,
                errors: _,
            } => {
                pattern.walk_mut(visitor);
                tokens.walk_mut(visitor);
            }
        }
    }
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        use Pattern::*;
        visitor.visit_pattern(self);
        match self {
            Grouping(o, p, e) => {
                o.walk(visitor);
                p.walk(visitor);
                e.walk(visitor);
            }
            Constant(p, t) => {
                p.walk(visitor);
                t.walk(visitor);
            }
            Relation(o, p) => {
                o.walk(visitor);
                p.walk(visitor);
            }
            Range(s, o, e) => {
                s.walk(visitor);
                o.walk(visitor);
                e.walk(visitor);
            }
            Discard(t) => t.walk(visitor),
            Bind(p, t) => {
                p.walk(visitor);
                t.walk(visitor);
            }
            Record(s, p, e) => {
                s.walk(visitor);
                for sub_pattern in p.iter() {
                    sub_pattern.walk(visitor);
                }
                e.walk(visitor);
            }
            Array(s, p, e) => {
                s.walk(visitor);
                for sub_pattern in p.iter() {
                    sub_pattern.walk(visitor);
                }
                e.walk(visitor);
            }
            SpreadDiscard => {}
            And(l, o, r) => {
                l.walk(visitor);
                o.walk(visitor);
                r.walk(visitor);
            }
            Or(l, o, r) => {
                l.walk(visitor);
                o.walk(visitor);
                r.walk(visitor);
            }
            Not(o, p) => {
                o.walk(visitor);
                p.walk(visitor);
            }
            Unknown {
                pattern,
                tokens,
                errors: _,
            } => {
                pattern.walk(visitor);
                tokens.walk(visitor);
            }
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
