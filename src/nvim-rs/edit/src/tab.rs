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
    fn nvim_edit_AppendToRedobuff(str: *const c_char);

    // stop_arrow
    fn stop_arrow() -> c_int;

    // State (REPLACE_FLAG / VREPLACE_FLAG)
    fn nvim_get_State() -> c_int;

    // p_sta (smarttab) -- from option_shim.c
    fn nvim_get_p_sta() -> c_int;

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
    fn nvim_edit_ins_str(s: *const c_char, len: usize);
    fn replace_push_nul();

    // Space-to-TAB replacement (complex C helper)
    fn nvim_edit_ins_tab_replace_spaces(p_sta_val: bool, ind: bool) -> bool;
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
    let sta = nvim_get_p_sta() != 0;
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
    nvim_edit_AppendToRedobuff(c"\t".as_ptr());

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
    let state = nvim_get_State();
    ins_char(c_int::from(b' '));
    let mut remaining = temp - 1;
    while remaining > 0 {
        if state & VREPLACE_FLAG != 0 {
            ins_char(c_int::from(b' '));
        } else {
            nvim_edit_ins_str(c" ".as_ptr(), 1);
            if state & REPLACE_FLAG != 0 {
                replace_push_nul();
            }
        }
        remaining -= 1;
    }

    // When 'expandtab' not set: replace spaces with TABs where possible.
    // This uses a complex C helper due to direct memline access.
    if !et && (vsts_cnt > 0 || sts_value > 0 || (sta && ind)) {
        nvim_edit_ins_tab_replace_spaces(sta, ind);
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
