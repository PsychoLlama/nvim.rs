//! Modeline processing for Neovim buffers.
//!
//! Implements `do_modelines()` and `chk_modeline()` which scan buffer lines
//! for embedded editor option settings (e.g. `vim: ts=4 sw=4:`).

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int, c_void};

use crate::BufHandle;

// Return values matching C OK/FAIL
const OK: c_int = 0;
const FAIL: c_int = -1;

// Option flags from option.h
const OPT_LOCAL: c_int = 0x02;
const OPT_MODELINE: c_int = 0x04;

// etype_T enum values (from runtime_defs.h)
// TOP=0, SCRIPT=1, UFUNC=2, AUCMD=3, MODELINE=4
const ETYPE_MODELINE: c_int = 4;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_curbuf_get_b_p_ml() -> c_int;
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> c_int;
    static p_mls: i64;

    fn nvim_ml_get(lnum: c_int) -> *const c_char;
    fn nvim_ml_get_len(lnum: c_int) -> c_int;
    fn nvim_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn skipwhite(s: *const c_char) -> *mut c_char;
    fn nvim_get_min_vim_version() -> c_int;

    /// Wrapper for `try_getdigits`. Returns bytes consumed, or -1 on failure.
    /// Sets *vers to the parsed value on success.
    fn nvim_try_getdigits(s: *const c_char, vers: *mut i64) -> c_int;

    /// Push a modeline entry onto the execution stack.
    fn estack_push(etype: c_int, name: *mut c_char, lnum: c_int) -> *mut c_void;
    /// Pop top entry from execution stack.
    fn estack_pop();

    /// Call `do_set(s, flags)`.
    fn nvim_do_set(s: *mut c_char, flags: c_int) -> c_int;
    /// Save `current_sctx` and set it to `SID_MODELINE` context for `lnum`.
    /// Returns a heap pointer; must pass to `nvim_modeline_sctx_restore()`.
    fn nvim_modeline_sctx_save_and_set(lnum: c_int) -> *mut c_void;
    /// Restore `current_sctx` from pointer returned by `nvim_modeline_sctx_save_and_set()` and free it.
    fn nvim_modeline_sctx_restore(saved: *mut c_void);
    static mut secure: c_int;

    fn nvim_xfree(p: *mut c_void);
}

// =============================================================================
// chk_modeline implementation
// =============================================================================

/// Check a single line for a modeline and process any options found.
///
/// Returns OK (0) on success, FAIL (-1) if an error was encountered.
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
unsafe fn chk_modeline(lnum: c_int, flags: c_int) -> c_int {
    let mut retval: c_int = OK;

    let line_ptr = nvim_ml_get(lnum);
    let line_len = nvim_ml_get_len(lnum) as usize;

    // Work in a byte slice over the C string
    let line_bytes = std::slice::from_raw_parts(line_ptr.cast::<u8>(), line_len);

    // Scan for a modeline marker: "ex:", "vi:", "vim:", "Vim:"
    // The marker must be at the start or preceded by whitespace.
    let mut found_offset: Option<usize> = None;
    let mut prev: i32 = -1;

    let mut i = 0usize;
    while i < line_bytes.len() {
        let ch = line_bytes[i];
        let ch_i32 = i32::from(ch);

        if prev == -1 || (ch as char).is_ascii_whitespace() || prev == 0 {
            // Check for whitespace-preceded markers
            if prev != -1 && line_bytes[i..].starts_with(b"ex:") {
                found_offset = Some(i);
                break;
            }
            if line_bytes[i..].starts_with(b"vi:") {
                found_offset = Some(i);
                break;
            }

            // Accept "vim" or "Vim"
            let is_vim = (ch == b'v' || ch == b'V')
                && i + 2 < line_bytes.len()
                && line_bytes[i + 1] == b'i'
                && line_bytes[i + 2] == b'm';
            if is_vim {
                // Parse optional version constraint: vim<N:, vim=N:, vim>N:, vimN:, vim:
                let version_char_offset = i + 3;
                let vc = if version_char_offset < line_bytes.len() {
                    line_bytes[version_char_offset]
                } else {
                    0
                };

                let (digits_start, has_cmp) = if vc == b'<' || vc == b'=' || vc == b'>' {
                    (version_char_offset + 1, true)
                } else {
                    (version_char_offset, false)
                };

                // Try to parse digits
                let digits_ptr = line_ptr.add(digits_start);
                let mut vers: i64 = 0;
                let consumed = nvim_try_getdigits(digits_ptr, std::ptr::addr_of_mut!(vers));

                // For "vim:" (no digits) consumed would be 0 and we need to check for ':'
                let after_digits = digits_start + consumed.max(0) as usize;
                let next_ch = if after_digits < line_bytes.len() {
                    line_bytes[after_digits]
                } else {
                    0
                };

                let vim_version = nvim_get_min_vim_version();

                // Match conditions (mirrors the C logic):
                // vim:     -> vc == ':'
                // Vim:set  -> s[0] == 'V' requires "set" after ':'
                // vimN:    -> vc.is_ascii_digit() and version check
                // vim<N:, vim>N:, vim=N:
                let matches = next_ch == b':'
                    && (ch != b'V' || {
                        // "Vim:" requires "set" at start of option portion
                        let set_ptr = skipwhite(line_ptr.add(after_digits + 1));
                        let set_bytes = std::slice::from_raw_parts(
                            set_ptr.cast::<u8>(),
                            3.min(line_len.saturating_sub(set_ptr.offset_from(line_ptr) as usize)),
                        );
                        set_bytes.starts_with(b"set")
                    })
                    && (vc == b':'
                        || (vc.is_ascii_digit()
                            && consumed >= 0
                            && i64::from(vim_version) >= vers)
                        || (vc == b'<' && consumed >= 0 && i64::from(vim_version) < vers)
                        || (vc == b'>' && consumed >= 0 && i64::from(vim_version) > vers)
                        || (vc == b'=' && consumed >= 0 && i64::from(vim_version) == vers));
                let _ = has_cmp; // used implicitly via vc

                if matches {
                    found_offset = Some(i);
                    break;
                }
            }
        }

        prev = ch_i32;
        i += 1;
    }

    let Some(start_offset) = found_offset else {
        return retval;
    };

    // Skip over "ex:", "vi:", or "vim:"
    let mut skip = start_offset;
    while skip < line_bytes.len() {
        let ch = line_bytes[skip];
        skip += 1;
        if ch == b':' {
            break;
        }
    }

    // Copy the rest of the line (after the marker's colon)
    let rest_len = line_len.saturating_sub(skip);
    let line_copy_ptr = nvim_xstrnsave(line_ptr.add(skip), rest_len);
    if line_copy_ptr.is_null() {
        return retval;
    }
    // rest_len + 1 would include the NUL terminator

    // Prepare error stack
    estack_push(
        ETYPE_MODELINE,
        c"modelines".as_ptr().cast_mut().cast::<c_char>(),
        lnum,
    );

    // Work through option settings separated by ':'
    let line_copy_end = line_copy_ptr.add(rest_len);
    let mut s = line_copy_ptr;
    let mut end = false;

    while !end {
        // skipwhite
        s = skipwhite(s);

        if *s == 0 {
            break;
        }

        // Find end of set command: ':' or end of line.
        // Skip over "\:", replacing it with ":".
        let mut e = s;
        loop {
            let ch = *e;
            if ch == b':' as c_char || ch == 0 {
                break;
            }
            // Handle "\:" escape: remove the backslash by shifting
            if ch == b'\\' as c_char && *e.add(1) == b':' as c_char {
                // memmove(e, e+1, remaining+1)
                let remaining = line_copy_end.offset_from(e.add(1)) as usize;
                std::ptr::copy(e.add(1), e, remaining + 1);
                // Don't advance e - recheck same position
            } else {
                e = e.add(1);
            }
        }

        if *e == 0 {
            end = true;
        }

        // If there is a "set" or "se" command, require a terminating ':' and
        // ignore the stuff after the ':'.
        let s_bytes_4: [u8; 4] = [*s as u8, *s.add(1) as u8, *s.add(2) as u8, *s.add(3) as u8];
        let s_bytes_3: [u8; 3] = [*s as u8, *s.add(1) as u8, *s.add(2) as u8];

        if s_bytes_4 == *b"set " {
            if *e != b':' as c_char {
                break;
            }
            end = true;
            s = s.add(4);
        } else if s_bytes_3 == *b"se " {
            if *e != b':' as c_char {
                break;
            }
            end = true;
            s = s.add(3);
        }

        // Truncate at the set command end
        *e = 0;

        if *s != 0 {
            // skip over empty "::"
            let secure_save = secure;
            let saved_sctx = nvim_modeline_sctx_save_and_set(lnum);
            secure = 1;
            retval = nvim_do_set(s, OPT_MODELINE | OPT_LOCAL | flags);
            secure = secure_save;
            nvim_modeline_sctx_restore(saved_sctx);
            if retval == FAIL {
                break;
            }
        }

        // Advance to next part
        s = if e == line_copy_end { e } else { e.add(1) };
    }

    estack_pop();
    nvim_xfree(line_copy_ptr.cast::<c_void>());

    retval
}

// =============================================================================
// do_modelines implementation
// =============================================================================

/// Process modelines for the current buffer.
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
pub unsafe fn do_modelines_impl(flags: c_int) {
    static ENTERED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

    if nvim_curbuf_get_b_p_ml() == 0 {
        return;
    }

    let mut nmlines = p_mls as c_int;
    if nmlines == 0 {
        return;
    }

    // Disallow recursive entry: can happen when executing a modeline
    // triggers an autocommand that reloads modelines with ":do".
    if ENTERED.load(std::sync::atomic::Ordering::Relaxed) != 0 {
        return;
    }
    ENTERED.store(1, std::sync::atomic::Ordering::Relaxed);

    let curbuf = nvim_get_curbuf();
    let line_count = nvim_buf_get_ml_line_count(curbuf);

    // Check first nmlines lines
    let mut lnum = 1;
    while nvim_curbuf_get_b_p_ml() != 0 && lnum <= line_count && lnum <= nmlines {
        if chk_modeline(lnum, flags) == FAIL {
            nmlines = 0;
        }
        lnum += 1;
    }

    // Check last nmlines lines
    let line_count2 = nvim_buf_get_ml_line_count(curbuf);
    let mut lnum = line_count2;
    while nvim_curbuf_get_b_p_ml() != 0
        && lnum > 0
        && lnum > nmlines
        && lnum > line_count2 - nmlines
    {
        if chk_modeline(lnum, flags) == FAIL {
            nmlines = 0;
        }
        lnum -= 1;
    }

    ENTERED.store(0, std::sync::atomic::Ordering::Relaxed);
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Process modelines for the current buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_do_modelines(flags: c_int) {
    do_modelines_impl(flags);
}

/// C export: `do_modelines`.
#[unsafe(export_name = "do_modelines")]
pub unsafe extern "C" fn do_modelines_export(flags: c_int) {
    do_modelines_impl(flags);
}
