use std::fmt::{self, Display, Formatter};

use super::Expression;

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Value(token) => write!(f, "{}", token),
            Expression::Grouping(exp) => write!(f, "({})", exp),
            Expression::Tuple(exps) => {
                if exps.is_empty() {
                    return write!(f, "[<unnamed>]");
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
            Expression::NamedTuple(exps) => {
                if exps.is_empty() {
                    return write!(f, "[<named>]");
                }
                write!(f, "[")?;
                let mut iter = exps.iter();
                if let Some((name, exp)) = iter.next() {
                    write!(f, "{}: {}", name, exp)?;
                    for (name, exp) in iter {
                        write!(f, ", {}: {}", name, exp)?;
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
            Expression::And(exp1, exp2) => write!(f, "{} and {}", exp1, exp2),
            Expression::Or(exp1, exp2) => write!(f, "{} or {}", exp1, exp2),
        }
    }
}
