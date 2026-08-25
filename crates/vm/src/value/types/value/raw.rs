use super::Payload;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ValueTag {
    Nil,
    Boolean,
    StaticStr,
    String,
    Array,
    Record,
    Function,
    Module,
    Extern,
    Uninitialized,
}

impl ValueTag {
    #[inline]
    const fn header(self) -> u16 {
        let (negative, value) = match self {
            Self::Nil => (false, 1),
            Self::Boolean => (false, 2),
            Self::StaticStr => (false, 3),
            Self::String => (false, 4),
            Self::Array => (false, 5),
            Self::Record => (false, 6),
            Self::Function => (false, 7),
            Self::Module => (true, 1),
            Self::Extern => (true, 2),
            Self::Uninitialized => (true, 3),
        };
        0x7FF8 | ((negative as u16) << 15) | value
    }

    #[inline]
    const fn from_header(header: u16) -> Option<Self> {
        if header & 0x7FF8 != 0x7FF8 {
            return None;
        }
        Some(match (header & 0x8000 != 0, (header & 0x0007) as u8) {
            (false, 1) => Self::Nil,
            (false, 2) => Self::Boolean,
            (false, 3) => Self::StaticStr,
            (false, 4) => Self::String,
            (false, 5) => Self::Array,
            (false, 6) => Self::Record,
            (false, 7) => Self::Function,
            (true, 1) => Self::Module,
            (true, 2) => Self::Extern,
            (true, 3) => Self::Uninitialized,
            _ => return None,
        })
    }
}

/// The raw bit representation used by [`MiraValue`].
///
/// Numeric values use their `f64` bit pattern. Tagged values use a quiet NaN
/// header and store a fixed little-endian six-byte payload in the low 48 bits.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub(crate) struct RawValue(pub u64);

impl RawValue {
    const QUIET_NAN: u64 = 0x7FF8_0000_0000_0000;
    const SIGN_MASK: u64 = 0x8000_0000_0000_0000;

    #[inline]
    pub const fn number(value: f64) -> Self {
        let bits = value.to_bits();
        if value.is_nan() {
            Self(Self::QUIET_NAN | (bits & Self::SIGN_MASK))
        } else {
            Self(bits)
        }
    }

    #[inline]
    pub const fn tagged(tag: ValueTag, payload: [u8; 6]) -> Self {
        Self(((tag.header() as u64) << 48) | Payload::from_bytes(payload).to_bits())
    }

    #[inline]
    pub const fn empty(tag: ValueTag) -> Self {
        Self::tagged(tag, [0; 6])
    }

    #[inline]
    pub const fn tag(self) -> Option<ValueTag> {
        ValueTag::from_header((self.0 >> 48) as u16)
    }

    #[inline]
    pub const fn payload(self) -> [u8; 6] {
        Payload::from_bits(self.0 & Payload::MASK).to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_value_layout_roundtrips() {
        let payload = [1, 2, 3, 4, 5, 6];
        let raw = RawValue::tagged(ValueTag::Nil, payload);

        assert_eq!(raw.0, 0x7FF9_0605_0403_0201);
        assert_eq!(raw.tag(), Some(ValueTag::Nil));
        assert_eq!(raw.payload(), payload);

        for tag in [
            ValueTag::Boolean,
            ValueTag::StaticStr,
            ValueTag::String,
            ValueTag::Array,
            ValueTag::Record,
            ValueTag::Function,
            ValueTag::Module,
            ValueTag::Extern,
            ValueTag::Uninitialized,
        ] {
            let raw = RawValue::tagged(tag, payload);
            assert_eq!(raw.tag(), Some(tag));
            assert_eq!(raw.payload(), payload);
        }
    }
}
