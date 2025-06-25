use std::fmt::{self, Display, Formatter};

use crate::ansi::{DisplayIdent, GROUP, INTERPOLATED, RECOVER, RESET, STRING};

use super::{ArrayElement, AstVisitor, AstWalker, RecordElement, prelude::*};

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
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        use Callable::*;
        match self {
            Type(token) => token.walk(visitor),
            Expression(exp) => exp.walk(visitor),
        }
    }
}

impl Display for Callable<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Callable<'_> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        use Callable::*;
        match self {
            Type(token) => token.fmt_ident(f, ident)?,
            Expression(exp) => exp.fmt_ident(f, ident)?,
        }
        Ok(())
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
        Option<(TokenRef<'s>, Box<Expression<'s>>)>,
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
        Option<(TokenRef<'s>, Box<Expression<'s>>)>,
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
        Option<(TokenRef<'s>, Box<Expression<'s>>)>,
    ),
    /// `match` expression `{` `case` ((literal | '_') block_expression)* `}`
    ///
    /// The value of the block is the value of the matched expression.
    ///
    /// If no match is found, the value is `nil`.
    Match(
        TokenRef<'s>,
        Box<Expression<'s>>,
        TokenRef<'s>,
        Vec<(TokenRef<'s>, Pattern<'s>, Expression<'s>)>,
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
            While(kw, expression, block, Some((kw_else, else_block))) => {
                kw.collect_diagnostics(collector);
                expression.collect_diagnostics(collector);
                block.collect_diagnostics(collector);
                kw_else.collect_diagnostics(collector);
                else_block.collect_diagnostics(collector);
            }
            ForIn(kw_for, pattern, kw_in, iter, block, None) => {
                kw_for.collect_diagnostics(collector);
                pattern.collect_diagnostics(collector);
                kw_in.collect_diagnostics(collector);
                iter.collect_diagnostics(collector);
                block.collect_diagnostics(collector);
            }
            ForIn(kw_for, pattern, kw_in, iter, block, Some((kw_else, else_block))) => {
                kw_for.collect_diagnostics(collector);
                pattern.collect_diagnostics(collector);
                kw_in.collect_diagnostics(collector);
                iter.collect_diagnostics(collector);
                block.collect_diagnostics(collector);
                kw_else.collect_diagnostics(collector);
                else_block.collect_diagnostics(collector);
            }
            If(kw_if, cond, then_block, Some((kw_else, else_block))) => {
                kw_if.collect_diagnostics(collector);
                cond.collect_diagnostics(collector);
                then_block.collect_diagnostics(collector);
                kw_else.collect_diagnostics(collector);
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
                for (kw_case, pattern, block) in arms {
                    kw_case.collect_diagnostics(collector);
                    pattern.collect_diagnostics(collector);
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

    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        use Expression::*;
        visitor.visit_expression(self);
        match self {
            Literal(token) => token.walk(visitor),
            InterpolatedString(_, exps) => {
                // skip token ref
                exps.walk(visitor);
            }
            Variable(token) => token.walk(visitor),
            Grouping(op, exp, cp) => {
                op.walk(visitor);
                exp.walk(visitor);
                cp.walk(visitor);
            }
            Record(op, exps, cp) => {
                op.walk(visitor);
                exps.walk(visitor);
                cp.walk(visitor);
            }
            Array(op, exps, cp) => {
                op.walk(visitor);
                exps.walk(visitor);
                cp.walk(visitor);
            }
            Call(exp, op, args, cp) => {
                exp.walk(visitor);
                op.walk(visitor);
                args.walk(visitor);
                cp.walk(visitor);
            }
            Extension(exp, e, ext, op, args, cp) => {
                exp.walk(visitor);
                e.walk(visitor);
                ext.walk(visitor);
                op.walk(visitor);
                args.walk(visitor);
                cp.walk(visitor);
            }
            Access(exp, dot, token) => {
                exp.walk(visitor);
                dot.walk(visitor);
                token.walk(visitor);
            }
            Index(exp, l, index, r) => {
                exp.walk(visitor);
                l.walk(visitor);
                index.walk(visitor);
                r.walk(visitor);
            }
            Slice(exp, l, start, op, end, r) => {
                exp.walk(visitor);
                l.walk(visitor);
                start.walk(visitor);
                op.walk(visitor);
                end.walk(visitor);
                r.walk(visitor);
            }
            NonNil(exp, op) => {
                exp.walk(visitor);
                op.walk(visitor);
            }
            Prefix(op, exp) => {
                op.walk(visitor);
                exp.walk(visitor);
            }
            Infix(exp1, op, exp2) => {
                exp1.walk(visitor);
                op.walk(visitor);
                exp2.walk(visitor);
            }
            Is(exp1, op, pattern) => {
                exp1.walk(visitor);
                op.walk(visitor);
                pattern.walk(visitor);
            }
            Block(op, statements, expression, cp) => {
                op.walk(visitor);
                statements.walk(visitor);
                expression.walk(visitor);
                cp.walk(visitor);
            }
            Loop(kw, expression) => {
                kw.walk(visitor);
                expression.walk(visitor);
            }
            While(kw, expression, block, None) => {
                kw.walk(visitor);
                expression.walk(visitor);
                block.walk(visitor);
            }
            While(kw, expression, block, Some((kw_else, else_block))) => {
                kw.walk(visitor);
                expression.walk(visitor);
                block.walk(visitor);
                kw_else.walk(visitor);
                else_block.walk(visitor);
            }
            ForIn(kw_for, pattern, kw_in, iter, block, None) => {
                kw_for.walk(visitor);
                pattern.walk(visitor);
                kw_in.walk(visitor);
                iter.walk(visitor);
                block.walk(visitor);
            }
            ForIn(kw_for, pattern, kw_in, iter, block, Some((kw_else, else_block))) => {
                kw_for.walk(visitor);
                pattern.walk(visitor);
                kw_in.walk(visitor);
                iter.walk(visitor);
                block.walk(visitor);
                kw_else.walk(visitor);
                else_block.walk(visitor);
            }
            If(kw_if, cond, then_block, Some((kw_else, else_block))) => {
                kw_if.walk(visitor);
                cond.walk(visitor);
                then_block.walk(visitor);
                kw_else.walk(visitor);
                else_block.walk(visitor);
            }
            If(kw_if, cond, then_block, None) => {
                kw_if.walk(visitor);
                cond.walk(visitor);
                then_block.walk(visitor);
            }
            Match(kw, expression, op, arms, cp) => {
                kw.walk(visitor);
                expression.walk(visitor);
                op.walk(visitor);
                for (kw_case, pattern, block) in arms {
                    kw_case.walk(visitor);
                    pattern.walk(visitor);
                    block.walk(visitor);
                }
                cp.walk(visitor);
            }
            Function(kw, None, block) => {
                kw.walk(visitor);
                block.walk(visitor);
            }
            Function(kw, Some(params), block) => {
                kw.walk(visitor);
                params.walk(visitor);
                block.walk(visitor);
            }
            Unknown {
                recovered,
                tokens,
                errors: _,
            } => {
                recovered.walk(visitor);
                tokens.walk(visitor);
            }
        }
    }

    fn range(&self) -> SourceRange {
        use Expression::*;
        match self {
            Grouping(op, _, cp) | Record(op, _, cp) | Array(op, _, cp) | Block(op, _, _, cp) => {
                op.range.start..cp.range.end
            }
            Index(exp, _, _, r) | Slice(exp, _, _, _, _, r) => exp.range().start..r.range.end,
            _ => self.range_slow(),
        }
    }
}

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_ident(f, 0)
    }
}

fn write_grouping(f: &mut Formatter<'_>, ident: usize, token: &Token<'_>) -> fmt::Result {
    write!(f, "{GROUP}")?;
    token.fmt_ident(f, ident)?;
    write!(f, "{RESET}")?;
    Ok(())
}

impl DisplayIdent for Expression<'_> {
    fn fmt_ident(&self, f: &mut Formatter<'_>, ident: usize) -> fmt::Result {
        use Expression::*;
        match self {
            Literal(token) => token.fmt_ident(f, ident)?,
            InterpolatedString(token, e) => {
                let Token {
                    kind: TokenKind::InterpolatedString(s, _),
                    ..
                } = *token
                else {
                    return write!(f, "{RECOVER}(\"<???>\"){RESET}");
                };
                write!(f, "{STRING}\"")?;
                assert_eq!(s.len(), e.len() + 1, "Invalid string interpolation");
                let mut s_iter = s.iter();
                let first = s_iter.next().ok_or(std::fmt::Error)?;
                write!(f, "{}", first.0.escape_debug())?;
                for (s, e) in s_iter.zip(e.iter()) {
                    write!(f, "{RESET}{INTERPOLATED}${RESET}")?;
                    e.fmt_ident(f, ident)?;
                    write!(f, "{STRING}")?;
                    write!(f, "{}", s.0.escape_debug())?;
                }
                write!(f, "\"{RESET}")?;
            }
            Variable(token) => token.fmt_ident(f, ident)?,
            Grouping(op, exp, cp) => {
                write_grouping(f, ident, op)?;
                exp.fmt_ident(f, ident)?;
                write_grouping(f, ident, cp)?;
            }
            Record(op, exps, cp) => {
                op.fmt_ident(f, ident)?;
                for exp in exps {
                    exp.fmt_ident(f, ident)?;
                }
                cp.fmt_ident(f, ident)?;
            }
            Array(op, exps, cp) => {
                op.fmt_ident(f, ident)?;
                for exp in exps {
                    exp.fmt_ident(f, ident)?;
                }
                cp.fmt_ident(f, ident)?;
            }
            Call(exp, op, args, cp) => {
                exp.fmt_ident(f, ident)?;
                op.fmt_ident(f, ident)?;
                let mut iter = args.iter();
                if let Some(arg) = iter.next() {
                    arg.fmt_ident(f, ident)?;
                    for arg in iter {
                        write!(f, ", ")?;
                        arg.fmt_ident(f, ident)?;
                    }
                }
                cp.fmt_ident(f, ident)?;
            }
            Extension(exp, e, ext, op, args, cp) => {
                exp.fmt_ident(f, ident)?;
                e.fmt_ident(f, ident)?;
                ext.fmt_ident(f, ident)?;
                op.fmt_ident(f, ident)?;
                let mut iter = args.iter();
                if let Some(arg) = iter.next() {
                    arg.fmt_ident(f, ident)?;
                    for arg in iter {
                        write!(f, ", ")?;
                        arg.fmt_ident(f, ident)?;
                    }
                }
                cp.fmt_ident(f, ident)?;
            }
            Access(exp, dot, token) => {
                exp.fmt_ident(f, ident)?;
                dot.fmt_ident(f, ident)?;
                token.fmt_ident(f, ident)?;
            }
            Index(exp, l, index, r) => {
                exp.fmt_ident(f, ident)?;
                l.fmt_ident(f, ident)?;
                index.fmt_ident(f, ident)?;
                r.fmt_ident(f, ident)?;
            }
            Slice(exp, l, start, op, end, r) => {
                exp.fmt_ident(f, ident)?;
                l.fmt_ident(f, ident)?;
                start.fmt_ident(f, ident)?;
                op.fmt_ident(f, ident)?;
                end.fmt_ident(f, ident)?;
                r.fmt_ident(f, ident)?;
            }
            NonNil(exp, op) => {
                exp.fmt_ident(f, ident)?;
                op.fmt_ident(f, ident)?;
            }
            Prefix(op, exp) => {
                op.fmt_ident(f, ident)?;
                if !op.is_operator() {
                    write!(f, " ")?;
                }
                exp.fmt_ident(f, ident)?;
            }
            Infix(exp1, op, exp2) => {
                exp1.fmt_ident(f, ident)?;
                write!(f, " ")?;
                op.fmt_ident(f, ident)?;
                write!(f, " ")?;
                exp2.fmt_ident(f, ident)?;
            }
            Is(exp1, op, pattern) => {
                exp1.fmt_ident(f, ident)?;
                write!(f, " ")?;
                op.fmt_ident(f, ident)?;
                write!(f, " ")?;
                pattern.fmt_ident(f, ident)?;
            }
            Block(op, statements, expression, ed) => {
                let next_ident = Self::next_ident(ident);
                write_grouping(f, ident, op)?;
                if statements.is_empty() {
                    if let Some(expression) = expression {
                        write!(f, " ")?;
                        if expression.is_block_like() || expression.is_function() {
                            writeln!(f)?;
                            Self::write_ident(f, next_ident, "block ret")?;
                        }
                        expression.fmt_ident(f, next_ident)?;
                        if expression.is_block_like() || expression.is_function() {
                            writeln!(f)?;
                            Self::write_ident(f, ident, "")?;
                        } else {
                            write!(f, " ")?;
                        }
                    } else {
                        write!(f, " ")?;
                    };
                } else {
                    writeln!(f)?;
                    for statement in statements {
                        statement.fmt_ident(f, next_ident)?;
                    }
                    if let Some(expression) = expression {
                        Self::write_ident(f, next_ident, "block ret")?;
                        expression.fmt_ident(f, Self::next_ident(ident))?;
                        writeln!(f)?;
                    }
                    Self::write_ident(f, ident, "")?;
                }
                write_grouping(f, ident, ed)?;
            }
            Loop(kw, expression) => {
                kw.fmt_ident(f, ident)?;
                write!(f, " ")?;
                expression.fmt_ident(f, ident)?;
            }
            While(kw, expression, block, None) => {
                kw.fmt_ident(f, ident)?;
                write!(f, " ")?;
                expression.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
            }
            While(kw, expression, block, Some((kw_else, else_block))) => {
                kw.fmt_ident(f, ident)?;
                write!(f, " ")?;
                expression.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
                write!(f, " ")?;
                kw_else.fmt_ident(f, ident)?;
                write!(f, " ")?;
                else_block.fmt_ident(f, ident)?;
            }
            ForIn(kw_for, pattern, kw_in, iter, block, None) => {
                kw_for.fmt_ident(f, ident)?;
                write!(f, " ")?;
                pattern.fmt_ident(f, ident)?;
                write!(f, " ")?;
                kw_in.fmt_ident(f, ident)?;
                write!(f, " ")?;
                iter.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
            }
            ForIn(kw_for, pattern, kw_in, iter, block, Some((kw_else, else_block))) => {
                kw_for.fmt_ident(f, ident)?;
                write!(f, " ")?;
                pattern.fmt_ident(f, ident)?;
                write!(f, " ")?;
                kw_in.fmt_ident(f, ident)?;
                write!(f, " ")?;
                iter.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
                write!(f, " ")?;
                kw_else.fmt_ident(f, ident)?;
                write!(f, " ")?;
                else_block.fmt_ident(f, ident)?;
            }
            If(kw_if, cond, then_block, Some((kw_else, else_block))) => {
                kw_if.fmt_ident(f, ident)?;
                write!(f, " ")?;
                cond.fmt_ident(f, ident)?;
                write!(f, " ")?;
                then_block.fmt_ident(f, ident)?;
                write!(f, " ")?;
                kw_else.fmt_ident(f, ident)?;
                write!(f, " ")?;
                else_block.fmt_ident(f, ident)?;
            }
            If(kw_if, cond, then_block, None) => {
                kw_if.fmt_ident(f, ident)?;
                write!(f, " ")?;
                cond.fmt_ident(f, ident)?;
                write!(f, " ")?;
                then_block.fmt_ident(f, ident)?;
            }
            Match(kw, expression, op, arms, ed) => {
                kw.fmt_ident(f, ident)?;
                write!(f, " ")?;
                expression.fmt_ident(f, ident)?;
                write!(f, " ")?;
                write_grouping(f, ident, op)?;
                writeln!(f)?;
                let next_ident = Self::next_ident(ident);
                for (kw_case, pattern, block) in arms {
                    Self::write_ident(f, next_ident, "")?;
                    kw_case.fmt_ident(f, next_ident)?;
                    write!(f, " ")?;
                    pattern.fmt_ident(f, next_ident)?;
                    write!(f, " ")?;
                    block.fmt_ident(f, next_ident)?;
                    writeln!(f)?;
                }
                Self::write_ident(f, ident, "")?;
                write_grouping(f, ident, ed)?;
            }
            Function(kw, None, block) => {
                kw.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
            }
            Function(kw, Some(params), block) => {
                kw.fmt_ident(f, ident)?;
                write!(f, " {GROUP}({RESET}")?;
                params.fmt_ident(f, ident)?;
                write!(f, "{GROUP}){RESET} ")?;
                block.fmt_ident(f, ident)?;
            }
            Unknown { tokens, .. } => {
                write!(f, "{RECOVER}<expression{RESET}")?;
                for token in tokens {
                    write!(f, " {token}")?;
                }
                write!(f, "{RECOVER}>{RESET}")?;
            }
        }
        Ok(())
    }
}
