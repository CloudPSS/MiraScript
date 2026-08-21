use mirascript_vm::{MiraArray, Runtime};

#[derive(Clone, MiraArray)]
#[mira(crate = mirascript_vm)]
struct Array<T>(String, #[mira(skip)] (), T);

fn main() {
    let mut runtime = Runtime::new();
    let array = runtime
        .insert(Array("example".to_string(), (), 1_u8))
        .unwrap();
    runtime.insert_global("array", array).unwrap();
    assert_eq!(
        runtime.eval("array::len()").unwrap().as_number().unwrap(),
        2f64
    );
    assert_eq!(
        runtime
            .eval("array.0")
            .unwrap()
            .as_str(&runtime)
            .unwrap()
            .unwrap(),
        "example"
    );
    assert_eq!(runtime.eval("array.1").unwrap().as_number().unwrap(), 1f64);
    assert!(runtime.eval("array.2").unwrap().is_nil());
    assert!(
        runtime
            .eval("'example' in array")
            .unwrap()
            .as_boolean()
            .unwrap()
    );
}
