use std::borrow::Cow;

pub type Range = std::ops::Range<usize>;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceError {
    pub range: Range,
    pub error: Cow<'static, str>,
}

impl SourceError {
    pub fn new<E: Into<Cow<'static, str>>>(range: Range, error: E) -> Self {
        SourceError {
            range,
            error: error.into(),
        }
    }
}
