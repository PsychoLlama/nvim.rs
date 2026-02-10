//! Character input handling for Neovim
//!
//! Provides Rust implementations of typeahead buffer and input functions.
//!
//! # Modules
//!
//! - [`typebuf`]: Typeahead buffer data structures and manipulation
//! - [`buffheader`]: Buffer header for stuff/redo/recording buffers
//! - [`input`]: Character input functions and key translation
//! - [`macro_recording`]: Macro recording and playback
//! - [`mapping`]: Key mapping expansion
//! - [`stuff`]: Stuffbuffer and special key handling

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(unsafe_code)]

pub mod buffheader;
pub mod input;
pub mod macro_recording;
pub mod mapping;
pub mod stuff;
pub mod typebuf;

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
    /// Get `KeyNoremap`
    fn nvim_get_keynoremap() -> c_int;
    /// Get `RM_NONE` constant
    fn nvim_get_rm_none() -> c_int;
    /// Get `RM_SCRIPT` constant
    fn nvim_get_rm_script() -> c_int;
    /// Get `State` global
    fn nvim_get_state() -> c_int;
    /// Get `arrow_used` global
    fn nvim_get_arrow_used() -> c_int;
    /// Call `u_sync(force)`
    fn nvim_call_u_sync(force: c_int);
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

/// Return true when keys cannot be remapped.
///
/// Keys cannot be remapped when `KeyNoremap` has `RM_NONE` or `RM_SCRIPT` set.
///
/// # Safety
/// Calls C accessor functions for `KeyNoremap` and remap constants.
#[no_mangle]
pub unsafe extern "C" fn rs_noremap_keys() -> c_int {
    let keynoremap = nvim_get_keynoremap();
    let rm_none = nvim_get_rm_none();
    let rm_script = nvim_get_rm_script();
    c_int::from((keynoremap & (rm_none | rm_script)) != 0)
}

/// Mode flags
const MODE_INSERT: c_int = 0x10;
const MODE_CMDLINE: c_int = 0x08;

/// Sync undo. Called when typed characters are obtained from the typeahead
/// buffer, or when a menu is used.
///
/// Do not sync in Insert or Cmdline mode unless cursor key has been used,
/// and not while reading a script file.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_may_sync_undo() {
    let state = nvim_get_state();
    let arrow_used = nvim_get_arrow_used() != 0;
    let curscript = nvim_get_curscript();

    if (state & (MODE_INSERT | MODE_CMDLINE) == 0 || arrow_used) && curscript < 0 {
        nvim_call_u_sync(0);
    }
}
