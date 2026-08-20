//! The wildcard key press.
//!
//! [`command_line_wildchar_complete`] is what `<Tab>` (and `'wildchar'`)
//! reaches: it drives `'wildmode'` through [`check_opt_wim`], calls
//! `nextwild` once per configured stage, and decides whether the popup menu
//! or the wildmenu comes up.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::{WildMode, WildOpts};
use crate::keycodes::Ctrl_Z;
use crate::options::OptWimFlags;
use crate::types::{ExpandContext, FAIL, NUL, OK};

/// Whether stage `idx` of `'wildmode'` carries `flag` (a `kOptWimFlag*`).
///
/// `wim_flags` has four entries — `'wildmode'` takes at most four
/// comma-separated stages — and [`check_opt_wim`] fills the unused tail with
/// a copy of the last one, so every index answers something.
pub(crate) fn wim_has(idx: ::core::ffi::c_int, flag: OptWimFlags) -> bool {
    wim_flags.get()[idx as usize] as OptWimFlags & flag != 0
}

/// One `'wildchar'` press: run the current `'wildmode'` stage.
pub(crate) unsafe fn command_line_wildchar_complete(s: *mut CommandLineState) -> KeyOutcome {
    unsafe {
        let cc = ccline.ptr();
        let res;
        let mut options = WildOpts::NO_BEEP;
        let escape = (*s).firstc != '@' as ::core::ffi::c_int;
        let redraw_if_menu_empty = (*s).c == K_WILD;
        let wim_noselect = p_wmnu.get() != 0 && wim_has(0, kOptWimFlagNoselect);

        if wim_has((*s).wim_index, kOptWimFlagLastused) {
            options |= WildOpts::BUFLASTUSED;
        }

        if (*s).xpc.xp_numfiles > 0 {
            // Typed 'wildchar' at least twice. If "list" is present, list the
            // matches unless they are already listed.
            if (*s).xpc.xp_numfiles > 1
                && !(*s).did_wild_list
                && wim_has((*s).wim_index, kOptWimFlagList)
            {
                showmatches(&raw mut (*s).xpc, false, true, wim_noselect);
                redrawcmd();
                (*s).did_wild_list = true;
            }
            if wim_has((*s).wim_index, kOptWimFlagLongest) {
                res = nextwild(&raw mut (*s).xpc, WildMode::Longest, options, escape);
            } else if wim_has((*s).wim_index, kOptWimFlagFull) {
                res = nextwild(&raw mut (*s).xpc, WildMode::Next, options, escape);
            } else {
                res = OK; // don't insert 'wildchar' now
            }
        } else {
            // Typed 'wildchar' for the first time.
            let wim_longest = wim_has(0, kOptWimFlagLongest);
            let wim_list = wim_has(0, kOptWimFlagList);
            let wim_full = wim_has(0, kOptWimFlagFull);

            (*s).wim_index = 0;
            if (*s).c as OptInt == p_wc.get()
                || (*s).c as OptInt == p_wcm.get()
                || (*s).c == K_WILD
                || (*s).c == Ctrl_Z
            {
                options |= WildOpts::MAY_EXPAND_PATTERN;
                if (*s).c == K_WILD {
                    options |= WildOpts::FUNC_TRIGGER;
                }
                (*s).xpc.xp_pre_incsearch_pos = (*s).is_state.search_start;
            }
            let cmdpos_before = (*cc).cmdpos;

            // If 'wildmode' starts with "longest", get the longest common
            // part.
            if wim_longest {
                res = nextwild(&raw mut (*s).xpc, WildMode::Longest, options, escape);
            } else {
                if wim_noselect || wim_list {
                    options |= WildOpts::NOSELECT;
                }
                res = nextwild(&raw mut (*s).xpc, WildMode::ExpandKeep, options, escape);
            }

            // Remove the popup menu if no completion items are available.
            if redraw_if_menu_empty && (*s).xpc.xp_numfiles <= 0 {
                pum_check_clear();
            }

            // If interrupted while completing, behave as if it failed.
            if got_int.get() {
                vpeekc(); // remove <C-C> from the input stream
                got_int.set(false); // don't abandon the command line
                expand_one(
                    &raw mut (*s).xpc,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    WildOpts::NONE,
                    WildMode::Free,
                );
                (*s).xpc.xp_context = ExpandContext::Nothing;
                return KeyOutcome::Changed;
            }

            // Display the matches.
            if res == OK && (*s).xpc.xp_numfiles > if wim_noselect { 0 } else { 1 } {
                if wim_longest {
                    let found_longest_prefix = (*cc).cmdpos != cmdpos_before;
                    if wim_list || (p_wmnu.get() != 0 && wim_full) {
                        showmatches(&raw mut (*s).xpc, p_wmnu.get() != 0, wim_list, true);
                    } else if !found_longest_prefix {
                        // Nothing was inserted, so look at what the *next*
                        // 'wildmode' stage asks for and do that now.
                        let wim_list_next = wim_has(1, kOptWimFlagList);
                        let wim_full_next = wim_has(1, kOptWimFlagFull);
                        let wim_noselect_next = wim_has(1, kOptWimFlagNoselect);
                        if wim_list_next
                            || (p_wmnu.get() != 0 && (wim_full_next || wim_noselect_next))
                        {
                            if wim_full_next && !wim_noselect_next {
                                nextwild(&raw mut (*s).xpc, WildMode::Next, options, escape);
                            } else {
                                showmatches(
                                    &raw mut (*s).xpc,
                                    p_wmnu.get() != 0,
                                    wim_list_next,
                                    wim_noselect_next,
                                );
                            }
                            if wim_list_next {
                                (*s).did_wild_list = true;
                            }
                        }
                    }
                } else if wim_list || (p_wmnu.get() != 0 && (wim_full || wim_noselect)) {
                    showmatches(&raw mut (*s).xpc, p_wmnu.get() != 0, wim_list, wim_noselect);
                } else {
                    vim_beep(kOptBoFlagWildmode as ::core::ffi::c_int as ::core::ffi::c_uint);
                }

                redrawcmd();
                if wim_list {
                    (*s).did_wild_list = true;
                }
            } else if (*s).xpc.xp_numfiles == -1 {
                (*s).xpc.xp_context = ExpandContext::Nothing;
            }
        }

        if (*s).wim_index < 3 {
            (*s).wim_index += 1;
        }

        if (*s).c == ESC {
            (*s).gotesc = true;
        }

        if res == OK {
            KeyOutcome::Changed
        } else {
            KeyOutcome::NotChanged
        }
    }
}

/// The `'wildmode'` words, in the order `check_opt_wim` tests them.  Keep in
/// sync with `opt_wim_values`.
const WIM_WORDS: [(&[u8], OptWimFlags); 5] = [
    (b"longest", kOptWimFlagLongest),
    (b"full", kOptWimFlagFull),
    (b"list", kOptWimFlagList),
    (b"lastused", kOptWimFlagLastused),
    (b"noselect", kOptWimFlagNoselect),
];

/// Read the `'wildmode'` option and fill `wim_flags[]`.  Answers `FAIL` on a
/// malformed value, leaving `wim_flags` alone.
pub unsafe fn check_opt_wim() -> ::core::ffi::c_int {
    unsafe {
        let mut new_wim_flags: [uint8_t; 4] = [0; 4];
        let mut idx = 0usize;

        let mut p = p_wim.get();
        while *p != 0 {
            // The stage name runs to the first non-alphabetic byte, which has
            // to be one of the separators.
            let mut len = 0isize;
            while ascii_isalpha(*p.offset(len) as ::core::ffi::c_int) {
                len += 1;
            }
            let after = *p.offset(len) as ::core::ffi::c_int;
            if after != NUL
                && after != ',' as ::core::ffi::c_int
                && after != ':' as ::core::ffi::c_int
            {
                return FAIL;
            }

            let word = ::core::slice::from_raw_parts(p as *const u8, len as usize);
            let Some(&(_, flag)) = WIM_WORDS.iter().find(|(name, _)| *name == word) else {
                return FAIL;
            };
            new_wim_flags[idx] |= flag as uint8_t;

            p = p.offset(len);
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                if idx == 3 {
                    return FAIL;
                }
                idx += 1;
            }
            p = p.offset(1);
        }

        // Fill the remaining entries with the last flag.
        while idx < 3 {
            new_wim_flags[idx + 1] = new_wim_flags[idx];
            idx += 1;
        }

        // Only when there are no errors is wim_flags[] changed.
        wim_flags.set(new_wim_flags);
        OK
    }
}
