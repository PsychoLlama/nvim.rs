//! FFI declarations for C accessor functions used by session migration.
//!
//! Each `nvim_ses_*` function is a thin C accessor defined in `ex_session.c`
//! that provides access to struct fields and global variables.

use std::ffi::{c_char, c_int, c_uint, c_void};

/// Opaque handle types for C structs
pub type WinPtr = *mut c_void;
pub type FramePtr = *mut c_void;
pub type FramePtrConst = *const c_void;
pub type BufPtr = *mut c_void;
pub type TabpagePtr = *mut c_void;
pub type ExargPtr = *mut c_void;
pub type GarrayPtr = *mut c_void;

extern "C" {
    // --- Window accessors (Phase 2) ---
    pub fn nvim_ses_win_get_floating(wp: WinPtr) -> bool;
    pub fn nvim_ses_win_get_buffer(wp: WinPtr) -> BufPtr;
    pub fn nvim_ses_win_get_next(wp: WinPtr) -> WinPtr;

    // --- Buffer query accessors (Phase 2) ---
    pub fn nvim_ses_buf_get_fname(buf: BufPtr) -> *const c_char;
    pub fn nvim_ses_buf_is_terminal(buf: BufPtr) -> bool;
    pub fn nvim_ses_bt_nofilename(buf: BufPtr) -> bool;
    pub fn nvim_ses_bt_help(buf: BufPtr) -> bool;
    pub fn nvim_ses_bt_terminal(buf: BufPtr) -> bool;

    // --- Session flags (Phase 2) ---
    pub fn nvim_ses_get_ssop_flags() -> c_uint;
    pub fn nvim_ses_get_ssop_flags_ptr() -> *const c_uint;

    // --- Frame accessors (Phase 2) ---
    pub fn nvim_ses_frame_get_layout(fr: FramePtr) -> c_int;
    pub fn nvim_ses_frame_get_child(fr: FramePtr) -> FramePtr;
    pub fn nvim_ses_frame_get_next(fr: FramePtr) -> FramePtr;
    pub fn nvim_ses_frame_get_win(fr: FramePtr) -> WinPtr;

    // --- Filename helper accessors (Phase 3) ---
    pub fn nvim_ses_buf_get_sfname(buf: BufPtr) -> *const c_char;
    pub fn nvim_ses_buf_get_ffname(buf: BufPtr) -> *const c_char;
    pub fn nvim_ses_get_vop_flags_ptr() -> *const c_uint;
    pub fn nvim_ses_get_p_acd() -> c_int;
    pub fn nvim_ses_get_did_lcd() -> c_int;
    pub fn nvim_ses_set_did_lcd(val: c_int);
    pub fn nvim_ses_home_replace_save(name: *const c_char) -> *mut c_char;
    pub fn nvim_ses_vim_strsave_fnameescape(name: *const c_char) -> *mut c_char;
    pub fn nvim_ses_xfree(p: *mut c_void);
    pub fn nvim_ses_utfc_ptr2len(p: *const c_char) -> c_int;
}
