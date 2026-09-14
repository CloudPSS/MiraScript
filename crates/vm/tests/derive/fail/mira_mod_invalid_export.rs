use mirascript_vm::mira;

#[mira]
mod duplicate {
    #[mira]
    type Ty = f64;
    #[mira]
    trait T {}
    #[mira]
    enum E {
        Value,
    }
    #[mira]
    union U {}
    #[mira]
    mod M {}
    #[mira]
    macro_rules! m {}
    #[mira]
    const C: u32 = 42;
    #[mira]
    static S: u32 = 42;
    #[mira]
    fn f() {}
    #[mira]
    impl Default for E {
        fn default() -> Self {
            Self::Value
        }
    }
}

fn main() {}
