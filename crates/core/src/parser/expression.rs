use crate::parser::helper::unknown_range;

use super::prelude::*;

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum Callable<'s> {
    /// `type`
    Type(TokenRef<'s>),
    /// expression
    Expression(Box<Expression<'s>>),
}

impl<'s> AstWalker<'s> for Callable<'s> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        use Callable::*;
        match self {
            Type(token) => token.collect_diagnostics(collector),
            Expression(exp) => exp.collect_diagnostics(collector),
        }
    }
    fn range(&self) -> SourceRange {
        use Callable::*;
        match self {
            Type(token) => token.range(),
            Expression(exp) => exp.range(),
        }
    }
}

/// `else` (block_expr | if_expr)
#[derive(Debug, Clone, PartialEq)]
pub struct ElseBlock<'s>(pub TokenRef<'s>, pub Box<Expression<'s>>);

impl<'s> AstWalker<'s> for ElseBlock<'s> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        self.0.collect_diagnostics(collector);
        self.1.collect_diagnostics(collector);
    }
    fn range(&self) -> SourceRange {
        self.0.range.start..self.1.range().end
    }
}

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum Expression<'s> {
    // primary
    /// string | number | ordinal | `true` | `false` | `nil`
    Literal(TokenRef<'s>),
    /// interpolated_string
    ///
    /// Holds a [crate::lexer::TokenKind::InterpolatedString], and a list of expressions
    /// that are interpolated into the string.
    InterpolatedString(&'s Token<'s>, Vec<Expression<'s>>),
    /// identifier
    Variable(TokenRef<'s>),
    /// `(` expression `)`
    Grouping(TokenRef<'s>, Box<Expression<'s>>, TokenRef<'s>),
    /// `(` element* `)`
    ///
    /// Use `()` for an empty record.
    ///
    /// For a single-element-unnamed record, use `(` expression `,` `)`.
    Record(TokenRef<'s>, Vec<RecordElement<'s>>, TokenRef<'s>),
    /// `[` element* `]`
    ///
    /// Use `[]` for an empty list.
    Array(TokenRef<'s>, Vec<ArrayElement<'s>>, TokenRef<'s>),

    // postfix
    /// callable `(` arguments `)`
    ///
    /// Arguments are a list of expressions, trailing comma is optional.
    Call(
        Callable<'s>,
        TokenRef<'s>,
        Vec<ArrayElement<'s>>,
        TokenRef<'s>,
    ),
    /// expression `::` extension `(` arguments `)`
    /// extension
    ///     : identifier (`.` ( identifier | ordinal ))*
    ///     | pseudo_function
    ///     | `(` expression `)`
    ///     ;
    ///
    /// Like `Call`, but `expression` is used as the first argument.
    Extension(
        Box<Expression<'s>>,
        TokenRef<'s>,
        Callable<'s>,
        TokenRef<'s>,
        Vec<ArrayElement<'s>>,
        TokenRef<'s>,
    ),
    /// expression `.` field
    ///
    /// Field must be an identifier or an ordinal.
    Access(Box<Expression<'s>>, TokenRef<'s>, TokenRef<'s>),
    /// expression `[` expression `]`
    Index(
        Box<Expression<'s>>,
        TokenRef<'s>,
        Box<Expression<'s>>,
        TokenRef<'s>,
    ),
    /// expression `[` additive_expression? (`..` | `..<`) additive_expression? `]`
    Slice(
        Box<Expression<'s>>,
        TokenRef<'s>,
        Option<Box<Expression<'s>>>,
        TokenRef<'s>,
        Option<Box<Expression<'s>>>,
        TokenRef<'s>,
    ),
    /// expression `!`
    NonNil(Box<Expression<'s>>, TokenRef<'s>),

    /// op expression
    ///
    /// Prefix operators are:
    /// - `!` logical not
    /// - `-` negation
    /// - `+` unary plus
    Prefix(TokenRef<'s>, Box<Expression<'s>>),

    // infix
    /// expression op expression
    ///
    /// Infix operators are:
    /// 1. `^` exponentiation
    /// 1. `*` `/` `%` multiplicative
    /// 1. `+` `-` additive
    /// 1. `is` matching *Use [Expression::Is] for this
    /// 1. `>` `<` `>=` `<=` `in` relational
    /// 1. `==` `!=` `~=` `~!` equality
    /// 1. `&&` logical and
    /// 1. `||` logical or
    Infix(Box<Expression<'s>>, TokenRef<'s>, Box<Expression<'s>>),
    /// expression `is` pattern
    Is(Box<Expression<'s>>, TokenRef<'s>, Box<Pattern<'s>>),

    // block-like
    /// `{` statements* expression? `}`
    ///
    /// The value of the block is the value of the last expression.
    /// If no expression is present, the value is `nil`.
    Block(
        TokenRef<'s>,
        Vec<Statement<'s>>,
        Option<Box<Expression<'s>>>,
        TokenRef<'s>,
    ),
    /// `loop` block_expression
    ///
    /// The final expression of the block must not present.
    ///
    /// The value of the block is the expression of the `break` statement if present. Otherwise, `nil`.
    Loop(TokenRef<'s>, Box<Expression<'s>>),
    /// `while` expression block_expression (`else` expression)?
    ///
    /// The final expression of the block must not present.
    ///
    /// The `else_block` is a block expression or an if expression.
    ///
    /// The value of the block is the expression of the `break` statement if present.
    /// Otherwise, if the `else_block` is present,
    /// the value is the value of the `else_block`. Otherwise, `nil`.
    While(
        TokenRef<'s>,
        Box<Expression<'s>>,
        Box<Expression<'s>>,
        Option<ElseBlock<'s>>,
    ),
    /// `for` pattern `in` expression block_expression (`else` expression)?
    ///
    /// The final expression of the block must not present.
    ///
    /// The `else_block` is a block expression or an if expression.
    ///
    /// The value of the block is the expression of the `break` statement if present.
    /// Otherwise, if the `else_block` is present,
    /// the value is the value of the `else_block`. Otherwise, `nil`.
    ForIn(
        TokenRef<'s>,
        Box<Pattern<'s>>,
        TokenRef<'s>,
        Box<Iterable<'s>>,
        Box<Expression<'s>>,
        Option<ElseBlock<'s>>,
    ),
    /// `if` expression block_expression (`else` expression)?
    ///
    /// The `then_block` is a block expression.
    ///
    /// The `else_block` is a block expression or an if expression.
    If(
        TokenRef<'s>,
        Box<Expression<'s>>,
        Box<Expression<'s>>,
        Option<ElseBlock<'s>>,
    ),
    /// `match` expression `{` ( `case` pattern (`if` expression)? block_expression)* `}`
    ///
    /// The value of the block is the value of the matched expression.
    ///
    /// If no match is found, the value is `nil`.
    Match(
        TokenRef<'s>,
        Box<Expression<'s>>,
        TokenRef<'s>,
        Vec<(
            TokenRef<'s>,
            Pattern<'s>,
            Option<(TokenRef<'s>, Expression<'s>)>,
            Expression<'s>,
        )>,
        TokenRef<'s>,
    ),
    /// `fn` parameters? block_expression
    ///
    /// Just like function declarations, but without the identifier.
    /// See [Statement::Function] for more details.
    Function(TokenRef<'s>, Option<ParameterList<'s>>, Box<Expression<'s>>),
    /// Unknown expression
    Unknown {
        recovered: Option<Box<Expression<'s>>>,
        tokens: Vec<TokenRef<'s>>,
        errors: Vec<SourceDiagnostic>,
    },
}

impl<'s> Expression<'s> {
    pub(crate) fn wrap_as_unknown<T: Into<Vec<TokenRef<'s>>>>(
        self,
        tokens: T,
        error: DiagnosticCode,
    ) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Expression::Unknown {
            recovered: Some(Box::new(self)),
            tokens,
            errors: vec![SourceDiagnostic::new(range, error)],
        }
    }

    pub(crate) fn is_block_like(&self) -> bool {
        matches!(
            self,
            Expression::Block(..)
                | Expression::Loop(..)
                | Expression::While(..)
                | Expression::ForIn(..)
                | Expression::If(..)
                | Expression::Match(..)
        )
    }

    pub(crate) fn unknown<T: Into<Vec<TokenRef<'s>>>>(tokens: T, error: DiagnosticCode) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Expression::Unknown {
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
        Expression::Unknown {
            recovered: None,
            tokens: tokens.into(),
            errors: vec![SourceDiagnostic::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<T: Into<Vec<TokenRef<'s>>>, E: Into<Vec<SourceDiagnostic>>>(
        tokens: T,
        errors: E,
    ) -> Self {
        Expression::Unknown {
            recovered: None,
            tokens: tokens.into(),
            errors: errors.into(),
        }
    }
}

impl<'s> AstWalker<'s> for Expression<'s> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        use Expression::*;
        match self {
            Literal(token) => token.collect_diagnostics(collector),
            InterpolatedString(_, exps) => {
                // skip token ref
                exps.collect_diagnostics(collector);
            }
            Variable(token) => token.collect_diagnostics(collector),
            Grouping(op, exp, cp) => {
                op.collect_diagnostics(collector);
                exp.collect_diagnostics(collector);
                cp.collect_diagnostics(collector);
            }
            Record(op, exps, cp) => {
                op.collect_diagnostics(collector);
                exps.collect_diagnostics(collector);
                cp.collect_diagnostics(collector);
            }
            Array(op, exps, cp) => {
                op.collect_diagnostics(collector);
                exps.collect_diagnostics(collector);
                cp.collect_diagnostics(collector);
            }
            Call(exp, op, args, cp) => {
                exp.collect_diagnostics(collector);
                op.collect_diagnostics(collector);
                args.collect_diagnostics(collector);
                cp.collect_diagnostics(collector);
            }
            Extension(exp, e, ext, op, args, cp) => {
                exp.collect_diagnostics(collector);
                e.collect_diagnostics(collector);
                ext.collect_diagnostics(collector);
                op.collect_diagnostics(collector);
                args.collect_diagnostics(collector);
                cp.collect_diagnostics(collector);
            }
            Access(exp, dot, token) => {
                exp.collect_diagnostics(collector);
                dot.collect_diagnostics(collector);
                token.collect_diagnostics(collector);
            }
            Index(exp, l, index, r) => {
                exp.collect_diagnostics(collector);
                l.collect_diagnostics(collector);
                index.collect_diagnostics(collector);
                r.collect_diagnostics(collector);
            }
            Slice(exp, l, start, op, end, r) => {
                exp.collect_diagnostics(collector);
                l.collect_diagnostics(collector);
                start.collect_diagnostics(collector);
                op.collect_diagnostics(collector);
                end.collect_diagnostics(collector);
                r.collect_diagnostics(collector);
            }
            NonNil(exp, op) => {
                exp.collect_diagnostics(collector);
                op.collect_diagnostics(collector);
            }
            Prefix(op, exp) => {
                op.collect_diagnostics(collector);
                exp.collect_diagnostics(collector);
            }
            Infix(exp1, op, exp2) => {
                exp1.collect_diagnostics(collector);
                op.collect_diagnostics(collector);
                exp2.collect_diagnostics(collector);
            }
            Is(exp1, op, pattern) => {
                exp1.collect_diagnostics(collector);
                op.collect_diagnostics(collector);
                pattern.collect_diagnostics(collector);
            }
            Block(op, statements, expression, cp) => {
                op.collect_diagnostics(collector);
                statements.collect_diagnostics(collector);
                expression.collect_diagnostics(collector);
                cp.collect_diagnostics(collector);
            }
            Loop(kw, expression) => {
                kw.collect_diagnostics(collector);
                expression.collect_diagnostics(collector);
            }
            While(kw, expression, block, None) => {
                kw.collect_diagnostics(collector);
                expression.collect_diagnostics(collector);
                block.collect_diagnostics(collector);
            }
            While(kw, expression, block, Some(else_block)) => {
                kw.collect_diagnostics(collector);
                expression.collect_diagnostics(collector);
                block.collect_diagnostics(collector);
                else_block.collect_diagnostics(collector);
            }
            ForIn(kw_for, pattern, kw_in, iter, block, None) => {
                kw_for.collect_diagnostics(collector);
                pattern.collect_diagnostics(collector);
                kw_in.collect_diagnostics(collector);
                iter.collect_diagnostics(collector);
                block.collect_diagnostics(collector);
            }
            ForIn(kw_for, pattern, kw_in, iter, block, Some(else_block)) => {
                kw_for.collect_diagnostics(collector);
                pattern.collect_diagnostics(collector);
                kw_in.collect_diagnostics(collector);
                iter.collect_diagnostics(collector);
                block.collect_diagnostics(collector);
                else_block.collect_diagnostics(collector);
            }
            If(kw_if, cond, then_block, Some(else_block)) => {
                kw_if.collect_diagnostics(collector);
                cond.collect_diagnostics(collector);
                then_block.collect_diagnostics(collector);
                else_block.collect_diagnostics(collector);
            }
            If(kw_if, cond, then_block, None) => {
                kw_if.collect_diagnostics(collector);
                cond.collect_diagnostics(collector);
                then_block.collect_diagnostics(collector);
            }
            Match(kw, expression, op, arms, cp) => {
                kw.collect_diagnostics(collector);
                expression.collect_diagnostics(collector);
                op.collect_diagnostics(collector);
                for (kw_case, pattern, guard, block) in arms {
                    kw_case.collect_diagnostics(collector);
                    pattern.collect_diagnostics(collector);
                    if let Some((kw, expr)) = guard {
                        kw.collect_diagnostics(collector);
                        expr.collect_diagnostics(collector);
                    }
                    block.collect_diagnostics(collector);
                }
                cp.collect_diagnostics(collector);
            }
            Function(kw, None, block) => {
                kw.collect_diagnostics(collector);
                block.collect_diagnostics(collector);
            }
            Function(kw, Some(params), block) => {
                kw.collect_diagnostics(collector);
                params.collect_diagnostics(collector);
                block.collect_diagnostics(collector);
            }
            Unknown {
                recovered,
                tokens,
                errors,
            } => {
                collector.append(errors);
                tokens.collect_diagnostics(collector);
                if let Some(mut recovered) = std::mem::take(recovered) {
                    recovered.collect_diagnostics(collector);
                    *self = *recovered;
                }
            }
        }
    }
    fn range(&self) -> SourceRange {
        use Expression::*;
        match self {
            Grouping(op, _, cp) | Record(op, _, cp) | Array(op, _, cp) | Block(op, _, _, cp) => {
                op.range.start..cp.range.end
            }
            InterpolatedString(token, _) => token.range.clone(),
            Literal(token) | Variable(token) => token.range.clone(),
            Index(exp, _, _, r) | Slice(exp, _, _, _, _, r) => exp.range().start..r.range.end,
            Call(callable, _, _, cp) => callable.range().start..cp.range.end,
            Extension(expression, _, _, _, _, cp) => expression.range().start..cp.range.end,
            Access(expression, _, cp) => expression.range().start..cp.range.end,
            NonNil(expression, bang) => expression.range().start..bang.range.end,
            Prefix(op, expression) => op.range.start..expression.range().end,
            Infix(left, _, right) => left.range().start..right.range().end,
            Is(expression, _, pattern) => expression.range().start..pattern.range().end,
            Loop(kw, expression) => kw.range.start..expression.range().end,
            While(kw, _, body, None) | ForIn(kw, _, _, _, body, None) | If(kw, _, body, None) => {
                kw.range.start..body.range().end
            }
            While(kw, _, _, Some(else_block))
            | ForIn(kw, _, _, _, _, Some(else_block))
            | If(kw, _, _, Some(else_block)) => kw.range.start..else_block.range().end,
            Match(kw, _, _, _, cp) => kw.range.start..cp.range.end,
            Function(kw, _, expression) => kw.range.start..expression.range().end,
            Unknown {
                recovered,
                tokens,
                errors: _,
            } => unknown_range(recovered, tokens),
        }
    }
}
