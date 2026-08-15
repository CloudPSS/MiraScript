use mirascript::{MiraAny, MiraContext, MiraRecord, MiraShared, compile};

#[derive(Clone, MiraRecord)]
struct User {
    name: String,
}

#[test]
fn facade_reexports_runtime_and_derives() -> mirascript::Result<()> {
    let mut context = MiraContext::new();
    context.insert("user", MiraShared::new(User { name: "Ada".into() }));

    assert_eq!(
        compile("user.name")?.run(&context)?,
        MiraAny::String("Ada".into()),
    );
    Ok(())
}
