use std::str::FromStr;

use super::Keyword;

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "true" => Ok(Keyword::True),
            "false" => Ok(Keyword::False),
            "nil" => Ok(Keyword::Nil),

            "and" => Ok(Keyword::And),
            "or" => Ok(Keyword::Or),
            "not" => Ok(Keyword::Not),

            "if" => Ok(Keyword::If),
            "else" => Ok(Keyword::Else),
            "match" => Ok(Keyword::Match),
            "for" => Ok(Keyword::For),
            "in" => Ok(Keyword::In),
            "while" => Ok(Keyword::While),
            "loop" => Ok(Keyword::Loop),
            "break" => Ok(Keyword::Break),
            "continue" => Ok(Keyword::Continue),
            "return" => Ok(Keyword::Return),

            "fn" => Ok(Keyword::Fn),
            "op" => Ok(Keyword::Op),
            "let" => Ok(Keyword::Let),
            "const" => Ok(Keyword::Const),
            "record" => Ok(Keyword::Record),

            "effect" => Ok(Keyword::Effect),
            "try" => Ok(Keyword::Try),
            "handle" => Ok(Keyword::Handle),
            "perform" => Ok(Keyword::Perform),
            "resume" => Ok(Keyword::Resume),

            _ => Err(()),
        }
    }
}
