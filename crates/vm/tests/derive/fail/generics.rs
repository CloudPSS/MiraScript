use mirascript_vm::{MiraArray, MiraRecord, MiraValue, Runtime};

#[derive(Clone, Debug, PartialEq, MiraRecord)]
#[mira(crate = mirascript_vm)]
struct Record<'a, T: Into<u8>, U> {
    #[mira(rename = "value")]
    item: T,
    #[mira(skip)]
    hidden: &'a U,
}

#[derive(Clone, Debug, PartialEq, MiraArray)]
struct Array<'a, T, U>(T, #[mira(skip)] &'a U);

#[derive(Clone, Debug, PartialEq, MiraRecord)]
struct Tuple<'a, T, U>(T, #[mira(skip)] &'a U);

#[derive(Clone, PartialEq, Debug)]
struct NotMira;

fn main() {
    let mut runtime = Runtime::new();
    let record_handle = runtime
        .insert_record(Record {
            item: NotMira,
            hidden: &NotMira,
        })
        .unwrap();
    let array_handle = runtime.insert_array(Array(NotMira, &NotMira)).unwrap();
    let tuple_handle = runtime.insert_record(Tuple(NotMira, &NotMira)).unwrap();
    let _ = MiraValue::record(record_handle);
    let _ = MiraValue::array(array_handle);
    let _ = MiraValue::record(tuple_handle);
}
