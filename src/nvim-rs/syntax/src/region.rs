//! Region handling for syntax highlighting.
//!
//! This module handles:
//! - Region start/skip/end pattern matching
//! - Region nesting logic
//! - Matchgroup handling
//! - find_endpos and update_si_end implementations

use std::ffi::{c_char, c_int};

use crate::offset::{limit_pos, syn_add_end_off, RegMatch};
use crate::state::Position;
use crate::types::{
    ExtMatchHandle, StateItemHandle, SynPatHandle, HL_ONELINE, SPO_COUNT, SPO_HE_OFF, SPO_LC_OFF,
    SPO_ME_OFF, SPO_RE_OFF, SPTYPE_END, SPTYPE_MATCH, SPTYPE_SKIP, SPTYPE_START,
};

// =============================================================================
// FFI declarations for region operations
// =============================================================================

extern "C" {
    // Pattern type accessors (from syntax.c)
    fn nvim_synpat_get_type(pat: SynPatHandle) -> c_int;
    fn nvim_syn_get_pattern_type(idx: c_int) -> c_int;

    // Matchgroup accessors
    fn nvim_synpat_get_syn_match_id(pat: SynPatHandle) -> i16;
    fn nvim_syn_get_pattern_syn_match_id(idx: c_int) -> c_int;

    // State item region accessors
    fn nvim_stateitem_get_end_idx(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_set_end_idx(item: StateItemHandle, end_idx: c_int);
    // Bulk position setter (pass c_int::MIN to skip a field)
    #[allow(clippy::too_many_arguments)]
    fn nvim_stateitem_set_positions(
        item: StateItemHandle,
        m_lnum: c_int,
        m_startcol: c_int,
        m_end_lnum: c_int,
        m_end_col: c_int,
        h_start_lnum: c_int,
        h_start_col: c_int,
        h_end_lnum: c_int,
        h_end_col: c_int,
        eoe_lnum: c_int,
        eoe_col: c_int,
    );
    #[allow(clippy::too_many_arguments)]
    fn nvim_stateitem_get_positions(
        item: StateItemHandle,
        m_lnum: *mut c_int,
        m_startcol: *mut c_int,
        m_end_lnum: *mut c_int,
        m_end_col: *mut c_int,
        h_start_lnum: *mut c_int,
        h_start_col: *mut c_int,
        h_end_lnum: *mut c_int,
        h_end_col: *mut c_int,
        eoe_lnum: *mut c_int,
        eoe_col: *mut c_int,
    );

    // Pattern accessors for find_endpos
    fn nvim_syn_get_synblock_pattern_count() -> c_int;
    fn nvim_syn_get_pattern_flags(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_syn_id(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_offset(pat_idx: c_int, off_idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_off_flags(pat_idx: c_int) -> c_int;

    // (nvim_syn_regexec_pat replaced by crate::regexec::syn_regexec_pat)

    // Extmatch management
    fn nvim_syn_set_extmatch_in(em: ExtMatchHandle);
    fn nvim_syn_clear_extmatch_in();

    // Chartab management
    fn nvim_syn_save_chartab(buf: *mut c_char);
    fn nvim_syn_restore_chartab(buf: *mut c_char);

    // Line operations
    fn nvim_syn_getcurline_len() -> c_int;
    fn nvim_syn_get_line_len(lnum: c_int) -> c_int;
    fn nvim_syn_get_current_lnum() -> c_int;

    // State item accessors for update_si_end
    fn nvim_stateitem_get_idx(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_flags(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_get_extmatch(item: StateItemHandle) -> ExtMatchHandle;
    fn nvim_stateitem_set_ends(item: StateItemHandle, ends: c_int);
}

const SKIP: c_int = c_int::MIN;

// =============================================================================
// Pattern type enumeration
// =============================================================================

/// Type of syntax pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    /// Match pattern (single match).
    Match,
    /// Start pattern (beginning of a region).
    Start,
    /// End pattern (end of a region).
    End,
    /// Skip pattern (skip within a region).
    Skip,
    /// Unknown pattern type.
    Unknown(i32),
}

impl PatternType {
    /// Create from a C integer constant.
    #[must_use]
    pub const fn from_c_int(val: i32) -> Self {
        match val {
            SPTYPE_MATCH => Self::Match,
            SPTYPE_START => Self::Start,
            SPTYPE_END => Self::End,
            SPTYPE_SKIP => Self::Skip,
            _ => Self::Unknown(val),
        }
    }

    /// Convert to a C integer constant.
    #[must_use]
    pub const fn to_c_int(self) -> i32 {
        match self {
            Self::Match => SPTYPE_MATCH,
            Self::Start => SPTYPE_START,
            Self::End => SPTYPE_END,
            Self::Skip => SPTYPE_SKIP,
            Self::Unknown(val) => val,
        }
    }

    /// Check if this is a region pattern type (start, end, or skip).
    #[must_use]
    pub const fn is_region_type(self) -> bool {
        matches!(self, Self::Start | Self::End | Self::Skip)
    }

    /// Check if this is the start of a region.
    #[must_use]
    pub const fn is_start(self) -> bool {
        matches!(self, Self::Start)
    }

    /// Check if this is the end of a region.
    #[must_use]
    pub const fn is_end(self) -> bool {
        matches!(self, Self::End)
    }

    /// Check if this is a skip pattern.
    #[must_use]
    pub const fn is_skip(self) -> bool {
        matches!(self, Self::Skip)
    }
}

// =============================================================================
// Pattern type accessors
// =============================================================================

/// Get the pattern type for a synpat_T.
#[must_use]
pub fn synpat_type(pat: SynPatHandle) -> PatternType {
    if pat.is_null() {
        return PatternType::Unknown(0);
    }
    PatternType::from_c_int(unsafe { nvim_synpat_get_type(pat) })
}

/// Get the pattern type for the current synblock's pattern by index.
#[must_use]
pub fn pattern_type_at_idx(idx: i32) -> PatternType {
    PatternType::from_c_int(unsafe { nvim_syn_get_pattern_type(idx) })
}

// =============================================================================
// Region end position finding — Rust implementation
// =============================================================================

/// Result of finding a region end position.
#[derive(Debug, Clone, Copy, Default)]
pub struct RegionEndResult {
    /// End position of the match.
    pub m_endpos: Position,
    /// End position of the highlighting.
    pub hl_endpos: Position,
    /// End position of the end pattern itself.
    pub end_endpos: Position,
    /// Flags from the matching.
    pub flags: i32,
    /// Index of the end pattern that matched (or 0).
    pub end_idx: i32,
}

/// Helper: execute regex on a pattern at idx, returning match positions.
fn regexec_pat(idx: i32, lnum: i32, col: i32) -> Option<RegMatch> {
    let mut sl: c_int = 0;
    let mut sc: c_int = 0;
    let mut el: c_int = 0;
    let mut ec: c_int = 0;
    let r = unsafe {
        crate::regexec::syn_regexec_pat(idx, lnum, col, &mut sl, &mut sc, &mut el, &mut ec)
    };
    if r != 0 {
        Some(RegMatch {
            startpos: Position { lnum: sl, col: sc },
            endpos: Position { lnum: el, col: ec },
        })
    } else {
        None
    }
}

/// Get pattern type at index from current synblock.
fn pat_type_at(idx: i32) -> i32 {
    unsafe { nvim_syn_get_pattern_type(idx) }
}

/// Find the end of a start/skip/end syntax region after startpos.
/// Only checks one line.
///
/// This is the Rust implementation of the C find_endpos() function.
///
/// # Safety
/// Accesses global C state (syn_block, syn_buf, re_extmatch_in).
pub unsafe fn find_endpos(
    mut idx: i32,
    startpos: &Position,
    start_ext: ExtMatchHandle,
) -> RegionEndResult {
    let mut result = RegionEndResult::default();

    // Just in case invoked for a keyword
    if idx < 0 {
        return result;
    }

    // Check for being called with a START pattern.
    if pat_type_at(idx) != SPTYPE_START {
        result.hl_endpos = *startpos;
        return result;
    }

    // Find the SKIP or first END pattern after the last START pattern.
    let pat_count = nvim_syn_get_synblock_pattern_count();
    while idx < pat_count && pat_type_at(idx) == SPTYPE_START {
        idx += 1;
    }

    // Lookup the SKIP pattern (if present)
    let spp_skip_idx = if idx < pat_count && pat_type_at(idx) == SPTYPE_SKIP {
        let skip = idx;
        idx += 1;
        Some(skip)
    } else {
        None
    };

    // Setup external matches for syn_regexec
    nvim_syn_set_extmatch_in(start_ext);

    let mut matchcol = startpos.col;
    let start_idx = idx; // Remember the first END pattern

    // Use syntax iskeyword option
    let mut buf_chartab = [0i8; 32];
    nvim_syn_save_chartab(buf_chartab.as_mut_ptr());

    let mut had_match = false;

    loop {
        // Find end pattern that matches first after matchcol.
        let mut best_idx: i32 = -1;
        let mut best_regmatch = RegMatch::default();

        let mut eidx = start_idx;
        while eidx < pat_count {
            if pat_type_at(eidx) != SPTYPE_END {
                break; // Past last END pattern
            }

            let lc_off = nvim_syn_get_pattern_offset(eidx, SPO_LC_OFF);
            let lc_col = (matchcol - lc_off).max(0);

            if let Some(m) = regexec_pat(eidx, startpos.lnum, lc_col) {
                if best_idx == -1 || m.startpos.col < best_regmatch.startpos.col {
                    best_idx = eidx;
                    best_regmatch.startpos = m.startpos;
                    best_regmatch.endpos = m.endpos;
                }
            }
            eidx += 1;
        }

        // If all end patterns have been tried and there is no match,
        // the item continues until end-of-line.
        if best_idx == -1 {
            break;
        }

        // If the skip pattern matches before the end pattern, continue
        // searching after the skip pattern.
        if let Some(skip_idx) = spp_skip_idx {
            let lc_off = nvim_syn_get_pattern_offset(skip_idx, SPO_LC_OFF);
            let lc_col = (matchcol - lc_off).max(0);

            if let Some(skip_match) = regexec_pat(skip_idx, startpos.lnum, lc_col) {
                if skip_match.startpos.col <= best_regmatch.startpos.col {
                    // Add offset to skip pattern match
                    let pos = syn_add_end_off(
                        &RegMatch {
                            startpos: skip_match.startpos,
                            endpos: skip_match.endpos,
                        },
                        skip_idx,
                        SPO_ME_OFF,
                        1,
                    );

                    // If the skip pattern goes on to the next line, no match with
                    // an end pattern in this line.
                    if pos.lnum > startpos.lnum {
                        break;
                    }

                    let line_len = nvim_syn_get_line_len(startpos.lnum);

                    // Take care of an empty match or negative offset
                    if pos.col <= matchcol {
                        matchcol += 1;
                    } else if pos.col <= skip_match.endpos.col {
                        matchcol = pos.col;
                    } else {
                        // Be careful not to jump over the NUL at the end-of-line
                        matchcol = skip_match.endpos.col;
                        while matchcol < line_len && matchcol < pos.col {
                            matchcol += 1;
                        }
                    }

                    // If the skip pattern includes end-of-line, break here
                    if matchcol >= line_len {
                        break;
                    }

                    continue; // Start with first end pattern again
                }
            }
        }

        // Match from start pattern to end pattern.
        // Correct for match and highlight offset of end pattern.
        result.m_endpos = syn_add_end_off(&best_regmatch, best_idx, SPO_ME_OFF, 1);
        // Can't end before the start
        if result.m_endpos.lnum == startpos.lnum && result.m_endpos.col < startpos.col {
            result.m_endpos.col = startpos.col;
        }

        result.end_endpos = syn_add_end_off(&best_regmatch, best_idx, SPO_HE_OFF, 1);
        // Can't end before the start
        if result.end_endpos.lnum == startpos.lnum && result.end_endpos.col < startpos.col {
            result.end_endpos.col = startpos.col;
        }
        // Can't end after the match
        limit_pos(&mut result.end_endpos, &result.m_endpos);

        // If the end group is highlighted differently, adjust the pointers.
        let best_match_id = nvim_syn_get_pattern_syn_match_id(best_idx);
        let best_syn_id = nvim_syn_get_pattern_syn_id(best_idx);
        if best_match_id != best_syn_id && best_match_id != 0 {
            result.end_idx = best_idx;
            let off_flags = nvim_syn_get_pattern_off_flags(best_idx);
            if off_flags & (1 << (SPO_RE_OFF + SPO_COUNT)) != 0 {
                result.hl_endpos.lnum = best_regmatch.endpos.lnum;
                result.hl_endpos.col = best_regmatch.endpos.col;
            } else {
                result.hl_endpos.lnum = best_regmatch.startpos.lnum;
                result.hl_endpos.col = best_regmatch.startpos.col;
            }
            result.hl_endpos.col += nvim_syn_get_pattern_offset(best_idx, SPO_RE_OFF);

            // Can't end before the start
            if result.hl_endpos.lnum == startpos.lnum && result.hl_endpos.col < startpos.col {
                result.hl_endpos.col = startpos.col;
            }
            limit_pos(&mut result.hl_endpos, &result.m_endpos);

            // Now the match ends where the highlighting ends, it is turned
            // into the matchgroup for the end
            result.m_endpos = result.hl_endpos;
        } else {
            result.end_idx = 0;
            result.hl_endpos = result.end_endpos;
        }

        result.flags = nvim_syn_get_pattern_flags(best_idx);

        had_match = true;
        break;
    }

    // No match for an END pattern in this line
    if !had_match {
        result.m_endpos.lnum = 0;
    }

    nvim_syn_restore_chartab(buf_chartab.as_mut_ptr());

    // Remove external matches
    nvim_syn_clear_extmatch_in();

    result
}

/// Update an entry in the current_state stack for a start-skip-end pattern.
/// This finds the end of the current item, if it's in the current line.
///
/// This is the Rust implementation of the C update_si_end() function.
///
/// # Safety
/// Accesses global C state.
pub unsafe fn update_si_end(sip: StateItemHandle, startcol: i32, force: bool) {
    if sip.is_null() {
        return;
    }

    let si_idx = nvim_stateitem_get_idx(sip);

    // Return quickly for a keyword
    if si_idx < 0 {
        return;
    }

    let current_lnum = nvim_syn_get_current_lnum();

    // Don't update when it's already done
    let mut m_end_lnum_check: c_int = 0;
    nvim_stateitem_get_positions(
        sip,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut m_end_lnum_check,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if !force && m_end_lnum_check >= current_lnum {
        return;
    }

    // Find the end of the region
    let startpos = Position {
        lnum: current_lnum,
        col: startcol,
    };
    let extmatch = nvim_stateitem_get_extmatch(sip);
    let result = find_endpos(si_idx, &startpos, extmatch);

    if result.m_endpos.lnum == 0 {
        // No end pattern matched
        let pat_flags = nvim_syn_get_pattern_flags(si_idx);
        let line_len = nvim_syn_getcurline_len();
        if pat_flags & HL_ONELINE != 0 {
            // A "oneline" never continues in the next line
            nvim_stateitem_set_ends(sip, 1);
            nvim_stateitem_set_positions(
                sip,
                SKIP,
                SKIP,
                current_lnum,
                line_len,
                SKIP,
                SKIP,
                current_lnum,
                line_len,
                SKIP,
                SKIP,
            );
        } else {
            // Continues in the next line
            nvim_stateitem_set_ends(sip, 0);
            nvim_stateitem_set_positions(sip, SKIP, SKIP, 0, 0, SKIP, SKIP, 0, 0, SKIP, SKIP);
        }
    } else {
        // Match within this line
        nvim_stateitem_set_positions(
            sip,
            SKIP,
            SKIP,
            result.m_endpos.lnum,
            result.m_endpos.col,
            SKIP,
            SKIP,
            result.hl_endpos.lnum,
            result.hl_endpos.col,
            result.end_endpos.lnum,
            result.end_endpos.col,
        );
        nvim_stateitem_set_ends(sip, 1);
        nvim_stateitem_set_end_idx(sip, result.end_idx);
    }
}

// =============================================================================
// Exported FFI: rs_find_endpos
// =============================================================================

/// Rust implementation of find_endpos, callable from C.
/// Replaces the C find_endpos() function.
#[no_mangle]
pub unsafe extern "C" fn rs_find_endpos(
    idx: c_int,
    start_lnum: c_int,
    start_col: c_int,
    m_endpos_lnum: *mut c_int,
    m_endpos_col: *mut c_int,
    hl_endpos_lnum: *mut c_int,
    hl_endpos_col: *mut c_int,
    flagsp: *mut c_int,
    end_endpos_lnum: *mut c_int,
    end_endpos_col: *mut c_int,
    end_idx: *mut c_int,
    start_ext: ExtMatchHandle,
) {
    let startpos = Position {
        lnum: start_lnum,
        col: start_col,
    };
    let result = find_endpos(idx, &startpos, start_ext);

    *m_endpos_lnum = result.m_endpos.lnum;
    *m_endpos_col = result.m_endpos.col;
    *hl_endpos_lnum = result.hl_endpos.lnum;
    *hl_endpos_col = result.hl_endpos.col;
    *flagsp = result.flags;
    *end_endpos_lnum = result.end_endpos.lnum;
    *end_endpos_col = result.end_endpos.col;
    *end_idx = result.end_idx;
}

/// Rust implementation of update_si_end, callable from C.
#[no_mangle]
pub unsafe extern "C" fn rs_update_si_end(sip: StateItemHandle, startcol: c_int, force: c_int) {
    update_si_end(sip, startcol, force != 0);
}

// =============================================================================
// Matchgroup handling
// =============================================================================

/// Get the matchgroup ID for a pattern.
#[must_use]
pub fn synpat_matchgroup_id(pat: SynPatHandle) -> i16 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_syn_match_id(pat) }
}

/// Get the matchgroup ID for a pattern by index.
#[must_use]
pub fn matchgroup_id_at_idx(idx: i32) -> i32 {
    unsafe { nvim_syn_get_pattern_syn_match_id(idx) }
}

// =============================================================================
// State item region accessors
// =============================================================================

/// Get the end pattern index for a state item.
#[must_use]
pub fn stateitem_end_idx(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_end_idx(item) }
}

/// Get the end-of-end position for a state item.
#[must_use]
pub fn stateitem_eoe_pos(item: StateItemHandle) -> Position {
    if item.is_null() {
        return Position::default();
    }
    let mut lnum: c_int = 0;
    let mut col: c_int = 0;
    unsafe {
        nvim_stateitem_get_positions(
            item,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut lnum,
            &mut col,
        );
    }
    Position { lnum, col }
}

/// Set the end-of-end position for a state item.
pub unsafe fn set_stateitem_eoe_pos(item: StateItemHandle, pos: Position) {
    if !item.is_null() {
        nvim_stateitem_set_positions(
            item, SKIP, SKIP, SKIP, SKIP, SKIP, SKIP, SKIP, SKIP, pos.lnum, pos.col,
        );
    }
}

/// Set the end pattern index for a state item.
pub unsafe fn set_stateitem_end_idx(item: StateItemHandle, end_idx: i32) {
    if !item.is_null() {
        nvim_stateitem_set_end_idx(item, end_idx);
    }
}

// =============================================================================
// Region state helpers
// =============================================================================

/// Check if a state item represents a region (has a start pattern type).
#[must_use]
pub fn stateitem_is_region(item: StateItemHandle, si_idx: i32) -> bool {
    if item.is_null() || si_idx < 0 {
        return false;
    }
    pattern_type_at_idx(si_idx).is_start()
}

/// Check if a state item has a pending end pattern match.
#[must_use]
pub fn stateitem_has_end_match(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    stateitem_end_idx(item) != 0
}

/// Find the end position of a region (convenience wrapper for existing callers).
///
/// # Safety
/// Accesses C global state.
#[must_use]
pub unsafe fn find_region_end(
    idx: i32,
    start: Position,
    start_ext: ExtMatchHandle,
) -> RegionEndResult {
    find_endpos(idx, &start, start_ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_type() {
        assert_eq!(PatternType::from_c_int(SPTYPE_MATCH), PatternType::Match);
        assert_eq!(PatternType::from_c_int(SPTYPE_START), PatternType::Start);
        assert_eq!(PatternType::from_c_int(SPTYPE_END), PatternType::End);
        assert_eq!(PatternType::from_c_int(SPTYPE_SKIP), PatternType::Skip);
        assert_eq!(PatternType::from_c_int(999), PatternType::Unknown(999));

        assert_eq!(PatternType::Match.to_c_int(), SPTYPE_MATCH);
        assert_eq!(PatternType::Start.to_c_int(), SPTYPE_START);
        assert_eq!(PatternType::End.to_c_int(), SPTYPE_END);
        assert_eq!(PatternType::Skip.to_c_int(), SPTYPE_SKIP);
        assert_eq!(PatternType::Unknown(999).to_c_int(), 999);
    }

    #[test]
    fn test_pattern_type_checks() {
        assert!(!PatternType::Match.is_region_type());
        assert!(PatternType::Start.is_region_type());
        assert!(PatternType::End.is_region_type());
        assert!(PatternType::Skip.is_region_type());

        assert!(PatternType::Start.is_start());
        assert!(!PatternType::End.is_start());

        assert!(PatternType::End.is_end());
        assert!(!PatternType::Start.is_end());

        assert!(PatternType::Skip.is_skip());
        assert!(!PatternType::Start.is_skip());
    }

    #[test]
    fn test_null_handle_checks() {
        let null_pat = SynPatHandle(std::ptr::null_mut());
        let null_item = StateItemHandle(std::ptr::null_mut());

        assert!(null_pat.is_null());
        assert!(null_item.is_null());

        let fake_ptr = std::ptr::dangling_mut::<std::ffi::c_void>();
        let non_null_pat = SynPatHandle(fake_ptr);
        assert!(!non_null_pat.is_null());
    }

    #[test]
    fn test_region_end_result_default() {
        let result = RegionEndResult::default();
        assert_eq!(result.m_endpos, Position::default());
        assert_eq!(result.hl_endpos, Position::default());
        assert_eq!(result.end_endpos, Position::default());
        assert_eq!(result.flags, 0);
        assert_eq!(result.end_idx, 0);
    }
}
