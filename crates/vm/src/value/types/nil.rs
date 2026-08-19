use super::MiraValue;
pub use MiraValue::Nil;

impl<T: Into<MiraValue> + Clone + 'static> From<Option<T>> for MiraValue {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Nil,
        }
    }
}
