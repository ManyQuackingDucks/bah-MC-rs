fn main() {
    #[cfg(not(debug_assertions))]
    {
        // Merges empty `.rdata` and `.pdata` into .text section saving a few bytes in data
        // directories portion  of PE header.
        println!("cargo:rustc-link-arg-bins=/MERGE:.rdata=.text");
        println!("cargo:rustc-link-arg-bins=/MERGE:.pdata=.text");
        // Removes `IMAGE_DEBUG_DIRECTORY` from PE.
        println!("cargo:rustc-link-arg-bins=/EMITPOGOPHASEINFO");
        println!("cargo:rustc-link-arg-bins=/DEBUG:NONE");
        // See: https://github.com/mcountryman/min-sized-rust-windows/pull/7
        println!("cargo:rustc-link-arg-bins=/STUB:stub.exe");
    }
}
