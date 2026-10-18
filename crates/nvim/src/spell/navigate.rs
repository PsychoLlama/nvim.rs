//! Walking the buffer to the next misspelled word.
//!
//! This is what `]s`, `[s`, `]S` and `[S` do, and what `spellbadword()`
//! uses to find the bad word on the current line. [`spell_move_to`] scans
//! line by line calling [`spell_check`] on every word, in whichever
//! direction was asked for, wrapping at the ends of the buffer when
//! `'wrapscan'` allows.
//!
//! Scanning always starts at the *beginning* of a line, even going
//! backwards, because a word's start cannot be found from its middle. For
//! backwards search the whole line is scanned and the last match before the
//! cursor is kept.
//!
//! A word split over a line break — "et<newline>cetera" — is handled by
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
//! Ephemeral decorations live in the global `decor_state`, so the scan
//! saves it, drives the `_on_spell_nav` callbacks itself, and restores it
//! on the way out.

#![forbid(unsafe_code)]

use crate::cstr::{byte_at, until_nul};
use crate::strings::has_char;
use crate::winlayer::Win;
use core::ffi::c_int;

use crate::charset::skip;
use crate::decoration::decor_state;
use crate::decoration::{DecorStateRef, decor_redraw_col, decor_redraw_line, decor_state_free};
use crate::decoration_provider::decor_providers_invoke_spell;
use crate::getchar::state::got_int;
use crate::message::give_warning;
use crate::message::state::{bot_top_msg, top_bot_msg};
use crate::option::shortmess;
use crate::option::vars::p_ws;
use crate::options::kOptSpoFlagNoplainbuffer;
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::pos::{MAXCOL, clearpos};
use crate::search::{BACKWARD, FORWARD};
use crate::syntax::syntax_present;
use crate::types::{ColNr, Hlf, LineNr, Pos, ShmFlag, SpellMoveType, size_t};

use super::chartab::tail;
use super::check::{check_need_cap, no_spell_checking, spell_check};
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
    state: DecorStateRef,
) -> Option<bool> {
    if *decor_lnum != lnum {
        window.decor_redraw_reset(state);
        decor_providers_invoke_spell(window, lnum as c_int - 1, col, lnum as c_int - 1, -1);
        decor_redraw_line(window, lnum as c_int - 1, state);
        *decor_lnum = lnum;
    }
    decor_redraw_col(window, col, 0, false, state, MAXCOL as c_int);
    state.spell
}

/// Whether the syntax at this position is one that gets spell-checked.
#[inline]
fn can_syn_spell(window: Win, lnum: LineNr, col: c_int) -> bool {
    let mut can_spell = false;
    window.syntax_id(lnum, col as ColNr, false, Some(&mut can_spell), false);
    can_spell
}

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
    mut window: Win,
    dir: c_int,
    behaviour: SpellMoveType,
    curline: bool,
    mut found_attr: Option<&mut Hlf>,
) -> size_t {
    if no_spell_checking(window) {
        return 0;
    }

    let mut found_pos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut found_len: size_t = 0;
    let mut attr: Hlf = HLF_COUNT;
    let has_syntax = syntax_present(window);
    // The line, the start of the next one after it, and a terminator; kept
    // between lines while it is long enough.
    let mut buf: Vec<u8> = Vec::new();
    let mut skip = 0;
    let mut capcol: ColNr = -1;
    let mut found_one = false;
    let mut wrapped = false;
    let mut ret: size_t = 0;
    let mut done = false;

    let mut lnum = window.w_cursor.lnum;
    clearpos(&mut found_pos);

    // Ephemeral extmarks live in the global decor_state, so it has to be
    // put aside and rebuilt per line here, then restored. The scan's own
    // state occupies the same cell -- a provider's ephemeral mark reaches
    // it from the API side, exactly as during a redraw -- so this is the
    // one acquisition, and the address it names does not move when the
    // contents are swapped.
    let (saved_decor_start, decor) = DecorStateRef::set_aside();
    let mut decor_lnum: LineNr = -1;

    let mut lines = window.buffer().lines();
    while !got_int.get() {
        let line_len = lines.line_len(lnum) as usize;
        if buf.len() < line_len + MAXWLEN + 2 {
            buf = vec![0; line_len + MAXWLEN + 2];
        }

        // The first word of the first line is always capital-checked.
        if lnum == 1 {
            capcol = 0;
        }

        if capcol == 0 {
            capcol = skip::white(lines.line(lnum)) as ColNr;
        } else if curline && window == Win::current() {
            // For spellbadword(): does the first word need a capital?
            // The borrow ends before the call, which reads the line above.
            let col = skip::white(lines.line(lnum)) as ColNr;
            if check_need_cap(Win::current(), lnum, col) {
                capcol = col;
            }
        }

        // Copy the line and append the start of the next one.
        let empty_line = {
            let line = lines.line(lnum);
            buf[..line_len].copy_from_slice(line);
            buf[line_len] = 0;
            skip::white(line) == line.len()
        };
        if lnum < window.w_buffer.b_ml.ml_line_count {
            let next = lines.line(lnum + 1);
            spell_cat_line(&mut buf[line_len..], next, MAXWLEN as c_int);
        }

        let mut p = skip;
        let endp = line_len;
        while p < endp {
            // Searching backwards, stop at the cursor — unless the search
            // already wrapped past the end of the buffer.
            if dir == BACKWARD
                && lnum == window.w_cursor.lnum
                && !wrapped
                && p as ColNr >= window.w_cursor.col
            {
                break;
            }

            let len;
            (len, attr) = spell_check(window, &buf[p..], Some(&mut capcol), false);

            if attr != HLF_COUNT
                && (behaviour == SMT_ALL
                    || (behaviour == SMT_BAD && attr == HLF_SPB)
                    || (behaviour == SMT_RARE && attr == HLF_SPR))
            {
                // Searching forward, only a bad word after the cursor
                // counts.
                let col = p as ColNr;
                let past_cursor = dir == BACKWARD
                    || lnum != window.w_cursor.lnum
                    || wrapped
                    || col + if curline { len as ColNr } else { 0 } > window.w_cursor.col;

                if past_cursor {
                    let no_plain_buffer =
                        window.syn_block().b_p_spo_flags & kOptSpoFlagNoplainbuffer != 0;
                    let mut can_spell = !no_plain_buffer;
                    let decor_says = decor_spell_nav_col(window, lnum, &mut decor_lnum, col, decor);
                    if decor_says == Some(true) {
                        can_spell = true;
                    } else if decor_says == Some(false) {
                        can_spell = false;
                    } else if has_syntax {
                        can_spell = can_syn_spell(window, lnum, col);
                    }

                    if !can_spell {
                        attr = HLF_COUNT;
                    } else {
                        found_one = true;
                        found_pos = Pos {
                            lnum,
                            col,
                            coladd: 0,
                        };
                        if dir == FORWARD {
                            // Nothing further to look for.
                            window.w_cursor = found_pos;
                            if let Some(found_attr) = found_attr.as_deref_mut() {
                                *found_attr = attr;
                            }
                            ret = len;
                            done = true;
                            break;
                        } else if curline {
                            // Insert-mode completion wants the cursor
                            // after the bad word.
                            found_pos.col += len as c_int;
                        }
                        found_len = len;
                    }
                } else {
                    found_one = true;
                }
            }

            // On to the character after the word.
            p += len;
            capcol -= len as c_int;
        }

        if done {
            break; // found it, going forward
        }

        if dir == BACKWARD && found_pos.lnum != 0 {
            // Take the last match in the line, before the cursor.
            window.w_cursor = found_pos;
            ret = found_len;
            break;
        }

        if curline {
            break; // only the cursor line was wanted
        }

        // Back at the starting line having searched it twice: give up.
        if lnum == window.w_cursor.lnum && wrapped {
            break;
        }

        if dir == BACKWARD {
            if lnum > 1 {
                lnum -= 1;
            } else if !p_ws() {
                break; // at the first line and 'nowrapscan'
            } else {
                // Wrap to the end. The starting line may be searched
                // again, to accept its last match.
                lnum = window.w_buffer.b_ml.ml_line_count;
                wrapped = true;
                if !shortmess(ShmFlag::SEARCH) {
                    give_warning(gettext(top_bot_msg), true, false);
                }
            }
            capcol = -1;
        } else {
            if lnum < window.w_buffer.b_ml.ml_line_count {
                lnum += 1;
            } else if !p_ws() {
                break; // at the last line and 'nowrapscan'
            } else {
                // Wrap to the start. The starting line may be searched
                // again, to accept its first match.
                lnum = 1;
                wrapped = true;
                if !shortmess(ShmFlag::SEARCH) {
                    give_warning(gettext(bot_top_msg), true, false);
                }
            }

            // Back at the starting line with nothing found: give up.
            if lnum == window.w_cursor.lnum && !found_one {
                break;
            }

            // Skip whatever of the next line a match already covered.
            skip = if attr == HLF_COUNT { p - endp } else { 0 };

            // One column for the space standing in for the line break.
            capcol -= 1;

            // After an empty line, check the next line's first word.
            if empty_line {
                capcol = 0;
            }
        }

        line_breakcheck();
    }

    decor_state_free(decor);
    decor_state.with_mut(|state| *state = saved_decor_start);
    ret
}

/// Append the start of `line` to `buf`, blanking out the leading white
/// space and any comment leaders, so that a word split over a line break
/// can still be looked up.
///
/// The blanks are kept rather than dropped so that the caller's columns
/// still refer to the same places. At most `maxlen` bytes of `buf` are
/// written, terminator included; `line` ends at its end or at a NUL.
pub fn spell_cat_line(buf: &mut [u8], line: &[u8], maxlen: c_int) {
    let maxlen = usize::try_from(maxlen).unwrap_or(0);
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
