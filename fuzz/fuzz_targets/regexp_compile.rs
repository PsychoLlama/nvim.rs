//! Fuzz target for Vim regexp pattern compilation.
//!
//! This is a skeleton target. When the Rust regexp crate exists, uncomment
//! the actual compilation call and add the crate dependency in fuzz/Cargo.toml.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Convert fuzz input to a string (regexp patterns are text)
    if let Ok(pattern) = std::str::from_utf8(data) {
        // Skip overly long patterns to avoid timeouts
        if pattern.len() > 1024 {
            return;
        }

        // TODO: When regexp crate exists, call the compile function:
        //
        // use nvim_regexp::vim_regcomp;
        //
        // // Test compilation doesn't panic or infinite-loop
        // let _ = vim_regcomp(pattern, RE_MAGIC);
        //
        // // Also test with different magic modes
        // let _ = vim_regcomp(pattern, RE_NOMAGIC);
        // let _ = vim_regcomp(&format!("\\v{}", pattern), RE_MAGIC);

        let _ = pattern; // placeholder
    }
});
