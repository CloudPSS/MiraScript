use std::{iter::FusedIterator, ops::Range, vec};

use crate::{
    MiraArrayHandle, MiraError, MiraManageable, MiraValue, Result, Runtime, RuntimeErrorKind,
};

enum ArrayEntryValue {
    Indexed(MiraArrayHandle),
    Iterated(Result<MiraManageable>),
}

pub(crate) struct ArrayEntry {
    index: usize,
    value: ArrayEntryValue,
}

impl ArrayEntry {
    pub(crate) const fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn get(self, runtime: &mut Runtime) -> Result<MiraValue> {
        match self.value {
            ArrayEntryValue::Indexed(handle) => handle.get(runtime, self.index),
            ArrayEntryValue::Iterated(value) => runtime.insert(value?),
        }
    }
}

pub(crate) struct ArrayIter {
    handle: MiraArrayHandle,
    indices: Range<usize>,
    entries: Option<vec::IntoIter<ArrayEntry>>,
}

impl Iterator for ArrayIter {
    type Item = ArrayEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.entries {
            Some(entries) => entries.next(),
            None => self.indices.next().map(|index| ArrayEntry {
                index,
                value: ArrayEntryValue::Indexed(self.handle),
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.entries {
            Some(entries) => entries.size_hint(),
            None => self.indices.size_hint(),
        }
    }
}

impl DoubleEndedIterator for ArrayIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.entries {
            Some(entries) => entries.next_back(),
            None => self.indices.next_back().map(|index| ArrayEntry {
                index,
                value: ArrayEntryValue::Indexed(self.handle),
            }),
        }
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
    let (length, entries) = {
        let array = runtime.get_array_dyn(handle)?;
        let length = array.len();
        let entries = array.iter(handle, runtime).map(|iter| {
            iter.enumerate()
                .map(|(index, entry)| match entry {
                    Ok(entry) => ArrayEntry {
                        index: entry.index(),
                        value: ArrayEntryValue::Iterated(Ok(entry.into_value())),
                    },
                    Err(error) => ArrayEntry {
                        index,
                        value: ArrayEntryValue::Iterated(Err(error)),
                    },
                })
                .collect::<Vec<_>>()
                .into_iter()
        });
        (length, entries)
    };
    Ok(ArrayIter {
        handle,
        indices: 0..length,
        entries,
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

    use crate::{MiraArray, MiraArrayEntry, MiraArrayIter, MiraManageable};

    use super::*;

    struct CountingArray {
        reads: Rc<Cell<usize>>,
    }

    struct SequentialArray {
        reads: Rc<Cell<usize>>,
    }

    impl MiraArray for SequentialArray {
        fn len(&self) -> usize {
            2
        }

        fn get(
            &self,
            _self_handle: MiraArrayHandle,
            _runtime: &Runtime,
            _index: usize,
        ) -> Result<MiraManageable> {
            panic!("indexed value access should not be used")
        }

        fn iter<'a>(
            &'a self,
            _self_handle: MiraArrayHandle,
            _runtime: &'a Runtime,
        ) -> Option<MiraArrayIter<'a>> {
            let reads = Rc::clone(&self.reads);
            Some(Box::new((0..2).map(move |index| {
                reads.set(reads.get() + 1);
                Ok(MiraArrayEntry::new(
                    index,
                    MiraValue::number(index as f64).into(),
                ))
            })))
        }
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

    #[test]
    fn iterate_array_uses_the_trait_iterator_when_available() {
        let reads = Rc::new(Cell::new(0));
        let mut runtime = Runtime::new();
        let value = runtime
            .insert(MiraManageable::from_array(SequentialArray {
                reads: reads.clone(),
            }))
            .unwrap();

        let values = iterate_array(&runtime, value)
            .unwrap()
            .map(|entry| entry.get(&mut runtime))
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert_eq!(
            values
                .into_iter()
                .map(|value| value.as_number_unchecked())
                .collect::<Vec<_>>(),
            [0.0, 1.0]
        );
        assert_eq!(reads.get(), 2);
    }
}
