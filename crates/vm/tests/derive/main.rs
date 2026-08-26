#[test]
fn derive_macros_reject_shapes() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/derive/fail/*.rs");
}

mod array_tuple;
mod array_unit;
mod complex;
mod generics;
mod mira_module;
mod record_named;
mod record_tuple;
mod record_unit;
