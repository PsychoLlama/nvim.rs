//! Option conversion functions for VimL variables.
//!
//! Phase 1: Migrated from `src/nvim/eval/vars.c`.
//!
//! Functions:
//! - `rs_tv_to_optval`: Convert typval_T to OptVal
//! - `rs_optval_as_tv`: Convert OptVal to typval_T (output pointer)
//! - `rs_set_option_from_tv`: Set an option from a typval value
//! - `rs_set_cmdarg`: Build v:cmdarg string from exarg_T fields

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unused_assignments)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::if_not_else)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::manual_c_str_literals)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// =============================================================================
// Type Aliases
// =============================================================================

/// OptIndex: option index (matches C's OptIndex = int)
type OptIndex = c_int;

/// OptInt: option integer value (matches C's OptInt = int64_t)
type OptInt = i64;

/// Opaque pointer to typval_T
type TvPtr = *mut c_void;

/// Opaque pointer to exarg_T
type EapPtr = *mut c_void;

// =============================================================================
// Constants
// =============================================================================

/// kOptInvalid: invalid option index
const K_OPT_INVALID: OptIndex = -1;

/// kOptValTypeNil
const K_OPT_VAL_TYPE_NIL: c_int = -1;
/// kOptValTypeBoolean
const K_OPT_VAL_TYPE_BOOLEAN: c_int = 0;
/// kOptValTypeNumber
const K_OPT_VAL_TYPE_NUMBER: c_int = 1;
/// kOptValTypeString
const K_OPT_VAL_TYPE_STRING: c_int = 2;

/// VAR_NUMBER
const VAR_NUMBER: c_int = 1;
/// VAR_STRING
const VAR_STRING: c_int = 2;
/// VAR_FUNC
const VAR_FUNC: c_int = 3;
/// VAR_BOOL
const VAR_BOOL: c_int = 7;
/// VAR_SPECIAL
const VAR_SPECIAL: c_int = 8;
/// VAR_PARTIAL
const VAR_PARTIAL: c_int = 9;
/// kSpecialVarNull
const K_SPECIAL_VAR_NULL: c_int = 0;

/// kNone (TriState)
const K_NONE: c_int = -1;
/// kFalse (TriState)
const K_FALSE: c_int = 0;
/// kTrue (TriState)
const K_TRUE: c_int = 1;

/// kOptFlagFunc: option can be set to a function ref or lambda
const K_OPT_FLAG_FUNC: u32 = 1 << 25;

/// OPT_LOCAL: use local value
const OPT_LOCAL: c_int = 0x02;

/// FORCE_BIN: ++bin flag
const FORCE_BIN: c_int = 1;
/// FORCE_NOBIN: ++nobin flag
const FORCE_NOBIN: c_int = 2;

/// BAD_KEEP: keep bad chars
const BAD_KEEP: c_int = -1;
/// BAD_DROP: drop bad chars
const BAD_DROP: c_int = -2;

// =============================================================================
// OptVal / NvimString representation (matches C layout)
// =============================================================================

/// Nvim String type (matches api/private/defs.h NvimString / String)
#[repr(C)]
#[derive(Clone, Copy)]
struct NvimString {
    data: *mut c_char,
    size: usize,
}

/// Union data for OptVal
#[repr(C)]
#[derive(Clone, Copy)]
union OptValData {
    boolean: c_int,
    number: OptInt,
    string: NvimString,
}

/// Option value (matches OptVal in option_defs.h)
/// Layout: { type_: i32, data: union { boolean: i32, number: i64, string: {*mut c_char, usize} } }
#[repr(C)]
#[derive(Clone, Copy)]
pub struct OptVal {
    type_: c_int,
    data: OptValData,
}

impl OptVal {
    fn nil() -> Self {
        Self {
            type_: K_OPT_VAL_TYPE_NIL,
            data: OptValData { boolean: 0 },
        }
    }

    fn boolean(val: c_int) -> Self {
        Self {
            type_: K_OPT_VAL_TYPE_BOOLEAN,
            data: OptValData { boolean: val },
        }
    }

    fn number(val: OptInt) -> Self {
        Self {
            type_: K_OPT_VAL_TYPE_NUMBER,
            data: OptValData { number: val },
        }
    }

    /// Create a string OptVal that takes ownership of `data`.
    fn cstr_as(data: *mut c_char) -> Self {
        let size = if data.is_null() {
            0
        } else {
            unsafe { strlen(data as *const c_char) }
        };
        Self {
            type_: K_OPT_VAL_TYPE_STRING,
            data: OptValData {
                string: NvimString { data, size },
            },
        }
    }

    /// Create a string OptVal by copying `src` (as cstr_to_optval / CSTR_TO_OPTVAL).
    unsafe fn cstr_to(src: *const c_char) -> Self {
        if src.is_null() {
            return Self::cstr_as(ptr::null_mut());
        }
        let len = strlen(src);
        let data = xmemdupz(src, len) as *mut c_char;
        Self::cstr_as(data)
    }
}

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // Option functions
    fn find_option(name: *const c_char) -> OptIndex;
    fn option_has_type(opt_idx: OptIndex, val_type: c_int) -> bool;
    fn set_option_value_handle_tty(
        name: *const c_char,
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: c_int,
    ) -> *const c_char;
    fn nvim_get_option_flags(opt_idx: OptIndex) -> u32;

    // Typval functions
    fn nvim_tv_get_type(tv: TvPtr) -> c_int;
    fn nvim_tv_get_string_val(tv: TvPtr) -> *mut c_char;
    fn nvim_tv_set_type(tv: TvPtr, vtype: c_int);
    fn nvim_tv_set_number(tv: TvPtr, n: i64);
    fn nvim_tv_set_bool(tv: TvPtr, val: c_int);
    fn nvim_tv_set_special(tv: TvPtr, val: c_int);
    fn nvim_tv_set_string_val(tv: TvPtr, s: *mut c_char);

    // typval query functions
    fn tv_get_number_chk(tv: TvPtr, err: *mut bool) -> i64;
    fn tv_get_bool_chk(tv: TvPtr, err: *mut bool) -> i64;
    fn tv_get_string_buf_chk(tv: TvPtr, buf: *mut c_char) -> *const c_char;
    fn encode_tv2string(tv: TvPtr, len: *mut usize) -> *mut c_char;

    // rs_is_tty_option: check if option name is a tty option
    fn rs_is_tty_option(name: *const c_char) -> c_int;

    // rs_optval_free: free an OptVal's memory
    fn rs_optval_free(o: OptVal);

    // vimvar accessor for v:cmdarg
    fn get_vim_var_tv(idx: c_int) -> TvPtr;

    // message functions
    fn semsg(fmt: *const c_char, ...) -> c_int;
    fn emsg(msg: *const c_char) -> c_int;

    // memory
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xmemdupz(src: *const c_char, len: usize) -> *mut c_void;

    // string
    fn strlen(s: *const c_char) -> usize;
    fn snprintf(s: *mut c_char, maxlen: usize, fmt: *const c_char, ...) -> c_int;
}

// NUMBUFLEN matches C #define (30 bytes for number as string)
const NUMBUFLEN: usize = 30;

// =============================================================================
// Error message strings (static C strings)
// =============================================================================

static E_STRING_REQUIRED: &[u8] = b"E928: String required\0";
static E_UNKNOWN_OPTION: &[u8] = b"E355: Unknown option: %s\0";
static E_521: &[u8] = b"E521: Number required: &%s = '%s'\0";

// =============================================================================
// VV_CMDARG index
// =============================================================================
const VV_CMDARG: c_int = 22;

// =============================================================================
// Implementation
// =============================================================================

/// Convert typval to option value for a particular option.
///
/// # Safety
/// `tv` must be a valid non-null typval_T pointer.
/// `option` must be a valid null-terminated C string.
/// `error` may be null; if non-null it is set to true on error.
///
/// Returns an OptVal that must be freed by caller via `rs_optval_free`.
/// Returns a nil OptVal on invalid option.
#[no_mangle]
pub unsafe extern "C" fn rs_tv_to_optval(
    tv: TvPtr,
    opt_idx: OptIndex,
    option: *const c_char,
    error: *mut bool,
) -> OptVal {
    let mut err = false;
    let is_tty_opt = rs_is_tty_option(option) != 0;
    let option_has_bool = !is_tty_opt && option_has_type(opt_idx, K_OPT_VAL_TYPE_BOOLEAN);
    let option_has_num = !is_tty_opt && option_has_type(opt_idx, K_OPT_VAL_TYPE_NUMBER);
    let option_has_str = is_tty_opt || option_has_type(opt_idx, K_OPT_VAL_TYPE_STRING);

    let opt_flags = if !is_tty_opt {
        nvim_get_option_flags(opt_idx)
    } else {
        0
    };

    let value;

    let vtype_for_func = nvim_tv_get_type(tv);
    let is_func = vtype_for_func == VAR_FUNC || vtype_for_func == VAR_PARTIAL;
    if !is_tty_opt && (opt_flags & K_OPT_FLAG_FUNC) != 0 && is_func {
        // If the option can be set to a function reference or a lambda
        // and the passed value is a function reference, convert to name (string).
        let strval = encode_tv2string(tv, ptr::null_mut());
        err = strval.is_null();
        value = OptVal::cstr_as(strval);
    } else if option_has_bool || option_has_num {
        let n: i64 = if option_has_num {
            tv_get_number_chk(tv, &mut err)
        } else {
            tv_get_bool_chk(tv, &mut err) as i64
        };

        // This could be "0" or a non-number string. Check if it's actually a number.
        if !err && nvim_tv_get_type(tv) == VAR_STRING && n == 0 {
            let s = nvim_tv_get_string_val(tv);
            if !s.is_null() {
                // Skip leading zeros
                let mut idx = 0usize;
                while *s.add(idx) == b'0' as c_char {
                    idx += 1;
                }
                let c_after = *s.add(idx) as u8;
                if c_after != 0 || idx == 0 {
                    // Not a pure zero string
                    err = true;
                    semsg(E_521.as_ptr() as *const c_char, option, s);
                }
            }
        }

        value = if option_has_num {
            OptVal::number(n)
        } else {
            // TRISTATE_FROM_INT: 0 -> kFalse, >=1 -> kTrue, <0 -> kNone
            let tri = if n == 0 {
                K_FALSE
            } else if n >= 1 {
                K_TRUE
            } else {
                K_NONE
            };
            OptVal::boolean(tri)
        };
    } else if option_has_str {
        let vtype = nvim_tv_get_type(tv);
        // Avoid setting string option to a boolean or a special value.
        if vtype != VAR_BOOL && vtype != VAR_SPECIAL {
            let mut nbuf = [0i8; NUMBUFLEN];
            let strval = tv_get_string_buf_chk(tv, nbuf.as_mut_ptr());
            err = strval.is_null();
            value = OptVal::cstr_to(strval);
        } else if !is_tty_opt {
            err = true;
            emsg(E_STRING_REQUIRED.as_ptr() as *const c_char);
            value = OptVal::nil();
        } else {
            value = OptVal::nil();
        }
    } else {
        // This should never happen (the C code has abort() here)
        value = OptVal::nil();
    }

    if !error.is_null() {
        *error = err;
    }
    value
}

/// Convert an option value to typval_T.
///
/// Writes the result into `rettv` (which must be a valid non-null typval_T pointer).
/// The string data (if string) is NOT copied -- it is moved to the typval.
/// `value.data.string.data` ownership transfers to `rettv->vval.v_string`.
///
/// # Safety
/// `rettv` must be a valid non-null typval_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_optval_as_tv(value: OptVal, numbool: bool, rettv: TvPtr) {
    // Initialize to v:null
    nvim_tv_set_type(rettv, VAR_SPECIAL);
    nvim_tv_set_special(rettv, K_SPECIAL_VAR_NULL);

    match value.type_ {
        K_OPT_VAL_TYPE_NIL => {
            // Leave as v:null
        }
        K_OPT_VAL_TYPE_BOOLEAN => {
            let b = value.data.boolean;
            if numbool {
                nvim_tv_set_type(rettv, VAR_NUMBER);
                nvim_tv_set_number(rettv, b as i64);
            } else if b != K_NONE {
                nvim_tv_set_type(rettv, VAR_BOOL);
                nvim_tv_set_bool(rettv, if b == K_TRUE { 1 } else { 0 });
            }
            // b == K_NONE: leave as v:null
        }
        K_OPT_VAL_TYPE_NUMBER => {
            nvim_tv_set_type(rettv, VAR_NUMBER);
            nvim_tv_set_number(rettv, value.data.number);
        }
        K_OPT_VAL_TYPE_STRING => {
            nvim_tv_set_type(rettv, VAR_STRING);
            // Transfer ownership of the string data to the typval.
            nvim_tv_set_string_val(rettv, value.data.string.data);
        }
        _ => {
            // Unknown type, leave as v:null
        }
    }
}

/// Set option "varname" to the value of typval_T "varp" for current buffer/window.
///
/// # Safety
/// `varname` must be a valid null-terminated C string.
/// `varp` must be a valid non-null typval_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_set_option_from_tv(varname: *const c_char, varp: TvPtr) {
    let opt_idx = find_option(varname);
    if opt_idx == K_OPT_INVALID {
        semsg(E_UNKNOWN_OPTION.as_ptr() as *const c_char, varname);
        return;
    }

    let mut error = false;
    let value = rs_tv_to_optval(varp, opt_idx, varname, &mut error);

    if !error {
        let errmsg = set_option_value_handle_tty(varname, opt_idx, value, OPT_LOCAL);
        if !errmsg.is_null() {
            emsg(errmsg);
        }
    } else {
        rs_optval_free(value);
    }
}

/// Build v:cmdarg string from exarg_T fields.
///
/// If eap is non-null: compute new v:cmdarg from eap fields, store it,
/// return old v:cmdarg string (caller must not free it -- it's already stored or will be
/// freed by the next call).
///
/// If eap is null: restore v:cmdarg to oldarg (the value from the paired forward call),
/// free the current value, return NULL.
///
/// Must always be called in pairs!
///
/// # Safety
/// `eap` may be null (triggers restore path).
/// `oldarg` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_set_cmdarg(eap: EapPtr, oldarg: *mut c_char) -> *mut c_char {
    let tv = get_vim_var_tv(VV_CMDARG);
    let oldval = nvim_tv_get_string_val(tv);

    if eap.is_null() {
        // Restore path: free current value, restore oldarg.
        xfree(oldval as *mut c_void);
        nvim_tv_set_string_val(tv, oldarg);
        return ptr::null_mut();
    }

    // Access eap fields via known-offset struct view.
    let eap_fields = &*(eap as *const ExargFields);
    let force_bin = eap_fields.force_bin;
    let read_edit = eap_fields.read_edit;
    let force_ff = eap_fields.force_ff;
    let force_enc = eap_fields.force_enc;
    let bad_char = eap_fields.bad_char;
    let mkdir_p = eap_fields.mkdir_p;
    let cmd = eap_fields.cmd;

    // Calculate total buffer length needed
    let mut len: usize = 0;

    if force_bin == FORCE_BIN {
        len += 6; // " ++bin"
    } else if force_bin == FORCE_NOBIN {
        len += 8; // " ++nobin"
    }

    if read_edit != 0 {
        len += 7; // " ++edit"
    }

    if force_ff != 0 {
        len += 10; // " ++ff=xxxx" (max: " ++ff=unix")
    }

    if force_enc != 0 && !cmd.is_null() {
        len += strlen(cmd.add(force_enc as usize) as *const c_char) + 7; // " ++enc=<name>"
    }

    if bad_char != 0 {
        len += 7 + 4; // " ++bad=" + up to 4 chars ("keep" or "drop")
    }

    if mkdir_p != 0 {
        len += 4; // " ++p"
    }

    let newval_len = len + 1;
    let newval = xmalloc(newval_len) as *mut c_char;
    if newval.is_null() {
        // Allocation failure - restore
        xfree(oldval as *mut c_void);
        nvim_tv_set_string_val(tv, oldarg);
        return ptr::null_mut();
    }

    let mut xlen: usize = 0;
    let mut failed = false;

    macro_rules! do_snprintf {
        ($fmt:expr $(, $arg:expr)*) => {{
            if !failed {
                let rc = snprintf(newval.add(xlen), newval_len - xlen,
                                  $fmt as *const u8 as *const c_char $(, $arg)*);
                if rc < 0 {
                    failed = true;
                } else {
                    xlen += rc as usize;
                }
            }
        }};
    }

    // force_bin
    if force_bin == FORCE_BIN {
        do_snprintf!(b" ++bin\0");
    } else if force_bin == FORCE_NOBIN {
        do_snprintf!(b" ++nobin\0");
    } else {
        *newval = 0;
    }

    // read_edit
    if read_edit != 0 {
        do_snprintf!(b" ++edit\0");
    }

    // force_ff
    if force_ff != 0 {
        let ff_name: *const c_char = if force_ff == b'u' as c_int {
            b"unix\0".as_ptr() as *const c_char
        } else if force_ff == b'd' as c_int {
            b"dos\0".as_ptr() as *const c_char
        } else {
            b"mac\0".as_ptr() as *const c_char
        };
        do_snprintf!(b" ++ff=%s\0", ff_name);
    }

    // force_enc
    if force_enc != 0 && !cmd.is_null() {
        let enc_str = cmd.add(force_enc as usize);
        do_snprintf!(b" ++enc=%s\0", enc_str);
    }

    // bad_char
    if bad_char == BAD_KEEP {
        do_snprintf!(b" ++bad=keep\0");
    } else if bad_char == BAD_DROP {
        do_snprintf!(b" ++bad=drop\0");
    } else if bad_char != 0 {
        do_snprintf!(b" ++bad=%c\0", bad_char);
    }

    // mkdir_p
    if mkdir_p != 0 {
        do_snprintf!(b" ++p\0");
    }

    if failed {
        xfree(newval as *mut c_void);
        xfree(oldval as *mut c_void);
        nvim_tv_set_string_val(tv, oldarg);
        return ptr::null_mut();
    }

    nvim_tv_set_string_val(tv, newval);
    oldval
}

// =============================================================================
// ExargFields: partial layout of exarg_T for set_cmdarg field access
// =============================================================================

/// Partial layout of exarg_T matching the fields needed by rs_set_cmdarg.
/// Must match `struct exarg` in ex_cmds_defs.h at these exact byte offsets.
///
/// Field offsets from ExargT in ex_eval/src/lib.rs:
///   arg: 0, args: 8, arglens: 16, argc: 24, nextcmd: 32, cmd: 40, cmdlinep: 48,
///   cmdline_tofree: 56, cmdidx: 64, argt: 68, skip: 72, forceit: 76,
///   addr_count: 80, line1: 84, line2: 88, addr_type: 92, flags: 96,
///   _padding_flags: 100, do_ecmd_cmd: 104, do_ecmd_lnum: 112, append: 116,
///   usefilter: 120, amount: 124, regname: 128, force_bin: 132, read_edit: 136,
///   mkdir_p: 140, force_ff: 144, force_enc: 148, bad_char: 152
#[repr(C)]
struct ExargFields {
    _arg: *mut c_char,            // offset 0
    _args: *mut *mut c_char,      // offset 8
    _arglens: *mut usize,         // offset 16
    _argc: usize,                 // offset 24
    _nextcmd: *mut c_char,        // offset 32
    cmd: *mut c_char,             // offset 40
    _cmdlinep: *mut *mut c_char,  // offset 48
    _cmdline_tofree: *mut c_char, // offset 56
    _cmdidx: c_int,               // offset 64
    _argt: u32,                   // offset 68
    _skip: c_int,                 // offset 72
    _forceit: c_int,              // offset 76
    _addr_count: c_int,           // offset 80
    _line1: i32,                  // offset 84
    _line2: i32,                  // offset 88
    _addr_type: c_int,            // offset 92
    _flags: c_int,                // offset 96
    _padding_flags: [u8; 4],      // offset 100
    _do_ecmd_cmd: *mut c_char,    // offset 104
    _do_ecmd_lnum: i32,           // offset 112
    _append: c_int,               // offset 116
    _usefilter: c_int,            // offset 120
    _amount: c_int,               // offset 124
    _regname: c_int,              // offset 128
    force_bin: c_int,             // offset 132
    read_edit: c_int,             // offset 136
    mkdir_p: c_int,               // offset 140
    force_ff: c_int,              // offset 144
    force_enc: c_int,             // offset 148
    bad_char: c_int,              // offset 152
}
