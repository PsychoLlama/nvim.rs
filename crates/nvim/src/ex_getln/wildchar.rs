//! The wildcard key press.
//!
//! [`command_line_wildchar_complete`] is what `<Tab>` (and `'wildchar'`)
//! reaches: it drives `'wildmode'` through [`check_opt_wim`], calls
//! `nextwild` once per configured stage, and decides whether the popup menu
//! or the wildmenu comes up.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::cmdexpand::{WildMode, WildOpts};
use crate::cstr;
use crate::keycodes::{Ctrl_Z, Key};
use crate::option::vars::P_WIM;
use crate::options::OptWimFlags;
use crate::types::{ExpandContext, Failed, OK};

/// Whether stage `idx` of `'wildmode'` carries `flag` (a `kOptWimFlag*`).
///
/// `wim_flags` has four entries — `'wildmode'` takes at most four
/// comma-separated stages — and [`check_opt_wim`] fills the unused tail with
/// a copy of the last one, so every index answers something.
pub(crate) fn wim_has(idx: ::core::ffi::c_int, flag: OptWimFlags) -> bool {
    OptWimFlags::from(wim_flags.get()[usize::try_from(idx).unwrap_or(usize::MAX)]) & flag != 0
}

/// One `'wildchar'` press: run the current `'wildmode'` stage.
pub(crate) fn command_line_wildchar_complete(s: &mut CommandLineState) -> KeyOutcome {
    let cc = Cc::current();
    let res;
    let mut options = WildOpts::NO_BEEP;
    let redraw_if_menu_empty = s.c == Key::Wild.code();
    let wim_noselect = p_wmnu() && wim_has(0, kOptWimFlagNoselect);

    if wim_has(s.wim_index, kOptWimFlagLastused) {
        options |= WildOpts::BUFLASTUSED;
    }

    if match_count() > 0 {
        // Typed 'wildchar' at least twice. If "list" is present, list the
        // matches unless they are already listed.
        if match_count() > 1 && !s.did_wild_list && wim_has(s.wim_index, kOptWimFlagList) {
            show_matches(false, true, wim_noselect);
            redrawcmd();
            s.did_wild_list = true;
        }
        if wim_has(s.wim_index, kOptWimFlagLongest) {
            res = s.next_wild(WildMode::Longest, options);
        } else if wim_has(s.wim_index, kOptWimFlagFull) {
            res = s.next_wild(WildMode::Next, options);
        } else {
            res = OK; // don't insert 'wildchar' now
        }
    } else {
        // Typed 'wildchar' for the first time.
        let wim_longest = wim_has(0, kOptWimFlagLongest);
        let wim_list = wim_has(0, kOptWimFlagList);
        let wim_full = wim_has(0, kOptWimFlagFull);

        s.wim_index = 0;
        if OptInt::from(s.c) == p_wc()
            || OptInt::from(s.c) == p_wcm()
            || s.c == Key::Wild.code()
            || s.c == Ctrl_Z
        {
            options |= WildOpts::MAY_EXPAND_PATTERN;
            if s.c == Key::Wild.code() {
                options |= WildOpts::FUNC_TRIGGER;
            }
            let start = s.is_state.search_start;
            completion_mut(|xpc| xpc.pre_incsearch_pos = start);
        }
        let cmdpos_before = cc.cmdpos;

        // If 'wildmode' starts with "longest", get the longest common
        // part.
        if wim_longest {
            res = s.next_wild(WildMode::Longest, options);
        } else {
            if wim_noselect || wim_list {
                options |= WildOpts::NOSELECT;
            }
            res = s.next_wild(WildMode::ExpandKeep, options);
        }

        // Remove the popup menu if no completion items are available.
        if redraw_if_menu_empty && match_count() <= 0 {
            pum_check_clear();
        }

        // If interrupted while completing, behave as if it failed.
        if got_int.get() {
            vpeekc(); // remove <C-C> from the input stream
            got_int.set(false); // don't abandon the command line
            expand_completion(WildOpts::NONE, WildMode::Free);
            completion_mut(|xpc| xpc.context = ExpandContext::Nothing);
            return KeyOutcome::Changed;
        }

        // Display the matches.
        if res == OK && match_count() > if wim_noselect { 0 } else { 1 } {
            if wim_longest {
                let found_longest_prefix = cc.cmdpos != cmdpos_before;
                if wim_list || (p_wmnu() && wim_full) {
                    show_matches(p_wmnu(), wim_list, true);
                } else if !found_longest_prefix {
                    // Nothing was inserted, so look at what the *next*
                    // 'wildmode' stage asks for and do that now.
                    let wim_list_next = wim_has(1, kOptWimFlagList);
                    let wim_full_next = wim_has(1, kOptWimFlagFull);
                    let wim_noselect_next = wim_has(1, kOptWimFlagNoselect);
                    if wim_list_next || (p_wmnu() && (wim_full_next || wim_noselect_next)) {
                        if wim_full_next && !wim_noselect_next {
                            s.next_wild(WildMode::Next, options);
                        } else {
                            show_matches(p_wmnu(), wim_list_next, wim_noselect_next);
                        }
                        if wim_list_next {
                            s.did_wild_list = true;
                        }
                    }
                }
            } else if wim_list || (p_wmnu() && (wim_full || wim_noselect)) {
                show_matches(p_wmnu(), wim_list, wim_noselect);
            } else {
                vim_beep(kOptBoFlagWildmode);
            }

            redrawcmd();
            if wim_list {
                s.did_wild_list = true;
            }
        } else if match_count() == -1 {
            completion_mut(|xpc| xpc.context = ExpandContext::Nothing);
        }
    }

    if s.wim_index < 3 {
        s.wim_index += 1;
    }

    if s.c == ESC {
        s.gotesc = true;
    }

    if res == OK {
        KeyOutcome::Changed
    } else {
        KeyOutcome::NotChanged
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

/// Read the `'wildmode'` option and fill `wim_flags[]`.  Answers `Err` on a
/// malformed value, leaving `wim_flags` alone.
pub fn check_opt_wim() -> Result<(), Failed> {
    let mut new_wim_flags: [uint8_t; 4] = [0; 4];
    let mut idx = 0usize;

    // A copy: the walk below outlives a projection's borrow.
    let wim = P_WIM.get();
    let wim = wim.as_cstr().to_bytes();
    let mut p = 0;
    while p < wim.len() {
        // The stage name runs to the first non-alphabetic byte, which has
        // to be one of the separators.
        let len = wim[p..]
            .iter()
            .take_while(|b| b.is_ascii_alphabetic())
            .count();
        let after = cstr::byte_at(wim, p + len);
        if !matches!(after, 0 | b',' | b':') {
            return Err(Failed);
        }

        let word = &wim[p..p + len];
        let Some(&(_, flag)) = WIM_WORDS.iter().find(|(name, _)| *name == word) else {
            return Err(Failed);
        };
        // Every 'wildmode' flag is a bit of the low byte.
        new_wim_flags[idx] |= uint8_t::try_from(flag).unwrap_or(0);

        p += len;
        if after == 0 {
            break;
        }
        if after == b',' {
            if idx == 3 {
                return Err(Failed);
            }
            idx += 1;
        }
        p += 1;
    }

    // Fill the remaining entries with the last flag.
    while idx < 3 {
        new_wim_flags[idx + 1] = new_wim_flags[idx];
        idx += 1;
    }

    // Only when there are no errors is wim_flags[] changed.
    wim_flags.set(new_wim_flags);
    Ok(())
}
