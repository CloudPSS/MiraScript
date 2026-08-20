use mirascript_vm::{MiraArray, MiraRecord, MiraValue, Runtime, compile};

#[derive(MiraRecord)]
struct User {
    name: String,
}

#[derive(MiraArray)]
struct Arr(u64, f64, User);

fn main() -> mirascript_vm::Result<()> {
    let mut runtime = Runtime::new();
    let user = runtime.insert_record(User { name: "Ada".into() })?;
    let arr = runtime.insert_array(Arr(1, 10.0, User { name: "Bob".into() }))?;
    runtime.insert_global("user", MiraValue::Record(user.erase_record()))?;
    runtime.insert_global("arr", MiraValue::Array(arr.erase_array()))?;

    let script = compile("`Hello, $(user.name)! $(arr::len())/$arr`")?;
    let value = runtime.run(&script)?;
    assert_eq!(
        value.as_string(&runtime)?,
        Some("Hello, Ada! 3/1, 10, (name: Bob)")
    );

    runtime.get_record_mut(user)?.name = "Grace".into();
    let value = runtime.run(&script)?;
    assert_eq!(
        value.as_string(&runtime)?,
        Some("Hello, Grace! 3/1, 10, (name: Bob)")
    );
    Ok(())
}
