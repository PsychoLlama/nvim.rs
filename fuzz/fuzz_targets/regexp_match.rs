//! Fuzz target for Vim regexp pattern matching.
//!
//! This is a skeleton target. When the Rust regexp crate exists, uncomment
//! the actual matching call and add the crate dependency in fuzz/Cargo.toml.

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

    // TODO: When regexp crate exists, call the match function:
    //
    // use nvim_regexp::{vim_regcomp, vim_regexec, RE_MAGIC};
    //
    // if let Ok(prog) = vim_regcomp(pattern, RE_MAGIC) {
    //     // Test matching doesn't panic or infinite-loop
    //     let _ = vim_regexec(&prog, input, 0);
    // }

    let _ = (pattern, input); // placeholder
});
