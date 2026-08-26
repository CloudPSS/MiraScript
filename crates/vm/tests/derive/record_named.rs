use mirascript_vm::{MiraError, MiraRecord, MiraValue, Runtime, RuntimeErrorKind};

#[derive(Clone, MiraRecord)]
#[mira(crate = mirascript_vm)]
struct Record<T> {
    key: String,
    #[mira(rename = "value")]
    item: T,
    #[mira(skip)]
    hidden: bool,
}

#[test]
fn main() {
    let mut runtime = Runtime::new();
    let record = runtime
        .insert_record(Record {
            key: "example".to_string(),
            item: 1_u8,
            hidden: true,
        })
        .unwrap();
    runtime
        .insert_global("record", MiraValue::record(record))
        .unwrap();
    assert_eq!(
        runtime
            .eval("record.key")
            .unwrap()
            .as_str(&runtime)
            .unwrap()
            .unwrap(),
        "example"
    );
    assert_eq!(
        runtime.eval_unchecked("record.value").as_number_unchecked(),
        1f64
    );
    assert!(
        !runtime
            .eval_unchecked("'hidden' in record")
            .as_boolean_unchecked(),
    );
    assert!(
        !runtime
            .eval_unchecked("'item' in record")
            .as_boolean_unchecked(),
    );
    let record = runtime.take_record(record).unwrap();
    assert_eq!(record.key, "example");
    assert_eq!(record.item, 1_u8);
    assert!(record.hidden);

    assert!(matches!(
        runtime.eval("record.key").unwrap_err().as_ref(),
        MiraError::Runtime {
            kind: RuntimeErrorKind::InvalidHandle { .. },
            ..
        }
    ));
}
