#pragma once

/// Open a command-line window for the current command-line type (q:, q/, q?).
/// Implemented in Rust: src/nvim-rs/cmdline/src/cmdwin.rs
int nvim_open_cmdwin(void);
