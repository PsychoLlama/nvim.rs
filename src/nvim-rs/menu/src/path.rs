//! Menu path parsing utilities.
//!
//! This module provides functions for parsing menu paths (e.g., "File.Open"),
//! extracting mnemonics (the character after '&'), and handling accelerator
//! text (the text after TAB).

use std::ffi::{c_char, c_int, CStr};

use crate::handle::VimMenuHandle;

/// TAB character.
const TAB: u8 = b'\t';

/// NUL character.
const NUL: u8 = 0;

/// Compare two menu names, ignoring text after TAB.
///
/// This matches the C function `menu_namecmp`:
/// ```c
/// static bool menu_namecmp(const char *const name, const char *const mname)
/// {
///   int i;
///   for (i = 0; name[i] != NUL && name[i] != TAB; i++) {
///     if (name[i] != mname[i]) {
///       break;
///     }
///   }
///   return (name[i] == NUL || name[i] == TAB)
///          && (mname[i] == NUL || mname[i] == TAB);
/// }
/// ```
fn menu_namecmp(name: &[u8], mname: &[u8]) -> bool {
    let mut i = 0;
    while i < name.len() && name[i] != NUL && name[i] != TAB {
        if i >= mname.len() || name[i] != mname[i] {
            return false;
        }
        i += 1;
    }
    // Both should be at end or TAB
    let name_ended = i >= name.len() || name[i] == NUL || name[i] == TAB;
    let mname_ended = i >= mname.len() || mname[i] == NUL || mname[i] == TAB;
    name_ended && mname_ended
}

/// Check if a menu name matches any of a menu's name variants.
///
/// Compares `name` against the menu's:
/// - English name (en_name)
/// - English display name (en_dname)
/// - Translated name (name)
/// - Translated display name (dname)
///
/// The comparison ignores text after TAB (accelerator text).
///
/// # Safety
/// All pointers from the menu handle must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_name_equal(name: *const c_char, menu: VimMenuHandle) -> bool {
    if name.is_null() || menu.is_null() {
        return false;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_bytes = name_cstr.to_bytes();

    // Check English name and English display name
    let en_name = menu.en_name();
    if !en_name.is_null() {
        let en_name_cstr = unsafe { CStr::from_ptr(en_name) };
        if menu_namecmp(name_bytes, en_name_cstr.to_bytes()) {
            return true;
        }

        let en_dname = menu.en_dname();
        if !en_dname.is_null() {
            let en_dname_cstr = unsafe { CStr::from_ptr(en_dname) };
            if menu_namecmp(name_bytes, en_dname_cstr.to_bytes()) {
                return true;
            }
        }
    }

    // Check translated name and display name
    let menu_name = menu.name();
    if !menu_name.is_null() {
        let menu_name_cstr = unsafe { CStr::from_ptr(menu_name) };
        if menu_namecmp(name_bytes, menu_name_cstr.to_bytes()) {
            return true;
        }
    }

    let dname = menu.dname();
    if !dname.is_null() {
        let dname_cstr = unsafe { CStr::from_ptr(dname) };
        if menu_namecmp(name_bytes, dname_cstr.to_bytes()) {
            return true;
        }
    }

    false
}

/// Result of parsing menu text for mnemonic and accelerator.
#[repr(C)]
pub struct MenuTextResult {
    /// The display text (without mnemonic markers and accelerator).
    /// This is a newly allocated C string that must be freed by the caller.
    pub text: *mut c_char,
    /// The mnemonic character (the character after '&'), or 0 if none.
    pub mnemonic: c_int,
    /// The accelerator text (text after TAB), or NULL if none.
    /// This is a newly allocated C string that must be freed by the caller.
    pub actext: *mut c_char,
}

/// Parse menu text to extract mnemonic and accelerator text.
///
/// Given menu text like "&File\tCtrl+N", this extracts:
/// - Display text: "File"
/// - Mnemonic: 'F' (the character after '&')
/// - Accelerator text: "Ctrl+N" (the text after TAB)
///
/// The mnemonic marker '&' is removed from the display text.
/// The sequence "&&" is reduced to a single '&'.
///
/// # Returns
/// A `MenuTextResult` containing newly allocated strings for text and actext.
/// The caller is responsible for freeing these with `nvim_xfree`.
///
/// # Safety
/// The `str` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_text(str: *const c_char) -> MenuTextResult {
    if str.is_null() {
        return MenuTextResult {
            text: std::ptr::null_mut(),
            mnemonic: 0,
            actext: std::ptr::null_mut(),
        };
    }

    let cstr = unsafe { CStr::from_ptr(str) };
    let bytes = cstr.to_bytes();

    // Find TAB position for accelerator text
    let tab_pos = bytes.iter().position(|&b| b == TAB);

    // Extract accelerator text if present
    let (text_bytes, actext_bytes) = if let Some(pos) = tab_pos {
        (&bytes[..pos], Some(&bytes[pos + 1..]))
    } else {
        (bytes, None)
    };

    // Process mnemonic markers
    let mut result_text = Vec::with_capacity(text_bytes.len());
    let mut mnemonic: c_int = 0;
    let mut i = 0;
    while i < text_bytes.len() {
        if text_bytes[i] == b'&' {
            if i + 1 < text_bytes.len() {
                if text_bytes[i + 1] == b'&' {
                    // "&&" becomes "&"
                    result_text.push(b'&');
                    i += 2;
                } else {
                    // "&X" - X is the mnemonic
                    if mnemonic == 0 {
                        mnemonic = c_int::from(text_bytes[i + 1]);
                    }
                    // Skip the '&', but include the next character
                    result_text.push(text_bytes[i + 1]);
                    i += 2;
                }
            } else {
                // Trailing "&" - just include it
                result_text.push(b'&');
                i += 1;
            }
        } else {
            result_text.push(text_bytes[i]);
            i += 1;
        }
    }

    // Allocate C strings for the results
    let text_ptr = allocate_c_string(&result_text);
    let actext_ptr = if let Some(ac) = actext_bytes {
        allocate_c_string(ac)
    } else {
        std::ptr::null_mut()
    };

    MenuTextResult {
        text: text_ptr,
        mnemonic,
        actext: actext_ptr,
    }
}

/// Allocate a C string from a byte slice using nvim_xmalloc.
///
/// # Safety
/// This calls into C to allocate memory.
unsafe fn allocate_c_string(bytes: &[u8]) -> *mut c_char {
    extern "C" {
        fn nvim_xmalloc(size: usize) -> *mut std::ffi::c_void;
    }

    let ptr = unsafe { nvim_xmalloc(bytes.len() + 1) } as *mut c_char;
    if ptr.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
        *ptr.add(bytes.len()) = 0; // NUL terminator
    }

    ptr
}

/// Calculate the length of a menu name component (up to '.' or end).
///
/// This is useful for parsing menu paths like "File.Open.Recent".
///
/// # Safety
/// The `name` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_namelen(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let cstr = unsafe { CStr::from_ptr(name) };
    let bytes = cstr.to_bytes();

    let mut len = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' {
            break;
        }
        // Handle escape sequences
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            // Escaped character - count both
            len += 1;
            i += 1;
        }
        len += 1;
        i += 1;
    }

    len
}

/// Ctrl_V character value (0x16 = 22).
const CTRL_V: u8 = 22;

extern "C" {
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;
}

/// Skip over this element of the menu path and return the start of the next
/// element. Any `\` and `^V` escapes are removed from the current element.
///
/// The buffer is modified in-place: escape characters are stripped from the
/// current path component, and the `.` separator is replaced with NUL.
///
/// Returns a pointer to the start of the next path component, or to the
/// NUL terminator if there is no next component.
///
/// This is the Rust implementation of C `menu_name_skip()`.
///
/// # Safety
/// `name` must be a valid pointer to a mutable NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_name_skip(name: *mut c_char) -> *mut c_char {
    if name.is_null() {
        return name;
    }

    let mut p = name;

    // Walk through the current component
    while unsafe { *p } != 0 && unsafe { *p } as u8 != b'.' {
        let ch = unsafe { *p } as u8;
        if ch == b'\\' || ch == CTRL_V {
            // Remove the escape character by shifting everything left by 1
            let src = unsafe { p.add(1) };
            let len = unsafe { strlen(src) };
            unsafe {
                std::ptr::copy(src, p, len + 1);
            }
            // If we hit NUL after removing escape, stop
            if unsafe { *p } == 0 {
                break;
            }
        }
        // Advance by multibyte character length
        let char_len = unsafe { utfc_ptr2len(p) };
        p = unsafe { p.add(char_len as usize) };
    }

    if unsafe { *p } != 0 {
        // Replace '.' with NUL and advance past it
        unsafe {
            *p = 0;
        }
        p = unsafe { p.add(1) };
    }

    p
}

/// Find the character just after one part of a menu name.
///
/// Like `menu_name_skip` but does NOT modify the buffer. Stops at NUL,
/// `.`, or whitespace. Handles `\` and Ctrl_V escapes (skipping the next
/// character).
///
/// This is the Rust implementation of C `menu_skip_part()`.
///
/// # Safety
/// `p` must be a valid pointer to a NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_skip_part(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }

    let mut cur = p;
    while unsafe { *cur } != 0
        && unsafe { *cur } as u8 != b'.'
        && !(unsafe { *cur } as u8).is_ascii_whitespace()
    {
        let ch = unsafe { *cur } as u8;
        if (ch == b'\\' || ch == CTRL_V) && unsafe { *cur.add(1) } != 0 {
            cur = unsafe { cur.add(1) };
        }
        cur = unsafe { cur.add(1) };
    }

    cur
}

/// Remove `\` escapes from a menu name up to the first `.` or NUL.
///
/// Modifies the buffer in-place by removing backslash escape characters.
/// Unlike `menu_name_skip`, this uses `MB_PTR_ADV` for multibyte handling.
///
/// This is the Rust implementation of C `menu_unescape_name()`.
///
/// # Safety
/// `name` must be a valid pointer to a mutable NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_unescape_name(name: *mut c_char) {
    if name.is_null() {
        return;
    }

    let mut p = name;
    while unsafe { *p } != 0 && unsafe { *p } as u8 != b'.' {
        if unsafe { *p } as u8 == b'\\' {
            // Remove the backslash by shifting everything left by 1
            let src = unsafe { p.add(1) };
            let len = unsafe { strlen(src) };
            unsafe {
                std::ptr::copy(src, p, len + 1);
            }
        }
        // Advance by multibyte character length
        let char_len = unsafe { utfc_ptr2len(p) };
        p = unsafe { p.add(char_len as usize) };
    }
}

/// Isolate the menu name and translate `<Tab>` text into a real TAB byte.
///
/// Stops at whitespace (the whitespace char is NUL-terminated).
/// Returns a pointer to the text after the whitespace (skipping leading whitespace).
///
/// This is the Rust implementation of C `menu_translate_tab_and_shift()`.
///
/// # Safety
/// `arg_start` must be a valid pointer to a mutable NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_translate_tab_and_shift(arg_start: *mut c_char) -> *mut c_char {
    if arg_start.is_null() {
        return arg_start;
    }

    let mut arg = arg_start;

    while unsafe { *arg } != 0 && !(unsafe { *arg } as u8).is_ascii_whitespace() {
        let ch = unsafe { *arg } as u8;
        if (ch == b'\\' || ch == CTRL_V) && unsafe { *arg.add(1) } != 0 {
            arg = unsafe { arg.add(1) };
        } else if strnicmp_5(arg, b"<TAB>") {
            // Replace '<' with TAB and remove "TAB>"
            unsafe {
                *arg = TAB as i8;
            }
            // STRMOVE(arg + 1, arg + 5)
            let src = unsafe { arg.add(5) };
            let dst = unsafe { arg.add(1) };
            let len = unsafe { strlen(src) };
            unsafe {
                std::ptr::copy(src, dst, len + 1);
            }
        }
        arg = unsafe { arg.add(1) };
    }

    if unsafe { *arg } != 0 {
        unsafe {
            *arg = 0;
        }
        arg = unsafe { arg.add(1) };
    }

    // Skip whitespace
    while unsafe { *arg } != 0 && (unsafe { *arg } as u8).is_ascii_whitespace() {
        arg = unsafe { arg.add(1) };
    }

    arg
}

/// Case-insensitive comparison of 5 bytes against "<TAB>".
///
/// # Safety
/// `p` must point to at least 5 readable bytes.
unsafe fn strnicmp_5(p: *const c_char, pattern: &[u8; 5]) -> bool {
    for (i, &pat_byte) in pattern.iter().enumerate() {
        let ch = unsafe { *p.add(i) } as u8;
        if ch == 0 {
            return false;
        }
        if !ch.eq_ignore_ascii_case(&pat_byte) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_namecmp_equal() {
        assert!(menu_namecmp(b"File", b"File"));
        assert!(menu_namecmp(b"Edit", b"Edit"));
        assert!(menu_namecmp(b"", b""));
    }

    #[test]
    fn test_menu_namecmp_with_tab() {
        // Names are equal up to TAB
        assert!(menu_namecmp(b"File\tCtrl+N", b"File"));
        assert!(menu_namecmp(b"File", b"File\tCtrl+N"));
        assert!(menu_namecmp(b"File\tCtrl+N", b"File\tCtrl+O"));
    }

    #[test]
    fn test_menu_namecmp_not_equal() {
        assert!(!menu_namecmp(b"File", b"Edit"));
        assert!(!menu_namecmp(b"File", b"Files"));
        assert!(!menu_namecmp(b"Files", b"File"));
    }

    #[test]
    fn test_menu_namecmp_with_nul() {
        // NUL terminates comparison
        assert!(menu_namecmp(b"File\0extra", b"File"));
        assert!(menu_namecmp(b"File", b"File\0extra"));
    }
}
