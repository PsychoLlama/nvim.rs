//! Command line history navigation
//!
//! This module provides history navigation functionality for command-line mode,
//! including Up/Down arrow navigation, prefix matching, and history search.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int};

use crate::keys::{history_uses_prefix, is_history_newer_key};

// =============================================================================
// History Type Constants
// =============================================================================

/// History type constants (matching nvim's HistoryType enum)
pub mod hist_type {
    use std::ffi::c_int;

    /// Default (use last) history
    pub const HIST_DEFAULT: c_int = -2;
    /// Invalid history type
    pub const HIST_INVALID: c_int = -1;
    /// Command (:) history
    pub const HIST_CMD: c_int = 0;
    /// Search (/ ?) history
    pub const HIST_SEARCH: c_int = 1;
    /// Expression (=) history
    pub const HIST_EXPR: c_int = 2;
    /// Input (@) history
    pub const HIST_INPUT: c_int = 3;
    /// Debug (>) history
    pub const HIST_DEBUG: c_int = 4;
}

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // History access
    fn nvim_get_hislen() -> c_int;
    fn get_hisidx(histype: c_int) -> *mut c_int;
    fn get_histentry(histype: c_int) -> *mut HistoryEntry;

    // Command line state
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_set_ccline_cmdpos(pos: c_int);
    fn nvim_set_ccline_cmdlen(len: c_int);
    fn nvim_set_ccline_cmdbuff_at(idx: c_int, val: c_char);

    // Buffer management
    fn alloc_cmdbuff(len: c_int);
    fn dealloc_cmdbuff();

    // Redraw
    fn redrawcmd();

    // Beep
    fn beep_flush();

    // String copy
    fn nvim_strcpy_cmdbuff(src: *const c_char);
}

/// C structure for history entry (must match histentry_T in cmdhist.h)
#[repr(C)]
pub struct HistoryEntry {
    /// Entry identifier number
    pub hisnum: c_int,
    // 4 bytes padding (alignment to 8-byte boundary for hisstr pointer)
    _pad: [u8; 4],
    /// Actual entry, separator char after the NUL
    pub hisstr: *mut c_char,
    /// Length of hisstr (excluding the NUL)
    pub hisstrlen: usize,
    /// Time when entry was added
    pub timestamp: u64,
    /// Additional entries from ShaDa file
    pub additional_data: *mut std::ffi::c_void,
}

// =============================================================================
// History Navigation Direction
// =============================================================================

/// Direction for history navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryDirection {
    /// Navigate to older history entries
    Older,
    /// Navigate to newer history entries
    Newer,
}

// =============================================================================
// History Navigator
// =============================================================================

/// State for navigating command history.
///
/// This tracks the current position in the history list and supports
/// prefix matching when using Up/Down arrows.
#[derive(Debug, Clone)]
pub struct HistoryNavigator {
    /// Current history index
    pub hiscnt: i32,
    /// Saved history index (before navigation started)
    pub save_hiscnt: i32,
    /// History type being navigated
    pub histype: i32,
    /// Prefix to match when navigating
    pub lookfor: Option<Vec<u8>>,
    /// Length of prefix being matched
    pub lookforlen: usize,
}

impl Default for HistoryNavigator {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryNavigator {
    /// Create a new history navigator with uninitialized state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hiscnt: 0,
            save_hiscnt: 0,
            histype: hist_type::HIST_INVALID,
            lookfor: None,
            lookforlen: 0,
        }
    }

    /// Initialize the navigator for a specific history type.
    ///
    /// Sets hiscnt to an "impossible" value (hislen) indicating we're not
    /// currently on any history entry.
    ///
    /// # Safety
    ///
    /// Calls C function to get history length.
    pub unsafe fn init(&mut self, histype: i32) {
        self.histype = histype;
        self.hiscnt = nvim_get_hislen();
        self.save_hiscnt = self.hiscnt;
        self.lookfor = None;
        self.lookforlen = 0;
    }

    /// Clear the lookfor pattern.
    pub fn clear_lookfor(&mut self) {
        self.lookfor = None;
        self.lookforlen = 0;
    }

    /// Check if we're currently on a history entry.
    ///
    /// # Safety
    ///
    /// Calls C function to get history length.
    #[must_use]
    pub unsafe fn on_history_entry(&self) -> bool {
        self.hiscnt < nvim_get_hislen()
    }

    /// Check if history is valid for navigation.
    ///
    /// # Safety
    ///
    /// Calls C function to get history length.
    #[must_use]
    pub unsafe fn is_valid(&self) -> bool {
        self.histype != hist_type::HIST_INVALID && nvim_get_hislen() > 0
    }

    /// Get the history length.
    ///
    /// # Safety
    ///
    /// Calls C function.
    #[must_use]
    pub unsafe fn get_hislen() -> i32 {
        nvim_get_hislen()
    }

    /// Get the current history index for a history type.
    ///
    /// # Safety
    ///
    /// Calls C function and dereferences raw pointer.
    #[must_use]
    pub unsafe fn get_current_hisidx(histype: i32) -> i32 {
        let ptr = get_hisidx(histype);
        if ptr.is_null() {
            -1
        } else {
            *ptr
        }
    }

    /// Get a history entry.
    ///
    /// # Safety
    ///
    /// Calls C function and dereferences raw pointer.
    #[must_use]
    pub unsafe fn get_entry(histype: i32, idx: i32) -> Option<&'static HistoryEntry> {
        if idx < 0 || idx >= nvim_get_hislen() {
            return None;
        }
        let entries = get_histentry(histype);
        if entries.is_null() {
            return None;
        }
        let entry = entries.add(idx as usize);
        if (*entry).hisstr.is_null() {
            return None;
        }
        Some(&*entry)
    }

    /// Get the string for a history entry.
    ///
    /// # Safety
    ///
    /// Dereferences raw C pointers.
    #[must_use]
    pub unsafe fn get_entry_str(entry: &HistoryEntry) -> Option<&'static [u8]> {
        if entry.hisstr.is_null() {
            return None;
        }
        Some(std::slice::from_raw_parts(
            entry.hisstr.cast::<u8>(),
            entry.hisstrlen,
        ))
    }

    /// Move to the next history index in the given direction.
    ///
    /// Handles wraparound in the circular history buffer.
    ///
    /// # Safety
    ///
    /// Calls C functions for history access.
    pub unsafe fn next_histidx(
        &mut self,
        direction: HistoryDirection,
        use_prefix_match: bool,
    ) -> bool {
        let hislen = nvim_get_hislen();
        let hisidx = Self::get_current_hisidx(self.histype);

        loop {
            match direction {
                HistoryDirection::Older => {
                    if self.hiscnt == hislen {
                        // First time - go to most recent entry
                        self.hiscnt = hisidx;
                    } else if self.hiscnt == 0 && hisidx != hislen - 1 {
                        // Wrap from 0 to end
                        self.hiscnt = hislen - 1;
                    } else if self.hiscnt != hisidx + 1 {
                        // Move to older entry
                        self.hiscnt -= 1;
                    } else {
                        // At oldest entry, restore and exit
                        self.hiscnt = self.save_hiscnt;
                        return false;
                    }
                }
                HistoryDirection::Newer => {
                    // On newest entry - clear to command line
                    if self.hiscnt == hisidx {
                        self.hiscnt = hislen;
                        return true;
                    }

                    // Not on any history entry
                    if self.hiscnt == hislen {
                        return false;
                    }

                    // Wrap from end to 0
                    if self.hiscnt == hislen - 1 {
                        self.hiscnt = 0;
                    } else {
                        // Move to newer entry
                        self.hiscnt += 1;
                    }
                }
            }

            // Check if entry is valid
            if self.hiscnt < 0 {
                self.hiscnt = self.save_hiscnt;
                return false;
            }

            let Some(entry) = Self::get_entry(self.histype, self.hiscnt) else {
                self.hiscnt = self.save_hiscnt;
                return false;
            };

            // Check prefix match if required
            if !use_prefix_match || self.hiscnt == self.save_hiscnt {
                return true;
            }

            // Check if entry matches our prefix
            if let Some(ref lookfor) = self.lookfor {
                if lookfor.len() <= entry.hisstrlen {
                    let entry_bytes =
                        std::slice::from_raw_parts(entry.hisstr.cast::<u8>(), lookfor.len());
                    if entry_bytes == lookfor.as_slice() {
                        return true;
                    }
                }
            } else {
                // No prefix - accept any entry
                return true;
            }

            // Entry didn't match - continue searching
        }
    }
}

// =============================================================================
// History Navigation Result
// =============================================================================

/// Result of a history navigation operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryResult {
    /// Command line was changed
    Changed,
    /// Command line was not changed
    NotChanged,
    /// Navigation failed
    Failed,
}

// =============================================================================
// Pure Rust History Functions
// =============================================================================

/// Check if a key should trigger history navigation.
///
/// Returns the navigation direction if the key navigates history.
#[must_use]
pub const fn key_to_history_direction(key: c_int) -> Option<HistoryDirection> {
    // Key codes from keycodes.h
    const K_UP: c_int = 0x100 + 0x48;
    const K_DOWN: c_int = 0x100 + 0x49;
    const K_S_UP: c_int = 0x100 + 0x5E + 11; // KS_EXTRA + KE_S_UP
    const K_S_DOWN: c_int = 0x100 + 0x5E + 12;
    const K_PAGEUP: c_int = 0x100 + 0x5E + 7;
    const K_PAGEDOWN: c_int = 0x100 + 0x5E + 8;
    const K_KPAGEUP: c_int = 0x100 + 0x5E + 55;
    const K_KPAGEDOWN: c_int = 0x100 + 0x5E + 56;
    const CTRL_P: c_int = 16;
    const CTRL_N: c_int = 14;

    match key {
        K_UP | K_S_UP | K_PAGEUP | K_KPAGEUP | CTRL_P => Some(HistoryDirection::Older),
        K_DOWN | K_S_DOWN | K_PAGEDOWN | K_KPAGEDOWN | CTRL_N => Some(HistoryDirection::Newer),
        _ => None,
    }
}

/// Check if a key uses prefix matching (Up/Down) vs full history (Page Up/Down).
#[must_use]
pub const fn key_uses_prefix_match(key: c_int) -> bool {
    const K_UP: c_int = 0x100 + 0x48;
    const K_DOWN: c_int = 0x100 + 0x49;

    key == K_UP || key == K_DOWN
}

/// Translate a history character to the associated type number.
#[must_use]
pub const fn hist_char2type(c: u8) -> i32 {
    match c {
        b':' => hist_type::HIST_CMD,
        b'=' => hist_type::HIST_EXPR,
        b'@' => hist_type::HIST_INPUT,
        b'>' => hist_type::HIST_DEBUG,
        0 | b'/' | b'?' => hist_type::HIST_SEARCH,
        _ => hist_type::HIST_INVALID,
    }
}

/// Translate a history type number to the associated character.
#[must_use]
pub const fn hist_type2char(histype: i32) -> u8 {
    match histype {
        hist_type::HIST_CMD => b':',
        hist_type::HIST_SEARCH => b'/',
        hist_type::HIST_EXPR => b'=',
        hist_type::HIST_INPUT => b'@',
        hist_type::HIST_DEBUG => b'>',
        _ => 0,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if a key triggers history navigation.
///
/// Returns 1 for older (backwards), -1 for newer (forwards), 0 for not history.
#[unsafe(no_mangle)]
pub extern "C" fn rs_key_to_history_direction(key: c_int) -> c_int {
    match key_to_history_direction(key) {
        Some(HistoryDirection::Older) => 1,
        Some(HistoryDirection::Newer) => -1,
        None => 0,
    }
}

/// Check if a key uses prefix matching for history.
#[unsafe(no_mangle)]
pub extern "C" fn rs_key_uses_prefix_match(key: c_int) -> c_int {
    c_int::from(key_uses_prefix_match(key))
}

// Note: rs_hist_char2type and rs_hist_type2char are defined in the cmdhist crate

// =============================================================================
// History Browsing
// =============================================================================

/// Result codes for command line operations (matching C enum)
mod cmdline_result {
    use std::ffi::c_int;
    /// Command line not changed
    pub const NOT_CHANGED: c_int = 1;
    /// Command line changed
    pub const CHANGED: c_int = 2;
}

/// State needed for history browsing passed from C.
///
/// This mirrors the relevant fields from CommandLineState in C.
#[repr(C)]
pub struct HistoryBrowseState {
    /// Current key code
    pub c: c_int,
    /// First character of prompt (: / ? = @ >)
    pub firstc: c_int,
    /// Current history index
    pub hiscnt: c_int,
    /// Saved history index
    pub save_hiscnt: c_int,
    /// History type
    pub histype: c_int,
    /// Prefix to match (owned by C)
    pub lookfor: *mut c_char,
    /// Length of lookfor prefix
    pub lookforlen: c_int,
}

/// Copy search history entry with separator replacement.
///
/// When recalling a search history entry, we may need to replace the
/// separator character if it differs from the current search separator.
///
/// # Safety
///
/// Caller must ensure pointers are valid and buffer is allocated.
unsafe fn copy_search_history_with_sep_replace(
    src: *const c_char,
    src_len: c_int,
    old_sep: u8,
    new_sep: u8,
) {
    // First pass: count the length needed
    let mut len = 0i32;
    for j in 0..src_len {
        let ch = *src.add(j as usize) as u8;
        let prev = if j > 0 {
            *src.add((j - 1) as usize) as u8
        } else {
            0
        };

        if ch == old_sep && prev != b'\\' {
            // Replace old sep with new sep
            len += 1;
        } else if ch == new_sep && prev != b'\\' {
            // Escape new sep
            len += 2;
        } else {
            len += 1;
        }
    }

    // Allocate buffer
    alloc_cmdbuff(len);

    // Second pass: copy with replacements
    let mut pos = 0i32;
    for j in 0..src_len {
        let ch = *src.add(j as usize) as u8;
        let prev = if j > 0 {
            *src.add((j - 1) as usize) as u8
        } else {
            0
        };

        if ch == old_sep && prev != b'\\' {
            // Replace old sep with new sep
            nvim_set_ccline_cmdbuff_at(pos, new_sep as c_char);
        } else if ch == new_sep && prev != b'\\' {
            // Escape new sep
            nvim_set_ccline_cmdbuff_at(pos, b'\\' as c_char);
            pos += 1;
            nvim_set_ccline_cmdbuff_at(pos, ch as c_char);
        } else {
            nvim_set_ccline_cmdbuff_at(pos, ch as c_char);
        }
        pos += 1;
    }

    // Null terminate
    nvim_set_ccline_cmdbuff_at(pos, 0);
    nvim_set_ccline_cmdpos(len);
    nvim_set_ccline_cmdlen(len);
}

/// Browse command history.
///
/// Handles Up, Down, Page Up, Page Down, Ctrl-N, Ctrl-P keys.
///
/// # Safety
///
/// Caller must ensure state pointer is valid and properly initialized.
/// The lookfor field may be modified by this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_command_line_browse_history(state: *mut HistoryBrowseState) -> c_int {
    let state = &mut *state;

    // Check if history is available
    let hislen = nvim_get_hislen();
    if state.histype == hist_type::HIST_INVALID || hislen == 0 || state.firstc == 0 {
        return cmdline_result::NOT_CHANGED;
    }

    state.save_hiscnt = state.hiscnt;

    // Save current command for prefix matching if not already saved
    // Note: lookfor allocation/deallocation is handled by C caller
    let _cmdpos = nvim_get_ccline_cmdpos();

    // Determine direction
    let next_match = is_history_newer_key(state.c);
    let use_prefix = history_uses_prefix(state.c);

    // Navigate history
    let mut nav = HistoryNavigator {
        hiscnt: state.hiscnt,
        save_hiscnt: state.save_hiscnt,
        histype: state.histype,
        lookfor: if state.lookfor.is_null() {
            None
        } else {
            Some(
                std::slice::from_raw_parts(state.lookfor.cast::<u8>(), state.lookforlen as usize)
                    .to_vec(),
            )
        },
        lookforlen: state.lookforlen as usize,
    };

    let direction = if next_match {
        HistoryDirection::Newer
    } else {
        HistoryDirection::Older
    };

    // Navigate to next history entry
    nav.next_histidx(direction, use_prefix);

    // Update state from navigator
    state.hiscnt = nav.hiscnt;

    if state.hiscnt != state.save_hiscnt {
        // Jumped to a different entry
        dealloc_cmdbuff();

        let (p, plen, entry_firstc): (*const c_char, c_int, Option<u8>) = if state.hiscnt == hislen
        {
            // Back to the saved command line
            (state.lookfor.cast_const(), state.lookforlen, None)
        } else {
            let entries = get_histentry(state.histype);
            let entry = &*entries.add(state.hiscnt as usize);

            // For search history, check if we need to replace separators
            let old_firstc = if state.histype == hist_type::HIST_SEARCH {
                // The separator is stored after the string + NUL
                let sep_byte = *entry.hisstr.add(entry.hisstrlen + 1) as u8;
                if sep_byte != 0 && sep_byte != state.firstc as u8 {
                    Some(sep_byte)
                } else {
                    None
                }
            } else {
                None
            };

            (
                entry.hisstr.cast_const(),
                entry.hisstrlen as c_int,
                old_firstc,
            )
        };

        // Handle search history separator replacement
        if state.histype == hist_type::HIST_SEARCH
            && !p.is_null()
            && !std::ptr::eq(p, state.lookfor)
        {
            if let Some(old_sep) = entry_firstc {
                copy_search_history_with_sep_replace(p, plen, old_sep, state.firstc as u8);
                redrawcmd();
                return cmdline_result::CHANGED;
            }
        }

        // Normal case: just copy the string
        alloc_cmdbuff(plen);
        if !p.is_null() {
            nvim_strcpy_cmdbuff(p);
        }
        nvim_set_ccline_cmdpos(plen);
        nvim_set_ccline_cmdlen(plen);

        redrawcmd();
        return cmdline_result::CHANGED;
    }

    beep_flush();
    cmdline_result::NOT_CHANGED
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hist_char2type() {
        assert_eq!(hist_char2type(b':'), hist_type::HIST_CMD);
        assert_eq!(hist_char2type(b'='), hist_type::HIST_EXPR);
        assert_eq!(hist_char2type(b'@'), hist_type::HIST_INPUT);
        assert_eq!(hist_char2type(b'>'), hist_type::HIST_DEBUG);
        assert_eq!(hist_char2type(0), hist_type::HIST_SEARCH);
        assert_eq!(hist_char2type(b'/'), hist_type::HIST_SEARCH);
        assert_eq!(hist_char2type(b'?'), hist_type::HIST_SEARCH);
        assert_eq!(hist_char2type(b'a'), hist_type::HIST_INVALID);
    }

    #[test]
    fn test_hist_type2char() {
        assert_eq!(hist_type2char(hist_type::HIST_CMD), b':');
        assert_eq!(hist_type2char(hist_type::HIST_SEARCH), b'/');
        assert_eq!(hist_type2char(hist_type::HIST_EXPR), b'=');
        assert_eq!(hist_type2char(hist_type::HIST_INPUT), b'@');
        assert_eq!(hist_type2char(hist_type::HIST_DEBUG), b'>');
        assert_eq!(hist_type2char(hist_type::HIST_INVALID), 0);
    }

    #[test]
    fn test_roundtrip() {
        for histype in [
            hist_type::HIST_CMD,
            hist_type::HIST_SEARCH,
            hist_type::HIST_EXPR,
            hist_type::HIST_INPUT,
            hist_type::HIST_DEBUG,
        ] {
            let c = hist_type2char(histype);
            assert_eq!(hist_char2type(c), histype);
        }
    }

    #[test]
    fn test_history_direction() {
        // Up key codes
        const K_UP: c_int = 0x100 + 0x48;
        const CTRL_P: c_int = 16;

        // Down key codes
        const K_DOWN: c_int = 0x100 + 0x49;
        const CTRL_N: c_int = 14;

        assert_eq!(
            key_to_history_direction(K_UP),
            Some(HistoryDirection::Older)
        );
        assert_eq!(
            key_to_history_direction(CTRL_P),
            Some(HistoryDirection::Older)
        );
        assert_eq!(
            key_to_history_direction(K_DOWN),
            Some(HistoryDirection::Newer)
        );
        assert_eq!(
            key_to_history_direction(CTRL_N),
            Some(HistoryDirection::Newer)
        );
        assert_eq!(key_to_history_direction(c_int::from(b'a')), None);
    }

    #[test]
    fn test_key_uses_prefix_match() {
        const K_UP: c_int = 0x100 + 0x48;
        const K_DOWN: c_int = 0x100 + 0x49;
        const CTRL_P: c_int = 16;

        assert!(key_uses_prefix_match(K_UP));
        assert!(key_uses_prefix_match(K_DOWN));
        assert!(!key_uses_prefix_match(CTRL_P));
    }

    #[test]
    fn test_history_navigator_new() {
        let nav = HistoryNavigator::new();
        assert_eq!(nav.histype, hist_type::HIST_INVALID);
        assert!(nav.lookfor.is_none());
        assert_eq!(nav.lookforlen, 0);
    }
}
