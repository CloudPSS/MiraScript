use paste::paste;
use std::{any::Any, ops::Deref};

use crate::interpreter::Runtime;

use super::{
    MiraValue,
    types::{MiraArray, MiraFunction, MiraModule, MiraRecord},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ArenaKey(usize);

#[derive(Debug)]
struct Arena<T> {
    vec: Vec<Option<T>>,
}

impl<T> Arena<T> {
    fn new() -> Self {
        Self { vec: Vec::new() }
    }

    fn insert(&mut self, value: T) -> ArenaKey {
        let key = ArenaKey(self.vec.len());
        self.vec.push(Some(value));
        key
    }

    fn remove(&mut self, key: ArenaKey) -> Option<T> {
        self.vec.get_mut(key.0).and_then(|slot| slot.take())
    }

    fn get(&self, key: ArenaKey) -> Option<&T> {
        self.vec.get(key.0).and_then(|slot| slot.as_ref())
    }
}

pub struct MiraHandle<T: Any + ?Sized> {
    key: ArenaKey,
    _marker: std::marker::PhantomData<&'static T>,
}

pub enum MiraManageable {
    Value(MiraValue),
    String(String),
    Array(Box<dyn MiraArray>),
    Record(Box<dyn MiraRecord>),
    Function(Box<dyn MiraFunction>),
    Module(Box<dyn MiraModule>),
}

impl Into<MiraManageable> for MiraValue {
    fn into(self) -> MiraManageable {
        MiraManageable::Value(self)
    }
}

impl<T: Any + ?Sized> Copy for MiraHandle<T> {}
impl<T: Any + ?Sized> Clone for MiraHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Any + ?Sized> std::fmt::Debug for MiraHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MiraHandle").field(&self.key.0).finish()
    }
}

impl<T: Any + ?Sized> PartialEq for MiraHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

pub(crate) struct MiraArena {
    strings: Arena<String>,
    arrays: Arena<Box<dyn MiraArray>>,
    records: Arena<Box<dyn MiraRecord>>,
    functions: Arena<Box<dyn MiraFunction>>,
    modules: Arena<Box<dyn MiraModule>>,
}

impl MiraArena {
    pub fn new() -> Self {
        Self {
            strings: Arena::new(),
            arrays: Arena::new(),
            records: Arena::new(),
            functions: Arena::new(),
            modules: Arena::new(),
        }
    }
}

fn insert_boxed<T: ?Sized + Any + 'static>(
    arena: &mut Arena<Box<T>>,
    value: Box<T>,
) -> MiraHandle<T> {
    let key = arena.insert(value);
    MiraHandle {
        key,
        _marker: std::marker::PhantomData::<&T>,
    }
}

impl From<String> for MiraManageable {
    fn from(value: String) -> Self {
        MiraManageable::String(value)
    }
}
impl Runtime<'_> {
    pub fn insert(&mut self, value: MiraManageable) -> MiraValue {
        match value {
            MiraManageable::Value(value) => value,
            MiraManageable::String(value) => MiraValue::new_string(value, self),
            MiraManageable::Array(value) => {
                MiraValue::Array(insert_boxed(&mut self.arena.arrays, value))
            }
            MiraManageable::Record(value) => {
                MiraValue::Record(insert_boxed(&mut self.arena.records, value))
            }
            MiraManageable::Function(value) => {
                MiraValue::Function(insert_boxed(&mut self.arena.functions, value))
            }
            MiraManageable::Module(value) => {
                MiraValue::Module(insert_boxed(&mut self.arena.modules, value))
            }
        }
    }

    pub fn insert_string(&mut self, value: impl Into<String>) -> MiraHandle<String> {
        let key = self.arena.strings.insert(value.into());
        MiraHandle {
            key,
            _marker: std::marker::PhantomData::<&String>,
        }
    }
    pub fn take_string(&mut self, handle: MiraHandle<String>) -> Option<String> {
        self.arena.strings.remove(handle.key)
    }

    pub fn get_string(&self, handle: MiraHandle<String>) -> Option<&str> {
        self.arena.strings.get(handle.key).map(Deref::deref)
    }
}

macro_rules! impl_arena_handle {
    ($($ty:ty: [$field:ident, $Field:ident]),* $(,)?) => {$ (paste! {
impl<T: $ty> From<MiraHandle<T>> for MiraHandle<dyn $ty> {
    fn from(handle: MiraHandle<T>) -> Self {
        MiraHandle {
            key: handle.key,
            _marker: std::marker::PhantomData::<&dyn $ty>,
        }
    }
}
impl MiraHandle<dyn $ty> {
    /// Casts the handle to a specific type. This is unsafe because it does not check if the underlying value is actually of type `T`.
    ///
    /// # Safety
    /// The caller must ensure that the underlying value is of type `T`. If it is not, using the returned handle may lead to undefined behavior.
    pub unsafe fn upcast<T: $ty>(self) -> MiraHandle<T> {
        MiraHandle {
            key: self.key,
            _marker: std::marker::PhantomData::<&T>,
        }
    }
}

impl MiraManageable {
    pub fn [<from_ $field>](value: impl $ty) -> Self {
        MiraManageable::$Field(Box::new(value))
    }
}

impl Runtime<'_> {
    pub fn [<insert_ $field>]<T: $ty>(&mut self, value: T) -> MiraHandle<T> {
        let key = self.arena.[<$field s>].insert(Box::new(value));
        MiraHandle {
            key,
            _marker: std::marker::PhantomData::<&T>,
        }
    }

    pub fn [<take_ $field>]<T: $ty>(&mut self, handle: MiraHandle<T>) -> Option<T> {
        let value = self.arena.[<$field s>].remove(handle.key)?;
        let value: Box<dyn Any> = value;
        value.downcast::<T>().ok().map(|boxed| *boxed)
    }

    pub fn [<get_ $field>]<T: $ty>(&self, handle: MiraHandle<T>) -> Option<&T> {
        let value = self.arena.[<$field s>].get(handle.key)?;
        let value: &dyn Any = value.as_ref();
        value.downcast_ref::<T>()
    }
}
        })* };
}

impl_arena_handle! {
    MiraArray: [array, Array],
    MiraRecord: [record, Record],
    MiraFunction: [function, Function],
    MiraModule: [module, Module],
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::MiraHandle;

    impl<T: Any + ?Sized> MiraHandle<T> {
        pub fn empty() -> Self {
            MiraHandle {
                key: super::ArenaKey(0),
                _marker: std::marker::PhantomData,
            }
        }
    }
}
