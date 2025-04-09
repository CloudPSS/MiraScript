use std::borrow::Cow;

use super::Block;

pub(crate) struct Closure<'a> {
    pub name: Option<Cow<'a, str>>,
    pub body: Block<'a>,
}
