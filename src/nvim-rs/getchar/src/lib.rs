//! Character input handling for Neovim
//!
//! Provides Rust implementations of typeahead buffer and input functions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]

use std::ffi::c_int;

extern "C" {
    /// Check if `readbuf1.bh_first.b_next` is NULL
    fn nvim_readbuf1_is_empty() -> c_int;
    /// Check if `readbuf2.bh_first.b_next` is NULL
    fn nvim_readbuf2_is_empty() -> c_int;
    /// Get `typebuf.tb_change_cnt`
    fn nvim_get_typebuf_change_cnt() -> c_int;
    /// Get `typebuf_was_filled`
    fn nvim_get_typebuf_was_filled() -> c_int;
    /// Get `typebuf.tb_maplen`
    fn nvim_get_typebuf_maplen() -> c_int;
    /// Get `curscript`
    fn nvim_get_curscript() -> c_int;
}

/// Returns true if the stuff buffer is empty.
///
/// # Safety
/// Calls C accessor functions for `readbuf1` and `readbuf2`.
#[no_mangle]
pub unsafe extern "C" fn rs_stuff_empty() -> c_int {
    c_int::from(nvim_readbuf1_is_empty() != 0 && nvim_readbuf2_is_empty() != 0)
}

/// Returns true if `readbuf1` is empty. There may still be redo characters in
/// `readbuf2`.
///
/// # Safety
/// Calls C accessor function for `readbuf1`.
#[no_mangle]
pub unsafe extern "C" fn rs_readbuf1_empty() -> c_int {
    nvim_readbuf1_is_empty()
}

/// Check if the typeahead buffer was changed.
///
/// Returns true if `tb_change_cnt` changed or `typebuf_was_filled` is true.
///
/// # Safety
/// Calls C accessor functions for `typebuf`.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_changed(tb_change_cnt: c_int) -> c_int {
    if tb_change_cnt == 0 {
        return 0;
    }
    let current_cnt = nvim_get_typebuf_change_cnt();
    let was_filled = nvim_get_typebuf_was_filled();
    c_int::from(current_cnt != tb_change_cnt || was_filled != 0)
}

/// Return true if there are no characters in the typeahead buffer that have
/// not been typed (result from a mapping or come from `:normal`).
///
/// # Safety
/// Calls C accessor function for `typebuf.tb_maplen`.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_typed() -> c_int {
    c_int::from(nvim_get_typebuf_maplen() == 0)
}

/// Get the number of characters that are mapped (or not typed).
///
/// # Safety
/// Calls C accessor function for `typebuf.tb_maplen`.
#[no_mangle]
pub unsafe extern "C" fn rs_typebuf_maplen() -> c_int {
    nvim_get_typebuf_maplen()
}

/// Return true when reading keys from a script file.
///
/// # Safety
/// Calls C accessor function for `curscript`.
#[no_mangle]
pub unsafe extern "C" fn rs_using_script() -> c_int {
    c_int::from(nvim_get_curscript() >= 0)
}
