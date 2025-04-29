use std::borrow::Cow;

use super::Block;

pub(crate) struct Closure<'s> {
    pub name: Option<Cow<'s, str>>,
    pub body: Block<'s>,
}
