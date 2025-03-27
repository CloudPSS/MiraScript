use std::borrow::Cow;

pub type SourceRange = std::ops::Range<usize>;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceError {
    pub range: SourceRange,
    pub error: Cow<'static, str>,
}

impl SourceError {
    pub fn new<E: Into<Cow<'static, str>>>(range: SourceRange, error: E) -> Self {
        SourceError {
            range,
            error: error.into(),
        }
    }
}
