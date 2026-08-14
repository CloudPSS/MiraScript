use std::any::type_name;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::{MiraError, Result, RunOptions};

/// Shared ownership used when the host needs to retain and mutate a value.
pub struct MiraShared<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> MiraShared<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }

    fn identity(&self) -> usize {
        Rc::as_ptr(&self.inner) as usize
    }
}

impl<T> Clone for MiraShared<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for MiraShared<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.try_borrow() {
            Ok(value) => f.debug_tuple("MiraShared").field(&*value).finish(),
            Err(_) => f.write_str("MiraShared(<borrowed>)"),
        }
    }
}

/// A live Rust value that appears as a read-only MiraScript record.
pub trait MiraRecord: 'static {
    fn keys(&self) -> Vec<String>;
    fn get(&self, key: &str) -> Result<Option<MiraAny>>;
}

/// A live Rust value that appears as a read-only MiraScript array.
pub trait MiraArray: 'static {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Result<Option<MiraAny>>;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A live Rust object with MiraScript-visible identity and mutable fields.
pub trait MiraExtern: 'static {
    fn tag(&self) -> &str {
        type_name::<Self>()
    }

    fn keys(&self) -> Vec<String>;
    fn get(&self, key: &str) -> Result<Option<MiraAny>>;

    fn has(&self, key: &str) -> bool {
        self.keys().iter().any(|candidate| candidate == key)
    }

    fn set(&mut self, _key: &str, _value: MiraAny) -> Result<bool> {
        Ok(false)
    }

    fn is_callable(&self) -> bool {
        false
    }

    fn call(&mut self, _context: &mut MiraCallContext<'_>, _args: &[MiraAny]) -> Result<MiraAny> {
        Err(MiraError::runtime(format!(
            "Not a callable extern: {}",
            self.tag()
        )))
    }

    fn array_len(&self) -> Option<usize> {
        None
    }

    fn get_index(&self, index: usize) -> Result<Option<MiraAny>> {
        self.get(&index.to_string())
    }

    fn iterate(&self) -> Result<Option<Vec<MiraAny>>> {
        Ok(None)
    }
}

#[doc(hidden)]
pub trait MiraBridge: Sized + 'static {
    fn into_mira_shared(value: MiraShared<Self>) -> MiraAny;
}

#[doc(hidden)]
pub trait RecordObject {
    fn identity(&self) -> usize;
    fn tag(&self) -> &'static str;
    fn keys(&self) -> Result<Vec<String>>;
    fn get(&self, key: &str) -> Result<Option<MiraAny>>;
}

struct SharedRecord<T> {
    value: MiraShared<T>,
}

impl<T: MiraRecord> RecordObject for SharedRecord<T> {
    fn identity(&self) -> usize {
        self.value.identity()
    }

    fn tag(&self) -> &'static str {
        type_name::<T>()
    }

    fn keys(&self) -> Result<Vec<String>> {
        self.value
            .inner
            .try_borrow()
            .map(|value| value.keys())
            .map_err(|_| MiraError::BorrowConflict {
                operation: "read",
                tag: type_name::<T>().into(),
            })
    }

    fn get(&self, key: &str) -> Result<Option<MiraAny>> {
        self.value
            .inner
            .try_borrow()
            .map_err(|_| MiraError::BorrowConflict {
                operation: "read",
                tag: type_name::<T>().into(),
            })?
            .get(key)
    }
}

#[doc(hidden)]
pub trait ArrayObject {
    fn identity(&self) -> usize;
    fn tag(&self) -> &'static str;
    fn len(&self) -> Result<usize>;
    fn get(&self, index: usize) -> Result<Option<MiraAny>>;
}

struct SharedArray<T> {
    value: MiraShared<T>,
}

impl<T: MiraArray> ArrayObject for SharedArray<T> {
    fn identity(&self) -> usize {
        self.value.identity()
    }

    fn tag(&self) -> &'static str {
        type_name::<T>()
    }

    fn len(&self) -> Result<usize> {
        self.value
            .inner
            .try_borrow()
            .map(|value| value.len())
            .map_err(|_| MiraError::BorrowConflict {
                operation: "read",
                tag: type_name::<T>().into(),
            })
    }

    fn get(&self, index: usize) -> Result<Option<MiraAny>> {
        self.value
            .inner
            .try_borrow()
            .map_err(|_| MiraError::BorrowConflict {
                operation: "read",
                tag: type_name::<T>().into(),
            })?
            .get(index)
    }
}

trait ExternObject {
    fn identity(&self) -> usize;
    fn tag(&self) -> Result<String>;
    fn keys(&self) -> Result<Vec<String>>;
    fn has(&self, key: &str) -> Result<bool>;
    fn get(&self, key: &str) -> Result<Option<MiraAny>>;
    fn set(&self, key: &str, value: MiraAny) -> Result<bool>;
    fn is_callable(&self) -> Result<bool>;
    fn call(&self, context: &mut MiraCallContext<'_>, args: &[MiraAny]) -> Result<MiraAny>;
    fn array_len(&self) -> Result<Option<usize>>;
    fn get_index(&self, index: usize) -> Result<Option<MiraAny>>;
    fn iterate(&self) -> Result<Option<Vec<MiraAny>>>;
}

struct SharedExtern<T> {
    value: MiraShared<T>,
}

impl<T: MiraExtern> ExternObject for SharedExtern<T> {
    fn identity(&self) -> usize {
        self.value.identity()
    }

    fn tag(&self) -> Result<String> {
        self.try_read("read", |value| Ok(value.tag().to_owned()))
    }

    fn keys(&self) -> Result<Vec<String>> {
        self.try_read("read", |value| Ok(value.keys()))
    }

    fn has(&self, key: &str) -> Result<bool> {
        self.try_read("read", |value| Ok(value.has(key)))
    }

    fn get(&self, key: &str) -> Result<Option<MiraAny>> {
        self.try_read("read", |value| value.get(key))
    }

    fn set(&self, key: &str, value: MiraAny) -> Result<bool> {
        self.try_write("write", |object| object.set(key, value))
    }

    fn is_callable(&self) -> Result<bool> {
        self.try_read("read", |value| Ok(value.is_callable()))
    }

    fn call(&self, context: &mut MiraCallContext<'_>, args: &[MiraAny]) -> Result<MiraAny> {
        self.try_write("call", |value| value.call(context, args))
    }

    fn array_len(&self) -> Result<Option<usize>> {
        self.try_read("read", |value| Ok(value.array_len()))
    }

    fn get_index(&self, index: usize) -> Result<Option<MiraAny>> {
        self.try_read("read", |value| value.get_index(index))
    }

    fn iterate(&self) -> Result<Option<Vec<MiraAny>>> {
        self.try_read("iterate", |value| value.iterate())
    }
}

impl<T: MiraExtern> SharedExtern<T> {
    fn try_read<R>(&self, operation: &'static str, f: impl FnOnce(&T) -> Result<R>) -> Result<R> {
        let value = self
            .value
            .inner
            .try_borrow()
            .map_err(|_| MiraError::BorrowConflict {
                operation,
                tag: type_name::<T>().into(),
            })?;
        f(&value)
    }

    fn try_write<R>(
        &self,
        operation: &'static str,
        f: impl FnOnce(&mut T) -> Result<R>,
    ) -> Result<R> {
        let mut value =
            self.value
                .inner
                .try_borrow_mut()
                .map_err(|_| MiraError::BorrowConflict {
                    operation,
                    tag: type_name::<T>().into(),
                })?;
        f(&mut value)
    }
}

#[derive(Clone)]
pub struct MiraExternValue {
    inner: Rc<dyn ExternObject>,
}

impl MiraExternValue {
    pub fn identity(&self) -> usize {
        self.inner.identity()
    }

    pub fn tag(&self) -> Result<String> {
        self.inner.tag()
    }

    pub fn keys(&self) -> Result<Vec<String>> {
        self.inner.keys()
    }

    pub fn has(&self, key: &str) -> Result<bool> {
        self.inner.has(key)
    }

    pub fn get(&self, key: &str) -> Result<Option<MiraAny>> {
        self.inner.get(key)
    }

    pub fn set(&self, key: &str, value: MiraAny) -> Result<bool> {
        self.inner.set(key, value)
    }

    pub fn is_callable(&self) -> Result<bool> {
        self.inner.is_callable()
    }

    pub(crate) fn call(
        &self,
        context: &mut MiraCallContext<'_>,
        args: &[MiraAny],
    ) -> Result<MiraAny> {
        self.inner.call(context, args)
    }

    pub fn array_len(&self) -> Result<Option<usize>> {
        self.inner.array_len()
    }

    pub fn get_index(&self, index: usize) -> Result<Option<MiraAny>> {
        self.inner.get_index(index)
    }

    pub fn iterate(&self) -> Result<Option<Vec<MiraAny>>> {
        self.inner.iterate()
    }
}

impl fmt::Debug for MiraExternValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MiraExternValue")
            .field("identity", &self.identity())
            .field("tag", &self.tag().unwrap_or_else(|_| "<borrowed>".into()))
            .finish()
    }
}

pub(crate) trait NativeRuntime {
    fn call_value(&self, function: &MiraAny, args: &[MiraAny]) -> Result<MiraAny>;
    fn get_value(&self, value: &MiraAny, key: &MiraAny) -> Result<MiraAny>;
    fn options(&self) -> &RunOptions;
    fn checkpoint(&self) -> Result<()>;
}

pub struct MiraCallContext<'a> {
    pub(crate) runtime: &'a dyn NativeRuntime,
}

impl MiraCallContext<'_> {
    pub fn call(&mut self, function: &MiraAny, args: &[MiraAny]) -> Result<MiraAny> {
        self.runtime.call_value(function, args)
    }

    /// Read a field using the same rules as a MiraScript expression.
    pub fn get(&mut self, value: &MiraAny, key: impl Into<MiraAny>) -> Result<MiraAny> {
        self.runtime.get_value(value, &key.into())
    }

    pub fn options(&self) -> &RunOptions {
        self.runtime.options()
    }

    pub fn checkpoint(&mut self) -> Result<()> {
        self.runtime.checkpoint()
    }
}

type NativeCallback = dyn for<'a> Fn(&mut MiraCallContext<'a>, &[MiraAny]) -> Result<MiraAny>;

#[derive(Clone)]
pub struct MiraNativeFn {
    callback: Rc<NativeCallback>,
    name: Rc<str>,
}

impl MiraNativeFn {
    pub fn new(
        name: impl Into<String>,
        callback: impl for<'a> Fn(&mut MiraCallContext<'a>, &[MiraAny]) -> Result<MiraAny> + 'static,
    ) -> Self {
        Self {
            callback: Rc::new(callback),
            name: Rc::from(name.into()),
        }
    }

    pub fn anonymous(
        callback: impl for<'a> Fn(&mut MiraCallContext<'a>, &[MiraAny]) -> Result<MiraAny> + 'static,
    ) -> Self {
        Self::new("<native>", callback)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn shared_name(&self) -> Rc<str> {
        Rc::clone(&self.name)
    }

    pub(crate) fn call(
        &self,
        context: &mut MiraCallContext<'_>,
        args: &[MiraAny],
    ) -> Result<MiraAny> {
        (self.callback)(context, args)
    }

    fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.callback, &other.callback)
    }
}

impl<F> From<F> for MiraNativeFn
where
    F: for<'a> Fn(&mut MiraCallContext<'a>, &[MiraAny]) -> Result<MiraAny> + 'static,
{
    fn from(value: F) -> Self {
        Self::anonymous(value)
    }
}

impl fmt::Debug for MiraNativeFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MiraNativeFn").field(&self.name).finish()
    }
}

#[derive(Debug, Clone)]
pub enum MiraFunction {
    Native(MiraNativeFn),
    #[doc(hidden)]
    Script {
        execution: u64,
        function: usize,
        frame: usize,
        name: Option<Rc<str>>,
    },
}

impl MiraFunction {
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Native(function) => Some(function.name()),
            Self::Script { name, .. } => name.as_deref(),
        }
    }

    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Native(a), Self::Native(b)) => a.same(b),
            (
                Self::Script {
                    execution: ae,
                    function: af,
                    frame: ac,
                    ..
                },
                Self::Script {
                    execution: be,
                    function: bf,
                    frame: bc,
                    ..
                },
            ) => ae == be && af == bf && ac == bc,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct ScriptModule {
    pub execution: u64,
    pub frame: usize,
    pub exports: IndexMap<String, usize>,
    pub name: Rc<str>,
}

#[derive(Clone)]
pub enum MiraModule {
    Native {
        name: Rc<str>,
        values: Rc<IndexMap<String, MiraAny>>,
    },
    #[doc(hidden)]
    Script(Rc<ScriptModule>),
}

impl MiraModule {
    pub fn new(name: impl Into<String>, values: IndexMap<String, MiraAny>) -> Self {
        Self::Native {
            name: Rc::from(name.into()),
            values: Rc::new(values),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Native { name, .. } => name,
            Self::Script(module) => &module.name,
        }
    }

    pub fn keys(&self) -> Vec<String> {
        match self {
            Self::Native { values, .. } => values.keys().cloned().collect(),
            Self::Script(module) => module.exports.keys().cloned().collect(),
        }
    }

    pub fn get_native(&self, key: &str) -> Option<MiraAny> {
        match self {
            Self::Native { values, .. } => values.get(key).cloned(),
            Self::Script(_) => None,
        }
    }

    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Native { values: a, .. }, Self::Native { values: b, .. }) => Rc::ptr_eq(a, b),
            (Self::Script(a), Self::Script(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Debug for MiraModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MiraModule")
            .field("name", &self.name())
            .field("keys", &self.keys())
            .finish()
    }
}

/// A value understood by the Rust VM.
#[derive(Clone, Default)]
pub enum MiraAny {
    #[doc(hidden)]
    Uninitialized,
    #[default]
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<MiraAny>),
    Record(IndexMap<String, MiraAny>),
    Function(MiraFunction),
    Module(MiraModule),
    Extern(MiraExternValue),
    #[doc(hidden)]
    RustRecord(Rc<dyn RecordObject>),
    #[doc(hidden)]
    RustArray(Rc<dyn ArrayObject>),
}

impl MiraAny {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Uninitialized | Self::Nil => "nil",
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Array(_) | Self::RustArray(_) => "array",
            Self::Record(_) | Self::RustRecord(_) => "record",
            Self::Function(_) => "function",
            Self::Module(_) => "module",
            Self::Extern(_) => "extern",
        }
    }

    pub fn from_record<T: MiraRecord>(value: T) -> Self {
        Self::from_record_shared(MiraShared::new(value))
    }

    pub fn from_record_shared<T: MiraRecord>(value: MiraShared<T>) -> Self {
        Self::RustRecord(Rc::new(SharedRecord { value }))
    }

    pub fn from_array<T: MiraArray>(value: T) -> Self {
        Self::from_array_shared(MiraShared::new(value))
    }

    pub fn from_array_shared<T: MiraArray>(value: MiraShared<T>) -> Self {
        Self::RustArray(Rc::new(SharedArray { value }))
    }

    pub fn from_extern<T: MiraExtern>(value: T) -> Self {
        Self::from_extern_shared(MiraShared::new(value))
    }

    pub fn from_extern_shared<T: MiraExtern>(value: MiraShared<T>) -> Self {
        Self::Extern(MiraExternValue {
            inner: Rc::new(SharedExtern { value }),
        })
    }

    pub fn is_initialized(&self) -> bool {
        !matches!(self, Self::Uninitialized)
    }

    pub(crate) fn into_element(self) -> Result<Self> {
        match self {
            Self::Uninitialized => Err(MiraError::runtime("Uninitialized value")),
            Self::Function(_) | Self::Module(_) | Self::Extern(_) => Ok(Self::Nil),
            value => Ok(value),
        }
    }

    pub(crate) fn contains_script_reference(&self, execution: u64) -> bool {
        match self {
            Self::Function(MiraFunction::Script {
                execution: owner, ..
            }) => *owner == execution,
            Self::Module(MiraModule::Script(module)) => module.execution == execution,
            Self::Array(values) => values
                .iter()
                .any(|value| value.contains_script_reference(execution)),
            Self::Record(values) => values
                .values()
                .any(|value| value.contains_script_reference(execution)),
            _ => false,
        }
    }

    pub(crate) fn record_keys(&self) -> Result<Option<Vec<String>>> {
        match self {
            Self::Record(record) => Ok(Some(record.keys().cloned().collect())),
            Self::RustRecord(record) => record.keys().map(Some),
            _ => Ok(None),
        }
    }

    pub(crate) fn record_get(&self, key: &str) -> Result<Option<MiraAny>> {
        match self {
            Self::Record(record) => Ok(record.get(key).cloned()),
            Self::RustRecord(record) => record.get(key),
            _ => Ok(None),
        }
    }

    pub(crate) fn array_len(&self) -> Result<Option<usize>> {
        match self {
            Self::Array(array) => Ok(Some(array.len())),
            Self::RustArray(array) => array.len().map(Some),
            Self::Extern(value) => value.array_len(),
            _ => Ok(None),
        }
    }

    pub(crate) fn array_get(&self, index: usize) -> Result<Option<MiraAny>> {
        match self {
            Self::Array(array) => Ok(array.get(index).cloned()),
            Self::RustArray(array) => array.get(index),
            Self::Extern(value) => value.get_index(index),
            _ => Ok(None),
        }
    }
}

fn same_record(a: &MiraAny, b: &MiraAny) -> bool {
    let Ok(Some(a_keys)) = a.record_keys() else {
        return false;
    };
    let Ok(Some(b_keys)) = b.record_keys() else {
        return false;
    };
    if a_keys.len() != b_keys.len() {
        return false;
    }
    a_keys.into_iter().all(|key| {
        if !b_keys.contains(&key) {
            return false;
        }
        match (
            crate::operations::get(a, &key),
            crate::operations::get(b, &key),
        ) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    })
}

fn same_array(a: &MiraAny, b: &MiraAny) -> bool {
    let (Ok(Some(a_len)), Ok(Some(b_len))) = (a.array_len(), b.array_len()) else {
        return false;
    };
    if a_len != b_len {
        return false;
    }
    (0..a_len).all(|index| match (a.array_get(index), b.array_get(index)) {
        (Ok(a), Ok(b)) => a.unwrap_or(MiraAny::Nil) == b.unwrap_or(MiraAny::Nil),
        _ => false,
    })
}

impl PartialEq for MiraAny {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Uninitialized, Self::Uninitialized) | (Self::Nil, Self::Nil) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b || (a.is_nan() && b.is_nan()),
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Array(_), Self::Array(_))
            | (Self::Array(_), Self::RustArray(_))
            | (Self::RustArray(_), Self::Array(_))
            | (Self::RustArray(_), Self::RustArray(_)) => same_array(self, other),
            (Self::Record(_), Self::Record(_))
            | (Self::Record(_), Self::RustRecord(_))
            | (Self::RustRecord(_), Self::Record(_))
            | (Self::RustRecord(_), Self::RustRecord(_)) => same_record(self, other),
            (Self::Function(a), Self::Function(b)) => a.same(b),
            (Self::Module(a), Self::Module(b)) => a.same(b),
            (Self::Extern(a), Self::Extern(b)) => a.identity() == b.identity(),
            _ => false,
        }
    }
}

impl fmt::Debug for MiraAny {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uninitialized => f.write_str("Uninitialized"),
            Self::Nil => f.write_str("Nil"),
            Self::Boolean(value) => f.debug_tuple("Boolean").field(value).finish(),
            Self::Number(value) => f.debug_tuple("Number").field(value).finish(),
            Self::String(value) => f.debug_tuple("String").field(value).finish(),
            Self::Array(value) => f.debug_tuple("Array").field(value).finish(),
            Self::Record(value) => f.debug_tuple("Record").field(value).finish(),
            Self::Function(value) => f.debug_tuple("Function").field(value).finish(),
            Self::Module(value) => f.debug_tuple("Module").field(value).finish(),
            Self::Extern(value) => f.debug_tuple("Extern").field(value).finish(),
            Self::RustRecord(value) => f
                .debug_struct("RustRecord")
                .field("tag", &value.tag())
                .field("identity", &value.identity())
                .finish(),
            Self::RustArray(value) => f
                .debug_struct("RustArray")
                .field("tag", &value.tag())
                .field("identity", &value.identity())
                .finish(),
        }
    }
}

impl From<()> for MiraAny {
    fn from(_: ()) -> Self {
        Self::Nil
    }
}

impl From<bool> for MiraAny {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for MiraAny {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for MiraAny {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

macro_rules! number_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl From<$ty> for MiraAny {
            fn from(value: $ty) -> Self {
                Self::Number(value as f64)
            }
        }
    )* };
}

number_from!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

impl<T: Into<MiraAny>> From<Option<T>> for MiraAny {
    fn from(value: Option<T>) -> Self {
        value.map(Into::into).unwrap_or(Self::Nil)
    }
}

impl<T: Into<MiraAny>> From<Vec<T>> for MiraAny {
    fn from(value: Vec<T>) -> Self {
        Self::Array(value.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<MiraAny>, const N: usize> From<[T; N]> for MiraAny {
    fn from(value: [T; N]) -> Self {
        Self::Array(value.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<MiraAny>> From<IndexMap<String, T>> for MiraAny {
    fn from(value: IndexMap<String, T>) -> Self {
        Self::Record(
            value
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        )
    }
}

impl<K, T, S> From<HashMap<K, T, S>> for MiraAny
where
    K: Into<String> + Eq + Hash,
    T: Into<MiraAny>,
    S: BuildHasher,
{
    fn from(value: HashMap<K, T, S>) -> Self {
        let mut entries: Vec<_> = value
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        Self::Record(entries.into_iter().collect())
    }
}

impl<K, T> From<BTreeMap<K, T>> for MiraAny
where
    K: Into<String> + Ord,
    T: Into<MiraAny>,
{
    fn from(value: BTreeMap<K, T>) -> Self {
        Self::Record(
            value
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }
}

impl From<MiraNativeFn> for MiraAny {
    fn from(value: MiraNativeFn) -> Self {
        Self::Function(MiraFunction::Native(value))
    }
}

impl From<MiraModule> for MiraAny {
    fn from(value: MiraModule) -> Self {
        Self::Module(value)
    }
}

impl<T: MiraBridge> From<MiraShared<T>> for MiraAny {
    fn from(value: MiraShared<T>) -> Self {
        T::into_mira_shared(value)
    }
}

impl TryFrom<MiraAny> for bool {
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        match value {
            MiraAny::Boolean(value) => Ok(value),
            value => Err(MiraError::conversion("bool", &value)),
        }
    }
}

impl TryFrom<MiraAny> for String {
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        match value {
            MiraAny::String(value) => Ok(value),
            value => Err(MiraError::conversion("String", &value)),
        }
    }
}

impl<T> TryFrom<MiraAny> for Option<T>
where
    T: TryFrom<MiraAny, Error = MiraError>,
{
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        if value == MiraAny::Nil {
            Ok(None)
        } else {
            T::try_from(value).map(Some)
        }
    }
}

impl<T> TryFrom<MiraAny> for Vec<T>
where
    T: TryFrom<MiraAny, Error = MiraError>,
{
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        let MiraAny::Array(values) = value else {
            return Err(MiraError::conversion("Vec", &value));
        };
        values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                T::try_from(value).map_err(|error| error.at_path(index.to_string()))
            })
            .collect()
    }
}

impl<T, const N: usize> TryFrom<MiraAny> for [T; N]
where
    T: TryFrom<MiraAny, Error = MiraError>,
{
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        let values = Vec::<T>::try_from(value)?;
        let actual = values.len();
        values.try_into().map_err(|_| MiraError::Conversion {
            expected: format!("array of length {N}"),
            actual: format!("array of length {actual}"),
            path: None,
        })
    }
}

impl<T> TryFrom<MiraAny> for IndexMap<String, T>
where
    T: TryFrom<MiraAny, Error = MiraError>,
{
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        let MiraAny::Record(values) = value else {
            return Err(MiraError::conversion("record", &value));
        };
        values
            .into_iter()
            .map(|(key, value)| {
                T::try_from(value)
                    .map(|value| (key.clone(), value))
                    .map_err(|error| error.at_path(key))
            })
            .collect()
    }
}

macro_rules! unsigned_integer_try_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl TryFrom<MiraAny> for $ty {
            type Error = MiraError;

            fn try_from(value: MiraAny) -> Result<Self> {
                let MiraAny::Number(number) = value else {
                    return Err(MiraError::conversion(stringify!($ty), &value));
                };
                if !number.is_finite()
                    || number.trunc() != number
                    || number < 0.0
                    || number >= 2_f64.powi(<$ty>::BITS as i32)
                {
                    return Err(MiraError::Conversion {
                        expected: stringify!($ty).into(),
                        actual: format!("number {number}"),
                        path: None,
                    });
                }
                Ok(number as $ty)
            }
        }
    )* };
}

macro_rules! signed_integer_try_from {
    ($($ty:ty),* $(,)?) => {$ (
        impl TryFrom<MiraAny> for $ty {
            type Error = MiraError;

            fn try_from(value: MiraAny) -> Result<Self> {
                let MiraAny::Number(number) = value else {
                    return Err(MiraError::conversion(stringify!($ty), &value));
                };
                let limit = 2_f64.powi(<$ty>::BITS as i32 - 1);
                if !number.is_finite()
                    || number.trunc() != number
                    || number < -limit
                    || number >= limit
                {
                    return Err(MiraError::Conversion {
                        expected: stringify!($ty).into(),
                        actual: format!("number {number}"),
                        path: None,
                    });
                }
                Ok(number as $ty)
            }
        }
    )* };
}

unsigned_integer_try_from!(u8, u16, u32, u64, usize);
signed_integer_try_from!(i8, i16, i32, i64, isize);

impl TryFrom<MiraAny> for f64 {
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        match value {
            MiraAny::Number(value) => Ok(value),
            value => Err(MiraError::conversion("f64", &value)),
        }
    }
}

impl TryFrom<MiraAny> for f32 {
    type Error = MiraError;

    fn try_from(value: MiraAny) -> Result<Self> {
        let value = f64::try_from(value)?;
        if value.is_finite() && (value < f32::MIN as f64 || value > f32::MAX as f64) {
            return Err(MiraError::Conversion {
                expected: "f32".into(),
                actual: format!("number {value}"),
                path: None,
            });
        }
        Ok(value as f32)
    }
}
