//! The match-time context: the lines an engine reads, the capture slots it
//! clears, and the position tests (`\%V`, `\<`, a back-reference) both
//! engines ask this layer rather than answering themselves.
//!
//! `rex` is that context. It is a global rather than a parameter because
//! the engines reach it from everywhere; [`super::api`] saves and restores
//! it around a nested match, which is what lets a `\=` expression run a
//! search of its own.
//!
//! Its fields are read through `GlobalCell::ptr`, not `with`/`with_mut`.
//! Everything here is on the per-character path of both engines, and
//! `with` is an outlined call that pushes and pops a debug borrow-table
//! entry: routing these accesses through it measurably slowed syntax
//! highlighting — enough to trip 'redrawtime' on a large file, which
//! disables highlighting for the buffer. `ptr` carries exactly the
//! obligations the C did.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::option::vars::P_SEL;
use crate::regexp::RegCompiler;
use core::ffi::{CStr, c_char, c_int};

use super::submatch::Rsm;
use super::{
    BtProg, MULTI_MULT, NSUBEXP, RA_FAIL, RA_MATCH, RA_NOMATCH, REGMAGIC, Rex, cstrncmp,
    nfa_regengine, peekchr, re_multi_type,
};
use crate::charset::vim_iswordc_buf;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::mbyte::{mb_get_class_tab, mb_strnicmp, utf_head_off};
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::message::e_re_corr;
use crate::message::emsg;
use crate::normal::{VisualMode, visual_ever_started, visual_selection};
use crate::os::cshim::gettext;
use crate::os::input::fast_breakcheck;
use crate::plines::{getvvcol, win_linetabsize};
use crate::pos::{MAXCOL, lt};
use crate::regexp::RE_NOBREAK;
use crate::regexp::state::rc_did_emsg;
use crate::semsg;
use crate::types::{ColNr, LPos, LineNr, RegMMatch, RegMatch, uint8_t};

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

/// Which line numbering to resolve against: the running match, or the
/// snapshot `submatch()` and a `\=` expression see.
#[derive(Clone, Copy)]
pub(crate) enum LineOrigin {
    Exec,
    Submatch,
}

impl LineOrigin {
    /// The buffer line the origin's line 0 sits on.
    fn first(self, rex: Rex) -> LineNr {
        match self {
            LineOrigin::Exec => rex.reg_firstlnum(),
            // SAFETY: `can_f_submatch` gates every path that gets here.
            LineOrigin::Submatch => unsafe { Rsm::acquire() }.firstlnum(),
        }
    }

    /// The last line the origin reaches, relative to [`LineOrigin::first`].
    fn maxline(self, rex: Rex) -> LineNr {
        match self {
            LineOrigin::Exec => rex.reg_maxline(),
            // SAFETY: as `first`.
            LineOrigin::Submatch => unsafe { Rsm::acquire() }.maxline(),
        }
    }
}

/// Where a line relative to the match's first line lands.
enum Located {
    /// Above line 1 — the caller asked for context before the buffer.
    Before,
    /// Past the match's last line.
    Past,
    At(LineNr),
}

fn locate(lnum: LineNr, first: LineNr, maxline: LineNr) -> Located {
    if first + lnum < 1 {
        Located::Before
    } else if lnum > maxline {
        Located::Past
    } else {
        Located::At(first + lnum)
    }
}

/// The text `lnum` lines into the match: NULL above the buffer, an empty
/// string past the match's last line. Note that a submatch line comes from
/// `rex`'s buffer even though its numbering comes from `rsm`.
pub(crate) fn reg_line(rex: Rex, lnum: LineNr, origin: LineOrigin) -> *mut c_char {
    match locate(lnum, origin.first(rex), origin.maxline(rex)) {
        Located::Before => core::ptr::null_mut(),
        Located::Past => c"".as_ptr().cast_mut(),
        // SAFETY: `reg_buf` is the buffer being matched and `locate` has
        // established the line is in it.
        Located::At(lnum) => unsafe { ml_get_buf(rex.reg_buf(), lnum) },
    }
}

/// The length of [`reg_line`]'s text; 0 for either stand-in.
pub(crate) fn reg_line_len(rex: Rex, lnum: LineNr, origin: LineOrigin) -> ColNr {
    match locate(lnum, origin.first(rex), origin.maxline(rex)) {
        Located::Before | Located::Past => 0,
        Located::At(lnum) => ml_get_buf_len(rex.reg_buf(), lnum),
    }
}

pub(crate) fn reg_getline(rex: Rex, lnum: LineNr) -> *mut c_char {
    reg_line(rex, lnum, LineOrigin::Exec)
}

pub(crate) fn reg_getline_len(rex: Rex, lnum: LineNr) -> ColNr {
    reg_line_len(rex, lnum, LineOrigin::Exec)
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
        let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
        self.matches[no] = Some(cstr::owned(&text[..end]));
    }
}

/// The `\z` traffic of one syntax match: the captures of the region's
/// start match that `\z1`..`\z9` read, and the slot the match's own `\z(`
/// captures go to when it succeeds.
pub(crate) struct ExtMatchIo<'a> {
    pub(crate) input: Option<&'a ExtMatch>,
    pub(crate) output: &'a mut Option<ExtMatchRef>,
}

/// The character class of the character before the cursor, or -1 at the
/// start of the line. Backs `\<` and `\>`.
pub(crate) fn reg_prev_class(rex: Rex) -> c_int {
    if rex.col() <= 0 {
        return -1;
    }
    // SAFETY: the cursor is past the start of the line being matched, so the
    // byte before it is in the line and `utf_head_off` walks back no further
    // than `line`; `reg_buf` is the buffer whose 'iskeyword' applies.
    let line = rex.line().cast::<c_char>();
    let prev = unsafe { rex.input_str().sub(1) };
    let chartab = (&raw mut rex.reg_buf().b_chartab).cast::<u64>();
    unsafe { mb_get_class_tab(prev.sub(utf_head_off(line, prev) as usize), chartab) }
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
        let (mut start, mut end, mut start2, mut end2) = (0, 0, 0, 0);
        let (mut top, mut bot) = (top, bot);
        // SAFETY: `wp` is a live window and the two positions are in its
        // buffer; the null argument is the "start of the character" output
        // this caller does not want.
        let nul = core::ptr::null_mut();
        unsafe { getvvcol(wp, &raw mut top, &raw mut start, nul, &raw mut end) };
        unsafe { getvvcol(wp, &raw mut bot, &raw mut start2, nul, &raw mut end2) };
        start = start.min(start2);
        end = end.max(end2);
        // `$` in blockwise Visual stretches the block to end of line.
        if top.col == MAXCOL as c_int || bot.col == MAXCOL as c_int || curswant == MAXCOL as c_int {
            end = MAXCOL as c_int;
        }
        // `getvvcol` can have moved the line out from under the match.
        let line = reg_getline(rex, rex.lnum()).cast::<uint8_t>();
        rex.seek(line, col);
        // SAFETY: `line` is the NUL-terminated line just fetched and `col` a
        // byte offset into it.
        let cols = unsafe { win_linetabsize(wp, rex.buf_lnum(), line.cast(), col) };
        cols >= start && cols <= end - ColNr::from(P_SEL.first_byte() == b'e')
    } else {
        true
    }
}

/// Does the running program belong to the backtracking engine, and has its
/// magic number survived? Only that engine's programs carry one.
pub(crate) fn prog_magic_wrong(rex: Rex) -> c_int {
    // SAFETY: a running match holds a live program.
    let nfa = unsafe { (*rex.regprog()).engine.cast_const() } == &raw const nfa_regengine;
    let wrong = !nfa && BtProg::of_match(rex).is_some_and(|p| p.magic() != REGMAGIC);
    if wrong {
        emsg(gettext(e_re_corr));
        return 1;
    }
    0
}

/// Reset the `\1`..`\9` slots, or with `z` the `\z1`..`\z9` ones — which
/// live in this module's own arrays rather than in the caller's match
/// structure. Lazy: an engine only pays for this if a match reaches a
/// back-reference.
fn cleanup(rex: Rex, z: bool) {
    let need = if z {
        rex.need_clear_zsubexpr()
    } else {
        rex.need_clear_subexpr()
    };
    if need == 0 {
        return;
    }
    // A buffer match records positions and marks a slot unset with -1; a
    // string match records pointers and marks it unset with NULL.
    let n = NSUBEXP as usize;
    if z {
        // The `\z(` slots are the context's own, so blanking them is a
        // whole-value write rather than a walk over the caller's memory.
        rex.clear_zslots();
    } else if rex.multi() {
        // SAFETY: the caller's match structure holds NSUBEXP of each.
        blank(
            unsafe { core::slice::from_raw_parts_mut(rex.reg_startpos(), n) },
            UNSET_POS,
        );
        blank(
            unsafe { core::slice::from_raw_parts_mut(rex.reg_endpos(), n) },
            UNSET_POS,
        );
    } else {
        // SAFETY: as above.
        blank(
            unsafe { core::slice::from_raw_parts_mut(rex.reg_startp(), n) },
            core::ptr::null_mut::<uint8_t>(),
        );
        blank(
            unsafe { core::slice::from_raw_parts_mut(rex.reg_endp(), n) },
            core::ptr::null_mut::<uint8_t>(),
        );
    }
    if z {
        rex.set_need_clear_zsubexpr(0);
    } else {
        rex.set_need_clear_subexpr(0);
    }
}

/// What a buffer match's unset capture slot holds.
const UNSET_POS: LPos = LPos { lnum: -1, col: -1 };

fn blank<T: Copy>(slots: &mut [T], unset: T) {
    slots.fill(unset);
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
    rex.set_line(reg_getline(rex, rex.lnum()).cast());
    rex.set_input(rex.line());
    reg_breakcheck(rex);
}

/// Match the text a back-reference captured, which may span lines. On
/// success `*bytelen` is the length matched on the *last* line, which is
/// what the caller advances by.
///
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
        // `reg_getline` below hands out the memline's own buffer, so it can
        // invalidate the line being matched. Take a private copy first.
        take_line_copy(rex);

        let p = reg_getline(rex, clnum);
        debug_assert!(!p.is_null(), "p");
        let mut len = if clnum == end_lnum {
            end_col - ccol
        } else {
            reg_getline_len(rex, clnum) - ccol
        };
        // `cstrncmp` can shorten `len` when a fold changed the encoded
        // length, and the caller is told how far to advance from it.
        // SAFETY: `p` is the captured line and `ccol` a column in it; the
        // cursor and both strings are NUL-terminated, so both compares stop.
        let differs = unsafe {
            let captured = p.offset(ccol as isize);
            if rex.reg_ic() {
                mb_strnicmp(captured, rex.input_str(), len as usize) != 0
            } else {
                cstrncmp(rex, captured, rex.input_str(), &mut len) != 0
            }
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

/// The copy of the line a back-reference match works over: see
/// [`take_line_copy`]. Kept between matches so an ordinary one never
/// allocates; [`trim_line_copy`] gives back what a long line grew it to.
static LINE_COPY: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());

/// How much of [`LINE_COPY`] may be left lying around between matches.
const LINE_COPY_KEEP: usize = 400;

/// Move the line being matched into this module's scratch buffer and
/// re-anchor the cursor onto the copy.
///
/// Fetching another line hands out the memline's own buffer, which can
/// invalidate the one the match is standing on; the copy is what makes a
/// back-reference able to read both.
fn take_line_copy(rex: Rex) {
    let col = rex.col();
    LINE_COPY.with_mut(|copy| {
        if rex.line() == copy.as_mut_ptr() {
            return;
        }
        // SAFETY: `rex.line` is the NUL-terminated line being matched, and
        // not the copy, so refilling the copy cannot move it.
        let line = unsafe { cstr::bytes_at(rex.line().cast()) };
        copy.clear();
        copy.extend_from_slice(line);
        copy.push(0);
        rex.seek(copy.as_mut_ptr(), col);
    });
}

/// Give back the line copy if a long line grew it past what is worth
/// keeping. Called between matches only: the running match's line may be
/// the copy.
pub(crate) fn trim_line_copy() {
    LINE_COPY.with_mut(|copy| {
        if copy.capacity() > LINE_COPY_KEEP {
            *copy = Vec::new();
        }
    });
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

/// Point the context at a string match about to run. `line_lbr` says the
/// text holds newlines to be matched rather than ends of line.
///
/// `rmp` must be live, with a compiled program, for the match's duration.
pub(crate) fn init_regexec(rex: Rex, rmp: *mut RegMatch, line_lbr: bool) {
    rex.set_reg_match(rmp);
    rex.set_reg_mmatch(core::ptr::null_mut::<RegMMatch>());
    rex.set_reg_maxline(0);
    rex.set_reg_line_lbr(line_lbr);
    // A string match has no buffer of its own, but `\k` and friends still
    // need an 'iskeyword' to read.
    rex.set_reg_buf(Buf::current());
    rex.set_reg_win(None);
    // SAFETY: the caller's match structure, live with a program.
    rex.set_reg_ic(unsafe { (*rmp).rm_ic });
    rex.set_reg_nobreak(unsafe { (*(*rmp).regprog).re_flags } & RE_NOBREAK as u32 != 0);
    rex.set_reg_icombine(false);
    rex.set_reg_maxcol(0);
}

/// Point the context at a buffer match about to run.
///
/// `rmp` must be live, with a compiled program, for the match's duration,
/// and `buffer` must be the buffer it runs over.
pub(crate) fn init_regexec_multi(
    rex: Rex,
    rmp: *mut RegMMatch,
    win: Option<Win>,
    buffer: Buf,
    lnum: LineNr,
) {
    rex.set_reg_match(core::ptr::null_mut::<RegMatch>());
    rex.set_reg_mmatch(rmp);
    rex.set_reg_buf(buffer);
    rex.set_reg_win(win);
    rex.set_reg_firstlnum(lnum);
    rex.set_reg_line_lbr(false);
    rex.set_reg_icombine(false);
    // SAFETY: the caller's match structure and buffer, live for the match.
    rex.set_reg_maxline(buffer.b_ml.ml_line_count - lnum);
    rex.set_reg_ic(unsafe { (*rmp).rmm_ic } != 0);
    rex.set_reg_nobreak(unsafe { (*(*rmp).regprog).re_flags } & RE_NOBREAK as u32 != 0);
    rex.set_reg_maxcol(unsafe { (*rmp).rmm_maxcol });
}
