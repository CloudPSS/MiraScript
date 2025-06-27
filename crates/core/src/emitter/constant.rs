use std::{borrow::Cow, io::Write};
use strum::{EnumDiscriminants, IntoDiscriminant};

#[derive(EnumDiscriminants, Clone, Debug)]
#[repr(u8)]
pub enum Constant<'s> {
    // Nil = 0,
    True = 1,
    False = 2,
    Ordinal(i32),
    Number(f64),
    String(Cow<'s, str>),
}

impl PartialEq for Constant<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0.to_bits() == r0.to_bits(),
            (Self::Ordinal(l0), Self::Ordinal(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (l, r) => l.discriminant() == r.discriminant(),
        }
    }
}

impl Eq for Constant<'_> {}

impl<'s> Constant<'s> {
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[self.discriminant() as u8])?;
        match self {
            Self::Number(n) => writer.write_all(&n.to_le_bytes())?,
            Self::Ordinal(i) => writer.write_all(&i.to_le_bytes())?,
            Self::String(s) => {
                writer.write_all(&(s.len() as u32).to_le_bytes())?;
                writer.write_all(s.as_bytes())?
            }
            _ => (),
        }
        Ok(())
    }
}
