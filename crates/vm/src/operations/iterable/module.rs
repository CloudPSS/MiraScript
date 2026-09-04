use std::{iter::FusedIterator, ops::Range};

use crate::{MiraError, MiraModuleHandle, MiraValue, Result, Runtime, RuntimeErrorKind};

#[derive(Clone, Copy)]
pub(crate) struct ModuleEntry {
    handle: MiraModuleHandle,
    index: usize,
}

impl ModuleEntry {
    pub(crate) fn key(self, runtime: &Runtime) -> Result<&str> {
        self.handle.key(runtime, self.index)
    }

    pub(crate) fn get(self, runtime: &mut Runtime) -> Result<MiraValue> {
        self.handle.get(runtime, self.index)
    }
}

#[derive(Clone)]
pub(crate) struct ModuleIter {
    handle: MiraModuleHandle,
    indices: Range<usize>,
}

impl Iterator for ModuleIter {
    type Item = ModuleEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|index| ModuleEntry {
            handle: self.handle,
            index,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl DoubleEndedIterator for ModuleIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.indices.next_back().map(|index| ModuleEntry {
            handle: self.handle,
            index,
        })
    }
}

impl ExactSizeIterator for ModuleIter {}
impl FusedIterator for ModuleIter {}

pub(crate) fn iterate_module(runtime: &Runtime, value: MiraValue) -> Result<ModuleIter> {
    let Some(handle) = value.as_module() else {
        return Err(MiraError::runtime(RuntimeErrorKind::TypeMismatch {
            expected: "module",
            actual: value.value_type(),
        }));
    };
    let length = handle.len(runtime)?;
    Ok(ModuleIter {
        handle,
        indices: 0..length,
    })
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use crate::{MiraHandle, MiraManageable, MiraModule};

    use super::*;

    struct CountingModule {
        keys: Rc<Cell<usize>>,
        reads: Rc<Cell<usize>>,
    }

    impl MiraModule for CountingModule {
        fn name(&self) -> &str {
            "counting"
        }

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
            _self_handle: MiraHandle<dyn MiraModule>,
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
            .insert(MiraManageable::from_module(CountingModule {
                keys: keys.clone(),
                reads: reads.clone(),
            }))
            .unwrap();

        let iter = iterate_module(&runtime, value).unwrap();
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
}
