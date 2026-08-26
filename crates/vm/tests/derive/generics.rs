use mirascript_vm::{MiraArray, MiraRecord, Runtime};

#[derive(Clone, MiraRecord)]
#[mira(crate = mirascript_vm)]
struct Record<'a, T: Into<u8>> {
    #[mira(rename = "value")]
    item: T,
    #[mira(skip)]
    _hidden: &'a (),
}

#[derive(Clone, MiraArray)]
struct Array<T>(T, #[mira(skip)] ());

#[derive(Clone, MiraRecord)]
struct Tuple<T>(T, #[mira(skip)] ());

#[test]
fn main() {
    let mut runtime = Runtime::new();
    let record = runtime
        .insert(Record {
            item: 1_u8,
            _hidden: &(),
        })
        .unwrap();
    let array = runtime.insert(Array(2_u16, ())).unwrap();
    let tuple = runtime.insert(Tuple(3_u32, ())).unwrap();
    assert_eq!(record.type_name(), "record");
    assert_eq!(array.type_name(), "array");
    assert_eq!(tuple.type_name(), "record");

    runtime.insert_global("array", array).unwrap();
    runtime.insert_global("record", record).unwrap();
    runtime.insert_global("tuple", tuple).unwrap();
    assert_eq!(
        runtime.eval("record.value").unwrap().as_number().unwrap(),
        1f64
    );
    assert_eq!(runtime.eval("array.0").unwrap().as_number().unwrap(), 2f64);
    assert_eq!(runtime.eval("tuple.0").unwrap().as_number().unwrap(), 3f64);
}
