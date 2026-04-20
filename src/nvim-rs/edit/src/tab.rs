//! Tab insertion handler migrated from edit.c
//!
//! Implements `ins_tab` -- TAB handling in Insert/Replace/VReplace mode
//! with expandtab, softtabstop, vartabstop support.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_long};

/// Column number type (matches `colnr_T` in Neovim).
type ColnrT = i32;

/// Line number type (matches `linenr_T` in Neovim).
type LinenrT = i32;

// ============================================================================
// C accessor functions
// ============================================================================

extern "C" {
    static mut State: c_int;
    static mut p_sta: c_int;
}

extern "C" {
    // Insstart_blank_vcol
    fn nvim_get_Insstart_blank_vcol() -> ColnrT;
    fn nvim_set_Insstart_blank_vcol(val: ColnrT);

    // Insstart
    fn nvim_get_Insstart_lnum() -> LinenrT;

    // Cursor
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;

    // get_nolist_virtcol (canonical name, exported from Rust)
    fn get_nolist_virtcol() -> ColnrT;

    // Abbreviation check
    fn echeck_abbr(c: c_int) -> c_int;

    // Indent check
    fn inindent(extra: c_int) -> c_int;

    // can_cindent
    fn nvim_set_can_cindent(val: c_int);

    // did_ai, did_si, can_si, can_si_back (from change_ffi.c, using bool)
    fn nvim_set_did_ai(val: bool);
    fn nvim_set_did_si(val: bool);
    fn nvim_set_can_si(val: bool);
    fn nvim_set_can_si_back(val: bool);

    // Redo buffer via existing accessor
    fn AppendToRedobuff(str: *const c_char);

    // stop_arrow
    fn stop_arrow() -> c_int;

    // State (REPLACE_FLAG / VREPLACE_FLAG)

    // expandtab and tabstop options
    fn nvim_curbuf_get_b_p_et() -> c_int; // from option_shim.c
    fn nvim_curbuf_get_b_p_ts() -> ColnrT; // from change_ffi.c (returns colnr_T)
    fn nvim_curbuf_get_b_p_sts() -> c_long;
    fn nvim_curbuf_tabstop_count_vts() -> c_int;
    fn nvim_curbuf_tabstop_count_vsts() -> c_int;
    fn nvim_curbuf_tabstop_first_vts() -> c_long;
    fn nvim_curbuf_get_sw_value() -> c_long;
    fn nvim_get_sts_value() -> c_long;

    // Tabstop padding
    fn nvim_curbuf_tabstop_padding_sts() -> c_int;
    fn nvim_curbuf_tabstop_padding_ts() -> c_int;

    // Character insertion
    fn ins_char(c: c_int);
    #[link_name = "ins_str"]
    fn nvim_ins_str(s: *const c_char, len: usize);
    fn replace_push_nul();

    // --- Space-to-TAB replacement helpers ---
    fn nvim_edit_tab_save_list() -> c_int;
    fn nvim_edit_tab_restore_list(save_list: c_int);
    fn nvim_edit_tab_cursor_col(lnum_out: *mut LinenrT) -> ColnrT;
    fn nvim_edit_tab_set_cursor_col(col: ColnrT);
    fn nvim_edit_tab_get_cursor(lnum: *mut LinenrT, col: *mut ColnrT);
    fn nvim_edit_tab_is_vreplace() -> bool;
    fn nvim_edit_tab_is_replace() -> bool;
    fn nvim_edit_tab_strnsave_cursor_line() -> *mut c_char;
    fn nvim_edit_tab_get_cursor_pos_ptr() -> *mut c_char;
    fn nvim_ascii_iswhite(c: c_char) -> bool;
    fn nvim_edit_tab_getvcol(lnum: LinenrT, col: ColnrT) -> ColnrT;
    fn nvim_edit_tab_charsize_tab(vcol: ColnrT) -> c_int;
    fn nvim_edit_tab_charsize_space(vcol: ColnrT, ptr: *const c_char) -> c_int;
    fn nvim_edit_tab_get_Insstart(lnum: *mut LinenrT, col: *mut ColnrT);
    fn nvim_edit_tab_set_Insstart_col(col: ColnrT);
    fn nvim_edit_tab_rewrite_line(
        ptr: *mut c_char,
        i: c_int,
        change_col: ColnrT,
        cursor_col: ColnrT,
        fpos_col: ColnrT,
        fpos_lnum: LinenrT,
    );
    fn nvim_edit_tab_strmove(ptr: *mut c_char, i: c_int);
    #[link_name = "backspace_until_column"]
    fn nvim_edit_tab_backspace_until_column(col: ColnrT);
    #[link_name = "ins_bytes_len"]
    fn nvim_edit_tab_ins_bytes_len(s: *const c_char, len: usize);
    #[link_name = "replace_join"]
    fn nvim_edit_tab_replace_join(off: c_int);
    fn xfree(ptr: *mut std::ffi::c_void);
}

// ============================================================================
// Constants (verified via `_Static_assert` in `edit.c`)
// ============================================================================

/// `MAXCOL` from `pos_defs.h`
const MAXCOL: ColnrT = 0x7fff_ffff;

/// `FAIL` from `vim_defs.h`
const FAIL: c_int = 0;

/// `ABBR_OFF` from `keycodes.h`
const ABBR_OFF: c_int = 0x100;

/// TAB character
const TAB: c_int = b'\t' as c_int;

/// `REPLACE_FLAG` from `vim_defs.h`
const REPLACE_FLAG: c_int = 0x100;

/// `VREPLACE_FLAG` from `vim_defs.h`
const VREPLACE_FLAG: c_int = 0x200;

// ============================================================================
// ins_tab_replace_spaces implementation (ported from nvim_edit_ins_tab_replace_spaces)
// ============================================================================

/// Replace spaces with TABs in the current line after TAB expansion.
///
/// This is the space-to-TAB optimisation that runs when 'expandtab' is off
/// but softtabstop / vartabstop / smarttab caused us to insert spaces.
///
/// Mirrors the C `nvim_edit_ins_tab_replace_spaces` function that was formerly
/// in `edit.c`.  Raw memline manipulation is delegated to
/// `nvim_edit_tab_rewrite_line` (in `edit_shim.c`).
///
/// # Safety
/// Accesses global Neovim state via C helpers.
#[allow(clippy::too_many_lines)]
unsafe fn ins_tab_replace_spaces_impl(_p_sta_val: bool, _ind: bool) {
    let vreplace = nvim_edit_tab_is_vreplace();
    let replace_mode = nvim_edit_tab_is_replace();

    // Obtain a pointer to the region to scan.
    // For VREPLACE we save the line first and work on the copy;
    // otherwise we work directly in the line buffer.
    let mut cursor_lnum: LinenrT = 0;
    let mut cursor_col: ColnrT = 0;
    nvim_edit_tab_get_cursor(&raw mut cursor_lnum, &raw mut cursor_col);

    let (saved_line, mut ptr) = if vreplace {
        let sl = nvim_edit_tab_strnsave_cursor_line();
        let p = sl.add(cursor_col as usize);
        (sl, p)
    } else {
        (std::ptr::null_mut(), nvim_edit_tab_get_cursor_pos_ptr())
    };

    // Save and optionally clear 'list' (CPO_LISTWM check is inside the shim).
    let save_list = nvim_edit_tab_save_list();

    // Build fpos = curwin->w_cursor; then walk back over whitespace.
    let fpos_lnum: LinenrT = cursor_lnum;
    let mut fpos_col: ColnrT = cursor_col;

    while fpos_col > 0 && nvim_ascii_iswhite(*ptr.offset(-1)) {
        fpos_col -= 1;
        ptr = ptr.offset(-1);
    }

    // In REPLACE mode, don't back up past Insstart.col.
    let mut insstart_lnum: LinenrT = 0;
    let mut insstart_col: ColnrT = 0;
    nvim_edit_tab_get_Insstart(&raw mut insstart_lnum, &raw mut insstart_col);
    if replace_mode && fpos_lnum == insstart_lnum && fpos_col < insstart_col {
        ptr = ptr.add((insstart_col - fpos_col) as usize);
        fpos_col = insstart_col;
    }

    let vcol_start = nvim_edit_tab_getvcol(fpos_lnum, fpos_col);
    let vcol_want = nvim_edit_tab_getvcol(cursor_lnum, cursor_col);

    let mut change_col: c_int = -1;
    let mut vcol = vcol_start;

    // Replace spaces with TABs where they fit.
    while nvim_ascii_iswhite(*ptr) {
        let tab_width = nvim_edit_tab_charsize_tab(vcol);
        if vcol + tab_width > vcol_want {
            break;
        }
        if *ptr != b'\t' as c_char {
            *ptr = b'\t' as c_char;
            if change_col < 0 {
                change_col = fpos_col as c_int;
                if fpos_lnum == insstart_lnum && fpos_col < insstart_col {
                    nvim_edit_tab_set_Insstart_col(fpos_col);
                }
            }
        }
        fpos_col += 1;
        ptr = ptr.offset(1);
        vcol += tab_width;
    }

    if change_col >= 0 {
        // Skip remaining spaces up to want_vcol.
        let mut repl_off: c_int = 0;
        while vcol < vcol_want && *ptr == b' ' as c_char {
            let space_width = nvim_edit_tab_charsize_space(vcol, ptr);
            vcol += space_width;
            ptr = ptr.offset(1);
            repl_off += 1;
        }
        if vcol > vcol_want {
            ptr = ptr.offset(-1);
            repl_off -= 1;
        }
        fpos_col += repl_off as ColnrT;

        let i: c_int = cursor_col - fpos_col;
        if i > 0 {
            if vreplace {
                nvim_edit_tab_strmove(ptr, i);
            } else {
                // Raw memline rewrite (xmalloc/memmove/xfree) – done in C shim.
                nvim_edit_tab_rewrite_line(
                    ptr,
                    i,
                    change_col as ColnrT,
                    cursor_col,
                    fpos_col,
                    fpos_lnum,
                );
                // replace_join for each deleted byte in REPLACE mode.
                if replace_mode {
                    let mut temp = i;
                    while temp > 0 {
                        nvim_edit_tab_replace_join(repl_off);
                        temp -= 1;
                    }
                }
            }
        }
        nvim_edit_tab_set_cursor_col(cursor_col - i as ColnrT);

        if vreplace {
            let new_col = nvim_edit_tab_cursor_col(std::ptr::null_mut());
            nvim_edit_tab_backspace_until_column(change_col as ColnrT);
            nvim_edit_tab_ins_bytes_len(
                saved_line.add(change_col as usize),
                (new_col - change_col as ColnrT) as usize,
            );
        }
    }

    if vreplace {
        xfree(saved_line.cast::<std::ffi::c_void>());
    }
    nvim_edit_tab_restore_list(save_list);
}

// ============================================================================
// ins_tab implementation
// ============================================================================

/// Handle TAB in Insert or Replace mode.
///
/// Returns true when the TAB needs to be inserted like a normal character.
///
/// # Safety
/// Calls numerous C functions that access global state (curwin, curbuf, etc.)
unsafe fn ins_tab_impl() -> bool {
    // If Insstart_blank_vcol hasn't been set yet, record current virtual column.
    if nvim_get_Insstart_blank_vcol() == MAXCOL
        && nvim_curwin_get_cursor_lnum() == nvim_get_Insstart_lnum()
    {
        nvim_set_Insstart_blank_vcol(get_nolist_virtcol());
    }

    // Check for abbreviation
    if echeck_abbr(TAB + ABBR_OFF) != 0 {
        return false;
    }

    let ind = inindent(0) != 0;
    if ind {
        nvim_set_can_cindent(0);
    }

    // When nothing special, insert TAB like a normal character.
    // Conditions: expandtab is off AND smarttab/softtabstop don't apply.
    let et = nvim_curbuf_get_b_p_et() != 0;
    let sta = p_sta != 0;
    let vts_count = nvim_curbuf_tabstop_count_vts();
    // Use a distinct name to avoid the "similar binding" warning
    let vsts_cnt = nvim_curbuf_tabstop_count_vsts();
    let sw_value = nvim_curbuf_get_sw_value();
    let ts = nvim_curbuf_get_b_p_ts();
    let sts_value = nvim_get_sts_value();

    // Equivalent to C:
    //   `!b_p_et`
    //   `&& !(p_sta && ind && (tabstop != shiftwidth))`
    //   `&& vsts_count == 0`
    //   `&& sts == 0`
    let sta_override = sta
        && ind
        && ((vts_count > 1)
            || (vts_count == 1 && nvim_curbuf_tabstop_first_vts() != sw_value)
            || (vts_count == 0 && c_long::from(ts) != sw_value));

    if !et && !sta_override && vsts_cnt == 0 && sts_value == 0 {
        return true;
    }

    if stop_arrow() == FAIL {
        return true;
    }

    nvim_set_did_ai(false);
    nvim_set_did_si(false);
    nvim_set_can_si(false);
    nvim_set_can_si_back(false);
    AppendToRedobuff(c"\t".as_ptr());

    // Determine how many spaces to insert
    let temp: c_int = if sta && ind {
        // Use shiftwidth
        let sw = sw_value as c_int;
        let vcol = get_nolist_virtcol() as c_int;
        sw - (vcol % sw)
    } else if vsts_cnt > 0 || nvim_curbuf_get_b_p_sts() != 0 {
        // Use softtabstop
        nvim_curbuf_tabstop_padding_sts()
    } else {
        // Use tabstop
        nvim_curbuf_tabstop_padding_ts()
    };

    // Insert spaces (first via ins_char, rest via ins_str or ins_char for VREPLACE).
    // ins_char handles replace mode (deletes one char).
    // ins_str does not delete chars.
    let state = State;
    ins_char(c_int::from(b' '));
    let mut remaining = temp - 1;
    while remaining > 0 {
        if state & VREPLACE_FLAG != 0 {
            ins_char(c_int::from(b' '));
        } else {
            nvim_ins_str(c" ".as_ptr(), 1);
            if state & REPLACE_FLAG != 0 {
                replace_push_nul();
            }
        }
        remaining -= 1;
    }

    // When 'expandtab' not set: replace spaces with TABs where possible.
    if !et && (vsts_cnt > 0 || sts_value > 0 || (sta && ind)) {
        ins_tab_replace_spaces_impl(sta, ind);
    }

    false
}

/// Handle TAB in Insert or Replace mode.
///
/// Returns true when the TAB needs to be inserted like a normal character
/// (i.e. no special handling needed).
///
/// # Safety
/// Accesses global Neovim state.
#[must_use]
#[unsafe(export_name = "ins_tab")]
pub unsafe extern "C" fn rs_ins_tab() -> bool {
    ins_tab_impl()
}
