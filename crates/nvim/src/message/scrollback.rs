//! The message scrollback, which `g<` and the pager page through.
//!
//! Every line [`crate::message::msg_bytes_to_grid`] emits is also
//! copied into a queue of [`Chunk`]s ([`store_sb_text`]), so that the pager
//! can scroll backwards past what the screen still holds.
//!
//! Upstream's chunks were a doubly-linked list the pager held raw pointers
//! into across `get_keystroke` -- which runs timers, which can print, which
//! can clear the list. Here a chunk is named by its [`SbPos`], a number that
//! stays put as chunks are added at the back and dropped at the front; a
//! position whose chunk has gone simply finds nothing.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::types::CmdLine;
use core::ffi::{c_int, c_uint};
use std::collections::VecDeque;

/// One run of displayed message text.
struct Chunk {
    text: Box<[u8]>,
    /// The run ends its screen line.
    eol: bool,
    /// The column the run started at.
    msg_col: c_int,
    hl_id: c_int,
}

/// Where a chunk is in the scrollback. Stable while chunks are added at
/// the back and dropped at the front.
pub(crate) type SbPos = usize;

/// The scrollback: `chunks[0]` is at position `first`.
struct Scrollback {
    chunks: VecDeque<Chunk>,
    first: SbPos,
}

impl Scrollback {
    fn get(&self, pos: SbPos) -> Option<&Chunk> {
        self.chunks.get(pos.checked_sub(self.first)?)
    }

    fn last(&self) -> Option<SbPos> {
        (!self.chunks.is_empty()).then(|| self.first + self.chunks.len() - 1)
    }

    /// The chunk before `pos`, if there is one.
    fn prev(&self, pos: SbPos) -> Option<SbPos> {
        let prev = pos.checked_sub(1)?;
        self.get(prev).map(|_| prev)
    }

    /// The chunk that starts the screen line `pos` is part of.
    fn line_start(&self, pos: SbPos) -> Option<SbPos> {
        self.get(pos)?;
        let mut at = pos;
        while let Some(prev) = self.prev(at) {
            if self.chunks[prev - self.first].eol {
                break;
            }
            at = prev;
        }
        Some(at)
    }

    /// Drop the chunks before `pos`.
    fn drop_before(&mut self, pos: SbPos) {
        let n = pos.saturating_sub(self.first).min(self.chunks.len());
        self.chunks.drain(..n);
        self.first += n;
    }

    /// Mark the last chunk as finishing its screen line.
    fn end_line(&mut self) {
        if let Some(last) = self.chunks.back_mut() {
            last.eol = true;
        }
    }
}

static SCROLLBACK: GlobalCell<Scrollback> = GlobalCell::new(Scrollback {
    chunks: VecDeque::new(),
    first: 0,
});

/// Remember `run` for scrolling back over later.
///
/// `finish` marks the chunk as ending its screen line. `sb_col` is the column
/// the run started at, so the pager can put it back where it was; it is reset
/// here, because the next run starts at the left margin of whatever comes
/// after this one.
pub(crate) fn store_sb_text(run: &[u8], hl_id: c_int, sb_col: &mut c_int, finish: bool) {
    let mut run = run;
    let clear = do_clear_sb_text.get();
    if clear == SB_CLEAR_ALL || clear == SB_CLEAR_CMDLINE_DONE {
        clear_sb_text(clear == SB_CLEAR_ALL);
        msg_sb_eol(); // prevent messages from overlapping
        if clear == SB_CLEAR_CMDLINE_DONE && run.first() == Some(&b'\n') {
            run = &run[1..];
        }
        do_clear_sb_text.set(SB_CLEAR_NONE);
    }

    SCROLLBACK.with_mut(|sb| {
        if !run.is_empty() {
            sb.chunks.push_back(Chunk {
                text: run.into(),
                eol: finish,
                msg_col: *sb_col,
                hl_id,
            });
        } else if finish {
            sb.end_line();
        }
    });

    *sb_col = 0;
}

/// Finished showing messages: clear the scroll-back text on the next one.
pub fn may_clear_sb_text() {
    msg_ext_ui_flush(); // ensure messages until now are emitted
    do_clear_sb_text.set(SB_CLEAR_ALL);
    do_clear_hist_temp.set(true);
}

/// Starting to edit the command line: do not clear messages now.
pub fn sb_text_start_cmdline() {
    if do_clear_sb_text.get() == SB_CLEAR_CMDLINE_BUSY {
        // A recursive command line: the outer one need not be remembered,
        // it will be redrawn when this level returns.
        sb_text_restart_cmdline();
    } else {
        msg_sb_eol();
        do_clear_sb_text.set(SB_CLEAR_CMDLINE_BUSY);
    }
}

/// Redrawing the command line: drop the last unfinished line.
pub fn sb_text_restart_cmdline() {
    // Needed when returning from a nested command line.
    do_clear_sb_text.set(SB_CLEAR_CMDLINE_BUSY);
    SCROLLBACK.with_mut(|sb| {
        // No unfinished line: don't clear anything.
        let Some(last) = sb.last().filter(|&last| !sb.chunks[last - sb.first].eol) else {
            return;
        };
        let start = sb.line_start(last).unwrap_or(last);
        sb.chunks.truncate(start - sb.first);
    });
}

/// Finished editing the command line: clear the old lines, but the last one
/// only later.
pub fn sb_text_end_cmdline() {
    do_clear_sb_text.set(SB_CLEAR_CMDLINE_DONE);
}

/// Forget the remembered text. With `all` false the last screen line is kept.
pub fn clear_sb_text(all: bool) {
    SCROLLBACK.with_mut(|sb| {
        let keep_from = if all {
            sb.last().map(|last| last + 1)
        } else {
            sb.last().and_then(|last| sb.line_start(last))
        };
        if let Some(keep_from) = keep_from {
            sb.drop_before(keep_from);
        }
    });
}

/// The `g<` command.
pub fn show_sb_text() {
    if ui_has(kUIMessages) {
        let mut ea = ExArg {
            line: CmdLine::from_bytes(b""),
            skip: true,
            ..ExArg::default()
        };
        ex_messages(&mut ea);
        return;
    }
    // Only show something when there is more than one line: a command
    // with no output would otherwise leave one line looking odd.
    let first_of_last = msg_sb_start(sb_last());
    if first_of_last.and_then(sb_prev).is_none() {
        vim_beep(kOptBoFlagMess as c_uint);
    } else {
        do_more_prompt(c_int::from(b'G'));
        wait_return(0);
    }
}

/// The newest chunk.
pub(crate) fn sb_last() -> Option<SbPos> {
    SCROLLBACK.with(Scrollback::last)
}

/// The chunk before `pos`.
pub(crate) fn sb_prev(pos: SbPos) -> Option<SbPos> {
    SCROLLBACK.with(|sb| sb.prev(pos))
}

/// The chunk that starts the screen line `pos` is part of.
pub(crate) fn msg_sb_start(pos: Option<SbPos>) -> Option<SbPos> {
    SCROLLBACK.with(|sb| sb.line_start(pos?))
}

/// Mark the last chunk as finishing its screen line.
pub fn msg_sb_eol() {
    SCROLLBACK.with_mut(Scrollback::end_line);
}

/// Redisplay one remembered screen line at `row`, starting at `pos`;
/// answers the chunk the next line starts at.
///
/// Each chunk is copied out before it is drawn: drawing can redraw, and
/// the scrollback must not be borrowed when it does.
pub(crate) fn disp_sb_line(row: c_int, pos: SbPos) -> Option<SbPos> {
    let mut at = pos;
    loop {
        let (text, col, hl_id, eol) = SCROLLBACK.with(|sb| {
            sb.get(at)
                .map(|chunk| (chunk.text.clone(), chunk.msg_col, chunk.hl_id, chunk.eol))
        })?;
        msg_row.set(row);
        msg_col.set(col);
        msg_bytes_to_grid(&text, hl_id, true);
        let next = at + 1;
        let has_next = SCROLLBACK.with(|sb| sb.get(next).is_some());
        if eol || !has_next {
            return has_next.then_some(next);
        }
        at = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(text: &[u8], eol: bool) -> Chunk {
        Chunk {
            text: text.into(),
            eol,
            msg_col: 0,
            hl_id: 0,
        }
    }

    /// Two screen lines: `a` `b` | `c` `d`, with the second unfinished.
    fn two_lines() -> Scrollback {
        let mut sb = Scrollback {
            chunks: VecDeque::new(),
            first: 0,
        };
        sb.chunks.push_back(chunk(b"a", false));
        sb.chunks.push_back(chunk(b"b", true));
        sb.chunks.push_back(chunk(b"c", false));
        sb.chunks.push_back(chunk(b"d", false));
        sb
    }

    #[test]
    fn a_line_starts_after_the_previous_line_ends() {
        let sb = two_lines();
        assert_eq!(sb.last(), Some(3));
        assert_eq!(sb.line_start(3), Some(2));
        assert_eq!(sb.line_start(2), Some(2));
        assert_eq!(sb.line_start(1), Some(0));
        assert_eq!(sb.prev(0), None);
    }

    #[test]
    fn a_position_survives_dropping_the_front() {
        let mut sb = two_lines();
        sb.drop_before(2);
        // The surviving chunks keep their numbers ...
        assert_eq!(sb.get(3).map(|chunk| &*chunk.text), Some(&b"d"[..]));
        assert_eq!(sb.line_start(3), Some(2));
        // ... and the dropped ones are simply not there.
        assert!(sb.get(1).is_none());
        assert_eq!(sb.prev(2), None);
        assert_eq!(sb.line_start(0), None);
    }

    #[test]
    fn dropping_past_the_end_empties_it() {
        let mut sb = two_lines();
        sb.drop_before(10);
        assert_eq!(sb.last(), None);
        assert_eq!(sb.first, 4);
        sb.end_line();
        assert_eq!(sb.last(), None);
    }
}
