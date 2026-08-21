use mirascript_vm::{MiraRecord, Runtime};

#[derive(Clone, MiraRecord)]
#[mira(crate = mirascript_vm)]
struct Record<T> {
    key: String,
    items: Vec<T>,
}

#[test]
fn test_complex() {
    let mut runtime = Runtime::new();

    let num_record = Record {
        key: "numbers".to_string(),
        items: vec![1, 2, 3],
    };
    let record_record = Record {
        key: "records".to_string(),
        items: vec![num_record.clone(), num_record.clone()],
    };

    runtime.insert_global("num_record", num_record).unwrap();
    runtime
        .insert_global("record_record", record_record)
        .unwrap();

    let result = runtime
        .eval(
            r#"
            record_record::to_json()
            "#,
        )
        .unwrap()
        .as_str(&runtime)
        .unwrap()
        .unwrap();
    assert_eq!(
        result,
        r#"{"key":"records","items":[{"key":"numbers","items":[1,2,3]},{"key":"numbers","items":[1,2,3]}]}"#
    );
}
