mod handle;
mod id;
mod key;
mod manageable;
mod typed_arena;

use std::{any::Any, ops::Deref, rc::Rc};

use crate::{MiraError, Result, Runtime, RuntimeErrorKind};

use super::{MiraArray, MiraFunction, MiraModule, MiraRecord, MiraValue, MiraValueKind};

pub use handle::MiraHandle;
use id::ArenaId;
use key::ArenaKey;
pub use manageable::MiraManageable;
use typed_arena::{Arena, CAT_ARRAY, CAT_FUNCTION, CAT_MODULE, CAT_RECORD, CAT_STRING};

pub(crate) struct MiraArena {
    id: ArenaId,
    strings: Arena<CAT_STRING, String>,
    arrays: Arena<CAT_ARRAY, Box<dyn MiraArray>>,
    records: Arena<CAT_RECORD, Box<dyn MiraRecord>>,
    functions: Arena<CAT_FUNCTION, Rc<dyn MiraFunction>>,
    modules: Arena<CAT_MODULE, Box<dyn MiraModule>>,
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
                let key = self.arena.arrays.insert(self.arena.id, value)?;
                Ok(MiraValue::from_array_handle(MiraHandle::new(key)))
            }
            MiraManageable::Record(value) => {
                let key = self.arena.records.insert(self.arena.id, value)?;
                Ok(MiraValue::from_record_handle(MiraHandle::new(key)))
            }
            MiraManageable::Function(value) => {
                let key = self.arena.functions.insert(self.arena.id, value)?;
                Ok(MiraValue::from_function_handle(MiraHandle::new(key)))
            }
            MiraManageable::Module(value) => {
                let key = self.arena.modules.insert(self.arena.id, value)?;
                Ok(MiraValue::from_module_handle(MiraHandle::new(key)))
            }
        }
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

/// Functions for working with string handles.
impl Runtime {
    /// Insert an owned string and return its typed handle.
    pub fn insert_string(&mut self, value: impl Into<String>) -> Result<MiraHandle<String>> {
        let key = self.arena.strings.insert(self.arena.id, value.into())?;
        Ok(MiraHandle::new(key))
    }

    /// Read a string handle owned by this Runtime.
    pub fn get_string(&self, handle: MiraHandle<String>) -> Result<&str> {
        self.arena
            .strings
            .get(self.arena.id, handle.key)
            .map(Deref::deref)
    }

    /// Mutably read a string handle owned by this Runtime.
    pub fn get_string_mut(&mut self, handle: MiraHandle<String>) -> Result<&mut String> {
        self.arena.strings.get_mut(self.arena.id, handle.key)
    }

    /// Take a string handle owned by this Runtime, consuming the handle.
    pub fn take_string(&mut self, handle: MiraHandle<String>) -> Result<String> {
        self.arena.strings.take(self.arena.id, handle.key)
    }
}

/// Functions for working with function handles.
impl Runtime {
    /// Insert a function and return a concrete typed handle.
    pub fn insert_function<T: MiraFunction>(&mut self, value: T) -> Result<MiraHandle<T>> {
        let key = self.arena.functions.insert(self.arena.id, Rc::new(value))?;
        Ok(MiraHandle::new(key))
    }

    pub(crate) fn get_function_dyn(
        &self,
        handle: MiraHandle<dyn MiraFunction>,
    ) -> Result<Rc<dyn MiraFunction>> {
        self.arena
            .functions
            .get(self.arena.id, handle.key)
            .map(Rc::clone)
    }

    /// Read the concrete function stored behind a typed handle.
    pub fn get_function<T: MiraFunction>(&self, handle: MiraHandle<T>) -> Result<&T> {
        let value = self.arena.functions.get(self.arena.id, handle.key)?;
        let value: &dyn Any = value.as_ref();
        value.downcast_ref::<T>().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch {
                category: "function",
            })
        })
    }

    /// Take the concrete function stored behind a typed handle, consuming the handle.
    pub fn take_function<T: MiraFunction>(&mut self, handle: MiraHandle<T>) -> Result<Rc<T>> {
        let value = self.arena.functions.take(self.arena.id, handle.key)?;
        let value: Rc<dyn Any> = value;
        value.downcast::<T>().map_err(|_| {
            MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch {
                category: "function",
            })
        })
    }
}

macro_rules! impl_typed_functions {
    ($field:ident, $name:ident, $trait:ident) => { paste::paste! {
        #[doc = concat!("Functions for working with ", stringify!($name), " handles.")]
        impl Runtime {
            #[doc = concat!("Insert a ", stringify!($name), " and return a concrete typed handle.")]
            pub fn [<insert_ $name>]<T: $trait>(&mut self, value: T) -> Result<MiraHandle<T>> {
                let key = self.arena.$field.insert(self.arena.id, Box::new(value))?;
                Ok(MiraHandle::new(key))
            }

            pub(crate) fn [<get_ $name _dyn>](
                &self,
                handle: MiraHandle<dyn $trait>,
            ) -> Result<&dyn $trait> {
                self.arena
                    .$field
                    .get(self.arena.id, handle.key)
                    .map(|value| value.as_ref())
            }

            #[doc = concat!("Read the concrete target represented by a typed ", stringify!($name), " handle.")]
            pub fn [<get_ $name>]<T: $trait>(&self, handle: MiraHandle<T>) -> Result<&T> {
                let value = self.arena.$field.get(self.arena.id, handle.key)?;
                let value: &dyn Any = value.resolve(self)?.unwrap_or(value.as_ref());
                value.downcast_ref::<T>().ok_or_else(|| {
                    MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch { category: stringify!($name) })
                })
            }

            #[doc = concat!("Mutably read the concrete object stored directly behind a ", stringify!($name), " handle.")]
            pub fn [<get_ $name _mut>]<T: $trait>(&mut self, handle: MiraHandle<T>) -> Result<&mut T> {
                let value = self.arena.$field.get_mut(self.arena.id, handle.key)?;
                let value: &mut dyn Any = value.as_mut();
                value.downcast_mut::<T>().ok_or_else(|| {
                    MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch { category: stringify!($name) })
                })
            }

            #[doc = concat!("Take the concrete object stored directly behind a ", stringify!($name), " handle, consuming the handle.")]
            pub fn [<take_ $name>]<T: $trait>(&mut self, handle: MiraHandle<T>) -> Result<T> {
                let value = self.arena.$field.take(self.arena.id, handle.key)?;
                let value: Box<dyn Any> = value;
                value.downcast::<T>().map(|boxed| *boxed).map_err(|_| {
                    MiraError::runtime(RuntimeErrorKind::HandleTypeMismatch { category: stringify!($name) })
                })
            }
        }
    } };
}

impl_typed_functions!(arrays, array, MiraArray);
impl_typed_functions!(records, record, MiraRecord);
impl_typed_functions!(modules, module, MiraModule);
