use std::{fmt, num::NonZeroU8};

use boxing::nan::raw::{RawBox, RawTag, Value};

use crate::{MiraType, value::arena::MiraHandle};

use super::{MiraArray, MiraExtern, MiraFunction, MiraModule, MiraRecord};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueTag {
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
    fn raw(self) -> RawTag {
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
        RawTag::new(
            negative,
            NonZeroU8::new(value).expect("value tags are non-zero"),
        )
    }

    #[inline]
    fn from_header(header: u16) -> Option<Self> {
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
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct MiraValue(u64);

const _: () = assert!(std::mem::size_of::<MiraValue>() == 8);
const _: () = assert!(std::mem::align_of::<MiraValue>() == 8);
const _: () = assert!(std::mem::size_of::<Value>() == 8);

impl MiraValue {
    const PAYLOAD_MASK: usize = 0x0000_FFFF_FFFF_FFFF;

    #[inline]
    fn from_raw(raw: RawBox) -> Self {
        Self(raw.into_float_unchecked().to_bits())
    }

    #[inline]
    fn empty(tag: ValueTag) -> Self {
        Self::from_raw(RawBox::from_value(Value::empty(tag.raw())))
    }

    #[inline]
    fn handle<T: std::any::Any + ?Sized>(tag: ValueTag, handle: MiraHandle<T>) -> Self {
        Self::from_raw(RawBox::from_value(Value::new(tag.raw(), handle.payload())))
    }

    #[inline]
    pub(super) fn boxed_nil() -> Self {
        Self::empty(ValueTag::Nil)
    }

    #[inline]
    pub(super) fn boxed_boolean(value: bool) -> Self {
        Self::from_raw(RawBox::from_value(Value::new(
            ValueTag::Boolean.raw(),
            [u8::from(value), 0, 0, 0, 0, 0],
        )))
    }

    #[inline]
    pub(super) fn boxed_number(value: f64) -> Self {
        Self::from_raw(RawBox::from_float(value))
    }

    #[inline]
    pub(super) fn boxed_static_str(value: &'static &'static str) -> Self {
        let address = (value as *const &'static str).expose_provenance();
        assert!(
            address <= Self::PAYLOAD_MASK,
            "Pointer too large to store in MiraValue"
        );
        Self::from_raw(RawBox::from_value(Value::new(
            ValueTag::StaticStr.raw(),
            Self::address_payload(address),
        )))
    }

    #[inline]
    pub(crate) fn from_string_handle(handle: MiraHandle<String>) -> Self {
        Self::handle(ValueTag::String, handle)
    }

    #[inline]
    pub(crate) fn from_array_handle(handle: MiraHandle<dyn MiraArray>) -> Self {
        Self::handle(ValueTag::Array, handle)
    }

    #[inline]
    pub(crate) fn from_record_handle(handle: MiraHandle<dyn MiraRecord>) -> Self {
        Self::handle(ValueTag::Record, handle)
    }

    #[inline]
    pub(crate) fn from_function_handle(handle: MiraHandle<dyn MiraFunction>) -> Self {
        Self::handle(ValueTag::Function, handle)
    }

    #[inline]
    pub(crate) fn from_module_handle(handle: MiraHandle<dyn MiraModule>) -> Self {
        Self::handle(ValueTag::Module, handle)
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn from_extern_handle(handle: MiraHandle<dyn MiraExtern>) -> Self {
        Self::handle(ValueTag::Extern, handle)
    }

    #[inline]
    pub(crate) fn uninitialized() -> Self {
        Self::empty(ValueTag::Uninitialized)
    }

    #[inline]
    pub(crate) fn is_uninitialized(&self) -> bool {
        self.tag() == Some(ValueTag::Uninitialized)
    }

    #[inline]
    fn is_float(&self) -> bool {
        self.tag().is_none()
    }

    #[inline]
    fn raw_value(&self) -> Value {
        // SAFETY: Every non-float MiraValue is constructed through RawBox from
        // a Value whose tag is non-zero. Value is exactly eight bytes with no
        // padding. Callers first check `tag`, so the original representation
        // can be restored by bitcast.
        unsafe { std::mem::transmute::<u64, Value>(self.0) }
    }

    #[inline]
    fn tag(&self) -> Option<ValueTag> {
        ValueTag::from_header((self.0 >> 48) as u16)
    }

    #[inline]
    pub(super) fn boxed_is_nil(&self) -> bool {
        self.tag() == Some(ValueTag::Nil)
    }

    #[inline]
    pub(super) fn boxed_boolean_value(&self) -> Option<bool> {
        (self.tag() == Some(ValueTag::Boolean)).then(|| self.raw_value().data()[0] != 0)
    }

    #[inline]
    pub(super) fn boxed_number_value(&self) -> Option<f64> {
        self.is_float().then_some(f64::from_bits(self.0))
    }

    fn address_payload(address: usize) -> [u8; 6] {
        let bytes = address.to_ne_bytes();
        let mut payload = [0; 6];
        let length = bytes.len().min(payload.len());
        #[cfg(target_endian = "little")]
        {
            payload[..length].copy_from_slice(&bytes[..length]);
        }
        #[cfg(target_endian = "big")]
        {
            let payload_start = payload.len() - length;
            let bytes_start = bytes.len() - length;
            payload[payload_start..].copy_from_slice(&bytes[bytes_start..]);
        }
        payload
    }

    fn payload_address(payload: [u8; 6]) -> usize {
        let mut bytes = [0; std::mem::size_of::<usize>()];
        let length = bytes.len().min(payload.len());
        #[cfg(target_endian = "little")]
        {
            bytes[..length].copy_from_slice(&payload[..length]);
        }
        #[cfg(target_endian = "big")]
        {
            let bytes_start = bytes.len() - length;
            let payload_start = payload.len() - length;
            bytes[bytes_start..].copy_from_slice(&payload[payload_start..]);
        }
        usize::from_ne_bytes(bytes)
    }

    #[inline]
    pub(crate) fn kind(&self) -> MiraValueKind {
        let Some(tag) = self.tag() else {
            return MiraValueKind::Number(f64::from_bits(self.0));
        };

        let value = self.raw_value();
        match tag {
            ValueTag::Nil => MiraValueKind::Nil,
            ValueTag::Boolean => MiraValueKind::Boolean(value.data()[0] != 0),
            ValueTag::StaticStr => {
                let address = Self::payload_address(*value.data());
                let pointer = std::ptr::with_exposed_provenance::<&'static str>(address);
                // SAFETY: `str` only accepts an outer static reference. Its
                // provenance was exposed by the constructor and restored here,
                // and the checked tag prevents interpreting another payload.
                MiraValueKind::StaticStr(unsafe { &*pointer })
            }
            ValueTag::String => MiraValueKind::String(MiraHandle::from_payload(*value.data())),
            ValueTag::Array => MiraValueKind::Array(MiraHandle::from_payload(*value.data())),
            ValueTag::Record => MiraValueKind::Record(MiraHandle::from_payload(*value.data())),
            ValueTag::Function => MiraValueKind::Function(MiraHandle::from_payload(*value.data())),
            ValueTag::Module => MiraValueKind::Module(MiraHandle::from_payload(*value.data())),
            ValueTag::Extern => MiraValueKind::Extern(MiraHandle::from_payload(*value.data())),
            ValueTag::Uninitialized => panic!("uninitialized register escaped as MiraValue"),
        }
    }

    /// Return this value's MiraScript category.
    #[inline]
    pub fn value_type(&self) -> MiraType {
        match self.kind() {
            MiraValueKind::Nil => MiraType::Nil,
            MiraValueKind::Boolean(_) => MiraType::Boolean,
            MiraValueKind::Number(_) => MiraType::Number,
            MiraValueKind::StaticStr(_) | MiraValueKind::String(_) => MiraType::String,
            MiraValueKind::Array(_) => MiraType::Array,
            MiraValueKind::Record(_) => MiraType::Record,
            MiraValueKind::Function(_) => MiraType::Function,
            MiraValueKind::Module(_) => MiraType::Module,
            MiraValueKind::Extern(_) => MiraType::Extern,
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
}
