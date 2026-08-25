mod raw;

use std::fmt;

use crate::{MiraType, value::arena::MiraHandle};

use super::{MiraArray, MiraExtern, MiraFunction, MiraModule, MiraRecord, Payload};

pub(super) use raw::*;

/// A decoded view of a compact [`MiraValue`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum MiraValueKind {
    Nil,
    Boolean(bool),
    Number(f64),
    StaticStr(&'static &'static str),
    String(MiraHandle<String>),
    Array(MiraHandle<dyn MiraArray>),
    Record(MiraHandle<dyn MiraRecord>),
    Function(MiraHandle<dyn MiraFunction>),
    Module(MiraHandle<dyn MiraModule>),
    Extern(MiraHandle<dyn MiraExtern>),
}

/// A compact value understood by the Rust VM.
///
/// Numbers are stored inline as `f64`. Other values use checked tags and a
/// 48-bit payload in a NaN-box. Runtime-owned payloads remain checked handles
/// into a [`crate::Runtime`] arena.
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct MiraValue(u64);

const _: () = assert!(std::mem::size_of::<MiraValue>() == 8);
const _: () = assert!(std::mem::align_of::<MiraValue>() == 8);

impl MiraValue {
    #[inline]
    pub(super) const fn from_raw(raw: RawValue) -> Self {
        Self(raw.0)
    }

    #[inline]
    pub(super) const fn raw(self) -> RawValue {
        RawValue(self.0)
    }

    #[inline]
    pub(super) const fn empty(tag: ValueTag) -> Self {
        Self::from_raw(RawValue::empty(tag))
    }

    #[inline]
    pub(super) const fn handle<T: std::any::Any + ?Sized>(
        tag: ValueTag,
        handle: MiraHandle<T>,
    ) -> Self {
        Self::from_raw(RawValue::tagged(tag, handle.payload()))
    }

    #[inline]
    pub(super) const fn tag(&self) -> Option<ValueTag> {
        self.raw().tag()
    }

    #[inline]
    pub(crate) fn kind(&self) -> MiraValueKind {
        let Some(tag) = self.tag() else {
            return MiraValueKind::Number(f64::from_bits(self.0));
        };

        let payload = self.raw().payload();
        match tag {
            ValueTag::Nil => MiraValueKind::Nil,
            ValueTag::Boolean => MiraValueKind::Boolean(payload[0] != 0),
            ValueTag::StaticStr => {
                let pointer: *const &'static str = Payload::from_bytes(payload).to_address();
                // SAFETY: `str` only accepts an outer static reference. Its
                // provenance was exposed by the constructor and restored here,
                // and the checked tag prevents interpreting another payload.
                MiraValueKind::StaticStr(unsafe { &*pointer })
            }
            ValueTag::String => MiraValueKind::String(MiraHandle::from_payload(payload)),
            ValueTag::Array => MiraValueKind::Array(MiraHandle::from_payload(payload)),
            ValueTag::Record => MiraValueKind::Record(MiraHandle::from_payload(payload)),
            ValueTag::Function => MiraValueKind::Function(MiraHandle::from_payload(payload)),
            ValueTag::Module => MiraValueKind::Module(MiraHandle::from_payload(payload)),
            ValueTag::Extern => MiraValueKind::Extern(MiraHandle::from_payload(payload)),
            ValueTag::Uninitialized => panic!("uninitialized register escaped as MiraValue"),
        }
    }

    /// Return this value's MiraScript category.
    #[inline]
    pub const fn value_type(&self) -> MiraType {
        let Some(tag) = self.tag() else {
            return MiraType::Number;
        };
        match tag {
            ValueTag::Nil => MiraType::Nil,
            ValueTag::Boolean => MiraType::Boolean,
            ValueTag::StaticStr | ValueTag::String => MiraType::String,
            ValueTag::Array => MiraType::Array,
            ValueTag::Record => MiraType::Record,
            ValueTag::Function => MiraType::Function,
            ValueTag::Module => MiraType::Module,
            ValueTag::Extern => MiraType::Extern,
            ValueTag::Uninitialized => panic!("uninitialized register escaped as MiraValue"),
        }
    }

    /// Return the MiraScript type name for this value.
    #[inline]
    pub fn type_name(&self) -> &'static str {
        self.value_type().name()
    }
}

impl Default for MiraValue {
    #[inline]
    fn default() -> Self {
        Self::empty(ValueTag::Nil)
    }
}

impl PartialEq for MiraValue {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

impl fmt::Debug for MiraValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            MiraValueKind::Nil => formatter.write_str("Nil"),
            MiraValueKind::Boolean(value) => {
                formatter.debug_tuple("Boolean").field(&value).finish()
            }
            MiraValueKind::Number(value) => formatter.debug_tuple("Number").field(&value).finish(),
            MiraValueKind::StaticStr(value) => {
                formatter.debug_tuple("StaticStr").field(&value).finish()
            }
            MiraValueKind::String(value) => formatter.debug_tuple("String").field(&value).finish(),
            MiraValueKind::Array(value) => formatter.debug_tuple("Array").field(&value).finish(),
            MiraValueKind::Record(value) => formatter.debug_tuple("Record").field(&value).finish(),
            MiraValueKind::Function(value) => {
                formatter.debug_tuple("Function").field(&value).finish()
            }
            MiraValueKind::Module(value) => formatter.debug_tuple("Module").field(&value).finish(),
            MiraValueKind::Extern(value) => formatter.debug_tuple("Extern").field(&value).finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;

    static STATIC_TEXT: &str = "static text";

    #[test]
    fn scalar_nan_box_roundtrips() {
        for number in [0.0, -0.0, 42.5, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                MiraValue::number(number).as_number().unwrap().to_bits(),
                number.to_bits()
            );
        }

        for nan in [f64::NAN, -f64::NAN] {
            let decoded = MiraValue::number(nan).as_number().unwrap();
            assert!(decoded.is_nan());
            assert_eq!(decoded.is_sign_negative(), nan.is_sign_negative());
        }

        assert!(MiraValue::nil().is_nil());
        assert_eq!(MiraValue::boolean(false).as_boolean(), Some(false));
        assert_eq!(MiraValue::boolean(true).as_boolean(), Some(true));
    }

    #[test]
    fn static_string_pointer_roundtrips() {
        let runtime = Runtime::new();
        let value = MiraValue::str(&STATIC_TEXT);

        assert_eq!(value.as_str(&runtime).unwrap(), Some(STATIC_TEXT));
    }

    #[test]
    fn dbg_fmt() {
        assert_eq!(
            format!("{:?}", MiraValue::from(&STATIC_TEXT)),
            format!("StaticStr(\"{}\")", STATIC_TEXT)
        );
        assert_eq!(format!("{:?}", MiraValue::default()), "Nil");
        assert_eq!(format!("{:?}", MiraValue::from(false)), "Boolean(false)");
        assert_eq!(format!("{:?}", MiraValue::from(12)), "Number(12.0)");
    }
}
