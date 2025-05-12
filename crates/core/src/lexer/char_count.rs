use winnow::{stream::Location, stream::Stream};

use super::Input;

#[inline]
pub fn count_chars(s: &str) -> usize {
    s.chars().count()
}

#[inline]
pub fn count_from_start(s: &mut Input<'_>) -> usize {
    let end = s.previous_token_end();
    let cp = s.checkpoint();
    s.reset_to_start();
    let count = count_chars(&s[..end]);
    s.reset(&cp);
    count
}

#[inline]
pub fn count_from_cp<'s>(s: &mut Input<'s>, cp: &<Input<'s> as Stream>::Checkpoint) -> usize {
    let end = s.previous_token_end();
    let current_cp = s.checkpoint();
    s.reset(cp);
    let start = s.previous_token_end();
    s.reset_to_start();
    let count = count_chars(&s[start..end]);
    s.reset(&current_cp);
    count
}
