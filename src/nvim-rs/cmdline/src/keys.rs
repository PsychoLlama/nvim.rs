//! Command line key handling
//!
//! This module provides key classification and dispatch logic for
//! command-line mode, including special keys, control combinations,
//! and completion triggers.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int, c_void};

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
const KE_NOP: c_int = 97;
const KE_EVENT: c_int = 102;
const KE_LUA: c_int = 103;
const KE_COMMAND: c_int = 104;
const KS_ZERO: c_int = 255;

// Arrow keys: TERMCAP2KEY('k', 'u'), etc.
pub const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
pub const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
pub const K_LEFT: c_int = termcap2key(b'k' as c_int, b'l' as c_int);
pub const K_RIGHT: c_int = termcap2key(b'k' as c_int, b'r' as c_int);

// Shifted arrow keys
pub const K_S_UP: c_int = termcap2key(KS_EXTRA, KE_S_UP);
pub const K_S_DOWN: c_int = termcap2key(KS_EXTRA, KE_S_DOWN);
pub const K_S_LEFT: c_int = termcap2key(b'#' as c_int, b'4' as c_int);
pub const K_S_RIGHT: c_int = termcap2key(b'%' as c_int, b'i' as c_int);

// Control arrow keys
pub const K_C_LEFT: c_int = termcap2key(KS_EXTRA, KE_C_LEFT);
pub const K_C_RIGHT: c_int = termcap2key(KS_EXTRA, KE_C_RIGHT);

// Home/End
pub const K_HOME: c_int = termcap2key(b'k' as c_int, b'h' as c_int);
pub const K_END: c_int = termcap2key(b'@' as c_int, b'7' as c_int);
pub const K_XHOME: c_int = termcap2key(KS_EXTRA, KE_XHOME);
pub const K_XEND: c_int = termcap2key(KS_EXTRA, KE_XEND);
pub const K_S_HOME: c_int = termcap2key(b'#' as c_int, b'2' as c_int);
pub const K_S_END: c_int = termcap2key(b'*' as c_int, b'7' as c_int);

// Delete/Insert
pub const K_DEL: c_int = termcap2key(b'k' as c_int, b'D' as c_int);
pub const K_INS: c_int = termcap2key(b'k' as c_int, b'I' as c_int);
pub const K_KDEL: c_int = termcap2key(KS_EXTRA, KE_KDEL);

// Page Up/Down
pub const K_PAGEUP: c_int = termcap2key(b'k' as c_int, b'P' as c_int);
pub const K_PAGEDOWN: c_int = termcap2key(b'k' as c_int, b'N' as c_int);
pub const K_KPAGEUP: c_int = termcap2key(b'K' as c_int, b'3' as c_int);
pub const K_KPAGEDOWN: c_int = termcap2key(b'K' as c_int, b'5' as c_int);

// Enter keys
pub const K_KENTER: c_int = termcap2key(b'K' as c_int, b'A' as c_int);

// Tab
pub const K_TAB: c_int = termcap2key(KS_EXTRA, KE_TAB);
pub const K_S_TAB: c_int = termcap2key(b'k' as c_int, b'B' as c_int);

// Mouse events
pub const K_LEFTMOUSE: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE);
pub const K_LEFTMOUSE_NM: c_int = termcap2key(KS_EXTRA, KE_LEFTMOUSE_NM);
pub const K_LEFTRELEASE: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE);
pub const K_LEFTRELEASE_NM: c_int = termcap2key(KS_EXTRA, KE_LEFTRELEASE_NM);
pub const K_MIDDLEMOUSE: c_int = termcap2key(KS_EXTRA, KE_MIDDLEMOUSE);
pub const K_MIDDLERELEASE: c_int = termcap2key(KS_EXTRA, KE_MIDDLERELEASE);
pub const K_RIGHTMOUSE: c_int = termcap2key(KS_EXTRA, KE_RIGHTMOUSE);
pub const K_RIGHTRELEASE: c_int = termcap2key(KS_EXTRA, KE_RIGHTRELEASE);
pub const K_LEFTDRAG: c_int = termcap2key(KS_EXTRA, KE_LEFTDRAG);
pub const K_RIGHTDRAG: c_int = termcap2key(KS_EXTRA, KE_RIGHTDRAG);
pub const K_MIDDLEDRAG: c_int = termcap2key(KS_EXTRA, KE_MIDDLEDRAG);
pub const K_MOUSEDOWN: c_int = termcap2key(KS_EXTRA, KE_MOUSEDOWN);
pub const K_MOUSEUP: c_int = termcap2key(KS_EXTRA, KE_MOUSEUP);
pub const K_MOUSELEFT: c_int = termcap2key(KS_EXTRA, KE_MOUSELEFT);
pub const K_MOUSERIGHT: c_int = termcap2key(KS_EXTRA, KE_MOUSERIGHT);

// Wildcard and special keys (K_IGNORE, K_CMDWIN defined in extended block below)
pub const K_WILD: c_int = termcap2key(KS_EXTRA, KE_WILD);
pub const K_NOP: c_int = termcap2key(KS_EXTRA, KE_NOP);
pub const K_EVENT: c_int = termcap2key(KS_EXTRA, KE_EVENT);
pub const K_COMMAND: c_int = termcap2key(KS_EXTRA, KE_COMMAND);
pub const K_LUA: c_int = termcap2key(KS_EXTRA, KE_LUA);

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
// Phase 5: command_line_handle_ctrl_bsl, command_line_insert_reg,
//          command_line_erase_chars
// =============================================================================

// Return codes matching C enum in ex_getln.c
const CMDLINE_NOT_CHANGED: c_int = 1;
const CMDLINE_CHANGED: c_int = 2;
const GOTO_NORMAL_MODE: c_int = 3;
const PROCESS_NEXT_KEY: c_int = 4;

unsafe extern "C" {
    static Rows: c_int;
    static Columns: c_int;
    static mut got_int: bool;
    // Getters/setters for globals needed by these functions
    fn nvim_get_ccline_cmdfirstc() -> c_int;
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_set_ccline_cmdpos(pos: c_int);
    fn nvim_set_ccline_cmdlen(len: c_int);
    fn nvim_get_ccline_cmdprompt() -> *mut c_char;
    fn nvim_set_ccline_special_char(c: c_int);
    fn nvim_get_cmdline_star_count() -> c_int;
    fn nvim_get_new_cmdpos() -> c_int;
    fn nvim_set_new_cmdpos(val: c_int);
    fn nvim_get_key_typed_cmdline() -> c_int;
    fn nvim_set_key_typed(val: c_int);
    fn nvim_inc_textlock();
    fn nvim_dec_textlock();
    fn nvim_get_no_mapping() -> c_int;
    fn nvim_set_no_mapping(val: c_int);
    fn nvim_get_allow_keys() -> c_int;
    fn nvim_set_allow_keys(val: c_int);
    fn nvim_set_did_emsg(val: c_int);
    fn nvim_set_emsg_on_display(val: c_int);
    fn nvim_get_exmode_active() -> bool;
    fn nvim_get_cmd_silent() -> c_int;
    fn nvim_set_redraw_cmdline(val: bool);
    fn ui_has(what: c_int) -> c_int;
    fn nvim_set_msg_col(val: c_int);
    fn msg_putchar(c: c_int);
    fn realloc_cmdbuff(len: c_int) -> c_int;
    fn nvim_strcpy_cmdbuff(src: *const c_char);
    fn dealloc_cmdbuff();
    fn nvim_plain_vgetc_wrapper() -> c_int;
    fn vungetc(c: c_int);
    fn get_expr_register() -> c_int;
    fn get_expr_line() -> *mut c_char;
    fn xfree(ptr: *mut c_char);
    fn beep_flush();
    fn nvim_cmdline_paste(regname: c_int, literally: bool, remcr: bool) -> bool;
    fn aborting() -> c_int;
    fn rs_is_literal_register(regname: c_int) -> c_int;
    fn mb_off_next(base: *const c_char, p: *const c_char) -> c_int;
    fn putcmdline(c: c_char, shift: bool);
}

const K_UI_CMDLINE: c_int = 24; // kUICmdline

/// Handle CTRL-\ in command-line mode.
/// Rust replacement for `command_line_handle_ctrl_bsl(CommandLineState *s)`.
/// The C wrapper passes `&s->c` and `&s->gotesc`.
///
/// # Safety
/// `c_ptr` and `gotesc_ptr` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_command_line_handle_ctrl_bsl(
    c_ptr: *mut c_int,
    gotesc_ptr: *mut bool,
) -> c_int {
    nvim_set_no_mapping(nvim_get_no_mapping() + 1);
    nvim_set_allow_keys(nvim_get_allow_keys() + 1);
    let c = nvim_plain_vgetc_wrapper();
    *c_ptr = c;
    nvim_set_no_mapping(nvim_get_no_mapping() - 1);
    nvim_set_allow_keys(nvim_get_allow_keys() - 1);

    // CTRL-\ e doesn't work when obtaining an expression, unless it is in a mapping.
    if c != CTRL_N
        && c != CTRL_G
        && (c != b'e' as c_int
            || (nvim_get_ccline_cmdfirstc() == b'=' as c_int && nvim_get_key_typed_cmdline() != 0)
            || nvim_get_cmdline_star_count() > 0)
    {
        vungetc(c);
        return PROCESS_NEXT_KEY;
    }

    if c == b'e' as c_int {
        // Replace command line with result of an expression.
        let cmdpos = nvim_get_ccline_cmdpos();
        let cmdlen = nvim_get_ccline_cmdlen();
        if cmdpos == cmdlen {
            nvim_set_new_cmdpos(99999); // keep at end
        } else {
            nvim_set_new_cmdpos(cmdpos);
        }

        let expr_reg = get_expr_register();
        *c_ptr = expr_reg;
        if expr_reg == b'=' as c_int {
            // Evaluate the expression. Set textlock to avoid nasty things.
            nvim_inc_textlock();
            let p = get_expr_line();
            nvim_dec_textlock();

            if !p.is_null() {
                let mut len: c_int = 0;
                while *p.add(len as usize) != 0 {
                    len += 1;
                }
                realloc_cmdbuff(len + 1);
                nvim_set_ccline_cmdlen(len);
                nvim_strcpy_cmdbuff(p);
                xfree(p);

                // Restore cursor or use the position from set_cmdline_pos().
                let new_cmdpos = nvim_get_new_cmdpos();
                let new_pos = if new_cmdpos < nvim_get_ccline_cmdlen() {
                    new_cmdpos
                } else {
                    nvim_get_ccline_cmdlen()
                };
                nvim_set_ccline_cmdpos(new_pos);
                nvim_set_key_typed(0); // Don't do p_wc completion.
                crate::screen::redrawcmd_rs();
                return CMDLINE_CHANGED;
            }
        }
        beep_flush();
        unsafe {
            got_int = false;
        } // don't abandon the command line
        nvim_set_did_emsg(0);
        nvim_set_emsg_on_display(0);
        crate::screen::redrawcmd_rs();
        return CMDLINE_NOT_CHANGED;
    }

    *gotesc_ptr = true; // will free ccline.cmdbuff after putting it in history
    GOTO_NORMAL_MODE
}

/// Handle CTRL-R in command-line mode.
/// Rust replacement for `command_line_insert_reg(CommandLineState *s)`.
/// The C wrapper passes `&s->c` and `&s->gotesc`.
///
/// # Safety
/// `c_ptr` and `gotesc_ptr` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_command_line_insert_reg(
    c_ptr: *mut c_int,
    gotesc_ptr: *mut bool,
) -> c_int {
    let save_new_cmdpos = nvim_get_new_cmdpos();

    putcmdline(b'"' as c_char, true);
    nvim_set_no_mapping(nvim_get_no_mapping() + 1);
    nvim_set_allow_keys(nvim_get_allow_keys() + 1);
    let mut i = nvim_plain_vgetc_wrapper(); // CTRL-R <char>
    *c_ptr = i;
    if i == CTRL_O {
        i = CTRL_R; // CTRL-R CTRL-O == CTRL-R CTRL-R
    }
    if i == CTRL_R {
        *c_ptr = nvim_plain_vgetc_wrapper(); // CTRL-R CTRL-R <char>
    }
    nvim_set_no_mapping(nvim_get_no_mapping() - 1);
    nvim_set_allow_keys(nvim_get_allow_keys() - 1);

    // Insert the result of an expression.
    nvim_set_new_cmdpos(-1);
    let c = *c_ptr;
    if c == b'=' as c_int {
        if nvim_get_ccline_cmdfirstc() == b'=' as c_int // can't do this recursively
            || nvim_get_cmdline_star_count() > 0
        // or when typing a password
        {
            beep_flush();
            *c_ptr = ESC;
        } else {
            *c_ptr = get_expr_register();
        }
    }

    let mut literally = false;
    let c = *c_ptr;
    if c != ESC {
        // use ESC to cancel inserting register
        literally = i == CTRL_R || rs_is_literal_register(c) != 0;
        nvim_cmdline_paste(c, literally, false);

        // When there was a serious error, abort getting the command line.
        if aborting() != 0 {
            *gotesc_ptr = true; // will free ccline.cmdbuff after putting it in history
            return GOTO_NORMAL_MODE;
        }
        nvim_set_key_typed(0); // Don't do p_wc completion.
        let new_pos = nvim_get_new_cmdpos();
        if new_pos >= 0 {
            // set_cmdline_pos() was used
            let cmdlen = nvim_get_ccline_cmdlen();
            nvim_set_ccline_cmdpos(if new_pos < cmdlen { new_pos } else { cmdlen });
        }
    }
    nvim_set_new_cmdpos(save_new_cmdpos);

    // remove the double quote
    nvim_set_ccline_special_char(0);
    crate::screen::redrawcmd_rs();

    // With "literally": the command line has already changed.
    // Else: the text has been stuffed, but the command line didn't change yet.
    if literally {
        CMDLINE_CHANGED
    } else {
        CMDLINE_NOT_CHANGED
    }
}

/// Handle backspace, delete, and CTRL-W in command-line mode.
/// Rust replacement for `command_line_erase_chars(CommandLineState *s)`.
/// The C wrapper passes `s->c`, `s->indent`, and `&s->is_state`.
///
/// # Safety
/// `is_state` must be a valid non-null pointer to an IncsearchStateT.
#[allow(clippy::too_many_lines)]
#[no_mangle]
pub unsafe extern "C" fn rs_command_line_erase_chars(
    c: c_int,
    indent: c_int,
    is_state: *mut crate::search::IncsearchStateT,
) -> c_int {
    let mut c = c;
    if c == K_KDEL {
        c = K_DEL;
    }

    // Delete current character is the same as backspace on next
    // character, except at end of line
    let cmdpos = nvim_get_ccline_cmdpos();
    let cmdlen = nvim_get_ccline_cmdlen();
    if c == K_DEL && cmdpos != cmdlen {
        nvim_set_ccline_cmdpos(cmdpos + 1);
    }
    if c == K_DEL {
        let cmdbuff = nvim_get_ccline_cmdbuff();
        let cmdpos = nvim_get_ccline_cmdpos();
        let off = mb_off_next(cmdbuff, cmdbuff.add(cmdpos as usize));
        nvim_set_ccline_cmdpos(cmdpos + off);
    }

    let cmdpos = nvim_get_ccline_cmdpos();
    if cmdpos > 0 {
        let result = if c == CTRL_W {
            crate::edit::rs_cmdline_delete_word_before()
        } else {
            crate::edit::rs_cmdline_delete_char_before()
        };

        if result > 0 {
            // Line was changed
            if nvim_get_ccline_cmdlen() == 0 {
                (*is_state).search_start = (*is_state).save_cursor;
                // save view settings, so that the screen won't be restored at the wrong position
                (*is_state).old_viewstate = (*is_state).init_viewstate;
            }
            crate::screen::redrawcmd_rs();
            return CMDLINE_CHANGED;
        }
        return CMDLINE_NOT_CHANGED;
    } else if nvim_get_ccline_cmdlen() == 0
        && c != CTRL_W
        && nvim_get_ccline_cmdprompt().is_null()
        && indent == 0
    {
        // In ex and debug mode it doesn't make sense to return.
        if nvim_get_exmode_active() || nvim_get_ccline_cmdfirstc() == b'>' as c_int {
            return CMDLINE_NOT_CHANGED;
        }

        dealloc_cmdbuff(); // no commandline to return

        if nvim_get_cmd_silent() == 0 && ui_has(K_UI_CMDLINE) == 0 {
            nvim_set_msg_col(0);
            msg_putchar(b' ' as c_int); // delete ':'
        }
        (*is_state).search_start = (*is_state).save_cursor;
        nvim_set_redraw_cmdline(true);
        return GOTO_NORMAL_MODE;
    }
    CMDLINE_CHANGED
}

// =============================================================================
// Phase 3: command_line_handle_key
// =============================================================================

// Additional key constants needed for command_line_handle_key
const K_BS: c_int = termcap2key(b'k' as c_int, b'b' as c_int);
const CTRL_RSB: c_int = 29;
const CTRL_HAT: c_int = 30;
const CTRL_UU: c_int = 31; // Ctrl-_ (Ctrl-underscore)
const NUL: c_int = 0;
const FAIL: c_int = 0;

// Key extra codes needed
const KE_KINS: c_int = 79;
const KE_CMDWIN: c_int = 84;
const KE_X1MOUSE: c_int = 89;
const KE_X1DRAG: c_int = 90;
const KE_X1RELEASE: c_int = 91;
const KE_X2MOUSE: c_int = 92;
const KE_X2DRAG: c_int = 93;
const KE_X2RELEASE: c_int = 94;
const KE_MOUSEMOVE: c_int = 100;
const KE_IGNORE: c_int = 53;
const KS_SELECT: c_int = 245;
const KE_FILLER: c_int = 88; // 'X'

const K_KINS: c_int = termcap2key(KS_EXTRA, KE_KINS);
pub const K_CMDWIN: c_int = termcap2key(KS_EXTRA, KE_CMDWIN);
const K_X1MOUSE: c_int = termcap2key(KS_EXTRA, KE_X1MOUSE);
const K_X1DRAG: c_int = termcap2key(KS_EXTRA, KE_X1DRAG);
const K_X1RELEASE: c_int = termcap2key(KS_EXTRA, KE_X1RELEASE);
const K_X2MOUSE: c_int = termcap2key(KS_EXTRA, KE_X2MOUSE);
const K_X2DRAG: c_int = termcap2key(KS_EXTRA, KE_X2DRAG);
const K_X2RELEASE: c_int = termcap2key(KS_EXTRA, KE_X2RELEASE);
const K_MOUSEMOVE: c_int = termcap2key(KS_EXTRA, KE_MOUSEMOVE);
pub const K_IGNORE: c_int = termcap2key(KS_EXTRA, KE_IGNORE);
const K_SELECT: c_int = termcap2key(KS_SELECT, KE_FILLER);
pub const K_ZERO: c_int = termcap2key(KS_ZERO, KE_FILLER);

// Additional Home/End keys
const K_KHOME: c_int = termcap2key(b'K' as c_int, b'1' as c_int);
const K_KEND: c_int = termcap2key(b'K' as c_int, b'4' as c_int);
// K_S_HOME and K_S_END defined earlier in this file
const K_C_HOME: c_int = -22101; // TERMCAP2KEY(KS_EXTRA, KE_C_HOME)
const K_C_END: c_int = -22101 - 1; // placeholder, will use accessor

// Modifier masks
const MOD_MASK_SHIFT: c_int = 0x02;
const MOD_MASK_CTRL: c_int = 0x04;

// Wild expansion types
const WILD_ALL: c_int = 6;
const WILD_LONGEST: c_int = 7;
const WILD_NEXT: c_int = 4;
const WILD_PREV: c_int = 5;
const WILD_PAGEUP: c_int = 11;
const WILD_PAGEDOWN: c_int = 12;
const EXPAND_NOTHING: c_int = 0;

// IS_SPECIAL(c) macro: c < 0
#[inline]
const fn is_special_key(c: c_int) -> bool {
    c < 0
}

// ABBR_OFF constant
const ABBR_OFF: c_int = 0x100;

// kOptWimFlagNoselect
const K_OPT_WIM_FLAG_NOSELECT: u8 = 0x10;

// kUICmdline
const K_UI_CMDLINE_MAIN: c_int = 24;

unsafe extern "C" {
    // CommandLineState field accessors (only those used by rs_command_line_handle_key)
    fn nvim_cls_get_c(s: *mut c_void) -> c_int;
    fn nvim_cls_set_c(s: *mut c_void, val: c_int);
    fn nvim_cls_get_firstc(s: *mut c_void) -> c_int;
    fn nvim_cls_get_count(s: *mut c_void) -> c_int;
    fn nvim_cls_get_indent(s: *mut c_void) -> c_int;
    fn nvim_cls_get_gotesc(s: *mut c_void) -> c_int;
    fn nvim_cls_set_gotesc(s: *mut c_void, val: c_int);
    fn nvim_cls_get_do_abbr(s: *mut c_void) -> c_int;
    fn nvim_cls_set_do_abbr(s: *mut c_void, val: c_int);
    fn nvim_cls_get_ignore_drag_release(s: *mut c_void) -> c_int;
    fn nvim_cls_set_ignore_drag_release(s: *mut c_void, val: c_int);
    fn nvim_cls_set_did_hist_navigate(s: *mut c_void, val: c_int);
    fn nvim_cls_set_did_wild_list(s: *mut c_void, val: c_int);
    fn nvim_cls_get_is_state(s: *mut c_void) -> *mut c_void;
    fn nvim_cls_get_xpc(s: *mut c_void) -> *mut c_void;
    fn nvim_cls_get_xpc_numfiles(s: *mut c_void) -> c_int;
    fn nvim_cls_set_xpc_context(s: *mut c_void, val: c_int);
    fn nvim_cls_get_ccline_mouse_used() -> c_int;
    fn nvim_cls_set_ccline_mouse_used_val(val: c_int);

    // Wrapper functions for CLS operations
    fn nvim_command_line_not_changed(s: *mut c_void) -> c_int;
    fn nvim_command_line_changed(s: *mut c_void) -> c_int;
    fn nvim_command_line_toggle_langmap(s: *mut c_void);
    fn nvim_command_line_left_right_mouse(s: *mut c_void);
    fn nvim_command_line_browse_history(s: *mut c_void) -> c_int;
    fn nvim_cmdline_pum_cleanup();
    fn nvim_showmatches(
        xp: *mut c_void,
        display_wildmenu: bool,
        display_list: bool,
        noselect: bool,
    ) -> c_int;
    fn nvim_nextwild(xp: *mut c_void, wild_type: c_int, options: c_int, escape: bool) -> c_int;
    fn nvim_may_add_char_to_search(firstc: c_int, c: *mut c_int, is_state: *mut c_void) -> c_int;

    // Global accessors
    fn nvim_get_ccline_one_key() -> c_int;
    fn nvim_get_ccline_special_char() -> c_int;
    fn nvim_get_ccline_special_shift() -> c_int;
    fn nvim_set_ccline_overstrike(val: c_int);
    fn nvim_get_ccline_overstrike() -> c_int;
    fn nvim_get_mod_mask() -> c_int;
    fn nvim_set_mod_mask(val: c_int);
    fn nvim_get_iobuff() -> *mut c_char;
    fn nvim_get_mouse_row() -> c_int;
    fn nvim_get_cmdline_row() -> c_int;
    fn nvim_get_ex_normal_busy() -> c_int;
    fn nvim_get_getln_interrupted_highlight() -> c_int;
    fn nvim_set_getln_interrupted_highlight(val: c_int);
    fn nvim_get_typebuf_len() -> c_int;
    fn nvim_get_p_ari() -> c_int;
    fn nvim_ccline_cmdbuff_set_nul();
    fn nvim_get_wim_flags(idx: c_int) -> u8;

    // Screen position accessors (not in the earlier extern block)
    fn nvim_get_ccline_cmdspos() -> c_int;
    fn nvim_set_ccline_cmdspos(val: c_int);

    // Functions to call
    fn ui_cursor_shape();
    fn may_trigger_modechanged();
    fn status_redraw_curbuf();
    fn redraw_statuslines();
    // putcmdline(c, shift) declared in the earlier extern block in this file
    fn unputcmdline();
    fn nvim_get_literal_call(no_simplify: bool) -> c_int;
    fn nvim_get_digraph(flag: bool) -> c_int;
    fn nvim_utf_iscomposing_first(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn nvim_get_special_key_name(c: c_int, modifiers: c_int) -> *const c_char;
    fn vim_iswordc(c: c_int) -> bool;
    fn eval_has_provider(feat: *const c_char, throw_if_fast: bool) -> bool;
    fn cmdline_pum_active() -> c_int;
    fn cmd_screencol(bytepos: c_int) -> c_int;
    fn nvim_get_cedit_key() -> c_int;
    fn nvim_open_cmdwin() -> c_int;
}

// Inline helper to get cedit key
#[inline]
unsafe fn get_cedit_key() -> c_int {
    nvim_get_cedit_key()
}

/// Rust replacement for `command_line_handle_key(CommandLineState *s)`.
///
/// Handles a key press in command-line mode.
///
/// # Safety
///
/// `s` must be a valid non-null pointer to a `CommandLineState`.
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::similar_names)]
#[unsafe(export_name = "command_line_handle_key")]
pub unsafe extern "C" fn rs_command_line_handle_key(s: *mut c_void) -> c_int {
    let c = nvim_cls_get_c(s);
    let one_key = nvim_get_ccline_one_key() != 0;

    // For one key prompt, avoid putting ESC and Ctrl_C onto cmdline.
    // For all other keys, just put onto cmdline and exit.
    if one_key && c != ESC && c != CTRL_C {
        // goto end
        return handle_key_end(s, c);
    }

    // Big switch for a typed command line character.
    match c {
        // Backspace / Delete
        k if k == K_BS || k == BS || k == K_DEL || k == K_KDEL || k == CTRL_W => {
            let indent = nvim_cls_get_indent(s);
            let is_state = nvim_cls_get_is_state(s);
            match rs_command_line_erase_chars(c, indent, is_state.cast()) {
                x if x == CMDLINE_NOT_CHANGED => return nvim_command_line_not_changed(s),
                x if x == GOTO_NORMAL_MODE => return 0,
                _ => return nvim_command_line_changed(s),
            }
        }

        // Insert toggle
        k if k == K_INS || k == K_KINS => {
            let overstrike = nvim_get_ccline_overstrike();
            nvim_set_ccline_overstrike((overstrike == 0) as c_int);
            ui_cursor_shape();
            may_trigger_modechanged();
            status_redraw_curbuf();
            redraw_statuslines();
            return nvim_command_line_not_changed(s);
        }

        // CTRL-^: toggle language map
        x if x == CTRL_HAT => {
            nvim_command_line_toggle_langmap(s);
            return nvim_command_line_not_changed(s);
        }

        // CTRL-U: delete all chars left of cursor
        x if x == CTRL_U => {
            let j = nvim_get_ccline_cmdpos();
            let new_len = nvim_get_ccline_cmdlen() - j;
            nvim_set_ccline_cmdlen(new_len);
            let mut i = 0;
            let mut jj = j;
            let cmdbuff = nvim_get_ccline_cmdbuff();
            while i < new_len {
                *cmdbuff.add(i as usize) = *cmdbuff.add(jj as usize);
                i += 1;
                jj += 1;
            }
            // Truncate at end
            *cmdbuff.add(new_len as usize) = 0;
            nvim_set_ccline_cmdpos(0);
            if nvim_get_ccline_cmdlen() == 0 {
                let is_state = nvim_cls_get_is_state(s);
                let is_state_ptr: *mut crate::search::IncsearchStateT = is_state.cast();
                (*is_state_ptr).search_start = (*is_state_ptr).save_cursor;
            }
            crate::screen::redrawcmd_rs();
            return nvim_command_line_changed(s);
        }

        // ESC / CTRL-C: cancel command line
        x if x == ESC || x == CTRL_C => {
            let exmode = nvim_get_exmode_active();
            let ex_normal = nvim_get_ex_normal_busy();
            let typebuf_len = nvim_get_typebuf_len();
            let getln_interrupted = nvim_get_getln_interrupted_highlight();

            if (exmode && (ex_normal == 0 || typebuf_len > 0))
                || (getln_interrupted != 0 && c == CTRL_C)
            {
                nvim_set_getln_interrupted_highlight(0);
                return nvim_command_line_not_changed(s);
            }

            nvim_cls_set_gotesc(s, 1);
            return 0;
        }

        // CTRL-R: insert register
        x if x == CTRL_R => {
            let mut c_val = nvim_cls_get_c(s);
            let mut gotesc_val = (nvim_cls_get_gotesc(s) != 0) as bool;
            match rs_command_line_insert_reg(&raw mut c_val, &raw mut gotesc_val) {
                x if x == GOTO_NORMAL_MODE => {
                    nvim_cls_set_c(s, c_val);
                    nvim_cls_set_gotesc(s, gotesc_val as c_int);
                    return 0;
                }
                x if x == CMDLINE_CHANGED => {
                    nvim_cls_set_c(s, c_val);
                    nvim_cls_set_gotesc(s, gotesc_val as c_int);
                    return nvim_command_line_changed(s);
                }
                _ => {
                    nvim_cls_set_c(s, c_val);
                    nvim_cls_set_gotesc(s, gotesc_val as c_int);
                    return nvim_command_line_not_changed(s);
                }
            }
        }

        // CTRL-D: show matches
        x if x == CTRL_D => {
            let xp = nvim_cls_get_xpc(s);
            let wim_noselect = (nvim_get_wim_flags(0) & K_OPT_WIM_FLAG_NOSELECT) != 0;
            if nvim_showmatches(xp, false, true, wim_noselect) == EXPAND_NOTHING {
                // fall through to normal char handling
            } else {
                crate::screen::redrawcmd_rs();
                return 1; // don't do incremental search now
            }
        }

        // Arrow right / CTRL-right
        k if k == K_RIGHT || k == K_S_RIGHT || k == K_C_RIGHT => {
            loop {
                let cmdpos = nvim_get_ccline_cmdpos();
                let cmdlen = nvim_get_ccline_cmdlen();
                if cmdpos >= cmdlen {
                    break;
                }
                let cells = crate::screen::rs_cmdline_charsize(cmdpos);
                let cols = Columns;
                let rows = Rows;
                let cmdspos = nvim_get_ccline_cmdspos();
                if nvim_get_key_typed_cmdline() != 0 && cmdspos + cells >= cols * rows {
                    break;
                }
                nvim_set_ccline_cmdspos(cmdspos + cells);
                let cmdbuff = nvim_get_ccline_cmdbuff();
                let adv = utfc_ptr2len(cmdbuff.add(cmdpos as usize));
                nvim_set_ccline_cmdpos(cmdpos + adv);
                let c_curr = nvim_cls_get_c(s);
                let mod_mask = nvim_get_mod_mask();
                let cmdbuff = nvim_get_ccline_cmdbuff();
                let cmdpos = nvim_get_ccline_cmdpos();
                if !((c_curr == K_S_RIGHT
                    || c_curr == K_C_RIGHT
                    || (mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL)) != 0)
                    && *cmdbuff.add(cmdpos as usize) != b' ' as c_char)
                {
                    break;
                }
            }
            let cmdpos = nvim_get_ccline_cmdpos();
            nvim_set_ccline_cmdspos(cmd_screencol(cmdpos));
            return nvim_command_line_not_changed(s);
        }

        // Arrow left / CTRL-left
        k if k == K_LEFT || k == K_S_LEFT || k == K_C_LEFT => {
            if nvim_get_ccline_cmdpos() == 0 {
                return nvim_command_line_not_changed(s);
            }
            loop {
                let cmdpos = nvim_get_ccline_cmdpos();
                nvim_set_ccline_cmdpos(cmdpos - 1);
                let cmdbuff = nvim_get_ccline_cmdbuff();
                let new_cmdpos = nvim_get_ccline_cmdpos();
                let head_off = utf_head_off(cmdbuff, cmdbuff.add(new_cmdpos as usize));
                nvim_set_ccline_cmdpos(new_cmdpos - head_off);
                let new_cmdpos = nvim_get_ccline_cmdpos();
                let cells = crate::screen::rs_cmdline_charsize(new_cmdpos);
                let cmdspos = nvim_get_ccline_cmdspos();
                nvim_set_ccline_cmdspos(cmdspos - cells);

                let c_curr = nvim_cls_get_c(s);
                let mod_mask = nvim_get_mod_mask();
                let cmdpos = nvim_get_ccline_cmdpos();
                if !(cmdpos > 0
                    && (c_curr == K_S_LEFT
                        || c_curr == K_C_LEFT
                        || (mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL)) != 0)
                    && *cmdbuff.add(cmdpos as usize - 1) != b' ' as c_char)
                {
                    break;
                }
            }
            let cmdpos = nvim_get_ccline_cmdpos();
            nvim_set_ccline_cmdspos(cmd_screencol(cmdpos));
            let special_char = nvim_get_ccline_special_char();
            if special_char != NUL {
                putcmdline(special_char as c_char, nvim_get_ccline_special_shift() != 0);
            }
            return nvim_command_line_not_changed(s);
        }

        // K_IGNORE: ignore mouse event or open_cmdwin() result
        x if x == K_IGNORE => {
            return nvim_command_line_not_changed(s);
        }

        // Middle mouse drag/release: ignored
        k if k == K_MIDDLEDRAG || k == K_MIDDLERELEASE => {
            return nvim_command_line_not_changed(s);
        }

        // Middle mouse button: paste
        x if x == K_MIDDLEMOUSE => {
            let clipboard_star = b'*' as c_int;
            let regname = if eval_has_provider(c"clipboard".as_ptr(), false) {
                clipboard_star
            } else {
                0
            };
            nvim_cmdline_paste(regname, true, true);
            crate::screen::redrawcmd_rs();
            return nvim_command_line_changed(s);
        }

        // Left/right drag and release (may be ignored)
        k if k == K_LEFTDRAG || k == K_LEFTRELEASE || k == K_RIGHTDRAG || k == K_RIGHTRELEASE => {
            if nvim_cls_get_ignore_drag_release(s) != 0 {
                return nvim_command_line_not_changed(s);
            }
            // FALLTHROUGH to K_LEFTMOUSE / K_RIGHTMOUSE
            nvim_command_line_left_right_mouse(s);
            return nvim_command_line_not_changed(s);
        }

        // Left mouse: check if above number prompt
        x if x == K_LEFTMOUSE => {
            let mouse_used = nvim_cls_get_ccline_mouse_used();
            if mouse_used != 0 {
                let mouse_row = nvim_get_mouse_row();
                let cmdline_row = nvim_get_cmdline_row();
                if mouse_row < cmdline_row {
                    nvim_cls_set_ccline_mouse_used_val(1);
                    return 0;
                }
            }
            nvim_command_line_left_right_mouse(s);
            return nvim_command_line_not_changed(s);
        }

        // Right mouse
        x if x == K_RIGHTMOUSE => {
            nvim_command_line_left_right_mouse(s);
            return nvim_command_line_not_changed(s);
        }

        // Mouse scroll/alternate buttons: ignored
        k if k == K_MOUSEDOWN
            || k == K_MOUSEUP
            || k == K_MOUSELEFT
            || k == K_MOUSERIGHT
            || k == K_X1MOUSE
            || k == K_X1DRAG
            || k == K_X1RELEASE
            || k == K_X2MOUSE
            || k == K_X2DRAG
            || k == K_X2RELEASE
            || k == K_MOUSEMOVE
            || k == K_SELECT =>
        {
            return nvim_command_line_not_changed(s);
        }

        // CTRL-B / Home: beginning of command line
        k if k == CTRL_B
            || k == K_HOME
            || k == K_XHOME
            || k == K_KHOME
            || k == K_S_HOME
            || k == K_C_HOME =>
        {
            nvim_set_ccline_cmdpos(0);
            nvim_set_ccline_cmdspos(crate::screen::rs_cmd_startcol());
            return nvim_command_line_not_changed(s);
        }

        // CTRL-E / End: end of command line
        k if k == CTRL_E
            || k == K_END
            || k == K_XEND
            || k == K_KEND
            || k == K_S_END
            || k == K_C_END =>
        {
            let cmdlen = nvim_get_ccline_cmdlen();
            nvim_set_ccline_cmdpos(cmdlen);
            nvim_set_ccline_cmdspos(cmd_screencol(cmdlen));
            return nvim_command_line_not_changed(s);
        }

        // CTRL-A: all matches
        x if x == CTRL_A => {
            if cmdline_pum_active() != 0 {
                nvim_cmdline_pum_cleanup();
            }
            let xp = nvim_cls_get_xpc(s);
            let firstc = nvim_cls_get_firstc(s);
            if nvim_nextwild(xp, WILD_ALL, 0, firstc != b'@' as c_int) == FAIL {
                // fall through to normal char
            } else {
                nvim_cls_set_xpc_context(s, EXPAND_NOTHING);
                nvim_cls_set_did_wild_list(s, 0);
                return nvim_command_line_changed(s);
            }
        }

        // CTRL-L: longest common part
        x if x == CTRL_L => {
            let firstc = nvim_cls_get_firstc(s);
            let is_state = nvim_cls_get_is_state(s);
            let mut c_val = c;
            if nvim_may_add_char_to_search(firstc, &raw mut c_val, is_state) == 1 {
                // OK
                nvim_cls_set_c(s, c_val);
                return nvim_command_line_not_changed(s);
            }
            let xp = nvim_cls_get_xpc(s);
            if nvim_nextwild(xp, WILD_LONGEST, 0, firstc != b'@' as c_int) == FAIL {
                // fall through
            } else {
                return nvim_command_line_changed(s);
            }
        }

        // CTRL-N / CTRL-P: next/prev match (or history)
        k if k == CTRL_N || k == CTRL_P => {
            let xpc_numfiles = nvim_cls_get_xpc_numfiles(s);
            if xpc_numfiles > 0 {
                let wild_type = if c == CTRL_P { WILD_PREV } else { WILD_NEXT };
                let xp = nvim_cls_get_xpc(s);
                let firstc = nvim_cls_get_firstc(s);
                if nvim_nextwild(xp, wild_type, 0, firstc != b'@' as c_int) == FAIL {
                    // fall through to history
                } else {
                    return nvim_command_line_changed(s);
                }
            }
            // FALLTHROUGH to history
            match nvim_command_line_browse_history(s) {
                x if x == CMDLINE_CHANGED => {
                    nvim_cls_set_did_hist_navigate(s, 1);
                    return nvim_command_line_changed(s);
                }
                x if x == GOTO_NORMAL_MODE => return 0,
                _ => return nvim_command_line_not_changed(s),
            }
        }

        // K_UP / K_DOWN / K_S_UP / K_S_DOWN / K_PAGEUP / K_KPAGEUP / K_PAGEDOWN / K_KPAGEDOWN
        k if k == K_UP
            || k == K_DOWN
            || k == K_S_UP
            || k == K_S_DOWN
            || k == K_PAGEUP
            || k == K_KPAGEUP
            || k == K_PAGEDOWN
            || k == K_KPAGEDOWN =>
        {
            if cmdline_pum_active() != 0
                && (c == K_PAGEUP || c == K_PAGEDOWN || c == K_KPAGEUP || c == K_KPAGEDOWN)
            {
                let wild_type = if c == K_PAGEDOWN || c == K_KPAGEDOWN {
                    WILD_PAGEDOWN
                } else {
                    WILD_PAGEUP
                };
                let xp = nvim_cls_get_xpc(s);
                let firstc = nvim_cls_get_firstc(s);
                if nvim_nextwild(xp, wild_type, 0, firstc != b'@' as c_int) == FAIL {
                    // fall through
                } else {
                    return nvim_command_line_changed(s);
                }
            } else {
                match nvim_command_line_browse_history(s) {
                    x if x == CMDLINE_CHANGED => {
                        nvim_cls_set_did_hist_navigate(s, 1);
                        return nvim_command_line_changed(s);
                    }
                    x if x == GOTO_NORMAL_MODE => return 0,
                    _ => return nvim_command_line_not_changed(s),
                }
            }
        }

        // CTRL-G / CTRL-T: next/previous incsearch match
        k if k == CTRL_G || k == CTRL_T => {
            let firstc = nvim_cls_get_firstc(s);
            let count = nvim_cls_get_count(s);
            let is_state = nvim_cls_get_is_state(s);
            if crate::search::rs_may_do_command_line_next_incsearch(
                firstc,
                count,
                is_state.cast(),
                c == CTRL_G,
            ) == FAIL
            {
                return nvim_command_line_not_changed(s);
            }
            // break: fall through to end
        }

        // CTRL-V / CTRL-Q: literal insert
        k if k == CTRL_V || k == CTRL_Q => {
            nvim_cls_set_ignore_drag_release(s, 1);
            putcmdline(b'^' as c_char, true);

            let mod_mask = nvim_get_mod_mask();
            let c_new = nvim_get_literal_call((mod_mask & MOD_MASK_SHIFT) != 0);
            nvim_cls_set_c(s, c_new);
            nvim_cls_set_do_abbr(s, 0);
            nvim_set_ccline_special_char(NUL);

            let c_curr = nvim_cls_get_c(s);
            if nvim_utf_iscomposing_first(c_curr) != 0 && nvim_get_cmd_silent() == 0 {
                if ui_has(K_UI_CMDLINE_MAIN) != 0 {
                    unputcmdline();
                } else {
                    let cmdpos = nvim_get_ccline_cmdpos();
                    let cmdlen = nvim_get_ccline_cmdlen();
                    crate::screen::draw_cmdline_rs(cmdpos, cmdlen - cmdpos);
                    msg_putchar(b' ' as c_int);
                    crate::screen::cursorcmd_rs();
                }
            }
            // break: fall through to end
        }

        // CTRL-K: digraph
        x if x == CTRL_K => {
            nvim_cls_set_ignore_drag_release(s, 1);
            putcmdline(b'?' as c_char, true);
            let c_new = nvim_get_digraph(true);
            nvim_cls_set_c(s, c_new);
            nvim_set_ccline_special_char(NUL);

            if nvim_cls_get_c(s) == NUL {
                crate::screen::redrawcmd_rs();
                return nvim_command_line_not_changed(s);
            }
            // else: break - fall through to end
        }

        // CTRL-_: switch language mode (Arabic)
        x if x == CTRL_UU => {
            if nvim_get_p_ari() == 0 {
                // fall through to normal char
            } else {
                return nvim_command_line_not_changed(s);
            }
        }

        // 'q': return on mouse prompt with NUL
        x if x == b'q' as c_int => {
            if nvim_cls_get_ccline_mouse_used() != 0 {
                nvim_ccline_cmdbuff_set_nul();
                return 0;
            }
            // FALLTHROUGH to default
        }

        // Open command window
        k if k == get_cedit_key() || k == K_CMDWIN => {
            let ex_normal = nvim_get_ex_normal_busy();
            if (c == K_CMDWIN || ex_normal == 0) && nvim_get_ccline_cmdfirstc() != b'@' as c_int {
                nvim_open_cmdwin();
                // K_IGNORE after command window means nothing changed
                let c_new = nvim_cls_get_c(s);
                if c_new == K_IGNORE {
                    return nvim_command_line_not_changed(s);
                }
                // Otherwise re-check the char (it may have been replaced)
                let c_check = nvim_cls_get_c(s);
                if c_check == K_IGNORE {
                    return nvim_command_line_not_changed(s);
                }
                // fall through to handle key
            }
            // break: fall through to normal char handling
        }

        // Default: normal character
        _ => {
            let c_curr = nvim_cls_get_c(s);
            if !is_special_key(c_curr) {
                nvim_set_mod_mask(0);
            }
            // break: fall through to end
        }
    }

    // End of switch: handle as normal character or check abbreviation
    let c_curr = nvim_cls_get_c(s);
    let do_abbr = nvim_cls_get_do_abbr(s) != 0;
    if do_abbr
        && (is_special_key(c_curr) || !vim_iswordc(c_curr))
        && (crate::edit::rs_ccheck_abbr(if c_curr >= 0x100 {
            c_curr + ABBR_OFF
        } else {
            c_curr
        }) != 0
            || c_curr == CTRL_RSB)
    {
        return nvim_command_line_changed(s);
    }

    handle_key_end(s, c_curr)
}

/// Put the character in the command line (the "end:" goto target in C).
#[inline]
unsafe fn handle_key_end(s: *mut c_void, c: c_int) -> c_int {
    let mod_mask = nvim_get_mod_mask();
    if is_special_key(c) || mod_mask != 0 {
        let key_name = nvim_get_special_key_name(c, mod_mask);
        crate::edit::put_on_cmdline_rs(key_name, -1, true);
    } else {
        let iobuff = nvim_get_iobuff();
        let j = utf_char2bytes(c, iobuff);
        *iobuff.add(j as usize) = 0; // NUL terminate, exclude composing chars
        crate::edit::put_on_cmdline_rs(iobuff, j, true);
    }
    let one_key = nvim_get_ccline_one_key() != 0;
    if one_key {
        0
    } else {
        nvim_command_line_changed(s)
    }
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
