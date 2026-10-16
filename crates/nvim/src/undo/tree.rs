//! The undo tree's own bookkeeping: freeing headers and branches, and the
//! single-line `U` shadow copy.
//!
//! Every header here is named by an [`UndoLink`] and reached through the
//! buffer's store (see [`super::store`]). The freeing functions read a
//! header's links afresh at each step, as upstream read them through the
//! pointer, so a branch freed underneath a step ends that step instead of
//! reading freed memory.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::store::{header_chain, header_free, store_release};
use super::*;
use crate::memline::ml_replace_buf_owned;
use crate::memory::XString;
use crate::narrow::len_as_int;
use crate::winlayer::{Buf, Win};

/// Marks `start` and everything reachable from it backwards as changed.
pub(crate) fn u_unch_branch(mut buffer: Buf, start: UndoLink) {
    for link in header_chain(buffer, start, |uh| uh.prev) {
        let Some(uh) = buffer.header_mut(link) else {
            break;
        };
        uh.flags |= UH_CHANGED;
        let alt_next = uh.alt_next;
        if alt_next.is_some() {
            u_unch_branch(buffer, alt_next);
        }
    }
}

/// Whether the newest header holds any entries, complaining if the tree has
/// come apart.
pub(crate) fn u_get_headentry(buffer: Buf) -> bool {
    let has_entries = buffer
        .header(buffer.b_u_newhead)
        .is_some_and(|uh| !uh.entries.is_empty());
    if !has_entries {
        iemsg(gettext(c"E439: Undo list corrupt"));
    }
    has_entries
}

/// Fills in the `bot` the newest header deferred, and marks the buffer
/// synced.
pub(crate) fn u_getbot(mut buffer: Buf) {
    if !u_get_headentry(buffer) {
        return;
    }
    let line_count = buffer.b_ml.ml_line_count;
    let newhead = buffer.b_u_newhead;
    let Some(uh) = buffer.header_mut(newhead) else {
        return;
    };
    if let Some(entry) = uh.getbot.take().and_then(|at| uh.entries.get_mut(at)) {
        let extra: LineNr = line_count - entry.lcount;
        entry.bot = entry.top + entry.size() + 1 + extra;
        if entry.bot < 1 || entry.bot > line_count {
            iemsg(gettext(c"E440: Undo line missing"));
            entry.bot = entry.top + 1;
        }
    }
    buffer.b_u_synced = true;
}

/// Clears `held` when it names `link`. The caller's own copy of a link it is
/// about to read again, which a free must not leave naming nothing real.
fn forget(held: Option<&mut UndoLink>, link: UndoLink) {
    if let Some(held) = held
        && *held == link
    {
        *held = UndoLink::NONE;
    }
}

/// Unlinks one header from the tree and frees it, along with the alternate
/// branch hanging off it.
///
/// `held`, when it is given, is a link the caller is still holding: it is
/// cleared if it named a header that went away.
pub(crate) fn u_freeheader(mut buffer: Buf, link: UndoLink, mut held: Option<&mut UndoLink>) {
    let alt_next = buffer.header_link(link, |uh| uh.alt_next);
    if buffer.has_header(alt_next) {
        u_freebranch(buffer, alt_next, held.as_deref_mut());
    }
    // Freeing the alternate branch can take the oldest header, and with it
    // this one; then there is nothing left to unlink.
    let Some(uh) = buffer.header(link) else {
        return;
    };
    let (alt_prev, next, prev) = (uh.alt_prev, uh.next, uh.prev);
    if let Some(before) = buffer.header_mut(alt_prev) {
        before.alt_next = UndoLink::NONE;
    }
    match buffer.header_mut(next) {
        Some(above) => above.prev = prev,
        None => buffer.b_u_oldhead = prev,
    }
    if prev.is_none() {
        buffer.b_u_newhead = next;
    } else {
        // The alternate headers at `prev` all claim this header's successor.
        for alt in header_chain(buffer, prev, |uh| uh.alt_next) {
            if let Some(uh) = buffer.header_mut(alt) {
                uh.next = next;
            }
        }
    }
    u_freeentries(buffer, link, held);
}

/// Frees a whole alternate branch, oldest header first.
pub(crate) fn u_freebranch(mut buffer: Buf, link: UndoLink, mut held: Option<&mut UndoLink>) {
    // Freeing the oldest header takes the whole tree with it, so let
    // `u_freeheader` do the unlinking rather than walking here.
    if link == buffer.b_u_oldhead {
        while buffer.has_header(buffer.b_u_oldhead) {
            u_freeheader(buffer, buffer.b_u_oldhead, held.as_deref_mut());
        }
        return;
    }
    let alt_prev = buffer.header_link(link, |uh| uh.alt_prev);
    if let Some(before) = buffer.header_mut(alt_prev) {
        before.alt_next = UndoLink::NONE;
    }
    // Not `header_chain`: the step would have to read a header this loop
    // has already freed.
    let mut next = link;
    while buffer.has_header(next) {
        let tofree = next;
        let alt = buffer.header_link(tofree, |uh| uh.alt_next);
        if buffer.has_header(alt) {
            u_freebranch(buffer, alt, held.as_deref_mut());
        }
        next = buffer.header_link(tofree, |uh| uh.prev);
        u_freeentries(buffer, tofree, held.as_deref_mut());
    }
}

/// Frees one header, its entries and its extmark list, clearing every link
/// the buffer (or the caller, through `held`) holds to it.
pub(crate) fn u_freeentries(mut buffer: Buf, link: UndoLink, held: Option<&mut UndoLink>) {
    if buffer.b_u_curhead == link {
        buffer.b_u_curhead = UndoLink::NONE;
    }
    if buffer.b_u_newhead == link {
        buffer.b_u_newhead = UndoLink::NONE;
    }
    forget(held, link);
    header_free(buffer, link);
    buffer.b_u_numhead -= 1;
}

/// Detaches the buffer's undo tree without freeing it.
///
/// The headers stay in the store; a command preview puts them back, and the
/// `U` line with them (it took that line before calling here).
pub fn u_clearall(mut buffer: Buf) {
    buffer.b_u_curhead = UndoLink::NONE;
    buffer.b_u_oldhead = UndoLink::NONE;
    buffer.b_u_newhead = UndoLink::NONE;
    buffer.b_u_synced = true;
    buffer.b_u_numhead = 0;
    buffer.b_u_line = None;
    buffer.b_u_line_lnum = 0;
}

/// Frees every header the buffer's tree still reaches, and the `U` line.
pub fn u_blockfree(mut buffer: Buf) {
    while buffer.has_header(buffer.b_u_oldhead) {
        let previous_oldhead = buffer.b_u_oldhead;
        // Each pass frees the oldest header, and the assert is the
        // transpiled loop's own guard against not making progress.
        u_freeheader(buffer, previous_oldhead, None);
        debug_assert!(
            buffer.b_u_oldhead != previous_oldhead,
            "buf->b_u_oldhead != previous_oldhead"
        );
    }
    buffer.b_u_line = None;
    store_release(buffer);
}

/// [`u_blockfree`], then [`u_clearall`].
pub fn u_clearallandblockfree(buffer: Buf) {
    u_blockfree(buffer);
    u_clearall(buffer);
}

/// Remembers one line so `U` can put it back.
pub(crate) fn u_saveline(mut buffer: Buf, lnum: LineNr) {
    if lnum == buffer.b_u_line_lnum {
        return;
    }
    if lnum < 1 || lnum > buffer.b_ml.ml_line_count {
        return;
    }
    u_clearline(buffer);
    buffer.b_u_line_lnum = lnum;
    let win = Win::current();
    buffer.b_u_line_colnr = if win.w_buffer == buffer && win.w_cursor.lnum == lnum {
        win.w_cursor.col
    } else {
        0
    };
    buffer.b_u_line = Some(line_copy(buffer, lnum));
}

/// Forgets the line `U` would have put back.
pub fn u_clearline(mut buffer: Buf) {
    if buffer.b_u_line.take().is_some() {
        buffer.b_u_line_lnum = 0;
    }
}

/// `U`: swap the current line against the one `u_saveline` kept.
pub fn u_undoline() {
    let mut buf = Buf::current();
    if buf.b_u_line.is_none() || buf.b_u_line_lnum > buf.b_ml.ml_line_count {
        beep_flush();
        return;
    }
    let lnum = buf.b_u_line_lnum;
    if u_savecommon(buf, lnum - 1, lnum + 1, 0, false).is_err() {
        return;
    }
    let lnum = buf.b_u_line_lnum;
    let Some(saved) = buf.b_u_line.take() else {
        return;
    };
    // What is in the buffer now becomes the line the next `U` puts back,
    // and the saved line goes in without a copy.
    let current = line_copy(buf, lnum);
    let (old_len, new_len) = (current.len(), saved.len());
    let _ = ml_replace_buf_owned(buf, lnum, saved);
    buf.b_u_line = Some(current);
    extmark_splice_cols(
        buf,
        lnum - 1,
        0,
        len_as_int(old_len),
        len_as_int(new_len),
        kExtmarkUndo,
    );
    changed_bytes(lnum, 0);
    let mut win = Win::current();
    let col: ColNr = buf.b_u_line_colnr;
    if win.w_cursor.lnum == buf.b_u_line_lnum {
        buf.b_u_line_colnr = win.w_cursor.col;
    }
    win.w_cursor.col = col;
    win.w_cursor.lnum = buf.b_u_line_lnum;
    check_cursor_col(win);
}

/// A fresh copy of line `lnum` of `buffer`.
pub(crate) fn line_copy(buffer: Buf, lnum: LineNr) -> XString {
    XString::from_bytes(buffer.lines().line(lnum))
}

#[cfg(test)]
mod tests {
    use super::super::store::{Marks, header_adopt, header_chain};
    use super::*;
    use crate::buffer::{bare_buffer, free_bare_buffer};
    use crate::global_cell::editor_state_lock;

    /// A header with its four links and one saved line, which dropping must
    /// release.
    fn header(
        seq: c_int,
        next: c_int,
        prev: c_int,
        alt_next: c_int,
        alt_prev: c_int,
    ) -> Box<UndoHeader> {
        Box::new(UndoHeader {
            seq,
            next: UndoLink::to_seq(next),
            prev: UndoLink::to_seq(prev),
            alt_next: UndoLink::to_seq(alt_next),
            alt_prev: UndoLink::to_seq(alt_prev),
            entries: vec![UndoEntry {
                top: 0,
                bot: 2,
                lcount: 0,
                lines: vec![XString::from_bytes(format!("saved by {seq}").as_bytes())],
            }],
            ..Default::default()
        })
    }

    /// 1 is the oldest change and 3 the newest on the main branch; 4 forks
    /// at the same place 3 does.
    ///
    /// ```text
    ///   1 -- 2 -- 3
    ///         \-- 4
    /// ```
    fn small_tree(mut buffer: Buf) {
        for uh in [
            header(1, 0, 2, 0, 0),
            header(2, 1, 3, 0, 0),
            header(3, 2, 0, 4, 0),
            header(4, 2, 0, 0, 3),
        ] {
            header_adopt(buffer, uh);
        }
        buffer.b_u_oldhead = UndoLink::to_seq(1);
        buffer.b_u_newhead = UndoLink::to_seq(3);
        buffer.b_u_curhead = UndoLink::NONE;
        buffer.b_u_numhead = 4;
    }

    fn seqs(links: impl Iterator<Item = UndoLink>) -> Vec<c_int> {
        links.map(UndoLink::seq).collect()
    }

    #[test]
    fn the_walks_reach_every_header_by_link() {
        let _held = editor_state_lock();
        let buffer = bare_buffer();
        small_tree(buffer);

        let mut seen: Vec<c_int> = buffer
            .tree_walk(buffer.b_u_oldhead, Marks::next())
            .filter(|visit| visit.first)
            .map(|visit| visit.link.seq())
            .collect();
        seen.sort_unstable();
        assert_eq!(seen, vec![1, 2, 3, 4]);
        assert_eq!(
            seqs(header_chain(buffer, UndoLink::to_seq(1), |uh| uh.prev)),
            vec![1, 2, 3]
        );
        assert_eq!(
            seqs(header_chain(buffer, UndoLink::to_seq(3), |uh| uh.alt_next)),
            vec![3, 4]
        );
        // A chain into a header nobody holds stops before it.
        assert_eq!(
            seqs(header_chain(buffer, UndoLink::to_seq(9), |uh| uh.prev)),
            Vec::<c_int>::new()
        );

        u_blockfree(buffer);
        free_bare_buffer(buffer);
    }

    #[test]
    fn freeing_a_branch_unlinks_it_and_drops_what_it_saved() {
        let _held = editor_state_lock();
        let mut buffer = bare_buffer();
        small_tree(buffer);
        let mut held = UndoLink::to_seq(4);

        u_freebranch(buffer, UndoLink::to_seq(4), Some(&mut held));
        assert!(
            held.is_none(),
            "the caller's link to the freed header is cleared"
        );
        assert!(!buffer.has_header(UndoLink::to_seq(4)));
        assert!(
            buffer
                .header_link(UndoLink::to_seq(3), |uh| uh.alt_next)
                .is_none()
        );
        assert_eq!(buffer.b_u_numhead, 3);

        // Freeing the oldest header unlinks it from the one above.
        u_freeheader(buffer, UndoLink::to_seq(1), None);
        assert_eq!(buffer.b_u_oldhead, UndoLink::to_seq(2));
        assert!(
            buffer
                .header_link(UndoLink::to_seq(2), |uh| uh.next)
                .is_none()
        );
        assert_eq!(buffer.b_u_numhead, 2);

        // And the rest goes with the store.
        buffer.b_u_line = Some(XString::from_bytes(b"the U line"));
        u_blockfree(buffer);
        assert!(buffer.b_u_store.is_none());
        assert!(buffer.b_u_line.is_none());
        assert_eq!(buffer.b_u_numhead, 0);
        free_bare_buffer(buffer);
    }

    /// `u_clearall` detaches the tree but the store keeps it, which is what
    /// lets a command preview put it back.
    #[test]
    fn a_detached_tree_stays_in_the_store() {
        let _held = editor_state_lock();
        let mut buffer = bare_buffer();
        small_tree(buffer);
        u_clearall(buffer);
        assert!(buffer.b_u_oldhead.is_none());
        u_blockfree(buffer);
        assert_eq!(
            buffer.b_u_store.as_deref().map(|store| store.len()),
            Some(4)
        );
        // Put back and freed for real.
        buffer.b_u_oldhead = UndoLink::to_seq(1);
        buffer.b_u_numhead = 4;
        u_blockfree(buffer);
        assert!(buffer.b_u_store.is_none());
        free_bare_buffer(buffer);
    }
}
