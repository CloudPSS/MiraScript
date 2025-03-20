use anstyle::{AnsiColor, Reset, Style};

pub(crate) const KEYWORD: Style = AnsiColor::BrightBlue.on_default();
pub(crate) const STRING: Style = AnsiColor::BrightMagenta.on_default();
pub(crate) const INTERPOLATED: Style = STRING.bold();
pub(crate) const RESET: Reset = Reset;
