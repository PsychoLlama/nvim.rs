//! The `ComplItem` match list: adding, freeing and ordering the matches.
//!
//! [`ins_compl_add`] links a new match into the doubly linked list
//! `compl_first_match` heads, rejecting duplicates unless the caller allows
//! them; [`ins_compl_make_cyclic`] closes the ring and
//! [`ins_compl_make_linear`] opens it again.  [`sort_compl_match_list`] is
//! `'completeopt'`'s `fuzzy` and `nearest` orderings.
//!
//! The list is [`ComplMatches`]: matches live in a `Vec` and link to each
//! other by [`MatchId`]. Every algorithm that rearranges links is a method
//! over the list that runs under one borrow and calls nothing outside it;
//! everything else reaches a match one access at a time.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::cstr;
use crate::mbyte::{char_at, cluster_len};
use crate::memory::XString;
use crate::types::{Failed, VarLock};
use crate::winlayer::Win;
use core::cmp::Ordering;

/// No `abbr`/`kind`/`menu`/`info` strings.
pub(crate) const NO_EXTRA: [Option<XString>; CPT_COUNT] = [None, None, None, None];

/// No user highlight for the `abbr` and `kind` columns.
pub(crate) const NO_HL: [c_int; 2] = [-1, -1];

/// What [`ins_compl_add`] did with a match.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Added {
    /// It was linked into the list.
    Linked,
    /// Its text is already in the list, so it was not added again.
    Duplicate,
    /// It was not added and the caller should stop adding: the user
    /// interrupted, or (`complete_add()`) there was no word to add.
    Refused,
}

impl Added {
    /// `complete_add()`'s answer: 1 added, 2 already there, 0 not added.
    pub(crate) fn number(self) -> VarNumber {
        match self {
            Added::Linked => 1,
            Added::Duplicate => 2,
            Added::Refused => 0,
        }
    }
}

/// Add one match to the list.
///
/// `text` is the match (cut at a NUL, as every C reader of it would be),
/// `fname` the file it came from, `extra` the `abbr`/`kind`/`menu`/`info`
/// strings (an empty one is dropped), `user_data` taken over only when the
/// match is added, `cdir` the side of `compl_curr_match` to link it on
/// (`DIRECTION_NOT_SET` means `compl_direction`), `adup` whether a duplicate
/// is acceptable, and `user_hl` the `abbr` and `kind` highlight attributes.
///
/// Answers [`Added::Duplicate`] when the text is already in the list and
/// [`Added::Refused`] on interrupt.
#[allow(clippy::too_many_arguments)]
pub(crate) fn ins_compl_add(
    text: &[u8],
    fname: Option<&CStr>,
    extra: [Option<XString>; CPT_COUNT],
    user_data: Option<&mut TypVal>,
    cdir: Direction,
    flags: c_int,
    adup: bool,
    user_hl: [c_int; 2],
    score: c_int,
) -> Added {
    let dir = if cdir == DIRECTION_NOT_SET {
        COMPL_DIRECTION.get()
    } else {
        cdir
    };

    if flags & CP_FAST != 0 {
        fast_breakcheck();
    } else {
        os_breakcheck();
    }
    if GOT_INT.get() {
        return Added::Refused;
    }
    let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];

    // If the same match is already present, don't add it.
    if !adup && let Some(dup) = MATCHES.with(|list| list.find_text(text)) {
        if is_nearest_active() && score > 0 {
            dup.update(|m| m.score = m.score.min(score));
        }
        return Added::Duplicate;
    }
    let text = XString::from_bytes(text);

    // Remove any popup menu before changing the list of matches.
    ins_compl_del_pum();

    // The match's file name is `compl_curr_match`'s when it is the same
    // name, else a copy of `fname`.  -- Acevedo
    let fname = fname.map(|name| {
        let current = curr_match().and_then(|curr| curr.with(|m| m.fname.clone()));
        current
            .filter(|curr| curr.as_cstr() == name)
            .unwrap_or_else(|| Rc::new(XString::from_cstr(name)))
    });
    let item = ComplItem {
        next: None,
        prev: None,
        text,
        extra: extra.map(|text| text.filter(|text| !text.is_empty())),
        user_data: user_data.map_or(TYPVAL_T_INIT, TypVal::take),
        fname,
        flags,
        number: if flags & CP_ORIGINAL_TEXT != 0 { 0 } else { -1 },
        score,
        in_match_array: false,
        user_abbr_hlattr: user_hl[0],
        user_kind_hlattr: user_hl[1],
        cpt_source_idx: cpt_sources().index(),
        cpt_generation: cpt_sources().generation(),
    };
    // The direction is ignored under `longest` + `fuzzy`, because matches
    // are inserted sorted by score.
    let by_score = cot_fuzzy() && score != FUZZY_SCORE_NONE && COMPL_GET_LONGEST.get();
    let added = MATCHES.with_mut(|list| {
        let id = list.push(item);
        list.link(id, dir, by_score);
        id
    });

    // Find the longest common string if still doing that.
    if COMPL_GET_LONGEST.get()
        && flags & CP_ORIGINAL_TEXT == 0
        && !cot_fuzzy()
        && !ins_compl_preinsert_longest()
        && !ctrl_x_mode_thesaurus()
    {
        ins_compl_longest_match(added);
    }
    Added::Linked
}

impl ComplMatches {
    /// The first match in list order, the original text aside, whose text
    /// is `text`.
    fn find_text(&self, text: &[u8]) -> Option<MatchId> {
        let first = self.COMPL_FIRST_MATCH?;
        let mut at = first;
        loop {
            let m = self.get(at);
            if !m.is_original() && *m.text == *text {
                return Some(at);
            }
            at = m.next.filter(|&next| next != first)?;
        }
    }

    /// Link the new match `id` after (FORWARD) or before (BACKWARD) the
    /// current one -- or, `by_score`, before the first match scoring lower
    /// -- and make it current.
    fn link(&mut self, id: MatchId, dir: Direction, by_score: bool) {
        match self.COMPL_FIRST_MATCH {
            // An empty list: the new match stands alone.
            None => {}
            Some(first) if by_score => {
                let score = self.get(id).score;
                let mut current = self.get(first).next;
                let mut prev = first;
                let mut inserted = false;
                while let Some(cur) = current.filter(|&cur| cur != first) {
                    if self.get(cur).score < score {
                        let before = self.get(cur).prev;
                        let new = self.get_mut(id);
                        (new.next, new.prev) = (Some(cur), before);
                        if let Some(before) = before {
                            self.get_mut(before).next = Some(id);
                        }
                        self.get_mut(cur).prev = Some(id);
                        inserted = true;
                        break;
                    }
                    prev = cur;
                    current = self.get(cur).next;
                }
                if !inserted {
                    // At the tail, closing the ring.
                    self.get_mut(prev).next = Some(id);
                    let new = self.get_mut(id);
                    (new.prev, new.next) = (Some(prev), Some(first));
                    self.get_mut(first).prev = Some(id);
                }
            }
            Some(_) => {
                // A non-empty list always has a current match, which upstream
                // dereferences here without checking.
                let curr = self
                    .COMPL_CURR_MATCH
                    .expect("a non-empty match list has a current match");
                let (after, before) = (self.get(curr).next, self.get(curr).prev);
                let new = self.get_mut(id);
                (new.next, new.prev) = if dir == FORWARD {
                    (after, Some(curr))
                } else {
                    (Some(curr), before)
                };
            }
        }
        let (next, prev) = (self.get(id).next, self.get(id).prev);
        if let Some(next) = next {
            self.get_mut(next).prev = Some(id);
        }
        match prev {
            Some(prev) => self.get_mut(prev).next = Some(id),
            // Nothing before it: it is the first match.
            None => self.COMPL_FIRST_MATCH = Some(id),
        }
        self.COMPL_CURR_MATCH = Some(id);
    }

    /// Close the list into a ring; answers the number of matches after the
    /// first.
    fn make_cyclic(&mut self) -> c_int {
        let Some(first) = self.COMPL_FIRST_MATCH else {
            return 0;
        };
        let mut last = first;
        let mut count = 0;
        while let Some(next) = self.get(last).next.filter(|&next| next != first) {
            last = next;
            count += 1;
        }
        self.get_mut(last).next = Some(first);
        self.get_mut(first).prev = Some(last);
        count
    }

    /// Open the ring back into a list with an end.
    fn make_linear(&mut self) {
        let Some(first) = self.COMPL_FIRST_MATCH else {
            return;
        };
        let Some(last) = self.get(first).prev else {
            return;
        };
        self.get_mut(last).next = None;
        self.get_mut(first).prev = None;
    }

    /// The opened chain from `head` to its end, in order.
    fn chain(&self, head: Option<MatchId>) -> Vec<MatchId> {
        let mut chain = Vec::new();
        let mut at = head;
        while let Some(id) = at {
            chain.push(id);
            at = self.get(id).next;
        }
        chain
    }

    /// Link `chain` up in its order, its ends open; answers its head.
    fn relink(&mut self, chain: &[MatchId]) -> Option<MatchId> {
        for (i, &id) in chain.iter().enumerate() {
            let m = self.get_mut(id);
            m.prev = i.checked_sub(1).map(|before| chain[before]);
            m.next = chain.get(i + 1).copied();
        }
        chain.first().copied()
    }

    /// Sort the opened chain from `head` by `order`; answers the new head.
    fn sort_chain(&mut self, head: Option<MatchId>, order: MatchOrder) -> Option<MatchId> {
        let mut chain = self.chain(head);
        merge_sort(&mut chain, |&a, &b| {
            order.compare(self.get(a).score, self.get(b).score)
        });
        self.relink(&chain)
    }

    /// Sort the ring by `order`, leaving the original text where it is: at
    /// the head when the shown matches run `forward`, else at the tail.
    fn sort(&mut self, order: MatchOrder, forward: bool) {
        let Some(first) = self.COMPL_FIRST_MATCH else {
            return;
        };
        if self.is_first(self.get(first).next) {
            return;
        }

        let tail = self.get(first).prev;
        self.make_linear();
        if forward {
            // The leader sits at the head; sort everything after it.
            let rest = self.get(first).next;
            let sorted = self.sort_chain(rest, order);
            self.get_mut(first).next = sorted;
            if let Some(sorted) = sorted {
                self.get_mut(sorted).prev = Some(first);
            }
        } else {
            // The leader sits at the tail; sort everything before it.
            // Upstream dereferences both links here without checking.
            let tail = tail.expect("a cyclic list's head has a predecessor");
            let before = self
                .get(tail)
                .prev
                .expect("the leader is not the only match");
            self.get_mut(before).next = None;
            let head = self.sort_chain(Some(first), order);
            self.COMPL_FIRST_MATCH = head;
            let last = *self
                .chain(head)
                .last()
                .expect("the sort answers a non-empty list");
            self.get_mut(last).next = Some(tail);
            self.get_mut(tail).prev = Some(last);
        }
        self.make_cyclic();
    }

    /// Number the matches around the current one, in the running direction.
    fn number_from_current(&mut self, forward: bool) {
        // The link away from the direction the walk runs in, and back.
        let away = |list: &Self, id: MatchId| {
            let m = list.get(id);
            if forward { m.prev } else { m.next }
        };
        let back = |list: &Self, id: MatchId| {
            let m = list.get(id);
            if forward { m.next } else { m.prev }
        };
        let mut number = 0;
        // Upstream dereferences `compl_curr_match` here without checking.
        let curr = self
            .COMPL_CURR_MATCH
            .expect("a running completion has a current match");
        // Look away from the walk for the first match with a number...
        let mut at = away(self, curr);
        while let Some(id) = at.filter(|&id| !self.is_first(Some(id))) {
            if self.get(id).number != -1 {
                number = self.get(id).number;
                break;
            }
            at = away(self, id);
        }
        // ...then come back numbering every match that has none.
        if let Some(id) = at {
            let mut next = back(self, id);
            while let Some(id) = next.filter(|&id| self.get(id).number == -1) {
                number += 1;
                self.get_mut(id).number = number;
                next = back(self, id);
            }
        }
    }

    /// Unlink every match from `'complete'` entry `source` out of an opened
    /// list, answering them so the caller drops them outside the borrow.
    /// The shown match, if it went, moves to the head (`forward`) or the
    /// tail; the current match moves to the last match from an earlier
    /// entry.
    fn remove_source(&mut self, source: c_int, forward: bool) -> Vec<ComplItem> {
        let mut shown_match_removed = false;
        let mut removed = Vec::new();
        // Under `'completeopt'` `fuzzy` the items are not in source order, so
        // they have to be removed one by one rather than as a run.
        let mut current = self.COMPL_FIRST_MATCH;
        while let Some(cur) = current {
            let (next, prev) = (self.get(cur).next, self.get(cur).prev);
            current = next;
            if self.get(cur).cpt_source_idx != source {
                continue;
            }
            if self.COMPL_SHOWN_MATCH == Some(cur) {
                shown_match_removed = true;
            }
            if Some(cur) == self.COMPL_FIRST_MATCH {
                // Head.
                self.COMPL_FIRST_MATCH = next;
                if let Some(head) = next {
                    self.get_mut(head).prev = None;
                }
            } else if let Some(prev) = prev {
                // Middle or tail.
                self.get_mut(prev).next = next;
                if let Some(next) = next {
                    self.get_mut(next).prev = Some(prev);
                }
            }
            removed.push(self.take(cur));
        }

        if shown_match_removed {
            if forward {
                self.COMPL_SHOWN_MATCH = self.COMPL_FIRST_MATCH;
            } else if let Some(first) = self.COMPL_FIRST_MATCH {
                // The last node carries the prefix being completed.
                self.COMPL_SHOWN_MATCH = self.chain(Some(first)).last().copied();
            }
        }

        self.COMPL_CURR_MATCH = self.COMPL_FIRST_MATCH;
        let mut current = self.COMPL_FIRST_MATCH;
        while let Some(cur) = current {
            let (idx, next) = (self.get(cur).cpt_source_idx, self.get(cur).next);
            let earlier = if forward { idx < source } else { idx > source };
            if !earlier {
                break;
            }
            self.COMPL_CURR_MATCH = if forward { Some(cur) } else { next };
            current = next;
        }
        removed
    }

    /// Empty the list, answering what it held so the caller drops it
    /// outside the borrow.
    fn clear(&mut self) -> Vec<Option<ComplItem>> {
        self.COMPL_FIRST_MATCH = None;
        self.COMPL_CURR_MATCH = None;
        self.COMPL_SHOWN_MATCH = None;
        self.COMPL_OLD_MATCH = None;
        self.MATCH_GENERATION = self.MATCH_GENERATION.wrapping_add(1);
        core::mem::take(&mut self.MATCH_SLOTS)
    }
}

/// How [`sort_compl_match_list`] orders the matches.
#[derive(Clone, Copy, Debug)]
pub(crate) enum MatchOrder {
    /// Highest fuzzy score first.
    Fuzzy,
    /// Nearest to the cursor first; an unscored match compares equal to
    /// everything.
    Nearest,
}

impl MatchOrder {
    fn compare(self, a: c_int, b: c_int) -> Ordering {
        match self {
            MatchOrder::Fuzzy => b.cmp(&a),
            MatchOrder::Nearest if a == FUZZY_SCORE_NONE || b == FUZZY_SCORE_NONE => {
                Ordering::Equal
            }
            MatchOrder::Nearest => a.cmp(&b),
        }
    }
}

/// Upstream's bottom-up merge sort, over a slice: runs of 1, 2, 4, …
/// merged pairwise from the left, the left run winning a tie.
///
/// Not `slice::sort_by`: `nearest`'s comparator is not transitive (an
/// unscored match equals everything), and with such a comparator the order
/// is the algorithm's, so it has to be this one.
fn merge_sort<T: Copy>(items: &mut Vec<T>, compare: impl Fn(&T, &T) -> Ordering) {
    let n = items.len();
    let mut size = 1;
    let mut merged = Vec::with_capacity(n);
    while size < n {
        merged.clear();
        let mut start = 0;
        while start < n {
            let mid = (start + size).min(n);
            let end = (start + 2 * size).min(n);
            let (mut left, mut right) = (start, mid);
            while left < mid || right < end {
                let take_left =
                    left < mid && (right >= end || compare(&items[left], &items[right]).is_le());
                if take_left {
                    merged.push(items[left]);
                    left += 1;
                } else {
                    merged.push(items[right]);
                    right += 1;
                }
            }
            start = end;
        }
        core::mem::swap(items, &mut merged);
        size *= 2;
    }
}

/// [`ins_compl_add`] for the original text: the first match every completion
/// starts with, taken from `compl_orig_text`.
pub(crate) fn ins_compl_add_orig_text(flags: c_int) -> Result<(), Failed> {
    let text = compl_orig_text().to_vec();
    let (dir, score) = (DIRECTION_NOT_SET, FUZZY_SCORE_NONE);
    let added = ins_compl_add(&text, None, NO_EXTRA, None, dir, flags, false, NO_HL, score);
    // A text already in the list is "not added" to upstream's callers here.
    if added == Added::Linked {
        Ok(())
    } else {
        Err(Failed)
    }
}

/// Does the text of `m` start with `leader`, honouring its `CP_ICASE` /
/// `CP_EQUAL` flags?
pub(crate) fn ins_compl_equal(m: MatchId, leader: ComplStr) -> bool {
    leader.with_bytes(|leader| m.with(|item| starts_with_leader(item, leader)))
}

/// C's `strncmp(text, leader, len) == 0` (`strncasecmp` under `CP_ICASE`)
/// over the whole of `leader`.
fn starts_with_leader(item: &ComplItem, leader: &[u8]) -> bool {
    if item.flags & CP_EQUAL != 0 {
        return true;
    }
    if item.flags & CP_ICASE != 0 {
        // `strncasecmp` over the leader's length.
        return cstr::ncase_eq_ascii(&item.text, leader, leader.len());
    }
    // `strncmp` stops at a NUL in either: the text has none inside it, so a
    // NUL in the leader matches only the text's terminator.
    let cut = leader.iter().position(|&b| b == 0);
    let leader = &leader[..cut.unwrap_or(leader.len())];
    item.text.starts_with(leader) && cut.is_none_or(|at| item.text.len() == at)
}

/// The byte length of the longest prefix of `leader` that `text` shares,
/// compared a character at a time (case-folded under `icase`).
fn common_prefix_len(leader: &[u8], text: &[u8], icase: bool) -> usize {
    let (mut p, mut s) = (0, 0);
    while p < leader.len() {
        let (c1, c2) = (char_at(&leader[p..]), char_at(&text[s..]));
        let differ = if icase {
            mb_tolower(c1) != mb_tolower(c2)
        } else {
            c1 != c2
        };
        if differ {
            break;
        }
        p += cluster_len(&leader[p..]);
        s += cluster_len(&text[s..]);
    }
    p
}

/// Shorten `compl_leader` to the longest prefix it shares with `m`, and
/// put that prefix in the buffer.
pub(crate) fn ins_compl_longest_match(m: MatchId) {
    if compl_leader().is_unset() {
        compl_leader().set_string(m.text_copy());
        let had_match = Win::current().w_cursor.col > COMPL_COL.get();
        ins_compl_longest_insert(&compl_leader().to_vec());
        if !had_match {
            ins_compl_delete(false);
        }
        COMPL_USED_MATCH.set(false);
        return;
    }

    let icase = m.with(|m| m.flags) & CP_ICASE != 0;
    let keep = compl_leader()
        .with_bytes(|leader| m.with(|item| common_prefix_len(leader, &item.text, icase)));
    if keep < compl_leader().len() {
        compl_leader().truncate(keep);
        let had_match = Win::current().w_cursor.col > COMPL_COL.get();
        ins_compl_longest_insert(&compl_leader().to_vec());
        if !had_match {
            ins_compl_delete(false);
        }
    }
    COMPL_USED_MATCH.set(false);
}

/// Add each of `matches` in turn, the first in the completion's direction
/// and the rest forwards.
pub(crate) fn ins_compl_add_match_list(matches: &[XString], icase: bool) {
    let mut dir = COMPL_DIRECTION.get();
    let flags = CP_FAST | if icase { CP_ICASE } else { 0 };
    for text in matches {
        let text = text.as_cstr().to_bytes();
        let score = FUZZY_SCORE_NONE;
        match ins_compl_add(text, None, NO_EXTRA, None, dir, flags, false, NO_HL, score) {
            Added::Refused => break,
            Added::Linked => dir = FORWARD,
            Added::Duplicate => {}
        }
    }
}

/// Close the list into a ring; returns the number of matches after the first.
pub(crate) fn ins_compl_make_cyclic() -> c_int {
    MATCHES.with_mut(ComplMatches::make_cyclic)
}

/// Open the ring back into a list with an end.
pub(crate) fn ins_compl_make_linear() {
    MATCHES.with_mut(ComplMatches::make_linear);
}

/// Score every match against the leader (or, with no leader, against the
/// original text).
pub(crate) fn set_fuzzy_score() {
    let Some(first) = first_match() else {
        return;
    };

    // Determine the pattern to match against.
    let use_leader = !compl_leader().is_unset() && !compl_leader().is_empty();
    if use_leader {
        // Clear the leader cache once before the loop; the pattern is
        // then computed per match, since each may have its own startcol.
        clear_adjusted_leader();
    } else if compl_orig_text().is_unset() || compl_orig_text().is_empty() {
        return;
    }

    for comp in matches_from(Some(first)) {
        let pattern = if use_leader {
            get_leader_for_startcol(comp, true)
        } else {
            compl_orig_text()
        };
        let score = pattern.with_cstr(|pat| comp.with(|m| fuzzy_match_str(m.text.as_cstr(), pat)));
        comp.update(|m| m.score = score);
    }
}

/// Sort the match list by `order`, leaving the node holding the leader
/// (the original text) where it is.
pub(crate) fn sort_compl_match_list(order: MatchOrder) {
    let forward = compl_shows_dir_forward();
    MATCHES.with_mut(|list| list.sort(order, forward));
}

/// Free the whole match list and the pattern and leader that built it.
pub(crate) fn ins_compl_free() {
    compl_pattern().clear();
    compl_leader().clear();

    if first_match().is_none() {
        return;
    }

    ins_compl_del_pum();
    pum_clear();

    // Dropped here, after the borrow: a match's user data is a Vimscript
    // value, and releasing one is not a leaf.
    let freed = MATCHES.with_mut(ComplMatches::clear);
    drop(freed);
}

/// Reset everything a completion left behind, without freeing the list.
pub fn ins_compl_clear() {
    COMPL_CONT_STATUS.set(0);
    COMPL_STARTED.set(false);
    COMPL_MATCHES.set(0);
    COMPL_SELECTED_ITEM.set(-1);
    COMPL_INS_END_COL.set(ColNr::ZERO);
    COMPL_CURR_WIN.set(None);
    COMPL_CURR_BUF.set(None);
    compl_pattern().clear();
    compl_leader().clear();
    EDIT_SUBMODE_EXTRA.set(None);
    compl_orig_extmarks().clear();
    compl_orig_text().clear();
    COMPL_ENTER_SELECTS.set(false);
    cpt_sources().clear();
    COMPL_AUTOCOMPLETE.set(false);
    COMPL_FROM_NONKEYWORD.set(false);
    COMPL_NUM_BESTS.set(0);
    set_vim_var_dict(Vv::CompletedItem, Some(tv_dict_alloc_lock(VarLock::Fixed)));
}

/// Score the matches and, unless `'completeopt'` says `nosort`, reorder them.
pub(crate) fn ins_compl_fuzzy_sort() {
    let cur_cot_flags = completeopt_flags();

    set_fuzzy_score();
    if cur_cot_flags & kOptCotFlagNosort != 0 {
        return;
    }
    sort_compl_match_list(MatchOrder::Fuzzy);

    // Sorting reorders the items, so the shown one has to be reset.
    if cur_cot_flags & (kOptCotFlagNoinsert | kOptCotFlagNoselect) != kOptCotFlagNoinsert {
        return;
    }
    let first = first_match();
    let unselected = if compl_shows_dir_forward() {
        first
    } else {
        first.and_then(MatchId::prev)
    };
    if shown_match() == unselected {
        return;
    }
    let next = if !COMPL_AUTOCOMPLETE.get() && compl_shows_dir_forward() {
        first.and_then(MatchId::next)
    } else {
        first
    };
    COMPL_SHOWN_MATCH.set(next);
}

/// Number the matches around `compl_curr_match`, in the direction the
/// completion is running.
pub(crate) fn ins_compl_update_sequence_numbers() {
    let forward = compl_dir_forward();
    debug_assert!(forward || COMPL_DIRECTION.get() == BACKWARD);
    MATCHES.with_mut(|list| list.number_from_current(forward));
}

/// Drop every match the current `'complete'` source contributed, so it can be
/// re-run (`refresh: 'always'`). The list is opened.
pub(crate) fn remove_old_matches() {
    // Upstream dereferences `compl_first_match` here without checking.
    let head = first_match().expect("a refresh runs on a non-empty match list");
    let forward = head.with(|m| m.cpt_source_idx) < 0;
    let source = cpt_sources().index();
    if source < 0 {
        return;
    }

    COMPL_DIRECTION.set(if forward { FORWARD } else { BACKWARD });
    COMPL_SHOWS_DIR.set(COMPL_DIRECTION.get());

    // Dropped after the borrow, as in `ins_compl_free`.
    let removed = MATCHES.with_mut(|list| list.remove_source(source, forward));
    drop(removed);
}
