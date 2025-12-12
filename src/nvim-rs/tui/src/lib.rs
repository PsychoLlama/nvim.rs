//! Terminal UI utilities for Neovim
//!
//! This crate provides terminfo-related functions for the terminal UI.

use std::ffi::{c_char, c_int, CStr};

// ============================================================================
// Terminfo Functions
// ============================================================================

/// Checks if `term` is a member of the given `family`.
///
/// A terminal is considered a member of a family if:
/// - `term` starts with `family`
/// - Either `term` equals `family` exactly, or the character following `family`
///   in `term` is '-' or '.'
///
/// For example, "xterm-256color" is in the "xterm" family.
/// "screen.xterm" is in the "screen" family.
///
/// # Safety
///
/// Both `term` and `family` must be valid, NUL-terminated C strings.
///
/// # Arguments
/// * `term` - The terminal name to check (may be NULL)
/// * `family` - The family name to check against
///
/// # Returns
/// 1 if `term` is in the `family`, 0 otherwise
#[no_mangle]
pub unsafe extern "C" fn rs_terminfo_is_term_family(
    term: *const c_char,
    family: *const c_char,
) -> c_int {
    if term.is_null() {
        return 0;
    }

    // Safety: caller guarantees these are valid C strings
    let term_cstr = unsafe { CStr::from_ptr(term) };
    let family_cstr = unsafe { CStr::from_ptr(family) };

    let term_bytes = term_cstr.to_bytes();
    let family_bytes = family_cstr.to_bytes();

    let tlen = term_bytes.len();
    let flen = family_bytes.len();

    if tlen < flen {
        return 0;
    }

    // Check if term starts with family
    if &term_bytes[..flen] != family_bytes {
        return 0;
    }

    // Check the separator condition:
    // Either term equals family exactly, or the next char is '-' or '.'
    if tlen == flen {
        return 1;
    }

    let next_char = term_bytes[flen];
    c_int::from(next_char == b'-' || next_char == b'.')
}

/// Checks if the terminal is a BSD console.
///
/// This function detects BSD console terminals:
/// - On OpenBSD: "vt220"
/// - On NetBSD: "vt100"
/// - On FreeBSD: "xterm" when XTERM_VERSION env var exists (degraded xterm)
///
/// # Safety
///
/// `term` must be a valid, NUL-terminated C string, or NULL.
///
/// # Arguments
/// * `term` - The terminal name to check (may be NULL)
///
/// # Returns
/// 1 if the terminal is a BSD console, 0 otherwise
#[no_mangle]
pub unsafe extern "C" fn rs_terminfo_is_bsd_console(term: *const c_char) -> c_int {
    // This is only relevant on BSD systems
    #[cfg(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    {
        if term.is_null() {
            return 0;
        }

        let term_cstr = unsafe { CStr::from_ptr(term) };
        let term_bytes = term_cstr.to_bytes();

        // OpenBSD
        if term_bytes == b"vt220" {
            return 1;
        }

        // NetBSD
        if term_bytes == b"vt100" {
            return 1;
        }

        // FreeBSD specific check
        #[cfg(target_os = "freebsd")]
        {
            if term_bytes == b"xterm" {
                // Check if XTERM_VERSION env var exists
                // FreeBSD console sets TERM=xterm but doesn't support xterm features
                extern "C" {
                    fn os_env_exists(name: *const c_char, use_strequal: c_int) -> c_int;
                }
                let name = c"XTERM_VERSION";
                if unsafe { os_env_exists(name.as_ptr(), 1) } != 0 {
                    return 1;
                }
            }
        }

        0
    }

    #[cfg(not(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    )))]
    {
        let _ = term; // Suppress unused warning
        0
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn is_term_family(term: &str, family: &str) -> bool {
        let term_c = CString::new(term).unwrap();
        let family_c = CString::new(family).unwrap();
        unsafe { rs_terminfo_is_term_family(term_c.as_ptr(), family_c.as_ptr()) != 0 }
    }

    fn is_term_family_null(family: &str) -> bool {
        let family_c = CString::new(family).unwrap();
        unsafe { rs_terminfo_is_term_family(std::ptr::null(), family_c.as_ptr()) != 0 }
    }

    #[test]
    fn test_terminfo_is_term_family_exact_match() {
        assert!(is_term_family("xterm", "xterm"));
        assert!(is_term_family("screen", "screen"));
        assert!(is_term_family("tmux", "tmux"));
    }

    #[test]
    fn test_terminfo_is_term_family_with_dash() {
        assert!(is_term_family("xterm-256color", "xterm"));
        assert!(is_term_family("screen-256color", "screen"));
        assert!(is_term_family("tmux-256color", "tmux"));
        assert!(is_term_family("rxvt-unicode", "rxvt"));
    }

    #[test]
    fn test_terminfo_is_term_family_with_dot() {
        // screen.xterm is in the screen family
        assert!(is_term_family("screen.xterm", "screen"));
        assert!(is_term_family("iterm.app", "iterm"));
    }

    #[test]
    fn test_terminfo_is_term_family_no_match() {
        // Not a match - different family
        assert!(!is_term_family("xterm", "screen"));
        assert!(!is_term_family("rxvt", "xterm"));

        // Not a match - prefix but no separator
        assert!(!is_term_family("xterminator", "xterm"));
        assert!(!is_term_family("screenx", "screen"));
    }

    #[test]
    fn test_terminfo_is_term_family_null_term() {
        assert!(!is_term_family_null("xterm"));
    }

    #[test]
    fn test_terminfo_is_term_family_shorter_term() {
        // term is shorter than family
        assert!(!is_term_family("xt", "xterm"));
        assert!(!is_term_family("x", "xterm"));
    }

    #[test]
    fn test_terminfo_is_bsd_console_non_bsd() {
        // On non-BSD systems, should always return 0
        #[cfg(not(any(
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "dragonfly"
        )))]
        {
            let term = CString::new("vt100").unwrap();
            assert_eq!(
                unsafe { rs_terminfo_is_bsd_console(term.as_ptr()) },
                0
            );
            let term = CString::new("vt220").unwrap();
            assert_eq!(
                unsafe { rs_terminfo_is_bsd_console(term.as_ptr()) },
                0
            );
        }
    }

    #[test]
    fn test_terminfo_is_bsd_console_null() {
        assert_eq!(unsafe { rs_terminfo_is_bsd_console(std::ptr::null()) }, 0);
    }
}
