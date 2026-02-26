//! FuncExeT: Rust mirror of C's funcexe_T struct used for function calls.
//!
//! This replaces the C wrappers `nvim_callback_call_func`,
//! `nvim_call_func_tv_wrapper`, `nvim_eval_call_func_partial`,
//! `nvim_eval_call_func_simple`, `nvim_call_func_tv_with_selfdict`,
//! `nvim_call_func_with_partial`, and `nvim_eval_provider_call_func`.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// FuncExeT: #[repr(C)] mirror of funcexe_T
// =============================================================================

/// Function pointer type matching C `ArgvFunc`.
///
/// C definition: `int (*ArgvFunc)(int current_argcount, typval_T *argv,
///                                int partial_argcount, ufunc_T *called_func)`
type ArgvFunc = Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, *mut c_void) -> c_int>;

/// Rust mirror of the C `funcexe_T` struct.
///
/// Fields must match exactly (order and types) the C definition in
/// `src/nvim/eval/userfunc.h` lines 64-76.
///
/// # C definition:
/// ```c
/// typedef struct {
///   ArgvFunc fe_argv_func;     // offset 0, size 8 (fn ptr)
///   linenr_T fe_firstline;     // offset 8, size 4
///   linenr_T fe_lastline;      // offset 12, size 4
///   bool *fe_doesrange;        // offset 16, size 8
///   bool fe_evaluate;          // offset 24, size 1 + 7 pad
///   partial_T *fe_partial;     // offset 32, size 8
///   dict_T *fe_selfdict;       // offset 40, size 8
///   typval_T *fe_basetv;       // offset 48, size 8
///   bool fe_found_var;         // offset 56, size 1 + 7 pad
/// } funcexe_T;                 // sizeof = 64
/// ```
///
/// The `FUNCEXE_INIT` macro zero-initializes all fields, matching `FuncExeT::new()`.
#[repr(C)]
pub struct FuncExeT {
    /// When not NULL, can be used to fill in arguments only when invoked function uses them.
    /// Always NULL in our usage (only set in regexp_shim.c).
    pub fe_argv_func: ArgvFunc,
    /// First line of range (linenr_T = int32_t).
    pub fe_firstline: i32,
    /// Last line of range (linenr_T = int32_t).
    pub fe_lastline: i32,
    /// [out] if not NULL: function handled range. Always NULL in our usage.
    pub fe_doesrange: *mut bool,
    /// Actually evaluate expressions.
    pub fe_evaluate: bool,
    // 7 bytes padding (added by #[repr(C)] to align fe_partial to 8 bytes)
    /// For extra arguments (partial_T*).
    pub fe_partial: *mut c_void,
    /// Dict for "self" (dict_T*).
    pub fe_selfdict: *mut c_void,
    /// Base for base->method() (typval_T*).
    pub fe_basetv: *mut c_void,
    /// If the function is not found, give an error that a variable is not callable.
    pub fe_found_var: bool,
    // 7 bytes trailing padding
}

impl FuncExeT {
    /// Create a zero-initialized funcexe_T.
    ///
    /// Matches `FUNCEXE_INIT` macro behavior: all fields NULL/false/0.
    pub fn new() -> Self {
        Self {
            fe_argv_func: None,
            fe_firstline: 0,
            fe_lastline: 0,
            fe_doesrange: std::ptr::null_mut(),
            fe_evaluate: false,
            fe_partial: std::ptr::null_mut(),
            fe_selfdict: std::ptr::null_mut(),
            fe_basetv: std::ptr::null_mut(),
            fe_found_var: false,
        }
    }
}

impl Default for FuncExeT {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Direct C extern declarations for call_func and get_func_tv
// =============================================================================

extern "C" {
    /// Call a VimL function by name.
    ///
    /// C signature: `int call_func(const char *funcname, int len, typval_T *rettv,
    ///                             int argcount, typval_T *argvars, funcexe_T *funcexe)`
    pub fn call_func(
        funcname: *const c_char,
        len: c_int,
        rettv: *mut c_void, // typval_T*
        argcount: c_int,
        argvars: *mut c_void, // typval_T*
        funcexe: *mut FuncExeT,
    ) -> c_int;

    /// Get function typval by name, parsing from arg string.
    ///
    /// C signature: `int get_func_tv(const char *name, int len, typval_T *rettv,
    ///                               char **arg, evalarg_T *evalarg, funcexe_T *funcexe)`
    pub fn get_func_tv(
        name: *const c_char,
        len: c_int,
        rettv: *mut c_void, // typval_T*
        arg: *mut *mut c_char,
        evalarg: *mut c_void, // evalarg_T*
        funcexe: *mut FuncExeT,
    ) -> c_int;
}
