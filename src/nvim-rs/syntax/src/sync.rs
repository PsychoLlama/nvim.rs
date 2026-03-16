//! Synchronization for syntax highlighting.
//!
//! This module handles:
//! - syn_sync() for scroll/jump synchronization
//! - Sync methods (lines, ccomment, minlines, etc.)
//! - Recovery point detection

use std::ffi::c_int;

use crate::check_ends::{check_keepend, check_state_ends, update_si_attr};

use crate::current_attr::syn_finish_line;
use crate::region::update_si_end;
use crate::types::{
    SynBlockHandle, SynPatHandle, SynStateHandle, WinHandle, KEYWORD_IDX, SPTYPE_START,
};

// =============================================================================
// Sync flag constants
// =============================================================================

/// Sync flag: use C-style comments for sync.
pub const SF_CCOMMENT: i32 = 0x01;

/// Sync flag: use match patterns for sync.
pub const SF_MATCH: i32 = 0x02;

/// HL_SYNC_HERE flag value.
const HL_SYNC_HERE: i32 = 0x10;

// =============================================================================
// FFI declarations for sync operations
// =============================================================================

extern "C" {
    // Synblock sync accessors
    fn nvim_synblock_get_sync_flags(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sync_id(block: SynBlockHandle) -> i16;
    fn nvim_synblock_get_sync_minlines(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sync_maxlines(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sync_linebreaks(block: SynBlockHandle) -> c_int;

    // Pattern sync accessors
    fn nvim_synblock_get_pattern(block: SynBlockHandle, idx: c_int) -> SynPatHandle;

    // Current synblock sync accessors
    fn nvim_syn_get_sync_minlines() -> c_int;
    fn nvim_syn_get_sync_maxlines() -> c_int;
    fn nvim_syn_get_sync_flags() -> c_int;
    fn nvim_syn_get_sync_id() -> c_int;

    // Update ends with syncing parameter
    #[link_name = "rs_syn_update_ends"]
    fn nvim_syn_call_syn_update_ends(syncing: c_int);

    // State management
    #[link_name = "rs_invalidate_current_state"]
    fn nvim_syn_invalidate_current_state();
    #[link_name = "rs_validate_current_state"]
    fn nvim_syn_validate_current_state();
    #[link_name = "rs_syn_start_line"]
    fn nvim_syn_start_line();
    fn nvim_syn_get_got_int() -> c_int;
    fn line_breakcheck();

    // Phase 3 accessors
    #[link_name = "rs_load_current_state"]
    fn nvim_syn_load_current_state(from: SynStateHandle);
    // (nvim_syn_match_linecont deleted: call Rust directly)
    fn nvim_synstate_get_lnum(state: SynStateHandle) -> c_int;

    // C-comment sync: thin helper that saves/restores curwin/curbuf/cursor,
    // calls find_start_comment, and returns the adjusted start_lnum via out-param.
    // Returns 1 if find_start_comment found a comment, 0 otherwise.
    fn nvim_syn_ccomment_find(
        wp: WinHandle,
        start_lnum: c_int,
        out_start_lnum: *mut c_int,
    ) -> c_int;

    // Synblock pattern count
    fn nvim_synblock_get_pattern_count(block: SynBlockHandle) -> c_int;
    fn nvim_syn_get_syn_block() -> SynBlockHandle;

    // Line content access
    #[link_name = "rs_syn_getcurline"]
    fn nvim_syn_getcurline() -> *mut i8;
}

// =============================================================================
// Synblock sync accessors
// =============================================================================

/// Get the sync flags for a synblock.
#[must_use]
pub fn synblock_sync_flags(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sync_flags(block) }
}

/// Get the sync ID for a synblock.
#[must_use]
pub fn synblock_sync_id(block: SynBlockHandle) -> i16 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sync_id(block) }
}

/// Get the sync minlines for a synblock.
#[must_use]
pub fn synblock_sync_minlines(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sync_minlines(block) }
}

/// Get the sync maxlines for a synblock.
#[must_use]
pub fn synblock_sync_maxlines(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sync_maxlines(block) }
}

/// Get the sync linebreaks for a synblock.
#[must_use]
pub fn synblock_sync_linebreaks(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sync_linebreaks(block) }
}

// =============================================================================
// Pattern sync accessors
// =============================================================================

/// Check if a pattern is a syncing pattern.
#[must_use]
pub fn synpat_is_syncing(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { (*pat.as_ptr()).sp_syncing }
}

/// Get the sync index for a pattern.
#[must_use]
pub fn synpat_sync_idx(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { (*pat.as_ptr()).sp_sync_idx }
}

/// Check if a pattern at a given index is a syncing pattern.
/// Implements nvim_synblock_pattern_is_syncing in Rust using get_pattern + get_syncing.
#[must_use]
pub fn synblock_pattern_is_syncing(block: SynBlockHandle, idx: i32) -> bool {
    if block.is_null() || idx < 0 {
        return false;
    }
    let pat = unsafe { nvim_synblock_get_pattern(block, idx) };
    if pat.is_null() {
        return false;
    }
    unsafe { (*pat.as_ptr()).sp_syncing }
}

// =============================================================================
// Current synblock sync accessors
// =============================================================================

/// Get the sync minlines for the current synblock.
#[must_use]
pub fn sync_minlines() -> i32 {
    unsafe { nvim_syn_get_sync_minlines() }
}

/// Get the sync maxlines for the current synblock.
#[must_use]
pub fn sync_maxlines() -> i32 {
    unsafe { nvim_syn_get_sync_maxlines() }
}

/// Get the sync flags for the current synblock.
#[must_use]
pub fn sync_flags() -> i32 {
    unsafe { nvim_syn_get_sync_flags() }
}

/// Get the sync ID for the current synblock.
#[must_use]
pub fn sync_id() -> i32 {
    unsafe { nvim_syn_get_sync_id() }
}

// =============================================================================
// Sync highlight accessors
// =============================================================================

/// Get the HL_SYNC_HERE flag status.
#[must_use]
pub fn hl_sync_here() -> i32 {
    HL_SYNC_HERE
}

/// Get the HL_SYNC_THERE flag status.
#[must_use]
pub fn hl_sync_there() -> i32 {
    crate::types::HL_SYNC_THERE
}

// =============================================================================
// Main sync function - Rust implementation
// =============================================================================

/// Synchronize syntax state for a window at a given line.
///
/// This is the main sync function that determines the syntax state at a given
/// line by searching backwards for sync points, C-style comments, or using
/// minlines.
///
/// # Safety
/// This modifies global state and must be called from the main thread.
pub unsafe fn syn_sync_impl(wp: WinHandle, mut start_lnum: i32, last_valid: SynStateHandle) {
    // Clear any current state that might be hanging around.
    nvim_syn_invalidate_current_state();

    // Calculate start_lnum based on minlines/maxlines.
    let sync_minlines = nvim_syn_get_sync_minlines();
    let sync_maxlines = nvim_syn_get_sync_maxlines();
    let sync_fl = nvim_syn_get_sync_flags();

    if sync_minlines > start_lnum {
        start_lnum = 1;
    } else {
        let mut lnum = if sync_minlines == 1 {
            1
        } else if sync_minlines < 10 {
            sync_minlines * 2
        } else {
            sync_minlines * 3 / 2
        };
        if sync_maxlines != 0 && lnum > sync_maxlines {
            lnum = sync_maxlines;
        }
        if lnum >= start_lnum {
            start_lnum = 1;
        } else {
            start_lnum -= lnum;
        }
    }
    crate::statics::CURRENT_LNUM = start_lnum;

    // 1. Search backwards for the end of a C-style comment.
    if sync_fl & SF_CCOMMENT != 0 {
        // The C helper saves/restores curwin/curbuf/cursor, skips backslash
        // continuations, sets current_lnum, and calls find_start_comment.
        // It returns 1 if we are inside a comment.
        let mut adjusted_lnum = start_lnum;
        let found = nvim_syn_ccomment_find(wp, start_lnum, &mut adjusted_lnum);
        crate::statics::CURRENT_LNUM = adjusted_lnum;

        if found != 0 {
            // Inside a comment: find the syntax item that defines the comment.
            let sync_id = nvim_syn_get_sync_id();
            let blk = nvim_syn_get_syn_block();
            let ga_len = nvim_synblock_get_pattern_count(blk);
            let mut idx = ga_len - 1;
            while idx >= 0 {
                let pp = crate::statics::syn_item_at(blk, idx);
                if !pp.is_null()
                    && (*pp).sp_syn.id as c_int == sync_id
                    && (*pp).sp_type as c_int == SPTYPE_START
                {
                    nvim_syn_validate_current_state();
                    crate::state_ops::rs_syn_push_current_state(idx);
                    update_si_attr(crate::statics::CURRENT_STATE.ga_len - 1);
                    break;
                }
                idx -= 1;
            }
        }
    } else if sync_fl & SF_MATCH != 0 {
        // 2. Search backwards for given sync patterns.
        let break_lnum = if sync_maxlines != 0 && start_lnum > sync_maxlines {
            start_lnum - sync_maxlines
        } else {
            0
        };

        let mut found_flags: i32 = 0;
        let mut found_match_idx: i32 = 0;
        let mut found_current_lnum: i32 = 0;
        let mut found_current_col: i32 = 0;
        let mut found_m_endpos_lnum: i32 = 0;
        let mut found_m_endpos_col: i32 = 0;

        let mut end_lnum = start_lnum;
        let mut lnum = start_lnum;
        lnum -= 1; // --lnum before first check
        while lnum > break_lnum {
            // This can take a long time: break when CTRL-C pressed.
            line_breakcheck();
            if nvim_syn_get_got_int() != 0 {
                nvim_syn_invalidate_current_state();
                crate::statics::CURRENT_LNUM = start_lnum;
                break;
            }

            // Check if we have run into a valid saved state stack now.
            if !last_valid.is_null() && lnum == nvim_synstate_get_lnum(last_valid) {
                nvim_syn_load_current_state(last_valid);
                break;
            }

            // Check if the previous line has the line-continuation pattern.
            if lnum > 1 && crate::line_init::rs_syn_match_linecont(lnum - 1) != 0 {
                lnum -= 1;
                continue;
            }

            // Start with nothing on the state stack
            nvim_syn_validate_current_state();

            let mut current_lnum = lnum;
            while current_lnum < end_lnum {
                crate::statics::CURRENT_LNUM = current_lnum;
                nvim_syn_start_line();
                loop {
                    let had_sync_point = syn_finish_line(true);
                    // When a sync point has been found, remember where, and
                    // continue to look for another one, further on in the line.
                    if had_sync_point && crate::statics::CURRENT_STATE.ga_len > 0 {
                        let cur_si = crate::statics::current_state_top();
                        let (si_m_endpos_lnum, si_m_endpos_col) = {
                            let p = cur_si.as_ptr();
                            ((*p).si_m_endpos.lnum, (*p).si_m_endpos.col)
                        };

                        if si_m_endpos_lnum > start_lnum {
                            // ignore match that goes to after where started
                            current_lnum = end_lnum;
                            break;
                        }

                        let si_idx = (*cur_si.as_ptr()).si_idx;
                        if si_idx < 0 {
                            // Cannot happen?
                            found_flags = 0;
                            found_match_idx = KEYWORD_IDX;
                        } else {
                            let blk2 = nvim_syn_get_syn_block();
                            let si_pp = crate::statics::syn_item_at(blk2, si_idx);
                            if si_pp.is_null() {
                                found_flags = 0;
                                found_match_idx = -1;
                            } else {
                                found_flags = (*si_pp).sp_flags;
                                found_match_idx = (*si_pp).sp_sync_idx;
                            }
                        }
                        found_current_lnum = crate::statics::CURRENT_LNUM;
                        found_current_col = crate::statics::CURRENT_COL;
                        found_m_endpos_lnum = si_m_endpos_lnum;
                        found_m_endpos_col = si_m_endpos_col;

                        // Continue after the match (be aware of a zero-length match).
                        if found_m_endpos_lnum > current_lnum {
                            current_lnum = found_m_endpos_lnum;
                            crate::statics::CURRENT_LNUM = current_lnum;
                            crate::statics::CURRENT_COL = found_m_endpos_col;
                            if current_lnum >= end_lnum {
                                break;
                            }
                        } else if found_m_endpos_col > crate::statics::CURRENT_COL {
                            crate::statics::CURRENT_COL = found_m_endpos_col;
                        } else {
                            crate::statics::CURRENT_COL += 1;
                        }

                        // syn_current_attr() will have skipped the check for
                        // an item that ends here, need to do that now. Be
                        // careful not to go past the NUL.
                        let prev_current_col = crate::statics::CURRENT_COL;
                        let curline = nvim_syn_getcurline();
                        if !curline.is_null() && *curline.offset(prev_current_col as isize) != 0 {
                            crate::statics::CURRENT_COL = prev_current_col + 1;
                        }
                        check_state_ends();
                        crate::statics::CURRENT_COL = prev_current_col;
                    } else {
                        break;
                    }
                }
                current_lnum += 1;
            }

            // If a sync point was encountered, break here.
            if found_flags != 0 {
                // Put the item that was specified by the sync point on the
                // state stack. If there was no item specified, make the
                // state stack empty.
                crate::state_ops::rs_syn_clear_current_state();
                if found_match_idx >= 0 {
                    crate::state_ops::rs_syn_push_current_state(found_match_idx);
                    update_si_attr(crate::statics::CURRENT_STATE.ga_len - 1);
                }

                // When using "grouphere", continue from the sync point
                // match, until the end of the line. Parsing starts at
                // the next line.
                // For "groupthere" the parsing starts at start_lnum.
                if found_flags & HL_SYNC_HERE != 0 {
                    if crate::statics::CURRENT_STATE.ga_len > 0 {
                        let cur_si = crate::statics::current_state_top();
                        {
                            let p = cur_si.as_ptr();
                            (*p).si_h_startpos.lnum = found_current_lnum;
                            (*p).si_h_startpos.col = found_current_col;
                        }
                        update_si_end(cur_si, crate::statics::CURRENT_COL, true);
                        check_keepend();
                    }
                    crate::statics::CURRENT_COL = found_m_endpos_col;
                    crate::statics::CURRENT_LNUM = found_m_endpos_lnum;
                    syn_finish_line(false);
                    crate::statics::CURRENT_LNUM += 1;
                } else {
                    crate::statics::CURRENT_LNUM = start_lnum;
                }

                break;
            }

            end_lnum = lnum;
            nvim_syn_invalidate_current_state();
            lnum -= 1;
        }

        // Ran into start of the file or exceeded maximum number of lines
        if lnum <= break_lnum {
            nvim_syn_invalidate_current_state();
            crate::statics::CURRENT_LNUM = break_lnum + 1;
        }
    }

    nvim_syn_validate_current_state();
}

/// Call syn_update_ends with syncing parameter.
///
/// # Safety
/// This modifies global state.
pub unsafe fn call_syn_update_ends(syncing: bool) {
    nvim_syn_call_syn_update_ends(if syncing { 1 } else { 0 });
}

// =============================================================================
// Exported FFI functions
// =============================================================================

/// Rust implementation of syn_sync.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_sync(wp: WinHandle, start_lnum: c_int, last_valid: SynStateHandle) {
    syn_sync_impl(wp, start_lnum, last_valid);
}

// =============================================================================
// Sync settings struct
// =============================================================================

/// Sync settings for a synblock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SyncSettings {
    /// Sync flags (SF_CCOMMENT, SF_MATCH).
    pub flags: i32,
    /// Sync ID.
    pub id: i16,
    /// Minimum lines to search back.
    pub minlines: i32,
    /// Maximum lines to search back.
    pub maxlines: i32,
    /// Line breaks setting.
    pub linebreaks: i32,
}

impl SyncSettings {
    /// Create sync settings from a synblock.
    #[must_use]
    pub fn from_synblock(block: SynBlockHandle) -> Self {
        if block.is_null() {
            return Self::default();
        }
        Self {
            flags: synblock_sync_flags(block),
            id: synblock_sync_id(block),
            minlines: synblock_sync_minlines(block),
            maxlines: synblock_sync_maxlines(block),
            linebreaks: synblock_sync_linebreaks(block),
        }
    }

    /// Create sync settings from the current synblock.
    #[must_use]
    pub fn current() -> Self {
        Self {
            flags: sync_flags(),
            id: sync_id() as i16,
            minlines: sync_minlines(),
            maxlines: sync_maxlines(),
            linebreaks: 0, // Not directly accessible for current
        }
    }

    /// Check if C-style comment sync is enabled.
    #[must_use]
    pub const fn uses_ccomment(&self) -> bool {
        (self.flags & SF_CCOMMENT) != 0
    }

    /// Check if match-based sync is enabled.
    #[must_use]
    pub const fn uses_match(&self) -> bool {
        (self.flags & SF_MATCH) != 0
    }

    /// Check if any sync method is configured.
    #[must_use]
    pub fn has_sync_method(&self) -> bool {
        self.uses_ccomment() || self.uses_match() || self.minlines > 0
    }
}

// =============================================================================
// Sync method enum
// =============================================================================

/// The synchronization method to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMethod {
    /// No synchronization.
    None,
    /// Synchronize by counting minlines back.
    Minlines,
    /// Synchronize using C-style comments.
    CComment,
    /// Synchronize using match patterns.
    Match,
}

impl SyncMethod {
    /// Determine the sync method from flags.
    #[must_use]
    pub const fn from_flags(flags: i32) -> Self {
        if (flags & SF_CCOMMENT) != 0 {
            Self::CComment
        } else if (flags & SF_MATCH) != 0 {
            Self::Match
        } else {
            Self::None
        }
    }

    /// Determine the sync method from sync settings.
    #[must_use]
    pub fn from_settings(settings: &SyncSettings) -> Self {
        if settings.uses_ccomment() {
            Self::CComment
        } else if settings.uses_match() {
            Self::Match
        } else if settings.minlines > 0 {
            Self::Minlines
        } else {
            Self::None
        }
    }
}

// =============================================================================
// Recovery point helper
// =============================================================================

/// Calculate the line to start searching from for synchronization.
///
/// # Arguments
/// * `start_lnum` - Current line number
/// * `settings` - Sync settings
///
/// # Returns
/// The line number to start searching from.
#[must_use]
pub fn calculate_sync_start(start_lnum: i32, settings: &SyncSettings) -> i32 {
    if settings.minlines >= start_lnum {
        // Start from the beginning
        1
    } else {
        // Calculate based on minlines
        let mut lnum = if settings.minlines == 1 {
            settings.minlines
        } else if settings.minlines < 10 {
            settings.minlines * 2
        } else {
            settings.minlines * 3 / 2
        };

        // Apply maxlines limit
        if settings.maxlines > 0 && lnum > settings.maxlines {
            lnum = settings.maxlines;
        }

        start_lnum.saturating_sub(lnum).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_flags() {
        assert_eq!(SF_CCOMMENT, 0x01);
        assert_eq!(SF_MATCH, 0x02);
    }

    #[test]
    fn test_hl_sync_here() {
        assert_eq!(HL_SYNC_HERE, 0x10);
    }

    #[test]
    fn test_sync_settings_default() {
        let settings = SyncSettings::default();
        assert_eq!(settings.flags, 0);
        assert_eq!(settings.id, 0);
        assert_eq!(settings.minlines, 0);
        assert_eq!(settings.maxlines, 0);
        assert_eq!(settings.linebreaks, 0);
        assert!(!settings.uses_ccomment());
        assert!(!settings.uses_match());
        assert!(!settings.has_sync_method());
    }

    #[test]
    fn test_sync_settings_methods() {
        let settings = SyncSettings {
            flags: SF_CCOMMENT,
            id: 0,
            minlines: 0,
            maxlines: 0,
            linebreaks: 0,
        };
        assert!(settings.uses_ccomment());
        assert!(!settings.uses_match());
        assert!(settings.has_sync_method());

        let settings = SyncSettings {
            flags: SF_MATCH,
            id: 0,
            minlines: 0,
            maxlines: 0,
            linebreaks: 0,
        };
        assert!(!settings.uses_ccomment());
        assert!(settings.uses_match());
        assert!(settings.has_sync_method());

        let settings = SyncSettings {
            flags: 0,
            id: 0,
            minlines: 10,
            maxlines: 0,
            linebreaks: 0,
        };
        assert!(!settings.uses_ccomment());
        assert!(!settings.uses_match());
        assert!(settings.has_sync_method());
    }

    #[test]
    fn test_sync_method() {
        assert_eq!(SyncMethod::from_flags(0), SyncMethod::None);
        assert_eq!(SyncMethod::from_flags(SF_CCOMMENT), SyncMethod::CComment);
        assert_eq!(SyncMethod::from_flags(SF_MATCH), SyncMethod::Match);
        // CComment takes precedence
        assert_eq!(
            SyncMethod::from_flags(SF_CCOMMENT | SF_MATCH),
            SyncMethod::CComment
        );
    }

    #[test]
    fn test_sync_method_from_settings() {
        let settings = SyncSettings::default();
        assert_eq!(SyncMethod::from_settings(&settings), SyncMethod::None);

        let settings = SyncSettings {
            flags: SF_CCOMMENT,
            ..Default::default()
        };
        assert_eq!(SyncMethod::from_settings(&settings), SyncMethod::CComment);

        let settings = SyncSettings {
            flags: SF_MATCH,
            ..Default::default()
        };
        assert_eq!(SyncMethod::from_settings(&settings), SyncMethod::Match);

        let settings = SyncSettings {
            minlines: 10,
            ..Default::default()
        };
        assert_eq!(SyncMethod::from_settings(&settings), SyncMethod::Minlines);
    }

    #[test]
    fn test_calculate_sync_start() {
        // Minlines >= start_lnum
        let settings = SyncSettings {
            minlines: 100,
            maxlines: 0,
            ..Default::default()
        };
        assert_eq!(calculate_sync_start(50, &settings), 1);

        // minlines == 1
        let settings = SyncSettings {
            minlines: 1,
            maxlines: 0,
            ..Default::default()
        };
        assert_eq!(calculate_sync_start(100, &settings), 99);

        // minlines < 10
        let settings = SyncSettings {
            minlines: 5,
            maxlines: 0,
            ..Default::default()
        };
        assert_eq!(calculate_sync_start(100, &settings), 90); // 100 - 10

        // minlines >= 10
        let settings = SyncSettings {
            minlines: 20,
            maxlines: 0,
            ..Default::default()
        };
        assert_eq!(calculate_sync_start(100, &settings), 70); // 100 - 30

        // maxlines limit
        let settings = SyncSettings {
            minlines: 20,
            maxlines: 10,
            ..Default::default()
        };
        assert_eq!(calculate_sync_start(100, &settings), 90); // 100 - 10 (limited)
    }

    #[test]
    fn test_null_handles() {
        let null_block = SynBlockHandle(std::ptr::null_mut());
        let null_pat = SynPatHandle(std::ptr::null_mut());
        let null_state = SynStateHandle(std::ptr::null_mut());
        let null_win = WinHandle(std::ptr::null_mut());

        assert!(null_block.is_null());
        assert!(null_pat.is_null());
        assert!(null_state.is_null());
        assert!(null_win.is_null());
    }
}
