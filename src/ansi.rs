use anstyle::{AnsiColor, Color, Reset, Style};

pub(crate) const KEYWORD: Style = AnsiColor::Cyan.on_default();
pub(crate) const STRING: Style = AnsiColor::BrightMagenta.on_default();
pub(crate) const INTERPOLATED: Style = AnsiColor::BrightCyan.on_default();
pub(crate) const NUMBER: Style = AnsiColor::BrightGreen.on_default();
pub(crate) const ORDINAL: Style = AnsiColor::BrightGreen.on_default().italic();
pub(crate) const VARIABLE: Style = AnsiColor::BrightYellow.on_default();
pub(crate) const RECOVER: Style = Style::new().bg_color(Some(Color::Ansi(AnsiColor::Red)));
pub(crate) const SPACE: Style = AnsiColor::Black.on_default().dimmed();

pub(crate) const RESET: Reset = Reset;
