use mirascript::{MiraRecord, MiraValue, Runtime, compile};

#[derive(MiraRecord)]
struct User {
    name: String,
}

#[test]
fn facade_reexports_runtime_and_derives() -> mirascript::Result<()> {
    let mut runtime = Runtime::new();
    let user = runtime.insert_record(User { name: "Ada".into() })?;
    runtime.insert_global("user", MiraValue::Record(user.erase_record()))?;

    let value = runtime.run(&compile("user.name")?)?;
    assert_eq!(value.as_str(&runtime)?, Some("Ada"));
    Ok(())
}
