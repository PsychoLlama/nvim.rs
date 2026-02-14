//! Search command helpers
//!
//! This module provides functions for search commands like character search
//! (f/F/t/T), line search, and related operations.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// C External Functions
// =============================================================================

extern "C" {
    // Character search state accessors
    fn nvim_get_lastcdir() -> c_int;
    fn nvim_set_lastcdir(dir: c_int);
    fn nvim_get_last_t_cmd() -> c_int;
    fn nvim_set_last_t_cmd(t_cmd: c_int);
    fn nvim_get_lastc(idx: c_int) -> u8;
    fn nvim_set_lastc(idx: c_int, val: u8);
    fn nvim_get_lastc_bytes() -> *const c_char;
    fn nvim_get_lastc_bytelen() -> c_int;
    fn nvim_set_lastc_bytelen(len: c_int);
    fn nvim_set_lastc_bytes_raw(s: *const c_char, len: c_int);

    // Phase 5: searchc() accessors
    fn nvim_searchc_save_lastc_state(c: c_int, nchar_len: c_int, composing_bytes: *const c_char);
    fn nvim_cap_get_nchar(cap: *const c_void) -> c_int;
    fn nvim_cap_get_arg(cap: *const c_void) -> c_int;
    fn nvim_cap_get_count1(cap: *const c_void) -> c_int;
    fn nvim_cap_get_nchar_len(cap: *const c_void) -> c_int;
    fn nvim_cap_get_nchar_composing_ptr(cap: *const c_void) -> *const c_char;
    fn nvim_cap_get_oap(cap: *const c_void) -> *mut c_void;
    fn nvim_oap_set_inclusive(oap: *mut c_void, val: bool);
    fn nvim_get_keystuffed() -> c_int;
    fn nvim_get_cursor_line_ptr() -> *const c_char;
    fn nvim_get_cursor_line_len() -> c_int;
    fn nvim_get_curwin_cursor_col() -> c_int;
    fn nvim_set_curwin_cursor_col(col: c_int);
    fn nvim_utfc_ptr2len(p: *const c_char) -> c_int;
    fn nvim_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn nvim_vim_strchr_p_cpo(c: c_int) -> bool;
}

// =============================================================================
// Direction Constants
// =============================================================================

/// Direction FORWARD = 1
pub const DIRECTION_FORWARD: c_int = 1;
/// Direction BACKWARD = -1
pub const DIRECTION_BACKWARD: c_int = -1;

// =============================================================================
// Character Search State
// =============================================================================

/// State for character search commands (f/F/t/T).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CharSearchState {
    /// The character being searched for (first byte or codepoint)
    pub char_code: c_int,
    /// Direction of search (FORWARD=1, BACKWARD=-1)
    pub direction: c_int,
    /// Whether this is a 't' command (search until, not to)
    pub until: bool,
    /// Length of the character in bytes (for multibyte)
    pub byte_len: c_int,
}

impl CharSearchState {
    /// Create a new character search state.
    pub const fn new() -> Self {
        Self {
            char_code: 0,
            direction: DIRECTION_FORWARD,
            until: false,
            byte_len: 0,
        }
    }

    /// Create a state for a forward 'f' command.
    pub fn forward_f(char_code: c_int, byte_len: c_int) -> Self {
        Self {
            char_code,
            direction: DIRECTION_FORWARD,
            until: false,
            byte_len,
        }
    }

    /// Create a state for a forward 't' command.
    pub fn forward_t(char_code: c_int, byte_len: c_int) -> Self {
        Self {
            char_code,
            direction: DIRECTION_FORWARD,
            until: true,
            byte_len,
        }
    }

    /// Create a state for a backward 'F' command.
    pub fn backward_f(char_code: c_int, byte_len: c_int) -> Self {
        Self {
            char_code,
            direction: DIRECTION_BACKWARD,
            until: false,
            byte_len,
        }
    }

    /// Create a state for a backward 'T' command.
    pub fn backward_t(char_code: c_int, byte_len: c_int) -> Self {
        Self {
            char_code,
            direction: DIRECTION_BACKWARD,
            until: true,
            byte_len,
        }
    }

    /// Check if search is forward.
    #[inline]
    pub fn is_forward(&self) -> bool {
        self.direction == DIRECTION_FORWARD
    }

    /// Check if search is backward.
    #[inline]
    pub fn is_backward(&self) -> bool {
        self.direction == DIRECTION_BACKWARD
    }

    /// Check if this is a 'to' command (f/F).
    #[inline]
    pub fn is_to(&self) -> bool {
        !self.until
    }

    /// Check if this is an 'until' command (t/T).
    #[inline]
    pub fn is_until(&self) -> bool {
        self.until
    }

    /// Reverse the direction.
    pub fn reverse(&mut self) {
        self.direction = -self.direction;
    }

    /// Get a reversed copy.
    pub fn reversed(&self) -> Self {
        Self {
            direction: -self.direction,
            ..*self
        }
    }
}

// =============================================================================
// Character Search State Management
// =============================================================================

/// Get the last character search direction.
///
/// Returns FORWARD (1) or BACKWARD (-1).
#[inline]
pub fn get_csearch_direction() -> c_int {
    // SAFETY: Accessing global variable through accessor
    unsafe { nvim_get_lastcdir() }
}

/// Set the character search direction.
#[inline]
pub fn set_csearch_direction(dir: c_int) {
    // SAFETY: Setting global variable through accessor
    unsafe { nvim_set_lastcdir(dir) }
}

/// Get whether the last character search was a 't' command.
#[inline]
pub fn get_csearch_until() -> bool {
    // SAFETY: Accessing global variable through accessor
    unsafe { nvim_get_last_t_cmd() != 0 }
}

/// Set whether the character search is a 't' command.
#[inline]
pub fn set_csearch_until(until: bool) {
    // SAFETY: Setting global variable through accessor
    unsafe { nvim_set_last_t_cmd(c_int::from(until)) }
}

/// Get the last searched character code.
#[inline]
pub fn get_last_csearch_char() -> c_int {
    // SAFETY: Accessing global variable through accessor
    // Index 0 is the first character
    unsafe { c_int::from(nvim_get_lastc(0)) }
}

/// Set the last searched character.
#[inline]
pub fn set_last_csearch_char(c: u8) {
    // SAFETY: Setting global variable through accessor
    unsafe { nvim_set_lastc(0, c) }
}

/// Get the byte length of the last searched character.
#[inline]
pub fn get_csearch_bytelen() -> c_int {
    // SAFETY: Accessing global variable through accessor
    unsafe { nvim_get_lastc_bytelen() }
}

/// Set the byte length of the last searched character.
#[inline]
pub fn set_csearch_bytelen(len: c_int) {
    // SAFETY: Setting global variable through accessor
    unsafe { nvim_set_lastc_bytelen(len) }
}

/// Get a pointer to the last searched character bytes.
///
/// # Safety
///
/// Returns a pointer to a static buffer. The caller must ensure the
/// pointer is not used after the buffer is modified.
#[inline]
pub unsafe fn get_csearch_bytes() -> *const c_char {
    nvim_get_lastc_bytes()
}

/// Check if a character search has been performed.
#[inline]
pub fn has_csearch() -> bool {
    get_last_csearch_char() != 0 || get_csearch_bytelen() > 1
}

/// Get the current character search state from globals.
pub fn get_current_csearch_state() -> CharSearchState {
    CharSearchState {
        char_code: get_last_csearch_char(),
        direction: get_csearch_direction(),
        until: get_csearch_until(),
        byte_len: get_csearch_bytelen(),
    }
}

// =============================================================================
// Character Search Direction Helpers
// =============================================================================

/// Check if the last character search was forward.
#[inline]
pub fn last_csearch_was_forward() -> bool {
    get_csearch_direction() == DIRECTION_FORWARD
}

/// Check if the last character search was backward.
#[inline]
pub fn last_csearch_was_backward() -> bool {
    get_csearch_direction() == DIRECTION_BACKWARD
}

/// Get the repeat direction for ';' command (same direction).
#[inline]
pub fn csearch_repeat_direction() -> c_int {
    get_csearch_direction()
}

/// Get the reverse direction for ',' command (opposite direction).
#[inline]
pub fn csearch_reverse_direction() -> c_int {
    -get_csearch_direction()
}

// =============================================================================
// Search Repeat Helpers
// =============================================================================

/// Check if a character search can be repeated.
///
/// A search can be repeated if there's a valid last searched character
/// (either single-byte or multi-byte).
#[inline]
pub fn can_repeat_csearch() -> bool {
    has_csearch()
}

/// Get the direction for a repeat/reverse command.
///
/// If `reverse` is true, returns the opposite direction;
/// otherwise returns the same direction.
#[inline]
pub fn get_repeat_direction(reverse: bool) -> c_int {
    if reverse {
        csearch_reverse_direction()
    } else {
        csearch_repeat_direction()
    }
}

// =============================================================================
// Offset Calculation Helpers
// =============================================================================

/// Calculate the column offset when backing up for 't' command.
///
/// When the 't' command is used, we need to stop before the character,
/// not on it. This calculates the adjustment needed.
#[inline]
pub fn t_cmd_backup_offset(dir: c_int, char_bytelen: c_int) -> c_int {
    if dir > 0 {
        // Forward: move back one position (before the found char)
        -1
    } else {
        // Backward: stay on the found char but add bytelen - 1
        char_bytelen - 1
    }
}

/// Check if we need to force movement for ';' with 't' command.
///
/// When using ';' to repeat a 't' command with count 1, we need to force
/// movement of at least one character so we don't stay in place if we're
/// right in front of the target character.
///
/// Returns true if we should NOT stop on the first match.
#[inline]
pub fn should_force_csearch_movement(count: c_int, until: bool, cpo_has_scolon: bool) -> bool {
    !cpo_has_scolon && count == 1 && until
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Set the last character search state (character, byte string, and byte length).
///
/// This is the Rust equivalent of `set_last_csearch()` in search.c.
///
/// # Safety
/// `s` must be a valid pointer to `len` bytes (or NULL if `len` is 0).
pub unsafe fn set_last_csearch(c: c_int, s: *const c_char, len: c_int) {
    nvim_set_lastc(0, c as u8);
    nvim_set_lastc_bytelen(len);
    nvim_set_lastc_bytes_raw(s, len);
}

/// FFI: Set the last character search state.
///
/// # Safety
/// `s` must be a valid pointer to `len` bytes (or NULL if `len` is 0).
#[no_mangle]
pub unsafe extern "C" fn rs_set_last_csearch(c: c_int, s: *const c_char, len: c_int) {
    set_last_csearch(c, s, len);
}

/// FFI: Create a new character search state.
#[no_mangle]
pub extern "C" fn rs_csearch_state_new() -> CharSearchState {
    CharSearchState::new()
}

/// FFI: Get the character search direction.
#[no_mangle]
pub extern "C" fn rs_get_csearch_direction() -> c_int {
    get_csearch_direction()
}

/// FFI: Set the character search direction.
#[no_mangle]
pub extern "C" fn rs_set_csearch_direction(dir: c_int) {
    set_csearch_direction(dir);
}

/// FFI: Get whether last csearch was 't' command.
#[no_mangle]
pub extern "C" fn rs_get_csearch_until() -> c_int {
    c_int::from(get_csearch_until())
}

/// FFI: Set whether csearch is 't' command.
#[no_mangle]
pub extern "C" fn rs_set_csearch_until(until: c_int) {
    set_csearch_until(until != 0);
}

/// FFI: Get the last searched character.
#[no_mangle]
pub extern "C" fn rs_get_last_csearch_char() -> c_int {
    get_last_csearch_char()
}

/// FFI: Set the last searched character.
#[no_mangle]
pub extern "C" fn rs_set_last_csearch_char(c: c_int) {
    set_last_csearch_char(c as u8);
}

/// FFI: Get csearch byte length.
#[no_mangle]
pub extern "C" fn rs_get_csearch_bytelen() -> c_int {
    get_csearch_bytelen()
}

/// FFI: Set csearch byte length.
#[no_mangle]
pub extern "C" fn rs_set_csearch_bytelen(len: c_int) {
    set_csearch_bytelen(len);
}

/// FFI: Check if csearch has been performed.
#[no_mangle]
pub extern "C" fn rs_has_csearch() -> c_int {
    c_int::from(has_csearch())
}

/// FFI: Check if last csearch was forward.
#[no_mangle]
pub extern "C" fn rs_last_csearch_was_forward() -> c_int {
    c_int::from(last_csearch_was_forward())
}

/// FFI: Check if last csearch was backward.
#[no_mangle]
pub extern "C" fn rs_last_csearch_was_backward() -> c_int {
    c_int::from(last_csearch_was_backward())
}

/// FFI: Check if csearch can be repeated.
#[no_mangle]
pub extern "C" fn rs_can_repeat_csearch() -> c_int {
    c_int::from(can_repeat_csearch())
}

/// FFI: Get repeat direction (same or reverse).
#[no_mangle]
pub extern "C" fn rs_get_repeat_direction(reverse: c_int) -> c_int {
    get_repeat_direction(reverse != 0)
}

/// FFI: Calculate t_cmd backup offset.
#[no_mangle]
pub extern "C" fn rs_t_cmd_backup_offset(dir: c_int, char_bytelen: c_int) -> c_int {
    t_cmd_backup_offset(dir, char_bytelen)
}

/// FFI: Check if should force csearch movement.
#[no_mangle]
pub extern "C" fn rs_should_force_csearch_movement(
    count: c_int,
    until: c_int,
    cpo_has_scolon: c_int,
) -> c_int {
    c_int::from(should_force_csearch_movement(
        count,
        until != 0,
        cpo_has_scolon != 0,
    ))
}

/// FFI: Get current csearch state from globals.
#[no_mangle]
pub extern "C" fn rs_get_current_csearch_state() -> CharSearchState {
    get_current_csearch_state()
}

// =============================================================================
// searchc() — f/F/t/T character search
// =============================================================================

/// CPO_SCOLON character value (';' = 59).
const CPO_SCOLON: c_int = b';' as c_int;

/// OK return value (matches C OK = 1).
const OK: c_int = 1;
/// FAIL return value (matches C FAIL = 0).
const FAIL: c_int = 0;

/// Search for a character in a line (f/F/t/T commands).
///
/// If `t_cmd` is false, move to the position of the character,
/// otherwise move to just before the char.
/// Does this `cap->count1` times.
///
/// # Safety
///
/// `cap` must be a valid, non-null pointer to a `cmdarg_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_searchc(cap: *mut c_void, t_cmd_arg: bool) -> c_int {
    let mut c = nvim_cap_get_nchar(cap);
    let mut dir = nvim_cap_get_arg(cap);
    let mut count = nvim_cap_get_count1(cap);
    let mut t_cmd = t_cmd_arg;
    let mut stop = true;

    if c != 0 {
        // Normal search: remember args for repeat
        if nvim_get_keystuffed() == 0 {
            // Don't remember when redoing
            let nchar_len = nvim_cap_get_nchar_len(cap);
            let composing_ptr = nvim_cap_get_nchar_composing_ptr(cap);
            nvim_searchc_save_lastc_state(c, nchar_len, composing_ptr);
            set_csearch_direction(dir);
            set_csearch_until(t_cmd);
        }
    } else {
        // Repeat previous search
        if nvim_get_lastc(0) == 0 && nvim_get_lastc_bytelen() <= 1 {
            return FAIL;
        }
        let lastcdir = nvim_get_lastcdir();
        dir = if dir != 0 { -lastcdir } else { lastcdir };
        t_cmd = nvim_get_last_t_cmd() != 0;
        c = nvim_get_lastc(0) as c_int;
        // For multi-byte re-use last lastc_bytes[] and lastc_bytelen.

        // Force a move of at least one char, so ";" and "," will move the
        // cursor, even if the cursor is right in front of char we are looking at.
        if !nvim_vim_strchr_p_cpo(CPO_SCOLON) && count == 1 && t_cmd {
            stop = false;
        }
    }

    // Set oap->inclusive
    let oap = nvim_cap_get_oap(cap);
    nvim_oap_set_inclusive(oap, dir != DIRECTION_BACKWARD);

    let p = nvim_get_cursor_line_ptr();
    let mut col = nvim_get_curwin_cursor_col();
    let len = nvim_get_cursor_line_len();
    let lastc_bytelen = nvim_get_lastc_bytelen();

    while count > 0 {
        count -= 1;
        loop {
            if dir > 0 {
                col += nvim_utfc_ptr2len(p.add(col as usize));
                if col >= len {
                    return FAIL;
                }
            } else {
                if col == 0 {
                    return FAIL;
                }
                col -= nvim_utf_head_off(p, p.add(col as usize - 1)) + 1;
            }
            if lastc_bytelen <= 1 {
                if *p.add(col as usize) == c as c_char && stop {
                    break;
                }
            } else if col + lastc_bytelen <= len {
                let lastc_bytes = nvim_get_lastc_bytes();
                let blen = lastc_bytelen as usize;
                let line_slice = std::slice::from_raw_parts(p.add(col as usize) as *const u8, blen);
                let pat_slice = std::slice::from_raw_parts(lastc_bytes as *const u8, blen);
                if line_slice == pat_slice && stop {
                    break;
                }
            }
            stop = true;
        }
    }

    if t_cmd {
        // Backup to before the character (possibly double-byte).
        col -= dir;
        if dir < 0 {
            // Landed on the search char which is lastc_bytelen long.
            col += lastc_bytelen - 1;
        } else {
            // To previous char, which may be multi-byte.
            col -= nvim_utf_head_off(p, p.add(col as usize));
        }
    }
    nvim_set_curwin_cursor_col(col);

    OK
}

// =============================================================================
// Phase 7c: search_for_exact_line
// =============================================================================

extern "C" {
    fn nvim_get_p_ws() -> c_int;
    fn nvim_buf_ml_line_count(buf: *mut c_void) -> c_int;
    fn nvim_buf_get_line_skipwhite(
        buf: *mut c_void,
        lnum: c_int,
        skipwhite_off: *mut c_int,
    ) -> *const c_char;
    fn nvim_search_compl_status_adding() -> c_int;
    fn nvim_search_compl_status_sol() -> c_int;
    fn nvim_search_ins_compl_len() -> c_int;
    fn nvim_mb_strcmp_ic_wrapper(ic: c_int, s1: *const c_char, s2: *const c_char) -> c_int;
    fn nvim_mb_strnicmp_wrapper(s1: *const c_char, s2: *const c_char, len: usize) -> c_int;
    fn nvim_search_get_p_ic() -> c_int;
    fn nvim_shortmess_search() -> c_int;
    fn nvim_give_search_wrap_warning(at_top: c_int);
}

/// Search for an exact line in a buffer (completion search).
///
/// Returns OK (1) if found, FAIL (0) if not.
///
/// # Safety
/// `buf` must be a valid buffer handle. `pat` must be a valid C string.
/// `pos_lnum` and `pos_col` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_search_for_exact_line(
    buf: *mut c_void,
    pos_lnum: *mut c_int,
    pos_col: *mut c_int,
    dir: c_int,
    pat: *const c_char,
) -> c_int {
    let mut start: c_int = 0;
    let line_count = nvim_buf_ml_line_count(buf);

    if line_count == 0 {
        return FAIL;
    }

    loop {
        *pos_lnum += dir;

        if *pos_lnum < 1 {
            if nvim_get_p_ws() != 0 {
                *pos_lnum = line_count;
                if nvim_shortmess_search() == 0 {
                    nvim_give_search_wrap_warning(1); // top_bot_msg
                }
            } else {
                *pos_lnum = 1;
                break;
            }
        } else if *pos_lnum > line_count {
            if nvim_get_p_ws() != 0 {
                *pos_lnum = 1;
                if nvim_shortmess_search() == 0 {
                    nvim_give_search_wrap_warning(0); // bot_top_msg
                }
            } else {
                *pos_lnum = 1;
                break;
            }
        }

        if *pos_lnum == start {
            break;
        }
        if start == 0 {
            start = *pos_lnum;
        }

        let mut skipwhite_off: c_int = 0;
        let p = nvim_buf_get_line_skipwhite(buf, *pos_lnum, &mut skipwhite_off);
        *pos_col = skipwhite_off;

        // when adding lines the matching line may be empty but it is not
        // ignored because we are interested in the next line -- Acevedo
        if nvim_search_compl_status_adding() != 0 && nvim_search_compl_status_sol() == 0 {
            if nvim_mb_strcmp_ic_wrapper(nvim_search_get_p_ic(), p, pat) == 0 {
                return OK;
            }
        } else if !p.is_null() && *p != 0 {
            // Ignore empty lines.
            // Expanding lines or words.
            let compl_len = nvim_search_ins_compl_len();
            assert!(compl_len >= 0);
            let cmp = if nvim_search_get_p_ic() != 0 {
                nvim_mb_strnicmp_wrapper(p, pat, compl_len as usize)
            } else {
                // Case-sensitive strncmp equivalent
                let len = compl_len as usize;
                let s1 = std::slice::from_raw_parts(p as *const u8, len);
                let s2 = std::slice::from_raw_parts(pat as *const u8, len);
                match s1.cmp(s2) {
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Greater => 1,
                }
            };
            if cmp == 0 {
                return OK;
            }
        }
    }

    FAIL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_search_state_new() {
        let state = CharSearchState::new();
        assert_eq!(state.char_code, 0);
        assert_eq!(state.direction, DIRECTION_FORWARD);
        assert!(!state.until);
        assert_eq!(state.byte_len, 0);
    }

    #[test]
    fn test_char_search_state_forward_f() {
        let state = CharSearchState::forward_f(b'x' as c_int, 1);
        assert_eq!(state.char_code, b'x' as c_int);
        assert!(state.is_forward());
        assert!(state.is_to());
        assert!(!state.is_until());
    }

    #[test]
    fn test_char_search_state_forward_t() {
        let state = CharSearchState::forward_t(b'y' as c_int, 1);
        assert_eq!(state.char_code, b'y' as c_int);
        assert!(state.is_forward());
        assert!(state.is_until());
        assert!(!state.is_to());
    }

    #[test]
    fn test_char_search_state_backward_f() {
        let state = CharSearchState::backward_f(b'z' as c_int, 1);
        assert_eq!(state.char_code, b'z' as c_int);
        assert!(state.is_backward());
        assert!(!state.is_until());
    }

    #[test]
    fn test_char_search_state_backward_t() {
        let state = CharSearchState::backward_t(b'a' as c_int, 1);
        assert_eq!(state.char_code, b'a' as c_int);
        assert!(state.is_backward());
        assert!(state.is_until());
    }

    #[test]
    fn test_char_search_state_reverse() {
        let mut state = CharSearchState::forward_f(b'x' as c_int, 1);
        assert!(state.is_forward());

        state.reverse();
        assert!(state.is_backward());

        state.reverse();
        assert!(state.is_forward());
    }

    #[test]
    fn test_char_search_state_reversed() {
        let state = CharSearchState::forward_t(b'x' as c_int, 1);
        let reversed = state.reversed();

        assert!(state.is_forward()); // Original unchanged
        assert!(reversed.is_backward());
        assert_eq!(state.char_code, reversed.char_code);
        assert_eq!(state.until, reversed.until);
    }

    #[test]
    fn test_t_cmd_backup_offset() {
        // Forward: always -1
        assert_eq!(t_cmd_backup_offset(1, 1), -1);
        assert_eq!(t_cmd_backup_offset(1, 3), -1);

        // Backward: char_bytelen - 1
        assert_eq!(t_cmd_backup_offset(-1, 1), 0);
        assert_eq!(t_cmd_backup_offset(-1, 3), 2);
    }

    #[test]
    fn test_should_force_csearch_movement() {
        // Should force when: !cpo_scolon AND count==1 AND until
        assert!(should_force_csearch_movement(1, true, false));

        // Should not force when cpo_scolon is set
        assert!(!should_force_csearch_movement(1, true, true));

        // Should not force when count != 1
        assert!(!should_force_csearch_movement(2, true, false));

        // Should not force when not until (is 'f' command)
        assert!(!should_force_csearch_movement(1, false, false));
    }

    #[test]
    fn test_direction_constants() {
        assert_eq!(DIRECTION_FORWARD, 1);
        assert_eq!(DIRECTION_BACKWARD, -1);
        assert_eq!(DIRECTION_FORWARD, -DIRECTION_BACKWARD);
    }
}
