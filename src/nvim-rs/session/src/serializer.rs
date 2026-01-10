//! Session file serialization utilities
//!
//! This module provides helpers for writing Vim script commands to session files.

use std::ffi::{c_char, c_int};

// =============================================================================
// Common Session Commands as Static Strings
// =============================================================================

// These are pre-formatted strings for common session file lines

/// "let v:this_session=expand(\"<sfile>:p\")"
pub const CMD_SET_THIS_SESSION: &[u8] = b"let v:this_session=expand(\"<sfile>:p\")\n\0";

/// "silent only"
pub const CMD_SILENT_ONLY: &[u8] = b"silent only\n\0";

/// "silent tabonly"
pub const CMD_SILENT_TABONLY: &[u8] = b"silent tabonly\n\0";

/// "wincmd ="
pub const CMD_WINCMD_EQUAL: &[u8] = b"wincmd =\n\0";

/// "wincmd _ | wincmd |"
pub const CMD_WINCMD_MAXIMIZE: &[u8] = b"wincmd _ | wincmd |\n\0";

/// "tabnext"
pub const CMD_TABNEXT: &[u8] = b"tabnext\n\0";

/// "tabrewind"
pub const CMD_TABREWIND: &[u8] = b"tabrewind\n\0";

/// "tabnew +setlocal\\ bufhidden=wipe"
pub const CMD_TABNEW: &[u8] = b"tabnew +setlocal\\ bufhidden=wipe\n\0";

/// "set shortmess+=aoO"
pub const CMD_SET_SHORTMESS: &[u8] = b"set shortmess+=aoO\n\0";

/// "let s:shortmess_save = &shortmess"
pub const CMD_SAVE_SHORTMESS: &[u8] = b"let s:shortmess_save = &shortmess\n\0";

/// "let &shortmess = s:shortmess_save"
pub const CMD_RESTORE_SHORTMESS: &[u8] = b"let &shortmess = s:shortmess_save\n\0";

/// "let s:save_splitbelow = &splitbelow"
pub const CMD_SAVE_SPLITBELOW: &[u8] = b"let s:save_splitbelow = &splitbelow\n\0";

/// "let s:save_splitright = &splitright"
pub const CMD_SAVE_SPLITRIGHT: &[u8] = b"let s:save_splitright = &splitright\n\0";

/// "set splitbelow splitright"
pub const CMD_SET_SPLIT: &[u8] = b"set splitbelow splitright\n\0";

/// "let &splitbelow = s:save_splitbelow"
pub const CMD_RESTORE_SPLITBELOW: &[u8] = b"let &splitbelow = s:save_splitbelow\n\0";

/// "let &splitright = s:save_splitright"
pub const CMD_RESTORE_SPLITRIGHT: &[u8] = b"let &splitright = s:save_splitright\n\0";

// Empty buffer detection script
pub const CMD_CHECK_EMPTY_BUFFER: &[u8] = b"if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''\n  let s:wipebuf = bufnr('%')\nendif\n\0";

// =============================================================================
// FFI Exports for Command Strings
// =============================================================================

/// Get "let v:this_session" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_set_this_session() -> *const c_char {
    CMD_SET_THIS_SESSION.as_ptr().cast::<c_char>()
}

/// Get "silent only" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_silent_only() -> *const c_char {
    CMD_SILENT_ONLY.as_ptr().cast::<c_char>()
}

/// Get "silent tabonly" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_silent_tabonly() -> *const c_char {
    CMD_SILENT_TABONLY.as_ptr().cast::<c_char>()
}

/// Get "wincmd =" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_wincmd_equal() -> *const c_char {
    CMD_WINCMD_EQUAL.as_ptr().cast::<c_char>()
}

/// Get "wincmd _ | wincmd |" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_wincmd_maximize() -> *const c_char {
    CMD_WINCMD_MAXIMIZE.as_ptr().cast::<c_char>()
}

/// Get "tabnext" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_tabnext() -> *const c_char {
    CMD_TABNEXT.as_ptr().cast::<c_char>()
}

/// Get "tabrewind" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_tabrewind() -> *const c_char {
    CMD_TABREWIND.as_ptr().cast::<c_char>()
}

/// Get "tabnew" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_tabnew() -> *const c_char {
    CMD_TABNEW.as_ptr().cast::<c_char>()
}

/// Get "set shortmess" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_set_shortmess() -> *const c_char {
    CMD_SET_SHORTMESS.as_ptr().cast::<c_char>()
}

/// Get "save shortmess" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_save_shortmess() -> *const c_char {
    CMD_SAVE_SHORTMESS.as_ptr().cast::<c_char>()
}

/// Get "restore shortmess" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_restore_shortmess() -> *const c_char {
    CMD_RESTORE_SHORTMESS.as_ptr().cast::<c_char>()
}

/// Get "save splitbelow" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_save_splitbelow() -> *const c_char {
    CMD_SAVE_SPLITBELOW.as_ptr().cast::<c_char>()
}

/// Get "save splitright" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_save_splitright() -> *const c_char {
    CMD_SAVE_SPLITRIGHT.as_ptr().cast::<c_char>()
}

/// Get "set split" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_set_split() -> *const c_char {
    CMD_SET_SPLIT.as_ptr().cast::<c_char>()
}

/// Get "restore splitbelow" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_restore_splitbelow() -> *const c_char {
    CMD_RESTORE_SPLITBELOW.as_ptr().cast::<c_char>()
}

/// Get "restore splitright" command
#[no_mangle]
pub extern "C" fn rs_session_cmd_restore_splitright() -> *const c_char {
    CMD_RESTORE_SPLITRIGHT.as_ptr().cast::<c_char>()
}

/// Get empty buffer check script
#[no_mangle]
pub extern "C" fn rs_session_cmd_check_empty_buffer() -> *const c_char {
    CMD_CHECK_EMPTY_BUFFER.as_ptr().cast::<c_char>()
}

// =============================================================================
// Format String Templates
// =============================================================================

/// Format specifier for cd command: "cd %s\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_cd() -> *const c_char {
    static FMT: &[u8] = b"cd %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for edit command: "edit %s\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_edit() -> *const c_char {
    static FMT: &[u8] = b"edit %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for badd command: "badd +%d %s\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_badd() -> *const c_char {
    static FMT: &[u8] = b"badd +%d %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for argglobal command
#[no_mangle]
pub extern "C" fn rs_session_fmt_argglobal() -> *const c_char {
    static FMT: &[u8] = b"argglobal\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for argadd command: "argadd %s\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_argadd() -> *const c_char {
    static FMT: &[u8] = b"argadd %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for set lines/columns: "set lines=%d columns=%d\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_set_size() -> *const c_char {
    static FMT: &[u8] = b"set lines=%d columns=%d\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for resize
#[no_mangle]
pub extern "C" fn rs_session_fmt_resize() -> *const c_char {
    static FMT: &[u8] = b"exe '%dresize ' . ((&lines * %d + %d) / %d)\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for vertical resize
#[no_mangle]
pub extern "C" fn rs_session_fmt_vert_resize() -> *const c_char {
    static FMT: &[u8] = b"exe 'vert %dresize ' . ((&columns * %d + %d) / %d)\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for normal cursor: "normal! 0%d|\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_normal_cursor() -> *const c_char {
    static FMT: &[u8] = b"normal! 0%d|\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for normal cursor at EOL: "normal! $\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_normal_eol() -> *const c_char {
    static FMT: &[u8] = b"normal! $\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for wincmd with count: "%dwincmd %c\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_wincmd() -> *const c_char {
    static FMT: &[u8] = b"%dwincmd %c\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for let global variable: "let g:%s = %s\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_let_global() -> *const c_char {
    static FMT: &[u8] = b"let g:%s = %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for lcd command: "lcd %s\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_lcd() -> *const c_char {
    static FMT: &[u8] = b"lcd %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Format specifier for tcd command: "tcd %s\n"
#[no_mangle]
pub extern "C" fn rs_session_fmt_tcd() -> *const c_char {
    static FMT: &[u8] = b"tcd %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

// =============================================================================
// Session Write State
// =============================================================================

/// State for tracking session file writing progress
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SessionWriteState {
    /// Whether :lcd or :tcd was produced
    pub did_lcd: c_int,
    /// Current window number
    pub win_nr: c_int,
    /// Current tab number
    pub tab_nr: c_int,
    /// Number of windows written
    pub win_count: c_int,
    /// Number of tabs written
    pub tab_count: c_int,
    /// Error code (0 = OK)
    pub error: c_int,
}

impl Default for SessionWriteState {
    fn default() -> Self {
        Self {
            did_lcd: 0,
            win_nr: 1,
            tab_nr: 1,
            win_count: 0,
            tab_count: 0,
            error: 0,
        }
    }
}

/// Create a new session write state
#[no_mangle]
pub extern "C" fn rs_session_write_state_new() -> SessionWriteState {
    SessionWriteState::default()
}

/// Reset session write state
///
/// # Safety
/// The `state` pointer must be valid for writing.
#[no_mangle]
pub unsafe extern "C" fn rs_session_write_state_reset(state: *mut SessionWriteState) {
    if !state.is_null() {
        *state = SessionWriteState::default();
    }
}

/// Check if session write state has error
///
/// # Safety
/// The `state` pointer must be valid for reading.
#[no_mangle]
pub unsafe extern "C" fn rs_session_write_state_has_error(state: *const SessionWriteState) -> bool {
    if state.is_null() {
        return true;
    }
    (*state).error != 0
}

/// Set error in session write state
///
/// # Safety
/// The `state` pointer must be valid for writing.
#[no_mangle]
pub unsafe extern "C" fn rs_session_write_state_set_error(
    state: *mut SessionWriteState,
    error: c_int,
) {
    if !state.is_null() {
        (*state).error = error;
    }
}

/// Increment window count and return new window number
///
/// # Safety
/// The `state` pointer must be valid for reading and writing.
#[no_mangle]
pub unsafe extern "C" fn rs_session_write_state_next_win(state: *mut SessionWriteState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).win_count += 1;
    (*state).win_nr = (*state).win_count;
    (*state).win_nr
}

/// Increment tab count and return new tab number
///
/// # Safety
/// The `state` pointer must be valid for reading and writing.
#[no_mangle]
pub unsafe extern "C" fn rs_session_write_state_next_tab(state: *mut SessionWriteState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).tab_count += 1;
    (*state).tab_nr = (*state).tab_count;
    // Reset window count for new tab
    (*state).win_count = 0;
    (*state).win_nr = 1;
    (*state).tab_nr
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    #[allow(clippy::borrow_as_ptr)]
    fn test_write_state() {
        let mut state = rs_session_write_state_new();
        assert_eq!(state.error, 0);
        assert_eq!(state.win_nr, 1);

        unsafe {
            let win = rs_session_write_state_next_win(ptr::addr_of_mut!(state));
            assert_eq!(win, 1);
            assert_eq!(state.win_count, 1);

            let win = rs_session_write_state_next_win(ptr::addr_of_mut!(state));
            assert_eq!(win, 2);

            let tab = rs_session_write_state_next_tab(ptr::addr_of_mut!(state));
            assert_eq!(tab, 1);
            assert_eq!(state.win_count, 0); // Reset

            assert!(!rs_session_write_state_has_error(ptr::addr_of!(state)));
            rs_session_write_state_set_error(ptr::addr_of_mut!(state), 1);
            assert!(rs_session_write_state_has_error(ptr::addr_of!(state)));
        }
    }
}
