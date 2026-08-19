//! The `compl_T` match list: adding, freeing and ordering the matches.
//!
//! [`ins_compl_add`] links a new match into the circular doubly-linked list
//! `compl_first_match` heads, rejecting duplicates unless the caller allows
//! them; [`ins_compl_make_cyclic`] closes the ring and
//! [`ins_compl_make_linear`] opens it again.  The comparators and
//! [`sort_compl_match_list`] are `'completeopt'`'s `fuzzy` and `nearest`
//! orderings.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{FAIL, NUL, OK, VAR_FIXED};

/// Free the four `cptext` strings a caller handed to [`ins_compl_add`].
#[inline]
pub(crate) unsafe fn free_cptext(cptext: *const *mut c_char) {
    if cptext.is_null() {
        return;
    }
    for i in 0..CPT_COUNT as isize {
        unsafe { xfree((*cptext.offset(i)).cast::<c_void>()) };
    }
}

/// Add one match to the list.
///
/// `str`/`len` is the text (`len < 0` measures it), `fname` the file it came
/// from, `cptext` the four `abbr`/`kind`/`menu`/`info` strings (exactly
/// `CPT_COUNT` of them, taken over rather than copied when
/// `cptext_allocated`), `cdir` the side of `compl_curr_match` to link it on
/// (`kDirectionNotSet` means `compl_direction`), and `adup` whether a
/// duplicate is acceptable.
///
/// Returns `NOTDONE` when the text is already in the list, `FAIL` on
/// interrupt, `OK` when it was linked in.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn ins_compl_add(
    str: *mut c_char,
    mut len: c_int,
    fname: *mut c_char,
    cptext: *const *mut c_char,
    cptext_allocated: bool,
    user_data: *mut typval_T,
    cdir: Direction,
    flags_arg: c_int,
    adup: bool,
    user_hl: *const c_int,
    score: c_int,
) -> c_int {
    unsafe {
        let dir = if cdir == kDirectionNotSet {
            compl_direction.get()
        } else {
            cdir
        };
        let mut flags = flags_arg;

        if flags & CP_FAST != 0 {
            fast_breakcheck();
        } else {
            os_breakcheck();
        }
        if got_int.get() {
            if cptext_allocated {
                free_cptext(cptext);
            }
            return FAIL;
        }
        if len < 0 {
            len = strlen(str) as c_int;
        }

        // If the same match is already present, don't add it.
        if !compl_first_match.get().is_null() && !adup {
            let mut m = compl_first_match.get();
            loop {
                if !match_at_original_text(m)
                    && strncmp((*m).cp_str.data(), str, len as size_t) == 0
                    && ((*m).cp_str.len() as c_int <= len
                        || *(*m).cp_str.data().offset(len as isize) as c_int == NUL)
                {
                    if is_nearest_active() && score > 0 && score < (*m).cp_score {
                        (*m).cp_score = score;
                    }
                    if cptext_allocated {
                        free_cptext(cptext);
                    }
                    return NOTDONE;
                }
                m = (*m).cp_next;
                if m.is_null() || is_first_match(m) {
                    break;
                }
            }
        }

        // Remove any popup menu before changing the list of matches.
        ins_compl_del_pum();

        let match_0 = xcalloc(1, size_of::<compl_T>()) as *mut compl_T;
        (*match_0).cp_number = if flags & CP_ORIGINAL_TEXT != 0 { 0 } else { -1 };
        (*match_0).cp_str = cbuf_to_string(str, len as size_t);

        // The match's fname is `compl_curr_match`'s when it is an equal
        // string, else a copy of `fname` (with CP_FREE_FNAME so it is freed
        // later), else NULL.  -- Acevedo
        let curr = compl_curr_match.get();
        if !fname.is_null()
            && !curr.is_null()
            && !(*curr).cp_fname.is_null()
            && strcmp(fname, (*curr).cp_fname) == 0
        {
            (*match_0).cp_fname = (*curr).cp_fname;
        } else if !fname.is_null() {
            (*match_0).cp_fname = xstrdup(fname);
            flags |= CP_FREE_FNAME;
        } else {
            (*match_0).cp_fname = ptr::null_mut();
        }
        (*match_0).cp_flags = flags;
        (*match_0).cp_user_abbr_hlattr = if user_hl.is_null() { -1 } else { *user_hl };
        (*match_0).cp_user_kind_hlattr = if user_hl.is_null() {
            -1
        } else {
            *user_hl.add(1)
        };
        (*match_0).cp_score = score;
        (*match_0).cp_cpt_source_idx = cpt_sources_index.get();

        if !cptext.is_null() {
            for i in 0..CPT_COUNT as isize {
                let text = *cptext.offset(i);
                if text.is_null() {
                    continue;
                }
                if *text as c_int != NUL {
                    (*match_0).cp_text[i as usize] = if cptext_allocated {
                        text
                    } else {
                        xstrdup(text)
                    };
                } else if cptext_allocated {
                    xfree(text.cast::<c_void>());
                }
            }
        }

        if !user_data.is_null() {
            (*match_0).cp_user_data = *user_data;
        }

        // Link the new match after (FORWARD) or before (BACKWARD) the current
        // match in the list.
        let first = compl_first_match.get();
        if first.is_null() {
            (*match_0).cp_prev = ptr::null_mut();
            (*match_0).cp_next = ptr::null_mut();
        } else if cot_fuzzy() && score != FUZZY_SCORE_NONE && compl_get_longest.get() {
            // The direction is ignored under `longest` + `fuzzy`, because
            // matches are inserted sorted by score.
            let mut current = (*first).cp_next;
            let mut prev = first;
            let mut inserted = false;
            while !current.is_null() && current != first {
                if (*current).cp_score < score {
                    (*match_0).cp_next = current;
                    (*match_0).cp_prev = (*current).cp_prev;
                    if !(*current).cp_prev.is_null() {
                        (*(*current).cp_prev).cp_next = match_0;
                    }
                    (*current).cp_prev = match_0;
                    inserted = true;
                    break;
                }
                prev = current;
                current = (*current).cp_next;
            }
            if !inserted {
                (*prev).cp_next = match_0;
                (*match_0).cp_prev = prev;
                (*match_0).cp_next = first;
                (*first).cp_prev = match_0;
            }
        } else if dir == FORWARD {
            (*match_0).cp_next = (*curr).cp_next;
            (*match_0).cp_prev = curr;
        } else {
            (*match_0).cp_next = curr;
            (*match_0).cp_prev = (*curr).cp_prev;
        }
        if !(*match_0).cp_next.is_null() {
            (*(*match_0).cp_next).cp_prev = match_0;
        }
        if (*match_0).cp_prev.is_null() {
            // Nothing before it: it is the first match.
            compl_first_match.set(match_0);
        } else {
            (*(*match_0).cp_prev).cp_next = match_0;
        }
        compl_curr_match.set(match_0);

        // Find the longest common string if still doing that.
        if compl_get_longest.get()
            && flags & CP_ORIGINAL_TEXT == 0
            && !cot_fuzzy()
            && !ins_compl_preinsert_longest()
            && !ctrl_x_mode_thesaurus()
        {
            ins_compl_longest_match(match_0);
        }
        OK
    }
}

/// Does `str[..len]` match `match_0`'s text, honouring its `CP_ICASE` /
/// `CP_EQUAL` flags?
pub(crate) unsafe fn ins_compl_equal(match_0: *mut compl_T, str: *mut c_char, len: size_t) -> bool {
    unsafe {
        if (*match_0).cp_flags & CP_EQUAL != 0 {
            return true;
        }
        if (*match_0).cp_flags & CP_ICASE != 0 {
            return strncasecmp((*match_0).cp_str.data(), str, len) == 0;
        }
        strncmp((*match_0).cp_str.data(), str, len) == 0
    }
}

/// Shorten `compl_leader` to the longest prefix it shares with `match_0`, and
/// put that prefix in the buffer.
pub(crate) unsafe fn ins_compl_longest_match(match_0: *mut compl_T) {
    unsafe {
        if compl_leader.get().data().is_null() {
            compl_leader.set(copy_string((*match_0).cp_str, ptr::null_mut::<Arena>()));
            let had_match = (*curwin.get()).w_cursor.col > compl_col.get();
            ins_compl_longest_insert(compl_leader.get().data());
            if !had_match {
                ins_compl_delete(false);
            }
            compl_used_match.set(false);
            return;
        }

        let mut p = compl_leader.get().data();
        let mut s = (*match_0).cp_str.data();
        while *p as c_int != NUL {
            let c1 = utf_ptr2char(p);
            let c2 = utf_ptr2char(s);
            let differ = if (*match_0).cp_flags & CP_ICASE != 0 {
                mb_tolower(c1) != mb_tolower(c2)
            } else {
                c1 != c2
            };
            if differ {
                break;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
            s = s.offset(utfc_ptr2len(s) as isize);
        }

        if *p as c_int != NUL {
            *p = NUL as c_char;
            let leader = compl_leader.get();
            compl_leader.set(String_0::from_raw_parts(
                leader.data(),
                p.offset_from(leader.data()) as size_t,
            ));
            let had_match = (*curwin.get()).w_cursor.col > compl_col.get();
            ins_compl_longest_insert(compl_leader.get().data());
            if !had_match {
                ins_compl_delete(false);
            }
        }
        compl_used_match.set(false);
    }
}

/// Add every string of an expansion's `matches` array, then free the array.
pub(crate) unsafe fn ins_compl_add_matches(
    num_matches: c_int,
    matches: *mut *mut c_char,
    icase: c_int,
) {
    unsafe {
        let mut dir = compl_direction.get();
        for i in 0..num_matches as isize {
            let add_r = ins_compl_add(
                *matches.offset(i),
                -1,
                ptr::null_mut(),
                ptr::null(),
                false,
                ptr::null_mut(),
                dir,
                CP_FAST | if icase != 0 { CP_ICASE } else { 0 },
                false,
                ptr::null(),
                FUZZY_SCORE_NONE,
            );
            if add_r == FAIL {
                break;
            }
            if add_r == OK {
                dir = FORWARD;
            }
        }
        FreeWild(num_matches, matches);
    }
}

/// Close the list into a ring; returns the number of matches after the first.
pub(crate) unsafe fn ins_compl_make_cyclic() -> c_int {
    unsafe {
        let first = compl_first_match.get();
        if first.is_null() {
            return 0;
        }
        let mut m = first;
        let mut count = 0;
        while !(*m).cp_next.is_null() && !is_first_match((*m).cp_next) {
            m = (*m).cp_next;
            count += 1;
        }
        (*m).cp_next = first;
        (*first).cp_prev = m;
        count
    }
}

/// Open the ring back into a NULL-terminated list.
pub(crate) unsafe fn ins_compl_make_linear() {
    unsafe {
        let first = compl_first_match.get();
        if first.is_null() || (*first).cp_prev.is_null() {
            return;
        }
        (*(*first).cp_prev).cp_next = ptr::null_mut();
        (*first).cp_prev = ptr::null_mut();
    }
}

// The four link accessors `mergesort_list` walks the list through, and the two
// score comparators it orders it by.  All six are held as function pointers,
// so they keep their C ABI.

pub(crate) unsafe fn cp_get_next(node: *mut c_void) -> *mut c_void {
    unsafe { (*(node as *mut compl_T)).cp_next as *mut c_void }
}

pub(crate) unsafe fn cp_set_next(node: *mut c_void, next: *mut c_void) {
    unsafe { (*(node as *mut compl_T)).cp_next = next as *mut compl_T };
}

pub(crate) unsafe fn cp_get_prev(node: *mut c_void) -> *mut c_void {
    unsafe { (*(node as *mut compl_T)).cp_prev as *mut c_void }
}

pub(crate) unsafe fn cp_set_prev(node: *mut c_void, prev: *mut c_void) {
    unsafe { (*(node as *mut compl_T)).cp_prev = prev as *mut compl_T };
}

/// Highest fuzzy score first.
pub(crate) unsafe fn cp_compare_fuzzy(a: *const c_void, b: *const c_void) -> c_int {
    let (score_a, score_b) = unsafe {
        (
            (*(a as *const compl_T)).cp_score,
            (*(b as *const compl_T)).cp_score,
        )
    };
    score_b.cmp(&score_a) as c_int
}

/// Nearest to the cursor first; unscored matches compare equal to everything.
pub(crate) unsafe fn cp_compare_nearest(a: *const c_void, b: *const c_void) -> c_int {
    let (score_a, score_b) = unsafe {
        (
            (*(a as *const compl_T)).cp_score,
            (*(b as *const compl_T)).cp_score,
        )
    };
    if score_a == FUZZY_SCORE_NONE || score_b == FUZZY_SCORE_NONE {
        return 0;
    }
    score_a.cmp(&score_b) as c_int
}

/// Order two indices into `compl_fuzzy_scores` by score, highest first, with
/// the index itself as the tie-break — so the order is total and the sort is
/// the permutation upstream's `qsort` produced.
pub(crate) unsafe extern "C" fn compare_scores(a: *const c_void, b: *const c_void) -> c_int {
    unsafe {
        let idx_a = *(a as *const c_int);
        let idx_b = *(b as *const c_int);
        let scores = compl_fuzzy_scores.get();
        let score_a = *scores.offset(idx_a as isize);
        let score_b = *scores.offset(idx_b as isize);
        if score_a == score_b {
            idx_a.cmp(&idx_b) as c_int
        } else {
            score_b.cmp(&score_a) as c_int
        }
    }
}

/// Score every match against the leader (or, with no leader, against the
/// original text).
pub(crate) unsafe fn set_fuzzy_score() {
    unsafe {
        let first = compl_first_match.get();
        if first.is_null() {
            return;
        }

        // Determine the pattern to match against.
        let leader = compl_leader.get();
        let use_leader = !leader.data().is_null() && leader.len() > 0;
        let mut pattern: *mut c_char = ptr::null_mut();
        if use_leader {
            // Clear the leader cache once before the loop; the pattern is
            // then computed per match, since each may have its own startcol.
            get_leader_for_startcol(ptr::null_mut(), true);
        } else {
            let orig = compl_orig_text.get();
            if orig.data().is_null() || orig.len() == 0 {
                return;
            }
            pattern = orig.data();
        }

        let mut comp = first;
        loop {
            if use_leader {
                pattern = (*get_leader_for_startcol(comp, true)).data();
            }
            (*comp).cp_score = fuzzy_match_str((*comp).cp_str.data(), pattern);
            comp = (*comp).cp_next;
            if comp.is_null() || is_first_match(comp) {
                break;
            }
        }
    }
}

/// Sort the match list with `compare`, leaving the node holding the leader
/// (the original text) where it is.
pub(crate) unsafe fn sort_compl_match_list(compare: MergeSortCompareFunc) {
    unsafe {
        let first = compl_first_match.get();
        if first.is_null() || is_first_match((*first).cp_next) {
            return;
        }

        let comp = (*first).cp_prev;
        ins_compl_make_linear();
        if compl_shows_dir_forward() {
            // The leader sits at the head; sort everything after it.
            (*(*first).cp_next).cp_prev = ptr::null_mut();
            (*first).cp_next = mergesort_list(
                (*first).cp_next as *mut c_void,
                Some(cp_get_next),
                Some(cp_set_next),
                Some(cp_get_prev),
                Some(cp_set_prev),
                compare,
            ) as *mut compl_T;
            (*(*first).cp_next).cp_prev = first;
        } else {
            // The leader sits at the tail; sort everything before it.
            (*(*comp).cp_prev).cp_next = ptr::null_mut();
            compl_first_match.set(mergesort_list(
                first as *mut c_void,
                Some(cp_get_next),
                Some(cp_set_next),
                Some(cp_get_prev),
                Some(cp_set_prev),
                compare,
            ) as *mut compl_T);
            let mut tail = compl_first_match.get();
            while !(*tail).cp_next.is_null() {
                tail = (*tail).cp_next;
            }
            (*tail).cp_next = comp;
            (*comp).cp_prev = tail;
        }
        ins_compl_make_cyclic();
    }
}

/// Free one match and everything hanging off it.
pub(crate) unsafe fn ins_compl_item_free(match_0: *mut compl_T) {
    unsafe {
        xfree((*match_0).cp_str.data().cast::<c_void>());
        (*match_0).cp_str = String_0::NULL;
        if (*match_0).cp_flags & CP_FREE_FNAME != 0 {
            xfree((*match_0).cp_fname.cast::<c_void>());
        }
        free_cptext((&raw mut (*match_0).cp_text).cast::<*mut c_char>());
        tv_clear(&raw mut (*match_0).cp_user_data);
        xfree(match_0.cast::<c_void>());
    }
}

/// Free the whole match list and the pattern and leader that built it.
pub(crate) unsafe fn ins_compl_free() {
    unsafe {
        clear_string(&compl_pattern);
        clear_string(&compl_leader);

        if compl_first_match.get().is_null() {
            return;
        }

        ins_compl_del_pum();
        pum_clear();

        compl_curr_match.set(compl_first_match.get());
        loop {
            let m = compl_curr_match.get();
            compl_curr_match.set((*m).cp_next);
            ins_compl_item_free(m);
            let next = compl_curr_match.get();
            if next.is_null() || is_first_match(next) {
                break;
            }
        }
        compl_curr_match.set(ptr::null_mut());
        compl_first_match.set(ptr::null_mut());
        compl_shown_match.set(ptr::null_mut());
        compl_old_match.set(ptr::null_mut());
    }
}

/// Reset everything a completion left behind, without freeing the list.
pub unsafe fn ins_compl_clear() {
    unsafe {
        compl_cont_status.set(0);
        compl_started.set(false);
        compl_matches.set(0);
        compl_selected_item.set(-1);
        compl_ins_end_col.set(0);
        compl_curr_win.set(ptr::null_mut());
        compl_curr_buf.set(ptr::null_mut());
        clear_string(&compl_pattern);
        clear_string(&compl_leader);
        edit_submode_extra.set(ptr::null_mut());
        xfree(compl_orig_extmarks.get().items.cast::<c_void>());
        compl_orig_extmarks.set(EXTMARK_UNDO_VEC_INIT);
        clear_string(&compl_orig_text);
        compl_enter_selects.set(false);
        cpt_sources_clear();
        compl_autocomplete.set(false);
        compl_from_nonkeyword.set(false);
        compl_num_bests.set(0);
        set_vim_var_dict(Vv::CompletedItem, tv_dict_alloc_lock(VAR_FIXED));
    }
}

/// Score the matches and, unless `'completeopt'` says `nosort`, reorder them.
pub(crate) unsafe fn ins_compl_fuzzy_sort() {
    unsafe {
        let cur_cot_flags = get_cot_flags();

        set_fuzzy_score();
        if cur_cot_flags & kOptCotFlagNosort != 0 {
            return;
        }
        sort_compl_match_list(Some(cp_compare_fuzzy));

        // Sorting reorders the items, so the shown one has to be reset.
        if cur_cot_flags & (kOptCotFlagNoinsert | kOptCotFlagNoselect) != kOptCotFlagNoinsert {
            return;
        }
        let first = compl_first_match.get();
        let none_selected = compl_shown_match.get()
            == if compl_shows_dir_forward() {
                first
            } else {
                (*first).cp_prev
            };
        if !none_selected {
            compl_shown_match.set(if !compl_autocomplete.get() && compl_shows_dir_forward() {
                (*first).cp_next
            } else {
                first
            });
        }
    }
}

/// Number the matches around `compl_curr_match`, in the direction the
/// completion is running.
pub(crate) unsafe fn ins_compl_update_sequence_numbers() {
    unsafe {
        let mut number = 0;
        let mut m;
        if compl_dir_forward() {
            // Search backwards for the first match with a number.
            m = (*compl_curr_match.get()).cp_prev;
            while !m.is_null() && !is_first_match(m) {
                if (*m).cp_number != -1 {
                    number = (*m).cp_number;
                    break;
                }
                m = (*m).cp_prev;
            }
            if !m.is_null() {
                // Go up and assign all numbers which are not assigned yet.
                m = (*m).cp_next;
                while !m.is_null() && (*m).cp_number == -1 {
                    number += 1;
                    (*m).cp_number = number;
                    m = (*m).cp_next;
                }
            }
        } else {
            debug_assert!(compl_direction.get() == BACKWARD);
            // Search forwards (upwards) for the first match with a number.
            m = (*compl_curr_match.get()).cp_next;
            while !m.is_null() && !is_first_match(m) {
                if (*m).cp_number != -1 {
                    number = (*m).cp_number;
                    break;
                }
                m = (*m).cp_next;
            }
            if !m.is_null() {
                // Go down and assign all numbers which are not assigned yet.
                m = (*m).cp_prev;
                while !m.is_null() && (*m).cp_number == -1 {
                    number += 1;
                    (*m).cp_number = number;
                    m = (*m).cp_prev;
                }
            }
        }
    }
}

/// Drop every match the current `'complete'` source contributed, so it can be
/// re-run (`refresh: 'always'`).
pub(crate) unsafe fn remove_old_matches() {
    unsafe {
        let mut shown_match_removed = false;
        let forward = (*compl_first_match.get()).cp_cpt_source_idx < 0;

        if cpt_sources_index.get() < 0 {
            return;
        }

        compl_direction.set(if forward { FORWARD } else { BACKWARD });
        compl_shows_dir.set(compl_direction.get());

        // Under `'completeopt'` `fuzzy` the items are not in source order, so
        // they have to be removed one by one rather than as a run.
        let mut current = compl_first_match.get();
        while !current.is_null() {
            if (*current).cp_cpt_source_idx != cpt_sources_index.get() {
                current = (*current).cp_next;
                continue;
            }
            let to_delete = current;
            if !shown_match_removed && compl_shown_match.get() == current {
                shown_match_removed = true;
            }
            current = (*current).cp_next;

            if to_delete == compl_first_match.get() {
                // Head.
                compl_first_match.set((*to_delete).cp_next);
                (*compl_first_match.get()).cp_prev = ptr::null_mut();
            } else if (*to_delete).cp_next.is_null() {
                // Tail.
                (*(*to_delete).cp_prev).cp_next = ptr::null_mut();
            } else {
                // Middle.
                (*(*to_delete).cp_prev).cp_next = (*to_delete).cp_next;
                (*(*to_delete).cp_next).cp_prev = (*to_delete).cp_prev;
            }
            ins_compl_item_free(to_delete);
        }

        if shown_match_removed {
            if forward {
                compl_shown_match.set(compl_first_match.get());
            } else {
                // The last node carries the prefix being completed.
                let mut last = compl_first_match.get();
                while !(*last).cp_next.is_null() {
                    last = (*last).cp_next;
                }
                compl_shown_match.set(last);
            }
        }

        compl_curr_match.set(compl_first_match.get());
        let mut current = compl_first_match.get();
        while !current.is_null() {
            let before = if forward {
                (*current).cp_cpt_source_idx < cpt_sources_index.get()
            } else {
                (*current).cp_cpt_source_idx > cpt_sources_index.get()
            };
            if !before {
                break;
            }
            compl_curr_match.set(if forward { current } else { (*current).cp_next });
            current = (*current).cp_next;
        }
    }
}
