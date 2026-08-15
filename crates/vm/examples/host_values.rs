use mirascript_vm::{MiraContext, MiraExtern, MiraRecord, MiraShared, compile};

#[derive(Clone, MiraRecord)]
struct User {
    name: String,
}

#[derive(Clone, MiraExtern)]
#[mira(tag = "Counter")]
struct Counter {
    value: i64,
    #[mira(readonly)]
    limit: i64,
}

fn main() -> mirascript_vm::Result<()> {
    let user = MiraShared::new(User { name: "Ada".into() });
    let counter = MiraShared::new(Counter {
        value: 1,
        limit: 10,
    });

    let mut context = MiraContext::new();
    context.insert("user", user.clone());
    context.insert("counter", counter.clone());

    let script = compile("counter.value += 1; `Hello, $(user.name)! Count: $(counter.value)`")?;
    println!("{:?}", script.run(&context)?);

    user.borrow_mut().name = "Grace".into();
    println!("{:?}", script.run(&context)?);
    assert_eq!(counter.borrow().value, 3);
    Ok(())
}
