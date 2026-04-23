//! Cursor shape handling for Neovim
//!
//! Provides Rust implementations of cursor shape functions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::redundant_else)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)]
#![allow(unsafe_code)]

use std::ffi::{c_char, c_int};

/// Cursor shape types matching `CursorShape` enum in C
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    Block = 0,
    Hor = 1,
    Ver = 2,
}

/// Mode shape indices matching `ModeShape` enum in C
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeShape {
    N = 0,     // Normal mode
    V = 1,     // Visual mode
    I = 2,     // Insert mode
    R = 3,     // Replace mode
    C = 4,     // Command line Normal mode
    Ci = 5,    // Command line Insert mode
    Cr = 6,    // Command line Replace mode
    O = 7,     // Operator-pending mode
    Ve = 8,    // Visual mode with 'selection' exclusive
    Cline = 9, // On command line
    Status = 10,
    Sdrag = 11,
    Vsep = 12,
    Vdrag = 13,
    More = 14,
    Morel = 15,
    Sm = 16,   // showing matching paren
    Term = 17, // Terminal mode
    Count = 18,
}

// State mode flags from state_defs.h
const MODE_CMDLINE: c_int = 0x08;
const MODE_INSERT: c_int = 0x10;
const MODE_TERMINAL: c_int = 0x80;
const REPLACE_FLAG: c_int = 0x100;
const VREPLACE_FLAG: c_int = 0x200;
const MODE_SHOWMATCH: c_int = 0x6000 | MODE_INSERT;

// Shape usage flags from cursor_shape.h
#[allow(dead_code)]
const SHAPE_MOUSE: c_int = 1;
#[allow(dead_code)]
const SHAPE_CURSOR: c_int = 2;

extern "C" {
    /// Get the cursor shape for a mode index
    fn nvim_get_shape_table_shape(idx: c_int) -> c_int;
    /// Get the percentage for a mode index
    fn nvim_get_shape_table_percentage(idx: c_int) -> c_int;
    /// Get the blinkwait value for a mode index
    fn nvim_get_shape_table_blinkwait(idx: c_int) -> c_int;
    /// Get the blinkon value for a mode index
    fn nvim_get_shape_table_blinkon(idx: c_int) -> c_int;
    /// Get the blinkoff value for a mode index
    fn nvim_get_shape_table_blinkoff(idx: c_int) -> c_int;
    /// Get the highlight id for a mode index
    fn nvim_get_shape_table_id(idx: c_int) -> c_int;
    /// Get the langmap highlight id for a mode index
    fn nvim_get_shape_table_id_lm(idx: c_int) -> c_int;
    /// Check if guicursor option is empty
    fn nvim_is_guicursor_empty() -> c_int;
    /// Get current editor State
    fn nvim_get_state() -> c_int;
    /// Check if operator is pending
    static mut finish_op: bool;
    /// Check if Visual mode is active
    static mut VIsual_active: bool;
    /// Get first char of 'selection' option
    fn nvim_get_p_sel_first() -> c_char;
    /// Check if at end of command line
    fn rs_cmdline_at_end() -> c_int;
    /// Check if in overstrike mode on command line
    fn rs_cmdline_overstrike() -> c_int;
    /// Get the full name for a mode index
    fn nvim_get_shape_table_name(idx: c_int) -> *const c_char;
    /// Get the short name for a mode index
    fn nvim_get_shape_table_short_name(idx: c_int) -> *const c_char;
    /// Get the used_for flags for a mode index
    fn nvim_get_shape_table_used_for(idx: c_int) -> c_int;

    // Setter accessors for shape_table
    /// Set the cursor shape for a mode index
    fn nvim_set_shape_table_shape(idx: c_int, shape: c_int);
    /// Set the percentage for a mode index
    fn nvim_set_shape_table_percentage(idx: c_int, pct: c_int);
    /// Set the blinkwait value for a mode index
    fn nvim_set_shape_table_blinkwait(idx: c_int, val: c_int);
    /// Set the blinkon value for a mode index
    fn nvim_set_shape_table_blinkon(idx: c_int, val: c_int);
    /// Set the blinkoff value for a mode index
    fn nvim_set_shape_table_blinkoff(idx: c_int, val: c_int);
    /// Set the highlight id for a mode index
    fn nvim_set_shape_table_id(idx: c_int, id: c_int);
    /// Set the langmap highlight id for a mode index
    fn nvim_set_shape_table_id_lm(idx: c_int, id: c_int);

    // Additional accessors for parse_shape_opt
    /// Get the guicursor option value
    fn nvim_get_p_guicursor() -> *const c_char;
    /// Check/create highlight group by name
    fn nvim_syn_check_group(name: *const c_char, len: usize) -> c_int;
    /// Notify UI of mode info changes
    fn nvim_ui_mode_info_set();
}

/// Returns true if the cursor is non-blinking "block" shape during
/// visual selection.
///
/// # Safety
/// Calls C accessor functions for `shape_table`.
#[export_name = "cursor_is_block_during_visual"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_cursor_is_block_during_visual(exclusive: bool) -> bool {
    let mode_idx = if exclusive {
        ModeShape::Ve as c_int
    } else {
        ModeShape::V as c_int
    };

    let shape = nvim_get_shape_table_shape(mode_idx);
    let blinkon = nvim_get_shape_table_blinkon(mode_idx);

    shape == CursorShape::Block as c_int && blinkon == 0
}

/// Check if a syntax id is used as a cursor style.
///
/// # Safety
/// Calls C accessor functions for `shape_table` and `guicursor`.
#[export_name = "cursor_mode_uses_syn_id"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_cursor_mode_uses_syn_id(syn_id: c_int) -> bool {
    if nvim_is_guicursor_empty() != 0 {
        return false;
    }

    for mode_idx in 0..ModeShape::Count as c_int {
        let id = nvim_get_shape_table_id(mode_idx);
        let id_lm = nvim_get_shape_table_id_lm(mode_idx);
        if id == syn_id || id_lm == syn_id {
            return true;
        }
    }

    false
}

/// Return the index into shape_table[] for the current mode.
///
/// # Safety
/// Calls C accessor functions for global state.
#[must_use]
#[export_name = "cursor_get_mode_idx"]
pub unsafe extern "C" fn rs_cursor_get_mode_idx() -> c_int {
    let state = nvim_get_state();

    if state == MODE_SHOWMATCH {
        return ModeShape::Sm as c_int;
    }
    if state == MODE_TERMINAL {
        return ModeShape::Term as c_int;
    }
    if (state & VREPLACE_FLAG) != 0 {
        return ModeShape::R as c_int;
    }
    if (state & REPLACE_FLAG) != 0 {
        return ModeShape::R as c_int;
    }
    if (state & MODE_INSERT) != 0 {
        return ModeShape::I as c_int;
    }
    if (state & MODE_CMDLINE) != 0 {
        if rs_cmdline_at_end() != 0 {
            return ModeShape::C as c_int;
        } else if rs_cmdline_overstrike() != 0 {
            return ModeShape::Cr as c_int;
        } else {
            return ModeShape::Ci as c_int;
        }
    }
    if finish_op {
        return ModeShape::O as c_int;
    }
    if VIsual_active {
        if nvim_get_p_sel_first() == b'e' as c_char {
            return ModeShape::Ve as c_int;
        }
        return ModeShape::V as c_int;
    }

    ModeShape::N as c_int
}

/// Convert a mode name string to its index in shape_table.
///
/// # Safety
/// - `mode` must be a valid, NUL-terminated C string.
/// - Calls C accessor function for `shape_table`.
///
/// # Returns
/// The mode index (0-17) if found, or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_mode_str2int(mode: *const c_char) -> c_int {
    use std::ffi::CStr;

    if mode.is_null() {
        return -1;
    }

    let mode_str = CStr::from_ptr(mode);

    for mode_idx in 0..ModeShape::Count as c_int {
        let name_ptr = nvim_get_shape_table_name(mode_idx);
        if !name_ptr.is_null() {
            let name = CStr::from_ptr(name_ptr);
            if mode_str == name {
                return mode_idx;
            }
        }
    }

    -1
}

/// Clear all entries in shape_table to block, blinkon0, and default color.
///
/// # Safety
/// Calls C setter accessor functions for `shape_table`.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_shape_table() {
    for idx in 0..ModeShape::Count as c_int {
        nvim_set_shape_table_shape(idx, CursorShape::Block as c_int);
        nvim_set_shape_table_blinkwait(idx, 0);
        nvim_set_shape_table_blinkon(idx, 0);
        nvim_set_shape_table_blinkoff(idx, 0);
        nvim_set_shape_table_id(idx, 0);
        nvim_set_shape_table_id_lm(idx, 0);
    }
}

// Error messages for parse_shape_opt (must match C exactly for gettext)
const E545_MISSING_COLON: &[u8] = b"E545: Missing colon\0";
const E546_ILLEGAL_MODE: &[u8] = b"E546: Illegal mode\0";
const E548_DIGIT_EXPECTED: &[u8] = b"E548: Digit expected\0";
const E549_ILLEGAL_PERCENTAGE: &[u8] = b"E549: Illegal percentage\0";

/// Helper to do case-insensitive comparison of byte slices
fn strnicmp(a: &[u8], b: &[u8], len: usize) -> bool {
    if a.len() < len || b.len() < len {
        return false;
    }
    for i in 0..len {
        if !a[i].eq_ignore_ascii_case(&b[i]) {
            return false;
        }
    }
    true
}

/// Parse digits from a byte slice, returning the number and new position
fn getdigits(bytes: &[u8], pos: usize) -> (c_int, usize) {
    let mut n: c_int = 0;
    let mut i = pos;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        n = n
            .saturating_mul(10)
            .saturating_add(c_int::from(bytes[i] - b'0'));
        i += 1;
    }
    (n, i)
}

/// Find a character in a byte slice starting at position, returns index or None
fn find_char(bytes: &[u8], start: usize, ch: u8) -> Option<usize> {
    (start..bytes.len()).find(|&i| bytes[i] == ch)
}

/// Copy settings from one shape table index to another
unsafe fn copy_shape_settings(from: c_int, to: c_int) {
    nvim_set_shape_table_shape(to, nvim_get_shape_table_shape(from));
    nvim_set_shape_table_percentage(to, nvim_get_shape_table_percentage(from));
    nvim_set_shape_table_blinkwait(to, nvim_get_shape_table_blinkwait(from));
    nvim_set_shape_table_blinkon(to, nvim_get_shape_table_blinkon(from));
    nvim_set_shape_table_blinkoff(to, nvim_get_shape_table_blinkoff(from));
    nvim_set_shape_table_id(to, nvim_get_shape_table_id(from));
    nvim_set_shape_table_id_lm(to, nvim_get_shape_table_id_lm(from));
}

/// Parse the 'guicursor' option.
///
/// Clears `shape_table` if 'guicursor' is empty.
///
/// # Arguments
/// * `what` - SHAPE_CURSOR or SHAPE_MOUSE ('mouseshape')
///
/// # Returns
/// Error message for an illegal option, NULL otherwise.
///
/// # Safety
/// Calls C accessor functions for `shape_table` and global state.
#[must_use]
#[export_name = "parse_shape_opt"]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_parse_shape_opt(what: c_int) -> *const c_char {
    use std::ffi::CStr;

    let guicursor_ptr = nvim_get_p_guicursor();
    if guicursor_ptr.is_null() {
        rs_clear_shape_table();
        nvim_ui_mode_info_set();
        return std::ptr::null();
    }

    let guicursor = CStr::from_ptr(guicursor_ptr).to_bytes();

    // Empty guicursor - just clear and return
    if guicursor.is_empty() {
        rs_clear_shape_table();
        nvim_ui_mode_info_set();
        return std::ptr::null();
    }

    let mut found_ve = false;

    // First round: check for errors; second round: do it for real.
    for round in 1..=2 {
        if round == 2 {
            // Set all entries to default before applying
            rs_clear_shape_table();
        }

        // Repeat for all comma separated parts
        let mut modep: usize = 0;

        while modep < guicursor.len() {
            // Find colon and comma positions
            let colonp = find_char(guicursor, modep, b':');
            let commap = find_char(guicursor, modep, b',');

            // Check for missing or misplaced colon
            let colon_pos = match colonp {
                Some(pos) => {
                    // If comma exists and comes before colon, error
                    if let Some(comma_pos) = commap {
                        if comma_pos < pos {
                            return E545_MISSING_COLON.as_ptr().cast::<c_char>();
                        }
                    }
                    pos
                }
                None => {
                    return E545_MISSING_COLON.as_ptr().cast::<c_char>();
                }
            };

            // Colon at start means empty mode
            if colon_pos == modep {
                return E546_ILLEGAL_MODE.as_ptr().cast::<c_char>();
            }

            // Repeat for all modes before the colon
            // For the 'a' mode, we loop to handle all the modes
            let mut all_idx: c_int = -1;
            let mut mode_pos = modep;
            let mut idx: c_int = 0; // Will be set in the loop
            let mut p: usize = colon_pos + 1; // Will be set in the loop

            while mode_pos < colon_pos || all_idx >= 0 {
                if all_idx < 0 {
                    // Find the mode length
                    let len = if mode_pos + 1 < guicursor.len()
                        && (guicursor[mode_pos + 1] == b'-' || guicursor[mode_pos + 1] == b':')
                    {
                        1
                    } else {
                        2
                    };

                    // Check for 'a' (all) mode
                    if len == 1 && guicursor[mode_pos].eq_ignore_ascii_case(&b'a') {
                        all_idx = ModeShape::Count as c_int - 1;
                    } else {
                        // Find matching mode in shape_table
                        idx = -1;
                        for i in 0..ModeShape::Count as c_int {
                            let name_ptr = nvim_get_shape_table_short_name(i);
                            if !name_ptr.is_null() {
                                let name = CStr::from_ptr(name_ptr).to_bytes();
                                if strnicmp(&guicursor[mode_pos..], name, len) {
                                    idx = i;
                                    break;
                                }
                            }
                        }

                        if idx < 0 || (nvim_get_shape_table_used_for(idx) & what) == 0 {
                            return E546_ILLEGAL_MODE.as_ptr().cast::<c_char>();
                        }

                        // Check if this is the 've' mode
                        if len == 2
                            && guicursor[mode_pos] == b'v'
                            && guicursor[mode_pos + 1] == b'e'
                        {
                            found_ve = true;
                        }
                    }
                    mode_pos += len + 1; // skip mode and separator
                }

                if all_idx >= 0 {
                    idx = all_idx;
                    all_idx -= 1;
                }
                // Otherwise idx was set above when we found the mode

                // Parse the part after the colon
                p = colon_pos + 1;
                while p < guicursor.len() && guicursor[p] != b',' {
                    // First handle the ones with a number argument
                    let first_char = guicursor[p];
                    let keyword_len;

                    if strnicmp(&guicursor[p..], b"ver", 3) || strnicmp(&guicursor[p..], b"hor", 3)
                    {
                        keyword_len = 3;
                    } else if strnicmp(&guicursor[p..], b"blinkwait", 9) {
                        keyword_len = 9;
                    } else if strnicmp(&guicursor[p..], b"blinkon", 7) {
                        keyword_len = 7;
                    } else if strnicmp(&guicursor[p..], b"blinkoff", 8) {
                        keyword_len = 8;
                    } else {
                        keyword_len = 0;
                    }

                    if keyword_len != 0 {
                        p += keyword_len;
                        if p >= guicursor.len() || !guicursor[p].is_ascii_digit() {
                            return E548_DIGIT_EXPECTED.as_ptr().cast::<c_char>();
                        }
                        let (n, new_p) = getdigits(guicursor, p);
                        p = new_p;

                        if keyword_len == 3 {
                            // "ver" or "hor"
                            if n == 0 {
                                return E549_ILLEGAL_PERCENTAGE.as_ptr().cast::<c_char>();
                            }
                            if round == 2 {
                                if first_char.eq_ignore_ascii_case(&b'v') {
                                    nvim_set_shape_table_shape(idx, CursorShape::Ver as c_int);
                                } else {
                                    nvim_set_shape_table_shape(idx, CursorShape::Hor as c_int);
                                }
                                nvim_set_shape_table_percentage(idx, n);
                            }
                        } else if round == 2 {
                            if keyword_len == 9 {
                                nvim_set_shape_table_blinkwait(idx, n);
                            } else if keyword_len == 7 {
                                nvim_set_shape_table_blinkon(idx, n);
                            } else {
                                nvim_set_shape_table_blinkoff(idx, n);
                            }
                        }
                    } else if strnicmp(&guicursor[p..], b"block", 5) {
                        if round == 2 {
                            nvim_set_shape_table_shape(idx, CursorShape::Block as c_int);
                        }
                        p += 5;
                    } else {
                        // Must be a highlight group name
                        let dash_pos = find_char(guicursor, p, b'-');
                        let end_pos = commap.map_or_else(
                            || dash_pos.unwrap_or(guicursor.len()),
                            |comma_pos| {
                                dash_pos.map_or(comma_pos, |d| {
                                    if d < comma_pos {
                                        d
                                    } else {
                                        comma_pos
                                    }
                                })
                            },
                        );

                        let slash_pos = find_char(guicursor, p, b'/');
                        let mut group_id = 0;

                        if let Some(slash) = slash_pos {
                            if slash < end_pos {
                                // "group/langmap_group"
                                group_id = nvim_syn_check_group(
                                    guicursor[p..].as_ptr().cast::<c_char>(),
                                    slash - p,
                                );
                                p = slash + 1;
                            }
                        }

                        if round == 2 {
                            let id = nvim_syn_check_group(
                                guicursor[p..].as_ptr().cast::<c_char>(),
                                end_pos - p,
                            );
                            nvim_set_shape_table_id(idx, id);
                            nvim_set_shape_table_id_lm(idx, id);
                            if slash_pos.is_some_and(|s| s < end_pos) {
                                nvim_set_shape_table_id(idx, group_id);
                            }
                        }
                        p = end_pos;
                    }

                    if p < guicursor.len() && guicursor[p] == b'-' {
                        p += 1;
                    }
                }

                // For non-'a' mode, we only process once per mode in the list
                if all_idx < 0 {
                    break;
                }
            }

            // Move to next comma-separated part
            modep = p;
            if modep < guicursor.len() && guicursor[modep] == b',' {
                modep += 1;
            }
        }
    }

    // If the 've' flag is not given, use the 'v' cursor for 've'
    if !found_ve {
        copy_shape_settings(ModeShape::V as c_int, ModeShape::Ve as c_int);
    }

    nvim_ui_mode_info_set();
    std::ptr::null()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_shape_values() {
        // Verify CursorShape enum values match C definitions
        assert_eq!(CursorShape::Block as c_int, 0);
        assert_eq!(CursorShape::Hor as c_int, 1);
        assert_eq!(CursorShape::Ver as c_int, 2);
    }

    #[test]
    fn test_mode_shape_values() {
        // Verify key ModeShape enum values match C definitions
        assert_eq!(ModeShape::N as c_int, 0); // Normal
        assert_eq!(ModeShape::V as c_int, 1); // Visual
        assert_eq!(ModeShape::I as c_int, 2); // Insert
        assert_eq!(ModeShape::R as c_int, 3); // Replace
        assert_eq!(ModeShape::C as c_int, 4); // Command line Normal
        assert_eq!(ModeShape::Term as c_int, 17); // Terminal
        assert_eq!(ModeShape::Count as c_int, 18); // Total modes
    }

    #[test]
    fn test_mode_shape_all_values() {
        // Verify all ModeShape enum values are sequential
        assert_eq!(ModeShape::N as c_int, 0);
        assert_eq!(ModeShape::V as c_int, 1);
        assert_eq!(ModeShape::I as c_int, 2);
        assert_eq!(ModeShape::R as c_int, 3);
        assert_eq!(ModeShape::C as c_int, 4);
        assert_eq!(ModeShape::Ci as c_int, 5);
        assert_eq!(ModeShape::Cr as c_int, 6);
        assert_eq!(ModeShape::O as c_int, 7);
        assert_eq!(ModeShape::Ve as c_int, 8);
        assert_eq!(ModeShape::Cline as c_int, 9);
        assert_eq!(ModeShape::Status as c_int, 10);
        assert_eq!(ModeShape::Sdrag as c_int, 11);
        assert_eq!(ModeShape::Vsep as c_int, 12);
        assert_eq!(ModeShape::Vdrag as c_int, 13);
        assert_eq!(ModeShape::More as c_int, 14);
        assert_eq!(ModeShape::Morel as c_int, 15);
        assert_eq!(ModeShape::Sm as c_int, 16);
        assert_eq!(ModeShape::Term as c_int, 17);
        assert_eq!(ModeShape::Count as c_int, 18);
    }

    #[test]
    fn test_mode_flags() {
        // Verify mode flag constants match C definitions
        assert_eq!(MODE_CMDLINE, 0x08);
        assert_eq!(MODE_INSERT, 0x10);
        assert_eq!(MODE_TERMINAL, 0x80);
        assert_eq!(REPLACE_FLAG, 0x100);
        assert_eq!(VREPLACE_FLAG, 0x200);
        // MODE_SHOWMATCH should combine flags
        assert_eq!(MODE_SHOWMATCH, 0x6000 | MODE_INSERT);
    }

    #[test]
    fn test_mode_flags_distinct() {
        // Mode flags should be distinct bit patterns
        assert_eq!(MODE_CMDLINE & MODE_INSERT, 0);
        assert_eq!(MODE_INSERT & MODE_TERMINAL, 0);
        assert_eq!(MODE_TERMINAL & REPLACE_FLAG, 0);
        assert_eq!(REPLACE_FLAG & VREPLACE_FLAG, 0);
    }

    #[test]
    fn test_enum_sizes() {
        // CursorShape and ModeShape should be C-compatible
        assert_eq!(
            std::mem::size_of::<CursorShape>(),
            std::mem::size_of::<c_int>()
        );
        assert_eq!(
            std::mem::size_of::<ModeShape>(),
            std::mem::size_of::<c_int>()
        );
    }

    #[test]
    fn test_mode_shape_count_matches_variant_count() {
        // Count should equal the number of actual mode variants
        let modes = [
            ModeShape::N,
            ModeShape::V,
            ModeShape::I,
            ModeShape::R,
            ModeShape::C,
            ModeShape::Ci,
            ModeShape::Cr,
            ModeShape::O,
            ModeShape::Ve,
            ModeShape::Cline,
            ModeShape::Status,
            ModeShape::Sdrag,
            ModeShape::Vsep,
            ModeShape::Vdrag,
            ModeShape::More,
            ModeShape::Morel,
            ModeShape::Sm,
            ModeShape::Term,
        ];
        assert_eq!(modes.len(), ModeShape::Count as usize);
    }

    #[test]
    fn test_cursor_shape_distinct() {
        // All cursor shapes should have distinct values
        let shapes = [CursorShape::Block, CursorShape::Hor, CursorShape::Ver];
        for (i, &shape_a) in shapes.iter().enumerate() {
            for (j, &shape_b) in shapes.iter().enumerate() {
                if i != j {
                    assert_ne!(shape_a as c_int, shape_b as c_int);
                }
            }
        }
    }
}
