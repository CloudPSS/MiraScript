use std::{iter::FusedIterator, ops::Range};

use crate::{MiraArrayHandle, MiraError, MiraValue, Result, Runtime, RuntimeErrorKind};

#[derive(Clone, Copy)]
pub(crate) struct ArrayEntry {
    handle: MiraArrayHandle,
    index: usize,
}

impl ArrayEntry {
    pub(crate) const fn index(self) -> usize {
        self.index
    }

    pub(crate) fn get(self, runtime: &mut Runtime) -> Result<MiraValue> {
        self.handle.get(runtime, self.index)
    }
}

#[derive(Clone)]
pub(crate) struct ArrayIter {
    handle: MiraArrayHandle,
    indices: Range<usize>,
}

impl Iterator for ArrayIter {
    type Item = ArrayEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|index| ArrayEntry {
            handle: self.handle,
            index,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl DoubleEndedIterator for ArrayIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.indices.next_back().map(|index| ArrayEntry {
            handle: self.handle,
            index,
        })
    }
}

impl ExactSizeIterator for ArrayIter {}
impl FusedIterator for ArrayIter {}

pub(crate) fn iterate_array(runtime: &Runtime, value: MiraValue) -> Result<ArrayIter> {
    let Some(handle) = value.as_array() else {
        return Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "array",
            actual: value.value_type(),
        }));
    };
    let length = handle.len(runtime)?;
    Ok(ArrayIter {
        handle,
        indices: 0..length,
    })
}

pub(crate) fn iterable_array(runtime: &mut Runtime, value: MiraValue) -> Result<Vec<MiraValue>> {
    iterate_array(runtime, value)?
        .map(|entry| entry.get(runtime))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use crate::{MiraArray, MiraManageable};

    use super::*;

    struct CountingArray {
        reads: Rc<Cell<usize>>,
    }

    impl MiraArray for CountingArray {
        fn len(&self) -> usize {
            4
        }

        fn get(
            &self,
            _self_handle: MiraArrayHandle,
            _runtime: &Runtime,
            index: usize,
        ) -> Result<MiraManageable> {
            self.reads.set(self.reads.get() + 1);
            Ok(MiraValue::number(index as f64).into())
        }
    }

    #[test]
    fn iterator_does_not_borrow_runtime_and_only_resolves_visited_entries() {
        let reads = Rc::new(Cell::new(0));
        let mut runtime = Runtime::new();
        let value = runtime
            .insert(MiraManageable::from_array(CountingArray {
                reads: reads.clone(),
            }))
            .unwrap();

        let iter = iterate_array(&runtime, value).unwrap();
        assert_eq!(iter.len(), 4);
        let visited = iter
            .take(2)
            .map(|entry| {
                let index = entry.index();
                let value = entry.get(&mut runtime)?.as_number_unchecked();
                Ok((index, value))
            })
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert_eq!(visited, vec![(0, 0.0), (1, 1.0)]);
        assert_eq!(reads.get(), 2);
    }
}
