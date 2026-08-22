mod id;
mod key;

use std::{any::Any, fmt, marker::PhantomData, ops::Deref, rc::Rc};

use crate::{MiraError, Result, Runtime, RuntimeErrorKind};

use super::{MiraArray, MiraFunction, MiraModule, MiraRecord, MiraValue, MiraValueKind};

use id::ArenaId;
use key::ArenaKey;

#[derive(Debug)]
struct Arena<T> {
    values: Vec<T>,
}

impl<T> Arena<T> {
    fn new() -> Self {
        Self { values: Vec::new() }
    }

    fn insert(&mut self, arena_id: ArenaId, category: &'static str, value: T) -> Result<ArenaKey> {
        let key = ArenaKey::new(arena_id, self.values.len(), category)?;
        self.values.push(value);
        Ok(key)
    }

    fn get(&self, arena_id: ArenaId, category: &'static str, key: ArenaKey) -> Result<&T> {
        if key.arena_id() != arena_id {
            return Err(MiraError::runtime(RuntimeErrorKind::ForeignHandle));
        }
        self.values
            .get(key.index())
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::InvalidHandle { category }))
    }

    fn get_mut(
        &mut self,
        arena_id: ArenaId,
        category: &'static str,
        key: ArenaKey,
    ) -> Result<&mut T> {
        if key.arena_id() != arena_id {
            return Err(MiraError::runtime(RuntimeErrorKind::ForeignHandle));
        }
        self.values
            .get_mut(key.index())
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::InvalidHandle { category }))
    }
}

/// A compact, runtime-checked handle to an arena-managed value.
pub struct MiraHandle<T: Any + ?Sized> {
    key: ArenaKey,
    marker: PhantomData<&'static T>,
}

impl<T: Any + ?Sized> Copy for MiraHandle<T> {}

impl<T: Any + ?Sized> Clone for MiraHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Any + ?Sized> fmt::Debug for MiraHandle<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MiraHandle")
            .field("arena", &self.key.arena_id().get())
            .field("slot", &self.key.index())
            .finish()
    }
}

impl<T: Any + ?Sized> PartialEq for MiraHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: Any + ?Sized> Eq for MiraHandle<T> {}

impl<T: Any + ?Sized> std::hash::Hash for MiraHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl<T: Any + ?Sized> MiraHandle<T> {
    fn new(key: ArenaKey) -> Self {
        Self {
            key,
            marker: PhantomData,
        }
    }

    pub(crate) fn payload(self) -> [u8; 6] {
        self.key.payload()
    }

    pub(crate) fn from_payload(payload: [u8; 6]) -> Self {
        Self::new(ArenaKey::from_payload(payload))
    }
}

macro_rules! impl_handle_cast {
    ($trait:path, $erase:ident) => {
        impl<T: $trait + ?Sized> MiraHandle<T> {
            /// Erase the concrete Rust type while preserving its MiraScript category.
            pub fn $erase(self) -> MiraHandle<dyn $trait> {
                MiraHandle::new(self.key)
            }
        }

        impl MiraHandle<dyn $trait> {
            /// Reinterpret an erased handle as a concrete typed handle.
            ///
            /// The subsequent Runtime lookup still validates the concrete type,
            /// so a wrong generated cast returns an error rather than dereferencing
            /// an invalid pointer.
            #[doc(hidden)]
            pub unsafe fn upcast<T: $trait>(self) -> MiraHandle<T> {
                MiraHandle::new(self.key)
            }
        }
    };
}

impl_handle_cast!(MiraArray, erase_array);
impl_handle_cast!(MiraRecord, erase_record);
impl_handle_cast!(MiraFunction, erase_function);
impl_handle_cast!(MiraModule, erase_module);

/// An owned value waiting to be inserted into a Runtime arena.
pub enum MiraManageable {
    /// A value whose payload is already inline or already belongs to this Runtime.
    Value(MiraValue),
    /// An owned string.
    String(String),
    /// An owned array implementation.
    Array(Box<dyn MiraArray>),
    /// An owned record implementation.
    Record(Box<dyn MiraRecord>),
    /// An owned callable implementation.
    Function(Rc<dyn MiraFunction>),
    /// An owned module implementation.
    Module(Box<dyn MiraModule>),
}

impl From<MiraValue> for MiraManageable {
    fn from(value: MiraValue) -> Self {
        Self::Value(value)
    }
}

impl From<String> for MiraManageable {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for MiraManageable {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl MiraManageable {
    /// Wrap an array implementation for insertion into a Runtime.
    pub fn from_array(value: impl MiraArray) -> Self {
        Self::Array(Box::new(value))
    }

    /// Wrap a record implementation for insertion into a Runtime.
    pub fn from_record(value: impl MiraRecord) -> Self {
        Self::Record(Box::new(value))
    }

    /// Wrap a function implementation for insertion into a Runtime.
    pub fn from_function(value: impl MiraFunction) -> Self {
        Self::Function(Rc::new(value))
    }

    /// Wrap a module implementation for insertion into a Runtime.
    pub fn from_module(value: impl MiraModule) -> Self {
        Self::Module(Box::new(value))
    }

    /// Build a simple named module from already materialized Runtime values.
    pub fn map_module(
        name: impl Into<String>,
        values: indexmap::IndexMap<String, MiraValue>,
    ) -> Self {
        crate::value::types::map_module(name, values)
    }
}

pub(crate) struct MiraArena {
    id: ArenaId,
    strings: Arena<String>,
    arrays: Arena<Box<dyn MiraArray>>,
    records: Arena<Box<dyn MiraRecord>>,
    functions: Arena<Rc<dyn MiraFunction>>,
    modules: Arena<Box<dyn MiraModule>>,
}

impl MiraArena {
    pub(crate) fn new() -> Self {
        Self {
            id: ArenaId::next(),
            strings: Arena::new(),
            arrays: Arena::new(),
            records: Arena::new(),
            functions: Arena::new(),
            modules: Arena::new(),
        }
    }
}

impl Runtime {
    /// Insert an owned value into this Runtime when necessary.
    pub fn insert(&mut self, value: impl Into<MiraManageable>) -> Result<MiraValue> {
        match value.into() {
            MiraManageable::Value(value) => {
                self.validate_value(&value)?;
                Ok(value)
            }
            MiraManageable::String(value) => {
                Ok(MiraValue::from_string_handle(self.insert_string(value)?))
            }
            MiraManageable::Array(value) => {
                let key = self.arena.arrays.insert(self.arena.id, "array", value)?;
                Ok(MiraValue::from_array_handle(MiraHandle::new(key)))
            }
            MiraManageable::Record(value) => {
                let key = self.arena.records.insert(self.arena.id, "record", value)?;
                Ok(MiraValue::from_record_handle(MiraHandle::new(key)))
            }
            MiraManageable::Function(value) => {
                let key = self
                    .arena
                    .functions
                    .insert(self.arena.id, "function", value)?;
                Ok(MiraValue::from_function_handle(MiraHandle::new(key)))
            }
            MiraManageable::Module(value) => {
                let key = self.arena.modules.insert(self.arena.id, "module", value)?;
                Ok(MiraValue::from_module_handle(MiraHandle::new(key)))
            }
        }
    }

    /// Insert an owned string and return its typed handle.
    pub fn insert_string(&mut self, value: impl Into<String>) -> Result<MiraHandle<String>> {
        let key = self
            .arena
            .strings
            .insert(self.arena.id, "string", value.into())?;
        Ok(MiraHandle::new(key))
    }

    /// Read a string handle owned by this Runtime.
    pub fn get_string(&self, handle: MiraHandle<String>) -> Result<&str> {
        self.arena
            .strings
            .get(self.arena.id, "string", handle.key)
            .map(Deref::deref)
    }

    /// Mutably read a string handle owned by this Runtime.
    pub fn get_string_mut(&mut self, handle: MiraHandle<String>) -> Result<&mut String> {
        self.arena
            .strings
            .get_mut(self.arena.id, "string", handle.key)
    }

    /// Insert an array and return a concrete typed handle.
    pub fn insert_array<T: MiraArray>(&mut self, value: T) -> Result<MiraHandle<T>> {
        let key = self
            .arena
            .arrays
            .insert(self.arena.id, "array", Box::new(value))?;
        Ok(MiraHandle::new(key))
    }

    /// Insert a record and return a concrete typed handle.
    pub fn insert_record<T: MiraRecord>(&mut self, value: T) -> Result<MiraHandle<T>> {
        let key = self
            .arena
            .records
            .insert(self.arena.id, "record", Box::new(value))?;
        Ok(MiraHandle::new(key))
    }

    /// Insert a function and return a concrete typed handle.
    pub fn insert_function<T: MiraFunction>(&mut self, value: T) -> Result<MiraHandle<T>> {
        let key = self
            .arena
            .functions
            .insert(self.arena.id, "function", Rc::new(value))?;
        Ok(MiraHandle::new(key))
    }

    /// Insert a module and return a concrete typed handle.
    pub fn insert_module<T: MiraModule>(&mut self, value: T) -> Result<MiraHandle<T>> {
        let key = self
            .arena
            .modules
            .insert(self.arena.id, "module", Box::new(value))?;
        Ok(MiraHandle::new(key))
    }

    pub(crate) fn get_array_dyn(
        &self,
        handle: MiraHandle<dyn MiraArray>,
    ) -> Result<&dyn MiraArray> {
        self.arena
            .arrays
            .get(self.arena.id, "array", handle.key)
            .map(|value| value.as_ref())
    }

    pub(crate) fn get_record_dyn(
        &self,
        handle: MiraHandle<dyn MiraRecord>,
    ) -> Result<&dyn MiraRecord> {
        self.arena
            .records
            .get(self.arena.id, "record", handle.key)
            .map(|value| value.as_ref())
    }

    pub(crate) fn get_function_dyn(
        &self,
        handle: MiraHandle<dyn MiraFunction>,
    ) -> Result<Rc<dyn MiraFunction>> {
        self.arena
            .functions
            .get(self.arena.id, "function", handle.key)
            .map(Rc::clone)
    }

    pub(crate) fn get_module_dyn(
        &self,
        handle: MiraHandle<dyn MiraModule>,
    ) -> Result<&dyn MiraModule> {
        self.arena
            .modules
            .get(self.arena.id, "module", handle.key)
            .map(|value| value.as_ref())
    }

    /// Read the concrete target represented by a typed array handle.
    pub fn get_array<T: MiraArray>(&self, handle: MiraHandle<T>) -> Result<&T> {
        let value = self.arena.arrays.get(self.arena.id, "array", handle.key)?;
        let value: &dyn Any = value.resolve(self)?.unwrap_or(value.as_ref());
        value.downcast_ref::<T>().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch { category: "array" })
        })
    }

    /// Mutably read the concrete object stored directly behind an array handle.
    pub fn get_array_mut<T: MiraArray>(&mut self, handle: MiraHandle<T>) -> Result<&mut T> {
        let value = self
            .arena
            .arrays
            .get_mut(self.arena.id, "array", handle.key)?;
        let value: &mut dyn Any = value.as_mut();
        value.downcast_mut::<T>().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch { category: "array" })
        })
    }

    /// Read the concrete target represented by a typed record handle.
    pub fn get_record<T: MiraRecord>(&self, handle: MiraHandle<T>) -> Result<&T> {
        let value = self
            .arena
            .records
            .get(self.arena.id, "record", handle.key)?;
        let value: &dyn Any = value.resolve(self)?.unwrap_or(value.as_ref());
        value.downcast_ref::<T>().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch { category: "record" })
        })
    }

    /// Mutably read the concrete object stored directly behind a record handle.
    pub fn get_record_mut<T: MiraRecord>(&mut self, handle: MiraHandle<T>) -> Result<&mut T> {
        let value = self
            .arena
            .records
            .get_mut(self.arena.id, "record", handle.key)?;
        let value: &mut dyn Any = value.as_mut();
        value.downcast_mut::<T>().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch { category: "record" })
        })
    }

    /// Read the concrete function stored behind a typed handle.
    pub fn get_function<T: MiraFunction>(&self, handle: MiraHandle<T>) -> Result<&T> {
        let value = self
            .arena
            .functions
            .get(self.arena.id, "function", handle.key)?;
        let value: &dyn Any = value.as_ref();
        value.downcast_ref::<T>().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch {
                category: "function",
            })
        })
    }

    /// Read the concrete module stored behind a typed handle.
    pub fn get_module<T: MiraModule>(&self, handle: MiraHandle<T>) -> Result<&T> {
        let value = self
            .arena
            .modules
            .get(self.arena.id, "module", handle.key)?;
        let value: &dyn Any = value.as_ref();
        value.downcast_ref::<T>().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch { category: "module" })
        })
    }

    /// Mutably read the concrete module stored behind a typed handle.
    pub fn get_module_mut<T: MiraModule>(&mut self, handle: MiraHandle<T>) -> Result<&mut T> {
        let value = self
            .arena
            .modules
            .get_mut(self.arena.id, "module", handle.key)?;
        let value: &mut dyn Any = value.as_mut();
        value.downcast_mut::<T>().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch { category: "module" })
        })
    }

    pub(crate) fn validate_value(&self, value: &MiraValue) -> Result<()> {
        match value.kind() {
            MiraValueKind::String(handle) => self.get_string(handle).map(|_| ()),
            MiraValueKind::Array(handle) => self.get_array_dyn(handle).map(|_| ()),
            MiraValueKind::Record(handle) => self.get_record_dyn(handle).map(|_| ()),
            MiraValueKind::Function(handle) => self.get_function_dyn(handle).map(|_| ()),
            MiraValueKind::Module(handle) => self.get_module_dyn(handle).map(|_| ()),
            MiraValueKind::Extern(_) => Err(MiraError::runtime(RuntimeErrorKind::InvalidHandle {
                category: "extern",
            })),
            MiraValueKind::Nil
            | MiraValueKind::Boolean(_)
            | MiraValueKind::Number(_)
            | MiraValueKind::StaticStr(_) => Ok(()),
        }
    }
}

