use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::rc::Rc;

/// Shared ownership used when the host needs to retain and mutate a value.
pub struct MiraShared<T> {
    pub(super) inner: Rc<RefCell<T>>,
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

    pub(super) fn identity(&self) -> usize {
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
