//! VimL string function implementations (f_* functions)
//!
//! Implements the string-related VimL built-in functions:
//! byteidx, byteidxcomp, charidx, str2list, str2nr, strgetchar, stridx,
//! string, strlen, strcharlen, strchars, strutf16len, strdisplaywidth,
//! strwidth, strcharpart, strpart, strridx, strtrans, utf16idx,
//! tolower, toupper, tr, trim

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::useless_let_if_seq)]
#![allow(clippy::if_not_else)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Types
// =============================================================================

/// Opaque pointer to typval_T
pub type TypvalPtr = *mut c_void;

/// Opaque handle for EvalFuncData union
pub type EvalFuncData = *mut c_void;

/// varnumber_T (matches C's int64_t)
type VarNumber = i64;

// =============================================================================
// Constants
// =============================================================================

const VAR_UNKNOWN: c_int = 0;
const VAR_STRING: c_int = 2;
const NUMBUFLEN: usize = 65;
const FAIL: c_int = -1;

// str2nr flags
const STR2NR_BIN: c_int = 1 << 0;
const STR2NR_OCT: c_int = 1 << 1;
const STR2NR_OOCT: c_int = 1 << 3;
const STR2NR_HEX: c_int = 1 << 2;
const STR2NR_FORCE: c_int = 1 << 7;
const STR2NR_QUOTE: c_int = 1 << 4;

// =============================================================================
// garray_T repr
// garray_T { int ga_len, ga_maxlen, ga_itemsize, ga_growsize; void *ga_data; }
// =============================================================================

#[repr(C)]
struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

impl GArray {
    fn zeroed() -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// C function declarations
// =============================================================================

/// Minimal repr(C) mirror for typval_T (v_type + v_lock + vval.v_string).
/// Used only in tv_set_vstring_owned.
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct TvForString {
    v_type: c_int,
    v_lock: c_int,
    v_string: *mut c_char,
}

/// Set v_type = VAR_STRING (2) and vval.v_string = s in a typval_T.
/// Inlined from nvim_tv_set_vstring_owned.
///
/// # Safety
/// `tv` must be a valid non-null pointer to a typval_T.
#[inline]
unsafe fn tv_set_vstring_owned(tv: TypvalPtr, s: *mut c_char) {
    let t = &mut *tv.cast::<TvForString>();
    t.v_type = 2; // VAR_STRING
    t.v_string = s;
}

extern "C" {
    // typval accessors (eval/typval.c and eval_shim.c)
    fn nvim_tv_idx(argvars: TypvalPtr, i: c_int) -> TypvalPtr;
    fn nvim_tv_set_number(rettv: TypvalPtr, n: i64);
    fn nvim_tv_get_type(tv: TypvalPtr) -> c_int;
    fn nvim_tv_get_list(rettv: TypvalPtr) -> *mut c_void;

    // eval/typval functions
    fn tv_get_string(tv: TypvalPtr) -> *const c_char;
    fn tv_get_string_chk(tv: TypvalPtr) -> *const c_char;
    fn tv_get_string_buf_chk(tv: TypvalPtr, buf: *mut c_char) -> *const c_char;
    fn tv_get_number(tv: TypvalPtr) -> VarNumber;
    fn tv_get_number_chk(tv: TypvalPtr, error: *mut bool) -> VarNumber;
    fn tv_get_bool(tv: TypvalPtr) -> VarNumber;
    fn tv_get_bool_chk(tv: TypvalPtr, error: *mut bool) -> VarNumber;
    fn tv_check_for_string_arg(argvars: TypvalPtr, idx: c_int) -> c_int;
    fn tv_check_for_number_arg(argvars: TypvalPtr, idx: c_int) -> c_int;
    fn tv_check_for_opt_bool_arg(argvars: TypvalPtr, idx: c_int) -> c_int;
    fn tv_check_for_opt_number_arg(argvars: TypvalPtr, idx: c_int) -> c_int;
    fn tv_check_for_opt_string_arg(argvars: TypvalPtr, idx: c_int) -> c_int;
    fn tv_list_alloc_ret(rettv: TypvalPtr, len: c_int);
    fn tv_list_append_number(list: *mut c_void, nr: VarNumber);

    // mbyte functions
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_ptr2len(p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn mb_ptr2char_adv(pp: *mut *const c_char) -> c_int;
    fn mb_cptr2char_adv(pp: *mut *const c_char) -> c_int;
    fn mb_string2cells(s: *const c_char) -> c_int;

    // message functions
    fn emsg(s: *const c_char) -> bool;
    fn semsg(fmt: *const c_char, ...) -> bool;

    // charset functions
    fn vim_str2nr(
        start: *const c_char,
        prep: *mut c_int,
        len: *mut c_int,
        what: c_int,
        nptr: *mut VarNumber,
        unptr: *mut u64,
        maxlen: c_int,
        strict: bool,
        overflow: *mut bool,
    );
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn linetabsize_col(col: c_int, p: *mut c_char) -> c_int;
    fn transstr(s: *const c_char, untab: bool) -> *mut c_char;

    // eval/encode
    fn encode_tv2string(tv: TypvalPtr, tofree: *mut *mut c_char) -> *mut c_char;

    // memory
    fn xmemdupz(p: *const c_char, len: usize) -> *mut c_char;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // garray functions
    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GArray, n: c_int);
    fn ga_append(gap: *mut GArray, c: c_char);
    fn ga_clear(gap: *mut GArray);

    // strcase_save (now Rust)
    #[link_name = "strcase_save"]
    fn strcase_save(orig: *const c_char, upper: bool) -> *mut c_char;

    // error strings
    fn nvim_get_e_invarg() -> *const c_char;
    fn nvim_get_e_invarg2() -> *const c_char;
    fn nvim_get_e_using_number_as_bool_nr() -> *const c_char;
}

// =============================================================================
// Helper: get the string for a translated error message
// =============================================================================

// We use the C gettext mechanism via these inline helpers.
// Error message constants are accessed via C accessor functions in ex_docmd.c.
// Since we can't use _() macro in Rust, we call the C wrappers.

/// Helper: emit "E475: Invalid argument: %s" via semsg.
unsafe fn semsg_invarg2(s: *const c_char) {
    unsafe {
        semsg(nvim_get_e_invarg2(), s);
    }
}

/// Helper: emit "E805: Using a Number as a Bool: ..." via semsg.
unsafe fn semsg_number_as_bool(n: VarNumber) {
    unsafe {
        semsg(nvim_get_e_using_number_as_bool_nr(), n);
    }
}

// =============================================================================
// Implementation helpers
// =============================================================================

/// Implementation of "byteidx()" and "byteidxcomp()" functions
unsafe fn byteidx_common(argvars: TypvalPtr, rettv: TypvalPtr, comp: bool) {
    unsafe {
        nvim_tv_set_number(rettv, -1);

        let tv0 = nvim_tv_idx(argvars, 0);
        let tv1 = nvim_tv_idx(argvars, 1);
        let str = tv_get_string_chk(tv0);
        let idx = tv_get_number_chk(tv1, std::ptr::null_mut());
        if str.is_null() || idx < 0 {
            return;
        }

        let tv2 = nvim_tv_idx(argvars, 2);
        let mut utf16idx: VarNumber = 0;
        if nvim_tv_get_type(tv2) != VAR_UNKNOWN {
            let mut error = false;
            utf16idx = tv_get_bool_chk(tv2, &raw mut error);
            if error {
                return;
            }
            if utf16idx < 0 || utf16idx > 1 {
                semsg_number_as_bool(utf16idx);
                return;
            }
        }

        let mut t = str;
        let mut idx = idx;
        loop {
            if idx <= 0 {
                break;
            }
            if *t == 0 {
                return; // EOL reached
            }
            if utf16idx != 0 {
                let clen = if comp {
                    utf_ptr2len(t)
                } else {
                    utfc_ptr2len(t)
                };
                let c = if clen > 1 {
                    utf_ptr2char(t)
                } else {
                    c_int::from(*t as u8)
                };
                if c > 0xFFFF {
                    idx -= 1;
                }
            }
            if idx > 0 {
                t = t.add(if comp {
                    utf_ptr2len(t)
                } else {
                    utfc_ptr2len(t)
                } as usize);
            }
            idx -= 1;
        }
        nvim_tv_set_number(rettv, t.offset_from(str) as i64);
    }
}

/// Implementation of "strcharlen()" and "strchars(skipcc=true/false)"
unsafe fn strchar_common(argvars: TypvalPtr, rettv: TypvalPtr, skipcc: bool) {
    unsafe {
        let tv0 = nvim_tv_idx(argvars, 0);
        let s = tv_get_string(tv0);
        let mut len: VarNumber = 0;
        let mut s = s;
        while *s != 0 {
            if skipcc {
                mb_ptr2char_adv(&raw mut s);
            } else {
                mb_cptr2char_adv(&raw mut s);
            }
            len += 1;
        }
        nvim_tv_set_number(rettv, len);
    }
}

// =============================================================================
// f_byteidx
// =============================================================================

/// "byteidx()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_byteidx"]
pub unsafe extern "C" fn rs_f_byteidx(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe { byteidx_common(argvars, rettv, false) }
}

/// "byteidxcomp()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_byteidxcomp"]
pub unsafe extern "C" fn rs_f_byteidxcomp(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe { byteidx_common(argvars, rettv, true) }
}

// =============================================================================
// f_charidx
// =============================================================================

/// "charidx()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_charidx"]
pub unsafe extern "C" fn rs_f_charidx(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        nvim_tv_set_number(rettv, -1);

        if tv_check_for_string_arg(argvars, 0) == FAIL
            || tv_check_for_number_arg(argvars, 1) == FAIL
            || tv_check_for_opt_bool_arg(argvars, 2) == FAIL
        {
            return;
        }
        let tv2 = nvim_tv_idx(argvars, 2);
        if nvim_tv_get_type(tv2) != VAR_UNKNOWN {
            if tv_check_for_opt_bool_arg(argvars, 3) == FAIL {
                return;
            }
        }

        let tv0 = nvim_tv_idx(argvars, 0);
        let tv1 = nvim_tv_idx(argvars, 1);
        let str = tv_get_string_chk(tv0);
        let idx = tv_get_number_chk(tv1, std::ptr::null_mut());
        if str.is_null() || idx < 0 {
            return;
        }

        let mut countcc: VarNumber = 0;
        let mut utf16idx: VarNumber = 0;
        if nvim_tv_get_type(tv2) != VAR_UNKNOWN {
            countcc = tv_get_bool(tv2);
            let tv3 = nvim_tv_idx(argvars, 3);
            if nvim_tv_get_type(tv3) != VAR_UNKNOWN {
                utf16idx = tv_get_bool(tv3);
            }
        }

        let mut p = str;
        let mut len: c_int = 0;
        let mut idx = idx;
        loop {
            let cond = if utf16idx != 0 {
                idx >= 0
            } else {
                p <= str.add(idx as usize)
            };
            if !cond {
                break;
            }
            if *p == 0 {
                // At end: check if index matches exactly
                let at_end = if utf16idx != 0 {
                    idx == 0
                } else {
                    p == str.add(idx as usize)
                };
                if at_end {
                    nvim_tv_set_number(rettv, i64::from(len));
                }
                return;
            }
            if utf16idx != 0 {
                idx -= 1;
                let clen = if countcc != 0 {
                    utf_ptr2len(p)
                } else {
                    utfc_ptr2len(p)
                };
                let c = if clen > 1 {
                    utf_ptr2char(p)
                } else {
                    c_int::from(*p as u8)
                };
                if c > 0xFFFF {
                    idx -= 1;
                }
            }
            p = p.add(if countcc != 0 {
                utf_ptr2len(p)
            } else {
                utfc_ptr2len(p)
            } as usize);
            len += 1;
        }

        nvim_tv_set_number(rettv, if len > 0 { i64::from(len) - 1 } else { 0 });
    }
}

// =============================================================================
// f_str2list
// =============================================================================

/// "str2list()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_str2list"]
pub unsafe extern "C" fn rs_f_str2list(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        tv_list_alloc_ret(rettv, -1); // kListLenUnknown = -1
        let tv0 = nvim_tv_idx(argvars, 0);
        let mut p = tv_get_string(tv0);
        let list = nvim_tv_get_list(rettv);
        while *p != 0 {
            let ch = utf_ptr2char(p);
            tv_list_append_number(list, i64::from(ch));
            p = p.add(utf_ptr2len(p) as usize);
        }
    }
}

// =============================================================================
// f_str2nr
// =============================================================================

/// "str2nr()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_str2nr"]
pub unsafe extern "C" fn rs_f_str2nr(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let mut base: c_int = 10;
        let mut what: c_int = 0;

        let tv1 = nvim_tv_idx(argvars, 1);
        if nvim_tv_get_type(tv1) != VAR_UNKNOWN {
            base = tv_get_number(tv1) as c_int;
            if base != 2 && base != 8 && base != 10 && base != 16 {
                emsg(nvim_get_e_invarg());
                return;
            }
            let tv2 = nvim_tv_idx(argvars, 2);
            if nvim_tv_get_type(tv2) != VAR_UNKNOWN && tv_get_bool(tv2) != 0 {
                what |= STR2NR_QUOTE;
            }
        }

        let tv0 = nvim_tv_idx(argvars, 0);
        let s0 = tv_get_string(tv0);
        let mut p = skipwhite(s0);
        let isneg = *p as u8 == b'-';
        if *p as u8 == b'+' || *p as u8 == b'-' {
            p = skipwhite(p.add(1));
        }
        match base {
            2 => what |= STR2NR_BIN | STR2NR_FORCE,
            8 => what |= STR2NR_OCT | STR2NR_OOCT | STR2NR_FORCE,
            16 => what |= STR2NR_HEX | STR2NR_FORCE,
            _ => {}
        }
        let mut n: VarNumber = 0;
        vim_str2nr(
            p,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            what,
            &raw mut n,
            std::ptr::null_mut(),
            0,
            false,
            std::ptr::null_mut(),
        );
        nvim_tv_set_number(rettv, if isneg { -n } else { n });
    }
}

// =============================================================================
// f_strgetchar
// =============================================================================

/// "strgetchar()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strgetchar"]
pub unsafe extern "C" fn rs_f_strgetchar(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        nvim_tv_set_number(rettv, -1);

        let tv0 = nvim_tv_idx(argvars, 0);
        let str = tv_get_string_chk(tv0);
        if str.is_null() {
            return;
        }
        let tv1 = nvim_tv_idx(argvars, 1);
        let mut error = false;
        let mut charidx = tv_get_number_chk(tv1, &raw mut error);
        if error {
            return;
        }

        let len = libc::strlen(str);
        let mut byteidx: usize = 0;
        while charidx >= 0 && byteidx < len {
            if charidx == 0 {
                let ch = utf_ptr2char(str.add(byteidx));
                nvim_tv_set_number(rettv, i64::from(ch));
                break;
            }
            charidx -= 1;
            byteidx += utf_ptr2len(str.add(byteidx)) as usize;
        }
    }
}

// =============================================================================
// f_stridx
// =============================================================================

/// "stridx()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_stridx"]
pub unsafe extern "C" fn rs_f_stridx(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        nvim_tv_set_number(rettv, -1);

        let mut buf = [0u8; NUMBUFLEN];
        let tv1 = nvim_tv_idx(argvars, 1);
        let tv0 = nvim_tv_idx(argvars, 0);
        let needle = tv_get_string_chk(tv1);
        let haystack = tv_get_string_buf_chk(tv0, buf.as_mut_ptr().cast());
        if needle.is_null() || haystack.is_null() {
            return;
        }
        let haystack_start = haystack;

        let tv2 = nvim_tv_idx(argvars, 2);
        let mut haystack = haystack;
        if nvim_tv_get_type(tv2) != VAR_UNKNOWN {
            let mut error = false;
            let start_idx = tv_get_number_chk(tv2, &raw mut error) as isize;
            if error || start_idx >= libc::strlen(haystack) as isize {
                return;
            }
            if start_idx >= 0 {
                haystack = haystack.add(start_idx as usize);
            }
        }

        let pos = libc::strstr(haystack, needle);
        if !pos.is_null() {
            nvim_tv_set_number(rettv, pos.offset_from(haystack_start) as i64);
        }
    }
}

// =============================================================================
// f_string
// =============================================================================

/// "string()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_string"]
pub unsafe extern "C" fn rs_f_string(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        *rettv.cast::<c_int>() = VAR_STRING;
        let tv0 = nvim_tv_idx(argvars, 0);
        let s = encode_tv2string(tv0, std::ptr::null_mut());
        // rettv->vval.v_string = s; but v_type already set via set_type
        tv_set_vstring_owned(rettv, s);
    }
}

// =============================================================================
// f_strlen
// =============================================================================

/// "strlen()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strlen"]
pub unsafe extern "C" fn rs_f_strlen(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let tv0 = nvim_tv_idx(argvars, 0);
        let s = tv_get_string(tv0);
        nvim_tv_set_number(rettv, libc::strlen(s) as i64);
    }
}

// =============================================================================
// f_strcharlen
// =============================================================================

/// "strcharlen()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strcharlen"]
pub unsafe extern "C" fn rs_f_strcharlen(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe { strchar_common(argvars, rettv, true) }
}

// =============================================================================
// f_strchars
// =============================================================================

/// "strchars()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strchars"]
pub unsafe extern "C" fn rs_f_strchars(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let mut skipcc: VarNumber = 0;

        let tv1 = nvim_tv_idx(argvars, 1);
        if nvim_tv_get_type(tv1) != VAR_UNKNOWN {
            let mut error = false;
            skipcc = tv_get_bool_chk(tv1, &raw mut error);
            if error {
                return;
            }
            if skipcc < 0 || skipcc > 1 {
                semsg_number_as_bool(skipcc);
                return;
            }
        }

        strchar_common(argvars, rettv, skipcc != 0);
    }
}

// =============================================================================
// f_strutf16len
// =============================================================================

/// "strutf16len()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strutf16len"]
pub unsafe extern "C" fn rs_f_strutf16len(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        nvim_tv_set_number(rettv, -1);

        if tv_check_for_string_arg(argvars, 0) == FAIL
            || tv_check_for_opt_bool_arg(argvars, 1) == FAIL
        {
            return;
        }

        let tv1 = nvim_tv_idx(argvars, 1);
        let mut countcc: VarNumber = 0;
        if nvim_tv_get_type(tv1) != VAR_UNKNOWN {
            countcc = tv_get_bool(tv1);
        }

        let tv0 = nvim_tv_idx(argvars, 0);
        let mut s = tv_get_string(tv0);
        let mut len: VarNumber = 0;
        while *s != 0 {
            let ch = if countcc != 0 {
                mb_cptr2char_adv(&raw mut s)
            } else {
                mb_ptr2char_adv(&raw mut s)
            };
            if ch > 0xFFFF {
                len += 1;
            }
            len += 1;
        }
        nvim_tv_set_number(rettv, len);
    }
}

// =============================================================================
// f_strdisplaywidth
// =============================================================================

/// "strdisplaywidth()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strdisplaywidth"]
pub unsafe extern "C" fn rs_f_strdisplaywidth(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        let tv0 = nvim_tv_idx(argvars, 0);
        let s = tv_get_string(tv0);
        let mut col: c_int = 0;

        let tv1 = nvim_tv_idx(argvars, 1);
        if nvim_tv_get_type(tv1) != VAR_UNKNOWN {
            col = tv_get_number(tv1) as c_int;
        }

        let result = linetabsize_col(col, s as *mut c_char) - col;
        nvim_tv_set_number(rettv, i64::from(result));
    }
}

// =============================================================================
// f_strwidth
// =============================================================================

/// "strwidth()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strwidth"]
pub unsafe extern "C" fn rs_f_strwidth(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let tv0 = nvim_tv_idx(argvars, 0);
        let s = tv_get_string(tv0);
        nvim_tv_set_number(rettv, i64::from(mb_string2cells(s)));
    }
}

// =============================================================================
// f_strcharpart
// =============================================================================

/// "strcharpart()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strcharpart"]
pub unsafe extern "C" fn rs_f_strcharpart(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    unsafe {
        let tv0 = nvim_tv_idx(argvars, 0);
        let p = tv_get_string(tv0);
        let slen = libc::strlen(p);

        let mut nbyte: c_int = 0;
        let mut skipcc: VarNumber = 0;
        let mut error = false;

        let tv1 = nvim_tv_idx(argvars, 1);
        let mut nchar = tv_get_number_chk(tv1, &raw mut error);
        if !error {
            let tv2 = nvim_tv_idx(argvars, 2);
            let tv3 = nvim_tv_idx(argvars, 3);
            if nvim_tv_get_type(tv2) != VAR_UNKNOWN && nvim_tv_get_type(tv3) != VAR_UNKNOWN {
                skipcc = tv_get_bool_chk(tv3, &raw mut error);
                if error {
                    return;
                }
                if skipcc < 0 || skipcc > 1 {
                    semsg_number_as_bool(skipcc);
                    return;
                }
            }

            if nchar > 0 {
                while nchar > 0 && (nbyte as usize) < slen {
                    if skipcc != 0 {
                        nbyte += utfc_ptr2len(p.add(nbyte as usize));
                    } else {
                        nbyte += utf_ptr2len(p.add(nbyte as usize));
                    }
                    nchar -= 1;
                }
            } else {
                nbyte = nchar as c_int;
            }
        }

        let mut len: c_int = 0;
        let tv2 = nvim_tv_idx(argvars, 2);
        if nvim_tv_get_type(tv2) != VAR_UNKNOWN {
            let mut charlen = tv_get_number(tv2) as c_int;
            while charlen > 0 && nbyte + len < slen as c_int {
                let off = nbyte + len;
                if off < 0 {
                    len += 1;
                } else if skipcc != 0 {
                    len += utfc_ptr2len(p.add(off as usize));
                } else {
                    len += utf_ptr2len(p.add(off as usize));
                }
                charlen -= 1;
            }
        } else {
            len = slen as c_int - nbyte;
        }

        // Clamp to valid range
        if nbyte < 0 {
            len += nbyte;
            nbyte = 0;
        } else if nbyte as usize > slen {
            nbyte = slen as c_int;
        }
        if len < 0 {
            len = 0;
        } else if nbyte + len > slen as c_int {
            len = slen as c_int - nbyte;
        }

        tv_set_vstring_owned(rettv, xmemdupz(p.add(nbyte as usize), len as usize));
    }
}

// =============================================================================
// f_strpart
// =============================================================================

/// "strpart()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strpart"]
pub unsafe extern "C" fn rs_f_strpart(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let mut error = false;

        let tv0 = nvim_tv_idx(argvars, 0);
        let p = tv_get_string(tv0);
        let slen = libc::strlen(p) as VarNumber;

        let tv1 = nvim_tv_idx(argvars, 1);
        let mut n = tv_get_number_chk(tv1, &raw mut error);
        let mut len: VarNumber;
        if error {
            len = 0;
        } else {
            let tv2 = nvim_tv_idx(argvars, 2);
            if nvim_tv_get_type(tv2) != VAR_UNKNOWN {
                len = tv_get_number(tv2);
            } else {
                len = slen - n;
            }
        }

        // Clamp
        if n < 0 {
            len += n;
            n = 0;
        } else if n > slen {
            n = slen;
        }
        if len < 0 {
            len = 0;
        } else if n + len > slen {
            len = slen - n;
        }

        let tv2 = nvim_tv_idx(argvars, 2);
        let tv3 = nvim_tv_idx(argvars, 3);
        if nvim_tv_get_type(tv2) != VAR_UNKNOWN && nvim_tv_get_type(tv3) != VAR_UNKNOWN {
            // length in characters
            let mut off = n;
            let mut remaining = len;
            while off < slen && remaining > 0 {
                off += i64::from(utfc_ptr2len(p.add(off as usize)));
                remaining -= 1;
            }
            len = off - n;
        }

        tv_set_vstring_owned(rettv, xmemdupz(p.add(n as usize), len as usize));
    }
}

// =============================================================================
// f_strridx
// =============================================================================

/// "strridx()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strridx"]
pub unsafe extern "C" fn rs_f_strridx(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let mut buf = [0u8; NUMBUFLEN];
        let tv1 = nvim_tv_idx(argvars, 1);
        let tv0 = nvim_tv_idx(argvars, 0);
        let needle = tv_get_string_chk(tv1);
        let haystack = tv_get_string_buf_chk(tv0, buf.as_mut_ptr().cast());

        nvim_tv_set_number(rettv, -1);
        if needle.is_null() || haystack.is_null() {
            return;
        }

        let haystack_len = libc::strlen(haystack);
        let tv2 = nvim_tv_idx(argvars, 2);
        let end_idx: isize = if nvim_tv_get_type(tv2) != VAR_UNKNOWN {
            let idx = tv_get_number_chk(tv2, std::ptr::null_mut()) as isize;
            if idx < 0 {
                return;
            }
            idx
        } else {
            haystack_len as isize
        };

        let lastmatch: *const c_char;
        if *needle == 0 {
            // Empty string matches past the end
            lastmatch = haystack.add(end_idx as usize);
        } else {
            let mut lm: *const c_char = std::ptr::null();
            let mut rest = haystack;
            loop {
                if *rest == 0 {
                    break;
                }
                let found = libc::strstr(rest, needle);
                if found.is_null() || found > haystack.add(end_idx as usize) as *mut c_char {
                    break;
                }
                lm = found;
                rest = found.add(1);
            }
            lastmatch = lm;
        }

        if !lastmatch.is_null() {
            nvim_tv_set_number(rettv, lastmatch.offset_from(haystack) as i64);
        }
    }
}

// =============================================================================
// f_strtrans
// =============================================================================

/// "strtrans()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_strtrans"]
pub unsafe extern "C" fn rs_f_strtrans(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let tv0 = nvim_tv_idx(argvars, 0);
        let s = tv_get_string(tv0);
        tv_set_vstring_owned(rettv, transstr(s, true));
    }
}

// =============================================================================
// f_utf16idx
// =============================================================================

/// "utf16idx()" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_utf16idx"]
pub unsafe extern "C" fn rs_f_utf16idx(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        nvim_tv_set_number(rettv, -1);

        if tv_check_for_string_arg(argvars, 0) == FAIL
            || tv_check_for_opt_number_arg(argvars, 1) == FAIL
            || tv_check_for_opt_bool_arg(argvars, 2) == FAIL
        {
            return;
        }
        let tv2 = nvim_tv_idx(argvars, 2);
        if nvim_tv_get_type(tv2) != VAR_UNKNOWN {
            if tv_check_for_opt_bool_arg(argvars, 3) == FAIL {
                return;
            }
        }

        let tv0 = nvim_tv_idx(argvars, 0);
        let tv1 = nvim_tv_idx(argvars, 1);
        let str = tv_get_string_chk(tv0);
        let idx = tv_get_number_chk(tv1, std::ptr::null_mut());
        if str.is_null() || idx < 0 {
            return;
        }

        let mut countcc: VarNumber = 0;
        let mut charidx: VarNumber = 0;
        if nvim_tv_get_type(tv2) != VAR_UNKNOWN {
            countcc = tv_get_bool(tv2);
            let tv3 = nvim_tv_idx(argvars, 3);
            if nvim_tv_get_type(tv3) != VAR_UNKNOWN {
                charidx = tv_get_bool(tv3);
            }
        }

        let mut p = str;
        let mut len: c_int = 0;
        let mut utf16idx_val: c_int = 0;
        let mut idx = idx;
        loop {
            let cond = if charidx != 0 {
                idx >= 0
            } else {
                p <= str.add(idx as usize)
            };
            if !cond {
                break;
            }
            if *p == 0 {
                let at_end = if charidx != 0 {
                    idx == 0
                } else {
                    p == str.add(idx as usize)
                };
                if at_end {
                    nvim_tv_set_number(rettv, i64::from(len));
                }
                return;
            }
            utf16idx_val = len;
            let clen = if countcc != 0 {
                utf_ptr2len(p)
            } else {
                utfc_ptr2len(p)
            };
            let c = if clen > 1 {
                utf_ptr2char(p)
            } else {
                c_int::from(*p as u8)
            };
            if c > 0xFFFF {
                len += 1;
            }
            p = p.add(if countcc != 0 {
                utf_ptr2len(p)
            } else {
                utfc_ptr2len(p)
            } as usize);
            if charidx != 0 {
                idx -= 1;
            }
            len += 1;
        }
        nvim_tv_set_number(rettv, i64::from(utf16idx_val));
    }
}

// =============================================================================
// f_tolower / f_toupper
// =============================================================================

/// "tolower(string)" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_tolower"]
pub unsafe extern "C" fn rs_f_tolower(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let tv0 = nvim_tv_idx(argvars, 0);
        let s = tv_get_string(tv0);
        tv_set_vstring_owned(rettv, strcase_save(s, false));
    }
}

/// "toupper(string)" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_toupper"]
pub unsafe extern "C" fn rs_f_toupper(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let tv0 = nvim_tv_idx(argvars, 0);
        let s = tv_get_string(tv0);
        tv_set_vstring_owned(rettv, strcase_save(s, true));
    }
}

// =============================================================================
// f_tr
// =============================================================================

/// "tr(string, fromstr, tostr)" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_tr"]
pub unsafe extern "C" fn rs_f_tr(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let mut buf = [0u8; NUMBUFLEN];
        let mut buf2 = [0u8; NUMBUFLEN];

        let tv0 = nvim_tv_idx(argvars, 0);
        let tv1 = nvim_tv_idx(argvars, 1);
        let tv2 = nvim_tv_idx(argvars, 2);
        let in_str = tv_get_string(tv0);
        let fromstr = tv_get_string_buf_chk(tv1, buf.as_mut_ptr().cast());
        let tostr = tv_get_string_buf_chk(tv2, buf2.as_mut_ptr().cast());

        // Default return: empty string
        *rettv.cast::<c_int>() = VAR_STRING;
        // rettv->vval.v_string = NULL; already zeroed by type setter

        if fromstr.is_null() || tostr.is_null() {
            return;
        }

        let mut ga = GArray::zeroed();
        ga_init(&raw mut ga, 1, 80);

        let mut first = true;
        let mut in_str = in_str;
        while *in_str != 0 {
            let cpstr = in_str;
            let inlen = utfc_ptr2len(in_str);
            let mut idx: c_int = 0;
            let mut p = fromstr;
            let mut found = false;
            while *p != 0 {
                let fromlen = utfc_ptr2len(p);
                if fromlen == inlen && libc::strncmp(in_str, p, inlen as usize) == 0 {
                    // Find matching position in tostr
                    let mut q = tostr;
                    let mut tolen;
                    loop {
                        if *q == 0 {
                            // tostr shorter than fromstr
                            semsg_invarg2(fromstr);
                            ga_clear(&raw mut ga);
                            return;
                        }
                        tolen = utfc_ptr2len(q);
                        if idx == 0 {
                            // copy tolen bytes from q into ga
                            ga_grow(&raw mut ga, tolen);
                            let base = (ga.ga_data as *mut c_char).add(ga.ga_len as usize);
                            std::ptr::copy_nonoverlapping(q, base, tolen as usize);
                            ga.ga_len += tolen;
                            found = true;
                            break;
                        }
                        idx -= 1;
                        q = q.add(tolen as usize);
                    }
                    break;
                }
                idx += 1;
                p = p.add(fromlen as usize);
            }

            if !found {
                // Char not in fromstr
                if first {
                    first = false;
                    // Verify fromstr and tostr have same char count
                    let mut q = tostr;
                    while *q != 0 {
                        let tolen = utfc_ptr2len(q);
                        q = q.add(tolen as usize);
                        idx -= 1;
                    }
                    if idx != 0 {
                        semsg_invarg2(fromstr);
                        ga_clear(&raw mut ga);
                        return;
                    }
                }
                // Copy original char
                ga_grow(&raw mut ga, inlen);
                let base = (ga.ga_data as *mut c_char).add(ga.ga_len as usize);
                std::ptr::copy_nonoverlapping(cpstr, base, inlen as usize);
                ga.ga_len += inlen;
            }

            in_str = in_str.add(inlen as usize);
        }

        // Add NUL terminator
        ga_append(&raw mut ga, 0);
        tv_set_vstring_owned(rettv, ga.ga_data as *mut c_char);
    }
}

// =============================================================================
// f_trim
// =============================================================================

/// "trim({expr})" function
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_trim"]
pub unsafe extern "C" fn rs_f_trim(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let mut buf1 = [0u8; NUMBUFLEN];
        let mut buf2 = [0u8; NUMBUFLEN];

        *rettv.cast::<c_int>() = VAR_STRING;

        let tv0 = nvim_tv_idx(argvars, 0);
        let mut head = tv_get_string_buf_chk(tv0, buf1.as_mut_ptr().cast());
        if head.is_null() {
            return;
        }

        if tv_check_for_opt_string_arg(argvars, 1) == FAIL {
            return;
        }

        let tv1 = nvim_tv_idx(argvars, 1);
        let mut mask: *const c_char = std::ptr::null();
        let mut dir: c_int = 0;

        if nvim_tv_get_type(tv1) == VAR_STRING {
            mask = tv_get_string_buf_chk(tv1, buf2.as_mut_ptr().cast());
            if !mask.is_null() && *mask == 0 {
                mask = std::ptr::null();
            }

            let tv2 = nvim_tv_idx(argvars, 2);
            if nvim_tv_get_type(tv2) != VAR_UNKNOWN {
                let mut error = false;
                dir = tv_get_number_chk(tv2, &raw mut error) as c_int;
                if error {
                    return;
                }
                if dir < 0 || dir > 2 {
                    let err_s = tv_get_string(tv2);
                    semsg_invarg2(err_s);
                    return;
                }
            }
        }

        if dir == 0 || dir == 1 {
            // Trim leading characters
            while *head != 0 {
                let c1 = utf_ptr2char(head);
                if mask.is_null() {
                    if c1 > b' ' as c_int && c1 != 0xa0 {
                        break;
                    }
                } else {
                    let mut p = mask;
                    let mut found_in_mask = false;
                    while *p != 0 {
                        if c1 == utf_ptr2char(p) {
                            found_in_mask = true;
                            break;
                        }
                        p = p.add(utfc_ptr2len(p) as usize);
                    }
                    if !found_in_mask {
                        break;
                    }
                }
                // MB_PTR_ADV: advance by utfc_ptr2len
                head = head.add(utfc_ptr2len(head) as usize);
            }
        }

        let head_len = libc::strlen(head);
        let mut tail = head.add(head_len);

        if dir == 0 || dir == 2 {
            // Trim trailing characters
            while tail > head {
                // MB_PTR_BACK: find previous char start by scanning back
                let prev = mb_ptr_back(head, tail);
                let c1 = utf_ptr2char(prev);
                if mask.is_null() {
                    if c1 > b' ' as c_int && c1 != 0xa0 {
                        break;
                    }
                } else {
                    let mut p = mask;
                    let mut found_in_mask = false;
                    while *p != 0 {
                        if c1 == utf_ptr2char(p) {
                            found_in_mask = true;
                            break;
                        }
                        p = p.add(utfc_ptr2len(p) as usize);
                    }
                    if !found_in_mask {
                        break;
                    }
                }
                tail = prev;
            }
        }

        tv_set_vstring_owned(rettv, xstrnsave(head, tail.offset_from(head) as usize));
    }
}

/// Implement MB_PTR_BACK: find the start of the character before `ptr` in `base..ptr`.
/// Scans backward to find a non-continuation byte (first byte of UTF-8 sequence).
unsafe fn mb_ptr_back(base: *const c_char, ptr: *const c_char) -> *const c_char {
    unsafe {
        let mut p = ptr.sub(1);
        // UTF-8 continuation bytes are 0x80-0xBF (10xxxxxx)
        while p > base && (*p as u8) >= 0x80 && (*p as u8) < 0xC0 {
            p = p.sub(1);
        }
        p
    }
}
