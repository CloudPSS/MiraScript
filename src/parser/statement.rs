use std::fmt::{self, Display, Formatter};

use crate::lexer::Token;

use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'a> {
    /// `;`
    ///
    /// An empty statement.
    Empty(Box<Token<'a>>),
    /// `expression;`
    Expression(Box<Expression<'a>>, Box<Token<'a>>),
    /// `expression_ends_with_block`
    ///
    /// No trailing semicolon in this case. For expressions that end with a semicolon, use [Statement::Expression].
    BlockExpression(Box<Expression<'a>>),
    /// `('var' | 'val') identifier = expression;`
    Bind(Box<Token<'a>>, Box<Token<'a>>, Box<Expression<'a>>),
    /// `identifier = expression;`
    Rebind(Box<Token<'a>>, Box<Expression<'a>>),
    /// `expression.identifier = expression;`
    ///
    /// expression must evaluate to an external.
    Assign(Box<Expression<'a>>, Box<Token<'a>>, Box<Expression<'a>>),
    /// `fn identifier (parameters) block_expression`
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
    Function(Box<Token<'a>>, Option<Vec<Token<'a>>>, Box<Expression<'a>>),
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
        match self {
            Statement::Empty(s) => writeln!(f, "{}", s),
            Statement::Expression(expr, s) => writeln!(f, "{}{}", expr, s),
            Statement::BlockExpression(expr) => writeln!(f, "{}", expr),
            Statement::Bind(keyword, id, expr) => writeln!(f, "{} {} = {};", keyword, id, expr),
            Statement::Rebind(id, expr) => writeln!(f, "{} = {};", id, expr),
            Statement::Assign(exp, id, expr) => writeln!(f, "{}.{} = {};", exp, id, expr),
            Statement::Function(id, None, body) => writeln!(f, "fn {} {}", id, body),
            Statement::Function(id, Some(params), body) => {
                write!(f, "fn {} (", id)?;
                let mut iter = params.iter();
                if let Some(param) = iter.next() {
                    write!(f, "{}", param)?;
                    for param in iter {
                        write!(f, ", {}", param)?;
                    }
                }
                writeln!(f, ") {}", body)
            }
            Statement::Return(Some(expr)) => writeln!(f, "return {};", expr),
            Statement::Return(None) => writeln!(f, "return;"),
            Statement::Break(Some(expr)) => writeln!(f, "break {};", expr),
            Statement::Break(None) => writeln!(f, "break;"),
            Statement::Continue => writeln!(f, "continue;"),
        }
    }
}
