//! Buffer name completion and buffer scanning support.
//!
//! This module provides helper functions for buffer name completion
//! and buffer scanning during keyword completion.
//! The core buffer operations remain in C.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int, c_void};

use nvim_buffer::{buf_struct::buf_ref, BufHandle};

// Win pointer: use *mut u8 to match ctrl_x.rs and entry.rs conventions.
type WinPtr = *mut u8;

// C accessor functions
extern "C" {

    // nvim_get_next_bufname_token_impl: deleted (Phase 15), inlined below

    // ins_compl_next_buf buf accessors (Phase 14) - use BufHandle to match next.rs
    fn nvim_buf_has_ml_mfp(buf: BufHandle) -> c_int;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_firstbuf_wrapper() -> BufHandle;
    fn path_tail(path: *const c_char) -> *mut c_char;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn rs_ins_compl_add(
        str_: *const c_char,
        len: c_int,
        fname: *const c_char,
        cptext: *const c_void,
        cptext_allocated: c_int,
        user_data: *const c_void,
        dir: c_int,
        flags: c_int,
        adup: c_int,
        user_hl: *const c_int,
        score: c_int,
    ) -> c_int;
}

// ins_compl_next_buf win accessors (Phase 14) - use *mut u8 to match entry.rs/ctrl_x.rs
#[allow(clashing_extern_declarations)]
extern "C" {
    fn nvim_win_get_w_next(wp: WinPtr) -> WinPtr;
    fn nvim_win_get_w_buffer_raw(wp: WinPtr) -> BufHandle;
    fn nvim_win_get_focusable(wp: WinPtr) -> c_int;
    fn nvim_get_curwin() -> WinPtr;
    fn nvim_get_firstwin() -> WinPtr;
    fn rs_win_valid(wp: WinPtr) -> c_int;
}

// CTRL-X mode for buffer names
const CTRL_X_BUFNAMES: c_int = 18;

// Continuation status flags
const CONT_LOCAL: c_int = 32;

// CP flags (matching C enum)
const CP_ICASE: c_int = 16;

// =============================================================================
// Phase 2 (pass 4): get_next_bufname_token
// =============================================================================

/// Get the next buffer name completion matches.
///
/// Iterates over all loaded buffers looking for buffer names that start with
/// the current completion leader (compl_orig_text), and adds matching buffer
/// tail names to the completion list.
///
/// # Safety
/// Requires valid buffer list state; called from insert mode completion only.
#[no_mangle]
pub unsafe extern "C" fn rs_get_next_bufname_token() {
    // Inline nvim_get_next_bufname_token_impl (Phase 15):
    // FOR_ALL_BUFFERS: start from firstbuf, walk b_next until NULL
    let mut b = nvim_get_firstbuf_wrapper();
    while !b.is_null() {
        if buf_ref(b).b_p_bl != 0 {
            let sfname = buf_ref(b).b_sfname;
            if !sfname.is_null() {
                let tail = path_tail(sfname);
                let orig_data = crate::vars::compl_orig_text.data.cast_const();
                let orig_size = crate::vars::compl_orig_text.size;
                if !tail.is_null()
                    && !orig_data.is_null()
                    && strncmp(tail.cast_const(), orig_data, orig_size) == 0
                {
                    let flags = if crate::vars::nvim_get_p_ic() != 0 {
                        CP_ICASE
                    } else {
                        0
                    };
                    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                    rs_ins_compl_add(
                        tail.cast_const(),
                        strlen(tail.cast_const()) as c_int,
                        std::ptr::null(),
                        std::ptr::null(),
                        0,
                        std::ptr::null(),
                        0,
                        flags,
                        0,
                        std::ptr::null(),
                        -1, // FUZZY_SCORE_NONE
                    );
                }
            }
        }
        b = BufHandle::from_ptr(buf_ref(b).b_next);
    }
}

// =============================================================================
// Phase 14: ins_compl_next_buf migration
// =============================================================================

/// Persistent window pointer for the 'w' (windows) scan mode.
///
/// This mirrors the C `static win_T *wp = NULL` in `ins_compl_next_buf`.
/// Initialized to null; updated as windows are walked.
static mut NEXT_BUF_WP: WinPtr = std::ptr::null_mut();

/// Iterate through buffers/windows to find the next unscanned buffer for
/// completion.
///
/// Mirrors the C `ins_compl_next_buf(buf_T *buf, int flag)` function.
///
/// - flag `'w'` (119): walk windows, return the window's buffer.
/// - flag `'b'` (98): walk loaded buffers only.
/// - flag `'u'` (117): walk non-loaded buffers only.
/// - flag `'U'` (85): walk unlisted buffers only.
///
/// Returns the next buffer to scan (may be curbuf if no unscanned buffer
/// is found, which signals "done" to the caller).
///
/// # Safety
/// Requires that the global buffer/window lists are valid. Uses a `static mut`
/// for the persistent window pointer, so this function must not be called
/// concurrently.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_next_buf(buf: BufHandle, flag: c_int) -> BufHandle {
    let curbuf = nvim_get_curbuf();
    let curwin = nvim_get_curwin();

    if flag == c_int::from(b'w') {
        // Walk windows to find an unscanned buffer
        if buf == curbuf || rs_win_valid(NEXT_BUF_WP) == 0 {
            // First call or window was closed: start from curwin
            NEXT_BUF_WP = curwin;
        }

        loop {
            // Advance to next window (wrap to firstwin at the end)
            let next = nvim_win_get_w_next(NEXT_BUF_WP);
            NEXT_BUF_WP = if next.is_null() {
                nvim_get_firstwin()
            } else {
                next
            };

            // Stop if we wrapped back to curwin, or found an unscanned focusable buffer
            let wp_buf = nvim_win_get_w_buffer_raw(NEXT_BUF_WP);
            let scanned = buf_ref(wp_buf).b_scanned != 0;
            let focusable = nvim_win_get_focusable(NEXT_BUF_WP) != 0;
            if NEXT_BUF_WP == curwin || (!scanned && focusable) {
                break;
            }
        }

        return nvim_win_get_w_buffer_raw(NEXT_BUF_WP);
    }

    // Walk the buffer list
    let mut cur = buf;
    loop {
        let next = BufHandle::from_ptr(buf_ref(cur).b_next);
        cur = if next.is_null() {
            nvim_get_firstbuf_wrapper()
        } else {
            next
        };

        // Stop if we wrapped back to curbuf
        if cur == curbuf {
            break;
        }

        // Decide whether to skip this buffer based on flag
        let skip = if flag == c_int::from(b'U') {
            // 'U': unlisted buffers -- skip listed ones
            buf_ref(cur).b_p_bl != 0
        } else {
            // 'b': only listed loaded buffers
            // 'u': only listed unloaded buffers
            // skip if not listed, or loaded/unloaded mismatch
            let is_listed = buf_ref(cur).b_p_bl != 0;
            let is_loaded = nvim_buf_has_ml_mfp(cur) != 0;
            let want_unloaded = flag == c_int::from(b'u');
            !is_listed || (is_loaded == want_unloaded)
        };

        if !skip && buf_ref(cur).b_scanned == 0 {
            break;
        }
    }

    cur
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_bufnames_constant() {
        assert_eq!(CTRL_X_BUFNAMES, 18);
    }

    #[test]
    fn test_cont_local_constant() {
        assert_eq!(CONT_LOCAL, 32);
    }
}
