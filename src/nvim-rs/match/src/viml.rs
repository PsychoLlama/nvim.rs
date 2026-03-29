//! Vimscript function helpers for match operations
//!
//! This module provides helpers for the Vimscript functions:
//! - `matchadd()` - Add a match with a pattern
//! - `matchaddpos()` - Add a match with positions
//! - `matchdelete()` - Delete a match by ID
//! - `matcharg()` - Get match info for `:match` commands
//! - `getmatches()` - Get list of all matches
//! - `setmatches()` - Restore matches from a list
//! - `clearmatches()` - Clear all matches
//!
//! Note: The actual typval handling stays in C; this module provides
//! validation and computation helpers.

use std::ffi::c_int;

use crate::add::{is_reserved_id, is_valid_matchadd_id, is_valid_matchaddpos_id};
use crate::{MATCH_ID_1, MATCH_ID_2, MATCH_ID_3, MIN_MATCH_ID};

// =============================================================================
// `matchadd()` helpers
// =============================================================================

/// Validate arguments for `matchadd()`.
///
/// Returns 0 if valid, negative error code otherwise.
/// Priority is accepted for API compatibility but not validated (any value is valid).
#[must_use]
pub fn validate_matchadd_args(id: i32, _priority: i32) -> i32 {
    // Priority can be any value, no validation needed

    // ID validation
    if !is_valid_matchadd_id(id) {
        if is_reserved_id(id) {
            return -1; // Reserved ID error
        }
        return -2; // Invalid ID error
    }

    0
}

/// Check if an ID is valid for `matchadd()` (excluding reserved 1, 2, 3).
#[must_use]
pub const fn matchadd_id_valid(id: i32) -> bool {
    id == -1 || id >= 4
}

// =============================================================================
// `matchaddpos()` helpers
// =============================================================================

/// Validate arguments for `matchaddpos()`.
///
/// Returns 0 if valid, negative error code otherwise.
#[must_use]
pub fn validate_matchaddpos_args(id: i32, _priority: i32) -> i32 {
    // ID validation - 3 is allowed for matchaddpos (substitutes :3match)
    if !is_valid_matchaddpos_id(id) {
        if id == MATCH_ID_1 || id == MATCH_ID_2 {
            return -1; // Reserved ID error
        }
        return -2; // Invalid ID error
    }

    0
}

/// Check if an ID is valid for `matchaddpos()` (allows 3, excludes 1, 2).
#[must_use]
pub const fn matchaddpos_id_valid(id: i32) -> bool {
    id == -1 || id == MATCH_ID_3 || id >= 4
}

// =============================================================================
// `matchdelete()` helpers
// =============================================================================

/// Validate arguments for `matchdelete()`.
///
/// Returns 0 if valid, negative error code otherwise.
#[must_use]
pub fn validate_matchdelete_args(id: i32) -> i32 {
    if id < MIN_MATCH_ID {
        return -1; // Invalid ID
    }
    0
}

// =============================================================================
// `matcharg()` helpers
// =============================================================================

/// Check if an ID is valid for `matcharg()` (1, 2, or 3).
#[must_use]
pub const fn is_matcharg_valid_id(id: i32) -> bool {
    id >= MATCH_ID_1 && id <= MATCH_ID_3
}

/// Get the number of return list items for `matcharg()`.
///
/// Returns 2 if ID is valid (1-3), 0 otherwise.
#[must_use]
pub const fn matcharg_result_len(id: i32) -> i32 {
    if is_matcharg_valid_id(id) {
        2
    } else {
        0
    }
}

// =============================================================================
// `getmatches()` / `setmatches()` helpers
// =============================================================================

/// Maximum number of position entries in a match (pos1..pos8).
pub const MAX_POS_ENTRIES: i32 = 8;

/// Check if a position key index is valid (1-8).
#[must_use]
pub const fn is_valid_pos_key_index(idx: i32) -> bool {
    idx >= 1 && idx <= MAX_POS_ENTRIES
}

/// Generate a position key name for the given index (1-based).
///
/// Returns "pos1", "pos2", etc.
#[must_use]
pub fn pos_key_name(idx: i32) -> Option<&'static str> {
    match idx {
        1 => Some("pos1"),
        2 => Some("pos2"),
        3 => Some("pos3"),
        4 => Some("pos4"),
        5 => Some("pos5"),
        6 => Some("pos6"),
        7 => Some("pos7"),
        8 => Some("pos8"),
        _ => None,
    }
}

/// Required keys for a valid match dict in `setmatches()`.
pub const REQUIRED_KEYS: &[&str] = &["group", "priority", "id"];

/// Either "pattern" or "pos1" must be present.
pub const PATTERN_OR_POS_KEYS: (&str, &str) = ("pattern", "pos1");

/// Check if a dict has the required keys for `setmatches()`.
///
/// Requires: group, priority, id, and either pattern or pos1.
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub fn has_required_match_keys(
    has_group: bool,
    has_priority: bool,
    has_id: bool,
    has_pattern: bool,
    has_pos1: bool,
) -> bool {
    has_group && has_priority && has_id && (has_pattern || has_pos1)
}

// =============================================================================
// `:match` command helpers
// =============================================================================

/// Validate the line number for `:match` command (1, 2, or 3).
#[must_use]
pub const fn is_valid_match_cmd_line(line: i64) -> bool {
    line >= 1 && line <= 3
}

/// Convert `:match` command line number to match ID.
#[must_use]
pub const fn match_cmd_line_to_id(line: i64) -> i32 {
    line as i32
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Validate `matchadd()` arguments.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matchadd_validate_args(id: c_int, priority: c_int) -> c_int {
    validate_matchadd_args(id, priority)
}

/// Check if ID is valid for `matchadd()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matchadd_id_valid(id: c_int) -> c_int {
    c_int::from(matchadd_id_valid(id))
}

/// Validate `matchaddpos()` arguments.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matchaddpos_validate_args(id: c_int, priority: c_int) -> c_int {
    validate_matchaddpos_args(id, priority)
}

/// Check if ID is valid for `matchaddpos()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matchaddpos_id_valid(id: c_int) -> c_int {
    c_int::from(matchaddpos_id_valid(id))
}

/// Validate `matchdelete()` arguments.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matchdelete_validate_args(id: c_int) -> c_int {
    validate_matchdelete_args(id)
}

/// Check if ID is valid for `matcharg()` (1, 2, or 3).
#[unsafe(no_mangle)]
pub extern "C" fn rs_matcharg_valid_id(id: c_int) -> c_int {
    c_int::from(is_matcharg_valid_id(id))
}

/// Get result list length for `matcharg()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_matcharg_result_len(id: c_int) -> c_int {
    matcharg_result_len(id)
}

/// Check if position key index is valid (1-8).
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_pos_key_index(idx: c_int) -> c_int {
    c_int::from(is_valid_pos_key_index(idx))
}

/// Get max position entries (8).
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_max_pos_entries() -> c_int {
    MAX_POS_ENTRIES
}

/// Check if dict has required keys for `setmatches()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_has_required_keys(
    has_group: c_int,
    has_priority: c_int,
    has_id: c_int,
    has_pattern: c_int,
    has_pos1: c_int,
) -> c_int {
    c_int::from(has_required_match_keys(
        has_group != 0,
        has_priority != 0,
        has_id != 0,
        has_pattern != 0,
        has_pos1 != 0,
    ))
}

/// Check if `:match` command line number is valid (1, 2, or 3).
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_cmd_line_valid(line: i64) -> c_int {
    c_int::from(is_valid_match_cmd_line(line))
}

/// Convert `:match` command line to match ID.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_cmd_line_to_id(line: i64) -> c_int {
    match_cmd_line_to_id(line)
}

// =============================================================================
// VimL f_* function implementations
// =============================================================================

/// Opaque handle to a `win_T` (re-use from core to avoid type mismatches)
use crate::core::WinHandle;

/// Opaque pointer to `typval_T`
type TypvalPtr = *mut std::ffi::c_void;

/// Opaque pointer to a list (`list_T`)
type ListPtr = *mut std::ffi::c_void;

/// Opaque pointer to a list item (`listitem_T`)
type ListItemPtr = *mut std::ffi::c_void;

/// `EvalFuncData` (unused callback data, 8-byte union passed by value as pointer)
type EvalFuncData = *mut std::ffi::c_void;

/// `VAR_UNKNOWN` = 0, `VAR_NUMBER` = 1, `VAR_LIST` = 4, `VAR_DICT` = 5
const VAR_UNKNOWN: c_int = 0;
const VAR_NUMBER: c_int = 1;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;

/// Size of number conversion buffer (NUMBUFLEN = 65)
const NUMBUFLEN: usize = 65;

extern "C" {
    fn get_optional_window(argvars: TypvalPtr, idx: c_int) -> *mut WinHandle;
    fn nvim_tv_get_number(tv: TypvalPtr) -> i64;
    fn nvim_tv_set_number(tv: TypvalPtr, n: i64);
    fn nvim_tv_get_type(tv: TypvalPtr) -> c_int;
    fn nvim_tv_get_list(tv: TypvalPtr) -> ListPtr;
    fn nvim_tv_get_number_chk(tv: TypvalPtr, error: *mut bool) -> i64;
    fn nvim_tv_get_string_buf_chk(
        tv: TypvalPtr,
        buf: *mut std::ffi::c_char,
    ) -> *const std::ffi::c_char;
    fn nvim_tv_idx(argvars: TypvalPtr, i: c_int) -> TypvalPtr;
    fn nvim_list_get_first(l: ListPtr) -> ListItemPtr;
    fn nvim_listitem_get_next(li: ListItemPtr) -> ListItemPtr;
    fn nvim_listitem_get_tv(li: ListItemPtr) -> TypvalPtr;
    fn nvim_tv_list_len(l: ListPtr) -> c_int;
    fn tv_dict_find(
        dict: *mut std::ffi::c_void,
        key: *const std::ffi::c_char,
        len: c_int,
    ) -> *mut std::ffi::c_void;
    fn tv_get_string(tv: TypvalPtr) -> *const std::ffi::c_char;
    fn find_win_by_nr_or_id(tv: TypvalPtr) -> *mut WinHandle;
    fn clear_matches(wp: *mut WinHandle);
    fn match_delete(wp: *mut WinHandle, id: c_int, perr: bool) -> c_int;
    fn emsg(s: *const std::ffi::c_char);
    fn semsg(fmt: *const std::ffi::c_char, ...);
    fn nvim_get_curwin() -> *mut WinHandle;
    // ex_match helpers
    fn nvim_eap_get_line2(eap: *const std::ffi::c_void) -> i32;
    fn nvim_eap_get_skip(eap: *const std::ffi::c_void) -> c_int;
    fn nvim_eap_get_arg(eap: *const std::ffi::c_void) -> *mut std::ffi::c_char;
    fn nvim_eap_set_nextcmd(eap: *mut std::ffi::c_void, p: *mut std::ffi::c_char);
    fn nvim_eap_set_errmsg_const(eap: *mut std::ffi::c_void, msg: *const std::ffi::c_char);
    fn nvim_docmd_errmsg_trailing_arg(arg: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn nvim_excmds_emsg_by_id(id: c_int);
    fn ends_excmd(c: c_int) -> c_int;
    fn skiptowhite(p: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn skipwhite(p: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn skip_regexp(p: *mut std::ffi::c_char, delim: c_int, magic: c_int) -> *mut std::ffi::c_char;
    fn find_nextcmd(p: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn xmemdupz(data: *const std::ffi::c_void, len: usize) -> *mut std::ffi::c_char;
    fn xfree(ptr: *mut std::ffi::c_void);
    static e_dictreq: [std::ffi::c_char; 0];
    static e_listarg: [std::ffi::c_char; 0];
    static e_invarg2: [std::ffi::c_char; 0];
}

// =============================================================================
// Phase 1 VimL f_* implementations
// =============================================================================

/// Helper that extracts optional conceal char and window from a dict arg.
///
/// Mirrors C `matchadd_dict_arg`.
///
/// `dictitem_T` layout: `{ typval_T di_tv; uint8_t di_flags; char di_key[]; }`.
/// So `&di->di_tv` is at offset 0 -- a `dictitem_T *` can be cast directly
/// to `typval_T *` (i.e. `TypvalPtr`).
///
/// `typval_T` layout: `{ int v_type; int v_lock; union vval; }` (16 bytes on
/// 64-bit). `v_dict` is the pointer in `vval`, at offset 8.
///
/// # Safety
///
/// `tv` must be a valid `typval_T *`. `conceal_char` and `win` must be valid.
unsafe fn matchadd_dict_arg_impl(
    tv: TypvalPtr,
    conceal_char: *mut *const std::ffi::c_char,
    win: *mut *mut WinHandle,
) -> c_int {
    const FAIL: c_int = 0;
    const OK: c_int = 1;

    if nvim_tv_get_type(tv) != VAR_DICT {
        emsg(e_dictreq.as_ptr());
        return FAIL;
    }

    // typval_T: { int v_type (4) + int v_lock (4) + union vval (8) }
    // v_dict is a pointer stored in vval at offset 8.
    #[allow(clippy::cast_ptr_alignment)]
    let dict_ptr: *mut std::ffi::c_void =
        std::ptr::read_unaligned(tv.cast::<u8>().add(8).cast::<*mut std::ffi::c_void>());

    let conceal_key = c"conceal".as_ptr();
    let window_key = c"window".as_ptr();

    // tv_dict_find returns dictitem_T*, whose first field is typval_T di_tv.
    // So the returned pointer is directly usable as TypvalPtr.
    let di_conceal = tv_dict_find(dict_ptr, conceal_key, 7);
    if !di_conceal.is_null() {
        // di_conceal points to dictitem_T; di_tv is at offset 0, so cast directly.
        *conceal_char = tv_get_string(di_conceal.cast::<std::ffi::c_void>());
    }

    let di_window = tv_dict_find(dict_ptr, window_key, 6);
    if di_window.is_null() {
        return OK;
    }

    *win = find_win_by_nr_or_id(di_window.cast::<std::ffi::c_void>());
    if (*win).is_null() {
        emsg(E_INVALWINDOW.as_ptr().cast());
        return FAIL;
    }

    OK
}

/// Error message: "E957: Invalid window number"
static E_INVALWINDOW: &[u8] = b"E957: Invalid window number\0";

/// `matchadd()` `VimL` function.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *` pointers.
/// `argvars` must point to an array of at least 5 elements.
#[export_name = "f_matchadd"]
pub unsafe extern "C" fn rs_f_matchadd(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    use std::ffi::c_char;

    nvim_tv_set_number(rettv, -1);

    let mut grpbuf = [0u8; NUMBUFLEN];
    let mut patbuf = [0u8; NUMBUFLEN];

    let grp = nvim_tv_get_string_buf_chk(argvars, grpbuf.as_mut_ptr().cast::<c_char>());
    let pat = nvim_tv_get_string_buf_chk(
        nvim_tv_idx(argvars, 1),
        patbuf.as_mut_ptr().cast::<c_char>(),
    );

    if grp.is_null() || pat.is_null() {
        return;
    }

    let mut prio: c_int = 10;
    let mut id: c_int = -1;
    let mut error = false;
    let mut conceal_char: *const c_char = std::ptr::null();
    let mut win: *mut WinHandle = nvim_get_curwin();

    let arg2 = nvim_tv_idx(argvars, 2);
    if nvim_tv_get_type(arg2) != VAR_UNKNOWN {
        prio = nvim_tv_get_number_chk(arg2, &raw mut error) as c_int;
        let arg3 = nvim_tv_idx(argvars, 3);
        if nvim_tv_get_type(arg3) != VAR_UNKNOWN {
            id = nvim_tv_get_number_chk(arg3, &raw mut error) as c_int;
            let arg4 = nvim_tv_idx(argvars, 4);
            if nvim_tv_get_type(arg4) != VAR_UNKNOWN
                && matchadd_dict_arg_impl(arg4, &raw mut conceal_char, &raw mut win) == 0
            {
                return;
            }
        }
    }

    if error {
        return;
    }

    if (1..=3).contains(&id) {
        let fmt = b"E798: ID is reserved for \":match\": %ld\0";
        semsg(fmt.as_ptr().cast::<c_char>(), std::ffi::c_long::from(id));
        return;
    }

    let result = crate::core::rs_match_add(win, grp, pat, prio, id, conceal_char);
    nvim_tv_set_number(rettv, i64::from(result));
}

/// `matchaddpos()` `VimL` function.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *` pointers.
/// `argvars` must point to an array of at least 5 elements.
#[allow(clippy::too_many_lines)]
#[export_name = "f_matchaddpos"]
pub unsafe extern "C" fn rs_f_matchaddpos(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    use std::ffi::c_char;

    nvim_tv_set_number(rettv, -1);

    let mut buf = [0u8; NUMBUFLEN];
    let group = nvim_tv_get_string_buf_chk(argvars, buf.as_mut_ptr().cast::<c_char>());
    if group.is_null() {
        return;
    }

    let arg1 = nvim_tv_idx(argvars, 1);
    if nvim_tv_get_type(arg1) != VAR_LIST {
        semsg(e_listarg.as_ptr(), c"matchaddpos()".as_ptr());
        return;
    }

    let l = nvim_tv_get_list(arg1);
    if nvim_tv_list_len(l) == 0 {
        return;
    }

    let mut error = false;
    let mut prio: c_int = 10;
    let mut id: c_int = -1;
    let mut conceal_char: *const c_char = std::ptr::null();
    let mut win: *mut WinHandle = nvim_get_curwin();

    let arg2 = nvim_tv_idx(argvars, 2);
    if nvim_tv_get_type(arg2) != VAR_UNKNOWN {
        prio = nvim_tv_get_number_chk(arg2, &raw mut error) as c_int;
        let arg3 = nvim_tv_idx(argvars, 3);
        if nvim_tv_get_type(arg3) != VAR_UNKNOWN {
            id = nvim_tv_get_number_chk(arg3, &raw mut error) as c_int;
            let arg4 = nvim_tv_idx(argvars, 4);
            if nvim_tv_get_type(arg4) != VAR_UNKNOWN
                && matchadd_dict_arg_impl(arg4, &raw mut conceal_char, &raw mut win) == 0
            {
                return;
            }
        }
    }

    if error {
        return;
    }

    // id == 3 is ok because matchaddpos() substitutes :3match
    if id == 1 || id == 2 {
        let fmt = b"E798: ID is reserved for \"match\": %ld\0";
        semsg(fmt.as_ptr().cast::<c_char>(), std::ffi::c_long::from(id));
        return;
    }

    // Extract positions from VimL list
    let count = nvim_tv_list_len(l);
    let mut lnums: Vec<i32> = Vec::with_capacity(count as usize);
    let mut cols: Vec<i32> = Vec::with_capacity(count as usize);
    let mut lens: Vec<c_int> = Vec::with_capacity(count as usize);

    let mut li = nvim_list_get_first(l);
    let mut idx: c_int = 0;
    while !li.is_null() {
        let tv_li = nvim_listitem_get_tv(li);
        let tv_type = nvim_tv_get_type(tv_li);

        let (lnum, col, len): (i32, i32, c_int);

        if tv_type == VAR_LIST {
            let subl = nvim_tv_get_list(tv_li);
            let mut subli = nvim_list_get_first(subl);
            if subli.is_null() {
                let fmt = b"E5030: Empty list at position %d\0";
                semsg(fmt.as_ptr().cast::<c_char>(), idx as std::ffi::c_int);
                return;
            }

            let mut err = false;
            lnum = nvim_tv_get_number_chk(nvim_listitem_get_tv(subli), &raw mut err) as i32;
            if err {
                return;
            }
            if lnum <= 0 {
                li = nvim_listitem_get_next(li);
                idx += 1;
                continue;
            }

            subli = nvim_listitem_get_next(subli);
            col = if subli.is_null() {
                len = 1;
                0
            } else {
                let c = nvim_tv_get_number_chk(nvim_listitem_get_tv(subli), &raw mut err) as i32;
                if err {
                    return;
                }
                if c < 0 {
                    li = nvim_listitem_get_next(li);
                    idx += 1;
                    continue;
                }
                subli = nvim_listitem_get_next(subli);
                len = if subli.is_null() {
                    1
                } else {
                    let ln =
                        nvim_tv_get_number_chk(nvim_listitem_get_tv(subli), &raw mut err) as c_int;
                    if err {
                        return;
                    }
                    if ln < 0 {
                        li = nvim_listitem_get_next(li);
                        idx += 1;
                        continue;
                    }
                    ln
                };
                c
            };
        } else if tv_type == VAR_NUMBER {
            let n = nvim_tv_get_number(tv_li);
            if n <= 0 {
                li = nvim_listitem_get_next(li);
                idx += 1;
                continue;
            }
            lnum = n as i32;
            col = 0;
            len = 0;
        } else {
            let fmt = b"E5031: List or number required at position %d\0";
            semsg(fmt.as_ptr().cast::<c_char>(), idx as std::ffi::c_int);
            return;
        }

        lnums.push(lnum);
        cols.push(col);
        lens.push(len);
        li = nvim_listitem_get_next(li);
        idx += 1;
    }

    let actual = lnums.len() as c_int;
    let result = crate::core::rs_match_add_pos(
        win,
        group,
        prio,
        id,
        conceal_char,
        lnums.as_ptr(),
        cols.as_ptr(),
        lens.as_ptr(),
        actual,
    );
    nvim_tv_set_number(rettv, i64::from(result));
}

/// `clearmatches()` `VimL` function.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *` pointers.
#[export_name = "f_clearmatches"]
pub unsafe extern "C" fn rs_f_clearmatches(
    argvars: TypvalPtr,
    _rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    let win = get_optional_window(argvars, 0);
    if !win.is_null() {
        clear_matches(win);
    }
}

/// `matchdelete()` `VimL` function.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *` pointers. `argvars` must
/// point to an array of at least 2 elements (argvars[0] = id, argvars[1] = win).
#[export_name = "f_matchdelete"]
pub unsafe extern "C" fn rs_f_matchdelete(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    // get_optional_window(argvars, 1) internally indexes into argvars[1]
    let win = get_optional_window(argvars, 1);
    if win.is_null() {
        nvim_tv_set_number(rettv, -1);
    } else {
        // tv_get_number(&argvars[0]) -- argvars points to argvars[0]
        let id = nvim_tv_get_number(argvars) as c_int;
        let result = match_delete(win, id, true);
        nvim_tv_set_number(rettv, i64::from(result));
    }
}

// =============================================================================
// Phase 2: ex_match
// =============================================================================

/// `:[N]match {group} {pattern}` Ex command.
///
/// Sets `eap->nextcmd` to the start of the next command if any. Also called
/// when skipping commands to find the next command.
///
/// # Safety
///
/// `eap` must be a valid `exarg_T *` pointer.
#[export_name = "ex_match"]
pub unsafe extern "C" fn rs_ex_match(eap: *mut std::ffi::c_void) {
    use std::ffi::c_char;

    let line2 = nvim_eap_get_line2(eap.cast_const());
    let id: c_int = if line2 <= 3 {
        line2
    } else {
        nvim_excmds_emsg_by_id(5); // e_invcmd
        return;
    };

    let skip = nvim_eap_get_skip(eap.cast_const()) != 0;
    let arg = nvim_eap_get_arg(eap.cast_const());

    // First clear any old pattern.
    if !skip {
        match_delete(nvim_get_curwin(), id, false);
    }

    // STRNICMP(eap->arg, "none", 4) == 0
    let arg_is_none = {
        let a = arg.cast::<u8>();
        ((*a | 0x20) == b'n')
            && ((*a.add(1) | 0x20) == b'o')
            && ((*a.add(2) | 0x20) == b'n')
            && ((*a.add(3) | 0x20) == b'e')
            && ((*a.add(4) == b' ')
                || (*a.add(4) == b'\t')
                || ends_excmd(i32::from(*a.add(4))) != 0)
    };

    let end: *mut c_char = if ends_excmd(i32::from(*arg as u8)) != 0 {
        arg
    } else if arg_is_none {
        arg.add(4)
    } else {
        let p = skiptowhite(arg);
        let g = if skip {
            std::ptr::null_mut()
        } else {
            xmemdupz(arg.cast(), p.offset_from(arg) as usize)
        };
        let p = skipwhite(p);
        if *p == 0 {
            // There must be two arguments.
            xfree(g.cast());
            semsg(e_invarg2.as_ptr(), arg);
            return;
        }
        let end = skip_regexp(p.add(1), i32::from(*p as u8), 1);
        if !skip {
            if *end != 0 && ends_excmd(i32::from(*skipwhite(end.add(1)) as u8)) == 0 {
                xfree(g.cast());
                nvim_eap_set_errmsg_const(eap, nvim_docmd_errmsg_trailing_arg(end));
                return;
            }
            if *end != *p {
                xfree(g.cast());
                semsg(e_invarg2.as_ptr(), p);
                return;
            }

            let c = *end as u8;
            *end = 0;
            crate::core::rs_match_add(nvim_get_curwin(), g, p.add(1), 10, id, std::ptr::null());
            xfree(g.cast());
            *end = c as c_char;
        }
        end
    };

    nvim_eap_set_nextcmd(eap, find_nextcmd(end));
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matchadd_id_valid() {
        assert!(matchadd_id_valid(-1)); // Auto
        assert!(matchadd_id_valid(4)); // First valid
        assert!(matchadd_id_valid(100));

        assert!(!matchadd_id_valid(0)); // Invalid
        assert!(!matchadd_id_valid(1)); // Reserved
        assert!(!matchadd_id_valid(2)); // Reserved
        assert!(!matchadd_id_valid(3)); // Reserved
    }

    #[test]
    fn test_matchaddpos_id_valid() {
        assert!(matchaddpos_id_valid(-1)); // Auto
        assert!(matchaddpos_id_valid(3)); // :3match substitute
        assert!(matchaddpos_id_valid(4)); // First valid
        assert!(matchaddpos_id_valid(100));

        assert!(!matchaddpos_id_valid(0)); // Invalid
        assert!(!matchaddpos_id_valid(1)); // Reserved
        assert!(!matchaddpos_id_valid(2)); // Reserved
    }

    #[test]
    fn test_validate_matchadd_args() {
        assert_eq!(validate_matchadd_args(-1, 10), 0); // Auto ID
        assert_eq!(validate_matchadd_args(4, 10), 0); // Valid manual ID
        assert_eq!(validate_matchadd_args(100, -5), 0); // Negative priority is OK

        assert!(validate_matchadd_args(1, 10) < 0); // Reserved
        assert!(validate_matchadd_args(0, 10) < 0); // Invalid
    }

    #[test]
    fn test_validate_matchaddpos_args() {
        assert_eq!(validate_matchaddpos_args(-1, 10), 0);
        assert_eq!(validate_matchaddpos_args(3, 10), 0); // 3 is allowed
        assert_eq!(validate_matchaddpos_args(4, 10), 0);

        assert!(validate_matchaddpos_args(1, 10) < 0); // Reserved
        assert!(validate_matchaddpos_args(2, 10) < 0); // Reserved
    }

    #[test]
    fn test_matcharg_valid_id() {
        assert!(is_matcharg_valid_id(1));
        assert!(is_matcharg_valid_id(2));
        assert!(is_matcharg_valid_id(3));

        assert!(!is_matcharg_valid_id(0));
        assert!(!is_matcharg_valid_id(4));
        assert!(!is_matcharg_valid_id(-1));
    }

    #[test]
    fn test_matcharg_result_len() {
        assert_eq!(matcharg_result_len(1), 2);
        assert_eq!(matcharg_result_len(2), 2);
        assert_eq!(matcharg_result_len(3), 2);

        assert_eq!(matcharg_result_len(0), 0);
        assert_eq!(matcharg_result_len(4), 0);
    }

    #[test]
    fn test_is_valid_pos_key_index() {
        assert!(is_valid_pos_key_index(1));
        assert!(is_valid_pos_key_index(8));

        assert!(!is_valid_pos_key_index(0));
        assert!(!is_valid_pos_key_index(9));
    }

    #[test]
    fn test_pos_key_name() {
        assert_eq!(pos_key_name(1), Some("pos1"));
        assert_eq!(pos_key_name(8), Some("pos8"));
        assert_eq!(pos_key_name(0), None);
        assert_eq!(pos_key_name(9), None);
    }

    #[test]
    fn test_has_required_match_keys() {
        // Pattern match
        assert!(has_required_match_keys(true, true, true, true, false));
        // Position match
        assert!(has_required_match_keys(true, true, true, false, true));
        // Both
        assert!(has_required_match_keys(true, true, true, true, true));

        // Missing required keys
        assert!(!has_required_match_keys(false, true, true, true, false));
        assert!(!has_required_match_keys(true, false, true, true, false));
        assert!(!has_required_match_keys(true, true, false, true, false));
        // Missing both pattern and pos1
        assert!(!has_required_match_keys(true, true, true, false, false));
    }

    #[test]
    fn test_match_cmd_line_valid() {
        assert!(is_valid_match_cmd_line(1));
        assert!(is_valid_match_cmd_line(2));
        assert!(is_valid_match_cmd_line(3));

        assert!(!is_valid_match_cmd_line(0));
        assert!(!is_valid_match_cmd_line(4));
        assert!(!is_valid_match_cmd_line(-1));
    }

    #[test]
    fn test_match_cmd_line_to_id() {
        assert_eq!(match_cmd_line_to_id(1), 1);
        assert_eq!(match_cmd_line_to_id(2), 2);
        assert_eq!(match_cmd_line_to_id(3), 3);
    }
}
