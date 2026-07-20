use crate::parser::helper::unknown_range;

use super::prelude::*;

#[derive(Debug, PartialEq, strum::EnumIs)]
pub enum Pattern<'s, 'a> {
    /// `(` pattern `)`
    ///
    /// Grouping pattern.
    Grouping(TokenRef<'s>, ABox<'a, Pattern<'s, 'a>>, TokenRef<'s>),
    /// ( `+` | `-` )? literal
    ///
    /// Matches against a literal value.
    Literal(Option<TokenRef<'s>>, TokenRef<'s>),
    /// constant
    ///
    /// Matches against a constant value.
    Constant(TokenRef<'s>),
    /// ( `>` | `>=` | `<=` | `<` | `==` | `!=` | `=~` | `!~` ) (pattern_constant | pattern_literal)
    ///
    /// Matches against a relation with constant values.
    Relation(TokenRef<'s>, ABox<'a, Pattern<'s, 'a>>),
    /// (pattern_constant | pattern_literal) ( `..` | `..<` ) (pattern_constant | pattern_literal)
    ///
    /// Matches against a range of constant values.
    Range(ABox<'a, Pattern<'s, 'a>>, TokenRef<'s>, ABox<'a, Pattern<'s, 'a>>),
    /// `_`
    ///
    /// Matches and discards a value.
    Discard(TokenRef<'s>),
    /// `mut`? identifier
    ///
    /// Matches and binds a value to a variable.
    Bind(Option<TokenRef<'s>>, TokenRef<'s>),
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
    Record(TokenRef<'s>, Vec<RecordPattern<'s, 'a>>, TokenRef<'s>),
    /// ```antlr
    /// pattern_array
    ///     : '[' sub_pattern* ']'
    ///     ;
    /// sub_pattern
    ///     : pattern ','?
    ///     | '..' pattern? ','?
    ///     ;
    /// ```
    Array(TokenRef<'s>, Vec<ArrayPattern<'s, 'a>>, TokenRef<'s>),
    /// prefix<`..`>
    ///
    /// Contains no token.
    /// Used in [ArrayPattern]::Spread and [RecordPattern]::Spread
    /// as a placeholder since `_`
    /// must be omitted in these cases.
    ///
    /// The value is the position of the spread discard in the pattern.
    /// Used in the [AstWalker::range] method
    SpreadDiscard(usize),

    /// pattern `and` pattern
    ///
    /// Matches all of the patterns.
    And(ABox<'a, Pattern<'s, 'a>>, TokenRef<'s>, ABox<'a, Pattern<'s, 'a>>),
    /// pattern `or` pattern
    ///
    /// Matches any of the patterns.
    Or(ABox<'a, Pattern<'s, 'a>>, TokenRef<'s>, ABox<'a, Pattern<'s, 'a>>),
    /// `not` pattern
    ///
    /// Matches if the pattern does not match.
    Not(TokenRef<'s>, ABox<'a, Pattern<'s, 'a>>),

    /// Unknown pattern.
    Unknown {
        recovered: Option<ABox<'a, Pattern<'s, 'a>>>,
        tokens: Vec<TokenRef<'s>>,
        errors: Vec<SourceDiagnostic>,
    },
}

impl<'s, 'a> Pattern<'s, 'a> {
    pub(crate) fn wrap_as_unknown<T: Into<Vec<TokenRef<'s>>>>(
        self,
        arena: &'a AstArena,
        tokens: T,
        error: DiagnosticCode,
    ) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Pattern::Unknown {
            recovered: Some(arena.alloc(self)),
            tokens,
            errors: vec![SourceDiagnostic::new(range, error)],
        }
    }

    pub(crate) fn unknown<T: Into<Vec<TokenRef<'s>>>>(tokens: T, error: DiagnosticCode) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Pattern::Unknown {
            recovered: None,
            tokens,
            errors: vec![SourceDiagnostic::new(range, error)],
        }
    }
    pub(crate) fn unknown_range<T: Into<Vec<TokenRef<'s>>>>(
        tokens: T,
        error_range: SourceRange,
        error: DiagnosticCode,
    ) -> Self {
        Pattern::Unknown {
            recovered: None,
            tokens: tokens.into(),
            errors: vec![SourceDiagnostic::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<T: Into<Vec<TokenRef<'s>>>, E: Into<Vec<SourceDiagnostic>>>(
        tokens: T,
        errors: E,
    ) -> Self {
        Pattern::Unknown {
            recovered: None,
            tokens: tokens.into(),
            errors: errors.into(),
        }
    }
}

impl<'s, 'a> AstWalker<'s> for Pattern<'s, 'a> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        use Pattern::*;
        match self {
            Grouping(o, p, e) => {
                o.collect_diagnostics(collector);
                p.collect_diagnostics(collector);
                e.collect_diagnostics(collector);
            }
            Literal(p, t) => {
                p.collect_diagnostics(collector);
                t.collect_diagnostics(collector);
            }
            Constant(t) => {
                t.collect_diagnostics(collector);
            }
            Relation(o, p) => {
                o.collect_diagnostics(collector);
                p.collect_diagnostics(collector);
            }
            Range(s, o, e) => {
                s.collect_diagnostics(collector);
                o.collect_diagnostics(collector);
                e.collect_diagnostics(collector);
            }
            Discard(t) => t.collect_diagnostics(collector),
            Bind(p, t) => {
                p.collect_diagnostics(collector);
                t.collect_diagnostics(collector);
            }
            Record(s, p, e) => {
                s.collect_diagnostics(collector);
                for sub_pattern in p.iter_mut() {
                    sub_pattern.collect_diagnostics(collector);
                }
                e.collect_diagnostics(collector);
            }
            Array(s, p, e) => {
                s.collect_diagnostics(collector);
                for sub_pattern in p.iter_mut() {
                    sub_pattern.collect_diagnostics(collector);
                }
                e.collect_diagnostics(collector);
            }
            SpreadDiscard(_) => {}
            And(l, o, r) => {
                l.collect_diagnostics(collector);
                o.collect_diagnostics(collector);
                r.collect_diagnostics(collector);
            }
            Or(l, o, r) => {
                l.collect_diagnostics(collector);
                o.collect_diagnostics(collector);
                r.collect_diagnostics(collector);
            }
            Not(o, p) => {
                o.collect_diagnostics(collector);
                p.collect_diagnostics(collector);
            }
            Unknown {
                recovered,
                tokens,
                errors,
            } => {
                collector.append(errors);
                tokens.collect_diagnostics(collector);
                if let Some(recovered) = std::mem::take(recovered) {
                    let mut recovered = ABox::into_inner(recovered);
                    recovered.collect_diagnostics(collector);
                    *self = recovered;
                }
            }
        }
    }
    fn range(&self) -> SourceRange {
        use Pattern::*;
        match self {
            Grouping(o, _, e) | Record(o, _, e) | Array(o, _, e) => o.range.start..e.range.end,
            Range(o, _, e) => o.range().start..e.range().end,
            SpreadDiscard(pos) => *pos..*pos,
            Relation(op, pattern) => op.range.start..pattern.range().end,
            Discard(underscore) => underscore.range(),
            Literal(None, token) | Bind(None, token) => token.range(),
            Literal(Some(op), token) | Bind(Some(op), token) => op.range.start..token.range.end,
            Constant(token) => token.range(),
            And(left, _, right) | Or(left, _, right) => left.range().start..right.range().end,
            Not(op, pattern) => op.range.start..pattern.range().end,
            Unknown {
                recovered,
                tokens,
                errors: _,
            } => unknown_range(recovered, tokens),
        }
    }
}
