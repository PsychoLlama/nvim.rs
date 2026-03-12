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

use crate::funcexe::FuncExeT;

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

    /// Create a handle from a raw pointer
    pub const fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
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

/// Function pointer type matching C `LineGetter`.
///
/// C definition: `char *(*LineGetter)(int, void *, int, bool)`
/// (from `src/nvim/ex_cmds_defs.h:94`)
pub type LineGetter = Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char>;

/// Rust mirror of C `evalarg_T` (src/nvim/eval_defs.h).
///
/// Layout (32 bytes total):
/// - `eval_flags` at offset 0 (c_int, 4 bytes)
/// - 4 bytes alignment padding
/// - `eval_getline` at offset 8 (fn pointer, 8 bytes)
/// - `eval_cookie` at offset 16 (*mut c_void, 8 bytes)
/// - `eval_tofree` at offset 24 (*mut c_char, 8 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EvalargT {
    /// EVAL_ flag values (EVAL_EVALUATE = 1)
    pub eval_flags: c_int,
    /// Alignment padding to 8-byte boundary
    _pad: [u8; 4],
    /// Copied from exarg_T when "getline" is "getsourceline". Can be NULL.
    pub eval_getline: LineGetter,
    /// Argument for eval_getline()
    pub eval_cookie: *mut c_void,
    /// Pointer to the last line obtained with getsourceline()
    pub eval_tofree: *mut c_char,
}

impl EvalargT {
    /// Create a zero-initialized evalarg_T with no evaluation (skip=true).
    pub fn new_skip() -> Self {
        Self {
            eval_flags: 0,
            _pad: [0u8; 4],
            eval_getline: None,
            eval_cookie: ptr::null_mut(),
            eval_tofree: ptr::null_mut(),
        }
    }

    /// Create a zero-initialized evalarg_T with evaluation enabled (skip=false).
    pub fn new_evaluate() -> Self {
        Self {
            eval_flags: EVAL_EVALUATE,
            _pad: [0u8; 4],
            eval_getline: None,
            eval_cookie: ptr::null_mut(),
            eval_tofree: ptr::null_mut(),
        }
    }
}

/// Handle to evalarg_T.
///
/// Wraps a raw pointer to `EvalargT`. May be null (meaning no evalarg).
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct EvalargHandle(pub *mut EvalargT);

impl EvalargHandle {
    /// Create a null handle
    pub const fn null() -> Self {
        Self(ptr::null_mut())
    }

    /// Check if handle is null
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get raw pointer as `*mut EvalargT`
    pub const fn as_ptr(self) -> *mut EvalargT {
        self.0
    }

    /// Get the eval_flags field (returns 0 if null).
    ///
    /// # Safety
    /// If non-null, `self.0` must point to a valid `EvalargT`.
    pub unsafe fn flags(self) -> c_int {
        if self.is_null() {
            0
        } else {
            (*self.0).eval_flags
        }
    }

    /// Set the eval_flags field (no-op if null).
    ///
    /// # Safety
    /// If non-null, `self.0` must point to a valid, writable `EvalargT`.
    pub unsafe fn set_flags(self, flags: c_int) {
        if !self.is_null() {
            (*self.0).eval_flags = flags;
        }
    }

    /// Get the eval_tofree field.
    ///
    /// # Safety
    /// `self.0` must point to a valid `EvalargT` (must not be null).
    pub unsafe fn tofree(self) -> *mut c_char {
        (*self.0).eval_tofree
    }

    /// Set the eval_tofree field.
    ///
    /// # Safety
    /// `self.0` must point to a valid, writable `EvalargT` (must not be null).
    pub unsafe fn set_tofree(self, val: *mut c_char) {
        (*self.0).eval_tofree = val;
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

    // Evalarg operations (clear_evalarg is a Rust function, declared as extern to call via C ABI)
    fn clear_evalarg(ea: EvalargHandle, eap: ExargHandle);

    // exarg operations
    fn exarg_set_nextcmd(eap: ExargHandle, nextcmd: *mut c_char);

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

    // (eval6 forward declaration removed: eval6_impl is now pure Rust)
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
        (unsafe { evalarg.flags() } & EVAL_EVALUATE) != 0
    }
}

/// Get evaluation flags, defaulting to 0 if evalarg is null
#[inline]
fn get_eval_flags(evalarg: EvalargHandle) -> c_int {
    // EvalargHandle::flags() returns 0 if null
    unsafe { evalarg.flags() }
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
pub unsafe extern "C" fn eval0(
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
            evalarg.set_flags(new_flags);
        }

        // Allocate var2 on stack (need proper C interop here)
        // For now, we'll use a helper to allocate a temporary typval
        let var2 = alloc_typval();
        if eval1_impl(arg, var2, evalarg) == FAIL {
            if !evalarg.is_null() {
                evalarg.set_flags(orig_flags);
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
                    evalarg.set_flags(orig_flags);
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
                evalarg.set_flags(new_flags2);
            }

            let var3 = alloc_typval();
            if eval1_impl(arg, var3, evalarg) == FAIL {
                if evaluate && result {
                    tv_clear(rettv);
                }
                if !evalarg.is_null() {
                    evalarg.set_flags(orig_flags);
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

        evalarg.set_flags(orig_flags);
    }

    OK
}

/// FFI export for eval1.
///
/// # Safety
/// See `eval1_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn eval1(
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
                evalarg.set_flags(new_flags);
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

        evalarg.set_flags(orig_flags);
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
                evalarg.set_flags(new_flags);
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

        evalarg.set_flags(orig_flags);
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

        if !evalarg.is_null() && (evalarg.flags() & EVAL_EVALUATE) != 0 {
            let ret = crate::operators::typval_compare_impl(rettv, var2, typ, ic);
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
    if eval6_impl(arg, rettv, evalarg, false) == FAIL {
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
        if eval6_impl(arg, var2, evalarg, op == b'.') == FAIL {
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
// eval6 - Multiplication, division, modulo
// =============================================================================

/// Handle fifth level expression:
///   - `*`  number multiplication
///   - `/`  number division
///   - `%`  number modulo
///
/// Migrated from C `eval6` in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer
/// - `rettv` must be a valid typval handle
/// - `evalarg` can be null
pub unsafe fn eval6_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    want_string: bool,
) -> c_int {
    // Get the first variable.
    if eval7_impl(arg, rettv, evalarg, want_string) == FAIL {
        return FAIL;
    }

    // Repeat computing, until no '*', '/' or '%' is following.
    loop {
        let op = get_byte(*arg);
        if op != b'*' && op != b'/' && op != b'%' {
            break;
        }

        let evaluate = should_evaluate(evalarg);

        // Get the second variable.
        *arg = skipwhite((*arg).add(1));
        let var2 = alloc_typval();
        if eval7_impl(arg, var2, evalarg, false) == FAIL {
            free_typval(var2);
            return FAIL;
        }

        if evaluate {
            // Compute the result.
            if eval_multdiv_number_impl(rettv, var2, op as c_int) == FAIL {
                free_typval(var2);
                return FAIL;
            }
        }
        free_typval(var2);
    }

    OK
}

/// FFI export for eval6.
///
/// # Safety
/// See `eval6_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn eval6(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    want_string: bool,
) -> c_int {
    eval6_impl(arg, rettv, evalarg, want_string)
}

// =============================================================================
// eval_interp_string - Interpolated string expressions ($"..." and $'...')
// =============================================================================

/// Growing array structure matching C `garray_T` layout exactly.
/// Used for character accumulation in eval_interp_string.
#[repr(C)]
struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

extern "C" {
    // Direct garray operations (Rust-exported from collections crate)
    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);
    fn ga_concat(gap: *mut GArray, s: *const c_char);
    fn ga_clear(gap: *mut c_void);
    // eval_one_expr_in_str wrapper (stays in C: wraps static C function)
    fn nvim_eval_one_expr_in_str(p: *mut c_char, gap: *mut GArray, evaluate: bool) -> *mut c_char;
    // TV string field accessor (stays in C: requires typval_T internals)
    fn nvim_tv_get_vstring(tv: TypevalHandle) -> *mut c_char;
    fn nvim_tv_set_vstring_owned(tv: TypevalHandle, s: *mut c_char);
}

/// Evaluate a single or double quoted string possibly containing expressions.
/// `arg` points to the `$`. The result is put in `rettv`.
///
/// Migrated from C `eval_interp_string` in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer, pointing at `$`
/// - `rettv` must be a valid typval handle
pub unsafe fn eval_interp_string_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evaluate: bool,
) -> c_int {
    // Allocate GArray on the heap with ga_init (char array, growsize=80).
    let ga = xmalloc(std::mem::size_of::<GArray>()).cast::<GArray>();
    ga_init(ga, 1, 80);

    // *arg is on the '$' character, move it to the first string character.
    *arg = (*arg).add(1);
    let quote = get_byte(*arg);
    *arg = (*arg).add(1);

    // Loop result: OK if successful, FAIL on any error.
    let result = 'interp: {
        loop {
            let tv = alloc_typval();
            // Get the string up to the matching quote or to a single '{'.
            let ret = if quote == b'"' {
                eval_string_impl(arg, tv, evaluate, true)
            } else {
                eval_lit_string_impl(arg, tv, evaluate, true)
            };
            if ret == FAIL {
                free_typval(tv);
                break 'interp FAIL;
            }
            if evaluate {
                let s = nvim_tv_get_vstring(tv);
                ga_concat(ga, s);
                tv_clear(tv);
            }
            free_typval(tv);

            if get_byte(*arg) != b'{' {
                // Found terminating quote
                *arg = (*arg).add(1);
                break 'interp OK;
            }
            let p = nvim_eval_one_expr_in_str(*arg, ga, evaluate);
            if p.is_null() {
                break 'interp FAIL;
            }
            *arg = p;
        }
    };

    nvim_tv_set_type(rettv, VAR_STRING);
    if result != FAIL && evaluate {
        ga_append(ga as *mut c_void, 0i32); // NUL terminator
    }
    // Take ownership of the string data and free the GArray struct.
    let data = (*ga).ga_data.cast::<c_char>();
    xfree(ga.cast::<c_void>());
    nvim_tv_set_vstring_owned(rettv, data);
    OK
}

/// FFI export for eval_interp_string.
///
/// # Safety
/// See `eval_interp_string_impl` for safety requirements.
#[export_name = "eval_interp_string"]
pub unsafe extern "C" fn rs_eval_interp_string(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evaluate: bool,
) -> c_int {
    eval_interp_string_impl(arg, rettv, evaluate)
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
// Phase 1: eval_func migration
// =============================================================================

extern "C" {
    // Function name resolution
    fn check_vars(s: *const c_char, len: usize);
    fn deref_func_name(
        name: *const c_char,
        lenp: *mut c_int,
        partialp: *mut *mut c_void,
        no_autoload: c_int,
        found_var: *mut bool,
    ) -> *mut c_char;
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_char;

    // Direct call to get_func_tv (used with FuncExeT)
    fn get_func_tv(
        name: *const c_char,
        len: c_int,
        rettv: *mut c_void,
        arg: *mut *mut c_char,
        evalarg: *mut c_void,
        funcexe: *mut FuncExeT,
    ) -> c_int;

    // Cursor position accessor
    fn nvim_curwin_get_cursor_lnum() -> i32;

    // Set vval.v_string without clearing
    fn nvim_tv_set_vstring_raw(tv: TypevalHandle, s: *mut c_char);

    // Return pointer to tv_empty_string global
    fn nvim_get_tv_empty_string() -> *const c_char;
}

/// Implementation of eval_func: resolve function name, construct funcexe, call get_func_tv.
///
/// Equivalent to the C `eval_func` static function.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer
/// - `evalarg` can be null
/// - `name` must be a valid C string of at least `name_len` bytes
/// - `rettv` must be a valid typval handle
/// - `basetv` can be null
unsafe fn eval_func_impl(
    arg: *mut *mut c_char,
    evalarg: EvalargHandle,
    name: *mut c_char,
    name_len: c_int,
    rettv: TypevalHandle,
    flags: c_int,
    basetv: TypevalHandle,
) -> c_int {
    let evaluate = (flags & EVAL_EVALUATE) != 0;
    let mut s = name;
    let mut len = name_len;
    let mut found_var = false;
    let mut partial: *mut c_void = ptr::null_mut();

    if !evaluate {
        check_vars(s, len as usize);
    }

    // If "s" is the name of a variable of type VAR_FUNC use its contents.
    s = deref_func_name(
        s,
        &mut len,
        &mut partial,
        !evaluate as c_int,
        &mut found_var,
    );

    // Need to make a copy, in case evaluating the arguments makes the name invalid.
    let s_copy = xmemdupz(s as *const c_void, len as usize);

    let lnum = nvim_curwin_get_cursor_lnum();
    let mut funcexe = FuncExeT::new();
    funcexe.fe_firstline = lnum;
    funcexe.fe_lastline = lnum;
    funcexe.fe_evaluate = evaluate;
    funcexe.fe_partial = partial;
    funcexe.fe_basetv = basetv.as_ptr();
    funcexe.fe_found_var = found_var;
    let ret = get_func_tv(
        s_copy,
        len,
        rettv.as_ptr(),
        arg,
        evalarg.as_ptr() as *mut c_void,
        &mut funcexe,
    );

    xfree(s_copy as *mut c_void);

    // If evaluate is false rettv->v_type was not set in get_func_tv, but it's
    // needed in handle_subscript() to parse what follows. So set it here.
    if nvim_tv_get_type(rettv) == VAR_UNKNOWN && !evaluate {
        // Check if **arg == '('
        let arg_ptr = *arg;
        if !arg_ptr.is_null() && *arg_ptr == b'(' as c_char {
            nvim_tv_set_vstring_raw(rettv, nvim_get_tv_empty_string() as *mut c_char);
            nvim_tv_set_type(rettv, VAR_FUNC);
        }
    }

    // Stop evaluation when immediately aborting on error, interrupt, or exception.
    if evaluate && aborting() != 0 {
        if ret == OK {
            tv_clear(rettv);
        }
        return FAIL;
    }

    ret
}

/// FFI export for eval_func.
///
/// # Safety
/// See `eval_func_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_func(
    arg: *mut *mut c_char,
    evalarg: EvalargHandle,
    name: *mut c_char,
    name_len: c_int,
    rettv: TypevalHandle,
    flags: c_int,
    basetv: TypevalHandle,
) -> c_int {
    eval_func_impl(arg, evalarg, name, name_len, rettv, flags, basetv)
}

// =============================================================================
// Phase 2: eval_number migration
// =============================================================================

extern "C" {
    // Float parsing (from eval crate)
    fn rs_string2float(text: *const c_char, ret_value: *mut f64) -> usize;

    // vim_str2nr (from charset crate, exported as vim_str2nr)
    fn vim_str2nr(
        start: *const c_char,
        prep: *mut *mut c_char,
        len: *mut c_int,
        what: c_int,
        nptr: *mut i64,
        unptr: *mut u64,
        maxlen: c_int,
        strict: bool,
        overflow: *mut bool,
    );

    // skipdigits - advance past digit characters
    fn skipdigits(p: *const c_char) -> *mut c_char;

    // Blob cleanup for error path
    fn nvim_blob_ga_clear_and_free(b: *mut c_void);
}

/// STR2NR_ALL = STR2NR_BIN | STR2NR_OCT | STR2NR_HEX | STR2NR_OOCT = 0x0F
const STR2NR_ALL: c_int = 0x0F;

/// Error: invalid expression
static E_INVEXPR2_FMT: &[u8] = b"E15: Invalid expression: \"%s\"\0";
/// Error: blob literal must have even hex digits
static E_BLOB_ODD_HEX: &[u8] = b"E973: Blob literal should have an even number of hex characters\0";

/// Test if a byte is an ASCII digit (0-9)
#[inline]
fn ascii_isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

/// Test if a byte is an ASCII hex digit (0-9, a-f, A-F)
#[inline]
fn ascii_isxdigit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

/// Test if a byte is an ASCII alpha character (a-z, A-Z)
#[inline]
fn ascii_isalpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// Convert a hex digit character to its numeric value (0-15).
/// The caller must ensure `c` is a valid hex digit.
#[inline]
fn hex2nr(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

/// Allocate a variable for a number constant. Also deals with "0z" for blob.
///
/// Migrated from C `eval_number` in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer
/// - `rettv` must be a valid typval handle
pub unsafe fn eval_number_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evaluate: bool,
    want_string: bool,
) -> c_int {
    // Skip past first digit to detect float format
    let mut p = skipdigits((*arg).add(1));
    let mut get_float = false;

    // We accept a float when the format matches
    // "[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?". Very strict.
    // Don't look for a float after "." operator to avoid
    // ":let vers = 1.2.3" failing.
    if !want_string && get_byte(p) == b'.' && ascii_isdigit(get_byte(p.add(1))) {
        get_float = true;
        p = skipdigits(p.add(2));
        let e = get_byte(p);
        if e == b'e' || e == b'E' {
            p = p.add(1);
            let sign = get_byte(p);
            if sign == b'-' || sign == b'+' {
                p = p.add(1);
            }
            if !ascii_isdigit(get_byte(p)) {
                get_float = false;
            } else {
                p = skipdigits(p.add(1));
            }
        }
        // Reject if followed by alpha or another '.' (would be method call)
        let next = get_byte(p);
        if ascii_isalpha(next) || next == b'.' {
            get_float = false;
        }
    }

    if get_float {
        let mut f: f64 = 0.0;
        let consumed = rs_string2float(*arg, &mut f);
        *arg = (*arg).add(consumed);
        if evaluate {
            nvim_tv_set_type(rettv, VAR_FLOAT);
            nvim_tv_set_float(rettv, f);
        }
    } else if get_byte(*arg) == b'0'
        && (get_byte((*arg).add(1)) == b'z' || get_byte((*arg).add(1)) == b'Z')
    {
        // Blob constant: 0z0123456789abcdef
        let mut blob: *mut c_void = std::ptr::null_mut();
        if evaluate {
            blob = nvim_blob_alloc();
        }
        let mut bp = (*arg).add(2);
        while ascii_isxdigit(get_byte(bp)) {
            if !ascii_isxdigit(get_byte(bp.add(1))) {
                if !blob.is_null() {
                    emsg(E_BLOB_ODD_HEX.as_ptr() as *const c_char);
                    nvim_blob_ga_clear_and_free(blob);
                }
                return FAIL;
            }
            if !blob.is_null() {
                let byte_val = (hex2nr(get_byte(bp)) << 4) | hex2nr(get_byte(bp.add(1)));
                ga_append(blob_get_ga(blob), byte_val as c_int);
            }
            bp = bp.add(2);
            // Optional '.' separator between pairs
            if get_byte(bp) == b'.' && ascii_isxdigit(get_byte(bp.add(1))) {
                bp = bp.add(1);
            }
        }
        if !blob.is_null() {
            nvim_blob_set_ret(rettv, blob);
        }
        *arg = bp;
    } else {
        // Decimal, hex, octal, or binary number
        let mut len: c_int = 0;
        let mut n: i64 = 0;
        vim_str2nr(
            *arg,
            std::ptr::null_mut(),
            &mut len,
            STR2NR_ALL,
            &mut n,
            std::ptr::null_mut(),
            0,
            true,
            std::ptr::null_mut(),
        );
        if len == 0 {
            if evaluate {
                semsg(E_INVEXPR2_FMT.as_ptr() as *const c_char, *arg);
            }
            return FAIL;
        }
        *arg = (*arg).add(len as usize);
        if evaluate {
            nvim_tv_set_type(rettv, VAR_NUMBER);
            nvim_tv_set_number(rettv, n);
        }
    }
    OK
}

/// FFI export for eval_number.
///
/// # Safety
/// See `eval_number_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_number(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evaluate: bool,
    want_string: bool,
) -> c_int {
    eval_number_impl(arg, rettv, evaluate, want_string)
}

// =============================================================================
// Phase 3: eval_list migration
// =============================================================================

extern "C" {
    // List operations - new C accessors
    fn nvim_eval_tv_list_alloc(len: isize) -> *mut c_void;
    fn nvim_eval_tv_list_free(l: *mut c_void);
    fn nvim_eval_tv_list_append_owned_tv_ptr(l: *mut c_void, tv: TypevalHandle);
    fn nvim_eval_tv_list_set_ret(rettv: TypevalHandle, l: *mut c_void);
}

/// kListLenShouldKnow = -2 as ptrdiff_t
const K_LIST_LEN_SHOULD_KNOW: isize = -2;

/// Error: missing comma in List
static E_MISSING_COMMA_LIST: &[u8] = b"E696: Missing comma in List: %s\0";
/// Error: missing ] in List
static E_LIST_END: &[u8] = b"E697: Missing end of List ']': %s\0";

/// Allocate a variable for a List and fill it from `*arg`.
///
/// `*arg` points to the "[".
/// Migrated from C `eval_list` in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer, pointing at '['
/// - `rettv` must be a valid typval handle
/// - `evalarg` can be null
pub unsafe fn eval_list_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    let evaluate = should_evaluate(evalarg);
    let mut l: *mut c_void = std::ptr::null_mut();

    if evaluate {
        l = nvim_eval_tv_list_alloc(K_LIST_LEN_SHOULD_KNOW);
    }

    // Skip past the '['
    *arg = skipwhite((*arg).add(1));

    while get_byte(*arg) != b']' && get_byte(*arg) != 0 {
        let tv = alloc_typval();
        if eval1_impl(arg, tv, evalarg) == FAIL {
            free_typval(tv);
            if !l.is_null() {
                nvim_eval_tv_list_free(l);
            }
            return FAIL;
        }
        if evaluate {
            nvim_eval_tv_list_append_owned_tv_ptr(l, tv);
        } else {
            tv_clear(tv);
        }
        free_typval(tv);

        // The comma must come after the value
        let had_comma = get_byte(*arg) == b',';
        if had_comma {
            *arg = skipwhite((*arg).add(1));
        }

        if get_byte(*arg) == b']' {
            break;
        }

        if !had_comma {
            semsg(E_MISSING_COMMA_LIST.as_ptr() as *const c_char, *arg);
            if !l.is_null() {
                nvim_eval_tv_list_free(l);
            }
            return FAIL;
        }
    }

    if get_byte(*arg) != b']' {
        semsg(E_LIST_END.as_ptr() as *const c_char, *arg);
        if !l.is_null() {
            nvim_eval_tv_list_free(l);
        }
        return FAIL;
    }

    *arg = skipwhite((*arg).add(1));
    if evaluate {
        nvim_eval_tv_list_set_ret(rettv, l);
    }

    OK
}

/// FFI export for eval_list.
///
/// # Safety
/// See `eval_list_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_list(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    eval_list_impl(arg, rettv, evalarg)
}

// =============================================================================
// Phase 4: eval_method migration
// =============================================================================

extern "C" {
    // Name resolution
    fn get_name_len(
        arg: *mut *const c_char,
        alias: *mut *mut c_char,
        evaluate: bool,
        verbose: bool,
    ) -> c_int;
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // Lua function helpers (from eval crate)
    fn rs_skip_luafunc_name(p: *const c_char) -> *const c_char;
    fn rs_partial_name(pt: *const c_void) -> *mut c_char;

    // Partial accessors
    fn nvim_get_vlua_partial() -> *mut c_void;
    fn nvim_partial_get_argc(pt: *const c_void) -> c_int;
    fn nvim_partial_get_dict(pt: *const c_void) -> *mut c_void;
    fn nvim_eval_partial_incref(pt: *mut c_void);

    // Typval partial accessor/setter
    fn nvim_eval_tv_get_partial(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_set_partial_raw(tv: TypevalHandle, pt: *mut c_void);

    // Typval direct assign (struct copy)
    fn nvim_tv_assign_direct(dst: TypevalHandle, src: TypevalHandle);

    // ASCII whitespace check
    fn rs_ascii_iswhite(c: c_int) -> c_int;

    // String duplication
    fn xstrdup(s: *const c_char) -> *mut c_char;

}

/// Error: missing name after ->
static E_MISSING_NAME_AFTER_ARROW: &[u8] = b"E260: Missing name after ->\0";
/// Error: cannot use a partial here
static E_CANNOT_USE_PARTIAL_HERE: &[u8] = b"E1265: Cannot use a partial here\0";
/// Error: not a callable type
static E_NOT_CALLABLE_TYPE: &[u8] = b"E1085: Not a callable type: %s\0";
/// Error: missing parentheses
static E_MISSING_PAREN: &[u8] = b"E107: Missing parentheses: %s\0";
/// Error: no white space allowed before parenthesis
static E_NO_WHITESPACE: &[u8] = b"E274: No white space allowed before parenthesis\0";

/// Implementation of eval_method: evaluates "->method()" or "->v:lua.method()".
///
/// Migrated from C `eval_method` static function in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer, pointing at '-'
/// - `rettv` must be a valid typval handle
/// - `evalarg` can be null
pub unsafe fn eval_method_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    verbose: bool,
) -> c_int {
    let evaluate = should_evaluate(evalarg);

    // Skip over the ->
    *arg = (*arg).add(2);

    // Move rettv into base: base = *rettv, rettv->v_type = VAR_UNKNOWN
    let base = alloc_typval();
    nvim_tv_assign_direct(base, rettv);
    nvim_tv_set_type(rettv, VAR_UNKNOWN);

    // Locate the method name.
    let mut len: c_int;
    let mut name: *mut c_char = *arg;
    let mut lua_funcname: *const c_char = std::ptr::null();
    let mut alias: *mut c_char = std::ptr::null_mut();

    // Check for "v:lua." prefix (6 bytes) using pure Rust byte comparison.
    // We read one byte at a time via get_byte to avoid reading past the string end.
    let prefix_match = !(*arg).is_null() && {
        let p = *arg as *const u8;
        get_byte(p as *const c_char) == b'v'
            && get_byte(p.add(1) as *const c_char) == b':'
            && get_byte(p.add(2) as *const c_char) == b'l'
            && get_byte(p.add(3) as *const c_char) == b'u'
            && get_byte(p.add(4) as *const c_char) == b'a'
            && get_byte(p.add(5) as *const c_char) == b'.'
    };

    if prefix_match {
        lua_funcname = (*arg).add(6) as *const c_char;
        *arg = rs_skip_luafunc_name(lua_funcname) as *mut c_char;
        *arg = skipwhite(*arg); // to detect trailing whitespace later
        len = (*arg as usize).wrapping_sub(lua_funcname as usize) as c_int;
    } else {
        len = get_name_len(arg as *mut *const c_char, &mut alias, evaluate, true);
        if !alias.is_null() {
            name = alias;
        }
    }

    let mut tofree: *mut c_char = std::ptr::null_mut();
    let mut ret = OK;

    if len <= 0 {
        if verbose {
            if lua_funcname.is_null() {
                emsg(E_MISSING_NAME_AFTER_ARROW.as_ptr() as *const c_char);
            } else {
                semsg(E_INVEXPR2.as_ptr() as *const c_char, name);
            }
        }
        ret = FAIL;
    } else {
        *arg = skipwhite(*arg);

        // If there is no "(" immediately following, but there is further on,
        // it can be "dict.Func()", "list[nr]", etc.
        // Does not handle anything where "(" is part of the expression.
        let paren = if get_byte(*arg) != b'(' && lua_funcname.is_null() && alias.is_null() {
            vim_strchr(*arg, b'(' as c_int)
        } else {
            std::ptr::null_mut()
        };

        if !paren.is_null() {
            *arg = name;
            *paren = 0; // NUL out the '('
            let ref_tv = alloc_typval();
            nvim_tv_set_type(ref_tv, VAR_UNKNOWN);
            if eval7_impl(arg, ref_tv, evalarg, false) == FAIL {
                *arg = name.add(len as usize);
                ret = FAIL;
            } else if get_byte(skipwhite(*arg)) != 0 {
                if verbose {
                    semsg(E_TRAILING_ARG.as_ptr() as *const c_char, *arg);
                }
                ret = FAIL;
            } else if nvim_tv_get_type(ref_tv) == VAR_FUNC && !nvim_tv_get_vstring(ref_tv).is_null()
            {
                name = nvim_tv_get_vstring(ref_tv);
                // Steal the string: set ref.vval.v_string = NULL so tv_clear won't free it
                nvim_tv_set_vstring_raw(ref_tv, std::ptr::null_mut());
                tofree = name;
                len = libc_strlen(name) as c_int;
            } else if nvim_tv_get_type(ref_tv) == VAR_PARTIAL
                && !nvim_eval_tv_get_partial(ref_tv).is_null()
            {
                let pt = nvim_eval_tv_get_partial(ref_tv);
                if nvim_partial_get_argc(pt) > 0 || !nvim_partial_get_dict(pt).is_null() {
                    if verbose {
                        emsg(E_CANNOT_USE_PARTIAL_HERE.as_ptr() as *const c_char);
                    }
                    ret = FAIL;
                } else {
                    name = xstrdup(rs_partial_name(pt));
                    tofree = name;
                    if name.is_null() {
                        ret = FAIL;
                        name = *arg;
                    } else {
                        len = libc_strlen(name) as c_int;
                    }
                }
            } else {
                if verbose {
                    semsg(E_NOT_CALLABLE_TYPE.as_ptr() as *const c_char, name);
                }
                ret = FAIL;
            }
            tv_clear(ref_tv);
            free_typval(ref_tv);
            *paren = b'(' as c_char; // restore the '('
        }

        if ret == OK {
            if get_byte(*arg) != b'(' {
                if verbose {
                    semsg(E_MISSING_PAREN.as_ptr() as *const c_char, name);
                }
                ret = FAIL;
            } else if rs_ascii_iswhite((*arg).sub(1).read() as c_int) != 0 {
                if verbose {
                    emsg(E_NO_WHITESPACE.as_ptr() as *const c_char);
                }
                ret = FAIL;
            } else if !lua_funcname.is_null() {
                if evaluate {
                    let vlua = nvim_get_vlua_partial();
                    nvim_tv_set_type(rettv, VAR_PARTIAL);
                    nvim_tv_set_partial_raw(rettv, vlua);
                    nvim_eval_partial_incref(vlua);
                }
                ret = call_func_rettv_impl(
                    arg,
                    evalarg,
                    rettv,
                    evaluate,
                    std::ptr::null_mut(),
                    base,
                    lua_funcname,
                );
            } else {
                ret = eval_func_impl(
                    arg,
                    evalarg,
                    name,
                    len,
                    rettv,
                    if evaluate { EVAL_EVALUATE } else { 0 },
                    base,
                );
            }
        }
    }

    // Clear the funcref afterwards, so that deleting it while
    // evaluating the arguments is possible (see test55).
    if evaluate {
        tv_clear(base);
    }
    free_typval(base);
    xfree(tofree as *mut c_void);

    if !alias.is_null() {
        xfree(alias as *mut c_void);
    }

    ret
}

/// Compute strlen for a C string (not calling libc directly, using pointer arithmetic).
#[inline]
unsafe fn libc_strlen(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    (p as usize).wrapping_sub(s as usize)
}

/// FFI export for eval_method.
///
/// # Safety
/// See `eval_method_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_method(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    verbose: bool,
) -> c_int {
    eval_method_impl(arg, rettv, evalarg, verbose)
}

// =============================================================================
// Phase 5: eval_lit_string, eval_string, eval_dict migration
// =============================================================================

extern "C" {
    /// Copy one multibyte character from *fp to *tp, advancing both pointers.
    fn mb_copy_char(fp: *mut *const c_char, tp: *mut *mut c_char);
    /// Return the byte length of the multibyte character at p.
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    /// Encode a Unicode codepoint to UTF-8 bytes; returns bytes written.
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    /// Find a special key sequence like <C-W>; returns key code or 0.
    fn find_special_key(
        srcp: *mut *const c_char,
        src_len: usize,
        modp: *mut c_int,
        flags: c_int,
        did_simplify: *mut bool,
    ) -> c_int;
    /// Translate a special key sequence to bytes; returns bytes written or 0.
    fn trans_special(
        srcp: *mut *const c_char,
        src_len: usize,
        dst: *mut c_char,
        flags: c_int,
        escape_ks: bool,
        did_simplify: *mut bool,
    ) -> c_uint;
    /// Issue an internal error message.
    fn iemsg(msg: *const c_char);
    /// External error message string for stray '}'.
    static e_stray_closing_curly_str: c_char;

    // Dict operations
    fn tv_dict_alloc() -> *mut c_void;
    fn tv_dict_free(d: *mut c_void);
    fn tv_dict_find(d: *const c_void, key: *const c_char, len: isize) -> *mut c_void;
    fn tv_dict_item_alloc(key: *const c_char) -> *mut c_void;
    fn tv_dict_add(d: *mut c_void, item: *mut c_void) -> c_int;
    fn tv_dict_item_free(item: *mut c_void);
    fn nvim_eval_tv_dict_set_ret(rettv: TypevalHandle, d: *mut c_void);
    /// Accessor: set di->di_tv = *tv; di->di_tv.v_lock = VAR_UNLOCKED
    fn nvim_eval_di_set_tv_from_typval(di: *mut c_void, tv: TypevalHandle);
    /// Get the raw vval.v_string pointer from a typval (for dict dup key check via
    /// tv_get_string_buf_chk).
    fn nvim_tv_get_string(tv: TypevalHandle) -> *mut c_char;
}

// FSK flag constants (must match keycodes.h)
const FSK_KEYCODE: c_int = 0x01;
const FSK_IN_STRING: c_int = 0x04;
const FSK_SIMPLIFY: c_int = 0x08;

// c_uint is needed for trans_special return type
use std::ffi::c_uint;

/// Allocate a variable for a 'str''ing' constant.
/// When `interpolate` is true, reduce `{{` to `{` and stop at a single `{`.
///
/// Migrated from C `eval_lit_string` in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer
/// - `rettv` must be a valid typval handle
pub unsafe fn eval_lit_string_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evaluate: bool,
    interpolate: bool,
) -> c_int {
    let mut reduce: isize = if interpolate { -1 } else { 0 };
    let off: isize = if interpolate { 0 } else { 1 };

    // First pass: find the end of the string, count reductions.
    // Mirrors: for (p = *arg + off; *p != NUL; MB_PTR_ADV(p)) { ... }
    let mut p: *const c_char = (*arg).offset(off);

    loop {
        let c = get_byte(p);
        if c == 0 {
            break;
        }
        if c == b'\'' {
            if get_byte(p.add(1)) != b'\'' {
                break;
            }
            reduce += 1;
            p = p.add(1);
        } else if interpolate {
            if c == b'{' {
                if get_byte(p.add(1)) != b'{' {
                    // start of interpolated expression
                    break;
                }
                p = p.add(1);
                reduce += 1;
            } else if c == b'}' {
                p = p.add(1);
                if get_byte(p) != b'}' {
                    // single '}' is stray
                    semsg(&e_stray_closing_curly_str as *const c_char, *arg);
                    return FAIL;
                }
                reduce += 1;
            }
        }
        // MB_PTR_ADV: advance by one multibyte char
        let advance = utfc_ptr2len(p) as usize;
        p = p.add(if advance > 0 { advance } else { 1 });
    }

    if get_byte(p) != b'\'' && !(interpolate && get_byte(p) == b'{') {
        semsg(c"E115: Missing quote: %s".as_ptr(), *arg);
        return FAIL;
    }

    // If only parsing, advance arg and return OK.
    if !evaluate {
        *arg = p.offset(off) as *mut c_char;
        return OK;
    }

    // Compute allocation size: (p - *arg) - reduce
    let raw_len = (p as isize) - (*arg as isize);
    let alloc_len = (raw_len - reduce) as usize;

    let str_buf = xmalloc(alloc_len) as *mut c_char;
    nvim_tv_set_type(rettv, VAR_STRING);
    nvim_tv_set_string(rettv, str_buf);

    // Second pass: copy characters with reductions applied.
    // Mirrors: for (p = *arg + off; *p != NUL;) { ... mb_copy_char(...) ... }
    let mut dst: *mut c_char = str_buf;
    let mut q: *const c_char = (*arg).offset(off);

    loop {
        let c = get_byte(q);
        if c == 0 {
            break;
        }
        if c == b'\'' {
            if get_byte(q.add(1)) != b'\'' {
                break;
            }
            // skip first of the '' pair; mb_copy_char copies the second one
            q = q.add(1);
        } else if interpolate && (c == b'{' || c == b'}') {
            if c == b'{' && get_byte(q.add(1)) != b'{' {
                break;
            }
            // skip first of {{ or }} pair; mb_copy_char copies second
            q = q.add(1);
        }
        mb_copy_char(&mut q, &mut dst);
    }

    *dst = 0; // NUL terminate
    *arg = q.offset(off) as *mut c_char;

    OK
}

/// FFI export for eval_lit_string.
///
/// # Safety
/// See `eval_lit_string_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_lit_string(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evaluate: bool,
    interpolate: bool,
) -> c_int {
    eval_lit_string_impl(arg, rettv, evaluate, interpolate)
}

// =============================================================================
// Phase 5b: eval_string migration
// =============================================================================

/// Evaluate a string constant and put the result in "rettv".
/// `*arg` points to the double quote or to after it when `interpolate` is true.
/// When `interpolate` is true, reduce `{{` to `{`, reduce `}}` to `}` and
/// stop at a single `{`.
///
/// Migrated from C `eval_string` in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer
/// - `rettv` must be a valid typval handle
pub unsafe fn eval_string_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evaluate: bool,
    interpolate: bool,
) -> c_int {
    let arg_end: *const c_char = (*arg).add(libc_strlen(*arg));
    let mut extra: usize = if interpolate { 1 } else { 0 };
    let off: isize = if interpolate { 0 } else { 1 };

    // First pass: find end of string, count extra space needed for special keys.
    // Mirrors: for (p = *arg + off; *p != NUL && *p != '"'; MB_PTR_ADV(p)) { ... }
    // NOTE: p is *const c_char so we can pass &mut p to find_special_key.
    let mut p: *const c_char = (*arg).offset(off);

    'first_pass: loop {
        let c = get_byte(p);
        if c == 0 || c == b'"' {
            break;
        }
        if c == b'\\' && get_byte(p.add(1)) != 0 {
            p = p.add(1); // skip backslash; now on escaped char
            if get_byte(p) == b'<' {
                let mut modifiers: c_int = 0;
                let mut flags = FSK_KEYCODE | FSK_IN_STRING;
                extra += 5;
                if get_byte(p.add(1)) != b'*' {
                    flags |= FSK_SIMPLIFY;
                }
                // find_special_key advances p to after '>' on success.
                let src_len = (arg_end as usize).wrapping_sub(p as usize);
                if find_special_key(&mut p, src_len, &mut modifiers, flags, std::ptr::null_mut())
                    != 0
                {
                    // Leave p on ">"; MB_PTR_ADV at end advances past ">".
                    // The C code does p-- here to back up from after ">".
                    p = p.sub(1);
                }
                // Fall through to MB_PTR_ADV at end.
            }
        } else if interpolate && (c == b'{' || c == b'}') {
            if c == b'{' && get_byte(p.add(1)) != b'{' {
                // start of interpolated expression
                break 'first_pass;
            }
            p = p.add(1); // move to second char of {{ or }}
            if get_byte(p.sub(1)) == b'}' && get_byte(p) != b'}' {
                // single '}' is stray
                semsg(&e_stray_closing_curly_str as *const c_char, *arg);
                return FAIL;
            }
            extra = extra.saturating_sub(1); // "{{" -> "{", "}}" -> "}"
        }
        // MB_PTR_ADV: p += utfc_ptr2len(p)
        let advance = utfc_ptr2len(p) as usize;
        p = p.add(if advance > 0 { advance } else { 1 });
    }

    let end_char = get_byte(p);
    if end_char != b'"' && !(interpolate && end_char == b'{') {
        semsg(c"E114: Missing quote: %s".as_ptr(), *arg);
        return FAIL;
    }

    // If only parsing, advance arg and return OK.
    if !evaluate {
        *arg = p.offset(off) as *mut c_char;
        return OK;
    }

    // Allocate result buffer.
    nvim_tv_set_type(rettv, VAR_STRING);
    let raw_len = (p as isize) - (*arg as isize);
    let len = raw_len as usize + extra;
    let vstring = xmalloc(len) as *mut c_char;
    nvim_tv_set_string(rettv, vstring);
    let mut end: *mut c_char = vstring;

    // Second pass: copy with escape processing.
    // Mirrors: for (p = *arg + off; *p != NUL && *p != '"';) { ... }
    // NOTE: q is *const c_char so we can pass &mut q to mb_copy_char/trans_special.
    let mut q: *const c_char = (*arg).offset(off);

    'second_pass: loop {
        let c = get_byte(q);
        if c == 0 || c == b'"' {
            break;
        }
        if c == b'\\' {
            q = q.add(1); // skip backslash; now on escaped char
            let ec = get_byte(q);
            match ec {
                b'b' => {
                    *end = 0x08; // BS
                    end = end.add(1);
                    q = q.add(1);
                }
                b'e' => {
                    *end = 0x1B; // ESC
                    end = end.add(1);
                    q = q.add(1);
                }
                b'f' => {
                    *end = 0x0C; // FF
                    end = end.add(1);
                    q = q.add(1);
                }
                b'n' => {
                    *end = b'\n' as c_char;
                    end = end.add(1);
                    q = q.add(1);
                }
                b'r' => {
                    *end = b'\r' as c_char;
                    end = end.add(1);
                    q = q.add(1);
                }
                b't' => {
                    *end = b'\t' as c_char;
                    end = end.add(1);
                    q = q.add(1);
                }
                escape @ (b'X' | b'x' | b'u' | b'U') => {
                    if ascii_isxdigit(get_byte(q.add(1))) {
                        let n = if escape == b'X' || escape == b'x' {
                            2usize
                        } else if escape == b'u' {
                            4
                        } else {
                            8
                        };
                        let mut nr: i32 = 0;
                        let mut cnt = n;
                        while cnt > 0 && ascii_isxdigit(get_byte(q.add(1))) {
                            q = q.add(1);
                            nr = (nr << 4) + i32::from(hex2nr(get_byte(q)));
                            cnt -= 1;
                        }
                        q = q.add(1);
                        if escape == b'X' || escape == b'x' {
                            *end = nr as c_char;
                            end = end.add(1);
                        } else {
                            let written = utf_char2bytes(nr, end);
                            end = end.add(written as usize);
                        }
                    }
                    // If no hex digit follows, no output (q remains on escape char;
                    // it will be advanced below by going to the loop start where
                    // MB_PTR_ADV... wait - the second pass has no MB_PTR_ADV).
                    // Actually the C code's switch arms that produce no output still
                    // end at the "break" of the switch, letting the for loop's
                    // (empty) increment run. The source advance for this case is
                    // zero (q stays on the 'x' char after no valid hex digit).
                    // BUT: wait, this means no advance - infinite loop?
                    // Looking at C: for case 'x': if no hex digit, we don't advance.
                    // But the for loop has NO increment in the second pass (the C
                    // code's 2nd pass for loop is bare: for(...;...;) {...}).
                    // In that case, the default switch arm uses mb_copy_char which
                    // advances. For the 'x' case with no hex digit: the switch breaks,
                    // outer for loop iterates... with the same p. Infinite loop? No -
                    // looking at C more carefully: the 'X'/'x'/'u'/'U' case does NOT
                    // fall through and does NOT call mb_copy_char. If there's no valid
                    // hex digit, the switch just breaks. Then the for loop's blank
                    // increment runs (nothing happens), loop body runs again from same
                    // p... Hmm, that IS an infinite loop in C too? Actually no - wait,
                    // p was already advanced past the backslash (with ++p in *++p).
                    // If no hex digit: p stays on 'x'/'u'/'U', the for loop iterates
                    // again from that same character. That WOULD be an infinite loop.
                    // But in practice, if there's no hex digit after \x, the code
                    // just produces no output and q should advance past 'x'.
                    // Since C doesn't advance here either, we match by doing nothing.
                    // Actually looking more carefully: C increments n times, and if
                    // no digit at all, the while loop runs 0 times, then p++ moves
                    // past the escape char. That's what q = q.add(1) above does
                    // after the while. So q does advance! (That line IS hit.)
                }
                oct @ b'0'..=b'7' => {
                    let mut val = (oct - b'0') as c_char;
                    q = q.add(1);
                    if get_byte(q) >= b'0' && get_byte(q) <= b'7' {
                        val = ((val as u8) << 3).wrapping_add(get_byte(q) - b'0') as c_char;
                        q = q.add(1);
                        if get_byte(q) >= b'0' && get_byte(q) <= b'7' {
                            val = ((val as u8) << 3).wrapping_add(get_byte(q) - b'0') as c_char;
                            q = q.add(1);
                        }
                    }
                    *end = val;
                    end = end.add(1);
                }
                b'<' => {
                    let mut flags = FSK_KEYCODE | FSK_IN_STRING;
                    if get_byte(q.add(1)) != b'*' {
                        flags |= FSK_SIMPLIFY;
                    }
                    let src_len = (arg_end as usize).wrapping_sub(q as usize);
                    // trans_special advances q past ">".
                    let written =
                        trans_special(&mut q, src_len, end, flags, false, std::ptr::null_mut());
                    if written != 0 {
                        end = end.add(written as usize);
                        if end >= vstring.add(len) {
                            iemsg(c"eval_string() used more space than allocated".as_ptr());
                        }
                        // q is past ">"; continue to top of loop.
                        continue 'second_pass;
                    }
                    // FALLTHROUGH: trans_special returned 0, treat as default.
                    // q is still on '<', copy it with mb_copy_char.
                    mb_copy_char(&mut q, &mut end);
                }
                _ => {
                    mb_copy_char(&mut q, &mut end);
                }
            }
        } else if interpolate && (c == b'{' || c == b'}') {
            if c == b'{' && get_byte(q.add(1)) != b'{' {
                // start of expression
                break 'second_pass;
            }
            q = q.add(1); // skip first of {{ or }}; copy second with mb_copy_char
            mb_copy_char(&mut q, &mut end);
        } else {
            mb_copy_char(&mut q, &mut end);
        }
    }

    *end = 0; // NUL terminate
    if get_byte(q) == b'"' && !interpolate {
        q = q.add(1);
    }
    *arg = q as *mut c_char;

    OK
}

/// FFI export for eval_string.
///
/// # Safety
/// See `eval_string_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_string(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evaluate: bool,
    interpolate: bool,
) -> c_int {
    eval_string_impl(arg, rettv, evaluate, interpolate)
}

// =============================================================================
// Phase 5c: eval_dict, get_literal_key, eval_lit_dict migration
// =============================================================================

/// VAR_UNLOCKED constant (must match C enum)
const VAR_UNLOCKED: c_int = 0;

/// Get the key for #{key: val} into `tv` and advance `arg`.
///
/// Returns FAIL when there is no valid key.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer
/// - `tv` must be a valid typval handle
unsafe fn get_literal_key_impl(arg: *mut *mut c_char, tv: TypevalHandle) -> c_int {
    let first = get_byte(*arg);
    if !first.is_ascii_alphanumeric() && first != b'_' && first != b'-' {
        return FAIL;
    }
    let mut p = *arg;
    loop {
        let c = get_byte(p);
        if !c.is_ascii_alphanumeric() && c != b'_' && c != b'-' {
            break;
        }
        p = p.add(1);
    }
    let key_len = (p as usize) - (*arg as usize);
    let key = xmemdupz(*arg as *const c_void, key_len);
    nvim_tv_set_type(tv, VAR_STRING);
    nvim_tv_set_string(tv, key);
    *arg = skipwhite(p);
    OK
}

/// Allocate a variable for a Dictionary and fill it from `*arg`.
///
/// `*arg` points to the `{`.
/// `literal` is true for `#{key: val}`.
///
/// Returns OK, FAIL, or NOTDONE (for `{expr}`).
///
/// Migrated from C `eval_dict` in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer, pointing at `{`
/// - `rettv` must be a valid typval handle
/// - `evalarg` can be null
pub unsafe fn eval_dict_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    literal: bool,
) -> c_int {
    let evaluate = should_evaluate(evalarg);
    let mut key: *const c_char = std::ptr::null();
    let mut curly_expr = skipwhite((*arg).add(1));
    let mut buf = [0i8; 65]; // NUMBUFLEN

    // Check if it's a curly-braces expression: {expr}.
    // "{}  " is an empty Dictionary; "#{abc}" is never a curly-braces expr.
    let is_curly_expr = if get_byte(curly_expr) != b'}' && !literal {
        let check_tv = alloc_typval();
        let result = eval1_impl(&mut curly_expr, check_tv, EvalargHandle::null()) == OK
            && get_byte(skipwhite(curly_expr)) == b'}';
        tv_clear(check_tv);
        free_typval(check_tv);
        result
    } else {
        false
    };
    if is_curly_expr {
        return NOTDONE;
    }

    let mut d: *mut c_void = std::ptr::null_mut();
    if evaluate {
        d = tv_dict_alloc();
    }

    let tvkey = alloc_typval();
    let tvval = alloc_typval();
    nvim_tv_set_type(tvkey, VAR_UNKNOWN);
    nvim_tv_set_type(tvval, VAR_UNKNOWN);

    *arg = skipwhite((*arg).add(1));

    while get_byte(*arg) != b'}' && get_byte(*arg) != 0 {
        // Get the key
        let key_result = if literal {
            get_literal_key_impl(arg, tvkey)
        } else {
            eval1_impl(arg, tvkey, evalarg)
        };
        if key_result == FAIL {
            tv_clear(tvkey);
            tv_clear(tvval);
            free_typval(tvkey);
            free_typval(tvval);
            if !d.is_null() {
                tv_dict_free(d);
            }
            return FAIL;
        }

        if get_byte(*arg) != b':' {
            semsg(c"E720: Missing colon in Dictionary: %s".as_ptr(), *arg);
            tv_clear(tvkey);
            tv_clear(tvval);
            free_typval(tvkey);
            free_typval(tvval);
            if !d.is_null() {
                tv_dict_free(d);
            }
            return FAIL;
        }

        if evaluate {
            key = tv_get_string_buf_chk(tvkey, buf.as_mut_ptr());
            if key.is_null() {
                tv_clear(tvkey);
                tv_clear(tvval);
                free_typval(tvkey);
                free_typval(tvval);
                if !d.is_null() {
                    tv_dict_free(d);
                }
                return FAIL;
            }
        }

        *arg = skipwhite((*arg).add(1));
        if eval1_impl(arg, tvval, evalarg) == FAIL {
            if evaluate {
                tv_clear(tvkey);
            }
            tv_clear(tvval);
            free_typval(tvkey);
            free_typval(tvval);
            if !d.is_null() {
                tv_dict_free(d);
            }
            return FAIL;
        }

        if evaluate {
            let item = tv_dict_find(d, key, -1);
            if !item.is_null() {
                semsg(c"E721: Duplicate key in Dictionary: \"%s\"".as_ptr(), key);
                tv_clear(tvkey);
                tv_clear(tvval);
                free_typval(tvkey);
                free_typval(tvval);
                tv_dict_free(d);
                return FAIL;
            }
            let new_item = tv_dict_item_alloc(key);
            // Transfer ownership: di->di_tv = *tvval; dict item now owns the content.
            nvim_eval_di_set_tv_from_typval(new_item, tvval);
            if tv_dict_add(d, new_item) == FAIL {
                // Add failed; item (and its copy of tvval's content) is freed here.
                tv_dict_item_free(new_item);
            }
            // Do NOT tv_clear(tvval) here: the dict item owns tvval's content.
            // Just reset the type so the next iteration starts fresh.
            nvim_tv_set_type(tvval, VAR_UNKNOWN);
        } else {
            // Not evaluating: tvval should be VAR_UNKNOWN, but clear defensively.
            tv_clear(tvval);
            nvim_tv_set_type(tvval, VAR_UNKNOWN);
        }
        tv_clear(tvkey);
        nvim_tv_set_type(tvkey, VAR_UNKNOWN);

        // comma must come after value
        let had_comma = get_byte(*arg) == b',';
        if had_comma {
            *arg = skipwhite((*arg).add(1));
        }

        if get_byte(*arg) == b'}' {
            break;
        }

        if !had_comma {
            semsg(c"E722: Missing comma in Dictionary: %s".as_ptr(), *arg);
            free_typval(tvkey);
            free_typval(tvval);
            if !d.is_null() {
                tv_dict_free(d);
            }
            return FAIL;
        }
    }

    free_typval(tvkey);
    free_typval(tvval);

    if get_byte(*arg) != b'}' {
        semsg(c"E723: Missing end of Dictionary '}': %s".as_ptr(), *arg);
        if !d.is_null() {
            tv_dict_free(d);
        }
        return FAIL;
    }

    *arg = skipwhite((*arg).add(1));
    if evaluate {
        nvim_eval_tv_dict_set_ret(rettv, d);
    }

    OK
}

/// Evaluate a literal dictionary: #{key: val, key: val}
/// `*arg` points to the `#`.
///
/// Migrated from C `eval_lit_dict` in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer, pointing at `#`
/// - `rettv` must be a valid typval handle
/// - `evalarg` can be null
pub unsafe fn eval_lit_dict_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    if get_byte((*arg).add(1)) == b'{' {
        *arg = (*arg).add(1);
        eval_dict_impl(arg, rettv, evalarg, true)
    } else {
        NOTDONE
    }
}

/// FFI export for eval_dict.
///
/// # Safety
/// See `eval_dict_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_dict(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    literal: bool,
) -> c_int {
    eval_dict_impl(arg, rettv, evalarg, literal)
}

/// FFI export for eval_lit_dict.
///
/// # Safety
/// See `eval_lit_dict_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_lit_dict(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
) -> c_int {
    eval_lit_dict_impl(arg, rettv, evalarg)
}

// =============================================================================
// Phase 6: eval7 and eval7_leader migration
// =============================================================================

extern "C" {
    // eval7 C dependencies
    fn eval_interp_string(arg: *mut *mut c_char, rettv: TypevalHandle, evaluate: bool) -> c_int;
    fn handle_subscript(
        arg: *mut *const c_char,
        rettv: TypevalHandle,
        evalarg: EvalargHandle,
        verbose: bool,
    ) -> c_int;
    fn get_lambda_tv(arg: *mut *mut c_char, rettv: TypevalHandle, evalarg: EvalargHandle) -> c_int;
    fn eval_variable(
        name: *const c_char,
        len: c_int,
        rettv: TypevalHandle,
        dip: *mut *mut c_void,
        verbose: bool,
        no_autoload: bool,
    ) -> c_int;
    fn get_reg_contents(regname: c_int, flags: c_int) -> *mut c_char;
}

/// Error: missing ')'
static E_MISSING_PAREN_CLOSE: &[u8] = b"E110: Missing ')'\0";

/// Register flag: use expression register source
const K_G_REG_EXPR_SRC: c_int = 2;

/// Apply the leading "!" and "-" before an eval7 expression to "rettv".
/// Adjusts "end_leaderp" until it is at "start_leader".
///
/// If `numeric_only` is true, only handle "+" and "-" (not "!").
///
/// # Safety
/// - `rettv` must be a valid typval handle
/// - `start_leader` and `*end_leaderp` must be valid pointers into the same buffer
unsafe fn eval7_leader_impl(
    rettv: TypevalHandle,
    numeric_only: bool,
    start_leader: *const c_char,
    end_leaderp: *mut *const c_char,
) -> c_int {
    let mut end_leader = *end_leaderp;
    let mut ret = OK;
    let mut error = false;

    let is_float = nvim_tv_get_type(rettv) == VAR_FLOAT;
    let mut f: f64 = 0.0;
    let mut val: i64 = 0;

    if is_float {
        f = nvim_tv_get_float(rettv);
    } else {
        val = tv_get_number_chk(rettv, &mut error);
    }

    if error {
        tv_clear(rettv);
        ret = FAIL;
    } else {
        while end_leader > start_leader {
            end_leader = end_leader.sub(1);
            if *end_leader == b'!' as c_char {
                if numeric_only {
                    end_leader = end_leader.add(1);
                    break;
                }
                if is_float {
                    f = if f != 0.0 { 0.0 } else { 1.0 };
                } else {
                    val = if val != 0 { 0 } else { 1 };
                }
            } else if *end_leader == b'-' as c_char {
                if is_float {
                    f = -f;
                } else {
                    val = val.wrapping_neg();
                }
            }
            // '+' is ignored
        }

        if is_float {
            tv_clear(rettv);
            nvim_tv_set_type(rettv, VAR_FLOAT);
            nvim_tv_set_float(rettv, f);
        } else {
            tv_clear(rettv);
            nvim_tv_set_type(rettv, VAR_NUMBER);
            nvim_tv_set_number(rettv, val);
        }
    }

    *end_leaderp = end_leader;
    ret
}

/// Handle seventh level expression (lowest-precedence primaries).
///
/// Dispatches on the first character of `*arg` to parse numbers, strings,
/// lists, dicts, lambdas, options, env vars, registers, nested expressions,
/// function calls, and variable lookups. Also handles leading `!`/`-`/`+`
/// operators and trailing subscript/method chains.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer
/// - `rettv` must be a valid typval handle
/// - `evalarg` can be null
pub unsafe fn eval7_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    want_string: bool,
) -> c_int {
    // Static recursion counter - safe because Neovim is single-threaded
    #[allow(static_mut_refs)]
    static mut RECURSE: i32 = 0;

    let evaluate = should_evaluate(evalarg);
    let mut ret = OK;

    // Initialize rettv so tv_clear() won't mistake it for a string
    nvim_tv_set_type(rettv, VAR_UNKNOWN);

    // Skip '!', '-' and '+' characters. They are handled later.
    let start_leader = *arg as *const c_char;
    while get_byte(*arg) == b'!' || get_byte(*arg) == b'-' || get_byte(*arg) == b'+' {
        *arg = skipwhite((*arg).add(1));
    }
    let mut end_leader = *arg as *const c_char;

    // Limit recursion to prevent stack overflow
    #[allow(static_mut_refs)]
    if RECURSE == MAX_RECURSION {
        semsg(E_EXPRESSION_TOO_RECURSIVE.as_ptr() as *const c_char, *arg);
        return FAIL;
    }
    #[allow(static_mut_refs)]
    {
        RECURSE += 1;
    }

    let first_byte = get_byte(*arg);
    match first_byte {
        // Number constant: 0-9
        b'0'..=b'9' => {
            ret = eval_number_impl(arg, rettv, evaluate, want_string);
            // Apply prefixed "-" and "+" now. Matters especially when "->" follows.
            if ret == OK && evaluate && end_leader > start_leader {
                ret = eval7_leader_impl(rettv, true, start_leader, &mut end_leader);
            }
        }

        // String constant: "string"
        b'"' => {
            ret = eval_string_impl(arg, rettv, evaluate, false);
        }

        // Literal string constant: 'str''ing'
        b'\'' => {
            ret = eval_lit_string_impl(arg, rettv, evaluate, false);
        }

        // List: [expr, expr]
        b'[' => {
            ret = eval_list_impl(arg, rettv, evalarg);
        }

        // Literal Dictionary: #{key: val, key: val}
        b'#' => {
            ret = eval_lit_dict_impl(arg, rettv, evalarg);
        }

        // Lambda: {arg, arg -> expr} or Dictionary: {'key': val, 'key': val}
        b'{' => {
            ret = get_lambda_tv(arg, rettv, evalarg);
            if ret == NOTDONE {
                ret = eval_dict_impl(arg, rettv, evalarg, false);
            }
        }

        // Option value: &name
        b'&' => {
            ret = rs_eval_option(arg as *mut *const c_char, rettv, evaluate);
        }

        // Environment variable: $VAR or interpolated string: $"string" / $'string'
        b'$' => {
            let next = get_byte((*arg).add(1));
            if next == b'"' || next == b'\'' {
                ret = eval_interp_string(arg, rettv, evaluate);
            } else {
                ret = rs_eval_env_var(arg, rettv, if evaluate { 1 } else { 0 });
            }
        }

        // Register contents: @r
        b'@' => {
            *arg = (*arg).add(1);
            if evaluate {
                nvim_tv_set_type(rettv, VAR_STRING);
                let contents = get_reg_contents(get_byte(*arg) as c_int, K_G_REG_EXPR_SRC);
                nvim_tv_set_vstring_raw(rettv, contents);
            }
            if get_byte(*arg) != 0 {
                *arg = (*arg).add(1);
            }
        }

        // Nested expression: (expression)
        b'(' => {
            *arg = skipwhite((*arg).add(1));
            ret = eval1_impl(arg, rettv, evalarg); // recursive!
            if get_byte(*arg) == b')' {
                *arg = (*arg).add(1);
            } else if ret == OK {
                emsg(E_MISSING_PAREN_CLOSE.as_ptr() as *const c_char);
                tv_clear(rettv);
                ret = FAIL;
            }
        }

        // Default: variable or function name
        _ => {
            ret = NOTDONE;
        }
    }

    if ret == NOTDONE {
        // Must be a variable or function name.
        // Can also be a curly-braces kind of name: {expr}.
        let mut s = *arg;
        let mut alias: *mut c_char = ptr::null_mut();
        let len = get_name_len(arg as *mut *const c_char, &mut alias, evaluate, true);
        if !alias.is_null() {
            s = alias;
        }

        if len <= 0 {
            ret = FAIL;
        } else {
            let flags = get_eval_flags(evalarg);
            if get_byte(skipwhite(*arg)) == b'(' {
                // "name(..."  recursive!
                *arg = skipwhite(*arg);
                ret = eval_func_impl(arg, evalarg, s, len, rettv, flags, TypevalHandle::null());
            } else if evaluate {
                // get value of variable
                ret = eval_variable(s, len, rettv, ptr::null_mut(), true, false);
            } else {
                // skip the name
                check_vars(s, len as usize);
                // If evaluate is false rettv->v_type was not set, but it's needed
                // in handle_subscript() to parse v:lua, so set it here.
                if nvim_tv_get_type(rettv) == VAR_UNKNOWN && !evaluate {
                    // Check for "v:lua." prefix (6 bytes)
                    let p = s as *const u8;
                    let is_vlua = get_byte(p as *const c_char) == b'v'
                        && get_byte(p.add(1) as *const c_char) == b':'
                        && get_byte(p.add(2) as *const c_char) == b'l'
                        && get_byte(p.add(3) as *const c_char) == b'u'
                        && get_byte(p.add(4) as *const c_char) == b'a'
                        && get_byte(p.add(5) as *const c_char) == b'.';
                    if is_vlua {
                        let vlua = nvim_get_vlua_partial();
                        nvim_tv_set_type(rettv, VAR_PARTIAL);
                        nvim_tv_set_partial_raw(rettv, vlua);
                        nvim_eval_partial_incref(vlua);
                    }
                }
                ret = OK;
            }
        }

        xfree(alias as *mut c_void);
    }

    *arg = skipwhite(*arg);

    // Handle following '[', '(', '.' and '->' for expr[expr], expr.name,
    // expr(expr), expr->name(expr)
    if ret == OK {
        ret = handle_subscript(arg as *mut *const c_char, rettv, evalarg, true);
    }

    // Apply logical NOT and unary '-', from right to left, ignore '+'.
    if ret == OK && evaluate && end_leader > start_leader {
        ret = eval7_leader_impl(rettv, false, start_leader, &mut end_leader);
    }

    #[allow(static_mut_refs)]
    {
        RECURSE -= 1;
    }
    ret
}

/// FFI export for eval7.
///
/// # Safety
/// See `eval7_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval7(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    want_string: bool,
) -> c_int {
    eval7_impl(arg, rettv, evalarg, want_string)
}

// =============================================================================
// Phase 9: handle_subscript + set_selfdict
// =============================================================================

extern "C" {
    // Get tv->vval.v_dict
    fn nvim_tv_get_dict(tv: TypevalHandle) -> *mut c_void;
    // Increment dict->dv_refcount
    fn nvim_dict_refcount_inc(dict: *mut c_void);
    // tv_dict_unref wrapper
    fn nvim_dict_unref(dict: *mut c_void);
    // make_partial wrapper
    fn nvim_make_partial(selfdict: *mut c_void, rettv: TypevalHandle);
    // aborting() wrapper
    fn nvim_aborting() -> bool;
    // Get partial->pt_auto
    fn nvim_partial_get_pt_auto(pt: *const c_void) -> bool;
    // Get partial->pt_dict (for set_selfdict check)
    fn nvim_eval_partial_get_dict(pt: *const c_void) -> *mut c_void;
    // tv_is_func wrapper (returns int, matches other declarations in this crate)
    fn nvim_tv_is_func(tv: TypevalHandle) -> c_int;
    // rs_check_luafunc_name is in the eval crate (different crate)
    fn rs_check_luafunc_name(str: *const c_char, paren: bool) -> c_int;
}

/// Inline implementation of set_selfdict: bind selfdict to a funcref/partial.
///
/// Does nothing if `rettv` is a partial that was explicitly bound (pt_auto is false and pt_dict != NULL).
///
/// # Safety
/// - `rettv` must be a valid typval handle with type VAR_PARTIAL or VAR_FUNC
/// - `selfdict` must be a valid dict pointer (may be null)
#[inline]
unsafe fn set_selfdict_impl(rettv: TypevalHandle, selfdict: *mut c_void) {
    if nvim_tv_get_type(rettv) == VAR_PARTIAL {
        let pt = nvim_eval_tv_get_partial(rettv);
        if !pt.is_null()
            && !nvim_partial_get_pt_auto(pt)
            && !nvim_eval_partial_get_dict(pt).is_null()
        {
            return;
        }
    }
    nvim_make_partial(selfdict, rettv);
}

/// FFI export for set_selfdict: bind selfdict to a funcref/partial.
///
/// Called from the thin C wrapper `set_selfdict()` in `eval_shim.c`.
///
/// # Safety
/// - `rettv` must be a valid typval handle
/// - `selfdict` must be a valid dict pointer (may be null)
#[export_name = "set_selfdict"]
pub unsafe extern "C" fn rs_set_selfdict(rettv: TypevalHandle, selfdict: *mut c_void) {
    set_selfdict_impl(rettv, selfdict);
}

/// Implementation of handle_subscript: dispatch loop for subscripts, calls, lambdas, methods.
///
/// Migrated from C `handle_subscript` + `set_selfdict` in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a C string pointer
/// - `rettv` must be a valid typval handle
/// - `evalarg` can be null
pub unsafe fn handle_subscript_impl(
    arg: *mut *const c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    verbose: bool,
) -> c_int {
    let evaluate = !evalarg.is_null() && (evalarg.flags() & EVAL_EVALUATE) != 0;
    let mut ret = OK;
    let mut selfdict: *mut c_void = std::ptr::null_mut();
    let mut lua_funcname: *const c_char = std::ptr::null();

    // The arg we mutate is `*const char *` from C perspective (const char **arg).
    // We cast to *mut *mut c_char for internal use.
    let arg_mut = arg as *mut *mut c_char;

    if nvim_tv_get_type(rettv) == VAR_PARTIAL && rs_is_luafunc(nvim_eval_tv_get_partial(rettv)) {
        if !evaluate {
            tv_clear(rettv);
        }
        if get_byte(*arg_mut as *const c_char) != b'.' {
            tv_clear(rettv);
            ret = FAIL;
        } else {
            *arg_mut = (*arg_mut).add(1);
            lua_funcname = *arg_mut;
            let len = rs_check_luafunc_name(*arg_mut, true);
            if len == 0 {
                tv_clear(rettv);
                ret = FAIL;
            }
            *arg_mut = (*arg_mut).add(len as usize);
        }
    }

    // "." is ".name" lookup when we found a dict.
    while ret == OK {
        let ch = get_byte(*arg_mut as *const c_char);
        let prev_ch = get_byte((*arg_mut as *const c_char).sub(1));
        let next_ch = get_byte((*arg_mut as *const c_char).add(1));

        let tv_type = nvim_tv_get_type(rettv);
        let is_subscript = (ch == b'['
            || (ch == b'.' && tv_type == VAR_DICT)
            || (ch == b'(' && (!evaluate || nvim_tv_is_func(rettv) != 0)))
            && rs_ascii_iswhite(c_int::from(prev_ch)) == 0;
        let is_arrow = ch == b'-' && next_ch == b'>';

        if !is_subscript && !is_arrow {
            break;
        }

        if ch == b'(' {
            ret = call_func_rettv_impl(
                arg_mut,
                evalarg,
                rettv,
                evaluate,
                selfdict,
                TypevalHandle::null(),
                lua_funcname,
            );
            // Stop on aborting (interrupt, exception, etc.)
            if nvim_aborting() {
                if ret == OK {
                    tv_clear(rettv);
                }
                ret = FAIL;
            }
            nvim_dict_unref(selfdict);
            selfdict = std::ptr::null_mut();
        } else if ch == b'-' {
            let after_arrow = get_byte((*arg_mut as *const c_char).add(2));
            if after_arrow == b'{' {
                // expr->{lambda}()
                ret = eval_lambda_impl(arg_mut, rettv, evalarg, verbose);
            } else {
                // expr->name()
                ret = eval_method_impl(arg_mut, rettv, evalarg, verbose);
            }
        } else {
            // '[' or '.'
            nvim_dict_unref(selfdict);
            if nvim_tv_get_type(rettv) == VAR_DICT {
                selfdict = nvim_tv_get_dict(rettv);
                nvim_dict_refcount_inc(selfdict);
            } else {
                selfdict = std::ptr::null_mut();
            }
            if crate::index::eval_index_impl(arg_mut, rettv, evalarg, verbose) == FAIL {
                tv_clear(rettv);
                ret = FAIL;
            }
        }
    }

    // Turn "dict.Func" into a partial for "Func" bound to "dict".
    if !selfdict.is_null() && nvim_tv_is_func(rettv) != 0 {
        set_selfdict_impl(rettv, selfdict);
    }

    nvim_dict_unref(selfdict);
    ret
}

/// FFI export for handle_subscript.
///
/// # Safety
/// See `handle_subscript_impl` for safety requirements.
#[export_name = "handle_subscript"]
pub unsafe extern "C" fn rs_handle_subscript(
    arg: *mut *const c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    verbose: bool,
) -> c_int {
    handle_subscript_impl(arg, rettv, evalarg, verbose)
}

// =============================================================================
// Phase 1 (eval_shim pass 4): call_func_rettv + eval_lambda + eval1_emsg
// =============================================================================

extern "C" {
    // rs_is_luafunc from eval crate
    fn rs_is_luafunc(pt: *const c_void) -> bool;

    // Wrap get_lambda_tv
    fn nvim_get_lambda_tv(
        arg: *mut *mut c_char,
        rettv: TypevalHandle,
        evalarg: EvalargHandle,
    ) -> c_int;
    // Emit e_nowhitespace, e_missingparen, e_empty_function_name: now in nvim_eval::errors
    // Raw copy typval bytes from src to dst, sets src type to VAR_UNKNOWN
    fn nvim_tv_raw_copy_and_reset(dst: TypevalHandle, src: TypevalHandle);
}

/// Implementation of call_func_rettv: invoke a funcref/partial stored in rettv.
///
/// Equivalent to the C `call_func_rettv` static function in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer (NONNULL)
/// - `rettv` must be a valid typval handle (NONNULL)
/// - `evalarg` may be null
/// - `selfdict` may be null
/// - `basetv` may be null
/// - `lua_funcname` may be null
unsafe fn call_func_rettv_impl(
    arg: *mut *mut c_char,
    evalarg: EvalargHandle,
    rettv: TypevalHandle,
    evaluate: bool,
    selfdict: *mut c_void,
    basetv: TypevalHandle,
    lua_funcname: *const c_char,
) -> c_int {
    let mut pt: *mut c_void = std::ptr::null_mut();
    let mut is_lua = false;

    // Allocate a temporary typval for saving rettv contents
    let functv = alloc_typval();

    let funcname: *const c_char;
    if evaluate {
        // Copy *rettv into functv, then reset rettv to VAR_UNKNOWN
        nvim_tv_raw_copy_and_reset(functv, rettv);

        let tv_type = nvim_tv_get_type(functv);
        if tv_type == VAR_PARTIAL {
            pt = nvim_eval_tv_get_partial(functv);
            is_lua = rs_is_luafunc(pt);
            funcname = if is_lua {
                lua_funcname
            } else {
                rs_partial_name(pt) as *const c_char
            };
        } else {
            let vstr = nvim_tv_get_vstring(functv) as *const c_char;
            if vstr.is_null() || *vstr == 0 {
                nvim_eval::errors::emsg_e_empty_function_name();
                // jump to theend
                tv_clear(functv);
                free_typval(functv);
                return FAIL;
            }
            funcname = vstr;
        }
    } else {
        // Not evaluating: use empty string as funcname
        funcname = c"".as_ptr();
    }

    let lnum = nvim_curwin_get_cursor_lnum();
    let name_len: c_int = if is_lua {
        // lua funcname length: from funcname to current *arg
        ((*arg as usize).wrapping_sub(funcname as usize)) as c_int
    } else {
        -1
    };

    let mut funcexe2 = FuncExeT::new();
    funcexe2.fe_firstline = lnum;
    funcexe2.fe_lastline = lnum;
    funcexe2.fe_evaluate = evaluate;
    funcexe2.fe_partial = pt;
    funcexe2.fe_selfdict = selfdict;
    funcexe2.fe_basetv = basetv.as_ptr();
    let ret = get_func_tv(
        funcname,
        name_len,
        rettv.as_ptr(),
        arg,
        evalarg.as_ptr() as *mut c_void,
        &mut funcexe2,
    );

    // theend: clear the saved funcref
    if evaluate {
        tv_clear(functv);
    }
    free_typval(functv);

    ret
}

/// FFI export for call_func_rettv (selfdict=NULL, basetv provided).
///
/// Replaces `nvim_call_func_rettv_wrapper` (selfdict=NULL variant).
///
/// # Safety
/// See `call_func_rettv_impl`.
#[no_mangle]
pub unsafe extern "C" fn rs_call_func_rettv(
    arg: *mut *mut c_char,
    evalarg: EvalargHandle,
    rettv: TypevalHandle,
    evaluate: bool,
    selfdict: *mut c_void,
    basetv: TypevalHandle,
    lua_funcname: *const c_char,
) -> c_int {
    call_func_rettv_impl(
        arg,
        evalarg,
        rettv,
        evaluate,
        selfdict,
        basetv,
        lua_funcname,
    )
}

/// Error message: "lambda"
static E_LAMBDA_NAME: &[u8] = b"lambda\0";

/// Implementation of eval_lambda: evaluate "->{ ... }()".
///
/// Equivalent to the C `eval_lambda` static function in eval_shim.c.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer (NONNULL)
/// - `rettv` must be a valid typval handle (NONNULL)
/// - `evalarg` may be null
unsafe fn eval_lambda_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    verbose: bool,
) -> c_int {
    let evaluate = !evalarg.is_null() && (evalarg.flags() & EVAL_EVALUATE) != 0;

    // Skip over the ->
    *arg = (*arg).add(2);

    // Save base typval and reset rettv
    let base = alloc_typval();
    nvim_tv_raw_copy_and_reset(base, rettv);

    let ret;
    let lambda_ret = nvim_get_lambda_tv(arg, rettv, evalarg);
    if lambda_ret != OK {
        ret = FAIL;
    } else if get_byte(*arg) != b'(' {
        if verbose {
            if get_byte(skipwhite(*arg)) == b'(' {
                nvim_eval::errors::emsg_e_nowhitespace();
            } else {
                nvim_eval::errors::semsg_e_missingparen(E_LAMBDA_NAME.as_ptr() as *const c_char);
            }
        }
        tv_clear(rettv);
        ret = FAIL;
    } else {
        ret = call_func_rettv_impl(
            arg,
            evalarg,
            rettv,
            evaluate,
            std::ptr::null_mut(),
            base,
            std::ptr::null(),
        );
    }

    // Clear the funcref afterwards
    if evaluate {
        tv_clear(base);
    }
    free_typval(base);

    ret
}

/// FFI export for eval_lambda.
///
/// # Safety
/// See `eval_lambda_impl`.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_lambda(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    verbose: bool,
) -> c_int {
    eval_lambda_impl(arg, rettv, evalarg, verbose)
}

// eval1_emsg implementation is in eval_top.rs (rs_eval1_emsg).

// =============================================================================
// Phase 2 (eval_shim pass 4): eval_option + eval_env_var
// =============================================================================

extern "C" {
    // Check if a name is a tty option (from strings crate)
    fn rs_is_tty_option(name: *const c_char) -> c_int;
    // Get length of env var name, advancing *arg past the name (from eval crate)
    fn rs_get_env_len(arg: *mut *const c_char) -> c_int;
    // Parse &[g:|l:]optname from *arg, set opt_idx and opt_flags, return end pointer.
    fn nvim_find_option_var_end(
        arg: *mut *const c_char,
        opt_idxp: *mut c_int,
        opt_flagsp: *mut c_int,
    ) -> *const c_char;
    // Check if option is hidden (returns non-zero if hidden)
    #[link_name = "is_option_hidden"]
    fn nvim_opt_is_hidden(opt_idx: c_int) -> c_int;
    // Get option value as typval (get_option_value + optval_as_tv)
    fn nvim_get_option_value_as_tv(opt_idx: c_int, opt_flags: c_int, rettv: TypevalHandle);
    // Get tty option value as typval
    fn nvim_get_tty_option_as_tv(name: *const c_char, rettv: TypevalHandle);
    // Error messages for eval_option
    // nvim_semsg_e112_option_name_missing and nvim_semsg_e113_unknown_option: now in nvim_eval::errors
    // vim_getenv: returns allocated string or NULL
    fn nvim_vim_getenv(name: *const c_char) -> *mut c_char;
    // expand_env_save: expand $VAR from src
    fn nvim_expand_env_save(src: *const c_char) -> *mut c_char;
    // v_lock setter (VAR_UNLOCKED = 0)
    fn nvim_tv_set_v_lock(tv: TypevalHandle, lock: c_int);
}

/// kOptInvalid value - must match C kOptInvalid
const K_OPT_INVALID: c_int = -1;

/// Evaluate `&option`, `&g:option`, `&l:option` expressions.
///
/// # Safety
/// - `arg` must be a non-null pointer to a C string pointer pointing to `&` or `+`
/// - `rettv` may be null (caller won't use the value)
///
/// # C equivalent
/// Replaces the C `eval_option` function in eval_shim.c.
#[export_name = "eval_option"]
pub unsafe extern "C" fn rs_eval_option(
    arg: *mut *const c_char,
    rettv: TypevalHandle,
    evaluate: bool,
) -> c_int {
    let working = get_byte(*arg) == b'+'; // has("+option")
    let mut opt_idx: c_int = K_OPT_INVALID;
    let mut opt_flags: c_int = 0;

    // Parse the option name, advance *arg to option name, return end pointer.
    let option_end = nvim_find_option_var_end(arg, &mut opt_idx, &mut opt_flags);

    if option_end.is_null() {
        if !rettv.is_null() {
            nvim_eval::errors::semsg_e112_option_name_missing(*arg);
        }
        return FAIL;
    }

    if !evaluate {
        *arg = option_end;
        return OK;
    }

    // Temporarily NUL-terminate the option name for API calls that need it.
    let c = *option_end;
    // SAFETY: option_end is a valid mutable pointer (it points into the
    // source buffer which is not const at this position in C).
    let option_end_mut = option_end as *mut c_char;
    *option_end_mut = 0;

    let ret;
    let is_tty_opt = rs_is_tty_option(*arg) != 0;

    if opt_idx == K_OPT_INVALID && !is_tty_opt {
        // Only give error if result is going to be used.
        if !rettv.is_null() {
            nvim_eval::errors::semsg_e113_unknown_option(*arg);
        }
        ret = FAIL;
    } else if !rettv.is_null() {
        if is_tty_opt {
            nvim_get_tty_option_as_tv(*arg, rettv);
        } else {
            nvim_get_option_value_as_tv(opt_idx, opt_flags, rettv);
        }
        ret = OK;
    } else if working && !is_tty_opt && nvim_opt_is_hidden(opt_idx) != 0 {
        ret = FAIL;
    } else {
        ret = OK;
    }

    // Restore original character.
    *option_end_mut = c;
    *arg = option_end;

    ret
}

/// Evaluate `$ENVVAR` expressions.
///
/// # Safety
/// - `arg` must be a valid pointer to a C string pointer pointing to `$`
/// - `rettv` may be null if evaluate is 0
///
/// # C equivalent
/// Replaces the static C `eval_env_var` function in eval_shim.c.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_env_var(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evaluate: c_int,
) -> c_int {
    // Advance past '$'
    *arg = (*arg).add(1);
    let name = *arg;
    let len = rs_get_env_len(arg as *mut *const c_char);

    if evaluate != 0 {
        if len == 0 {
            return FAIL; // Invalid empty name.
        }
        let cc = *name.add(len as usize) as c_int;
        *name.add(len as usize) = 0; // NUL-terminate temporarily

        // First try vim_getenv() - fast for normal env vars.
        let mut string = nvim_vim_getenv(name);
        if string.is_null() || *string == 0 {
            xfree(string as *mut c_void);

            // Next try expanding things like $VIM and ${HOME}.
            // Pass name-1 to include the '$' prefix for expand_env_save.
            string = nvim_expand_env_save(name.sub(1));
            if !string.is_null() && *string == b'$' as c_char {
                xfree(string as *mut c_void);
                string = std::ptr::null_mut();
            }
        }

        // Restore the original character.
        *name.add(len as usize) = cc as c_char;

        nvim_tv_set_type(rettv, VAR_STRING);
        nvim_tv_set_vstring_raw(rettv, string);
        nvim_tv_set_v_lock(rettv, VAR_UNLOCKED);
    }

    OK
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
