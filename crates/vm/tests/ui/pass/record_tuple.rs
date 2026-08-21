use mirascript_vm::{MiraRecord, Runtime};

#[derive(Clone, MiraRecord)]
#[mira(crate = ::mirascript_vm)]
struct Record<T>(String, #[mira(skip)] (), T);

fn main() {
    let mut runtime = Runtime::new();
    let record = runtime
        .insert(Record("example".to_string(), (), 1_u8))
        .unwrap();
    runtime.insert_global("record", record).unwrap();
    assert_eq!(
        runtime
            .eval("record.0")
            .unwrap()
            .as_str(&runtime)
            .unwrap()
            .unwrap(),
        "example"
    );
    assert_eq!(runtime.eval("record.2").unwrap().as_number().unwrap(), 1f64);

    assert!(runtime.eval("record.1").unwrap().is_nil());
    assert!(!runtime.eval("'1' in record").unwrap().as_boolean().unwrap(),);
}
