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

#[derive(Clone, Debug, PartialEq, MiraRecord)]
struct Nested<T> {
    items: Vec<T>,
}

#[derive(Clone, PartialEq, Debug)]
struct NotMira;

#[test]
fn main() {
    let mut runtime = Runtime::new();
    let record_handle = runtime
        .insert_record(Record {
            item: 1_u8,
            hidden: &NotMira,
        })
        .unwrap();
    let array_handle = runtime.insert_array(Array(2_u16, &NotMira)).unwrap();
    let tuple_handle = runtime.insert_record(Tuple(3_u32, &NotMira)).unwrap();
    let nested_handle = runtime
        .insert_record(Nested { items: vec![4_u64] })
        .unwrap();
    let record = MiraValue::record(record_handle);
    let array = MiraValue::array(array_handle);
    let tuple = MiraValue::record(tuple_handle);
    assert_eq!(record.type_name(), "record");
    assert_eq!(array.type_name(), "array");
    assert_eq!(tuple.type_name(), "record");

    runtime.insert_global("array", array).unwrap();
    runtime.insert_global("record", record).unwrap();
    runtime.insert_global("tuple", tuple).unwrap();
    runtime
        .insert_global("nested", MiraValue::record(nested_handle))
        .unwrap();
    assert_eq!(
        runtime.eval_unchecked("record.value").as_number_unchecked(),
        1f64
    );
    assert_eq!(
        runtime.eval_unchecked("array.0").as_number_unchecked(),
        2f64
    );
    assert_eq!(
        runtime.eval_unchecked("tuple.0").as_number_unchecked(),
        3f64
    );
    assert_eq!(
        runtime
            .eval_unchecked("nested.items[0]")
            .as_number_unchecked(),
        4f64
    );

    let record = runtime.take_record(record_handle).unwrap();
    let array = runtime.take_array(array_handle).unwrap();
    let tuple = runtime.take_record(tuple_handle).unwrap();
    let nested = runtime.take_record(nested_handle).unwrap();
    assert_eq!(
        record,
        Record {
            item: 1_u8,
            hidden: &NotMira,
        }
    );
    assert_eq!(array, Array(2_u16, &NotMira));
    assert_eq!(tuple, Tuple(3_u32, &NotMira));
    assert_eq!(nested, Nested { items: vec![4_u64] });
}
