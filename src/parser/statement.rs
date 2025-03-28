use std::fmt::{self, Display, Formatter};

use crate::lexer::Token;

use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'a> {
    /// `';'`
    ///
    /// An empty statement.
    Empty(Box<Token<'a>>),
    /// `expression ';'`
    Expression(Box<Expression<'a>>, Box<Token<'a>>),
    /// `expression_ends_with_block`
    ///
    /// No trailing semicolon in this case. For expressions that end with a semicolon, use [Statement::Expression].
    BlockExpression(Box<Expression<'a>>),
    /// `('var' | 'val') identifier '=' expression ';'`
    Bind(
        Box<Token<'a>>,
        Box<Token<'a>>,
        Box<Token<'a>>,
        Box<Expression<'a>>,
        Box<Token<'a>>,
    ),
    /// `(expression_variable | expression_access | expression_index) '=' expression ';'`
    ///
    /// expression must evaluate to an external.
    Assign(
        Box<Expression<'a>>,
        Box<Token<'a>>,
        Box<Expression<'a>>,
        Box<Token<'a>>,
    ),
    /// `'fn' identifier (parameters) block_expression`
    ///
    /// Parameters are a list of identifiers, trailing comma is optional.
    ///
    /// If parameters and parentheses are omitted,
    /// the function is considered with an implicit parameter called `it`.
    ///
    /// ```
    /// fn filter { it % 2 == 0 }
    /// ```
    ///
    /// The function body is a block expression.
    Function(
        Box<Token<'a>>,
        Box<Token<'a>>,
        Option<Vec<Token<'a>>>,
        Box<Expression<'a>>,
    ),
    /// `return expression;` or `return;`
    ///
    /// If the expression is omitted, the return value is `nil`.
    Return(Option<Box<Expression<'a>>>),
    /// `break expression;` or `break;`
    ///
    /// The expression is only allowed in a `loop` expression.
    Break(Option<Box<Expression<'a>>>),
    /// `continue;`
    Continue,
}
impl Display for Statement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Statement::*;
        match self {
            Empty(c) => writeln!(f, "{c}"),
            Expression(expr, c) => writeln!(f, "{expr}{c}"),
            BlockExpression(expr) => writeln!(f, "{}", expr),
            Bind(keyword, id, eq, expr, c) => {
                writeln!(f, "{keyword} {id} {eq} {expr}{c}")
            }
            Assign(exp, eq, expr, c) => {
                writeln!(f, "{exp} {eq} {expr}{c}")
            }
            Function(kw, id, None, body) => writeln!(f, "{kw} {id} {body}"),
            Function(kw, id, Some(params), body) => {
                write!(f, "{kw} {id} (")?;
                let mut iter = params.iter();
                if let Some(param) = iter.next() {
                    write!(f, "{param}")?;
                    for param in iter {
                        write!(f, ", {param}")?;
                    }
                }
                writeln!(f, ") {body}")
            }
            Return(Some(expr)) => writeln!(f, "return {};", expr),
            Return(None) => writeln!(f, "return;"),
            Break(Some(expr)) => writeln!(f, "break {};", expr),
            Break(None) => writeln!(f, "break;"),
            Continue => writeln!(f, "continue;"),
        }
    }
}
