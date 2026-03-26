//! Open line functionality - adding new lines with smart indentation.
//!
//! This module provides the complex `open_line()` function which handles
//! creating new lines (Enter/o/O commands) with proper indentation,
//! comment continuation, and smart formatting.

use std::ffi::{c_char, c_int, c_void};

use crate::{BcountT, ColnrT, LinenrT, OpenlineFlags, NUL, OK};

// =============================================================================
// Direction Constants
// =============================================================================

/// Direction: forward (below current line).
pub const FORWARD: c_int = 1;
/// Direction: backward (above current line).
pub const BACKWARD: c_int = 0;

// =============================================================================
// Mode Constants
// =============================================================================

/// Insert mode flag.
const MODE_INSERT: c_int = 0x10;
/// VReplace mode flag.
const VREPLACE_FLAG: c_int = 0x80;
/// Normal replace state check.
#[inline]
const fn replace_normal(state: c_int) -> bool {
    (state & (VREPLACE_FLAG | MODE_INSERT)) == MODE_INSERT
}

// =============================================================================
// Key Constants
// =============================================================================

/// Key code for forward open.
const KEY_OPEN_FORW: c_int = 0x100 + 0x1b; // K_SPECIAL + 0x1b
/// Key code for backward open.
const KEY_OPEN_BACK: c_int = 0x100 + 0x1c; // K_SPECIAL + 0x1c

// =============================================================================
// Maximum Lengths
// =============================================================================

/// Maximum length for comment leader.
const COM_MAX_LEN: usize = 256;

// =============================================================================
// Extmark Constants
// =============================================================================

/// Extmark undo operation type.
const KEXTMARK_UNDO: c_int = 0;
/// Extmark no-op type.
const KEXTMARK_NOOP: c_int = 1;

// =============================================================================
// Position Type
// =============================================================================

/// Position in buffer (line, column).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PosT {
    /// Line number.
    pub lnum: LinenrT,
    /// Column number.
    pub col: ColnrT,
    /// Coladd for virtual column.
    pub coladd: ColnrT,
}

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // State accessors
    fn nvim_get_state() -> c_int;
    fn nvim_set_state(state: c_int);
    fn nvim_may_do_si() -> bool;
    fn nvim_get_did_si() -> bool;
    fn nvim_set_did_si(val: bool);
    fn nvim_get_can_si() -> bool;
    fn nvim_set_can_si(val: bool);
    fn nvim_get_can_si_back() -> bool;
    fn nvim_set_can_si_back(val: bool);
    fn nvim_get_did_ai() -> bool;
    fn nvim_set_did_ai(val: bool);
    fn nvim_get_ai_col() -> ColnrT;
    fn nvim_set_ai_col(val: ColnrT);
    fn nvim_p_paste() -> bool;
    fn nvim_p_sr() -> bool;
    fn nvim_get_end_comment_pending() -> c_int;
    fn nvim_set_end_comment_pending(val: c_int);
    fn nvim_get_orig_line_count() -> LinenrT;
    fn nvim_get_insstart() -> PosT;
    fn nvim_get_vr_lines_changed() -> c_int;
    fn nvim_set_vr_lines_changed(val: c_int);
    static mut curbuf_splice_pending: c_int;
    fn nvim_get_inhibit_delete_count() -> c_int;
    fn nvim_set_inhibit_delete_count(val: c_int);

    // Cursor/window accessors
    fn nvim_change_get_curwin_cursor() -> PosT;
    fn nvim_set_curwin_cursor(pos: PosT);
    fn nvim_get_curwin_cursor_lnum() -> LinenrT;
    fn nvim_set_curwin_cursor_lnum(lnum: LinenrT);
    fn nvim_get_curwin_cursor_col() -> ColnrT;
    fn nvim_set_curwin_cursor_col(col: ColnrT);
    fn nvim_set_curwin_cursor_coladd(coladd: ColnrT);

    // Global state
    fn nvim_get_curbuf() -> crate::BufHandle;

    // Buffer accessors
    fn nvim_curbuf_get_b_p_ai() -> bool;
    fn nvim_curbuf_get_b_p_ci() -> bool;
    fn nvim_curbuf_get_b_p_cin() -> bool;
    fn nvim_curbuf_get_b_p_lisp() -> bool;
    fn nvim_curbuf_get_b_p_pi() -> bool;
    fn nvim_curbuf_set_b_p_pi(val: bool);
    fn nvim_curbuf_get_b_p_ts() -> ColnrT;
    fn nvim_curbuf_get_b_p_vts_array() -> *const ColnrT;
    fn nvim_curbuf_get_b_p_inde_ptr() -> *const c_char;
    fn nvim_curbuf_get_b_p_com() -> *mut c_char;
    fn nvim_curbuf_get_b_ml_ml_line_count() -> LinenrT;
    fn nvim_change_bt_prompt() -> bool;
    fn nvim_get_curbuf_b_prompt_start_mark_lnum() -> LinenrT;
    fn nvim_get_cmdmod_cmod_flags() -> c_int;
    fn nvim_set_cmdmod_cmod_flags(val: c_int);

    // Line access
    fn nvim_ml_get(lnum: LinenrT) -> *mut c_char;
    fn nvim_ml_get_len(lnum: LinenrT) -> ColnrT;
    fn nvim_ml_replace(lnum: LinenrT, line: *mut c_char, copy: bool) -> c_int;
    fn nvim_ml_append(lnum: LinenrT, line: *const c_char, len: ColnrT, newfile: bool) -> c_int;
    fn nvim_get_cursor_line_ptr() -> *mut c_char;
    fn nvim_get_cursor_line_len() -> ColnrT;

    // Memory allocation
    fn nvim_xmalloc(size: usize) -> *mut c_char;
    fn nvim_xfree(ptr: *mut c_void);
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // String functions
    fn nvim_skipwhite(s: *const c_char) -> *mut c_char;
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn nvim_concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;

    // Indent functions
    fn nvim_indent_size_ts(line: *const c_char, ts: ColnrT, vts_array: *const ColnrT) -> c_int;
    fn nvim_get_indent() -> c_int;
    fn nvim_set_indent(size: c_int, flags: c_int) -> bool;
    fn nvim_copy_indent(size: c_int, src: *const c_char) -> bool;
    fn nvim_change_get_sw_value() -> c_int;
    #[link_name = "getwhitecols_curline"]
    fn nvim_getwhitecols_curline() -> c_int;
    fn nvim_linewhite(lnum: LinenrT) -> bool;
    fn nvim_truncate_spaces(line: *mut c_char, col: usize);

    // Comment leader functions
    fn nvim_get_leader_len(
        line: *const c_char,
        flags: *mut *mut c_char,
        backward: bool,
        include_space: bool,
    ) -> c_int;
    fn nvim_check_linecomment(line: *const c_char) -> ColnrT;
    fn nvim_change_copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: c_int,
        sep: *const c_char,
    ) -> usize;

    // Format option functions
    fn nvim_has_format_option(opt: c_int) -> bool;
    fn nvim_in_cinkeys(keytyped: c_int, when: c_int, line_is_white: bool) -> bool;
    fn nvim_cin_is_cinword(line: *const c_char) -> bool;

    // Findmatch function
    fn nvim_findmatch(initc: *mut c_char, ch: c_char) -> *mut PosT;

    // Undo functions
    fn nvim_u_clearline();
    fn nvim_u_save_cursor() -> c_int;

    // Replace stack functions
    fn nvim_replace_push(p: *const c_char, len: usize);
    fn nvim_replace_push_nul();

    // Mark functions
    fn nvim_mark_adjust(
        lnum: LinenrT,
        lnume: LinenrT,
        amount: LinenrT,
        amount_after: LinenrT,
        op: c_int,
    );
    fn nvim_mark_col_adjust(
        lnum: LinenrT,
        col: ColnrT,
        amount_lnum: LinenrT,
        amount_col: LinenrT,
        spaces_removed: ColnrT,
    );

    // Extmark functions
    fn nvim_extmark_splice(
        buf: crate::BufHandle,
        lnum: c_int,
        col: ColnrT,
        old_row: c_int,
        old_col: c_int,
        old_byte: c_int,
        new_row: c_int,
        new_col: c_int,
        new_byte: BcountT,
        undo: c_int,
    );
    fn nvim_extmark_splice_cols(
        buf: crate::BufHandle,
        lnum: c_int,
        col: ColnrT,
        old_col: c_int,
        new_col: c_int,
        undo: c_int,
    );

    // Changed notification functions
    fn nvim_changed_lines(
        buf: crate::BufHandle,
        lnum: LinenrT,
        col: ColnrT,
        lnume: LinenrT,
        extra: LinenrT,
        last_u: bool,
    );
    fn nvim_changed_bytes(lnum: LinenrT, col: ColnrT);

    // Indentation functions
    fn nvim_use_indentexpr_for_lisp() -> bool;
    fn nvim_fixthisline(get_indent_fn: *const c_void);
    fn nvim_get_lisp_indent() -> c_int;
    fn nvim_do_c_expr_indent();

    // Prompt functions
    fn nvim_prompt_text() -> *const c_char;

    // Multi-byte functions
    fn nvim_utfc_ptr2len(s: *const c_char) -> c_int;
    #[link_name = "utf_head_off"]
    fn nvim_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn nvim_ptr2cells(p: *const c_char) -> c_int;
    fn nvim_vim_strnsize(s: *const c_char, len: c_int) -> c_int;
    fn nvim_utf_iscomposing_first(c: c_int) -> c_int;
    fn nvim_utf_ptr2char(s: *const c_char) -> c_int;

    // Ins_bytes function
    fn nvim_ins_bytes(p: *const c_char);
}

#[inline]
unsafe fn nvim_strlen(s: *const c_char) -> usize {
    libc::strlen(s)
}

#[inline]
unsafe fn nvim_strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int {
    libc::strncmp(s1, s2, n)
}

#[inline]
unsafe fn nvim_strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char {
    libc::strcat(dest, src)
}

#[inline]
unsafe fn nvim_strmove(dest: *mut c_char, src: *const c_char) {
    libc::memmove(dest.cast(), src.cast(), libc::strlen(src) + 1);
}

#[inline]
unsafe fn nvim_xmemcpyz(dest: *mut c_char, src: *const c_char, len: usize) {
    std::ptr::copy_nonoverlapping(src, dest, len);
    *dest.add(len) = 0;
}

#[inline]
fn nvim_change_ascii_iswhite(c: c_int) -> bool {
    c == 0x20 || c == 0x09
}

// =============================================================================
// Format option constants
// =============================================================================

/// FO_NO_OPEN_COMS - don't open comments on o/O
const FO_NO_OPEN_COMS: c_int = 'o' as c_int;

// =============================================================================
// SIN (set indent) flags
// =============================================================================

/// SIN_INSERT - insert whitespace chars
const SIN_INSERT: c_int = 1;
/// SIN_NOMARK - don't adjust marks
const SIN_NOMARK: c_int = 8;

// =============================================================================
// CMOD flags
// =============================================================================

/// CMOD_LOCKMARKS - :lockmarks modifier
const CMOD_LOCKMARKS: c_int = 0x80;

// =============================================================================
// Comment Flags
// =============================================================================

/// COM_START - Start of multi-part comment
const COM_START: c_char = b's' as c_char;
/// COM_MIDDLE - Middle of multi-part comment
const COM_MIDDLE: c_char = b'm' as c_char;
/// COM_END - End of multi-part comment
const COM_END: c_char = b'e' as c_char;
/// COM_FIRST - First line comment only
const COM_FIRST: c_char = b'f' as c_char;
/// COM_BLANK - Blank required after leader
const COM_BLANK: c_char = b'b' as c_char;
/// COM_LEFT - Left adjusted
const COM_LEFT: c_char = b'l' as c_char;
/// COM_RIGHT - Right adjusted
const COM_RIGHT: c_char = b'r' as c_char;
/// COM_AUTO_END - Automatic end comment
const COM_AUTO_END: c_char = b'O' as c_char;

// =============================================================================
// MAXCOL constant
// =============================================================================

/// Maximum column number (used as sentinel value).
const MAXCOL: ColnrT = 0x7fffffff;

// =============================================================================
// MAXLNUM constant
// =============================================================================

/// Maximum line number.
const MAXLNUM: LinenrT = 0x7fffffff;

// =============================================================================
// TAB constant
// =============================================================================

/// Tab character.
const TAB: c_char = b'\t' as c_char;

// =============================================================================
// Open Line Implementation
// =============================================================================

/// Open a new line below or above the current line.
///
/// For MODE_VREPLACE state, we only add a new line when we get to the end of
/// the file, otherwise we just start replacing the next line.
///
/// Caller must take care of undo. Since MODE_VREPLACE may affect any number of
/// lines however, it may call u_save_cursor() again when starting to change a
/// new line.
///
/// # Arguments
/// * `dir` - FORWARD or BACKWARD
/// * `flags` - OPENLINE_* flags controlling behavior
/// * `second_line_indent` - indent for after ^^D or COM_LIST
/// * `did_do_comment` - set to true when intentionally putting comment leader
///
/// # Returns
/// true on success, false on failure
#[allow(clippy::too_many_lines)]
fn open_line_impl(
    dir: c_int,
    flags: c_int,
    second_line_indent: c_int,
    did_do_comment: *mut bool,
) -> bool {
    // SAFETY: All operations use safe FFI calls through accessor functions.
    // The function mirrors the C implementation exactly.
    unsafe {
        let mut next_line: *mut c_char = std::ptr::null_mut();
        let mut p_extra: *mut c_char = std::ptr::null_mut();
        let mut less_cols: ColnrT = 0;
        let mut less_cols_off: ColnrT = 0;
        let mut old_cursor: PosT;
        let mut newcol: ColnrT = 0;
        let mut newindent: c_int = 0;
        let mut trunc_line = false;
        #[allow(unused_assignments)]
        let mut retval = false;
        let mut extra_len: c_int = 0;
        let mut lead_len: c_int;
        let mut comment_start: c_int = 0;
        let mut lead_flags: *mut c_char = std::ptr::null_mut();
        let mut leader: *mut c_char = std::ptr::null_mut();
        let mut allocated: *mut c_char = std::ptr::null_mut();
        let mut p: *mut c_char;
        let mut saved_char: c_char = NUL;
        let do_si = nvim_may_do_si();
        let mut no_si = false;
        let mut first_char: c_int = NUL as c_int;
        let vreplace_mode: c_int;
        let mut did_append: bool;
        let saved_pi = nvim_curbuf_get_b_p_pi();

        let state = nvim_get_state();
        let lnum = nvim_get_curwin_cursor_lnum();
        let mincol = nvim_get_curwin_cursor_col() + 1;

        // Make a copy of the current line so we can mess with it
        let mut saved_line = nvim_xstrnsave(
            nvim_get_cursor_line_ptr(),
            nvim_get_cursor_line_len() as usize,
        );

        if (state & VREPLACE_FLAG) != 0 {
            // With MODE_VREPLACE we make a copy of the next line
            let cursor_lnum = nvim_get_curwin_cursor_lnum();
            if cursor_lnum < nvim_get_orig_line_count() {
                let next = nvim_ml_get(cursor_lnum + 1);
                next_line = nvim_xstrnsave(next, nvim_ml_get_len(cursor_lnum + 1) as usize);
            } else {
                next_line = nvim_xstrdup(c"".as_ptr());
            }

            // Push rest of line onto replace stack
            nvim_replace_push_nul();
            nvim_replace_push_nul();
            let col = nvim_get_curwin_cursor_col();
            p = saved_line.add(col as usize);
            nvim_replace_push(p, nvim_strlen(p));
            *saved_line.add(col as usize) = NUL;
        }

        if (state & MODE_INSERT) != 0 && (state & VREPLACE_FLAG) == 0 {
            let col = nvim_get_curwin_cursor_col();
            p_extra = saved_line.add(col as usize);
            if do_si {
                // Need first char after new line break
                p = nvim_skipwhite(p_extra);
                first_char = *p as u8 as c_int;
            }
            extra_len = nvim_strlen(p_extra) as c_int;
            saved_char = *p_extra;
            *p_extra = NUL;
        }

        nvim_u_clearline(); // Cannot do "U" command when adding lines
        nvim_set_did_si(false);
        nvim_set_ai_col(0);

        // If we just did an auto-indent, truncate the line
        if dir == FORWARD && nvim_get_did_ai() {
            trunc_line = true;
        }

        let openline_flags = OpenlineFlags::from_raw(flags);

        if openline_flags.contains(OpenlineFlags::FORCE_INDENT) {
            newindent = second_line_indent;
        } else if nvim_curbuf_get_b_p_ai() || do_si {
            // If 'autoindent' and/or 'smartindent' is set, figure out indent
            newindent = nvim_indent_size_ts(
                saved_line,
                nvim_curbuf_get_b_p_ts(),
                nvim_curbuf_get_b_p_vts_array(),
            );
            if newindent == 0 && !openline_flags.contains(OpenlineFlags::COM_LIST) {
                newindent = second_line_indent;
            }

            // Do smart indenting
            if !trunc_line
                && do_si
                && *saved_line != NUL
                && (p_extra.is_null() || first_char != b'{' as c_int)
            {
                old_cursor = nvim_change_get_curwin_cursor();
                let ptr = saved_line;

                if openline_flags.contains(OpenlineFlags::DO_COM) {
                    lead_len = nvim_get_leader_len(ptr, std::ptr::null_mut(), false, true);
                } else {
                    lead_len = 0;
                }

                if dir == FORWARD {
                    // Handle preprocessor directives and smart indent
                    if lead_len == 0 && *ptr == b'#' as c_char {
                        let mut cursor_lnum = nvim_get_curwin_cursor_lnum();
                        let mut current_ptr = ptr;
                        while *current_ptr == b'#' as c_char && cursor_lnum > 1 {
                            cursor_lnum -= 1;
                            nvim_set_curwin_cursor_lnum(cursor_lnum);
                            current_ptr = nvim_ml_get(cursor_lnum);
                        }
                        newindent = nvim_get_indent();
                    }

                    // Check for comment leader after skipping preprocessor
                    if openline_flags.contains(OpenlineFlags::DO_COM) {
                        lead_len = nvim_get_leader_len(ptr, std::ptr::null_mut(), false, true);
                    } else {
                        lead_len = 0;
                    }

                    if lead_len == 0 {
                        // Not a comment line - check for smart indent triggers
                        p = ptr.add(nvim_strlen(ptr) - 1);
                        while p > ptr && nvim_change_ascii_iswhite(*p as c_int) {
                            p = p.sub(1);
                        }
                        let last_char = *p;

                        if last_char == b'{' as c_char || last_char == b';' as c_char {
                            if p > ptr {
                                p = p.sub(1);
                            }
                            while p > ptr && nvim_change_ascii_iswhite(*p as c_int) {
                                p = p.sub(1);
                            }
                        }

                        if *p == b')' as c_char {
                            let col_offset = p.offset_from(ptr) as ColnrT;
                            let mut cursor = nvim_change_get_curwin_cursor();
                            cursor.col = col_offset;
                            nvim_set_curwin_cursor(cursor);
                            let pos = nvim_findmatch(std::ptr::null_mut(), b'(' as c_char);
                            if !pos.is_null() {
                                nvim_set_curwin_cursor_lnum((*pos).lnum);
                                newindent = nvim_get_indent();
                            }
                        }

                        if last_char == b'{' as c_char {
                            nvim_set_did_si(true);
                            no_si = true;
                        } else if last_char != b';' as c_char
                            && last_char != b'}' as c_char
                            && nvim_cin_is_cinword(ptr)
                        {
                            nvim_set_did_si(true);
                        }
                    }
                } else {
                    // dir == BACKWARD
                    if lead_len == 0 && *ptr == b'#' as c_char {
                        // Handle backward preprocessor directives
                        let mut was_backslashed = false;
                        let mut cursor_lnum = nvim_get_curwin_cursor_lnum();
                        let mut current_ptr = ptr;

                        while (*current_ptr == b'#' as c_char || was_backslashed)
                            && cursor_lnum < nvim_curbuf_get_b_ml_ml_line_count()
                        {
                            let len = nvim_strlen(current_ptr);
                            was_backslashed =
                                len > 0 && *current_ptr.add(len - 1) == b'\\' as c_char;
                            cursor_lnum += 1;
                            nvim_set_curwin_cursor_lnum(cursor_lnum);
                            current_ptr = nvim_ml_get(cursor_lnum);
                        }
                        newindent = if was_backslashed {
                            0
                        } else {
                            nvim_get_indent()
                        };
                    }
                    p = nvim_skipwhite(ptr);
                    if *p == b'}' as c_char {
                        nvim_set_did_si(true);
                    } else {
                        nvim_set_can_si_back(true);
                    }
                }
                nvim_set_curwin_cursor(old_cursor);
            }
            if do_si {
                nvim_set_can_si(true);
            }

            nvim_set_did_ai(true);
        }

        // May do indenting after opening a new line
        let do_cindent = !nvim_p_paste()
            && (nvim_curbuf_get_b_p_cin() || *nvim_curbuf_get_b_p_inde_ptr() != NUL)
            && nvim_in_cinkeys(
                if dir == FORWARD {
                    KEY_OPEN_FORW
                } else {
                    KEY_OPEN_BACK
                },
                b' ' as c_int,
                nvim_linewhite(nvim_get_curwin_cursor_lnum()),
            )
            && !openline_flags.contains(OpenlineFlags::FORCE_INDENT);

        // Find out if the current line starts with a comment leader
        nvim_set_end_comment_pending(NUL as c_int);
        if openline_flags.contains(OpenlineFlags::DO_COM) {
            lead_len = nvim_get_leader_len(saved_line, &mut lead_flags, dir == BACKWARD, true);
            if lead_len == 0
                && nvim_curbuf_get_b_p_cin()
                && do_cindent
                && dir == FORWARD
                && (!nvim_has_format_option(FO_NO_OPEN_COMS)
                    || openline_flags.contains(OpenlineFlags::FORMAT))
            {
                // Check for a line comment after code
                comment_start = nvim_check_linecomment(saved_line);
                if comment_start != MAXCOL {
                    lead_len = nvim_get_leader_len(
                        saved_line.add(comment_start as usize),
                        &mut lead_flags,
                        false,
                        true,
                    );
                    if lead_len != 0 {
                        lead_len += comment_start;
                        if !did_do_comment.is_null() {
                            *did_do_comment = true;
                        }
                    }
                }
            }
        } else {
            lead_len = 0;
        }

        // Process comment leader if present
        if lead_len > 0 {
            let mut lead_repl: *const c_char = std::ptr::null();
            let mut lead_repl_len: c_int = 0;
            let mut lead_middle: [c_char; COM_MAX_LEN] = [0; COM_MAX_LEN];
            let mut lead_end: [c_char; COM_MAX_LEN] = [0; COM_MAX_LEN];
            let mut comment_end: *mut c_char = std::ptr::null_mut();
            let mut extra_space = false;
            let mut require_blank = false;

            // Check comment flags for start/middle/end handling
            p = lead_flags;
            while !p.is_null() && *p != NUL && *p != b':' as c_char {
                if *p == COM_BLANK {
                    require_blank = true;
                    p = p.add(1);
                    continue;
                }
                if *p == COM_START || *p == COM_MIDDLE {
                    let current_flag = *p;
                    if *p == COM_START {
                        // Doing "O" on start of comment does not insert leader
                        if dir == BACKWARD {
                            lead_len = 0;
                            break;
                        }
                        // Find start of middle part
                        nvim_change_copy_option_part(
                            &mut p,
                            lead_middle.as_mut_ptr(),
                            COM_MAX_LEN as c_int,
                            c",".as_ptr(),
                        );
                        require_blank = false;
                    }

                    // Isolate middle and end leader strings
                    while !p.is_null() && *p.sub(1) != b':' as c_char {
                        if *p == COM_BLANK {
                            require_blank = true;
                        }
                        p = p.add(1);
                    }
                    nvim_change_copy_option_part(
                        &mut p,
                        lead_middle.as_mut_ptr(),
                        COM_MAX_LEN as c_int,
                        c",".as_ptr(),
                    );

                    while !p.is_null() && *p.sub(1) != b':' as c_char {
                        if *p == COM_AUTO_END {
                            nvim_set_end_comment_pending(-1);
                        }
                        p = p.add(1);
                    }
                    let n = nvim_change_copy_option_part(
                        &mut p,
                        lead_end.as_mut_ptr(),
                        COM_MAX_LEN as c_int,
                        c",".as_ptr(),
                    );

                    if nvim_get_end_comment_pending() == -1 && n > 0 {
                        nvim_set_end_comment_pending(lead_end[n - 1] as u8 as c_int);
                    }

                    // If end of comment is in same line, don't use leader
                    if dir == FORWARD {
                        let mut check_p = saved_line.add(lead_len as usize);
                        while *check_p != NUL {
                            if nvim_strncmp(check_p, lead_end.as_ptr(), n) == 0 {
                                comment_end = check_p;
                                lead_len = 0;
                                break;
                            }
                            check_p = check_p.add(1);
                        }
                    }

                    // Doing "o" on start of comment inserts middle leader
                    if lead_len > 0 {
                        if current_flag == COM_START {
                            lead_repl = lead_middle.as_ptr();
                            lead_repl_len = nvim_strlen(lead_middle.as_ptr()) as c_int;
                        }

                        // Add extra space if needed
                        let col = nvim_get_curwin_cursor_col();
                        if !nvim_change_ascii_iswhite(
                            *saved_line.add((lead_len - 1) as usize) as c_int
                        ) && ((!p_extra.is_null() && col == lead_len)
                            || (p_extra.is_null() && *saved_line.add(lead_len as usize) == NUL)
                            || require_blank)
                        {
                            extra_space = true;
                        }
                    }
                    break;
                }
                if *p == COM_END {
                    // Doing "o" on end of comment does not insert leader
                    if dir == FORWARD {
                        comment_end = nvim_skipwhite(saved_line);
                        lead_len = 0;
                        break;
                    }
                    // Doing "O" on end of comment inserts middle leader
                    // Find middle leader by searching backwards
                    let com = nvim_curbuf_get_b_p_com();
                    while p > com && *p != b',' as c_char {
                        p = p.sub(1);
                    }
                    let mut repl_p = p;
                    while repl_p > com && *repl_p.sub(1) != b':' as c_char {
                        repl_p = repl_p.sub(1);
                    }
                    lead_repl = repl_p;
                    lead_repl_len = p.offset_from(repl_p) as c_int;
                    extra_space = true;

                    // Check for auto-end
                    let mut p2 = p;
                    while *p2 != NUL && *p2 != b':' as c_char {
                        if *p2 == COM_AUTO_END {
                            nvim_set_end_comment_pending(-1);
                        }
                        p2 = p2.add(1);
                    }
                    if nvim_get_end_comment_pending() == -1 {
                        while *p2 != NUL && *p2 != b',' as c_char {
                            p2 = p2.add(1);
                        }
                        nvim_set_end_comment_pending(*p2.sub(1) as u8 as c_int);
                    }
                    break;
                }
                if *p == COM_FIRST {
                    // Comment leader for first line only
                    if dir == BACKWARD {
                        lead_len = 0;
                    } else {
                        lead_repl = c"".as_ptr();
                        lead_repl_len = 0;
                    }
                    break;
                }
                p = p.add(1);
            }

            if lead_len > 0 {
                // Allocate buffer for leader
                let bytes = lead_len
                    + lead_repl_len
                    + (if extra_space { 1 } else { 0 })
                    + extra_len
                    + (if second_line_indent > 0 {
                        second_line_indent
                    } else {
                        0
                    })
                    + 1;
                leader = nvim_xmalloc(bytes as usize);
                allocated = leader;

                nvim_xmemcpyz(leader, saved_line, lead_len as usize);

                // Replace non-whitespace in comment_start region with spaces
                for li in 0..comment_start {
                    if !nvim_change_ascii_iswhite(*leader.add(li as usize) as c_int) {
                        *leader.add(li as usize) = b' ' as c_char;
                    }
                }

                // Replace leader with lead_repl if needed
                if !lead_repl.is_null() {
                    // Handle right/left adjusted leaders
                    let mut c: c_int = 0;
                    let mut off: c_int = 0;

                    let mut flags_p = lead_flags;
                    while !flags_p.is_null() && *flags_p != NUL && *flags_p != b':' as c_char {
                        if *flags_p == COM_RIGHT || *flags_p == COM_LEFT {
                            c = *flags_p as u8 as c_int;
                            flags_p = flags_p.add(1);
                        } else if (*flags_p as u8 as char).is_ascii_digit()
                            || *flags_p == b'-' as c_char
                        {
                            // Parse offset number
                            let mut sign = 1;
                            if *flags_p == b'-' as c_char {
                                sign = -1;
                                flags_p = flags_p.add(1);
                            }
                            off = 0;
                            while (*flags_p as u8 as char).is_ascii_digit() {
                                off = off * 10 + (*flags_p as u8 - b'0') as c_int;
                                flags_p = flags_p.add(1);
                            }
                            off *= sign;
                        } else {
                            flags_p = flags_p.add(1);
                        }
                    }

                    if c == COM_RIGHT as c_int {
                        // Right adjusted leader
                        p = leader.add((lead_len - 1) as usize);
                        while p > leader && nvim_change_ascii_iswhite(*p as c_int) {
                            p = p.sub(1);
                        }
                        p = p.add(1);

                        let repl_size = nvim_vim_strnsize(lead_repl, lead_repl_len);
                        let mut old_size = 0;
                        let endp = p;

                        while old_size < repl_size && p > leader {
                            let head_off = nvim_utf_head_off(leader, p.sub(1));
                            p = p.sub(1 + head_off as usize);
                            old_size += nvim_ptr2cells(p);
                        }
                        let l = lead_repl_len - (endp.offset_from(p) as c_int);
                        if l != 0 {
                            std::ptr::copy(
                                endp,
                                endp.offset(l as isize),
                                (leader.add(lead_len as usize).offset_from(endp)) as usize,
                            );
                        }
                        lead_len += l;
                        std::ptr::copy_nonoverlapping(lead_repl, p, lead_repl_len as usize);
                        if p.offset(lead_repl_len as isize) > leader.add(lead_len as usize) {
                            *p.add(lead_repl_len as usize) = NUL;
                        }

                        // Blank out other chars
                        while p > leader {
                            p = p.sub(1);
                            let head_off = nvim_utf_head_off(leader, p);
                            if head_off > 1 {
                                p = p.sub(head_off as usize);
                                if nvim_ptr2cells(p) > 1 {
                                    *p.add(1) = b' ' as c_char;
                                }
                                std::ptr::copy(
                                    p.add(head_off as usize + 1),
                                    p.add(1),
                                    (leader
                                        .add(lead_len as usize)
                                        .offset_from(p.add(head_off as usize + 1)))
                                        as usize,
                                );
                                lead_len -= head_off;
                                *p = b' ' as c_char;
                            } else if !nvim_change_ascii_iswhite(*p as c_int) {
                                *p = b' ' as c_char;
                            }
                        }
                    } else {
                        // Left adjusted leader
                        p = nvim_skipwhite(leader);
                        let repl_size = nvim_vim_strnsize(lead_repl, lead_repl_len);
                        let mut i = 0;
                        while i < lead_len && *p.add(i as usize) != NUL {
                            let l = nvim_utfc_ptr2len(p.add(i as usize));
                            if nvim_vim_strnsize(p, i + l) > repl_size {
                                break;
                            }
                            i += l;
                        }
                        if i != lead_repl_len {
                            std::ptr::copy(
                                p.add(i as usize),
                                p.add(lead_repl_len as usize),
                                (lead_len - i - (p.offset_from(leader) as c_int)) as usize,
                            );
                            lead_len += lead_repl_len - i;
                        }
                        std::ptr::copy_nonoverlapping(lead_repl, p, lead_repl_len as usize);

                        // Replace remaining non-white chars with spaces
                        p = p.add(lead_repl_len as usize);
                        while p < leader.add(lead_len as usize) {
                            if !nvim_change_ascii_iswhite(*p as c_int) {
                                let l = nvim_utfc_ptr2len(p);
                                if l > 1 {
                                    if nvim_ptr2cells(p) > 1 {
                                        *p = b' ' as c_char;
                                        p = p.add(1);
                                    }
                                    std::ptr::copy(
                                        p.add(l as usize),
                                        p.add(1),
                                        (leader
                                            .add(lead_len as usize)
                                            .offset_from(p.add(l as usize)))
                                            as usize,
                                    );
                                    lead_len -= l - 1;
                                }
                                *p = b' ' as c_char;
                            }
                            p = p.add(1);
                        }
                        *p = NUL;
                    }

                    // Recompute indent
                    if nvim_curbuf_get_b_p_ai() || do_si {
                        newindent = nvim_indent_size_ts(
                            leader,
                            nvim_curbuf_get_b_p_ts(),
                            nvim_curbuf_get_b_p_vts_array(),
                        );
                    }

                    // Add indent offset
                    if newindent + off < 0 {
                        off = -newindent;
                        newindent = 0;
                    } else {
                        newindent += off;
                    }

                    // Correct trailing spaces
                    while off > 0
                        && lead_len > 0
                        && *leader.add((lead_len - 1) as usize) == b' ' as c_char
                    {
                        if !nvim_vim_strchr(nvim_skipwhite(leader), TAB as c_int).is_null() {
                            break;
                        }
                        lead_len -= 1;
                        off -= 1;
                    }

                    // If leader ends in whitespace, don't add extra space
                    if lead_len > 0
                        && nvim_change_ascii_iswhite(*leader.add((lead_len - 1) as usize) as c_int)
                    {
                        extra_space = false;
                    }
                    *leader.add(lead_len as usize) = NUL;
                }

                if extra_space {
                    *leader.add(lead_len as usize) = b' ' as c_char;
                    lead_len += 1;
                    *leader.add(lead_len as usize) = NUL;
                }

                newcol = lead_len;

                // Remove indent in comment leader if new indent will be set
                if newindent != 0 || nvim_get_did_si() {
                    while lead_len > 0 && nvim_change_ascii_iswhite(*leader as c_int) {
                        lead_len -= 1;
                        newcol -= 1;
                        leader = leader.add(1);
                    }
                }
                nvim_set_did_si(false);
                nvim_set_can_si(false);
            } else if !comment_end.is_null() {
                // Finished a comment, align with start
                if *comment_end == b'*' as c_char
                    && *comment_end.add(1) == b'/' as c_char
                    && (nvim_curbuf_get_b_p_ai() || do_si)
                {
                    old_cursor = nvim_change_get_curwin_cursor();
                    let col_offset = comment_end.offset_from(saved_line) as ColnrT;
                    let mut cursor = nvim_change_get_curwin_cursor();
                    cursor.col = col_offset;
                    nvim_set_curwin_cursor(cursor);
                    let pos = nvim_findmatch(std::ptr::null_mut(), NUL as c_char);
                    if !pos.is_null() {
                        nvim_set_curwin_cursor_lnum((*pos).lnum);
                        newindent = nvim_get_indent();
                    }
                    nvim_set_curwin_cursor(old_cursor);
                }
            }
        }

        // (State == MODE_INSERT || State == MODE_REPLACE), only when dir == FORWARD
        if !p_extra.is_null() {
            *p_extra = saved_char;

            // When 'ai' set or OPENLINE_DELSPACES, skip to first non-blank
            if replace_normal(state) {
                nvim_replace_push_nul();
            }
            if nvim_curbuf_get_b_p_ai() || openline_flags.contains(OpenlineFlags::DELSPACES) {
                while (*p_extra == b' ' as c_char || *p_extra == TAB)
                    && nvim_utf_iscomposing_first(nvim_utf_ptr2char(p_extra.add(1))) == 0
                {
                    if replace_normal(state) {
                        nvim_replace_push(p_extra, 1);
                    }
                    p_extra = p_extra.add(1);
                    less_cols_off += 1;
                }
            }

            // Columns for marks adjusted for removed columns
            less_cols = p_extra.offset_from(saved_line) as ColnrT;
        }

        if p_extra.is_null() {
            p_extra = c"".as_ptr() as *mut c_char;
        }

        // Concatenate leader and p_extra if there is a leader
        if lead_len > 0 {
            if openline_flags.contains(OpenlineFlags::COM_LIST) && second_line_indent > 0 {
                let padding = second_line_indent - (newindent + nvim_strlen(leader) as c_int);
                for _ in 0..padding {
                    nvim_strcat(leader, c" ".as_ptr());
                    less_cols -= 1;
                    newcol += 1;
                }
            }
            nvim_strcat(leader, p_extra);
            p_extra = leader;
            nvim_set_did_ai(true);
            less_cols -= lead_len;
        } else {
            nvim_set_end_comment_pending(NUL as c_int);
        }

        // Increment splice pending counter
        let splice_pending = curbuf_splice_pending;
        curbuf_splice_pending = splice_pending + 1;

        old_cursor = nvim_change_get_curwin_cursor();
        let old_cmod_flags = nvim_get_cmdmod_cmod_flags();
        let mut prompt_moved: *mut c_char = std::ptr::null_mut();

        if dir == BACKWARD {
            // Handle prompt buffer case
            let cursor_lnum = nvim_get_curwin_cursor_lnum();
            if nvim_change_bt_prompt() && cursor_lnum == nvim_get_curbuf_b_prompt_start_mark_lnum()
            {
                let prompt_line = nvim_ml_get(cursor_lnum);
                let prompt = nvim_prompt_text();
                let prompt_len = nvim_strlen(prompt);

                if nvim_strncmp(prompt_line, prompt, prompt_len) == 0 {
                    nvim_strmove(prompt_line, prompt_line.add(prompt_len));
                    nvim_set_cmdmod_cmod_flags(nvim_get_cmdmod_cmod_flags() | CMOD_LOCKMARKS);
                    nvim_ml_replace(cursor_lnum, prompt_line, true);
                    prompt_moved = nvim_concat_str(prompt, p_extra);
                    p_extra = prompt_moved;
                }
            }
            nvim_set_curwin_cursor_lnum(nvim_get_curwin_cursor_lnum() - 1);
        }

        if (state & VREPLACE_FLAG) == 0 || old_cursor.lnum >= nvim_get_orig_line_count() {
            let cursor_lnum = nvim_get_curwin_cursor_lnum();
            if nvim_ml_append(cursor_lnum, p_extra, 0, false) != OK {
                // Cleanup and return on failure
                curbuf_splice_pending -= 1;
                nvim_curbuf_set_b_p_pi(saved_pi);
                nvim_xfree(saved_line.cast());
                nvim_xfree(next_line.cast());
                nvim_xfree(allocated.cast());
                nvim_xfree(prompt_moved.cast());
                nvim_set_cmdmod_cmod_flags(old_cmod_flags);
                return false;
            }
            // Postpone calling changed_lines
            nvim_mark_adjust(cursor_lnum + 1, MAXLNUM, 1, 0, KEXTMARK_NOOP);
            did_append = true;
        } else {
            // MODE_VREPLACE state - replacing next line
            let mut cursor_lnum = nvim_get_curwin_cursor_lnum();
            cursor_lnum += 1;
            nvim_set_curwin_cursor_lnum(cursor_lnum);
            let insstart = nvim_get_insstart();
            let vr_lines = nvim_get_vr_lines_changed();
            if cursor_lnum >= insstart.lnum + vr_lines as LinenrT {
                nvim_u_save_cursor();
                nvim_set_vr_lines_changed(vr_lines + 1);
            }
            nvim_ml_replace(cursor_lnum, p_extra, true);
            nvim_changed_bytes(cursor_lnum, 0);
            nvim_set_curwin_cursor_lnum(cursor_lnum - 1);
            did_append = false;
        }

        // Handle indentation
        let inhibit = nvim_get_inhibit_delete_count();
        nvim_set_inhibit_delete_count(inhibit + 1);

        if newindent != 0 || nvim_get_did_si() {
            nvim_set_curwin_cursor_lnum(nvim_get_curwin_cursor_lnum() + 1);
            if nvim_get_did_si() {
                let sw = nvim_change_get_sw_value();
                if nvim_p_sr() {
                    newindent -= newindent % sw;
                }
                newindent += sw;
            }

            // Copy the indent
            if nvim_curbuf_get_b_p_ci() {
                nvim_copy_indent(newindent, saved_line);
                nvim_curbuf_set_b_p_pi(true);
            } else {
                nvim_set_indent(newindent, SIN_INSERT | SIN_NOMARK);
            }
            less_cols -= nvim_get_curwin_cursor_col();

            nvim_set_ai_col(nvim_get_curwin_cursor_col());

            // In MODE_REPLACE, push NULs for indent
            if replace_normal(state) {
                let col = nvim_get_curwin_cursor_col();
                for _ in 0..col {
                    nvim_replace_push_nul();
                }
            }
            newcol += nvim_get_curwin_cursor_col();
            if no_si {
                nvim_set_did_si(false);
            }
        }

        nvim_set_inhibit_delete_count(nvim_get_inhibit_delete_count() - 1);

        // Push NULs for extra leader in MODE_REPLACE
        if replace_normal(state) {
            let mut lead_remaining = lead_len;
            while lead_remaining > 0 {
                nvim_replace_push_nul();
                lead_remaining -= 1;
            }
        }

        nvim_set_curwin_cursor(old_cursor);

        if dir == FORWARD {
            if trunc_line || (state & MODE_INSERT) != 0 {
                // Truncate current line at cursor
                let col = nvim_get_curwin_cursor_col();
                *saved_line.add(col as usize) = NUL;
                // Remove trailing whitespace unless KEEPTRAIL
                if trunc_line && !openline_flags.contains(OpenlineFlags::KEEPTRAIL) {
                    nvim_truncate_spaces(saved_line, col as usize);
                }
                let cursor_lnum = nvim_get_curwin_cursor_lnum();
                nvim_ml_replace(cursor_lnum, saved_line, false);

                let new_len = nvim_strlen(saved_line) as c_int;

                let mut cols_spliced = 0;
                let curbuf = nvim_get_curbuf();
                if new_len < col {
                    nvim_extmark_splice_cols(
                        curbuf,
                        (cursor_lnum - 1) as c_int,
                        new_len,
                        col - new_len,
                        0,
                        KEXTMARK_UNDO,
                    );
                    cols_spliced = col - new_len;
                }

                saved_line = std::ptr::null_mut();
                if did_append {
                    // Move extmarks
                    let cols_added = mincol - 1 + less_cols_off - less_cols;
                    nvim_extmark_splice(
                        curbuf,
                        (lnum - 1) as c_int,
                        mincol - 1 - cols_spliced,
                        0,
                        less_cols_off,
                        less_cols_off,
                        1,
                        cols_added,
                        (1 + cols_added) as BcountT,
                        KEXTMARK_UNDO,
                    );

                    nvim_changed_lines(
                        nvim_get_curbuf(),
                        cursor_lnum,
                        col,
                        cursor_lnum + 1,
                        1,
                        true,
                    );
                    did_append = false;

                    // Move marks after line break
                    if openline_flags.contains(OpenlineFlags::MARKFIX) {
                        nvim_mark_col_adjust(
                            cursor_lnum,
                            col + less_cols_off,
                            1,
                            -(less_cols as LinenrT),
                            0,
                        );
                    }
                } else {
                    nvim_changed_bytes(cursor_lnum, col);
                }
            }

            // Put cursor on new line
            nvim_set_curwin_cursor_lnum(old_cursor.lnum + 1);
        }

        if did_append {
            let cursor_lnum = nvim_get_curwin_cursor_lnum();
            let extra = nvim_ml_get_len(cursor_lnum) as BcountT;
            nvim_extmark_splice(
                nvim_get_curbuf(),
                (cursor_lnum - 1) as c_int,
                0,
                0,
                0,
                0,
                1,
                0,
                1 + extra,
                KEXTMARK_UNDO,
            );
            nvim_changed_lines(nvim_get_curbuf(), cursor_lnum, 0, cursor_lnum, 1, true);
        }

        curbuf_splice_pending -= 1;

        nvim_set_curwin_cursor_col(newcol);
        nvim_set_curwin_cursor_coladd(0);

        // Handle MODE_VREPLACE state
        if (state & VREPLACE_FLAG) != 0 {
            vreplace_mode = state;
            nvim_set_state(MODE_INSERT);
        } else {
            vreplace_mode = 0;
        }

        // Do indentation
        if !nvim_p_paste() {
            if leader.is_null()
                && !nvim_use_indentexpr_for_lisp()
                && nvim_curbuf_get_b_p_lisp()
                && nvim_curbuf_get_b_p_ai()
            {
                // Do lisp indenting
                nvim_fixthisline(nvim_get_lisp_indent as *const c_void);
                nvim_set_ai_col(nvim_getwhitecols_curline() as ColnrT);
            } else if do_cindent || (nvim_curbuf_get_b_p_ai() && nvim_use_indentexpr_for_lisp()) {
                // Do 'cindent' or 'indentexpr' indenting
                nvim_do_c_expr_indent();
                nvim_set_ai_col(nvim_getwhitecols_curline() as ColnrT);
            }
        }

        if vreplace_mode != 0 {
            nvim_set_state(vreplace_mode);
        }

        // MODE_VREPLACE final handling
        if (nvim_get_state() & VREPLACE_FLAG) != 0 {
            // Put new line in p_extra
            let new_p_extra = nvim_xstrnsave(
                nvim_get_cursor_line_ptr(),
                nvim_get_cursor_line_len() as usize,
            );

            // Put back original line
            nvim_ml_replace(nvim_get_curwin_cursor_lnum(), next_line, false);

            // Insert new stuff into line
            nvim_set_curwin_cursor_col(0);
            nvim_set_curwin_cursor_coladd(0);
            nvim_ins_bytes(new_p_extra);
            nvim_xfree(new_p_extra.cast());
            next_line = std::ptr::null_mut();
        }

        retval = true;

        // Cleanup
        nvim_curbuf_set_b_p_pi(saved_pi);
        nvim_xfree(saved_line.cast());
        nvim_xfree(next_line.cast());
        nvim_xfree(allocated.cast());
        nvim_xfree(prompt_moved.cast());
        nvim_set_cmdmod_cmod_flags(old_cmod_flags);

        retval
    }
}

/// FFI wrapper for `open_line`.
///
/// Open a new line below or above the current line.
///
/// # Arguments
/// * `dir` - FORWARD or BACKWARD
/// * `flags` - OPENLINE_* flags
/// * `second_line_indent` - indent for after ^^D or COM_LIST
/// * `did_do_comment` - set to true when comment leader was added
///
/// # Returns
/// true on success, false on failure
#[export_name = "open_line"]
pub extern "C" fn rs_open_line(
    dir: c_int,
    flags: c_int,
    second_line_indent: c_int,
    did_do_comment: *mut bool,
) -> bool {
    open_line_impl(dir, flags, second_line_indent, did_do_comment)
}

/// Get FORWARD direction constant.
#[no_mangle]
pub extern "C" fn rs_open_line_forward() -> c_int {
    FORWARD
}

/// Get BACKWARD direction constant.
#[no_mangle]
pub extern "C" fn rs_open_line_backward() -> c_int {
    BACKWARD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, 0);
    }

    #[test]
    fn test_mode_constants() {
        assert_eq!(MODE_INSERT, 0x10);
        assert_eq!(VREPLACE_FLAG, 0x80);
    }

    #[test]
    fn test_replace_normal() {
        // Only MODE_INSERT without VREPLACE is "normal" replace
        assert!(replace_normal(MODE_INSERT));
        assert!(!replace_normal(MODE_INSERT | VREPLACE_FLAG));
        assert!(!replace_normal(VREPLACE_FLAG));
        assert!(!replace_normal(0));
    }
}
