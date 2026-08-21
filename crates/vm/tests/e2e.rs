use mirascript_vm::{MiraRecord, Runtime};

#[derive(Clone, MiraRecord)]
#[mira(crate = mirascript_vm)]
struct Record<T> {
    key: String,
    #[mira(rename = "value")]
    item: T,
    #[mira(skip)]
    hidden: bool,
}
