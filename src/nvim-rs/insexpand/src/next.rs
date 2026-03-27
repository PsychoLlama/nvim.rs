//! Completion navigation and insertion pipeline.
//!
//! This module provides Rust implementations for the completion navigation
//! functions: `find_next_completion_match` and `ins_compl_next`.

#![allow(clippy::too_many_lines)]

use std::os::raw::{c_int, c_uint};

use nvim_buffer::BufHandle;

use crate::match_list::ComplMatch;

use crate::match_list::{is_first_match, nvim_compl_get_shown_match, nvim_compl_set_shown_match};

// C accessor functions
extern "C" {
    // cot flags
    fn rs_get_cot_flags() -> c_uint;

    // match list traversal
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_get_prev(m: ComplMatch) -> ComplMatch;
    // (nvim_get_compl_match_array_exists: inlined in vars.rs)

    // (nvim_compl_shown_match_str_eq_orig: inlined in match_list.rs)

    // leader/startcol: rs_get_leader_for_startcol_data/size are defined in Rust (leader.rs)
    // (nvim_get_compl_leader_data: inlined in vars.rs)

    // equal check
    fn rs_ins_compl_equal(m: ComplMatch, str_: *const std::ffi::c_char, len: usize) -> c_int;

    // direction
    fn rs_compl_shows_dir_forward() -> c_int;
    fn rs_compl_shows_dir_backward() -> c_int;

    // navigation in menu
    fn rs_find_next_match_in_menu();

    // (compl_pending moved to Rust static in state.rs)

    // compl_direction / compl_shows_dir

    // (nvim_get_compl_startpos_lnum and _col: inlined in vars.rs)

    // fuzzy
    fn rs_cot_fuzzy() -> c_int;

    // compl state
    // (compl_restarting moved to Rust static in state.rs)

    // insert/delete operations
    fn rs_ins_compl_delete(new_leader: c_int);
    fn rs_ins_compl_insert(move_cursor: c_int, insert_prefix: c_int);
    fn rs_ins_compl_update_shown_match();
    fn rs_ins_compl_show_pum();
    fn rs_ins_compl_show_filename();
    fn nvim_update_screen();
    // nvim_ins_compl_insert_bytes: deleted (Phase 2), use rs_ins_compl_insert_bytes
    fn rs_ins_compl_insert_bytes(p: *const std::ffi::c_char, len: c_int);
    fn rs_restore_orig_extmarks();

    // compl_orig_text
    // (nvim_get_compl_orig_text_data: inlined in vars.rs)
    fn rs_get_compl_len() -> c_int;

    // enter selects

    // preinsert
    fn rs_ins_compl_has_preinsert() -> c_int;
    fn rs_ins_compl_preinsert_longest() -> c_int;

    // p_acl: inlined in vars.rs (Phase 28)

    // curbuf
    fn nvim_get_curbuf() -> BufHandle;
}

// completeopt flags
const K_OPT_COT_FLAG_NOINSERT: c_uint = 0x20;
const K_OPT_COT_FLAG_NOSELECT: c_uint = 0x40;

// Return values
const OK: c_int = 1;
const FUZZY_SCORE_NONE: c_int = i32::MIN;

/// Find the next completion match.
///
/// Iterates through matches in the popup or expands to get more completions.
/// Handles pending completions, wrapping, and direction.
///
/// This is a translation of the C `find_next_completion_match` function.
///
/// # Safety
/// Requires valid global completion state.
unsafe fn find_next_completion_match(
    allow_get_expansion: bool,
    mut todo: c_int,
    advance: bool,
    num_matches: *mut c_int,
) -> c_int {
    let mut found_compl = ComplMatch::null();
    let cur_cot_flags = rs_get_cot_flags();
    let compl_no_select = (cur_cot_flags & K_OPT_COT_FLAG_NOSELECT) != 0
        || (crate::vars::nvim_get_compl_autocomplete() != 0 && rs_ins_compl_has_preinsert() == 0);

    loop {
        todo -= 1;
        if todo < 0 {
            break;
        }

        let shown = nvim_compl_get_shown_match();
        let shown_next = nvim_compl_match_get_next(shown);
        let shown_prev = nvim_compl_match_get_prev(shown);

        let found_end: bool;
        if rs_compl_shows_dir_forward() != 0 && !shown_next.is_null() {
            if crate::vars::nvim_get_compl_match_array_exists() != 0 {
                rs_find_next_match_in_menu();
            } else {
                nvim_compl_set_shown_match(shown_next);
            }
            let updated_shown = nvim_compl_get_shown_match();
            let updated_shown_next = nvim_compl_match_get_next(updated_shown);
            found_end = !crate::match_list::compl_first_match.is_null()
                && ((!updated_shown_next.is_null() && is_first_match(updated_shown_next))
                    || is_first_match(updated_shown));
        } else if rs_compl_shows_dir_backward() != 0 && !shown_prev.is_null() {
            let was_first = is_first_match(shown);
            if crate::vars::nvim_get_compl_match_array_exists() != 0 {
                rs_find_next_match_in_menu();
            } else {
                nvim_compl_set_shown_match(shown_prev);
            }
            let updated_shown = nvim_compl_get_shown_match();
            found_end = was_first || is_first_match(updated_shown);
        } else {
            if !allow_get_expansion {
                if advance {
                    let pending = crate::state::COMPL_PENDING;
                    if rs_compl_shows_dir_backward() != 0 {
                        crate::state::COMPL_PENDING = pending - (todo + 1);
                    } else {
                        crate::state::COMPL_PENDING = pending + (todo + 1);
                    }
                }
                return -1;
            }

            if !compl_no_select && advance {
                let pending = crate::state::COMPL_PENDING;
                if rs_compl_shows_dir_backward() != 0 {
                    crate::state::COMPL_PENDING = pending - 1;
                } else {
                    crate::state::COMPL_PENDING = pending + 1;
                }
            }

            // Find matches.
            let lnum = crate::vars::nvim_get_compl_startpos_lnum();
            let col = crate::vars::nvim_get_compl_startpos_col();
            *num_matches = crate::expand::rs_ins_compl_get_exp(lnum, col);

            // Handle any pending completions
            loop {
                let pending = crate::state::COMPL_PENDING;
                if pending == 0
                    || crate::vars::nvim_get_compl_direction()
                        != crate::vars::nvim_get_compl_shows_dir()
                    || !advance
                {
                    break;
                }
                let shown2 = nvim_compl_get_shown_match();
                if pending > 0 {
                    let n = nvim_compl_match_get_next(shown2);
                    if n.is_null() {
                        break;
                    }
                    nvim_compl_set_shown_match(n);
                    crate::state::COMPL_PENDING = pending - 1;
                } else {
                    // pending < 0
                    let p = nvim_compl_match_get_prev(shown2);
                    if p.is_null() {
                        break;
                    }
                    nvim_compl_set_shown_match(p);
                    crate::state::COMPL_PENDING = pending + 1;
                }
            }
            found_end = false;
        }

        let shown = nvim_compl_get_shown_match();
        let leader_data = crate::leader::rs_get_leader_for_startcol_data(shown, 0);
        let leader_size = crate::leader::rs_get_leader_for_startcol_size(shown, 0);

        if !crate::match_list::shown_match_at_orig_text()
            && !leader_data.is_null()
            && rs_ins_compl_equal(shown, leader_data, leader_size) == 0
            && (rs_cot_fuzzy() == 0 || crate::match_list::shown_match_score() == FUZZY_SCORE_NONE)
        {
            todo += 1;
        } else {
            // Remember a matching item.
            found_compl = shown;
        }

        // Stop at the end of the list when we found a usable match.
        if found_end {
            if !found_compl.is_null() {
                nvim_compl_set_shown_match(found_compl);
                break;
            }
            todo = 1; // use first usable match after wrapping around
        }
    }

    OK
}

/// Fill in the next completion in the current direction.
///
/// If `allow_get_expansion` is true, then we may call `ins_compl_get_exp()` to
/// get more completions. If it is false, then we just do nothing when there
/// are no more completions in a given direction. The latter case is used when
/// we are still in the middle of finding completions, to allow browsing
/// through the ones found so far.
///
/// Returns the total number of matches, or -1 if still unknown.
///
/// Note that this function may be called recursively once only. First with
/// `allow_get_expansion` true, which calls `ins_compl_get_exp()`, which in turn
/// calls this function with `allow_get_expansion` false.
///
/// # Safety
/// Requires valid global completion state. Mutates many C static globals.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_next(
    allow_get_expansion: c_int,
    count: c_int,
    insert_match: c_int,
) -> c_int {
    let allow_get_expansion = allow_get_expansion != 0;
    let insert_match = insert_match != 0;

    let mut num_matches: c_int = -1;
    let todo = count;
    let started = crate::vars::nvim_get_compl_started() != 0;
    let orig_curbuf = nvim_get_curbuf();
    let cur_cot_flags = rs_get_cot_flags();
    let compl_no_insert = (cur_cot_flags & K_OPT_COT_FLAG_NOINSERT) != 0
        || (crate::vars::nvim_get_compl_autocomplete() != 0 && rs_ins_compl_has_preinsert() == 0);
    let compl_preinsert = rs_ins_compl_has_preinsert() != 0;
    let has_autocomplete_delay =
        crate::vars::nvim_get_compl_autocomplete() != 0 && crate::vars::nvim_get_p_acl() > 0;

    // When user complete function return -1 for findstart which is next
    // time of 'always', compl_shown_match become NULL.
    if crate::match_list::compl_shown_match.is_null() {
        return -1;
    }

    if !crate::vars::nvim_get_compl_leader_data().is_null()
        && !crate::match_list::shown_match_at_orig_text()
        && rs_cot_fuzzy() == 0
    {
        // Update "compl_shown_match" to the actually shown match
        rs_ins_compl_update_shown_match();
    }

    if allow_get_expansion
        && insert_match
        && (crate::vars::nvim_get_compl_get_longest() == 0
            || crate::vars::nvim_get_compl_used_match() != 0)
    {
        // Delete old text to be replaced
        rs_ins_compl_delete(0);
    }

    // When finding the longest common text we stick at the original text,
    // don't let CTRL-N or CTRL-P move to the first match.
    let mut advance =
        count != 1 || !allow_get_expansion || crate::vars::nvim_get_compl_get_longest() == 0;

    // When restarting the search don't insert the first match either.
    if crate::state::COMPL_RESTARTING {
        advance = false;
        crate::state::COMPL_RESTARTING = false;
    }

    // Repeat this for when <PageUp> or <PageDown> is typed.  But don't wrap
    // around.
    if find_next_completion_match(allow_get_expansion, todo, advance, &raw mut num_matches) == -1 {
        return -1;
    }

    if nvim_get_curbuf() != orig_curbuf {
        // In case some completion function switched buffer, don't want to
        // insert the completion elsewhere.
        return -1;
    }

    // Insert the text of the new completion, or the compl_leader.
    if !started && rs_ins_compl_preinsert_longest() != 0 {
        rs_ins_compl_insert(1, 1);
        if has_autocomplete_delay {
            nvim_update_screen(); // Show the inserted text right away
        }
    } else if compl_no_insert && !started && !compl_preinsert {
        let orig_data = crate::vars::nvim_get_compl_orig_text_data();
        let compl_len = rs_get_compl_len();
        debug_assert!(compl_len >= 0);
        #[allow(clippy::cast_sign_loss)]
        rs_ins_compl_insert_bytes(orig_data.add(compl_len as usize), -1);
        crate::vars::nvim_set_compl_used_match(0);
        rs_restore_orig_extmarks();
    } else if insert_match {
        if crate::vars::nvim_get_compl_get_longest() == 0
            || crate::vars::nvim_get_compl_used_match() != 0
        {
            let preinsert_longest = rs_ins_compl_preinsert_longest() != 0
                && crate::match_list::shown_match_at_orig_text(); // none selected
            rs_ins_compl_insert(
                c_int::from(compl_preinsert || preinsert_longest),
                c_int::from(preinsert_longest),
            );
        } else {
            let leader_data = crate::vars::nvim_get_compl_leader_data();
            debug_assert!(!leader_data.is_null());
            let compl_len = rs_get_compl_len();
            debug_assert!(compl_len >= 0);
            #[allow(clippy::cast_sign_loss)]
            rs_ins_compl_insert_bytes(leader_data.add(compl_len as usize), -1);
        }
        if crate::match_list::shown_match_str_eq_orig() {
            rs_restore_orig_extmarks();
        }
    } else {
        crate::vars::nvim_set_compl_used_match(0);
    }

    if !allow_get_expansion {
        // Redraw to show the user what was inserted
        nvim_update_screen(); // TODO(bfredl): no!

        if !has_autocomplete_delay {
            // Display the updated popup menu
            rs_ins_compl_show_pum();
        }

        // Delete old text to be replaced, since we're still searching and
        // don't want to match ourselves!
        rs_ins_compl_delete(0);
    }

    // Enter will select a match when the match wasn't inserted and the popup
    // menu is visible.
    if compl_no_insert && !started && !crate::match_list::shown_match_at_orig_text() {
        crate::vars::nvim_set_compl_enter_selects(1);
    } else {
        crate::vars::nvim_set_compl_enter_selects(c_int::from(
            !insert_match && crate::vars::nvim_get_compl_match_array_exists() != 0,
        ));
    }

    // Show the file name for the match (if any)
    if crate::match_list::shown_match_has_fname() {
        rs_ins_compl_show_filename();
    }

    num_matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_constants() {
        assert_eq!(K_OPT_COT_FLAG_NOINSERT, 0x20);
        assert_eq!(K_OPT_COT_FLAG_NOSELECT, 0x40);
    }

    #[test]
    fn test_fuzzy_score_none() {
        assert_eq!(FUZZY_SCORE_NONE, i32::MIN);
    }
}
