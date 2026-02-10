//! Command modifier types and utilities for Ex commands.
//!
//! This module defines the types and constants used for command modifiers
//! like `:silent`, `:vertical`, `:noautocmd`, etc.

use std::ffi::{c_char, c_int};

// =============================================================================
// Command modifier table
// =============================================================================

/// Entry in the command modifier table.
struct CmdMod {
    name: &'static [u8],
    minlen: i32,
    has_count: bool,
}

/// The command modifier table, matching `cmdmods[]` in ex_docmd.c.
/// Values verified against ex_docmd.c lines 3214-3239.
const CMDMODS: &[CmdMod] = &[
    CmdMod {
        name: b"aboveleft",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"belowright",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"botright",
        minlen: 2,
        has_count: false,
    },
    CmdMod {
        name: b"browse",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"confirm",
        minlen: 4,
        has_count: false,
    },
    CmdMod {
        name: b"filter",
        minlen: 4,
        has_count: false,
    },
    CmdMod {
        name: b"hide",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"horizontal",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"keepalt",
        minlen: 5,
        has_count: false,
    },
    CmdMod {
        name: b"keepjumps",
        minlen: 5,
        has_count: false,
    },
    CmdMod {
        name: b"keepmarks",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"keeppatterns",
        minlen: 5,
        has_count: false,
    },
    CmdMod {
        name: b"leftabove",
        minlen: 5,
        has_count: false,
    },
    CmdMod {
        name: b"lockmarks",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"noautocmd",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"noswapfile",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"rightbelow",
        minlen: 6,
        has_count: false,
    },
    CmdMod {
        name: b"sandbox",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"silent",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"tab",
        minlen: 3,
        has_count: true,
    },
    CmdMod {
        name: b"topleft",
        minlen: 2,
        has_count: false,
    },
    CmdMod {
        name: b"unsilent",
        minlen: 3,
        has_count: false,
    },
    CmdMod {
        name: b"verbose",
        minlen: 4,
        has_count: true,
    },
    CmdMod {
        name: b"vertical",
        minlen: 4,
        has_count: false,
    },
];

// =============================================================================
// Command modifier flags (CMOD_*)
// =============================================================================

/// `:sandbox` - execute in sandbox mode
pub const CMOD_SANDBOX: c_int = 0x0001;
/// `:silent` - suppress messages
pub const CMOD_SILENT: c_int = 0x0002;
/// `:silent!` - suppress error messages too
pub const CMOD_ERRSILENT: c_int = 0x0004;
/// `:unsilent` - cancel silence
pub const CMOD_UNSILENT: c_int = 0x0008;
/// `:noautocmd` - disable autocommands
pub const CMOD_NOAUTOCMD: c_int = 0x0010;
/// `:hide` - hide buffer when leaving
pub const CMOD_HIDE: c_int = 0x0020;
/// `:browse` - invoke file dialog
pub const CMOD_BROWSE: c_int = 0x0040;
/// `:confirm` - invoke yes/no dialog
pub const CMOD_CONFIRM: c_int = 0x0080;
/// `:keepalt` - keep alternate file
pub const CMOD_KEEPALT: c_int = 0x0100;
/// `:keepmarks` - keep marks
pub const CMOD_KEEPMARKS: c_int = 0x0200;
/// `:keepjumps` - keep jump list
pub const CMOD_KEEPJUMPS: c_int = 0x0400;
/// `:lockmarks` - lock marks
pub const CMOD_LOCKMARKS: c_int = 0x0800;
/// `:keeppatterns` - keep search patterns
pub const CMOD_KEEPPATTERNS: c_int = 0x1000;
/// `:noswapfile` - don't create swap file
pub const CMOD_NOSWAPFILE: c_int = 0x2000;

// =============================================================================
// Window split flags (WSP_*)
// =============================================================================

/// Split horizontally
pub const WSP_HOR: c_int = 0x01;
/// Split vertically
pub const WSP_VERT: c_int = 0x02;
/// Split at top
pub const WSP_TOP: c_int = 0x04;
/// Split at bottom
pub const WSP_BOT: c_int = 0x08;
/// Split above current window
pub const WSP_ABOVE: c_int = 0x10;
/// Split below current window
pub const WSP_BELOW: c_int = 0x20;

// =============================================================================
// Flag checking utilities
// =============================================================================

/// Check if the CMOD_SANDBOX flag is set.
#[inline]
pub const fn has_sandbox(flags: c_int) -> bool {
    (flags & CMOD_SANDBOX) != 0
}

/// Check if the CMOD_SILENT flag is set.
#[inline]
pub const fn has_silent(flags: c_int) -> bool {
    (flags & CMOD_SILENT) != 0
}

/// Check if the CMOD_ERRSILENT flag is set.
#[inline]
pub const fn has_errsilent(flags: c_int) -> bool {
    (flags & CMOD_ERRSILENT) != 0
}

/// Check if the CMOD_NOAUTOCMD flag is set.
#[inline]
pub const fn has_noautocmd(flags: c_int) -> bool {
    (flags & CMOD_NOAUTOCMD) != 0
}

/// FFI wrapper to check if CMOD_SANDBOX flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_sandbox(flags: c_int) -> c_int {
    c_int::from(has_sandbox(flags))
}

/// FFI wrapper to check if CMOD_SILENT flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_silent(flags: c_int) -> c_int {
    c_int::from(has_silent(flags))
}

/// FFI wrapper to check if CMOD_ERRSILENT flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_errsilent(flags: c_int) -> c_int {
    c_int::from(has_errsilent(flags))
}

/// FFI wrapper to check if CMOD_NOAUTOCMD flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_noautocmd(flags: c_int) -> c_int {
    c_int::from(has_noautocmd(flags))
}

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn rs_skipdigits(q: *const c_char) -> *const c_char;
}

// =============================================================================
// Modifier length
// =============================================================================

/// Get the length of a command modifier (including optional count prefix).
///
/// Returns 0 if the string does not start with a modifier.
///
/// Matches the C `modifier_len()` function.
///
/// # Safety
///
/// `cmd` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_modifier_len(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        return 0;
    }

    let mut p = cmd;

    // Skip optional count prefix (digits then whitespace)
    if (*p as u8).is_ascii_digit() {
        p = skipwhite(rs_skipdigits(cmd.add(1)) as *const c_char) as *const c_char;
    }

    for entry in CMDMODS {
        let mut j = 0i32;
        loop {
            let c = *p.add(j as usize) as u8;
            if c == 0 {
                break;
            }
            if j as usize >= entry.name.len() || c != entry.name[j as usize] {
                break;
            }
            j += 1;
        }
        if j >= entry.minlen
            && !(*p.add(j as usize) as u8).is_ascii_alphabetic()
            && (p == cmd || entry.has_count)
        {
            return j + (p as usize - cmd as usize) as c_int;
        }
    }
    0
}

/// Check if a name matches a command modifier.
///
/// Returns:
/// - 0 if name doesn't match any modifier
/// - 1 if name is a prefix match (abbreviation) of a modifier
/// - 2 if name is an exact match of a modifier
///
/// Used by `cmd_exists()`.
pub unsafe fn check_modifier(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    for entry in CMDMODS {
        let mut j = 0usize;
        loop {
            let c = *name.add(j) as u8;
            if c == 0 {
                break;
            }
            if j >= entry.name.len() || c != entry.name[j] {
                break;
            }
            j += 1;
        }
        if *name.add(j) as u8 == 0 && j >= entry.minlen as usize {
            return if j == entry.name.len() { 2 } else { 1 };
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmod_flag_checks() {
        // Test individual flags
        assert!(has_sandbox(CMOD_SANDBOX));
        assert!(!has_sandbox(CMOD_SILENT));

        assert!(has_silent(CMOD_SILENT));
        assert!(!has_silent(CMOD_SANDBOX));

        assert!(has_errsilent(CMOD_ERRSILENT));
        assert!(!has_errsilent(CMOD_SILENT));

        assert!(has_noautocmd(CMOD_NOAUTOCMD));
        assert!(!has_noautocmd(CMOD_SILENT));

        // Test combined flags
        let combined = CMOD_SANDBOX | CMOD_SILENT | CMOD_NOAUTOCMD;
        assert!(has_sandbox(combined));
        assert!(has_silent(combined));
        assert!(!has_errsilent(combined));
        assert!(has_noautocmd(combined));
    }

    #[test]
    fn test_cmod_ffi_wrappers() {
        assert_eq!(rs_cmod_has_sandbox(CMOD_SANDBOX), 1);
        assert_eq!(rs_cmod_has_sandbox(CMOD_SILENT), 0);

        assert_eq!(rs_cmod_has_silent(CMOD_SILENT), 1);
        assert_eq!(rs_cmod_has_silent(CMOD_SANDBOX), 0);
    }

    #[test]
    fn test_wsp_flags() {
        // Verify flag values match expected constants
        assert_eq!(WSP_HOR, 0x01);
        assert_eq!(WSP_VERT, 0x02);
        assert_eq!(WSP_TOP, 0x04);
        assert_eq!(WSP_BOT, 0x08);
        assert_eq!(WSP_ABOVE, 0x10);
        assert_eq!(WSP_BELOW, 0x20);
    }

    #[test]
    fn test_cmdmods_table() {
        // Verify the table has 24 entries matching C
        assert_eq!(CMDMODS.len(), 24);

        // Verify first and last entries
        assert_eq!(CMDMODS[0].name, b"aboveleft");
        assert_eq!(CMDMODS[0].minlen, 3);
        assert!(!CMDMODS[0].has_count);

        assert_eq!(CMDMODS[23].name, b"vertical");
        assert_eq!(CMDMODS[23].minlen, 4);
        assert!(!CMDMODS[23].has_count);

        // Verify the two has_count entries
        let tab_entry = CMDMODS.iter().find(|m| m.name == b"tab").unwrap();
        assert!(tab_entry.has_count);
        assert_eq!(tab_entry.minlen, 3);

        let verbose_entry = CMDMODS.iter().find(|m| m.name == b"verbose").unwrap();
        assert!(verbose_entry.has_count);
        assert_eq!(verbose_entry.minlen, 4);

        // Verify sorted order (matches C)
        for i in 1..CMDMODS.len() {
            assert!(
                CMDMODS[i - 1].name < CMDMODS[i].name,
                "cmdmods table not sorted at index {}",
                i
            );
        }
    }

    #[test]
    fn test_check_modifier() {
        use std::ffi::CString;
        unsafe {
            // Exact match
            let name = CString::new("silent").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 2);

            // Abbreviation match
            let name = CString::new("sil").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 1);

            // Too short to match (minlen=3)
            let name = CString::new("si").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 0);

            // Not a modifier
            let name = CString::new("foobar").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 0);

            // "tab" exact match
            let name = CString::new("tab").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 2);

            // "verb" abbreviation for verbose
            let name = CString::new("verb").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 1);

            // "vertical" exact match
            let name = CString::new("vertical").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 2);

            // "vert" - this matches "vertical" (minlen=4)
            let name = CString::new("vert").unwrap();
            assert_eq!(check_modifier(name.as_ptr()), 1);
        }
    }
}
