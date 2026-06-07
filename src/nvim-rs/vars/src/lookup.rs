//! Variable lookup and scope resolution for VimL.
//!
//! This module provides Rust implementations of variable lookup functions
//! from `src/nvim/eval/vars.c`. It handles:
//!
//! - Scope detection from variable name prefixes (g:, l:, s:, etc.)
//! - Variable lookup in appropriate hashtables
//! - Support for implicit scope resolution
//!
//! ## Scope Prefixes
//!
//! - `g:` - Global variables
//! - `v:` - Vim predefined variables
//! - `b:` - Buffer-local variables
//! - `w:` - Window-local variables
//! - `t:` - Tab-local variables
//! - `l:` - Function-local variables
//! - `s:` - Script-local variables
//! - `a:` - Function argument variables

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::if_not_else)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Autoload character in variable names
const AUTOLOAD_CHAR: u8 = b'#';

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a hashtab_T.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct HashtabHandle(*mut c_void);

impl HashtabHandle {
    /// Create a null handle.
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if the handle is null.
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer.
    pub fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to a dictitem_T.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DictitemHandle(*mut c_void);

impl DictitemHandle {
    /// Create a null handle.
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if the handle is null.
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer.
    pub fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to a dict_T.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DictHandle(*mut c_void);

impl DictHandle {
    /// Create a null handle.
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if the handle is null.
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer.
    pub fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

// =============================================================================
// Scope Detection
// =============================================================================

/// Variable scope prefix.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopePrefix {
    /// No explicit scope prefix (implicit scope)
    Implicit = 0,
    /// Global scope (g:)
    Global = b'g',
    /// Vim predefined variables (v:)
    Vim = b'v',
    /// Buffer-local (b:)
    Buffer = b'b',
    /// Window-local (w:)
    Window = b'w',
    /// Tab-local (t:)
    Tab = b't',
    /// Function-local (l:)
    Local = b'l',
    /// Script-local (s:)
    Script = b's',
    /// Function arguments (a:)
    Argument = b'a',
}

impl ScopePrefix {
    /// Parse a scope prefix from a variable name.
    ///
    /// Returns the scope prefix and the index where the actual name starts.
    /// For implicit scope, the name starts at index 0.
    /// For explicit scope (like "g:var"), the name starts at index 2.
    #[inline]
    pub fn from_name(name: &[u8]) -> (Self, usize) {
        if name.len() <= 1 {
            return (Self::Implicit, 0);
        }

        // Check for explicit scope prefix (letter followed by colon)
        if name.len() >= 2 && name[1] == b':' {
            let prefix = match name[0] {
                b'g' => Self::Global,
                b'v' => Self::Vim,
                b'b' => Self::Buffer,
                b'w' => Self::Window,
                b't' => Self::Tab,
                b'l' => Self::Local,
                b's' => Self::Script,
                b'a' => Self::Argument,
                _ => return (Self::Implicit, 0),
            };
            return (prefix, 2);
        }

        (Self::Implicit, 0)
    }

    /// Check if this is an explicit scope (has a prefix like "g:")
    #[inline]
    pub const fn is_explicit(self) -> bool {
        !matches!(self, Self::Implicit)
    }
}

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // Scope dictionaries
    fn get_globvar_dict() -> DictHandle;
    fn get_vimvar_dict() -> DictHandle;
    fn get_funccal_local_dict() -> DictHandle;
    fn get_funccal_args_dict() -> DictHandle;

    // Buffer/window/tab variables
    fn nvim_curbuf_get_vars() -> DictHandle;
    fn nvim_curwin_get_vars() -> DictHandle;
    fn nvim_curtab_get_vars() -> DictHandle;

    // Dictionary hashtab access
    fn nvim_dict_get_hashtab(dict: DictHandle) -> HashtabHandle;

    // Hashtab operations
    fn hash_find_len(ht: HashtabHandle, key: *const c_char, len: usize) -> *mut c_void;
    fn nvim_hashitem_empty(hi: *mut c_void) -> c_int;
    fn nvim_hi2dictitem(hi: *mut c_void) -> DictitemHandle;

    // Compat hashtab (for variables like "version" accessible without v:)
    fn nvim_get_compat_hashtab() -> HashtabHandle;

    // Script variables
    fn nvim_get_script_vars_dict(sid: c_int) -> DictHandle;
    fn nvim_get_current_sctx_sid() -> c_int;

    // Variable lookup
    fn find_var(
        name: *const c_char,
        name_len: usize,
        htp: *mut HashtabHandle,
        no_autoload: c_int,
    ) -> DictitemHandle;

    fn find_var_in_scoped_ht(
        name: *const c_char,
        name_len: usize,
        no_autoload: c_int,
    ) -> DictitemHandle;

    // Dictitem access
    fn nvim_dictitem_get_tv(di: DictitemHandle) -> *mut c_void;
    fn tv_get_string(tv: *mut c_void) -> *const c_char;

    // eval_variable / check_vars additions
    fn semsg(fmt: *const c_char, ...) -> c_int;
    fn tv_copy(from: *mut c_void, to: *mut c_void);
    fn nvim_vars_get_eval_lavars_used() -> *mut bool;
    fn nvim_vars_get_funccal_local_ht() -> *mut c_void;
    fn nvim_vars_get_funccal_args_ht() -> *mut c_void;

    // Phase 13: scope-dict item accessors (return DictitemHandle)
    fn nvim_vars_globvars_var() -> DictitemHandle;
    fn nvim_vars_vimvars_var() -> DictitemHandle;
    fn nvim_vars_curbuf_bufvar() -> DictitemHandle;
    fn nvim_vars_curwin_winvar() -> DictitemHandle;
    fn nvim_vars_curtab_winvar() -> DictitemHandle;
    fn nvim_vars_script_sv_var(sid: c_int) -> DictitemHandle;

    // Funccal scope-var accessors (linkable from userfunc.c)
    fn get_funccal_local_var() -> DictitemHandle;
    fn get_funccal_args_var() -> DictitemHandle;

    // Globvar hashtab (for autoload comparison)
    fn get_globvar_ht() -> HashtabHandle;

    // Autoload
    fn script_autoload(name: *const c_char, name_len: usize, reload: bool) -> bool;
    fn aborting() -> bool;
}

// =============================================================================
// Variable Name Validation
// =============================================================================

/// Check if a variable name is valid (doesn't start with colon or autoload char).
#[inline]
pub fn is_valid_varname_start(c: u8) -> bool {
    c != b':' && c != AUTOLOAD_CHAR
}

/// Check if the rest of a variable name is valid (no colons or autoload chars
/// after the initial scope prefix).
#[inline]
pub fn has_invalid_chars_in_name(name: &[u8]) -> bool {
    name.iter().any(|&c| c == b':' || c == AUTOLOAD_CHAR)
}

// =============================================================================
// Scope Resolution Implementation
// =============================================================================

/// Find the dictionary for a given scope.
///
/// Returns the dictionary handle for the specified scope, or null if the scope
/// is invalid or unavailable.
///
/// # Safety
/// Must be called when the appropriate scope context is available
/// (e.g., inside a function for local/argument scope).
pub unsafe fn get_scope_dict(scope: ScopePrefix) -> DictHandle {
    match scope {
        ScopePrefix::Implicit => {
            // For implicit scope, try local first, then global
            let local = get_funccal_local_dict();
            if local.is_null() {
                get_globvar_dict()
            } else {
                local
            }
        }
        ScopePrefix::Global => get_globvar_dict(),
        ScopePrefix::Vim => get_vimvar_dict(),
        ScopePrefix::Buffer => nvim_curbuf_get_vars(),
        ScopePrefix::Window => nvim_curwin_get_vars(),
        ScopePrefix::Tab => nvim_curtab_get_vars(),
        ScopePrefix::Local => get_funccal_local_dict(),
        ScopePrefix::Argument => get_funccal_args_dict(),
        ScopePrefix::Script => {
            let sid = nvim_get_current_sctx_sid();
            if sid > 0 {
                nvim_get_script_vars_dict(sid)
            } else {
                DictHandle::null()
            }
        }
    }
}

/// Find the hashtable for a variable name.
///
/// This determines which scope to use based on the variable name prefix
/// and returns the appropriate hashtable.
///
/// # Safety
/// - `name` must be a valid pointer to at least `name_len` bytes
pub unsafe fn find_var_ht_impl(
    name: *const c_char,
    name_len: usize,
    varname_out: *mut *const c_char,
) -> HashtabHandle {
    if name_len == 0 {
        return HashtabHandle::null();
    }

    let name_slice = std::slice::from_raw_parts(name.cast::<u8>(), name_len);

    // Check for implicit scope (no "x:" prefix)
    if name_len == 1 || name_slice[1] != b':' {
        // Name must not start with colon or autoload char
        if !is_valid_varname_start(name_slice[0]) {
            return HashtabHandle::null();
        }

        // Set varname to the full name
        if !varname_out.is_null() {
            *varname_out = name;
        }

        // Check compat hashtab first (for variables like "version")
        let compat_ht = nvim_get_compat_hashtab();
        let hi = hash_find_len(compat_ht, name, name_len);
        if nvim_hashitem_empty(hi) == 0 {
            return compat_ht;
        }

        // Try local scope first
        let local_dict = get_funccal_local_dict();
        if !local_dict.is_null() {
            return nvim_dict_get_hashtab(local_dict);
        }

        // Fall back to global scope
        let global_dict = get_globvar_dict();
        return nvim_dict_get_hashtab(global_dict);
    }

    // Explicit scope prefix
    if !varname_out.is_null() {
        *varname_out = name.add(2); // Skip "x:"
    }

    let (scope, _) = ScopePrefix::from_name(name_slice);

    // For non-global explicit scope, check for invalid chars in the rest of the name
    if scope != ScopePrefix::Global && name_len > 2 {
        let rest = &name_slice[2..];
        if has_invalid_chars_in_name(rest) {
            return HashtabHandle::null();
        }
    }

    let dict = get_scope_dict(scope);
    if dict.is_null() {
        HashtabHandle::null()
    } else {
        nvim_dict_get_hashtab(dict)
    }
}

// =============================================================================
// Variable Lookup Implementation
// =============================================================================

/// Get the string value of a variable by name.
///
/// # Safety
/// - `name` must be a valid null-terminated C string
///
/// # Returns
/// Pointer to the string value, or null if the variable doesn't exist.
pub unsafe fn get_var_value_impl(name: *const c_char) -> *const c_char {
    if name.is_null() {
        return std::ptr::null();
    }

    // Calculate name length
    let mut len = 0usize;
    let mut p = name;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    // Find the variable
    let di = find_var(name, len, std::ptr::null_mut(), 0);
    if di.is_null() {
        return std::ptr::null();
    }

    // Get the typval and convert to string
    let tv = nvim_dictitem_get_tv(di);
    tv_get_string(tv)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Find the hashtable for a variable name.
///
/// # Safety
/// - `name` must be valid for `name_len` bytes
/// - `varname` must be valid if not null
#[no_mangle]
pub unsafe extern "C" fn rs_find_var_ht(
    name: *const c_char,
    name_len: usize,
    varname: *mut *const c_char,
) -> HashtabHandle {
    find_var_ht_impl(name, name_len, varname)
}

/// Get the string value of a variable.
///
/// # Safety
/// - `name` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_get_var_value(name: *const c_char) -> *const c_char {
    get_var_value_impl(name)
}

/// Parse scope prefix from a variable name.
///
/// Returns the scope character (g, v, b, w, t, l, s, a) or 0 for implicit scope.
#[no_mangle]
pub extern "C" fn rs_parse_scope_prefix(name: *const c_char, name_len: usize) -> c_int {
    if name.is_null() || name_len == 0 {
        return 0;
    }

    let name_slice = unsafe { std::slice::from_raw_parts(name.cast::<u8>(), name_len) };
    let (scope, _) = ScopePrefix::from_name(name_slice);

    match scope {
        ScopePrefix::Implicit => 0,
        _ => scope as c_int,
    }
}

/// Get the variable name without scope prefix.
///
/// # Safety
/// - `name` must be valid for `name_len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_skip_scope_prefix(
    name: *const c_char,
    name_len: usize,
) -> *const c_char {
    if name.is_null() || name_len == 0 {
        return name;
    }

    let name_slice = std::slice::from_raw_parts(name.cast::<u8>(), name_len);
    let (_, offset) = ScopePrefix::from_name(name_slice);

    name.add(offset)
}

// =============================================================================
// Phase 12: eval_variable and check_vars
// =============================================================================

// OK / FAIL return codes
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Get the value of internal variable "name".
///
/// Matches C `eval_variable`. Returns OK or FAIL.
/// If OK is returned "rettv" must be cleared by the caller.
///
/// # Safety
/// - `name` must be a valid pointer to at least `len` bytes.
/// - `rettv` must be a valid `typval_T*` or null.
/// - `dip` must be a valid `dictitem_T**` or null.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_variable(
    name: *const c_char,
    len: c_int,
    rettv: *mut c_void,
    dip: *mut *mut c_void,
    verbose: bool,
    no_autoload: bool,
) -> c_int {
    let v = find_var(
        name,
        len as usize,
        std::ptr::null_mut(),
        no_autoload as c_int,
    );
    if v.is_null() {
        if !rettv.is_null() && verbose {
            semsg(
                b"E121: Undefined variable: %.*s\0".as_ptr() as *const c_char,
                len,
                name,
            );
        }
        return FAIL;
    }

    if !dip.is_null() {
        *dip = v.as_ptr();
    }

    if !rettv.is_null() {
        let tv = nvim_dictitem_get_tv(v);
        tv_copy(tv, rettv);
    }

    OK
}

/// Check if variable "name[len]" is a local variable or an argument.
/// Sets *eval_lavars_used = true if it is.
///
/// Matches C `check_vars`.
///
/// # Safety
/// - `name` must be valid for `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_check_vars(name: *const c_char, len: usize) {
    let lavars = nvim_vars_get_eval_lavars_used();
    if lavars.is_null() {
        return;
    }

    let mut varname: *const c_char = std::ptr::null();
    let ht = find_var_ht_impl(name, len, std::ptr::addr_of_mut!(varname));
    let local_ht = nvim_vars_get_funccal_local_ht();
    let args_ht = nvim_vars_get_funccal_args_ht();
    if !ht.is_null() && (ht.as_ptr() == local_ht || ht.as_ptr() == args_ht) {
        // Variable is local or arg - check it actually exists
        let di = find_var(name, len, std::ptr::null_mut(), 1);
        if !di.is_null() {
            *lavars = true;
        }
    }
}

// =============================================================================
// Phase 13: find_var_in_ht — replacing the C implementation
// =============================================================================

/// Find a variable in a hashtable, handling scope-dict shortcuts and autoload.
///
/// When `varname_len == 0`, the `htname` character selects the scope-dict item
/// directly (matching the C `switch(htname)` behaviour). Otherwise we do a
/// hashtable lookup with optional g: autoload retry.
///
/// Exported as the `find_var_in_ht` symbol, replacing the C function body.
///
/// # Safety
/// - `ht` must be a valid `hashtab_T*` (or null only when `varname_len == 0`)
/// - `varname` must be valid for `varname_len` bytes
#[unsafe(export_name = "find_var_in_ht")]
pub unsafe extern "C" fn rs_find_var_in_ht(
    ht: HashtabHandle,
    htname: c_int,
    varname: *const c_char,
    varname_len: usize,
    no_autoload: c_int,
) -> DictitemHandle {
    if varname_len == 0 {
        // Empty varname: return the scope-dict item for this scope character.
        // Mirrors the C switch(htname) block.
        let ch = htname as u8;
        return match ch {
            b's' => {
                let sid = nvim_get_current_sctx_sid();
                nvim_vars_script_sv_var(sid)
            }
            b'g' => nvim_vars_globvars_var(),
            b'v' => nvim_vars_vimvars_var(),
            b'b' => nvim_vars_curbuf_bufvar(),
            b'w' => nvim_vars_curwin_winvar(),
            b't' => nvim_vars_curtab_winvar(),
            b'l' => get_funccal_local_var(),
            b'a' => get_funccal_args_var(),
            _ => DictitemHandle::null(),
        };
    }

    // Normal hashtable lookup.
    let mut hi = hash_find_len(ht, varname, varname_len);
    if nvim_hashitem_empty(hi) != 0 {
        // For global variables, attempt to autoload the script once.
        // NOTE: script_autoload() may invalidate `hi`; always re-fetch after.
        let globvarht = get_globvar_ht();
        if ht.as_ptr() == globvarht.as_ptr() && no_autoload == 0 {
            if !script_autoload(varname, varname_len, false) || aborting() {
                return DictitemHandle::null();
            }
            hi = hash_find_len(ht, varname, varname_len);
        }
        if nvim_hashitem_empty(hi) != 0 {
            return DictitemHandle::null();
        }
    }
    nvim_hi2dictitem(hi)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_prefix_parsing() {
        // Explicit scopes
        assert_eq!(ScopePrefix::from_name(b"g:var"), (ScopePrefix::Global, 2));
        assert_eq!(ScopePrefix::from_name(b"v:version"), (ScopePrefix::Vim, 2));
        assert_eq!(ScopePrefix::from_name(b"b:buf"), (ScopePrefix::Buffer, 2));
        assert_eq!(ScopePrefix::from_name(b"w:win"), (ScopePrefix::Window, 2));
        assert_eq!(ScopePrefix::from_name(b"t:tab"), (ScopePrefix::Tab, 2));
        assert_eq!(ScopePrefix::from_name(b"l:local"), (ScopePrefix::Local, 2));
        assert_eq!(
            ScopePrefix::from_name(b"s:script"),
            (ScopePrefix::Script, 2)
        );
        assert_eq!(ScopePrefix::from_name(b"a:arg"), (ScopePrefix::Argument, 2));

        // Implicit scope
        assert_eq!(ScopePrefix::from_name(b"myvar"), (ScopePrefix::Implicit, 0));
        assert_eq!(ScopePrefix::from_name(b"x"), (ScopePrefix::Implicit, 0));
        assert_eq!(ScopePrefix::from_name(b""), (ScopePrefix::Implicit, 0));

        // Invalid prefixes (treated as implicit)
        assert_eq!(ScopePrefix::from_name(b"x:var"), (ScopePrefix::Implicit, 0));
        assert_eq!(ScopePrefix::from_name(b"1:var"), (ScopePrefix::Implicit, 0));
    }

    #[test]
    fn test_scope_prefix_is_explicit() {
        assert!(!ScopePrefix::Implicit.is_explicit());
        assert!(ScopePrefix::Global.is_explicit());
        assert!(ScopePrefix::Vim.is_explicit());
        assert!(ScopePrefix::Local.is_explicit());
    }

    #[test]
    fn test_varname_validation() {
        assert!(is_valid_varname_start(b'a'));
        assert!(is_valid_varname_start(b'_'));
        assert!(!is_valid_varname_start(b':'));
        assert!(!is_valid_varname_start(b'#'));

        assert!(!has_invalid_chars_in_name(b"myvar"));
        assert!(!has_invalid_chars_in_name(b"my_var_123"));
        assert!(has_invalid_chars_in_name(b"my:var"));
        assert!(has_invalid_chars_in_name(b"my#var"));
    }
}
