use mirascript_vm::{MiraRecord, MiraValue, Runtime};

#[derive(MiraRecord)]
struct Root {
    list: Vec<Item>,
    arr: Box<[Arr]>,
    fixed: [u8; 3],
}

#[derive(MiraRecord)]
struct Item {
    value: u8,
    name: String,
    children: Box<[Item]>,
}

#[derive(MiraRecord)]
struct Arr {
    value: u8,
    name: String,
}

#[test]
fn main() {
    let mut runtime = Runtime::new();
    let root = Root {
        list: vec![
            Item {
                value: 1,
                name: "one".to_string(),
                children: Box::new([Item {
                    value: 11,
                    name: "child".to_string(),
                    children: Box::new([]),
                }]),
            },
            Item {
                value: 2,
                name: "two".to_string(),
                children: Box::new([]),
            },
        ],
        arr: Box::new([
            Arr {
                value: 3,
                name: "three".to_string(),
            },
            Arr {
                value: 4,
                name: "four".to_string(),
            },
        ]),
        fixed: [5, 6, 7],
    };
    let root_handle = runtime.insert_record(root).unwrap();
    runtime
        .insert_global("root", MiraValue::record(root_handle))
        .unwrap();
    assert_eq!(
        runtime
            .eval_unchecked("root.list[0].value")
            .as_number_unchecked(),
        1f64
    );
    assert_eq!(
        runtime
            .eval_unchecked("root.list[1].name")
            .as_str_unchecked(&runtime),
        "two"
    );
    assert_eq!(
        runtime
            .eval_unchecked("root.list[0].children[0].value")
            .as_number_unchecked(),
        11f64
    );
}
