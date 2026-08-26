use crate::{MiraArray, MiraHandle, MiraManageable, MiraRecord, MiraValue};

use super::MiraField;

impl<T: Into<MiraValue> + Copy + 'static> MiraField for T {
    fn from_record<P: MiraRecord>(
        &self,
        _parent: MiraHandle<P>,
        _index: usize,
        _getter: fn(&P, usize) -> &Self,
    ) -> MiraManageable {
        MiraManageable::Value((*self).into())
    }

    fn from_array<P: MiraArray>(
        &self,
        _parent: MiraHandle<P>,
        _index: usize,
        _getter: fn(&P, usize) -> &Self,
    ) -> MiraManageable {
        MiraManageable::Value((*self).into())
    }
}

impl MiraField for String {
    fn from_record<P: MiraRecord>(
        &self,
        _parent: MiraHandle<P>,
        _index: usize,
        _getter: fn(&P, usize) -> &Self,
    ) -> MiraManageable {
        self.as_str().into()
    }

    fn from_array<P: MiraArray>(
        &self,
        _parent: MiraHandle<P>,
        _index: usize,
        _getter: fn(&P, usize) -> &Self,
    ) -> MiraManageable {
        self.as_str().into()
    }
}

impl MiraField for &'static str {
    fn from_record<P: MiraRecord>(
        &self,
        _parent: MiraHandle<P>,
        _index: usize,
        _getter: fn(&P, usize) -> &Self,
    ) -> MiraManageable {
        (*self).into()
    }

    fn from_array<P: MiraArray>(
        &self,
        _parent: MiraHandle<P>,
        _index: usize,
        _getter: fn(&P, usize) -> &Self,
    ) -> MiraManageable {
        (*self).into()
    }
}
