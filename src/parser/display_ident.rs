use std::fmt::{Display, Write};

use crate::ansi::{RESET, SPACE};

pub(super) trait DisplayIdent: Display {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result;

    fn write_ident(f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        write!(f, "{SPACE}")?;
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
