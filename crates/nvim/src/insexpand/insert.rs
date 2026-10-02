//! Putting the selected match into the buffer, and moving between matches.
//!
//! [`ins_compl_insert`] writes the match at `compl_col` and
//! [`ins_compl_delete`] takes it out again; [`ins_compl_next`] is what
//! CTRL-N / CTRL-P reach, walking to the next match through
//! [`find_next_completion_match`] and asking for more when the list runs
//! out.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::mbyte::cluster_len;
use crate::types::{NUL, OK, VarLock};
use crate::winlayer::{Buf, Win};

/// Insert `text` at the cursor.
pub(crate) fn ins_compl_insert_text(text: &[u8]) {
    // SAFETY: `text` is readable for its length, which is all
    // `ins_bytes_len` reads; it writes nothing through the pointer.
    unsafe { ins_bytes_len(text.as_ptr().cast_mut().cast(), text.len()) };
    compl_ins_end_col.set(Win::current().w_cursor.col);
}

/// `text` from where the typed part of it ends: the part a completion
/// still has to insert. Upstream offsets the pointer unchecked.
pub(crate) fn untyped_part(text: &[u8]) -> &[u8] {
    &text[(get_compl_len() as usize).min(text.len())..]
}

/// Insert `prefix` as the completion, and redraw.
pub(crate) fn ins_compl_longest_insert(prefix: &[u8]) {
    ins_compl_delete(false);
    ins_compl_insert_text(untyped_part(prefix));
    ins_redraw(false);
}

/// The byte length of what `prefix` and `text` share, counted the way
/// upstream counts it -- in *characters*, compared a character at a time
/// and stopping after `limit` of them -- which is then used as a byte
/// length. Kept: it only ever shortens the prefix.
fn shared_char_count(prefix: &[u8], text: &[u8], limit: usize) -> usize {
    let (mut p, mut m, mut count) = (0, 0, 0);
    while count < limit && p < prefix.len() && m < text.len() {
        let step = cluster_len(&prefix[p..]);
        if !text[m..].starts_with(&prefix[p..p + step]) {
            break;
        }
        p += step;
        m += cluster_len(&text[m..]);
        count += 1;
    }
    count
}

/// Insert the longest common prefix of the best fuzzy matches as `'longest'`.
pub(crate) fn fuzzy_longest_match() {
    let num_bests = compl_num_bests.get();
    if num_bests == 0 {
        return;
    }

    // Upstream dereferences the first two links here without checking.
    let first = first_match().expect("a fuzzy completion has matches");
    let second = first.next().expect("a fuzzy completion has a second match");
    let more_candidates = second.next().is_some_and(|nn| nn != first);

    let start = if ctrl_x_mode_whole_line() {
        first
    } else {
        second
    };
    if num_bests == 1 {
        // No more candidates: insert the match string itself.
        if !more_candidates {
            ins_compl_longest_insert(&start.text_copy());
        }
        compl_num_bests.set(0);
        return;
    }

    // The best matches, from a list `ins_compl_make_cyclic` has closed; a
    // shorter list than expected simply yields fewer candidates.
    let best: Vec<MatchId> = ::core::iter::successors(Some(start), |m| m.next())
        .take(num_bests as usize)
        .collect();
    let prefix = best[0].text_copy();
    let prefix_len = MATCHES.with(|list| {
        best[1..].iter().fold(prefix.len(), |len, &m| {
            match shared_char_count(&prefix, &list.get(m).text, len) {
                0 => len,
                shared => shared,
            }
        })
    });

    // Skip non-consecutive prefixes.
    let leader = ins_compl_leader_str().to_vec();
    if leader.is_empty() || prefix.starts_with(&leader) {
        ins_compl_longest_insert(&prefix[..prefix_len.min(prefix.len())]);
    }
    compl_num_bests.set(0);
}

/// Move `compl_shown_match` onto the match actually shown: `compl_leader` may
/// have hidden the one it points at.
pub(crate) fn ins_compl_update_shown_match() {
    clear_adjusted_leader();
    // Upstream dereferences `compl_shown_match` throughout without checking.
    let mut shown = shown_match().expect("a running completion has a shown match");
    let mut leader = get_leader_for_startcol(shown, true);

    while leader_hides(leader, shown, shown.next()) {
        shown = shown.next().expect("`leader_hides` checked the link");
        compl_shown_match.set(Some(shown));
        leader = get_leader_for_startcol(shown, true);
    }

    // If we didn't find it searching forward, and compl_shows_dir is
    // backward, find the last match.
    let equal = ins_compl_equal(shown, leader);
    if compl_shows_dir_backward() && !equal && shown.next().is_none_or(MatchId::is_first) {
        while leader_hides(leader, shown, shown.prev()) {
            shown = shown.prev().expect("`leader_hides` checked the link");
            compl_shown_match.set(Some(shown));
            leader = get_leader_for_startcol(shown, true);
        }
    }
}

/// True while `leader` hides `shown` and the walk can take one more `step`.
fn leader_hides(leader: ComplStr, shown: MatchId, step: Option<MatchId>) -> bool {
    !ins_compl_equal(shown, leader) && step.is_some_and(|step| !step.is_first())
}

/// Delete the old text being completed.
pub fn ins_compl_delete(new_leader: bool) {
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
    if ins_compl_preinsert_effect() {
        col += ins_compl_leader_len() as c_int;
        Win::current().w_cursor.col = compl_ins_end_col.get();
    }

    // What follows the cursor on the last line, which the line deletion
    // below would take with it; re-inserted at the end.
    let mut remaining = String_0::NULL;
    if Win::current().w_cursor.lnum > compl_lnum.get() {
        if Win::current().w_cursor.col < get_cursor_line_len() {
            remaining =
                unsafe { cbuf_to_string(get_cursor_pos_ptr(), get_cursor_pos_len() as size_t) };
        }
        while Win::current().w_cursor.lnum > compl_lnum.get() {
            if ml_delete(Win::current().w_cursor.lnum).is_err() {
                return;
            }
            deleted_lines_mark(Win::current().w_cursor.lnum, 1);
            Win::current().w_cursor.lnum -= 1;
        }
        // Move cursor to end of line.
        Win::current().w_cursor.col = get_cursor_line_len();
    }

    if Win::current().w_cursor.col > col {
        if stop_arrow().is_err() {
            return;
        }
        backspace_until_column(col);
        compl_ins_end_col.set(Win::current().w_cursor.col);
    }

    if !remaining.data().is_null() {
        orig_col = Win::current().w_cursor.col;
        unsafe { ins_str(remaining.data(), remaining.len()) };
        Win::current().w_cursor.col = orig_col;
    }

    // TODO(vim): is this sufficient for redrawing?  Redrawing everything
    // causes flicker, thus we can't do that.
    changed_cline_bef_curs(Win::current());
    // Clear v:completed_item.
    set_vim_var_dict(Vv::CompletedItem, Some(tv_dict_alloc_lock(VarLock::Fixed)));
}

/// Insert a completion string that contains newlines, line by line.
pub(crate) fn ins_compl_expand_multiple(text: &[u8]) {
    let base_indent = get_indent();
    let mut lines = text.split(|&b| b == b'\n');
    let mut line = lines.next().unwrap_or_default();
    for next in lines {
        insert_chars(line);
        // SAFETY: no comment leader is asked for.
        let flags = OPENLINE_KEEPTRAIL | OPENLINE_FORCE_INDENT;
        unsafe { open_line(FORWARD, flags, base_indent, ptr::null_mut()) };
        line = next;
    }
    // Handle remaining text after the last newline (if any).
    insert_chars(line);
    compl_ins_end_col.set(Win::current().w_cursor.col);
}

/// `ins_char_bytes` over `text`, which C hands it whole whatever it holds.
fn insert_chars(text: &[u8]) {
    if !text.is_empty() {
        // SAFETY: `text` is readable for its length, which is all
        // `ins_char_bytes` reads.
        unsafe { ins_char_bytes(text.as_ptr().cast_mut().cast(), text.len()) };
    }
}

/// Insert the new text being completed.
///
/// `move_cursor` is for `'completeopt'` `preinsert`: when true the cursor
/// moves back from the inserted text to `compl_leader`. With `insert_prefix`
/// the longest common prefix goes in instead of the shown match.
pub fn ins_compl_insert(move_cursor: bool, insert_prefix: bool) {
    // Upstream dereferences `compl_shown_match` here without checking.
    let shown = shown_match().expect("a running completion has a shown match");
    let compl_len = get_compl_len() as usize;
    let preinsert = ins_compl_has_preinsert();
    let leader_len = ins_compl_leader_len();
    // What to insert: `len` bytes of `text` from `start`. A copy, because
    // inserting runs buffer callbacks.
    let mut text = shown.text_copy();
    let has_multiple = text.contains(&b'\n');
    let (mut start, mut len) = (0, text.len());

    if insert_prefix {
        if let Some((prefix, prefix_len)) =
            find_common_prefix(false).or_else(|| find_common_prefix(true))
        {
            (text, len) = (prefix, prefix_len);
        }
    } else if !cpt_sources().is_unset() {
        // Since completion sources may provide matches with varying start
        // positions, insert only the portion of the match that corresponds
        // to the intended replacement range.
        let cpt_idx = shown.with(|m| m.cpt_source_idx);
        if cpt_idx >= 0 && compl_col.get() >= 0 {
            let startcol = cpt_sources().row(cpt_idx).cs_startcol;
            if startcol >= 0 && startcol < compl_col.get() {
                let skip = (compl_col.get() - startcol) as usize;
                if skip <= len {
                    len -= skip;
                    start = skip;
                }
            }
        }
    }

    // Make sure we don't go over the end of the string, this can happen
    // with illegal bytes.
    if compl_len < len {
        // Several lines go in up to the text's end, as upstream's walk to
        // the NUL does even for a prefix.
        let rest = &text[start + compl_len..];
        if has_multiple {
            ins_compl_expand_multiple(rest);
        } else {
            ins_compl_insert_text(if insert_prefix {
                &rest[..len - compl_len]
            } else {
                rest
            });
            if (preinsert || insert_prefix) && move_cursor {
                // `wrapping_sub` as the transpile has it: nothing here
                // proves the match is longer than the leader (a fuzzy
                // match need not start with it), and upstream's `size_t`
                // underflow narrows to a negative `ColNr`, i.e. the
                // cursor moves the other way.
                Win::current().w_cursor.col -= len.wrapping_sub(leader_len) as ColNr;
            }
        }
    }

    // Upstream reads the shown match afresh: inserting ran buffer callbacks.
    if let Some(shown) = shown_match() {
        compl_used_match.set(!(shown.is_original() || (preinsert && !insert_prefix)));
        set_vim_var_dict(Vv::CompletedItem, Some(ins_compl_dict_alloc(shown)));
    }
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
pub(crate) fn find_next_completion_match(
    allow_get_expansion: bool,
    mut todo: c_int,
    advance: bool,
    num_matches: &mut c_int,
) -> c_int {
    let mut found_end;
    let mut found_compl: Option<MatchId> = None;
    let has_preinsert = ins_compl_has_preinsert();
    let compl_no_select = completeopt_flags() & kOptCotFlagNoselect as c_uint != 0
        || compl_autocomplete.get() && !has_preinsert;

    loop {
        todo -= 1;
        if todo < 0 {
            break;
        }
        // Upstream dereferences `compl_shown_match` here without checking.
        let shown = shown_match().expect("a running completion has a shown match");
        let (next, prev) = shown.with(|m| (m.next, m.prev));
        if compl_shows_dir_forward() && next.is_some() {
            let next = if !compl_match_array().is_unset() {
                Some(find_next_match_in_menu())
            } else {
                next
            };
            compl_shown_match.set(next);
            let now = shown_match().expect("just set from a link");
            found_end = first_match().is_some() && (is_first_match(now.next()) || now.is_first());
        } else if compl_shows_dir_backward() && prev.is_some() {
            found_end = shown.is_first();
            let prev = if !compl_match_array().is_unset() {
                Some(find_next_match_in_menu())
            } else {
                prev
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

            // Find matches. That runs user code, which can free the list
            // (`complete()` from a timer) and leave a remembered match
            // naming nothing.
            let list = match_list_generation();
            *num_matches = ins_compl_get_exp(compl_startpos.get());
            if match_list_generation() != list {
                found_compl = None;
            }

            // Handle any pending completions.
            while compl_pending.get() != 0
                && compl_direction.get() == compl_shows_dir.get()
                && advance
            {
                // Upstream dereferences `compl_shown_match` here unchecked.
                let shown = shown_match().expect("a running completion has a shown match");
                let (next, prev) = shown.with(|m| (m.next, m.prev));
                if compl_pending.get() > 0 && next.is_some() {
                    compl_shown_match.set(next);
                    compl_pending.set(compl_pending.get() - 1);
                } else if compl_pending.get() < 0 && prev.is_some() {
                    compl_shown_match.set(prev);
                    compl_pending.set(compl_pending.get() + 1);
                } else {
                    break;
                }
            }
            found_end = false;
        }

        let shown = shown_match().expect("a running completion has a shown match");
        let leader = get_leader_for_startcol(shown, false);
        let hidden = !shown.is_original()
            && !leader.is_unset()
            && !ins_compl_equal(shown, leader)
            && !(cot_fuzzy() && shown.with(|m| m.score) != FUZZY_SCORE_NONE);
        if hidden {
            todo += 1;
        } else {
            // Remember a matching item.
            found_compl = Some(shown);
        }

        // Stop at the end of the list when we found a usable match.
        if found_end {
            if let Some(found) = found_compl {
                compl_shown_match.set(Some(found));
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
pub(crate) fn ins_compl_next(allow_get_expansion: bool, count: c_int, insert_match: bool) -> c_int {
    let mut num_matches = -1;
    let started = compl_started.get();
    // Taken as an identity, not an address: a completion function can wipe
    // the buffer and the allocator can hand the same address back, so the
    // pointer comparison upstream does cannot tell "still here" from "gone
    // and replaced". See the re-entry rule in [`crate::winlayer`].
    let orig_curbuf = Buf::current().id();
    let cur_cot_flags = completeopt_flags();
    let compl_preinsert = ins_compl_has_preinsert();
    let compl_no_insert = cur_cot_flags & kOptCotFlagNoinsert as c_uint != 0
        || compl_autocomplete.get() && !compl_preinsert;
    let has_autocomplete_delay = compl_autocomplete.get() && p_acl() > 0;

    // When a user completion function answers -1 for findstart, which is
    // the next time round with 'always', compl_shown_match becomes NULL.
    let Some(shown) = shown_match() else {
        return -1;
    };

    if !compl_leader().is_unset() && !shown.is_original() && !cot_fuzzy() {
        ins_compl_update_shown_match();
    }

    if allow_get_expansion && insert_match && (!compl_get_longest.get() || compl_used_match.get()) {
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
    if find_next_completion_match(allow_get_expansion, count, advance, &mut num_matches) == -1 {
        return -1;
    }

    if Buf::current().id() != orig_curbuf {
        // In case some completion function switched buffer, don't insert
        // the completion elsewhere.
        return -1;
    }

    // Insert the text of the new completion, or the compl_leader.
    if !started && ins_compl_preinsert_longest() {
        ins_compl_insert(true, true);
        if has_autocomplete_delay {
            let _ = update_screen(); // Show the inserted text right away
        }
    } else if compl_no_insert && !started && !compl_preinsert {
        ins_compl_insert_text(untyped_part(&compl_orig_text().to_vec()));
        compl_used_match.set(false);
        compl_orig_extmarks().restore();
    } else if insert_match {
        if !compl_get_longest.get() || compl_used_match.get() {
            // None selected.
            let preinsert_longest =
                ins_compl_preinsert_longest() && shown_match().is_some_and(MatchId::is_original);
            ins_compl_insert(compl_preinsert || preinsert_longest, preinsert_longest);
        } else {
            debug_assert!(!compl_leader().is_unset());
            ins_compl_insert_text(untyped_part(&compl_leader().to_vec()));
        }
        // C's `strequal(compl_shown_match->cp_str.data, compl_orig_text.data)`.
        let orig = compl_orig_text();
        let shown_is_orig_text = match shown_match() {
            Some(shown) => {
                !orig.is_unset() && orig.with_bytes(|orig| shown.with(|m| *m.text == *orig))
            }
            None => orig.is_unset(),
        };
        if shown_is_orig_text {
            compl_orig_extmarks().restore();
        }
    } else {
        compl_used_match.set(false);
    }

    if !allow_get_expansion {
        // Redraw to show the user what was inserted.
        let _ = update_screen(); // TODO(bfredl): no!
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
    let shown_is_orig = shown_match().is_some_and(MatchId::is_original);
    if compl_no_insert && !started && !shown_is_orig {
        compl_enter_selects.set(true);
    } else {
        compl_enter_selects.set(!insert_match && !compl_match_array().is_unset());
    }

    // Show the file name for the match (if any).
    if shown_match().is_some_and(|shown| shown.with(|m| m.fname.is_some())) {
        ins_compl_show_filename();
    }

    num_matches
}
