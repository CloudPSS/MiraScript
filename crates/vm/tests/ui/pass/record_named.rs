use mirascript_vm::{MiraRecord, Runtime};

#[derive(Clone, MiraRecord)]
#[mira(crate = mirascript_vm)]
struct Record<T> {
    key: String,
    #[mira(rename = "value")]
    item: T,
    #[mira(skip)]
    hidden: bool,
}

fn main() {
    let mut runtime = Runtime::new();
    let record = runtime
        .insert(Record {
            key: "example".to_string(),
            item: 1_u8,
            hidden: true,
        })
        .unwrap();
    runtime.insert_global("record", record).unwrap();
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
        runtime.eval("record.value").unwrap().as_number().unwrap(),
        1f64
    );
    assert!(
        !runtime
            .eval("'hidden' in record")
            .unwrap()
            .as_boolean()
            .unwrap(),
    );
    assert!(
        !runtime
            .eval("'item' in record")
            .unwrap()
            .as_boolean()
            .unwrap(),
    );
}
