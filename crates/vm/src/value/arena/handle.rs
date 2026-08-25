use std::{any::Any, fmt, hash, marker::PhantomData};

use super::{ArenaKey, MiraArray, MiraFunction, MiraModule, MiraRecord};

/// A compact, runtime-checked handle to an arena-managed value.
pub struct MiraHandle<T: Any + ?Sized> {
    pub(super) key: ArenaKey,
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

impl<T: Any + ?Sized> hash::Hash for MiraHandle<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl<T: Any + ?Sized> MiraHandle<T> {
    pub(super) const fn new(key: ArenaKey) -> Self {
        Self {
            key,
            marker: PhantomData,
        }
    }

    pub(crate) const fn payload(self) -> [u8; 6] {
        self.key.payload()
    }

    pub(crate) const fn from_payload(payload: [u8; 6]) -> Self {
        Self::new(ArenaKey::from_payload(payload))
    }
}

macro_rules! impl_handle_cast {
    ($trait:path, $erase:ident) => {
        impl<T: $trait + ?Sized> MiraHandle<T> {
            /// Erase the concrete Rust type while preserving its MiraScript category.
            pub const fn $erase(self) -> MiraHandle<dyn $trait> {
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
