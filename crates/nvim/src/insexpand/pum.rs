//! The popup menu: turning the match list into `PumItem`s and showing it.
//!
//! [`ins_compl_build_pum`] is the whole of it — it filters the list by the
//! current leader, scores and sorts what survives, and fills
//! `compl_match_array`.  [`ins_compl_show_pum`] then hands that to
//! `pum_display`, and [`trigger_complete_changed_event`] fires
//! `CompleteChanged` with the selected item.

#![forbid(unsafe_code)]

use super::*;
use crate::autocmd::fire_autocmds;
use crate::formatted;
use crate::guard::{Allow, Lock};
use crate::memline::Lines;
use crate::memory::XString;
use crate::message::msg;
use crate::tr;
use crate::types::{IOSIZE, ShmFlag};
use crate::winlayer::Buf;
use crate::winlayer::Win;

/// The popup menu's view of the match list.
///
/// [`ins_compl_build_pum`] filters the cyclic `ComplItem` list by the current
/// leader, sorts what survives and flattens it into a `PumItem` array;
/// [`ins_compl_show_pum`] shares that array with `pum_display`. Upstream
/// keeps it as a bare `PumItem *` with `compl_match_arraysize` beside it,
/// with "is the menu up?" spelled as a null check on the pointer; here it is
/// an `Rc<[PumItem]>` in [`ComplState`], empty for that null, with the
/// matches each row was built from beside it.
///
/// The rows own copies of their match's strings, so the menu's share stays
/// good whatever happens to the match list. Upstream finds the shown match's
/// row by comparing string *addresses*; the copies have addresses of their
/// own, so the row is found by the match id it came from instead.
#[derive(Clone, Copy)]
pub(crate) struct ComplMatchArray(());

/// The popup menu's item array. See [`ComplMatchArray`].
pub(crate) fn compl_match_array() -> ComplMatchArray {
    ComplMatchArray(())
}

/// The array behind [`ComplMatchArray`]: the rows, and the match each came
/// from.
pub(crate) struct MatchRows {
    rows: Option<Rc<[PumItem]>>,
    ids: Vec<MatchId>,
}

impl MatchRows {
    /// No array: upstream's null `compl_match_array`.
    pub(crate) const EMPTY: MatchRows = MatchRows {
        rows: None,
        ids: Vec::new(),
    };
}

impl ComplMatchArray {
    /// Whether there is no array at all -- upstream's
    /// `compl_match_array == NULL`, which is how the completion asks whether
    /// a menu is up.
    pub(crate) fn is_unset(self) -> bool {
        COMPL_MATCH_ARRAY.with(|array| array.ids.is_empty())
    }

    /// The number of items.
    #[cfg(test)]
    pub(crate) fn len(self) -> c_int {
        COMPL_MATCH_ARRAY.with(|array| array.ids.len()) as c_int
    }

    /// The index of the row built from match `id`.
    pub(crate) fn position_of(self, id: MatchId) -> Option<usize> {
        COMPL_MATCH_ARRAY.with(|array| array.ids.iter().position(|&row| row == id))
    }

    /// The rows, shared: what `pum_display` keeps.
    pub(crate) fn rows(self) -> Rc<[PumItem]> {
        COMPL_MATCH_ARRAY
            .with(|array| array.rows.clone())
            .unwrap_or_else(|| Rc::from([]))
    }

    /// Take `rows`, built from the matches `ids`, as the new array, dropping
    /// whatever was there. An empty `rows` leaves the array unset.
    pub(crate) fn set(self, rows: Rc<[PumItem]>, ids: Vec<MatchId>) {
        debug_assert_eq!(rows.len(), ids.len(), "one match per row");
        let rows = (!rows.is_empty()).then_some(rows);
        COMPL_MATCH_ARRAY.set(MatchRows { rows, ids });
    }

    /// Drop the array. The menu must already be down.
    pub(crate) fn clear(self) {
        COMPL_MATCH_ARRAY.set(MatchRows::EMPTY);
    }
}

/// The highlight attribute for the inserted-but-not-accepted text at
/// `lnum`/`col`, or −1 where there is none.
pub fn ins_compl_col_range_attr(lnum: LineNr, col: c_int) -> c_int {
    let (preinsert, longest) = (ins_compl_has_preinsert(), ins_compl_preinsert_longest());
    if cot_fuzzy() || (!compl_hi_on_autocompl_longest.get() && longest) {
        return -1;
    }
    // Either kind of preview takes `PreInsert`; only an accepted match that
    // has not been confirmed yet takes `ComplMatchIns`.
    let name = if preinsert || longest {
        c"PreInsert"
    } else {
        c"ComplMatchIns"
    };
    let attr = syn_name2attr(name);
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
    let cursor_lnum = Win::current().w_cursor.lnum;
    let inside = (lnum == compl_lnum.get() && col >= start_col && col < MAXCOL)
        || (lnum > compl_lnum.get() && lnum < cursor_lnum)
        || (lnum == cursor_lnum && col <= compl_ins_end_col.get());
    if inside { attr } else { -1 }
}

/// Take the popup menu down and drop the item array it was built from.
pub(crate) fn ins_compl_del_pum() {
    if compl_match_array().is_unset() {
        return;
    }
    pum_undisplay(false);
    compl_match_array().clear();
}

/// Whether a popup menu is wanted at all: `'completeopt'` has `menu` or
/// `menuone`, or autocompletion is on.
pub fn pum_wanted() -> bool {
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
pub(crate) fn trigger_complete_changed_event(cur: c_int) {
    if complete_changed_busy.get() {
        return;
    }

    // The dict is a fresh allocation `v:event` takes over.
    let item_held = if cur < 0 {
        tv_dict_alloc()
    } else {
        ins_compl_dict_alloc(curr_match().expect("a selected item is the current match"))
    };
    let mut save_v_event = SAVE_V_EVENT_INIT;
    let v_event = get_v_event(&mut save_v_event);
    let buffer = Buf::current();
    let _ = v_event.edit().add_dict(b"completed_item", Some(item_held));
    pum_set_event_info(v_event.edit());
    v_event.edit().set_keys_readonly();

    complete_changed_busy.set(true);
    let locked = Lock::text();

    fire_autocmds(AutoEvent::CompleteChanged, false, Some(buffer));
    drop(locked);
    complete_changed_busy.set(false);

    restore_v_event(v_event, &mut save_v_event);
}

/// C's `"match %d of %d"` / `"match %d"`, the line `showmode` shows under
/// the menu. `total` of zero or less selects the short form.
fn match_position_message(number: c_int, total: c_int) -> XString {
    let text = if total > 0 {
        tr!("match {number} of {total}")
    } else {
        tr!("match {number}")
    };
    XString::from(text.as_str())
}

/// Build `dest` by prepending the buffer text from `startcol` to `compl_col`
/// to `src`.
pub(crate) fn prepend_startcol_text(dest: ComplStr, src: ComplStr, startcol: c_int) {
    let prepend_len = usize::try_from(compl_col.get() - startcol).unwrap_or(0);
    let new_length = prepend_len + src.len();

    let mut text = XString::with_capacity(new_length);
    {
        let mut lines = Lines::current();
        let line = lines.line(Win::current().w_cursor.lnum);
        let at = usize::try_from(startcol).unwrap_or(0).min(line.len());
        text.push_bytes(&line[at..at.saturating_add(prepend_len).min(line.len())]);
    }
    src.with_bytes(|src| text.push_bytes(src));
    dest.set_string(text);
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
pub(crate) fn get_leader_for_startcol(match_0: MatchId, cached: bool) -> ComplStr {
    'theend: {
        if cpt_sources().is_unset() {
            break 'theend;
        }
        let Some(cpt_idx) = match_0.with(ComplItem::cpt_source) else {
            break 'theend;
        };
        let startcol = cpt_sources().row(cpt_idx as c_int).cs_startcol;

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

/// Build `compl_match_array` from the match list.
///
/// Returns the entry that should be selected, or −1 for none.
pub(crate) fn ins_compl_build_pum() -> c_int {
    // Under a user completion function with `refresh: 'always'` the leader
    // is not a prefix filter, so drop it.
    //
    // Upstream writes `XFREE_CLEAR(compl_leader)` — the *struct*, not
    // `.data` — which frees and NULLs the first member and leaves `.size`
    // stale. Every reader of the length asks `is_unset` first, so the stale
    // count was never read, and the null string is the same thing.
    if ins_compl_need_restart() {
        compl_leader().clear();
    }

    let has_preinsert = ins_compl_has_preinsert();
    let compl_no_select = completeopt_flags() & kOptCotFlagNoselect != 0
        || (compl_autocomplete.get() && !has_preinsert);

    // The matches the menu shows, in list order.
    let mut shown_in_menu: Vec<MatchId> = Vec::new();
    let is_forward = compl_shows_dir_forward();
    let is_cpt_completion = !cpt_sources().is_unset();

    // If the current match is the original text, don't find the first
    // match after it and don't highlight anything.
    let mut shown_match_ok = shown_match().is_some_and(MatchId::is_original);

    // C's `strequal`: both unset, or both set to the same C string.
    let leader_is_orig = match (compl_leader().is_unset(), compl_orig_text().is_unset()) {
        (true, true) => true,
        (false, false) => {
            compl_leader().with_cstr(|leader| compl_orig_text().with_cstr(|orig| leader == orig))
        }
        _ => false,
    };
    if leader_is_orig && !shown_match_ok {
        // Upstream dereferences `compl_first_match` here without checking.
        let head = first_match().expect("a menu is built from a non-empty list");
        compl_shown_match.set(if compl_no_select {
            Some(head)
        } else {
            head.next()
        });
    }

    let mut did_find_shown_match = false;
    let mut shown_compl: Option<MatchId> = None;
    let mut i = 0;
    let mut cur = -1;

    // How many matches each `'complete'` source has in the menu so far.
    let mut match_count = vec![
        0;
        if is_cpt_completion {
            cpt_sources().len()
        } else {
            0
        }
    ];

    clear_adjusted_leader();

    for comp in matches_from(first_match()) {
        comp.update(|m| m.in_match_array = false);

        let leader = get_leader_for_startcol(comp, true);

        // Apply 'smartcase' behaviour during normal mode.
        if ctrl_x_mode_normal()
            && !p_inf()
            && !leader.is_unset()
            && !leader.with_bytes(ignorecase)
            && !cot_fuzzy()
        {
            comp.update(|m| m.flags &= !CP_ICASE);
        }

        let displayed = !comp.is_original()
            && (leader.is_unset()
                || ins_compl_equal(comp, leader)
                || (cot_fuzzy() && comp.with(|m| m.score) != FUZZY_SCORE_NONE));
        if displayed {
            // Limit the number of items from each source where
            // `cs_max_matches` is set.
            let mut match_limit_exceeded = false;
            let cur_source = comp.with(ComplItem::cpt_source);
            if let Some(source) = cur_source.filter(|_| is_forward && is_cpt_completion)
                && let Some(count) = match_count.get_mut(source)
            {
                *count += 1;
                let max_matches = cpt_sources().row(source as c_int).cs_max_matches;
                if max_matches > 0 && *count > max_matches {
                    match_limit_exceeded = true;
                }
            }

            if !match_limit_exceeded {
                comp.update(|m| m.in_match_array = true);
                shown_in_menu.push(comp);

                if !shown_match_ok && !cot_fuzzy() {
                    if shown_match() == Some(comp) || did_find_shown_match {
                        // This item is the shown match, or the first
                        // displayed item after it.
                        compl_shown_match.set(Some(comp));
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
                compl_shown_match.set(Some(prev));
                shown_match_ok = true;
            }
        }
    }

    if shown_in_menu.is_empty() {
        compl_match_array().clear();
        return -1;
    }

    if cot_fuzzy() && !compl_no_select && !shown_match_ok {
        compl_shown_match.set(shown_compl);
        shown_match_ok = true;
        cur = 0;
    }

    // Collected straight into the shared array: one allocation, not a
    // vector copied into one.
    let array: Rc<[PumItem]> = MATCHES.with(|list| {
        let item = |id: MatchId| {
            let m = list.get(id);
            let extra = |which: c_int| m.extra[which as usize].clone();
            PumItem {
                pum_text: extra(CPT_ABBR).unwrap_or_else(|| m.text.clone()),
                pum_kind: extra(CPT_KIND),
                pum_info: extra(CPT_INFO),
                pum_cpt_source_idx: m.cpt_source_idx,
                pum_user_abbr_hlattr: m.user_abbr_hlattr,
                pum_user_kind_hlattr: m.user_kind_hlattr,
                pum_extra: extra(CPT_MENU).or_else(|| m.fname.as_deref().cloned()),
            }
        };
        shown_in_menu.iter().map(|&id| item(id)).collect()
    });
    compl_match_array().set(array, shown_in_menu);

    if !shown_match_ok {
        // No displayed match at all.
        cur = -1;
    }
    cur
}

/// Show the popup menu, adjusting `compl_shown_match` to an entry that is
/// actually displayed.
pub fn ins_compl_show_pum() {
    if !pum_wanted() || !pum_enough_matches() {
        return;
    }

    // Update the screen before drawing the popup menu over it.
    let _ = update_screen();

    let mut cur = -1;
    let mut array_changed = false;

    if compl_match_array().is_unset() {
        array_changed = true;
        cur = ins_compl_build_pum();
    } else if let Some(shown) = shown_match() {
        // The menu already exists; only the current item has to be found.
        if let Some(i) = compl_match_array().position_of(shown) {
            cur = i as c_int;
        }
    }

    if compl_match_array().is_unset() {
        if compl_started.get() && has_event(AutoEvent::CompleteChanged) {
            trigger_complete_changed_event(cur);
        }
        return;
    }

    // Avoid a wait_return() when the message is cleared.
    dollar_vcol.set(-1);

    // Move the cursor to the start of the match for the popup menu's sake,
    // then put it back.
    let col = Win::current().w_cursor.col;
    Win::current().w_cursor.col = compl_col.get();
    compl_selected_item.set(cur);
    pum_display(compl_match_array().rows(), cur, array_changed, 0);
    Win::current().w_cursor.col = col;

    if compl_started.get() && compl_curr_match.get() != compl_shown_match.get() {
        compl_curr_match.set(compl_shown_match.get());
    }

    if has_event(AutoEvent::CompleteChanged) {
        trigger_complete_changed_event(cur);
    }
}

/// Is `selected` (a menu index) the match `compl_curr_match` points at?
pub(crate) fn compl_match_curr_select(selected: c_int) -> bool {
    if selected < 0 {
        return false;
    }
    let mut selected_idx = -1;
    let mut list_idx = 0;
    for match_0 in matches_from(first_match()) {
        if match_0.is_original() {
            continue;
        }
        if curr_match().is_some_and(|curr| curr.with(|m| m.number) == match_0.with(|m| m.number)) {
            selected_idx = list_idx;
            break;
        }
        list_idx += 1;
    }
    selected == selected_idx
}

/// Report which file the shown match came from, truncating the name on the
/// left to whatever room the status line leaves.
pub(crate) fn ins_compl_show_filename() {
    let lead = gettext(c"match in file");
    let mut space = sc_col.get() - strsize(lead.to_bytes()) - 2;
    if space <= 0 {
        return;
    }

    // Find `s` such that `s..e` fits in `space` cells.
    let shown = shown_match().expect("the file name of a shown match");
    let Some(name) = shown.with(|m| m.fname.clone()) else {
        return;
    };
    let fname: &[u8] = &name;
    let mut s = 0;
    let mut e = 0;
    while e < fname.len() {
        space -= ptr2cells_at(&fname[e..]);
        while space < 0 {
            space += ptr2cells_at(&fname[s..]);
            s += c_cluster_len(&fname[s..]);
        }
        e += c_cluster_len(&fname[e..]);
    }

    if compl_autocomplete.get() {
        return;
    }
    msg_hist_off.set(true);
    let ellipsis: &[u8] = if s > 0 { b"<" } else { b"" };
    // Upstream formats into `IObuff`, which cuts it there.
    let text = formatted!(IOSIZE as usize, b"%s %s%s", lead, ellipsis, &fname[s..]);
    msg(XString::from_vec(text).as_cstr(), 0);
    msg_hist_off.set(false);
    redraw_cmdline.set(false);
}

/// The next match that is actually in the menu, in the direction the menu is
/// being walked.
pub(crate) fn find_next_match_in_menu() -> MatchId {
    let is_forward = compl_shows_dir_forward();
    let mut at = shown_match().expect("the menu is walked from a shown match");
    MATCHES.with(|list| {
        loop {
            let m = list.get(at);
            let step = if is_forward { m.next } else { m.prev };
            at = step.expect("the ring closes, so neither link is null");
            let m = list.get(at);
            if m.next.is_none() || m.in_match_array || m.is_original() {
                break at;
            }
        }
    })
}

/// The "match 3 of 17" / "Back at original" line under the menu.
pub(crate) fn ins_compl_show_statusmsg() {
    // Show a message about what (completion) mode we're in.
    // Upstream dereferences `compl_first_match` here without checking.
    let head = first_match().expect("a completion showing a message has matches");
    if is_first_match(head.next()) {
        let text = if compl_status_adding() && compl_length.get() > 1 {
            E_HITEND
        } else {
            e_patnotf
        };
        edit_submode_extra.set(Some(XString::from_cstr(gettext(text))));
        edit_submode_highl.set(HLF_E);
    }

    if edit_submode_extra.with(Option::is_none) {
        let curr = curr_match().expect("a running completion has a current match");
        if curr.is_original() {
            edit_submode_extra.set(Some(XString::from_cstr(gettext(c"Back at original"))));
            edit_submode_highl.set(HLF_W);
        } else if compl_cont_status.get() & CONT_S_IPOS != 0 {
            edit_submode_extra.set(Some(XString::from_cstr(gettext(c"Word from other line"))));
            edit_submode_highl.set(HLF_COUNT);
        } else if curr.next() == curr.prev() {
            edit_submode_extra.set(Some(XString::from_cstr(gettext(c"The only match"))));
            edit_submode_highl.set(HLF_COUNT);
            curr.update(|m| m.number = 1);
        } else {
            // Update the number, it is used in `msg_ext_set_kind`.
            if curr.with(|m| m.number) == -1 {
                ins_compl_update_sequence_numbers();
            }
            let number = curr.with(|m| m.number);
            if number != -1 {
                let msg = match_position_message(number, compl_matches.get());
                edit_submode_extra.set(Some(msg));
                edit_submode_highl.set(HLF_R);
                if dollar_vcol.get() >= 0 {
                    curs_columns(Win::current(), 0);
                }
            }
        }
    }

    redraw_mode.set(true);
    if shortmess(ShmFlag::COMPLETIONMENU) {
        return;
    }
    // A copy: showing it runs the message machinery.
    match edit_submode_extra.with(Clone::clone) {
        None => msg_clr_cmdline(),
        Some(extra) if !p_smd() => {
            msg_hist_off.set(true);
            let attr = if (edit_submode_highl.get() as c_uint) < HLF_COUNT as c_uint {
                edit_submode_highl.get() as c_int + 1
            } else {
                0
            };
            msg_ext_set_kind(c"completion");
            msg(extra.as_cstr(), attr);
            msg_hist_off.set(false);
        }
        Some(_) => {}
    }
}

/// Redraw the popup menu after the cursor may have moved, with redrawing
/// forced back on.
pub(crate) fn show_pum(prev_w_wrow: c_int, prev_w_leftcol: c_int) {
    // RedrawingDisabled may be set when invoked through complete().
    let _redraw = Allow::redraw();

    // If the cursor moved or the display scrolled, the menu has to be
    // rebuilt rather than only redrawn.
    setcursor();
    if prev_w_wrow != Win::current().w_wrow || prev_w_leftcol != Win::current().w_leftcol {
        ins_compl_del_pum();
    }
    ins_compl_show_pum();
    setcursor();
}
