//! Fuzz target for Vim regexp pattern matching.
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
    // Split fuzz input into pattern and input text at the first null byte.
    // This lets the fuzzer explore both pattern and input spaces.
    let split_pos = data.iter().position(|&b| b == 0);
    let (pattern_bytes, input_bytes) = match split_pos {
        Some(pos) => (&data[..pos], &data[pos + 1..]),
        None => return, // need both pattern and input
    };

    let Ok(pattern) = std::str::from_utf8(pattern_bytes) else {
        return;
    };
    let Ok(input) = std::str::from_utf8(input_bytes) else {
        return;
    };

    // Skip overly long inputs to avoid timeouts
    if pattern.len() > 512 || input.len() > 4096 {
        return;
    }

    let _ = (pattern, input); // placeholder — see `just regexp-fuzz` for active fuzzing
});
