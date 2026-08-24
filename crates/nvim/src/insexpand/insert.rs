//! Putting the selected match into the buffer, and moving between matches.
//!
//! [`ins_compl_insert`] writes the match at `compl_col` and
//! [`ins_compl_delete`] takes it out again; [`ins_compl_next`] is what
//! CTRL-N / CTRL-P reach, walking to the next match through
//! [`find_next_completion_match`] and asking for more when the list runs
//! out.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{FAIL, NUL, OK, VAR_FIXED};

/// Insert `len` bytes of `p` at the cursor, `-1` meaning up to its NUL.
pub(crate) unsafe fn ins_compl_insert_bytes(p: *mut c_char, mut len: c_int) {
    unsafe {
        if len == -1 {
            len = strlen(p) as c_int;
        }
        debug_assert!(len >= 0);
        ins_bytes_len(p, len as size_t);
        compl_ins_end_col.set((*curwin.get()).w_cursor.col);
    }
}

/// Insert `prefix` as the completion, and redraw.
pub(crate) unsafe fn ins_compl_longest_insert(prefix: *mut c_char) {
    unsafe {
        ins_compl_delete(false);
        ins_compl_insert_bytes(prefix.offset(get_compl_len() as isize), -1);
        ins_redraw(false);
    }
}

/// Insert the longest common prefix of the best fuzzy matches as `'longest'`.
pub(crate) unsafe fn fuzzy_longest_match() {
    unsafe {
        let num_bests = compl_num_bests.get();
        if num_bests == 0 {
            return;
        }

        let first = compl_first_match.get();
        let nn_compl = (*(*first).cp_next).cp_next;
        let more_candidates = !nn_compl.is_null() && nn_compl != first;

        let mut compl = if ctrl_x_mode_whole_line() {
            first
        } else {
            (*first).cp_next
        };
        if num_bests == 1 {
            // No more candidates: insert the match string itself.
            if !more_candidates {
                ins_compl_longest_insert((*compl).cp_str.data());
            }
            compl_num_bests.set(0);
            return;
        }

        // Upstream keeps these in an `xmalloc`ed array in a static that no
        // other function reads; the walk fills it from a list made cyclic by
        // `ins_compl_make_cyclic`, so `compl` is never null and every slot is
        // written. Collecting instead means the shorter-than-expected list
        // upstream would read uninitialised simply yields fewer candidates.
        let mut best: Vec<*mut compl_T> = Vec::with_capacity(num_bests as usize);
        while !compl.is_null() && best.len() < num_bests as usize {
            best.push(compl);
            compl = (*compl).cp_next;
        }

        let mut prefix = (*best[0]).cp_str.data();
        let mut prefix_len = (*best[0]).cp_str.len() as c_int;
        for &m in &best[1..] {
            let mut prefix_ptr = prefix;
            let mut match_ptr = (*m).cp_str.data();
            let mut j: c_int = 0;
            while j < prefix_len && *match_ptr as c_int != NUL && *prefix_ptr as c_int != NUL {
                if strncmp(prefix_ptr, match_ptr, utfc_ptr2len(prefix_ptr) as size_t) != 0 {
                    break;
                }
                prefix_ptr = prefix_ptr.offset(utfc_ptr2len(prefix_ptr) as isize);
                match_ptr = match_ptr.offset(utfc_ptr2len(match_ptr) as isize);
                j += 1;
            }
            if j > 0 {
                prefix_len = j;
            }
        }

        // Skip non-consecutive prefixes.
        let leader_len = ins_compl_leader_len();
        if leader_len == 0 || strncmp(prefix, ins_compl_leader(), leader_len) == 0 {
            prefix = xmemdupz(prefix.cast(), prefix_len as size_t).cast::<c_char>();
            ins_compl_longest_insert(prefix);
            xfree(prefix.cast::<c_void>());
        }
        compl_num_bests.set(0);
    }
}

/// Move `compl_shown_match` onto the match actually shown: `compl_leader` may
/// have hidden the one it points at.
pub(crate) unsafe fn ins_compl_update_shown_match() {
    unsafe {
        clear_adjusted_leader();
        let mut leader = get_leader_for_startcol(compl_shown_match.get(), true);

        // True while the leader hides the shown match and `step` can move on.
        let hidden = |leader: ComplStr, step: *mut compl_T| {
            !ins_compl_equal(compl_shown_match.get(), leader.data(), leader.len())
                && !step.is_null()
                && !is_first_match(step)
        };

        while hidden(leader, (*compl_shown_match.get()).cp_next) {
            compl_shown_match.set((*compl_shown_match.get()).cp_next);
            leader = get_leader_for_startcol(compl_shown_match.get(), true);
        }

        // If we didn't find it searching forward, and compl_shows_dir is
        // backward, find the last match.
        if compl_shows_dir_backward()
            && !ins_compl_equal(compl_shown_match.get(), leader.data(), leader.len())
            && ((*compl_shown_match.get()).cp_next.is_null()
                || is_first_match((*compl_shown_match.get()).cp_next))
        {
            while hidden(leader, (*compl_shown_match.get()).cp_prev) {
                compl_shown_match.set((*compl_shown_match.get()).cp_prev);
                leader = get_leader_for_startcol(compl_shown_match.get(), true);
            }
        }
    }
}

/// Delete the old text being completed.
pub unsafe fn ins_compl_delete(new_leader: bool) {
    unsafe {
        // Avoid deleting text that will be reinserted when changing leader.
        // This allows marks present on the original text to shrink/grow
        // appropriately.
        let mut orig_col = 0;
        if new_leader {
            let mut orig = compl_orig_text().data();
            let mut leader = ins_compl_leader();
            while *orig as c_int != NUL && utf_ptr2char(orig) == utf_ptr2char(leader) {
                leader = leader.offset(utf_ptr2len(leader) as isize);
                orig = orig.offset(utf_ptr2len(orig) as isize);
            }
            orig_col = orig.offset_from(compl_orig_text().data()) as c_int;
        }

        // In insert mode: delete the typed part.
        // In replace mode: put the old characters back, if any.
        let mut col = compl_col.get()
            + if compl_status_adding() {
                compl_length.get()
            } else {
                orig_col
            };
        if ins_compl_preinsert_effect() {
            col += ins_compl_leader_len() as c_int;
            (*curwin.get()).w_cursor.col = compl_ins_end_col.get();
        }

        // What follows the cursor on the last line, which the line deletion
        // below would take with it; re-inserted at the end.
        let mut remaining = String_0::NULL;
        if (*curwin.get()).w_cursor.lnum > compl_lnum.get() {
            if (*curwin.get()).w_cursor.col < get_cursor_line_len() {
                remaining = cbuf_to_string(get_cursor_pos_ptr(), get_cursor_pos_len() as size_t);
            }
            while (*curwin.get()).w_cursor.lnum > compl_lnum.get() {
                if ml_delete((*curwin.get()).w_cursor.lnum) == FAIL {
                    xfree(remaining.data().cast::<c_void>());
                    return;
                }
                deleted_lines_mark((*curwin.get()).w_cursor.lnum, 1);
                (*curwin.get()).w_cursor.lnum -= 1;
            }
            // Move cursor to end of line.
            (*curwin.get()).w_cursor.col = get_cursor_line_len();
        }

        if (*curwin.get()).w_cursor.col > col {
            if stop_arrow() == FAIL {
                xfree(remaining.data().cast::<c_void>());
                return;
            }
            backspace_until_column(col);
            compl_ins_end_col.set((*curwin.get()).w_cursor.col);
        }

        if !remaining.data().is_null() {
            orig_col = (*curwin.get()).w_cursor.col;
            ins_str(remaining.data(), remaining.len());
            (*curwin.get()).w_cursor.col = orig_col;
            xfree(remaining.data().cast::<c_void>());
        }

        // TODO(vim): is this sufficient for redrawing?  Redrawing everything
        // causes flicker, thus we can't do that.
        changed_cline_bef_curs(curwin.get());
        // Clear v:completed_item.
        set_vim_var_dict(Vv::CompletedItem, tv_dict_alloc_lock(VAR_FIXED));
    }
}

/// Insert a completion string that contains newlines, line by line.
pub(crate) unsafe fn ins_compl_expand_multiple(str: *mut c_char) {
    unsafe {
        let mut start = str;
        let mut curr = str;
        let base_indent = get_indent();
        while *curr as c_int != NUL {
            if *curr as c_int == '\n' as c_int {
                if curr > start {
                    ins_char_bytes(start, curr.offset_from(start) as size_t);
                }
                open_line(
                    FORWARD,
                    OPENLINE_KEEPTRAIL | OPENLINE_FORCE_INDENT,
                    base_indent,
                    ptr::null_mut(),
                );
                start = curr.offset(1);
            }
            curr = curr.offset(1);
        }
        // Handle remaining text after the last newline (if any).
        if curr > start {
            ins_char_bytes(start, curr.offset_from(start) as size_t);
        }
        compl_ins_end_col.set((*curwin.get()).w_cursor.col);
    }
}

/// Insert the new text being completed.
///
/// `move_cursor` is for `'completeopt'` `preinsert`: when true the cursor
/// moves back from the inserted text to `compl_leader`. With `insert_prefix`
/// the longest common prefix goes in instead of the shown match.
pub unsafe fn ins_compl_insert(move_cursor: bool, insert_prefix: bool) {
    unsafe {
        let shown = compl_shown_match.get();
        let compl_len = get_compl_len();
        let preinsert = ins_compl_has_preinsert();
        let mut cp_str = (*shown).cp_str.data();
        let mut cp_str_len = (*shown).cp_str.len();
        let leader_len = ins_compl_leader_len();
        let has_multiple = !strchr(cp_str, '\n' as c_int).is_null();

        if insert_prefix {
            cp_str = find_common_prefix(&raw mut cp_str_len, false);
            if cp_str.is_null() {
                cp_str = find_common_prefix(&raw mut cp_str_len, true);
                if cp_str.is_null() {
                    cp_str = (*shown).cp_str.data();
                    cp_str_len = (*shown).cp_str.len();
                }
            }
        } else if !(*cpt_sources_array.ptr()).is_null() {
            // Since completion sources may provide matches with varying start
            // positions, insert only the portion of the match that corresponds
            // to the intended replacement range.
            let cpt_idx = (*shown).cp_cpt_source_idx;
            if cpt_idx >= 0 && compl_col.get() >= 0 {
                let startcol = (*(*cpt_sources_array.ptr()).offset(cpt_idx as isize)).cs_startcol;
                if startcol >= 0 && startcol < compl_col.get() {
                    let skip = compl_col.get() - startcol;
                    if skip as size_t <= cp_str_len {
                        cp_str_len -= skip as size_t;
                        cp_str = cp_str.offset(skip as isize);
                    }
                }
            }
        }

        // Make sure we don't go over the end of the string, this can happen
        // with illegal bytes.
        if compl_len < cp_str_len as c_int {
            if has_multiple {
                ins_compl_expand_multiple(cp_str.offset(compl_len as isize));
            } else {
                ins_compl_insert_bytes(
                    cp_str.offset(compl_len as isize),
                    if insert_prefix {
                        cp_str_len as c_int - compl_len
                    } else {
                        -1
                    },
                );
                if (preinsert || insert_prefix) && move_cursor {
                    // `wrapping_sub` as the transpile has it: nothing here
                    // proves the match is longer than the leader (a fuzzy
                    // match need not start with it), and upstream's `size_t`
                    // underflow narrows to a negative `colnr_T`, i.e. the
                    // cursor moves the other way.
                    (*curwin.get()).w_cursor.col -= cp_str_len.wrapping_sub(leader_len) as colnr_T;
                }
            }
        }
        compl_used_match.set(!(match_at_original_text(shown) || (preinsert && !insert_prefix)));

        set_vim_var_dict(Vv::CompletedItem, ins_compl_dict_alloc(shown));
        compl_hi_on_autocompl_longest.set(insert_prefix && move_cursor);
    }
}

/// Step `compl_shown_match` on `todo` matches in the current direction,
/// answering the number of matches found in `num_matches`.
///
/// With `allow_get_expansion` [`ins_compl_get_exp`] may be called for more
/// completions; without it, running out in the given direction does nothing.
/// `advance` moves to the first match rather than showing the original text.
///
/// Answers `OK`, or `-1` when the number of matches is still unknown.
pub(crate) unsafe fn find_next_completion_match(
    allow_get_expansion: bool,
    mut todo: c_int,
    advance: bool,
    num_matches: *mut c_int,
) -> c_int {
    unsafe {
        let mut found_end = false;
        let mut found_compl: *mut compl_T = ptr::null_mut();
        let compl_no_select = get_cot_flags() & kOptCotFlagNoselect as c_uint != 0
            || compl_autocomplete.get() && !ins_compl_has_preinsert();

        loop {
            todo -= 1;
            if todo < 0 {
                break;
            }
            let shown = compl_shown_match.get();
            if compl_shows_dir_forward() && !(*shown).cp_next.is_null() {
                compl_shown_match.set(if !(*compl_match_array.ptr()).is_null() {
                    find_next_match_in_menu()
                } else {
                    (*shown).cp_next
                });
                found_end = !compl_first_match.get().is_null()
                    && (is_first_match((*compl_shown_match.get()).cp_next)
                        || is_first_match(compl_shown_match.get()));
            } else if compl_shows_dir_backward() && !(*shown).cp_prev.is_null() {
                found_end = is_first_match(shown);
                compl_shown_match.set(if !(*compl_match_array.ptr()).is_null() {
                    find_next_match_in_menu()
                } else {
                    (*shown).cp_prev
                });
                found_end |= is_first_match(compl_shown_match.get());
            } else {
                if !allow_get_expansion {
                    if advance {
                        if compl_shows_dir_backward() {
                            compl_pending.set(compl_pending.get() - (todo + 1));
                        } else {
                            compl_pending.set(compl_pending.get() + (todo + 1));
                        }
                    }
                    return -1;
                }

                if !compl_no_select && advance {
                    if compl_shows_dir_backward() {
                        compl_pending.set(compl_pending.get() - 1);
                    } else {
                        compl_pending.set(compl_pending.get() + 1);
                    }
                }

                // Find matches.
                *num_matches = ins_compl_get_exp(compl_startpos.get());

                // Handle any pending completions.
                while compl_pending.get() != 0
                    && compl_direction.get() == compl_shows_dir.get()
                    && advance
                {
                    let shown = compl_shown_match.get();
                    if compl_pending.get() > 0 && !(*shown).cp_next.is_null() {
                        compl_shown_match.set((*shown).cp_next);
                        compl_pending.set(compl_pending.get() - 1);
                    } else if compl_pending.get() < 0 && !(*shown).cp_prev.is_null() {
                        compl_shown_match.set((*shown).cp_prev);
                        compl_pending.set(compl_pending.get() + 1);
                    } else {
                        break;
                    }
                }
                found_end = false;
            }

            let shown = compl_shown_match.get();
            let leader = get_leader_for_startcol(shown, false);
            if !match_at_original_text(shown)
                && !leader.data().is_null()
                && !ins_compl_equal(shown, leader.data(), leader.len())
                && !(cot_fuzzy() && (*shown).cp_score != FUZZY_SCORE_NONE)
            {
                todo += 1;
            } else {
                // Remember a matching item.
                found_compl = shown;
            }

            // Stop at the end of the list when we found a usable match.
            if found_end {
                if !found_compl.is_null() {
                    compl_shown_match.set(found_compl);
                    break;
                }
                todo = 1; // use first usable match after wrapping around
            }
        }
        OK
    }
}

/// Fill in the next completion in the current direction; answers the total
/// number of matches, or `-1` if still unknown.
///
/// `compl_curr_match` belongs to [`ins_compl_get_exp`] while it runs, so this
/// works through `compl_shown_match`. It recurses at most once: first with
/// `allow_get_expansion` true, which calls [`ins_compl_get_exp`], which calls
/// back in with it false.
///
/// `count` is at least 1; `insert_match` inserts the newly selected match.
pub(crate) unsafe fn ins_compl_next(
    allow_get_expansion: bool,
    count: c_int,
    insert_match: bool,
) -> c_int {
    unsafe {
        let mut num_matches = -1;
        let started = compl_started.get();
        let orig_curbuf = curbuf.get();
        let cur_cot_flags = get_cot_flags();
        let compl_no_insert = cur_cot_flags & kOptCotFlagNoinsert as c_uint != 0
            || compl_autocomplete.get() && !ins_compl_has_preinsert();
        let compl_preinsert = ins_compl_has_preinsert();
        let has_autocomplete_delay = compl_autocomplete.get() && p_acl.get() > 0;

        // When a user completion function answers -1 for findstart, which is
        // the next time round with 'always', compl_shown_match becomes NULL.
        if compl_shown_match.get().is_null() {
            return -1;
        }

        if !compl_leader().is_unset()
            && !match_at_original_text(compl_shown_match.get())
            && !cot_fuzzy()
        {
            ins_compl_update_shown_match();
        }

        if allow_get_expansion
            && insert_match
            && (!compl_get_longest.get() || compl_used_match.get())
        {
            // Delete old text to be replaced.
            ins_compl_delete(false);
        }

        // When finding the longest common text we stick at the original text,
        // don't let CTRL-N or CTRL-P move to the first match.
        let mut advance = count != 1 || !allow_get_expansion || !compl_get_longest.get();

        // When restarting the search don't insert the first match either.
        if compl_restarting.get() {
            advance = false;
            compl_restarting.set(false);
        }

        // Repeat this for when <PageUp> or <PageDown> is typed.  But don't
        // wrap around.
        if find_next_completion_match(allow_get_expansion, count, advance, &raw mut num_matches)
            == -1
        {
            return -1;
        }

        if curbuf.get() != orig_curbuf {
            // In case some completion function switched buffer, don't insert
            // the completion elsewhere.
            return -1;
        }

        // Insert the text of the new completion, or the compl_leader.
        if !started && ins_compl_preinsert_longest() {
            ins_compl_insert(true, true);
            if has_autocomplete_delay {
                update_screen(); // Show the inserted text right away
            }
        } else if compl_no_insert && !started && !compl_preinsert {
            ins_compl_insert_bytes(
                compl_orig_text().data().offset(get_compl_len() as isize),
                -1,
            );
            compl_used_match.set(false);
            restore_orig_extmarks();
        } else if insert_match {
            if !compl_get_longest.get() || compl_used_match.get() {
                // None selected.
                let preinsert_longest = ins_compl_preinsert_longest()
                    && match_at_original_text(compl_shown_match.get());
                ins_compl_insert(compl_preinsert || preinsert_longest, preinsert_longest);
            } else {
                debug_assert!(!compl_leader().is_unset());
                ins_compl_insert_bytes(compl_leader().data().offset(get_compl_len() as isize), -1);
            }
            if strequal(
                (*compl_shown_match.get()).cp_str.data(),
                compl_orig_text().data(),
            ) {
                restore_orig_extmarks();
            }
        } else {
            compl_used_match.set(false);
        }

        if !allow_get_expansion {
            // Redraw to show the user what was inserted.
            update_screen(); // TODO(bfredl): no!
            if !has_autocomplete_delay {
                // Display the updated popup menu.
                ins_compl_show_pum();
            }
            // Delete old text to be replaced, since we're still searching and
            // don't want to match ourselves!
            ins_compl_delete(false);
        }

        // Enter will select a match when the match wasn't inserted and the
        // popup menu is visible.
        if compl_no_insert && !started && !match_at_original_text(compl_shown_match.get()) {
            compl_enter_selects.set(true);
        } else {
            compl_enter_selects.set(!insert_match && !(*compl_match_array.ptr()).is_null());
        }

        // Show the file name for the match (if any).
        if !(*compl_shown_match.get()).cp_fname.is_null() {
            ins_compl_show_filename();
        }

        num_matches
    }
}
