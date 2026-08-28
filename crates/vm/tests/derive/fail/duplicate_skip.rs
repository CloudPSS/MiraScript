use mirascript_vm::{MiraRecord, mira};

#[derive(MiraRecord)]
struct Duplicate {
    value: u8,
    #[mira(skip, skip)]
    another: u8,
}

#[mira(skip, skip)]
mod test {
    #[mira(skip, skip)]
    const VALUE: u8 = 1;
}

fn main() {}
