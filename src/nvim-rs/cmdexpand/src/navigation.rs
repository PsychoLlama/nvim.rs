//! Match navigation functions for command-line completion.
//!
//! This module provides functions for navigating between completion matches
//! (next/prev/pageup/pagedown), starting expansions, and finding the longest
//! common match prefix.

use libc::{c_char, c_int};

use crate::context::wild_mode::{
    WILD_ALL, WILD_ALL_KEEP, WILD_EXPAND_FREE, WILD_LONGEST, WILD_NEXT, WILD_PAGEDOWN, WILD_PAGEUP,
    WILD_PREV,
};
use crate::context::wild_options::{WILD_NO_BEEP, WILD_SILENT};
use crate::context::ExpandContext;
use crate::ExpandHandle;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // expand_T field accessors
    fn nvim_expand_get_numfiles(xp: ExpandHandle) -> c_int;
    fn nvim_expand_get_selected(xp: ExpandHandle) -> c_int;
    fn nvim_expand_set_selected(xp: ExpandHandle, selected: c_int);
    fn nvim_expand_get_context(xp: ExpandHandle) -> c_int;
    fn nvim_expand_get_orig(xp: ExpandHandle) -> *const c_char;
    fn nvim_expand_get_files_item(xp: ExpandHandle, i: c_int) -> *const c_char;

    // Popup menu functions
    fn pum_get_height() -> c_int;
    fn pum_clear();

    // Static variable accessors
    fn nvim_get_compl_match_array_not_null() -> c_int;
    fn nvim_set_compl_selected(val: c_int);
    fn nvim_get_cmd_showtail() -> c_int;
    fn nvim_get_p_wmnu() -> c_int;
    fn nvim_get_p_fic() -> c_int;
    fn nvim_get_pum_want_active() -> c_int;
    fn nvim_get_pum_want_item() -> c_int;

    // Display functions (still in C, called via FFI wrappers)
    fn nvim_cmdexpand_pum_display(changed_array: c_int);
    fn nvim_cmdexpand_pum_create_for_nav(xp: ExpandHandle, showtail: c_int, noselect: c_int);
    fn nvim_cmdexpand_redraw_wildmenu(
        xp: ExpandHandle,
        num_matches: c_int,
        findex: c_int,
        showtail: c_int,
    );

    // Completion check (already migrated, but available via C)
    fn rs_cmdline_compl_use_pum(need_wildmenu: c_int) -> c_int;

    // String/memory functions
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xmemdupz(s: *const c_char, len: usize) -> *mut c_char;

    // Expansion functions (still in C)
    fn nvim_cmdexpand_expand_from_context(
        xp: ExpandHandle,
        pat: *const c_char,
        options: c_int,
    ) -> c_int;
    fn nvim_cmdexpand_expand_escape(xp: ExpandHandle, str_: *const c_char, options: c_int);
    fn nvim_cmdexpand_match_suffix(xp: ExpandHandle, i: c_int) -> c_int;

    // Message functions
    fn nvim_cmdexpand_semsg_nomatch(str_: *const c_char);
    fn nvim_cmdexpand_emsg_toomany();
    fn beep_flush();
    fn vim_beep(flag: c_int);

    // Multibyte functions
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn mb_tolower(c: c_int) -> c_int;
}

/// `kOptBoFlagWildmode` value (generated enum, 0x80000).
const K_OPT_BO_FLAG_WILDMODE: c_int = 0x0008_0000;

// =============================================================================
// `get_next_or_prev_match`
// =============================================================================

/// Navigate to the next or previous completion match.
///
/// Handles `WILD_NEXT`, `WILD_PREV`, `WILD_PAGEUP`, `WILD_PAGEDOWN`, and
/// `WILD_PUM_WANT` modes.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle with valid file list if
/// `xp_numfiles > 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_next_or_prev_match(mode: c_int, xp: ExpandHandle) -> *mut c_char {
    let numfiles = nvim_expand_get_numfiles(xp);
    if numfiles <= 0 {
        return std::ptr::null_mut();
    }

    let mut findex = nvim_expand_get_selected(xp);

    if mode == WILD_PREV {
        if findex == -1 {
            findex = numfiles;
        }
        findex -= 1;
    } else if mode == WILD_NEXT {
        findex += 1;
    } else if mode == WILD_PAGEUP || mode == WILD_PAGEDOWN {
        let mut ht = pum_get_height();
        if ht > 3 {
            ht -= 2;
        }

        if mode == WILD_PAGEUP {
            match findex.cmp(&0) {
                std::cmp::Ordering::Equal => findex = -1,
                std::cmp::Ordering::Less => findex = numfiles - 1,
                std::cmp::Ordering::Greater => {
                    findex -= ht;
                    if findex < 0 {
                        findex = 0;
                    }
                }
            }
        } else {
            // WILD_PAGEDOWN
            match findex.cmp(&(numfiles - 1)) {
                std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => findex = -1,
                std::cmp::Ordering::Less => {
                    if findex < 0 {
                        findex = 0;
                    } else {
                        findex += ht;
                        if findex > numfiles - 1 {
                            findex = numfiles - 1;
                        }
                    }
                }
            }
        }
    } else {
        // WILD_PUM_WANT
        debug_assert!(nvim_get_pum_want_active() != 0);
        findex = nvim_get_pum_want_item();
    }

    // Handle wrapping around
    if findex < 0 || findex >= numfiles {
        if !nvim_expand_get_orig(xp).is_null() {
            findex = -1;
        } else if findex < 0 {
            findex = numfiles - 1;
        } else {
            findex = 0;
        }
    }

    // Display matches on screen
    if nvim_get_p_wmnu() != 0 {
        if nvim_get_compl_match_array_not_null() != 0 {
            nvim_set_compl_selected(findex);
            nvim_cmdexpand_pum_display(0); // false
        } else if rs_cmdline_compl_use_pum(1) != 0 {
            let showtail = nvim_get_cmd_showtail();
            nvim_cmdexpand_pum_create_for_nav(xp, showtail, 0); // noselect=false
            nvim_set_compl_selected(findex);
            pum_clear();
            nvim_cmdexpand_pum_display(1); // true
        } else {
            let showtail = nvim_get_cmd_showtail();
            nvim_cmdexpand_redraw_wildmenu(xp, numfiles, findex, showtail);
        }
    }

    nvim_expand_set_selected(xp, findex);

    // Return the original text or the selected match
    if findex == -1 {
        xstrdup(nvim_expand_get_orig(xp))
    } else {
        xstrdup(nvim_expand_get_files_item(xp, findex))
    }
}

// =============================================================================
// `ExpandOne_start`
// =============================================================================

/// Start a new expansion: call `ExpandFromContext`, escape results, and check
/// suffix matching.
///
/// Returns the first matching file (allocated), or NULL.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `str_` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_expand_one_start(
    mode: c_int,
    xp: ExpandHandle,
    str_: *const c_char,
    options: c_int,
) -> *mut c_char {
    // Do the expansion.
    let result = nvim_cmdexpand_expand_from_context(xp, str_, options);
    let numfiles = nvim_expand_get_numfiles(xp);

    if result == FAIL {
        // FNAME_ILLEGAL is not defined on Linux, skip that branch.
        return std::ptr::null_mut();
    }

    if numfiles == 0 {
        if options & WILD_SILENT == 0 {
            nvim_cmdexpand_semsg_nomatch(str_);
        }
        return std::ptr::null_mut();
    }

    // Escape the matches for use on the command line.
    nvim_cmdexpand_expand_escape(xp, str_, options);

    // Check for matching suffixes in file names.
    if mode != WILD_ALL && mode != WILD_ALL_KEEP && mode != WILD_LONGEST {
        let non_suf_match = if numfiles > 0 {
            let mut nsm = numfiles;
            let ctx = nvim_expand_get_context(xp);
            if (ctx == ExpandContext::Files.to_raw() || ctx == ExpandContext::Directories.to_raw())
                && numfiles > 1
            {
                // More than one match; check suffix.
                nsm = 0;
                for i in 0..2 {
                    if nvim_cmdexpand_match_suffix(xp, i) != 0 {
                        nsm += 1;
                    }
                }
            }
            nsm
        } else {
            1
        };

        if non_suf_match != 1 {
            if options & WILD_SILENT == 0 {
                nvim_cmdexpand_emsg_toomany();
            } else if options & WILD_NO_BEEP == 0 {
                beep_flush();
            }
        }

        if !(non_suf_match != 1 && mode == WILD_EXPAND_FREE) {
            return xstrdup(nvim_expand_get_files_item(xp, 0));
        }
    }

    std::ptr::null_mut()
}

/// `FAIL` constant (0).
const FAIL: c_int = 0;

// =============================================================================
// `find_longest_match`
// =============================================================================

/// Find the longest common prefix among all completion matches.
///
/// Returns the longest common string (allocated).
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle with at least one file match.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_find_longest_match(xp: ExpandHandle, options: c_int) -> *mut c_char {
    let numfiles = nvim_expand_get_numfiles(xp);
    let first_file = nvim_expand_get_files_item(xp, 0);

    let mut len: usize = 0;
    loop {
        let c_at_len = *first_file.add(len);
        if c_at_len == 0 {
            break;
        }

        let mb_len = utfc_ptr2len(first_file.add(len)) as usize;
        let c0 = utf_ptr2char(first_file.add(len));

        let ctx = nvim_expand_get_context(xp);
        let use_icase = nvim_get_p_fic() != 0
            && (ctx == ExpandContext::Directories.to_raw()
                || ctx == ExpandContext::Files.to_raw()
                || ctx == ExpandContext::Shellcmd.to_raw()
                || ctx == ExpandContext::Buffers.to_raw());

        let mut all_match = true;
        for i in 1..numfiles {
            let ci = utf_ptr2char(nvim_expand_get_files_item(xp, i).add(len));
            if use_icase {
                if mb_tolower(c0) != mb_tolower(ci) {
                    all_match = false;
                    break;
                }
            } else if c0 != ci {
                all_match = false;
                break;
            }
        }

        if !all_match {
            if options & WILD_NO_BEEP == 0 {
                vim_beep(K_OPT_BO_FLAG_WILDMODE);
            }
            break;
        }

        len += mb_len;
    }

    xmemdupz(first_file, len)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fail_constant() {
        assert_eq!(FAIL, 0);
    }

    #[test]
    fn test_wildmode_flag() {
        assert_eq!(K_OPT_BO_FLAG_WILDMODE, 0x0008_0000);
    }
}
