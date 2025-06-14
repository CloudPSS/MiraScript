fn main() {
    // Fixed in https://github.com/purpleprotocol/mimalloc_rust/pull/136
    // Remove this when the next version of mimalloc is released.
    println!("cargo:rustc-link-lib=Advapi32");
}
