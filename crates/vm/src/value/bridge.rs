use std::any::type_name;

use crate::{MiraError, Result};

use super::{MiraAny, MiraCallContext, MiraShared};

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

impl<T: MiraRecord> RecordObject for MiraShared<T> {
    fn identity(&self) -> usize {
        self.identity()
    }

    fn tag(&self) -> &'static str {
        type_name::<T>()
    }

    fn keys(&self) -> Result<Vec<String>> {
        self.inner
            .try_borrow()
            .map(|value| value.keys())
            .map_err(|_| MiraError::BorrowConflict {
                operation: "read",
                tag: type_name::<T>().into(),
            })
    }

    fn get(&self, key: &str) -> Result<Option<MiraAny>> {
        self.inner
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

impl<T: MiraArray> ArrayObject for MiraShared<T> {
    fn identity(&self) -> usize {
        self.identity()
    }

    fn tag(&self) -> &'static str {
        type_name::<T>()
    }

    fn len(&self) -> Result<usize> {
        self.inner
            .try_borrow()
            .map(|value| value.len())
            .map_err(|_| MiraError::BorrowConflict {
                operation: "read",
                tag: type_name::<T>().into(),
            })
    }

    fn get(&self, index: usize) -> Result<Option<MiraAny>> {
        self.inner
            .try_borrow()
            .map_err(|_| MiraError::BorrowConflict {
                operation: "read",
                tag: type_name::<T>().into(),
            })?
            .get(index)
    }
}

#[doc(hidden)]
pub trait ExternObject {
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

impl<T: MiraExtern> ExternObject for MiraShared<T> {
    fn identity(&self) -> usize {
        self.identity()
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

impl<T: MiraExtern> MiraShared<T> {
    fn try_read<R>(&self, operation: &'static str, f: impl FnOnce(&T) -> Result<R>) -> Result<R> {
        let value = self
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
        let mut value = self
            .inner
            .try_borrow_mut()
            .map_err(|_| MiraError::BorrowConflict {
                operation,
                tag: type_name::<T>().into(),
            })?;
        f(&mut value)
    }
}
