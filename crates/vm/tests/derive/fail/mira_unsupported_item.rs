use mirascript_vm::mira;

#[mira]
const TEST: u8 = 1;

#[mira]
struct TestStruct {
    value: u8,
}

#[mira]
enum TestEnum {}

#[mira]
use std::rc::Rc;

#[mira]
type TestType = u8;

fn main() {}
