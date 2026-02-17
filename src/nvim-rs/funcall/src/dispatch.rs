//! Function call dispatch infrastructure.
//!
//! This module provides the core function call dispatch logic,
//! including function lookup, partial handling, and argument
//! management. Migrated from `src/nvim/eval/userfunc.c`.
//!
//! ## Call Flow
//!
//! 1. Parse function name and handle `g:` prefix
//! 2. Look up partial or find function by name
//! 3. Handle partial arguments (prepend to call args)
//! 4. Route to appropriate handler (builtin, user, lua)
//! 5. Handle errors and return value

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void};

use super::{ArgSpec, FuncError, FuncType};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of function arguments.
pub const MAX_FUNC_ARGS: usize = 20;

/// Function name buffer size.
pub const FLEN_FIXED: usize = 40;

// Function call error codes (matching C's FnameTransError)
const FCERR_NONE: c_int = 0;
const FCERR_UNKNOWN: c_int = 1;
const FCERR_TOOMANY: c_int = 2;
const FCERR_TOOFEW: c_int = 3;
const FCERR_DICT: c_int = 4;
const FCERR_SELFDICT: c_int = 5;
const FCERR_DELETED: c_int = 6;
const FCERR_NOTMETHOD: c_int = 7;

// Return values
const OK: c_int = 1;
const FAIL: c_int = 0;
const NOTDONE: c_int = 2;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a typval_T.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct TypevalHandle(*mut c_void);

impl TypevalHandle {
    /// Create a null handle.
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if null.
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get raw pointer.
    pub fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to a ufunc_T (user function).
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct UfuncHandle(*mut c_void);

impl UfuncHandle {
    /// Create a null handle.
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if null.
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get raw pointer.
    pub fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to a partial_T.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct PartialHandle(*mut c_void);

impl PartialHandle {
    /// Create a null handle.
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if null.
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get raw pointer.
    pub fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to funcexe_T (function execution context).
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct FuncexeHandle(*mut c_void);

impl FuncexeHandle {
    /// Create a null handle.
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if null.
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // Function lookup
    fn find_func(name: *const c_char) -> UfuncHandle;

    // Function info access
    fn nvim_ufunc_get_flags(fp: UfuncHandle) -> c_int;
    fn nvim_ufunc_get_min_args(fp: UfuncHandle) -> c_int;
    fn nvim_ufunc_get_max_args(fp: UfuncHandle) -> c_int;

    // Partial access
    fn nvim_partial_get_func(partial: PartialHandle) -> UfuncHandle;
    fn nvim_partial_get_argc(partial: PartialHandle) -> c_int;
    fn nvim_partial_get_dict(partial: PartialHandle) -> *mut c_void;
    fn nvim_partial_is_auto(partial: PartialHandle) -> c_int;

    // Funcexe access
    fn nvim_funcexe_get_partial(fe: FuncexeHandle) -> PartialHandle;
    fn nvim_funcexe_get_selfdict(fe: FuncexeHandle) -> *mut c_void;
    fn nvim_funcexe_get_evaluate(fe: FuncexeHandle) -> c_int;

    // Function type checks
    fn nvim_is_builtin_function(name: *const c_char, len: c_int) -> c_int;
    fn rs_is_luafunc(partial: PartialHandle) -> c_int;

    // Typval operations
    fn tv_clear(tv: TypevalHandle);
    fn tv_copy(from: TypevalHandle, to: TypevalHandle);
    fn nvim_tv_set_number(tv: TypevalHandle, n: i64);
    fn nvim_tv_set_type(tv: TypevalHandle, vtype: c_int);

    // Execution
    fn call_internal_func(
        name: *const c_char,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: TypevalHandle,
    ) -> c_int;
    fn call_internal_method(
        name: *const c_char,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: TypevalHandle,
        basetv: TypevalHandle,
    ) -> c_int;
    fn call_user_func_check(
        fp: UfuncHandle,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: TypevalHandle,
        funcexe: FuncexeHandle,
        selfdict: *mut c_void,
    ) -> c_int;
    fn nlua_typval_call(
        name: *const c_char,
        name_len: usize,
        argvars: *mut c_void,
        argcount: c_int,
        rettv: TypevalHandle,
    );

    // Error handling
    fn aborting() -> c_int;
    fn user_func_error(error: c_int, name: *const c_char, found_var: c_int);
    fn update_force_abort();

    // Script autoload
    fn script_autoload(name: *const c_char, name_len: usize, reload: c_int) -> c_int;

    // Autocmd
    fn apply_autocmds_for_funcundefined(name: *const c_char) -> c_int;
}

// =============================================================================
// Function Flags (matching C's FC_* flags)
// =============================================================================

bitflags::bitflags! {
    /// Flags from ufunc_T.uf_flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UfuncFlags: u32 {
        /// Function was created with :function!
        const BANG = 0x01;
        /// Function handles abort
        const ABORT = 0x02;
        /// Function takes range
        const RANGE = 0x04;
        /// Function is a closure
        const CLOSURE = 0x08;
        /// Function is a lambda
        const LAMBDA = 0x10;
        /// Function is being deleted
        const DELETED = 0x20;
        /// Function is a method
        const METHOD = 0x40;
        /// Function is variadic
        const VARARGS = 0x80;
        /// Function is for dict
        const DICT = 0x100;
        /// Function is in sandbox
        const SANDBOX = 0x200;
        /// Function is profiled
        const PROFILING = 0x400;
        /// Function is lua
        const LUA = 0x800;
    }
}

// =============================================================================
// Function Lookup
// =============================================================================

/// Result of looking up a function.
#[derive(Debug)]
pub enum FuncLookupResult {
    /// Found a user function.
    UserFunc(UfuncHandle),
    /// Found a builtin function.
    Builtin,
    /// Found a Lua function (via partial).
    Lua,
    /// Function not found.
    NotFound,
    /// Function was deleted.
    Deleted,
}

/// Look up a function by name, handling partials and builtins.
///
/// # Safety
/// - `name` must be a valid null-terminated C string
/// - `partial` may be null
pub unsafe fn lookup_function(
    name: *const c_char,
    name_len: c_int,
    partial: PartialHandle,
) -> FuncLookupResult {
    // Check for Lua function via partial
    if !partial.is_null() && rs_is_luafunc(partial) != 0 {
        return FuncLookupResult::Lua;
    }

    // Get function pointer from partial if available
    let mut fp = if !partial.is_null() {
        nvim_partial_get_func(partial)
    } else {
        UfuncHandle::null()
    };

    if fp.is_null() {
        // Check if it's a builtin function
        if nvim_is_builtin_function(name, name_len) != 0 {
            return FuncLookupResult::Builtin;
        }

        // Look up user function
        fp = find_func(name);

        // Try FuncUndefined autocmd
        if fp.is_null() && apply_autocmds_for_funcundefined(name) != 0 && aborting() == 0 {
            fp = find_func(name);
        }

        // Try autoloading
        if fp.is_null() {
            let len = if name_len > 0 {
                name_len as usize
            } else {
                // Calculate length
                let mut l = 0usize;
                let mut p = name;
                while *p != 0 {
                    l += 1;
                    p = p.add(1);
                }
                l
            };
            if script_autoload(name, len, 1) != 0 && aborting() == 0 {
                fp = find_func(name);
            }
        }
    }

    if fp.is_null() {
        return FuncLookupResult::NotFound;
    }

    // Check if function was deleted
    let flags = nvim_ufunc_get_flags(fp) as u32;
    if flags & UfuncFlags::DELETED.bits() != 0 {
        return FuncLookupResult::Deleted;
    }

    FuncLookupResult::UserFunc(fp)
}

/// Check if a function name starts with "g:" (global scope).
///
/// # Safety
/// - `name` must be valid for at least 2 bytes if non-null
pub unsafe fn is_global_function(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    let b0 = *name as u8;
    let b1 = *name.add(1) as u8;
    b0 == b'g' && b1 == b':'
}

/// Skip the "g:" prefix from a function name if present.
///
/// # Safety
/// - `name` must be a valid pointer
pub unsafe fn skip_global_prefix(name: *const c_char) -> *const c_char {
    if is_global_function(name) {
        name.add(2)
    } else {
        name
    }
}

// =============================================================================
// Argument Validation
// =============================================================================

/// Validate argument count against function specification.
pub fn validate_args_for_func(fp: UfuncHandle, argcount: c_int) -> c_int {
    if fp.is_null() {
        return FCERR_UNKNOWN;
    }

    unsafe {
        let min_args = nvim_ufunc_get_min_args(fp);
        let max_args = nvim_ufunc_get_max_args(fp);

        if argcount < min_args {
            FCERR_TOOFEW
        } else if max_args >= 0 && argcount > max_args {
            FCERR_TOOMANY
        } else {
            FCERR_NONE
        }
    }
}

/// Get argument spec from a user function.
pub fn get_func_argspec(fp: UfuncHandle) -> ArgSpec {
    if fp.is_null() {
        return ArgSpec::new(0, 0);
    }

    unsafe {
        let min_args = nvim_ufunc_get_min_args(fp);
        let max_args = nvim_ufunc_get_max_args(fp);
        ArgSpec::new(min_args, max_args)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Look up a function and return its type.
///
/// # Safety
/// - `name` must be a valid null-terminated string
#[no_mangle]
pub unsafe extern "C" fn rs_funcall_lookup(
    name: *const c_char,
    name_len: c_int,
    partial: PartialHandle,
) -> c_int {
    match lookup_function(name, name_len, partial) {
        FuncLookupResult::UserFunc(_) => FuncType::User as c_int,
        FuncLookupResult::Builtin => FuncType::Builtin as c_int,
        FuncLookupResult::Lua => FuncType::Lua as c_int,
        FuncLookupResult::NotFound => -1,
        FuncLookupResult::Deleted => -2,
    }
}

/// Check if a function name has global prefix.
///
/// # Safety
/// - `name` must be valid for at least 2 bytes if non-null
#[no_mangle]
pub unsafe extern "C" fn rs_funcall_is_global(name: *const c_char) -> c_int {
    c_int::from(is_global_function(name))
}

/// Skip global prefix from function name.
///
/// # Safety
/// - `name` must be a valid pointer
#[no_mangle]
pub unsafe extern "C" fn rs_funcall_skip_global(name: *const c_char) -> *const c_char {
    skip_global_prefix(name)
}

/// Validate argument count for a function.
#[no_mangle]
pub extern "C" fn rs_funcall_validate_for_func(fp: UfuncHandle, argcount: c_int) -> c_int {
    validate_args_for_func(fp, argcount)
}

/// Get minimum arguments for a function.
///
/// # Safety
/// - `fp` must be a valid ufunc handle or null
#[no_mangle]
pub unsafe extern "C" fn rs_funcall_get_min_args(fp: UfuncHandle) -> c_int {
    if fp.is_null() {
        0
    } else {
        nvim_ufunc_get_min_args(fp)
    }
}

/// Get maximum arguments for a function (-1 for unlimited).
///
/// # Safety
/// - `fp` must be a valid ufunc handle or null
#[no_mangle]
pub unsafe extern "C" fn rs_funcall_get_max_args(fp: UfuncHandle) -> c_int {
    if fp.is_null() {
        0
    } else {
        nvim_ufunc_get_max_args(fp)
    }
}

/// Convert function call error code to FuncError.
#[no_mangle]
pub extern "C" fn rs_funcall_error_from_fcerr(fcerr: c_int) -> FuncError {
    match fcerr {
        FCERR_NONE => FuncError::None,
        FCERR_TOOMANY => FuncError::E118WrongArgCount,
        FCERR_TOOFEW => FuncError::E119NotEnoughArgs,
        FCERR_UNKNOWN => FuncError::E117FuncNotFound,
        FCERR_DELETED => FuncError::E131CannotDelete,
        FCERR_NOTMETHOD => FuncError::E276CannotUseAsMethod,
        _ => FuncError::None,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ufunc_flags() {
        let flags = UfuncFlags::ABORT | UfuncFlags::RANGE;
        assert!(flags.contains(UfuncFlags::ABORT));
        assert!(flags.contains(UfuncFlags::RANGE));
        assert!(!flags.contains(UfuncFlags::DELETED));
    }

    #[test]
    fn test_max_func_args() {
        assert_eq!(MAX_FUNC_ARGS, 20);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(FCERR_NONE, 0);
        assert_eq!(FCERR_TOOMANY, 2);
        assert_eq!(FCERR_TOOFEW, 3);
    }
}
