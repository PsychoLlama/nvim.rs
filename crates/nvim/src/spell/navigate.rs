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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::mem;

use crate::charset::{getwhitecols, skipwhite};
use crate::decoration::{
    DecorStateRef, decor_redraw_col, decor_redraw_line, decor_redraw_reset, decor_state_free,
};
use crate::decoration_provider::decor_providers_invoke_spell;
use crate::main::{bot_top_msg, curwin, decor_state, got_int, p_ws, top_bot_msg};
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::memory::{xfree, xmalloc, xstrlcpy};
use crate::message::give_warning;
use crate::option::shortmess;
use crate::options::kOptSpoFlagNoplainbuffer;
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::pos::{MAXCOL, clearpos};
use crate::search::{BACKWARD, FORWARD};
use crate::strings::vim_strchr;
use crate::syntax::{syn_get_id, syntax_present};
use crate::types::{NUL, ShmFlag, colnr_T, hlf_T, linenr_T, pos_T, size_t, smt_T, uint8_t, win_T};
use ::libc::{memset, strcpy, strlen};

use super::check::{check_need_cap, no_spell_checking, spell_check};
use super::{MAXWLEN, SMT_BAD, SMT_RARE};
use crate::highlight_group::{HLF_COUNT, HLF_SPB, HLF_SPR};
use crate::spell::SMT_ALL;

/// Ask the decoration providers whether column `col` of line `lnum` is
/// spell-checked, running the `_on_spell_nav` callbacks once per line.
///
/// # Safety
/// `wp` must be a live window and `state` the scan's own decoration state.
unsafe fn decor_spell_nav_col(
    wp: *mut win_T,
    lnum: linenr_T,
    decor_lnum: &mut linenr_T,
    col: c_int,
    state: DecorStateRef,
) -> Option<bool> {
    // SAFETY: the caller's window and state; the callbacks run Lua.
    unsafe {
        if *decor_lnum != lnum {
            decor_redraw_reset(wp, state);
            decor_providers_invoke_spell(wp, lnum as c_int - 1, col, lnum as c_int - 1, -1);
            decor_redraw_line(wp, lnum as c_int - 1, state);
            *decor_lnum = lnum;
        }
        decor_redraw_col(wp, col, 0, false, state, MAXCOL as c_int);
        state.spell
    }
}

/// Whether the syntax at this position is one that gets spell-checked.
#[inline]
unsafe fn can_syn_spell(wp: *mut win_T, lnum: linenr_T, col: c_int) -> bool {
    unsafe {
        let mut can_spell = false;
        syn_get_id(wp, lnum, col as colnr_T, 0, &raw mut can_spell, 0);
        can_spell
    }
}

/// Move the cursor to the next spelling error.
///
/// `dir` is `FORWARD` or `BACKWARD`; `behaviour` selects which kinds of bad
/// word count (`SMT_ALL`, `SMT_BAD` for `]S`, `SMT_RARE`); `curline` limits
/// the search to the cursor line, which is what `spellbadword()` and
/// Insert-mode completion want. `attrp`, when not null and searching
/// forward, receives the highlight of the word found.
///
/// Returns the length of the bad word, or 0 if none was found.
pub unsafe fn spell_move_to(
    wp: *mut win_T,
    dir: c_int,
    behaviour: smt_T,
    curline: bool,
    attrp: *mut hlf_T,
) -> size_t {
    unsafe {
        if no_spell_checking(wp) {
            return 0;
        }

        let mut found_pos: pos_T = mem::zeroed();
        let mut found_len: size_t = 0;
        let mut attr: hlf_T = HLF_COUNT;
        let has_syntax = syntax_present(wp);
        let mut buf: *mut c_char = core::ptr::null_mut();
        let mut buflen: size_t = 0;
        let mut skip = 0;
        let mut capcol: colnr_T = -1;
        let mut found_one = false;
        let mut wrapped = false;
        let mut ret: size_t = 0;
        let mut done = false;

        let mut lnum = (*wp).w_cursor.lnum;
        clearpos(&mut found_pos);

        // Ephemeral extmarks live in the global decor_state, so it has to be
        // put aside and rebuilt per line here, then restored. The scan's own
        // state occupies the same cell -- a provider's ephemeral mark reaches
        // it from the API side, exactly as during a redraw -- so this is the
        // one acquisition, and the address it names does not move when the
        // contents are swapped.
        let (saved_decor_start, decor) = (decor_state.take(), DecorStateRef::current());
        let mut decor_lnum: linenr_T = -1;

        while !got_int.get() {
            let mut line = ml_get_buf((*wp).w_buffer, lnum);
            let mut len = ml_get_buf_len((*wp).w_buffer, lnum) as size_t;
            if buflen < len + MAXWLEN as size_t + 2 {
                xfree(buf as *mut core::ffi::c_void);
                buflen = len + MAXWLEN as size_t + 2;
                buf = xmalloc(buflen) as *mut c_char;
            }

            // The first word of the first line is always capital-checked.
            if lnum == 1 {
                capcol = 0;
            }

            if capcol == 0 {
                capcol = getwhitecols(line) as colnr_T;
            } else if curline && wp == curwin.get() {
                // For spellbadword(): does the first word need a capital?
                let col = getwhitecols(line) as colnr_T;
                if check_need_cap(curwin.get(), lnum, col) {
                    capcol = col;
                }
                // check_need_cap() looked at the previous line, so the line
                // pointer has to be taken again.
                line = ml_get_buf((*wp).w_buffer, lnum);
            }

            // Copy the line and append the start of the next one. The
            // ml_get_buf() below can invalidate "line", so the empty test
            // comes first.
            let empty_line = *skipwhite(line) == 0;
            strcpy(buf, line);
            if lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
                spell_cat_line(
                    buf.add(strlen(buf)),
                    ml_get_buf((*wp).w_buffer, lnum + 1),
                    MAXWLEN as c_int,
                );
            }

            let mut p = buf.offset(skip as isize);
            let endp = buf.add(len);
            while p < endp {
                // Searching backwards, stop at the cursor — unless the search
                // already wrapped past the end of the buffer.
                if dir == BACKWARD
                    && lnum == (*wp).w_cursor.lnum
                    && !wrapped
                    && p.offset_from(buf) as colnr_T >= (*wp).w_cursor.col
                {
                    break;
                }

                attr = HLF_COUNT;
                len = spell_check(wp, p, &raw mut attr, &raw mut capcol, false);

                if attr != HLF_COUNT
                    && (behaviour == SMT_ALL
                        || (behaviour == SMT_BAD && attr == HLF_SPB)
                        || (behaviour == SMT_RARE && attr == HLF_SPR))
                {
                    // Searching forward, only a bad word after the cursor
                    // counts.
                    let past_cursor = dir == BACKWARD
                        || lnum != (*wp).w_cursor.lnum
                        || wrapped
                        || (if curline {
                            p.offset_from(buf) + len as isize
                        } else {
                            p.offset_from(buf)
                        }) as colnr_T
                            > (*wp).w_cursor.col;

                    if past_cursor {
                        let col = p.offset_from(buf) as colnr_T;

                        let no_plain_buffer =
                            (*(*wp).w_s).b_p_spo_flags & kOptSpoFlagNoplainbuffer != 0;
                        let mut can_spell = !no_plain_buffer;
                        let decor_says = decor_spell_nav_col(wp, lnum, &mut decor_lnum, col, decor);
                        if decor_says == Some(true) {
                            can_spell = true;
                        } else if decor_says == Some(false) {
                            can_spell = false;
                        } else if has_syntax {
                            can_spell = can_syn_spell(wp, lnum, col);
                        }

                        if !can_spell {
                            attr = HLF_COUNT;
                        } else {
                            found_one = true;
                            found_pos = pos_T {
                                lnum,
                                col,
                                coladd: 0,
                            };
                            if dir == FORWARD {
                                // Nothing further to look for.
                                (*wp).w_cursor = found_pos;
                                if !attrp.is_null() {
                                    *attrp = attr;
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
                p = p.add(len);
                capcol -= len as c_int;
            }

            if done {
                break; // found it, going forward
            }

            if dir == BACKWARD && found_pos.lnum != 0 {
                // Take the last match in the line, before the cursor.
                (*wp).w_cursor = found_pos;
                ret = found_len;
                break;
            }

            if curline {
                break; // only the cursor line was wanted
            }

            // Back at the starting line having searched it twice: give up.
            if lnum == (*wp).w_cursor.lnum && wrapped {
                break;
            }

            if dir == BACKWARD {
                if lnum > 1 {
                    lnum -= 1;
                } else if p_ws.get() == 0 {
                    break; // at the first line and 'nowrapscan'
                } else {
                    // Wrap to the end. The starting line may be searched
                    // again, to accept its last match.
                    lnum = (*(*wp).w_buffer).b_ml.ml_line_count;
                    wrapped = true;
                    if !shortmess(ShmFlag::SEARCH) {
                        give_warning(gettext(top_bot_msg.as_ptr()), true, false);
                    }
                }
                capcol = -1;
            } else {
                if lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
                    lnum += 1;
                } else if p_ws.get() == 0 {
                    break; // at the last line and 'nowrapscan'
                } else {
                    // Wrap to the start. The starting line may be searched
                    // again, to accept its first match.
                    lnum = 1;
                    wrapped = true;
                    if !shortmess(ShmFlag::SEARCH) {
                        give_warning(gettext(bot_top_msg.as_ptr()), true, false);
                    }
                }

                // Back at the starting line with nothing found: give up.
                if lnum == (*wp).w_cursor.lnum && !found_one {
                    break;
                }

                // Skip whatever of the next line a match already covered.
                skip = if attr == HLF_COUNT {
                    p.offset_from(endp) as c_int
                } else {
                    0
                };

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
        xfree(buf as *mut core::ffi::c_void);
        ret
    }
}

/// Append the start of `line` to `buf`, blanking out the leading white
/// space and any comment leaders, so that a word split over a line break
/// can still be looked up.
///
/// The blanks are kept rather than dropped so that the caller's columns
/// still refer to the same places.
pub unsafe fn spell_cat_line(buf: *mut c_char, line: *mut c_char, maxlen: c_int) {
    unsafe {
        let mut p = skipwhite(line);
        while !vim_strchr(c"*#/\"\t".as_ptr(), *p as uint8_t as c_int).is_null() {
            p = skipwhite(p.offset(1));
        }

        if *p == NUL as c_char {
            return;
        }

        // Only worth appending when there is more than white space to
        // append.
        let n = p.offset_from(line) as c_int + 1;
        if n < maxlen - 1 {
            memset(buf as *mut core::ffi::c_void, ' ' as c_int, n as usize);
            xstrlcpy(buf.offset(n as isize), p, (maxlen - n) as usize);
        }
    }
}
