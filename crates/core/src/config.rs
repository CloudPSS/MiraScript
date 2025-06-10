use std::cell::Cell;

#[cfg(feature = "track_references")]
thread_local! {
  static TRACK_REFERENCES: Cell<bool> = const { Cell::new(false) };
}

#[cfg(feature = "track_references")]
pub(crate) fn track_references() -> bool {
    TRACK_REFERENCES.get()
}

pub(crate) fn set_config(value: &Config) {
    #[cfg(feature = "track_references")]
    TRACK_REFERENCES.set(value.track_references);
}

pub struct Config {
    #[cfg(feature = "track_references")]
    pub track_references: bool,
}
