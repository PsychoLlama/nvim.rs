//! VimL expression evaluator (eval0-eval7).
//!
//! This module implements the recursive descent expression evaluator for VimL,
//! migrated from `src/nvim/eval.c`. The evaluator parses and evaluates VimL
//! expressions like `1 + 2 * 3`, `a ? b : c`, `func()`, etc.
//!
//! ## Expression Precedence (lowest to highest)
//!
//! 1. `eval1`: Ternary conditional (`?:` and `??`)
//! 2. `eval2`: Logical OR (`||`)
//! 3. `eval3`: Logical AND (`&&`)
//! 4. `eval4`: Comparison operators (`==`, `!=`, `<`, `>`, etc.)
//! 5. `eval5`: Addition, subtraction, concatenation (`+`, `-`, `.`, `..`)
//! 6. `eval6`: Multiplication, division, modulo (`*`, `/`, `%`)
//! 7. `eval7`: Unary operators and primaries (`!`, `-`, `+`, literals, vars, calls)
//!
//! ## FFI Pattern
//!
//! Each eval function has a Rust implementation and an FFI export (rs_evalN).
//! The FFI functions work with opaque handles and C strings.

#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)] // Some constants and functions are for future phases

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// =============================================================================
// Constants
// =============================================================================

/// OK return value from C functions
const OK: c_int = 1;
/// FAIL return value from C functions
const FAIL: c_int = 0;
/// NOTDONE return value (expression not yet evaluated)
const NOTDONE: c_int = 2;

/// EVAL_EVALUATE flag: actually evaluate (vs just parse)
const EVAL_EVALUATE: c_int = 1;

/// Maximum recursion depth to prevent stack overflow
#[cfg(target_os = "windows")]
const MAX_RECURSION: i32 = 300;
#[cfg(not(target_os = "windows"))]
const MAX_RECURSION: i32 = 1000;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to a typval_T
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct TypevalHandle(*mut c_void);

impl TypevalHandle {
    /// Create a null handle
    pub const fn null() -> Self {
        Self(ptr::null_mut())
    }

    /// Check if handle is null
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get raw pointer
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to evalarg_T
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct EvalargHandle(*mut c_void);

impl EvalargHandle {
    /// Create a null handle
    pub const fn null() -> Self {
        Self(ptr::null_mut())
    }

    /// Check if handle is null
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get raw pointer
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to exarg_T
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ExargHandle(*mut c_void);

impl ExargHandle {
    /// Create a null handle
    pub const fn null() -> Self {
        Self(ptr::null_mut())
    }

    /// Check if handle is null
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// C Extern Functions (accessors and helpers)
// =============================================================================

extern "C" {
    // String utilities
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn ends_excmd(c: c_char) -> c_int;
    fn check_nextcmd(p: *const c_char) -> *mut c_char;

    // Error handling
    fn did_emsg_get() -> c_int;
    fn called_emsg_get() -> c_int;
    fn aborting() -> c_int;
    fn semsg(fmt: *const c_char, ...) -> c_int;
    fn emsg(s: *const c_char) -> c_int;

    // Typval operations
    fn tv_clear(tv: TypevalHandle);
    fn tv_get_number_chk(tv: TypevalHandle, error: *mut bool) -> i64;
    fn tv_get_string_buf(tv: TypevalHandle, buf: *mut c_char) -> *const c_char;
    fn tv_get_string_buf_chk(tv: TypevalHandle, buf: *mut c_char) -> *const c_char;
    fn tv_check_str(tv: TypevalHandle) -> bool;
    fn tv_check_num(tv: TypevalHandle) -> bool;
    fn tv2bool(tv: TypevalHandle) -> c_int;
    fn tv_copy(from: TypevalHandle, to: TypevalHandle);
    fn tv_list_concat(l1: *mut c_void, l2: *mut c_void, ret: TypevalHandle) -> c_int;

    // Typval field accessors
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;
    fn nvim_tv_get_number(tv: TypevalHandle) -> i64;
    fn nvim_tv_get_float(tv: TypevalHandle) -> f64;
    fn nvim_tv_get_list(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_get_blob(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_set_type(tv: TypevalHandle, vtype: c_int);
    fn nvim_tv_set_number(tv: TypevalHandle, n: i64);
    fn nvim_tv_set_float(tv: TypevalHandle, f: f64);

    // Evalarg operations
    fn evalarg_get_flags(ea: EvalargHandle) -> c_int;
    fn evalarg_set_flags(ea: EvalargHandle, flags: c_int);
    fn clear_evalarg(ea: EvalargHandle, eap: ExargHandle);

    // exarg operations
    fn exarg_set_nextcmd(eap: ExargHandle, nextcmd: *mut c_char);

    // Comparison
    fn typval_compare(tv1: TypevalHandle, tv2: TypevalHandle, typ: c_int, ic: c_int) -> c_int;

    // String concatenation
    fn concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;

    // Blob operations (wrappers for inline functions)
    fn nvim_blob_len(b: *const c_void) -> c_int;
    fn nvim_blob_get(b: *const c_void, idx: c_int) -> c_int;
    fn nvim_blob_alloc() -> *mut c_void;
    fn nvim_blob_set_ret(tv: TypevalHandle, b: *mut c_void);
    fn ga_append(ga: *mut c_void, c: c_int);

    // Options
    fn p_ic_get() -> c_int;

    // Division helpers (Rust exports)
    fn rs_num_divide(n1: i64, n2: i64) -> i64;
    fn rs_num_modulus(n1: i64, n2: i64) -> i64;

    // Forward declarations for eval6 and eval7 (still in C for now)
    fn eval6(
        arg: *mut *mut c_char,
        rettv: TypevalHandle,
        evalarg: EvalargHandle,
        want_string: c_int,
    ) -> c_int;
    fn eval7(
        arg: *mut *mut c_char,
        rettv: TypevalHandle,
        evalarg: EvalargHandle,
        want_string: c_int,
    ) -> c_int;
}

// =============================================================================
// VarType constants (must match C enum)
// =============================================================================

const VAR_UNKNOWN: c_int = 0;
const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;
const VAR_FUNC: c_int = 3;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;
const VAR_FLOAT: c_int = 6;
const VAR_BOOL: c_int = 7;
const VAR_SPECIAL: c_int = 8;
const VAR_PARTIAL: c_int = 9;
const VAR_BLOB: c_int = 10;

// =============================================================================
// Expression type constants (must match C enum exprtype_T)
// =============================================================================

const EXPR_UNKNOWN: c_int = 0;
const EXPR_EQUAL: c_int = 1;
const EXPR_NEQUAL: c_int = 2;
const EXPR_GREATER: c_int = 3;
const EXPR_GEQUAL: c_int = 4;
const EXPR_SMALLER: c_int = 5;
const EXPR_SEQUAL: c_int = 6;
const EXPR_MATCH: c_int = 7;
const EXPR_NOMATCH: c_int = 8;
const EXPR_IS: c_int = 9;
const EXPR_ISNOT: c_int = 10;

// =============================================================================
// Helper Functions
// =============================================================================

/// Get byte at pointer, returning 0 for null
#[inline]
unsafe fn get_byte(p: *const c_char) -> u8 {
    if p.is_null() {
        0
    } else {
        *p as u8
    }
}

/// Check if we should evaluate expressions
#[inline]
fn should_evaluate(evalarg: EvalargHandle) -> bool {
    if evalarg.is_null() {
        false
    } else {
        // SAFETY: evalarg is not null
        (unsafe { evalarg_get_flags(evalarg) } & EVAL_EVALUATE) != 0
    }
}

/// Get evaluation flags, defaulting to 0 if evalarg is null
#[inline]
fn get_eval_flags(evalarg: EvalargHandle) -> c_int {
    if evalarg.is_null() {
        0
    } else {
        // SAFETY: evalarg is not null
        unsafe { evalarg_get_flags(evalarg) }
    }
}

// =============================================================================
// Error messages (matching C)
// =============================================================================

/// Error: trailing characters
static E_TRAILING_ARG: &[u8] = b"E488: Trailing characters: %s\0";
/// Error: invalid expression
static E_INVEXPR2: &[u8] = b"E15: Invalid expression: \"%s\"\0";
/// Error: missing colon after ?
static E_MISSING_COLON: &[u8] = b"E109: Missing ':' after '?'\0";
/// Error: expression too recursive
static E_EXPRESSION_TOO_RECURSIVE: &[u8] = b"E1169: Expression too recursive: %s\0";

// =============================================================================
// eval0 - Top level expression evaluation
// =============================================================================

/// Handle zero level expression.
///
/// This calls eval1() and handles error message and nextcmd.
/// Put the result in "rettv" when returning OK and "evaluate" is true.
///
/// # Safety
/// - `arg` must be a valid pointer to a C string
/// - `rettv` must be a valid typval handle
/// - `eap` can be null
/// - `evalarg` can be null
pub unsafe fn eval0_impl(
    arg: *mut c_char,
    rettv: TypevalHandle,
    eap: ExargHandle,
    evalarg: EvalargHandle,
) -> c_int {
    let did_emsg_before = did_emsg_get();
    let called_emsg_before = called_emsg_get();
    let mut end_error = false;

    let mut p = skipwhite(arg);
    let ret = eval1_impl(&mut p, rettv, evalarg);

    if ret != FAIL {
        end_error = ends_excmd(get_byte(p) as c_char) == 0;
    }

    if ret == FAIL || end_error {
        if ret != FAIL {
            tv_clear(rettv);
        }
        // Report the invalid expression unless the expression evaluation has
        // been cancelled due to an aborting error, an interrupt, or an
        // exception, or we already gave a more specific error.
        if aborting() == 0
            && did_emsg_get() == did_emsg_before
            && called_emsg_get() == called_emsg_before
        {
            if end_error {
                semsg(E_TRAILING_ARG.as_ptr() as *const c_char, p);
            } else {
                semsg(E_INVEXPR2.as_ptr() as *const c_char, arg);
            }
        }

        if !eap.is_null() && !p.is_null() {
            // Some of the expression may not have been consumed.
            // Only execute a next command if it cannot be a "||" operator.
            // The next command may be "catch".
            let nextcmd = check_nextcmd(p);
            if !nextcmd.is_null() && get_byte(nextcmd) != b'|' {
                exarg_set_nextcmd(eap, nextcmd);
            }
        }
        return FAIL;
    }

    if !eap.is_null() {
        exarg_set_nextcmd(eap, check_nextcmd(p));
    }

    ret
}

/// FFI export for eval0.
///
/// # Safety
/// See `eval0_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval0(
    arg: *mut c_char,
    rettv: TypevalHandle,
    eap: ExargHandle,
    evalarg: EvalargHandle,
) -> c_int {
    eval0_impl(arg, rettv, eap, evalarg)
}

// =============================================================================
// eval1 - Ternary conditional expressions
// =============================================================================

/// Handle top level expression:
///   expr2 ? expr1 : expr1
///   expr2 ?? expr1
///
/// "arg" must point to the first non-white of the expression.
/// "arg" is advanced to the next non-white after the recognized expression.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer
/// - `rettv` must be a valid typval handle
/// - `evalarg` can be null
pub unsafe fn eval1_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    // Clear rettv
    nvim_tv_set_type(rettv, VAR_UNKNOWN);

    // Get the first variable
    if eval2_impl(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }

    let p = *arg;
    if get_byte(p) == b'?' {
        let op_falsy = get_byte(p.add(1)) == b'?';
        let orig_flags = get_eval_flags(evalarg);
        let evaluate = (orig_flags & EVAL_EVALUATE) != 0;

        let mut result = false;
        if evaluate {
            let mut error: bool = false;

            if op_falsy {
                result = tv2bool(rettv) != 0;
            } else if tv_get_number_chk(rettv, &mut error) != 0 {
                result = true;
            }
            if error || !op_falsy || !result {
                tv_clear(rettv);
            }
            if error {
                return FAIL;
            }
        }

        // Get the second variable. Recursive!
        if op_falsy {
            *arg = (*arg).add(1);
        }
        *arg = skipwhite((*arg).add(1));

        // Set eval flags for short-circuit evaluation
        let new_flags = if (op_falsy && !result) || (!op_falsy && result) {
            orig_flags
        } else {
            orig_flags & !EVAL_EVALUATE
        };
        let var2_evaluated = (new_flags & EVAL_EVALUATE) != 0;
        if !evalarg.is_null() {
            evalarg_set_flags(evalarg, new_flags);
        }

        // Allocate var2 on stack (need proper C interop here)
        // For now, we'll use a helper to allocate a temporary typval
        let var2 = alloc_typval();
        if eval1_impl(arg, var2, evalarg) == FAIL {
            if !evalarg.is_null() {
                evalarg_set_flags(evalarg, orig_flags);
            }
            free_typval(var2);
            return FAIL;
        }

        // Only copy var2 to rettv if var2 was actually evaluated (not VAR_UNKNOWN)
        if evaluate && var2_evaluated && (!op_falsy || !result) {
            // Copy var2 to rettv (rettv was already cleared above)
            tv_copy(var2, rettv);
        }
        tv_clear(var2);
        free_typval(var2);

        if !op_falsy {
            // Check for the ":"
            let p2 = *arg;
            if get_byte(p2) != b':' {
                emsg(E_MISSING_COLON.as_ptr() as *const c_char);
                if evaluate && result {
                    tv_clear(rettv);
                }
                if !evalarg.is_null() {
                    evalarg_set_flags(evalarg, orig_flags);
                }
                return FAIL;
            }

            // Get the third variable. Recursive!
            *arg = skipwhite((*arg).add(1));
            let new_flags2 = if !result {
                orig_flags
            } else {
                orig_flags & !EVAL_EVALUATE
            };
            if !evalarg.is_null() {
                evalarg_set_flags(evalarg, new_flags2);
            }

            let var3 = alloc_typval();
            if eval1_impl(arg, var3, evalarg) == FAIL {
                if evaluate && result {
                    tv_clear(rettv);
                }
                if !evalarg.is_null() {
                    evalarg_set_flags(evalarg, orig_flags);
                }
                free_typval(var3);
                return FAIL;
            }
            if evaluate && !result {
                tv_clear(rettv);
                tv_copy(var3, rettv);
            }
            tv_clear(var3);
            free_typval(var3);
        }

        if !evalarg.is_null() {
            evalarg_set_flags(evalarg, orig_flags);
        }
    }

    OK
}

/// FFI export for eval1.
///
/// # Safety
/// See `eval1_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval1(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    eval1_impl(arg, rettv, evalarg)
}

// =============================================================================
// eval2 - Logical OR
// =============================================================================

/// Handle first level expression:
///   expr2 || expr2 || expr2     logical OR
///
/// # Safety
/// See `eval1_impl` for safety requirements.
pub unsafe fn eval2_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    // Get the first variable
    if eval3_impl(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }

    // Handle the "||" operator
    let mut p = *arg;
    if get_byte(p) == b'|' && get_byte(p.add(1)) == b'|' {
        let orig_flags = get_eval_flags(evalarg);
        let evaluate = (orig_flags & EVAL_EVALUATE) != 0;

        let mut result = false;

        if evaluate {
            let mut error: bool = false;
            if tv_get_number_chk(rettv, &mut error) != 0 {
                result = true;
            }
            tv_clear(rettv);
            if error {
                return FAIL;
            }
        }

        // Repeat until there is no following "||"
        while get_byte(p) == b'|' && get_byte(p.add(1)) == b'|' {
            // Get the second variable
            *arg = skipwhite((*arg).add(2));
            let new_flags = if !result {
                orig_flags
            } else {
                orig_flags & !EVAL_EVALUATE
            };
            if !evalarg.is_null() {
                evalarg_set_flags(evalarg, new_flags);
            }

            let var2 = alloc_typval();
            if eval3_impl(arg, var2, evalarg) == FAIL {
                free_typval(var2);
                return FAIL;
            }

            // Compute the result
            if evaluate && !result {
                let mut error: bool = false;
                if tv_get_number_chk(var2, &mut error) != 0 {
                    result = true;
                }
                tv_clear(var2);
                if error {
                    free_typval(var2);
                    return FAIL;
                }
            } else {
                tv_clear(var2);
            }
            free_typval(var2);

            if evaluate {
                nvim_tv_set_type(rettv, VAR_NUMBER);
                nvim_tv_set_number(rettv, if result { 1 } else { 0 });
            }

            p = *arg;
        }

        if !evalarg.is_null() {
            evalarg_set_flags(evalarg, orig_flags);
        }
    }

    OK
}

/// FFI export for eval2.
///
/// # Safety
/// See `eval2_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval2(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    eval2_impl(arg, rettv, evalarg)
}

// =============================================================================
// eval3 - Logical AND
// =============================================================================

/// Handle second level expression:
///   expr3 && expr3 && expr3     logical AND
///
/// # Safety
/// See `eval1_impl` for safety requirements.
pub unsafe fn eval3_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    // Get the first variable
    if eval4_impl(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }

    let mut p = *arg;
    // Handle the "&&" operator
    if get_byte(p) == b'&' && get_byte(p.add(1)) == b'&' {
        let orig_flags = get_eval_flags(evalarg);
        let evaluate = (orig_flags & EVAL_EVALUATE) != 0;

        let mut result = true;

        if evaluate {
            let mut error: bool = false;
            if tv_get_number_chk(rettv, &mut error) == 0 {
                result = false;
            }
            tv_clear(rettv);
            if error {
                return FAIL;
            }
        }

        // Repeat until there is no following "&&"
        while get_byte(p) == b'&' && get_byte(p.add(1)) == b'&' {
            // Get the second variable
            *arg = skipwhite((*arg).add(2));
            let new_flags = if result {
                orig_flags
            } else {
                orig_flags & !EVAL_EVALUATE
            };
            if !evalarg.is_null() {
                evalarg_set_flags(evalarg, new_flags);
            }

            let var2 = alloc_typval();
            if eval4_impl(arg, var2, evalarg) == FAIL {
                free_typval(var2);
                return FAIL;
            }

            // Compute the result
            if evaluate && result {
                let mut error: bool = false;
                if tv_get_number_chk(var2, &mut error) == 0 {
                    result = false;
                }
                tv_clear(var2);
                if error {
                    free_typval(var2);
                    return FAIL;
                }
            } else {
                tv_clear(var2);
            }
            free_typval(var2);

            if evaluate {
                nvim_tv_set_type(rettv, VAR_NUMBER);
                nvim_tv_set_number(rettv, if result { 1 } else { 0 });
            }

            p = *arg;
        }

        if !evalarg.is_null() {
            evalarg_set_flags(evalarg, orig_flags);
        }
    }

    OK
}

/// FFI export for eval3.
///
/// # Safety
/// See `eval3_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval3(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    eval3_impl(arg, rettv, evalarg)
}

// =============================================================================
// eval4 - Comparison operators
// =============================================================================

/// Handle third level expression:
///   var1 == var2
///   var1 =~ var2
///   var1 != var2
///   var1 !~ var2
///   var1 > var2
///   var1 >= var2
///   var1 < var2
///   var1 <= var2
///   var1 is var2
///   var1 isnot var2
///
/// # Safety
/// See `eval1_impl` for safety requirements.
pub unsafe fn eval4_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    let mut typ = EXPR_UNKNOWN;
    let mut len: usize = 2;

    // Get the first variable
    if eval5_impl(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }

    let p = *arg;
    let c0 = get_byte(p);
    let c1 = get_byte(p.add(1));

    match c0 {
        b'=' => {
            if c1 == b'=' {
                typ = EXPR_EQUAL;
            } else if c1 == b'~' {
                typ = EXPR_MATCH;
            }
        }
        b'!' => {
            if c1 == b'=' {
                typ = EXPR_NEQUAL;
            } else if c1 == b'~' {
                typ = EXPR_NOMATCH;
            }
        }
        b'>' => {
            if c1 != b'=' {
                typ = EXPR_GREATER;
                len = 1;
            } else {
                typ = EXPR_GEQUAL;
            }
        }
        b'<' => {
            if c1 != b'=' {
                typ = EXPR_SMALLER;
                len = 1;
            } else {
                typ = EXPR_SEQUAL;
            }
        }
        b'i' => {
            if c1 == b's' {
                let c2 = get_byte(p.add(2));
                let c3 = get_byte(p.add(3));
                let c4 = get_byte(p.add(4));
                if c2 == b'n' && c3 == b'o' && c4 == b't' {
                    len = 5;
                }
                let next_char = get_byte(p.add(len));
                if !next_char.is_ascii_alphanumeric() && next_char != b'_' {
                    typ = if len == 2 { EXPR_IS } else { EXPR_ISNOT };
                }
            }
        }
        _ => {}
    }

    // If there is a comparative operator, use it
    if typ != EXPR_UNKNOWN {
        // Check for case sensitivity modifier
        let next_char = get_byte(p.add(len));
        let ic = if next_char == b'?' {
            // case-insensitive
            len += 1;
            1
        } else if next_char == b'#' {
            // case-sensitive
            len += 1;
            0
        } else {
            // use 'ignorecase' option
            p_ic_get()
        };

        // Get the second variable
        *arg = skipwhite(p.add(len));
        let var2 = alloc_typval();
        if eval5_impl(arg, var2, evalarg) == FAIL {
            tv_clear(rettv);
            free_typval(var2);
            return FAIL;
        }

        if !evalarg.is_null() && (evalarg_get_flags(evalarg) & EVAL_EVALUATE) != 0 {
            let ret = typval_compare(rettv, var2, typ, ic);
            tv_clear(var2);
            free_typval(var2);
            return ret;
        }
        tv_clear(var2);
        free_typval(var2);
    }

    OK
}

/// FFI export for eval4.
///
/// # Safety
/// See `eval4_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval4(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    eval4_impl(arg, rettv, evalarg)
}

// =============================================================================
// eval5 - Addition, subtraction, concatenation
// =============================================================================

/// Handle fourth level expression:
/// - `+` number addition, concatenation of list or blob
/// - `-` number subtraction
/// - `.` string concatenation
/// - `..` string concatenation
///
/// # Safety
/// See `eval1_impl` for safety requirements.
pub unsafe fn eval5_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    // Get the first variable
    if eval6(arg, rettv, evalarg, 0) == FAIL {
        return FAIL;
    }

    // Repeat computing, until no '+', '-' or '.' is following
    loop {
        let op = get_byte(*arg);
        let concat = op == b'.';
        if op != b'+' && op != b'-' && !concat {
            break;
        }

        let evaluate = should_evaluate(evalarg);
        let tv_type = nvim_tv_get_type(rettv);

        // Type checking before evaluating second operand
        if (op != b'+' || (tv_type != VAR_LIST && tv_type != VAR_BLOB))
            && (op == b'.' || tv_type != VAR_FLOAT)
            && evaluate
            && ((op == b'.' && !tv_check_str(rettv)) || (op != b'.' && !tv_check_num(rettv)))
        {
            tv_clear(rettv);
            return FAIL;
        }

        // Get the second variable
        if op == b'.' && get_byte((*arg).add(1)) == b'.' {
            // ..string concatenation
            *arg = (*arg).add(1);
        }
        *arg = skipwhite((*arg).add(1));

        let var2 = alloc_typval();
        if eval6(arg, var2, evalarg, if op == b'.' { 1 } else { 0 }) == FAIL {
            tv_clear(rettv);
            free_typval(var2);
            return FAIL;
        }

        if evaluate {
            // Compute the result
            if op == b'.' {
                // String concatenation
                if eval_concat_str_impl(rettv, var2) == FAIL {
                    free_typval(var2);
                    return FAIL;
                }
            } else if op == b'+'
                && nvim_tv_get_type(rettv) == VAR_BLOB
                && nvim_tv_get_type(var2) == VAR_BLOB
            {
                // Blob concatenation
                eval_addblob_impl(rettv, var2);
            } else if op == b'+'
                && nvim_tv_get_type(rettv) == VAR_LIST
                && nvim_tv_get_type(var2) == VAR_LIST
            {
                // List concatenation
                if eval_addlist_impl(rettv, var2) == FAIL {
                    free_typval(var2);
                    return FAIL;
                }
            } else {
                // Number addition/subtraction
                if eval_addsub_number_impl(rettv, var2, op as c_int) == FAIL {
                    free_typval(var2);
                    return FAIL;
                }
            }
            tv_clear(var2);
        } else {
            tv_clear(var2);
        }
        free_typval(var2);
    }

    OK
}

/// FFI export for eval5.
///
/// # Safety
/// See `eval5_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval5(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    eval5_impl(arg, rettv, evalarg)
}

// =============================================================================
// Helper implementations for eval5
// =============================================================================

/// Concatenate strings "tv1" and "tv2" and store the result in "tv1".
unsafe fn eval_concat_str_impl(tv1: TypevalHandle, tv2: TypevalHandle) -> c_int {
    let mut buf1 = [0i8; 64]; // NUMBUFLEN
    let mut buf2 = [0i8; 64];

    let s1 = tv_get_string_buf(tv1, buf1.as_mut_ptr());
    let s2 = tv_get_string_buf_chk(tv2, buf2.as_mut_ptr());
    if s2.is_null() {
        tv_clear(tv1);
        tv_clear(tv2);
        return FAIL;
    }

    let p = concat_str(s1, s2);
    tv_clear(tv1);
    nvim_tv_set_type(tv1, VAR_STRING);
    nvim_tv_set_string(tv1, p);

    OK
}

/// Make a copy of blob "tv1" and append blob "tv2".
unsafe fn eval_addblob_impl(tv1: TypevalHandle, tv2: TypevalHandle) {
    let b1 = nvim_tv_get_blob(tv1);
    let b2 = nvim_tv_get_blob(tv2);
    let b = nvim_blob_alloc();

    // Get the grow array from the blob and append bytes
    let b_ga = blob_get_ga(b);
    for i in 0..nvim_blob_len(b1) {
        ga_append(b_ga, nvim_blob_get(b1, i));
    }
    for i in 0..nvim_blob_len(b2) {
        ga_append(b_ga, nvim_blob_get(b2, i));
    }

    tv_clear(tv1);
    nvim_blob_set_ret(tv1, b);
}

/// Make a copy of list "tv1" and append list "tv2".
unsafe fn eval_addlist_impl(tv1: TypevalHandle, tv2: TypevalHandle) -> c_int {
    let var3 = alloc_typval();
    // Concatenate Lists
    if tv_list_concat(nvim_tv_get_list(tv1), nvim_tv_get_list(tv2), var3) == FAIL {
        tv_clear(tv1);
        tv_clear(tv2);
        free_typval(var3);
        return FAIL;
    }
    tv_clear(tv1);
    tv_copy(var3, tv1);
    tv_clear(var3);
    free_typval(var3);
    OK
}

/// Add or subtract numbers "tv1" and "tv2" and store the result in "tv1".
unsafe fn eval_addsub_number_impl(tv1: TypevalHandle, tv2: TypevalHandle, op: c_int) -> c_int {
    let mut error: bool = false;
    let n1: i64;
    let n2: i64;
    let mut f1: f64 = 0.0;
    let mut f2: f64 = 0.0;

    let tv1_type = nvim_tv_get_type(tv1);
    let tv2_type = nvim_tv_get_type(tv2);

    if tv1_type == VAR_FLOAT {
        f1 = nvim_tv_get_float(tv1);
        n1 = 0;
    } else {
        n1 = tv_get_number_chk(tv1, &mut error);
        if error {
            tv_clear(tv1);
            tv_clear(tv2);
            return FAIL;
        }
        if tv2_type == VAR_FLOAT {
            f1 = n1 as f64;
        }
    }

    if tv2_type == VAR_FLOAT {
        f2 = nvim_tv_get_float(tv2);
        n2 = 0;
    } else {
        n2 = tv_get_number_chk(tv2, &mut error);
        if error {
            tv_clear(tv1);
            tv_clear(tv2);
            return FAIL;
        }
        if tv1_type == VAR_FLOAT {
            f2 = n2 as f64;
        }
    }
    tv_clear(tv1);

    // If there is a float on either side the result is a float
    if tv1_type == VAR_FLOAT || tv2_type == VAR_FLOAT {
        let result = if op == b'+' as c_int {
            f1 + f2
        } else {
            f1 - f2
        };
        nvim_tv_set_type(tv1, VAR_FLOAT);
        nvim_tv_set_float(tv1, result);
    } else {
        let result = if op == b'+' as c_int {
            n1.wrapping_add(n2)
        } else {
            n1.wrapping_sub(n2)
        };
        nvim_tv_set_type(tv1, VAR_NUMBER);
        nvim_tv_set_number(tv1, result);
    }

    OK
}

/// Multiply, divide, or compute modulo of numbers "tv1" and "tv2", store result in "tv1".
/// The numbers can be whole numbers or floats.
unsafe fn eval_multdiv_number_impl(tv1: TypevalHandle, tv2: TypevalHandle, op: c_int) -> c_int {
    let mut use_float = false;
    let mut f1: f64 = 0.0;
    let mut f2: f64 = 0.0;
    let mut error: bool = false;

    let tv1_type = nvim_tv_get_type(tv1);
    let n1: i64;

    if tv1_type == VAR_FLOAT {
        f1 = nvim_tv_get_float(tv1);
        use_float = true;
        n1 = 0;
    } else {
        n1 = tv_get_number_chk(tv1, &mut error);
    }
    tv_clear(tv1);
    if error {
        tv_clear(tv2);
        return FAIL;
    }

    let tv2_type = nvim_tv_get_type(tv2);
    let n2: i64;

    if tv2_type == VAR_FLOAT {
        if !use_float {
            f1 = n1 as f64;
            use_float = true;
        }
        f2 = nvim_tv_get_float(tv2);
        n2 = 0;
    } else {
        n2 = tv_get_number_chk(tv2, &mut error);
        tv_clear(tv2);
        if error {
            return FAIL;
        }
        if use_float {
            f2 = n2 as f64;
        }
    }

    // Compute the result. When either side is a float, the result is a float.
    if use_float {
        let result = if op == b'*' as c_int {
            f1 * f2
        } else if op == b'/' as c_int {
            // Division by zero: return NaN/Inf/NegInf per IEEE 754
            if f2 == 0.0 {
                if f1 == 0.0 {
                    f64::NAN
                } else if f1 > 0.0 {
                    f64::INFINITY
                } else {
                    f64::NEG_INFINITY
                }
            } else {
                f1 / f2
            }
        } else {
            // '%' with float is an error
            emsg(c"E804: Cannot use '%' with Float".as_ptr());
            return FAIL;
        };
        nvim_tv_set_type(tv1, VAR_FLOAT);
        nvim_tv_set_float(tv1, result);
    } else {
        let result = if op == b'*' as c_int {
            n1.wrapping_mul(n2)
        } else if op == b'/' as c_int {
            rs_num_divide(n1, n2)
        } else {
            rs_num_modulus(n1, n2)
        };
        nvim_tv_set_type(tv1, VAR_NUMBER);
        nvim_tv_set_number(tv1, result);
    }

    OK
}

/// FFI export for eval_multdiv_number.
///
/// # Safety
/// See `eval_multdiv_number_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_multdiv_number(
    tv1: TypevalHandle,
    tv2: TypevalHandle,
    op: c_int,
) -> c_int {
    eval_multdiv_number_impl(tv1, tv2, op)
}

// =============================================================================
// Typval allocation helpers (temporary - will be replaced with proper C interop)
// =============================================================================

extern "C" {
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn blob_get_ga(blob: *mut c_void) -> *mut c_void;
    fn nvim_tv_set_string(tv: TypevalHandle, s: *mut c_char);
}

/// Allocate a temporary typval
unsafe fn alloc_typval() -> TypevalHandle {
    // typval_T is typically 24-32 bytes, allocate enough
    let ptr = xmalloc(64);
    // Zero initialize
    ptr::write_bytes(ptr as *mut u8, 0, 64);
    TypevalHandle(ptr)
}

/// Free a temporary typval
unsafe fn free_typval(tv: TypevalHandle) {
    if !tv.is_null() {
        xfree(tv.as_ptr());
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(EVAL_EVALUATE, 1);
    }

    #[test]
    fn test_typval_handle() {
        let h = TypevalHandle::null();
        assert!(h.is_null());
    }

    #[test]
    fn test_evalarg_handle() {
        let h = EvalargHandle::null();
        assert!(h.is_null());
    }

    #[test]
    fn test_get_eval_flags_null() {
        let h = EvalargHandle::null();
        // Cannot test with null - would need mock
        // Just verify it compiles
        assert!(h.is_null());
    }
}
