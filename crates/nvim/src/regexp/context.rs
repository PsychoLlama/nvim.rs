//! What both engines ask of the running match rather than answering
//! themselves: the lines it reads, the capture slots it clears, and the
//! position tests (`\%V`, `\<`, a back-reference across lines).
//!
//! The match itself is a [`Rex`] — see [`super::rex`] for what it holds and
//! for the contract the lines it reads are borrowed under.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::option::vars::P_SEL;
use crate::regexp::RegCompiler;
use core::ffi::{CStr, c_int};

use super::{
    MULTI_MULT, MatchPos, NSUBEXP, PosKind, RA_FAIL, RA_MATCH, RA_NOMATCH, Rex, cstrncmp,
    head_off_at, peekchr, prefix_at, re_multi_type,
};
use crate::charset::vim_iswordc_buf;
use crate::getchar::state::got_int;
use crate::mbyte::{class_in, strnicmp_in};
use crate::normal::{VisualMode, visual_ever_started, visual_selection};
use crate::os::input::fast_breakcheck;
use crate::plines::win_linetabsize;
use crate::pos::{MAXCOL, lt};
use crate::regexp::state::rc_did_emsg;
use crate::semsg;
use crate::types::{ColNr, LineNr};

use crate::winlayer::{Buf, Win};

/// Let the user interrupt a long match, unless the caller asked for an
/// uninterruptible one (`RE_NOBREAK`, for matches run where input cannot
/// be read).
pub(crate) fn reg_breakcheck(rex: Rex) {
    if !rex.reg_nobreak() {
        fast_breakcheck();
    }
}

/// Is `c` a keyword character? 'iskeyword' is buffer-local and the buffer
/// being matched is not always the current one.
pub(crate) fn reg_iswordc(rex: Rex, c: c_int) -> bool {
    vim_iswordc_buf(c, rex.reg_buf())
}

/// What an empty line past the match's last one reads as.
const PAST_THE_END: &[u8] = b"\0";

/// Line `lnum` of the match, counted from its first line, with its NUL:
/// `None` above line 1 of the buffer, an empty line past the match's last
/// line.
///
/// A string match asks this only from `\%'m`, and then — as upstream — of
/// the current buffer, from the line the last buffer match started on.
pub(crate) fn reg_getline<'t>(rex: Rex<'t>, lnum: LineNr) -> Option<&'t [u8]> {
    let first = rex.reg_firstlnum();
    if first + lnum < 1 {
        None
    } else if lnum > rex.reg_maxline() {
        Some(PAST_THE_END)
    } else {
        Some(rex.buffer_line(first + lnum))
    }
}

/// The length of [`reg_getline`]'s line, its NUL left out; 0 for either
/// stand-in.
pub(crate) fn reg_getline_len(rex: Rex, lnum: LineNr) -> ColNr {
    reg_getline(rex, lnum).map_or(0, |line| {
        ColNr::try_from(line.len() - 1).unwrap_or(ColNr::MAX)
    })
}

/// The `\z1`..`\z9` captures of a syntax region's start match.
///
/// A syntax item keeps them for as long as it is on the parser's state
/// stack, and the state cache keeps a copy per cached line, so a set is
/// shared: [`ExtMatchRef`]. The end and skip patterns of the region read
/// them back through [`ExtMatchIo::input`].
#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct ExtMatch {
    /// Slot `n` is what `\z(` group `n` matched; slot 0 is never set.
    pub(crate) matches: [Option<::std::ffi::CString>; NSUBEXP as usize],
}

/// A shared [`ExtMatch`].
pub(crate) type ExtMatchRef = ::std::rc::Rc<ExtMatch>;

impl ExtMatch {
    /// What `\z(` group `no` matched, if it matched on one line.
    pub(crate) fn capture(&self, no: usize) -> Option<&CStr> {
        self.matches[no].as_deref()
    }

    /// Record `text` as group `no`'s match. The capture is a C string to
    /// the matcher that reads it back, so it ends at a NUL in `text`.
    pub(crate) fn set(&mut self, no: usize, text: &[u8]) {
        self.matches[no] = Some(cstr::owned(cstr::until_nul(text)));
    }
}

/// The `\z` traffic of one syntax match: the captures of the region's
/// start match that `\z1`..`\z9` read, and the slot, if any, the match's own
/// `\z(` captures go to when it succeeds.
pub(crate) struct ExtMatchIo<'a> {
    pub(crate) input: Option<&'a ExtMatch>,
    pub(crate) output: Option<&'a mut Option<ExtMatchRef>>,
}

/// The character class of the character before the cursor, or -1 at the
/// start of the line. Backs `\<` and `\>`.
pub(crate) fn reg_prev_class(rex: Rex) -> c_int {
    if rex.col() <= 0 {
        return -1;
    }
    // The cursor is past the start of the line, so the byte before it is on
    // the line and the walk back stops at the line's start.
    let line = rex.line();
    let prev = rex.input() - 1;
    class_in(&line[prev - head_off_at(line, prev)..], rex.reg_buf())
}

/// Is the position being matched inside the Visual area? Backs `\%V`.
pub(crate) fn reg_match_visual(rex: Rex) -> bool {
    // `reg_win` is the window the match is running for, or `curwin` when it
    // has none; both stay live for the length of the match.
    let wp = rex.reg_win().unwrap_or_else(Win::current);
    // `\%V` is a buffer-position test, so it only applies to a multi-line
    // match in the current buffer.
    if Some(rex.reg_buf()) != Buf::current_or_none() || !visual_ever_started() || !rex.multi() {
        return false;
    }

    let (top, bot, mode, curswant) = if let Some(sel) = visual_selection() {
        let (top, bot) = if lt(sel.anchor, wp.w_cursor) {
            (sel.anchor, wp.w_cursor)
        } else {
            (wp.w_cursor, sel.anchor)
        };
        (top, bot, sel.mode, wp.w_curswant)
    } else {
        let buf = Buf::current();
        let (start, end) = (buf.b_visual.vi_start, buf.b_visual.vi_end);
        let (top, mut bot) = if lt(start, end) {
            (start, end)
        } else {
            (end, start)
        };
        bot.lnum = bot.lnum.min(buf.b_ml.ml_line_count);
        (
            top,
            bot,
            VisualMode::from_raw(buf.b_visual.vi_mode),
            buf.b_visual.vi_curswant,
        )
    };

    let lnum = rex.buf_lnum();
    if lnum < top.lnum || lnum > bot.lnum {
        return false;
    }
    let col = rex.col();
    if mode.is_char() {
        // 'selection' decides whether the last character is included.
        let inclusive = ColNr::from(P_SEL.first_byte() != b'e');
        !((lnum == top.lnum && col < top.col) || (lnum == bot.lnum && col >= bot.col + inclusive))
    } else if mode.is_block() {
        let (start, end) = wp.virtual_vcol_span_at(top);
        let (start2, end2) = wp.virtual_vcol_span_at(bot);
        let start = start.min(start2);
        let mut end = end.max(end2);
        // `$` in blockwise Visual stretches the block to end of line.
        if top.col == MAXCOL as c_int || bot.col == MAXCOL as c_int || curswant == MAXCOL as c_int {
            end = MAXCOL as c_int;
        }
        // The virtual columns read other lines, which moves the line out
        // from under the match: fetch it again.
        let line = reg_getline(rex, rex.lnum()).unwrap_or(PAST_THE_END);
        rex.seek(line, col);
        let cols = win_linetabsize(wp, rex.buf_lnum(), line, col);
        cols >= start && cols <= end - ColNr::from(P_SEL.first_byte() == b'e')
    } else {
        true
    }
}

/// Reset the `\1`..`\9` slots, or with `z` the `\z1`..`\z9` ones. Lazy: an
/// engine only pays for this if a match reaches a back-reference.
fn cleanup(rex: Rex, z: bool) {
    let need = if z {
        rex.need_clear_zsubexpr()
    } else {
        rex.need_clear_subexpr()
    };
    if !need {
        return;
    }
    // An unset slot is line -1 column -1 in either kind of match: upstream
    // memset a buffer match's to 0xff and a string match's to NULL.
    let unset = MatchPos::unset(PosKind::Buf).as_pos();
    let (starts, ends) = if z {
        (rex.zstarts(), rex.zends())
    } else {
        (rex.starts(), rex.ends())
    };
    starts.set([unset; NSUBEXP as usize]);
    ends.set([unset; NSUBEXP as usize]);
    if z {
        rex.set_need_clear_zsubexpr(false);
    } else {
        rex.set_need_clear_subexpr(false);
    }
}

pub(crate) fn cleanup_subexpr(rex: Rex) {
    cleanup(rex, false);
}

pub(crate) fn cleanup_zsubexpr(rex: Rex) {
    cleanup(rex, true);
}

/// Step the match on to the next line.
pub(crate) fn reg_nextline(rex: Rex) {
    rex.set_lnum(rex.lnum() + 1);
    rex.set_line(reg_getline(rex, rex.lnum()).unwrap_or(PAST_THE_END));
    rex.set_input(0);
    reg_breakcheck(rex);
}

/// Match the text a back-reference captured, which may span lines. On
/// success `*bytelen` is the length matched on the *last* line, which is
/// what the caller advances by.
pub(crate) fn match_with_backref(
    rex: Rex,
    start_lnum: LineNr,
    start_col: ColNr,
    end_lnum: LineNr,
    end_col: ColNr,
    mut bytelen: Option<&mut c_int>,
) -> c_int {
    let mut clnum = start_lnum;
    let mut ccol = start_col;
    if let Some(n) = bytelen.as_deref_mut() {
        *n = 0;
    }
    loop {
        // Reading the captured line moves the one the match stands on, so
        // the captured text is copied out (to its line's end, as the
        // compare may look past the capture at combining characters) and
        // the match's own line fetched again after.
        let captured_line = reg_getline(rex, clnum).unwrap_or(PAST_THE_END);
        let mut len = if clnum == end_lnum {
            end_col - ccol
        } else {
            reg_getline_len(rex, clnum) - ccol
        };
        let mut captured = usize::try_from(ccol)
            .ok()
            .and_then(|at| captured_line.get(at..))
            .unwrap_or(PAST_THE_END)
            .to_vec();
        if captured.last() != Some(&0) {
            captured.push(0);
        }
        rex.set_line(reg_getline(rex, rex.lnum()).unwrap_or(PAST_THE_END));

        // `cstrncmp` can shorten `len` when a fold changed the encoded
        // length, and the caller is told how far to advance from it.
        let differs = if rex.reg_ic() {
            // A negative length was a huge `size_t` upstream: no bound.
            let n = usize::try_from(len).unwrap_or(usize::MAX);
            strnicmp_in(
                prefix_at(&captured, 0, n),
                prefix_at(rex.line(), rex.input(), n),
            ) != 0
        } else {
            cstrncmp(rex, &captured, rex.rest(), &mut len) != 0
        };
        if differs {
            return RA_NOMATCH;
        }
        if let Some(n) = bytelen.as_deref_mut() {
            *n += len;
        }
        if clnum == end_lnum {
            return RA_MATCH;
        }
        if rex.lnum() >= rex.reg_maxline() {
            return RA_NOMATCH;
        }
        // The capture continues on the next line, so the match must too, and
        // `*bytelen` restarts from that line's column 0.
        reg_nextline(rex);
        if let Some(n) = bytelen.as_deref_mut() {
            *n = 0;
        }
        clnum += 1;
        ccol = 0;
        if got_int.get() {
            return RA_FAIL;
        }
    }
}

/// Reject a repeat applied to `what`, a zero-width atom such as `\zs`.
pub(crate) fn re_mult_next(rc: &mut RegCompiler, what: &str) -> bool {
    if re_multi_type(peekchr(rc)) == MULTI_MULT {
        semsg!("E888: (NFA regexp) cannot repeat {what}");
        rc_did_emsg.set(true);
        return false;
    }
    true
}
