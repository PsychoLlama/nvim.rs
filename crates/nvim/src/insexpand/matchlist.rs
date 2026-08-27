//! The `compl_T` match list: adding, freeing and ordering the matches.
//!
//! [`ins_compl_add`] links a new match into the circular doubly-linked list
//! `compl_first_match` heads, rejecting duplicates unless the caller allows
//! them; [`ins_compl_make_cyclic`] closes the ring and
//! [`ins_compl_make_linear`] opens it again.  The comparators and
//! [`sort_compl_match_list`] are `'completeopt'`'s `fuzzy` and `nearest`
//! orderings.
//!
//! A node is held as a [`Cm`] — the `Copy` handle declared beside `compl_T`
//! itself — so the walks and the pointer surgery below are ordinary checked
//! code and only the C calls they make are not.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{FAIL, NUL, OK, VarLock};
use crate::winlayer::{Live, Win};

/// One node of the match list, whose caller has promised it outlives the
/// value.
///
/// The matches are a doubly linked chain headed by `compl_first_match`,
/// which [`ins_compl_make_cyclic`] closes into a ring and
/// [`ins_compl_make_linear`] opens again. Upstream passes `*mut compl_T`
/// around rather than holding a borrow because a completion runs
/// `'completefunc'`, autocommands and Lua, any of which can reach the same
/// list — which is exactly the timing [`Live`]'s `Deref` gives, a borrow
/// that lasts one field access. Wrapping is the unsafe step, once; every
/// `(*m).cp_field` after it is ordinary checked code.
pub(crate) type Cm = Live<compl_T>;

impl Cm {
    /// The match `p` names, `None` for null.
    ///
    /// Safe on the list's own invariant, the one every walk in this family
    /// already rests on: a non-null `compl_T` pointer reached from the four
    /// state cells or from a node's `cp_next`/`cp_prev`/`cp_match_next`
    /// names a node of the live list, and the one free path
    /// ([`ins_compl_item_free`]) is reached only with the node already
    /// unlinked, or with the whole list going at once. This is [`Frame`]'s
    /// bargain, for a list with no handle registry to ask instead.
    ///
    /// [`Frame`]: crate::winlayer::Frame
    #[inline(always)]
    pub(crate) fn at(p: *mut compl_T) -> Option<Self> {
        // SAFETY: the list invariant above.
        (!p.is_null()).then(|| unsafe { Self::new(p) })
    }

    /// The match after this one, `None` past the tail of an opened list.
    #[inline(always)]
    pub(crate) fn next(self) -> Option<Self> {
        Self::at(self.cp_next)
    }

    /// The match before this one, `None` before the head of an opened list.
    #[inline(always)]
    pub(crate) fn prev(self) -> Option<Self> {
        Self::at(self.cp_prev)
    }

    /// The next match in the `cp_match_next` chain the popup menu's filtering
    /// builds.
    #[inline(always)]
    pub(crate) fn match_next(self) -> Option<Self> {
        Self::at(self.cp_match_next)
    }

    /// Whether this is the head of the list — C's `is_first_match`.
    #[inline(always)]
    pub(crate) fn is_first(self) -> bool {
        is_first_match(self.raw())
    }

    /// Whether this is the original text the completion began with — C's
    /// `match_at_original_text`.
    #[inline(always)]
    pub(crate) fn is_original(self) -> bool {
        self.cp_flags & CP_ORIGINAL_TEXT != 0
    }
}

/// Free the four `cptext` strings a caller handed to [`ins_compl_add`].
#[inline]
pub(crate) unsafe fn free_cptext(cptext: *const *mut c_char) {
    if cptext.is_null() {
        return;
    }
    for i in 0..CPT_COUNT as isize {
        // SAFETY: the caller's promise -- `CPT_COUNT` strings, each this
        // module's own allocation or null, which `xfree` takes.
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
///
/// # Safety
/// `str` is readable for `len` bytes, or NUL-terminated when `len < 0`;
/// `fname` is null or NUL-terminated; `cptext` is null or `CPT_COUNT`
/// strings; `user_hl` is null or two `c_int`s; `user_data` is null or a live
/// `typval_T`.
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
            // SAFETY: the caller's four allocated strings.
            unsafe { free_cptext(cptext) };
        }
        return FAIL;
    }
    if len < 0 {
        // SAFETY: `len < 0` is the caller saying `str` is NUL-terminated.
        len = unsafe { strlen(str) } as c_int;
    }

    // If the same match is already present, don't add it.
    if !adup {
        for mut m in matches_from(first_match()) {
            let text = m.cp_str.data();
            // SAFETY: `str` is readable for `len` bytes and a match's
            // `cp_str` is a NUL-terminated allocation of its own length, so
            // the byte at `len` is inside it whenever the length test let
            // the read happen.
            let same = !m.is_original()
                && unsafe { strncmp(text, str, len as size_t) } == 0
                && (m.cp_str.len() as c_int <= len
                    || unsafe { *text.offset(len as isize) } as c_int == NUL);
            if !same {
                continue;
            }
            if is_nearest_active() && score > 0 && score < m.cp_score {
                m.cp_score = score;
            }
            if cptext_allocated {
                // SAFETY: as above.
                unsafe { free_cptext(cptext) };
            }
            return NOTDONE;
        }
    }

    // Remove any popup menu before changing the list of matches.
    // SAFETY: a completion is running, which is what the pum belongs to.
    unsafe { ins_compl_del_pum() };

    // SAFETY: `xcalloc` answers a fresh zeroed `compl_T` or aborts, and this
    // is the allocation the list takes over.
    let mut match_0 = unsafe { Cm::new(xcalloc(1, size_of::<compl_T>()) as *mut compl_T) };
    match_0.cp_number = if flags & CP_ORIGINAL_TEXT != 0 { 0 } else { -1 };
    // SAFETY: `str` is readable for `len` bytes -- the caller's promise.
    match_0.cp_str = unsafe { cbuf_to_string(str, len as size_t) };

    // The match's fname is `compl_curr_match`'s when it is an equal
    // string, else a copy of `fname` (with CP_FREE_FNAME so it is freed
    // later), else NULL.  -- Acevedo
    let curr = curr_match();
    let curr_fname = curr.map_or(ptr::null_mut(), |c| c.cp_fname);
    if !fname.is_null()
        && !curr_fname.is_null()
        // SAFETY: both are NUL-terminated file names.
        && unsafe { strcmp(fname, curr_fname) } == 0
    {
        match_0.cp_fname = curr_fname;
    } else if !fname.is_null() {
        // SAFETY: `fname` is NUL-terminated -- the caller's promise.
        match_0.cp_fname = unsafe { xstrdup(fname) };
        flags |= CP_FREE_FNAME;
    } else {
        match_0.cp_fname = ptr::null_mut();
    }
    match_0.cp_flags = flags;
    // SAFETY: a non-null `user_hl` is two `c_int`s, abbr then kind.
    let (abbr_hl, kind_hl) = unsafe {
        if user_hl.is_null() {
            (-1, -1)
        } else {
            (*user_hl, *user_hl.add(1))
        }
    };
    match_0.cp_user_abbr_hlattr = abbr_hl;
    match_0.cp_user_kind_hlattr = kind_hl;
    match_0.cp_score = score;
    match_0.cp_cpt_source_idx = cpt_sources().index();

    if !cptext.is_null() {
        for i in 0..CPT_COUNT as isize {
            // SAFETY: `cptext` is `CPT_COUNT` entries, each null or a
            // NUL-terminated string.
            let text = unsafe { *cptext.offset(i) };
            if text.is_null() {
                continue;
            }
            // SAFETY: as above.
            if unsafe { *text } as c_int != NUL {
                match_0.cp_text[i as usize] = if cptext_allocated {
                    text
                } else {
                    // SAFETY: as above.
                    unsafe { xstrdup(text) }
                };
            } else if cptext_allocated {
                // SAFETY: `cptext_allocated` says the string is ours now.
                unsafe { xfree(text.cast::<c_void>()) };
            }
        }
    }

    if !user_data.is_null() {
        // SAFETY: a non-null `user_data` is a live `typval_T`, which the
        // caller has handed over.
        match_0.cp_user_data = unsafe { *user_data };
    }

    // Link the new match after (FORWARD) or before (BACKWARD) the current
    // match in the list.
    let first = first_match();
    if first.is_none() {
        match_0.cp_prev = ptr::null_mut();
        match_0.cp_next = ptr::null_mut();
    } else if cot_fuzzy() && score != FUZZY_SCORE_NONE && compl_get_longest.get() {
        // The direction is ignored under `longest` + `fuzzy`, because
        // matches are inserted sorted by score.
        let mut first = first.expect("checked just above");
        let mut current = first.next();
        let mut prev = first;
        let mut inserted = false;
        while let Some(mut cur) = current.filter(|cur| *cur != first) {
            if cur.cp_score < score {
                match_0.cp_next = cur.raw();
                match_0.cp_prev = cur.cp_prev;
                if let Some(mut before) = cur.prev() {
                    before.cp_next = match_0.raw();
                }
                cur.cp_prev = match_0.raw();
                inserted = true;
                break;
            }
            prev = cur;
            current = cur.next();
        }
        if !inserted {
            prev.cp_next = match_0.raw();
            match_0.cp_prev = prev.raw();
            match_0.cp_next = first.raw();
            first.cp_prev = match_0.raw();
        }
    } else {
        // A non-empty list always has a current match, which upstream
        // dereferences here without checking.
        let curr = curr.expect("a non-empty match list has a current match");
        if dir == FORWARD {
            match_0.cp_next = curr.cp_next;
            match_0.cp_prev = curr.raw();
        } else {
            match_0.cp_next = curr.raw();
            match_0.cp_prev = curr.cp_prev;
        }
    }
    if let Some(mut next) = match_0.next() {
        next.cp_prev = match_0.raw();
    }
    match match_0.prev() {
        Some(mut prev) => prev.cp_next = match_0.raw(),
        // Nothing before it: it is the first match.
        None => compl_first_match.set(match_0.raw()),
    }
    compl_curr_match.set(match_0.raw());

    // Find the longest common string if still doing that.
    if compl_get_longest.get()
        && flags & CP_ORIGINAL_TEXT == 0
        && !cot_fuzzy()
        && !unsafe { ins_compl_preinsert_longest() }
        && !ctrl_x_mode_thesaurus()
    {
        // SAFETY: `match_0` is the node just linked in.
        unsafe { ins_compl_longest_match(match_0) };
    }
    OK
}

/// [`ins_compl_add`] for the original text: the first match every completion
/// starts with, taken from `compl_orig_text`.
///
/// # Safety
/// `compl_orig_text` holds the text being completed.
pub(crate) unsafe fn ins_compl_add_orig_text(flags: c_int) -> c_int {
    let text = compl_orig_text().data();
    let len = compl_orig_text().len() as c_int;
    // SAFETY: the caller's promise; the nulls say there is no file name, no
    // `cptext`, no user data and no highlight pair.
    unsafe {
        ins_compl_add(
            text,
            len,
            ptr::null_mut(),
            ptr::null(),
            false,
            ptr::null_mut(),
            kDirectionNotSet,
            flags,
            false,
            ptr::null(),
            FUZZY_SCORE_NONE,
        )
    }
}

/// Does `str[..len]` match `match_0`'s text, honouring its `CP_ICASE` /
/// `CP_EQUAL` flags?
///
/// # Safety
/// `str` is readable for `len` bytes.
pub(crate) unsafe fn ins_compl_equal(match_0: Cm, str: *mut c_char, len: size_t) -> bool {
    if match_0.cp_flags & CP_EQUAL != 0 {
        return true;
    }
    let text = match_0.cp_str.data();
    if match_0.cp_flags & CP_ICASE != 0 {
        // SAFETY: `str` has `len` readable bytes and `cp_str` is a
        // NUL-terminated allocation, so neither read runs off its end.
        return unsafe { strncasecmp(text, str, len) } == 0;
    }
    // SAFETY: as above.
    unsafe { strncmp(text, str, len) == 0 }
}

/// Shorten `compl_leader` to the longest prefix it shares with `match_0`, and
/// put that prefix in the buffer.
///
/// # Safety
/// A completion is running, so that the leader and the buffer text the
/// insert touches are the ones this match belongs to.
pub(crate) unsafe fn ins_compl_longest_match(match_0: Cm) {
    if compl_leader().is_unset() {
        // SAFETY: `cp_str` is this match's own string; a null arena asks
        // `copy_string` for a fresh allocation.
        let copy = unsafe { copy_string(match_0.cp_str, ptr::null_mut::<Arena>()) };
        compl_leader().set(copy);
        let had_match = cur_win().w_cursor.col > compl_col.get();
        // SAFETY: the leader is a NUL-terminated string, and a completion is
        // running -- the caller's promise.
        unsafe { ins_compl_longest_insert(compl_leader().data()) };
        if !had_match {
            // SAFETY: as above.
            unsafe { ins_compl_delete(false) };
        }
        compl_used_match.set(false);
        return;
    }

    let icase = match_0.cp_flags & CP_ICASE != 0;
    let mut p = compl_leader().data();
    let mut s = match_0.cp_str.data();
    loop {
        // SAFETY: `p` walks the NUL-terminated leader and stops at its NUL.
        if unsafe { *p } as c_int == NUL {
            break;
        }
        // SAFETY: `p` and `s` each point at a character of a NUL-terminated
        // string; the leader is a prefix of every match that got this far,
        // so `s` has not run out either.
        let (c1, c2) = unsafe { (utf_ptr2char(p), utf_ptr2char(s)) };
        let differ = if icase {
            mb_tolower(c1) != mb_tolower(c2)
        } else {
            c1 != c2
        };
        if differ {
            break;
        }
        // SAFETY: as above -- a character is `utfc_ptr2len` bytes, so the
        // step lands on the next character or on the NUL.
        p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
        // SAFETY: as above.
        s = unsafe { s.offset(utfc_ptr2len(s) as isize) };
    }

    // SAFETY: `p` is inside the leader.
    if unsafe { *p } as c_int != NUL {
        // SAFETY: `p` is inside the leader, which is this module's own
        // writable allocation.
        unsafe { *p = NUL as c_char };
        let leader = compl_leader().value();
        // SAFETY: `p` and the leader's bytes are the same allocation.
        let len = unsafe { p.offset_from(leader.data()) } as size_t;
        compl_leader().set(String_0::from_raw_parts(leader.data(), len));
        let had_match = cur_win().w_cursor.col > compl_col.get();
        // SAFETY: as in the branch above.
        unsafe { ins_compl_longest_insert(compl_leader().data()) };
        if !had_match {
            // SAFETY: as above.
            unsafe { ins_compl_delete(false) };
        }
    }
    compl_used_match.set(false);
}

/// Add every string of an expansion's `matches` array, then free the array.
///
/// # Safety
/// `matches` is `num_matches` NUL-terminated strings this call takes over.
pub(crate) unsafe fn ins_compl_add_matches(
    num_matches: c_int,
    matches: *mut *mut c_char,
    icase: c_int,
) {
    let mut dir = compl_direction.get();
    let flags = CP_FAST | if icase != 0 { CP_ICASE } else { 0 };
    for i in 0..num_matches as isize {
        // SAFETY: the caller's array holds `num_matches` strings.
        let text = unsafe { *matches.offset(i) };
        // SAFETY: `text` is NUL-terminated, which is what `len < 0` asks
        // for; the nulls say there is no file name, no `cptext`, no user
        // data and no highlight pair.
        let add_r = unsafe {
            ins_compl_add(
                text,
                -1,
                ptr::null_mut(),
                ptr::null(),
                false,
                ptr::null_mut(),
                dir,
                flags,
                false,
                ptr::null(),
                FUZZY_SCORE_NONE,
            )
        };
        if add_r == FAIL {
            break;
        }
        if add_r == OK {
            dir = FORWARD;
        }
    }
    // SAFETY: the caller handed the array over.
    unsafe { free_wild(num_matches, matches) };
}

/// Close the list into a ring; returns the number of matches after the first.
pub(crate) fn ins_compl_make_cyclic() -> c_int {
    let Some(mut first) = first_match() else {
        return 0;
    };
    let mut m = first;
    let mut count = 0;
    while let Some(next) = m.next().filter(|next| !next.is_first()) {
        m = next;
        count += 1;
    }
    m.cp_next = first.raw();
    first.cp_prev = m.raw();
    count
}

/// Open the ring back into a NULL-terminated list.
pub(crate) fn ins_compl_make_linear() {
    let Some(mut first) = first_match() else {
        return;
    };
    let Some(mut last) = first.prev() else {
        return;
    };
    last.cp_next = ptr::null_mut();
    first.cp_prev = ptr::null_mut();
}

// The four link accessors `mergesort_list` walks the list through, and the two
// score comparators it orders it by.  All six are held as function pointers,
// so they keep their C ABI.

/// # Safety
/// `node` is a live `compl_T`.
pub(crate) unsafe fn cp_get_next(node: *mut c_void) -> *mut c_void {
    // SAFETY: the caller's live node.
    unsafe { (*(node as *mut compl_T)).cp_next as *mut c_void }
}

/// # Safety
/// `node` is a live `compl_T` and `next` is one or null.
pub(crate) unsafe fn cp_set_next(node: *mut c_void, next: *mut c_void) {
    // SAFETY: the caller's live node.
    unsafe { (*(node as *mut compl_T)).cp_next = next as *mut compl_T };
}

/// # Safety
/// `node` is a live `compl_T`.
pub(crate) unsafe fn cp_get_prev(node: *mut c_void) -> *mut c_void {
    // SAFETY: the caller's live node.
    unsafe { (*(node as *mut compl_T)).cp_prev as *mut c_void }
}

/// # Safety
/// `node` is a live `compl_T` and `prev` is one or null.
pub(crate) unsafe fn cp_set_prev(node: *mut c_void, prev: *mut c_void) {
    // SAFETY: the caller's live node.
    unsafe { (*(node as *mut compl_T)).cp_prev = prev as *mut compl_T };
}

/// Highest fuzzy score first.
///
/// # Safety
/// `a` and `b` are live `compl_T`s.
pub(crate) unsafe fn cp_compare_fuzzy(a: *const c_void, b: *const c_void) -> c_int {
    // SAFETY: the caller's two live nodes.
    let (score_a, score_b) = unsafe {
        (
            (*(a as *const compl_T)).cp_score,
            (*(b as *const compl_T)).cp_score,
        )
    };
    score_b.cmp(&score_a) as c_int
}

/// Nearest to the cursor first; unscored matches compare equal to everything.
///
/// # Safety
/// `a` and `b` are live `compl_T`s.
pub(crate) unsafe fn cp_compare_nearest(a: *const c_void, b: *const c_void) -> c_int {
    // SAFETY: the caller's two live nodes.
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
///
/// # Safety
/// `a` and `b` are `c_int` indices in range of `compl_fuzzy_scores`, which
/// is what `qsort` hands back out of the array it was given.
pub(crate) unsafe extern "C" fn compare_scores(a: *const c_void, b: *const c_void) -> c_int {
    // SAFETY: the caller's two indices.
    let (idx_a, idx_b) = unsafe { (*(a as *const c_int), *(b as *const c_int)) };
    let scores = compl_fuzzy_scores.get();
    // SAFETY: both indices are in range of `compl_fuzzy_scores`.
    let (score_a, score_b) = unsafe {
        (
            *scores.offset(idx_a as isize),
            *scores.offset(idx_b as isize),
        )
    };
    if score_a == score_b {
        idx_a.cmp(&idx_b) as c_int
    } else {
        score_b.cmp(&score_a) as c_int
    }
}

/// Score every match against the leader (or, with no leader, against the
/// original text).
///
/// # Safety
/// A completion is running, so the leader and the original text belong to
/// the list being scored.
pub(crate) unsafe fn set_fuzzy_score() {
    let Some(first) = first_match() else {
        return;
    };

    // Determine the pattern to match against.
    let leader = compl_leader().value();
    let use_leader = !leader.data().is_null() && !leader.is_empty();
    let mut pattern: *mut c_char = ptr::null_mut();
    if use_leader {
        // Clear the leader cache once before the loop; the pattern is
        // then computed per match, since each may have its own startcol.
        clear_adjusted_leader();
    } else {
        let orig = compl_orig_text().value();
        if orig.data().is_null() || orig.is_empty() {
            return;
        }
        pattern = orig.data();
    }

    for mut comp in matches_from(Some(first)) {
        if use_leader {
            // SAFETY: `comp` is a live node, and a completion is running.
            pattern = unsafe { get_leader_for_startcol(comp, true) }.data();
        }
        // SAFETY: both strings are NUL-terminated.
        comp.cp_score = unsafe { fuzzy_match_str(comp.cp_str.data(), pattern) };
    }
}

/// [`mergesort_list`] over an opened match list, with this module's four link
/// accessors.
///
/// # Safety
/// `head` is null or the first node of a match list [`ins_compl_make_linear`]
/// has opened.
unsafe fn sort_nodes(head: *mut compl_T, compare: MergeSortCompareFunc) -> *mut compl_T {
    let head = head.cast::<c_void>();
    let get_next: MergeSortGetFunc = Some(cp_get_next);
    let set_next: MergeSortSetFunc = Some(cp_set_next);
    let get_prev: MergeSortGetFunc = Some(cp_get_prev);
    let set_prev: MergeSortSetFunc = Some(cp_set_prev);
    // SAFETY: the four accessors are this module's own and read nothing but
    // the two links, and `head` is the caller's opened list.
    let sorted = unsafe { mergesort_list(head, get_next, set_next, get_prev, set_prev, compare) };
    sorted.cast::<compl_T>()
}

/// Sort the match list with `compare`, leaving the node holding the leader
/// (the original text) where it is.
///
/// # Safety
/// `compare` is `Some` and orders two live `compl_T`s.
pub(crate) unsafe fn sort_compl_match_list(compare: MergeSortCompareFunc) {
    let Some(mut first) = first_match() else {
        return;
    };
    if is_first_match(first.cp_next) {
        return;
    }

    let comp = first.prev();
    ins_compl_make_linear();
    if compl_shows_dir_forward() {
        // The leader sits at the head; sort everything after it.
        if let Some(mut next) = first.next() {
            next.cp_prev = ptr::null_mut();
        }
        // SAFETY: the list has just been opened, and `compare` is the
        // caller's.
        let sorted = unsafe { sort_nodes(first.cp_next, compare) };
        first.cp_next = sorted;
        if let Some(mut next) = first.next() {
            next.cp_prev = first.raw();
        }
    } else {
        // The leader sits at the tail; sort everything before it.
        // Upstream dereferences both links here without checking.
        let mut comp = comp.expect("a cyclic list's head has a predecessor");
        let mut before = comp.prev().expect("the leader is not the only match");
        before.cp_next = ptr::null_mut();
        // SAFETY: as above.
        compl_first_match.set(unsafe { sort_nodes(first.raw(), compare) });
        let mut tail = first_match().expect("the sort answers a non-empty list");
        while let Some(next) = tail.next() {
            tail = next;
        }
        tail.cp_next = comp.raw();
        comp.cp_prev = tail.raw();
    }
    ins_compl_make_cyclic();
}

/// Free one match and everything hanging off it.
///
/// # Safety
/// `match_0` is already unlinked from the list, or the whole list is going
/// at once, and nothing else holds it.
pub(crate) unsafe fn ins_compl_item_free(mut match_0: Cm) {
    // SAFETY: the text is this match's own allocation.
    unsafe { xfree(match_0.cp_str.data().cast::<c_void>()) };
    match_0.cp_str = String_0::NULL;
    if match_0.cp_flags & CP_FREE_FNAME != 0 {
        // SAFETY: `CP_FREE_FNAME` says the file name is this match's own.
        unsafe { xfree(match_0.cp_fname.cast::<c_void>()) };
    }
    // The addresses come off the raw pointer rather than through `DerefMut`,
    // so no borrow of the node is live while it is freed.
    let raw = match_0.raw();
    // SAFETY: the `cptext` strings, the user data and the node itself are
    // all this match's own, and the caller has unlinked it.
    unsafe {
        free_cptext((&raw mut (*raw).cp_text).cast::<*mut c_char>());
        tv_clear(&raw mut (*raw).cp_user_data);
        xfree(raw.cast::<c_void>());
    }
}

/// Free the whole match list and the pattern and leader that built it.
///
/// # Safety
/// Nothing outside the list holds one of its nodes.
pub(crate) unsafe fn ins_compl_free() {
    compl_pattern().clear();
    compl_leader().clear();

    if first_match().is_none() {
        return;
    }

    // SAFETY: a completion is running, which is what the pum belongs to.
    unsafe { ins_compl_del_pum() };
    pum_clear();

    compl_curr_match.set(compl_first_match.get());
    loop {
        let m = curr_match().expect("the walk stops before it runs off the list");
        compl_curr_match.set(m.cp_next);
        // SAFETY: `m` is off the walk before it is freed, and the whole list
        // is going.
        unsafe { ins_compl_item_free(m) };
        match curr_match() {
            Some(next) if !next.is_first() => {}
            _ => break,
        }
    }
    compl_curr_match.set(ptr::null_mut());
    compl_first_match.set(ptr::null_mut());
    compl_shown_match.set(ptr::null_mut());
    compl_old_match.set(ptr::null_mut());
}

/// Reset everything a completion left behind, without freeing the list.
///
/// # Safety
/// The editor exists; `v:completed_item` is set from this thread.
pub unsafe fn ins_compl_clear() {
    compl_cont_status.set(0);
    compl_started.set(false);
    compl_matches.set(0);
    compl_selected_item.set(-1);
    compl_ins_end_col.set(0);
    compl_curr_win.set(None);
    compl_curr_buf.set(None);
    compl_pattern().clear();
    compl_leader().clear();
    edit_submode_extra.set(ptr::null_mut());
    compl_orig_extmarks().clear();
    compl_orig_text().clear();
    compl_enter_selects.set(false);
    cpt_sources().clear();
    compl_autocomplete.set(false);
    compl_from_nonkeyword.set(false);
    compl_num_bests.set(0);
    // SAFETY: a fresh locked dict, which `set_vim_var_dict` takes over.
    unsafe { set_vim_var_dict(Vv::CompletedItem, tv_dict_alloc_lock(VarLock::Fixed)) };
}

/// Score the matches and, unless `'completeopt'` says `nosort`, reorder them.
///
/// # Safety
/// A completion is running.
pub(crate) unsafe fn ins_compl_fuzzy_sort() {
    let cur_cot_flags = completeopt_flags();

    // SAFETY: a completion is running -- the caller's promise.
    unsafe { set_fuzzy_score() };
    if cur_cot_flags & kOptCotFlagNosort != 0 {
        return;
    }
    // SAFETY: `cp_compare_fuzzy` reads two live nodes' scores.
    unsafe { sort_compl_match_list(Some(cp_compare_fuzzy)) };

    // Sorting reorders the items, so the shown one has to be reset.
    if cur_cot_flags & (kOptCotFlagNoinsert | kOptCotFlagNoselect) != kOptCotFlagNoinsert {
        return;
    }
    let first = first_match();
    let unselected = if compl_shows_dir_forward() {
        first
    } else {
        first.and_then(Cm::prev)
    };
    if shown_match() == unselected {
        return;
    }
    let next = if !compl_autocomplete.get() && compl_shows_dir_forward() {
        first.map_or(ptr::null_mut(), |first| first.cp_next)
    } else {
        first.map_or(ptr::null_mut(), Cm::raw)
    };
    compl_shown_match.set(next);
}

/// Number the matches around `compl_curr_match`, in the direction the
/// completion is running.
pub(crate) fn ins_compl_update_sequence_numbers() {
    let mut number = 0;
    // Upstream dereferences `compl_curr_match` here without checking.
    let curr = curr_match().expect("a running completion has a current match");
    let mut m;
    if compl_dir_forward() {
        // Search backwards for the first match with a number.
        m = curr.prev();
        while let Some(node) = m.filter(|node| !node.is_first()) {
            if node.cp_number != -1 {
                number = node.cp_number;
                break;
            }
            m = node.prev();
        }
        if let Some(node) = m {
            // Go up and assign all numbers which are not assigned yet.
            let mut next = node.next();
            while let Some(mut node) = next.filter(|node| node.cp_number == -1) {
                number += 1;
                node.cp_number = number;
                next = node.next();
            }
        }
    } else {
        debug_assert!(compl_direction.get() == BACKWARD);
        // Search forwards (upwards) for the first match with a number.
        m = curr.next();
        while let Some(node) = m.filter(|node| !node.is_first()) {
            if node.cp_number != -1 {
                number = node.cp_number;
                break;
            }
            m = node.next();
        }
        if let Some(node) = m {
            // Go down and assign all numbers which are not assigned yet.
            let mut prev = node.prev();
            while let Some(mut node) = prev.filter(|node| node.cp_number == -1) {
                number += 1;
                node.cp_number = number;
                prev = node.prev();
            }
        }
    }
}

/// Drop every match the current `'complete'` source contributed, so it can be
/// re-run (`refresh: 'always'`).
///
/// # Safety
/// A completion is running and nothing outside the list holds one of the
/// nodes this source contributed.
pub(crate) unsafe fn remove_old_matches() {
    let mut shown_match_removed = false;
    // Upstream dereferences `compl_first_match` here without checking.
    let head = first_match().expect("a refresh runs on a non-empty match list");
    let forward = head.cp_cpt_source_idx < 0;

    if cpt_sources().index() < 0 {
        return;
    }

    compl_direction.set(if forward { FORWARD } else { BACKWARD });
    compl_shows_dir.set(compl_direction.get());

    // Under `'completeopt'` `fuzzy` the items are not in source order, so
    // they have to be removed one by one rather than as a run.
    let mut current = first_match();
    while let Some(cur) = current {
        if cur.cp_cpt_source_idx != cpt_sources().index() {
            current = cur.next();
            continue;
        }
        let to_delete = cur;
        if !shown_match_removed && shown_match() == Some(cur) {
            shown_match_removed = true;
        }
        current = cur.next();

        if Some(to_delete) == first_match() {
            // Head.
            compl_first_match.set(to_delete.cp_next);
            if let Some(mut head) = first_match() {
                head.cp_prev = ptr::null_mut();
            }
        } else if let Some(mut prev) = to_delete.prev() {
            match to_delete.next() {
                // Tail.
                None => prev.cp_next = ptr::null_mut(),
                // Middle.
                Some(mut next) => {
                    prev.cp_next = next.raw();
                    next.cp_prev = prev.raw();
                }
            }
        }
        // SAFETY: `to_delete` is unlinked above and the walk has moved past
        // it.
        unsafe { ins_compl_item_free(to_delete) };
    }

    if shown_match_removed {
        if forward {
            compl_shown_match.set(compl_first_match.get());
        } else if let Some(mut last) = first_match() {
            // The last node carries the prefix being completed.
            while let Some(next) = last.next() {
                last = next;
            }
            compl_shown_match.set(last.raw());
        }
    }

    compl_curr_match.set(compl_first_match.get());
    let mut current = first_match();
    while let Some(cur) = current {
        let before = if forward {
            cur.cp_cpt_source_idx < cpt_sources().index()
        } else {
            cur.cp_cpt_source_idx > cpt_sources().index()
        };
        if !before {
            break;
        }
        compl_curr_match.set(if forward { cur.raw() } else { cur.cp_next });
        current = cur.next();
    }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
