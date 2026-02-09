//! Fuzz target for Vim regexp pattern compilation.
//!
//! NOTE: This target cannot be used with `cargo fuzz` because the regexp
//! Rust code calls ~100+ C accessor functions via FFI. Without linking the
//! entire Neovim C codebase, the functions will fail to resolve at link time.
//!
//! For practical fuzz testing, use `just regexp-fuzz` which runs a VimL-based
//! fuzzer inside a running nvim process, exercising the full regexp pipeline.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Convert fuzz input to a string (regexp patterns are text)
    if let Ok(pattern) = std::str::from_utf8(data) {
        // Skip overly long patterns to avoid timeouts
        if pattern.len() > 1024 {
            return;
        }

        let _ = pattern; // placeholder — see `just regexp-fuzz` for active fuzzing
    }
});
