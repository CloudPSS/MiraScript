#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

extern crate self as mirascript;

/// The complete MiraScript virtual-machine API.
pub use mirascript_vm as vm;
pub use mirascript_vm::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(MiraRecord)]
    struct TestRecord {
        a: i32,
        b: String,
        #[mira(rename = "c")]
        foo: Vec<u8>,
        #[mira(skip)]
        bar: Option<bool>,
    }

    #[test]
    fn test() {
        let mut runtime = Runtime::new();
        let record = TestRecord {
            a: 42,
            b: "Hello".to_string(),
            foo: vec![1, 2, 3],
            bar: Some(true),
        };
        let record_handle = runtime.insert_record(record).unwrap();
        runtime
            .insert_global("test", MiraValue::record(record_handle))
            .unwrap();
        assert_eq!(
            runtime
                .eval("'$(test.a + test.c[0])$(test.b)'")
                .unwrap()
                .as_str(&runtime)
                .unwrap(),
            Some("43Hello")
        );
        let record = runtime.get_record(record_handle).unwrap();
        assert_eq!(record.a, 42);
        assert_eq!(record.b, "Hello");
        assert_eq!(record.foo, vec![1, 2, 3]);
        assert_eq!(record.bar, Some(true));

        runtime.take_record(record_handle).unwrap();
        assert!(runtime.get_record(record_handle).is_err());
        assert!(runtime.get_global("test").is_some());
    }
}
