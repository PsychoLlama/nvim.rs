//! Terminal emulator utilities for Neovim
//!
//! This crate provides Rust implementations for terminal-related functions,
//! primarily working with the libvterm-based terminal emulator.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;
use std::os::raw::c_void;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to Terminal struct (terminal.c struct terminal)
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalHandle(*mut c_void);

impl TerminalHandle {
    /// Create a handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `Terminal*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    /// Get the `closed` field from a Terminal.
    fn nvim_terminal_get_closed(term: TerminalHandle) -> c_int;

    /// Get the `buf_handle` field from a Terminal.
    fn nvim_terminal_get_buf_handle(term: TerminalHandle) -> c_int;

    /// Get the `theme_updates` field from a Terminal.
    fn nvim_terminal_get_theme_updates(term: TerminalHandle) -> c_int;

    /// Get the `forward_mouse` field from a Terminal.
    fn nvim_terminal_get_forward_mouse(term: TerminalHandle) -> c_int;

    /// Get the cursor row from a Terminal.
    fn nvim_terminal_get_cursor_row(term: TerminalHandle) -> c_int;

    /// Get the cursor col from a Terminal.
    fn nvim_terminal_get_cursor_col(term: TerminalHandle) -> c_int;

    /// Get the cursor visible flag from a Terminal.
    fn nvim_terminal_get_cursor_visible(term: TerminalHandle) -> c_int;

    /// Get the cursor shape from a Terminal.
    fn nvim_terminal_get_cursor_shape(term: TerminalHandle) -> c_int;

    /// Get the cursor blink flag from a Terminal.
    fn nvim_terminal_get_cursor_blink(term: TerminalHandle) -> c_int;
}

// =============================================================================
// Terminal Status Functions
// =============================================================================

/// Check if a terminal is running (not closed).
///
/// This is the Rust equivalent of `terminal_running()` in terminal.c.
#[inline]
fn terminal_running_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    unsafe { nvim_terminal_get_closed(term) == 0 }
}

/// FFI wrapper for `terminal_running`.
///
/// Returns 1 if the terminal is running, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_terminal_running(term: TerminalHandle) -> c_int {
    c_int::from(terminal_running_impl(term))
}

// =============================================================================
// Terminal Buffer Functions
// =============================================================================

/// Get the buffer handle associated with a terminal.
///
/// This is the Rust equivalent of `terminal_buf()` in terminal.c.
#[inline]
fn terminal_buf_impl(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { nvim_terminal_get_buf_handle(term) }
}

/// FFI wrapper for `terminal_buf`.
#[no_mangle]
pub extern "C" fn rs_terminal_buf(term: TerminalHandle) -> c_int {
    terminal_buf_impl(term)
}

// =============================================================================
// Terminal Cursor Functions
// =============================================================================

/// Get the cursor row for a terminal.
#[inline]
fn terminal_cursor_row_impl(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { nvim_terminal_get_cursor_row(term) }
}

/// FFI wrapper for getting terminal cursor row.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_row(term: TerminalHandle) -> c_int {
    terminal_cursor_row_impl(term)
}

/// Get the cursor column for a terminal.
#[inline]
fn terminal_cursor_col_impl(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { nvim_terminal_get_cursor_col(term) }
}

/// FFI wrapper for getting terminal cursor column.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_col(term: TerminalHandle) -> c_int {
    terminal_cursor_col_impl(term)
}

/// Check if the terminal cursor is visible.
#[inline]
fn terminal_cursor_visible_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    unsafe { nvim_terminal_get_cursor_visible(term) != 0 }
}

/// FFI wrapper for checking if terminal cursor is visible.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_visible(term: TerminalHandle) -> c_int {
    c_int::from(terminal_cursor_visible_impl(term))
}

/// Get the terminal cursor shape.
#[inline]
fn terminal_cursor_shape_impl(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { nvim_terminal_get_cursor_shape(term) }
}

/// FFI wrapper for getting terminal cursor shape.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_shape(term: TerminalHandle) -> c_int {
    terminal_cursor_shape_impl(term)
}

/// Check if the terminal cursor should blink.
#[inline]
fn terminal_cursor_blink_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    unsafe { nvim_terminal_get_cursor_blink(term) != 0 }
}

/// FFI wrapper for checking if terminal cursor should blink.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_blink(term: TerminalHandle) -> c_int {
    c_int::from(terminal_cursor_blink_impl(term))
}

// =============================================================================
// Terminal Property Functions
// =============================================================================

/// Check if the terminal forwards mouse events.
#[inline]
fn terminal_forward_mouse_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    unsafe { nvim_terminal_get_forward_mouse(term) != 0 }
}

/// FFI wrapper for checking if terminal forwards mouse.
#[no_mangle]
pub extern "C" fn rs_terminal_forward_mouse(term: TerminalHandle) -> c_int {
    c_int::from(terminal_forward_mouse_impl(term))
}

/// Check if the terminal wants theme update notifications.
#[inline]
fn terminal_theme_updates_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    unsafe { nvim_terminal_get_theme_updates(term) != 0 }
}

/// FFI wrapper for checking if terminal wants theme updates.
#[no_mangle]
pub extern "C" fn rs_terminal_theme_updates(term: TerminalHandle) -> c_int {
    c_int::from(terminal_theme_updates_impl(term))
}

// =============================================================================
// VTerm Key Constants (from vterm_keycodes.h)
// =============================================================================

/// No key.
pub const VTERM_KEY_NONE: c_int = 0;
/// Enter key.
pub const VTERM_KEY_ENTER: c_int = 1;
/// Tab key.
pub const VTERM_KEY_TAB: c_int = 2;
/// Backspace key.
pub const VTERM_KEY_BACKSPACE: c_int = 3;
/// Escape key.
pub const VTERM_KEY_ESCAPE: c_int = 4;
/// Up arrow key.
pub const VTERM_KEY_UP: c_int = 5;
/// Down arrow key.
pub const VTERM_KEY_DOWN: c_int = 6;
/// Left arrow key.
pub const VTERM_KEY_LEFT: c_int = 7;
/// Right arrow key.
pub const VTERM_KEY_RIGHT: c_int = 8;
/// Insert key.
pub const VTERM_KEY_INS: c_int = 9;
/// Delete key.
pub const VTERM_KEY_DEL: c_int = 10;
/// Home key.
pub const VTERM_KEY_HOME: c_int = 11;
/// End key.
pub const VTERM_KEY_END: c_int = 12;
/// Page Up key.
pub const VTERM_KEY_PAGEUP: c_int = 13;
/// Page Down key.
pub const VTERM_KEY_PAGEDOWN: c_int = 14;
/// Keypad 0.
pub const VTERM_KEY_KP_0: c_int = 16;
/// Keypad 1.
pub const VTERM_KEY_KP_1: c_int = 17;
/// Keypad 2.
pub const VTERM_KEY_KP_2: c_int = 18;
/// Keypad 3.
pub const VTERM_KEY_KP_3: c_int = 19;
/// Keypad 4.
pub const VTERM_KEY_KP_4: c_int = 20;
/// Keypad 5.
pub const VTERM_KEY_KP_5: c_int = 21;
/// Keypad 6.
pub const VTERM_KEY_KP_6: c_int = 22;
/// Keypad 7.
pub const VTERM_KEY_KP_7: c_int = 23;
/// Keypad 8.
pub const VTERM_KEY_KP_8: c_int = 24;
/// Keypad 9.
pub const VTERM_KEY_KP_9: c_int = 25;
/// Keypad multiply (*).
pub const VTERM_KEY_KP_MULT: c_int = 26;
/// Keypad plus (+).
pub const VTERM_KEY_KP_PLUS: c_int = 27;
/// Keypad comma (,).
pub const VTERM_KEY_KP_COMMA: c_int = 28;
/// Keypad minus (-).
pub const VTERM_KEY_KP_MINUS: c_int = 29;
/// Keypad period (.).
pub const VTERM_KEY_KP_PERIOD: c_int = 30;
/// Keypad divide (/).
pub const VTERM_KEY_KP_DIVIDE: c_int = 31;
/// Keypad Enter.
pub const VTERM_KEY_KP_ENTER: c_int = 32;
/// Keypad equal (=).
pub const VTERM_KEY_KP_EQUAL: c_int = 33;
/// Maximum keypad key (sentinel for function keys).
pub const VTERM_KEY_MAX: c_int = VTERM_KEY_KP_EQUAL;
/// Number of function keys supported.
pub const VTERM_KEY_FUNCTION_MAX: c_int = 66;

/// Generate function key code F1-F66.
#[inline]
pub const fn vterm_key_function(n: c_int) -> c_int {
    if n < 1 || n > VTERM_KEY_FUNCTION_MAX {
        VTERM_KEY_NONE
    } else {
        VTERM_KEY_MAX + n
    }
}

// =============================================================================
// VTerm Modifier Constants
// =============================================================================

/// No modifier.
pub const VTERM_MOD_NONE: c_int = 0;
/// Shift modifier.
pub const VTERM_MOD_SHIFT: c_int = 1;
/// Alt modifier.
pub const VTERM_MOD_ALT: c_int = 2;
/// Ctrl modifier.
pub const VTERM_MOD_CTRL: c_int = 4;

// =============================================================================
// Neovim Key Constants (from keycodes.h)
// =============================================================================

/// Convert termcap codes to internal key representation
const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -((a) + (b << 8))
}

// KS_EXTRA for special keys
const KS_EXTRA: c_int = 253;

// Neovim special key codes
const K_BS: c_int = termcap2key(b'k' as c_int, b'b' as c_int);
const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
const K_LEFT: c_int = termcap2key(b'k' as c_int, b'l' as c_int);
const K_RIGHT: c_int = termcap2key(b'k' as c_int, b'r' as c_int);
const K_HOME: c_int = termcap2key(b'k' as c_int, b'h' as c_int);
const K_END: c_int = termcap2key(b'@' as c_int, b'7' as c_int);
const K_PAGEUP: c_int = termcap2key(b'k' as c_int, b'P' as c_int);
const K_PAGEDOWN: c_int = termcap2key(b'k' as c_int, b'N' as c_int);
const K_INS: c_int = termcap2key(b'k' as c_int, b'I' as c_int);
const K_DEL: c_int = termcap2key(b'k' as c_int, b'D' as c_int);
const K_S_TAB: c_int = termcap2key(b'k' as c_int, b'B' as c_int);

// Keypad keys
const K_KUP: c_int = termcap2key(b'K' as c_int, b'u' as c_int);
const K_KDOWN: c_int = termcap2key(b'K' as c_int, b'd' as c_int);
const K_KLEFT: c_int = termcap2key(b'K' as c_int, b'l' as c_int);
const K_KRIGHT: c_int = termcap2key(b'K' as c_int, b'r' as c_int);
const K_KHOME: c_int = termcap2key(b'K' as c_int, b'1' as c_int);
const K_KEND: c_int = termcap2key(b'K' as c_int, b'4' as c_int);
const K_KPAGEUP: c_int = termcap2key(b'K' as c_int, b'3' as c_int);
const K_KPAGEDOWN: c_int = termcap2key(b'K' as c_int, b'5' as c_int);
const K_KORIGIN: c_int = termcap2key(b'K' as c_int, b'7' as c_int);
const K_K0: c_int = termcap2key(b'K' as c_int, b'0' as c_int);
const K_K1: c_int = termcap2key(b'K' as c_int, b'a' as c_int);
const K_K2: c_int = termcap2key(b'K' as c_int, b'b' as c_int);
const K_K3: c_int = termcap2key(b'K' as c_int, b'c' as c_int);
const K_K4: c_int = termcap2key(b'K' as c_int, b'e' as c_int);
const K_K5: c_int = termcap2key(b'K' as c_int, b'f' as c_int);
const K_K6: c_int = termcap2key(b'K' as c_int, b'g' as c_int);
const K_K7: c_int = termcap2key(b'K' as c_int, b'h' as c_int);
const K_K8: c_int = termcap2key(b'K' as c_int, b'i' as c_int);
const K_K9: c_int = termcap2key(b'K' as c_int, b'j' as c_int);
const K_KINS: c_int = termcap2key(b'K' as c_int, b'I' as c_int);
const K_KDEL: c_int = termcap2key(b'K' as c_int, b'D' as c_int);
const K_KPOINT: c_int = termcap2key(b'K' as c_int, b'.' as c_int);
const K_KENTER: c_int = termcap2key(b'K' as c_int, b'\r' as c_int);
const K_KPLUS: c_int = termcap2key(b'K' as c_int, b'+' as c_int);
const K_KMINUS: c_int = termcap2key(b'K' as c_int, b'-' as c_int);
const K_KMULTIPLY: c_int = termcap2key(b'K' as c_int, b'*' as c_int);
const K_KDIVIDE: c_int = termcap2key(b'K' as c_int, b'/' as c_int);

// Function keys F1-F12
const K_F1: c_int = termcap2key(b'k' as c_int, b'1' as c_int);
const K_F2: c_int = termcap2key(b'k' as c_int, b'2' as c_int);
const K_F3: c_int = termcap2key(b'k' as c_int, b'3' as c_int);
const K_F4: c_int = termcap2key(b'k' as c_int, b'4' as c_int);
const K_F5: c_int = termcap2key(b'k' as c_int, b'5' as c_int);
const K_F6: c_int = termcap2key(b'k' as c_int, b'6' as c_int);
const K_F7: c_int = termcap2key(b'k' as c_int, b'7' as c_int);
const K_F8: c_int = termcap2key(b'k' as c_int, b'8' as c_int);
const K_F9: c_int = termcap2key(b'k' as c_int, b'9' as c_int);
const K_F10: c_int = termcap2key(b'k' as c_int, b';' as c_int);
const K_F11: c_int = termcap2key(b'F' as c_int, b'1' as c_int);
const K_F12: c_int = termcap2key(b'F' as c_int, b'2' as c_int);

// Shifted function keys (KS_EXTRA variants)
const KE_S_UP: c_int = 4;
const KE_S_DOWN: c_int = 5;
const KE_S_F1: c_int = 6;
const KE_S_F2: c_int = 7;
const KE_S_F3: c_int = 8;
const KE_S_F4: c_int = 9;
const KE_S_F5: c_int = 10;
const KE_S_F6: c_int = 11;
const KE_S_F7: c_int = 12;
const KE_S_F8: c_int = 13;
const KE_S_F9: c_int = 14;
const KE_S_F10: c_int = 15;
const KE_S_F11: c_int = 16;
const KE_S_F12: c_int = 17;
const KE_C_LEFT: c_int = 85;
const KE_C_RIGHT: c_int = 86;
const KE_C_HOME: c_int = 87;
const KE_C_END: c_int = 88;

const K_S_UP: c_int = termcap2key(KS_EXTRA, KE_S_UP);
const K_S_DOWN: c_int = termcap2key(KS_EXTRA, KE_S_DOWN);
const K_S_LEFT: c_int = termcap2key(b'#' as c_int, b'4' as c_int);
const K_S_RIGHT: c_int = termcap2key(b'%' as c_int, b'i' as c_int);
const K_S_HOME: c_int = termcap2key(b'#' as c_int, b'2' as c_int);
const K_S_END: c_int = termcap2key(b'*' as c_int, b'7' as c_int);
const K_S_F1: c_int = termcap2key(KS_EXTRA, KE_S_F1);
const K_S_F2: c_int = termcap2key(KS_EXTRA, KE_S_F2);
const K_S_F3: c_int = termcap2key(KS_EXTRA, KE_S_F3);
const K_S_F4: c_int = termcap2key(KS_EXTRA, KE_S_F4);
const K_S_F5: c_int = termcap2key(KS_EXTRA, KE_S_F5);
const K_S_F6: c_int = termcap2key(KS_EXTRA, KE_S_F6);
const K_S_F7: c_int = termcap2key(KS_EXTRA, KE_S_F7);
const K_S_F8: c_int = termcap2key(KS_EXTRA, KE_S_F8);
const K_S_F9: c_int = termcap2key(KS_EXTRA, KE_S_F9);
const K_S_F10: c_int = termcap2key(KS_EXTRA, KE_S_F10);
const K_S_F11: c_int = termcap2key(KS_EXTRA, KE_S_F11);
const K_S_F12: c_int = termcap2key(KS_EXTRA, KE_S_F12);

const K_C_LEFT: c_int = termcap2key(KS_EXTRA, KE_C_LEFT);
const K_C_RIGHT: c_int = termcap2key(KS_EXTRA, KE_C_RIGHT);
const K_C_HOME: c_int = termcap2key(KS_EXTRA, KE_C_HOME);
const K_C_END: c_int = termcap2key(KS_EXTRA, KE_C_END);

// Modifier mask constants (from keycodes.h)
const MOD_MASK_SHIFT: c_int = 0x02;
const MOD_MASK_CTRL: c_int = 0x04;
const MOD_MASK_ALT: c_int = 0x08;

// ASCII constants
const TAB: c_int = 0x09;
const ESC: c_int = 0x1B;
const CTRL_M: c_int = 0x0D;

// =============================================================================
// Key Conversion Result
// =============================================================================

/// Result of converting a Neovim key to `VTerm` key with modifiers.
#[repr(C)]
pub struct VTermKeyResult {
    /// The `VTerm` key code (`VTERM_KEY_*` or `vterm_key_function(n)`).
    /// Returns `VTERM_KEY_NONE` if not a special key (send as character).
    pub key: c_int,
    /// The `VTerm` modifier flags (`VTERM_MOD_*`).
    pub modifiers: c_int,
}

// =============================================================================
// Key Conversion Functions
// =============================================================================

/// Convert Neovim modifier mask to `VTerm` modifier mask.
///
/// This handles the modifier conversion and updates the key if Ctrl is pressed
/// with an uppercase letter (vterm expects lowercase with Ctrl).
const fn convert_modifiers(key: c_int, nvim_mod_mask: c_int) -> (c_int, c_int) {
    let mut vterm_mod: c_int = 0;
    let mut result_key = key;

    if (nvim_mod_mask & MOD_MASK_SHIFT) != 0 {
        vterm_mod |= VTERM_MOD_SHIFT;
    }
    if (nvim_mod_mask & MOD_MASK_CTRL) != 0 {
        vterm_mod |= VTERM_MOD_CTRL;
        // vterm interprets CTRL+A as SHIFT+CTRL, change to CTRL+a
        if (nvim_mod_mask & MOD_MASK_SHIFT) == 0
            && result_key >= b'A' as c_int
            && result_key <= b'Z' as c_int
        {
            result_key += b'a' as c_int - b'A' as c_int;
        }
    }
    if (nvim_mod_mask & MOD_MASK_ALT) != 0 {
        vterm_mod |= VTERM_MOD_ALT;
    }

    (result_key, vterm_mod)
}

/// Convert a Neovim key code to a `VTerm` key code.
///
/// Takes a Neovim key code and converts it to the corresponding `VTerm` key.
/// Also converts modifier keys that are embedded in the key code (like `K_S_UP`).
///
/// # Arguments
/// * `key` - The Neovim key code
/// * `nvim_mod_mask` - The Neovim modifier mask (`MOD_MASK_*`)
///
/// # Returns
/// A `VTermKeyResult` with the `VTerm` key code and modifiers.
/// If key is `VTERM_KEY_NONE`, the key should be sent as a character instead.
#[no_mangle]
#[allow(clippy::too_many_lines)]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_convert_key(key: c_int, nvim_mod_mask: c_int) -> VTermKeyResult {
    let (result_key, mut vterm_mod) = convert_modifiers(key, nvim_mod_mask);

    // Handle keys that have shift/ctrl embedded in the key code
    match result_key {
        K_S_TAB | K_S_UP | K_S_DOWN | K_S_LEFT | K_S_RIGHT | K_S_HOME | K_S_END | K_S_F1
        | K_S_F2 | K_S_F3 | K_S_F4 | K_S_F5 | K_S_F6 | K_S_F7 | K_S_F8 | K_S_F9 | K_S_F10
        | K_S_F11 | K_S_F12 => {
            vterm_mod |= VTERM_MOD_SHIFT;
        }
        K_C_LEFT | K_C_RIGHT | K_C_HOME | K_C_END => {
            vterm_mod |= VTERM_MOD_CTRL;
        }
        _ => {}
    }

    // Convert the key to VTerm key code
    let vterm_key = match result_key {
        K_BS => VTERM_KEY_BACKSPACE,
        K_S_TAB | TAB => VTERM_KEY_TAB,
        CTRL_M => VTERM_KEY_ENTER,
        ESC => VTERM_KEY_ESCAPE,

        K_S_UP | K_UP => VTERM_KEY_UP,
        K_S_DOWN | K_DOWN => VTERM_KEY_DOWN,
        K_S_LEFT | K_C_LEFT | K_LEFT => VTERM_KEY_LEFT,
        K_S_RIGHT | K_C_RIGHT | K_RIGHT => VTERM_KEY_RIGHT,

        K_INS => VTERM_KEY_INS,
        K_DEL => VTERM_KEY_DEL,
        K_S_HOME | K_C_HOME | K_HOME => VTERM_KEY_HOME,
        K_S_END | K_C_END | K_END => VTERM_KEY_END,
        K_PAGEUP => VTERM_KEY_PAGEUP,
        K_PAGEDOWN => VTERM_KEY_PAGEDOWN,

        // Keypad keys
        K_K0 | K_KINS => VTERM_KEY_KP_0,
        K_K1 | K_KEND => VTERM_KEY_KP_1,
        K_K2 | K_KDOWN => VTERM_KEY_KP_2,
        K_K3 | K_KPAGEDOWN => VTERM_KEY_KP_3,
        K_K4 | K_KLEFT => VTERM_KEY_KP_4,
        K_K5 | K_KORIGIN => VTERM_KEY_KP_5,
        K_K6 | K_KRIGHT => VTERM_KEY_KP_6,
        K_K7 | K_KHOME => VTERM_KEY_KP_7,
        K_K8 | K_KUP => VTERM_KEY_KP_8,
        K_K9 | K_KPAGEUP => VTERM_KEY_KP_9,
        K_KDEL | K_KPOINT => VTERM_KEY_KP_PERIOD,
        K_KENTER => VTERM_KEY_KP_ENTER,
        K_KPLUS => VTERM_KEY_KP_PLUS,
        K_KMINUS => VTERM_KEY_KP_MINUS,
        K_KMULTIPLY => VTERM_KEY_KP_MULT,
        K_KDIVIDE => VTERM_KEY_KP_DIVIDE,

        // Function keys
        K_S_F1 | K_F1 => vterm_key_function(1),
        K_S_F2 | K_F2 => vterm_key_function(2),
        K_S_F3 | K_F3 => vterm_key_function(3),
        K_S_F4 | K_F4 => vterm_key_function(4),
        K_S_F5 | K_F5 => vterm_key_function(5),
        K_S_F6 | K_F6 => vterm_key_function(6),
        K_S_F7 | K_F7 => vterm_key_function(7),
        K_S_F8 | K_F8 => vterm_key_function(8),
        K_S_F9 | K_F9 => vterm_key_function(9),
        K_S_F10 | K_F10 => vterm_key_function(10),
        K_S_F11 | K_F11 => vterm_key_function(11),
        K_S_F12 | K_F12 => vterm_key_function(12),

        // Not a special key - return VTERM_KEY_NONE to indicate
        // the key should be sent as a character
        _ => VTERM_KEY_NONE,
    };

    VTermKeyResult {
        key: vterm_key,
        modifiers: vterm_mod,
    }
}

/// Check if a character should be filtered when sending to terminal.
///
/// Some characters like NUL shouldn't be sent to the terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_is_filter_char(c: c_int) -> c_int {
    // Filter out NUL bytes and certain control characters
    c_int::from(c == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_handle_null() {
        let handle = unsafe { TerminalHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!terminal_running_impl(handle));
        assert!(!terminal_forward_mouse_impl(handle));
        assert!(!terminal_theme_updates_impl(handle));
    }

    #[test]
    fn test_terminal_handle_non_null() {
        let fake_ptr = 0x1000 as *mut c_void;
        let handle = unsafe { TerminalHandle::from_ptr(fake_ptr) };
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), fake_ptr);
    }

    #[test]
    fn test_vterm_key_function() {
        // Valid function keys
        assert_eq!(vterm_key_function(1), VTERM_KEY_MAX + 1);
        assert_eq!(vterm_key_function(12), VTERM_KEY_MAX + 12);
        assert_eq!(vterm_key_function(66), VTERM_KEY_MAX + 66);

        // Invalid function keys
        assert_eq!(vterm_key_function(0), VTERM_KEY_NONE);
        assert_eq!(vterm_key_function(67), VTERM_KEY_NONE);
        assert_eq!(vterm_key_function(-1), VTERM_KEY_NONE);
    }

    #[test]
    fn test_convert_key_arrow_keys() {
        let result = rs_terminal_convert_key(K_UP, 0);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_NONE);

        let result = rs_terminal_convert_key(K_DOWN, 0);
        assert_eq!(result.key, VTERM_KEY_DOWN);

        let result = rs_terminal_convert_key(K_LEFT, 0);
        assert_eq!(result.key, VTERM_KEY_LEFT);

        let result = rs_terminal_convert_key(K_RIGHT, 0);
        assert_eq!(result.key, VTERM_KEY_RIGHT);
    }

    #[test]
    fn test_convert_key_shifted_arrows() {
        let result = rs_terminal_convert_key(K_S_UP, 0);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_SHIFT);

        let result = rs_terminal_convert_key(K_S_DOWN, 0);
        assert_eq!(result.key, VTERM_KEY_DOWN);
        assert_eq!(result.modifiers, VTERM_MOD_SHIFT);
    }

    #[test]
    fn test_convert_key_ctrl_arrows() {
        let result = rs_terminal_convert_key(K_C_LEFT, 0);
        assert_eq!(result.key, VTERM_KEY_LEFT);
        assert_eq!(result.modifiers, VTERM_MOD_CTRL);

        let result = rs_terminal_convert_key(K_C_RIGHT, 0);
        assert_eq!(result.key, VTERM_KEY_RIGHT);
        assert_eq!(result.modifiers, VTERM_MOD_CTRL);
    }

    #[test]
    fn test_convert_key_function_keys() {
        let result = rs_terminal_convert_key(K_F1, 0);
        assert_eq!(result.key, vterm_key_function(1));

        let result = rs_terminal_convert_key(K_F12, 0);
        assert_eq!(result.key, vterm_key_function(12));

        // Shifted function key
        let result = rs_terminal_convert_key(K_S_F1, 0);
        assert_eq!(result.key, vterm_key_function(1));
        assert_eq!(result.modifiers, VTERM_MOD_SHIFT);
    }

    #[test]
    fn test_convert_key_keypad() {
        let result = rs_terminal_convert_key(K_K0, 0);
        assert_eq!(result.key, VTERM_KEY_KP_0);

        let result = rs_terminal_convert_key(K_KENTER, 0);
        assert_eq!(result.key, VTERM_KEY_KP_ENTER);
    }

    #[test]
    fn test_convert_key_with_modifiers() {
        // Shift modifier
        let result = rs_terminal_convert_key(K_UP, MOD_MASK_SHIFT);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_SHIFT);

        // Ctrl modifier
        let result = rs_terminal_convert_key(K_UP, MOD_MASK_CTRL);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_CTRL);

        // Alt modifier
        let result = rs_terminal_convert_key(K_UP, MOD_MASK_ALT);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_ALT);

        // Multiple modifiers
        let result = rs_terminal_convert_key(K_UP, MOD_MASK_CTRL | MOD_MASK_ALT);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_CTRL | VTERM_MOD_ALT);
    }

    #[test]
    fn test_convert_key_regular_char() {
        // Regular ASCII character returns VTERM_KEY_NONE
        let result = rs_terminal_convert_key(c_int::from(b'a'), 0);
        assert_eq!(result.key, VTERM_KEY_NONE);
    }

    #[test]
    fn test_filter_char() {
        assert_eq!(rs_terminal_is_filter_char(0), 1);
        assert_eq!(rs_terminal_is_filter_char(c_int::from(b'a')), 0);
        assert_eq!(rs_terminal_is_filter_char(c_int::from(b' ')), 0);
    }

    #[test]
    fn test_vterm_constants() {
        // Verify constant values are as expected
        assert_eq!(VTERM_KEY_NONE, 0);
        assert_eq!(VTERM_KEY_ENTER, 1);
        assert_eq!(VTERM_KEY_TAB, 2);
        assert_eq!(VTERM_KEY_KP_0, 16);
        assert_eq!(VTERM_KEY_KP_9, 25);
        assert_eq!(VTERM_KEY_MAX, 36);
    }
}
