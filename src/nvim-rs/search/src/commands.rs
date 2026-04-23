//! Search command helpers
//!
//! This module provides functions for search commands like character search
//! (f/F/t/T), line search, and related operations.

use nvim_normal::types::{CmdargT, OpargT};
use std::ffi::{c_char, c_int, c_void};

use crate::char_search_state;

// =============================================================================
// C External Functions
// =============================================================================

extern "C" {
    fn nvim_cap_get_nchar_composing_ptr(cap: *const c_void) -> *const c_char;
    static KeyStuffed: c_int;
    fn nvim_get_cursor_line_ptr() -> *const c_char;
    fn nvim_get_cursor_line_len() -> c_int;
    fn nvim_get_curwin_cursor_col() -> c_int;
    fn nvim_set_curwin_cursor_col(col: c_int);
    fn nvim_utfc_ptr2len(p: *const c_char) -> c_int;
    #[link_name = "utf_head_off"]
    fn nvim_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn nvim_vim_strchr_p_cpo(c: c_int) -> bool;

    // Phase 7d: current_search accessors (direct globals)
    static mut VIsual_active: bool;
    static mut VIsual: crate::searchit::PosT;
    static mut VIsual_mode: c_int;
    fn nvim_set_p_ws(val: c_int);
    fn nvim_get_p_sel_first() -> c_char;
    fn nvim_get_curwin_cursor_lnum() -> c_int;
    fn nvim_search_get_curwin_cursor_coladd() -> c_int;
    fn nvim_set_curwin_cursor_lnum(lnum: c_int);
    fn nvim_set_curwin_cursor_coladd(coladd: c_int);
    fn nvim_inc_cursor() -> c_int;
    fn nvim_dec_cursor() -> c_int;
    fn nvim_search_incl_pos(lnum: *mut c_int, col: *mut c_int, coladd: *mut c_int) -> c_int;
    fn nvim_search_decl_pos(lnum: *mut c_int, col: *mut c_int, coladd: *mut c_int) -> c_int;
    fn nvim_search_get_line_count() -> c_int;
    #[link_name = "ml_get_len"]
    fn nvim_search_ml_get_len(lnum: c_int) -> c_int;
    fn nvim_search_current_searchit(
        dir: c_int,
        flags: c_int,
        count: c_int,
        pos_lnum: *mut c_int,
        pos_col: *mut c_int,
        pos_coladd: *mut c_int,
        end_lnum: *mut c_int,
        end_col: *mut c_int,
        end_coladd: *mut c_int,
    ) -> c_int;
    static mut fdo_flags: u32;
    static mut KeyTyped: bool;
    fn setmouse();
    fn showmode();
    fn nvim_redraw_curbuf_inverted();

    // Rust functions callable from search crate
    fn rs_is_zero_width(
        pat: *const c_char,
        patlen: usize,
        whole: bool,
        lnum: c_int,
        col: c_int,
        coladd: c_int,
        dir: c_int,
    ) -> c_int;
    fn rs_foldOpenCursor();
    fn rs_may_start_select(c: c_int);
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
    char_search_state::get_lastcdir()
}

/// Set the character search direction.
#[inline]
pub fn set_csearch_direction(dir: c_int) {
    char_search_state::set_lastcdir(dir);
}

/// Get whether the last character search was a 't' command.
#[inline]
pub fn get_csearch_until() -> bool {
    char_search_state::get_last_t_cmd()
}

/// Set whether the character search is a 't' command.
#[inline]
pub fn set_csearch_until(until: bool) {
    char_search_state::set_last_t_cmd(until);
}

/// Get the last searched character code.
#[inline]
pub fn get_last_csearch_char() -> c_int {
    c_int::from(char_search_state::get_lastc(0))
}

/// Set the last searched character.
#[inline]
pub fn set_last_csearch_char(c: u8) {
    char_search_state::set_lastc(0, c);
}

/// Get the byte length of the last searched character.
#[inline]
pub fn get_csearch_bytelen() -> c_int {
    char_search_state::get_lastc_bytelen()
}

/// Set the byte length of the last searched character.
#[inline]
pub fn set_csearch_bytelen(len: c_int) {
    char_search_state::set_lastc_bytelen(len);
}

/// Get a pointer to the last searched character bytes.
///
/// Returns a pointer to a static buffer. The caller must ensure the
/// pointer is not used after the buffer is modified.
#[inline]
pub fn get_csearch_bytes() -> *const c_char {
    char_search_state::get_lastc_bytes_ptr()
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
    char_search_state::set_lastc(0, c as u8);
    char_search_state::set_lastc_bytelen(len);
    char_search_state::set_lastc_bytes_raw(s, len);
}

/// FFI: Set the last character search state.
///
/// # Safety
/// `s` must be a valid pointer to `len` bytes (or NULL if `len` is 0).
#[unsafe(export_name = "set_last_csearch")]
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
#[unsafe(export_name = "set_csearch_direction")]
pub extern "C" fn rs_set_csearch_direction(dir: c_int) {
    set_csearch_direction(dir);
}

/// FFI: Get whether last csearch was 't' command.
#[no_mangle]
pub extern "C" fn rs_get_csearch_until() -> c_int {
    c_int::from(get_csearch_until())
}

/// FFI: Set whether csearch is 't' command.
#[unsafe(export_name = "set_csearch_until")]
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
#[unsafe(export_name = "searchc")]
pub unsafe extern "C" fn rs_searchc(cap: *mut c_void, t_cmd_arg: bool) -> c_int {
    let mut c = (*cap.cast::<CmdargT>()).nchar;
    let mut dir = (*cap.cast::<CmdargT>()).arg;
    let mut count = (*cap.cast::<CmdargT>()).count1;
    let mut t_cmd = t_cmd_arg;
    let mut stop = true;

    if c != 0 {
        // Normal search: remember args for repeat
        if KeyStuffed == 0 {
            // Don't remember when redoing
            let nchar_len = (*cap.cast::<CmdargT>()).nchar_len;
            let composing_ptr = nvim_cap_get_nchar_composing_ptr(cap);
            char_search_state::searchc_save_lastc_state(c, nchar_len, composing_ptr);
            set_csearch_direction(dir);
            set_csearch_until(t_cmd);
        }
    } else {
        // Repeat previous search
        if char_search_state::get_lastc(0) == 0 && char_search_state::get_lastc_bytelen() <= 1 {
            return FAIL;
        }
        let lastcdir = char_search_state::get_lastcdir();
        dir = if dir != 0 { -lastcdir } else { lastcdir };
        t_cmd = char_search_state::get_last_t_cmd();
        c = char_search_state::get_lastc(0) as c_int;
        // For multi-byte re-use last lastc_bytes[] and lastc_bytelen.

        // Force a move of at least one char, so ";" and "," will move the
        // cursor, even if the cursor is right in front of char we are looking at.
        if !nvim_vim_strchr_p_cpo(CPO_SCOLON) && count == 1 && t_cmd {
            stop = false;
        }
    }

    // Set oap->inclusive
    let oap = (*cap.cast::<CmdargT>()).oap;
    (*oap.cast::<OpargT>()).inclusive = dir != DIRECTION_BACKWARD;

    let p = nvim_get_cursor_line_ptr();
    let mut col = nvim_get_curwin_cursor_col();
    let len = nvim_get_cursor_line_len();
    let lastc_bytelen = char_search_state::get_lastc_bytelen();

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
                let lastc_bytes = char_search_state::get_lastc_bytes_ptr();
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
    fn rs_compl_status_adding() -> c_int;
    fn rs_compl_status_sol() -> c_int;
    fn rs_ins_compl_len() -> c_int;
    #[link_name = "mb_strcmp_ic"]
    fn nvim_mb_strcmp_ic_wrapper(ic: c_int, s1: *const c_char, s2: *const c_char) -> c_int;
    #[link_name = "mb_strnicmp"]
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
        if rs_compl_status_adding() != 0 && rs_compl_status_sol() == 0 {
            if nvim_mb_strcmp_ic_wrapper(nvim_search_get_p_ic(), p, pat) == 0 {
                return OK;
            }
        } else if !p.is_null() && *p != 0 {
            // Ignore empty lines.
            // Expanding lines or words.
            let compl_len = rs_ins_compl_len();
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

/// search_for_exact_line: C-ABI entry point accepting pos_T* directly.
///
/// # Safety
/// All pointer arguments must be valid.
#[unsafe(export_name = "search_for_exact_line")]
pub unsafe extern "C" fn search_for_exact_line_export(
    buf: *mut c_void,
    pos: *mut crate::searchit::PosT,
    dir: c_int,
    pat: *const c_char,
) -> c_int {
    let pos_ref = &mut *pos;
    let mut lnum = pos_ref.lnum;
    let mut col = pos_ref.col;
    let result = rs_search_for_exact_line(buf, &mut lnum, &mut col, dir, pat);
    pos_ref.lnum = lnum;
    pos_ref.col = col;
    result
}

// =============================================================================
// Phase 7d: current_search
// =============================================================================

const SEARCH_END: c_int = 0x40;
const K_OPT_FDO_FLAG_SEARCH: u32 = 0x40;

/// Migrate current_search from search.c.
///
/// Searches forward or backward for the last search pattern and adjusts
/// Visual mode selection to cover the match.
///
/// # Safety
/// Requires valid editor state (curwin, curbuf, completion state).
#[export_name = "current_search"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_current_search(count: c_int, forward: bool) -> c_int {
    const OK: c_int = 0;
    const FAIL: c_int = -1;

    let old_p_ws = nvim_get_p_ws();
    // Save VIsual position
    let save_visual_lnum = VIsual.lnum;
    let save_visual_col = VIsual.col;
    let save_visual_coladd = VIsual.coladd;

    let visual_active = VIsual_active;

    // Correct cursor when 'selection' is exclusive
    if visual_active && nvim_get_p_sel_first() == b'e' as c_char {
        let vis_lnum = VIsual.lnum;
        let vis_col = VIsual.col;
        let vis_coladd = VIsual.coladd;
        let cur_lnum = nvim_get_curwin_cursor_lnum();
        let cur_col = nvim_get_curwin_cursor_col();
        let cur_coladd = nvim_search_get_curwin_cursor_coladd();
        // lt(VIsual, curwin->w_cursor)
        if lt_pos(vis_lnum, vis_col, vis_coladd, cur_lnum, cur_col, cur_coladd) {
            nvim_dec_cursor();
        }
    }

    // Re-read visual_active since dec_cursor might not change it,
    // but also re-read cursor in case it changed.
    let visual_active = VIsual_active;

    let cur_lnum = nvim_get_curwin_cursor_lnum();
    let cur_col = nvim_get_curwin_cursor_col();
    let cur_coladd = nvim_search_get_curwin_cursor_coladd();

    // When searching forward and cursor is at start of Visual area, skip
    // the first backward search.
    let skip_first_backward = forward
        && visual_active
        && lt_pos(
            cur_lnum,
            cur_col,
            cur_coladd,
            VIsual.lnum,
            VIsual.col,
            VIsual.coladd,
        );

    let mut pos_lnum = cur_lnum;
    let mut pos_col = cur_col;
    let mut pos_coladd = cur_coladd;
    let orig_lnum = cur_lnum;
    let orig_col = cur_col;
    let orig_coladd = cur_coladd;

    if visual_active {
        if forward {
            nvim_search_incl_pos(&mut pos_lnum, &mut pos_col, &mut pos_coladd);
        } else {
            nvim_search_decl_pos(&mut pos_lnum, &mut pos_col, &mut pos_coladd);
        }
    }

    // Check if the pattern is zero-width
    let zero_width = rs_is_zero_width(
        std::ptr::null(),
        0,
        true,
        cur_lnum,
        cur_col,
        cur_coladd,
        DIRECTION_FORWARD,
    );
    if zero_width == -1 {
        return FAIL;
    }

    let mut end_lnum = 0i32;
    let mut end_col = 0i32;
    let mut end_coladd = 0i32;
    let mut result = 0i32;

    for i in 0..2i32 {
        let dir = if forward {
            if i == 0 && skip_first_backward {
                continue;
            }
            i
        } else if i == 0 {
            1
        } else {
            0
        };

        let flags = if dir == 0 && zero_width == 0 {
            SEARCH_END
        } else {
            0
        };

        end_lnum = pos_lnum;
        end_col = pos_col;
        end_coladd = pos_coladd;

        // Wrapping should not occur in the first round
        if i == 0 {
            nvim_set_p_ws(0);
        }

        result = nvim_search_current_searchit(
            dir,
            flags,
            if i != 0 { count } else { 1 },
            &mut pos_lnum,
            &mut pos_col,
            &mut pos_coladd,
            &mut end_lnum,
            &mut end_col,
            &mut end_coladd,
        );

        nvim_set_p_ws(old_p_ws);

        if i == 1 && result == 0 {
            // Not found, abort
            nvim_set_curwin_cursor_lnum(orig_lnum);
            nvim_set_curwin_cursor_col(orig_col);
            nvim_set_curwin_cursor_coladd(orig_coladd);
            if visual_active {
                VIsual = crate::searchit::PosT {
                    lnum: save_visual_lnum,
                    col: save_visual_col,
                    coladd: save_visual_coladd,
                };
            }
            return FAIL;
        } else if i == 0 && result == 0 {
            if forward {
                // Try again from start of buffer
                pos_lnum = 0;
                pos_col = 0;
                pos_coladd = 0;
            } else {
                // Try again from end of buffer
                let last_lnum = nvim_search_get_line_count();
                let last_col = nvim_search_ml_get_len(last_lnum);
                pos_lnum = last_lnum;
                pos_col = last_col;
                pos_coladd = 0;
            }
        }
    }

    let start_lnum = pos_lnum;
    let start_col = pos_col;
    let start_coladd = pos_coladd;

    if !visual_active {
        VIsual = crate::searchit::PosT {
            lnum: start_lnum,
            col: start_col,
            coladd: start_coladd,
        };
    }

    // Put cursor after the match
    nvim_set_curwin_cursor_lnum(end_lnum);
    nvim_set_curwin_cursor_col(end_col);
    nvim_set_curwin_cursor_coladd(end_coladd);

    let vis_lnum = VIsual.lnum;
    let vis_col = VIsual.col;
    let vis_coladd = VIsual.coladd;

    if lt_pos(vis_lnum, vis_col, vis_coladd, end_lnum, end_col, end_coladd) && forward {
        if skip_first_backward {
            // Put cursor on start of match
            nvim_set_curwin_cursor_lnum(pos_lnum);
            nvim_set_curwin_cursor_col(pos_col);
            nvim_set_curwin_cursor_coladd(pos_coladd);
        } else {
            // Put cursor on last character of match
            nvim_dec_cursor();
        }
    } else {
        let cur_lnum2 = nvim_get_curwin_cursor_lnum();
        let cur_col2 = nvim_get_curwin_cursor_col();
        let cur_coladd2 = nvim_search_get_curwin_cursor_coladd();
        if visual_active
            && lt_pos(
                cur_lnum2,
                cur_col2,
                cur_coladd2,
                vis_lnum,
                vis_col,
                vis_coladd,
            )
            && forward
        {
            nvim_set_curwin_cursor_lnum(pos_lnum);
            nvim_set_curwin_cursor_col(pos_col);
            nvim_set_curwin_cursor_coladd(pos_coladd);
        }
    }

    VIsual_active = true;
    VIsual_mode = b'v' as c_int;

    if nvim_get_p_sel_first() == b'e' as c_char {
        let cur_lnum3 = nvim_get_curwin_cursor_lnum();
        let cur_col3 = nvim_get_curwin_cursor_col();
        let cur_coladd3 = nvim_search_get_curwin_cursor_coladd();
        let vis_lnum2 = VIsual.lnum;
        let vis_col2 = VIsual.col;
        let vis_coladd2 = VIsual.coladd;
        if forward
            && ltoreq_pos(
                vis_lnum2,
                vis_col2,
                vis_coladd2,
                cur_lnum3,
                cur_col3,
                cur_coladd3,
            )
        {
            nvim_inc_cursor();
        } else if !forward
            && ltoreq_pos(
                cur_lnum3,
                cur_col3,
                cur_coladd3,
                vis_lnum2,
                vis_col2,
                vis_coladd2,
            )
        {
            let mut vl = vis_lnum2;
            let mut vc = vis_col2;
            let mut vca = vis_coladd2;
            nvim_search_incl_pos(&mut vl, &mut vc, &mut vca);
            VIsual = crate::searchit::PosT {
                lnum: vl,
                col: vc,
                coladd: vca,
            };
        }
    }

    if (fdo_flags & K_OPT_FDO_FLAG_SEARCH) != 0 && KeyTyped {
        rs_foldOpenCursor();
    }

    rs_may_start_select(b'c' as c_int);
    setmouse();
    nvim_redraw_curbuf_inverted();
    showmode();

    let _ = result;
    OK
}

/// Helper: lt(a, b) - true if position a < position b.
/// Positions are compared lnum first, then col, then coladd.
#[inline]
fn lt_pos(
    a_lnum: c_int,
    a_col: c_int,
    a_coladd: c_int,
    b_lnum: c_int,
    b_col: c_int,
    b_coladd: c_int,
) -> bool {
    a_lnum < b_lnum
        || (a_lnum == b_lnum && (a_col < b_col || (a_col == b_col && a_coladd < b_coladd)))
}

/// Helper: ltoreq(a, b) - true if position a <= position b.
#[inline]
fn ltoreq_pos(
    a_lnum: c_int,
    a_col: c_int,
    a_coladd: c_int,
    b_lnum: c_int,
    b_col: c_int,
    b_coladd: c_int,
) -> bool {
    !lt_pos(b_lnum, b_col, b_coladd, a_lnum, a_col, a_coladd)
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
