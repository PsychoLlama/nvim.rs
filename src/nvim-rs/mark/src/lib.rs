//! Mark utilities for Neovim
//!
//! This crate provides functions for working with marks and positions.

use std::ffi::{c_char, c_int, c_uint, c_void};

use nvim_buffer::BufHandle;
use nvim_window::WinHandle;

/// Opaque handle to a tabpage (tab_T*).
/// Used only for iteration, not for field access.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TabHandle(*mut c_void);

impl TabHandle {
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// FFI: C accessor functions called from Rust
// =============================================================================
extern "C" {
    // Memory management
    fn nvim_mark_xfree(ptr: *mut c_void);
    fn nvim_mark_xstrdup(s: *const c_char) -> *mut c_char;

    // Buffer accessors
    fn nvim_buf_get_handle(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;
    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;

    // Global state
    fn nvim_mark_get_namedfm() -> *mut XfmarkT;
    fn nvim_mark_path_fnamecmp(a: *const c_char, b: *const c_char) -> c_int;
    // Window jumplist accessors
    fn nvim_mark_win_get_jumplistlen(win: WinHandle) -> c_int;
    fn nvim_mark_win_set_jumplistlen(win: WinHandle, len: c_int);
    fn nvim_mark_win_get_jumplistidx(win: WinHandle) -> c_int;
    fn nvim_mark_win_set_jumplistidx(win: WinHandle, idx: c_int);
    fn nvim_mark_win_get_jumplist_entry(win: WinHandle, idx: c_int) -> *mut XfmarkT;

    // Window pcmark/cursor accessors
    fn nvim_mark_win_get_pcmark(win: WinHandle) -> PosT;
    fn nvim_mark_win_set_pcmark(win: WinHandle, pos: PosT);
    fn nvim_mark_win_get_prev_pcmark(win: WinHandle) -> PosT;
    fn nvim_mark_win_set_prev_pcmark(win: WinHandle, pos: PosT);
    fn nvim_mark_win_get_cursor(win: WinHandle) -> PosT;
    fn nvim_mark_win_get_buffer(win: WinHandle) -> BufHandle;
    fn nvim_mark_win_set_topline(win: WinHandle, topline: LinenrT);

    // Buffer mark accessors
    fn nvim_mark_buf_get_last_cursor(buf: BufHandle) -> *mut FmarkT;
    fn nvim_mark_buf_get_namedm(buf: BufHandle, idx: c_int) -> *mut FmarkT;
    fn nvim_mark_buf_get_last_insert(buf: BufHandle) -> *mut FmarkT;
    fn nvim_mark_buf_get_last_change(buf: BufHandle) -> *mut FmarkT;
    fn nvim_mark_buf_get_op_start(buf: BufHandle) -> *mut PosT;
    fn nvim_mark_buf_get_op_end(buf: BufHandle) -> *mut PosT;
    fn nvim_mark_buf_get_op_start_val(buf: BufHandle) -> PosT;
    fn nvim_mark_buf_get_op_end_val(buf: BufHandle) -> PosT;
    fn nvim_mark_buf_get_visual_start(buf: BufHandle) -> PosT;
    fn nvim_mark_buf_get_visual_end(buf: BufHandle) -> PosT;
    fn nvim_mark_buf_get_visual_mode(buf: BufHandle) -> c_int;
    fn nvim_mark_buf_set_visual_mode(buf: BufHandle, mode: c_int);
    fn nvim_mark_buf_get_prompt_start(buf: BufHandle) -> *mut FmarkT;
    fn nvim_mark_buf_get_changelist(buf: BufHandle, idx: c_int) -> *mut FmarkT;
    fn nvim_mark_buf_get_changelistlen(buf: BufHandle) -> c_int;
    fn nvim_mark_buf_set_changelistlen(buf: BufHandle, len: c_int);

    // Error message strings
    fn nvim_mark_get_e_umark() -> *const c_char;
    fn nvim_mark_get_e_marknotset() -> *const c_char;
    fn nvim_mark_get_e_markinval() -> *const c_char;

    // Timestamp
    fn nvim_mark_os_time() -> Timestamp;

    // Global state / cross-function callbacks
    fn nvim_mark_clear_namedfm();
    fn nvim_mark_get_curwin() -> WinHandle;
    fn nvim_mark_get_curbuf() -> BufHandle;
    fn nvim_mark_buflist_findnr(fnum: c_int) -> BufHandle;
    fn nvim_mark_bt_prompt(buf: BufHandle) -> c_int;
    fn nvim_mark_fname2fnum(xfm: *mut XfmarkT);
    fn nvim_buf_get_ffname(buf: BufHandle) -> *const c_char;

    // Phase 4: Global state
    fn nvim_mark_get_global_busy() -> c_int;
    fn nvim_mark_get_listcmd_busy() -> c_int;
    fn nvim_mark_get_jop_flags() -> c_uint;
    fn nvim_mark_get_cmod_flags() -> c_uint;
    fn nvim_mark_setpcmark();

    // Phase 4: Window topline/changelist
    fn nvim_mark_win_get_topline(win: WinHandle) -> LinenrT;
    fn nvim_mark_win_get_changelistidx(win: WinHandle) -> c_int;
    fn nvim_mark_win_set_changelistidx(win: WinHandle, idx: c_int);

    // Phase 4: Jumplist manipulation
    fn nvim_mark_win_jumplist_remove(win: WinHandle, from_idx: c_int, len: c_int);
    fn nvim_mark_win_jumplist_shift_down(win: WinHandle);
    fn nvim_mark_win_jumplist_copy_entry(win: WinHandle, to_idx: c_int, from_idx: c_int);
    fn nvim_mark_win_set_jumplist_xfmark(
        win: WinHandle,
        idx: c_int,
        mark: PosT,
        fnum: c_int,
        view: FmarkvT,
    );
    fn nvim_mark_win_get_jumplist_fnum(win: WinHandle, idx: c_int) -> c_int;
    fn nvim_mark_win_get_jumplist_lnum(win: WinHandle, idx: c_int) -> LinenrT;
    fn nvim_mark_win_jumplist_free_fname(win: WinHandle, idx: c_int);

    // Phase 4: Tag stack
    fn nvim_mark_win_get_tagstacklen(win: WinHandle) -> c_int;
    fn nvim_mark_win_set_tagstacklen(win: WinHandle, len: c_int);
    fn nvim_mark_win_get_tagstackidx(win: WinHandle) -> c_int;
    fn nvim_mark_win_set_tagstackidx(win: WinHandle, idx: c_int);
    fn nvim_mark_win_get_tagstack_fnum(win: WinHandle, idx: c_int) -> c_int;
    fn nvim_mark_win_tagstack_clear_entry(win: WinHandle, idx: c_int);
    fn nvim_mark_win_tagstack_remove(win: WinHandle, from_idx: c_int, len: c_int);
    fn nvim_mark_buflist_nr2name(fnum: c_int, listed: c_int, unstripped: c_int) -> *mut c_char;
    fn nvim_mark_mark_line(pos: *mut PosT, lead_len: c_int) -> *mut c_char;

    // Phase 5: Mark adjustment accessors
    fn nvim_mark_buf_get_visual_start_ptr(buf: BufHandle) -> *mut PosT;
    fn nvim_mark_buf_get_visual_end_ptr(buf: BufHandle) -> *mut PosT;
    fn nvim_mark_buf_get_has_qf_entry(buf: BufHandle) -> c_int;
    fn nvim_mark_buf_set_has_qf_entry(buf: BufHandle, val: c_int);
    fn nvim_mark_get_saved_cursor() -> *mut PosT;
    fn nvim_mark_win_get_next(win: WinHandle) -> WinHandle;
    fn nvim_mark_win_get_buf(win: WinHandle) -> BufHandle;
    fn nvim_mark_win_get_old_cursor_lnum(win: WinHandle) -> LinenrT;
    fn nvim_mark_win_get_old_cursor_lnum_ptr(win: WinHandle) -> *mut LinenrT;
    fn nvim_mark_win_get_old_visual_lnum_ptr(win: WinHandle) -> *mut LinenrT;
    fn nvim_mark_win_get_topline_val(win: WinHandle) -> LinenrT;
    fn nvim_mark_win_set_topline_val(win: WinHandle, val: LinenrT);
    fn nvim_mark_win_set_topfill(win: WinHandle, val: c_int);
    fn nvim_mark_win_get_cursor_ptr(win: WinHandle) -> *mut PosT;
    fn nvim_mark_win_get_pcmark_ptr(win: WinHandle) -> *mut PosT;
    fn nvim_mark_win_get_prev_pcmark_ptr(win: WinHandle) -> *mut PosT;

    // Phase 5: Tabpage iteration
    fn nvim_mark_get_first_tabpage() -> TabHandle;
    fn nvim_mark_tabpage_next(tp: TabHandle) -> TabHandle;
    fn nvim_mark_tabpage_firstwin(tp: TabHandle) -> WinHandle;

    // Phase 5: External function callbacks
    fn nvim_mark_qf_mark_adjust(
        buf: BufHandle,
        win: WinHandle,
        line1: LinenrT,
        line2: LinenrT,
        amount: LinenrT,
        amount_after: LinenrT,
    ) -> c_int;
    fn nvim_mark_extmark_adjust(
        buf: BufHandle,
        line1: LinenrT,
        line2: LinenrT,
        amount: LinenrT,
        amount_after: LinenrT,
        op: c_int,
    );
    fn nvim_mark_diff_adjust(
        buf: BufHandle,
        line1: LinenrT,
        line2: LinenrT,
        amount: LinenrT,
        amount_after: LinenrT,
    );
    fn nvim_mark_fold_adjust(
        win: WinHandle,
        line1: LinenrT,
        line2: LinenrT,
        amount: LinenrT,
        amount_after: LinenrT,
    );

    // Phase 5: Wininfo iteration
    fn nvim_mark_buf_get_wininfo_count(buf: BufHandle) -> c_int;
    fn nvim_mark_buf_get_wininfo_mark(buf: BufHandle, idx: c_int) -> *mut PosT;

    // Phase 5: Jumplist/tagstack mark pointers for col_adjust
    fn nvim_mark_win_get_jumplist_mark_ptr(win: WinHandle, idx: c_int) -> *mut PosT;
    fn nvim_mark_win_get_tagstack_mark_ptr(win: WinHandle, idx: c_int) -> *mut PosT;

    // Phase 5: curtab
    fn nvim_mark_get_curtab() -> TabHandle;

    // Phase 6: Error message wrappers
    fn nvim_mark_emsg_invarg();
    fn nvim_mark_emsg_argreq();
    fn nvim_mark_semsg_invarg2(p: *const c_char);

    // Phase 6: Multibyte functions
    fn nvim_mark_ml_get_buf(buf: BufHandle, lnum: LinenrT) -> *const c_char;
    fn nvim_mark_ml_get_buf_len(buf: BufHandle, lnum: LinenrT) -> ColnrT;
    fn nvim_mark_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn nvim_mark_utf_ptr2char(p: *const c_char) -> c_int;
    fn nvim_mark_vim_isprintc(c: c_int) -> c_int;
    fn nvim_mark_ptr2cells(p: *const c_char) -> c_int;

    // Phase 6: Motion functions
    fn nvim_mark_findpar(
        inclusive: *mut c_int,
        dir: c_int,
        count: LinenrT,
        what: c_int,
        do_sentences: c_int,
    ) -> c_int;
    fn nvim_mark_findsent(dir: c_int, count: LinenrT) -> c_int;
    fn nvim_mark_set_listcmd_busy(val: c_int);
    fn nvim_mark_win_set_cursor(win: WinHandle, pos: PosT);
}

/// Number of possible named marks (a-z)
pub const NMARKS: c_int = 26;

/// pos_T structure matching Neovim's pos_defs.h
/// Position in file or buffer
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PosT {
    /// line number
    pub lnum: i32,
    /// column number
    pub col: i32,
    /// column add (for virtual columns)
    pub coladd: i32,
}

/// Check if a character is an ASCII uppercase letter (A-Z).
#[inline]
const fn ascii_isupper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// Check if a character is an ASCII lowercase letter (a-z).
#[inline]
const fn ascii_islower(c: u8) -> bool {
    c >= b'a' && c <= b'z'
}

/// Check if a character is an ASCII digit (0-9).
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Convert mark name to the global mark index.
///
/// Returns the offset for uppercase marks (A-Z) or digit marks (0-9),
/// or -1 if the name is not a valid global mark.
#[no_mangle]
pub extern "C" fn rs_mark_global_index(name: c_int) -> c_int {
    let Ok(c) = u8::try_from(name) else {
        return -1;
    };
    if ascii_isupper(c) {
        c_int::from(c - b'A')
    } else if ascii_isdigit(c) {
        NMARKS + c_int::from(c - b'0')
    } else {
        -1
    }
}

/// Convert local mark name to the offset.
///
/// Returns the offset for lowercase marks (a-z) or special marks (", ^, .),
/// or -1 if the name is not a valid local mark.
#[no_mangle]
pub extern "C" fn rs_mark_local_index(name: c_int) -> c_int {
    let Ok(c) = u8::try_from(name) else {
        return -1;
    };
    if ascii_islower(c) {
        c_int::from(c - b'a')
    } else if c == b'"' {
        NMARKS
    } else if c == b'^' {
        NMARKS + 1
    } else if c == b'.' {
        NMARKS + 2
    } else {
        -1
    }
}

/// Return true if position a is before (less than) position b.
#[no_mangle]
pub extern "C" fn rs_lt(a: PosT, b: PosT) -> c_int {
    let result = if a.lnum != b.lnum {
        a.lnum < b.lnum
    } else if a.col != b.col {
        a.col < b.col
    } else {
        a.coladd < b.coladd
    };
    c_int::from(result)
}

/// Return true if position a equals position b.
#[no_mangle]
pub extern "C" fn rs_equalpos(a: PosT, b: PosT) -> c_int {
    c_int::from(a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd)
}

/// Return true if position a is less than or equal to position b.
#[no_mangle]
pub extern "C" fn rs_ltoreq(a: PosT, b: PosT) -> c_int {
    c_int::from(rs_lt(a, b) != 0 || rs_equalpos(a, b) != 0)
}

/// Return true if position is empty (all fields are 0).
///
/// Matches the C macro: `EMPTY_POS(a) ((a).lnum == 0 && (a).col == 0 && (a).coladd == 0)`
#[no_mangle]
pub extern "C" fn rs_empty_pos(a: PosT) -> c_int {
    c_int::from(a.lnum == 0 && a.col == 0 && a.coladd == 0)
}

/// Clear a position by setting all fields to 0.
///
/// # Safety
///
/// `a` must be a valid, non-null pointer to a PosT struct.
#[no_mangle]
pub unsafe extern "C" fn rs_clearpos(a: *mut PosT) {
    if a.is_null() {
        return;
    }
    (*a).lnum = 0;
    (*a).col = 0;
    (*a).coladd = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_global_index() {
        // Uppercase letters A-Z map to 0-25
        assert_eq!(rs_mark_global_index(c_int::from(b'A')), 0);
        assert_eq!(rs_mark_global_index(c_int::from(b'Z')), 25);
        assert_eq!(rs_mark_global_index(c_int::from(b'M')), 12);

        // Digits 0-9 map to NMARKS + 0..9 (26-35)
        assert_eq!(rs_mark_global_index(c_int::from(b'0')), NMARKS);
        assert_eq!(rs_mark_global_index(c_int::from(b'9')), NMARKS + 9);
        assert_eq!(rs_mark_global_index(c_int::from(b'5')), NMARKS + 5);

        // Invalid marks return -1
        assert_eq!(rs_mark_global_index(c_int::from(b'a')), -1);
        assert_eq!(rs_mark_global_index(c_int::from(b'!')), -1);
        assert_eq!(rs_mark_global_index(-1), -1);
        assert_eq!(rs_mark_global_index(256), -1);
    }

    #[test]
    fn test_mark_local_index() {
        // Lowercase letters a-z map to 0-25
        assert_eq!(rs_mark_local_index(c_int::from(b'a')), 0);
        assert_eq!(rs_mark_local_index(c_int::from(b'z')), 25);
        assert_eq!(rs_mark_local_index(c_int::from(b'm')), 12);

        // Special marks
        assert_eq!(rs_mark_local_index(c_int::from(b'"')), NMARKS);
        assert_eq!(rs_mark_local_index(c_int::from(b'^')), NMARKS + 1);
        assert_eq!(rs_mark_local_index(c_int::from(b'.')), NMARKS + 2);

        // Invalid marks return -1
        assert_eq!(rs_mark_local_index(c_int::from(b'A')), -1);
        assert_eq!(rs_mark_local_index(c_int::from(b'0')), -1);
        assert_eq!(rs_mark_local_index(c_int::from(b'!')), -1);
        assert_eq!(rs_mark_local_index(-1), -1);
    }

    #[test]
    fn test_lt() {
        let pos1 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };
        let pos2 = PosT {
            lnum: 2,
            col: 3,
            coladd: 0,
        };
        let pos3 = PosT {
            lnum: 1,
            col: 10,
            coladd: 0,
        };
        let pos4 = PosT {
            lnum: 1,
            col: 5,
            coladd: 1,
        };
        let pos5 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };

        // Different lines
        assert_ne!(rs_lt(pos1, pos2), 0); // pos1 < pos2 (line 1 < line 2)
        assert_eq!(rs_lt(pos2, pos1), 0); // pos2 > pos1

        // Same line, different columns
        assert_ne!(rs_lt(pos1, pos3), 0); // pos1 < pos3 (col 5 < col 10)
        assert_eq!(rs_lt(pos3, pos1), 0); // pos3 > pos1

        // Same line and column, different coladd
        assert_ne!(rs_lt(pos1, pos4), 0); // pos1 < pos4 (coladd 0 < coladd 1)
        assert_eq!(rs_lt(pos4, pos1), 0); // pos4 > pos1

        // Equal positions
        assert_eq!(rs_lt(pos1, pos5), 0); // not less than
    }

    #[test]
    fn test_equalpos() {
        let pos1 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };
        let pos2 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };
        let pos3 = PosT {
            lnum: 1,
            col: 5,
            coladd: 1,
        };
        let pos4 = PosT {
            lnum: 1,
            col: 6,
            coladd: 0,
        };
        let pos5 = PosT {
            lnum: 2,
            col: 5,
            coladd: 0,
        };

        assert_ne!(rs_equalpos(pos1, pos2), 0); // equal
        assert_eq!(rs_equalpos(pos1, pos3), 0); // different coladd
        assert_eq!(rs_equalpos(pos1, pos4), 0); // different col
        assert_eq!(rs_equalpos(pos1, pos5), 0); // different lnum
    }

    #[test]
    fn test_ltoreq() {
        let pos1 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };
        let pos2 = PosT {
            lnum: 1,
            col: 5,
            coladd: 0,
        };
        let pos3 = PosT {
            lnum: 1,
            col: 10,
            coladd: 0,
        };
        let pos4 = PosT {
            lnum: 2,
            col: 1,
            coladd: 0,
        };

        // Equal positions
        assert_ne!(rs_ltoreq(pos1, pos2), 0);

        // Less than
        assert_ne!(rs_ltoreq(pos1, pos3), 0);
        assert_ne!(rs_ltoreq(pos1, pos4), 0);

        // Greater than
        assert_eq!(rs_ltoreq(pos3, pos1), 0);
        assert_eq!(rs_ltoreq(pos4, pos1), 0);
    }

    #[test]
    fn test_empty_pos() {
        // Empty position (all zeros)
        let empty = PosT {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        assert_ne!(rs_empty_pos(empty), 0);

        // Non-empty positions (at least one field non-zero)
        let non_empty1 = PosT {
            lnum: 1,
            col: 0,
            coladd: 0,
        };
        assert_eq!(rs_empty_pos(non_empty1), 0);

        let non_empty2 = PosT {
            lnum: 0,
            col: 1,
            coladd: 0,
        };
        assert_eq!(rs_empty_pos(non_empty2), 0);

        let non_empty3 = PosT {
            lnum: 0,
            col: 0,
            coladd: 1,
        };
        assert_eq!(rs_empty_pos(non_empty3), 0);

        let non_empty4 = PosT {
            lnum: 1,
            col: 5,
            coladd: 2,
        };
        assert_eq!(rs_empty_pos(non_empty4), 0);
    }

    #[test]
    fn test_clearpos() {
        // Clear a non-empty position
        let mut pos = PosT {
            lnum: 10,
            col: 5,
            coladd: 2,
        };
        unsafe { rs_clearpos(&mut pos) };
        assert_eq!(pos.lnum, 0);
        assert_eq!(pos.col, 0);
        assert_eq!(pos.coladd, 0);

        // Should be empty after clearing
        assert_ne!(rs_empty_pos(pos), 0);

        // Null pointer should be handled gracefully
        unsafe { rs_clearpos(std::ptr::null_mut()) };
    }

    #[test]
    fn test_nmarks_constant() {
        // Verify NMARKS matches C definition (26 named marks a-z)
        assert_eq!(NMARKS, 26);
    }

    #[test]
    fn test_ascii_helpers() {
        // Test ascii_isupper
        assert!(ascii_isupper(b'A'));
        assert!(ascii_isupper(b'Z'));
        assert!(ascii_isupper(b'M'));
        assert!(!ascii_isupper(b'a'));
        assert!(!ascii_isupper(b'z'));
        assert!(!ascii_isupper(b'0'));
        assert!(!ascii_isupper(b'@')); // before A
        assert!(!ascii_isupper(b'[')); // after Z

        // Test ascii_islower
        assert!(ascii_islower(b'a'));
        assert!(ascii_islower(b'z'));
        assert!(ascii_islower(b'm'));
        assert!(!ascii_islower(b'A'));
        assert!(!ascii_islower(b'Z'));
        assert!(!ascii_islower(b'0'));
        assert!(!ascii_islower(b'`')); // before a
        assert!(!ascii_islower(b'{')); // after z

        // Test ascii_isdigit
        assert!(ascii_isdigit(b'0'));
        assert!(ascii_isdigit(b'9'));
        assert!(ascii_isdigit(b'5'));
        assert!(!ascii_isdigit(b'a'));
        assert!(!ascii_isdigit(b'A'));
        assert!(!ascii_isdigit(b'/')); // before 0
        assert!(!ascii_isdigit(b':')); // after 9
    }

    #[test]
    fn test_pos_t_default() {
        // Default should be an empty position
        let pos = PosT::default();
        assert_eq!(pos.lnum, 0);
        assert_eq!(pos.col, 0);
        assert_eq!(pos.coladd, 0);
        assert_ne!(rs_empty_pos(pos), 0);
    }

    #[test]
    fn test_pos_t_clone_and_eq() {
        let pos1 = PosT {
            lnum: 10,
            col: 5,
            coladd: 2,
        };
        let pos2 = pos1;
        assert_eq!(pos1, pos2);

        let pos3 = PosT {
            lnum: 10,
            col: 5,
            coladd: 3,
        };
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_pos_t_debug() {
        let pos = PosT {
            lnum: 10,
            col: 5,
            coladd: 2,
        };
        let debug_str = format!("{pos:?}");
        assert!(debug_str.contains("lnum: 10"));
        assert!(debug_str.contains("col: 5"));
        assert!(debug_str.contains("coladd: 2"));
    }

    #[test]
    fn test_mark_global_index_all_uppercase() {
        // Test all uppercase letters map correctly
        for (i, c) in (b'A'..=b'Z').enumerate() {
            assert_eq!(
                rs_mark_global_index(c_int::from(c)),
                i as c_int,
                "Failed for {}",
                c as char
            );
        }
    }

    #[test]
    fn test_mark_global_index_all_digits() {
        // Test all digits map correctly
        for (i, c) in (b'0'..=b'9').enumerate() {
            assert_eq!(
                rs_mark_global_index(c_int::from(c)),
                NMARKS + i as c_int,
                "Failed for {}",
                c as char
            );
        }
    }

    #[test]
    fn test_mark_local_index_all_lowercase() {
        // Test all lowercase letters map correctly
        for (i, c) in (b'a'..=b'z').enumerate() {
            assert_eq!(
                rs_mark_local_index(c_int::from(c)),
                i as c_int,
                "Failed for {}",
                c as char
            );
        }
    }

    #[test]
    fn test_lt_negative_values() {
        // Test with negative values
        let pos1 = PosT {
            lnum: -1,
            col: 0,
            coladd: 0,
        };
        let pos2 = PosT {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        assert_ne!(rs_lt(pos1, pos2), 0); // -1 < 0
        assert_eq!(rs_lt(pos2, pos1), 0); // 0 > -1
    }

    #[test]
    fn test_position_comparison_transitivity() {
        // Test transitivity: if a < b and b < c, then a < c
        let a = PosT {
            lnum: 1,
            col: 0,
            coladd: 0,
        };
        let b = PosT {
            lnum: 2,
            col: 0,
            coladd: 0,
        };
        let c = PosT {
            lnum: 3,
            col: 0,
            coladd: 0,
        };

        assert_ne!(rs_lt(a, b), 0); // a < b
        assert_ne!(rs_lt(b, c), 0); // b < c
        assert_ne!(rs_lt(a, c), 0); // a < c (transitivity)
    }
}

// =============================================================================
// Phase 5: Mark System Foundation - Additional Functions
// =============================================================================

/// Number of file marks (A-Z + 0-9)
pub const NGLOBALMARKS: c_int = NMARKS + 10; // 36

/// Max value of local mark
pub const NMARK_LOCAL_MAX: c_int = 126; // Index of '~'

/// MarkGet enum values matching C
const MARK_BUF_LOCAL: c_int = 0;
const MARK_ALL_NO_RESOLVE: c_int = 2;

/// Check if a character is a valid named mark (a-z).
#[no_mangle]
pub extern "C" fn rs_mark_is_valid_named(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    ascii_islower(c)
}

/// Check if a character is a valid file mark (A-Z or 0-9).
#[no_mangle]
pub extern "C" fn rs_mark_is_file_mark(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    ascii_isupper(c) || ascii_isdigit(c)
}

/// Check if a mark name is a jump mark (' or `).
#[no_mangle]
pub extern "C" fn rs_mark_is_jump_mark(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    c == b'\'' || c == b'`'
}

/// Check if a mark name is a special mark.
#[no_mangle]
pub extern "C" fn rs_mark_is_special(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    matches!(
        c,
        b'"' | b'^' | b'.' | b'[' | b']' | b'<' | b'>' | b'\'' | b'`'
    )
}

/// Check if a mark name is a visual mark (< or >).
#[no_mangle]
pub extern "C" fn rs_mark_is_visual(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    c == b'<' || c == b'>'
}

/// Check if a mark name is the last cursor position mark (").
#[no_mangle]
pub extern "C" fn rs_mark_is_last_cursor(name: c_int) -> bool {
    name == c_int::from(b'"')
}

/// Check if a mark name is the last insert position mark (^).
#[no_mangle]
pub extern "C" fn rs_mark_is_last_insert(name: c_int) -> bool {
    name == c_int::from(b'^')
}

/// Check if a mark name is the last change position mark (.).
#[no_mangle]
pub extern "C" fn rs_mark_is_last_change(name: c_int) -> bool {
    name == c_int::from(b'.')
}

/// Check if a mark name is a sentence boundary mark ([ or ]).
#[no_mangle]
pub extern "C" fn rs_mark_is_sentence(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    c == b'[' || c == b']'
}

/// Check if a position is valid (non-zero line number).
#[no_mangle]
pub extern "C" fn rs_pos_is_valid(pos: PosT) -> c_int {
    c_int::from(pos.lnum > 0)
}

/// Check if a position line is in range for a given buffer line count.
#[no_mangle]
pub extern "C" fn rs_pos_in_range(pos: PosT, line_count: i32) -> c_int {
    c_int::from(pos.lnum > 0 && pos.lnum <= line_count)
}

/// Compare two positions and return -1, 0, or 1.
#[no_mangle]
pub extern "C" fn rs_pos_compare(a: PosT, b: PosT) -> c_int {
    if rs_lt(a, b) != 0 {
        -1
    } else if rs_equalpos(a, b) != 0 {
        0
    } else {
        1
    }
}

/// Copy position from source to destination.
///
/// # Safety
///
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_copy(dst: *mut PosT, src: *const PosT) {
    if !dst.is_null() && !src.is_null() {
        *dst = *src;
    }
}

/// Get the line number from a position.
#[no_mangle]
pub extern "C" fn rs_pos_get_lnum(pos: PosT) -> i32 {
    pos.lnum
}

/// Get the column number from a position.
#[no_mangle]
pub extern "C" fn rs_pos_get_col(pos: PosT) -> i32 {
    pos.col
}

/// Get the virtual column add from a position.
#[no_mangle]
pub extern "C" fn rs_pos_get_coladd(pos: PosT) -> i32 {
    pos.coladd
}

/// Set the line number in a position.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_set_lnum(pos: *mut PosT, lnum: i32) {
    if !pos.is_null() {
        (*pos).lnum = lnum;
    }
}

/// Set the column number in a position.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_set_col(pos: *mut PosT, col: i32) {
    if !pos.is_null() {
        (*pos).col = col;
    }
}

/// Set the virtual column add in a position.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_set_coladd(pos: *mut PosT, coladd: i32) {
    if !pos.is_null() {
        (*pos).coladd = coladd;
    }
}

// =============================================================================
// Phase 6: Mark Operations - Additional Functions
// =============================================================================

/// Get the display name for a mark character.
///
/// # Safety
///
/// The `buf` pointer must be valid and point to at least `buf_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_get_name(name: c_int, buf: *mut u8, buf_len: usize) {
    if buf.is_null() || buf_len < 2 {
        return;
    }

    let buf_slice = std::slice::from_raw_parts_mut(buf, buf_len);

    if name == -1 {
        // No mark
        buf_slice[0] = b'-';
        buf_slice[1] = 0;
    } else if let Ok(c) = u8::try_from(name) {
        buf_slice[0] = c;
        buf_slice[1] = 0;
    } else {
        buf_slice[0] = b'?';
        buf_slice[1] = 0;
    }
}

/// Get a category string for a mark.
/// Returns a static string identifying the mark category.
#[no_mangle]
pub extern "C" fn rs_mark_get_category(name: c_int) -> *const std::ffi::c_char {
    let Ok(c) = u8::try_from(name) else {
        return c"unknown".as_ptr();
    };

    if ascii_islower(c) {
        c"local".as_ptr()
    } else if ascii_isupper(c) {
        c"file".as_ptr()
    } else if ascii_isdigit(c) {
        c"numbered".as_ptr()
    } else if c == b'"' {
        c"cursor".as_ptr()
    } else if c == b'^' || c == b'.' {
        c"change".as_ptr()
    } else if c == b'[' || c == b']' {
        c"text".as_ptr()
    } else if c == b'<' || c == b'>' {
        c"visual".as_ptr()
    } else if c == b'\'' || c == b'`' {
        c"jump".as_ptr()
    } else {
        c"special".as_ptr()
    }
}

/// Check if mark name is user-settable (not automatic).
#[no_mangle]
pub extern "C" fn rs_mark_is_user_settable(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    // User can set named marks (a-z, A-Z) and some special marks
    ascii_islower(c) || ascii_isupper(c) || c == b'\'' || c == b'`' || c == b'<' || c == b'>'
}

/// Check if mark should be persisted to shada.
#[no_mangle]
pub extern "C" fn rs_mark_is_persistent(name: c_int) -> bool {
    let Ok(c) = u8::try_from(name) else {
        return false;
    };
    // Named marks (a-z, A-Z), numbered marks (0-9), and special marks (", ^, .)
    ascii_islower(c) || ascii_isupper(c) || ascii_isdigit(c) || c == b'"' || c == b'^' || c == b'.'
}

/// Create a new position with given values.
#[no_mangle]
pub extern "C" fn rs_pos_new(lnum: i32, col: i32, coladd: i32) -> PosT {
    PosT { lnum, col, coladd }
}

/// Create a zero position.
#[no_mangle]
pub extern "C" fn rs_pos_zero() -> PosT {
    PosT {
        lnum: 0,
        col: 0,
        coladd: 0,
    }
}

/// Adjust position line number by delta.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_adjust_line(pos: *mut PosT, delta: i32) {
    if !pos.is_null() {
        (*pos).lnum += delta;
    }
}

/// Adjust position column by delta.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_adjust_col(pos: *mut PosT, delta: i32) {
    if !pos.is_null() {
        (*pos).col += delta;
    }
}

/// Clamp a position to valid buffer bounds.
///
/// # Safety
///
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_clamp(pos: *mut PosT, max_lnum: i32, max_col: i32) {
    if pos.is_null() {
        return;
    }

    if (*pos).lnum < 1 {
        (*pos).lnum = 1;
    } else if (*pos).lnum > max_lnum {
        (*pos).lnum = max_lnum;
    }

    if (*pos).col < 0 {
        (*pos).col = 0;
    } else if (*pos).col > max_col {
        (*pos).col = max_col;
    }

    if (*pos).coladd < 0 {
        (*pos).coladd = 0;
    }
}

/// Get the distance (in lines) between two positions.
#[no_mangle]
pub extern "C" fn rs_pos_line_distance(a: PosT, b: PosT) -> i32 {
    (b.lnum - a.lnum).abs()
}

/// Check if two positions are on the same line.
#[no_mangle]
pub extern "C" fn rs_pos_same_line(a: PosT, b: PosT) -> c_int {
    c_int::from(a.lnum == b.lnum)
}

/// Swap two positions if a > b (ensure a <= b).
///
/// # Safety
///
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_pos_order(a: *mut PosT, b: *mut PosT) {
    if a.is_null() || b.is_null() {
        return;
    }
    if rs_lt(*b, *a) != 0 {
        std::ptr::swap(a, b);
    }
}

// =============================================================================
// Phase 5 & 6 Tests
// =============================================================================

#[cfg(test)]
mod phase56_tests {
    use super::*;

    #[test]
    fn test_mark_validation() {
        // Named mark validation
        assert!(rs_mark_is_valid_named(c_int::from(b'a')));
        assert!(rs_mark_is_valid_named(c_int::from(b'z')));
        assert!(!rs_mark_is_valid_named(c_int::from(b'A')));
        assert!(!rs_mark_is_valid_named(c_int::from(b'0')));

        // File mark validation
        assert!(rs_mark_is_file_mark(c_int::from(b'A')));
        assert!(rs_mark_is_file_mark(c_int::from(b'Z')));
        assert!(rs_mark_is_file_mark(c_int::from(b'0')));
        assert!(!rs_mark_is_file_mark(c_int::from(b'a')));

        // Jump mark validation
        assert!(rs_mark_is_jump_mark(c_int::from(b'\'')));
        assert!(rs_mark_is_jump_mark(c_int::from(b'`')));
        assert!(!rs_mark_is_jump_mark(c_int::from(b'a')));
    }

    #[test]
    fn test_mark_type_categorization() {
        // Special marks
        assert!(rs_mark_is_special(c_int::from(b'"')));
        assert!(rs_mark_is_special(c_int::from(b'^')));
        assert!(rs_mark_is_special(c_int::from(b'.')));
        assert!(rs_mark_is_special(c_int::from(b'[')));
        assert!(rs_mark_is_special(c_int::from(b']')));
        assert!(rs_mark_is_special(c_int::from(b'<')));
        assert!(rs_mark_is_special(c_int::from(b'>')));
        assert!(!rs_mark_is_special(c_int::from(b'a')));

        // Visual marks
        assert!(rs_mark_is_visual(c_int::from(b'<')));
        assert!(rs_mark_is_visual(c_int::from(b'>')));
        assert!(!rs_mark_is_visual(c_int::from(b'a')));

        // Sentence marks
        assert!(rs_mark_is_sentence(c_int::from(b'[')));
        assert!(rs_mark_is_sentence(c_int::from(b']')));
        assert!(!rs_mark_is_sentence(c_int::from(b'a')));
    }

    #[test]
    fn test_pos_constructors() {
        let pos = rs_pos_new(10, 5, 2);
        assert_eq!(pos.lnum, 10);
        assert_eq!(pos.col, 5);
        assert_eq!(pos.coladd, 2);

        let zero = rs_pos_zero();
        assert_eq!(zero.lnum, 0);
        assert_eq!(zero.col, 0);
        assert_eq!(zero.coladd, 0);
    }

    #[test]
    fn test_pos_getters() {
        let pos = rs_pos_new(10, 5, 2);
        assert_eq!(rs_pos_get_lnum(pos), 10);
        assert_eq!(rs_pos_get_col(pos), 5);
        assert_eq!(rs_pos_get_coladd(pos), 2);
    }

    #[test]
    fn test_pos_validity() {
        let valid = rs_pos_new(1, 0, 0);
        assert_ne!(rs_pos_is_valid(valid), 0);

        let invalid = rs_pos_new(0, 0, 0);
        assert_eq!(rs_pos_is_valid(invalid), 0);

        let negative = rs_pos_new(-1, 0, 0);
        assert_eq!(rs_pos_is_valid(negative), 0);
    }

    #[test]
    fn test_pos_in_range() {
        let pos = rs_pos_new(5, 0, 0);
        assert_ne!(rs_pos_in_range(pos, 10), 0);
        assert_eq!(rs_pos_in_range(pos, 4), 0);

        let zero = rs_pos_zero();
        assert_eq!(rs_pos_in_range(zero, 10), 0);
    }

    #[test]
    fn test_pos_compare() {
        let a = rs_pos_new(1, 0, 0);
        let b = rs_pos_new(2, 0, 0);
        let c = rs_pos_new(1, 0, 0);

        assert_eq!(rs_pos_compare(a, b), -1);
        assert_eq!(rs_pos_compare(b, a), 1);
        assert_eq!(rs_pos_compare(a, c), 0);
    }

    #[test]
    fn test_pos_same_line() {
        let a = rs_pos_new(1, 0, 0);
        let b = rs_pos_new(1, 5, 0);
        let c = rs_pos_new(2, 0, 0);

        assert_ne!(rs_pos_same_line(a, b), 0);
        assert_eq!(rs_pos_same_line(a, c), 0);
    }

    #[test]
    fn test_pos_line_distance() {
        let a = rs_pos_new(1, 0, 0);
        let b = rs_pos_new(5, 0, 0);
        assert_eq!(rs_pos_line_distance(a, b), 4);
        assert_eq!(rs_pos_line_distance(b, a), 4);
    }

    #[test]
    fn test_mark_persistence() {
        // Named marks (a-z) are persistent
        assert!(rs_mark_is_persistent(c_int::from(b'a')));
        assert!(rs_mark_is_persistent(c_int::from(b'z')));

        // File marks (A-Z) are persistent
        assert!(rs_mark_is_persistent(c_int::from(b'A')));
        assert!(rs_mark_is_persistent(c_int::from(b'Z')));

        // Numbered marks (0-9) are persistent
        assert!(rs_mark_is_persistent(c_int::from(b'0')));
        assert!(rs_mark_is_persistent(c_int::from(b'9')));

        // Special persistent marks
        assert!(rs_mark_is_persistent(c_int::from(b'"')));
        assert!(rs_mark_is_persistent(c_int::from(b'^')));
        assert!(rs_mark_is_persistent(c_int::from(b'.')));

        // Non-persistent marks
        assert!(!rs_mark_is_persistent(c_int::from(b'<')));
        assert!(!rs_mark_is_persistent(c_int::from(b'>')));
    }

    #[test]
    fn test_mark_user_settable() {
        // Named marks are user-settable
        assert!(rs_mark_is_user_settable(c_int::from(b'a')));
        assert!(rs_mark_is_user_settable(c_int::from(b'A')));

        // Jump marks are user-settable
        assert!(rs_mark_is_user_settable(c_int::from(b'\'')));
        assert!(rs_mark_is_user_settable(c_int::from(b'`')));

        // Visual marks are user-settable
        assert!(rs_mark_is_user_settable(c_int::from(b'<')));
        assert!(rs_mark_is_user_settable(c_int::from(b'>')));

        // Automatic marks are not user-settable
        assert!(!rs_mark_is_user_settable(c_int::from(b'"')));
        assert!(!rs_mark_is_user_settable(c_int::from(b'^')));
        assert!(!rs_mark_is_user_settable(c_int::from(b'.')));
    }
}

// =============================================================================
// Phase 1: Mark View and Memory Operations
// =============================================================================

/// linenr_T equivalent from Neovim
pub type LinenrT = i32;

/// MAXLNUM value - represents no view
pub const MAXLNUM: LinenrT = 0x7fffffff;

/// fmarkv_T structure matching Neovim's mark_defs.h
/// Represents view in which the mark was created
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FmarkvT {
    /// Amount of lines from the mark lnum to the top of the window.
    /// Use MAXLNUM to indicate that the mark does not have a view.
    pub topline_offset: LinenrT,
}

/// Create a new fmarkv_T with MAXLNUM (no view).
#[no_mangle]
pub extern "C" fn rs_fmarkv_init() -> FmarkvT {
    FmarkvT {
        topline_offset: MAXLNUM,
    }
}

/// Create an fmarkv_T from topline and position.
///
/// This stores the offset between the mark's line number and the window's
/// topline, allowing the view to be restored later.
///
/// # Arguments
/// * `topline` - The window's current topline
/// * `pos_lnum` - The mark's line number
///
/// # Returns
/// An fmarkv_T with the calculated topline offset
#[no_mangle]
pub extern "C" fn rs_mark_view_make(topline: LinenrT, pos_lnum: LinenrT) -> FmarkvT {
    FmarkvT {
        topline_offset: pos_lnum - topline,
    }
}

/// Calculate the topline to restore from a mark view.
///
/// This computes the topline based on the mark's line number and the stored
/// topline offset. Returns -1 if the view should not be restored (offset >= MAXLNUM
/// or calculated topline < 1).
///
/// # Arguments
/// * `mark_lnum` - The mark's line number
/// * `topline_offset` - The stored topline offset from fmarkv_T
///
/// # Returns
/// The topline to set, or -1 if view should not be restored
#[no_mangle]
pub extern "C" fn rs_mark_view_calc_topline(
    mark_lnum: LinenrT,
    topline_offset: LinenrT,
) -> LinenrT {
    // If topline_offset is MAXLNUM (no view) or negative, don't restore view
    if !(0..MAXLNUM).contains(&topline_offset) {
        return -1;
    }

    let topline = mark_lnum - topline_offset;
    if topline >= 1 {
        topline
    } else {
        -1
    }
}

/// Check if an fmarkv_T has a valid view.
#[no_mangle]
pub extern "C" fn rs_fmarkv_has_view(view: FmarkvT) -> c_int {
    c_int::from((0..MAXLNUM).contains(&view.topline_offset))
}

// =============================================================================
// Phase 1 Tests
// =============================================================================

#[cfg(test)]
mod phase1_tests {
    use super::*;

    #[test]
    fn test_fmarkv_init() {
        let view = rs_fmarkv_init();
        assert_eq!(view.topline_offset, MAXLNUM);
    }

    #[test]
    fn test_mark_view_make() {
        // Normal case: mark at line 10, topline at line 5
        let view = rs_mark_view_make(5, 10);
        assert_eq!(view.topline_offset, 5); // 10 - 5 = 5

        // Mark at topline
        let view = rs_mark_view_make(10, 10);
        assert_eq!(view.topline_offset, 0);

        // Mark above topline (shouldn't happen in practice, but handle it)
        let view = rs_mark_view_make(10, 5);
        assert_eq!(view.topline_offset, -5);
    }

    #[test]
    fn test_mark_view_calc_topline() {
        // Normal case: mark at line 10, offset 5 -> topline should be 5
        let topline = rs_mark_view_calc_topline(10, 5);
        assert_eq!(topline, 5);

        // Mark at line 10, offset 0 -> topline should be 10
        let topline = rs_mark_view_calc_topline(10, 0);
        assert_eq!(topline, 10);

        // MAXLNUM offset (no view) -> should return -1
        let topline = rs_mark_view_calc_topline(10, MAXLNUM);
        assert_eq!(topline, -1);

        // Negative offset -> should return -1
        let topline = rs_mark_view_calc_topline(10, -1);
        assert_eq!(topline, -1);

        // Calculated topline would be < 1 -> should return -1
        let topline = rs_mark_view_calc_topline(1, 5);
        assert_eq!(topline, -1); // 1 - 5 = -4, which is < 1
    }

    #[test]
    fn test_fmarkv_has_view() {
        // Valid view with offset 0
        let view = FmarkvT { topline_offset: 0 };
        assert_ne!(rs_fmarkv_has_view(view), 0);

        // Valid view with positive offset
        let view = FmarkvT { topline_offset: 10 };
        assert_ne!(rs_fmarkv_has_view(view), 0);

        // No view (MAXLNUM)
        let view = FmarkvT {
            topline_offset: MAXLNUM,
        };
        assert_eq!(rs_fmarkv_has_view(view), 0);

        // Invalid view (negative)
        let view = FmarkvT { topline_offset: -1 };
        assert_eq!(rs_fmarkv_has_view(view), 0);
    }

    #[test]
    fn test_view_roundtrip() {
        // Create a view at mark line 100, topline 50
        let view = rs_mark_view_make(50, 100);
        assert_eq!(view.topline_offset, 50);

        // Restore the view - should get topline 50
        let restored_topline = rs_mark_view_calc_topline(100, view.topline_offset);
        assert_eq!(restored_topline, 50);
    }
}

// =============================================================================
// Phase 2: Mark Structures and Validation
// =============================================================================

/// Timestamp type matching Neovim's time_defs.h
pub type Timestamp = u64;

/// colnr_T equivalent from Neovim
pub type ColnrT = i32;

/// MAXCOL value - represents maximum column
pub const MAXCOL: ColnrT = 0x7fffffff;

/// Opaque pointer to AdditionalData from C
#[repr(C)]
pub struct AdditionalData {
    _private: [u8; 0],
}

/// fmark_T structure matching Neovim's mark_defs.h
/// Structure defining single local mark
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FmarkT {
    /// Cursor position
    pub mark: PosT,
    /// File number
    pub fnum: c_int,
    /// Time when this mark was last set
    pub timestamp: Timestamp,
    /// View the mark was created on
    pub view: FmarkvT,
    /// Additional data from ShaDa file (opaque pointer)
    pub additional_data: *mut AdditionalData,
}

impl Default for FmarkT {
    fn default() -> Self {
        FmarkT {
            mark: PosT::default(),
            fnum: 0,
            timestamp: 0,
            view: FmarkvT {
                topline_offset: MAXLNUM,
            },
            additional_data: std::ptr::null_mut(),
        }
    }
}

/// xfmark_T structure matching Neovim's mark_defs.h
/// Structure defining extended mark (mark with file name attached)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct XfmarkT {
    /// Actual mark
    pub fmark: FmarkT,
    /// File name, used when fnum == 0
    pub fname: *mut std::ffi::c_char,
}

/// Mark validation result codes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkValidation {
    /// Mark is valid
    Valid = 0,
    /// Mark pointer is NULL (unknown mark)
    NullMark = 1,
    /// Mark line number is 0 (mark not set)
    NotSet = 2,
    /// Mark line number is negative (invalid)
    Negative = 3,
    /// Mark line number exceeds buffer line count
    OutOfBounds = 4,
}

/// Validate a mark's position.
///
/// Checks for:
/// - Line number <= 0 (mark not set or invalid)
///
/// # Arguments
/// * `mark_lnum` - The mark's line number
///
/// # Returns
/// MarkValidation indicating the result
#[no_mangle]
pub extern "C" fn rs_mark_validate_lnum(mark_lnum: LinenrT) -> MarkValidation {
    if mark_lnum == 0 {
        MarkValidation::NotSet
    } else if mark_lnum < 0 {
        MarkValidation::Negative
    } else {
        MarkValidation::Valid
    }
}

/// Validate a mark's line number against buffer bounds.
///
/// # Arguments
/// * `mark_lnum` - The mark's line number
/// * `buf_line_count` - The buffer's line count
///
/// # Returns
/// MarkValidation indicating the result
#[no_mangle]
pub extern "C" fn rs_mark_validate_bounds(
    mark_lnum: LinenrT,
    buf_line_count: LinenrT,
) -> MarkValidation {
    let lnum_valid = rs_mark_validate_lnum(mark_lnum);
    if lnum_valid != MarkValidation::Valid {
        return lnum_valid;
    }
    if mark_lnum > buf_line_count {
        MarkValidation::OutOfBounds
    } else {
        MarkValidation::Valid
    }
}

/// Check if a mark line number is valid (> 0).
#[no_mangle]
pub extern "C" fn rs_mark_lnum_is_valid(mark_lnum: LinenrT) -> c_int {
    c_int::from(mark_lnum > 0)
}

/// Check if a mark line number is within buffer bounds.
#[no_mangle]
pub extern "C" fn rs_mark_lnum_in_bounds(mark_lnum: LinenrT, buf_line_count: LinenrT) -> c_int {
    c_int::from(mark_lnum > 0 && mark_lnum <= buf_line_count)
}

/// Initialize an fmark_T with default values.
///
/// # Safety
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fmark_init(fm: *mut FmarkT) {
    if fm.is_null() {
        return;
    }
    (*fm).mark = PosT::default();
    (*fm).fnum = 0;
    (*fm).timestamp = 0;
    (*fm).view.topline_offset = MAXLNUM;
    (*fm).additional_data = std::ptr::null_mut();
}

/// Check if an fmark_T has a valid mark position (lnum > 0).
#[no_mangle]
pub extern "C" fn rs_fmark_is_set(fm: FmarkT) -> c_int {
    c_int::from(fm.mark.lnum > 0)
}

/// Get the line number from an fmark_T.
#[no_mangle]
pub extern "C" fn rs_fmark_get_lnum(fm: FmarkT) -> LinenrT {
    fm.mark.lnum
}

/// Get the column from an fmark_T.
#[no_mangle]
pub extern "C" fn rs_fmark_get_col(fm: FmarkT) -> ColnrT {
    fm.mark.col
}

/// Get the file number from an fmark_T.
#[no_mangle]
pub extern "C" fn rs_fmark_get_fnum(fm: FmarkT) -> c_int {
    fm.fnum
}

/// Get the timestamp from an fmark_T.
#[no_mangle]
pub extern "C" fn rs_fmark_get_timestamp(fm: FmarkT) -> Timestamp {
    fm.timestamp
}

/// Set the mark position in an fmark_T.
///
/// # Safety
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fmark_set_pos(fm: *mut FmarkT, lnum: LinenrT, col: ColnrT) {
    if fm.is_null() {
        return;
    }
    (*fm).mark.lnum = lnum;
    (*fm).mark.col = col;
}

/// Set the file number in an fmark_T.
///
/// # Safety
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fmark_set_fnum(fm: *mut FmarkT, fnum: c_int) {
    if fm.is_null() {
        return;
    }
    (*fm).fnum = fnum;
}

/// Set the timestamp in an fmark_T.
///
/// # Safety
/// The pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fmark_set_timestamp(fm: *mut FmarkT, timestamp: Timestamp) {
    if fm.is_null() {
        return;
    }
    (*fm).timestamp = timestamp;
}

/// Copy an fmark_T from source to destination.
///
/// # Safety
/// Both pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fmark_copy(dst: *mut FmarkT, src: *const FmarkT) {
    if dst.is_null() || src.is_null() {
        return;
    }
    // Don't copy additional_data pointer - that needs special handling
    (*dst).mark = (*src).mark;
    (*dst).fnum = (*src).fnum;
    (*dst).timestamp = (*src).timestamp;
    (*dst).view = (*src).view;
}

/// Compare two positions and determine visual order.
/// Returns which position should be considered "start" vs "end" for visual selection.
///
/// This implements the logic: if name == '<', return the lesser position;
/// if name == '>', return the greater position.
///
/// # Arguments
/// * `start_lnum`, `start_col` - First position (vi_start)
/// * `end_lnum`, `end_col` - Second position (vi_end)
/// * `name` - Mark name ('<' or '>')
///
/// # Returns
/// 0 to use start position, 1 to use end position
#[no_mangle]
pub extern "C" fn rs_visual_mark_select(
    start_lnum: LinenrT,
    start_col: ColnrT,
    end_lnum: LinenrT,
    end_col: ColnrT,
    name: c_int,
) -> c_int {
    let start = PosT {
        lnum: start_lnum,
        col: start_col,
        coladd: 0,
    };
    let end = PosT {
        lnum: end_lnum,
        col: end_col,
        coladd: 0,
    };

    let start_is_less = rs_lt(start, end) != 0;

    // '<' wants the lesser position, '>' wants the greater
    // But also handle edge cases: if end.lnum == 0 or start.lnum == 0
    let name_is_less = name == c_int::from(b'<');

    if end_lnum == 0 && start_lnum != 0 {
        // End is invalid, use start
        return 0;
    }

    if (name_is_less == start_is_less || end_lnum == 0) && start_lnum != 0 {
        0 // use start
    } else {
        1 // use end
    }
}

// =============================================================================
// Phase 2 Tests
// =============================================================================

#[cfg(test)]
mod phase2_tests {
    use super::*;

    #[test]
    fn test_mark_validate_lnum() {
        assert_eq!(rs_mark_validate_lnum(1), MarkValidation::Valid);
        assert_eq!(rs_mark_validate_lnum(100), MarkValidation::Valid);
        assert_eq!(rs_mark_validate_lnum(0), MarkValidation::NotSet);
        assert_eq!(rs_mark_validate_lnum(-1), MarkValidation::Negative);
    }

    #[test]
    fn test_mark_validate_bounds() {
        // Valid cases
        assert_eq!(rs_mark_validate_bounds(1, 100), MarkValidation::Valid);
        assert_eq!(rs_mark_validate_bounds(100, 100), MarkValidation::Valid);

        // Out of bounds
        assert_eq!(
            rs_mark_validate_bounds(101, 100),
            MarkValidation::OutOfBounds
        );

        // Invalid lnum
        assert_eq!(rs_mark_validate_bounds(0, 100), MarkValidation::NotSet);
        assert_eq!(rs_mark_validate_bounds(-1, 100), MarkValidation::Negative);
    }

    #[test]
    fn test_mark_lnum_checks() {
        assert_ne!(rs_mark_lnum_is_valid(1), 0);
        assert_eq!(rs_mark_lnum_is_valid(0), 0);
        assert_eq!(rs_mark_lnum_is_valid(-1), 0);

        assert_ne!(rs_mark_lnum_in_bounds(1, 100), 0);
        assert_ne!(rs_mark_lnum_in_bounds(100, 100), 0);
        assert_eq!(rs_mark_lnum_in_bounds(101, 100), 0);
        assert_eq!(rs_mark_lnum_in_bounds(0, 100), 0);
    }

    #[test]
    fn test_fmark_default() {
        let fm = FmarkT::default();
        assert_eq!(fm.mark.lnum, 0);
        assert_eq!(fm.mark.col, 0);
        assert_eq!(fm.fnum, 0);
        assert_eq!(fm.timestamp, 0);
        assert_eq!(fm.view.topline_offset, MAXLNUM);
        assert!(fm.additional_data.is_null());
    }

    #[test]
    fn test_fmark_init() {
        let mut fm = FmarkT {
            mark: PosT {
                lnum: 10,
                col: 5,
                coladd: 2,
            },
            fnum: 1,
            timestamp: 12345,
            view: FmarkvT { topline_offset: 3 },
            additional_data: std::ptr::null_mut(),
        };

        unsafe { rs_fmark_init(&mut fm) };

        assert_eq!(fm.mark.lnum, 0);
        assert_eq!(fm.mark.col, 0);
        assert_eq!(fm.fnum, 0);
        assert_eq!(fm.timestamp, 0);
        assert_eq!(fm.view.topline_offset, MAXLNUM);
    }

    #[test]
    fn test_fmark_is_set() {
        let mut fm = FmarkT::default();
        assert_eq!(rs_fmark_is_set(fm), 0);

        fm.mark.lnum = 1;
        assert_ne!(rs_fmark_is_set(fm), 0);

        fm.mark.lnum = -1;
        assert_eq!(rs_fmark_is_set(fm), 0);
    }

    #[test]
    fn test_fmark_getters() {
        let fm = FmarkT {
            mark: PosT {
                lnum: 10,
                col: 5,
                coladd: 2,
            },
            fnum: 3,
            timestamp: 12345,
            view: FmarkvT { topline_offset: 3 },
            additional_data: std::ptr::null_mut(),
        };

        assert_eq!(rs_fmark_get_lnum(fm), 10);
        assert_eq!(rs_fmark_get_col(fm), 5);
        assert_eq!(rs_fmark_get_fnum(fm), 3);
        assert_eq!(rs_fmark_get_timestamp(fm), 12345);
    }

    #[test]
    fn test_fmark_setters() {
        let mut fm = FmarkT::default();

        unsafe {
            rs_fmark_set_pos(&mut fm, 10, 5);
            rs_fmark_set_fnum(&mut fm, 3);
            rs_fmark_set_timestamp(&mut fm, 12345);
        }

        assert_eq!(fm.mark.lnum, 10);
        assert_eq!(fm.mark.col, 5);
        assert_eq!(fm.fnum, 3);
        assert_eq!(fm.timestamp, 12345);
    }

    #[test]
    fn test_fmark_copy() {
        let src = FmarkT {
            mark: PosT {
                lnum: 10,
                col: 5,
                coladd: 2,
            },
            fnum: 3,
            timestamp: 12345,
            view: FmarkvT { topline_offset: 7 },
            additional_data: std::ptr::null_mut(),
        };
        let mut dst = FmarkT::default();

        unsafe { rs_fmark_copy(&mut dst, &src) };

        assert_eq!(dst.mark.lnum, 10);
        assert_eq!(dst.mark.col, 5);
        assert_eq!(dst.mark.coladd, 2);
        assert_eq!(dst.fnum, 3);
        assert_eq!(dst.timestamp, 12345);
        assert_eq!(dst.view.topline_offset, 7);
    }

    #[test]
    fn test_visual_mark_select() {
        // '<' should select lesser position
        // start < end, name = '<' -> use start (0)
        assert_eq!(rs_visual_mark_select(1, 0, 10, 0, c_int::from(b'<')), 0);

        // start > end, name = '<' -> use end (1)
        assert_eq!(rs_visual_mark_select(10, 0, 1, 0, c_int::from(b'<')), 1);

        // '>' should select greater position
        // start < end, name = '>' -> use end (1)
        assert_eq!(rs_visual_mark_select(1, 0, 10, 0, c_int::from(b'>')), 1);

        // start > end, name = '>' -> use start (0)
        assert_eq!(rs_visual_mark_select(10, 0, 1, 0, c_int::from(b'>')), 0);

        // Edge case: end.lnum == 0, start.lnum != 0 -> use start
        assert_eq!(rs_visual_mark_select(5, 0, 0, 0, c_int::from(b'<')), 0);
        assert_eq!(rs_visual_mark_select(5, 0, 0, 0, c_int::from(b'>')), 0);
    }
}

// =============================================================================
// Phase 3 & 5: Jumplist and Changelist Operations
// =============================================================================

/// Maximum number of marks in jump list
pub const JUMPLISTSIZE: c_int = 100;

/// Maximum number of marks in change list
pub const GETMARKLIST_MAXCHANGES: c_int = 100;

/// CMOD_KEEPJUMPS flag value (from ex_cmds_defs.h)
const CMOD_KEEPJUMPS: c_uint = 0x0400;

/// kOptJopFlagStack flag value (from option_vars.generated.h)
const K_OPT_JOP_FLAG_STACK: c_uint = 0x01;

/// CMOD_LOCKMARKS flag value (from ex_cmds_defs.h)
const CMOD_LOCKMARKS: c_uint = 0x0800;

/// MarkAdjustMode values (from mark_defs.h)
const MARK_ADJUST_API: c_int = 1;
const MARK_ADJUST_TERM: c_int = 2;

/// ExtmarkOp values (from extmark_defs.h)
const EXTMARK_NOOP: c_int = 0;

/// Buffer quickfix flags (from buffer_defs.h)
const BUF_HAS_QF_ENTRY: c_int = 1;
const BUF_HAS_LL_ENTRY: c_int = 2;

/// Calculate the new jumplist length after incrementing.
///
/// Implements the logic: if ++len > JUMPLISTSIZE, len = JUMPLISTSIZE
///
/// # Arguments
/// * `current_len` - Current jumplist length
///
/// # Returns
/// New jumplist length (clamped to JUMPLISTSIZE)
#[no_mangle]
pub extern "C" fn rs_jumplist_new_len(current_len: c_int) -> c_int {
    let new_len = current_len + 1;
    if new_len > JUMPLISTSIZE {
        JUMPLISTSIZE
    } else {
        new_len
    }
}

/// Check if jumplist is full and needs oldest entry removed.
///
/// # Arguments
/// * `current_len` - Current jumplist length before increment
///
/// # Returns
/// 1 if full (oldest entry should be removed), 0 otherwise
#[no_mangle]
pub extern "C" fn rs_jumplist_is_full(current_len: c_int) -> c_int {
    c_int::from(current_len >= JUMPLISTSIZE)
}

/// Calculate jumplist trim length for stack mode.
///
/// When jumpoptions=stack, discard everything after current index.
///
/// # Arguments
/// * `idx` - Current jumplist index
/// * `len` - Current jumplist length
///
/// # Returns
/// New length if trim needed, or -1 if no trim needed
#[no_mangle]
pub extern "C" fn rs_jumplist_stack_trim(idx: c_int, len: c_int) -> c_int {
    if idx < len - 1 {
        idx + 1
    } else {
        -1 // No trim needed
    }
}

/// Calculate new jumplist index after a jump.
///
/// # Arguments
/// * `current_idx` - Current jumplist index
/// * `current_len` - Current jumplist length
/// * `count` - Jump count (negative for backward, positive for forward)
///
/// # Returns
/// New index, or -1 if out of bounds
#[no_mangle]
pub extern "C" fn rs_jumplist_calc_idx(
    current_idx: c_int,
    current_len: c_int,
    count: c_int,
) -> c_int {
    let new_idx = current_idx + count;
    if new_idx < 0 || new_idx >= current_len {
        -1
    } else {
        new_idx
    }
}

/// Calculate new changelist index after navigation.
///
/// # Arguments
/// * `current_idx` - Current changelist index
/// * `changelist_len` - Changelist length
/// * `count` - Navigation count (negative for backward, positive for forward)
///
/// # Returns
/// (new_idx, clamped) - new_idx is the calculated index, clamped indicates if the
/// value was clamped to bounds. Returns (-1, 0) if navigation not possible.
#[no_mangle]
pub extern "C" fn rs_changelist_calc_idx(
    current_idx: c_int,
    changelist_len: c_int,
    count: c_int,
) -> c_int {
    let n = current_idx;
    if n + count < 0 {
        if n == 0 {
            return -1; // Can't navigate further back
        }
        return 0; // Clamp to start
    } else if n + count >= changelist_len {
        if n == changelist_len - 1 {
            return -1; // Can't navigate further forward
        }
        return changelist_len - 1; // Clamp to end
    }
    n + count
}

/// Determine the target mark based on mark name.
///
/// Returns a code indicating which mark storage should be used:
/// - 0: Invalid/not handled
/// - 1: Global mark (A-Z, 0-9)
/// - 2: Local named mark (a-z)
/// - 3: Jump mark (' or `)
/// - 4: Last cursor mark (")
/// - 5: Sentence start ([)
/// - 6: Sentence end (])
/// - 7: Visual start (<)
/// - 8: Visual end (>)
/// - 9: Last insert (^)
/// - 10: Last change (.)
/// - 11: Prompt mark (:)
#[no_mangle]
pub extern "C" fn rs_mark_target_type(name: c_int) -> c_int {
    let Ok(c) = u8::try_from(name) else {
        return 0;
    };

    if ascii_isupper(c) || ascii_isdigit(c) {
        1 // Global mark
    } else if ascii_islower(c) {
        2 // Local named mark
    } else {
        match c {
            b'\'' | b'`' => 3, // Jump mark
            b'"' => 4,         // Last cursor
            b'[' => 5,         // Sentence start
            b']' => 6,         // Sentence end
            b'<' => 7,         // Visual start
            b'>' => 8,         // Visual end
            b'^' => 9,         // Last insert
            b'.' => 10,        // Last change
            b':' => 11,        // Prompt mark
            _ => 0,            // Not handled
        }
    }
}

/// Position clamp operation for mark setting.
///
/// Ensures lnum is at least 1 (valid for Vim positions).
#[no_mangle]
pub extern "C" fn rs_pos_clamp_lnum_min(lnum: LinenrT) -> LinenrT {
    if lnum < 1 {
        1
    } else {
        lnum
    }
}

// =============================================================================
// Phase 4: Jumplist/Changelist Navigation
// =============================================================================

/// Set the previous context mark to the current position and add it to the
/// jump list.
///
/// # Safety
/// `win` and `buf` must be valid pointers to the current window and buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_setpcmark(win: WinHandle, buf: BufHandle) {
    // for :global the mark is set only once
    if nvim_mark_get_global_busy() != 0
        || nvim_mark_get_listcmd_busy() != 0
        || (nvim_mark_get_cmod_flags() & CMOD_KEEPJUMPS) != 0
    {
        return;
    }

    let cursor = nvim_mark_win_get_cursor(win);
    let pcmark = nvim_mark_win_get_pcmark(win);
    nvim_mark_win_set_prev_pcmark(win, pcmark);
    let mut new_pcmark = cursor;
    new_pcmark.lnum = rs_pos_clamp_lnum_min(new_pcmark.lnum);
    nvim_mark_win_set_pcmark(win, new_pcmark);

    let mut jumplistlen = nvim_mark_win_get_jumplistlen(win);
    let jumplistidx = nvim_mark_win_get_jumplistidx(win);

    if (nvim_mark_get_jop_flags() & K_OPT_JOP_FLAG_STACK) != 0 {
        // jumpoptions=stack: discard everything after current index
        let trim_len = rs_jumplist_stack_trim(jumplistidx, jumplistlen);
        if trim_len >= 0 {
            jumplistlen = trim_len;
            nvim_mark_win_set_jumplistlen(win, jumplistlen);
        }
    }

    let is_full = jumplistlen >= JUMPLISTSIZE;
    jumplistlen = rs_jumplist_new_len(jumplistlen);
    nvim_mark_win_set_jumplistlen(win, jumplistlen);

    // If jumplist is full: remove oldest entry
    if is_full {
        rs_free_xfmark(*nvim_mark_win_get_jumplist_entry(win, 0));
        nvim_mark_win_jumplist_shift_down(win);
    }

    nvim_mark_win_set_jumplistidx(win, jumplistlen);

    let new_pcmark_val = nvim_mark_win_get_pcmark(win);
    let topline = nvim_mark_win_get_topline(win);
    let view = rs_mark_view_make(topline, new_pcmark_val.lnum);
    let fnum = nvim_buf_get_fnum(buf);
    nvim_mark_win_set_jumplist_xfmark(win, jumplistlen - 1, new_pcmark_val, fnum, view);
}

/// Get mark in "count" position in the jumplist relative to the current index.
///
/// If the mark is in a different buffer, it will be skipped unless the buffer exists.
/// Calls cleanup_jumplist and potentially setpcmark.
///
/// # Safety
/// `win` and `curbuf_ptr` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_get_jumplist(
    win: WinHandle,
    curbuf_ptr: BufHandle,
    count: c_int,
) -> *mut FmarkT {
    rs_cleanup_jumplist(win, 1);

    if nvim_mark_win_get_jumplistlen(win) == 0 {
        return std::ptr::null_mut();
    }

    let mut count = count;
    loop {
        let idx = nvim_mark_win_get_jumplistidx(win);
        let len = nvim_mark_win_get_jumplistlen(win);

        if idx + count < 0 || idx + count >= len {
            return std::ptr::null_mut();
        }

        // if first CTRL-O or CTRL-I command after a jump, add cursor position
        // to list. Careful: If there are duplicates (CTRL-O immediately after
        // starting Vim on a file), another entry may have been removed.
        if idx == len {
            nvim_mark_setpcmark();
            let new_idx = nvim_mark_win_get_jumplistidx(win) - 1;
            nvim_mark_win_set_jumplistidx(win, new_idx);
            if new_idx + count < 0 {
                return std::ptr::null_mut();
            }
        }

        let new_idx = nvim_mark_win_get_jumplistidx(win) + count;
        nvim_mark_win_set_jumplistidx(win, new_idx);

        let jmp = nvim_mark_win_get_jumplist_entry(win, new_idx);
        if (*jmp).fmark.fnum == 0 {
            // Resolve the fnum (buff number) in the mark before returning it (shada)
            nvim_mark_fname2fnum(jmp);
        }
        let curbuf_fnum = nvim_buf_get_fnum(curbuf_ptr);
        if (*jmp).fmark.fnum != curbuf_fnum {
            // Needs to switch buffer, if it can't find it skip the mark
            let found_buf = nvim_mark_buflist_findnr((*jmp).fmark.fnum);
            if found_buf.is_null() {
                count += if count < 0 { -1 } else { 1 };
                continue;
            }
        }
        return &mut (*jmp).fmark;
    }
}

/// Get mark in "count" position in the changelist relative to the current index.
///
/// # Safety
/// `buf` and `win` must be valid pointers.
#[export_name = "get_changelist"]
pub unsafe extern "C" fn rs_get_changelist(
    buf: BufHandle,
    win: WinHandle,
    count: c_int,
) -> *mut FmarkT {
    let changelistlen = nvim_mark_buf_get_changelistlen(buf);
    if changelistlen == 0 {
        return std::ptr::null_mut();
    }

    let n = nvim_mark_win_get_changelistidx(win);
    let new_n = rs_changelist_calc_idx(n, changelistlen, count);
    if new_n < 0 {
        return std::ptr::null_mut();
    }

    nvim_mark_win_set_changelistidx(win, new_n);
    let fm = nvim_mark_buf_get_changelist(buf, new_n);
    // Changelist marks are always buffer local
    let buf_handle = nvim_buf_get_handle(buf);
    (*fm).fnum = buf_handle;
    fm
}

/// Clean up the jumplist, removing duplicate entries.
///
/// When `loadfiles` is true, resolve all fnum values first.
///
/// # Safety
/// `wp` must be a valid window pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_jumplist(wp: WinHandle, loadfiles: c_int) {
    let loadfiles = loadfiles != 0;

    if loadfiles {
        // Load all files from the jump list to properly clean up duplicates
        let len = nvim_mark_win_get_jumplistlen(wp);
        for i in 0..len {
            let entry = nvim_mark_win_get_jumplist_entry(wp, i);
            if (*entry).fmark.fnum == 0 && (*entry).fmark.mark.lnum != 0 {
                nvim_mark_fname2fnum(entry);
            }
        }
    }

    let mut to = 0;
    let len = nvim_mark_win_get_jumplistlen(wp);
    let jop_flags = nvim_mark_get_jop_flags();

    for from in 0..len {
        if nvim_mark_win_get_jumplistidx(wp) == from {
            nvim_mark_win_set_jumplistidx(wp, to);
        }

        // Check if this entry is a duplicate of a later entry
        let from_fnum = nvim_mark_win_get_jumplist_fnum(wp, from);
        let from_lnum = nvim_mark_win_get_jumplist_lnum(wp, from);

        let mut dup_idx = len; // no duplicate found
        for i in (from + 1)..len {
            if nvim_mark_win_get_jumplist_fnum(wp, i) == from_fnum
                && from_fnum != 0
                && nvim_mark_win_get_jumplist_lnum(wp, i) == from_lnum
            {
                dup_idx = i;
                break;
            }
        }

        let mustfree;
        if dup_idx >= len {
            // not duplicate
            mustfree = false;
        } else if dup_idx > from + 1 {
            // non-adjacent duplicate
            // jumpoptions=stack: remove duplicates only when adjacent
            mustfree = (jop_flags & K_OPT_JOP_FLAG_STACK) == 0;
        } else {
            // adjacent duplicate
            mustfree = true;
        }

        if mustfree {
            nvim_mark_win_jumplist_free_fname(wp, from);
        } else {
            if to != from {
                nvim_mark_win_jumplist_copy_entry(wp, to, from);
            }
            to += 1;
        }
    }
    if nvim_mark_win_get_jumplistidx(wp) == len {
        nvim_mark_win_set_jumplistidx(wp, to);
    }
    nvim_mark_win_set_jumplistlen(wp, to);

    // When pointer is below last jump, remove the jump if it matches the current
    // line. This avoids useless/phantom jumps. #9805
    let new_len = nvim_mark_win_get_jumplistlen(wp);
    let new_idx = nvim_mark_win_get_jumplistidx(wp);
    if loadfiles && new_len > 0 && new_idx == new_len {
        let curbuf_ptr = nvim_mark_get_curbuf();
        let curbuf_fnum = nvim_buf_get_fnum(curbuf_ptr);
        let cursor_lnum = nvim_mark_win_get_cursor(wp).lnum;
        let last_fnum = nvim_mark_win_get_jumplist_fnum(wp, new_len - 1);
        let last_lnum = nvim_mark_win_get_jumplist_lnum(wp, new_len - 1);
        if last_fnum == curbuf_fnum && last_lnum == cursor_lnum {
            nvim_mark_win_jumplist_free_fname(wp, new_len - 1);
            nvim_mark_win_set_jumplistlen(wp, new_len - 1);
            nvim_mark_win_set_jumplistidx(wp, new_idx - 1);
        }
    }
}

/// Remove all jump list entries that match the given buffer fnum.
///
/// # Safety
/// `wp` must be a valid window pointer.
#[export_name = "mark_jumplist_forget_file"]
pub unsafe extern "C" fn rs_mark_jumplist_forget_file(wp: WinHandle, fnum: c_int) {
    let mut i = nvim_mark_win_get_jumplistlen(wp) - 1;
    while i >= 0 {
        if nvim_mark_win_get_jumplist_fnum(wp, i) == fnum {
            // Free the entry
            rs_free_xfmark(*nvim_mark_win_get_jumplist_entry(wp, i));

            // If the current jump list index is behind the entry, move it back
            if nvim_mark_win_get_jumplistidx(wp) > i {
                nvim_mark_win_set_jumplistidx(wp, nvim_mark_win_get_jumplistidx(wp) - 1);
            }

            // Remove the entry from the jump list
            let new_len = nvim_mark_win_get_jumplistlen(wp) - 1;
            nvim_mark_win_set_jumplistlen(wp, new_len);
            nvim_mark_win_jumplist_remove(wp, i, new_len);
        }
        i -= 1;
    }
}

/// Delete every entry referring to file "fnum" from both the jumplist and the
/// tag stack.
///
/// # Safety
/// `wp` must be a valid window pointer.
#[export_name = "mark_forget_file"]
pub unsafe extern "C" fn rs_mark_forget_file(wp: WinHandle, fnum: c_int) {
    rs_mark_jumplist_forget_file(wp, fnum);

    // Remove all tag stack entries that match the deleted buffer
    let mut i = nvim_mark_win_get_tagstacklen(wp) - 1;
    while i >= 0 {
        if nvim_mark_win_get_tagstack_fnum(wp, i) == fnum {
            nvim_mark_win_tagstack_clear_entry(wp, i);

            if nvim_mark_win_get_tagstackidx(wp) > i {
                nvim_mark_win_set_tagstackidx(wp, nvim_mark_win_get_tagstackidx(wp) - 1);
            }

            let new_len = nvim_mark_win_get_tagstacklen(wp) - 1;
            nvim_mark_win_set_tagstacklen(wp, new_len);
            nvim_mark_win_tagstack_remove(wp, i, new_len);
        }
        i -= 1;
    }
}

/// Find the next named mark in the given direction from startpos.
///
/// # Safety
/// `startpos` and `curbuf_ptr` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_getnextmark(
    startpos: *mut PosT,
    dir: c_int,
    begin_line: c_int,
    curbuf_ptr: BufHandle,
) -> *mut FmarkT {
    let mut result: *mut FmarkT = std::ptr::null_mut();
    let mut pos = *startpos;

    // Adjust column based on direction and begin_line
    pos.col = rs_getnextmark_adjust_col(pos.col, dir, begin_line);

    for i in 0..NMARKS {
        let namedm = nvim_mark_buf_get_namedm(curbuf_ptr, i);
        let result_lnum = if result.is_null() {
            0
        } else {
            (*result).mark.lnum
        };
        let result_col = if result.is_null() {
            0
        } else {
            (*result).mark.col
        };
        if rs_getnextmark_is_better(
            (*namedm).mark.lnum,
            (*namedm).mark.col,
            result_lnum,
            result_col,
            pos.lnum,
            pos.col,
            dir,
        ) != 0
        {
            result = namedm;
        }
    }

    result
}

// =============================================================================
// Phase 5: Mark Adjustment (Core)
// =============================================================================

/// Helper: apply ONE_ADJUST logic to a lnum pointer
unsafe fn one_adjust(
    lp: *mut LinenrT,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) {
    let result = rs_mark_adjust_lnum(*lp, line1, line2, amount, amount_after);
    if result.modified != 0 {
        *lp = result.new_lnum;
    }
}

/// Helper: apply ONE_ADJUST_NODEL logic to a lnum pointer
unsafe fn one_adjust_nodel(
    lp: *mut LinenrT,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) {
    let result = rs_mark_adjust_lnum_nodel(*lp, line1, line2, amount, amount_after);
    if result.modified != 0 {
        *lp = result.new_lnum;
    }
}

/// Helper: apply ONE_ADJUST_CURSOR logic to a pos pointer
unsafe fn one_adjust_cursor(
    pp: *mut PosT,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) {
    let result = rs_mark_adjust_cursor((*pp).lnum, (*pp).col, line1, line2, amount, amount_after);
    if result.modified != 0 {
        (*pp).lnum = result.new_lnum;
        (*pp).col = result.new_col;
    }
}

/// Helper: apply COL_ADJUST logic to a pos pointer
unsafe fn col_adjust(
    pp: *mut PosT,
    lnum: LinenrT,
    mincol: ColnrT,
    lnum_amount: LinenrT,
    col_amount: ColnrT,
    spaces_removed: c_int,
) {
    let result = rs_mark_col_adjust(
        (*pp).lnum,
        (*pp).col,
        lnum,
        mincol,
        lnum_amount,
        col_amount,
        spaces_removed,
    );
    if result.modified != 0 {
        (*pp).lnum = result.new_lnum;
        (*pp).col = result.new_col;
    }
}

/// Adjust marks between line1 and line2 (inclusive) to move amount lines.
///
/// Called from many places to adjust all marks when lines are inserted/deleted.
/// This is the highest-risk function in the mark migration.
///
/// # Safety
/// `buf` must be a valid buffer pointer. All window/tabpage pointers accessed
/// via C accessors must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_adjust_buf(
    buf: BufHandle,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
    adjust_folds: c_int,
    mode: c_int,
    op: c_int,
) {
    let fnum = nvim_buf_get_fnum(buf);
    let initpos = PosT {
        lnum: 1,
        col: 0,
        coladd: 0,
    };

    if line2 < line1 && amount_after == 0 {
        return; // nothing to do
    }

    let by_api = mode == MARK_ADJUST_API;
    let by_term = mode == MARK_ADJUST_TERM;
    let lockmarks = (nvim_mark_get_cmod_flags() & CMOD_LOCKMARKS) != 0;

    if !lockmarks {
        // named marks, lower case and upper case
        for i in 0..NMARKS {
            let namedm = nvim_mark_buf_get_namedm(buf, i);
            one_adjust(&mut (*namedm).mark.lnum, line1, line2, amount, amount_after);

            let namedfm_ptr = nvim_mark_get_namedfm();
            let gmark = &mut *namedfm_ptr.offset(i as isize);
            if gmark.fmark.fnum == fnum {
                one_adjust_nodel(
                    &mut gmark.fmark.mark.lnum,
                    line1,
                    line2,
                    amount,
                    amount_after,
                );
            }
        }
        for i in NMARKS..NGLOBALMARKS {
            let namedfm_ptr = nvim_mark_get_namedfm();
            let gmark = &mut *namedfm_ptr.offset(i as isize);
            if gmark.fmark.fnum == fnum {
                one_adjust_nodel(
                    &mut gmark.fmark.mark.lnum,
                    line1,
                    line2,
                    amount,
                    amount_after,
                );
            }
        }

        // last Insert position
        let last_insert = nvim_mark_buf_get_last_insert(buf);
        one_adjust(
            &mut (*last_insert).mark.lnum,
            line1,
            line2,
            amount,
            amount_after,
        );

        // last change position
        let last_change = nvim_mark_buf_get_last_change(buf);
        one_adjust(
            &mut (*last_change).mark.lnum,
            line1,
            line2,
            amount,
            amount_after,
        );

        // last cursor position, if it was set
        let last_cursor = nvim_mark_buf_get_last_cursor(buf);
        let lc_pos = (*last_cursor).mark;
        if !(lc_pos.lnum == initpos.lnum
            && lc_pos.col == initpos.col
            && lc_pos.coladd == initpos.coladd)
            && (!by_term || (*last_cursor).mark.lnum < nvim_buf_get_ml_line_count(buf))
        {
            one_adjust(
                &mut (*last_cursor).mark.lnum,
                line1,
                line2,
                amount,
                amount_after,
            );
        }

        // on prompt buffer adjust the last prompt start location mark
        if nvim_mark_bt_prompt(buf) != 0 {
            let prompt_start = nvim_mark_buf_get_prompt_start(buf);
            one_adjust_nodel(
                &mut (*prompt_start).mark.lnum,
                line1,
                line2,
                amount,
                amount_after,
            );
        }

        // list of change positions
        let changelistlen = nvim_mark_buf_get_changelistlen(buf);
        for i in 0..changelistlen {
            let cl = nvim_mark_buf_get_changelist(buf, i);
            one_adjust_nodel(&mut (*cl).mark.lnum, line1, line2, amount, amount_after);
        }

        // Visual area
        let vi_start = nvim_mark_buf_get_visual_start_ptr(buf);
        one_adjust_nodel(&mut (*vi_start).lnum, line1, line2, amount, amount_after);
        let vi_end = nvim_mark_buf_get_visual_end_ptr(buf);
        one_adjust_nodel(&mut (*vi_end).lnum, line1, line2, amount, amount_after);

        // quickfix marks
        let qf_result =
            nvim_mark_qf_mark_adjust(buf, WinHandle::null(), line1, line2, amount, amount_after);
        if qf_result == 0 {
            let has_qf = nvim_mark_buf_get_has_qf_entry(buf);
            nvim_mark_buf_set_has_qf_entry(buf, has_qf & !BUF_HAS_QF_ENTRY);
        }

        // location lists
        let mut found_one = false;
        let mut tp = nvim_mark_get_first_tabpage();
        while !tp.is_null() {
            let mut win = nvim_mark_tabpage_firstwin(tp);
            while !win.is_null() {
                let result = nvim_mark_qf_mark_adjust(buf, win, line1, line2, amount, amount_after);
                found_one |= result != 0;
                win = nvim_mark_win_get_next(win);
            }
            tp = nvim_mark_tabpage_next(tp);
        }
        if !found_one {
            let has_qf = nvim_mark_buf_get_has_qf_entry(buf);
            nvim_mark_buf_set_has_qf_entry(buf, has_qf & !BUF_HAS_LL_ENTRY);
        }
    }

    if op != EXTMARK_NOOP {
        nvim_mark_extmark_adjust(buf, line1, line2, amount, amount_after, op);
    }

    let curwin = nvim_mark_get_curwin();

    if nvim_mark_win_get_buf(curwin) == buf {
        // previous context mark
        let pcmark = nvim_mark_win_get_pcmark_ptr(curwin);
        one_adjust(&mut (*pcmark).lnum, line1, line2, amount, amount_after);

        // previous pcmark
        let prev_pcmark = nvim_mark_win_get_prev_pcmark_ptr(curwin);
        one_adjust(&mut (*prev_pcmark).lnum, line1, line2, amount, amount_after);

        // saved cursor for formatting
        let saved = nvim_mark_get_saved_cursor();
        if (*saved).lnum != 0 {
            one_adjust_nodel(&mut (*saved).lnum, line1, line2, amount, amount_after);
        }
    }

    // Adjust items in all windows related to the current buffer.
    let mut tp = nvim_mark_get_first_tabpage();
    while !tp.is_null() {
        let mut win = nvim_mark_tabpage_firstwin(tp);
        while !win.is_null() {
            if !lockmarks {
                // Marks in the jumplist
                let jlen = nvim_mark_win_get_jumplistlen(win);
                for i in 0..jlen {
                    if nvim_mark_win_get_jumplist_fnum(win, i) == fnum {
                        let entry = nvim_mark_win_get_jumplist_entry(win, i);
                        one_adjust_nodel(
                            &mut (*entry).fmark.mark.lnum,
                            line1,
                            line2,
                            amount,
                            amount_after,
                        );
                    }
                }
            }

            if nvim_mark_win_get_buf(win) == buf {
                if !lockmarks {
                    // marks in the tag stack
                    let tlen = nvim_mark_win_get_tagstacklen(win);
                    for i in 0..tlen {
                        if nvim_mark_win_get_tagstack_fnum(win, i) == fnum {
                            let tmark = nvim_mark_win_get_tagstack_mark_ptr(win, i);
                            one_adjust_nodel(
                                &mut (*tmark).lnum,
                                line1,
                                line2,
                                amount,
                                amount_after,
                            );
                        }
                    }
                }

                // the displayed Visual area
                if nvim_mark_win_get_old_cursor_lnum(win) != 0 {
                    let old_cursor = nvim_mark_win_get_old_cursor_lnum_ptr(win);
                    one_adjust_nodel(old_cursor, line1, line2, amount, amount_after);
                    let old_visual = nvim_mark_win_get_old_visual_lnum_ptr(win);
                    one_adjust_nodel(old_visual, line1, line2, amount, amount_after);
                }

                // topline and cursor position
                let line_count = nvim_buf_get_ml_line_count(buf);
                let cursor_lnum = (*nvim_mark_win_get_cursor_ptr(win)).lnum;
                if by_api
                    || (if by_term {
                        cursor_lnum < line_count
                    } else {
                        win != curwin
                    })
                {
                    let topline = nvim_mark_win_get_topline_val(win);
                    if topline >= line1 && topline <= line2 {
                        if amount == MAXLNUM {
                            // topline is deleted
                            if by_api && amount_after > line1 - line2 - 1 {
                                // api: deleted region replaced with new contents,
                                // topline adjusted later via fix_cursor()
                            } else {
                                let new_top = if line1 - 1 > 1 { line1 - 1 } else { 1 };
                                nvim_mark_win_set_topline_val(win, new_top);
                            }
                        } else if topline > line1 {
                            nvim_mark_win_set_topline_val(win, topline + amount);
                        }
                        nvim_mark_win_set_topfill(win, 0);
                    } else if amount_after != 0
                        && topline > line2 + (if by_api && line2 < line1 { 1 } else { 0 })
                    {
                        nvim_mark_win_set_topline_val(win, topline + amount_after);
                        nvim_mark_win_set_topfill(win, 0);
                    }
                }
                if !by_api
                    && (if by_term {
                        cursor_lnum < nvim_buf_get_ml_line_count(buf)
                    } else {
                        win != curwin
                    })
                {
                    let cursor_ptr = nvim_mark_win_get_cursor_ptr(win);
                    one_adjust_cursor(cursor_ptr, line1, line2, amount, amount_after);
                }

                if adjust_folds != 0 {
                    nvim_mark_fold_adjust(win, line1, line2, amount, amount_after);
                }
            }

            win = nvim_mark_win_get_next(win);
        }
        tp = nvim_mark_tabpage_next(tp);
    }

    // adjust diffs
    nvim_mark_diff_adjust(buf, line1, line2, amount, amount_after);

    // adjust per-window "last cursor" positions
    let winfo_count = nvim_mark_buf_get_wininfo_count(buf);
    for i in 0..winfo_count {
        let wmark = nvim_mark_buf_get_wininfo_mark(buf, i);
        if !by_term || (*wmark).lnum < nvim_buf_get_ml_line_count(buf) {
            one_adjust_cursor(wmark, line1, line2, amount, amount_after);
        }
    }
}

/// Adjust marks in line "lnum" at column "mincol" and further.
///
/// # Safety
/// All buffer/window pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_col_adjust_all(
    lnum: LinenrT,
    mincol: ColnrT,
    lnum_amount: LinenrT,
    col_amount: ColnrT,
    spaces_removed: c_int,
) {
    let curbuf_ptr = nvim_mark_get_curbuf();
    let fnum = nvim_buf_get_fnum(curbuf_ptr);

    if (col_amount == 0 && lnum_amount == 0) || (nvim_mark_get_cmod_flags() & CMOD_LOCKMARKS) != 0 {
        return; // nothing to do
    }

    // named marks, lower case and upper case
    for i in 0..NMARKS {
        let namedm = nvim_mark_buf_get_namedm(curbuf_ptr, i);
        col_adjust(
            &mut (*namedm).mark,
            lnum,
            mincol,
            lnum_amount,
            col_amount,
            spaces_removed,
        );

        let namedfm_ptr = nvim_mark_get_namedfm();
        let gmark = &mut *namedfm_ptr.offset(i as isize);
        if gmark.fmark.fnum == fnum {
            col_adjust(
                &mut gmark.fmark.mark,
                lnum,
                mincol,
                lnum_amount,
                col_amount,
                spaces_removed,
            );
        }
    }
    for i in NMARKS..NGLOBALMARKS {
        let namedfm_ptr = nvim_mark_get_namedfm();
        let gmark = &mut *namedfm_ptr.offset(i as isize);
        if gmark.fmark.fnum == fnum {
            col_adjust(
                &mut gmark.fmark.mark,
                lnum,
                mincol,
                lnum_amount,
                col_amount,
                spaces_removed,
            );
        }
    }

    // last Insert position
    let last_insert = nvim_mark_buf_get_last_insert(curbuf_ptr);
    col_adjust(
        &mut (*last_insert).mark,
        lnum,
        mincol,
        lnum_amount,
        col_amount,
        spaces_removed,
    );

    // last change position
    let last_change = nvim_mark_buf_get_last_change(curbuf_ptr);
    col_adjust(
        &mut (*last_change).mark,
        lnum,
        mincol,
        lnum_amount,
        col_amount,
        spaces_removed,
    );

    // list of change positions
    let changelistlen = nvim_mark_buf_get_changelistlen(curbuf_ptr);
    for i in 0..changelistlen {
        let cl = nvim_mark_buf_get_changelist(curbuf_ptr, i);
        col_adjust(
            &mut (*cl).mark,
            lnum,
            mincol,
            lnum_amount,
            col_amount,
            spaces_removed,
        );
    }

    // Visual area
    let vi_start = nvim_mark_buf_get_visual_start_ptr(curbuf_ptr);
    col_adjust(
        vi_start,
        lnum,
        mincol,
        lnum_amount,
        col_amount,
        spaces_removed,
    );
    let vi_end = nvim_mark_buf_get_visual_end_ptr(curbuf_ptr);
    col_adjust(
        vi_end,
        lnum,
        mincol,
        lnum_amount,
        col_amount,
        spaces_removed,
    );

    // previous context mark
    let curwin = nvim_mark_get_curwin();
    let pcmark = nvim_mark_win_get_pcmark_ptr(curwin);
    col_adjust(
        pcmark,
        lnum,
        mincol,
        lnum_amount,
        col_amount,
        spaces_removed,
    );

    // previous pcmark
    let prev_pcmark = nvim_mark_win_get_prev_pcmark_ptr(curwin);
    col_adjust(
        prev_pcmark,
        lnum,
        mincol,
        lnum_amount,
        col_amount,
        spaces_removed,
    );

    // saved cursor for formatting
    let saved = nvim_mark_get_saved_cursor();
    col_adjust(saved, lnum, mincol, lnum_amount, col_amount, spaces_removed);

    // Adjust items in all windows related to the current buffer (current tab only)
    let curtab = nvim_mark_get_curtab();
    let mut win = nvim_mark_tabpage_firstwin(curtab);
    while !win.is_null() {
        // marks in the jumplist
        let jlen = nvim_mark_win_get_jumplistlen(win);
        for i in 0..jlen {
            if nvim_mark_win_get_jumplist_fnum(win, i) == fnum {
                let jmark = nvim_mark_win_get_jumplist_mark_ptr(win, i);
                col_adjust(jmark, lnum, mincol, lnum_amount, col_amount, spaces_removed);
            }
        }

        if nvim_mark_win_get_buf(win) == curbuf_ptr {
            // marks in the tag stack
            let tlen = nvim_mark_win_get_tagstacklen(win);
            for i in 0..tlen {
                if nvim_mark_win_get_tagstack_fnum(win, i) == fnum {
                    let tmark = nvim_mark_win_get_tagstack_mark_ptr(win, i);
                    col_adjust(tmark, lnum, mincol, lnum_amount, col_amount, spaces_removed);
                }
            }

            // cursor position for other windows with the same buffer
            if win != curwin {
                let cursor_ptr = nvim_mark_win_get_cursor_ptr(win);
                col_adjust(
                    cursor_ptr,
                    lnum,
                    mincol,
                    lnum_amount,
                    col_amount,
                    spaces_removed,
                );
            }
        }

        win = nvim_mark_win_get_next(win);
    }
}

// =============================================================================
// Phase 6: Ex Commands + Remaining
// =============================================================================

/// NUL character constant
const NUL_CHAR: c_int = 0;
/// TAB character constant
const TAB_CHAR: u8 = 0x09;

/// Implementation of `:delmarks[!] [marks]` command.
///
/// Parses the argument string and deletes the specified marks.
/// If `forceit` is set and arg is empty, clears all marks.
///
/// # Safety
/// Pointers must be valid. `arg` must be a NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_delmarks(arg: *const c_char, forceit: c_int, curbuf: BufHandle) {
    let arg_empty = arg.is_null() || *arg == 0;

    if arg_empty && forceit != 0 {
        // :delmarks! — clear all marks
        let ts = nvim_mark_os_time();
        rs_clrallmarks(curbuf, ts);
        return;
    }

    if forceit != 0 {
        // :delmarks! with args — error
        nvim_mark_emsg_invarg();
        return;
    }

    if arg_empty {
        // :delmarks without args — error
        nvim_mark_emsg_argreq();
        return;
    }

    // Parse and clear specified marks
    let timestamp = nvim_mark_os_time();
    let namedfm_ptr = nvim_mark_get_namedfm();
    let mut p = arg;
    while *p != 0 {
        let ch = *p as u8;
        let is_lower = ascii_islower(ch);
        let is_digit = ascii_isdigit(ch);
        let is_upper = ascii_isupper(ch);

        if is_lower || is_digit || is_upper {
            let from: u8;
            let to: u8;
            if *p.add(1) == b'-' as c_char {
                // Range: e.g., "a-z"
                from = ch;
                to = *p.add(2) as u8;
                // Validate range types match and to >= from
                let valid = if is_lower {
                    ascii_islower(to)
                } else if is_digit {
                    ascii_isdigit(to)
                } else {
                    ascii_isupper(to)
                };
                if !valid || (to as c_int) < (from as c_int) {
                    nvim_mark_semsg_invarg2(p);
                    return;
                }
                p = p.add(2);
            } else {
                from = ch;
                to = ch;
            }

            for i in (from as c_int)..=(to as c_int) {
                if is_lower {
                    let idx = i - b'a' as c_int;
                    let fm = nvim_mark_buf_get_namedm(curbuf, idx);
                    (*fm).mark.lnum = 0;
                    (*fm).timestamp = timestamp;
                } else {
                    let n = if is_digit {
                        i - b'0' as c_int + NMARKS
                    } else {
                        i - b'A' as c_int
                    };
                    let xfm = namedfm_ptr.offset(n as isize);
                    (*xfm).fmark.mark.lnum = 0;
                    (*xfm).fmark.fnum = 0;
                    (*xfm).fmark.timestamp = timestamp;
                    // XFREE_CLEAR(namedfm[n].fname)
                    if !(*xfm).fname.is_null() {
                        nvim_mark_xfree((*xfm).fname as *mut c_void);
                        (*xfm).fname = std::ptr::null_mut();
                    }
                }
            }
        } else {
            // Special marks
            match ch {
                b'"' => {
                    let fm = nvim_mark_buf_get_last_cursor(curbuf);
                    rs_clear_fmark(fm, timestamp);
                }
                b'^' => {
                    let fm = nvim_mark_buf_get_last_insert(curbuf);
                    rs_clear_fmark(fm, timestamp);
                }
                b':' => {
                    // Readonly mark - no deletion allowed
                }
                b'.' => {
                    let fm = nvim_mark_buf_get_last_change(curbuf);
                    rs_clear_fmark(fm, timestamp);
                }
                b'[' => {
                    let op_start = nvim_mark_buf_get_op_start(curbuf);
                    (*op_start).lnum = 0;
                }
                b']' => {
                    let op_end = nvim_mark_buf_get_op_end(curbuf);
                    (*op_end).lnum = 0;
                }
                b'<' => {
                    let vis = nvim_mark_buf_get_visual_start_ptr(curbuf);
                    (*vis).lnum = 0;
                }
                b'>' => {
                    let vis = nvim_mark_buf_get_visual_end_ptr(curbuf);
                    (*vis).lnum = 0;
                }
                b' ' => {
                    // Space: skip
                }
                _ => {
                    nvim_mark_semsg_invarg2(p);
                    return;
                }
            }
        }
        p = p.add(1);
    }
}

/// Adjust position to point to the first byte of a multi-byte character.
///
/// If the position points to a tail byte, it is moved backwards to the head byte.
///
/// # Safety
/// `buf` and `lp` must be valid pointers.
#[export_name = "mark_mb_adjustpos"]
pub unsafe extern "C" fn rs_mark_mb_adjustpos(buf: BufHandle, lp: *mut PosT) {
    if (*lp).col > 0 || (*lp).coladd > 1 {
        let line = nvim_mark_ml_get_buf(buf, (*lp).lnum);
        if *line == 0 || nvim_mark_ml_get_buf_len(buf, (*lp).lnum) < (*lp).col {
            (*lp).col = 0;
        } else {
            (*lp).col -= nvim_mark_utf_head_off(line, line.offset((*lp).col as isize)) as ColnrT;
        }
        // Reset "coladd" when the cursor would be on the right half of a
        // double-wide character.
        if (*lp).coladd == 1
            && *line.offset((*lp).col as isize) != TAB_CHAR as c_char
            && nvim_mark_vim_isprintc(nvim_mark_utf_ptr2char(line.offset((*lp).col as isize))) != 0
            && nvim_mark_ptr2cells(line.offset((*lp).col as isize)) > 1
        {
            (*lp).coladd = 0;
        }
    }
}

/// Get marks that are actually motions but return them as marks.
///
/// Gets the following motions as marks: '{', '}', '(', ')'
///
/// # Safety
/// `buf` and `win` must be valid handles.
#[export_name = "mark_get_motion"]
pub unsafe extern "C" fn rs_mark_get_motion(
    buf: BufHandle,
    win: WinHandle,
    name: c_int,
) -> *mut FmarkT {
    let pos = nvim_mark_win_get_cursor(win);
    let slcb = nvim_mark_get_listcmd_busy();
    nvim_mark_set_listcmd_busy(1); // avoid that '' is changed

    let mark: *mut FmarkT;
    if name == b'{' as c_int || name == b'}' as c_int {
        // to previous/next paragraph
        let mut inclusive: c_int = 0;
        let dir = if name == b'}' as c_int {
            FORWARD
        } else {
            BACKWARD
        };
        if nvim_mark_findpar(&mut inclusive, dir, 1, NUL_CHAR, 0) != 0 {
            let cursor = nvim_mark_win_get_cursor(win);
            mark = rs_pos_to_mark(buf, std::ptr::null_mut(), cursor);
        } else {
            mark = std::ptr::null_mut();
        }
    } else if name == b'(' as c_int || name == b')' as c_int {
        // to previous/next sentence
        let dir = if name == b')' as c_int {
            FORWARD
        } else {
            BACKWARD
        };
        if nvim_mark_findsent(dir, 1) != 0 {
            let cursor = nvim_mark_win_get_cursor(win);
            mark = rs_pos_to_mark(buf, std::ptr::null_mut(), cursor);
        } else {
            mark = std::ptr::null_mut();
        }
    } else {
        mark = std::ptr::null_mut();
    }

    nvim_mark_win_set_cursor(win, pos);
    nvim_mark_set_listcmd_busy(slcb);
    mark
}

// =============================================================================
// Phase 3 & 5 Tests
// =============================================================================

#[cfg(test)]
mod phase35_tests {
    use super::*;

    #[test]
    fn test_jumplist_new_len() {
        // Normal increment
        assert_eq!(rs_jumplist_new_len(0), 1);
        assert_eq!(rs_jumplist_new_len(50), 51);
        assert_eq!(rs_jumplist_new_len(99), 100);

        // At max, should stay at max
        assert_eq!(rs_jumplist_new_len(100), 100);
        assert_eq!(rs_jumplist_new_len(200), 100);
    }

    #[test]
    fn test_jumplist_is_full() {
        assert_eq!(rs_jumplist_is_full(99), 0);
        assert_ne!(rs_jumplist_is_full(100), 0);
        assert_ne!(rs_jumplist_is_full(150), 0);
    }

    #[test]
    fn test_jumplist_stack_trim() {
        // idx < len - 1: should trim
        assert_eq!(rs_jumplist_stack_trim(5, 10), 6);
        assert_eq!(rs_jumplist_stack_trim(0, 10), 1);

        // idx >= len - 1: no trim needed
        assert_eq!(rs_jumplist_stack_trim(9, 10), -1);
        assert_eq!(rs_jumplist_stack_trim(10, 10), -1);
    }

    #[test]
    fn test_jumplist_calc_idx() {
        // Valid jumps
        assert_eq!(rs_jumplist_calc_idx(5, 10, -2), 3);
        assert_eq!(rs_jumplist_calc_idx(5, 10, 2), 7);
        assert_eq!(rs_jumplist_calc_idx(0, 10, 0), 0);

        // Out of bounds
        assert_eq!(rs_jumplist_calc_idx(0, 10, -1), -1);
        assert_eq!(rs_jumplist_calc_idx(9, 10, 1), -1);
    }

    #[test]
    fn test_changelist_calc_idx() {
        // Valid navigation
        assert_eq!(rs_changelist_calc_idx(5, 10, -2), 3);
        assert_eq!(rs_changelist_calc_idx(5, 10, 2), 7);

        // Clamp to start
        assert_eq!(rs_changelist_calc_idx(2, 10, -5), 0);

        // Clamp to end
        assert_eq!(rs_changelist_calc_idx(7, 10, 5), 9);

        // Already at boundary, can't navigate
        assert_eq!(rs_changelist_calc_idx(0, 10, -1), -1);
        assert_eq!(rs_changelist_calc_idx(9, 10, 1), -1);
    }

    #[test]
    fn test_mark_target_type() {
        // Global marks
        assert_eq!(rs_mark_target_type(c_int::from(b'A')), 1);
        assert_eq!(rs_mark_target_type(c_int::from(b'Z')), 1);
        assert_eq!(rs_mark_target_type(c_int::from(b'0')), 1);

        // Local named marks
        assert_eq!(rs_mark_target_type(c_int::from(b'a')), 2);
        assert_eq!(rs_mark_target_type(c_int::from(b'z')), 2);

        // Special marks
        assert_eq!(rs_mark_target_type(c_int::from(b'\'')), 3);
        assert_eq!(rs_mark_target_type(c_int::from(b'`')), 3);
        assert_eq!(rs_mark_target_type(c_int::from(b'"')), 4);
        assert_eq!(rs_mark_target_type(c_int::from(b'[')), 5);
        assert_eq!(rs_mark_target_type(c_int::from(b']')), 6);
        assert_eq!(rs_mark_target_type(c_int::from(b'<')), 7);
        assert_eq!(rs_mark_target_type(c_int::from(b'>')), 8);
        assert_eq!(rs_mark_target_type(c_int::from(b'^')), 9);
        assert_eq!(rs_mark_target_type(c_int::from(b'.')), 10);
        assert_eq!(rs_mark_target_type(c_int::from(b':')), 11);

        // Invalid
        assert_eq!(rs_mark_target_type(c_int::from(b'@')), 0);
        assert_eq!(rs_mark_target_type(-1), 0);
    }

    #[test]
    fn test_pos_clamp_lnum_min() {
        assert_eq!(rs_pos_clamp_lnum_min(5), 5);
        assert_eq!(rs_pos_clamp_lnum_min(1), 1);
        assert_eq!(rs_pos_clamp_lnum_min(0), 1);
        assert_eq!(rs_pos_clamp_lnum_min(-1), 1);
    }
}

// =============================================================================
// Phase 4: Mark Movement Functions
// =============================================================================

/// Flags for outcomes when moving to a mark.
/// These match MarkMoveRes in mark_defs.h
pub mod mark_move_res {
    pub const SUCCESS: i32 = 1;
    pub const FAILED: i32 = 2;
    pub const SWITCHED_BUF: i32 = 4;
    pub const CHANGED_COL: i32 = 8;
    pub const CHANGED_LINE: i32 = 16;
    pub const CHANGED_CURSOR: i32 = 32;
    pub const CHANGED_VIEW: i32 = 64;
}

/// Flags to configure the movement to a mark.
/// These match MarkMove in mark_defs.h
pub mod mark_move_flags {
    pub const BEGIN_LINE: i32 = 1;
    pub const CONTEXT: i32 = 2;
    pub const NO_CONTEXT: i32 = 4;
    pub const SET_VIEW: i32 = 8;
    pub const JUMP_LIST: i32 = 16;
}

/// Direction constants for mark searching
pub const FORWARD: c_int = 1;
pub const BACKWARD: c_int = -1;

/// Calculate MarkMoveRes flags based on position changes.
///
/// # Arguments
/// * `prev_lnum`, `prev_col` - Previous cursor position
/// * `new_lnum`, `new_col` - New cursor position
/// * `initial_res` - Initial result flags
///
/// # Returns
/// Updated result flags with CHANGED_LINE, CHANGED_COL, CHANGED_CURSOR set appropriately
#[no_mangle]
pub extern "C" fn rs_mark_move_calc_result(
    prev_lnum: LinenrT,
    prev_col: ColnrT,
    new_lnum: LinenrT,
    new_col: ColnrT,
    initial_res: c_int,
) -> c_int {
    let mut res = initial_res;
    if prev_lnum != new_lnum {
        res |= mark_move_res::CHANGED_LINE | mark_move_res::CHANGED_CURSOR;
    }
    if prev_col != new_col {
        res |= mark_move_res::CHANGED_COL | mark_move_res::CHANGED_CURSOR;
    }
    res
}

/// Check if mark_move_to should do additional cursor checking.
///
/// # Arguments
/// * `res` - Current result flags
///
/// # Returns
/// Non-zero if cursor check should be performed
#[no_mangle]
pub extern "C" fn rs_mark_move_needs_cursor_check(res: c_int) -> c_int {
    c_int::from(
        (res & mark_move_res::SWITCHED_BUF) != 0 || (res & mark_move_res::CHANGED_CURSOR) != 0,
    )
}

/// Prepare column for getnextmark search based on direction and begin_line.
///
/// # Arguments
/// * `col` - Current column
/// * `dir` - Direction (FORWARD or BACKWARD)
/// * `begin_line` - Whether to search from beginning of line
///
/// # Returns
/// Adjusted column value for the search
#[no_mangle]
pub extern "C" fn rs_getnextmark_adjust_col(col: ColnrT, dir: c_int, begin_line: c_int) -> ColnrT {
    if begin_line != 0 {
        if dir == BACKWARD {
            0
        } else {
            MAXCOL
        }
    } else {
        col
    }
}

/// Compare positions for getnextmark search.
///
/// Implements the logic for finding the next/previous mark relative to a position.
///
/// # Arguments
/// * `candidate_lnum`, `candidate_col` - Position of the candidate mark
/// * `current_best_lnum`, `current_best_col` - Position of the current best match (use 0,0 if none)
/// * `start_lnum`, `start_col` - Position to search from
/// * `dir` - Direction (FORWARD or BACKWARD)
///
/// # Returns
/// Non-zero if candidate is better than current_best
#[no_mangle]
pub extern "C" fn rs_getnextmark_is_better(
    candidate_lnum: LinenrT,
    candidate_col: ColnrT,
    current_best_lnum: LinenrT,
    current_best_col: ColnrT,
    start_lnum: LinenrT,
    start_col: ColnrT,
    dir: c_int,
) -> c_int {
    // Skip invalid candidates
    if candidate_lnum <= 0 {
        return 0;
    }

    let candidate = PosT {
        lnum: candidate_lnum,
        col: candidate_col,
        coladd: 0,
    };
    let start = PosT {
        lnum: start_lnum,
        col: start_col,
        coladd: 0,
    };
    let no_best = current_best_lnum == 0;

    if dir == FORWARD {
        // For forward: candidate must be after start, and closer than current best
        let after_start = rs_lt(start, candidate) != 0;
        if !after_start {
            return 0;
        }
        if no_best {
            return 1;
        }
        let best = PosT {
            lnum: current_best_lnum,
            col: current_best_col,
            coladd: 0,
        };
        c_int::from(rs_lt(candidate, best) != 0)
    } else {
        // For backward: candidate must be before start, and closer than current best
        let before_start = rs_lt(candidate, start) != 0;
        if !before_start {
            return 0;
        }
        if no_best {
            return 1;
        }
        let best = PosT {
            lnum: current_best_lnum,
            col: current_best_col,
            coladd: 0,
        };
        c_int::from(rs_lt(best, candidate) != 0)
    }
}

// =============================================================================
// Phase 4 Tests
// =============================================================================

#[cfg(test)]
mod phase4_tests {
    use super::*;

    #[test]
    fn test_mark_move_calc_result() {
        // No change
        let res = rs_mark_move_calc_result(10, 5, 10, 5, mark_move_res::SUCCESS);
        assert_eq!(res, mark_move_res::SUCCESS);

        // Line changed
        let res = rs_mark_move_calc_result(10, 5, 20, 5, mark_move_res::SUCCESS);
        assert_ne!(res & mark_move_res::CHANGED_LINE, 0);
        assert_ne!(res & mark_move_res::CHANGED_CURSOR, 0);
        assert_eq!(res & mark_move_res::CHANGED_COL, 0);

        // Column changed
        let res = rs_mark_move_calc_result(10, 5, 10, 15, mark_move_res::SUCCESS);
        assert_ne!(res & mark_move_res::CHANGED_COL, 0);
        assert_ne!(res & mark_move_res::CHANGED_CURSOR, 0);
        assert_eq!(res & mark_move_res::CHANGED_LINE, 0);

        // Both changed
        let res = rs_mark_move_calc_result(10, 5, 20, 15, mark_move_res::SUCCESS);
        assert_ne!(res & mark_move_res::CHANGED_LINE, 0);
        assert_ne!(res & mark_move_res::CHANGED_COL, 0);
        assert_ne!(res & mark_move_res::CHANGED_CURSOR, 0);
    }

    #[test]
    fn test_mark_move_needs_cursor_check() {
        assert_eq!(rs_mark_move_needs_cursor_check(mark_move_res::SUCCESS), 0);
        assert_ne!(
            rs_mark_move_needs_cursor_check(mark_move_res::SWITCHED_BUF),
            0
        );
        assert_ne!(
            rs_mark_move_needs_cursor_check(mark_move_res::CHANGED_CURSOR),
            0
        );
        assert_ne!(
            rs_mark_move_needs_cursor_check(
                mark_move_res::SWITCHED_BUF | mark_move_res::CHANGED_CURSOR
            ),
            0
        );
    }

    #[test]
    fn test_getnextmark_adjust_col() {
        // No begin_line adjustment
        assert_eq!(rs_getnextmark_adjust_col(5, FORWARD, 0), 5);
        assert_eq!(rs_getnextmark_adjust_col(5, BACKWARD, 0), 5);

        // begin_line adjustment
        assert_eq!(rs_getnextmark_adjust_col(5, FORWARD, 1), MAXCOL);
        assert_eq!(rs_getnextmark_adjust_col(5, BACKWARD, 1), 0);
    }

    #[test]
    fn test_getnextmark_is_better_forward() {
        // Forward from (10, 5): looking for marks after this position
        // No current best (0, 0), candidate at (20, 5) - should be better
        assert_ne!(rs_getnextmark_is_better(20, 5, 0, 0, 10, 5, FORWARD), 0);

        // Candidate before start - not better
        assert_eq!(rs_getnextmark_is_better(5, 5, 0, 0, 10, 5, FORWARD), 0);

        // Candidate closer than current best
        assert_ne!(rs_getnextmark_is_better(15, 5, 20, 5, 10, 5, FORWARD), 0);

        // Candidate farther than current best
        assert_eq!(rs_getnextmark_is_better(25, 5, 20, 5, 10, 5, FORWARD), 0);

        // Invalid candidate (lnum <= 0)
        assert_eq!(rs_getnextmark_is_better(0, 5, 0, 0, 10, 5, FORWARD), 0);
    }

    #[test]
    fn test_getnextmark_is_better_backward() {
        // Backward from (20, 5): looking for marks before this position
        // No current best, candidate at (10, 5) - should be better
        assert_ne!(rs_getnextmark_is_better(10, 5, 0, 0, 20, 5, BACKWARD), 0);

        // Candidate after start - not better
        assert_eq!(rs_getnextmark_is_better(25, 5, 0, 0, 20, 5, BACKWARD), 0);

        // Candidate closer than current best (closer means higher for backward)
        assert_ne!(rs_getnextmark_is_better(15, 5, 10, 5, 20, 5, BACKWARD), 0);

        // Candidate farther than current best
        assert_eq!(rs_getnextmark_is_better(5, 5, 10, 5, 20, 5, BACKWARD), 0);
    }
}

// =============================================================================
// Phase 6: Mark Adjustment Functions
// =============================================================================

/// Result of a line number adjustment.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineAdjustResult {
    /// New line number after adjustment
    pub new_lnum: LinenrT,
    /// Whether the line was modified
    pub modified: c_int,
}

/// Adjust a line number based on line deletion/insertion.
///
/// Implements ONE_ADJUST logic:
/// - If lnum in [line1, line2]: add amount (or set to 0 if amount is MAXLNUM)
/// - If lnum > line2: add amount_after
///
/// # Arguments
/// * `lnum` - The line number to adjust
/// * `line1` - Start of affected range
/// * `line2` - End of affected range
/// * `amount` - Amount to add for lines in range (MAXLNUM means delete)
/// * `amount_after` - Amount to add for lines after range
///
/// # Returns
/// LineAdjustResult with the new line number and modification flag
#[no_mangle]
pub extern "C" fn rs_mark_adjust_lnum(
    lnum: LinenrT,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) -> LineAdjustResult {
    if lnum >= line1 && lnum <= line2 {
        // Line is in the affected range
        if amount == MAXLNUM {
            // Deletion: set to 0
            LineAdjustResult {
                new_lnum: 0,
                modified: 1,
            }
        } else {
            LineAdjustResult {
                new_lnum: lnum + amount,
                modified: 1,
            }
        }
    } else if amount_after != 0 && lnum > line2 {
        // Line is after the range
        LineAdjustResult {
            new_lnum: lnum + amount_after,
            modified: 1,
        }
    } else {
        // No change
        LineAdjustResult {
            new_lnum: lnum,
            modified: 0,
        }
    }
}

/// Adjust a line number with no-delete behavior.
///
/// Implements ONE_ADJUST_NODEL logic:
/// - If lnum in [line1, line2]: add amount (or set to line1 if amount is MAXLNUM)
/// - If lnum > line2: add amount_after
///
/// # Arguments
/// * `lnum` - The line number to adjust
/// * `line1` - Start of affected range
/// * `line2` - End of affected range
/// * `amount` - Amount to add for lines in range (MAXLNUM means set to line1)
/// * `amount_after` - Amount to add for lines after range
///
/// # Returns
/// LineAdjustResult with the new line number and modification flag
#[no_mangle]
pub extern "C" fn rs_mark_adjust_lnum_nodel(
    lnum: LinenrT,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) -> LineAdjustResult {
    if lnum >= line1 && lnum <= line2 {
        // Line is in the affected range
        if amount == MAXLNUM {
            // No delete: set to line1
            LineAdjustResult {
                new_lnum: line1,
                modified: 1,
            }
        } else {
            LineAdjustResult {
                new_lnum: lnum + amount,
                modified: 1,
            }
        }
    } else if amount_after != 0 && lnum > line2 {
        // Line is after the range
        LineAdjustResult {
            new_lnum: lnum + amount_after,
            modified: 1,
        }
    } else {
        // No change
        LineAdjustResult {
            new_lnum: lnum,
            modified: 0,
        }
    }
}

/// Result of a cursor position adjustment.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorAdjustResult {
    /// New line number after adjustment
    pub new_lnum: LinenrT,
    /// New column after adjustment
    pub new_col: ColnrT,
    /// Whether the position was modified
    pub modified: c_int,
}

/// Adjust a cursor position based on line deletion/insertion.
///
/// Implements ONE_ADJUST_CURSOR logic:
/// - If lnum in [line1, line2] and amount is MAXLNUM: move to max(line1-1, 1), col 0
/// - If lnum in [line1, line2]: add amount to lnum
/// - If lnum > line2: add amount_after
///
/// # Arguments
/// * `lnum` - The line number to adjust
/// * `col` - The column to adjust
/// * `line1` - Start of affected range
/// * `line2` - End of affected range
/// * `amount` - Amount to add for lines in range (MAXLNUM means delete)
/// * `amount_after` - Amount to add for lines after range
///
/// # Returns
/// CursorAdjustResult with the new position and modification flag
#[no_mangle]
pub extern "C" fn rs_mark_adjust_cursor(
    lnum: LinenrT,
    col: ColnrT,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
) -> CursorAdjustResult {
    if lnum >= line1 && lnum <= line2 {
        // Cursor is in the affected range
        if amount == MAXLNUM {
            // Line with cursor is deleted
            let new_lnum = std::cmp::max(line1 - 1, 1);
            CursorAdjustResult {
                new_lnum,
                new_col: 0,
                modified: 1,
            }
        } else {
            // Keep cursor on the same line
            CursorAdjustResult {
                new_lnum: lnum + amount,
                new_col: col,
                modified: 1,
            }
        }
    } else if amount_after != 0 && lnum > line2 {
        // Cursor is after the range
        CursorAdjustResult {
            new_lnum: lnum + amount_after,
            new_col: col,
            modified: 1,
        }
    } else {
        // No change
        CursorAdjustResult {
            new_lnum: lnum,
            new_col: col,
            modified: 0,
        }
    }
}

/// Result of a column adjustment.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColAdjustResult {
    /// New line number after adjustment
    pub new_lnum: LinenrT,
    /// New column after adjustment
    pub new_col: ColnrT,
    /// Whether the position was modified
    pub modified: c_int,
}

/// Adjust a position's column based on column changes.
///
/// Implements COL_ADJUST logic for mark_col_adjust.
///
/// # Arguments
/// * `pos_lnum` - Position's line number
/// * `pos_col` - Position's column
/// * `lnum` - Line being modified
/// * `mincol` - Minimum column affected
/// * `lnum_amount` - Amount to add to line number
/// * `col_amount` - Amount to add to column
/// * `spaces_removed` - Number of spaces removed
///
/// # Returns
/// ColAdjustResult with the new position and modification flag
#[no_mangle]
pub extern "C" fn rs_mark_col_adjust(
    pos_lnum: LinenrT,
    pos_col: ColnrT,
    lnum: LinenrT,
    mincol: ColnrT,
    lnum_amount: LinenrT,
    col_amount: ColnrT,
    spaces_removed: c_int,
) -> ColAdjustResult {
    if pos_lnum != lnum || pos_col < mincol {
        // Position not affected
        return ColAdjustResult {
            new_lnum: pos_lnum,
            new_col: pos_col,
            modified: 0,
        };
    }

    let new_lnum = pos_lnum + lnum_amount;
    let new_col = if col_amount < 0 && pos_col <= -col_amount {
        0
    } else if pos_col < spaces_removed {
        col_amount + spaces_removed
    } else {
        pos_col + col_amount
    };

    ColAdjustResult {
        new_lnum,
        new_col,
        modified: 1,
    }
}

/// Check if mark adjustment should be skipped.
///
/// # Arguments
/// * `line1` - Start of range
/// * `line2` - End of range
/// * `amount_after` - Amount for lines after range
///
/// # Returns
/// Non-zero if adjustment should be skipped
#[no_mangle]
pub extern "C" fn rs_mark_adjust_should_skip(
    line1: LinenrT,
    line2: LinenrT,
    amount_after: LinenrT,
) -> c_int {
    c_int::from(line2 < line1 && amount_after == 0)
}

// =============================================================================
// Phase 6 Tests
// =============================================================================

#[cfg(test)]
mod phase6_tests {
    use super::*;

    #[test]
    fn test_mark_adjust_lnum_in_range() {
        // Line in range, add amount
        let result = rs_mark_adjust_lnum(5, 3, 7, 2, 0);
        assert_eq!(result.new_lnum, 7); // 5 + 2
        assert_ne!(result.modified, 0);

        // Line in range, MAXLNUM (delete)
        let result = rs_mark_adjust_lnum(5, 3, 7, MAXLNUM, 0);
        assert_eq!(result.new_lnum, 0);
        assert_ne!(result.modified, 0);
    }

    #[test]
    fn test_mark_adjust_lnum_after_range() {
        // Line after range
        let result = rs_mark_adjust_lnum(10, 3, 7, 2, 3);
        assert_eq!(result.new_lnum, 13); // 10 + 3
        assert_ne!(result.modified, 0);
    }

    #[test]
    fn test_mark_adjust_lnum_no_change() {
        // Line before range
        let result = rs_mark_adjust_lnum(2, 3, 7, 2, 3);
        assert_eq!(result.new_lnum, 2);
        assert_eq!(result.modified, 0);

        // Line after range but amount_after is 0
        let result = rs_mark_adjust_lnum(10, 3, 7, 2, 0);
        assert_eq!(result.new_lnum, 10);
        assert_eq!(result.modified, 0);
    }

    #[test]
    fn test_mark_adjust_lnum_nodel() {
        // Line in range, MAXLNUM (no delete - set to line1)
        let result = rs_mark_adjust_lnum_nodel(5, 3, 7, MAXLNUM, 0);
        assert_eq!(result.new_lnum, 3);
        assert_ne!(result.modified, 0);
    }

    #[test]
    fn test_mark_adjust_cursor() {
        // Cursor in range, deleted
        let result = rs_mark_adjust_cursor(5, 10, 3, 7, MAXLNUM, 0);
        assert_eq!(result.new_lnum, 2); // max(3-1, 1) = 2
        assert_eq!(result.new_col, 0);
        assert_ne!(result.modified, 0);

        // Edge case: line1 is 1
        let result = rs_mark_adjust_cursor(5, 10, 1, 7, MAXLNUM, 0);
        assert_eq!(result.new_lnum, 1); // max(1-1, 1) = 1
        assert_eq!(result.new_col, 0);
    }

    #[test]
    fn test_mark_col_adjust() {
        // Position on affected line, col >= mincol
        let result = rs_mark_col_adjust(5, 10, 5, 5, 0, 3, 0);
        assert_eq!(result.new_lnum, 5);
        assert_eq!(result.new_col, 13); // 10 + 3
        assert_ne!(result.modified, 0);

        // Position on different line - no change
        let result = rs_mark_col_adjust(4, 10, 5, 5, 0, 3, 0);
        assert_eq!(result.new_lnum, 4);
        assert_eq!(result.new_col, 10);
        assert_eq!(result.modified, 0);

        // Position col < mincol - no change
        let result = rs_mark_col_adjust(5, 3, 5, 5, 0, 3, 0);
        assert_eq!(result.new_lnum, 5);
        assert_eq!(result.new_col, 3);
        assert_eq!(result.modified, 0);

        // Negative col_amount, col would go negative
        let result = rs_mark_col_adjust(5, 3, 5, 0, 0, -5, 0);
        assert_eq!(result.new_col, 0);

        // spaces_removed case
        let result = rs_mark_col_adjust(5, 2, 5, 0, 0, 5, 4);
        assert_eq!(result.new_col, 9); // col_amount + spaces_removed = 5 + 4
    }

    #[test]
    fn test_mark_adjust_should_skip() {
        assert_ne!(rs_mark_adjust_should_skip(5, 3, 0), 0); // line2 < line1, amount_after == 0
        assert_eq!(rs_mark_adjust_should_skip(3, 5, 0), 0); // line2 >= line1
        assert_eq!(rs_mark_adjust_should_skip(5, 3, 1), 0); // amount_after != 0
    }
}

// =============================================================================
// Phase 7: Ex Command Helpers
// =============================================================================

/// Result of parsing a delmarks range like "a-z".
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelmarksRange {
    /// Starting character of range
    pub from: c_int,
    /// Ending character of range (same as from for single char)
    pub to: c_int,
    /// 0 if valid, error code otherwise:
    /// 1 = invalid range (different types or to < from)
    /// 2 = invalid character
    pub error: c_int,
    /// Number of characters consumed (1 for single, 3 for range "a-z")
    pub consumed: c_int,
}

/// Parse a mark character or range for :delmarks command.
///
/// Handles:
/// - Single character: 'a', 'A', '0'
/// - Range: 'a-z', 'A-Z', '0-9'
/// - Special marks: '"', '^', '.', '[', ']', '<', '>'
///
/// # Arguments
/// * `c` - First character
/// * `next1` - Second character (for range detection)
/// * `next2` - Third character (for range endpoint)
///
/// # Returns
/// DelmarksRange with from/to and error/consumed info
#[no_mangle]
pub extern "C" fn rs_delmarks_parse_range(c: c_int, next1: c_int, next2: c_int) -> DelmarksRange {
    let Ok(ch) = u8::try_from(c) else {
        return DelmarksRange {
            from: 0,
            to: 0,
            error: 2,
            consumed: 0,
        };
    };

    let is_lower = ascii_islower(ch);
    let is_upper = ascii_isupper(ch);
    let is_digit = ascii_isdigit(ch);

    if is_lower || is_upper || is_digit {
        // Check for range like "a-z"
        if next1 == c_int::from(b'-') {
            // Parse range
            let Ok(end_ch) = u8::try_from(next2) else {
                return DelmarksRange {
                    from: c,
                    to: c,
                    error: 1,
                    consumed: 1,
                };
            };

            // Validate range: must be same type (lower-lower, upper-upper, digit-digit)
            let valid = if is_lower {
                ascii_islower(end_ch)
            } else if is_digit {
                ascii_isdigit(end_ch)
            } else {
                ascii_isupper(end_ch)
            };

            if !valid || next2 < c {
                return DelmarksRange {
                    from: c,
                    to: c,
                    error: 1,
                    consumed: 1,
                };
            }

            DelmarksRange {
                from: c,
                to: next2,
                error: 0,
                consumed: 3,
            }
        } else {
            // Single character
            DelmarksRange {
                from: c,
                to: c,
                error: 0,
                consumed: 1,
            }
        }
    } else {
        // Special marks
        match ch {
            b'"' | b'^' | b'.' | b'[' | b']' | b'<' | b'>' | b' ' => DelmarksRange {
                from: c,
                to: c,
                error: 0,
                consumed: 1,
            },
            b':' => {
                // Readonly mark - no deletion but not an error
                DelmarksRange {
                    from: c,
                    to: c,
                    error: 0,
                    consumed: 1,
                }
            }
            _ => DelmarksRange {
                from: 0,
                to: 0,
                error: 2,
                consumed: 0,
            },
        }
    }
}

/// Calculate the global mark index for :delmarks.
///
/// # Arguments
/// * `c` - Mark character
///
/// # Returns
/// Index in namedfm array, or -1 if not a global mark
#[no_mangle]
pub extern "C" fn rs_delmarks_global_idx(c: c_int) -> c_int {
    let Ok(ch) = u8::try_from(c) else {
        return -1;
    };

    if ascii_isdigit(ch) {
        // '0'-'9' -> NMARKS + (0-9)
        c_int::from(ch - b'0') + NMARKS as c_int
    } else if ascii_isupper(ch) {
        // 'A'-'Z' -> 0-25
        c_int::from(ch - b'A')
    } else {
        -1
    }
}

/// Determine the type of special mark for :delmarks deletion.
///
/// # Arguments
/// * `c` - Mark character
///
/// # Returns
/// Code indicating which buffer field to clear:
/// - 0: Not a special mark
/// - 1: b_last_cursor (")
/// - 2: b_last_insert (^)
/// - 3: b_last_change (.)
/// - 4: b_op_start ([)
/// - 5: b_op_end (])
/// - 6: vi_start (<)
/// - 7: vi_end (>)
/// - 8: Readonly/skip (:)
/// - 9: Space (skip)
#[no_mangle]
pub extern "C" fn rs_delmarks_special_type(c: c_int) -> c_int {
    let Ok(ch) = u8::try_from(c) else {
        return 0;
    };

    match ch {
        b'"' => 1, // b_last_cursor
        b'^' => 2, // b_last_insert
        b'.' => 3, // b_last_change
        b'[' => 4, // b_op_start
        b']' => 5, // b_op_end
        b'<' => 6, // vi_start
        b'>' => 7, // vi_end
        b':' => 8, // readonly, skip
        b' ' => 9, // space, skip
        _ => 0,    // not a special mark
    }
}

/// Get the mark character to display for a given index.
///
/// For ex_marks output formatting.
///
/// # Arguments
/// * `idx` - Mark index (0-25 for A-Z, 26-35 for 0-9)
/// * `is_global` - Whether this is a global mark
///
/// # Returns
/// The character to display
#[no_mangle]
pub extern "C" fn rs_marks_index_to_char(idx: c_int, is_global: c_int) -> c_int {
    if is_global != 0 {
        if idx >= NMARKS as c_int {
            // 0-9
            c_int::from(b'0') + idx - NMARKS as c_int
        } else {
            // A-Z
            c_int::from(b'A') + idx
        }
    } else {
        // a-z
        c_int::from(b'a') + idx
    }
}

/// Check if a mark should be shown based on the filter argument.
///
/// # Arguments
/// * `mark_char` - The mark character
/// * `filter_len` - Length of filter string (0 means show all)
///
/// # Returns
/// Non-zero if mark should be shown (filter is empty or mark is in filter)
#[no_mangle]
pub extern "C" fn rs_marks_should_show(_mark_char: c_int, filter_len: c_int) -> c_int {
    // If no filter, show all marks
    // The actual character matching is done in C with vim_strchr
    c_int::from(filter_len == 0)
}

// =============================================================================
// Phase 1 (FFI): Memory/Field Operations
// =============================================================================

/// Free the additional_data pointer of an fmark_T.
/// C equivalent: `xfree(fm.additional_data)`
#[export_name = "free_fmark"]
pub extern "C" fn rs_free_fmark(fm: FmarkT) {
    if !fm.additional_data.is_null() {
        unsafe {
            nvim_mark_xfree(fm.additional_data as *mut c_void);
        }
    }
}

/// Free an xfmark_T: free fname and additional_data.
/// C equivalent: `xfree(fm.fname); free_fmark(fm.fmark)`
#[export_name = "free_xfmark"]
pub extern "C" fn rs_free_xfmark(fm: XfmarkT) {
    if !fm.fname.is_null() {
        unsafe {
            nvim_mark_xfree(fm.fname as *mut c_void);
        }
    }
    rs_free_fmark(fm.fmark);
}

/// Free and reinitialize an fmark_T with the given timestamp.
/// C equivalent of `clear_fmark`.
///
/// # Safety
/// `fm` must be a valid, non-null pointer to an `FmarkT`.
#[export_name = "clear_fmark"]
pub unsafe extern "C" fn rs_clear_fmark(fm: *mut FmarkT, timestamp: Timestamp) {
    rs_free_fmark(*fm);
    *fm = FmarkT::default();
    (*fm).timestamp = timestamp;
}

/// Wrap a pos_T into an fmark_T with the given buffer's fnum.
///
/// If `fmp` is non-null, writes into it; otherwise uses an internal static buffer.
/// Returns a pointer to the filled fmark_T.
///
/// # Safety
/// `buf` must be a valid buffer handle. If `fmp` is non-null, it must point to a valid `FmarkT`.
#[export_name = "pos_to_mark"]
pub unsafe extern "C" fn rs_pos_to_mark(
    buf: BufHandle,
    fmp: *mut FmarkT,
    pos: PosT,
) -> *mut FmarkT {
    static mut STATIC_FMARK: FmarkT = FmarkT {
        mark: PosT {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        fnum: 0,
        timestamp: 0,
        view: FmarkvT {
            topline_offset: MAXLNUM,
        },
        additional_data: std::ptr::null_mut(),
    };

    let fm = if fmp.is_null() {
        // Reset static to INIT_FMARK equivalent
        STATIC_FMARK = FmarkT::default();
        &raw mut STATIC_FMARK
    } else {
        fmp
    };
    (*fm).fnum = nvim_buf_get_handle(buf);
    (*fm).mark = pos;
    fm
}

/// Get a raw global mark by name.
/// Returns a copy of namedfm[mark_global_index(name)].
///
/// # Safety
/// `name` must be a valid global mark character (A-Z or 0-9).
#[no_mangle]
pub unsafe extern "C" fn rs_get_raw_global_mark(name: c_int) -> XfmarkT {
    let idx = rs_mark_global_index(name);
    let namedfm = nvim_mark_get_namedfm();
    *namedfm.offset(idx as isize)
}

/// Check if a mark's line number exceeds the buffer line count.
/// Returns true (1) if within bounds, false (0) if out of bounds.
/// Sets errormsg to e_markinval (via C accessor) if out of bounds.
///
/// # Safety
/// `buf` must be a valid buffer handle (or null). `errormsg` must be a valid
/// pointer (or null). `e_markinval_str` must be a valid C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_check_line_bounds(
    buf: BufHandle,
    fm_mark_lnum: LinenrT,
    errormsg: *mut *const c_char,
    e_markinval_str: *const c_char,
) -> c_int {
    if !buf.is_null() && fm_mark_lnum > nvim_buf_get_ml_line_count(buf) {
        if !errormsg.is_null() {
            *errormsg = e_markinval_str;
        }
        return 0;
    }
    1
}

/// Check a single xfmark_T: if fnum is 0 and fname matches, set fnum from buf and free fname.
/// C equivalent of `fmarks_check_one`.
///
/// # Safety
/// `fm` must be a valid pointer to an `XfmarkT`. `name` must be a valid C string.
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_fmarks_check_one(
    fm: *mut XfmarkT,
    name: *const c_char,
    buf: BufHandle,
) {
    if (*fm).fmark.fnum == 0
        && !(*fm).fname.is_null()
        && nvim_mark_path_fnamecmp(name, (*fm).fname) == 0
    {
        (*fm).fmark.fnum = nvim_buf_get_fnum(buf);
        // XFREE_CLEAR: free and null the pointer
        nvim_mark_xfree((*fm).fname as *mut c_void);
        (*fm).fname = std::ptr::null_mut();
    }
}

// =============================================================================
// Phase 2: Simple Window/Buffer Operations
// =============================================================================

/// Set the last cursor position for the window's buffer.
/// C equivalent of `set_last_cursor`.
///
/// # Safety
/// `win` must be a valid window handle.
#[export_name = "set_last_cursor"]
pub unsafe extern "C" fn rs_set_last_cursor(win: WinHandle) {
    let buf = nvim_mark_win_get_buffer(win);
    if !buf.is_null() {
        let cursor = nvim_mark_win_get_cursor(win);
        let last_cursor = nvim_mark_buf_get_last_cursor(buf);
        if !last_cursor.is_null() {
            // RESET_FMARK: free old, then set new
            rs_free_fmark(*last_cursor);
            (*last_cursor).mark = cursor;
            (*last_cursor).fnum = 0;
            (*last_cursor).timestamp = nvim_mark_os_time();
            (*last_cursor).view = FmarkvT {
                topline_offset: MAXLNUM,
            };
            (*last_cursor).additional_data = std::ptr::null_mut();
        }
    }
}

/// Free items in the jumplist of a window.
/// C equivalent of `free_jumplist`.
///
/// # Safety
/// `wp` must be a valid window handle.
#[export_name = "free_jumplist"]
pub unsafe extern "C" fn rs_free_jumplist(wp: WinHandle) {
    let len = nvim_mark_win_get_jumplistlen(wp);
    for i in 0..len {
        let entry = nvim_mark_win_get_jumplist_entry(wp, i);
        rs_free_xfmark(*entry);
    }
    nvim_mark_win_set_jumplistlen(wp, 0);
}

/// Clear the jump list (ex_clearjumps command).
///
/// # Safety
/// `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_clearjumps(win: WinHandle) {
    rs_free_jumplist(win);
    nvim_mark_win_set_jumplistlen(win, 0);
    nvim_mark_win_set_jumplistidx(win, 0);
}

/// Free all global marks (EXITFREE cleanup).
///
/// # Safety
/// Global namedfm array must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_free_all_marks() {
    let namedfm = nvim_mark_get_namedfm();
    for i in 0..NGLOBALMARKS {
        let entry = &*namedfm.offset(i as isize);
        if entry.fmark.mark.lnum != 0 {
            rs_free_xfmark(*entry);
        }
    }
    nvim_mark_clear_namedfm();
}

/// Copy the jumplist from one window to another.
/// C equivalent of `copy_jumplist`.
///
/// # Safety
/// Both `from` and `to` must be valid window handles.
#[export_name = "copy_jumplist"]
pub unsafe extern "C" fn rs_copy_jumplist(from: WinHandle, to: WinHandle) {
    let len = nvim_mark_win_get_jumplistlen(from);
    for i in 0..len {
        let src = nvim_mark_win_get_jumplist_entry(from, i);
        let dst = nvim_mark_win_get_jumplist_entry(to, i);
        *dst = *src;
        if !(*src).fname.is_null() {
            (*dst).fname = nvim_mark_xstrdup((*src).fname);
        }
    }
    nvim_mark_win_set_jumplistlen(to, len);
    nvim_mark_win_set_jumplistidx(to, nvim_mark_win_get_jumplistidx(from));
}

/// Check if pcmark should be restored to prev_pcmark.
/// C equivalent of `checkpcmark`.
///
/// # Safety
/// `win` must be a valid window handle (typically curwin).
#[no_mangle]
pub unsafe extern "C" fn rs_checkpcmark(win: WinHandle) {
    let prev_pcmark = nvim_mark_win_get_prev_pcmark(win);
    let pcmark = nvim_mark_win_get_pcmark(win);
    let cursor = nvim_mark_win_get_cursor(win);
    if prev_pcmark.lnum != 0 && (rs_equalpos(pcmark, cursor) != 0 || pcmark.lnum == 0) {
        nvim_mark_win_set_pcmark(win, prev_pcmark);
    }
    nvim_mark_win_set_prev_pcmark(
        win,
        PosT {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    );
}

/// Restore mark view by setting topline based on mark's view offset.
/// C equivalent of `mark_view_restore`.
///
/// # Safety
/// `fm` may be null (function returns early). `win` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_view_restore(fm: *const FmarkT, win: WinHandle) {
    if fm.is_null() {
        return;
    }
    let topline = rs_mark_view_calc_topline((*fm).mark.lnum, (*fm).view.topline_offset);
    if topline >= 1 {
        nvim_mark_win_set_topline(win, topline);
    }
}

/// Check the position in a mark is valid.
/// C equivalent of `mark_check`.
///
/// # Safety
/// `fm` may be null. `errormsg` must be a valid pointer. `curbuf_handle` is the
/// handle of the current buffer for comparing fnum.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_check(
    fm: *const FmarkT,
    errormsg: *mut *const c_char,
    curbuf: BufHandle,
) -> c_int {
    if fm.is_null() {
        *errormsg = nvim_mark_get_e_umark();
        return 0;
    }
    let lnum = (*fm).mark.lnum;
    if lnum <= 0 {
        if lnum == 0 {
            *errormsg = nvim_mark_get_e_marknotset();
        }
        return 0;
    }
    // Only check for valid line number if the buffer is loaded.
    let curbuf_handle = nvim_buf_get_handle(curbuf);
    if (*fm).fnum == curbuf_handle {
        let e_markinval_str = nvim_mark_get_e_markinval();
        if rs_mark_check_line_bounds(curbuf, lnum, errormsg, e_markinval_str) == 0 {
            return 0;
        }
    }
    1
}

// =============================================================================
// Phase 3: Mark Getting + Mark Setting
// =============================================================================

/// Get a global mark by name, optionally resolving fname to fnum.
///
/// # Safety
/// `name` must be a valid global mark character.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_get_global(resolve: c_int, name: c_int) -> *mut XfmarkT {
    let idx = rs_mark_global_index(name);
    assert!(idx >= 0);
    let namedfm = nvim_mark_get_namedfm();
    let mark = namedfm.offset(idx as isize);
    if resolve != 0 && (*mark).fmark.fnum == 0 {
        nvim_mark_fname2fnum(mark);
    }
    mark
}

/// Get a local mark (lowercase and symbols).
///
/// # Safety
/// `buf`, `win`, `curbuf_ptr` must be valid handles.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_get_local(
    buf: BufHandle,
    win: WinHandle,
    name: c_int,
    curbuf_ptr: BufHandle,
) -> *mut FmarkT {
    let idx = rs_mark_local_index(name);
    let mark: *mut FmarkT;

    if rs_mark_is_valid_named(name) {
        mark = nvim_mark_buf_get_namedm(buf, idx);
    } else if rs_mark_is_sentence(name) {
        let pos = if name == c_int::from(b'[') {
            nvim_mark_buf_get_op_start_val(buf)
        } else {
            nvim_mark_buf_get_op_end_val(buf)
        };
        mark = rs_pos_to_mark(buf, std::ptr::null_mut(), pos);
    } else if rs_mark_is_visual(name) {
        mark = rs_mark_get_visual(buf, name);
    } else if rs_mark_is_jump_mark(name) {
        let pcmark = nvim_mark_win_get_pcmark(win);
        mark = rs_pos_to_mark(curbuf_ptr, std::ptr::null_mut(), pcmark);
    } else if rs_mark_is_last_cursor(name) {
        mark = nvim_mark_buf_get_last_cursor(buf);
    } else if rs_mark_is_last_insert(name) {
        mark = nvim_mark_buf_get_last_insert(buf);
    } else if rs_mark_is_last_change(name) {
        mark = nvim_mark_buf_get_last_change(buf);
    } else if name == c_int::from(b':') && nvim_mark_bt_prompt(buf) != 0 {
        mark = nvim_mark_buf_get_prompt_start(buf);
    } else {
        mark = rs_mark_get_motion(buf, win, name);
    }

    if !mark.is_null() {
        (*mark).fnum = nvim_buf_get_fnum(buf);
    }

    mark
}

/// Get a visual mark for '<' or '>'.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[export_name = "mark_get_visual"]
pub unsafe extern "C" fn rs_mark_get_visual(buf: BufHandle, name: c_int) -> *mut FmarkT {
    if name != c_int::from(b'<') && name != c_int::from(b'>') {
        return std::ptr::null_mut();
    }
    let startp = nvim_mark_buf_get_visual_start(buf);
    let endp = nvim_mark_buf_get_visual_end(buf);
    let use_end = rs_visual_mark_select(startp.lnum, startp.col, endp.lnum, endp.col, name);
    let mark = if use_end == 0 {
        rs_pos_to_mark(buf, std::ptr::null_mut(), startp)
    } else {
        rs_pos_to_mark(buf, std::ptr::null_mut(), endp)
    };

    let vi_mode = nvim_mark_buf_get_visual_mode(buf);
    if vi_mode == c_int::from(b'V') {
        if name == c_int::from(b'<') {
            (*mark).mark.col = 0;
        } else {
            (*mark).mark.col = MAXCOL;
        }
        (*mark).mark.coladd = 0;
    }
    mark
}

/// Get a named mark. Dispatcher to global/local.
///
/// # Safety
/// All handle params must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_get(
    buf: BufHandle,
    win: WinHandle,
    fmp: *mut FmarkT,
    flag: c_int,
    name: c_int,
) -> *mut FmarkT {
    let mut fm: *mut FmarkT = std::ptr::null_mut();
    if rs_mark_is_file_mark(name) {
        let resolve = if flag != MARK_ALL_NO_RESOLVE { 1 } else { 0 };
        let xfm = rs_mark_get_global(resolve, name);
        fm = &raw mut (*xfm).fmark;
        if flag == MARK_BUF_LOCAL && (*xfm).fmark.fnum != nvim_buf_get_handle(buf) {
            let zero_pos = PosT {
                lnum: 0,
                col: 0,
                coladd: 0,
            };
            return rs_pos_to_mark(buf, std::ptr::null_mut(), zero_pos);
        }
    } else if name > 0 && name < NMARK_LOCAL_MAX {
        let curbuf_ptr = nvim_mark_get_curbuf();
        fm = rs_mark_get_local(buf, win, name, curbuf_ptr);
    }
    if !fmp.is_null() && !fm.is_null() {
        *fmp = *fm;
        return fmp;
    }
    fm
}

/// Set a global mark.
///
/// # Safety
/// `fm` is copied by value. Global namedfm array must be accessible.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_set_global(name: c_int, fm: XfmarkT, update: c_int) -> c_int {
    let idx = rs_mark_global_index(name);
    if idx == -1 {
        return 0;
    }
    let namedfm = nvim_mark_get_namedfm();
    let fm_tgt = namedfm.offset(idx as isize);
    if update != 0 && fm.fmark.timestamp <= (*fm_tgt).fmark.timestamp {
        return 0;
    }
    if (*fm_tgt).fmark.mark.lnum != 0 {
        rs_free_xfmark(*fm_tgt);
    }
    *fm_tgt = fm;
    1
}

/// Set a local mark in a buffer.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_mark_set_local(
    name: c_int,
    buf: BufHandle,
    fm: FmarkT,
    update: c_int,
) -> c_int {
    let idx = rs_mark_local_index(name);
    let fm_tgt: *mut FmarkT;

    if rs_mark_is_valid_named(name) {
        fm_tgt = nvim_mark_buf_get_namedm(buf, idx);
    } else if rs_mark_is_last_cursor(name) {
        fm_tgt = nvim_mark_buf_get_last_cursor(buf);
    } else if rs_mark_is_last_insert(name) {
        fm_tgt = nvim_mark_buf_get_last_insert(buf);
    } else if name == c_int::from(b':') {
        fm_tgt = nvim_mark_buf_get_prompt_start(buf);
    } else if rs_mark_is_last_change(name) {
        fm_tgt = nvim_mark_buf_get_last_change(buf);
    } else {
        return 0;
    }

    if update != 0 && fm.timestamp <= (*fm_tgt).timestamp {
        return 0;
    }
    if (*fm_tgt).mark.lnum != 0 {
        rs_free_fmark(*fm_tgt);
    }
    *fm_tgt = fm;
    1
}

/// Clear all marks and change list in the given buffer.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[export_name = "clrallmarks"]
pub unsafe extern "C" fn rs_clrallmarks(buf: BufHandle, timestamp: Timestamp) {
    for i in 0..NMARKS {
        let fm = nvim_mark_buf_get_namedm(buf, i);
        rs_clear_fmark(fm, timestamp);
    }
    let last_cursor = nvim_mark_buf_get_last_cursor(buf);
    rs_clear_fmark(last_cursor, timestamp);
    (*last_cursor).mark.lnum = 1;
    let last_insert = nvim_mark_buf_get_last_insert(buf);
    rs_clear_fmark(last_insert, timestamp);
    let last_change = nvim_mark_buf_get_last_change(buf);
    rs_clear_fmark(last_change, timestamp);
    let op_start = nvim_mark_buf_get_op_start(buf);
    (*op_start).lnum = 0;
    let op_end = nvim_mark_buf_get_op_end(buf);
    (*op_end).lnum = 0;
    let changelist_len = nvim_mark_buf_get_changelistlen(buf);
    for i in 0..changelist_len {
        let cl = nvim_mark_buf_get_changelist(buf, i);
        rs_clear_fmark(cl, timestamp);
    }
    nvim_mark_buf_set_changelistlen(buf, 0);
}

/// Get filename from a mark. Returns allocated string.
///
/// # Safety
/// `fmark` must be valid. `curbuf_ptr` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_fm_getname(
    fmark: *mut FmarkT,
    lead_len: c_int,
    curbuf_ptr: BufHandle,
) -> *mut c_char {
    if (*fmark).fnum == nvim_buf_get_fnum(curbuf_ptr) {
        return nvim_mark_mark_line(&raw mut (*fmark).mark, lead_len);
    }
    nvim_mark_buflist_nr2name((*fmark).fnum, 0, 1)
}

/// Check all file marks for a name that matches the file name in buf.
///
/// Iterates global marks (namedfm) and all windows in the current tabpage's
/// jumplist, calling rs_fmarks_check_one on each entry.
/// This is the Rust implementation of the C `fmarks_check_names` function.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_fmarks_check_names(buf: BufHandle) {
    let name = nvim_buf_get_ffname(buf);
    if name.is_null() {
        return;
    }

    // Check global marks (namedfm[0..NGLOBALMARKS])
    let namedfm = nvim_mark_get_namedfm();
    for i in 0..NGLOBALMARKS {
        rs_fmarks_check_one(namedfm.offset(i as isize), name, buf);
    }

    // Check jumplist of all windows in the current tabpage
    let curtab = nvim_mark_get_curtab();
    let mut win = nvim_mark_tabpage_firstwin(curtab);
    while !win.is_null() {
        let jumplistlen = nvim_mark_win_get_jumplistlen(win);
        for i in 0..jumplistlen {
            rs_fmarks_check_one(nvim_mark_win_get_jumplist_entry(win, i), name, buf);
        }
        win = nvim_mark_win_get_next(win);
    }
}

/// Set named mark "c" to position "pos".
///
/// When "c" is upper case use file "fnum".
/// Returns OK (0) on success, FAIL (1) if bad name given.
///
/// This is the Rust implementation of the C `setmark_pos` function.
///
/// # Safety
/// - `pos` must be a valid non-null pointer to a PosT.
/// - `view_pt` may be null (treated as no view).
#[no_mangle]
pub unsafe extern "C" fn rs_setmark_pos(
    c: c_int,
    pos: *mut PosT,
    fnum: c_int,
    view_pt: *const FmarkvT,
) -> c_int {
    const OK: c_int = 0;
    const FAIL: c_int = 1;
    const NUL: c_int = 0;

    // Dereference the view or use the default (MAXLNUM = no view).
    let view = if view_pt.is_null() {
        FmarkvT {
            topline_offset: MAXLNUM,
        }
    } else {
        *view_pt
    };

    // Check for a special key (may cause islower() to crash).
    if c < 0 {
        return FAIL;
    }

    let curwin = nvim_mark_get_curwin();

    if rs_mark_is_jump_mark(c) {
        // Compare pointer to see if pos is &curwin->w_cursor
        let cursor_ptr = nvim_mark_win_get_cursor_ptr(curwin);
        if pos == cursor_ptr {
            // setpcmark() then keep prev_pcmark
            nvim_mark_setpcmark();
            let pcmark = nvim_mark_win_get_pcmark_ptr(curwin);
            let prev_pcmark = nvim_mark_win_get_prev_pcmark_ptr(curwin);
            *prev_pcmark = *pcmark;
        } else {
            let pcmark = nvim_mark_win_get_pcmark_ptr(curwin);
            *pcmark = *pos;
        }
        return OK;
    }

    // Can't set a mark in a non-existent buffer.
    let buf = nvim_mark_buflist_findnr(fnum);
    if buf.is_null() {
        return FAIL;
    }

    if rs_mark_is_last_cursor(c) {
        let last_cursor = nvim_mark_buf_get_last_cursor(buf);
        // RESET_FMARK: free old, set new
        rs_free_fmark(*last_cursor);
        let buf_fnum = nvim_buf_get_fnum(buf);
        *last_cursor = FmarkT {
            mark: *pos,
            fnum: buf_fnum,
            timestamp: nvim_mark_os_time(),
            view,
            additional_data: std::ptr::null_mut(),
        };
        return OK;
    }

    // Allow setting '[ and '] for an autocommand that simulates reading a file.
    if rs_mark_is_sentence(c) {
        let op_ptr = if c == c_int::from(b'[') {
            nvim_mark_buf_get_op_start(buf)
        } else {
            nvim_mark_buf_get_op_end(buf)
        };
        *op_ptr = *pos;
        return OK;
    }

    if rs_mark_is_visual(c) {
        let vis_ptr = if c == c_int::from(b'<') {
            nvim_mark_buf_get_visual_start_ptr(buf)
        } else {
            nvim_mark_buf_get_visual_end_ptr(buf)
        };
        *vis_ptr = *pos;
        if nvim_mark_buf_get_visual_mode(buf) == NUL {
            // Visual_mode has not yet been set, use a sane default.
            nvim_mark_buf_set_visual_mode(buf, c_int::from(b'v'));
        }
        return OK;
    }

    if c == c_int::from(b':') && nvim_mark_bt_prompt(buf) != 0 {
        let prompt_start = nvim_mark_buf_get_prompt_start(buf);
        let buf_fnum = nvim_buf_get_fnum(buf);
        rs_free_fmark(*prompt_start);
        *prompt_start = FmarkT {
            mark: *pos,
            fnum: buf_fnum,
            timestamp: nvim_mark_os_time(),
            view,
            additional_data: std::ptr::null_mut(),
        };
        return OK;
    }

    let local_idx = rs_mark_local_index(c);
    if rs_mark_is_valid_named(c) {
        let fm_ptr = nvim_mark_buf_get_namedm(buf, local_idx);
        rs_free_fmark(*fm_ptr);
        *fm_ptr = FmarkT {
            mark: *pos,
            fnum,
            timestamp: nvim_mark_os_time(),
            view,
            additional_data: std::ptr::null_mut(),
        };
        return OK;
    }

    let global_idx = rs_mark_global_index(c);
    if global_idx >= 0 {
        let namedfm = nvim_mark_get_namedfm();
        let xfm_ptr = namedfm.offset(global_idx as isize);
        rs_free_xfmark(*xfm_ptr);
        *xfm_ptr = XfmarkT {
            fmark: FmarkT {
                mark: *pos,
                fnum,
                timestamp: nvim_mark_os_time(),
                view,
                additional_data: std::ptr::null_mut(),
            },
            fname: std::ptr::null_mut(),
        };
        return OK;
    }

    FAIL
}

// =============================================================================
// Phase 7 Tests
// =============================================================================

#[cfg(test)]
mod phase7_tests {
    use super::*;

    #[test]
    fn test_delmarks_parse_single() {
        // Single lowercase
        let result = rs_delmarks_parse_range(c_int::from(b'a'), 0, 0);
        assert_eq!(result.from, c_int::from(b'a'));
        assert_eq!(result.to, c_int::from(b'a'));
        assert_eq!(result.error, 0);
        assert_eq!(result.consumed, 1);

        // Single uppercase
        let result = rs_delmarks_parse_range(c_int::from(b'Z'), 0, 0);
        assert_eq!(result.error, 0);

        // Single digit
        let result = rs_delmarks_parse_range(c_int::from(b'5'), 0, 0);
        assert_eq!(result.error, 0);
    }

    #[test]
    fn test_delmarks_parse_range() {
        // Valid lowercase range a-z
        let result =
            rs_delmarks_parse_range(c_int::from(b'a'), c_int::from(b'-'), c_int::from(b'z'));
        assert_eq!(result.from, c_int::from(b'a'));
        assert_eq!(result.to, c_int::from(b'z'));
        assert_eq!(result.error, 0);
        assert_eq!(result.consumed, 3);

        // Valid digit range 0-5
        let result =
            rs_delmarks_parse_range(c_int::from(b'0'), c_int::from(b'-'), c_int::from(b'5'));
        assert_eq!(result.error, 0);

        // Invalid: mixed types a-Z
        let result =
            rs_delmarks_parse_range(c_int::from(b'a'), c_int::from(b'-'), c_int::from(b'Z'));
        assert_eq!(result.error, 1);

        // Invalid: reversed range z-a
        let result =
            rs_delmarks_parse_range(c_int::from(b'z'), c_int::from(b'-'), c_int::from(b'a'));
        assert_eq!(result.error, 1);
    }

    #[test]
    fn test_delmarks_parse_special() {
        let special_marks = [b'"', b'^', b'.', b'[', b']', b'<', b'>'];
        for &mark in &special_marks {
            let result = rs_delmarks_parse_range(c_int::from(mark), 0, 0);
            assert_eq!(result.error, 0);
            assert_eq!(result.consumed, 1);
        }

        // Invalid special mark
        let result = rs_delmarks_parse_range(c_int::from(b'@'), 0, 0);
        assert_eq!(result.error, 2);
    }

    #[test]
    fn test_delmarks_global_idx() {
        // Uppercase A-Z -> 0-25
        assert_eq!(rs_delmarks_global_idx(c_int::from(b'A')), 0);
        assert_eq!(rs_delmarks_global_idx(c_int::from(b'Z')), 25);

        // Digits 0-9 -> NMARKS + 0-9
        assert_eq!(rs_delmarks_global_idx(c_int::from(b'0')), NMARKS as c_int);
        assert_eq!(
            rs_delmarks_global_idx(c_int::from(b'9')),
            NMARKS as c_int + 9
        );

        // Lowercase - not a global mark
        assert_eq!(rs_delmarks_global_idx(c_int::from(b'a')), -1);
    }

    #[test]
    fn test_delmarks_special_type() {
        assert_eq!(rs_delmarks_special_type(c_int::from(b'"')), 1);
        assert_eq!(rs_delmarks_special_type(c_int::from(b'^')), 2);
        assert_eq!(rs_delmarks_special_type(c_int::from(b'.')), 3);
        assert_eq!(rs_delmarks_special_type(c_int::from(b'[')), 4);
        assert_eq!(rs_delmarks_special_type(c_int::from(b']')), 5);
        assert_eq!(rs_delmarks_special_type(c_int::from(b'<')), 6);
        assert_eq!(rs_delmarks_special_type(c_int::from(b'>')), 7);
        assert_eq!(rs_delmarks_special_type(c_int::from(b':')), 8);
        assert_eq!(rs_delmarks_special_type(c_int::from(b' ')), 9);
        assert_eq!(rs_delmarks_special_type(c_int::from(b'a')), 0);
    }

    #[test]
    fn test_marks_index_to_char() {
        // Global uppercase A-Z
        assert_eq!(rs_marks_index_to_char(0, 1), c_int::from(b'A'));
        assert_eq!(rs_marks_index_to_char(25, 1), c_int::from(b'Z'));

        // Global digits 0-9
        assert_eq!(
            rs_marks_index_to_char(NMARKS as c_int, 1),
            c_int::from(b'0')
        );
        assert_eq!(
            rs_marks_index_to_char(NMARKS as c_int + 9, 1),
            c_int::from(b'9')
        );

        // Local lowercase a-z
        assert_eq!(rs_marks_index_to_char(0, 0), c_int::from(b'a'));
        assert_eq!(rs_marks_index_to_char(25, 0), c_int::from(b'z'));
    }
}
