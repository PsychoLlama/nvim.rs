//! Callback generator functions for command-line completion.
//!
//! These functions match the `CompleteListItemGetter` signature:
//! `char *(*)(expand_T *, int)` — called by `ExpandGeneric` to iterate
//! over completion candidates by index.

use libc::{c_char, c_int};

use crate::ExpandHandle;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    fn nvim_cmdexpand_get_filetype_expand_what() -> c_int;
    fn nvim_cmdexpand_get_breakpt_expand_what() -> c_int;

    /// Check `SCRIPT_ID_VALID(idx+1)` for scriptnames expansion.
    fn nvim_cmdexpand_script_id_valid(idx: c_int) -> c_int;

    /// Get `home_replace()`-processed script name into `NameBuff` and return pointer.
    fn nvim_cmdexpand_get_script_name(idx: c_int) -> *mut c_char;
}

// =============================================================================
// Constants
// =============================================================================

/// Filetype expand-what values.
const EXP_FILETYPECMD_ALL: c_int = 0;
const EXP_FILETYPECMD_PLUGIN: c_int = 1;
const EXP_FILETYPECMD_INDENT: c_int = 2;
const EXP_FILETYPECMD_ONOFF: c_int = 3;

/// Breakpoint expand-what values.
const EXP_BREAKPT_ADD: c_int = 0;
const EXP_BREAKPT_DEL: c_int = 1;

// Static string constants (null-terminated for C compatibility)
static OPTS_INDENT: &[u8] = b"indent\0";
static OPTS_PLUGIN: &[u8] = b"plugin\0";
static OPTS_ON: &[u8] = b"on\0";
static OPTS_OFF: &[u8] = b"off\0";
static OPTS_EXPR: &[u8] = b"expr\0";
static OPTS_FILE: &[u8] = b"file\0";
static OPTS_FUNC: &[u8] = b"func\0";
static OPTS_HERE: &[u8] = b"here\0";
static OPTS_RETAB: &[u8] = b"-indentonly\0";
static OPTS_CLEAR: &[u8] = b"clear\0";
static OPTS_BUFFER: &[u8] = b"<buffer>\0";

/// Helper to return a static C string pointer.
const fn cstr(s: &[u8]) -> *mut c_char {
    s.as_ptr().cast_mut().cast::<c_char>()
}

// =============================================================================
// `get_filetypecmd_arg`
// =============================================================================

/// Return the possible arguments for the `:filetype` command by index.
///
/// # Safety
///
/// `_xp` is unused. Called from C as a `CompleteListItemGetter` callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_filetypecmd_arg(_xp: ExpandHandle, idx: c_int) -> *mut c_char {
    if idx < 0 {
        return std::ptr::null_mut();
    }

    let what = nvim_cmdexpand_get_filetype_expand_what();

    // All options: indent, plugin, on, off
    if what == EXP_FILETYPECMD_ALL && idx < 4 {
        return match idx {
            0 => cstr(OPTS_INDENT),
            1 => cstr(OPTS_PLUGIN),
            2 => cstr(OPTS_ON),
            3 => cstr(OPTS_OFF),
            _ => std::ptr::null_mut(),
        };
    }
    // Plugin: plugin, on, off
    if what == EXP_FILETYPECMD_PLUGIN && idx < 3 {
        return match idx {
            0 => cstr(OPTS_PLUGIN),
            1 => cstr(OPTS_ON),
            2 => cstr(OPTS_OFF),
            _ => std::ptr::null_mut(),
        };
    }
    // Indent: indent, on, off
    if what == EXP_FILETYPECMD_INDENT && idx < 3 {
        return match idx {
            0 => cstr(OPTS_INDENT),
            1 => cstr(OPTS_ON),
            2 => cstr(OPTS_OFF),
            _ => std::ptr::null_mut(),
        };
    }
    // On/Off
    if what == EXP_FILETYPECMD_ONOFF && idx < 2 {
        return match idx {
            0 => cstr(OPTS_ON),
            1 => cstr(OPTS_OFF),
            _ => std::ptr::null_mut(),
        };
    }

    std::ptr::null_mut()
}

// =============================================================================
// `get_breakadd_arg`
// =============================================================================

/// Return the possible arguments for the `:breakadd`/`:breakdel`/`:profdel`
/// commands by index.
///
/// # Safety
///
/// `_xp` is unused. Called from C as a `CompleteListItemGetter` callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_breakadd_arg(_xp: ExpandHandle, idx: c_int) -> *mut c_char {
    // opts = { "expr", "file", "func", "here" }
    if !(0..=3).contains(&idx) {
        return std::ptr::null_mut();
    }

    let what = nvim_cmdexpand_get_breakpt_expand_what();

    if what == EXP_BREAKPT_ADD {
        // breakadd: expr, file, func, here
        return match idx {
            0 => cstr(OPTS_EXPR),
            1 => cstr(OPTS_FILE),
            2 => cstr(OPTS_FUNC),
            3 => cstr(OPTS_HERE),
            _ => std::ptr::null_mut(),
        };
    }
    if what == EXP_BREAKPT_DEL {
        // breakdel: file, func, here (opts[1..=3])
        if idx <= 2 {
            return match idx {
                0 => cstr(OPTS_FILE),
                1 => cstr(OPTS_FUNC),
                2 => cstr(OPTS_HERE),
                _ => std::ptr::null_mut(),
            };
        }
    } else {
        // profdel: file, func (opts[1..=2])
        if idx <= 1 {
            return match idx {
                0 => cstr(OPTS_FILE),
                1 => cstr(OPTS_FUNC),
                _ => std::ptr::null_mut(),
            };
        }
    }

    std::ptr::null_mut()
}

// =============================================================================
// `get_retab_arg`
// =============================================================================

/// Return the possible arguments for the `:retab` command by index.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_retab_arg(_xp: ExpandHandle, idx: c_int) -> *mut c_char {
    if idx == 0 {
        cstr(OPTS_RETAB)
    } else {
        std::ptr::null_mut()
    }
}

// =============================================================================
// `get_messages_arg`
// =============================================================================

/// Return the possible arguments for the `:messages` command by index.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_messages_arg(_xp: ExpandHandle, idx: c_int) -> *mut c_char {
    if idx == 0 {
        cstr(OPTS_CLEAR)
    } else {
        std::ptr::null_mut()
    }
}

// =============================================================================
// `get_mapclear_arg`
// =============================================================================

/// Return the possible arguments for the `:mapclear` command by index.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_mapclear_arg(_xp: ExpandHandle, idx: c_int) -> *mut c_char {
    if idx == 0 {
        cstr(OPTS_BUFFER)
    } else {
        std::ptr::null_mut()
    }
}

// =============================================================================
// `get_scriptnames_arg`
// =============================================================================

/// Return the possible arguments for the `:scriptnames` command by index.
///
/// Returns the home-replaced script name or NULL when there are no more entries.
///
/// # Safety
///
/// `_xp` is unused. Called from C as a `CompleteListItemGetter` callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_scriptnames_arg(_xp: ExpandHandle, idx: c_int) -> *mut c_char {
    if nvim_cmdexpand_script_id_valid(idx) == 0 {
        return std::ptr::null_mut();
    }
    nvim_cmdexpand_get_script_name(idx)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retab_arg() {
        let p = rs_get_retab_arg(std::ptr::null_mut(), 0);
        assert!(!p.is_null());
        let p1 = rs_get_retab_arg(std::ptr::null_mut(), 1);
        assert!(p1.is_null());
    }

    #[test]
    fn test_messages_arg() {
        let p = rs_get_messages_arg(std::ptr::null_mut(), 0);
        assert!(!p.is_null());
        let p1 = rs_get_messages_arg(std::ptr::null_mut(), 1);
        assert!(p1.is_null());
    }

    #[test]
    fn test_mapclear_arg() {
        let p = rs_get_mapclear_arg(std::ptr::null_mut(), 0);
        assert!(!p.is_null());
        let p1 = rs_get_mapclear_arg(std::ptr::null_mut(), 1);
        assert!(p1.is_null());
    }
}
