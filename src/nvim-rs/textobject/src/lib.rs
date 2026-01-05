//! Text object selection and navigation for Neovim
//!
//! This crate provides Rust implementations of text object functions
//! from `src/nvim/textobject.c`. It handles text object selection (aw, iw, as, is,
//! ap, ip, a", i", a{, i{, etc.) and word motions (w, W, b, B, e, E).

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

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
    /// Get character at cursor position via gchar_cursor().
    fn nvim_textobj_gchar_cursor() -> c_int;

    /// Increment cursor position. Returns -1 at EOF, 1 at EOL, 0 otherwise.
    fn nvim_textobj_inc_cursor() -> c_int;

    /// Decrement cursor position. Returns -1 at start, 1 at line start, 0 otherwise.
    fn nvim_textobj_dec_cursor() -> c_int;

    /// Get UTF character class (0=whitespace, 1=punct, 2+=word).
    fn nvim_textobj_utf_class(c: c_int) -> c_int;

    /// Get current cursor column.
    fn nvim_textobj_get_cursor_col() -> c_int;

    /// Get current cursor line number.
    fn nvim_textobj_get_cursor_lnum() -> c_int;

    /// Get total line count in current buffer.
    fn nvim_textobj_get_ml_line_count() -> c_int;

    /// Check if a line is empty (only has NUL).
    fn nvim_textobj_is_lineempty(lnum: c_int) -> bool;

    /// Get pointer to cursor line content.
    fn nvim_textobj_get_cursor_line_ptr() -> *const std::ffi::c_char;

    /// Set cursor coladd to 0.
    fn nvim_textobj_set_cursor_coladd_zero();

    /// Check for folding at line, get first/last lines of fold.
    /// Returns true if line is folded.
    fn nvim_textobj_hasFolding(lnum: c_int, first: *mut c_int, last: *mut c_int) -> bool;

    /// Move cursor to given column (MAXCOL for end of line).
    fn nvim_textobj_coladvance(col: c_int);

    /// Adjust skipcol after cursor movement.
    fn nvim_textobj_adjust_skipcol();
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
    let c = unsafe { nvim_textobj_gchar_cursor() };

    // Whitespace check: space, tab, or NUL
    if c == i32::from(b' ') || c == i32::from(b'\t') || c == NUL {
        return CLASS_WHITESPACE;
    }

    // SAFETY: Accessor function is provided by C side
    let class = unsafe { nvim_textobj_utf_class(c) };

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
                nvim_textobj_inc_cursor()
            } else {
                nvim_textobj_dec_cursor()
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

            nvim_textobj_dec_cursor();

            // Stop at start of word (different class)
            if cls_impl(bigword) != sclass {
                nvim_textobj_inc_cursor();
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
#[no_mangle]
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
                nvim_textobj_coladvance(MAXCOL);
            }

            let sclass = cls_impl(bigword); // starting class

            // We always move at least one character, unless on the last
            // character in the buffer.
            let last_line = nvim_textobj_get_cursor_lnum() == nvim_textobj_get_ml_line_count();
            let i = nvim_textobj_inc_cursor();
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
                    let i = nvim_textobj_inc_cursor();
                    if i == -1 || (i >= 1 && eol && count == 0) {
                        return OK;
                    }
                }
            }

            // go to next non-white
            while cls_impl(bigword) == 0 {
                // We'll stop if we land on a blank line
                if nvim_textobj_get_cursor_col() == 0 && *nvim_textobj_get_cursor_line_ptr() == 0 {
                    break;
                }

                let i = nvim_textobj_inc_cursor();
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
#[no_mangle]
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
            if nvim_textobj_dec_cursor() == -1 {
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
                    if nvim_textobj_dec_cursor() == -1 {
                        // hit start of file, stop here
                        return OK;
                    }
                }

                // Move backward to start of this word.
                if skip_chars_impl(cls_impl(bigword), BACKWARD, bigword) {
                    return OK;
                }
            }

            nvim_textobj_inc_cursor(); // overshot - forward one
            stop = false;
        }
        nvim_textobj_adjust_skipcol();
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
#[no_mangle]
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
                nvim_textobj_coladvance(MAXCOL);
            }

            let sclass = cls_impl(bigword);
            if nvim_textobj_inc_cursor() == -1 {
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
                    if nvim_textobj_inc_cursor() == -1 {
                        // hit end of file, stop here
                        return FAIL;
                    }
                }

                // Move forward to the end of this word.
                if skip_chars_impl(cls_impl(bigword), FORWARD, bigword) {
                    return FAIL;
                }
            }

            nvim_textobj_dec_cursor(); // overshot - one char backward
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
#[no_mangle]
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
            let i = nvim_textobj_dec_cursor();
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
                    let i = nvim_textobj_dec_cursor();
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
                let i = nvim_textobj_dec_cursor();
                if i == -1 || (eol && i == 1) {
                    return OK;
                }
            }
        }
        nvim_textobj_adjust_skipcol();
    }
    OK
}

// Additional extern declarations for word motion functions
extern "C" {
    /// Set cursor line number.
    fn nvim_textobj_set_cursor_lnum(lnum: c_int);

    /// Set cursor column.
    fn nvim_textobj_set_cursor_col(col: c_int);

    /// Check and unadjust for exclusive selection if needed.
    fn nvim_textobj_unadjust_for_sel_if_needed();

    /// Get length of multibyte character at position.
    fn nvim_textobj_utfc_ptr2len(p: *const std::ffi::c_char) -> c_int;

    /// Get head offset for multibyte char (bytes before start of char).
    fn nvim_textobj_utf_head_off(
        base: *const std::ffi::c_char,
        p: *const std::ffi::c_char,
    ) -> c_int;

    /// Search for character in string (like vim_strchr).
    fn nvim_textobj_vim_strchr(s: *const std::ffi::c_char, c: c_int) -> *const std::ffi::c_char;
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
        if !escape.is_null() && !nvim_textobj_vim_strchr(escape, c).is_null() {
            col += 1;
            if *line.offset(col as isize) == 0 {
                return -1;
            }
        } else if c == quotechar {
            break;
        }
        col += nvim_textobj_utfc_ptr2len(line.offset(col as isize));
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
        col_start -= nvim_textobj_utf_head_off(line, line.offset(col_start as isize));

        let mut n: c_int = 0;
        if !escape.is_null() {
            while col_start - n > 0 {
                let prev_char = char_to_int(*line.offset((col_start - n - 1) as isize));
                if nvim_textobj_vim_strchr(escape, prev_char).is_null() {
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

/// Update type for redraw
const UPD_INVERTED: c_int = 40;

extern "C" {
    /// Get VIsual_active state.
    fn nvim_textobj_get_VIsual_active() -> bool;

    /// Get VIsual position lnum.
    fn nvim_textobj_get_VIsual_lnum() -> c_int;

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

    /// Call oneleft().
    fn nvim_textobj_oneleft() -> c_int;

    /// Call incl on cursor.
    fn nvim_textobj_incl_cursor() -> c_int;

    /// Call decl on cursor.
    fn nvim_textobj_decl_cursor() -> c_int;

    /// Call redraw_curbuf_later.
    fn nvim_textobj_redraw_curbuf_later(update_type: c_int);

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
#[no_mangle]
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
        let inclusive = nvim_textobj_oneleft() != FAIL;
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
            nvim_textobj_dec_cursor();
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
                    nvim_textobj_oneleft();
                }
                if include {
                    state.include_white = true;
                }
            }

            if nvim_textobj_get_VIsual_active() {
                nvim_textobj_set_VIsual(state.start_pos.lnum, state.start_pos.col);
                nvim_textobj_redraw_curbuf_later(UPD_INVERTED);
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
        if nvim_textobj_oneleft() == OK {
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
            nvim_textobj_inc_cursor();
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
    /// Get p_sections option pointer.
    fn nvim_textobj_get_p_sections() -> *const std::ffi::c_char;

    /// Get p_para option pointer.
    fn nvim_textobj_get_p_para() -> *const std::ffi::c_char;

    /// Get line content at lnum.
    fn nvim_textobj_ml_get(lnum: c_int) -> *const std::ffi::c_char;

    /// Get line length at lnum.
    fn nvim_textobj_ml_get_len(lnum: c_int) -> c_int;

    /// Check if line is all whitespace.
    fn nvim_textobj_linewhite(lnum: c_int) -> bool;

    /// Call setpcmark.
    fn nvim_textobj_setpcmark();

    /// Call showmode.
    fn nvim_textobj_showmode();
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
#[no_mangle]
pub unsafe extern "C" fn rs_startPS(lnum: c_int, para: c_int, both: bool) -> bool {
    startps_impl(lnum, para, both)
}

/// Implementation of startPS.
unsafe fn startps_impl(lnum: c_int, para: c_int, both: bool) -> bool {
    let s = nvim_textobj_ml_get(lnum);
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
#[no_mangle]
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
            let line = nvim_textobj_ml_get(curr);
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

    nvim_textobj_setpcmark();

    // include line with '}'
    if both {
        let line = nvim_textobj_ml_get(curr);
        if char_to_int(*line) == i32::from(b'}') {
            curr += 1;
        }
    }

    nvim_textobj_set_cursor_lnum(curr);

    if curr == ml_line_count && what != i32::from(b'}') && dir == FORWARD {
        let line = nvim_textobj_ml_get(curr);
        let line_len = nvim_textobj_ml_get_len(curr);

        // Put the cursor on the last character in the last line and make the
        // motion inclusive.
        if line_len != 0 {
            let mut col = line_len - 1;
            col -= nvim_textobj_utf_head_off(line, line.offset(col as isize));
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
#[no_mangle]
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
    let white_in_front = nvim_textobj_linewhite(start_lnum);
    while start_lnum > 1 {
        if white_in_front {
            if !nvim_textobj_linewhite(start_lnum - 1) {
                break;
            }
        } else if nvim_textobj_linewhite(start_lnum - 1) || startps_impl(start_lnum, 0, false) {
            break;
        }
        start_lnum -= 1;
    }

    // Move past the end of any white lines.
    let mut end_lnum = start_lnum;
    while end_lnum <= ml_line_count && nvim_textobj_linewhite(end_lnum) {
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
            do_white = nvim_textobj_linewhite(end_lnum + 1);
        }

        if include || !do_white {
            end_lnum += 1;
            // skip to end of paragraph
            while end_lnum < ml_line_count
                && !nvim_textobj_linewhite(end_lnum + 1)
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
            while end_lnum < ml_line_count && nvim_textobj_linewhite(end_lnum + 1) {
                end_lnum += 1;
            }
        }
    }

    // If there are no empty lines at the end, try to find some empty lines at
    // the start (unless that has been done already).
    if !white_in_front && !nvim_textobj_linewhite(end_lnum) && include {
        while start_lnum > 1 && nvim_textobj_linewhite(start_lnum - 1) {
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
            let start_is_white = c_int::from(nvim_textobj_linewhite(start_lnum));

            if prev_start_is_white == start_is_white {
                start_lnum -= dir;
                break;
            }

            loop {
                if start_lnum == if dir == BACKWARD { 1 } else { ml_line_count } {
                    break;
                }

                let next_is_white = nvim_textobj_linewhite(start_lnum + dir);
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
        nvim_textobj_redraw_curbuf_later(UPD_INVERTED);
        nvim_textobj_showmode();
    } else {
        nvim_textobj_set_oap_start(oap, start_lnum, 0);
        nvim_textobj_set_oap_motion_type(oap, MT_LINE_WISE);
    }

    nvim_textobj_set_cursor_lnum(end_lnum);
    nvim_textobj_set_cursor_col(0);
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
