use std::{fmt::Debug, ops::Deref};

use winnow::Stateful;
use winnow::combinator::peek;
use winnow::prelude::*;
use winnow::stream::TokenSlice;
use winnow::token::any;

use crate::lexer::{Token, TokenKind};

mod basic_expressions;
mod block_expressions;
mod display;
mod expressions;
mod helper;
mod statements;

pub use expressions::expression;

#[derive(Debug, Clone)]
pub struct State<'a> {
    #[allow(clippy::vec_box)]
    // Boxed to ensure that the reference is stable
    resumed: Vec<Box<Token<'a>>>,
}

impl<'a> State<'a> {
    pub fn new() -> Self {
        Self {
            resumed: Vec::new(),
        }
    }

    pub fn add_token(&mut self, token: Token<'a>) -> TokenRef<'a> {
        let cell = Box::new(token);
        let ptr = &*cell as *const Token<'a>;
        self.resumed.push(cell);
        unsafe { &*ptr }
    }
}

pub type Input<'a> = Stateful<TokenSlice<'a, Token<'a>>, State<'a>>;

type TokenRef<'a> = &'a Token<'a>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'a> {
    // primary
    /// literal | 'true' | 'false' | 'nil'
    Literal(TokenRef<'a>),
    /// interpolated_string
    ///
    /// Holds a ref of [TokenKind::InterpolatedString].
    InterpolatedString(TokenRef<'a>),
    /// identifier
    Variable(TokenRef<'a>),
    /// `(` expression `)`
    Grouping(Box<Expression<'a>>),
    /// `(` expression (`,` expression)* `)`
    ///
    /// Use `()` for an empty tuple.
    ///
    /// For a single-element tuple, use `(expression,)`.
    Tuple(Vec<Expression<'a>>),
    /// `(` name `:` expression (`,` name `:` expression)* `)`
    ///
    /// Name should be an identifier or an ordinal.
    ///
    /// All elements must be named or unnamed.
    NamedTuple(Vec<(TokenRef<'a>, Expression<'a>)>),
    /// `[` expression (`,` expression)* `]`
    ///
    /// Use `[]` for an empty list.
    Array(Vec<Expression<'a>>),

    // function
    /// expression `(` arguments `)`
    ///
    /// Arguments are a list of expressions, trailing comma is optional.
    Call(Box<Expression<'a>>, Vec<Expression<'a>>),
    /// expression `.` field
    ///
    /// Field must be an identifier or an ordinal.
    Access(Box<Expression<'a>>, TokenRef<'a>),

    // unary
    /// `not` expression
    Not(Box<Expression<'a>>),
    /// `-` expression
    Negate(Box<Expression<'a>>),
    /// `+` expression
    Plus(Box<Expression<'a>>),

    // exponent
    /// expression `^` expression
    Exponent(Box<Expression<'a>>, Box<Expression<'a>>),

    // factor
    /// expression `*` expression
    Multiply(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `/` expression
    Divide(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `%` expression
    Modulo(Box<Expression<'a>>, Box<Expression<'a>>),

    // term
    /// expression `+` expression
    Add(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `-` expression
    Subtract(Box<Expression<'a>>, Box<Expression<'a>>),

    // comparison
    /// expression `==` expression
    Equal(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `!=` expression
    NotEqual(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `<` expression
    Less(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `<=` expression
    LessEqual(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `>` expression
    Greater(Box<Expression<'a>>, Box<Expression<'a>>),
    /// expression `>=` expression
    GreaterEqual(Box<Expression<'a>>, Box<Expression<'a>>),

    // and
    /// expression `and` expression
    And(Box<Expression<'a>>, Box<Expression<'a>>),

    // or
    /// expression `or` expression
    Or(Box<Expression<'a>>, Box<Expression<'a>>),

    // block-like
    /// `{` statements* expression? `}`
    ///
    /// The value of the block is the value of the last expression.
    /// If no expression is present, the value is `nil`.
    Block(Vec<Statement<'a>>, Option<Box<Expression<'a>>>),
    /// `loop` block_expression
    ///
    /// The final expression of the block must not present.
    ///
    /// The value of the block is the expression of the `break` statement if present. Otherwise, `nil`.
    Loop(Box<Expression<'a>>),
    /// `while` expression block_expression
    ///
    /// The final expression of the block must not present.
    ///
    /// The value of the block is `nil`.
    While(Box<Expression<'a>>, Box<Expression<'a>>),
    /// `for` identifier `in` expression block_expression
    ///
    /// The final expression of the block must not present.
    ///
    /// The value of the block is `nil`.
    ForIn(TokenRef<'a>, Box<Expression<'a>>, Box<Expression<'a>>),
    /// `if` expression block_expression (`else` expression)?
    ///
    /// The `then_block` is a block expression.
    ///
    /// The `else_block` is a block expression or an if expression.
    If(
        Box<Expression<'a>>,
        Box<Expression<'a>>,
        Option<Box<Expression<'a>>>,
    ),
    /// `match` expression `{` ((literal | '_') block_expression)* `}`
    ///
    /// The value of the block is the value of the matched expression.
    ///
    /// If no match is found, the value is `nil`.
    Match(
        Box<Expression<'a>>,
        Vec<(Box<Expression<'a>>, Box<Expression<'a>>)>,
    ),
    /// `fn` (parameters) block_expression
    ///
    /// Just like function declarations, but without the identifier.
    /// See [Statement::Function] for more details.
    Function(Option<Vec<TokenRef<'a>>>, Box<Expression<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'a> {
    /// `;`
    ///
    /// An empty statement.
    Empty,
    /// `expression;`
    Expression(Expression<'a>),
    /// `expression_ends_with_block`
    ///
    /// No trailing semicolon in this case. For expressions that end with a semicolon, use [Statement::Expression].
    BlockExpression(Expression<'a>),
    /// `('var' | 'val') identifier = expression;`
    Bind(TokenRef<'a>, TokenRef<'a>, Expression<'a>),
    /// `identifier = expression;`
    Rebind(TokenRef<'a>, Expression<'a>),
    /// `expression.identifier = expression;`
    ///
    /// expression must evaluate to an external.
    Assign(Expression<'a>, TokenRef<'a>, Expression<'a>),
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
    Function(TokenRef<'a>, Option<Vec<TokenRef<'a>>>, Expression<'a>),
    /// `return expression;` or `return;`
    ///
    /// If the expression is omitted, the return value is `nil`.
    Return(Option<Expression<'a>>),
    /// `break expression;` or `break;`
    ///
    /// The expression is only allowed in a `loop` expression.
    Break(Option<Expression<'a>>),
    /// `continue;`
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Script<'a>(Vec<Statement<'a>>);

impl<'a> Deref for Script<'a> {
    type Target = Vec<Statement<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn to_input<'a>(tokens: &'a [Token<'a>]) -> Input<'a> {
    Stateful {
        input: TokenSlice::new(tokens),
        state: State::new(),
    }
}

pub fn parse<'a>(i: &mut Input<'a>) -> ModalResult<Script<'a>> {
    let mut statements = vec![];
    loop {
        let token = peek(any).parse_next(i)?;
        if *token == TokenKind::Eof {
            any.parse_next(i)?;
            break;
        }
        let statement = statements::statement.parse_next(i)?;
        statements.push(statement);
    }
    Ok(Script(statements))
}
