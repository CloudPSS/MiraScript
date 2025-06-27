use super::prelude::*;

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum Statement<'s> {
    /// `';'`
    ///
    /// An empty statement.
    Empty(TokenRef<'s>),
    /// `expression ';'`
    Expression(Box<Expression<'s>>, TokenRef<'s>),
    /// `expression_ends_with_block`
    ///
    /// No trailing semicolon in this case. For expressions that end with a semicolon, use [Statement::Expression].
    BlockExpression(Box<Expression<'s>>),
    /// `'let' pattern '=' expression ';'`
    Bind(
        TokenRef<'s>,
        Box<Pattern<'s>>,
        TokenRef<'s>,
        Box<Expression<'s>>,
        TokenRef<'s>,
    ),
    /// `pattern_rebind '=' expression ';'`
    Rebind(
        Box<Pattern<'s>>,
        TokenRef<'s>,
        Box<Expression<'s>>,
        TokenRef<'s>,
    ),
    /// `expression ('=' | '+=' | '-=' | '*=' | '/=' | '%=' | '^=' | '&&=' | '||=') expression ';'`
    ///
    /// The assigner must be one of the following:
    /// - `identifier`
    /// - `expression_access` where the accessed is an extern
    /// - `expression_index` where the indexed is an extern
    Assign(
        Box<Expression<'s>>,
        TokenRef<'s>,
        Box<Expression<'s>>,
        TokenRef<'s>,
    ),
    /// `'fn' identifier (parameters) block_expression`
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
        TokenRef<'s>,
        TokenRef<'s>,
        Option<ParameterList<'s>>,
        Box<Expression<'s>>,
    ),
    /// `return expression;` or `return;`
    ///
    /// If the expression is omitted, the return value is `nil`.
    Return(TokenRef<'s>, Option<Box<Expression<'s>>>, TokenRef<'s>),
    /// `break expression;` or `break;`
    ///
    /// The expression is only allowed in a `loop` expression.
    Break(TokenRef<'s>, Option<Box<Expression<'s>>>, TokenRef<'s>),
    /// `continue;`
    Continue(TokenRef<'s>, TokenRef<'s>),
    /// Unknown statement.
    Unknown {
        tokens: Vec<TokenRef<'s>>,
        errors: Vec<SourceDiagnostic>,
    },
}

impl<'s> Statement<'s> {
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

impl<'s> AstWalker<'s> for Statement<'s> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        use Statement::*;
        match self {
            Empty(c) => c.collect_diagnostics(collector),
            Expression(expr, c) => {
                expr.collect_diagnostics(collector);
                c.collect_diagnostics(collector);
            }
            BlockExpression(expr) => expr.collect_diagnostics(collector),
            Bind(kw_let, pattern, eq, expr, c) => {
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
            Assign(exp, eq, expr, c) => {
                exp.collect_diagnostics(collector);
                eq.collect_diagnostics(collector);
                expr.collect_diagnostics(collector);
                c.collect_diagnostics(collector);
            }
            Function(kw, id, params, body) => {
                kw.collect_diagnostics(collector);
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
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        use Statement::*;
        visitor.visit_statement(self);
        match self {
            Empty(c) => c.walk(visitor),
            Expression(expr, c) => {
                expr.walk(visitor);
                c.walk(visitor);
            }
            BlockExpression(expr) => expr.walk(visitor),
            Bind(kw_let, pattern, eq, expr, c) => {
                kw_let.walk(visitor);
                pattern.walk(visitor);
                eq.walk(visitor);
                expr.walk(visitor);
                c.walk(visitor);
            }
            Rebind(pattern, eq, expr, c) => {
                pattern.walk(visitor);
                eq.walk(visitor);
                expr.walk(visitor);
                c.walk(visitor);
            }
            Assign(exp, eq, expr, c) => {
                exp.walk(visitor);
                eq.walk(visitor);
                expr.walk(visitor);
                c.walk(visitor);
            }
            Function(kw, id, params, body) => {
                kw.walk(visitor);
                id.walk(visitor);
                params.walk(visitor);
                body.walk(visitor);
            }
            Return(kw, expr, c) => {
                kw.walk(visitor);
                expr.walk(visitor);
                c.walk(visitor);
            }
            Break(kw, expr, c) => {
                kw.walk(visitor);
                expr.walk(visitor);
                c.walk(visitor);
            }
            Continue(kw, c) => {
                kw.walk(visitor);
                c.walk(visitor);
            }
            Unknown { tokens, errors: _ } => {
                tokens.walk(visitor);
            }
        }
    }
}
