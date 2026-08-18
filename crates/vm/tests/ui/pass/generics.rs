use mirascript_vm::{MiraAny, MiraArray, MiraRecord, MiraShared};

#[derive(Clone, MiraRecord)]
#[mira(crate = "mirascript_vm")]
struct Record<T> {
    #[mira(rename = "value")]
    item: T,
    #[mira(skip)]
    hidden: bool,
}

#[derive(Clone, MiraArray)]
struct Array<T>(T, #[mira(skip)] bool);

fn main() {
    let record = MiraAny::from(MiraShared::new(Record {
        item: 1_u8,
        hidden: true,
    }));
    let array = MiraAny::from(Array(2_u16, false));
    assert_eq!(record.type_name(), "record");
    assert_eq!(array.type_name(), "array");
}
