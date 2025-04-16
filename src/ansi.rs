use std::fmt::Write;

use anstyle::{AnsiColor, Color, Reset, Style};

pub(crate) const KEYWORD: Style = AnsiColor::Cyan.on_default();
pub(crate) const STRING: Style = AnsiColor::BrightMagenta.on_default();
pub(crate) const INTERPOLATED: Style = AnsiColor::BrightCyan.on_default();
pub(crate) const NUMBER: Style = AnsiColor::BrightGreen.on_default();
pub(crate) const ORDINAL: Style = AnsiColor::BrightGreen.on_default().italic();
pub(crate) const RANGE: Style = AnsiColor::Green.on_default();
pub(crate) const VARIABLE: Style = AnsiColor::BrightYellow.on_default();
pub(crate) const RECOVER: Style = Style::new().bg_color(Some(Color::Ansi(AnsiColor::Red)));
pub(crate) const SPACE: Style = AnsiColor::Black.on_default().dimmed();
pub(crate) const GROUP: Style = AnsiColor::Blue.on_default();
pub(crate) const INLINE_HINT: Style =
    Style::new().bg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));
pub(crate) const COMMENT: Style = AnsiColor::BrightBlack.on_default().italic();

pub(crate) const RESET: Reset = Reset;

pub(crate) trait DisplayIdent {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result;

    fn write_ident(f: &mut std::fmt::Formatter<'_>, ident: usize, mark: &str) -> std::fmt::Result {
        write!(f, "{SPACE}{mark:8}|")?;
        for _ in 0..ident {
            f.write_char('·')?;
        }
        write!(f, "{RESET}")?;
        Ok(())
    }

    fn next_ident(ident: usize) -> usize {
        ident + 2
    }
}

impl<T: DisplayIdent> DisplayIdent for Box<T> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        T::fmt_ident(self, f, ident)
    }
    fn write_ident(f: &mut std::fmt::Formatter<'_>, ident: usize, mark: &str) -> std::fmt::Result {
        T::write_ident(f, ident, mark)
    }
    fn next_ident(ident: usize) -> usize {
        T::next_ident(ident)
    }
}

impl<T: DisplayIdent> DisplayIdent for Option<T> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        if let Some(t) = self {
            t.fmt_ident(f, ident)
        } else {
            Ok(())
        }
    }
    fn write_ident(f: &mut std::fmt::Formatter<'_>, ident: usize, mark: &str) -> std::fmt::Result {
        T::write_ident(f, ident, mark)
    }
    fn next_ident(ident: usize) -> usize {
        T::next_ident(ident)
    }
}
