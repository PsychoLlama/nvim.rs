//! Showcmd display routines for Normal mode.
//!
//! This module provides the Rust implementation of `clear_showcmd()`,
//! `push_showcmd()`, `pop_showcmd()`, `add_to_showcmd()`, and
//! `del_from_showcmd()` from `src/nvim/normal.c`.
//! The complex Visual mode character counting and formatting is delegated
//! to a C helper function.

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

const SHOWCMD_COLS: usize = 10;
const SHOWCMD_BUFLEN: usize = SHOWCMD_COLS + 1 + 30; // = 41

/// Maximum byte length of a UTF-8 encoded code point (mbyte.h MB_MAXCHAR = 6).
const MB_MAXCHAR: usize = 6;

const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -((a) + (b << 8))
}
const KS_EXTRA: c_int = 253;

const KE_IGNORE: c_int = 53;
const KE_LEFTMOUSE: c_int = 44;
const KE_LEFTDRAG: c_int = 45;
const KE_LEFTRELEASE: c_int = 46;
const KE_MOUSEMOVE: c_int = 100;
const KE_MIDDLEMOUSE: c_int = 47;
const KE_MIDDLEDRAG: c_int = 48;
const KE_MIDDLERELEASE: c_int = 49;
const KE_RIGHTMOUSE: c_int = 50;
const KE_RIGHTDRAG: c_int = 51;
const KE_RIGHTRELEASE: c_int = 52;
const KE_MOUSEDOWN: c_int = 75;
const KE_MOUSEUP: c_int = 76;
const KE_MOUSELEFT: c_int = 77;
const KE_MOUSERIGHT: c_int = 78;
const KE_X1MOUSE: c_int = 89;
const KE_X1DRAG: c_int = 90;
const KE_X1RELEASE: c_int = 91;
const KE_X2MOUSE: c_int = 92;
const KE_X2DRAG: c_int = 93;
const KE_X2RELEASE: c_int = 94;
const KE_EVENT: c_int = 102;

const K_IGNORE: c_int = termcap2key(KS_EXTRA, KE_IGNORE);
const K_LEFTMOUSE: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE);
const K_LEFTDRAG: c_int = termcap2key(KS_EXTRA, KE_LEFTDRAG);
const K_LEFTRELEASE: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE);
const K_MOUSEMOVE: c_int = termcap2key(KS_EXTRA, KE_MOUSEMOVE);
const K_MIDDLEMOUSE: c_int = termcap2key(KS_EXTRA, KE_MIDDLEMOUSE);
const K_MIDDLEDRAG: c_int = termcap2key(KS_EXTRA, KE_MIDDLEDRAG);
const K_MIDDLERELEASE: c_int = termcap2key(KS_EXTRA, KE_MIDDLERELEASE);
const K_RIGHTMOUSE: c_int = termcap2key(KS_EXTRA, KE_RIGHTMOUSE);
const K_RIGHTDRAG: c_int = termcap2key(KS_EXTRA, KE_RIGHTDRAG);
const K_RIGHTRELEASE: c_int = termcap2key(KS_EXTRA, KE_RIGHTRELEASE);
const K_MOUSEDOWN: c_int = termcap2key(KS_EXTRA, KE_MOUSEDOWN);
const K_MOUSEUP: c_int = termcap2key(KS_EXTRA, KE_MOUSEUP);
const K_MOUSELEFT: c_int = termcap2key(KS_EXTRA, KE_MOUSELEFT);
const K_MOUSERIGHT: c_int = termcap2key(KS_EXTRA, KE_MOUSERIGHT);
const K_X1MOUSE: c_int = termcap2key(KS_EXTRA, KE_X1MOUSE);
const K_X1DRAG: c_int = termcap2key(KS_EXTRA, KE_X1DRAG);
const K_X1RELEASE: c_int = termcap2key(KS_EXTRA, KE_X1RELEASE);
const K_X2MOUSE: c_int = termcap2key(KS_EXTRA, KE_X2MOUSE);
const K_X2DRAG: c_int = termcap2key(KS_EXTRA, KE_X2DRAG);
const K_X2RELEASE: c_int = termcap2key(KS_EXTRA, KE_X2RELEASE);
const K_EVENT: c_int = termcap2key(KS_EXTRA, KE_EVENT);

/// Keys ignored by add_to_showcmd (mouse events and non-input keys).
const SHOWCMD_IGNORE: &[c_int] = &[
    K_IGNORE,
    K_LEFTMOUSE,
    K_LEFTDRAG,
    K_LEFTRELEASE,
    K_MOUSEMOVE,
    K_MIDDLEMOUSE,
    K_MIDDLEDRAG,
    K_MIDDLERELEASE,
    K_RIGHTMOUSE,
    K_RIGHTDRAG,
    K_RIGHTRELEASE,
    K_MOUSEDOWN,
    K_MOUSEUP,
    K_MOUSELEFT,
    K_MOUSERIGHT,
    K_X1MOUSE,
    K_X1DRAG,
    K_X1RELEASE,
    K_X2MOUSE,
    K_X2DRAG,
    K_X2RELEASE,
    K_EVENT,
];

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    fn nvim_get_p_sc() -> c_int;
    fn nvim_get_showcmd_is_clear() -> bool;
    fn nvim_set_showcmd_is_clear(val: bool);
    fn nvim_get_showcmd_visual() -> bool;
    fn nvim_set_showcmd_visual(val: bool);
    fn nvim_normal_showcmd_buf_ptr() -> *mut std::ffi::c_char;
    fn nvim_old_showcmd_buf_ptr() -> *mut std::ffi::c_char;
    fn nvim_showcmd_buflen() -> usize;

    // Phase 2: display_showcmd accessors
    fn nvim_showcmd_get_p_sloc_first() -> c_int;
    fn nvim_showcmd_set_w_redr_status();
    fn nvim_showcmd_win_redr_status();
    fn nvim_showcmd_set_redraw_tabline();
    fn nvim_showcmd_draw_tabline();
    fn nvim_showcmd_ui_msg_showcmd(buf: *const std::ffi::c_char, is_clear: bool);
    fn nvim_showcmd_get_p_ch() -> c_int;
    fn nvim_showcmd_grid_render(buf: *const std::ffi::c_char, is_clear: bool);

    // Phase 1: Visual info accessors (formerly nvim_clear_showcmd_visual_info)
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_showcmd_char_avail() -> bool;
    fn nvim_lt_VIsual_cursor() -> bool;
    fn nvim_get_VIsual_lnum() -> c_int;
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_hasFolding_up(lnum: c_int, out_lnum: *mut c_int) -> bool;
    fn nvim_hasFolding_down(lnum: c_int, out_lnum: *mut c_int) -> bool;
    fn nvim_get_VIsual_mode() -> c_int;
    fn nvim_getvcols_visual_sbr_save(out_left: *mut c_int, out_right: *mut c_int);
    fn nvim_showcmd_ui_has_messages() -> bool;
    fn nvim_ml_get_pos_visual() -> *mut std::ffi::c_char;
    fn nvim_get_cursor_pos_ptr() -> *const std::ffi::c_char;
    fn nvim_utfc_ptr2len_wrapper(ptr: *const std::ffi::c_char) -> c_int;
    fn nvim_p_sel_is_exclusive() -> bool;

    // Phase 5: add_to_showcmd / del_from_showcmd
    fn nvim_transchar_wrapper(c: c_int) -> *const std::ffi::c_char;
    fn nvim_utf_char2bytes_wrapper(c: c_int, buf: *mut std::ffi::c_char) -> c_int;
    fn nvim_vim_isprintc_wrapper(c: c_int) -> bool;
    fn nvim_showcmd_msg_silent() -> c_int;
}

// =============================================================================
// Ctrl_V constant (same value as in lib.rs, = 22)
// =============================================================================

const CTRL_V: c_int = 22;

// =============================================================================
// Public Rust exports
// =============================================================================

/// Compute Visual area info and write result into showcmd_buf.
/// Returns true if in Visual mode and char_avail() is false.
///
/// Rust port of the former C `nvim_clear_showcmd_visual_info`.
///
/// # Safety
/// Calls C accessor functions; all pointers are valid while in C event loop.
unsafe fn clear_showcmd_visual_info() -> bool {
    if nvim_get_VIsual_active() == 0 || nvim_showcmd_char_avail() {
        return false;
    }

    let cursor_bot = nvim_lt_VIsual_cursor();
    let visual_lnum = nvim_get_VIsual_lnum();
    let cursor_lnum = nvim_get_cursor_lnum();

    let (mut top, mut bot) = if cursor_bot {
        (visual_lnum, cursor_lnum)
    } else {
        (cursor_lnum, visual_lnum)
    };

    nvim_hasFolding_up(top, &raw mut top);
    nvim_hasFolding_down(bot, &raw mut bot);
    let lines = bot - top + 1;

    let vmode = nvim_get_VIsual_mode();
    let showcmd_buf: *mut u8 = nvim_normal_showcmd_buf_ptr().cast();

    if vmode == CTRL_V {
        // Block Visual: LinesxCols
        let mut leftcol: c_int = 0;
        let mut rightcol: c_int = 0;
        nvim_getvcols_visual_sbr_save(&raw mut leftcol, &raw mut rightcol);
        let cols = rightcol - leftcol + 1;
        libc::snprintf(
            showcmd_buf.cast(),
            SHOWCMD_BUFLEN,
            c"%ldx%ld".as_ptr(),
            libc::c_long::from(lines),
            libc::c_long::from(cols),
        );
    } else if vmode == c_int::from(b'V') || visual_lnum != cursor_lnum {
        // Linewise or multi-line charwise
        libc::snprintf(
            showcmd_buf.cast(),
            SHOWCMD_BUFLEN,
            c"%ld".as_ptr(),
            libc::c_long::from(lines),
        );
    } else {
        // Single-line charwise: count bytes and chars
        let (s_init, e_init): (*const u8, *const u8) = if cursor_bot {
            (
                nvim_ml_get_pos_visual().cast(),
                nvim_get_cursor_pos_ptr().cast(),
            )
        } else {
            (
                nvim_get_cursor_pos_ptr().cast(),
                nvim_ml_get_pos_visual().cast(),
            )
        };

        let is_exclusive = nvim_p_sel_is_exclusive();
        let mut s: *const u8 = s_init;
        let e: *const u8 = e_init;
        let mut bytes: c_int = 0;
        let mut chars: c_int = 0;

        // Replicate: while ((*p_sel != 'e') ? s <= e : s < e)
        loop {
            let cond = if is_exclusive { s < e } else { s <= e };
            if !cond {
                break;
            }
            let l = nvim_utfc_ptr2len_wrapper(s.cast());
            if l == 0 {
                bytes += 1;
                chars += 1;
                break;
            }
            bytes += l;
            chars += 1;
            #[allow(clippy::cast_sign_loss)] // utfc_ptr2len returns >= 0
            let l_usize = l as usize;
            s = s.add(l_usize);
        }

        if bytes == chars {
            libc::snprintf(showcmd_buf.cast(), SHOWCMD_BUFLEN, c"%d".as_ptr(), chars);
        } else {
            libc::snprintf(
                showcmd_buf.cast(),
                SHOWCMD_BUFLEN,
                c"%d-%d".as_ptr(),
                chars,
                bytes,
            );
        }
    }

    // Truncate to the display limit.
    let limit = if nvim_showcmd_ui_has_messages() {
        SHOWCMD_BUFLEN - 1
    } else {
        SHOWCMD_COLS
    };
    *showcmd_buf.add(limit) = 0;

    true
}

/// Clear the showcmd display area.
///
/// In Visual mode, computes and displays the size of the visual selection.
/// Otherwise, clears the showcmd buffer and updates the display.
///
/// This is the Rust implementation of `clear_showcmd()` from normal.c.
#[no_mangle]
pub extern "C" fn rs_clear_showcmd() {
    unsafe {
        if nvim_get_p_sc() == 0 {
            return;
        }

        if clear_showcmd_visual_info() {
            // Visual info was computed and written into showcmd_buf.
            nvim_set_showcmd_visual(true);
        } else {
            // Not in Visual mode or char_avail() returned true.
            let buf = nvim_normal_showcmd_buf_ptr();
            *buf = 0; // NUL
            nvim_set_showcmd_visual(false);

            // Don't actually display something if there is nothing to clear.
            if nvim_get_showcmd_is_clear() {
                return;
            }
        }

        rs_display_showcmd();
    }
}

/// Save the current showcmd buffer for later restoration.
///
/// This is the Rust implementation of `push_showcmd()` from normal.c.
///
/// # Safety
/// Reads/writes the shared showcmd_buf and old_showcmd_buf C statics.
#[no_mangle]
pub unsafe extern "C" fn rs_push_showcmd() {
    if nvim_get_p_sc() != 0 {
        let src = nvim_normal_showcmd_buf_ptr();
        let dst = nvim_old_showcmd_buf_ptr();
        let len = nvim_showcmd_buflen();
        // Safe: both are valid C arrays of size SHOWCMD_BUFLEN
        std::ptr::copy_nonoverlapping(src, dst, len);
    }
}

/// Restore the showcmd buffer saved by push_showcmd().
///
/// This is the Rust implementation of `pop_showcmd()` from normal.c.
///
/// # Safety
/// Reads/writes the shared showcmd_buf and old_showcmd_buf C statics.
#[no_mangle]
pub unsafe extern "C" fn rs_pop_showcmd() {
    if nvim_get_p_sc() == 0 {
        return;
    }
    let src = nvim_old_showcmd_buf_ptr();
    let dst = nvim_normal_showcmd_buf_ptr();
    let len = nvim_showcmd_buflen();
    // Safe: both are valid C arrays of size SHOWCMD_BUFLEN
    std::ptr::copy_nonoverlapping(src, dst, len);
    rs_display_showcmd();
}

/// Append the representation of key `c` to the shown command string.
///
/// Filters out mouse events and other non-input keys. Handles overflow
/// by shifting out leading characters. Calls display_showcmd() if no
/// more input is immediately available.
///
/// Returns true if the display was updated (display_showcmd was called).
///
/// This is the Rust implementation of `add_to_showcmd()` from normal.c.
///
/// # Safety
/// Reads/writes the shared showcmd_buf C static and calls C helpers.
#[no_mangle]
pub unsafe extern "C" fn rs_add_to_showcmd(c: c_int) -> bool {
    if nvim_get_p_sc() == 0 || nvim_showcmd_msg_silent() != 0 {
        return false;
    }

    // If a Visual selection was last displayed, clear it first.
    if nvim_get_showcmd_visual() {
        let buf = nvim_normal_showcmd_buf_ptr();
        *buf = 0;
        nvim_set_showcmd_visual(false);
    }

    // IS_SPECIAL(c) is equivalent to c < 0.
    if c < 0 && SHOWCMD_IGNORE.contains(&c) {
        return false;
    }

    // MB_MAXCHAR + 1 = 7 bytes for UTF-8 encoding + NUL.
    let mut mbyte_buf = [0u8; MB_MAXCHAR + 1];

    // Build the display string into mbyte_buf, or point at transchar's static buf.
    // We always write into mbyte_buf so we own the data and avoid pointer casts.
    let char_len: usize;
    {
        if c <= 0x7f || !nvim_vim_isprintc_wrapper(c) {
            // Use transchar for ASCII/non-printable chars; result is a static buf.
            let tc: *const u8 = nvim_transchar_wrapper(c).cast();
            if *tc == b' ' {
                // transchar returned a space: show literal "<20>"
                mbyte_buf[0] = b'<';
                mbyte_buf[1] = b'2';
                mbyte_buf[2] = b'0';
                mbyte_buf[3] = b'>';
                mbyte_buf[4] = 0;
                char_len = 4;
            } else {
                // Copy the transchar output into our buffer.
                let mut i = 0usize;
                while *tc.add(i) != 0 && i < MB_MAXCHAR {
                    mbyte_buf[i] = *tc.add(i);
                    i += 1;
                }
                mbyte_buf[i] = 0;
                char_len = i;
            }
        } else {
            let len_i =
                nvim_utf_char2bytes_wrapper(c, mbyte_buf.as_mut_ptr().cast::<std::ffi::c_char>());
            let len = usize::try_from(len_i).unwrap_or(0).min(MB_MAXCHAR);
            mbyte_buf[len] = 0;
            char_len = len;
        }
    }

    // Compute lengths.
    let showcmd_buf: *mut u8 = nvim_normal_showcmd_buf_ptr().cast();
    let old_len = libc_strlen_u8(showcmd_buf);
    let extra_len = char_len;
    let limit = if nvim_showcmd_ui_has_messages() {
        SHOWCMD_BUFLEN - 1
    } else {
        SHOWCMD_COLS
    };

    if old_len + extra_len > limit {
        let overflow = old_len + extra_len - limit;
        // Shift showcmd_buf left by `overflow` bytes.
        std::ptr::copy(
            showcmd_buf.add(overflow),
            showcmd_buf,
            old_len - overflow + 1,
        );
    }

    // Append mbyte_buf[:char_len+1] to showcmd_buf (strcat equivalent).
    let new_old_len = libc_strlen_u8(showcmd_buf);
    std::ptr::copy_nonoverlapping(
        mbyte_buf.as_ptr(),
        showcmd_buf.add(new_old_len),
        extra_len + 1,
    );

    if nvim_showcmd_char_avail() {
        return false;
    }

    rs_display_showcmd();
    true
}

/// Remove `len` characters from the end of the shown command string.
///
/// This is the Rust implementation of `del_from_showcmd()` from normal.c.
///
/// # Safety
/// Reads/writes the shared showcmd_buf C static and calls C helpers.
#[no_mangle]
pub unsafe extern "C" fn rs_del_from_showcmd(len: c_int) {
    if nvim_get_p_sc() == 0 {
        return;
    }

    let showcmd_buf: *mut u8 = nvim_normal_showcmd_buf_ptr().cast();
    let old_len = libc_strlen_u8(showcmd_buf);
    let to_remove = usize::try_from(len).unwrap_or(0).min(old_len);
    *showcmd_buf.add(old_len - to_remove) = 0;

    if !nvim_showcmd_char_avail() {
        rs_display_showcmd();
    }
}

/// Render the showcmd buffer to the appropriate display location.
///
/// Dispatches to statusline, tabline, UI messages protocol, or last-line
/// grid rendering based on the `showcmdloc` option.
///
/// Rust port of the C `display_showcmd` function.
///
/// # Safety
/// Calls C accessor functions for rendering. All pointers returned by the
/// C accessors are valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn rs_display_showcmd() {
    let buf_ptr = nvim_normal_showcmd_buf_ptr();
    let is_clear = *buf_ptr == 0;
    nvim_set_showcmd_is_clear(is_clear);

    let sloc = nvim_showcmd_get_p_sloc_first();

    if sloc == c_int::from(b's') {
        // showcmdloc=statusline
        if is_clear {
            nvim_showcmd_set_w_redr_status();
        } else {
            nvim_showcmd_win_redr_status();
        }
        return;
    }

    if sloc == c_int::from(b't') {
        // showcmdloc=tabline
        if is_clear {
            nvim_showcmd_set_redraw_tabline();
        } else {
            nvim_showcmd_draw_tabline();
        }
        return;
    }

    // showcmdloc=last (or empty)
    if nvim_showcmd_ui_has_messages() {
        nvim_showcmd_ui_msg_showcmd(buf_ptr, is_clear);
        return;
    }

    if nvim_showcmd_get_p_ch() == 0 {
        return;
    }

    nvim_showcmd_grid_render(buf_ptr, is_clear);
}

// =============================================================================
// Helpers
// =============================================================================

/// Compute the byte length of a null-terminated byte string (`u8` pointer variant).
/// This is equivalent to `strlen(s)`.
///
/// # Safety
/// `s` must be a valid pointer to a null-terminated byte string.
unsafe fn libc_strlen_u8(s: *const u8) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_showcmd_constants() {
        // SHOWCMD_COLS = 10, SHOWCMD_BUFLEN = 10 + 1 + 30 = 41
        assert_eq!(SHOWCMD_COLS, 10);
        assert_eq!(SHOWCMD_BUFLEN, 41);
    }

    #[test]
    fn test_key_constants() {
        // Verify a few key constants against known values.
        // K_IGNORE = -(253 + 53*256) = -(253 + 13568) = -13821
        assert_eq!(K_IGNORE, -13821);
        // K_EVENT = -(253 + 102*256) = -(253 + 26112) = -26365
        assert_eq!(K_EVENT, -26365);
        // K_MOUSEMOVE = -(253 + 100*256) = -(253 + 25600) = -25853
        assert_eq!(K_MOUSEMOVE, -25853);
    }
}
