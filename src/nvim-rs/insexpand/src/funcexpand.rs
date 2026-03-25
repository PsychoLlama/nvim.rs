//! VimL completion function expansion (Phase 9 of insexpand migration).
//!
//! This module provides Rust wrappers for:
//! - `expand_by_function`: call user completefunc/omnifunc/thesaurusfunc
//! - `ins_compl_add_tv`: parse a typval_T into a completion match
//! - `ins_compl_add_list`: iterate a VimL list and add completions
//! - `ins_compl_add_dict`: extract "words"/"refresh" from a dict
//! - `set_completion`: orchestrate complete() builtin
//! - `f_complete`, `f_complete_add`, `f_complete_check`, `f_preinserted`
//! - `cpt_compl_refresh`, `remove_old_matches`, `get_callback_if_cpt_func`
//!
//! All functions that touch opaque C types (typval_T, dict_T, list_T,
//! Callback, compl_T) delegate to compound C accessors following the
//! established pattern in the insexpand crate.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

// =============================================================================
// Opaque pointer types
// =============================================================================

/// Opaque pointer to typval_T
type TypvalPtr = *mut c_void;

/// Opaque pointer to list_T
type ListPtr = *mut c_void;

/// Opaque pointer to dict_T
type DictPtr = *mut c_void;

/// Opaque pointer to Callback
type CallbackPtr = *mut c_void;

// =============================================================================
// Phase 1: VimL Completion Function Expansion
// =============================================================================

extern "C" {
    // Compound C accessors (contain the actual logic moved from static C fns)
    fn nvim_expand_by_function_full_impl(type_: c_int, base: *mut c_char, cb: CallbackPtr);
    fn nvim_ins_compl_add_tv_impl(tv: TypvalPtr, dir: c_int, fast: c_int) -> c_int;
    fn nvim_ins_compl_add_list_impl(list: ListPtr);
    fn nvim_ins_compl_add_dict_impl(dict: DictPtr);
}

/// Execute user-defined completion function and collect matches.
///
/// Calls completefunc/omnifunc/thesaurusfunc (findstart=0) and dispatches
/// the list/dict result to `rs_ins_compl_add_list` / `rs_ins_compl_add_dict`.
///
/// # Safety
/// Requires valid global completion state and a live C `Callback *` if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_expand_by_function(type_: c_int, base: *mut c_char, cb: CallbackPtr) {
    nvim_expand_by_function_full_impl(type_, base, cb);
}

/// Add a completion match from a VimL typval_T value.
///
/// Parses a string or dict typval and calls `ins_compl_add`.
///
/// Returns `OK`, `NOTDONE`, or `FAIL` (matching C return conventions).
///
/// # Safety
/// `tv` must be a valid `typval_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_add_tv(tv: TypvalPtr, dir: c_int, fast: c_int) -> c_int {
    nvim_ins_compl_add_tv_impl(tv, dir, fast)
}

/// Iterate a VimL list and add each item as a completion match.
///
/// # Safety
/// `list` must be a valid `list_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_add_list(list: ListPtr) {
    nvim_ins_compl_add_list_impl(list);
}

/// Extract `refresh` and `words` from a VimL dict and add completion matches.
///
/// # Safety
/// `dict` must be a valid `dict_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_add_dict(dict: DictPtr) {
    nvim_ins_compl_add_dict_impl(dict);
}

// =============================================================================
// Phase 2: VimL Builtin Functions
// =============================================================================

extern "C" {
    // Compound C accessors for all Phase 2 functions
    fn nvim_f_complete_impl(argvars: TypvalPtr, rettv: TypvalPtr);
    fn nvim_f_complete_add_impl(argvars: TypvalPtr, rettv: TypvalPtr);
    fn nvim_f_complete_check_impl(rettv: TypvalPtr);
    fn nvim_f_preinserted_impl(rettv: TypvalPtr);
}

/// VimL `complete_check()` builtin.
///
/// Saves/restores `RedrawingDisabled`, calls `rs_ins_compl_check_keys`,
/// and sets the return value to the interrupted flag.
///
/// # Safety
/// Requires valid global completion state.
#[export_name = "f_complete_check"]
pub unsafe extern "C" fn rs_f_complete_check(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: *mut c_void,
) {
    nvim_f_complete_check_impl(rettv);
}

/// VimL `preinserted()` builtin.
///
/// Returns 1 if the pre-insert effect is currently active.
///
/// # Safety
/// Requires valid global completion state.
#[export_name = "f_preinserted"]
pub unsafe extern "C" fn rs_f_preinserted(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: *mut c_void,
) {
    nvim_f_preinserted_impl(rettv);
}

/// VimL `complete()` builtin.
///
/// # Safety
/// `argvars` must be a valid `typval_T[2]` pointer; `rettv` a `typval_T*`.
#[export_name = "f_complete"]
pub unsafe extern "C" fn rs_f_complete(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: *mut c_void) {
    nvim_f_complete_impl(argvars, rettv);
}

/// VimL `complete_add()` builtin.
///
/// # Safety
/// `argvars` must be a valid `typval_T[1]` pointer; `rettv` a `typval_T*`.
#[export_name = "f_complete_add"]
pub unsafe extern "C" fn rs_f_complete_add(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: *mut c_void,
) {
    nvim_f_complete_add_impl(argvars, rettv);
}

// =============================================================================
// Phase 3: set_completion Orchestration
// =============================================================================

// nvim_set_completion_impl: deleted (Phase 24), inlined below as rs_set_completion

extern "C" {
    // helpers for inlined rs_set_completion (Phase 24)
    fn rs_ctrl_x_mode_not_default() -> c_int;
    fn rs_ins_compl_prep(c: c_int) -> c_int;
    fn rs_ins_compl_clear();
    fn rs_ins_compl_free();
    fn rs_get_cot_flags() -> u32;
    fn rs_save_orig_extmarks();
    fn nvim_ins_compl_add_simple(
        str_: *const c_char,
        len: c_int,
        dir: c_int,
        flags: c_int,
        score: c_int,
    ) -> c_int;
    #[link_name = "cbuf_to_string"]
    fn cbuf_to_string_set_completion(buf: *const c_char, size: usize) -> crate::vars::NvimString;
    #[link_name = "get_cursor_line_ptr"]
    fn get_cursor_line_ptr_set_completion() -> *mut c_char;
    #[link_name = "nvim_get_curwin_cursor_lnum"]
    fn nvim_get_curwin_cursor_lnum_sc() -> c_int;
    fn nvim_get_curwin_w_wrow() -> c_int;
    fn nvim_get_curwin_w_leftcol() -> c_int;
    fn ins_complete(c: c_int, enable_pum: c_int) -> c_int;
    fn rs_show_pum(prev_w_wrow: c_int, prev_w_leftcol: c_int);
    fn may_trigger_modechanged();
    #[link_name = "ui_flush"]
    fn nvim_ui_flush_set_completion();
}

// Flags for ins_compl_add
const CP_ORIGINAL_TEXT_SC: c_int = 1;
const CP_ICASE_SC: c_int = 16;
const CP_FAST_SC: c_int = 32;

// kOptCotFlag values
const COT_LONGEST_SC: u32 = 0x04;
const COT_NOINSERT_SC: u32 = 0x20;
const COT_NOSELECT_SC: u32 = 0x40;

// CTRL_X mode
const CTRL_X_EVAL_SC: c_int = 16;
// Direction
const FORWARD_SC: c_int = 1;
// Key codes
const K_DOWN_SC: c_int = -25707;
const K_UP_SC: c_int = -30059;
const CTRL_N_SC: c_int = 14;
// FUZZY_SCORE_NONE = INT_MIN
const FUZZY_SCORE_NONE_SC: c_int = c_int::MIN;

/// Orchestrate the `complete()` VimL function: clear state, add original
/// text and list matches, start completion, show popup menu.
///
/// Rust translation of nvim_set_completion_impl (Phase 24).
///
/// # Safety
/// `list` must be a valid `list_T *` pointer.
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_set_completion(startcol: c_int, list: ListPtr) {
    let mut startcol = startcol;

    let cur_cot_flags = rs_get_cot_flags();
    let compl_longest = (cur_cot_flags & COT_LONGEST_SC) != 0;
    let compl_no_insert = (cur_cot_flags & COT_NOINSERT_SC) != 0;
    let compl_no_select = (cur_cot_flags & COT_NOSELECT_SC) != 0;

    // If already doing completions stop it.
    if rs_ctrl_x_mode_not_default() != 0 {
        rs_ins_compl_prep(c_int::from(b' '));
    }
    rs_ins_compl_clear();
    rs_ins_compl_free();
    crate::vars::nvim_set_compl_get_longest(c_int::from(compl_longest));

    crate::vars::nvim_set_compl_direction(FORWARD_SC);

    // Clamp startcol to cursor column
    let cur_col = nvim_get_cursor_col();
    if startcol > cur_col {
        startcol = cur_col;
    }
    crate::vars::nvim_set_compl_col(startcol);
    crate::vars::nvim_set_compl_lnum(nvim_get_curwin_cursor_lnum_sc());
    crate::vars::nvim_set_compl_length(cur_col - startcol);

    // Set compl_orig_text from the line
    let line = get_cursor_line_ptr_set_completion();
    let col_usize = startcol as usize;
    let len_usize = crate::vars::nvim_get_compl_length() as usize;
    crate::vars::compl_orig_text =
        cbuf_to_string_set_completion(line.add(col_usize).cast_const(), len_usize);

    rs_save_orig_extmarks();

    let mut flags = CP_ORIGINAL_TEXT_SC;
    if crate::vars::nvim_get_p_ic() != 0 {
        flags |= CP_ICASE_SC;
    }

    let add_result = nvim_ins_compl_add_simple(
        crate::vars::compl_orig_text.data.cast_const(),
        crate::vars::compl_orig_text.size as c_int,
        0,
        flags | CP_FAST_SC,
        FUZZY_SCORE_NONE_SC,
    );
    if add_result != OK_FUNCEXPAND {
        return;
    }

    crate::vars::nvim_set_ctrl_x_mode(CTRL_X_EVAL_SC);

    // Add all matches from the list
    nvim_ins_compl_add_list_impl(list);

    crate::vars::nvim_set_compl_matches(rs_ins_compl_make_cyclic());
    crate::vars::nvim_set_compl_started(1);
    crate::vars::nvim_set_compl_used_match(1);
    crate::vars::nvim_set_compl_cont_status(0);

    let save_w_wrow = nvim_get_curwin_w_wrow();
    let save_w_leftcol = nvim_get_curwin_w_leftcol();

    crate::match_list::compl_curr_match = crate::match_list::compl_first_match;

    let no_select = compl_no_select || compl_longest;
    if compl_no_insert || no_select {
        ins_complete(K_DOWN_SC, 0);
        if no_select {
            ins_complete(K_UP_SC, 0);
        }
    } else {
        ins_complete(CTRL_N_SC, 0);
    }
    crate::vars::nvim_set_compl_enter_selects(c_int::from(compl_no_insert));

    if crate::vars::nvim_get_compl_interrupted() == 0 {
        rs_show_pum(save_w_wrow, save_w_leftcol);
    }

    may_trigger_modechanged();
    nvim_ui_flush_set_completion();
}

// =============================================================================
// Phase 4: Refresh Orchestration
// =============================================================================

// nvim_cpt_compl_refresh_impl: deleted (Phase 2), inlined below as rs_cpt_compl_refresh
extern "C" {
    fn nvim_get_callback_if_cpt_func_impl(p: *const c_char, idx: c_int) -> CallbackPtr;

    // helpers for inlined rs_cpt_compl_refresh
    fn nvim_curbuf_get_b_p_cpt() -> *const c_char;
    #[link_name = "xstrdup"]
    fn xstrdup_funcexpand(s: *const c_char) -> *mut c_char;
    #[link_name = "xfree"]
    fn xfree_funcexpand(p: *mut u8);
    fn copy_option_part(
        pp: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;
    fn rs_strip_caret_numbers_in_place(s: *mut c_char);
    fn rs_ins_compl_make_linear();
    fn rs_ins_compl_make_cyclic() -> c_int;
    fn rs_remove_old_matches();
    fn rs_get_userdefined_compl_info(
        col: c_int,
        cb: CallbackPtr,
        startcol_out: *mut c_int,
    ) -> c_int;
    fn rs_compl_source_start_timer(idx: c_int);
    fn rs_get_cpt_func_completion_matches(cb: CallbackPtr);
    fn rs_advance_cpt_sources_index_safe() -> c_int;
    fn rs_may_advance_cpt_index(cpt: *mut c_char) -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    #[link_name = "IObuff"]
    static mut IObuff_funcexpand: [std::ffi::c_char; 1025];
}

const IOSIZE_FUNCEXPAND: usize = 1025;
const FAIL_FUNCEXPAND: c_int = 0;
const OK_FUNCEXPAND: c_int = 1;

/// Refresh completion matches from 'cpt' function sources with `refresh:always`.
///
/// Rust translation of nvim_cpt_compl_refresh_impl (Phase 2).
///
/// # Safety
/// Requires valid global completion state and a live current buffer.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
#[no_mangle]
pub unsafe extern "C" fn rs_cpt_compl_refresh() {
    rs_ins_compl_make_linear();
    // Make a copy of 'cpt' in case the buffer gets wiped out
    let cpt = xstrdup_funcexpand(nvim_curbuf_get_b_p_cpt());
    rs_strip_caret_numbers_in_place(cpt);

    crate::vars::nvim_set_cpt_sources_index(0);
    let mut p = cpt;
    loop {
        // Skip delimiters
        while *p == b',' as std::ffi::c_char || *p == b' ' as std::ffi::c_char {
            p = p.add(1);
        }
        if *p == 0 {
            break;
        }

        let idx = crate::vars::nvim_get_cpt_sources_index();
        if crate::vars::nvim_cpt_sources_get_refresh_always(idx) != 0 {
            let cb = nvim_get_callback_if_cpt_func_impl(p, idx);
            if !cb.is_null() {
                rs_remove_old_matches();
                let mut startcol: c_int = 0;
                let ret =
                    rs_get_userdefined_compl_info(nvim_get_cursor_col(), cb, &raw mut startcol);
                if ret == FAIL_FUNCEXPAND {
                    if startcol == -3 {
                        crate::vars::nvim_cpt_sources_set_refresh_always(idx, 0);
                    } else {
                        startcol = -2;
                    }
                } else if startcol < 0 || startcol > nvim_get_cursor_col() {
                    startcol = nvim_get_cursor_col();
                }
                crate::vars::nvim_cpt_sources_set_startcol(idx, startcol);
                if ret == OK_FUNCEXPAND {
                    rs_compl_source_start_timer(idx);
                    rs_get_cpt_func_completion_matches(cb);
                }
            }
        }

        copy_option_part(
            &raw mut p,
            core::ptr::addr_of_mut!(IObuff_funcexpand).cast(),
            IOSIZE_FUNCEXPAND,
            c",".as_ptr(),
        );
        if rs_may_advance_cpt_index(p) != 0 {
            rs_advance_cpt_sources_index_safe();
        }
    }
    crate::vars::nvim_set_cpt_sources_index(-1);

    xfree_funcexpand(cpt.cast());
    // Make the list cyclic
    crate::vars::nvim_set_compl_matches(rs_ins_compl_make_cyclic());
}

/// Return the `Callback *` for a 'cpt' option entry if it is a function source.
///
/// Returns `NULL` if the entry at `p` is not an `o` or `F` function entry.
///
/// # Safety
/// `p` must be a valid C string pointer to the current position in the 'cpt' option.
#[no_mangle]
pub unsafe extern "C" fn rs_get_callback_if_cpt_func(p: *const c_char, idx: c_int) -> CallbackPtr {
    nvim_get_callback_if_cpt_func_impl(p, idx)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_phase_constants() {
        // Ensure module compiles
        assert_eq!(1, 1);
    }
}
