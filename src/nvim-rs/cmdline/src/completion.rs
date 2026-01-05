//! Core expansion logic for command-line completion
//!
//! This module provides the Rust implementation of the core expansion
//! engine from cmdexpand.c, including match navigation, suffix checking,
//! and file expansion flag calculation.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

use crate::expand::wild_mode::{WILD_NEXT, WILD_PAGEDOWN, WILD_PAGEUP, WILD_PREV, WILD_PUM_WANT};
use crate::expand::{ew_flags, ExpandContext};

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Popup menu height
    fn pum_get_height() -> c_int;

    // Access to pum_want global structure
    fn nvim_get_pum_want_active() -> c_int;
    fn nvim_get_pum_want_item() -> c_int;

    // Access to xp fields for navigation
    fn nvim_expand_get_numfiles(xp: *const ()) -> c_int;
    fn nvim_expand_get_selected(xp: *const ()) -> c_int;
    fn nvim_expand_get_orig_not_null(xp: *const ()) -> c_int;

    // Suffix matching
    fn match_suffix(fname: *const c_char) -> c_int;
}

// =============================================================================
// Match Navigation Logic
// =============================================================================

/// Calculate the next file index for match navigation.
///
/// This implements the core logic of `get_next_or_prev_match` for calculating
/// the new index based on the navigation mode (WILD_NEXT, WILD_PREV, etc.).
///
/// # Arguments
///
/// * `mode` - Navigation mode (WILD_NEXT, WILD_PREV, WILD_PAGEUP, WILD_PAGEDOWN, WILD_PUM_WANT)
/// * `current_selected` - Current selected index (-1 means original text)
/// * `num_files` - Total number of match files
/// * `has_orig` - Whether original text exists (xp->xp_orig != NULL)
/// * `pum_height` - Popup menu height (used for PAGEUP/PAGEDOWN)
/// * `pum_want_item` - Target item for WILD_PUM_WANT mode
///
/// # Returns
///
/// The new file index, or -1 to return to original text.
#[must_use]
pub const fn calculate_next_match_index(
    mode: c_int,
    current_selected: c_int,
    num_files: c_int,
    has_orig: bool,
    pum_height: c_int,
    pum_want_item: c_int,
) -> c_int {
    if num_files <= 0 {
        return -1;
    }

    let mut findex = current_selected;

    if mode == WILD_PREV {
        // Select the last entry if at original text
        if findex == -1 {
            findex = num_files;
        }
        // Otherwise select the previous entry
        findex -= 1;
    } else if mode == WILD_NEXT {
        // Select the next entry
        findex += 1;
    } else if mode == WILD_PAGEUP || mode == WILD_PAGEDOWN {
        // Get the effective height for paging
        let mut ht = pum_height;
        if ht > 3 {
            ht -= 2;
        }

        if mode == WILD_PAGEUP {
            if findex == 0 {
                // at the first entry, don't select any entries
                findex = -1;
            } else if findex < 0 {
                // no entry is selected. select the last entry
                findex = num_files - 1;
            } else {
                // go up by the pum height
                findex -= ht;
                if findex < 0 {
                    findex = 0;
                }
            }
        } else {
            // WILD_PAGEDOWN
            if findex == num_files - 1 {
                // at the last entry, don't select any entries
                findex = -1;
            } else if findex < 0 {
                // no entry is selected. select the first entry
                findex = 0;
            } else {
                // go down by the pum height
                findex += ht;
                if findex > num_files - 1 {
                    findex = num_files - 1;
                }
            }
        }
    } else if mode == WILD_PUM_WANT {
        findex = pum_want_item;
    }

    // Handle wrapping around
    if findex < 0 || findex >= num_files {
        // If original text exists, return to it when wrapping around
        if has_orig {
            findex = -1;
        } else {
            // Wrap around to opposite end
            findex = if findex < 0 { num_files - 1 } else { 0 };
        }
    }

    findex
}

/// FFI wrapper for match navigation calculation.
///
/// # Safety
///
/// `xp` must be a valid pointer to an `expand_T` structure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_calculate_next_match_index(mode: c_int, xp: *const ()) -> c_int {
    if xp.is_null() {
        return -1;
    }

    let num_files = nvim_expand_get_numfiles(xp);
    let current_selected = nvim_expand_get_selected(xp);
    let has_orig = nvim_expand_get_orig_not_null(xp) != 0;
    let pum_height = pum_get_height();
    let pum_want_item = if nvim_get_pum_want_active() != 0 {
        nvim_get_pum_want_item()
    } else {
        0
    };

    calculate_next_match_index(
        mode,
        current_selected,
        num_files,
        has_orig,
        pum_height,
        pum_want_item,
    )
}

// =============================================================================
// Suffix Matching
// =============================================================================

/// Count non-suffix matches in the first two file entries.
///
/// For file/directory expansion with multiple matches, check the first two
/// entries to determine if they have matching suffixes. This is used to decide
/// whether to show "too many matches" warning.
///
/// # Safety
///
/// `files` must be a valid pointer to an array of at least `min(num_files, 2)`
/// valid C string pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_count_non_suffix_matches(
    files: *const *const c_char,
    num_files: c_int,
) -> c_int {
    if files.is_null() || num_files <= 0 {
        return 1; // Default to 1 match
    }

    if num_files <= 1 {
        return num_files;
    }

    let mut non_suf_match = 0;

    // Check first two entries
    for i in 0..2 {
        let file = *files.add(i);
        if !file.is_null() && match_suffix(file) != 0 {
            non_suf_match += 1;
        }
    }

    non_suf_match
}

// =============================================================================
// File Context Expansion Flags
// =============================================================================

/// Calculate the expansion wildcard flags for file/directory contexts.
///
/// This translates the expand context type and options into the appropriate
/// EW_* flags used by `expand_wildcards()`.
///
/// # Arguments
///
/// * `context` - The expansion context type
/// * `base_flags` - Base flags from `map_wildopts_to_ewflags`
/// * `options` - WILD_* option flags
///
/// # Returns
///
/// The combined EW_* flags for the expansion.
#[must_use]
#[allow(clippy::wildcard_imports)]
pub const fn calculate_file_expand_flags(
    context: ExpandContext,
    base_flags: c_int,
    options: c_int,
) -> c_int {
    use crate::expand::wild_flags::WILD_ICASE;
    use ew_flags::*;

    let mut flags = base_flags;

    match context {
        ExpandContext::Files => {
            flags |= EW_FILE;
        }
        ExpandContext::FilesInPath => {
            flags |= EW_FILE | EW_PATH;
        }
        ExpandContext::DirsInCdpath => {
            // For cdpath, we want directories only, clear EW_FILE
            flags = (flags | EW_DIR | EW_CDPATH) & !EW_FILE;
        }
        ExpandContext::Directories => {
            // Directories only, clear EW_FILE
            flags = (flags | EW_DIR) & !EW_FILE;
        }
        _ => {}
    }

    // Add icase flag if requested
    if (options & WILD_ICASE) != 0 {
        flags |= EW_ICASE;
    }

    flags
}

/// FFI wrapper for file expand flags calculation.
#[unsafe(no_mangle)]
pub const extern "C" fn rs_calculate_file_expand_flags(
    context: c_int,
    base_flags: c_int,
    options: c_int,
) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return base_flags;
    };

    calculate_file_expand_flags(ctx, base_flags, options)
}

// =============================================================================
// Backslash Pattern Processing
// =============================================================================

/// Process backslashes in a file pattern for expansion.
///
/// This handles the various backslash escaping modes used for `:set path=`
/// and `:set tags=` style options.
///
/// # Arguments
///
/// * `pat` - The input pattern bytes
/// * `backslash_flags` - XP_BS_* flags indicating backslash handling mode
///
/// # Returns
///
/// A new Vec<u8> with processed backslash sequences, or the original if no
/// processing is needed.
#[must_use]
pub fn process_pattern_backslashes(pat: &[u8], backslash_flags: c_int) -> Vec<u8> {
    use crate::expand::{XP_BS_COMMA, XP_BS_NONE, XP_BS_ONE, XP_BS_THREE};

    if backslash_flags == XP_BS_NONE {
        return pat.to_vec();
    }

    let mut result = Vec::with_capacity(pat.len());
    let mut i = 0;

    while i < pat.len() {
        if pat[i] != b'\\' {
            result.push(pat[i]);
            i += 1;
            continue;
        }

        // Handle XP_BS_THREE: "\\\\ " -> " "
        if (backslash_flags & XP_BS_THREE) != 0
            && i + 3 < pat.len()
            && pat[i + 1] == b'\\'
            && pat[i + 2] == b'\\'
            && pat[i + 3] == b' '
        {
            result.push(b' ');
            i += 4;
            continue;
        }

        // Handle XP_BS_ONE: "\\ " -> " "
        if (backslash_flags & XP_BS_ONE) != 0 && i + 1 < pat.len() && pat[i + 1] == b' ' {
            result.push(b' ');
            i += 2;
            continue;
        }

        // Handle XP_BS_COMMA: "\\\\," -> ","
        // Note: BACKSLASH_IN_FILENAME case (Windows-only "\\,") is not handled here
        if (backslash_flags & XP_BS_COMMA) != 0
            && i + 2 < pat.len()
            && pat[i + 1] == b'\\'
            && pat[i + 2] == b','
        {
            result.push(b',');
            i += 3;
            continue;
        }

        // No special handling, keep the backslash
        result.push(pat[i]);
        i += 1;
    }

    result
}

/// FFI wrapper for backslash pattern processing.
///
/// # Safety
///
/// * `pat` must be a valid pointer to a NUL-terminated C string
/// * `out_buf` must be a valid pointer to a buffer of at least `out_buf_size` bytes
/// * The caller is responsible for ensuring the output buffer is large enough
///
/// # Returns
///
/// The number of bytes written to `out_buf` (not including NUL terminator),
/// or -1 on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_process_pattern_backslashes(
    pat: *const c_char,
    pat_len: usize,
    backslash_flags: c_int,
    out_buf: *mut c_char,
    out_buf_size: usize,
) -> c_int {
    if pat.is_null() || out_buf.is_null() || out_buf_size == 0 {
        return -1;
    }

    let pat_slice = std::slice::from_raw_parts(pat.cast::<u8>(), pat_len);
    let result = process_pattern_backslashes(pat_slice, backslash_flags);

    // Check if output buffer is large enough (need space for NUL)
    if result.len() >= out_buf_size {
        return -1;
    }

    // Copy result to output buffer
    std::ptr::copy_nonoverlapping(result.as_ptr(), out_buf.cast::<u8>(), result.len());

    // NUL terminate
    *out_buf.add(result.len()) = 0;

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        result.len() as c_int
    }
}

// =============================================================================
// Expansion Context Dispatch Helpers
// =============================================================================

/// Check if a context should use the file expansion path.
///
/// Returns true for contexts that expand file/directory names.
#[must_use]
pub const fn is_file_expansion_context(context: ExpandContext) -> bool {
    matches!(
        context,
        ExpandContext::Files
            | ExpandContext::Directories
            | ExpandContext::FilesInPath
            | ExpandContext::Findfunc
            | ExpandContext::DirsInCdpath
    )
}

/// FFI wrapper for file expansion context check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_file_expansion_context(context: c_int) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return 0;
    };

    c_int::from(is_file_expansion_context(ctx))
}

/// Check if a context requires regex pattern compilation.
///
/// Some contexts (like files, help, tags) don't use regex matching.
#[must_use]
pub const fn context_needs_regex(context: ExpandContext) -> bool {
    !matches!(
        context,
        ExpandContext::Files
            | ExpandContext::Directories
            | ExpandContext::FilesInPath
            | ExpandContext::Findfunc
            | ExpandContext::DirsInCdpath
            | ExpandContext::Help
            | ExpandContext::Shellcmd
            | ExpandContext::OldSetting
            | ExpandContext::Buffers
            | ExpandContext::DiffBuffers
            | ExpandContext::Tags
            | ExpandContext::TagsListfiles
            | ExpandContext::UserList
            | ExpandContext::UserLua
            | ExpandContext::Packadd
            | ExpandContext::Runtime
            | ExpandContext::PatternInBuf
            | ExpandContext::Lua
    )
}

/// FFI wrapper for regex requirement check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_context_needs_regex(context: c_int) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return 1; // Default to needing regex
    };

    c_int::from(context_needs_regex(ctx))
}

/// Check if a context is handled by ExpandOther's table dispatch.
///
/// These contexts have dedicated getter functions in the C dispatch table.
#[must_use]
pub const fn is_table_dispatched_context(context: ExpandContext) -> bool {
    matches!(
        context,
        ExpandContext::Commands
            | ExpandContext::Filetypecmd
            | ExpandContext::Mapclear
            | ExpandContext::Messages
            | ExpandContext::History
            | ExpandContext::UserCommands
            | ExpandContext::UserAddrType
            | ExpandContext::UserCmdFlags
            | ExpandContext::UserNargs
            | ExpandContext::UserComplete
            | ExpandContext::UserVars
            | ExpandContext::Functions
            | ExpandContext::UserFunc
            | ExpandContext::Expression
            | ExpandContext::Menus
            | ExpandContext::Menunames
            | ExpandContext::Syntax
            | ExpandContext::Syntime
            | ExpandContext::Highlight
            | ExpandContext::Events
            | ExpandContext::Augroup
            | ExpandContext::Sign
            | ExpandContext::Profile
            | ExpandContext::Language
            | ExpandContext::Locales
            | ExpandContext::EnvVars
            | ExpandContext::User
            | ExpandContext::Arglist
            | ExpandContext::Breakpoint
            | ExpandContext::Scriptnames
            | ExpandContext::Retab
            | ExpandContext::Checkhealth
    )
}

/// FFI wrapper for table-dispatched context check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_table_dispatched_context(context: c_int) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return 0;
    };

    c_int::from(is_table_dispatched_context(ctx))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_next_match_index_wild_next() {
        // Basic next navigation
        assert_eq!(calculate_next_match_index(WILD_NEXT, 0, 5, true, 10, 0), 1);
        assert_eq!(calculate_next_match_index(WILD_NEXT, 1, 5, true, 10, 0), 2);
        assert_eq!(calculate_next_match_index(WILD_NEXT, 3, 5, true, 10, 0), 4);

        // Next from original text (-1)
        assert_eq!(calculate_next_match_index(WILD_NEXT, -1, 5, true, 10, 0), 0);

        // Next at end wraps to original if has_orig
        assert_eq!(calculate_next_match_index(WILD_NEXT, 4, 5, true, 10, 0), -1);

        // Next at end wraps to 0 if no orig
        assert_eq!(calculate_next_match_index(WILD_NEXT, 4, 5, false, 10, 0), 0);
    }

    #[test]
    fn test_calculate_next_match_index_wild_prev() {
        // Basic prev navigation
        assert_eq!(calculate_next_match_index(WILD_PREV, 2, 5, true, 10, 0), 1);
        assert_eq!(calculate_next_match_index(WILD_PREV, 1, 5, true, 10, 0), 0);

        // Prev at first entry wraps to original if has_orig
        assert_eq!(calculate_next_match_index(WILD_PREV, 0, 5, true, 10, 0), -1);

        // Prev at first entry wraps to last if no orig
        assert_eq!(calculate_next_match_index(WILD_PREV, 0, 5, false, 10, 0), 4);

        // Prev from original goes to last
        assert_eq!(calculate_next_match_index(WILD_PREV, -1, 5, true, 10, 0), 4);
    }

    #[test]
    fn test_calculate_next_match_index_pageup_pagedown() {
        // Page down from -1 goes to 0
        assert_eq!(
            calculate_next_match_index(WILD_PAGEDOWN, -1, 10, true, 5, 0),
            0
        );

        // Page down from 0 with height 5 (effective 3) goes to 3
        assert_eq!(
            calculate_next_match_index(WILD_PAGEDOWN, 0, 10, true, 5, 0),
            3
        );

        // Page down at end goes to -1 if has_orig
        assert_eq!(
            calculate_next_match_index(WILD_PAGEDOWN, 9, 10, true, 5, 0),
            -1
        );

        // Page up from -1 goes to last
        assert_eq!(
            calculate_next_match_index(WILD_PAGEUP, -1, 10, true, 5, 0),
            9
        );

        // Page up at start goes to -1 if has_orig
        assert_eq!(
            calculate_next_match_index(WILD_PAGEUP, 0, 10, true, 5, 0),
            -1
        );
    }

    #[test]
    fn test_calculate_next_match_index_pum_want() {
        // Direct selection
        assert_eq!(
            calculate_next_match_index(WILD_PUM_WANT, 0, 10, true, 5, 3),
            3
        );
        assert_eq!(
            calculate_next_match_index(WILD_PUM_WANT, 5, 10, true, 5, 7),
            7
        );
    }

    #[test]
    fn test_calculate_next_match_index_no_files() {
        // No files returns -1
        assert_eq!(calculate_next_match_index(WILD_NEXT, 0, 0, true, 10, 0), -1);
        assert_eq!(calculate_next_match_index(WILD_PREV, 0, 0, true, 10, 0), -1);
    }

    #[test]
    fn test_calculate_file_expand_flags() {
        use crate::expand::wild_flags::WILD_ICASE;
        use ew_flags::*;

        // Files context adds EW_FILE
        let flags = calculate_file_expand_flags(ExpandContext::Files, EW_DIR, 0);
        assert_eq!(flags, EW_DIR | EW_FILE);

        // FilesInPath adds EW_FILE and EW_PATH
        let flags = calculate_file_expand_flags(ExpandContext::FilesInPath, EW_DIR, 0);
        assert_eq!(flags, EW_DIR | EW_FILE | EW_PATH);

        // Directories clears EW_FILE
        let flags = calculate_file_expand_flags(ExpandContext::Directories, EW_DIR | EW_FILE, 0);
        assert_eq!(flags, EW_DIR);

        // DirsInCdpath adds EW_CDPATH
        let flags = calculate_file_expand_flags(ExpandContext::DirsInCdpath, EW_DIR, 0);
        assert_eq!(flags, EW_DIR | EW_CDPATH);

        // WILD_ICASE adds EW_ICASE
        let flags = calculate_file_expand_flags(ExpandContext::Files, EW_DIR, WILD_ICASE);
        assert_eq!(flags, EW_DIR | EW_FILE | EW_ICASE);
    }

    #[test]
    fn test_process_pattern_backslashes_none() {
        let result = process_pattern_backslashes(b"hello world", 0);
        assert_eq!(result, b"hello world");
    }

    #[test]
    fn test_process_pattern_backslashes_one() {
        use crate::expand::XP_BS_ONE;

        // "\\ " -> " "
        let result = process_pattern_backslashes(b"hello\\ world", XP_BS_ONE);
        assert_eq!(result, b"hello world");

        // Multiple escapes
        let result = process_pattern_backslashes(b"a\\ b\\ c", XP_BS_ONE);
        assert_eq!(result, b"a b c");

        // Backslash without space is preserved
        let result = process_pattern_backslashes(b"a\\b", XP_BS_ONE);
        assert_eq!(result, b"a\\b");
    }

    #[test]
    fn test_process_pattern_backslashes_three() {
        use crate::expand::XP_BS_THREE;

        // "\\\\ " -> " "
        let result = process_pattern_backslashes(b"hello\\\\\\ world", XP_BS_THREE);
        assert_eq!(result, b"hello world");

        // Regular backslash preserved
        let result = process_pattern_backslashes(b"a\\b", XP_BS_THREE);
        assert_eq!(result, b"a\\b");
    }

    #[test]
    fn test_process_pattern_backslashes_comma() {
        use crate::expand::XP_BS_COMMA;

        // "\\\\," -> ","
        let result = process_pattern_backslashes(b"a\\\\,b", XP_BS_COMMA);
        assert_eq!(result, b"a,b");
    }

    #[test]
    fn test_is_file_expansion_context() {
        assert!(is_file_expansion_context(ExpandContext::Files));
        assert!(is_file_expansion_context(ExpandContext::Directories));
        assert!(is_file_expansion_context(ExpandContext::FilesInPath));
        assert!(is_file_expansion_context(ExpandContext::Findfunc));
        assert!(is_file_expansion_context(ExpandContext::DirsInCdpath));

        assert!(!is_file_expansion_context(ExpandContext::Commands));
        assert!(!is_file_expansion_context(ExpandContext::Help));
        assert!(!is_file_expansion_context(ExpandContext::Buffers));
    }

    #[test]
    fn test_context_needs_regex() {
        // These don't need regex
        assert!(!context_needs_regex(ExpandContext::Files));
        assert!(!context_needs_regex(ExpandContext::Help));
        assert!(!context_needs_regex(ExpandContext::Buffers));
        assert!(!context_needs_regex(ExpandContext::Tags));

        // These need regex
        assert!(context_needs_regex(ExpandContext::Commands));
        assert!(context_needs_regex(ExpandContext::Functions));
        assert!(context_needs_regex(ExpandContext::Settings));
        assert!(context_needs_regex(ExpandContext::Mappings));
    }

    #[test]
    fn test_is_table_dispatched_context() {
        // Table-dispatched contexts
        assert!(is_table_dispatched_context(ExpandContext::Commands));
        assert!(is_table_dispatched_context(ExpandContext::Functions));
        assert!(is_table_dispatched_context(ExpandContext::Highlight));
        assert!(is_table_dispatched_context(ExpandContext::Events));

        // Non-table-dispatched
        assert!(!is_table_dispatched_context(ExpandContext::Files));
        assert!(!is_table_dispatched_context(ExpandContext::Settings));
        assert!(!is_table_dispatched_context(ExpandContext::Mappings));
    }
}
