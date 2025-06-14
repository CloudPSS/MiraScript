use std::cell::Cell;

#[cfg(feature = "track_references")]
thread_local! {
  static TRACK_REFERENCES: Cell<bool> = const { Cell::new(false) };
}

#[cfg(feature = "track_references")]
pub(crate) fn track_references() -> bool {
    TRACK_REFERENCES.get()
}

#[cfg(feature = "trivia")]
thread_local! {
  static TRIVIA: Cell<bool> = const { Cell::new(false) };
}

#[cfg(feature = "trivia")]
pub(crate) fn trivia() -> bool {
    TRIVIA.get()
}

pub(crate) fn set_config(value: &Config) {
    #[cfg(feature = "track_references")]
    TRACK_REFERENCES.set(value.track_references);
    #[cfg(feature = "trivia")]
    TRIVIA.set(value.trivia);
}

pub struct Config {
    #[cfg(feature = "track_references")]
    pub track_references: bool,
    #[cfg(feature = "trivia")]
    pub trivia: bool,
}
