use super::prelude::*;

#[derive(Debug, PartialEq, strum::EnumIs)]
pub enum Statement<'s, 'a> {
    /// `';'`
    ///
    /// An empty statement.
    Empty(TokenRef<'s>),
    /// `expression ';'`
    Expression(ABox<'a, Expression<'s, 'a>>, TokenRef<'s>),
    /// `expression_ends_with_block`
    ///
    /// No trailing semicolon in this case. For expressions that end with a semicolon, use [Statement::Expression].
    BlockExpression(ABox<'a, Expression<'s, 'a>>),
    /// `'pub'? 'mod' identifier block_expression_no_expr`
    Module(
        Option<TokenRef<'s>>,
        TokenRef<'s>,
        TokenRef<'s>,
        ABox<'a, Expression<'s, 'a>>,
    ),
    /// `'pub'? 'let' pattern '=' expression ';'`
    Bind(
        Option<TokenRef<'s>>,
        TokenRef<'s>,
        ABox<'a, Pattern<'s, 'a>>,
        TokenRef<'s>,
        ABox<'a, Expression<'s, 'a>>,
        TokenRef<'s>,
    ),
    /// `pattern_rebind '=' expression ';'`
    Rebind(
        ABox<'a, Pattern<'s, 'a>>,
        TokenRef<'s>,
        ABox<'a, Expression<'s, 'a>>,
        TokenRef<'s>,
    ),
    /// `'pub'? 'const' @id '=' expression ';'`
    Const(
        Option<TokenRef<'s>>,
        TokenRef<'s>,
        TokenRef<'s>,
        TokenRef<'s>,
        ABox<'a, Expression<'s, 'a>>,
        TokenRef<'s>,
    ),
    /// `expression ('=' | '+=' | '-=' | '*=' | '/=' | '%=' | '^=' | '&&=' | '||=') expression ';'`
    ///
    /// The assigner must be one of the following:
    /// - `identifier`
    /// - `expression_access` where the accessed is an extern
    /// - `expression_index` where the indexed is an extern
    Assign(
        ABox<'a, Expression<'s, 'a>>,
        TokenRef<'s>,
        ABox<'a, Expression<'s, 'a>>,
        TokenRef<'s>,
    ),
    /// `'pub'? 'fn' identifier (parameters) block_expression`
    ///
    /// Parameters are a list of identifiers, trailing comma is optional.
    ///
    /// If parameters and parentheses are omitted,
    /// the function is considered with an implicit parameter called `it`.
    ///
    /// ```mira
    /// fn filter { it % 2 == 0 }
    /// ```
    ///
    /// The function body is a block expression.
    Function(
        Option<TokenRef<'s>>,
        TokenRef<'s>,
        TokenRef<'s>,
        Option<ParameterList<'s, 'a>>,
        ABox<'a, Expression<'s, 'a>>,
    ),
    /// `return expression;` or `return;`
    ///
    /// If the expression is omitted, the return value is `nil`.
    Return(TokenRef<'s>, Option<ABox<'a, Expression<'s, 'a>>>, TokenRef<'s>),
    /// `break expression;` or `break;`
    ///
    /// The expression is only allowed in a `loop` expression.
    Break(TokenRef<'s>, Option<ABox<'a, Expression<'s, 'a>>>, TokenRef<'s>),
    /// `continue;`
    Continue(TokenRef<'s>, TokenRef<'s>),
    /// Unknown statement.
    Unknown {
        tokens: Vec<TokenRef<'s>>,
        errors: Vec<SourceDiagnostic>,
    },
}

impl<'s, 'a> Statement<'s, 'a> {
    pub(crate) fn unknown<T: Into<Vec<TokenRef<'s>>>>(tokens: T, error: DiagnosticCode) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Statement::Unknown {
            tokens,
            errors: vec![SourceDiagnostic::new(range, error)],
        }
    }

    pub(crate) fn unknown_range<T: Into<Vec<TokenRef<'s>>>>(
        tokens: T,
        error_range: SourceRange,
        error: DiagnosticCode,
    ) -> Self {
        Statement::Unknown {
            tokens: tokens.into(),
            errors: vec![SourceDiagnostic::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<T: Into<Vec<TokenRef<'s>>>, E: Into<Vec<SourceDiagnostic>>>(
        tokens: T,
        errors: E,
    ) -> Self {
        Statement::Unknown {
            tokens: tokens.into(),
            errors: errors.into(),
        }
    }
}

impl<'s, 'a> AstWalker<'s> for Statement<'s, 'a> {
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        use Statement::*;
        match self {
            Empty(c) => c.collect_diagnostics(collector),
            Expression(expr, c) => {
                expr.collect_diagnostics(collector);
                c.collect_diagnostics(collector);
            }
            BlockExpression(expr) => expr.collect_diagnostics(collector),
            Module(kw_pub, kw_mod, id, body) => {
                kw_pub.collect_diagnostics(collector);
                kw_mod.collect_diagnostics(collector);
                id.collect_diagnostics(collector);
                body.collect_diagnostics(collector);
            }
            Bind(kw_pub, kw_let, pattern, eq, expr, c) => {
                kw_pub.collect_diagnostics(collector);
                kw_let.collect_diagnostics(collector);
                pattern.collect_diagnostics(collector);
                eq.collect_diagnostics(collector);
                expr.collect_diagnostics(collector);
                c.collect_diagnostics(collector);
            }
            Rebind(pattern, eq, expr, c) => {
                pattern.collect_diagnostics(collector);
                eq.collect_diagnostics(collector);
                expr.collect_diagnostics(collector);
                c.collect_diagnostics(collector);
            }
            Const(kw_pub, kw_const, id, eq, expr, c) => {
                kw_pub.collect_diagnostics(collector);
                kw_const.collect_diagnostics(collector);
                id.collect_diagnostics(collector);
                eq.collect_diagnostics(collector);
                expr.collect_diagnostics(collector);
                c.collect_diagnostics(collector);
            }
            Assign(exp, eq, expr, c) => {
                exp.collect_diagnostics(collector);
                eq.collect_diagnostics(collector);
                expr.collect_diagnostics(collector);
                c.collect_diagnostics(collector);
            }
            Function(kw_pub, kw_fn, id, params, body) => {
                kw_pub.collect_diagnostics(collector);
                kw_fn.collect_diagnostics(collector);
                id.collect_diagnostics(collector);
                params.collect_diagnostics(collector);
                body.collect_diagnostics(collector);
            }
            Return(kw, expr, c) => {
                kw.collect_diagnostics(collector);
                expr.collect_diagnostics(collector);
                c.collect_diagnostics(collector);
            }
            Break(kw, expr, c) => {
                kw.collect_diagnostics(collector);
                expr.collect_diagnostics(collector);
                c.collect_diagnostics(collector);
            }
            Continue(kw, c) => {
                kw.collect_diagnostics(collector);
                c.collect_diagnostics(collector);
            }
            Unknown { tokens, errors } => {
                collector.append(errors);
                tokens.collect_diagnostics(collector);
            }
        }
    }
    fn range(&self) -> SourceRange {
        use Statement::*;
        match self {
            Empty(c) => c.range(),
            Expression(expr, c) => expr.range().start..c.range.end,
            BlockExpression(expr) => expr.range(),
            Module(kw_pub, kw_mod, _, body) => {
                kw_pub.as_ref().unwrap_or(kw_mod).range.start..body.range().end
            }
            Bind(kw_pub, kw_let, _, _, _, c) => {
                kw_pub.as_ref().unwrap_or(kw_let).range.start..c.range.end
            }
            Rebind(pattern, _, _, c) => pattern.range().start..c.range.end,
            Const(kw_pub, kw_const, _, _, _, c) => {
                kw_pub.as_ref().unwrap_or(kw_const).range.start..c.range.end
            }
            Assign(exp, _, _, c) => exp.range().start..c.range.end,
            Function(kw_pub, kw_fn, _, _, body) => {
                kw_pub.as_ref().unwrap_or(kw_fn).range.start..body.range().end
            }
            Return(kw, _, c) | Break(kw, _, c) | Continue(kw, c) => kw.range.start..c.range.end,
            Unknown { tokens, errors: _ } => tokens.range(),
        }
    }
}
