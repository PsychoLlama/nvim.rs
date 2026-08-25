//! The popup menu: turning the match list into `pumitem_T`s and showing it.
//!
//! [`ins_compl_build_pum`] is the whole of it — it filters the list by the
//! current leader, scores and sorts what survives, and fills
//! `compl_match_array`.  [`ins_compl_show_pum`] then hands that to
//! `pum_display`, and [`trigger_complete_changed_event`] fires
//! `CompleteChanged` with the selected item.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::{Allow, Lock};
use crate::types::{IOSIZE, NUL, ShmFlag};

/// The popup menu's view of the match list.
///
/// [`ins_compl_build_pum`] filters the cyclic `compl_T` list by the current
/// leader, sorts what survives and flattens it into a `pumitem_T` array;
/// [`ins_compl_show_pum`] lends that array to `pum_display`, and
/// `pum_undisplay` gives the borrow back. Upstream keeps it as a bare
/// `pumitem_T *` with `compl_match_arraysize` beside it, `xcalloc`'d in one
/// function and `xfree`'d in three others, with "is the menu up?" spelled as
/// a null check on the pointer.
///
/// `ComplMatchArray` is the one owner of that allocation: a boxed slice
/// behind the two cells, handed out either as a safe `&[pumitem_T]` or -- for
/// `pum_display` alone, which wants the same C-shaped pair the external UI
/// protocol does -- as a raw address. The *strings* inside each item stay
/// borrowed from the `compl_T` they were read out of, so this owns the spine
/// and nothing else; the menu must come down before the match list is freed,
/// exactly as upstream required.
#[derive(Clone, Copy)]
pub(crate) struct ComplMatchArray(());

/// The popup menu's item array. See [`ComplMatchArray`].
pub(crate) fn compl_match_array() -> ComplMatchArray {
    ComplMatchArray(())
}

impl ComplMatchArray {
    /// Whether there is no array at all -- upstream's
    /// `compl_match_array == NULL`, which is how the completion asks whether
    /// a menu is up.
    pub(crate) fn is_unset(self) -> bool {
        COMPL_MATCH_ARRAY.get().is_null()
    }

    /// The number of items.
    pub(crate) fn len(self) -> c_int {
        COMPL_MATCH_ARRAYSIZE.get()
    }

    /// The items, empty while the menu is down.
    pub(crate) fn items(self) -> &'static [pumitem_T] {
        let array = COMPL_MATCH_ARRAY.get();
        if array.is_null() {
            return &[];
        }
        // SAFETY: `set` stored a boxed slice of exactly this length and only
        // `clear` drops it, after `pum_undisplay` has taken the menu down.
        unsafe { ::core::slice::from_raw_parts(array, COMPL_MATCH_ARRAYSIZE.get() as usize) }
    }

    /// The address `pum_display` borrows, null while the menu is down.
    pub(crate) fn as_mut_ptr(self) -> *mut pumitem_T {
        COMPL_MATCH_ARRAY.get()
    }

    /// Take `items` as the new array, dropping whatever was there. An empty
    /// `items` leaves the array unset: a zero-length boxed slice is a
    /// *dangling* pointer, not a null one, and `is_unset` is the null check
    /// upstream's callers do.
    pub(crate) fn set(self, items: Vec<pumitem_T>) {
        self.clear();
        if items.is_empty() {
            return;
        }
        let len = items.len() as c_int;
        let array = Box::into_raw(items.into_boxed_slice());
        COMPL_MATCH_ARRAY.set(array.cast::<pumitem_T>());
        COMPL_MATCH_ARRAYSIZE.set(len);
    }

    /// Drop the array. The menu must already be down.
    pub(crate) fn clear(self) {
        let array = COMPL_MATCH_ARRAY.get();
        let len = COMPL_MATCH_ARRAYSIZE.replace(0) as usize;
        COMPL_MATCH_ARRAY.set(ptr::null_mut());
        if array.is_null() {
            return;
        }
        // SAFETY: the allocation is this owner's own boxed slice, of exactly
        // `len` items; the strings inside it belong to the match list.
        drop(unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(array, len)) });
    }
}

/// The highlight attribute for the inserted-but-not-accepted text at
/// `lnum`/`col`, or −1 where there is none.
pub unsafe fn ins_compl_col_range_attr(lnum: linenr_T, col: c_int) -> c_int {
    unsafe {
        let has_preinsert = ins_compl_has_preinsert() || ins_compl_preinsert_longest();
        if cot_fuzzy() || (!compl_hi_on_autocompl_longest.get() && ins_compl_preinsert_longest()) {
            return -1;
        }
        let attr = syn_name2attr(if has_preinsert {
            c"PreInsert".as_ptr()
        } else {
            c"ComplMatchIns".as_ptr()
        });
        if attr == 0 {
            return -1;
        }

        let start_col = compl_col.get() + ins_compl_leader_len() as c_int;
        if !ins_compl_has_multiple() {
            return if col >= start_col && col < compl_ins_end_col.get() {
                attr
            } else {
                -1
            };
        }
        let cursor_lnum = (*curwin.get()).w_cursor.lnum;
        let inside = (lnum == compl_lnum.get() && col >= start_col && col < MAXCOL)
            || (lnum > compl_lnum.get() && lnum < cursor_lnum)
            || (lnum == cursor_lnum && col <= compl_ins_end_col.get());
        if inside { attr } else { -1 }
    }
}

/// Take the popup menu down and drop the item array it was built from.
pub(crate) unsafe fn ins_compl_del_pum() {
    unsafe {
        if compl_match_array().is_unset() {
            return;
        }
        pum_undisplay(false);
        compl_match_array().clear();
    }
}

/// Whether a popup menu is wanted at all: `'completeopt'` has `menu` or
/// `menuone`, or autocompletion is on.
pub unsafe fn pum_wanted() -> bool {
    unsafe {
        get_cot_flags() & (kOptCotFlagMenu | kOptCotFlagMenuone) != 0 || compl_autocomplete.get()
    }
}

/// Whether there are enough matches to show one: two, or one under
/// `menuone`/autocompletion.
pub(crate) unsafe fn pum_enough_matches() -> bool {
    unsafe {
        // Count the matches, but stop at two — that is all the answer needs.
        let mut comp = compl_first_match.get();
        let mut i = 0;
        while !comp.is_null() {
            if !match_at_original_text(comp) {
                i += 1;
                if i == 2 {
                    break;
                }
            }
            comp = (*comp).cp_next;
            if is_first_match(comp) {
                break;
            }
        }
        if get_cot_flags() & kOptCotFlagMenuone != 0 || compl_autocomplete.get() {
            return i >= 1;
        }
        i >= 2
    }
}

/// Fire `CompleteChanged` with `v:event.completed_item` set to match `cur`
/// (or to an empty dict when nothing is selected).
pub(crate) unsafe fn trigger_complete_changed_event(cur: c_int) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false);
        if recursive.get() {
            return;
        }

        let item = if cur < 0 {
            tv_dict_alloc()
        } else {
            ins_compl_dict_alloc(compl_curr_match.get())
        };
        let mut save_v_event = SAVE_V_EVENT_INIT;
        let v_event = get_v_event(&raw mut save_v_event);
        tv_dict_add_dict(v_event, c"completed_item".as_ptr(), 14, item);
        pum_set_event_info(v_event);
        tv_dict_set_keys_readonly(v_event);

        recursive.set(true);
        let locked = Lock::text();
        apply_autocmds(
            EVENT_COMPLETECHANGED,
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            curbuf.get(),
        );
        drop(locked);
        recursive.set(false);

        restore_v_event(v_event, &raw mut save_v_event);
    }
}

/// C's `"match %d of %d"` / `"match %d"`, formatted into the static buffer
/// `showmode` reads back through `edit_submode_extra`.
///
/// The buffer outlives the call by design — the message is shown on a later
/// redraw — so its address is taken once, here, and handed out as a
/// `char *` the way upstream's `match_ref[81]` was. `total` of zero or less
/// selects the short form.
///
/// # Safety
/// Must run on the main thread; the returned pointer stays valid until the
/// next call, which is the lifetime `edit_submode_extra` assumes.
unsafe fn match_position_message(number: c_int, total: c_int) -> *mut c_char {
    static match_ref: GlobalCell<[c_char; 81]> = GlobalCell::new([0; 81]);
    let msg = match_ref.ptr().cast::<c_char>();
    let size = size_of::<[c_char; 81]>();
    // SAFETY: `msg` addresses all 81 bytes, and both formats take `int`s.
    unsafe {
        if total > 0 {
            vim_snprintf(
                msg,
                size,
                gettext(c"match %d of %d".as_ptr()),
                number,
                total,
            );
        } else {
            vim_snprintf(msg, size, gettext(c"match %d".as_ptr()), number);
        }
    }
    msg
}

/// Build `dest` by prepending the buffer text from `startcol` to `compl_col`
/// to `src`.
pub(crate) unsafe fn prepend_startcol_text(dest: ComplStr, src: ComplStr, startcol: c_int) {
    unsafe {
        let prepend_len = compl_col.get() - startcol;
        let new_length = prepend_len + src.len() as c_int;

        dest.set_len(new_length as size_t);
        dest.set_data(xmalloc(new_length as size_t + 1) as *mut c_char); // +1 for NUL

        let line = ml_get((*curwin.get()).w_cursor.lnum);
        memmove(
            dest.data().cast::<c_void>(),
            line.offset(startcol as isize).cast::<c_void>(),
            prepend_len as size_t,
        );
        memmove(
            dest.data().offset(prepend_len as isize).cast::<c_void>(),
            src.data().cast::<c_void>(),
            src.len(),
        );
        *dest.data().offset(new_length as isize) = NUL as c_char;
    }
}

/// Drop the cached [`adjusted_leader`] — upstream's
/// `get_leader_for_startcol(NULL, …)`, which is a different job from the one
/// the rest of that function does and now says so.
pub(crate) fn clear_adjusted_leader() {
    adjusted_leader().clear();
}

/// The leader `match_0` should be filtered against, adjusted for the startcol
/// of the `'complete'` source it came from.
///
/// A source whose startcol is *before* `compl_col` matches text the leader
/// does not contain, so the leader has that text prepended; the result is
/// cached in `adjusted_leader`, which [`clear_adjusted_leader`] drops.
pub(crate) unsafe fn get_leader_for_startcol(match_0: *mut compl_T, cached: bool) -> ComplStr {
    unsafe {
        debug_assert!(!match_0.is_null());

        'theend: {
            if cpt_sources().is_unset() {
                break 'theend;
            }
            let cpt_idx = (*match_0).cp_cpt_source_idx;
            if cpt_idx < 0 {
                break 'theend;
            }
            let startcol = cpt_sources().row(cpt_idx).cs_startcol;

            if compl_leader().is_unset() {
                // The leader is not set yet (`'autocomplete'` fires before
                // `compl_leader` is initialised). Matches starting at or after
                // `compl_col` fall back to the original text; matches starting
                // *before* it carry pre-`compl_col` text that must not be
                // compared with the original text, so the empty
                // `compl_leader` is returned to mean "no prefix filter".
                if startcol < 0 || startcol >= compl_col.get() {
                    return compl_orig_text();
                }
                return compl_leader();
            }

            if compl_col.get() <= 0 {
                break 'theend;
            }
            if startcol >= 0 && startcol < compl_col.get() {
                let prepend_len = compl_col.get() - startcol;
                let new_length = prepend_len + compl_leader().len() as c_int;
                if cached
                    && new_length as size_t == adjusted_leader().len()
                    && !adjusted_leader().is_unset()
                {
                    return adjusted_leader();
                }
                adjusted_leader().clear();
                prepend_startcol_text(adjusted_leader(), compl_leader(), startcol);
                return adjusted_leader();
            }
        }
        compl_leader()
    }
}

/// Build `compl_match_array` from the match list.
///
/// Returns the entry that should be selected, or −1 for none.
pub(crate) unsafe fn ins_compl_build_pum() -> c_int {
    unsafe {
        // Under a user completion function with `refresh: 'always'` the leader
        // is not a prefix filter, so drop it.
        //
        // Upstream writes `XFREE_CLEAR(compl_leader)` — the *struct*, not
        // `.data` — which frees and NULLs the first member and leaves `.size`
        // stale. Reproduced deliberately; every reader guards on `.data`.
        if ins_compl_need_restart() {
            compl_leader().free_bytes_keep_len();
        }

        let compl_no_select = get_cot_flags() & kOptCotFlagNoselect != 0
            || (compl_autocomplete.get() && !ins_compl_has_preinsert());

        let mut match_head: *mut compl_T = ptr::null_mut();
        let mut match_tail: *mut compl_T = ptr::null_mut();
        let is_forward = compl_shows_dir_forward();
        let is_cpt_completion = !cpt_sources().is_unset();

        // If the current match is the original text, don't find the first
        // match after it and don't highlight anything.
        let mut shown_match_ok = match_at_original_text(compl_shown_match.get());

        if strequal(compl_leader().data(), compl_orig_text().data()) && !shown_match_ok {
            compl_shown_match.set(if compl_no_select {
                compl_first_match.get()
            } else {
                (*compl_first_match.get()).cp_next
            });
        }

        let mut match_arraysize = 0;
        let mut did_find_shown_match = false;
        let mut shown_compl: *mut compl_T = ptr::null_mut();
        let mut i = 0;
        let mut cur = -1;

        let match_count: *mut c_int = if is_cpt_completion {
            xcalloc(cpt_sources().rows().len() as size_t, size_of::<c_int>()) as *mut c_int
        } else {
            ptr::null_mut()
        };

        clear_adjusted_leader();

        let mut comp = compl_first_match.get();
        loop {
            (*comp).cp_in_match_array = false;

            let leader = get_leader_for_startcol(comp, true);

            // Apply 'smartcase' behaviour during normal mode.
            if ctrl_x_mode_normal()
                && p_inf.get() == 0
                && !leader.data().is_null()
                && ignorecase(leader.data()) == 0
                && !cot_fuzzy()
            {
                (*comp).cp_flags &= !CP_ICASE;
            }

            if !match_at_original_text(comp)
                && (leader.data().is_null()
                    || ins_compl_equal(comp, leader.data(), leader.len())
                    || (cot_fuzzy() && (*comp).cp_score != FUZZY_SCORE_NONE))
            {
                // Limit the number of items from each source where
                // `cs_max_matches` is set.
                let mut match_limit_exceeded = false;
                let cur_source = (*comp).cp_cpt_source_idx;
                if is_forward && cur_source != -1 && is_cpt_completion {
                    *match_count.offset(cur_source as isize) += 1;
                    let max_matches = cpt_sources().row(cur_source).cs_max_matches;
                    if max_matches > 0 && *match_count.offset(cur_source as isize) > max_matches {
                        match_limit_exceeded = true;
                    }
                }

                if !match_limit_exceeded {
                    match_arraysize += 1;
                    (*comp).cp_in_match_array = true;
                    if match_head.is_null() {
                        match_head = comp;
                    } else {
                        (*match_tail).cp_match_next = comp;
                    }
                    match_tail = comp;

                    if !shown_match_ok && !cot_fuzzy() {
                        if comp == compl_shown_match.get() || did_find_shown_match {
                            // This item is the shown match, or the first
                            // displayed item after it.
                            compl_shown_match.set(comp);
                            did_find_shown_match = true;
                            shown_match_ok = true;
                        } else {
                            // Remember this displayed match, for when the
                            // shown match turns out to be just below it.
                            shown_compl = comp;
                        }
                        cur = i;
                    } else if cot_fuzzy() {
                        if i == 0 {
                            shown_compl = comp;
                        }
                        if !shown_match_ok && comp == compl_shown_match.get() {
                            cur = i;
                            shown_match_ok = true;
                        }
                    }
                    i += 1;
                }
            }

            if comp == compl_shown_match.get() && !cot_fuzzy() {
                did_find_shown_match = true;
                // When the original text is the shown match, don't set
                // `compl_shown_match`.
                if match_at_original_text(comp) {
                    shown_match_ok = true;
                }
                if !shown_match_ok && !shown_compl.is_null() {
                    // The shown match isn't displayed; use the previously
                    // displayed match instead.
                    compl_shown_match.set(shown_compl);
                    shown_match_ok = true;
                }
            }
            comp = (*comp).cp_next;
            if comp.is_null() || is_first_match(comp) {
                break;
            }
        }

        xfree(match_count.cast::<c_void>());

        if match_arraysize == 0 {
            compl_match_array().clear();
            return -1;
        }

        if cot_fuzzy() && !compl_no_select && !shown_match_ok {
            compl_shown_match.set(shown_compl);
            shown_match_ok = true;
            cur = 0;
        }

        let mut array = Vec::with_capacity(match_arraysize as usize);
        let mut comp = match_head;
        while !comp.is_null() {
            array.push(pumitem_T {
                pum_text: if (*comp).cp_text[CPT_ABBR as usize].is_null() {
                    (*comp).cp_str.data()
                } else {
                    (*comp).cp_text[CPT_ABBR as usize]
                },
                pum_kind: (*comp).cp_text[CPT_KIND as usize],
                pum_info: (*comp).cp_text[CPT_INFO as usize],
                pum_cpt_source_idx: (*comp).cp_cpt_source_idx,
                pum_user_abbr_hlattr: (*comp).cp_user_abbr_hlattr,
                pum_user_kind_hlattr: (*comp).cp_user_kind_hlattr,
                pum_extra: if (*comp).cp_text[CPT_MENU as usize].is_null() {
                    (*comp).cp_fname
                } else {
                    (*comp).cp_text[CPT_MENU as usize]
                },
            });

            let match_next = (*comp).cp_match_next;
            (*comp).cp_match_next = ptr::null_mut();
            comp = match_next;
        }
        debug_assert_eq!(array.len(), match_arraysize as usize);
        compl_match_array().set(array);

        if !shown_match_ok {
            // No displayed match at all.
            cur = -1;
        }
        cur
    }
}

/// Show the popup menu, adjusting `compl_shown_match` to an entry that is
/// actually displayed.
pub unsafe fn ins_compl_show_pum() {
    unsafe {
        if !pum_wanted() || !pum_enough_matches() {
            return;
        }

        // Update the screen before drawing the popup menu over it.
        update_screen();

        let mut cur = -1;
        let mut array_changed = false;

        if compl_match_array().is_unset() {
            array_changed = true;
            cur = ins_compl_build_pum();
        } else {
            // The menu already exists; only the current item has to be found.
            let shown = compl_shown_match.get();
            for (i, item) in compl_match_array().items().iter().enumerate() {
                let text = item.pum_text;
                if text == (*shown).cp_str.data() || text == (*shown).cp_text[CPT_ABBR as usize] {
                    cur = i as c_int;
                    break;
                }
            }
        }

        if compl_match_array().is_unset() {
            if compl_started.get() && has_event(EVENT_COMPLETECHANGED) {
                trigger_complete_changed_event(cur);
            }
            return;
        }

        // Avoid a wait_return() when the message is cleared.
        dollar_vcol.set(-1);

        // Move the cursor to the start of the match for the popup menu's sake,
        // then put it back.
        let col = (*curwin.get()).w_cursor.col;
        (*curwin.get()).w_cursor.col = compl_col.get();
        compl_selected_item.set(cur);
        pum_display(
            compl_match_array().as_mut_ptr(),
            compl_match_array().len(),
            cur,
            array_changed,
            0,
        );
        (*curwin.get()).w_cursor.col = col;

        if compl_started.get() && compl_curr_match.get() != compl_shown_match.get() {
            compl_curr_match.set(compl_shown_match.get());
        }

        if has_event(EVENT_COMPLETECHANGED) {
            trigger_complete_changed_event(cur);
        }
    }
}

/// Is `selected` (a menu index) the match `compl_curr_match` points at?
pub unsafe fn compl_match_curr_select(selected: c_int) -> bool {
    unsafe {
        if selected < 0 {
            return false;
        }
        let mut match_0 = compl_first_match.get();
        let mut selected_idx = -1;
        let mut list_idx = 0;
        loop {
            if !match_at_original_text(match_0) {
                let curr = compl_curr_match.get();
                if !curr.is_null() && (*curr).cp_number == (*match_0).cp_number {
                    selected_idx = list_idx;
                    break;
                }
                list_idx += 1;
            }
            match_0 = (*match_0).cp_next;
            if match_0.is_null() || is_first_match(match_0) {
                break;
            }
        }
        selected == selected_idx
    }
}

/// Report which file the shown match came from, truncating the name on the
/// left to whatever room the status line leaves.
pub(crate) unsafe fn ins_compl_show_filename() {
    let mut line = [0 as c_char; IOSIZE as usize];
    unsafe {
        let lead = gettext(c"match in file".as_ptr());
        let mut space = sc_col.get() - vim_strsize(lead) - 2;
        if space <= 0 {
            return;
        }

        // Find `s` such that `s..e` fits in `space` cells.
        let fname = (*compl_shown_match.get()).cp_fname;
        let mut s = fname;
        let mut e = fname;
        while *e as c_int != NUL {
            space -= ptr2cells(e);
            while space < 0 {
                space += ptr2cells(s);
                s = s.offset(utfc_ptr2len(s) as isize);
            }
            e = e.offset(utfc_ptr2len(e) as isize);
        }

        if compl_autocomplete.get() {
            return;
        }
        msg_hist_off.set(true);
        vim_snprintf(
            line.as_mut_ptr(),
            IOSIZE as size_t,
            c"%s %s%s".as_ptr(),
            lead,
            if s > fname {
                c"<".as_ptr()
            } else {
                c"".as_ptr()
            },
            s,
        );
        msg(line.as_mut_ptr(), 0);
        msg_hist_off.set(false);
        redraw_cmdline.set(false);
    }
}

/// The next match that is actually in the menu, in the direction the menu is
/// being walked.
pub(crate) unsafe fn find_next_match_in_menu() -> *mut compl_T {
    unsafe {
        let is_forward = compl_shows_dir_forward();
        let mut match_0 = compl_shown_match.get();
        loop {
            match_0 = if is_forward {
                (*match_0).cp_next
            } else {
                (*match_0).cp_prev
            };
            if (*match_0).cp_next.is_null()
                || (*match_0).cp_in_match_array
                || match_at_original_text(match_0)
            {
                break;
            }
        }
        match_0
    }
}

/// The "match 3 of 17" / "Back at original" line under the menu.
pub(crate) unsafe fn ins_compl_show_statusmsg() {
    unsafe {
        // Show a message about what (completion) mode we're in.
        if is_first_match((*compl_first_match.get()).cp_next) {
            edit_submode_extra.set(if compl_status_adding() && compl_length.get() > 1 {
                gettext(E_HITEND.as_ptr())
            } else {
                gettext((&raw const e_patnotf).cast::<c_char>())
            });
            edit_submode_highl.set(HLF_E);
        }

        if edit_submode_extra.get().is_null() {
            let curr = compl_curr_match.get();
            if match_at_original_text(curr) {
                edit_submode_extra.set(gettext(c"Back at original".as_ptr()));
                edit_submode_highl.set(HLF_W);
            } else if compl_cont_status.get() & CONT_S_IPOS != 0 {
                edit_submode_extra.set(gettext(c"Word from other line".as_ptr()));
                edit_submode_highl.set(HLF_COUNT);
            } else if (*curr).cp_next == (*curr).cp_prev {
                edit_submode_extra.set(gettext(c"The only match".as_ptr()));
                edit_submode_highl.set(HLF_COUNT);
                (*curr).cp_number = 1;
            } else {
                // Update `cp_number`, it is used in `msg_ext_set_kind`.
                if (*curr).cp_number == -1 {
                    ins_compl_update_sequence_numbers();
                }
                if (*curr).cp_number != -1 {
                    let msg = match_position_message((*curr).cp_number, compl_matches.get());
                    edit_submode_extra.set(msg);
                    edit_submode_highl.set(HLF_R);
                    if dollar_vcol.get() >= 0 {
                        curs_columns(curwin.get(), 0);
                    }
                }
            }
        }

        redraw_mode.set(true);
        if shortmess(ShmFlag::COMPLETIONMENU) {
            return;
        }
        if edit_submode_extra.get().is_null() {
            msg_clr_cmdline();
        } else if p_smd.get() == 0 {
            msg_hist_off.set(true);
            msg_ext_set_kind(c"completion".as_ptr());
            let attr = if (edit_submode_highl.get() as c_uint) < HLF_COUNT as c_uint {
                edit_submode_highl.get() as c_int + 1
            } else {
                0
            };
            msg(edit_submode_extra.get(), attr);
            msg_hist_off.set(false);
        }
    }
}

/// Redraw the popup menu after the cursor may have moved, with redrawing
/// forced back on.
pub(crate) unsafe fn show_pum(prev_w_wrow: c_int, prev_w_leftcol: c_int) {
    unsafe {
        // RedrawingDisabled may be set when invoked through complete().
        let _redraw = Allow::redraw();

        // If the cursor moved or the display scrolled, the menu has to be
        // rebuilt rather than only redrawn.
        setcursor();
        if prev_w_wrow != (*curwin.get()).w_wrow || prev_w_leftcol != (*curwin.get()).w_leftcol {
            ins_compl_del_pum();
        }
        ins_compl_show_pum();
        setcursor();
    }
}
