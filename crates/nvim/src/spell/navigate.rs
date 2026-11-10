//! Walking the buffer to the next misspelled word.
//!
//! This is what `]s`, `[s`, `]S` and `[S` do, and what `spellbadword()`
//! uses to find the bad word on the current line. [`Win::spell_move_to`] scans
//! line by line calling [`Win::spell_check`] on every word, in whichever
//! direction was asked for, wrapping at the ends of the buffer when
//! `'wrapscan'` allows.
//!
//! Scanning always starts at the *beginning* of a line, even going
//! backwards, because a word's start cannot be found from its middle. For
//! backwards search the whole line is scanned and the last match before the
//! cursor is kept.
//!
//! A word split over a line break — "`et<newline>`cetera" — is handled by
//! appending the start of the next line to the buffer being scanned
//! ([`spell_cat_line`]), with the leading white space kept so that columns
//! still line up.
//!
//! # Whether a bad word counts
//!
//! Syntax highlighting and decoration providers can both declare that a
//! region is not spell-checked. Decorations win where they have an
//! opinion; otherwise syntax decides, and with `'spelloptions'`
//! `noplainbuffer` a region nothing claims is not checked at all.
//! Ephemeral decorations live in the one decoration state, so the scan
//! saves it, drives the `_on_spell_nav` callbacks itself, and restores it
//! on the way out.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr::{byte_at, until_nul};
use crate::memline::Lines;
use crate::narrow::len_as_int;
use crate::strings::has_char;
use crate::winlayer::Win;
use core::ffi::c_int;

use crate::charset::skip;
use crate::decoration::{
    decor_redraw_col, decor_redraw_line, decor_spell, decor_state_restore, decor_state_set_aside,
};
use crate::decoration_provider::decor_providers_invoke_spell;
use crate::getchar::state::GOT_INT;
use crate::highlight::Hlf;
use crate::message::give_warning;
use crate::message::state::{BOT_TOP_MSG, TOP_BOT_MSG};
use crate::option::defs::ShmFlag;
use crate::option::shortmess;
use crate::option::vars::p_ws;
use crate::options::OPT_SPO_FLAG_NOPLAINBUFFER;
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::os::libc::size_t;
use crate::pos::MAXCOL;
use crate::search::{BACKWARD, FORWARD};
use crate::spell::SpellMoveType;
use crate::syntax::syntax_present;
use crate::types::{ColNr, LineNr, Pos};

use super::chartab::tail;
use super::check::{check_need_cap, no_spell_checking};
use super::{MAXWLEN, SMT_BAD, SMT_RARE};
use crate::highlight_group::{HLF_COUNT, HLF_SPB, HLF_SPR};
use crate::spell::SMT_ALL;

/// Ask the decoration providers whether column `col` of line `lnum` is
/// spell-checked, running the `_on_spell_nav` callbacks once per line.
fn decor_spell_nav_col(
    window: Win,
    lnum: LineNr,
    decor_lnum: &mut LineNr,
    col: c_int,
) -> Option<bool> {
    if *decor_lnum != lnum {
        window.decor_redraw_reset();
        decor_providers_invoke_spell(window, lnum.get() - 1, col, lnum.get() - 1, -1);
        decor_redraw_line(window, lnum.get() - 1);
        *decor_lnum = lnum;
    }
    decor_redraw_col(window, col, 0, false, MAXCOL);
    decor_spell()
}

/// Whether the syntax at this position is one that gets spell-checked.
#[inline]
fn can_syn_spell(window: Win, lnum: LineNr, col: c_int) -> bool {
    let mut can_spell = false;
    window.syntax_id(lnum, ColNr::new(col), false, Some(&mut can_spell), false);
    can_spell
}

impl Win {
    /// Move the cursor to the next spelling error.
    ///
    /// `dir` is `FORWARD` or `BACKWARD`; `behaviour` selects which kinds of bad
    /// word count (`SMT_ALL`, `SMT_BAD` for `]S`, `SMT_RARE`); `curline` limits
    /// the search to the cursor line, which is what `spellbadword()` and
    /// Insert-mode completion want. `found_attr`, when given and searching forward,
    /// receives the highlight of the word found.
    ///
    /// Returns the length of the bad word, or 0 if none was found.
    pub fn spell_move_to(
        mut self,
        dir: c_int,
        behaviour: SpellMoveType,
        curline: bool,
        found_attr: Option<&mut Hlf>,
    ) -> size_t {
        if no_spell_checking(self) {
            return 0;
        }

        let mut scan = SpellScan {
            dir,
            behaviour,
            curline,
            has_syntax: syntax_present(self),
            lnum: self.w_cursor.lnum,
            buf: Vec::new(),
            skip: 0,
            capcol: ColNr::NONE,
            attr: HLF_COUNT,
            found_pos: Pos::default(),
            found_len: 0,
            found_one: false,
            wrapped: false,
            decor_lnum: LineNr::new(-1),
        };
        let mut ret: size_t = 0;

        // Ephemeral extmarks live in the one decoration state, so it has to be
        // put aside and rebuilt per line here, then restored. The scan's own
        // state occupies the same cell -- a provider's ephemeral mark reaches
        // it from the API side, exactly as during a redraw.
        let saved_decor_start = decor_state_set_aside();

        let mut lines = self.buffer().lines();
        while !GOT_INT.get() {
            let (line_len, blank) = scan.load_line(self, &mut lines);
            let p = match scan.scan_line(self, line_len) {
                LineScan::Found(len) => {
                    self.w_cursor = scan.found_pos;
                    if let Some(found_attr) = found_attr {
                        *found_attr = scan.attr;
                    }
                    ret = len;
                    break;
                }
                LineScan::Ended(p) => p,
            };

            if dir == BACKWARD && scan.found_pos.lnum != LineNr::ZERO {
                // Take the last match in the line, before the cursor.
                self.w_cursor = scan.found_pos;
                ret = scan.found_len;
                break;
            }
            if curline {
                break; // only the cursor line was wanted
            }
            if !scan.next_line(self, p, line_len, blank) {
                break;
            }
            line_breakcheck();
        }

        decor_state_restore(saved_decor_start);
        ret
    }
}

/// What [`Win::spell_move_to`] carries from one line to the next.
#[derive(Debug)]
struct SpellScan {
    dir: c_int,
    behaviour: SpellMoveType,
    curline: bool,
    has_syntax: bool,
    /// The line being scanned.
    lnum: LineNr,
    /// The line, the start of the next one after it, and a terminator; kept
    /// between lines while it is long enough.
    buf: Vec<u8>,
    /// How much of the line a match on the line before already covered.
    skip: usize,
    /// The column whose word must start with a capital, counted from the
    /// scan's position (see [`Win::spell_check`]).
    capcol: ColNr,
    /// The highlight of the last word checked: `HLF_COUNT` when it was good
    /// or did not count.
    attr: Hlf,
    /// Searching backwards: the last bad word before the cursor so far, and
    /// its length.
    found_pos: Pos,
    found_len: size_t,
    /// Whether any bad word was seen, counted or not.
    found_one: bool,
    /// Whether the scan went past an end of the buffer.
    wrapped: bool,
    /// The line the decoration providers were last asked about.
    decor_lnum: LineNr,
}

/// What scanning one line came to.
enum LineScan {
    /// Searching forward, a bad word that counts, this long, at
    /// [`SpellScan::found_pos`]: the search is over.
    Found(size_t),
    /// The scan stopped at this offset into the buffer: the end of the line,
    /// or (searching backwards) the cursor.
    Ended(usize),
}

impl SpellScan {
    /// Copy line `self.lnum` into the buffer with the start of the next one
    /// after it, and work out whether its first word needs a capital.
    /// Answers the line's length and whether it is blank.
    fn load_line(&mut self, window: Win, lines: &mut Lines) -> (usize, bool) {
        let lnum = self.lnum;
        let line_len = lines.line_len(lnum).as_usize();
        if self.buf.len() < line_len + MAXWLEN + 2 {
            self.buf = vec![0; line_len + MAXWLEN + 2];
        }

        // The first word of the first line is always capital-checked.
        if lnum == LineNr::ONE {
            self.capcol = ColNr::ZERO;
        }

        if self.capcol == ColNr::ZERO {
            self.capcol = ColNr::from_index(skip::white(lines.line(lnum)));
        } else if self.curline && window == Win::current() {
            // For spellbadword(): does the first word need a capital?
            // The borrow ends before the call, which reads the line above.
            let col = ColNr::from_index(skip::white(lines.line(lnum)));
            if check_need_cap(Win::current(), lnum, col) {
                self.capcol = col;
            }
        }

        // Copy the line and append the start of the next one.
        let blank = {
            let line = lines.line(lnum);
            self.buf[..line_len].copy_from_slice(line);
            self.buf[line_len] = 0;
            skip::white(line) == line.len()
        };
        if lnum < window.w_buffer.b_ml.ml_line_count {
            let next = lines.line(lnum + 1);
            spell_cat_line(&mut self.buf[line_len..], next, MAXWLEN);
        }
        (line_len, blank)
    }

    /// Check the words of the loaded line, from [`skip`](Self::skip) to
    /// `endp`, and keep (or, forward, stop at) the bad ones that count.
    fn scan_line(&mut self, window: Win, endp: usize) -> LineScan {
        let lnum = self.lnum;
        let mut p = self.skip;
        while p < endp {
            // Searching backwards, stop at the cursor — unless the search
            // already wrapped past the end of the buffer.
            if self.dir == BACKWARD
                && lnum == window.w_cursor.lnum
                && !self.wrapped
                && ColNr::from_index(p) >= window.w_cursor.col
            {
                break;
            }

            let len;
            (len, self.attr) = window.spell_check(&self.buf[p..], Some(&mut self.capcol), false);

            if self.attr != HLF_COUNT && self.counts(self.attr) {
                // Searching forward, only a bad word after the cursor
                // counts.
                let col = ColNr::from_index(p);
                let past_cursor = self.dir == BACKWARD
                    || lnum != window.w_cursor.lnum
                    || self.wrapped
                    || col + if self.curline { len_as_int(len) } else { 0 } > window.w_cursor.col;

                if !past_cursor {
                    self.found_one = true;
                } else if !self.can_spell_at(window, col) {
                    self.attr = HLF_COUNT;
                } else {
                    self.found_one = true;
                    self.found_pos = Pos {
                        lnum,
                        col,
                        coladd: 0,
                    };
                    if self.dir == FORWARD {
                        // Nothing further to look for.
                        return LineScan::Found(len);
                    } else if self.curline {
                        // Insert-mode completion wants the cursor after
                        // the bad word.
                        self.found_pos.col += len_as_int(len);
                    }
                    self.found_len = len;
                }
            }

            // On to the character after the word.
            p += len;
            self.capcol -= len_as_int(len);
        }
        LineScan::Ended(p)
    }

    /// Whether a bad word highlighted `attr` is one this search is after.
    fn counts(&self, attr: Hlf) -> bool {
        self.behaviour == SMT_ALL
            || (self.behaviour == SMT_BAD && attr == HLF_SPB)
            || (self.behaviour == SMT_RARE && attr == HLF_SPR)
    }

    /// Whether the text at `col` of the line is spell-checked at all:
    /// decorations decide where they have an opinion, then syntax, then
    /// 'spelloptions'.
    fn can_spell_at(&mut self, window: Win, col: ColNr) -> bool {
        let decor_says = decor_spell_nav_col(window, self.lnum, &mut self.decor_lnum, col.get());
        match decor_says {
            Some(says) => says,
            None if self.has_syntax => can_syn_spell(window, self.lnum, col.get()),
            None => window.syn_block().b_p_spo_flags & OPT_SPO_FLAG_NOPLAINBUFFER == 0,
        }
    }

    /// Step to the next line to scan, wrapping at an end of the buffer when
    /// 'wrapscan' allows. `p` is where the scan of this line stopped, `endp`
    /// the line's length and `blank` whether it was blank. Answers `false`
    /// when the search is over.
    fn next_line(&mut self, window: Win, p: usize, endp: usize, blank: bool) -> bool {
        let start = window.w_cursor.lnum;
        // Back at the starting line having searched it twice: give up.
        if self.lnum == start && self.wrapped {
            return false;
        }
        let line_count = window.w_buffer.b_ml.ml_line_count;

        if self.dir == BACKWARD {
            if self.lnum > LineNr::ONE {
                self.lnum -= 1;
            } else if !p_ws() {
                return false; // at the first line and 'nowrapscan'
            } else {
                // Wrap to the end. The starting line may be searched again,
                // to accept its last match.
                self.lnum = line_count;
                self.wrapped = true;
                if !shortmess(ShmFlag::SEARCH) {
                    give_warning(gettext(TOP_BOT_MSG), true, false);
                }
            }
            self.capcol = ColNr::NONE;
            return true;
        }

        if self.lnum < line_count {
            self.lnum += 1;
        } else if !p_ws() {
            return false; // at the last line and 'nowrapscan'
        } else {
            // Wrap to the start. The starting line may be searched again, to
            // accept its first match.
            self.lnum = LineNr::ONE;
            self.wrapped = true;
            if !shortmess(ShmFlag::SEARCH) {
                give_warning(gettext(BOT_TOP_MSG), true, false);
            }
        }

        // Back at the starting line with nothing found: give up.
        if self.lnum == start && !self.found_one {
            return false;
        }

        // Skip whatever of the next line a match already covered.
        self.skip = if self.attr == HLF_COUNT { p - endp } else { 0 };

        // One column for the space standing in for the line break.
        self.capcol -= 1;

        // After an empty line, check the next line's first word.
        if blank {
            self.capcol = ColNr::ZERO;
        }
        true
    }
}

/// Append the start of `line` to `buf`, blanking out the leading white
/// space and any comment leaders, so that a word split over a line break
/// can still be looked up.
///
/// The blanks are kept rather than dropped so that the caller's columns
/// still refer to the same places. At most `maxlen` bytes of `buf` are
/// written, terminator included; `line` ends at its end or at a NUL.
pub fn spell_cat_line(buf: &mut [u8], line: &[u8], maxlen: usize) {
    let mut p = skip::white(line);
    while has_char(c"*#/\"\t", c_int::from(byte_at(line, p))) {
        p += 1;
        p += skip::white(tail(line, p));
    }

    if byte_at(line, p) == 0 {
        return;
    }

    // Only worth appending when there is more than white space to
    // append.
    let n = p + 1;
    if n + 1 < maxlen {
        buf[..n].fill(b' ');
        let rest = until_nul(tail(line, p));
        let take = rest.len().min(maxlen - n - 1);
        buf[n..n + take].copy_from_slice(&rest[..take]);
        buf[n + take] = 0;
    }
}
