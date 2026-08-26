use mirascript::{MiraRecord, Runtime, compile, mira};

#[derive(MiraRecord)]
struct User {
    name: String,
}

#[mira(const = ADD)]
fn add(a: f64, b: f64) -> f64 {
    a + b
}

#[mira(const = MOD)]
mod r#module {

    #[mira]
    const PI: f64 = std::f64::consts::PI;
}
#[test]
fn facade_reexports_runtime_and_derives() -> mirascript::Result<()> {
    let mut runtime = Runtime::new();
    let user = runtime.insert(User { name: "Ada".into() })?;
    runtime.insert_global("user", user)?;
    runtime.insert_global("add", ADD)?;
    runtime.insert_global("module", MOD)?;

    let value = runtime.run(&compile("user.name")?)?;
    assert_eq!(value.as_str(&runtime)?, Some("Ada"));
    assert_eq!(runtime.run(&compile("add(1, 2)")?)?, 3.into());
    assert_eq!(
        runtime.eval("module.PI")?.as_number(),
        Some(std::f64::consts::PI)
    );
    Ok(())
}
