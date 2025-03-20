use std::borrow::Cow;

use super::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct TokenError {
    pub range: Range,
    pub error: Cow<'static, str>,
}

impl TokenError {
    pub fn new<E: Into<Cow<'static, str>>>(range: Range, error: E) -> Self {
        TokenError {
            range,
            error: error.into(),
        }
    }
}
