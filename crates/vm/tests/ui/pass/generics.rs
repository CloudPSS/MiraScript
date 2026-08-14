use mira_vm::{MiraAny, MiraArray, MiraExtern, MiraRecord, MiraShared};

#[derive(Clone, MiraRecord)]
#[mira(crate = "mira_vm")]
struct Record<T> {
    #[mira(rename = "value")]
    item: T,
    #[mira(skip)]
    hidden: bool,
}

#[derive(Clone, MiraArray)]
struct Array<T>(T, #[mira(skip)] bool);

#[derive(Clone, MiraExtern)]
#[mira(tag = "Box")]
struct Extern<T> {
    value: T,
    #[mira(readonly)]
    limit: T,
}

fn main() {
    let record = MiraAny::from(MiraShared::new(Record {
        item: 1_u8,
        hidden: true,
    }));
    let array = MiraAny::from(Array(2_u16, false));
    let external = MiraAny::from(Extern {
        value: 3_i32,
        limit: 4_i32,
    });
    assert_eq!(record.type_name(), "record");
    assert_eq!(array.type_name(), "array");
    assert_eq!(external.type_name(), "extern");
}
