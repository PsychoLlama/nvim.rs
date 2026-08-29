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
use crate::message::msg_ptr;
use crate::os::cshim::gettext_ptr;
use crate::types::{IOSIZE, NUL, ShmFlag};
use crate::winlayer::Win;

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
    // SAFETY: neither query has a precondition left; both are still
    // `unsafe fn`s for their call sites outside this family.
    let (preinsert, longest) =
        unsafe { (ins_compl_has_preinsert(), ins_compl_preinsert_longest()) };
    if cot_fuzzy() || (!compl_hi_on_autocompl_longest.get() && longest) {
        return -1;
    }
    // Either kind of preview takes `PreInsert`; only an accepted match that
    // has not been confirmed yet takes `ComplMatchIns`.
    let name = if preinsert || longest {
        c"PreInsert".as_ptr()
    } else {
        c"ComplMatchIns".as_ptr()
    };
    // SAFETY: a static NUL-terminated highlight group name.
    let attr = unsafe { syn_name2attr(name) };
    if attr == 0 {
        return -1;
    }

    let start_col = compl_col.get() + ins_compl_leader_len() as c_int;
    // SAFETY: the caller's promise -- a completion with a shown match.
    if !unsafe { ins_compl_has_multiple() } {
        return if col >= start_col && col < compl_ins_end_col.get() {
            attr
        } else {
            -1
        };
    }
    let cursor_lnum = cur_win().w_cursor.lnum;
    let inside = (lnum == compl_lnum.get() && col >= start_col && col < MAXCOL)
        || (lnum > compl_lnum.get() && lnum < cursor_lnum)
        || (lnum == cursor_lnum && col <= compl_ins_end_col.get());
    if inside { attr } else { -1 }
}

/// Take the popup menu down and drop the item array it was built from.
pub(crate) unsafe fn ins_compl_del_pum() {
    if compl_match_array().is_unset() {
        return;
    }
    // SAFETY: a menu is up, which is what `is_unset` just asked.
    unsafe { pum_undisplay(false) };
    compl_match_array().clear();
}

/// Whether a popup menu is wanted at all: `'completeopt'` has `menu` or
/// `menuone`, or autocompletion is on.
pub unsafe fn pum_wanted() -> bool {
    completeopt_flags() & (kOptCotFlagMenu | kOptCotFlagMenuone) != 0 || compl_autocomplete.get()
}

/// Whether there are enough matches to show one: two, or one under
/// `menuone`/autocompletion.
pub(crate) fn pum_enough_matches() -> bool {
    // Count the matches, but stop at two — that is all the answer needs.
    let mut i = 0;
    for comp in matches_from(first_match()) {
        if !comp.is_original() {
            i += 1;
            if i == 2 {
                break;
            }
        }
    }
    if completeopt_flags() & kOptCotFlagMenuone != 0 || compl_autocomplete.get() {
        return i >= 1;
    }
    i >= 2
}

/// Fire `CompleteChanged` with `v:event.completed_item` set to match `cur`
/// (or to an empty dict when nothing is selected).
pub(crate) unsafe fn trigger_complete_changed_event(cur: c_int) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false);
    if recursive.get() {
        return;
    }

    // SAFETY: a running completion's current match, and the dicts are fresh
    // allocations `v:event` takes over.
    let item = unsafe {
        if cur < 0 {
            tv_dict_alloc()
        } else {
            ins_compl_dict_alloc(compl_curr_match.get())
        }
    };
    let mut save_v_event = SAVE_V_EVENT_INIT;
    // SAFETY: `save_v_event` is this frame's, and lives until the restore
    // below hands the saved dict back.
    let v_event = unsafe { get_v_event(&raw mut save_v_event) };
    let buf = curbuf.get();
    // SAFETY: `v_event` is the dict just built, the key is a static string
    // of the length given, and `buf` is the current buffer.
    unsafe {
        tv_dict_add_dict(v_event, c"completed_item".as_ptr(), 14, item);
        pum_set_event_info(v_event);
        tv_dict_set_keys_readonly(v_event);
    }

    recursive.set(true);
    let locked = Lock::text();
    // SAFETY: as above -- `CompleteChanged` takes no file name pattern.
    unsafe {
        apply_autocmds(
            EVENT_COMPLETECHANGED,
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            buf,
        );
    }
    drop(locked);
    recursive.set(false);

    // SAFETY: the pair of `get_v_event`, with the same saved slot.
    unsafe { restore_v_event(v_event, &raw mut save_v_event) };
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
            let fmt = gettext(c"match %d of %d");
            vim_snprintf(msg, size, fmt.as_ptr(), number, total);
        } else {
            let fmt = gettext(c"match %d");
            vim_snprintf(msg, size, fmt.as_ptr(), number);
        }
    }
    msg
}

/// Build `dest` by prepending the buffer text from `startcol` to `compl_col`
/// to `src`.
pub(crate) unsafe fn prepend_startcol_text(dest: ComplStr, src: ComplStr, startcol: c_int) {
    let prepend_len = compl_col.get() - startcol;
    let new_length = prepend_len + src.len() as c_int;

    dest.set_len(new_length as size_t);
    // SAFETY: `xmalloc` answers `new_length + 1` writable bytes or aborts.
    let buf = unsafe { xmalloc(new_length as size_t + 1) } as *mut c_char; // +1 for NUL
    dest.set_data(buf);

    // SAFETY: the cursor line exists, `startcol .. compl_col` is inside it,
    // and `buf` has room for the two pieces and the NUL.
    unsafe {
        let line = ml_get(cur_win().w_cursor.lnum);
        let head = line.offset(startcol as isize);
        memmove(
            buf.cast::<c_void>(),
            head.cast::<c_void>(),
            prepend_len as size_t,
        );
        let tail = buf.offset(prepend_len as isize);
        memmove(
            tail.cast::<c_void>(),
            src.data().cast::<c_void>(),
            src.len(),
        );
        *buf.offset(new_length as isize) = NUL as c_char;
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
pub(crate) unsafe fn get_leader_for_startcol(match_0: Cm, cached: bool) -> ComplStr {
    'theend: {
        if cpt_sources().is_unset() {
            break 'theend;
        }
        let cpt_idx = match_0.cp_cpt_source_idx;
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
            // SAFETY: a completion is running, so the cursor line holds the
            // text before `compl_col` that the leader is extended with.
            unsafe { prepend_startcol_text(adjusted_leader(), compl_leader(), startcol) };
            return adjusted_leader();
        }
    }
    compl_leader()
}

/// Build `compl_match_array` from the match list.
///
/// Returns the entry that should be selected, or −1 for none.
pub(crate) unsafe fn ins_compl_build_pum() -> c_int {
    // Under a user completion function with `refresh: 'always'` the leader
    // is not a prefix filter, so drop it.
    //
    // Upstream writes `XFREE_CLEAR(compl_leader)` — the *struct*, not
    // `.data` — which frees and NULLs the first member and leaves `.size`
    // stale. Reproduced deliberately; every reader guards on `.data`.
    if ins_compl_need_restart() {
        compl_leader().free_bytes_keep_len();
    }

    // SAFETY: no precondition left; still an `unsafe fn` for the call sites
    // outside this family.
    let has_preinsert = unsafe { ins_compl_has_preinsert() };
    let compl_no_select = completeopt_flags() & kOptCotFlagNoselect != 0
        || (compl_autocomplete.get() && !has_preinsert);

    let mut match_head: Option<Cm> = None;
    let mut match_tail: Option<Cm> = None;
    let is_forward = compl_shows_dir_forward();
    let is_cpt_completion = !cpt_sources().is_unset();

    // If the current match is the original text, don't find the first
    // match after it and don't highlight anything.
    let mut shown_match_ok = shown_match().is_some_and(Cm::is_original);

    // SAFETY: both strings are NUL-terminated or null, which `strequal`
    // takes.
    let leader_is_orig = unsafe { strequal(compl_leader().data(), compl_orig_text().data()) };
    if leader_is_orig && !shown_match_ok {
        // Upstream dereferences `compl_first_match` here without checking.
        let head = first_match().expect("a menu is built from a non-empty list");
        compl_shown_match.set(if compl_no_select {
            head.raw()
        } else {
            head.cp_next
        });
    }

    let mut match_arraysize = 0;
    let mut did_find_shown_match = false;
    let mut shown_compl: Option<Cm> = None;
    let mut i = 0;
    let mut cur = -1;

    let match_count: *mut c_int = if is_cpt_completion {
        let rows = cpt_sources().rows().len() as size_t;
        // SAFETY: `xcalloc` answers a zeroed `c_int` per source, or aborts.
        unsafe { xcalloc(rows, size_of::<c_int>()) as *mut c_int }
    } else {
        ptr::null_mut()
    };

    clear_adjusted_leader();

    for mut comp in matches_from(first_match()) {
        comp.cp_in_match_array = false;

        // SAFETY: `comp` is a live node of the list being drawn.
        let leader = unsafe { get_leader_for_startcol(comp, true) };

        // Apply 'smartcase' behaviour during normal mode.
        if ctrl_x_mode_normal()
            && p_inf.get() == 0
            && !leader.data().is_null()
            // SAFETY: the leader is a NUL-terminated string.
            && unsafe { ignorecase(leader.data()) } == 0
            && !cot_fuzzy()
        {
            comp.cp_flags &= !CP_ICASE;
        }

        // SAFETY: the leader is readable for `leader.len()` bytes.
        let displayed = !comp.is_original()
            && (leader.data().is_null()
                || unsafe { ins_compl_equal(comp, leader.data(), leader.len()) }
                || (cot_fuzzy() && comp.cp_score != FUZZY_SCORE_NONE));
        if displayed {
            // Limit the number of items from each source where
            // `cs_max_matches` is set.
            let mut match_limit_exceeded = false;
            let cur_source = comp.cp_cpt_source_idx;
            if is_forward && cur_source != -1 && is_cpt_completion {
                // SAFETY: `match_count` holds one `c_int` per `'complete'`
                // source, and `cur_source` names one of them.
                let slot = unsafe { match_count.offset(cur_source as isize) };
                // SAFETY: as above.
                let count = unsafe { *slot } + 1;
                // SAFETY: as above.
                unsafe { *slot = count };
                let max_matches = cpt_sources().row(cur_source).cs_max_matches;
                if max_matches > 0 && count > max_matches {
                    match_limit_exceeded = true;
                }
            }

            if !match_limit_exceeded {
                match_arraysize += 1;
                comp.cp_in_match_array = true;
                if match_head.is_none() {
                    match_head = Some(comp);
                } else if let Some(mut tail) = match_tail {
                    tail.cp_match_next = comp.raw();
                }
                match_tail = Some(comp);

                if !shown_match_ok && !cot_fuzzy() {
                    if shown_match() == Some(comp) || did_find_shown_match {
                        // This item is the shown match, or the first
                        // displayed item after it.
                        compl_shown_match.set(comp.raw());
                        did_find_shown_match = true;
                        shown_match_ok = true;
                    } else {
                        // Remember this displayed match, for when the
                        // shown match turns out to be just below it.
                        shown_compl = Some(comp);
                    }
                    cur = i;
                } else if cot_fuzzy() {
                    if i == 0 {
                        shown_compl = Some(comp);
                    }
                    if !shown_match_ok && shown_match() == Some(comp) {
                        cur = i;
                        shown_match_ok = true;
                    }
                }
                i += 1;
            }
        }

        if shown_match() == Some(comp) && !cot_fuzzy() {
            did_find_shown_match = true;
            // When the original text is the shown match, don't set
            // `compl_shown_match`.
            if comp.is_original() {
                shown_match_ok = true;
            }
            if !shown_match_ok && let Some(prev) = shown_compl {
                // The shown match isn't displayed; use the previously
                // displayed match instead.
                compl_shown_match.set(prev.raw());
                shown_match_ok = true;
            }
        }
    }

    // SAFETY: this function's own allocation, and `xfree` takes null.
    unsafe { xfree(match_count.cast::<c_void>()) };

    if match_arraysize == 0 {
        compl_match_array().clear();
        return -1;
    }

    if cot_fuzzy() && !compl_no_select && !shown_match_ok {
        compl_shown_match.set(shown_compl.map_or(ptr::null_mut(), Cm::raw));
        shown_match_ok = true;
        cur = 0;
    }

    let mut array = Vec::with_capacity(match_arraysize as usize);
    let mut comp = match_head;
    while let Some(mut node) = comp {
        array.push(pumitem_T {
            pum_text: if node.cp_text[CPT_ABBR as usize].is_null() {
                node.cp_str.data()
            } else {
                node.cp_text[CPT_ABBR as usize]
            },
            pum_kind: node.cp_text[CPT_KIND as usize],
            pum_info: node.cp_text[CPT_INFO as usize],
            pum_cpt_source_idx: node.cp_cpt_source_idx,
            pum_user_abbr_hlattr: node.cp_user_abbr_hlattr,
            pum_user_kind_hlattr: node.cp_user_kind_hlattr,
            pum_extra: if node.cp_text[CPT_MENU as usize].is_null() {
                node.cp_fname
            } else {
                node.cp_text[CPT_MENU as usize]
            },
        });

        let match_next = node.match_next();
        node.cp_match_next = ptr::null_mut();
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

/// Show the popup menu, adjusting `compl_shown_match` to an entry that is
/// actually displayed.
pub unsafe fn ins_compl_show_pum() {
    // SAFETY: no precondition left; still an `unsafe fn` for the call sites
    // outside this family.
    if !unsafe { pum_wanted() } || !pum_enough_matches() {
        return;
    }

    // Update the screen before drawing the popup menu over it.
    // SAFETY: the editor exists and this runs on its own thread.
    unsafe { update_screen() };

    let mut cur = -1;
    let mut array_changed = false;

    if compl_match_array().is_unset() {
        array_changed = true;
        // SAFETY: a completion is running -- the caller's promise.
        cur = unsafe { ins_compl_build_pum() };
    } else if let Some(shown) = shown_match() {
        // The menu already exists; only the current item has to be found.
        for (i, item) in compl_match_array().items().iter().enumerate() {
            let text = item.pum_text;
            if text == shown.cp_str.data() || text == shown.cp_text[CPT_ABBR as usize] {
                cur = i as c_int;
                break;
            }
        }
    }

    if compl_match_array().is_unset() {
        if compl_started.get() && has_event(EVENT_COMPLETECHANGED) {
            // SAFETY: as above.
            unsafe { trigger_complete_changed_event(cur) };
        }
        return;
    }

    // Avoid a wait_return() when the message is cleared.
    dollar_vcol.set(-1);

    // Move the cursor to the start of the match for the popup menu's sake,
    // then put it back.
    let col = cur_win().w_cursor.col;
    cur_win().w_cursor.col = compl_col.get();
    compl_selected_item.set(cur);
    let items = compl_match_array().as_mut_ptr();
    let len = compl_match_array().len();
    // SAFETY: the array is `len` items long and stays put until
    // `pum_undisplay` gives the borrow back.
    unsafe { pum_display(items, len, cur, array_changed, 0) };
    cur_win().w_cursor.col = col;

    if compl_started.get() && compl_curr_match.get() != compl_shown_match.get() {
        compl_curr_match.set(compl_shown_match.get());
    }

    if has_event(EVENT_COMPLETECHANGED) {
        // SAFETY: as above.
        unsafe { trigger_complete_changed_event(cur) };
    }
}

/// Is `selected` (a menu index) the match `compl_curr_match` points at?
pub unsafe fn compl_match_curr_select(selected: c_int) -> bool {
    if selected < 0 {
        return false;
    }
    let mut selected_idx = -1;
    let mut list_idx = 0;
    for match_0 in matches_from(first_match()) {
        if match_0.is_original() {
            continue;
        }
        if curr_match().is_some_and(|curr| curr.cp_number == match_0.cp_number) {
            selected_idx = list_idx;
            break;
        }
        list_idx += 1;
    }
    selected == selected_idx
}

/// Report which file the shown match came from, truncating the name on the
/// left to whatever room the status line leaves.
pub(crate) unsafe fn ins_compl_show_filename() {
    let mut line = [0 as c_char; IOSIZE as usize];
    let lead = gettext(c"match in file");
    // SAFETY: as above.
    let mut space = sc_col.get() - unsafe { vim_strsize(lead.as_ptr()) } - 2;
    if space <= 0 {
        return;
    }

    // Find `s` such that `s..e` fits in `space` cells.
    // SAFETY: the caller's promise -- a shown match, whose `cp_fname` is a
    // NUL-terminated file name.
    let fname = unsafe { (*compl_shown_match.get()).cp_fname };
    let mut s = fname;
    let mut e = fname;
    loop {
        // SAFETY: `e` walks the NUL-terminated file name and stops at its
        // NUL.
        if unsafe { *e } as c_int == NUL {
            break;
        }
        // SAFETY: `e` points at a character of the name.
        space -= unsafe { ptr2cells(e) };
        while space < 0 {
            // SAFETY: `s` trails `e` inside the same name.
            space += unsafe { ptr2cells(s) };
            // SAFETY: as above -- a character is `utfc_ptr2len` bytes.
            s = unsafe { s.offset(utfc_ptr2len(s) as isize) };
        }
        // SAFETY: as above.
        e = unsafe { e.offset(utfc_ptr2len(e) as isize) };
    }

    if compl_autocomplete.get() {
        return;
    }
    msg_hist_off.set(true);
    let ellipsis = if s > fname {
        c"<".as_ptr()
    } else {
        c"".as_ptr()
    };
    let buf = line.as_mut_ptr();
    // SAFETY: `buf` addresses all `IOSIZE` bytes and the three `%s` are
    // NUL-terminated strings.
    unsafe {
        vim_snprintf(
            buf,
            IOSIZE as size_t,
            c"%s %s%s".as_ptr(),
            lead,
            ellipsis,
            s,
        );
        msg_ptr(buf, 0);
    }
    msg_hist_off.set(false);
    redraw_cmdline.set(false);
}

/// The next match that is actually in the menu, in the direction the menu is
/// being walked.
/// # Safety
/// A completion with a shown match is running: upstream walks the ring from
/// `compl_shown_match` and dereferences each link without checking it.
pub(crate) unsafe fn find_next_match_in_menu() -> Cm {
    let is_forward = compl_shows_dir_forward();
    let mut match_0 = shown_match().expect("the menu is walked from a shown match");
    loop {
        let step = if is_forward {
            match_0.next()
        } else {
            match_0.prev()
        };
        match_0 = step.expect("the ring closes, so neither link is null");
        if match_0.cp_next.is_null() || match_0.cp_in_match_array || match_0.is_original() {
            break;
        }
    }
    match_0
}

/// The "match 3 of 17" / "Back at original" line under the menu.
pub(crate) unsafe fn ins_compl_show_statusmsg() {
    // Show a message about what (completion) mode we're in.
    // Upstream dereferences `compl_first_match` here without checking.
    let head = first_match().expect("a completion showing a message has matches");
    if is_first_match(head.cp_next) {
        let text = if compl_status_adding() && compl_length.get() > 1 {
            E_HITEND.as_ptr()
        } else {
            e_patnotf.as_ptr()
        };
        // SAFETY: both are static NUL-terminated messages.
        edit_submode_extra.set(unsafe { gettext_ptr(text).as_ptr().cast_mut() });
        edit_submode_highl.set(HLF_E);
    }

    if edit_submode_extra.get().is_null() {
        let mut curr = curr_match().expect("a running completion has a current match");
        if curr.is_original() {
            edit_submode_extra.set(gettext(c"Back at original").as_ptr().cast_mut());
            edit_submode_highl.set(HLF_W);
        } else if compl_cont_status.get() & CONT_S_IPOS != 0 {
            edit_submode_extra.set(gettext(c"Word from other line").as_ptr().cast_mut());
            edit_submode_highl.set(HLF_COUNT);
        } else if curr.cp_next == curr.cp_prev {
            edit_submode_extra.set(gettext(c"The only match").as_ptr().cast_mut());
            edit_submode_highl.set(HLF_COUNT);
            curr.cp_number = 1;
        } else {
            // Update `cp_number`, it is used in `msg_ext_set_kind`.
            if curr.cp_number == -1 {
                ins_compl_update_sequence_numbers();
            }
            if curr.cp_number != -1 {
                // SAFETY: this thread owns the static message buffer, and
                // the pointer stays valid until the next call.
                let msg = unsafe { match_position_message(curr.cp_number, compl_matches.get()) };
                edit_submode_extra.set(msg);
                edit_submode_highl.set(HLF_R);
                if dollar_vcol.get() >= 0 {
                    // SAFETY: the current window is live.
                    unsafe { curs_columns(Win::current(), 0) };
                }
            }
        }
    }

    redraw_mode.set(true);
    if shortmess(ShmFlag::COMPLETIONMENU) {
        return;
    }
    if edit_submode_extra.get().is_null() {
        // SAFETY: the editor exists and this runs on its own thread.
        unsafe { msg_clr_cmdline() };
    } else if p_smd.get() == 0 {
        msg_hist_off.set(true);
        let attr = if (edit_submode_highl.get() as c_uint) < HLF_COUNT as c_uint {
            edit_submode_highl.get() as c_int + 1
        } else {
            0
        };
        let extra = edit_submode_extra.get();
        // SAFETY: a static kind name, and `extra` is a NUL-terminated
        // message set above or by the caller.
        unsafe {
            msg_ext_set_kind(c"completion".as_ptr());
            msg_ptr(extra, attr);
        }
        msg_hist_off.set(false);
    }
}

/// Redraw the popup menu after the cursor may have moved, with redrawing
/// forced back on.
pub(crate) unsafe fn show_pum(prev_w_wrow: c_int, prev_w_leftcol: c_int) {
    // RedrawingDisabled may be set when invoked through complete().
    let _redraw = Allow::redraw();

    // If the cursor moved or the display scrolled, the menu has to be
    // rebuilt rather than only redrawn.
    // SAFETY: the editor exists, a completion is running -- the caller's
    // promise -- and this runs on its own thread.
    unsafe { setcursor() };
    if prev_w_wrow != cur_win().w_wrow || prev_w_leftcol != cur_win().w_leftcol {
        // SAFETY: as above.
        unsafe { ins_compl_del_pum() };
    }
    // SAFETY: as above.
    unsafe {
        ins_compl_show_pum();
        setcursor();
    }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
