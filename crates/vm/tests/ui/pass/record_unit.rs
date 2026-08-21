use mirascript_vm::{MiraRecord, Runtime};

#[derive(Clone, MiraRecord)]
#[mira(crate = mirascript_vm)]
struct Record;

fn main() {
    let mut runtime = Runtime::new();
    let record = Record;
    assert!(record.is_empty());
    let record = runtime.insert(record).unwrap();
    runtime.insert_global("record", record).unwrap();
    assert_eq!(
        runtime
            .eval("record::to_json()")
            .unwrap()
            .as_str(&runtime)
            .unwrap()
            .unwrap(),
        "{}"
    );

    assert!(runtime.eval("record.anything").unwrap().is_nil());
    assert!(
        !runtime
            .eval("'anything' in record")
            .unwrap()
            .as_boolean()
            .unwrap(),
    );
}
