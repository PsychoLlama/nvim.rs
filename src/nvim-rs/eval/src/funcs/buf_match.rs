//! Rust implementations of `matchbufline()` and `matchstrlist()`.
//!
//! These functions match a pattern against buffer lines or a list of strings
//! and return a list of dicts describing the matches.

#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clashing_extern_declarations)]

use std::ffi::{c_char, c_int, c_void};

// ─── Opaque handles ──────────────────────────────────────────────────────────

/// Opaque `typval_T *`
type TypevalPtr = *mut c_void;

/// Opaque `list_T *`
type ListPtr = *mut c_void;

/// Opaque `dict_T *`
type DictPtr = *mut c_void;

/// Opaque `buf_T *`
type BufPtr = *mut c_void;

/// `EvalFuncData` (8-byte opaque union passed by value as pointer)
type EvalFuncData = *mut c_void;

// ─── Constants ───────────────────────────────────────────────────────────────

/// `VAR_UNKNOWN = 0`, `VAR_BOOL = 6`
const VAR_UNKNOWN: c_int = 0;
const VAR_BOOL: c_int = 6;

/// `RE_MAGIC = 1`, `RE_STRING = 32`
const RE_MAGIC: c_int = 1;
const RE_STRING: c_int = 32;

/// `NSUBEXP = 10`
const NSUBEXP: usize = 10;

/// `NUMBUFLEN = 65`
const NUMBUFLEN: usize = 65;

/// FAIL = 0 (what tv_check_for_* returns on failure)
const FAIL: c_int = 0;

// ─── regmatch_T mirror ───────────────────────────────────────────────────────

/// Mirror of C `regmatch_T` (`regexp_defs.h`).
///
/// Layout (64-bit):
/// - `regprog`:    `*mut c_void`         (8 bytes)
/// - `startp`:     `[*mut c_char; 10]`   (80 bytes)
/// - `endp`:       `[*mut c_char; 10]`   (80 bytes)
/// - `rm_matchcol`: `i32`                (4 bytes, `colnr_T`)
/// - `rm_ic`:      `bool`                (1 byte + 3 pad = 4 bytes)
///
/// Total: 176 bytes.
/// Verified by `_Static_assert` in `match.c`.
#[repr(C)]
struct RegmatchT {
    regprog: *mut c_void,
    startp: [*mut c_char; NSUBEXP],
    endp: [*mut c_char; NSUBEXP],
    rm_matchcol: i32,
    rm_ic: bool,
}

impl Default for RegmatchT {
    fn default() -> Self {
        // SAFETY: all-zero is a valid initial state (null pointers, zero ints, false bool)
        unsafe { std::mem::zeroed() }
    }
}

// ─── FFI imports ─────────────────────────────────────────────────────────────

extern "C" {
    // Type-checking helpers (FAIL=0, OK=1).
    // Use *const c_void to match existing declarations in simple.rs/fs.rs/cmdline.rs.
    fn tv_check_for_buffer_arg(args: *const c_void, idx: c_int) -> c_int;
    fn tv_check_for_string_arg(args: *const c_void, idx: c_int) -> c_int;
    fn tv_check_for_lnum_arg(args: *const c_void, idx: c_int) -> c_int;
    fn tv_check_for_opt_dict_arg(args: *const c_void, idx: c_int) -> c_int;
    fn tv_check_for_list_arg(args: *const c_void, idx: c_int) -> c_int;

    // Buffer accessors
    fn nvim_eval_tv_get_buf(tv: TypevalPtr) -> BufPtr;
    fn nvim_eval_buf_ml_get(buf: BufPtr, lnum: c_int) -> *const c_char;
    fn nvim_buf_ml_line_count(buf: BufPtr) -> c_int;
    fn nvim_buf_ml_mfp_is_null(buf: BufPtr) -> bool;

    // p_cpo save/restore (atomic slot in funcs_shim.c)
    fn nvim_eval_set_p_cpo_empty();
    fn nvim_eval_restore_p_cpo();

    // Error counter
    fn nvim_eval_get_did_emsg() -> c_int;

    // Regex. vim_regexec_nl: use *mut c_void for rmp to avoid struct-type clashes
    // with other modules that declare their own regmatch_T mirrors.
    fn vim_regcomp(expr: *const c_char, re_flags: c_int) -> *mut c_void;
    fn vim_regexec_nl(rmp: *mut c_void, line: *const c_char, col: i32) -> c_int;
    fn vim_regfree(prog: *mut c_void);

    // Typval indexing.
    // Use *const c_void to match dispatch.rs.
    fn nvim_tv_idx(argvars: TypevalPtr, i: c_int) -> TypevalPtr;
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_get_string_buf_chk(
        tv: TypevalPtr,
        buf: *mut c_char,
    ) -> *const c_char;

    // p_ic
    fn nvim_search_get_p_ic() -> c_int;

    // tv_get_lnum_buf: exported from typval crate as `tv_get_lnum_buf`
    fn tv_get_lnum_buf(tv: *const c_void, buf: BufPtr) -> i32;

    // tv_get_bool
    fn tv_get_bool(tv: *const c_void) -> c_int;

    // tv_get_string
    fn tv_get_string(tv: TypevalPtr) -> *const c_char;

    // List API
    #[link_name = "tv_list_alloc_ret"]
    fn tv_list_alloc_ret(rettv: TypevalPtr, len: isize) -> ListPtr;
    #[link_name = "tv_list_append_string"]
    fn tv_list_append_string(list: ListPtr, s: *const c_char, len: isize);
    #[link_name = "tv_list_alloc"]
    fn tv_list_alloc(count: c_int) -> ListPtr;

    // List iteration (for matchstrlist).
    // Use *const c_void to match dispatch.rs declarations.
    fn nvim_list_get_first(l: ListPtr) -> *const c_void;
    fn nvim_listitem_get_next(li: *const c_void) -> *const c_void;
    fn nvim_listitem_get_tv(li: *const c_void) -> *const c_void;

    // Dict API
    #[link_name = "tv_dict_alloc"]
    fn tv_dict_alloc() -> DictPtr;
    #[link_name = "tv_list_append_dict"]
    fn tv_list_append_dict(list: ListPtr, dict: DictPtr);
    #[link_name = "tv_dict_add_nr"]
    fn tv_dict_add_nr(dict: DictPtr, key: *const c_char, key_len: usize, nr: i64) -> c_int;
    #[link_name = "tv_dict_add_list"]
    fn tv_dict_add_list(dict: DictPtr, key: *const c_char, key_len: usize, list: ListPtr)
        -> c_int;
    fn tv_dict_add_str_len(
        dict: DictPtr,
        key: *const c_char,
        key_len: usize,
        val: *const c_char,
        len: c_int,
    ) -> c_int;
    // Use *const c_void and isize to match system.rs/misc.rs declarations.
    fn tv_dict_find(
        dict: *const c_void,
        key: *const c_char,
        len: isize,
    ) -> *mut c_void;

    // Error messages
    fn semsg(fmt: *const c_char, ...);
    fn emsg(s: *const c_char) -> bool;
    static e_invalid_buffer_name_str: [c_char; 0];
    static e_buffer_is_not_loaded: [c_char; 0];
    static e_invargval: [c_char; 0];
}

// ─── VAR_STRING / VAR_LIST constants ─────────────────────────────────────────

const VAR_STRING: c_int = 2;

// ─── Internal helper ─────────────────────────────────────────────────────────

/// Populate `mlist` with all matches of `rmp` in `str`.
///
/// Mirrors the C function `get_matches_in_str`.
///
/// # Safety
///
/// All pointers must be valid. `rmp` must have a valid `regprog`.
/// `str` must be a NUL-terminated C string.
unsafe fn get_matches_in_str(
    str_ptr: *const c_char,
    rmp: *mut RegmatchT,
    mlist: ListPtr,
    idx: i64,
    submatches: bool,
    matchbuf: bool,
) {
    let len = libc_strlen(str_ptr);
    let mut startidx: i32 = 0;

    loop {
        let matched = vim_regexec_nl(rmp.cast::<c_void>(), str_ptr, startidx);
        if matched == 0 {
            break;
        }

        let d = tv_dict_alloc();
        tv_list_append_dict(mlist, d);

        if matchbuf {
            tv_dict_add_nr(d, c"lnum".as_ptr(), 4, idx);
        } else {
            tv_dict_add_nr(d, c"idx".as_ptr(), 3, idx);
        }

        let start0 = (*rmp).startp[0];
        let end0 = (*rmp).endp[0];
        let byteidx = start0.offset_from(str_ptr) as i64;
        tv_dict_add_nr(d, c"byteidx".as_ptr(), 7, byteidx);

        let text_len = end0.offset_from(start0) as c_int;
        tv_dict_add_str_len(d, c"text".as_ptr(), 4, start0, text_len);

        if submatches {
            let sml = tv_list_alloc((NSUBEXP - 1) as c_int);
            tv_dict_add_list(d, c"submatches".as_ptr(), 10, sml);

            for i in 1..NSUBEXP {
                if (*rmp).endp[i].is_null() {
                    tv_list_append_string(sml, c"".as_ptr(), 0);
                } else {
                    let sub_len = (*rmp).endp[i].offset_from((*rmp).startp[i]);
                    tv_list_append_string(sml, (*rmp).startp[i], sub_len);
                }
            }
        }

        startidx = end0.offset_from(str_ptr) as i32;
        if startidx >= len as i32 || str_ptr.add(startidx as usize) <= start0 {
            break;
        }
    }
}

/// Return the length of a NUL-terminated C string without pulling in libc.
///
/// # Safety
///
/// `s` must be a valid NUL-terminated C string.
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    p.offset_from(s) as usize
}

// ─── f_matchbufline ──────────────────────────────────────────────────────────

/// `matchbufline()` VimL function.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *`.
#[export_name = "f_matchbufline"]
pub unsafe extern "C" fn rs_f_matchbufline(
    argvars: TypevalPtr,
    rettv: TypevalPtr,
    _fptr: EvalFuncData,
) {
    // Initialise return value to an empty list (mirrors tv_list_alloc_ret with kListLenUnknown=-2)
    let retlist = tv_list_alloc_ret(rettv, -2);

    // Argument type checks
    if tv_check_for_buffer_arg(argvars, 0) == FAIL
        || tv_check_for_string_arg(argvars, 1) == FAIL
        || tv_check_for_lnum_arg(argvars, 2) == FAIL
        || tv_check_for_lnum_arg(argvars, 3) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 4) == FAIL
    {
        return;
    }

    let prev_did_emsg = nvim_eval_get_did_emsg();
    let buf = nvim_eval_tv_get_buf(argvars);
    if buf.is_null() {
        if nvim_eval_get_did_emsg() == prev_did_emsg {
            semsg(
                e_invalid_buffer_name_str.as_ptr(),
                tv_get_string(argvars),
            );
        }
        return;
    }
    if nvim_buf_ml_mfp_is_null(buf) {
        emsg(e_buffer_is_not_loaded.as_ptr());
        return;
    }

    let mut patbuf = [0u8; NUMBUFLEN];
    let arg1 = nvim_tv_idx(argvars, 1);
    let pat = nvim_tv_get_string_buf_chk(arg1, patbuf.as_mut_ptr().cast::<c_char>());
    if pat.is_null() {
        return;
    }

    let did_emsg_before = nvim_eval_get_did_emsg();

    let arg2 = nvim_tv_idx(argvars, 2);
    let mut slnum = tv_get_lnum_buf(arg2, buf);
    if nvim_eval_get_did_emsg() > did_emsg_before {
        return;
    }
    if slnum < 1 {
        semsg(e_invargval.as_ptr(), c"lnum".as_ptr());
        return;
    }

    let arg3 = nvim_tv_idx(argvars, 3);
    let mut elnum = tv_get_lnum_buf(arg3, buf);
    if nvim_eval_get_did_emsg() > did_emsg_before {
        return;
    }
    if elnum < 1 || elnum < slnum {
        semsg(e_invargval.as_ptr(), c"end_lnum".as_ptr());
        return;
    }

    let ml_line_count = nvim_buf_ml_line_count(buf);
    if elnum > ml_line_count {
        elnum = ml_line_count;
    }

    // Read optional "submatches" from dict arg
    let mut submatches = false;
    let arg4 = nvim_tv_idx(argvars, 4);
    if nvim_tv_get_type(arg4) != VAR_UNKNOWN {
        // Get the dict pointer from typval (v_dict is at offset 8)
        let dict_ptr: DictPtr =
            std::ptr::read_unaligned(arg4.cast::<u8>().add(8).cast::<DictPtr>());
        if !dict_ptr.is_null() {
            let di = tv_dict_find(dict_ptr, c"submatches".as_ptr(), 10);
            if !di.is_null() {
                // dictitem_T*: first field is typval_T di_tv at offset 0
                let di_tv: TypevalPtr = di;
                if nvim_tv_get_type(di_tv) != VAR_BOOL {
                    semsg(e_invargval.as_ptr(), c"submatches".as_ptr());
                    return;
                }
                submatches = tv_get_bool(di_tv) != 0;
            }
        }
    }

    // Save and empty p_cpo
    nvim_eval_set_p_cpo_empty();

    let regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    let mut regmatch = RegmatchT { regprog, ..RegmatchT::default() };
    if !regmatch.regprog.is_null() {
        regmatch.rm_ic = nvim_search_get_p_ic() != 0;

        while slnum <= elnum {
            let str_ptr = nvim_eval_buf_ml_get(buf, slnum);
            get_matches_in_str(str_ptr, &raw mut regmatch, retlist, i64::from(slnum), submatches, true);
            slnum += 1;
        }

        vim_regfree(regmatch.regprog);
    }

    nvim_eval_restore_p_cpo();
}

// ─── f_matchstrlist ──────────────────────────────────────────────────────────

/// `matchstrlist()` VimL function.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *`.
#[export_name = "f_matchstrlist"]
pub unsafe extern "C" fn rs_f_matchstrlist(
    argvars: TypevalPtr,
    rettv: TypevalPtr,
    _fptr: EvalFuncData,
) {
    let retlist = tv_list_alloc_ret(rettv, -2);

    if tv_check_for_list_arg(argvars, 0) == FAIL
        || tv_check_for_string_arg(argvars, 1) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 2) == FAIL
    {
        return;
    }

    // Get list from argvars[0].vval.v_list (at offset 8 in typval_T)
    let l: ListPtr = std::ptr::read_unaligned(argvars.cast::<u8>().add(8).cast::<ListPtr>());
    if l.is_null() {
        return;
    }

    let mut patbuf = [0u8; NUMBUFLEN];
    let arg1 = nvim_tv_idx(argvars, 1);
    let pat = nvim_tv_get_string_buf_chk(arg1, patbuf.as_mut_ptr().cast::<c_char>());
    if pat.is_null() {
        return;
    }

    // Save and empty p_cpo
    nvim_eval_set_p_cpo_empty();

    let regprog2 = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if regprog2.is_null() {
        nvim_eval_restore_p_cpo();
        return;
    }
    let mut regmatch = RegmatchT {
        regprog: regprog2,
        rm_ic: nvim_search_get_p_ic() != 0,
        ..RegmatchT::default()
    };

    // Read optional "submatches"
    let mut submatches = false;
    let arg2 = nvim_tv_idx(argvars, 2);
    if nvim_tv_get_type(arg2) != VAR_UNKNOWN {
        let dict_ptr: DictPtr =
            std::ptr::read_unaligned(arg2.cast::<u8>().add(8).cast::<DictPtr>());
        if !dict_ptr.is_null() {
            let di = tv_dict_find(dict_ptr, c"submatches".as_ptr(), 10);
            if !di.is_null() {
                let di_tv: TypevalPtr = di;
                if nvim_tv_get_type(di_tv) != VAR_BOOL {
                    semsg(e_invargval.as_ptr(), c"submatches".as_ptr());
                    vim_regfree(regmatch.regprog);
                    nvim_eval_restore_p_cpo();
                    return;
                }
                submatches = tv_get_bool(di_tv) != 0;
            }
        }
    }

    // Iterate the list
    let mut li = nvim_list_get_first(l);
    let mut idx: i64 = 0;
    while !li.is_null() {
        let li_tv = nvim_listitem_get_tv(li);
        if nvim_tv_get_type(li_tv) == VAR_STRING {
            // Read the string pointer from v_string (at offset 8 in typval_T)
            let s: *const c_char =
                std::ptr::read_unaligned(li_tv.cast::<u8>().add(8).cast::<*const c_char>());
            if !s.is_null() {
                get_matches_in_str(s, &raw mut regmatch, retlist, idx, submatches, false);
            }
        }
        li = nvim_listitem_get_next(li);
        idx += 1;
    }

    vim_regfree(regmatch.regprog);
    nvim_eval_restore_p_cpo();

    // suppress unused warning: retlist is consumed by rettv ownership
    let _ = retlist;
}
