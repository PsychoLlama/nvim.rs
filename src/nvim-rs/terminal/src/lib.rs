//! Terminal emulator utilities for Neovim
//!
//! This crate provides Rust implementations for terminal-related functions,
//! primarily working with the libvterm-based terminal emulator.
//!
//! Re-exports vterm types for terminal emulation.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;
use std::os::raw::c_void;

// Re-export vterm types that don't conflict with existing definitions
// The terminal crate already has its own VTermRect, VTermPos, and modifier constants
pub mod vterm {
    pub use nvim_vterm::{
        decode_dec_drawing,
        decode_usascii,
        encode_button,
        encode_key,
        encode_move,
        encode_unichar,
        // Pen colors
        lookup_colour,
        lookup_colour_palette,
        lookup_keycode,
        parse_sgr_param,
        // Parser types
        CsiParserState,
        // Encoding
        Encoding,
        EncodingType,
        // Keyboard encoding
        KeyOutput,
        // Mouse encoding
        MouseOutput,
        // State types (use prefixed names to avoid conflicts)
        MouseProtocol as VTermMouseProtocol,
        MouseState,
        OscParserState,
        ParserState,
        Pen,
        SavedModes,
        // Screen buffer types
        Screen,
        ScreenCell,
        ScreenPen,
        SelectionState,
        TerminalModes,
        Utf8Decoder,
        VTermKey,
        // Mouse flags
        MOUSE_WANT_CLICK,
        MOUSE_WANT_DRAG,
        MOUSE_WANT_MOVE,
        UNICODE_INVALID,
    };
}

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

// =============================================================================
// VTerm Cursor Shape Constants (from vterm.h)
// =============================================================================

/// Block cursor shape.
pub const VTERM_PROP_CURSORSHAPE_BLOCK: c_int = 1;
/// Underline cursor shape.
pub const VTERM_PROP_CURSORSHAPE_UNDERLINE: c_int = 2;
/// Vertical bar cursor shape (left side).
pub const VTERM_PROP_CURSORSHAPE_BAR_LEFT: c_int = 3;

// =============================================================================
// Screen Damage and Invalidation Helpers
// =============================================================================

/// Result of an invalid region calculation.
#[repr(C)]
pub struct InvalidRegion {
    /// Start row of the invalid region.
    pub start_row: c_int,
    /// End row of the invalid region (exclusive).
    pub end_row: c_int,
}

/// Calculate the updated invalid region when damage occurs.
///
/// This computes the union of the current invalid region and the new damage.
/// Pass -1 for both current values to indicate the entire screen is invalid.
///
/// # Arguments
/// * `current_start` - Current invalid start row (-1 for full invalidation)
/// * `current_end` - Current invalid end row (-1 for full invalidation)
/// * `damage_start` - New damage start row
/// * `damage_end` - New damage end row
///
/// # Returns
/// The updated invalid region.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_update_invalid_region(
    current_start: c_int,
    current_end: c_int,
    damage_start: c_int,
    damage_end: c_int,
) -> InvalidRegion {
    // If requesting full invalidation
    if damage_start == -1 && damage_end == -1 {
        return InvalidRegion {
            start_row: current_start,
            end_row: current_end,
        };
    }

    // Compute union of regions
    let start = if current_start == -1 || damage_start < current_start {
        damage_start
    } else {
        current_start
    };

    let end = if current_end == -1 || damage_end > current_end {
        damage_end
    } else {
        current_end
    };

    InvalidRegion {
        start_row: start,
        end_row: end,
    }
}

/// Reset invalid region to indicate no pending damage.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_reset_invalid_region() -> InvalidRegion {
    InvalidRegion {
        start_row: i32::MAX,
        end_row: -1,
    }
}

// =============================================================================
// Resize Calculation Helpers
// =============================================================================

/// Result of terminal resize dimension calculation.
#[repr(C)]
pub struct ResizeDimensions {
    /// Calculated width (0 if no resize needed or invalid).
    pub width: u16,
    /// Calculated height (0 if no resize needed or invalid).
    pub height: u16,
    /// Whether a resize is needed.
    pub needs_resize: c_int,
}

/// Calculate terminal dimensions by taking the maximum of current and new values.
///
/// This is used when determining terminal size across multiple windows.
///
/// # Arguments
/// * `current_width` - Current accumulated width
/// * `current_height` - Current accumulated height
/// * `new_width` - Width of the new window
/// * `new_height` - Height of the new window
///
/// # Returns
/// The maximum dimensions.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_max_dimensions(
    current_width: u16,
    current_height: u16,
    new_width: u16,
    new_height: u16,
) -> ResizeDimensions {
    ResizeDimensions {
        width: current_width.max(new_width),
        height: current_height.max(new_height),
        needs_resize: 0, // Not used in this context
    }
}

/// Check if terminal needs resize based on current and target dimensions.
///
/// # Arguments
/// * `cur_width` - Current terminal width
/// * `cur_height` - Current terminal height
/// * `target_width` - Target width
/// * `target_height` - Target height
///
/// # Returns
/// `ResizeDimensions` with `needs_resize` set to 1 if resize is needed.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_check_resize(
    cur_width: c_int,
    cur_height: c_int,
    target_width: u16,
    target_height: u16,
) -> ResizeDimensions {
    // No resize needed if dimensions match or target is zero
    if target_width == 0
        || target_height == 0
        || (cur_width == c_int::from(target_width) && cur_height == c_int::from(target_height))
    {
        return ResizeDimensions {
            width: 0,
            height: 0,
            needs_resize: 0,
        };
    }

    ResizeDimensions {
        width: target_width,
        height: target_height,
        needs_resize: 1,
    }
}

// =============================================================================
// Scrollback Calculation Helpers
// =============================================================================

/// Maximum scrollback size constant.
pub const TERMINAL_SB_MAX: usize = 100_000;

/// Calculate the effective scrollback size.
///
/// If the provided size is less than 1 (typically -1 for "unlimited"),
/// returns the maximum scrollback size.
///
/// # Arguments
/// * `scrollback` - The requested scrollback size
///
/// # Returns
/// The effective scrollback size.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_effective_scrollback(scrollback: i64) -> usize {
    if scrollback < 1 {
        TERMINAL_SB_MAX
    } else {
        // Safe: we've verified scrollback >= 1 above
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let size = scrollback as usize;
        size
    }
}

/// Calculate how many scrollback lines to delete when reducing scrollback size.
///
/// # Arguments
/// * `current_sb` - Current number of scrollback lines
/// * `new_size` - New scrollback size limit
///
/// # Returns
/// Number of lines to delete (0 if none needed).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_scrollback_lines_to_delete(
    current_sb: usize,
    new_size: usize,
) -> usize {
    current_sb.saturating_sub(new_size)
}

/// Check if scrollback buffer is full and needs to wrap.
///
/// # Arguments
/// * `current_sb` - Current number of scrollback lines
/// * `sb_size` - Maximum scrollback size
///
/// # Returns
/// 1 if scrollback is full, 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_scrollback_is_full(current_sb: usize, sb_size: usize) -> c_int {
    c_int::from(current_sb == sb_size)
}

/// Calculate the buffer index for inserting a scrollback line.
///
/// # Arguments
/// * `line_count` - Total lines in buffer
/// * `height` - Terminal height
///
/// # Returns
/// The buffer index where the scrollback line should be inserted.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_scrollback_insert_index(line_count: c_int, height: c_int) -> c_int {
    line_count - height
}

// =============================================================================
// VTerm Callback Helper Types
// =============================================================================

/// `VTerm` rectangle structure (matches `VTermRect` from libvterm).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VTermRect {
    /// Start row (inclusive).
    pub start_row: c_int,
    /// End row (exclusive).
    pub end_row: c_int,
    /// Start column (inclusive).
    pub start_col: c_int,
    /// End column (exclusive).
    pub end_col: c_int,
}

/// `VTerm` position structure (matches `VTermPos` from libvterm).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VTermPos {
    /// Row position.
    pub row: c_int,
    /// Column position.
    pub col: c_int,
}

/// `VTerm` property constants.
pub const VTERM_PROP_ALTSCREEN: c_int = 1;
/// Cursor visible property.
pub const VTERM_PROP_CURSORVISIBLE: c_int = 2;
/// Title property.
pub const VTERM_PROP_TITLE: c_int = 3;
/// Icon name property.
pub const VTERM_PROP_ICONNAME: c_int = 4;
/// Reverse property.
pub const VTERM_PROP_REVERSE: c_int = 5;
/// Cursor shape property.
pub const VTERM_PROP_CURSORSHAPE: c_int = 6;
/// Mouse property.
pub const VTERM_PROP_MOUSE: c_int = 7;
/// Cursor blink property.
pub const VTERM_PROP_CURSORBLINK: c_int = 8;

// =============================================================================
// VTerm Callback Helpers
// =============================================================================

/// Calculate combined damage region from two rectangles.
///
/// Used in `term_moverect` callback to compute the union of source and
/// destination rectangles.
///
/// # Arguments
/// * `dest` - Destination rectangle
/// * `src` - Source rectangle
///
/// # Returns
/// The combined damage region (`start_row`, `end_row`).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_moverect_damage(dest: VTermRect, src: VTermRect) -> InvalidRegion {
    let start = dest.start_row.min(src.start_row);
    let end = dest.end_row.max(src.end_row);
    InvalidRegion {
        start_row: start,
        end_row: end,
    }
}

/// Result of processing a `VTerm` property change.
#[repr(C)]
pub struct VTermPropResult {
    /// Whether the property was handled.
    pub handled: c_int,
    /// Whether the terminal should be invalidated.
    pub invalidate: c_int,
    /// Whether cursor pending flag should be set.
    pub cursor_pending: c_int,
}

/// Check if a `VTerm` property change requires terminal invalidation.
///
/// # Arguments
/// * `prop` - The `VTerm` property that changed
///
/// # Returns
/// A `VTermPropResult` indicating how to handle the property change.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_prop_needs_invalidate(prop: c_int) -> VTermPropResult {
    match prop {
        // Properties that are handled but don't need invalidation
        VTERM_PROP_ALTSCREEN | VTERM_PROP_TITLE | VTERM_PROP_ICONNAME | VTERM_PROP_MOUSE => {
            VTermPropResult {
                handled: 1,
                invalidate: 0,
                cursor_pending: 0,
            }
        }
        // Cursor visibility needs invalidation but no cursor pending
        VTERM_PROP_CURSORVISIBLE => VTermPropResult {
            handled: 1,
            invalidate: 1,
            cursor_pending: 0,
        },
        // Cursor shape/blink needs both invalidation and cursor pending
        VTERM_PROP_CURSORBLINK | VTERM_PROP_CURSORSHAPE => VTermPropResult {
            handled: 1,
            invalidate: 1,
            cursor_pending: 1,
        },
        // Unknown properties
        _ => VTermPropResult {
            handled: 0,
            invalidate: 0,
            cursor_pending: 0,
        },
    }
}

/// Process a cursor move event from `VTerm`.
///
/// # Arguments
/// * `new_row` - New cursor row
/// * `new_col` - New cursor column
/// * `old_row` - Previous cursor row
/// * `old_col` - Previous cursor column
///
/// # Returns
/// 1 to indicate the callback was handled.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_movecursor_handled(
    _new_row: c_int,
    _new_col: c_int,
    _old_row: c_int,
    _old_col: c_int,
) -> c_int {
    // The actual cursor update is done in C by writing to term->cursor.row/col
    // This function just provides a hook point for potential future logic
    1
}

/// Calculate the number of columns to copy when popping scrollback.
///
/// # Arguments
/// * `requested_cols` - Number of columns requested
/// * `available_cols` - Number of columns available in scrollback row
///
/// # Returns
/// The number of columns to actually copy.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_sb_pop_cols(requested_cols: usize, available_cols: usize) -> usize {
    requested_cols.min(available_cols)
}

/// Check if dark theme should be reported to `VTerm`.
///
/// # Arguments
/// * `bg_char` - The background option character ('d' for dark, 'l' for light)
///
/// # Returns
/// 1 if dark theme, 0 if light theme.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_is_dark_theme(bg_char: u8) -> c_int {
    c_int::from(bg_char == b'd')
}

// =============================================================================
// Row/Line Number Conversion
// =============================================================================

/// Convert a terminal row number to a buffer line number.
///
/// Formula: `linenr = row + sb_current + 1`
///
/// The terminal has a scrollback buffer at the top of the nvim buffer.
/// Row 0 of the terminal is at line `sb_current + 1` in the buffer.
///
/// # Arguments
/// * `row` - Terminal row (0-based, can be negative for scrollback)
/// * `sb_current` - Current scrollback buffer size
///
/// # Returns
/// Buffer line number (1-based).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_terminal_row_to_linenr(row: c_int, sb_current: usize) -> c_int {
    if row == i32::MAX {
        return i32::MAX;
    }
    // Safe cast: sb_current is typically small (max ~100000)
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    let sb = sb_current as c_int;
    row + sb + 1
}

/// Convert a buffer line number to a terminal row number.
///
/// Formula: `row = linenr - sb_current - 1`
///
/// # Arguments
/// * `linenr` - Buffer line number (1-based)
/// * `sb_current` - Current scrollback buffer size
///
/// # Returns
/// Terminal row (0-based, negative for scrollback lines).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_terminal_linenr_to_row(linenr: c_int, sb_current: usize) -> c_int {
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    let sb = sb_current as c_int;
    linenr - sb - 1
}

// =============================================================================
// Mouse Button Conversion
// =============================================================================

// Mouse key constants (from keycodes.h)
const K_LEFTMOUSE: c_int = termcap2key(KS_EXTRA, 39);
const K_LEFTDRAG: c_int = termcap2key(KS_EXTRA, 40);
const K_LEFTRELEASE: c_int = termcap2key(KS_EXTRA, 41);
const K_MIDDLEMOUSE: c_int = termcap2key(KS_EXTRA, 42);
const K_MIDDLEDRAG: c_int = termcap2key(KS_EXTRA, 43);
const K_MIDDLERELEASE: c_int = termcap2key(KS_EXTRA, 44);
const K_RIGHTMOUSE: c_int = termcap2key(KS_EXTRA, 45);
const K_RIGHTDRAG: c_int = termcap2key(KS_EXTRA, 46);
const K_RIGHTRELEASE: c_int = termcap2key(KS_EXTRA, 47);
const K_MOUSEDOWN: c_int = termcap2key(KS_EXTRA, 54);
const K_MOUSEUP: c_int = termcap2key(KS_EXTRA, 55);
const K_MOUSELEFT: c_int = termcap2key(KS_EXTRA, 56);
const K_MOUSERIGHT: c_int = termcap2key(KS_EXTRA, 57);
const K_MOUSEMOVE: c_int = termcap2key(KS_EXTRA, 74);
const K_X1MOUSE: c_int = termcap2key(KS_EXTRA, 75);
const K_X1DRAG: c_int = termcap2key(KS_EXTRA, 76);
const K_X1RELEASE: c_int = termcap2key(KS_EXTRA, 77);
const K_X2MOUSE: c_int = termcap2key(KS_EXTRA, 78);
const K_X2DRAG: c_int = termcap2key(KS_EXTRA, 79);
const K_X2RELEASE: c_int = termcap2key(KS_EXTRA, 80);

/// Result of mouse button conversion.
#[repr(C)]
pub struct MouseButtonResult {
    /// Button number (1=left, 2=middle, 3=right, 4=scroll down, 5=scroll up, etc.)
    /// -1 if unknown key
    pub button: c_int,
    /// 1 if pressed/dragging, 0 if released
    pub pressed: c_int,
}

/// Convert a Neovim mouse key code to a `VTerm` button number.
///
/// This handles the conversion of mouse events for forwarding to the terminal.
///
/// # Arguments
/// * `key` - Neovim key code (`K_LEFTMOUSE`, `K_RIGHTDRAG`, etc.)
///
/// # Returns
/// `MouseButtonResult` with button number and pressed state.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_terminal_convert_mouse_button(key: c_int) -> MouseButtonResult {
    match key {
        K_LEFTDRAG | K_LEFTMOUSE => MouseButtonResult {
            button: 1,
            pressed: 1,
        },
        K_LEFTRELEASE => MouseButtonResult {
            button: 1,
            pressed: 0,
        },
        K_MIDDLEDRAG | K_MIDDLEMOUSE => MouseButtonResult {
            button: 2,
            pressed: 1,
        },
        K_MIDDLERELEASE => MouseButtonResult {
            button: 2,
            pressed: 0,
        },
        K_RIGHTDRAG | K_RIGHTMOUSE => MouseButtonResult {
            button: 3,
            pressed: 1,
        },
        K_RIGHTRELEASE => MouseButtonResult {
            button: 3,
            pressed: 0,
        },
        K_X1DRAG | K_X1MOUSE => MouseButtonResult {
            button: 8,
            pressed: 1,
        },
        K_X1RELEASE => MouseButtonResult {
            button: 8,
            pressed: 0,
        },
        K_X2DRAG | K_X2MOUSE => MouseButtonResult {
            button: 9,
            pressed: 1,
        },
        K_X2RELEASE => MouseButtonResult {
            button: 9,
            pressed: 0,
        },
        K_MOUSEDOWN => MouseButtonResult {
            button: 4,
            pressed: 1,
        },
        K_MOUSEUP => MouseButtonResult {
            button: 5,
            pressed: 1,
        },
        K_MOUSELEFT => MouseButtonResult {
            button: 7,
            pressed: 1,
        },
        K_MOUSERIGHT => MouseButtonResult {
            button: 6,
            pressed: 1,
        },
        K_MOUSEMOVE => MouseButtonResult {
            button: 0,
            pressed: 0,
        },
        _ => MouseButtonResult {
            button: -1,
            pressed: 0,
        },
    }
}

// =============================================================================
// Terminal Paste Filter (TPF) Flags
// =============================================================================

/// Filter backspace characters (0x08)
pub const TPF_BS: c_int = 0x001;
/// Filter horizontal tab characters (0x09)
pub const TPF_HT: c_int = 0x002;
/// Filter form feed characters (0x0C)
pub const TPF_FF: c_int = 0x004;
/// Filter escape characters (0x1B)
pub const TPF_ESC: c_int = 0x008;
/// Filter DEL characters (0x7F)
pub const TPF_DEL: c_int = 0x010;
/// Filter C0 control characters (0x01-0x1F, except specific ones)
pub const TPF_C0: c_int = 0x020;
/// Filter C1 control characters (0x80-0x9F)
pub const TPF_C1: c_int = 0x040;

/// Check if a character should be filtered when pasting to terminal.
///
/// This implements the 'termpastefilter' option logic. Certain control
/// characters can be filtered out when pasting to prevent security issues.
///
/// # Arguments
/// * `c` - Character to check
/// * `tpf_flags` - Current 'termpastefilter' flag settings
///
/// # Returns
/// 1 if the character should be filtered, 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_terminal_should_filter_char(c: c_int, tpf_flags: c_int) -> c_int {
    let flag = match c {
        0x08 => TPF_BS,
        0x09 => TPF_HT,
        // 0x0A (LF) and 0x0D (CR) are never filtered
        0x0A | 0x0D => return 0,
        0x0C => TPF_FF,
        0x1B => TPF_ESC,
        0x7F => TPF_DEL,
        _ if c > 0 && c < 0x20 => TPF_C0,
        _ if (0x80..=0x9F).contains(&c) => TPF_C1,
        _ => return 0,
    };
    c_int::from((tpf_flags & flag) != 0)
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

    #[test]
    fn test_cursor_shape_constants() {
        assert_eq!(VTERM_PROP_CURSORSHAPE_BLOCK, 1);
        assert_eq!(VTERM_PROP_CURSORSHAPE_UNDERLINE, 2);
        assert_eq!(VTERM_PROP_CURSORSHAPE_BAR_LEFT, 3);
    }

    #[test]
    fn test_update_invalid_region() {
        // First damage - use large initial values
        let region = rs_terminal_update_invalid_region(i32::MAX, -1, 5, 10);
        assert_eq!(region.start_row, 5);
        assert_eq!(region.end_row, 10);

        // Extend region with larger damage
        let region = rs_terminal_update_invalid_region(5, 10, 3, 15);
        assert_eq!(region.start_row, 3);
        assert_eq!(region.end_row, 15);

        // Damage within existing region
        let region = rs_terminal_update_invalid_region(3, 15, 5, 10);
        assert_eq!(region.start_row, 3);
        assert_eq!(region.end_row, 15);

        // Full invalidation request
        let region = rs_terminal_update_invalid_region(3, 15, -1, -1);
        assert_eq!(region.start_row, 3);
        assert_eq!(region.end_row, 15);
    }

    #[test]
    fn test_reset_invalid_region() {
        let region = rs_terminal_reset_invalid_region();
        assert_eq!(region.start_row, i32::MAX);
        assert_eq!(region.end_row, -1);
    }

    #[test]
    fn test_max_dimensions() {
        let dims = rs_terminal_max_dimensions(80, 24, 120, 30);
        assert_eq!(dims.width, 120);
        assert_eq!(dims.height, 30);

        let dims = rs_terminal_max_dimensions(120, 30, 80, 24);
        assert_eq!(dims.width, 120);
        assert_eq!(dims.height, 30);

        let dims = rs_terminal_max_dimensions(0, 0, 80, 24);
        assert_eq!(dims.width, 80);
        assert_eq!(dims.height, 24);
    }

    #[test]
    fn test_check_resize() {
        // Need resize - different dimensions
        let dims = rs_terminal_check_resize(80, 24, 120, 30);
        assert_eq!(dims.needs_resize, 1);
        assert_eq!(dims.width, 120);
        assert_eq!(dims.height, 30);

        // No resize needed - same dimensions
        let dims = rs_terminal_check_resize(80, 24, 80, 24);
        assert_eq!(dims.needs_resize, 0);

        // No resize - zero target
        let dims = rs_terminal_check_resize(80, 24, 0, 30);
        assert_eq!(dims.needs_resize, 0);

        let dims = rs_terminal_check_resize(80, 24, 80, 0);
        assert_eq!(dims.needs_resize, 0);
    }

    #[test]
    fn test_effective_scrollback() {
        assert_eq!(rs_terminal_effective_scrollback(-1), TERMINAL_SB_MAX);
        assert_eq!(rs_terminal_effective_scrollback(0), TERMINAL_SB_MAX);
        assert_eq!(rs_terminal_effective_scrollback(1000), 1000);
        assert_eq!(rs_terminal_effective_scrollback(50000), 50000);
    }

    #[test]
    fn test_scrollback_lines_to_delete() {
        // Need to delete lines
        assert_eq!(rs_terminal_scrollback_lines_to_delete(1000, 500), 500);
        assert_eq!(rs_terminal_scrollback_lines_to_delete(100, 50), 50);

        // No deletion needed
        assert_eq!(rs_terminal_scrollback_lines_to_delete(500, 1000), 0);
        assert_eq!(rs_terminal_scrollback_lines_to_delete(500, 500), 0);
    }

    #[test]
    fn test_scrollback_is_full() {
        assert_eq!(rs_terminal_scrollback_is_full(1000, 1000), 1);
        assert_eq!(rs_terminal_scrollback_is_full(500, 1000), 0);
        assert_eq!(rs_terminal_scrollback_is_full(0, 1000), 0);
    }

    #[test]
    fn test_scrollback_insert_index() {
        assert_eq!(rs_terminal_scrollback_insert_index(100, 24), 76);
        assert_eq!(rs_terminal_scrollback_insert_index(50, 24), 26);
        assert_eq!(rs_terminal_scrollback_insert_index(24, 24), 0);
    }

    #[test]
    fn test_vterm_rect_default() {
        let rect = VTermRect::default();
        assert_eq!(rect.start_row, 0);
        assert_eq!(rect.end_row, 0);
        assert_eq!(rect.start_col, 0);
        assert_eq!(rect.end_col, 0);
    }

    #[test]
    fn test_vterm_pos_default() {
        let pos = VTermPos::default();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 0);
    }

    #[test]
    fn test_moverect_damage() {
        // Destination before source
        let dest = VTermRect {
            start_row: 0,
            end_row: 5,
            start_col: 0,
            end_col: 80,
        };
        let src = VTermRect {
            start_row: 5,
            end_row: 10,
            start_col: 0,
            end_col: 80,
        };
        let region = rs_terminal_moverect_damage(dest, src);
        assert_eq!(region.start_row, 0);
        assert_eq!(region.end_row, 10);

        // Source before destination
        let dest = VTermRect {
            start_row: 10,
            end_row: 20,
            start_col: 0,
            end_col: 80,
        };
        let src = VTermRect {
            start_row: 5,
            end_row: 15,
            start_col: 0,
            end_col: 80,
        };
        let region = rs_terminal_moverect_damage(dest, src);
        assert_eq!(region.start_row, 5);
        assert_eq!(region.end_row, 20);
    }

    #[test]
    fn test_prop_needs_invalidate() {
        // Altscreen - handled but no invalidation
        let result = rs_terminal_prop_needs_invalidate(VTERM_PROP_ALTSCREEN);
        assert_eq!(result.handled, 1);
        assert_eq!(result.invalidate, 0);
        assert_eq!(result.cursor_pending, 0);

        // Cursor visible - invalidates
        let result = rs_terminal_prop_needs_invalidate(VTERM_PROP_CURSORVISIBLE);
        assert_eq!(result.handled, 1);
        assert_eq!(result.invalidate, 1);
        assert_eq!(result.cursor_pending, 0);

        // Cursor blink - invalidates and sets cursor pending
        let result = rs_terminal_prop_needs_invalidate(VTERM_PROP_CURSORBLINK);
        assert_eq!(result.handled, 1);
        assert_eq!(result.invalidate, 1);
        assert_eq!(result.cursor_pending, 1);

        // Cursor shape - invalidates and sets cursor pending
        let result = rs_terminal_prop_needs_invalidate(VTERM_PROP_CURSORSHAPE);
        assert_eq!(result.handled, 1);
        assert_eq!(result.invalidate, 1);
        assert_eq!(result.cursor_pending, 1);

        // Title - handled but no invalidation
        let result = rs_terminal_prop_needs_invalidate(VTERM_PROP_TITLE);
        assert_eq!(result.handled, 1);
        assert_eq!(result.invalidate, 0);

        // Unknown property
        let result = rs_terminal_prop_needs_invalidate(99);
        assert_eq!(result.handled, 0);
        assert_eq!(result.invalidate, 0);
    }

    #[test]
    fn test_movecursor_handled() {
        assert_eq!(rs_terminal_movecursor_handled(5, 10, 0, 0), 1);
        assert_eq!(rs_terminal_movecursor_handled(0, 0, 5, 10), 1);
    }

    #[test]
    fn test_sb_pop_cols() {
        assert_eq!(rs_terminal_sb_pop_cols(80, 100), 80);
        assert_eq!(rs_terminal_sb_pop_cols(100, 80), 80);
        assert_eq!(rs_terminal_sb_pop_cols(80, 80), 80);
    }

    #[test]
    fn test_is_dark_theme() {
        assert_eq!(rs_terminal_is_dark_theme(b'd'), 1);
        assert_eq!(rs_terminal_is_dark_theme(b'l'), 0);
        assert_eq!(rs_terminal_is_dark_theme(b'x'), 0);
    }

    #[test]
    fn test_vterm_prop_constants() {
        assert_eq!(VTERM_PROP_ALTSCREEN, 1);
        assert_eq!(VTERM_PROP_CURSORVISIBLE, 2);
        assert_eq!(VTERM_PROP_TITLE, 3);
        assert_eq!(VTERM_PROP_CURSORSHAPE, 6);
        assert_eq!(VTERM_PROP_MOUSE, 7);
        assert_eq!(VTERM_PROP_CURSORBLINK, 8);
    }

    #[test]
    fn test_row_to_linenr() {
        // sb_current = 10, row = 0 => linenr = 11
        assert_eq!(rs_terminal_row_to_linenr(0, 10), 11);
        // sb_current = 0, row = 0 => linenr = 1
        assert_eq!(rs_terminal_row_to_linenr(0, 0), 1);
        // sb_current = 5, row = 3 => linenr = 9
        assert_eq!(rs_terminal_row_to_linenr(3, 5), 9);
        // INT_MAX stays INT_MAX
        assert_eq!(rs_terminal_row_to_linenr(i32::MAX, 10), i32::MAX);
    }

    #[test]
    fn test_linenr_to_row() {
        // sb_current = 10, linenr = 11 => row = 0
        assert_eq!(rs_terminal_linenr_to_row(11, 10), 0);
        // sb_current = 0, linenr = 1 => row = 0
        assert_eq!(rs_terminal_linenr_to_row(1, 0), 0);
        // sb_current = 5, linenr = 9 => row = 3
        assert_eq!(rs_terminal_linenr_to_row(9, 5), 3);
    }

    #[test]
    fn test_mouse_button_conversion() {
        // Left mouse
        let result = rs_terminal_convert_mouse_button(K_LEFTMOUSE);
        assert_eq!(result.button, 1);
        assert_eq!(result.pressed, 1);

        let result = rs_terminal_convert_mouse_button(K_LEFTRELEASE);
        assert_eq!(result.button, 1);
        assert_eq!(result.pressed, 0);

        let result = rs_terminal_convert_mouse_button(K_LEFTDRAG);
        assert_eq!(result.button, 1);
        assert_eq!(result.pressed, 1);

        // Middle mouse
        let result = rs_terminal_convert_mouse_button(K_MIDDLEMOUSE);
        assert_eq!(result.button, 2);
        assert_eq!(result.pressed, 1);

        // Right mouse
        let result = rs_terminal_convert_mouse_button(K_RIGHTMOUSE);
        assert_eq!(result.button, 3);
        assert_eq!(result.pressed, 1);

        // Scroll
        let result = rs_terminal_convert_mouse_button(K_MOUSEDOWN);
        assert_eq!(result.button, 4);
        assert_eq!(result.pressed, 1);

        let result = rs_terminal_convert_mouse_button(K_MOUSEUP);
        assert_eq!(result.button, 5);
        assert_eq!(result.pressed, 1);

        // Mouse move
        let result = rs_terminal_convert_mouse_button(K_MOUSEMOVE);
        assert_eq!(result.button, 0);
        assert_eq!(result.pressed, 0);

        // Unknown key
        let result = rs_terminal_convert_mouse_button(0);
        assert_eq!(result.button, -1);
    }

    #[test]
    fn test_filter_char_detailed() {
        // Test various filter flags
        assert_eq!(rs_terminal_should_filter_char(0x08, TPF_BS), 1); // Backspace
        assert_eq!(rs_terminal_should_filter_char(0x08, 0), 0); // Backspace without flag
        assert_eq!(rs_terminal_should_filter_char(0x09, TPF_HT), 1); // Tab
        assert_eq!(rs_terminal_should_filter_char(0x0C, TPF_FF), 1); // Form feed
        assert_eq!(rs_terminal_should_filter_char(0x1B, TPF_ESC), 1); // Escape
        assert_eq!(rs_terminal_should_filter_char(0x7F, TPF_DEL), 1); // DEL

        // C0 control characters (0x01-0x1F excluding specific ones)
        assert_eq!(rs_terminal_should_filter_char(0x01, TPF_C0), 1);
        assert_eq!(rs_terminal_should_filter_char(0x1F, TPF_C0), 1);

        // C1 control characters (0x80-0x9F)
        assert_eq!(rs_terminal_should_filter_char(0x80, TPF_C1), 1);
        assert_eq!(rs_terminal_should_filter_char(0x9F, TPF_C1), 1);

        // Normal characters shouldn't be filtered
        assert_eq!(rs_terminal_should_filter_char(c_int::from(b'a'), 0xFFFF), 0);
        assert_eq!(rs_terminal_should_filter_char(c_int::from(b' '), 0xFFFF), 0);

        // Newline and carriage return are never filtered
        assert_eq!(rs_terminal_should_filter_char(0x0A, 0xFFFF), 0);
        assert_eq!(rs_terminal_should_filter_char(0x0D, 0xFFFF), 0);
    }
}
