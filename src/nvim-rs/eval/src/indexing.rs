//! Buffer index conversion and pattern matching utilities migrated from eval.c.
//!
//! - `buf_byteidx_to_charidx`: Byte→char index in buffer line
//! - `buf_charidx_to_byteidx`: Char→byte index in buffer line
//! - `pattern_match`: Regex pattern match wrapper
//! - `var2fpos`: Convert typval to buffer position
//! - `list2fpos`: Convert list typval to position

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr,
    clippy::borrow_as_ptr,
    unsafe_op_in_unsafe_fn
)]

use std::ffi::{c_char, c_int, c_void};

use super::typval::TypvalT as TypvalTRepr;

/// Inline: get v_list field from typval (replaces nvim_eval_tv_get_list).
#[inline]
unsafe fn tv_get_list_field(tv: *const c_void) -> *mut c_void {
    (*tv.cast::<TypvalTRepr>()).vval.v_list
}

/// Maximum number of subexpressions in a regexp (from regexp_defs.h)
const NSUBEXP: usize = 10;

/// RE_MAGIC flag for vim_regcomp (verified by _Static_assert in eval.c)
const RE_MAGIC: c_int = 1;
/// RE_STRING flag for vim_regcomp (verified by _Static_assert in eval.c)
const RE_STRING: c_int = 2;

/// Opaque buffer handle (buf_T*)
type BufHandle = *mut c_void;

extern "C" {
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // Buffer accessors (defined in eval.c)
    fn nvim_eval_buf_ml_valid(buf: BufHandle) -> c_int;
    #[link_name = "nvim_buf_get_ml_line_count"]
    fn nvim_eval_buf_line_count(buf: BufHandle) -> i32;
    #[link_name = "ml_get_buf"]
    fn nvim_eval_ml_get_buf(buf: BufHandle, lnum: i32) -> *const c_char;

    // Regex functions
    fn vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;
    fn vim_regexec_nl(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> c_int;
    fn vim_regfree(prog: *mut c_void);

    // p_cpo save/restore accessors (defined in eval.c)
    fn nvim_eval_save_set_cpo();
    fn nvim_eval_restore_cpo();
}

/// Structure matching regmatch_T for single-line matching.
/// Must match the C layout exactly.
#[repr(C)]
struct RegMatch {
    regprog: *mut c_void,
    startp: [*mut c_char; NSUBEXP],
    endp: [*mut c_char; NSUBEXP],
    rm_matchcol: c_int,
    rm_ic: bool,
}

impl Default for RegMatch {
    fn default() -> Self {
        Self {
            regprog: std::ptr::null_mut(),
            startp: [std::ptr::null_mut(); NSUBEXP],
            endp: [std::ptr::null_mut(); NSUBEXP],
            rm_matchcol: 0,
            rm_ic: false,
        }
    }
}

/// Convert the specified byte index of line `lnum` in buffer `buf` to a
/// character index. Works only for loaded buffers.
///
/// Returns -1 on failure.
///
/// # Safety
///
/// `buf` must be a valid buffer handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_byteidx_to_charidx(
    buf: BufHandle,
    mut lnum: i32,
    byteidx: c_int,
) -> c_int {
    if buf.is_null() || nvim_eval_buf_ml_valid(buf) == 0 {
        return -1;
    }

    let line_count = nvim_eval_buf_line_count(buf);
    if lnum > line_count {
        lnum = line_count;
    }

    let str = nvim_eval_ml_get_buf(buf, lnum);

    if *str == 0 {
        return 0;
    }

    // count the number of characters
    let mut t = str;
    let mut count: c_int = 0;
    while *t != 0 && t <= str.add(byteidx as usize) {
        t = t.add(utfc_ptr2len(t) as usize);
        count += 1;
    }

    // In insert mode, when the cursor is at the end of a non-empty line,
    // byteidx points to the NUL character immediately past the end of the
    // string. In this case, add one to the character count.
    if *t == 0 && byteidx != 0 && t == str.add(byteidx as usize) {
        count += 1;
    }

    count - 1
}

/// Convert the specified character index of line `lnum` in buffer `buf` to a
/// byte index. Works only for loaded buffers.
///
/// Returns -1 on failure.
///
/// # Safety
///
/// `buf` must be a valid buffer handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_charidx_to_byteidx(
    buf: BufHandle,
    mut lnum: i32,
    mut charidx: c_int,
) -> c_int {
    if buf.is_null() || nvim_eval_buf_ml_valid(buf) == 0 {
        return -1;
    }

    let line_count = nvim_eval_buf_line_count(buf);
    if lnum > line_count {
        lnum = line_count;
    }

    let str = nvim_eval_ml_get_buf(buf, lnum);

    // Convert the character offset to a byte offset
    let mut t = str;
    charidx -= 1;
    while *t != 0 && charidx > 0 {
        t = t.add(utfc_ptr2len(t) as usize);
        charidx -= 1;
    }

    t.offset_from(str) as c_int
}

/// Check if `pat` matches `text`.
///
/// Does not use 'cpo' and always uses 'magic'.
///
/// # Safety
///
/// `pat` and `text` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn pattern_match(pat: *const c_char, text: *const c_char, ic: bool) -> c_int {
    let mut matches: c_int = 0;

    // avoid 'l' flag in 'cpoptions'
    nvim_eval_save_set_cpo();

    let prog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !prog.is_null() {
        let mut regmatch = RegMatch {
            regprog: prog,
            rm_ic: ic,
            ..RegMatch::default()
        };
        matches = vim_regexec_nl(&raw mut regmatch, text, 0);
        vim_regfree(regmatch.regprog);
    }

    nvim_eval_restore_cpo();
    matches
}

// =============================================================================
// Position conversion: var2fpos / list2fpos
// =============================================================================

/// pos_T structure matching Neovim's pos_defs.h
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PosT {
    /// line number
    pub lnum: i32,
    /// column number
    pub col: i32,
    /// column add (for virtual columns)
    pub coladd: i32,
}

/// Bulk cursor/visual state (Phase 15).
/// Mirror of NvimCursorVisualState typedef in eval.h.
/// Must stay in sync with _Static_assert in eval_shim.c.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct NvimCursorVisualState {
    pub cursor_lnum: i32,
    pub cursor_col: c_int,
    pub cursor_coladd: c_int,
    pub topline: i32,
    pub botline: i32,
    pub visual_active: bool,
    // 3 bytes implicit padding (alignment of i32)
    pub visual_lnum: i32,
    pub visual_col: c_int,
    pub visual_coladd: c_int,
    pub curbuf_fnum: c_int,
}

/// VAR_LIST type constant (verified by _Static_assert in eval.c)
const VAR_LIST: c_int = 4;
/// OK return value
const OK: c_int = 1;
/// FAIL return value
const FAIL: c_int = 0;

/// Opaque list handle (list_T*)
type ListHandle = *mut c_void;

extern "C" {
    #[link_name = "tv_get_string_chk"]
    fn nvim_eval_tv_string_chk(tv: *mut c_void) -> *const c_char;
    fn tv_list_find_nr(l: ListHandle, n: c_int, error_out: *mut bool) -> i64;
    fn nvim_tv_list_item_is_dollar(l: ListHandle, idx: c_int) -> bool;
    fn nvim_tv_list_len(l: *const c_void) -> c_int;

    // Buffer accessors (Phase 3)
    fn nvim_curbuf_fnum() -> c_int;
    fn nvim_get_line_count() -> i32;
    #[link_name = "rs_buflist_findnr"]
    fn nvim_buflist_findnr(fnum: c_int) -> BufHandle;

    // Bulk cursor/visual state (Phase 15 bulk-read replaces 9 individual accessors)
    fn nvim_read_cursor_visual_state(out: *mut NvimCursorVisualState);

    // Mark accessor (Phase 3)
    fn nvim_mark_get_wrapper(
        mname: c_int,
        lnum_out: *mut i32,
        col_out: *mut c_int,
        coladd_out: *mut c_int,
        fnum_out: *mut c_int,
    ) -> bool;

    // Window validation (Phase 3)
    fn nvim_update_topline_curwin();
    fn nvim_validate_botline_curwin();
    fn nvim_check_cursor_moved_curwin();

    // Line length helpers (Phase 3)
    #[link_name = "ml_get_len"]
    fn nvim_ml_get_len(lnum: i32) -> c_int;
    fn nvim_mb_charlen_ml(lnum: i32) -> c_int;
    fn nvim_get_cursor_line_len() -> c_int;
    fn nvim_get_cursor_line_charlen() -> c_int;
}

/// Check if a character is an ASCII uppercase letter (A-Z).
#[inline]
const fn ascii_isupper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// Check if a character is an ASCII digit (0-9).
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Convert a typval to a buffer position.
///
/// Migrated from `var2fpos` in eval_shim.c.
///
/// Returns true and fills `out` if the position is valid, false otherwise.
/// `ret_fnum` is updated with the file number if a global/numbered mark is resolved.
///
/// # Safety
///
/// `tv` must be a valid typval_T pointer, `ret_fnum` and `out` must be valid.
#[allow(clippy::too_many_lines)]
#[no_mangle]
pub unsafe extern "C" fn rs_var2fpos(
    tv: *const c_void,
    dollar_lnum: bool,
    ret_fnum: *mut c_int,
    charcol: bool,
    out: *mut PosT,
) -> bool {
    // Argument can be [lnum, col, coladd].
    if (*tv.cast::<TypvalTRepr>()).v_type == VAR_LIST {
        let l = tv_get_list_field(tv);
        if l.is_null() {
            return false;
        }

        let mut error = false;

        // Get the line number.
        let lnum = tv_list_find_nr(l, 0, &raw mut error) as i32;
        if error || lnum <= 0 || lnum > nvim_get_line_count() {
            // Invalid line number.
            return false;
        }

        // Get the column number.
        let mut col = tv_list_find_nr(l, 1, &raw mut error) as i32;
        if error {
            return false;
        }

        let len = if charcol {
            nvim_mb_charlen_ml(lnum)
        } else {
            nvim_ml_get_len(lnum)
        };

        // We accept "$" for the column number: last column.
        if nvim_tv_list_item_is_dollar(l, 1) {
            col = len + 1;
        }

        // Accept a position up to the NUL after the line.
        if col == 0 || col > len + 1 {
            // Invalid column number.
            return false;
        }
        col -= 1;

        // Get the virtual offset. Defaults to zero.
        let coladd = tv_list_find_nr(l, 2, &raw mut error) as i32;
        let coladd = if error { 0 } else { coladd };

        *out = PosT { lnum, col, coladd };
        return true;
    }

    let name_ptr = nvim_eval_tv_string_chk(tv.cast_mut());
    if name_ptr.is_null() {
        return false;
    }

    // SAFETY: name_ptr is a valid NUL-terminated C string from nvim_tv_get_string_chk
    let name = std::slice::from_raw_parts(name_ptr as *const u8, {
        let mut len = 0usize;
        while *name_ptr.add(len) != 0 {
            len += 1;
        }
        len + 1 // include NUL
    });

    // Bulk-read cursor/visual state once (Phase 15).
    let mut cvs = NvimCursorVisualState::default();
    nvim_read_cursor_visual_state(&raw mut cvs);

    let mut pos = PosT::default();

    if name[0] == b'.' {
        // cursor
        pos.lnum = cvs.cursor_lnum;
        pos.col = cvs.cursor_col;
        pos.coladd = cvs.cursor_coladd;
    } else if name[0] == b'v' && name[1] == 0 {
        // Visual start
        if cvs.visual_active {
            pos.lnum = cvs.visual_lnum;
            pos.col = cvs.visual_col;
            pos.coladd = cvs.visual_coladd;
        } else {
            pos.lnum = cvs.cursor_lnum;
            pos.col = cvs.cursor_col;
            pos.coladd = cvs.cursor_coladd;
        }
    } else if name[0] == b'\'' {
        // mark
        let mname = c_int::from(name[1]);
        let mut mark_line: i32 = 0;
        let mut mark_col: c_int = 0;
        let mut mark_coladd: c_int = 0;
        let mut mark_filenum: c_int = 0;
        if !nvim_mark_get_wrapper(
            mname,
            &raw mut mark_line,
            &raw mut mark_col,
            &raw mut mark_coladd,
            &raw mut mark_filenum,
        ) {
            return false;
        }
        pos.lnum = mark_line;
        pos.col = mark_col;
        pos.coladd = mark_coladd;
        // Vimscript behavior: only provide fnum if mark is global.
        if ascii_isupper(name[1]) || ascii_isdigit(name[1]) {
            *ret_fnum = mark_filenum;
        }
    }

    if pos.lnum != 0 {
        if charcol {
            // Convert byte column to character column using current buffer.
            // We can reuse rs_buf_byteidx_to_charidx but it takes a BufHandle.
            // Instead replicate the logic: just call through the existing Rust fn.
            // Use curbuf (accessed via nvim_curbuf_fnum to get fnum, then buflist_findnr).
            let curbuf = nvim_buflist_findnr(nvim_curbuf_fnum());
            pos.col = rs_buf_byteidx_to_charidx(curbuf, pos.lnum, pos.col);
        }
        *out = pos;
        return true;
    }

    pos.coladd = 0;

    if name[0] == b'w' && dollar_lnum {
        // the "w_valid" flags are not reset when moving the cursor, but they
        // do matter for update_topline() and validate_botline().
        nvim_check_cursor_moved_curwin();

        pos.col = 0;
        if name[1] == b'0' {
            // "w0": first visible line
            nvim_update_topline_curwin();
            // Re-read state after update (topline may have changed).
            nvim_read_cursor_visual_state(&raw mut cvs);
            // In silent Ex mode topline is zero, but that's not a valid line
            // number; use one instead.
            pos.lnum = if cvs.topline > 0 { cvs.topline } else { 1 };
            *out = pos;
            return true;
        } else if name[1] == b'$' {
            // "w$": last visible line
            nvim_validate_botline_curwin();
            // Re-read state after update (botline may have changed).
            nvim_read_cursor_visual_state(&raw mut cvs);
            // In silent Ex mode botline is zero, return zero then.
            pos.lnum = if cvs.botline > 0 { cvs.botline - 1 } else { 0 };
            *out = pos;
            return true;
        }
    } else if name[0] == b'$' {
        // last column or line
        if dollar_lnum {
            pos.lnum = nvim_get_line_count();
            pos.col = 0;
        } else {
            pos.lnum = cvs.cursor_lnum;
            if charcol {
                pos.col = nvim_get_cursor_line_charlen();
            } else {
                pos.col = nvim_get_cursor_line_len();
            }
        }
        *out = pos;
        return true;
    }

    false
}

/// Thin exported wrapper for `rs_var2fpos` that owns the static pos_T.
///
/// Replaces the C `var2fpos` wrapper in eval_shim.c (Phase 12).
/// Returns a pointer to a Rust-owned static on success, null on failure.
///
/// # Safety
///
/// Same as `rs_var2fpos`. Single-threaded; static is safe under Neovim's
/// cooperative concurrency model (same guarantee as the original C static).
// SAFETY: Neovim is single-threaded for VimL evaluation; the static matches
// the behaviour of the original C `static pos_T pos` in the C wrapper.
#[allow(static_mut_refs)]
#[export_name = "var2fpos"]
pub unsafe extern "C" fn rs_var2fpos_export(
    tv: *const c_void,
    dollar_lnum: bool,
    ret_fnum: *mut c_int,
    charcol: bool,
) -> *mut PosT {
    static mut VAR2FPOS_BUF: PosT = PosT {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    if rs_var2fpos(tv, dollar_lnum, ret_fnum, charcol, &raw mut VAR2FPOS_BUF) {
        &raw mut VAR2FPOS_BUF
    } else {
        std::ptr::null_mut()
    }
}

/// Convert a list typval to a buffer position.
///
/// Migrated from `list2fpos` in eval_shim.c.
///
/// # Safety
///
/// `arg`, `posp` must be valid pointers.
#[export_name = "list2fpos"]
pub unsafe extern "C" fn rs_list2fpos(
    arg: *const c_void,
    posp: *mut PosT,
    fnump: *mut c_int,
    curswantp: *mut c_int,
    charcol: bool,
) -> c_int {
    // Validate: must be a list
    if (*arg.cast::<TypvalTRepr>()).v_type != VAR_LIST {
        return FAIL;
    }
    let l = tv_get_list_field(arg);
    if l.is_null() {
        return FAIL;
    }

    let list_len = nvim_tv_list_len(l);

    // List must be: [fnum, lnum, col, coladd, curswant], where "fnum" is only
    // there when "fnump" isn't NULL; "coladd" and "curswant" are optional.
    let min_len = if fnump.is_null() { 2 } else { 3 };
    let max_len = if fnump.is_null() { 4 } else { 5 };
    if list_len < min_len || list_len > max_len {
        return FAIL;
    }

    let mut i: c_int = 0;

    if !fnump.is_null() {
        let n = tv_list_find_nr(l, i, std::ptr::null_mut()) as c_int;
        i += 1;
        if n < 0 {
            return FAIL;
        }
        let n = if n == 0 { nvim_curbuf_fnum() } else { n };
        *fnump = n;
    }

    let lnum = tv_list_find_nr(l, i, std::ptr::null_mut()) as i32;
    i += 1;
    if lnum < 0 {
        return FAIL;
    }
    (*posp).lnum = lnum;

    let mut col = tv_list_find_nr(l, i, std::ptr::null_mut()) as c_int;
    i += 1;
    if col < 0 {
        return FAIL;
    }

    // If character position is specified, then convert to byte position.
    // If the line number is zero use the cursor line.
    if charcol {
        // Get the text for the specified line in a loaded buffer.
        let fnum = if fnump.is_null() {
            nvim_curbuf_fnum()
        } else {
            *fnump
        };
        let buf = nvim_buflist_findnr(fnum);
        if buf.is_null() || nvim_eval_buf_ml_valid(buf) == 0 {
            return FAIL;
        }
        let use_lnum = if lnum == 0 {
            let mut cvs2 = NvimCursorVisualState::default();
            nvim_read_cursor_visual_state(&raw mut cvs2);
            cvs2.cursor_lnum
        } else {
            lnum
        };
        col = rs_buf_charidx_to_byteidx(buf, use_lnum, col) + 1;
    }
    (*posp).col = col;

    let off = tv_list_find_nr(l, i, std::ptr::null_mut()) as c_int;
    (*posp).coladd = if off < 0 { 0 } else { off };

    if !curswantp.is_null() {
        *curswantp = tv_list_find_nr(l, i + 1, std::ptr::null_mut()) as c_int;
    }

    OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(RE_MAGIC, 1);
        assert_eq!(RE_STRING, 2);
        assert_eq!(NSUBEXP, 10);
    }
}
