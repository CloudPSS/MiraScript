use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("target_os not defined!");
    // Fixed in https://github.com/purpleprotocol/mimalloc_rust/pull/136
    // Remove this when the next version of mimalloc is released.

    if target_os == "windows" {
        println!("cargo:rustc-link-lib=Advapi32");
    }
}
