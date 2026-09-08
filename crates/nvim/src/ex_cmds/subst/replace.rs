//! Building one replacement, and putting the rebuilt line in the buffer.
//!
//! The replacement is produced twice: [`build_replacement`] first calls
//! `vim_regsub_multi` with no destination, purely to learn how long the
//! result will be, then again to write it -- the "measure or write" shape,
//! and the reason `\=` expressions are evaluated twice per match.  Both
//! passes run under `textlock`, so the expression cannot change the text
//! under the match.
//!
//! [`commit_line`] is the other half: the rebuilt line only reaches the
//! buffer once there is no further match on it, because replacing it earlier
//! would change what the pattern sees.
//!
//! [`SubLine`] is the text both halves work on.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::exec::{Sub, SubArgs, save_undo_once};
use super::subflags;
use crate::change::{appended_lines, changed_bytes, deleted_lines};
use crate::ex_cmds::sub_nsubs;
use crate::ex_cmds::{
    CAR, LineData, REGSUB_BACKSLASH, REGSUB_COPY, REGSUB_MAGIC, kExtmarkNOOP, kExtmarkUndo,
};
use crate::ex_eval::aborting;
use crate::extmark::extmark_splice;
use crate::guard::Lock;
use crate::mark::mark_adjust;
use crate::mbyte::cluster_len;
use crate::memline::{Lines, ml_append, ml_delete, ml_get_len, ml_replace};
use crate::option::magic_isset;
use crate::pos::MAXLNUM;
use crate::regexp::vim_regsub_multi;
use crate::types::{BCount, ColNr, LPos, LineNr, NUL};
use crate::undo::{u_inssub, u_savedel, u_savesub};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int};

/// One line of `:s` text the command owns.
///
/// `:s` builds the result for one line off to the side and only puts it in
/// the buffer once that line has no further match; the finished text then
/// becomes the *old* text the next line's matches copy from.  Upstream is
/// two `xmalloc`ed pointers -- `new_start` with its `new_start_len`, and
/// `sub_firstline` -- a hand-rolled growth policy, and a hand-over that
/// aliases one pointer into the other and frees what it replaced.  Here both
/// fields hold a `SubLine`, the hand-over is a move, and neither is freed by
/// hand.
///
/// The NUL a buffer line has after it is part of the storage and never part
/// of [`SubLine::bytes`]: `ml_replace`, `ml_append` and `vim_regsub_multi`
/// all read a C string, while everything on this side wants the bytes
/// without it.  The text holds no NUL of its own -- a buffer line cannot
/// contain one, and [`SubLine::append_written`] cuts the replacement at the
/// first one the expansion wrote -- so the terminator is unambiguous.
#[derive(Debug)]
pub(super) struct SubLine(Vec<u8>);

impl Default for SubLine {
    /// The empty line, which is upstream's `xstrdup("")`.
    fn default() -> SubLine {
        SubLine(vec![NUL as u8])
    }
}

impl SubLine {
    /// A copy of line `lnum` of the current buffer, so that the text cannot
    /// be taken away by a screen update or a multi-line match.
    pub(super) fn from_line(lnum: LineNr) -> SubLine {
        let mut lines = Lines::current();
        let text = lines.line(lnum);
        let mut bytes = Vec::with_capacity(text.len() + 1);
        bytes.extend_from_slice(text);
        bytes.push(NUL as u8);
        SubLine(bytes)
    }

    /// The text, terminator excluded.
    pub(super) fn bytes(&self) -> &[u8] {
        &self.0[..self.0.len() - 1]
    }

    /// How many bytes the text holds, terminator excluded.
    pub(super) fn len(&self) -> usize {
        self.0.len() - 1
    }

    /// The text as the NUL-terminated string a pointer-taking callee wants.
    pub(super) fn as_ptr(&self) -> *mut c_char {
        self.0.as_ptr().cast::<c_char>().cast_mut()
    }

    /// Append `bytes`.
    fn push_bytes(&mut self, bytes: &[u8]) {
        let end = self.0.len() - 1;
        self.0.truncate(end);
        self.0.extend_from_slice(bytes);
        self.0.push(NUL as u8);
    }

    /// Append what a pointer-taking callee writes past the end.
    ///
    /// `write` is handed a pointer to `room + 1` zeroed bytes and the `room`
    /// it was promised; the line then ends at the first NUL it left behind.
    /// That is `vim_regsub_multi`'s two-pass contract: the caller has already
    /// measured, so `room` is a length the callee agreed to, and the spare
    /// byte past it covers the one case where the two passes disagree --
    /// upstream's own buffer is `xcalloc`ed and zero-filled on every growth
    /// for the same reason, because an expansion that runs out of room or
    /// walks into a changed line leaves the text *unterminated*.
    fn append_written(&mut self, room: usize, write: impl FnOnce(*mut c_char, c_int)) {
        let base = self.0.len() - 1;
        self.0.resize(base + room + 1, NUL as u8);
        let room = c_int::try_from(room).unwrap_or(c_int::MAX);
        write(self.0[base..].as_mut_ptr().cast::<c_char>(), room);
        let written = self.0[base..]
            .iter()
            .position(|&byte| byte == NUL as u8)
            .expect("the reserved bytes were zeroed, so one of them terminates");
        self.0.truncate(base + written + 1);
    }

    /// Drop the byte at `at`, moving the rest of the text down over it.
    fn remove(&mut self, at: usize) {
        self.0.remove(at);
    }

    /// Write the terminator over byte `at`, so that [`SubLine::as_ptr`]
    /// answers the text before it and nothing else.
    ///
    /// The caller must follow with [`SubLine::drop_head`]: until it does, the
    /// line is only that head, and the tail past the terminator is text this
    /// type's invariant says is not there.
    fn terminate_at(&mut self, at: usize) {
        self.0[at] = NUL as u8;
    }

    /// Drop the first `n` bytes, keeping what follows as the whole text.
    fn drop_head(&mut self, n: usize) {
        self.0.drain(..n);
    }
}

/// The `vim_regsub_multi` flags this command uses, without the copy bit.
fn regsub_flags() -> c_int {
    REGSUB_BACKSLASH as c_int
        | if magic_isset() {
            REGSUB_MAGIC as c_int
        } else {
            0 as c_int
        }
}

/// Turn every CTRL-M in the freshly written replacement into a real line
/// break, and halve the backslashes that protect one.
///
/// That is Vi compatible: a line break can be avoided by preceding the
/// CTRL-M with a backslash, and to insert a backslash they must be doubled
/// in the replacement and are halved here.
///
/// `at` is where the replacement starts in the rebuilt line.
///
/// # Safety
/// Main thread; `st.new_line` must hold the rebuilt line.
unsafe fn split_carriage_returns(st: &mut Sub, at: usize) {
    let mut line = st
        .new_line
        .take()
        .expect("caller's contract -- a rebuilt line");
    let mut at = at;
    while at < line.len() {
        let here = line.bytes()[at];
        if here == b'\\' && at + 1 < line.len() {
            // Remove the backslash, and correct the byte count that
            // extmark_splice() will be given.
            st.sublen -= 1;
            line.remove(at);
            at += 1;
        } else if here as c_int == CAR {
            // Prepare for undo of the line about to be split.
            if u_inssub(st.lnum).is_ok() {
                // The text up to the CR becomes a line of its own, and is
                // then cut off the front of what is being rebuilt.
                line.terminate_at(at);
                // SAFETY: `ml_append` copies the `at + 1` bytes just
                // terminated, and `lnum` is a line of the buffer.
                let _ = unsafe {
                    ml_append(st.lnum - 1 as LineNr, line.as_ptr(), at as ColNr + 1, false)
                };
                line.drop_head(at + 1);
                // SAFETY: the lines below `lnum` all move down by one.
                unsafe {
                    mark_adjust(
                        st.lnum + 1 as LineNr,
                        MAXLNUM,
                        1 as LineNr,
                        0 as LineNr,
                        kExtmarkNOOP,
                    )
                };
                if subflags.with(|flags| flags.do_ask) {
                    appended_lines(st.lnum - 1 as LineNr, 1 as LineNr);
                } else {
                    if st.first_line == 0 as LineNr {
                        st.first_line = st.lnum;
                    }
                    st.last_line = st.lnum + 1 as LineNr;
                }
                // All line numbers increase.
                st.sub_firstlnum += 1;
                st.lnum += 1;
                st.line2 += 1;
                // Move the cursor to the new line, like Vi.
                Win::current().w_cursor.lnum += 1;
                // Restart from the beginning of what is left.
                at = 0;
            } else {
                at += 1;
            }
        } else {
            at += cluster_len(&line.bytes()[at..]);
        }
    }
    st.new_line = Some(line);
}

/// Substitute one match into the line being built.
///
/// Returning early is upstream's `goto skip`: the expression failed, the
/// command was aborted, or this is only a `:s///n` count.
///
/// # Safety
/// Main thread; `st` must describe a live match.
pub(super) unsafe fn build_replacement(
    st: &mut Sub,
    _args: &SubArgs,
    current_match: &mut super::super::SubResult,
) {
    st.lnum_start = st.lnum; // save the start lnum
    let save_ma = Buf::current().b_p_ma;
    let counting = subflags.with(|flags| flags.do_count);
    if counting {
        // Prevent a function from accidentally changing the buffer.
        Buf::current().b_p_ma = 0;
    }
    // Held to the end of the function: the only path that reaches here with
    // `counting` set is the early return below.
    let _sandboxed = counting.then(Lock::sandbox);
    // Save the flags for recursion: they can change for e.g.
    // ":s/^/\=execute("s#^##gn")".
    let subflags_save = subflags.get();

    // Disallow changing text or switching window in an expression, and get
    // the length of the substitution part including the NUL.  When it fails
    // sublen is zero.
    st.sublen = {
        let _locked = Lock::text();
        // The measuring pass writes nothing, but `vim_regsub_multi` refuses
        // a null destination outright, so it is handed the old line -- which
        // is what upstream passes here, for the same reason.
        let dest = st.old_line().as_ptr();
        // SAFETY: the match and the copied line are live; without
        // `REGSUB_COPY` and with length 0 this only measures.
        unsafe {
            vim_regsub_multi(
                &raw mut st.regmatch,
                st.sub_firstlnum - st.regmatch.startpos[0].lnum,
                st.sub,
                dest,
                0 as c_int,
                regsub_flags(),
            )
        }
    };

    // If getting the substitute string caused an error, don't do the
    // replacement.  Don't keep flags set by a recursive call.
    subflags.set(subflags_save);
    // SAFETY: main thread.
    if st.sublen == 0 as c_int || aborting() || subflags.with(|flags| flags.do_count) {
        Buf::current().b_p_ma = save_ma;
        return;
    }

    // A multi-line match consumes the lines below, which are appended to the
    // rebuilt line and deleted from the buffer once it is committed.
    if st.nmatch != 1 as c_int {
        st.nmatch_tl += st.nmatch as LineNr - 1 as LineNr;
    }

    // Copy the text up to the part that matched.  Upstream grows its buffer
    // by hand here, to `strlen(new_start)` plus the text up to the match,
    // the replacement, the text after the match and a NUL; the `Vec` does
    // that growth, and the tail after the match is `commit_line`'s to add.
    let copy_len = st.regmatch.startpos[0].col as usize - st.copied;
    {
        let (old, line) = (&st.old_line, &mut st.new_line);
        let old = old.as_ref().expect("caller's contract -- a live match");
        let line = line.get_or_insert_with(SubLine::default);
        line.push_bytes(&old.bytes()[st.copied..][..copy_len]);
    }

    // Only now can we know where the match will actually start in the new
    // text.
    let start_col = st.new_line().len() as c_int;
    current_match.start.col = start_col as ColNr;

    {
        let _locked = Lock::text();
        let (rmp, lnum, sub) = (
            &raw mut st.regmatch,
            st.sub_firstlnum - st.regmatch.startpos[0].lnum,
            st.sub,
        );
        let sublen = st.sublen as usize;
        st.new_line_mut().append_written(sublen, |dest, destlen| {
            // SAFETY: `dest` has room for `destlen` bytes, which is the
            // length the measuring pass just answered.
            unsafe {
                vim_regsub_multi(
                    rmp,
                    lnum,
                    sub,
                    dest,
                    destlen,
                    REGSUB_COPY as c_int | regsub_flags(),
                )
            };
        });
    }
    sub_nsubs.set(sub_nsubs.get() + 1);
    st.did_sub = true;

    // Move the cursor to the start of the line, to avoid it being beyond the
    // end of the line after the substitution.
    Win::current().w_cursor.col = 0 as ColNr;

    // Remember the next character to be copied.
    st.copied = st.regmatch.endpos[0].col as usize;

    // SAFETY: the buffer is live.
    unsafe { st.adjust_sub_firstlnum() };

    // TODO(bfredl): this has some robustness issues, look into later.
    let start: LPos = st.regmatch.startpos[0];
    let end: LPos = st.regmatch.endpos[0];
    let mut replaced_bytes = 0 as BCount;
    let mut i = 0 as c_int;
    while i < st.nmatch - 1 as c_int {
        replaced_bytes += ml_get_len(st.lnum_start + i as LineNr) as BCount + 1 as BCount;
        i += 1;
    }
    replaced_bytes += (end.col - start.col) as BCount;

    // Save the line number before processing newlines.
    let lnum_before_newlines = st.lnum;
    // SAFETY: the replacement was just written into the rebuilt line.
    unsafe { split_carriage_returns(st, start_col as usize) };

    let new_endcol = st.new_line().len() as ColNr;
    current_match.end.col = new_endcol;
    current_match.end.lnum = st.lnum;

    let matchcols = end.col
        - if end.lnum == start.lnum {
            start.col
        } else {
            0 as ColNr
        };
    let subcols = new_endcol
        - if st.lnum == st.lnum_start {
            start_col
        } else {
            0 as c_int
        };
    save_undo_once(st);

    // Store the extmark data for this match; the whole batch is sent once the
    // line has been replaced.
    st.line_matches.push(LineData {
        start_col,
        start,
        end,
        matchcols,
        matchbytes: replaced_bytes,
        subcols,
        subbytes: (st.sublen - 1 as c_int) as BCount,
        lnum_before: lnum_before_newlines,
        lnum_after: st.lnum,
    });
}

/// Delete the lines a multi-line match consumed: their text has already been
/// appended to the rebuilt line, so the buffer does not need it.
///
/// Answers false when undo could not be saved.
///
/// # Safety
/// Main thread; `st.lnum` must be the rebuilt line.
unsafe fn delete_matched_lines(st: &mut Sub) -> bool {
    st.lnum += 1;
    // SAFETY: the lines below `lnum` are the ones the match spanned.
    if u_savedel(st.lnum, st.nmatch_tl).is_err() {
        return false;
    }
    let mut i = 0 as LineNr;
    while i < st.nmatch_tl {
        // SAFETY: as above.
        let _ = unsafe { ml_delete(st.lnum) };
        i += 1;
    }
    // SAFETY: as above.
    unsafe {
        mark_adjust(
            st.lnum,
            st.lnum + st.nmatch_tl - 1 as LineNr,
            MAXLNUM,
            -st.nmatch_tl,
            kExtmarkNOOP,
        )
    };
    if subflags.with(|flags| flags.do_ask) {
        deleted_lines(st.lnum, st.nmatch_tl);
    }
    st.lnum -= 1;
    st.line2 -= st.nmatch_tl; // the number of lines decreases
    st.nmatch_tl = 0 as LineNr;
    true
}

/// Put the rebuilt line into the buffer, with the extmark splices the
/// substitutions on it produced.
///
/// Answers false when undo could not be saved, which abandons the line.
///
/// # Safety
/// Main thread; `st.new_line` must hold the rebuilt line.
pub(super) unsafe fn commit_line(st: &mut Sub) -> bool {
    // Copy the rest of the line, the part that didn't match.  "matchcol" has
    // to be adjusted using the end of the line as reference, because the
    // substitute may have changed the number of characters; same for
    // "prev_matchcol".
    let old_len = {
        let (old, line) = (&st.old_line, &mut st.new_line);
        let old = old.as_ref().expect("caller's contract -- an old line");
        let line = line.as_mut().expect("caller's contract -- a rebuilt line");
        line.push_bytes(&old.bytes()[st.copied..]);
        old.len() as ColNr
    };
    st.matchcol = old_len - st.matchcol;
    st.prev_matchcol = old_len - st.prev_matchcol;

    // SAFETY: `lnum` is a line of the buffer.
    if u_savesub(st.lnum).is_err() {
        return false;
    }
    // SAFETY: `lnum` is a line of the buffer and the rebuilt line is a live
    // C string, which `ml_replace` is told to copy.
    let _ = unsafe { ml_replace(st.lnum, st.new_line().as_ptr(), true) };

    // Call extmark_splice for each match on this line.
    for m in &st.line_matches {
        extmark_splice(
            Buf::current(),
            m.lnum_before as c_int - 1 as c_int,
            m.start_col as ColNr,
            m.end.lnum as c_int - m.start.lnum as c_int,
            m.matchcols as ColNr,
            m.matchbytes,
            m.lnum_after as c_int - m.lnum_before as c_int,
            m.subcols as ColNr,
            m.subbytes,
            kExtmarkUndo,
        );
    }
    // Reset the match data for the next line.
    st.line_matches.clear();

    if st.nmatch_tl > 0 as LineNr {
        // SAFETY: the rebuilt line is in the buffer.
        if !unsafe { delete_matched_lines(st) } {
            return false;
        }
    }

    // When asking, undo is saved each time, so the changed flag must be set
    // each time too.
    if subflags.with(|flags| flags.do_ask) {
        // SAFETY: `lnum` is a line of the buffer.
        unsafe { changed_bytes(st.lnum, 0 as ColNr) };
    } else {
        if st.first_line == 0 as LineNr {
            st.first_line = st.lnum;
        }
        st.last_line = st.lnum + 1 as LineNr;
    }

    st.sub_firstlnum = st.lnum;
    // `ml_replace` was told to *copy*, so the rebuilt line is still ours: it
    // becomes the old text for the next match, and the text it replaces is
    // dropped with it.  Upstream's `xfree` and pointer assignment are this
    // one move.
    st.old_line = st.new_line.take();
    let new_len = st.old_line().len() as ColNr;
    st.matchcol = new_len - st.matchcol;
    st.prev_matchcol = new_len - st.prev_matchcol;
    st.copied = 0;
    true
}
