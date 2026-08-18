use mirascript_vm::{MiraArray, MiraContext, MiraRecord, MiraShared, compile};

#[derive(Clone, MiraRecord)]
struct User {
    name: String,
}

#[derive(Clone, MiraArray)]
struct Arr(u64, f64, User);

fn main() -> mirascript_vm::Result<()> {
    let user = MiraShared::new(User { name: "Ada".into() });
    let arr = MiraShared::new(Arr(1, 10.0, User { name: "Bob".into() }));

    let mut context = MiraContext::new();
    context.insert("user", user.clone());
    context.insert("arr", arr.clone());

    let script = compile("`Hello, $(user.name)! $(arr::len())/$arr`")?;
    assert_eq!(
        script.run(&context)?,
        "Hello, Ada! 3/1, 10, (name: Bob)".into()
    );

    user.borrow_mut().name = "Grace".into();
    assert_eq!(
        script.run(&context)?,
        "Hello, Grace! 3/1, 10, (name: Bob)".into()
    );
    Ok(())
}
