use std::fmt::{self, Display, Formatter};

use super::{Expression, Script, Statement};

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Literal(token) => write!(f, "{}", token),
            Expression::InterpolatedString(token) => write!(f, "{}", token),
            Expression::Variable(token) => write!(f, "{}", token),
            Expression::Grouping(exp) => write!(f, "({})", exp),
            Expression::Tuple(exps) => {
                if exps.is_empty() {
                    return write!(f, "()");
                }
                if exps.len() == 1 {
                    return write!(f, "({},)", exps[0]);
                }
                write!(f, "(")?;
                let mut iter = exps.iter();
                if let Some(exp) = iter.next() {
                    write!(f, "{}", exp)?;
                    for exp in iter {
                        write!(f, ", {}", exp)?;
                    }
                } else {
                    write!(f, ",")?;
                }
                write!(f, ")")?;
                Ok(())
            }
            Expression::NamedTuple(exps) => {
                if exps.is_empty() {
                    return write!(f, "()");
                }
                write!(f, "(")?;
                let mut iter = exps.iter();
                if let Some((name, exp)) = iter.next() {
                    write!(f, "{}: {}", name, exp)?;
                    for (name, exp) in iter {
                        write!(f, ", {}: {}", name, exp)?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
            Expression::Array(exps) => {
                if exps.is_empty() {
                    return write!(f, "[]");
                }
                write!(f, "[")?;
                let mut iter = exps.iter();
                if let Some(exp) = iter.next() {
                    write!(f, "{}", exp)?;
                    for exp in iter {
                        write!(f, ", {}", exp)?;
                    }
                }
                write!(f, "]")?;
                Ok(())
            }
            Expression::Call(exp, args) => {
                write!(f, "{}(", exp)?;
                let mut iter = args.iter();
                if let Some(arg) = iter.next() {
                    write!(f, "{}", arg)?;
                    for arg in iter {
                        write!(f, ", {}", arg)?;
                    }
                }
                write!(f, ")")
            }
            Expression::Access(exp, token) => write!(f, "{}.{}", exp, token),
            Expression::Not(exp) => write!(f, "not {}", exp),
            Expression::Negate(exp) => write!(f, "-{}", exp),
            Expression::Plus(exp) => write!(f, "+{}", exp),
            Expression::Exponent(exp1, exp2) => write!(f, "{} ^ {}", exp1, exp2),
            Expression::Multiply(exp1, exp2) => write!(f, "{} * {}", exp1, exp2),
            Expression::Divide(exp1, exp2) => write!(f, "{} / {}", exp1, exp2),
            Expression::Modulo(exp1, exp2) => write!(f, "{} % {}", exp1, exp2),
            Expression::Add(exp1, exp2) => write!(f, "{} + {}", exp1, exp2),
            Expression::Subtract(exp1, exp2) => write!(f, "{} - {}", exp1, exp2),
            Expression::Equal(exp1, exp2) => write!(f, "{} == {}", exp1, exp2),
            Expression::NotEqual(exp1, exp2) => write!(f, "{} != {}", exp1, exp2),
            Expression::Less(exp1, exp2) => write!(f, "{} < {}", exp1, exp2),
            Expression::LessEqual(exp1, exp2) => write!(f, "{} <= {}", exp1, exp2),
            Expression::Greater(exp1, exp2) => write!(f, "{} > {}", exp1, exp2),
            Expression::GreaterEqual(exp1, exp2) => write!(f, "{} >= {}", exp1, exp2),
            Expression::And(exp1, exp2) => write!(f, "{} and {}", exp1, exp2),
            Expression::Or(exp1, exp2) => write!(f, "{} or {}", exp1, exp2),
            Expression::Block(statements, expression) => {
                writeln!(f, "{{")?;
                for statement in statements {
                    write!(f, "{}", statement)?;
                }
                if let Some(expression) = expression {
                    writeln!(f, "{}", expression)?;
                }
                write!(f, "}}")
            }
            Expression::Loop(expression) => write!(f, "loop {}", expression),
            Expression::While(expression, block) => {
                write!(f, "while {} {}", expression, block)
            }
            Expression::ForIn(token, expression, block) => {
                write!(f, "for {} in {} {}", token, expression, block)
            }
            Expression::If(expression, then_block, else_block) => {
                if let Some(else_block) = else_block {
                    write!(f, "if {} {} else {}", expression, then_block, else_block)
                } else {
                    write!(f, "if {} {}", expression, then_block)
                }
            }
            Expression::Match(expression, arms) => {
                writeln!(f, "match {} {{", expression)?;
                for (pattern, block) in arms {
                    writeln!(f, "{} {}", pattern, block)?;
                }
                write!(f, "}}")
            }
            Expression::Function(None, block) => write!(f, "fn {}", block),
            Expression::Function(Some(params), block) => {
                write!(f, "fn (")?;
                let mut iter = params.iter();
                if let Some(param) = iter.next() {
                    write!(f, "{}", param)?;
                    for param in iter {
                        write!(f, ", {}", param)?;
                    }
                }
                write!(f, ") {}", block)
            }
        }
    }
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Empty => writeln!(f, ";"),
            Statement::Expression(expr) => writeln!(f, "{};", expr),
            Statement::BlockExpression(expr) => writeln!(f, "{}", expr),
            Statement::Bind(keyword, id, expr) => writeln!(f, "{} {} = {};", keyword, id, expr),
            Statement::Rebind(id, expr) => writeln!(f, "{} = {};", id, expr),
            Statement::Assign(exp, id, expr) => writeln!(f, "{}.{} = {};", exp, id, expr),
            Statement::Function(id, None, body) => write!(f, "fn {} {}", id, body),
            Statement::Function(id, Some(params), body) => {
                write!(f, "fn {} (", id)?;
                let mut iter = params.iter();
                if let Some(param) = iter.next() {
                    write!(f, "{}", param)?;
                    for param in iter {
                        write!(f, ", {}", param)?;
                    }
                }
                write!(f, ") {}", body)
            }
            Statement::Return(Some(expr)) => writeln!(f, "return {};", expr),
            Statement::Return(None) => writeln!(f, "return;"),
            Statement::Break(Some(expr)) => writeln!(f, "break {};", expr),
            Statement::Break(None) => writeln!(f, "break;"),
            Statement::Continue => writeln!(f, "continue;"),
        }
    }
}

impl Display for Script<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for statement in self.iter() {
            write!(f, "{}", statement)?;
        }
        Ok(())
    }
}
