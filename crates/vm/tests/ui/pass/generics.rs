use mirascript_vm::{MiraArray, MiraRecord, Runtime};

#[derive(Clone, MiraRecord)]
#[mira(crate = "mirascript_vm")]
struct Record<'a, T: Into<u8>> {
    #[mira(rename = "value")]
    item: T,
    #[mira(skip)]
    hidden: &'a bool,
}

#[derive(Clone, MiraArray)]
struct Array<T>(T, #[mira(skip)] bool);

#[derive(Clone, MiraRecord)]
struct Tuple<T>(T, #[mira(skip)] bool);

fn main() {
    let mut runtime = Runtime::new();
    let record = runtime
        .insert(Record {
            item: 1_u8,
            hidden: &true,
        })
        .unwrap();
    let array = runtime.insert(Array(2_u16, false)).unwrap();
    let tuple = runtime.insert(Tuple(3_u32, true)).unwrap();
    assert_eq!(record.type_name(), "record");
    assert_eq!(array.type_name(), "array");
    assert_eq!(tuple.type_name(), "record");
}
