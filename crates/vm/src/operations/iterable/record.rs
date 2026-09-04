use std::{iter::FusedIterator, ops::Range, vec};

use indexmap::IndexMap;

use crate::{
    MiraError, MiraManageable, MiraRecordHandle, MiraValue, Result, Runtime, RuntimeErrorKind,
};

enum RecordEntryValue {
    Indexed(MiraRecordHandle),
    Iterated { key: String, value: MiraManageable },
}

pub(crate) struct RecordEntry {
    index: usize,
    value: RecordEntryValue,
}

impl RecordEntry {
    pub(crate) fn key<'a>(&'a self, runtime: &'a Runtime) -> Result<&'a str> {
        match &self.value {
            RecordEntryValue::Indexed(handle) => handle.key(runtime, self.index),
            RecordEntryValue::Iterated { key, .. } => Ok(key),
        }
    }

    pub(crate) fn get(self, runtime: &mut Runtime) -> Result<MiraValue> {
        match self.value {
            RecordEntryValue::Indexed(handle) => handle.get(runtime, self.index),
            RecordEntryValue::Iterated { value, .. } => runtime.insert(value),
        }
    }

    fn into_pair(self, runtime: &mut Runtime) -> Result<(String, MiraValue)> {
        match self.value {
            RecordEntryValue::Indexed(handle) => {
                let key = handle.key(runtime, self.index)?.to_owned();
                let value = handle.get(runtime, self.index)?;
                Ok((key, value))
            }
            RecordEntryValue::Iterated { key, value } => Ok((key, runtime.insert(value)?)),
        }
    }
}

pub(crate) struct RecordIter {
    handle: MiraRecordHandle,
    indices: Range<usize>,
    entries: Option<vec::IntoIter<RecordEntry>>,
}

impl Iterator for RecordIter {
    type Item = RecordEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.entries {
            Some(entries) => entries.next(),
            None => self.indices.next().map(|index| RecordEntry {
                index,
                value: RecordEntryValue::Indexed(self.handle),
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

impl DoubleEndedIterator for RecordIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.entries {
            Some(entries) => entries.next_back(),
            None => self.indices.next_back().map(|index| RecordEntry {
                index,
                value: RecordEntryValue::Indexed(self.handle),
            }),
        }
    }
}

impl ExactSizeIterator for RecordIter {}
impl FusedIterator for RecordIter {}

pub(crate) fn iterate_record(runtime: &Runtime, value: MiraValue) -> Result<RecordIter> {
    let Some(handle) = value.as_record() else {
        return Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "record",
            actual: value.value_type(),
        }));
    };
    let (length, entries) = {
        let record = runtime.get_record_dyn(handle)?;
        let length = record.len();
        let entries = record
            .iter(handle, runtime)
            .map(|iter| {
                iter.map(|entry| {
                    entry.map(|entry| RecordEntry {
                        index: entry.index(),
                        value: RecordEntryValue::Iterated {
                            key: entry.key().to_owned(),
                            value: entry.into_value(),
                        },
                    })
                })
                .collect::<Result<Vec<_>>>()
                .map(Vec::into_iter)
            })
            .transpose()?;
        (length, entries)
    };
    Ok(RecordIter {
        handle,
        indices: 0..length,
        entries,
    })
}

pub(crate) fn iterable_record(
    runtime: &mut Runtime,
    value: MiraValue,
) -> Result<IndexMap<String, MiraValue>> {
    let iter = iterate_record(runtime, value)?;
    let mut entries = IndexMap::with_capacity(iter.len());
    for entry in iter {
        let (key, value) = entry.into_pair(runtime)?;
        entries.insert(key, value);
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use crate::{MiraHandle, MiraManageable, MiraRecord, MiraRecordEntry, MiraRecordIter};

    use super::*;

    struct CountingRecord {
        keys: Rc<Cell<usize>>,
        reads: Rc<Cell<usize>>,
    }

    struct SequentialRecord {
        reads: Rc<Cell<usize>>,
    }

    impl MiraRecord for SequentialRecord {
        fn len(&self) -> usize {
            2
        }

        fn index_of(&self, _key: &str) -> Option<usize> {
            None
        }

        fn key(&self, _index: usize) -> Result<&str> {
            panic!("indexed key access should not be used")
        }

        fn get(
            &self,
            _self_handle: MiraHandle<dyn MiraRecord>,
            _runtime: &Runtime,
            _index: usize,
        ) -> Result<MiraManageable> {
            panic!("indexed value access should not be used")
        }

        fn iter<'a>(
            &'a self,
            _self_handle: MiraHandle<dyn MiraRecord>,
            _runtime: &'a Runtime,
        ) -> Option<MiraRecordIter<'a>> {
            let reads = Rc::clone(&self.reads);
            Some(Box::new(["a", "b"].into_iter().enumerate().map(
                move |(index, key)| {
                    reads.set(reads.get() + 1);
                    Ok(MiraRecordEntry::new(
                        index,
                        key,
                        MiraValue::number(index as f64).into(),
                    ))
                },
            )))
        }
    }

    impl MiraRecord for CountingRecord {
        fn len(&self) -> usize {
            4
        }

        fn index_of(&self, key: &str) -> Option<usize> {
            ["a", "b", "c", "d"]
                .iter()
                .position(|candidate| *candidate == key)
        }

        fn key(&self, index: usize) -> Result<&str> {
            self.keys.set(self.keys.get() + 1);
            ["a", "b", "c", "d"]
                .get(index)
                .copied()
                .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
        }

        fn get(
            &self,
            _self_handle: MiraHandle<dyn MiraRecord>,
            _runtime: &Runtime,
            index: usize,
        ) -> Result<MiraManageable> {
            self.reads.set(self.reads.get() + 1);
            (index < 4)
                .then(|| MiraValue::number(index as f64).into())
                .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::MissingIndexOrField))
        }
    }

    #[test]
    fn iterator_does_not_borrow_runtime_and_only_resolves_visited_entries() {
        let keys = Rc::new(Cell::new(0));
        let reads = Rc::new(Cell::new(0));
        let mut runtime = Runtime::new();
        let value = runtime
            .insert(MiraManageable::from_record(CountingRecord {
                keys: keys.clone(),
                reads: reads.clone(),
            }))
            .unwrap();

        let iter = iterate_record(&runtime, value).unwrap();
        assert_eq!(iter.len(), 4);
        let visited = iter
            .take(2)
            .enumerate()
            .map(|(index, entry)| {
                let key = entry.key(&runtime)?.to_owned();
                let value = entry.get(&mut runtime)?.as_number_unchecked();
                Ok((index, key, value))
            })
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert_eq!(visited, vec![(0, "a".into(), 0.0), (1, "b".into(), 1.0)]);
        assert_eq!(keys.get(), 2);
        assert_eq!(reads.get(), 2);
    }

    #[test]
    fn iterate_record_uses_the_trait_iterator_when_available() {
        let reads = Rc::new(Cell::new(0));
        let mut runtime = Runtime::new();
        let value = runtime
            .insert(MiraManageable::from_record(SequentialRecord {
                reads: reads.clone(),
            }))
            .unwrap();

        let iter = iterate_record(&runtime, value).unwrap();
        let mut entries = IndexMap::with_capacity(iter.len());
        for entry in iter {
            let key = entry.key(&runtime).unwrap().to_owned();
            let value = entry.get(&mut runtime).unwrap();
            entries.insert(key, value);
        }

        assert_eq!(
            entries.keys().map(String::as_str).collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(
            entries
                .values()
                .map(|value| value.as_number_unchecked())
                .collect::<Vec<_>>(),
            [0.0, 1.0]
        );
        assert_eq!(reads.get(), 2);
    }
}
