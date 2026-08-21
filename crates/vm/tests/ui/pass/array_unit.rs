use mirascript_vm::{MiraArray, Runtime};

#[derive(Clone, MiraArray)]
#[mira(crate = mirascript_vm)]
struct Array;

fn main() {
    let mut runtime = Runtime::new();
    let array = Array;
    assert!(array.is_empty());
    let array = runtime.insert(array).unwrap();
    runtime.insert_global("array", array).unwrap();
    assert_eq!(
        runtime.eval("array::len()").unwrap().as_number().unwrap(),
        0f64
    );
    assert_eq!(
        runtime
            .eval("array::to_json()")
            .unwrap()
            .as_str(&runtime)
            .unwrap()
            .unwrap(),
        "[]"
    );
    assert!(runtime.eval("array.0").unwrap().is_nil());
}
