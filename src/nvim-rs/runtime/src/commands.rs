//! Ex command handlers for runtime operations
//!
//! This module handles :source, :runtime, :packadd, and related commands.

use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::Ordering;

use crate::dip;
use crate::expand::RUNTIME_EXPAND_FLAGS;
use crate::pathsearch::rs_source_runtime;
use nvim_ex_eval::ExargT;

// =============================================================================
// Command Types
// =============================================================================

/// Runtime command types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCmd {
    /// :runtime command
    Runtime = 0,
    /// :source command
    Source = 1,
    /// :packadd command
    Packadd = 2,
    /// :packloadall command
    Packloadall = 3,
    /// :scriptnames command
    Scriptnames = 4,
}

impl RuntimeCmd {
    /// Convert from integer
    pub const fn from_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Runtime),
            1 => Some(Self::Source),
            2 => Some(Self::Packadd),
            3 => Some(Self::Packloadall),
            4 => Some(Self::Scriptnames),
            _ => None,
        }
    }
}

// =============================================================================
// :runtime Command
// =============================================================================

/// Parse :runtime command arguments.
///
/// Syntax: :runtime[!] [where] {file}
///
/// Returns DIP flags based on arguments.
#[allow(clippy::fn_params_excessive_bools)]
pub fn rs_runtime_flags(bang: bool, start: bool, opt: bool, after: bool) -> c_int {
    let mut flags = 0;

    // Bang means find all matches
    if bang {
        flags |= dip::ALL;
    }

    // where argument
    if start {
        flags |= dip::START;
    }
    if opt {
        flags |= dip::OPT;
    }
    if after {
        flags |= dip::AFTER;
    }

    flags
}

/// Check if :runtime should search all matches (bang used).
pub fn rs_runtime_find_all(flags: c_int) -> bool {
    (flags & dip::ALL) != 0
}

// =============================================================================
// :source Command
// =============================================================================

/// Parse :source command modifiers.
///
/// Returns true if this is a :source! (re-source) command.
pub fn rs_source_is_reload(bang: bool) -> bool {
    bang
}

// =============================================================================
// :packadd Command
// =============================================================================

/// Parse :packadd command.
///
/// Returns DIP flags for searching.
pub fn rs_packadd_flags(bang: bool) -> c_int {
    let mut flags = dip::OPT | dip::ALL | dip::DIRFILE;

    // Without bang, also search start directories
    if !bang {
        flags |= dip::START;
    }

    flags
}

/// Check if :packadd should only search opt directories (bang used).
pub fn rs_packadd_opt_only(bang: bool) -> bool {
    bang
}

// =============================================================================
// :packloadall Command
// =============================================================================

/// Check if packloadall should force reload (bang used).
pub fn rs_packloadall_force(bang: bool) -> bool {
    bang
}

// =============================================================================
// Argument Keyword Matching
// =============================================================================

/// :runtime "where" argument values
pub const WHERE_START: &[u8] = b"START\0";
pub const WHERE_OPT: &[u8] = b"OPT\0";
pub const WHERE_PACK: &[u8] = b"PACK\0";
pub const WHERE_ALL: &[u8] = b"ALL\0";

/// Get :runtime START keyword.
pub fn rs_where_start() -> *const c_char {
    WHERE_START.as_ptr().cast()
}

/// Get :runtime OPT keyword.
pub fn rs_where_opt() -> *const c_char {
    WHERE_OPT.as_ptr().cast()
}

/// Get :runtime PACK keyword.
pub fn rs_where_pack() -> *const c_char {
    WHERE_PACK.as_ptr().cast()
}

/// Get :runtime ALL keyword.
pub fn rs_where_all() -> *const c_char {
    WHERE_ALL.as_ptr().cast()
}

// =============================================================================
// FFI: Runtime Command Functions (Phase 8)
// =============================================================================

/// EXPAND_RUNTIME value from cmdexpand_defs.h
const EXPAND_RUNTIME: c_int = 51;

extern "C" {
    #[link_name = "skipwhite"]
    fn rs_skipwhite(s: *const c_char) -> *mut c_char;
    #[link_name = "skiptowhite"]
    fn rs_skiptowhite(s: *const c_char) -> *mut c_char;
    #[link_name = "skiptowhite_esc"]
    fn rs_skiptowhite_esc(s: *const c_char) -> *mut c_char;

    // (exarg_T fields accessed directly via nvim_ex_eval::ExargT)

    // expand_T accessor (in runtime_ffi.c)
    fn nvim_rt_cmd_expand_set_context(xp: *mut c_void, context: c_int, pattern: *const c_char);

    // (rs_source_runtime is in pathsearch.rs — call directly below)
}

/// Get DIP_ flags from the [where] argument of a :runtime command.
/// `*argp` is advanced to after the [where] argument.
///
/// # Safety
/// `argp` must point to a valid `*mut c_char` pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_runtime_cmd_flags(
    argp: *mut *mut c_char,
    where_len: usize,
) -> c_int {
    let arg = *argp;

    if where_len == 0 {
        return 0;
    }

    if libc::strncmp(arg, c"START".as_ptr(), where_len) == 0 {
        *argp = rs_skipwhite(arg.add(where_len));
        return dip::START + dip::NORTP;
    }
    if libc::strncmp(arg, c"OPT".as_ptr(), where_len) == 0 {
        *argp = rs_skipwhite(arg.add(where_len));
        return dip::OPT + dip::NORTP;
    }
    if libc::strncmp(arg, c"PACK".as_ptr(), where_len) == 0 {
        *argp = rs_skipwhite(arg.add(where_len));
        return dip::START + dip::OPT + dip::NORTP;
    }
    if libc::strncmp(arg, c"ALL".as_ptr(), where_len) == 0 {
        *argp = rs_skipwhite(arg.add(where_len));
        return dip::START + dip::OPT;
    }

    0
}

/// ":runtime [where] {name}"
///
/// # Safety
/// `eap` must be a valid pointer to an exarg_T.
#[export_name = "ex_runtime"]
pub unsafe extern "C" fn rs_ex_runtime(eap: *mut c_void) {
    let eap_ref = &*eap.cast::<ExargT>();
    let mut arg = eap_ref.arg;
    let forceit = eap_ref.forceit != 0;
    let mut flags: c_int = if forceit { dip::ALL } else { 0 };
    let p = rs_skiptowhite(arg);
    let where_len = p.offset_from(arg) as usize;
    flags += rs_get_runtime_cmd_flags(&raw mut arg, where_len);
    debug_assert!(!arg.is_null());
    rs_source_runtime(arg, flags);
}

/// Set the completion context for the :runtime command.
///
/// # Safety
/// `xp` must be a valid pointer to an expand_T.
/// `arg` must be a valid pointer to a NUL-terminated string.
#[export_name = "set_context_in_runtime_cmd"]
pub unsafe extern "C" fn rs_set_context_in_runtime_cmd(xp: *mut c_void, arg: *const c_char) {
    let mut arg = arg.cast_mut();
    let p = rs_skiptowhite(arg);
    let expand_flags = if *p != 0 {
        let where_len = p.offset_from(arg) as usize;
        rs_get_runtime_cmd_flags(&raw mut arg, where_len)
    } else {
        0
    };
    RUNTIME_EXPAND_FLAGS.store(expand_flags, Ordering::Relaxed);

    // Skip to the last argument.
    let mut p = rs_skiptowhite_esc(arg);
    while *p != 0 {
        if RUNTIME_EXPAND_FLAGS.load(Ordering::Relaxed) == 0 {
            // When there are multiple arguments and [where] is not specified,
            // use an unrelated non-zero flag to avoid expanding [where].
            RUNTIME_EXPAND_FLAGS.store(dip::ALL, Ordering::Relaxed);
        }
        arg = rs_skipwhite(p);
        p = rs_skiptowhite_esc(arg);
    }
    nvim_rt_cmd_expand_set_context(xp, EXPAND_RUNTIME, arg);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_cmd() {
        assert_eq!(RuntimeCmd::from_int(0), Some(RuntimeCmd::Runtime));
        assert_eq!(RuntimeCmd::from_int(1), Some(RuntimeCmd::Source));
        assert_eq!(RuntimeCmd::from_int(5), None);
    }

    #[test]
    fn test_runtime_flags() {
        let flags = rs_runtime_flags(true, false, false, false);
        assert!(rs_runtime_find_all(flags));

        let flags = rs_runtime_flags(false, true, false, false);
        assert!(!rs_runtime_find_all(flags));
        assert!((flags & dip::START) != 0);

        let flags = rs_runtime_flags(true, true, true, false);
        assert!(rs_runtime_find_all(flags));
        assert!((flags & dip::START) != 0);
        assert!((flags & dip::OPT) != 0);
    }

    #[test]
    fn test_packadd_flags() {
        let flags = rs_packadd_flags(false);
        assert!((flags & dip::START) != 0);
        assert!((flags & dip::OPT) != 0);

        let flags = rs_packadd_flags(true);
        assert!((flags & dip::START) == 0);
        assert!((flags & dip::OPT) != 0);
    }

    #[test]
    fn test_source_reload() {
        assert!(!rs_source_is_reload(false));
        assert!(rs_source_is_reload(true));
    }

    #[test]
    fn test_packloadall_force() {
        assert!(!rs_packloadall_force(false));
        assert!(rs_packloadall_force(true));
    }

    #[test]
    fn test_where_keywords() {
        assert!(!rs_where_start().is_null());
        assert!(!rs_where_opt().is_null());
        assert!(!rs_where_pack().is_null());
        assert!(!rs_where_all().is_null());
    }

    #[test]
    fn test_expand_runtime_constant() {
        assert_eq!(EXPAND_RUNTIME, 51);
    }
}
