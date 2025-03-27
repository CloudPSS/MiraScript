use anstyle::{AnsiColor, Color, Reset, Style};

pub(crate) const KEYWORD: Style = AnsiColor::BrightBlue.on_default();
pub(crate) const STRING: Style = AnsiColor::BrightMagenta.on_default();
pub(crate) const INTERPOLATED: Style = STRING.bold();
pub(crate) const VARIABLE: Style = AnsiColor::BrightGreen.on_default();
pub(crate) const RECOVER: Style = Style::new().bg_color(Some(Color::Ansi(AnsiColor::Red)));

pub(crate) const RESET: Reset = Reset;
