use mirascript_vm::{MiraRecord, Runtime};

#[derive(Clone, MiraRecord)]
#[mira(crate = ::mirascript_vm)]
struct Record<T>(#[mira(rename = "new")] String, #[mira(skip)] (), T);

#[test]
fn main() {
    let mut runtime = Runtime::new();
    let record = runtime
        .insert(Record("example".to_string(), (), 1_u8))
        .unwrap();
    runtime.insert_global("record", record).unwrap();
    assert_eq!(
        runtime
            .eval("record.new")
            .unwrap()
            .as_str(&runtime)
            .unwrap()
            .unwrap(),
        "example"
    );
    assert_eq!(
        runtime.eval_unchecked("record.2").as_number_unchecked(),
        1f64
    );

    assert!(runtime.eval_unchecked("record.1").is_nil());
    assert!(
        !runtime
            .eval_unchecked("'1' in record")
            .as_boolean_unchecked(),
    );

    assert!(runtime.eval_unchecked("record.0").is_nil());
    assert!(
        !runtime
            .eval_unchecked("'0' in record")
            .as_boolean_unchecked(),
    );
}
