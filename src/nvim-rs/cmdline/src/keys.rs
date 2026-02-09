//! Command line key handling
//!
//! This module provides key classification and dispatch logic for
//! command-line mode, including special keys, control combinations,
//! and completion triggers.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Key Code Constants
// =============================================================================
//
// Key codes use TERMCAP2KEY(a, b) = -((a) + ((int)(b) << 8))
// where KS_EXTRA = 253

/// Convert termcap codes to key code
const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -(a + (b << 8))
}

const KS_EXTRA: c_int = 253;

// KE_* values from keycodes.h enum key_extra
const KE_S_UP: c_int = 4;
const KE_S_DOWN: c_int = 5;
const KE_LEFTMOUSE: c_int = 44;
const KE_LEFTDRAG: c_int = 45;
const KE_LEFTRELEASE: c_int = 46;
const KE_MIDDLEMOUSE: c_int = 47;
const KE_MIDDLEDRAG: c_int = 48;
const KE_MIDDLERELEASE: c_int = 49;
const KE_RIGHTMOUSE: c_int = 50;
const KE_RIGHTDRAG: c_int = 51;
const KE_RIGHTRELEASE: c_int = 52;
const KE_TAB: c_int = 54;
const KE_XEND: c_int = 61;
const KE_XHOME: c_int = 63;
const KE_LEFTMOUSE_NM: c_int = 69;
const KE_LEFTRELEASE_NM: c_int = 70;
const KE_MOUSEDOWN: c_int = 75;
const KE_MOUSEUP: c_int = 76;
const KE_MOUSELEFT: c_int = 77;
const KE_MOUSERIGHT: c_int = 78;
const KE_KDEL: c_int = 80;
const KE_C_LEFT: c_int = 85;
const KE_C_RIGHT: c_int = 86;
const KE_WILD: c_int = 108;

// Arrow keys: TERMCAP2KEY('k', 'u'), etc.
const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
const K_LEFT: c_int = termcap2key(b'k' as c_int, b'l' as c_int);
const K_RIGHT: c_int = termcap2key(b'k' as c_int, b'r' as c_int);

// Shifted arrow keys
const K_S_UP: c_int = termcap2key(KS_EXTRA, KE_S_UP);
const K_S_DOWN: c_int = termcap2key(KS_EXTRA, KE_S_DOWN);
const K_S_LEFT: c_int = termcap2key(b'#' as c_int, b'4' as c_int);
const K_S_RIGHT: c_int = termcap2key(b'%' as c_int, b'i' as c_int);

// Control arrow keys
const K_C_LEFT: c_int = termcap2key(KS_EXTRA, KE_C_LEFT);
const K_C_RIGHT: c_int = termcap2key(KS_EXTRA, KE_C_RIGHT);

// Home/End
const K_HOME: c_int = termcap2key(b'k' as c_int, b'h' as c_int);
const K_END: c_int = termcap2key(b'@' as c_int, b'7' as c_int);
const K_XHOME: c_int = termcap2key(KS_EXTRA, KE_XHOME);
const K_XEND: c_int = termcap2key(KS_EXTRA, KE_XEND);
const K_S_HOME: c_int = termcap2key(b'#' as c_int, b'2' as c_int);
const K_S_END: c_int = termcap2key(b'*' as c_int, b'7' as c_int);

// Delete/Insert
const K_DEL: c_int = termcap2key(b'k' as c_int, b'D' as c_int);
const K_INS: c_int = termcap2key(b'k' as c_int, b'I' as c_int);
const K_KDEL: c_int = termcap2key(KS_EXTRA, KE_KDEL);

// Page Up/Down
const K_PAGEUP: c_int = termcap2key(b'k' as c_int, b'P' as c_int);
const K_PAGEDOWN: c_int = termcap2key(b'k' as c_int, b'N' as c_int);
const K_KPAGEUP: c_int = termcap2key(b'K' as c_int, b'3' as c_int);
const K_KPAGEDOWN: c_int = termcap2key(b'K' as c_int, b'5' as c_int);

// Enter keys
const K_KENTER: c_int = termcap2key(b'K' as c_int, b'A' as c_int);

// Tab
const K_TAB: c_int = termcap2key(KS_EXTRA, KE_TAB);
const K_S_TAB: c_int = termcap2key(b'k' as c_int, b'B' as c_int);

// Mouse events
const K_LEFTMOUSE: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE);
const K_LEFTMOUSE_NM: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE_NM);
const K_LEFTRELEASE: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE);
const K_LEFTRELEASE_NM: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE_NM);
const K_MIDDLEMOUSE: c_int = termcap2key(KS_EXTRA, KE_MIDDLEMOUSE);
const K_MIDDLERELEASE: c_int = termcap2key(KS_EXTRA, KE_MIDDLERELEASE);
const K_RIGHTMOUSE: c_int = termcap2key(KS_EXTRA, KE_RIGHTMOUSE);
const K_RIGHTRELEASE: c_int = termcap2key(KS_EXTRA, KE_RIGHTRELEASE);
const K_LEFTDRAG: c_int = termcap2key(KS_EXTRA, KE_LEFTDRAG);
const K_RIGHTDRAG: c_int = termcap2key(KS_EXTRA, KE_RIGHTDRAG);
const K_MIDDLEDRAG: c_int = termcap2key(KS_EXTRA, KE_MIDDLEDRAG);
const K_MOUSEDOWN: c_int = termcap2key(KS_EXTRA, KE_MOUSEDOWN);
const K_MOUSEUP: c_int = termcap2key(KS_EXTRA, KE_MOUSEUP);
const K_MOUSELEFT: c_int = termcap2key(KS_EXTRA, KE_MOUSELEFT);
const K_MOUSERIGHT: c_int = termcap2key(KS_EXTRA, KE_MOUSERIGHT);

// Wildcard
const K_WILD: c_int = termcap2key(KS_EXTRA, KE_WILD);

// Control characters
const CTRL_A: c_int = 1;
const CTRL_B: c_int = 2;
const CTRL_C: c_int = 3;
const CTRL_D: c_int = 4;
const CTRL_E: c_int = 5;
const CTRL_F: c_int = 6;
const CTRL_G: c_int = 7;
// Note: CTRL_H = 8 = BS (backspace)
// Note: CTRL_I = 9 = TAB
// Note: CTRL_J = 10 = NL (newline)
const CTRL_K: c_int = 11;
const CTRL_L: c_int = 12;
// Note: CTRL_M = 13 = CR (carriage return)
const CTRL_N: c_int = 14;
const CTRL_O: c_int = 15;
const CTRL_P: c_int = 16;
const CTRL_Q: c_int = 17;
const CTRL_R: c_int = 18;
const CTRL_S: c_int = 19;
const CTRL_T: c_int = 20;
const CTRL_U: c_int = 21;
const CTRL_V: c_int = 22;
const CTRL_W: c_int = 23;
const CTRL_X: c_int = 24;
const CTRL_Y: c_int = 25;
const CTRL_Z: c_int = 26;

const ESC: c_int = 27;
const BS: c_int = 8;
const TAB: c_int = 9;
const NL: c_int = 10;
const CR: c_int = 13;

// =============================================================================
// Key Classification
// =============================================================================

/// Category of key for command-line mode
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCategory {
    /// Navigation key (arrows, home/end)
    Navigation = 0,
    /// Editing key (backspace, delete)
    Editing = 1,
    /// History navigation key
    History = 2,
    /// Completion trigger key
    Completion = 3,
    /// Enter key (execute)
    Enter = 4,
    /// Escape key (cancel)
    Escape = 5,
    /// Mouse event
    Mouse = 6,
    /// Control key combination
    Control = 7,
    /// Regular character
    Character = 8,
    /// Unknown/unhandled
    Unknown = 9,
}

/// Navigation direction for cursor movement
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDirection {
    /// Move left
    Left = 0,
    /// Move right
    Right = 1,
    /// Move to start
    Home = 2,
    /// Move to end
    End = 3,
    /// Move up (history older)
    Up = 4,
    /// Move down (history newer)
    Down = 5,
    /// Page up
    PageUp = 6,
    /// Page down
    PageDown = 7,
}

/// Classify a key into a category.
///
/// Note: Some key constants have the same value (e.g., TAB=CTRL_I=9, BS=CTRL_H=8).
/// The classification is by value, so TAB and CTRL_I are treated identically.
#[must_use]
pub const fn classify_key(key: c_int) -> KeyCategory {
    match key {
        // Navigation keys (special keys with negative values)
        K_LEFT | K_RIGHT | K_S_LEFT | K_S_RIGHT | K_C_LEFT | K_C_RIGHT => KeyCategory::Navigation,
        K_HOME | K_END | K_XHOME | K_XEND | K_S_HOME | K_S_END => KeyCategory::Navigation,

        // Editing keys (BS=CTRL_H=8)
        BS | K_DEL | K_KDEL => KeyCategory::Editing,
        CTRL_U | CTRL_W | CTRL_K => KeyCategory::Editing,

        // History keys
        K_UP | K_DOWN | K_S_UP | K_S_DOWN | CTRL_P | CTRL_N => KeyCategory::History,
        K_PAGEUP | K_PAGEDOWN | K_KPAGEUP | K_KPAGEDOWN => KeyCategory::History,

        // Completion keys (TAB=CTRL_I=9)
        TAB | K_TAB | K_WILD => KeyCategory::Completion,
        CTRL_D | CTRL_L => KeyCategory::Completion,

        // Enter keys (NL=CTRL_J=10)
        CR | NL | K_KENTER => KeyCategory::Enter,

        // Escape
        ESC | CTRL_C => KeyCategory::Escape,

        // Mouse events
        K_LEFTMOUSE | K_LEFTMOUSE_NM | K_LEFTRELEASE | K_LEFTRELEASE_NM | K_MIDDLEMOUSE
        | K_MIDDLERELEASE | K_RIGHTMOUSE | K_RIGHTRELEASE => KeyCategory::Mouse,
        K_LEFTDRAG | K_RIGHTDRAG | K_MIDDLEDRAG => KeyCategory::Mouse,
        K_MOUSEDOWN | K_MOUSEUP | K_MOUSELEFT | K_MOUSERIGHT => KeyCategory::Mouse,

        // Other control keys (excluding those used above: 8, 9, 10 which are BS, TAB, NL)
        CTRL_A | CTRL_B | CTRL_E | CTRL_F | CTRL_G | CTRL_O | CTRL_Q | CTRL_R | CTRL_S | CTRL_T
        | CTRL_V | CTRL_X | CTRL_Y | CTRL_Z => KeyCategory::Control,

        // Regular printable characters
        32..=126 => KeyCategory::Character,

        // Extended ASCII and above
        128.. => KeyCategory::Character,

        _ => KeyCategory::Unknown,
    }
}

/// Get the navigation direction for a key.
#[must_use]
pub const fn get_nav_direction(key: c_int) -> Option<NavDirection> {
    match key {
        K_LEFT | CTRL_B => Some(NavDirection::Left),
        K_RIGHT | CTRL_F => Some(NavDirection::Right),
        K_HOME | CTRL_A => Some(NavDirection::Home),
        K_END | CTRL_E => Some(NavDirection::End),
        K_UP | CTRL_P | K_S_UP => Some(NavDirection::Up),
        K_DOWN | CTRL_N | K_S_DOWN => Some(NavDirection::Down),
        K_PAGEUP | K_KPAGEUP => Some(NavDirection::PageUp),
        K_PAGEDOWN | K_KPAGEDOWN => Some(NavDirection::PageDown),
        _ => None,
    }
}

/// Check if a key is a word navigation key (Ctrl+arrow or Shift+arrow).
#[must_use]
pub const fn is_word_nav_key(key: c_int) -> bool {
    matches!(key, K_S_LEFT | K_S_RIGHT | K_C_LEFT | K_C_RIGHT)
}

/// Check if a key triggers completion.
/// Note: TAB and CTRL_I are the same value (9).
#[must_use]
pub const fn is_completion_key(key: c_int) -> bool {
    matches!(key, TAB | K_TAB | K_WILD | CTRL_D | CTRL_L)
}

/// Check if a key is a history navigation key.
#[must_use]
pub const fn is_history_key(key: c_int) -> bool {
    matches!(
        key,
        K_UP | K_DOWN
            | K_S_UP
            | K_S_DOWN
            | CTRL_P
            | CTRL_N
            | K_PAGEUP
            | K_PAGEDOWN
            | K_KPAGEUP
            | K_KPAGEDOWN
    )
}

/// Check if a key uses prefix matching for history.
#[must_use]
pub const fn history_uses_prefix(key: c_int) -> bool {
    matches!(key, K_UP | K_DOWN)
}

/// Check if a key navigates to newer history (down direction).
#[must_use]
pub const fn is_history_newer_key(key: c_int) -> bool {
    matches!(key, K_DOWN | K_S_DOWN | CTRL_N | K_PAGEDOWN | K_KPAGEDOWN)
}

/// Check if a key is an enter/execute key.
/// Note: NL and CTRL_J are the same value (10).
#[must_use]
pub const fn is_enter_key(key: c_int) -> bool {
    matches!(key, CR | NL | K_KENTER)
}

/// Check if a key is an escape/cancel key.
#[must_use]
pub const fn is_escape_key(key: c_int) -> bool {
    matches!(key, ESC | CTRL_C)
}

/// Check if a key is a mouse event.
#[must_use]
pub const fn is_mouse_event(key: c_int) -> bool {
    matches!(
        key,
        K_LEFTMOUSE
            | K_LEFTMOUSE_NM
            | K_LEFTRELEASE
            | K_LEFTRELEASE_NM
            | K_MIDDLEMOUSE
            | K_MIDDLERELEASE
            | K_RIGHTMOUSE
            | K_RIGHTRELEASE
            | K_LEFTDRAG
            | K_RIGHTDRAG
            | K_MIDDLEDRAG
            | K_MOUSEDOWN
            | K_MOUSEUP
            | K_MOUSELEFT
            | K_MOUSERIGHT
    )
}

/// Check if a key is a drag event.
#[must_use]
pub const fn is_drag_event(key: c_int) -> bool {
    matches!(key, K_LEFTDRAG | K_RIGHTDRAG | K_MIDDLEDRAG)
}

/// Check if a key is a release event.
#[must_use]
pub const fn is_release_event(key: c_int) -> bool {
    matches!(
        key,
        K_LEFTRELEASE | K_LEFTRELEASE_NM | K_MIDDLERELEASE | K_RIGHTRELEASE
    )
}

/// Check if a key is a control key (not Ctrl+letter).
#[must_use]
pub const fn is_control_char(key: c_int) -> bool {
    key >= 1 && key <= 26
}

/// Check if a key is a printable ASCII character.
#[must_use]
pub const fn is_printable_ascii(key: c_int) -> bool {
    key >= 32 && key <= 126
}

/// Check if a key is a backspace.
/// Note: BS and CTRL_H are the same value (8).
#[must_use]
pub const fn is_backspace(key: c_int) -> bool {
    key == BS
}

/// Check if a key is a delete key.
#[must_use]
pub const fn is_delete(key: c_int) -> bool {
    key == K_DEL
}

/// Check if a key is insert (toggle overstrike mode).
#[must_use]
pub const fn is_insert(key: c_int) -> bool {
    key == K_INS
}

/// Check if key invokes register insert (Ctrl-R).
#[must_use]
pub const fn is_register_insert(key: c_int) -> bool {
    key == CTRL_R
}

/// Check if key invokes literal insert (Ctrl-V/Ctrl-Q).
#[must_use]
pub const fn is_literal_insert(key: c_int) -> bool {
    key == CTRL_V || key == CTRL_Q
}

// =============================================================================
// Key Dispatch Helpers
// =============================================================================

/// Invert horizontal movements for RTL command line.
///
/// When cmdmsg_rl is set, left/right keys are swapped for RTL display.
#[must_use]
pub const fn invert_rtl_key(key: c_int) -> c_int {
    match key {
        K_RIGHT => K_LEFT,
        K_S_RIGHT => K_S_LEFT,
        K_C_RIGHT => K_C_LEFT,
        K_LEFT => K_RIGHT,
        K_S_LEFT => K_S_RIGHT,
        K_C_LEFT => K_C_RIGHT,
        _ => key,
    }
}

/// Check if key should end the wildmenu.
///
/// Returns true if the key is not a wildcard/completion navigation key.
#[must_use]
pub const fn should_end_wildmenu(key: c_int, p_wc: c_int, p_wcm: c_int) -> bool {
    // Key is not wildchar or wildcharm
    let not_wc = key != p_wc && key != p_wcm && key != CTRL_Z;

    // Key is not a completion navigation key
    let not_nav = key != CTRL_N && key != CTRL_P && key != CTRL_A && key != CTRL_L;

    not_wc && not_nav
}

/// Check if key should end wildmenu for pum.
///
/// PageUp/PageDown are allowed in popup menu.
#[must_use]
pub const fn should_end_wildmenu_pum(key: c_int) -> bool {
    key != K_PAGEDOWN && key != K_PAGEUP && key != K_KPAGEDOWN && key != K_KPAGEUP
}

/// Check if key triggers CmdlineLeavePre autocmd.
#[must_use]
pub const fn triggers_cmdline_leave_pre(key: c_int) -> bool {
    key == b'\n' as c_int || key == b'\r' as c_int || key == K_KENTER || key == ESC || key == CTRL_C
}

/// Check if key should free history lookfor string.
///
/// Returns true if the key is NOT a history navigation or completion key.
#[must_use]
pub const fn should_free_lookfor(key: c_int) -> bool {
    !matches!(
        key,
        K_S_DOWN
            | K_S_UP
            | K_DOWN
            | K_UP
            | K_PAGEDOWN
            | K_PAGEUP
            | K_KPAGEDOWN
            | K_KPAGEUP
            | K_LEFT
            | K_RIGHT
    )
}

/// Check if S-Tab should be converted to Ctrl-P for completion.
#[must_use]
pub const fn is_stab_to_ctrl_p(key: c_int, p_wc: c_int) -> bool {
    key != p_wc && key == K_S_TAB
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Classify a key into a category (FFI).
#[no_mangle]
pub extern "C" fn rs_classify_cmdline_key(key: c_int) -> c_int {
    classify_key(key) as c_int
}

/// Get navigation direction for a key (FFI).
///
/// Returns -1 if not a navigation key.
#[no_mangle]
pub extern "C" fn rs_get_nav_direction(key: c_int) -> c_int {
    get_nav_direction(key).map_or(-1, |dir| dir as c_int)
}

/// Check if key is a word navigation key (FFI).
#[no_mangle]
pub extern "C" fn rs_is_word_nav_key(key: c_int) -> c_int {
    c_int::from(is_word_nav_key(key))
}

/// Check if key triggers completion (FFI).
#[no_mangle]
pub extern "C" fn rs_is_completion_key(key: c_int) -> c_int {
    c_int::from(is_completion_key(key))
}

/// Check if key is a history key (FFI).
#[no_mangle]
pub extern "C" fn rs_is_cmdline_history_key(key: c_int) -> c_int {
    c_int::from(is_history_key(key))
}

/// Check if key uses prefix matching for history (FFI).
#[no_mangle]
pub extern "C" fn rs_history_uses_prefix(key: c_int) -> c_int {
    c_int::from(history_uses_prefix(key))
}

/// Check if key is enter/execute (FFI).
#[no_mangle]
pub extern "C" fn rs_is_enter_key(key: c_int) -> c_int {
    c_int::from(is_enter_key(key))
}

/// Check if key is escape/cancel (FFI).
#[no_mangle]
pub extern "C" fn rs_is_escape_key(key: c_int) -> c_int {
    c_int::from(is_escape_key(key))
}

/// Check if key is a mouse event (FFI).
#[no_mangle]
pub extern "C" fn rs_is_mouse_event(key: c_int) -> c_int {
    c_int::from(is_mouse_event(key))
}

/// Check if key is a drag event (FFI).
#[no_mangle]
pub extern "C" fn rs_is_drag_event(key: c_int) -> c_int {
    c_int::from(is_drag_event(key))
}

/// Check if key is a release event (FFI).
#[no_mangle]
pub extern "C" fn rs_is_release_event(key: c_int) -> c_int {
    c_int::from(is_release_event(key))
}

/// Check if key is a control character (FFI).
#[no_mangle]
pub extern "C" fn rs_is_control_char(key: c_int) -> c_int {
    c_int::from(is_control_char(key))
}

/// Check if key is backspace (FFI).
#[no_mangle]
pub extern "C" fn rs_is_backspace(key: c_int) -> c_int {
    c_int::from(is_backspace(key))
}

/// Check if key is delete (FFI).
#[no_mangle]
pub extern "C" fn rs_is_delete_key(key: c_int) -> c_int {
    c_int::from(is_delete(key))
}

/// Check if key is insert (FFI).
#[no_mangle]
pub extern "C" fn rs_is_insert_key(key: c_int) -> c_int {
    c_int::from(is_insert(key))
}

/// Check if key is register insert Ctrl-R (FFI).
#[no_mangle]
pub extern "C" fn rs_is_register_insert(key: c_int) -> c_int {
    c_int::from(is_register_insert(key))
}

/// Check if key is literal insert Ctrl-V/Q (FFI).
#[no_mangle]
pub extern "C" fn rs_is_literal_insert(key: c_int) -> c_int {
    c_int::from(is_literal_insert(key))
}

/// Invert horizontal movements for RTL command line (FFI).
#[no_mangle]
pub extern "C" fn rs_invert_rtl_key(key: c_int) -> c_int {
    invert_rtl_key(key)
}

/// Check if key should end wildmenu (FFI).
#[no_mangle]
pub extern "C" fn rs_should_end_wildmenu(key: c_int, p_wc: c_int, p_wcm: c_int) -> c_int {
    c_int::from(should_end_wildmenu(key, p_wc, p_wcm))
}

/// Check if key should end wildmenu for pum (FFI).
#[no_mangle]
pub extern "C" fn rs_should_end_wildmenu_pum(key: c_int) -> c_int {
    c_int::from(should_end_wildmenu_pum(key))
}

/// Check if key triggers CmdlineLeavePre autocmd (FFI).
#[no_mangle]
pub extern "C" fn rs_triggers_cmdline_leave_pre(key: c_int) -> c_int {
    c_int::from(triggers_cmdline_leave_pre(key))
}

/// Check if key should free history lookfor string (FFI).
#[no_mangle]
pub extern "C" fn rs_should_free_lookfor(key: c_int) -> c_int {
    c_int::from(should_free_lookfor(key))
}

/// Check if S-Tab should be converted to Ctrl-P for completion (FFI).
#[no_mangle]
pub extern "C" fn rs_is_stab_to_ctrl_p(key: c_int, p_wc: c_int) -> c_int {
    c_int::from(is_stab_to_ctrl_p(key, p_wc))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_classification() {
        assert_eq!(classify_key(K_LEFT), KeyCategory::Navigation);
        assert_eq!(classify_key(K_RIGHT), KeyCategory::Navigation);
        assert_eq!(classify_key(BS), KeyCategory::Editing);
        assert_eq!(classify_key(K_UP), KeyCategory::History);
        assert_eq!(classify_key(TAB), KeyCategory::Completion);
        assert_eq!(classify_key(CR), KeyCategory::Enter);
        assert_eq!(classify_key(ESC), KeyCategory::Escape);
        assert_eq!(classify_key(K_LEFTMOUSE), KeyCategory::Mouse);
        assert_eq!(classify_key(CTRL_R), KeyCategory::Control);
        assert_eq!(classify_key(c_int::from(b'a')), KeyCategory::Character);
    }

    #[test]
    fn test_nav_direction() {
        assert_eq!(get_nav_direction(K_LEFT), Some(NavDirection::Left));
        assert_eq!(get_nav_direction(K_RIGHT), Some(NavDirection::Right));
        assert_eq!(get_nav_direction(K_HOME), Some(NavDirection::Home));
        assert_eq!(get_nav_direction(K_END), Some(NavDirection::End));
        assert_eq!(get_nav_direction(K_UP), Some(NavDirection::Up));
        assert_eq!(get_nav_direction(K_DOWN), Some(NavDirection::Down));
        assert_eq!(get_nav_direction(c_int::from(b'a')), None);
    }

    #[test]
    fn test_word_nav() {
        assert!(is_word_nav_key(K_S_LEFT));
        assert!(is_word_nav_key(K_S_RIGHT));
        assert!(is_word_nav_key(K_C_LEFT));
        assert!(is_word_nav_key(K_C_RIGHT));
        assert!(!is_word_nav_key(K_LEFT));
    }

    #[test]
    fn test_completion_keys() {
        assert!(is_completion_key(TAB));
        assert!(is_completion_key(CTRL_D));
        assert!(is_completion_key(CTRL_L));
        assert!(!is_completion_key(K_LEFT));
    }

    #[test]
    fn test_history_keys() {
        assert!(is_history_key(K_UP));
        assert!(is_history_key(K_DOWN));
        assert!(is_history_key(CTRL_P));
        assert!(is_history_key(CTRL_N));
        assert!(!is_history_key(K_LEFT));
    }

    #[test]
    fn test_history_prefix() {
        assert!(history_uses_prefix(K_UP));
        assert!(history_uses_prefix(K_DOWN));
        assert!(!history_uses_prefix(K_PAGEUP));
        assert!(!history_uses_prefix(CTRL_P));
    }

    #[test]
    fn test_enter_escape() {
        assert!(is_enter_key(CR));
        assert!(is_enter_key(NL));
        assert!(is_escape_key(ESC));
        assert!(is_escape_key(CTRL_C));
    }

    #[test]
    fn test_mouse_events() {
        assert!(is_mouse_event(K_LEFTMOUSE));
        assert!(is_drag_event(K_LEFTDRAG));
        assert!(is_release_event(K_LEFTRELEASE));
    }

    #[test]
    fn test_backspace_delete() {
        assert!(is_backspace(BS));
        // Note: CTRL_H = 8 = BS, so testing BS tests CTRL_H
        assert!(is_delete(K_DEL));
        assert!(!is_backspace(K_DEL));
    }

    #[test]
    fn test_insert_keys() {
        assert!(is_insert(K_INS));
        assert!(is_register_insert(CTRL_R));
        assert!(is_literal_insert(CTRL_V));
        assert!(is_literal_insert(CTRL_Q));
    }

    #[test]
    fn test_control_chars() {
        assert!(is_control_char(CTRL_A));
        assert!(is_control_char(CTRL_Z));
        assert!(!is_control_char(ESC));
        assert!(!is_control_char(c_int::from(b'a')));
    }
}
