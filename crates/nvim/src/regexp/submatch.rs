//! What the last match captured, for the code that runs *inside* a
//! substitution: `submatch()` and `submatch(n, 1)`, and the list a `\=`
//! expression's function is handed as its argument.
//!
//! These read `rsm`, not `rex`. A `\=` expression may run a search or a
//! substitution of its own, which takes `rex` over; `rsm` is the snapshot
//! [`super::substitute`] takes of the outermost match before evaluating, so
//! that `submatch()` keeps answering about the substitution the user wrote.
//! `can_f_submatch` is what says the snapshot is live.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::memory::XString;
use core::ffi::{c_char, c_int};

use super::{
    LineOrigin, RegMMatch, RegMatch, RegSubMatch, Rex, can_f_submatch, reg_line, reg_line_len, rsm,
};
use crate::eval::typval::{ListRef, SL_SIZE, tv_list_alloc};
use crate::strings::xstrnsave;
use crate::types::{ColNr, LineNr, ListItem, TypVal, UserFunc, VarLock};
use crate::winlayer::Live;

/// The snapshot `submatch()` answers about.
///
/// Obtained once with [`Rsm::acquire`] and passed around by value, for the
/// same reason [`Rex`] is: `RegSubMatch` holds two pointers into the
/// caller's match structures, and copying the whole thing out of the cell
/// would leave a second holder of them. The handle names the cell instead —
/// every accessor reads one field through it and nothing hands out a
/// reference into it.
#[derive(Clone, Copy)]
pub(crate) struct Rsm(*mut RegSubMatch);

impl Rsm {
    /// The snapshot of the substitution being evaluated.
    ///
    /// # Safety
    ///
    /// `can_f_submatch` must say a snapshot is live, and it must stay live
    /// for as long as the handle does — [`super::substitute`] sets it around
    /// one evaluation and puts the outer one back afterwards. Nothing else
    /// may hold a reference into the cell meanwhile.
    ///
    /// The text a string match ran over is part of that: the snapshot holds
    /// the capture spans' base, so [`Rsm::line`] borrows it and the caller
    /// of [`super::vim_regsub`] keeps it alive for the whole evaluation.
    #[inline(always)]
    pub(crate) unsafe fn acquire() -> Rsm {
        Rsm(rsm.ptr())
    }

    /// The string match the snapshot is about, or null for a buffer match.
    #[inline(always)]
    pub(crate) fn match_(self) -> *mut RegMatch {
        // SAFETY: the handle is a claim that the snapshot is live.
        unsafe { (*self.0).sm_match }
    }

    /// The buffer match it is about — meaningful only when [`Rsm::match_`]
    /// is null.
    #[inline(always)]
    pub(crate) fn mmatch(self) -> *mut RegMMatch {
        // SAFETY: as `match_`.
        unsafe { (*self.0).sm_mmatch }
    }

    /// The string a string match ran over, whose bytes its capture spans
    /// index. Empty for a buffer match, which names lines instead.
    #[inline(always)]
    pub(crate) fn line(self) -> &'static [u8] {
        // SAFETY: holding the handle is the claim that the snapshot is live
        // and the text with it -- see [`Rsm::acquire`].
        unsafe { cstr::bytes_at_or_empty((*self.0).sm_line) }
    }

    /// The buffer line the snapshot's line 0 sits on.
    #[inline(always)]
    pub(crate) fn firstlnum(self) -> LineNr {
        // SAFETY: as `match_`.
        unsafe { (*self.0).sm_firstlnum }
    }

    /// The last line it reaches, relative to [`Rsm::firstlnum`].
    #[inline(always)]
    pub(crate) fn maxline(self) -> LineNr {
        // SAFETY: as `match_`.
        unsafe { (*self.0).sm_maxline }
    }

    /// Whether the match treated `\n` as an ordinary character.
    #[inline(always)]
    pub(crate) fn line_lbr(self) -> bool {
        // SAFETY: as `match_`.
        unsafe { (*self.0).sm_line_lbr != 0 }
    }
}

/// The text of the submatch line `lnum` lines into the match `submatch()`
/// and a `\=` expression see.
pub(crate) fn reg_getline_submatch(rex: Rex, lnum: LineNr) -> *mut c_char {
    reg_line(rex, lnum, LineOrigin::Submatch)
}

/// Its length.
pub(crate) fn reg_getline_submatch_len(rex: Rex, lnum: LineNr) -> ColNr {
    reg_line_len(rex, lnum, LineOrigin::Submatch)
}

/// Fill `argv[argskip]` — a ten-item static list — with the submatches, and
/// report how many arguments the call now has.
///
/// This is the `fe_argv_func` a `\=` expression's function is called
/// through, so it runs before the function body sees its arguments. A
/// function that does not take a submatches argument gets none: the list
/// stays as the caller left it, which is what tells [`super::substitute`]
/// there is nothing to free.
///
/// # Safety
///
/// `func` must point at a live `UserFunc`, unaliased for the call.
pub(crate) unsafe fn fill_submatch_list(
    argv: &[TypVal],
    argskip: usize,
    func: *mut UserFunc,
) -> usize {
    // `argv[argskip]` holds the caller's own list, which it keeps alive
    // across the call.
    let listarg = &argv[argskip];
    // SAFETY: the caller's promise -- a live function.
    let declared = unsafe { (*func).uf_args.ga_len };
    if unsafe { (*func).uf_varargs } == 0 && declared as usize <= argskip {
        return argskip;
    }

    // The list is the caller's own storage; it starts empty and gets one
    // item per capture.
    let list = listarg.list_or_null();
    // SAFETY: the running string match is the caller's structure.
    let match_ = unsafe { Live::new(Rsm::acquire().match_()) };
    // SAFETY: the slot holds the caller's list, which nothing else names.
    let items = unsafe { &mut (*list).lv_items };
    items.reserve_exact(SL_SIZE);
    // SAFETY: the running string match is the caller's structure.
    let line = unsafe { Rsm::acquire() }.line();
    for i in 0..SL_SIZE {
        let text = match match_.group_bytes(i, line) {
            None => core::ptr::null_mut(),
            // SAFETY: a borrow of the line the match ran over.
            Some(bytes) => unsafe { xstrnsave(bytes.as_ptr().cast(), bytes.len()) },
        };
        items.push(ListItem {
            li_tv: TypVal::String(text),
            li_lock: VarLock::Fixed,
        });
    }
    argskip + 1
}

/// The text capture `no` matched, as an allocated string the caller owns.
/// Null outside a substitution and for a capture that did not participate.
///
/// A buffer match's capture can span lines, in which case the breaks come
/// back as newlines. Upstream walked the lines twice -- once to measure and
/// allocate, once to copy -- and the two rounds had to be kept in step by
/// hand; an [`XString`] grows as the walk goes, so there is one round.
pub(crate) fn reg_submatch(no: c_int) -> Option<XString> {
    if !can_f_submatch.get() || no < 0 {
        return None;
    }
    let no = no as usize;
    // SAFETY: guarded by `can_f_submatch`, which is only set while `rsm`
    // describes a live match — so the context still names the buffer the
    // submatch lines are read from; `no` is bounds-checked against the ten
    // capture slots by its caller (`submatch()` rejects anything else).
    let rex = unsafe { Rex::acquire() };

    // A string match has no lines to cross.
    let snapshot = unsafe { Rsm::acquire() };
    if !snapshot.match_().is_null() {
        // SAFETY: the snapshot names the string match that is running.
        let match_ = unsafe { Live::new(snapshot.match_()) };
        return match_
            .group_bytes(no, snapshot.line())
            .map(XString::from_bytes);
    }

    // SAFETY: the snapshot names the buffer match that is running.
    let mmatch = unsafe { Live::new(snapshot.mmatch()) };
    let mut lnum = mmatch.startpos[no].lnum;
    if lnum < 0 || mmatch.endpos[no].lnum < 0 {
        return None;
    }
    let line = reg_getline_submatch(rex, lnum);
    if line.is_null() {
        // Anti-crash check; cannot happen.
        return None;
    }
    let (scol, ecol) = (mmatch.startpos[no].col, mmatch.endpos[no].col);
    let mut text = XString::new();
    if mmatch.endpos[no].lnum == lnum {
        // Within one line: from the start column to the end one.
        text.push_bytes(submatch_head(rex, lnum, scol, (ecol - scol) as usize));
        return Some(text);
    }

    // The rest of the start line, then whole lines, then the head of the end
    // line. Each break travels as a newline.
    let rest = (reg_getline_submatch_len(rex, lnum) - scol) as usize;
    text.push_bytes(submatch_head(rex, lnum, scol, rest));
    text.push_byte(b'\n');
    lnum += 1;
    while lnum < mmatch.endpos[no].lnum {
        let whole = reg_getline_submatch_len(rex, lnum) as usize;
        text.push_bytes(submatch_head(rex, lnum, 0, whole));
        text.push_byte(b'\n');
        lnum += 1;
    }
    text.push_bytes(submatch_head(rex, lnum, 0, ecol as usize));
    Some(text)
}

/// `len` bytes of submatch line `lnum`, starting at column `from` and
/// stopping at the line's terminator whichever comes first.
///
/// A line the match names but the buffer no longer has answers no bytes,
/// which is upstream's anti-crash check moved inside the walk.
fn submatch_head(rex: Rex, lnum: LineNr, from: ColNr, len: usize) -> &'static [u8] {
    let line = reg_getline_submatch(rex, lnum);
    if line.is_null() {
        return &[];
    }
    // SAFETY: the line `rex`'s match names, NUL-terminated; `prefix_at`
    // never reads past that terminator, and `from` is a column in it.
    unsafe { cstr::prefix_at(line.offset(from as isize), len) }
}

/// [`reg_submatch`] as one list item per line, which is what
/// `submatch(no, 1)` returns. Unlike [`reg_submatch`] this keeps NULs in the
/// text apart from the line breaks, because each line is its own item.
pub(crate) fn reg_submatch_list(no: c_int) -> Option<ListRef> {
    if !can_f_submatch.get() || no < 0 {
        return None;
    }
    let no = no as usize;
    // SAFETY: as [`reg_submatch`].
    let rex = unsafe { Rex::acquire() };

    // A string match is one item.
    let snapshot = unsafe { Rsm::acquire() };
    if !snapshot.match_().is_null() {
        // SAFETY: the snapshot names the string match that is running.
        let match_ = unsafe { Live::new(snapshot.match_()) };
        let bytes = match_.group_bytes(no, snapshot.line())?;
        let list = tv_list_alloc(1);
        // SAFETY: a borrow of the line the match ran over.
        unsafe { (*list.as_ptr()).push_string(bytes.as_ptr().cast(), bytes.len() as isize) };
        return Some(list);
    }

    // SAFETY: the snapshot names the buffer match that is running.
    let mmatch = unsafe { Live::new(snapshot.mmatch()) };
    let slnum = mmatch.startpos[no].lnum;
    let elnum = mmatch.endpos[no].lnum;
    if slnum < 0 || elnum < 0 {
        return None;
    }
    let scol = mmatch.startpos[no].col;
    let ecol = mmatch.endpos[no].col;

    let list = tv_list_alloc((elnum - slnum + 1) as isize);
    let into = list.as_ptr();
    let s = unsafe { reg_getline_submatch(rex, slnum).offset(scol as isize) };
    if slnum == elnum {
        unsafe { (*into).push_string(s, (ecol - scol) as isize) };
    } else {
        // A negative length means "to the end of the line".
        unsafe { (*into).push_string(s, -1) };
        for lnum in slnum + 1..elnum {
            unsafe { (*into).push_string(reg_getline_submatch(rex, lnum), -1) };
        }
        unsafe { (*into).push_string(reg_getline_submatch(rex, elnum), ecol as isize) };
    }
    Some(list)
}
