//! Putting the selected match into the buffer, and moving between matches.
//!
//! [`ins_compl_insert`] writes the match at `compl_col` and
//! [`ins_compl_delete`] takes it out again; [`ins_compl_next`] is what
//! CTRL-N / CTRL-P reach, walking to the next match through
//! [`find_next_completion_match`] and asking for more when the list runs
//! out.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::types::{NUL, OK, VarLock};
use crate::winlayer::{Buf, Win};

/// Insert `len` bytes of `p` at the cursor, `-1` meaning up to its NUL.
pub(crate) unsafe fn ins_compl_insert_bytes(p: *mut c_char, mut len: c_int) {
    if len == -1 {
        len = unsafe { strlen(p) } as c_int;
    }
    debug_assert!(len >= 0);
    unsafe { ins_bytes_len(p, len as size_t) };
    compl_ins_end_col.set(cur_win().w_cursor.col);
}

/// Insert `prefix` as the completion, and redraw.
pub(crate) unsafe fn ins_compl_longest_insert(prefix: *mut c_char) {
    unsafe { ins_compl_delete(false) };
    unsafe { ins_compl_insert_bytes(prefix.offset(get_compl_len() as isize), -1) };
    unsafe { ins_redraw(false) };
}

/// Insert the longest common prefix of the best fuzzy matches as `'longest'`.
pub(crate) unsafe fn fuzzy_longest_match() {
    let num_bests = compl_num_bests.get();
    if num_bests == 0 {
        return;
    }

    // Upstream dereferences the first two links here without checking.
    let first = first_match().expect("a fuzzy completion has matches");
    let second = first.next().expect("a fuzzy completion has a second match");
    let more_candidates = second.next().is_some_and(|nn| nn != first);

    let mut compl = Some(if ctrl_x_mode_whole_line() {
        first
    } else {
        second
    });
    if num_bests == 1 {
        // No more candidates: insert the match string itself.
        if !more_candidates {
            let text = compl.expect("just set").cp_str.data();
            // SAFETY: a match's text is NUL-terminated, and a completion is
            // running.
            unsafe { ins_compl_longest_insert(text) };
        }
        compl_num_bests.set(0);
        return;
    }

    // Upstream keeps these in an `xmalloc`ed array in a static that no
    // other function reads; the walk fills it from a list made cyclic by
    // `ins_compl_make_cyclic`, so `compl` is never null and every slot is
    // written. Collecting instead means the shorter-than-expected list
    // upstream would read uninitialised simply yields fewer candidates.
    let mut best: Vec<Cm> = Vec::with_capacity(num_bests as usize);
    while let Some(node) = compl {
        if best.len() >= num_bests as usize {
            break;
        }
        best.push(node);
        compl = node.next();
    }

    let prefix = best[0].cp_str.data();
    let mut prefix_len = best[0].cp_str.len() as c_int;
    for &m in &best[1..] {
        let mut prefix_ptr = prefix;
        let mut match_ptr = m.cp_str.data();
        let mut j: c_int = 0;
        while j < prefix_len {
            // SAFETY: both pointers walk a match's NUL-terminated text and
            // stop at its NUL.
            let (at_match_end, at_prefix_end) =
                unsafe { (*match_ptr as c_int == NUL, *prefix_ptr as c_int == NUL) };
            if at_match_end || at_prefix_end {
                break;
            }
            // SAFETY: as above -- one whole character of each.
            let (prefix_step, match_step) =
                unsafe { (utfc_ptr2len(prefix_ptr), utfc_ptr2len(match_ptr)) };
            // SAFETY: as above.
            if !unsafe { cstr::prefix_eq(prefix_ptr, match_ptr, prefix_step as size_t) } {
                break;
            }
            // SAFETY: as above -- the step lands on the next character.
            prefix_ptr = unsafe { prefix_ptr.offset(prefix_step as isize) };
            // SAFETY: as above.
            match_ptr = unsafe { match_ptr.offset(match_step as isize) };
            j += 1;
        }
        if j > 0 {
            prefix_len = j;
        }
    }

    // Skip non-consecutive prefixes.
    let leader_len = ins_compl_leader_len();
    // SAFETY: the leader is readable for `leader_len` bytes and `prefix` is
    // a match's NUL-terminated text.
    let consecutive = unsafe { cstr::prefix_eq(prefix, ins_compl_leader(), leader_len) };
    if leader_len == 0 || consecutive {
        // SAFETY: `prefix_len` bytes of `prefix`, copied with a NUL added.
        let copy = unsafe { xmemdupz(prefix.cast(), prefix_len as size_t) }.cast::<c_char>();
        // SAFETY: a NUL-terminated string, and a completion is running.
        unsafe { ins_compl_longest_insert(copy) };
        // SAFETY: the copy just made, which nothing else holds.
        unsafe { xfree(copy.cast::<c_void>()) };
    }
    compl_num_bests.set(0);
}

/// Move `compl_shown_match` onto the match actually shown: `compl_leader` may
/// have hidden the one it points at.
pub(crate) unsafe fn ins_compl_update_shown_match() {
    clear_adjusted_leader();
    // Upstream dereferences `compl_shown_match` throughout without checking.
    let mut shown = shown_match().expect("a running completion has a shown match");
    // SAFETY: `shown` is a live node of the match list.
    let mut leader = unsafe { get_leader_for_startcol(shown, true) };

    loop {
        // SAFETY: the leader is readable for its own length.
        let hidden = unsafe { leader_hides(leader, shown, shown.next()) };
        if !hidden {
            break;
        }
        shown = shown.next().expect("`leader_hides` checked the link");
        compl_shown_match.set(shown.raw());
        // SAFETY: as above.
        leader = unsafe { get_leader_for_startcol(shown, true) };
    }

    // If we didn't find it searching forward, and compl_shows_dir is
    // backward, find the last match.
    // SAFETY: as above.
    let equal = unsafe { ins_compl_equal(shown, leader.data(), leader.len()) };
    if compl_shows_dir_backward() && !equal && shown.next().is_none_or(Cm::is_first) {
        loop {
            // SAFETY: as above.
            let hidden = unsafe { leader_hides(leader, shown, shown.prev()) };
            if !hidden {
                break;
            }
            shown = shown.prev().expect("`leader_hides` checked the link");
            compl_shown_match.set(shown.raw());
            // SAFETY: as above.
            leader = unsafe { get_leader_for_startcol(shown, true) };
        }
    }
}

/// True while `leader` hides `shown` and the walk can take one more `step`.
///
/// # Safety
/// `leader` is readable for its own length.
unsafe fn leader_hides(leader: ComplStr, shown: Cm, step: Option<Cm>) -> bool {
    // SAFETY: the caller's promise.
    let equal = unsafe { ins_compl_equal(shown, leader.data(), leader.len()) };
    !equal && step.is_some_and(|step| !step.is_first())
}

/// Delete the old text being completed.
pub unsafe fn ins_compl_delete(new_leader: bool) {
    // Avoid deleting text that will be reinserted when changing leader.
    // This allows marks present on the original text to shrink/grow
    // appropriately.
    let mut orig_col = 0;
    if new_leader {
        let mut orig = compl_orig_text().data();
        let mut leader = ins_compl_leader();
        while unsafe { *orig } as c_int != NUL
            && unsafe { utf_ptr2char(orig) } == unsafe { utf_ptr2char(leader) }
        {
            leader = unsafe { leader.offset(utf_ptr2len(leader) as isize) };
            orig = unsafe { orig.offset(utf_ptr2len(orig) as isize) };
        }
        orig_col = unsafe { orig.offset_from(compl_orig_text().data()) } as c_int;
    }

    // In insert mode: delete the typed part.
    // In replace mode: put the old characters back, if any.
    let mut col = compl_col.get()
        + if compl_status_adding() {
            compl_length.get()
        } else {
            orig_col
        };
    if unsafe { ins_compl_preinsert_effect() } {
        col += ins_compl_leader_len() as c_int;
        cur_win().w_cursor.col = compl_ins_end_col.get();
    }

    // What follows the cursor on the last line, which the line deletion
    // below would take with it; re-inserted at the end.
    let mut remaining = String_0::NULL;
    if cur_win().w_cursor.lnum > compl_lnum.get() {
        if cur_win().w_cursor.col < get_cursor_line_len() {
            remaining =
                unsafe { cbuf_to_string(get_cursor_pos_ptr(), get_cursor_pos_len() as size_t) };
        }
        while cur_win().w_cursor.lnum > compl_lnum.get() {
            if unsafe { ml_delete(cur_win().w_cursor.lnum) }.is_err() {
                unsafe { xfree(remaining.data().cast::<c_void>()) };
                return;
            }
            unsafe { deleted_lines_mark(cur_win().w_cursor.lnum, 1) };
            cur_win().w_cursor.lnum -= 1;
        }
        // Move cursor to end of line.
        cur_win().w_cursor.col = get_cursor_line_len();
    }

    if cur_win().w_cursor.col > col {
        if unsafe { stop_arrow() }.is_err() {
            unsafe { xfree(remaining.data().cast::<c_void>()) };
            return;
        }
        unsafe { backspace_until_column(col) };
        compl_ins_end_col.set(cur_win().w_cursor.col);
    }

    if !remaining.data().is_null() {
        orig_col = cur_win().w_cursor.col;
        unsafe { ins_str(remaining.data(), remaining.len()) };
        cur_win().w_cursor.col = orig_col;
        unsafe { xfree(remaining.data().cast::<c_void>()) };
    }

    // TODO(vim): is this sufficient for redrawing?  Redrawing everything
    // causes flicker, thus we can't do that.
    changed_cline_bef_curs(unsafe { Win::current() });
    // Clear v:completed_item.
    unsafe { set_vim_var_dict(Vv::CompletedItem, tv_dict_alloc_lock(VarLock::Fixed)) };
}

/// Insert a completion string that contains newlines, line by line.
pub(crate) unsafe fn ins_compl_expand_multiple(str: *mut c_char) {
    let mut start = str;
    let mut curr = str;
    let base_indent = get_indent();
    while unsafe { *curr } as c_int != NUL {
        if unsafe { *curr } as c_int == '\n' as c_int {
            if curr > start {
                unsafe { ins_char_bytes(start, curr.offset_from(start) as size_t) };
            }
            unsafe {
                open_line(
                    FORWARD,
                    OPENLINE_KEEPTRAIL | OPENLINE_FORCE_INDENT,
                    base_indent,
                    ptr::null_mut(),
                )
            };
            start = unsafe { curr.offset(1) };
        }
        curr = unsafe { curr.offset(1) };
    }
    // Handle remaining text after the last newline (if any).
    if curr > start {
        unsafe { ins_char_bytes(start, curr.offset_from(start) as size_t) };
    }
    compl_ins_end_col.set(cur_win().w_cursor.col);
}

/// Insert the new text being completed.
///
/// `move_cursor` is for `'completeopt'` `preinsert`: when true the cursor
/// moves back from the inserted text to `compl_leader`. With `insert_prefix`
/// the longest common prefix goes in instead of the shown match.
pub unsafe fn ins_compl_insert(move_cursor: bool, insert_prefix: bool) {
    // Upstream dereferences `compl_shown_match` here without checking.
    let shown = shown_match().expect("a running completion has a shown match");
    let compl_len = get_compl_len();
    // SAFETY: no precondition left; still an `unsafe fn` for its call sites
    // outside this family.
    let preinsert = unsafe { ins_compl_has_preinsert() };
    let mut cp_str = shown.cp_str.data();
    let mut cp_str_len = shown.cp_str.len();
    let leader_len = ins_compl_leader_len();
    let has_multiple = !unsafe { strchr(cp_str, '\n' as c_int) }.is_null();

    if insert_prefix {
        cp_str = unsafe { find_common_prefix(&raw mut cp_str_len, false) };
        if cp_str.is_null() {
            cp_str = unsafe { find_common_prefix(&raw mut cp_str_len, true) };
            if cp_str.is_null() {
                cp_str = shown.cp_str.data();
                cp_str_len = shown.cp_str.len();
            }
        }
    } else if !cpt_sources().is_unset() {
        // Since completion sources may provide matches with varying start
        // positions, insert only the portion of the match that corresponds
        // to the intended replacement range.
        let cpt_idx = shown.cp_cpt_source_idx;
        if cpt_idx >= 0 && compl_col.get() >= 0 {
            let startcol = cpt_sources().row(cpt_idx).cs_startcol;
            if startcol >= 0 && startcol < compl_col.get() {
                let skip = compl_col.get() - startcol;
                if skip as size_t <= cp_str_len {
                    cp_str_len -= skip as size_t;
                    cp_str = unsafe { cp_str.offset(skip as isize) };
                }
            }
        }
    }

    // Make sure we don't go over the end of the string, this can happen
    // with illegal bytes.
    if compl_len < cp_str_len as c_int {
        if has_multiple {
            unsafe { ins_compl_expand_multiple(cp_str.offset(compl_len as isize)) };
        } else {
            unsafe {
                ins_compl_insert_bytes(
                    cp_str.offset(compl_len as isize),
                    if insert_prefix {
                        cp_str_len as c_int - compl_len
                    } else {
                        -1
                    },
                )
            };
            if (preinsert || insert_prefix) && move_cursor {
                // `wrapping_sub` as the transpile has it: nothing here
                // proves the match is longer than the leader (a fuzzy
                // match need not start with it), and upstream's `size_t`
                // underflow narrows to a negative `colnr_T`, i.e. the
                // cursor moves the other way.
                cur_win().w_cursor.col -= cp_str_len.wrapping_sub(leader_len) as colnr_T;
            }
        }
    }
    compl_used_match.set(!(shown.is_original() || (preinsert && !insert_prefix)));

    // SAFETY: `shown` is a live node, and the fresh dict is handed over.
    unsafe { set_vim_var_dict(Vv::CompletedItem, ins_compl_dict_alloc(shown.raw())) };
    compl_hi_on_autocompl_longest.set(insert_prefix && move_cursor);
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
    let mut found_end = false;
    let mut found_compl: Option<Cm> = None;
    // SAFETY: no precondition left; still an `unsafe fn` for its call sites
    // outside this family.
    let has_preinsert = unsafe { ins_compl_has_preinsert() };
    let compl_no_select = completeopt_flags() & kOptCotFlagNoselect as c_uint != 0
        || compl_autocomplete.get() && !has_preinsert;

    loop {
        todo -= 1;
        if todo < 0 {
            break;
        }
        // Upstream dereferences `compl_shown_match` here without checking.
        let shown = shown_match().expect("a running completion has a shown match");
        if compl_shows_dir_forward() && !shown.cp_next.is_null() {
            // SAFETY: a completion with a shown match is running.
            let next = if !compl_match_array().is_unset() {
                unsafe { find_next_match_in_menu() }.raw()
            } else {
                shown.cp_next
            };
            compl_shown_match.set(next);
            let now = shown_match().expect("just set from a non-null link");
            found_end = first_match().is_some() && (is_first_match(now.cp_next) || now.is_first());
        } else if compl_shows_dir_backward() && !shown.cp_prev.is_null() {
            found_end = shown.is_first();
            // SAFETY: as above.
            let prev = if !compl_match_array().is_unset() {
                unsafe { find_next_match_in_menu() }.raw()
            } else {
                shown.cp_prev
            };
            compl_shown_match.set(prev);
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
            unsafe { *num_matches = ins_compl_get_exp(compl_startpos.get()) };

            // Handle any pending completions.
            while compl_pending.get() != 0
                && compl_direction.get() == compl_shows_dir.get()
                && advance
            {
                // Upstream dereferences `compl_shown_match` here unchecked.
                let shown = shown_match().expect("a running completion has a shown match");
                if compl_pending.get() > 0 && !shown.cp_next.is_null() {
                    compl_shown_match.set(shown.cp_next);
                    compl_pending.set(compl_pending.get() - 1);
                } else if compl_pending.get() < 0 && !shown.cp_prev.is_null() {
                    compl_shown_match.set(shown.cp_prev);
                    compl_pending.set(compl_pending.get() + 1);
                } else {
                    break;
                }
            }
            found_end = false;
        }

        let shown = shown_match().expect("a running completion has a shown match");
        // SAFETY: `shown` is a live node of the match list.
        let leader = unsafe { get_leader_for_startcol(shown, false) };
        // SAFETY: the leader is readable for its own length.
        let hidden = !shown.is_original()
            && !leader.data().is_null()
            && !unsafe { ins_compl_equal(shown, leader.data(), leader.len()) }
            && !(cot_fuzzy() && shown.cp_score != FUZZY_SCORE_NONE);
        if hidden {
            todo += 1;
        } else {
            // Remember a matching item.
            found_compl = Some(shown);
        }

        // Stop at the end of the list when we found a usable match.
        if found_end {
            if let Some(found) = found_compl {
                compl_shown_match.set(found.raw());
                break;
            }
            todo = 1; // use first usable match after wrapping around
        }
    }
    OK
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
    let mut num_matches = -1;
    let started = compl_started.get();
    // Taken as an identity, not an address: a completion function can wipe
    // the buffer and the allocator can hand the same address back, so the
    // pointer comparison upstream does cannot tell "still here" from "gone
    // and replaced". See the re-entry rule in [`crate::winlayer`].
    let orig_curbuf = cur_buf().id();
    let cur_cot_flags = completeopt_flags();
    // SAFETY: no precondition left; still an `unsafe fn` for its call sites
    // outside this family.
    let compl_preinsert = unsafe { ins_compl_has_preinsert() };
    let compl_no_insert = cur_cot_flags & kOptCotFlagNoinsert as c_uint != 0
        || compl_autocomplete.get() && !compl_preinsert;
    let has_autocomplete_delay = compl_autocomplete.get() && p_acl.get() > 0;

    // When a user completion function answers -1 for findstart, which is
    // the next time round with 'always', compl_shown_match becomes NULL.
    let Some(shown) = shown_match() else {
        return -1;
    };

    if !compl_leader().is_unset() && !shown.is_original() && !cot_fuzzy() {
        // SAFETY: a completion with a shown match is running.
        unsafe { ins_compl_update_shown_match() };
    }

    if allow_get_expansion && insert_match && (!compl_get_longest.get() || compl_used_match.get()) {
        // Delete old text to be replaced.
        unsafe { ins_compl_delete(false) };
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
    if unsafe {
        find_next_completion_match(allow_get_expansion, count, advance, &raw mut num_matches)
    } == -1
    {
        return -1;
    }

    if cur_buf().id() != orig_curbuf {
        // In case some completion function switched buffer, don't insert
        // the completion elsewhere.
        return -1;
    }

    // Insert the text of the new completion, or the compl_leader.
    // SAFETY: no precondition left; still an `unsafe fn` for its call sites
    // outside this family.
    if !started && unsafe { ins_compl_preinsert_longest() } {
        unsafe { ins_compl_insert(true, true) };
        if has_autocomplete_delay {
            let _ = unsafe { update_screen() }; // Show the inserted text right away
        }
    } else if compl_no_insert && !started && !compl_preinsert {
        unsafe {
            ins_compl_insert_bytes(
                compl_orig_text().data().offset(get_compl_len() as isize),
                -1,
            )
        };
        compl_used_match.set(false);
        unsafe { compl_orig_extmarks().restore() };
    } else if insert_match {
        if !compl_get_longest.get() || compl_used_match.get() {
            // None selected.
            // SAFETY: as above.
            let preinsert_longest = unsafe { ins_compl_preinsert_longest() }
                && shown_match().is_some_and(Cm::is_original);
            unsafe { ins_compl_insert(compl_preinsert || preinsert_longest, preinsert_longest) };
        } else {
            debug_assert!(!compl_leader().is_unset());
            unsafe {
                ins_compl_insert_bytes(compl_leader().data().offset(get_compl_len() as isize), -1)
            };
        }
        let shown_text = shown_match().map_or(ptr::null_mut(), |shown| shown.cp_str.data());
        // SAFETY: both are NUL-terminated or null, which `strequal` takes.
        if unsafe { strequal(shown_text, compl_orig_text().data()) } {
            unsafe { compl_orig_extmarks().restore() };
        }
    } else {
        compl_used_match.set(false);
    }

    if !allow_get_expansion {
        // Redraw to show the user what was inserted.
        let _ = unsafe { update_screen() }; // TODO(bfredl): no!
        if !has_autocomplete_delay {
            // Display the updated popup menu.
            unsafe { ins_compl_show_pum() };
        }
        // Delete old text to be replaced, since we're still searching and
        // don't want to match ourselves!
        unsafe { ins_compl_delete(false) };
    }

    // Enter will select a match when the match wasn't inserted and the
    // popup menu is visible.
    let shown_is_orig = shown_match().is_some_and(Cm::is_original);
    if compl_no_insert && !started && !shown_is_orig {
        compl_enter_selects.set(true);
    } else {
        compl_enter_selects.set(!insert_match && !compl_match_array().is_unset());
    }

    // Show the file name for the match (if any).
    if shown_match().is_some_and(|shown| !shown.cp_fname.is_null()) {
        // SAFETY: a completion with a shown match is running.
        unsafe { ins_compl_show_filename() };
    }

    num_matches
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
