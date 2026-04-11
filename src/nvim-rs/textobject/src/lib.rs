//! Text object selection and navigation for Neovim
//!
//! This crate provides Rust implementations of text object functions
//! from `src/nvim/textobject.c`. It handles text object selection (aw, iw, as, is,
//! ap, ip, a", i", a{, i{, etc.) and word motions (w, W, b, B, e, E).

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)] // export_name FFI functions don't need #[must_use]

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// Direction: move forward
pub const FORWARD: c_int = 1;

/// Direction: move backward
pub const BACKWARD: c_int = -1;

/// Function succeeded
pub const OK: c_int = 1;

/// Function failed
pub const FAIL: c_int = 0;

/// NUL character
pub const NUL: c_int = 0;

/// Whitespace character class
const CLASS_WHITESPACE: c_int = 0;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle to a buffer (buf_T*).
pub type BufHandle = *mut std::ffi::c_void;

/// Opaque handle to operator arguments (oparg_T*).
pub type OapHandle = *mut std::ffi::c_void;

/// Opaque handle to a position (pos_T*).
pub type PosHandle = *mut std::ffi::c_void;

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    fn gchar_cursor() -> c_int;
    fn inc_cursor() -> c_int;
    fn dec_cursor() -> c_int;
    fn utf_class(c: c_int) -> c_int;
    fn nvim_textobj_get_cursor_col() -> c_int;
    fn nvim_textobj_get_cursor_lnum() -> c_int;
    fn nvim_textobj_get_ml_line_count() -> c_int;
    fn nvim_textobj_is_lineempty(lnum: c_int) -> bool;
    fn get_cursor_line_ptr() -> *const std::ffi::c_char;
    fn nvim_textobj_set_cursor_coladd_zero();
    fn nvim_textobj_hasFolding(lnum: c_int, first: *mut c_int, last: *mut c_int) -> bool;
    fn nvim_get_curwin() -> *mut std::ffi::c_void;
    fn coladvance(wp: *mut std::ffi::c_void, col: c_int) -> c_int;
    fn adjust_skipcol();
}

// =============================================================================
// Character Classification
// =============================================================================

/// Get the class of character at cursor position.
///
/// Character classes:
/// - 0: whitespace (space, tab, NUL)
/// - 1: punctuation (or all non-blank when bigword is true)
/// - 2+: keyword characters (letters, digits, underscore)
///
/// If `bigword` is true (W, B, E motions), all non-blank characters
/// are reported as class 1 since only whitespace boundaries matter.
#[inline]
fn cls_impl(bigword: bool) -> c_int {
    // SAFETY: Accessor function is provided by C side
    let c = unsafe { gchar_cursor() };

    // Whitespace check: space, tab, or NUL
    if c == i32::from(b' ') || c == i32::from(b'\t') || c == NUL {
        return CLASS_WHITESPACE;
    }

    // SAFETY: Accessor function is provided by C side
    let class = unsafe { utf_class(c) };

    // If bigword is true, report all non-blanks as class 1
    if class != 0 && bigword {
        return 1;
    }

    class
}

/// FFI wrapper for cls() - get character class at cursor.
///
/// # Safety
/// Calls into C accessor functions which must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_cls(bigword: bool) -> c_int {
    cls_impl(bigword)
}

/// Skip characters of the same class in the given direction.
///
/// Returns true when end-of-file/start-of-file is reached, false otherwise.
#[inline]
fn skip_chars_impl(cclass: c_int, dir: c_int, bigword: bool) -> bool {
    // SAFETY: Accessor functions are provided by C side
    unsafe {
        while cls_impl(bigword) == cclass {
            let result = if dir == FORWARD {
                inc_cursor()
            } else {
                dec_cursor()
            };
            if result == -1 {
                return true;
            }
        }
    }
    false
}

/// FFI wrapper for skip_chars() - skip characters of same class.
///
/// # Safety
/// Calls into C accessor functions which must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_chars(cclass: c_int, dir: c_int, bigword: bool) -> bool {
    skip_chars_impl(cclass, dir, bigword)
}

/// Go back to the start of the word or the start of whitespace.
///
/// Moves cursor backward until it reaches the start of the line
/// or a different character class boundary.
#[inline]
fn back_in_line_impl(bigword: bool) {
    let sclass = cls_impl(bigword); // starting class

    // SAFETY: Accessor functions are provided by C side
    unsafe {
        loop {
            // Stop at start of line
            if nvim_textobj_get_cursor_col() == 0 {
                break;
            }

            dec_cursor();

            // Stop at start of word (different class)
            if cls_impl(bigword) != sclass {
                inc_cursor();
                break;
            }
        }
    }
}

/// FFI wrapper for back_in_line() - go back to word start.
///
/// # Safety
/// Calls into C accessor functions which must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_back_in_line(bigword: bool) {
    back_in_line_impl(bigword);
}

// =============================================================================
// Word Motion Functions
// =============================================================================

/// MAXCOL constant for moving to end of line.
const MAXCOL: c_int = 0x7FFF_FFFF;

/// Move forward one word.
///
/// Returns FAIL if the cursor was already at the end of the file.
/// If `eol` is true, last word stops at end of line (for operators).
///
/// # Safety
/// Calls into C accessor functions which must be valid.
#[export_name = "fwd_word"]
pub unsafe extern "C" fn rs_fwd_word(count: c_int, bigword: bool, eol: bool) -> c_int {
    fwd_word_impl(count, bigword, eol)
}

fn fwd_word_impl(mut count: c_int, bigword: bool, eol: bool) -> c_int {
    // SAFETY: All accessor functions are provided by C side
    unsafe {
        nvim_textobj_set_cursor_coladd_zero();

        while count > 0 {
            count -= 1;

            // When inside a range of folded lines, move to the last char of the
            // last line.
            let mut last_lnum: c_int = 0;
            if nvim_textobj_hasFolding(
                nvim_textobj_get_cursor_lnum(),
                std::ptr::null_mut(),
                &raw mut last_lnum,
            ) {
                // Note: In C code, this sets cursor.lnum to last_lnum then calls coladvance
                // We need an accessor that does both, or set lnum separately
                nvim_textobj_set_cursor_lnum(last_lnum);
                coladvance(nvim_get_curwin(), MAXCOL);
            }

            let sclass = cls_impl(bigword); // starting class

            // We always move at least one character, unless on the last
            // character in the buffer.
            let last_line = nvim_textobj_get_cursor_lnum() == nvim_textobj_get_ml_line_count();
            let i = inc_cursor();
            if i == -1 || (i >= 1 && last_line) {
                // started at last char in file
                return FAIL;
            }
            if i >= 1 && eol && count == 0 {
                // started at last char in line
                return OK;
            }

            // Go one char past end of current word (if any)
            if sclass != 0 {
                loop {
                    if cls_impl(bigword) != sclass {
                        break;
                    }
                    let i = inc_cursor();
                    if i == -1 || (i >= 1 && eol && count == 0) {
                        return OK;
                    }
                }
            }

            // go to next non-white
            while cls_impl(bigword) == 0 {
                // We'll stop if we land on a blank line
                if nvim_textobj_get_cursor_col() == 0 && *get_cursor_line_ptr() == 0 {
                    break;
                }

                let i = inc_cursor();
                if i == -1 || (i >= 1 && eol && count == 0) {
                    return OK;
                }
            }
        }
    }
    OK
}

/// Move backward count words.
///
/// If `stop` is true and we are already on the start of a word, move one less.
///
/// Returns FAIL if top of the file was reached.
///
/// # Safety
/// Calls into C accessor functions which must be valid.
#[export_name = "bck_word"]
pub unsafe extern "C" fn rs_bck_word(count: c_int, bigword: bool, stop: bool) -> c_int {
    bck_word_impl(count, bigword, stop)
}

fn bck_word_impl(mut count: c_int, bigword: bool, mut stop: bool) -> c_int {
    // SAFETY: All accessor functions are provided by C side
    unsafe {
        nvim_textobj_set_cursor_coladd_zero();

        'outer: while count > 0 {
            count -= 1;

            // When inside a range of folded lines, move to the first char of the
            // first line.
            let mut first_lnum: c_int = 0;
            if nvim_textobj_hasFolding(
                nvim_textobj_get_cursor_lnum(),
                &raw mut first_lnum,
                std::ptr::null_mut(),
            ) {
                nvim_textobj_set_cursor_lnum(first_lnum);
                nvim_textobj_set_cursor_col(0);
            }

            let sclass = cls_impl(bigword);
            if dec_cursor() == -1 {
                // started at start of file
                return FAIL;
            }

            if !stop || cls_impl(bigword) == sclass || sclass == 0 {
                // Skip white space before the word.
                // Stop on an empty line.
                while cls_impl(bigword) == 0 {
                    if nvim_textobj_get_cursor_col() == 0
                        && nvim_textobj_is_lineempty(nvim_textobj_get_cursor_lnum())
                    {
                        // goto finished - skip to next iteration
                        stop = false;
                        continue 'outer;
                    }
                    if dec_cursor() == -1 {
                        // hit start of file, stop here
                        return OK;
                    }
                }

                // Move backward to start of this word.
                if skip_chars_impl(cls_impl(bigword), BACKWARD, bigword) {
                    return OK;
                }
            }

            inc_cursor(); // overshot - forward one
            stop = false;
        }
        adjust_skipcol();
    }
    OK
}

/// Move to the end of the word.
///
/// Returns FAIL if end of the file was reached.
///
/// If `stop` is true and we are already on the end of a word, move one less.
/// If `empty` is true stop on an empty line.
///
/// # Safety
/// Calls into C accessor functions which must be valid.
#[export_name = "end_word"]
pub unsafe extern "C" fn rs_end_word(
    count: c_int,
    bigword: bool,
    stop: bool,
    empty: bool,
) -> c_int {
    end_word_impl(count, bigword, stop, empty)
}

fn end_word_impl(mut count: c_int, bigword: bool, mut stop: bool, empty: bool) -> c_int {
    // SAFETY: All accessor functions are provided by C side
    unsafe {
        nvim_textobj_set_cursor_coladd_zero();

        // If adjusted cursor position previously, unadjust it.
        // This requires visual mode state - delegate to C for now
        nvim_textobj_unadjust_for_sel_if_needed();

        'outer: while count > 0 {
            count -= 1;

            // When inside a range of folded lines, move to the last char of the
            // last line.
            let mut last_lnum: c_int = 0;
            if nvim_textobj_hasFolding(
                nvim_textobj_get_cursor_lnum(),
                std::ptr::null_mut(),
                &raw mut last_lnum,
            ) {
                nvim_textobj_set_cursor_lnum(last_lnum);
                coladvance(nvim_get_curwin(), MAXCOL);
            }

            let sclass = cls_impl(bigword);
            if inc_cursor() == -1 {
                return FAIL;
            }

            // If we're in the middle of a word, we just have to move to the end
            // of it.
            if cls_impl(bigword) == sclass && sclass != 0 {
                // Move forward to end of the current word
                if skip_chars_impl(sclass, FORWARD, bigword) {
                    return FAIL;
                }
            } else if !stop || sclass == 0 {
                // We were at the end of a word. Go to the end of the next word.
                // First skip white space, if 'empty' is true, stop at empty line.
                while cls_impl(bigword) == 0 {
                    if empty
                        && nvim_textobj_get_cursor_col() == 0
                        && nvim_textobj_is_lineempty(nvim_textobj_get_cursor_lnum())
                    {
                        // goto finished - skip dec_cursor and move to next iteration
                        stop = false;
                        continue 'outer;
                    }
                    if inc_cursor() == -1 {
                        // hit end of file, stop here
                        return FAIL;
                    }
                }

                // Move forward to the end of this word.
                if skip_chars_impl(cls_impl(bigword), FORWARD, bigword) {
                    return FAIL;
                }
            }

            dec_cursor(); // overshot - one char backward
            stop = false; // we move only one word less
        }
    }
    OK
}

/// Move back to the end of the word.
///
/// Returns FAIL if start of the file was reached.
///
/// # Safety
/// Calls into C accessor functions which must be valid.
#[export_name = "bckend_word"]
pub unsafe extern "C" fn rs_bckend_word(count: c_int, bigword: bool, eol: bool) -> c_int {
    bckend_word_impl(count, bigword, eol)
}

fn bckend_word_impl(mut count: c_int, bigword: bool, eol: bool) -> c_int {
    // SAFETY: All accessor functions are provided by C side
    unsafe {
        nvim_textobj_set_cursor_coladd_zero();

        while count > 0 {
            count -= 1;

            let sclass = cls_impl(bigword); // starting class
            let i = dec_cursor();
            if i == -1 {
                return FAIL;
            }
            if eol && i == 1 {
                return OK;
            }

            // Move backward to before the start of this word.
            if sclass != 0 {
                loop {
                    if cls_impl(bigword) != sclass {
                        break;
                    }
                    let i = dec_cursor();
                    if i == -1 || (eol && i == 1) {
                        return OK;
                    }
                }
            }

            // Move backward to end of the previous word
            while cls_impl(bigword) == 0 {
                if nvim_textobj_get_cursor_col() == 0
                    && nvim_textobj_is_lineempty(nvim_textobj_get_cursor_lnum())
                {
                    break;
                }
                let i = dec_cursor();
                if i == -1 || (eol && i == 1) {
                    return OK;
                }
            }
        }
        adjust_skipcol();
    }
    OK
}

// Additional extern declarations for word motion functions
extern "C" {
    fn nvim_textobj_set_cursor_lnum(lnum: c_int);
    fn nvim_textobj_set_cursor_col(col: c_int);
    fn nvim_textobj_unadjust_for_sel_if_needed();
    fn utfc_ptr2len(p: *const std::ffi::c_char) -> c_int;
    fn utf_head_off(base: *const std::ffi::c_char, p: *const std::ffi::c_char) -> c_int;
    fn vim_strchr(s: *const std::ffi::c_char, c: c_int) -> *const std::ffi::c_char;
}

// =============================================================================
// Quote Text Object Helpers
// =============================================================================

/// Helper to convert a C char (i8/u8) to c_int safely.
/// C chars are typically used as unsigned bytes for character values.
#[inline]
fn char_to_int(c: std::ffi::c_char) -> c_int {
    // Cast to u8 first to treat as unsigned byte, then widen to c_int
    #[allow(clippy::cast_sign_loss)]
    let byte = c as u8;
    c_int::from(byte)
}

/// Search forward for a quote character.
///
/// Returns column number of quote character or -1 when not found.
///
/// # Safety
/// - `line` must be a valid pointer to a NUL-terminated C string.
/// - `col` must be a valid starting index within the string.
/// - `escape` may be null or a valid pointer to a NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_find_next_quote(
    line: *const std::ffi::c_char,
    mut col: c_int,
    quotechar: c_int,
    escape: *const std::ffi::c_char,
) -> c_int {
    loop {
        let c = char_to_int(*line.offset(col as isize));
        if c == NUL {
            return -1;
        }
        if !escape.is_null() && !vim_strchr(escape, c).is_null() {
            col += 1;
            if *line.offset(col as isize) == 0 {
                return -1;
            }
        } else if c == quotechar {
            break;
        }
        col += utfc_ptr2len(line.offset(col as isize));
    }
    col
}

/// Search backward for a quote character.
///
/// Returns found column or zero.
///
/// # Safety
/// - `line` must be a valid pointer to a NUL-terminated C string.
/// - `col_start` must be a valid starting index within the string.
/// - `escape` may be null or a valid pointer to a NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_find_prev_quote(
    line: *const std::ffi::c_char,
    mut col_start: c_int,
    quotechar: c_int,
    escape: *const std::ffi::c_char,
) -> c_int {
    while col_start > 0 {
        col_start -= 1;
        col_start -= utf_head_off(line, line.offset(col_start as isize));

        let mut n: c_int = 0;
        if !escape.is_null() {
            while col_start - n > 0 {
                let prev_char = char_to_int(*line.offset((col_start - n - 1) as isize));
                if vim_strchr(escape, prev_char).is_null() {
                    break;
                }
                n += 1;
            }
        }

        if (n & 1) != 0 {
            col_start -= n; // uneven number of escape chars, skip it
        } else if char_to_int(*line.offset(col_start as isize)) == quotechar {
            break;
        }
    }
    col_start
}

// =============================================================================
// Current Word Text Object
// =============================================================================

/// Motion type: character-wise
const MT_CHAR_WISE: c_int = 0;

/// Update type for redraw (from drawscreen.h)
const UPD_INVERTED: c_int = 20;

extern "C" {
    /// Get VIsual_active state.
    fn nvim_textobj_get_VIsual_active() -> bool;

    /// Get VIsual position lnum.
    fn nvim_textobj_get_VIsual_lnum() -> c_int;

    /// Get VIsual position col.
    fn nvim_textobj_get_VIsual_col() -> c_int;

    /// Get VIsual_mode.
    fn nvim_textobj_get_VIsual_mode() -> c_int;

    /// Set VIsual_mode.
    fn nvim_textobj_set_VIsual_mode(mode: c_int);

    /// Set VIsual position.
    fn nvim_textobj_set_VIsual(lnum: c_int, col: c_int);

    /// Get selection option first char.
    fn nvim_textobj_get_p_sel_first() -> c_int;

    /// Check if cursor is less than VIsual.
    fn nvim_textobj_lt_cursor_VIsual() -> bool;

    /// Check if cursor equals VIsual.
    fn nvim_textobj_equalpos_cursor_VIsual() -> bool;

    /// Check if VIsual is less than cursor.
    fn nvim_textobj_lt_VIsual_cursor() -> bool;

    /// Check if VIsual is less than or equal to cursor.
    fn nvim_textobj_ltoreq_VIsual_cursor() -> bool;

    /// Set operator argument motion type.
    fn nvim_textobj_set_oap_motion_type(oap: OapHandle, motion_type: c_int);

    /// Set operator argument inclusive flag.
    fn nvim_textobj_set_oap_inclusive(oap: OapHandle, val: bool);

    fn oneleft() -> c_int;
    fn nvim_textobj_incl_cursor() -> c_int;
    fn nvim_textobj_decl_cursor() -> c_int;
    fn redraw_curbuf_later(update_type: c_int);

    /// Set redraw_cmdline flag.
    fn nvim_textobj_set_redraw_cmdline(val: bool);

    /// Get cursor position as lnum/col pair.
    fn nvim_textobj_get_cursor_pos(lnum: *mut c_int, col: *mut c_int);

    /// Set cursor position from lnum/col pair.
    fn nvim_textobj_set_cursor_pos(lnum: c_int, col: c_int);

    /// Set VIsual from cursor.
    fn nvim_textobj_set_VIsual_from_cursor();

    /// Set oap->start from stored position.
    fn nvim_textobj_set_oap_start(oap: OapHandle, lnum: c_int, col: c_int);
}

/// Find the current word under the cursor.
///
/// Handles iw, aw, iW, aW text objects.
///
/// # Safety
/// - `oap` must be a valid pointer to an oparg_T structure.
#[export_name = "current_word"]
pub unsafe extern "C" fn rs_current_word(
    oap: OapHandle,
    count: c_int,
    include: bool,
    bigword: bool,
) -> c_int {
    current_word_impl(oap, count, include, bigword)
}

/// Position stored as lnum/col pair for use across accessor calls.
#[derive(Clone, Copy, Default)]
struct SimplePos {
    lnum: c_int,
    col: c_int,
}

/// State for current_word operation.
struct CurrentWordState {
    start_pos: SimplePos,
    inclusive: bool,
    include_white: bool,
}

/// Extend word selection backward in Visual mode.
///
/// # Safety
/// Calls C accessor functions which must be valid.
unsafe fn extend_word_backward(include: bool, bigword: bool) -> c_int {
    // In Visual mode, with cursor at start: move cursor back.
    if nvim_textobj_decl_cursor() == -1 {
        return FAIL;
    }
    // Note: cls_impl(bigword) != 0 means we're on non-whitespace
    if include == (cls_impl(bigword) != 0) {
        if bckend_word_impl(1, bigword, true) == FAIL {
            return FAIL;
        }
        nvim_textobj_incl_cursor();
    } else if bck_word_impl(1, bigword, true) == FAIL {
        return FAIL;
    }
    OK
}

/// Extend word selection forward.
///
/// Returns (result, inclusive) where result is OK/FAIL.
///
/// # Safety
/// Calls C accessor functions which must be valid.
unsafe fn extend_word_forward(include: bool, bigword: bool, count: c_int) -> (c_int, bool) {
    // Move cursor forward one word and/or white area.
    if nvim_textobj_incl_cursor() == -1 {
        return (FAIL, true);
    }
    // Note: cls_impl(bigword) == 0 means we're on whitespace
    if include == (cls_impl(bigword) == 0) {
        if end_word_impl(1, bigword, true, true) == FAIL {
            return (FAIL, true);
        }
        (OK, true)
    } else {
        if fwd_word_impl(1, bigword, true) == FAIL && count > 1 {
            return (FAIL, true);
        }
        // If end is just past a new-line, we don't want to include
        // the first character on the line.
        // Put cursor on last char of white.
        let inclusive = oneleft() != FAIL;
        (OK, inclusive)
    }
}

/// Implementation of current_word.
#[allow(clippy::too_many_lines)]
fn current_word_impl(oap: OapHandle, mut count: c_int, include: bool, bigword: bool) -> c_int {
    // SAFETY: All accessor functions are provided by C side
    unsafe {
        let mut state = CurrentWordState {
            start_pos: SimplePos::default(),
            inclusive: true,
            include_white: false,
        };

        // Correct cursor when 'selection' is exclusive
        if nvim_textobj_get_VIsual_active()
            && nvim_textobj_get_p_sel_first() == i32::from(b'e')
            && nvim_textobj_lt_VIsual_cursor()
        {
            dec_cursor();
        }

        // When Visual mode is not active, or when the VIsual area is only one
        // character, select the word and/or white space under the cursor.
        if !nvim_textobj_get_VIsual_active() || nvim_textobj_equalpos_cursor_VIsual() {
            back_in_line_impl(bigword);
            nvim_textobj_get_cursor_pos(
                &raw mut state.start_pos.lnum,
                &raw mut state.start_pos.col,
            );

            if (cls_impl(bigword) == 0) == include {
                if end_word_impl(1, bigword, true, true) == FAIL {
                    return FAIL;
                }
            } else {
                fwd_word_impl(1, bigword, true);
                if nvim_textobj_get_cursor_col() == 0 {
                    nvim_textobj_decl_cursor();
                } else {
                    oneleft();
                }
                if include {
                    state.include_white = true;
                }
            }

            if nvim_textobj_get_VIsual_active() {
                nvim_textobj_set_VIsual(state.start_pos.lnum, state.start_pos.col);
                redraw_curbuf_later(UPD_INVERTED);
            } else {
                nvim_textobj_set_oap_start(oap, state.start_pos.lnum, state.start_pos.col);
                nvim_textobj_set_oap_motion_type(oap, MT_CHAR_WISE);
            }
            count -= 1;
        }

        // When count is still > 0, extend with more objects.
        while count > 0 {
            state.inclusive = true;
            if nvim_textobj_get_VIsual_active() && nvim_textobj_lt_cursor_VIsual() {
                if extend_word_backward(include, bigword) == FAIL {
                    return FAIL;
                }
            } else {
                let (result, inclusive) = extend_word_forward(include, bigword, count);
                if result == FAIL {
                    return FAIL;
                }
                state.inclusive = inclusive;
            }
            count -= 1;
        }

        current_word_adjust_whitespace(oap, &state, bigword);
        current_word_finalize(oap, &state);
    }

    OK
}

/// Adjust whitespace inclusion for "daw" style operations.
///
/// # Safety
/// Calls C accessor functions which must be valid.
unsafe fn current_word_adjust_whitespace(oap: OapHandle, state: &CurrentWordState, bigword: bool) {
    if state.include_white
        && (cls_impl(bigword) != 0 || (nvim_textobj_get_cursor_col() == 0 && !state.inclusive))
    {
        let mut saved_pos = SimplePos::default();
        nvim_textobj_get_cursor_pos(&raw mut saved_pos.lnum, &raw mut saved_pos.col);

        nvim_textobj_set_cursor_pos(state.start_pos.lnum, state.start_pos.col);
        if oneleft() == OK {
            back_in_line_impl(bigword);
            if cls_impl(bigword) == 0 && nvim_textobj_get_cursor_col() > 0 {
                if nvim_textobj_get_VIsual_active() {
                    nvim_textobj_set_VIsual_from_cursor();
                } else {
                    let mut new_start = SimplePos::default();
                    nvim_textobj_get_cursor_pos(&raw mut new_start.lnum, &raw mut new_start.col);
                    nvim_textobj_set_oap_start(oap, new_start.lnum, new_start.col);
                }
            }
        }
        nvim_textobj_set_cursor_pos(saved_pos.lnum, saved_pos.col);
    }
}

/// Finalize current_word operation (handle Visual mode and inclusive flag).
///
/// # Safety
/// Calls C accessor functions which must be valid.
unsafe fn current_word_finalize(oap: OapHandle, state: &CurrentWordState) {
    if nvim_textobj_get_VIsual_active() {
        if nvim_textobj_get_p_sel_first() == i32::from(b'e')
            && state.inclusive
            && nvim_textobj_ltoreq_VIsual_cursor()
        {
            inc_cursor();
        }
        if nvim_textobj_get_VIsual_mode() == i32::from(b'V') {
            nvim_textobj_set_VIsual_mode(i32::from(b'v'));
            nvim_textobj_set_redraw_cmdline(true);
        }
    } else {
        nvim_textobj_set_oap_inclusive(oap, state.inclusive);
    }
}

// =============================================================================
// Paragraph Functions
// =============================================================================

/// Motion type: line-wise (kMTLineWise = 1)
const MT_LINE_WISE: c_int = 1;

extern "C" {
    fn nvim_textobj_get_p_sections() -> *const std::ffi::c_char;
    fn nvim_textobj_get_p_para() -> *const std::ffi::c_char;
    fn ml_get(lnum: c_int) -> *const std::ffi::c_char;
    fn ml_get_len(lnum: c_int) -> c_int;
    fn linewhite(lnum: c_int) -> bool;
    fn setpcmark();
    fn showmode() -> c_int;
}

/// Check if the string 's' is a nroff macro that is in option 'opt'.
///
/// # Safety
/// - `opt` must be a valid pointer to a NUL-terminated string.
/// - `s` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_inmacro(
    opt: *const std::ffi::c_char,
    s: *const std::ffi::c_char,
) -> bool {
    inmacro_impl(opt, s)
}

/// Space character as c_char.
#[allow(clippy::cast_possible_wrap)]
const SPACE_CHAR: std::ffi::c_char = b' ' as std::ffi::c_char;

/// Implementation of inmacro.
#[allow(clippy::cast_possible_wrap)]
unsafe fn inmacro_impl(opt: *const std::ffi::c_char, s: *const std::ffi::c_char) -> bool {
    let mut macro_ptr = opt;

    while *macro_ptr != 0 {
        // Accept two characters in the option being equal to two characters
        // in the line. A space in the option matches with a space in the
        // line or the line having ended.
        let m0 = *macro_ptr;
        let m1 = *macro_ptr.offset(1);
        let s0 = *s;
        let s1 = *s.offset(1);

        let match0 = m0 == s0 || (m0 == SPACE_CHAR && (s0 == 0 || s0 == SPACE_CHAR));
        let match1 =
            m1 == s1 || ((m1 == 0 || m1 == SPACE_CHAR) && (s0 == 0 || s1 == 0 || s1 == SPACE_CHAR));

        if match0 && match1 {
            break;
        }

        macro_ptr = macro_ptr.offset(1);
        if *macro_ptr == 0 {
            break;
        }
        macro_ptr = macro_ptr.offset(1);
    }

    *macro_ptr != 0
}

/// Check if line 'lnum' is the start of a section or paragraph.
///
/// If 'para' is '{' or '}' only check for sections.
/// If 'both' is true also stop at '}'.
///
/// # Safety
/// Calls C accessor functions which must be valid.
#[export_name = "startPS"]
#[allow(non_snake_case)]
pub unsafe extern "C" fn rs_startPS(lnum: c_int, para: c_int, both: bool) -> bool {
    startps_impl(lnum, para, both)
}

/// Implementation of startPS.
unsafe fn startps_impl(lnum: c_int, para: c_int, both: bool) -> bool {
    let s = ml_get(lnum);
    let first_char = char_to_int(*s);

    // Check for paragraph/section start character
    if first_char == para
        || first_char == i32::from(b'\x0c')
        || (both && first_char == i32::from(b'}'))
    {
        return true;
    }

    // Check for nroff macro
    if first_char == i32::from(b'.') {
        let s1 = s.offset(1);
        let p_sections = nvim_textobj_get_p_sections();
        let p_para = nvim_textobj_get_p_para();

        if inmacro_impl(p_sections, s1) || (para == 0 && inmacro_impl(p_para, s1)) {
            return true;
        }
    }

    false
}

/// Find paragraph boundary.
///
/// # Safety
/// - `pincl` must be a valid pointer.
/// - Calls C accessor functions which must be valid.
#[export_name = "findpar"]
pub unsafe extern "C" fn rs_findpar(
    pincl: *mut bool,
    dir: c_int,
    count: c_int,
    what: c_int,
    both: bool,
) -> bool {
    findpar_impl(pincl, dir, count, what, both)
}

/// Implementation of findpar.
#[allow(clippy::too_many_lines)]
unsafe fn findpar_impl(
    pincl: *mut bool,
    dir: c_int,
    mut count: c_int,
    what: c_int,
    both: bool,
) -> bool {
    let mut curr = nvim_textobj_get_cursor_lnum();
    let ml_line_count = nvim_textobj_get_ml_line_count();

    while count > 0 {
        count -= 1;
        let mut did_skip = false;
        let mut first = true;

        loop {
            let line = ml_get(curr);
            if *line != 0 {
                did_skip = true;
            }

            // skip folded lines
            let mut fold_skipped = false;
            if first {
                let mut fold_first: c_int = 0;
                let mut fold_last: c_int = 0;
                if nvim_textobj_hasFolding(curr, &raw mut fold_first, &raw mut fold_last) {
                    curr = if dir > 0 { fold_last } else { fold_first } + dir;
                    fold_skipped = true;
                }
            }

            if !first && did_skip && startps_impl(curr, what, both) {
                break;
            }

            if fold_skipped {
                curr -= dir;
            }

            curr += dir;
            if curr < 1 || curr > ml_line_count {
                if count > 0 {
                    return false;
                }
                curr -= dir;
                break;
            }

            first = false;
        }
    }

    setpcmark();

    // include line with '}'
    if both {
        let line = ml_get(curr);
        if char_to_int(*line) == i32::from(b'}') {
            curr += 1;
        }
    }

    nvim_textobj_set_cursor_lnum(curr);

    if curr == ml_line_count && what != i32::from(b'}') && dir == FORWARD {
        let line = ml_get(curr);
        let line_len = ml_get_len(curr);

        // Put the cursor on the last character in the last line and make the
        // motion inclusive.
        if line_len != 0 {
            let mut col = line_len - 1;
            col -= utf_head_off(line, line.offset(col as isize));
            nvim_textobj_set_cursor_col(col);
            *pincl = true;
        } else {
            nvim_textobj_set_cursor_col(0);
        }
    } else {
        nvim_textobj_set_cursor_col(0);
    }

    true
}

/// Find paragraph under cursor.
///
/// # Safety
/// - `oap` must be a valid pointer.
/// - Calls C accessor functions which must be valid.
#[export_name = "current_par"]
pub unsafe extern "C" fn rs_current_par(
    oap: OapHandle,
    count: c_int,
    include: bool,
    par_type: c_int,
) -> c_int {
    current_par_impl(oap, count, include, par_type)
}

/// Implementation of current_par.
#[allow(clippy::too_many_lines)]
unsafe fn current_par_impl(oap: OapHandle, count: c_int, include: bool, par_type: c_int) -> c_int {
    // 'S' for sections not implemented yet
    if par_type == i32::from(b'S') {
        return FAIL;
    }

    let mut start_lnum = nvim_textobj_get_cursor_lnum();
    let ml_line_count = nvim_textobj_get_ml_line_count();

    // When visual area is more than one line: extend it.
    if nvim_textobj_get_VIsual_active() && start_lnum != nvim_textobj_get_VIsual_lnum() {
        return extend_paragraph(oap, count, include, start_lnum, ml_line_count);
    }

    // First move back to the start_lnum of the paragraph or white lines
    let white_in_front = linewhite(start_lnum);
    while start_lnum > 1 {
        if white_in_front {
            if !linewhite(start_lnum - 1) {
                break;
            }
        } else if linewhite(start_lnum - 1) || startps_impl(start_lnum, 0, false) {
            break;
        }
        start_lnum -= 1;
    }

    // Move past the end of any white lines.
    let mut end_lnum = start_lnum;
    while end_lnum <= ml_line_count && linewhite(end_lnum) {
        end_lnum += 1;
    }
    end_lnum -= 1;

    let mut i = count;
    let mut do_white = false;

    if !include && white_in_front {
        i -= 1;
    }

    while i > 0 {
        i -= 1;

        if end_lnum == ml_line_count {
            return FAIL;
        }

        if !include {
            do_white = linewhite(end_lnum + 1);
        }

        if include || !do_white {
            end_lnum += 1;
            // skip to end of paragraph
            while end_lnum < ml_line_count
                && !linewhite(end_lnum + 1)
                && !startps_impl(end_lnum + 1, 0, false)
            {
                end_lnum += 1;
            }
        }

        if i == 0 && white_in_front && include {
            break;
        }

        // skip to end of white lines after paragraph
        if include || do_white {
            while end_lnum < ml_line_count && linewhite(end_lnum + 1) {
                end_lnum += 1;
            }
        }
    }

    // If there are no empty lines at the end, try to find some empty lines at
    // the start (unless that has been done already).
    if !white_in_front && !linewhite(end_lnum) && include {
        while start_lnum > 1 && linewhite(start_lnum - 1) {
            start_lnum -= 1;
        }
    }

    finalize_paragraph(oap, start_lnum, end_lnum);

    OK
}

/// Extend paragraph selection in Visual mode.
unsafe fn extend_paragraph(
    _oap: OapHandle,
    count: c_int,
    include: bool,
    mut start_lnum: c_int,
    ml_line_count: c_int,
) -> c_int {
    let mut retval = OK;
    let dir = if start_lnum < nvim_textobj_get_VIsual_lnum() {
        BACKWARD
    } else {
        FORWARD
    };

    let mut i = count;
    while i > 0 {
        i -= 1;

        if start_lnum == if dir == BACKWARD { 1 } else { ml_line_count } {
            retval = FAIL;
            break;
        }

        let mut prev_start_is_white: c_int = -1;
        for _t in 0..2 {
            start_lnum += dir;
            let start_is_white = c_int::from(linewhite(start_lnum));

            if prev_start_is_white == start_is_white {
                start_lnum -= dir;
                break;
            }

            loop {
                if start_lnum == if dir == BACKWARD { 1 } else { ml_line_count } {
                    break;
                }

                let next_is_white = linewhite(start_lnum + dir);
                let at_start = if dir > 0 {
                    startps_impl(start_lnum + 1, 0, false)
                } else {
                    startps_impl(start_lnum, 0, false)
                };

                if (start_is_white != 0) != next_is_white || (start_is_white == 0 && at_start) {
                    break;
                }
                start_lnum += dir;
            }

            if !include {
                break;
            }

            if start_lnum == if dir == BACKWARD { 1 } else { ml_line_count } {
                break;
            }

            prev_start_is_white = start_is_white;
        }
    }

    nvim_textobj_set_cursor_lnum(start_lnum);
    nvim_textobj_set_cursor_col(0);
    retval
}

/// Finalize paragraph selection (set Visual/oap state).
unsafe fn finalize_paragraph(oap: OapHandle, start_lnum: c_int, end_lnum: c_int) {
    if nvim_textobj_get_VIsual_active() {
        // Problem: when doing "Vipipip" nothing happens in a single white
        // line, we get stuck there. Handle via extend_paragraph recursion.
        // For now, just set the Visual area.
        if nvim_textobj_get_VIsual_lnum() != start_lnum {
            nvim_textobj_set_VIsual(start_lnum, 0);
        }
        nvim_textobj_set_VIsual_mode(i32::from(b'V'));
        redraw_curbuf_later(UPD_INVERTED);
        showmode();
    } else {
        nvim_textobj_set_oap_start(oap, start_lnum, 0);
        nvim_textobj_set_oap_motion_type(oap, MT_LINE_WISE);
    }

    nvim_textobj_set_cursor_lnum(end_lnum);
    nvim_textobj_set_cursor_col(0);
}

// =============================================================================
// Sentence Functions
// =============================================================================

/// CPO_ENDOFSENT character ('J') - need 2 spaces after sentence ending
const CPO_ENDOFSENT: c_int = i32::from_be_bytes([0, 0, 0, b'J']);

extern "C" {
    fn gchar_pos(pos: PosHandle) -> c_int;
    fn incl(pos: PosHandle) -> c_int;
    fn decl(pos: PosHandle) -> c_int;
    fn inc(pos: PosHandle) -> c_int;

    /// Check if two positions are equal.
    fn nvim_textobj_equalpos(a: PosHandle, b: PosHandle) -> bool;

    /// Check if position a is less than position b.
    fn nvim_textobj_lt_pos(a: PosHandle, b: PosHandle) -> bool;

    /// Get p_cpo option string.
    fn nvim_textobj_get_p_cpo() -> *const std::ffi::c_char;

    /// Get cursor as pos_T*.
    fn nvim_textobj_get_cursor_ptr() -> PosHandle;

    /// Get VIsual as pos_T*.
    fn nvim_textobj_get_VIsual_ptr() -> PosHandle;

    /// Copy position from src to dst.
    fn nvim_textobj_copy_pos(dst: PosHandle, src: PosHandle);

    /// Get position lnum.
    fn nvim_textobj_pos_get_lnum(pos: PosHandle) -> c_int;

    /// Get position col.
    fn nvim_textobj_pos_get_col(pos: PosHandle) -> c_int;

    /// Set position lnum.
    fn nvim_textobj_pos_set_lnum(pos: PosHandle, lnum: c_int);

    /// Set position col.
    fn nvim_textobj_pos_set_col(pos: PosHandle, col: c_int);

    /// Check if line is empty (LINEEMPTY macro).
    fn nvim_textobj_lineempty(lnum: c_int) -> bool;

    /// Check if character is ASCII whitespace.
    fn nvim_textobj_ascii_iswhite(c: c_int) -> bool;

    /// Allocate a temporary pos_T on C side.
    fn nvim_textobj_alloc_pos() -> PosHandle;

    /// Free a temporary pos_T.
    fn nvim_textobj_free_pos(pos: PosHandle);

    /// Set cursor from position.
    fn nvim_textobj_set_cursor_from_pos(pos: PosHandle);
}

/// Find sentence boundary.
///
/// # Safety
/// Calls C accessor functions which must be valid.
#[export_name = "findsent"]
pub unsafe extern "C" fn rs_findsent(dir: c_int, count: c_int) -> c_int {
    findsent_impl(dir, count)
}

/// Check if character is a sentence-ending character.
#[inline]
fn is_sentence_end(c: c_int) -> bool {
    c == i32::from(b'.') || c == i32::from(b'!') || c == i32::from(b'?')
}

/// Check if character is a trailing sentence character.
#[inline]
fn is_sentence_trail(c: c_int) -> bool {
    c == i32::from(b')') || c == i32::from(b']') || c == i32::from(b'"') || c == i32::from(b'\'')
}

/// Implementation of findsent.
#[allow(clippy::too_many_lines)]
unsafe fn findsent_impl(dir: c_int, mut count: c_int) -> c_int {
    // Allocate position on C side
    let pos = nvim_textobj_alloc_pos();
    let cursor = nvim_textobj_get_cursor_ptr();
    nvim_textobj_copy_pos(pos, cursor);

    let mut noskip = false;

    while count > 0 {
        count -= 1;

        // Save previous position
        let prev_pos = nvim_textobj_alloc_pos();
        nvim_textobj_copy_pos(prev_pos, pos);

        // if on an empty line, skip up to a non-empty line
        if gchar_pos(pos) == NUL {
            loop {
                let result = if dir == FORWARD { incl(pos) } else { decl(pos) };
                if result == -1 {
                    break;
                }
                if gchar_pos(pos) != NUL {
                    break;
                }
            }
            if dir == FORWARD {
                nvim_textobj_free_pos(prev_pos);
                // Skip whitespace after finding
                findsent_skip_white(pos, &noskip);
                if nvim_textobj_equalpos(prev_pos, pos)
                    && findsent_retry(pos, dir, &mut count) == FAIL
                {
                    nvim_textobj_free_pos(pos);
                    return FAIL;
                }
                continue;
            }
        } else if dir == FORWARD
            && nvim_textobj_pos_get_col(pos) == 0
            && startps_impl(nvim_textobj_pos_get_lnum(pos), NUL, false)
        {
            // on the start of a paragraph or section when searching forward
            let lnum = nvim_textobj_pos_get_lnum(pos);
            if lnum == nvim_textobj_get_ml_line_count() {
                nvim_textobj_free_pos(prev_pos);
                nvim_textobj_free_pos(pos);
                return FAIL;
            }
            nvim_textobj_pos_set_lnum(pos, lnum + 1);
            nvim_textobj_free_pos(prev_pos);
            findsent_skip_white(pos, &noskip);
            continue;
        } else if dir == BACKWARD {
            decl(pos);
        }

        // go back to the previous non-white non-punctuation character
        let mut found_dot = false;
        loop {
            let c = gchar_pos(pos);
            if !nvim_textobj_ascii_iswhite(c) && !is_sentence_end(c) && !is_sentence_trail(c) {
                break;
            }

            let tpos = nvim_textobj_alloc_pos();
            nvim_textobj_copy_pos(tpos, pos);

            let dec_result = decl(tpos);
            let is_lineempty =
                nvim_textobj_lineempty(nvim_textobj_pos_get_lnum(tpos)) && dir == FORWARD;

            if dec_result == -1 || is_lineempty {
                nvim_textobj_free_pos(tpos);
                break;
            }

            if found_dot {
                nvim_textobj_free_pos(tpos);
                break;
            }

            if is_sentence_end(c) {
                found_dot = true;
            }

            if is_sentence_trail(c) {
                let tc = gchar_pos(tpos);
                if !is_sentence_end(tc) && !is_sentence_trail(tc) {
                    nvim_textobj_free_pos(tpos);
                    break;
                }
            }

            nvim_textobj_free_pos(tpos);
            decl(pos);
        }

        // remember the line where the search started
        let startlnum = nvim_textobj_pos_get_lnum(pos);
        let cpo_j = !vim_strchr(nvim_textobj_get_p_cpo(), CPO_ENDOFSENT).is_null();

        // find end of sentence
        loop {
            let c = gchar_pos(pos);

            if c == NUL
                || (nvim_textobj_pos_get_col(pos) == 0
                    && startps_impl(nvim_textobj_pos_get_lnum(pos), NUL, false))
            {
                if dir == BACKWARD && nvim_textobj_pos_get_lnum(pos) != startlnum {
                    nvim_textobj_pos_set_lnum(pos, nvim_textobj_pos_get_lnum(pos) + 1);
                }
                break;
            }

            if is_sentence_end(c) {
                let tpos = nvim_textobj_alloc_pos();
                nvim_textobj_copy_pos(tpos, pos);

                // skip trailing characters
                loop {
                    let inc_result = inc(tpos);
                    if inc_result == -1 {
                        break;
                    }
                    let tc = gchar_pos(tpos);
                    if !is_sentence_trail(tc) {
                        break;
                    }
                }

                let tc = gchar_pos(tpos);
                let is_space = tc == i32::from(b' ') || tc == i32::from(b'\t');
                let is_end = tc == -1 || tc == NUL;

                // Check for sentence end condition
                let sentence_ended = is_end
                    || (!cpo_j && is_space)
                    || (cpo_j && tc == i32::from(b' ') && {
                        let inc_result = inc(tpos);
                        inc_result >= 0 && gchar_pos(tpos) == i32::from(b' ')
                    });

                if sentence_ended {
                    nvim_textobj_copy_pos(pos, tpos);
                    if gchar_pos(pos) == NUL {
                        inc(pos);
                    }
                    nvim_textobj_free_pos(tpos);
                    break;
                }

                nvim_textobj_free_pos(tpos);
            }

            let func_result = if dir == FORWARD { incl(pos) } else { decl(pos) };

            if func_result == -1 {
                if count > 0 {
                    nvim_textobj_free_pos(prev_pos);
                    nvim_textobj_free_pos(pos);
                    return FAIL;
                }
                noskip = true;
                break;
            }
        }

        // skip white space
        findsent_skip_white(pos, &noskip);

        if nvim_textobj_equalpos(prev_pos, pos) {
            // didn't actually move, advance one character and try again
            if findsent_retry(pos, dir, &mut count) == FAIL {
                nvim_textobj_free_pos(prev_pos);
                nvim_textobj_free_pos(pos);
                return FAIL;
            }
        }

        nvim_textobj_free_pos(prev_pos);
    }

    setpcmark();
    nvim_textobj_set_cursor_from_pos(pos);
    nvim_textobj_free_pos(pos);

    OK
}

/// Skip whitespace after finding sentence boundary.
unsafe fn findsent_skip_white(pos: PosHandle, noskip: &bool) {
    if *noskip {
        return;
    }
    loop {
        let c = gchar_pos(pos);
        if c != i32::from(b' ') && c != i32::from(b'\t') {
            break;
        }
        if incl(pos) == -1 {
            break;
        }
    }
}

/// Retry sentence search when stuck.
unsafe fn findsent_retry(pos: PosHandle, dir: c_int, count: &mut c_int) -> c_int {
    let func_result = if dir == FORWARD { incl(pos) } else { decl(pos) };

    if func_result == -1 {
        if *count > 0 {
            return FAIL;
        }
    } else {
        *count += 1;
    }
    OK
}

/// Find first blank backward from position.
///
/// # Safety
/// - `posp` must be a valid pos_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_find_first_blank(posp: PosHandle) {
    find_first_blank_impl(posp);
}

/// Implementation of find_first_blank.
unsafe fn find_first_blank_impl(posp: PosHandle) {
    while decl(posp) != -1 {
        let c = gchar_pos(posp);
        if !nvim_textobj_ascii_iswhite(c) {
            incl(posp);
            break;
        }
    }
}

/// Skip count/2 sentences and count/2 separating white spaces.
///
/// # Safety
/// Calls C accessor functions which must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_findsent_forward(count: c_int, at_start_sent: bool) {
    findsent_forward_impl(count, at_start_sent);
}

/// Implementation of findsent_forward.
unsafe fn findsent_forward_impl(mut count: c_int, mut at_start_sent: bool) {
    let cursor = nvim_textobj_get_cursor_ptr();

    while count > 0 {
        count -= 1;

        findsent_impl(FORWARD, 1);

        if at_start_sent {
            find_first_blank_impl(cursor);
        }

        if count == 0 || at_start_sent {
            decl(cursor);
        }

        at_start_sent = !at_start_sent;
    }
}

/// Select sentence text object (as/is).
///
/// # Safety
/// - `oap` must be a valid oparg_T pointer.
#[export_name = "current_sent"]
pub unsafe extern "C" fn rs_current_sent(oap: OapHandle, count: c_int, include: bool) -> c_int {
    current_sent_impl(oap, count, include)
}

/// Implementation of current_sent.
#[allow(clippy::too_many_lines)]
unsafe fn current_sent_impl(oap: OapHandle, count: c_int, include: bool) -> c_int {
    let cursor = nvim_textobj_get_cursor_ptr();

    // Save start position
    let start_pos = nvim_textobj_alloc_pos();
    nvim_textobj_copy_pos(start_pos, cursor);

    // Position for checking
    let pos = nvim_textobj_alloc_pos();
    nvim_textobj_copy_pos(pos, start_pos);

    // Find start of next sentence
    findsent_impl(FORWARD, 1);

    // When the Visual area is bigger than one character: extend it.
    let visual = nvim_textobj_get_VIsual_ptr();
    if nvim_textobj_get_VIsual_active() && !nvim_textobj_equalpos(start_pos, visual) {
        let result = current_sent_extend(count, include, start_pos, pos);
        nvim_textobj_free_pos(start_pos);
        nvim_textobj_free_pos(pos);
        return result;
    }

    // Check if cursor started on a blank
    while nvim_textobj_ascii_iswhite(gchar_pos(pos)) {
        incl(pos);
    }

    let start_blank;
    if nvim_textobj_equalpos(pos, cursor) {
        start_blank = true;
        find_first_blank_impl(start_pos); // go back to first blank
    } else {
        start_blank = false;
        findsent_impl(BACKWARD, 1);
        nvim_textobj_copy_pos(start_pos, cursor);
    }

    let ncount = if include {
        count * 2
    } else {
        let mut n = count;
        if start_blank {
            n -= 1;
        }
        n
    };

    if ncount > 0 {
        findsent_forward_impl(ncount, true);
    } else {
        decl(cursor);
    }

    if include {
        // If the blank in front of the sentence is included, exclude the
        // blanks at the end of the sentence, go back to the first blank.
        // If there are no trailing blanks, try to include leading blanks.
        if start_blank {
            find_first_blank_impl(cursor);
            let c = gchar_pos(cursor);
            if nvim_textobj_ascii_iswhite(c) {
                decl(cursor);
            }
        } else {
            let c = gchar_cursor();
            if !nvim_textobj_ascii_iswhite(c) {
                find_first_blank_impl(start_pos);
            }
        }
    }

    if nvim_textobj_get_VIsual_active() {
        // Avoid getting stuck with "is" on a single space before a sentence.
        if nvim_textobj_equalpos(start_pos, cursor) {
            let result = current_sent_extend(count, include, start_pos, pos);
            nvim_textobj_free_pos(start_pos);
            nvim_textobj_free_pos(pos);
            return result;
        }
        if nvim_textobj_get_p_sel_first() == i32::from(b'e') {
            inc_cursor();
        }
        let start_lnum = nvim_textobj_pos_get_lnum(start_pos);
        let start_col = nvim_textobj_pos_get_col(start_pos);
        nvim_textobj_set_VIsual(start_lnum, start_col);
        nvim_textobj_set_VIsual_mode(i32::from(b'v'));
        nvim_textobj_set_redraw_cmdline(true);
        redraw_curbuf_later(UPD_INVERTED);
    } else {
        // include a newline after the sentence, if there is one
        if incl(cursor) == -1 {
            nvim_textobj_set_oap_inclusive(oap, true);
        } else {
            nvim_textobj_set_oap_inclusive(oap, false);
        }
        let start_lnum = nvim_textobj_pos_get_lnum(start_pos);
        let start_col = nvim_textobj_pos_get_col(start_pos);
        nvim_textobj_set_oap_start(oap, start_lnum, start_col);
        nvim_textobj_set_oap_motion_type(oap, MT_CHAR_WISE);
    }

    nvim_textobj_free_pos(start_pos);
    nvim_textobj_free_pos(pos);
    OK
}

/// Extend sentence selection in Visual mode.
#[allow(clippy::too_many_lines)]
unsafe fn current_sent_extend(
    mut count: c_int,
    include: bool,
    start_pos: PosHandle,
    pos: PosHandle,
) -> c_int {
    let cursor = nvim_textobj_get_cursor_ptr();
    let visual = nvim_textobj_get_VIsual_ptr();

    if nvim_textobj_lt_pos(start_pos, visual) {
        // Cursor at start of Visual area - move backward
        decl(pos);
        let mut at_start_sent = true;

        while nvim_textobj_lt_pos(pos, cursor) {
            let c = gchar_pos(pos);
            if !nvim_textobj_ascii_iswhite(c) {
                at_start_sent = false;
                break;
            }
            incl(pos);
        }

        if !at_start_sent {
            findsent_impl(BACKWARD, 1);
            if nvim_textobj_equalpos(cursor, start_pos) {
                at_start_sent = true;
            } else {
                findsent_impl(FORWARD, 1);
            }
        }

        if include {
            count *= 2;
        }

        while count > 0 {
            count -= 1;
            if at_start_sent {
                find_first_blank_impl(cursor);
            }
            let c = gchar_cursor();
            if !at_start_sent || (!include && !nvim_textobj_ascii_iswhite(c)) {
                findsent_impl(BACKWARD, 1);
            }
            at_start_sent = !at_start_sent;
        }
    } else {
        // Cursor at end of Visual area - move forward
        incl(pos);
        let mut at_start_sent = true;

        if !nvim_textobj_equalpos(pos, cursor) {
            at_start_sent = false;
            while nvim_textobj_lt_pos(pos, cursor) {
                let c = gchar_pos(pos);
                if !nvim_textobj_ascii_iswhite(c) {
                    at_start_sent = true;
                    break;
                }
                incl(pos);
            }

            if at_start_sent {
                findsent_impl(BACKWARD, 1);
            } else {
                nvim_textobj_copy_pos(cursor, start_pos);
            }
        }

        if include {
            count *= 2;
        }
        findsent_forward_impl(count, at_start_sent);

        if nvim_textobj_get_p_sel_first() == i32::from(b'e') {
            inc_cursor();
        }
    }

    OK
}

// =============================================================================
// Block Text Objects
// =============================================================================

/// FM_FORWARD flag for findmatchlimit
const FM_FORWARD: c_int = 0x20;

// Note: CPO_MATCHBSL constant not needed - using C accessor instead.

extern "C" {
    fn findmatch(oap: OapHandle, what: c_int) -> PosHandle;
    fn findmatchlimit(oap: OapHandle, what: c_int, flags: c_int, maxtravel: i64) -> PosHandle;
    fn inindent(extra: c_int) -> bool;
    fn nvim_textobj_set_p_cpo_temp(val: *const std::ffi::c_char);
    fn nvim_textobj_restore_p_cpo();
    fn nvim_textobj_cpo_has_matchbsl() -> bool;
    fn nvim_textobj_ltoreq_pos(a: PosHandle, b: PosHandle) -> bool;
}

/// Find block under cursor.
///
/// # Safety
/// - `oap` must be a valid oparg_T pointer.
#[export_name = "current_block"]
pub unsafe extern "C" fn rs_current_block(
    oap: OapHandle,
    count: c_int,
    include: bool,
    what: c_int,
    other: c_int,
) -> c_int {
    current_block_impl(oap, count, include, what, other)
}

/// Block selection state.
struct BlockState {
    start_pos: PosHandle,
    old_pos: PosHandle,
    old_end: PosHandle,
    old_start: PosHandle,
    sol: bool,
}

impl BlockState {
    unsafe fn new() -> Self {
        let cursor = nvim_textobj_get_cursor_ptr();
        let old_pos = nvim_textobj_alloc_pos();
        let old_end = nvim_textobj_alloc_pos();
        let old_start = nvim_textobj_alloc_pos();
        let start_pos = nvim_textobj_alloc_pos();

        nvim_textobj_copy_pos(old_pos, cursor);
        nvim_textobj_copy_pos(old_end, cursor);
        nvim_textobj_copy_pos(old_start, cursor);

        Self {
            start_pos,
            old_pos,
            old_end,
            old_start,
            sol: false,
        }
    }

    unsafe fn free(self) {
        nvim_textobj_free_pos(self.start_pos);
        nvim_textobj_free_pos(self.old_pos);
        nvim_textobj_free_pos(self.old_end);
        nvim_textobj_free_pos(self.old_start);
    }

    unsafe fn restore_and_fail(self) -> c_int {
        let cursor = nvim_textobj_get_cursor_ptr();
        nvim_textobj_copy_pos(cursor, self.old_pos);
        self.free();
        FAIL
    }
}

/// Implementation of current_block.
#[allow(clippy::too_many_lines)]
unsafe fn current_block_impl(
    oap: OapHandle,
    mut count: c_int,
    include: bool,
    what: c_int,
    other: c_int,
) -> c_int {
    let cursor = nvim_textobj_get_cursor_ptr();
    let visual = nvim_textobj_get_VIsual_ptr();
    let mut state = BlockState::new();
    let mut pos: PosHandle;

    // If we start on '(', '{', ')', '}', etc., use the whole block inclusive.
    if !nvim_textobj_get_VIsual_active() || nvim_textobj_equalpos(visual, cursor) {
        setpcmark();
        if what == i32::from(b'{') {
            // ignore indent
            while inindent(1) {
                if inc_cursor() != 0 {
                    break;
                }
            }
        }
        if gchar_cursor() == what {
            // cursor on '(' or '{', move cursor just after it
            nvim_textobj_set_cursor_col(nvim_textobj_get_cursor_col() + 1);
        }
    } else if nvim_textobj_lt_pos(visual, cursor) {
        nvim_textobj_copy_pos(state.old_start, visual);
        nvim_textobj_copy_pos(cursor, visual); // cursor at low end of Visual
    } else {
        nvim_textobj_copy_pos(state.old_end, visual);
    }

    // Set temporary p_cpo
    let cpo_temp = if nvim_textobj_cpo_has_matchbsl() {
        c"%M".as_ptr()
    } else {
        c"%".as_ptr()
    };
    nvim_textobj_set_p_cpo_temp(cpo_temp);

    // Search backwards for unclosed '(', '{', etc.
    pos = findmatch(std::ptr::null_mut(), what);
    if pos.is_null() {
        while count > 0 {
            count -= 1;
            pos = findmatchlimit(std::ptr::null_mut(), what, FM_FORWARD, 0);
            if pos.is_null() {
                break;
            }
            nvim_textobj_copy_pos(cursor, pos);
            nvim_textobj_copy_pos(state.start_pos, pos);
        }
    } else {
        while count > 0 {
            count -= 1;
            pos = findmatch(std::ptr::null_mut(), what);
            if pos.is_null() {
                break;
            }
            nvim_textobj_copy_pos(cursor, pos);
            nvim_textobj_copy_pos(state.start_pos, pos);
        }
    }

    nvim_textobj_restore_p_cpo();

    // Search for matching ')', '}', etc.
    if pos.is_null() {
        return state.restore_and_fail();
    }

    let end_pos = findmatch(std::ptr::null_mut(), other);
    if end_pos.is_null() {
        return state.restore_and_fail();
    }
    nvim_textobj_copy_pos(cursor, end_pos);

    // Try to exclude the brackets when include is false
    if !include {
        let result = block_exclude_brackets(oap, &mut state, what, other);
        if result == FAIL {
            return state.restore_and_fail();
        }
    }

    block_finalize(oap, &state);
    state.free();
    OK
}

/// Exclude brackets from block selection.
#[allow(clippy::too_many_lines)]
unsafe fn block_exclude_brackets(
    _oap: OapHandle,
    state: &mut BlockState,
    what: c_int,
    other: c_int,
) -> c_int {
    let cursor = nvim_textobj_get_cursor_ptr();

    loop {
        incl(state.start_pos);
        state.sol = nvim_textobj_get_cursor_col() == 0;
        decl(cursor);

        while inindent(1) {
            state.sol = true;
            if decl(cursor) != 0 {
                break;
            }
        }

        // In Visual mode, when resulting area is empty, abort.
        let end_pos = nvim_textobj_alloc_pos();
        nvim_textobj_copy_pos(end_pos, cursor);
        incl(end_pos); // Get what would be end_pos

        if nvim_textobj_equalpos(state.start_pos, end_pos) && nvim_textobj_get_VIsual_active() {
            nvim_textobj_free_pos(end_pos);
            return FAIL;
        }
        nvim_textobj_free_pos(end_pos);

        // In Visual mode, when the resulting area is not bigger than what we
        // started with, extend it to the next block, and then exclude again.
        if !nvim_textobj_lt_pos(state.start_pos, state.old_start)
            && !nvim_textobj_lt_pos(state.old_end, cursor)
            && !nvim_textobj_equalpos(state.start_pos, cursor)
            && nvim_textobj_get_VIsual_active()
        {
            nvim_textobj_copy_pos(cursor, state.old_start);
            decl(cursor);

            let pos = findmatch(std::ptr::null_mut(), what);
            if pos.is_null() {
                return FAIL;
            }
            nvim_textobj_copy_pos(state.start_pos, pos);
            nvim_textobj_copy_pos(cursor, pos);

            let end_pos = findmatch(std::ptr::null_mut(), other);
            if end_pos.is_null() {
                return FAIL;
            }
            nvim_textobj_copy_pos(cursor, end_pos);
        } else {
            break;
        }
    }

    OK
}

/// Finalize block selection (set Visual/oap state).
unsafe fn block_finalize(oap: OapHandle, state: &BlockState) {
    let cursor = nvim_textobj_get_cursor_ptr();

    if nvim_textobj_get_VIsual_active() {
        if nvim_textobj_get_p_sel_first() == i32::from(b'e') {
            inc(cursor);
        }
        if state.sol && gchar_cursor() != NUL {
            inc(cursor); // include the line break
        }
        let start_lnum = nvim_textobj_pos_get_lnum(state.start_pos);
        let start_col = nvim_textobj_pos_get_col(state.start_pos);
        nvim_textobj_set_VIsual(start_lnum, start_col);
        nvim_textobj_set_VIsual_mode(i32::from(b'v'));
        redraw_curbuf_later(UPD_INVERTED);
        showmode();
    } else {
        let start_lnum = nvim_textobj_pos_get_lnum(state.start_pos);
        let start_col = nvim_textobj_pos_get_col(state.start_pos);
        nvim_textobj_set_oap_start(oap, start_lnum, start_col);
        nvim_textobj_set_oap_motion_type(oap, MT_CHAR_WISE);
        nvim_textobj_set_oap_inclusive(oap, false);

        if state.sol {
            incl(cursor);
        } else if nvim_textobj_ltoreq_pos(state.start_pos, cursor) {
            // Include the character under the cursor.
            nvim_textobj_set_oap_inclusive(oap, true);
        } else {
            // End is before the start (no text in between)
            nvim_textobj_copy_pos(cursor, state.start_pos);
        }
    }
}

// =============================================================================
// Tag Text Objects
// =============================================================================

extern "C" {
    fn nvim_textobj_mb_ptr_back(base: *const std::ffi::c_char, p: *mut *mut std::ffi::c_char);
    fn nvim_textobj_mb_ptr_adv(p: *mut *mut std::ffi::c_char);
    fn ml_get_pos(pos: PosHandle) -> *const std::ffi::c_char;
}

/// Check if cursor is on an HTML tag.
///
/// # Arguments
/// * `end_tag` - When true, return true if cursor is on "</aaa>".
///
/// # Returns
/// True if the cursor is on a "<aaa>" tag. Ignores "<aaa/>".
///
/// # Safety
/// Calls C accessor functions which must be valid.
#[export_name = "in_html_tag"]
pub unsafe extern "C" fn rs_in_html_tag(end_tag: bool) -> bool {
    in_html_tag_impl(end_tag)
}

/// Implementation of in_html_tag.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
unsafe fn in_html_tag_impl(end_tag: bool) -> bool {
    let line = get_cursor_line_ptr();
    let cursor_col = nvim_textobj_get_cursor_col();

    // Find '<' under or before cursor
    let mut p = line.offset(cursor_col as isize).cast_mut();

    // Mimic the C for loop: check '<' first, then MB_PTR_BACK, then '>'
    loop {
        // Check if we found '<'
        if *p == b'<' as std::ffi::c_char {
            break;
        }
        // Check loop condition
        if p <= line.cast_mut() {
            break;
        }
        // Move backward
        nvim_textobj_mb_ptr_back(line, &raw mut p);
        // Check if we hit '>' (meaning we went past a tag end)
        if *p == b'>' as std::ffi::c_char {
            break;
        }
    }

    if *p != b'<' as std::ffi::c_char {
        return false;
    }

    // Create position for iteration
    let pos = nvim_textobj_alloc_pos();
    nvim_textobj_pos_set_lnum(pos, nvim_textobj_get_cursor_lnum());
    // Cast is safe: column offsets fit in c_int on all platforms
    let col = (p as isize - line as isize) as c_int;
    nvim_textobj_pos_set_col(pos, col);

    // Move past '<'
    nvim_textobj_mb_ptr_adv(&raw mut p);

    if end_tag {
        // Check that there is a '/' after the '<'
        let result = *p == b'/' as std::ffi::c_char;
        nvim_textobj_free_pos(pos);
        return result;
    }

    // Check that there is no '/' after the '<'
    if *p == b'/' as std::ffi::c_char {
        nvim_textobj_free_pos(pos);
        return false;
    }

    // Check that the matching '>' is not preceded by '/'
    let mut lc: c_int = NUL;
    loop {
        if inc(pos) < 0 {
            nvim_textobj_free_pos(pos);
            return false;
        }
        let c_ptr = ml_get_pos(pos);
        let c = char_to_int(*c_ptr);
        if c == i32::from(b'>') {
            break;
        }
        lc = c;
    }

    nvim_textobj_free_pos(pos);
    lc != i32::from(b'/')
}

// =============================================================================
// Quote Text Object (current_quote)
// =============================================================================

extern "C" {
    /// Decrement a position. Returns -1 at start, 1 at line start, 0 otherwise.
    #[link_name = "dec"]
    fn c_dec(pos: PosHandle) -> c_int;

    /// Returns curbuf->b_p_qe (the 'quoteescape' option string).
    fn nvim_textobj_get_curbuf_qe() -> *const std::ffi::c_char;
}

/// Find quote text object under cursor.
///
/// Handles `i"`, `a"`, `i'`, `a'`, etc.
///
/// Replaces the C `current_quote` thin wrapper: fetches the escape string
/// (`b_p_qe`) internally via accessor.
///
/// # Safety
/// - `oap` must be a valid oparg_T pointer.
#[unsafe(export_name = "current_quote")]
pub unsafe extern "C" fn rs_current_quote(
    oap: OapHandle,
    count: c_int,
    include: bool,
    quotechar: c_int,
) -> bool {
    let escape = nvim_textobj_get_curbuf_qe();
    current_quote_impl(oap, count, include, quotechar, escape)
}

/// State for Visual mode adjustments needed in abort_search cleanup.
struct QuoteAbortState {
    did_exclusive_adj: bool,
    restore_vis_bef: bool,
}

/// Undo the exclusive-selection adjustment made at the start of current_quote.
///
/// # Safety
/// Calls C accessor functions which must be valid.
unsafe fn current_quote_abort(abort_state: &QuoteAbortState) {
    if nvim_textobj_get_VIsual_active() && nvim_textobj_get_p_sel_first() == i32::from(b'e') {
        if abort_state.did_exclusive_adj {
            inc_cursor();
        }
        if abort_state.restore_vis_bef {
            // Swap cursor and VIsual back
            let cur_lnum = nvim_textobj_get_cursor_lnum();
            let cur_col = nvim_textobj_get_cursor_col();
            let vis_lnum = nvim_textobj_get_VIsual_lnum();
            let vis_col = nvim_textobj_get_VIsual_col();
            nvim_textobj_set_cursor_pos(vis_lnum, vis_col);
            nvim_textobj_set_VIsual(cur_lnum, cur_col);
        }
    }
}

/// Implementation of current_quote.
#[allow(
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    unused_assignments
)]
unsafe fn current_quote_impl(
    oap: OapHandle,
    count: c_int,
    include: bool,
    quotechar: c_int,
    escape: *const std::ffi::c_char,
) -> bool {
    let line = get_cursor_line_ptr();
    let mut col_end: c_int = 0;
    let mut col_start = nvim_textobj_get_cursor_col();
    let mut inclusive = false;
    let mut vis_empty = true; // Visual selection <= 1 char
    let mut vis_bef_curs = false; // Visual starts before cursor
    let mut did_exclusive_adj = false; // adjusted pos for 'selection'
    let mut inside_quotes = false; // Looks like "i'" done before
    let mut selected_quote = false; // Has quote inside selection

    let mut abort_state = QuoteAbortState {
        did_exclusive_adj: false,
        restore_vis_bef: false,
    };

    // When 'selection' is "exclusive" move the cursor to where it would be
    // with 'selection' "inclusive", so that the logic is the same for both.
    // The cursor then is moved forward after adjusting the area.
    if nvim_textobj_get_VIsual_active() {
        // this only works within one line
        if nvim_textobj_get_VIsual_lnum() != nvim_textobj_get_cursor_lnum() {
            return false;
        }

        // vis_bef_curs means "VIsual starts before cursor", i.e., VIsual < cursor
        // nvim_textobj_lt_VIsual_cursor returns lt(VIsual, cursor)
        vis_bef_curs = nvim_textobj_lt_VIsual_cursor();
        vis_empty = nvim_textobj_equalpos_cursor_VIsual();
        if nvim_textobj_get_p_sel_first() == i32::from(b'e') {
            if vis_bef_curs {
                dec_cursor();
                did_exclusive_adj = true;
                abort_state.did_exclusive_adj = true;
            } else if !vis_empty {
                // dec(&VIsual)
                let vis_ptr = nvim_textobj_get_VIsual_ptr();
                c_dec(vis_ptr);
                did_exclusive_adj = true;
                abort_state.did_exclusive_adj = true;
            }
            vis_empty = nvim_textobj_equalpos_cursor_VIsual();
            if !vis_bef_curs && !vis_empty {
                // VIsual needs to be start of Visual selection.
                let cur_lnum = nvim_textobj_get_cursor_lnum();
                let cur_col = nvim_textobj_get_cursor_col();
                let vis_lnum = nvim_textobj_get_VIsual_lnum();
                let vis_col = nvim_textobj_get_VIsual_col();
                nvim_textobj_set_cursor_pos(vis_lnum, vis_col);
                nvim_textobj_set_VIsual(cur_lnum, cur_col);
                vis_bef_curs = true;
                abort_state.restore_vis_bef = true;
            }
        }
    }

    if !vis_empty {
        // Check if the existing selection exactly spans the text inside quotes.
        if vis_bef_curs {
            let vis_col = nvim_textobj_get_VIsual_col();
            inside_quotes = vis_col > 0
                && char_to_int(*line.offset((vis_col - 1) as isize)) == quotechar
                && char_to_int(*line.offset(col_start as isize)) != NUL
                && char_to_int(*line.offset((col_start + 1) as isize)) == quotechar;
            let i = vis_col;
            col_end = col_start;
            // Find out if we have a quote in the selection.
            let mut j = i;
            while j <= col_end {
                if char_to_int(*line.offset(j as isize)) == NUL {
                    break;
                }
                if char_to_int(*line.offset(j as isize)) == quotechar {
                    selected_quote = true;
                    break;
                }
                j += 1;
            }
        } else {
            inside_quotes = col_start > 0
                && char_to_int(*line.offset((col_start - 1) as isize)) == quotechar
                && char_to_int(*line.offset(nvim_textobj_get_VIsual_col() as isize)) != NUL
                && char_to_int(*line.offset((nvim_textobj_get_VIsual_col() + 1) as isize))
                    == quotechar;
            let i = col_start;
            col_end = nvim_textobj_get_VIsual_col();
            // Find out if we have a quote in the selection.
            let mut j = i;
            while j <= col_end {
                if char_to_int(*line.offset(j as isize)) == NUL {
                    break;
                }
                if char_to_int(*line.offset(j as isize)) == quotechar {
                    selected_quote = true;
                    break;
                }
                j += 1;
            }
        }
    }
    // col_end is set to 0 above; when vis_empty is true the col_end value is not used
    // before being assigned in subsequent branches.

    if !vis_empty && char_to_int(*line.offset(col_start as isize)) == quotechar {
        // Already selecting something and on a quote character. Find the
        // next quoted string.
        if vis_bef_curs {
            // Assume we are on a closing quote: move to after the next opening quote.
            col_start = rs_find_next_quote(line, col_start + 1, quotechar, std::ptr::null());
            if col_start < 0 {
                current_quote_abort(&abort_state);
                return false;
            }
            col_end = rs_find_next_quote(line, col_start + 1, quotechar, escape);
            if col_end < 0 {
                // We were on a starting quote perhaps?
                col_end = col_start;
                col_start = nvim_textobj_get_cursor_col();
            }
        } else {
            col_end = rs_find_prev_quote(line, col_start, quotechar, std::ptr::null());
            if char_to_int(*line.offset(col_end as isize)) != quotechar {
                current_quote_abort(&abort_state);
                return false;
            }
            col_start = rs_find_prev_quote(line, col_end, quotechar, escape);
            if char_to_int(*line.offset(col_start as isize)) != quotechar {
                // We were on an ending quote perhaps?
                col_start = col_end;
                col_end = nvim_textobj_get_cursor_col();
            }
        }
    } else if char_to_int(*line.offset(col_start as isize)) == quotechar || !vis_empty {
        let first_col = if vis_empty {
            col_start
        } else if vis_bef_curs {
            rs_find_next_quote(line, col_start, quotechar, std::ptr::null())
        } else {
            rs_find_prev_quote(line, col_start, quotechar, std::ptr::null())
        };
        // The cursor is on a quote, we don't know if it's the opening or
        // closing quote. Search from the start of the line to find out.
        col_start = 0;
        loop {
            // Find open quote character.
            col_start = rs_find_next_quote(line, col_start, quotechar, std::ptr::null());
            if col_start < 0 || col_start > first_col {
                current_quote_abort(&abort_state);
                return false;
            }
            // Find close quote character.
            col_end = rs_find_next_quote(line, col_start + 1, quotechar, escape);
            if col_end < 0 {
                current_quote_abort(&abort_state);
                return false;
            }
            // If cursor is between start and end quote, it is the target text object.
            if col_start <= first_col && first_col <= col_end {
                break;
            }
            col_start = col_end + 1;
        }
    } else {
        // Search backward for a starting quote.
        col_start = rs_find_prev_quote(line, col_start, quotechar, escape);
        if char_to_int(*line.offset(col_start as isize)) != quotechar {
            // No quote before the cursor, look after the cursor.
            col_start = rs_find_next_quote(line, col_start, quotechar, std::ptr::null());
            if col_start < 0 {
                current_quote_abort(&abort_state);
                return false;
            }
        }

        // Find close quote character.
        col_end = rs_find_next_quote(line, col_start + 1, quotechar, escape);
        if col_end < 0 {
            current_quote_abort(&abort_state);
            return false;
        }
    }

    // When "include" is true, include spaces after closing quote or before
    // the starting quote.
    if include {
        if nvim_textobj_ascii_iswhite(char_to_int(*line.offset((col_end + 1) as isize))) {
            while nvim_textobj_ascii_iswhite(char_to_int(*line.offset((col_end + 1) as isize))) {
                col_end += 1;
            }
        } else {
            while col_start > 0
                && nvim_textobj_ascii_iswhite(char_to_int(*line.offset((col_start - 1) as isize)))
            {
                col_start -= 1;
            }
        }
    }

    // Set start position. After vi" another i" must include the ".
    // For v2i" include the quotes.
    if !include && count < 2 && (vis_empty || !inside_quotes) {
        col_start += 1;
    }
    nvim_textobj_set_cursor_col(col_start);

    if nvim_textobj_get_VIsual_active() {
        // Set the start of the Visual area when the Visual area was empty, we
        // were just inside quotes or the Visual area didn't start at a quote
        // and didn't include a quote.
        let vis_col = nvim_textobj_get_VIsual_col();
        if vis_empty
            || (vis_bef_curs
                && !selected_quote
                && (inside_quotes
                    || (char_to_int(*line.offset(vis_col as isize)) != quotechar
                        && (vis_col == 0
                            || char_to_int(*line.offset((vis_col - 1) as isize)) != quotechar))))
        {
            nvim_textobj_set_VIsual_from_cursor();
            redraw_curbuf_later(UPD_INVERTED);
        }
    } else {
        nvim_textobj_set_oap_start_from_cursor(oap);
        nvim_textobj_set_oap_motion_type(oap, MT_CHAR_WISE);
    }

    // Set end position.
    nvim_textobj_set_cursor_col(col_end);
    if (include
        || count > 1
        // After vi" another i" must include the ".
        || (!vis_empty && inside_quotes))
        && inc_cursor() == 2
    {
        inclusive = true;
    }

    if nvim_textobj_get_VIsual_active() {
        if vis_empty || vis_bef_curs {
            // decrement cursor when 'selection' is not exclusive
            if nvim_textobj_get_p_sel_first() != i32::from(b'e') {
                dec_cursor();
            }
        } else {
            // Cursor is at start of Visual area.  Set the end of the Visual
            // area when it was just inside quotes or it didn't end at a quote.
            let vis_col = nvim_textobj_get_VIsual_col();
            if inside_quotes
                || (!selected_quote
                    && char_to_int(*line.offset(vis_col as isize)) != quotechar
                    && (char_to_int(*line.offset(vis_col as isize)) == NUL
                        || char_to_int(*line.offset((vis_col + 1) as isize)) != quotechar))
            {
                dec_cursor();
                nvim_textobj_set_VIsual_from_cursor();
            }
            nvim_textobj_set_cursor_col(col_start);
        }
        if nvim_textobj_get_VIsual_mode() == i32::from(b'V') {
            nvim_textobj_set_VIsual_mode(i32::from(b'v'));
            nvim_textobj_set_redraw_cmdline(true);
        }
    } else {
        // Set inclusive and other oap's flags.
        nvim_textobj_set_oap_inclusive(oap, inclusive);
    }

    // All done - the abort_state is not needed
    let _ = did_exclusive_adj; // used in abort path only
    true
}

// Additional extern declarations for current_quote
extern "C" {
    /// Set oap->start from cursor position.
    fn nvim_textobj_set_oap_start_from_cursor(oap: OapHandle);
}

// =============================================================================
// Tag Block Text Object (current_tagblock)
// =============================================================================

extern "C" {
    /// Get p_ws (wrapscan option).
    fn nvim_textobj_get_p_ws() -> bool;

    /// Set p_ws (wrapscan option).
    fn nvim_textobj_set_p_ws(val: bool);

    /// Returns the byte at the current cursor position as int.
    fn nvim_textobj_get_cursor_char() -> c_int;

    /// Returns a pointer to the current cursor char (for reading chars).
    fn nvim_textobj_get_cursor_pos_ptr() -> *const std::ffi::c_char;

    /// Wrapper for do_searchpair with NULL skip/match_pos.
    fn nvim_textobj_do_searchpair(
        spat: *const std::ffi::c_char,
        mpat: *const std::ffi::c_char,
        epat: *const std::ffi::c_char,
        dir: c_int,
    ) -> c_int;
}

/// Find tag block under the cursor, cursor at end.
///
/// Handles `at`, `it`, `dat`, `dit`, `cat`, `cit` text objects.
///
/// # Safety
/// - `oap` must be a valid oparg_T pointer.
#[unsafe(export_name = "current_tagblock")]
pub unsafe extern "C" fn rs_current_tagblock(
    oap: OapHandle,
    count_arg: c_int,
    include: bool,
) -> bool {
    current_tagblock_impl(oap, count_arg, include)
}

/// Returns true if two SimplePos positions are equal.
#[inline]
fn simplepos_eq(a: SimplePos, b: SimplePos) -> bool {
    a.lnum == b.lnum && a.col == b.col
}

/// Returns true if a < b.
#[inline]
fn simplepos_lt(a: SimplePos, b: SimplePos) -> bool {
    a.lnum < b.lnum || (a.lnum == b.lnum && a.col < b.col)
}

/// Get current cursor position as SimplePos.
#[inline]
unsafe fn cursor_pos() -> SimplePos {
    let mut lnum = 0;
    let mut col = 0;
    nvim_textobj_get_cursor_pos(&raw mut lnum, &raw mut col);
    SimplePos { lnum, col }
}

/// Set cursor to a SimplePos.
#[inline]
unsafe fn set_cursor(pos: SimplePos) {
    nvim_textobj_set_cursor_pos(pos.lnum, pos.col);
}

/// Get VIsual as SimplePos.
#[inline]
unsafe fn visual_pos() -> SimplePos {
    SimplePos {
        lnum: nvim_textobj_get_VIsual_lnum(),
        col: nvim_textobj_get_VIsual_col(),
    }
}

/// Decrement a PosHandle position using the `decl` C function.
/// Returns the decrement result (same as decl: -1, 0, or 1).
#[inline]
unsafe fn decl_simplepos(pos: SimplePos) -> (SimplePos, c_int) {
    let ph = nvim_textobj_alloc_pos();
    nvim_textobj_pos_set_lnum(ph, pos.lnum);
    nvim_textobj_pos_set_col(ph, pos.col);
    let r = c_dec(ph);
    let out = SimplePos {
        lnum: nvim_textobj_pos_get_lnum(ph),
        col: nvim_textobj_pos_get_col(ph),
    };
    nvim_textobj_free_pos(ph);
    (out, r)
}

/// Implementation of current_tagblock.
///
/// # Safety
/// Calls multiple unsafe C accessor functions.
unsafe fn current_tagblock_impl(oap: OapHandle, count_arg: c_int, include: bool) -> bool {
    let mut count = count_arg;
    let mut do_include = include;
    let save_p_ws = nvim_textobj_get_p_ws();
    let mut retval = false;
    let mut is_inclusive = true;

    nvim_textobj_set_p_ws(false);

    let old_pos = cursor_pos();
    let (old_start, old_end) = tagblock_init_bounds(old_pos);

    let empty_pat = c"";
    let open_pat = c"<[^ \t>/!]\\+\\%(\\_s\\_[^>]\\{-}[^/]>\\|$\\|\\_s\\=>\\)";
    let close_pat = c"</[^>]*>";

    // 'again' loop: retry when we can't find matching end tag
    'again: loop {
        // Search backwards for unclosed "<aaa>".  Put this position in start_pos.
        for _n in 0..count {
            if nvim_textobj_do_searchpair(
                open_pat.as_ptr(),
                empty_pat.as_ptr(),
                close_pat.as_ptr(),
                BACKWARD,
            ) <= 0
            {
                set_cursor(old_pos);
                break 'again; // goto theend
            }
        }
        let start_pos = cursor_pos();

        // Isolate tag name and build patterns for forward search.
        let Some((spat, epat)) = tagblock_build_patterns(old_pos) else {
            break 'again; // cursor restored inside helper
        };

        let r =
            nvim_textobj_do_searchpair(spat.as_ptr(), empty_pat.as_ptr(), epat.as_ptr(), FORWARD);

        if r < 1 || simplepos_lt(cursor_pos(), old_end) {
            count = 1;
            set_cursor(start_pos);
            continue 'again;
        }

        // Adjust cursor to include/exclude the end tag boundary.
        is_inclusive = tagblock_adjust_end(do_include, &mut is_inclusive);

        let end_pos = cursor_pos();

        // Determine actual start position (may retry).
        let final_start = if do_include {
            start_pos
        } else {
            let excl = tagblock_exclude_start(start_pos, end_pos, old_start, old_end);
            match excl {
                TagblockExclude::Retry => {
                    do_include = true;
                    set_cursor(old_start);
                    count = count_arg;
                    continue 'again;
                }
                TagblockExclude::Start(s) => s,
            }
        };

        tagblock_apply_selection(oap, final_start, end_pos, is_inclusive);
        retval = true;
        break 'again;
    }

    nvim_textobj_set_p_ws(save_p_ws);
    retval
}

/// Initialize old_start and old_end for current_tagblock.
///
/// Positions the cursor appropriately based on Visual mode state and returns
/// the (old_start, old_end) pair.
///
/// # Safety
/// Calls unsafe C accessor functions.
unsafe fn tagblock_init_bounds(old_pos: SimplePos) -> (SimplePos, SimplePos) {
    let mut old_end = old_pos;
    let mut old_start = old_end;

    if !nvim_textobj_get_VIsual_active() || nvim_textobj_get_p_sel_first() == i32::from(b'e') {
        let (new_end, _) = decl_simplepos(old_end);
        old_end = new_end;
    }

    if !nvim_textobj_get_VIsual_active() || nvim_textobj_equalpos_cursor_VIsual() {
        setpcmark();
        while inindent(1) {
            if inc_cursor() != 0 {
                break;
            }
        }
        if in_html_tag_impl(false) {
            while nvim_textobj_get_cursor_char() != i32::from(b'>') {
                if inc_cursor() < 0 {
                    break;
                }
            }
        } else if in_html_tag_impl(true) {
            while nvim_textobj_get_cursor_char() != i32::from(b'<') {
                if dec_cursor() < 0 {
                    break;
                }
            }
            dec_cursor();
            old_end = cursor_pos();
        }
    } else if nvim_textobj_lt_VIsual_cursor() {
        old_start = visual_pos();
        set_cursor(visual_pos());
    } else {
        old_end = visual_pos();
    }

    (old_start, old_end)
}

/// Build the forward search patterns for a tag at current cursor (after inc_cursor).
///
/// Advances the cursor one position, isolates the tag name, and builds
/// CString patterns for `do_searchpair`. Returns None on failure (cursor
/// is restored to `fail_pos`).
///
/// # Safety
/// Calls unsafe C accessor functions.
unsafe fn tagblock_build_patterns(
    fail_pos: SimplePos,
) -> Option<(std::ffi::CString, std::ffi::CString)> {
    use std::ffi::CString;

    inc_cursor();
    let p_ptr = nvim_textobj_get_cursor_pos_ptr();
    let mut scan = p_ptr;
    loop {
        let ch = (*scan).cast_unsigned();
        if ch == 0 || ch == b'>' || ch == b' ' || ch == b'\t' {
            break;
        }
        scan = scan.add(1);
    }
    let len = scan.offset_from(p_ptr).unsigned_abs();
    if len == 0 {
        set_cursor(fail_pos);
        return None;
    }

    let tag_bytes = std::slice::from_raw_parts(p_ptr.cast::<u8>(), len);
    let tag_name = String::from_utf8_lossy(tag_bytes);
    let spat_str = format!("<{tag_name}\\>\\%(\\_.s\\_[^>]\\{{-}}\\_[^/]>\\|\\_s\\?>\\)\\c");
    let epat_str = format!("</{tag_name}>\\c");

    let Ok(spat) = CString::new(spat_str) else {
        set_cursor(fail_pos);
        return None;
    };
    let Ok(epat) = CString::new(epat_str) else {
        set_cursor(fail_pos);
        return None;
    };
    Some((spat, epat))
}

/// Adjust cursor to mark the end boundary of a tag block.
///
/// For include mode: advance to '>'.
/// For exclude mode: optionally back off from '<', updating is_inclusive.
///
/// Returns the updated `is_inclusive` value.
///
/// # Safety
/// Calls unsafe C accessor functions.
unsafe fn tagblock_adjust_end(do_include: bool, is_inclusive: &mut bool) -> bool {
    if do_include {
        while nvim_textobj_get_cursor_char() != i32::from(b'>') {
            if inc_cursor() < 0 {
                break;
            }
        }
    } else {
        let c = nvim_textobj_get_cursor_char();
        if c == i32::from(b'<') && !nvim_textobj_get_VIsual_active() && cursor_pos().col == 0 {
            *is_inclusive = false;
        } else if c == i32::from(b'<') {
            dec_cursor();
        }
    }
    *is_inclusive
}

/// Result of `tagblock_exclude_start`.
enum TagblockExclude {
    /// Retry with do_include=true.
    Retry,
    /// Use this start position.
    Start(SimplePos),
}

/// Exclude the start tag from the selection, skipping '>' inside quotes.
///
/// Returns `Retry` if Visual mode and selection unchanged (should switch to include).
/// Returns `Start(pos)` with the new start position (after '>' of start tag).
///
/// # Safety
/// Calls unsafe C accessor functions.
unsafe fn tagblock_exclude_start(
    start_pos: SimplePos,
    end_pos: SimplePos,
    old_start: SimplePos,
    old_end: SimplePos,
) -> TagblockExclude {
    let mut in_quotes = false;
    set_cursor(start_pos);
    let mut new_start = start_pos;
    while inc_cursor() >= 0 {
        let ch = nvim_textobj_get_cursor_char();
        if ch == i32::from(b'>') && !in_quotes {
            inc_cursor();
            new_start = cursor_pos();
            break;
        } else if ch == i32::from(b'"') || ch == i32::from(b'\'') {
            in_quotes = !in_quotes;
        }
    }
    set_cursor(end_pos);

    if nvim_textobj_get_VIsual_active()
        && simplepos_eq(new_start, old_start)
        && simplepos_eq(end_pos, old_end)
    {
        return TagblockExclude::Retry;
    }
    TagblockExclude::Start(new_start)
}

/// Apply the final Visual/operator selection for a tag block.
///
/// # Safety
/// Calls unsafe C accessor functions.
unsafe fn tagblock_apply_selection(
    oap: OapHandle,
    start_pos: SimplePos,
    end_pos: SimplePos,
    is_inclusive: bool,
) {
    if nvim_textobj_get_VIsual_active() {
        // If end is before start, no text between tags: select char under cursor.
        if simplepos_lt(end_pos, start_pos) {
            set_cursor(start_pos);
        } else if nvim_textobj_get_p_sel_first() == i32::from(b'e') {
            inc_cursor();
        }
        nvim_textobj_set_VIsual(start_pos.lnum, start_pos.col);
        nvim_textobj_set_VIsual_mode(i32::from(b'v'));
        redraw_curbuf_later(UPD_INVERTED);
        showmode();
    } else {
        nvim_textobj_set_oap_start(oap, start_pos.lnum, start_pos.col);
        nvim_textobj_set_oap_motion_type(oap, MT_CHAR_WISE);
        if simplepos_lt(end_pos, start_pos) {
            // End before start: operate on empty area.
            set_cursor(start_pos);
            nvim_textobj_set_oap_inclusive(oap, false);
        } else {
            nvim_textobj_set_oap_inclusive(oap, is_inclusive);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(NUL, 0);
    }
}
