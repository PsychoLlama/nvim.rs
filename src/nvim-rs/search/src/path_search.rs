//! Find patterns in included files: `find_pattern_in_path()` and helpers.
//!
//! Due to the extremely wide dependency surface (~40 external calls spanning
//! regexp, file I/O, completion API, window management, and messaging), this
//! module is handled entirely in C via nvim_fpip_init/run/cleanup batch helpers.
//! The Rust layer (rs_find_pattern_in_path) has been eliminated; find_pattern_in_path
//! now calls the batch helpers directly from C.
