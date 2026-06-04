//! Rust implementation of the VimL `split()` built-in function.
//!
//! Ported from `f_split` in `src/nvim/eval/funcs_shim.c` (Phase 29).
//! The C body has been deleted; this file is the authoritative implementation.

#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clashing_extern_declarations)]

use std::ffi::{c_char, c_int, c_void};

// ─── Constants ───────────────────────────────────────────────────────────────

const VAR_UNKNOWN: c_int = 0;

/// `RE_MAGIC = 1`, `RE_STRING = 32`
const RE_MAGIC: c_int = 1;
const RE_STRING: c_int = 32;

/// NSUBEXP = 10
const NSUBEXP: usize = 10;

/// `NUMBUFLEN = 65` (size of temp buf for number→string conversion in tv_get_string_buf)
const NUMBUFLEN: usize = 65;

/// `kListLenMayKnow = -1`
const K_LIST_LEN_MAY_KNOW: isize = -1;

/// EvalFuncData — opaque 8-byte union passed by value (use *mut c_void as pointer-sized stand-in)
type EvalFuncData = *mut c_void;

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
    // tv_get_string: returns a *const c_char from a typval_T*
    fn tv_get_string(tv: *const c_void) -> *const c_char;

    // tv_get_string_buf_chk: fills a NUMBUFLEN buffer and returns pointer, or NULL on type error
    fn nvim_tv_get_string_buf_chk(tv: *const c_void, buf: *mut c_char) -> *const c_char;

    // tv_get_bool_chk: returns bool value; sets *error on type error
    fn tv_get_bool_chk(tv: *const c_void, error: *mut bool) -> c_int;

    // tv_list_alloc_ret: allocates and assigns a new list into rettv; returns list pointer
    fn tv_list_alloc_ret(rettv: *mut c_void, len: isize) -> *mut c_void;

    // tv_list_append_string: appends a string item to a list
    fn tv_list_append_string(l: *mut c_void, s: *const c_char, len: isize);

    // p_cpo save/restore (single atomic slot in funcs_shim.c:239-242)
    fn nvim_eval_set_p_cpo_empty();
    fn nvim_eval_restore_p_cpo();

    // Regex
    fn vim_regcomp(expr: *const c_char, re_flags: c_int) -> *mut c_void;
    // vim_regexec_nl takes regmatch_T* but we cast to *mut c_void to avoid
    // clashing extern declarations with other modules.
    fn vim_regexec_nl(rmp: *mut c_void, line: *const c_char, col: i32) -> c_int;
    fn vim_regfree(prog: *mut c_void);

    // utfc_ptr2len: byte length of the UTF-8 character at *p
    fn utfc_ptr2len(p: *const c_char) -> c_int;
}

// ─── Accessor for argvars[i] ─────────────────────────────────────────────────

/// Return a pointer to `argvars[i]` (each typval_T is 16 bytes).
///
/// # Safety
///
/// `argvars` must be a valid pointer to an array of at least `i+1` typval_T values.
#[inline]
unsafe fn argvar(argvars: *const c_void, i: usize) -> *const c_void {
    argvars.cast::<u8>().add(i * 16).cast::<c_void>()
}

/// Read `v_type` (i32) from a typval_T.
///
/// # Safety
///
/// `tv` must be a valid pointer to a typval_T.
#[inline]
unsafe fn tv_type(tv: *const c_void) -> c_int {
    *tv.cast::<c_int>()
}

// ─── f_split ─────────────────────────────────────────────────────────────────

/// `split()` VimL function.
///
/// Splits `{expr}` into a list using regex `{pat}`, optionally keeping empty items.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *`.
#[export_name = "f_split"]
pub unsafe extern "C" fn rs_f_split(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: EvalFuncData,
) {
    let mut col: i32 = 0;
    let mut keepempty = false;
    let mut typeerr = false;

    // Read str = tv_get_string(&argvars[0])
    let str_start: *const c_char = tv_get_string(argvar(argvars, 0));

    // Optional pattern argument
    let mut pat: *const c_char = std::ptr::null();
    let mut patbuf = [0u8; NUMBUFLEN];

    let arg1 = argvar(argvars, 1);
    if tv_type(arg1) != VAR_UNKNOWN {
        pat = nvim_tv_get_string_buf_chk(arg1, patbuf.as_mut_ptr().cast::<c_char>());
        if pat.is_null() {
            typeerr = true;
        }
        let arg2 = argvar(argvars, 2);
        if tv_type(arg2) != VAR_UNKNOWN {
            keepempty = tv_get_bool_chk(arg2, &raw mut typeerr) != 0;
        }
    }

    // Default pattern: whitespace split
    if pat.is_null() || *pat == 0 {
        pat = c"[\\x01- ]\\+".as_ptr();
    }

    // Always alloc the return list first (matches C goto theend behaviour)
    let retlist = tv_list_alloc_ret(rettv, K_LIST_LEN_MAY_KNOW);

    if typeerr {
        return; // mirrors `goto theend` — list already allocated above
    }

    // Save/empty p_cpo around the regex compile+exec
    nvim_eval_set_p_cpo_empty();

    let regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    let mut regmatch = RegmatchT {
        regprog,
        ..RegmatchT::default()
    };

    if !regmatch.regprog.is_null() {
        let mut str_ptr: *const c_char = str_start;
        // Track how many items have been appended (mirrors tv_list_len usage in C)
        let mut appended: c_int = 0;

        while *str_ptr != 0 || keepempty {
            let match_found = if *str_ptr == 0 {
                // Empty item at the end.
                false
            } else {
                vim_regexec_nl((&raw mut regmatch).cast::<c_void>(), str_ptr, col) != 0
            };

            let end: *const c_char = if match_found {
                regmatch.startp[0].cast_const()
            } else {
                // strlen: walk to NUL
                let mut p = str_ptr;
                while *p != 0 {
                    p = p.add(1);
                }
                p
            };

            // Decide whether to append this item (mirrors C condition exactly)
            if keepempty
                || end > str_ptr
                || (appended > 0
                    && *str_ptr != 0
                    && match_found
                    && end < regmatch.endp[0].cast_const())
            {
                let item_len = end.offset_from(str_ptr);
                tv_list_append_string(retlist, str_ptr, item_len);
                appended += 1;
            }

            if !match_found {
                break;
            }

            // Advance past the match.
            if regmatch.endp[0] > str_ptr.cast_mut() {
                col = 0;
            } else {
                // Don't get stuck at the same position on an empty match.
                col = utfc_ptr2len(regmatch.endp[0]);
            }
            str_ptr = regmatch.endp[0].cast_const();
        }

        vim_regfree(regmatch.regprog);
    }

    // Restore p_cpo (mirrors C: p_cpo = save_cpo at theend label)
    nvim_eval_restore_p_cpo();
}
