use std::fmt::{self, Display, Formatter};

use crate::{
    ansi::{DisplayIdent, GROUP, INTERPOLATED, RECOVER, RESET, STRING},
    error::{ErrorCode, SourceError, SourceRange},
    lexer::{Token, TokenKind},
};

use super::{
    ArrayElement, AstVisitor, Iterable, Pattern, RecordElement, Statement, AstWalker,
};

#[derive(Debug, Clone, PartialEq, strum::EnumIs)]
pub enum Callable<'s> {
    /// `type`
    Type(Token<'s>),
    /// expression
    Expression(Expression<'s>),
}

impl<'s> AstWalker<'s> for Callable<'s> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>) {
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
    Literal(Box<Token<'s>>),
    /// interpolated_string
    ///
    /// Holds a [crate::lexer::TokenKind::InterpolatedString], and a list of expressions
    /// that are interpolated into the string.
    InterpolatedString(Box<Token<'s>>, Vec<Expression<'s>>),
    /// identifier
    Variable(Box<Token<'s>>),
    /// `(` expression `)`
    Grouping(Box<Token<'s>>, Box<Expression<'s>>, Box<Token<'s>>),
    /// `(` element* `)`
    ///
    /// Use `()` for an empty record.
    ///
    /// For a single-element-unnamed record, use `(` expression `,` `)`.
    Record(Box<Token<'s>>, Vec<RecordElement<'s>>, Box<Token<'s>>),
    /// `[` element* `]`
    ///
    /// Use `[]` for an empty list.
    Array(Box<Token<'s>>, Vec<ArrayElement<'s>>, Box<Token<'s>>),

    // postfix
    /// callable `(` arguments `)`
    ///
    /// Arguments are a list of expressions, trailing comma is optional.
    Call(
        Box<Callable<'s>>,
        Box<Token<'s>>,
        Vec<Expression<'s>>,
        Box<Token<'s>>,
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
        Box<Token<'s>>,
        Box<Callable<'s>>,
        Box<Token<'s>>,
        Vec<Expression<'s>>,
        Box<Token<'s>>,
    ),
    /// expression `.` field
    ///
    /// Field must be an identifier or an ordinal.
    Access(Box<Expression<'s>>, Box<Token<'s>>, Box<Token<'s>>),
    /// expression `[` expression `]`
    Index(
        Box<Expression<'s>>,
        Box<Token<'s>>,
        Box<Expression<'s>>,
        Box<Token<'s>>,
    ),
    /// expression `!`
    NonNil(Box<Expression<'s>>, Box<Token<'s>>),

    /// op expression
    ///
    /// Prefix operators are:
    /// - `!` logical not
    /// - `-` negation
    /// - `+` unary plus
    Prefix(Box<Token<'s>>, Box<Expression<'s>>),

    // infix
    /// expression op expression
    ///
    /// Infix operators are:
    /// 1. `^` exponentiation
    /// 1. `*` `/` `%` multiplicative
    /// 1. `+` `-` additive
    /// 1. `is` matching *Use [Expression::Is] for this
    /// 1. `>` `<` `>=` `<=` `in` relational
    /// 1. `==` `!=` `~=` `!~=` equality
    /// 1. `&&` logical and
    /// 1. `||` logical or
    /// 1. `|>` `<|` forward and backward pipe
    Infix(Box<Expression<'s>>, Box<Token<'s>>, Box<Expression<'s>>),
    /// expression `is` pattern
    Is(Box<Expression<'s>>, Box<Token<'s>>, Box<Pattern<'s>>),

    // block-like
    /// `{` statements* expression? `}`
    ///
    /// The value of the block is the value of the last expression.
    /// If no expression is present, the value is `nil`.
    Block(
        Box<Token<'s>>,
        Vec<Statement<'s>>,
        Option<Box<Expression<'s>>>,
        Box<Token<'s>>,
    ),
    /// `loop` block_expression
    ///
    /// The final expression of the block must not present.
    ///
    /// The value of the block is the expression of the `break` statement if present. Otherwise, `nil`.
    Loop(Box<Token<'s>>, Box<Expression<'s>>),
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
        Box<Token<'s>>,
        Box<Expression<'s>>,
        Box<Expression<'s>>,
        Option<(Box<Token<'s>>, Box<Expression<'s>>)>,
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
        Box<Token<'s>>,
        Box<Pattern<'s>>,
        Box<Token<'s>>,
        Box<Iterable<'s>>,
        Box<Expression<'s>>,
        Option<(Box<Token<'s>>, Box<Expression<'s>>)>,
    ),
    /// `if` expression block_expression (`else` expression)?
    ///
    /// The `then_block` is a block expression.
    ///
    /// The `else_block` is a block expression or an if expression.
    If(
        Box<Token<'s>>,
        Box<Expression<'s>>,
        Box<Expression<'s>>,
        Option<(Box<Token<'s>>, Box<Expression<'s>>)>,
    ),
    /// `match` expression `{` `case` ((literal | '_') block_expression)* `}`
    ///
    /// The value of the block is the value of the matched expression.
    ///
    /// If no match is found, the value is `nil`.
    Match(
        Box<Token<'s>>,
        Box<Expression<'s>>,
        Box<Token<'s>>,
        Vec<(Token<'s>, Pattern<'s>, Expression<'s>)>,
        Box<Token<'s>>,
    ),
    /// `fn` parameters? block_expression
    ///
    /// Just like function declarations, but without the identifier.
    /// See [Statement::Function] for more details.
    Function(Box<Token<'s>>, Option<Vec<Token<'s>>>, Box<Expression<'s>>),
    /// Unknown expression
    Unknown {
        expression: Option<Box<Expression<'s>>>,
        tokens: Vec<Token<'s>>,
        errors: Vec<SourceError>,
    },
}

impl<'s> Expression<'s> {
    pub(crate) fn wrap_as_unknown<T: Into<Vec<Token<'s>>>>(
        self,
        tokens: T,
        error: ErrorCode,
    ) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Expression::Unknown {
            expression: Some(Box::new(self)),
            tokens,
            errors: vec![SourceError::new(range, error)],
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
                | Expression::Function(..)
        )
    }

    pub(crate) fn unknown<T: Into<Vec<Token<'s>>>>(tokens: T, error: ErrorCode) -> Self {
        let tokens = tokens.into();
        assert!(!tokens.is_empty());
        let mut range = tokens[0].range.clone();
        range.end = tokens.last().unwrap().range.end;
        Expression::Unknown {
            expression: None,
            tokens,
            errors: vec![SourceError::new(range, error)],
        }
    }
    pub(crate) fn unknown_range<T: Into<Vec<Token<'s>>>>(
        tokens: T,
        error_range: SourceRange,
        error: ErrorCode,
    ) -> Self {
        Expression::Unknown {
            expression: None,
            tokens: tokens.into(),
            errors: vec![SourceError::new(error_range, error)],
        }
    }

    pub(crate) fn unknown_errors<T: Into<Vec<Token<'s>>>, E: Into<Vec<SourceError>>>(
        tokens: T,
        errors: E,
    ) -> Self {
        Expression::Unknown {
            expression: None,
            tokens: tokens.into(),
            errors: errors.into(),
        }
    }
}

impl<'s> AstWalker<'s> for Expression<'s> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>) {
        use Expression::*;
        visitor.visit_expression(self);
        match self {
            Literal(token) => token.walk(visitor),
            InterpolatedString(token, exps) => {
                token.walk(visitor);
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
            Block(op, statements, expression, ed) => {
                op.walk(visitor);
                statements.walk(visitor);
                expression.walk(visitor);
                ed.walk(visitor);
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
            Match(kw, expression, op, arms, ed) => {
                kw.walk(visitor);
                expression.walk(visitor);
                op.walk(visitor);
                for (kw_case, pattern, block) in arms {
                    kw_case.walk(visitor);
                    pattern.walk(visitor);
                    block.walk(visitor);
                }
                ed.walk(visitor);
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
                expression,
                tokens,
                errors: _,
            } => {
                expression.walk(visitor);
                tokens.walk(visitor);
            }
        }
    }
}

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Expression<'_> {
    fn fmt_ident(&self, f: &mut Formatter<'_>, ident: usize) -> fmt::Result {
        use Expression::*;
        match self {
            Literal(token) => write!(f, "{token}")?,
            InterpolatedString(token, e) => {
                let Token {
                    kind: TokenKind::InterpolatedString(s, _),
                    ..
                } = &**token
                else {
                    return write!(f, "{RECOVER}(\"<???>\"){RESET}");
                };
                write!(f, "{STRING}\"")?;
                assert_eq!(s.len(), e.len() + 1, "Invalid string interpolation");
                let mut s_iter = s.iter();
                let first = s_iter.next().ok_or(std::fmt::Error)?;
                write!(f, "{}", first.escape_debug())?;
                for (s, e) in s_iter.zip(e.iter()) {
                    write!(f, "{RESET}{INTERPOLATED}${RESET}")?;
                    e.fmt_ident(f, ident)?;
                    write!(f, "{STRING}")?;
                    write!(f, "{}", s.escape_debug())?;
                }
                write!(f, "\"{RESET}")?;
            }
            Variable(token) => write!(f, "{token}")?,
            Grouping(op, exp, cp) => {
                write!(f, "{GROUP}{op}{RESET}")?;
                exp.fmt_ident(f, ident)?;
                write!(f, "{GROUP}{cp}{RESET}")?;
            }
            Record(op, exps, cp) => {
                if exps.is_empty() {
                    return write!(f, "{op}{cp}");
                }
                write!(f, "{op}")?;
                for exp in exps {
                    exp.fmt_ident(f, ident)?;
                }
                write!(f, "{cp}")?;
            }
            Array(op, exps, cp) => {
                if exps.is_empty() {
                    return write!(f, "{op}{cp}");
                }
                write!(f, "{op}")?;
                for exp in exps {
                    exp.fmt_ident(f, ident)?;
                }
                write!(f, "{cp}")?;
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
            NonNil(exp, op) => {
                exp.fmt_ident(f, ident)?;
                write!(f, "{op}")?;
            }
            Prefix(op, exp) => {
                match op.kind {
                    TokenKind::Operator(_) => write!(f, "{op}")?,
                    _ => write!(f, "{op} ")?,
                }
                exp.fmt_ident(f, ident)?;
            }
            Infix(exp1, op, exp2) => {
                exp1.fmt_ident(f, ident)?;
                write!(f, " {op} ")?;
                exp2.fmt_ident(f, ident)?;
            }
            Is(exp1, op, pattern) => {
                exp1.fmt_ident(f, ident)?;
                write!(f, " {op} ")?;
                pattern.fmt_ident(f, ident)?;
            }
            Block(op, statements, expression, ed) => {
                let next_ident = Self::next_ident(ident);
                if statements.is_empty() {
                    return if let Some(expression) = expression {
                        write!(f, "{GROUP}{op}{RESET} ")?;
                        if expression.is_block_like() {
                            writeln!(f)?;
                            Self::write_ident(f, next_ident, "block ret")?;
                        }
                        expression.fmt_ident(f, next_ident)?;
                        if expression.is_block_like() {
                            writeln!(f)?;
                            Self::write_ident(f, ident, "")?;
                            write!(f, "{GROUP}{ed}{RESET}")
                        } else {
                            write!(f, " {GROUP}{ed}{RESET}")
                        }
                    } else {
                        write!(f, "{GROUP}{op} {ed}{RESET}")
                    };
                }
                writeln!(f, "{GROUP}{op}{RESET}")?;
                for statement in statements {
                    statement.fmt_ident(f, next_ident)?;
                }
                if let Some(expression) = expression {
                    Self::write_ident(f, next_ident, "block ret")?;
                    expression.fmt_ident(f, Self::next_ident(ident))?;
                    writeln!(f)?;
                }
                Self::write_ident(f, ident, "")?;
                write!(f, "{GROUP}{ed}{RESET}")?;
            }
            Loop(kw, expression) => {
                write!(f, "{kw} ")?;
                expression.fmt_ident(f, ident)?;
            }
            While(kw, expression, block, None) => {
                write!(f, "{kw} ")?;
                expression.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
            }
            While(kw, expression, block, Some((kw_else, else_block))) => {
                write!(f, "{kw} ")?;
                expression.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
                write!(f, " {kw_else} ")?;
                else_block.fmt_ident(f, ident)?;
            }
            ForIn(kw_for, pattern, kw_in, iter, block, None) => {
                write!(f, "{kw_for} ")?;
                pattern.fmt_ident(f, ident)?;
                write!(f, " {kw_in} ")?;
                iter.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
            }
            ForIn(kw_for, pattern, kw_in, iter, block, Some((kw_else, else_block))) => {
                write!(f, "{kw_for} ")?;
                pattern.fmt_ident(f, ident)?;
                write!(f, " {kw_in} ")?;
                iter.fmt_ident(f, ident)?;
                write!(f, " ")?;
                block.fmt_ident(f, ident)?;
                write!(f, " {kw_else} ")?;
                else_block.fmt_ident(f, ident)?;
            }
            If(kw_if, cond, then_block, Some((kw_else, else_block))) => {
                write!(f, "{kw_if} ")?;
                cond.fmt_ident(f, ident)?;
                write!(f, " ")?;
                then_block.fmt_ident(f, ident)?;
                write!(f, " {kw_else} ")?;
                else_block.fmt_ident(f, ident)?;
            }
            If(kw_if, cond, then_block, None) => {
                write!(f, "{kw_if} ")?;
                cond.fmt_ident(f, ident)?;
                write!(f, " ")?;
                then_block.fmt_ident(f, ident)?;
            }
            Match(kw, expression, op, arms, ed) => {
                write!(f, "{kw} ")?;
                expression.fmt_ident(f, ident)?;
                writeln!(f, " {GROUP}{op}{RESET}")?;
                let next_ident = Self::next_ident(ident);
                for (kw_case, pattern, block) in arms {
                    Self::write_ident(f, next_ident, "")?;
                    write!(f, "{kw_case} ")?;
                    pattern.fmt_ident(f, next_ident)?;
                    write!(f, " ")?;
                    block.fmt_ident(f, next_ident)?;
                    writeln!(f)?;
                }
                Self::write_ident(f, ident, "")?;
                write!(f, "{GROUP}{ed}{RESET}")?;
            }
            Function(kw, None, block) => {
                write!(f, "{kw} ")?;
                block.fmt_ident(f, ident)?;
            }
            Function(kw, Some(params), block) => {
                write!(f, "{kw} {GROUP}({RESET}")?;
                let mut iter = params.iter();
                if let Some(param) = iter.next() {
                    write!(f, "{}", param)?;
                    for param in iter {
                        write!(f, ", {}", param)?;
                    }
                }
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
