//! The objects delimited by a matched pair: `i(`/`a{`/... and `it`/`at`.
//!
//! Both halves answer "where does the region enclosing the cursor start and
//! end", and differ only in how the pair is found: [`current_block`] hands
//! the bracket to `findmatch`, [`current_tagblock`] hands a generated
//! start/end pattern to `do_searchpair`. The retry loops in both are
//! Visual-mode extension -- an object already selected whole grows outwards.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::normal::{VisualMode, set_visual_anchor, set_visual_mode, visual_active, visual_anchor};
use crate::winlayer::Win;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cursor::{
    dec_cursor, gchar_cursor, get_cursor_line_ptr, get_cursor_pos_ptr, inc_cursor,
};
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later, showmode};
use crate::eval::funcs::do_searchpair;
use crate::indent::inindent;
use crate::main::{p_cpo, p_sel, p_ws};
use crate::mark::setpcmark;
use crate::mbyte::{utf_head_off, utfc_ptr2len};
use crate::memline::{decl, inc, incl, ml_get_pos};
use crate::memory::{xfree, xmalloc};
use crate::option::cpo_has;
use crate::os::cshim::snprintf;
use crate::pos::{equalpos, lt, ltoreq};
use crate::search::{BACKWARD, FORWARD, findmatch, findmatchlimit};
use crate::types::{CpoFlag, FAIL, NUL, OK, colnr_T, oparg_T, pos_T, size_t};

/// The `do_searchpair` pattern that matches any HTML start tag, used to find
/// the one enclosing the cursor before its name is known.
const ANY_START_TAG: &::core::ffi::CStr =
    c"<[^ \t>/!]\\+\\%(\\_s\\_[^>]\\{-}[^/]>\\|$\\|\\_s\\=>\\)";
/// The matching end tag, likewise before the name is known.
const ANY_END_TAG: &::core::ffi::CStr = c"</[^>]*>";
/// `snprintf` format for the start tag of a *named* element, once the name
/// has been isolated: `<%.*s` takes the name, the rest matches the
/// attributes up to the `>`.
const NAMED_START_TAG: &::core::ffi::CStr =
    c"<%.*s\\>\\%%(\\_s\\_[^>]\\{-}\\_[^/]>\\|\\_s\\?>\\)\\c";
/// The matching end tag for a named element.
const NAMED_END_TAG: &::core::ffi::CStr = c"</%.*s>\\c";

/// `i(` / `a{` / ... : the region between the `count`th enclosing `what` and
/// its matching `other`, cursor left at the end.
///
/// `include` takes the brackets themselves; without it a closing bracket
/// preceded only by indent gives the object the whole line break, which is
/// what `sol` records.
///
/// # Safety
/// `oap` must be a live operator argument, and there must be a current line.
pub unsafe fn current_block(
    oap: *mut oparg_T,
    mut count: c_int,
    include: bool,
    what: c_int,
    other: c_int,
) -> c_int {
    let mut pos: Option<pos_T>;
    let mut start_pos = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    // `{` at the start of a line, so the object claims the line break.
    let mut sol = false;
    let old_pos = cur_win().w_cursor;
    let mut old_end = cur_win().w_cursor; // where we started
    let mut old_start = old_end;

    // Starting on a bracket takes the whole block, brackets included.
    if !visual_active() || equalpos(visual_anchor(), cur_win().w_cursor) {
        // SAFETY: on the main thread with a current window and buffer, which
        // is all `setpcmark` reads.
        setpcmark();
        if what == '{' as c_int {
            // Ignore the indent.
            // SAFETY: there is a current line; `inindent` only measures its
            // leading white space and `inc_cursor` only advances within it,
            // reporting the end of the line itself.
            while unsafe { inindent(1) } {
                if inc_cursor() != 0 {
                    break;
                }
            }
        }
        // SAFETY: there is a current line with the cursor on it, which is
        // the character `gchar_cursor` reads.
        if gchar_cursor() == what {
            // On the opening bracket: move just past it.
            cur_win().w_cursor.col += 1;
        }
    } else if lt(visual_anchor(), cur_win().w_cursor) {
        old_start = visual_anchor();
        cur_win().w_cursor = visual_anchor(); // cursor at the low end
    } else {
        old_end = visual_anchor();
    }

    // Search backwards for the unclosed bracket. Quotes are ignored here,
    // but 'cpoptions' `M` is kept because that is the user's choice.
    let save_cpo = p_cpo.get();
    p_cpo.set(if !cpo_has(CpoFlag::MATCHBSL) {
        c"%".as_ptr() as *mut c_char
    } else {
        c"%M".as_ptr() as *mut c_char
    });
    // `findmatch` answering null means the cursor is not inside a pair
    // at all, and `findmatchlimit` with no limit is the fallback that
    // finds one anyway. Upstream's probe *is* the first `pos`, which is
    // why it is kept rather than discarded.
    //
    // SAFETY: there is a current line with the cursor on it; the null
    // operator argument is what the search takes to mean "no operator".
    pos = unsafe { findmatch(ptr::null_mut(), what) };
    let unbounded = pos.is_none();
    loop {
        let this = count;
        count -= 1;
        if this <= 0 {
            break;
        }
        // SAFETY: as above -- the cursor is still on a line of the current
        // buffer, moved only to positions the search itself handed back.
        pos = if unbounded {
            unsafe { findmatchlimit(ptr::null_mut(), what, FM_FORWARD as c_int, 0) }
        } else {
            unsafe { findmatch(ptr::null_mut(), what) }
        };
        let Some(found) = pos else {
            break;
        };
        cur_win().w_cursor = found;
        start_pos = found;
    }
    p_cpo.set(save_cpo);

    // Then the matching closing bracket.
    if pos.is_none() {
        cur_win().w_cursor = old_pos;
        return FAIL;
    }
    // SAFETY: the cursor sits on the opening bracket the search above found.
    let Some(mut end_pos) = (unsafe { findmatch(ptr::null_mut(), other) }) else {
        cur_win().w_cursor = old_pos;
        return FAIL;
    };
    cur_win().w_cursor = end_pos;

    // Without `include`, leave the brackets out. A closing bracket
    // preceded only by indent takes that indent with it -- but only if
    // what is left is not smaller than what we started with, which is
    // what the retry is for. (Upstream spells this `while (!include)`,
    // whose condition never changes: the loop exists only for the retry.)
    if !include {
        loop {
            // SAFETY: `start_pos` and the cursor are positions in the
            // current buffer, which is what `incl`/`decl` step through;
            // both report running off the first or last line themselves.
            // `Pos` derefs to the cursor alone, not the whole window.
            unsafe { incl(&mut start_pos) };
            sol = cur_win().w_cursor.col == 0;
            unsafe { decl(&mut cur_win().cursor()) };
            // SAFETY: there is a current line with the cursor on it.
            while unsafe { inindent(1) } {
                sol = true;
                if unsafe { decl(&mut cur_win().cursor()) } != 0 {
                    break;
                }
            }

            // In Visual mode, an empty result means there is no inner block.
            if equalpos(start_pos, end_pos) && visual_active() {
                cur_win().w_cursor = old_pos;
                return FAIL;
            }
            // In Visual mode, a result no bigger than what we started with
            // extends to the next block out and excludes again. An empty area
            // is not expanded.
            if lt(start_pos, old_start)
                || lt(old_end, cur_win().w_cursor)
                || equalpos(start_pos, cur_win().w_cursor)
                || !visual_active()
            {
                break;
            }
            cur_win().w_cursor = old_start;
            // SAFETY: as above -- the cursor is a position in the current
            // buffer, and the searches take a null operator argument.
            unsafe { decl(&mut cur_win().cursor()) };
            pos = unsafe { findmatch(ptr::null_mut(), what) };
            let Some(found) = pos else {
                cur_win().w_cursor = old_pos;
                return FAIL;
            };
            start_pos = found;
            cur_win().w_cursor = found;
            let Some(found_end) = (unsafe { findmatch(ptr::null_mut(), other) }) else {
                cur_win().w_cursor = old_pos;
                return FAIL;
            };
            end_pos = found_end;
            cur_win().w_cursor = end_pos;
        }
    }

    if visual_active() {
        // SAFETY: `p_sel` holds the NUL-terminated 'selection' value, set
        // before any mapping can run.
        if unsafe { *p_sel.get() } as c_int == 'e' as c_int {
            // SAFETY: the cursor is a position in the current buffer.
            unsafe { inc(&mut cur_win().cursor()) };
        }
        // SAFETY: there is a current line with the cursor on it.
        if sol && gchar_cursor() != NUL {
            unsafe { inc(&mut cur_win().cursor()) }; // include the line break
        }
        set_visual_anchor(start_pos);
        set_visual_mode(VisualMode::CHAR);
        // SAFETY: on the main thread with a current window; both only mark
        // the screen dirty and reprint the mode message.
        redraw_curbuf_later(UPD_INVERTED); // update the inversion
        unsafe { showmode() };
    } else {
        // SAFETY: the caller passes a live operator argument, and nothing
        // reached from here reads it back through the raw pointer.
        let oap = unsafe { &mut *oap };
        oap.start = start_pos;
        oap.motion_type = kMTCharWise;
        oap.inclusive = false;
        if sol {
            // SAFETY: the cursor is a position in the current buffer.
            unsafe { incl(&mut cur_win().cursor()) };
        } else if ltoreq(start_pos, cur_win().w_cursor) {
            // Include the character under the cursor.
            oap.inclusive = true;
        } else {
            // The end is before the start -- nothing between `<>`, `[]`
            // and so on -- so operate on no text at all.
            cur_win().w_cursor = start_pos;
        }
    }
    OK
}

/// Whether the cursor is inside a `<aaa>` tag, or with `end_tag` a `</aaa>`
/// one. A self-closing `<aaa/>` is neither.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
unsafe fn in_html_tag(end_tag: bool) -> bool {
    // SAFETY: on the main thread with a current buffer, so this hands back
    // the NUL-terminated cursor line; the cursor's column indexes into it.
    let line = get_cursor_line_ptr();
    let mut lc = NUL;

    // Back to the `<` under or before the cursor, giving up at a `>`.
    let mut p = unsafe { line.offset(cur_win().w_cursor.col as isize) };
    while p > line {
        // SAFETY: `p` is inside `line`, at or before its NUL.
        if unsafe { *p } as c_int == '<' as c_int {
            break;
        }
        // SAFETY: `p > line`, so `p - 1` is still inside the line, and
        // `utf_head_off` walks back no further than `line` itself.
        p = unsafe { p.offset(-((utf_head_off(line, p.offset(-1)) + 1) as isize)) };
        // SAFETY: the step above left `p` inside `line`.
        if unsafe { *p } as c_int == '>' as c_int {
            break;
        }
    }
    // SAFETY: `p` is inside `line`, at or before its NUL.
    if unsafe { *p } as c_int != '<' as c_int {
        return false;
    }

    let mut pos = pos_T {
        lnum: cur_win().w_cursor.lnum,
        // SAFETY: `p` and `line` point into the same line.
        col: unsafe { p.offset_from(line) } as colnr_T,
        coladd: 0,
    };
    // SAFETY: `p` is on the `<`, so the character it starts is inside the
    // line and stepping over it lands at or before the NUL.
    p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
    if end_tag {
        // There must be a `/` after the `<`.
        // SAFETY: `p` is inside `line`, at or before its NUL.
        return unsafe { *p } as c_int == '/' as c_int;
    }
    // SAFETY: as above.
    if unsafe { *p } as c_int == '/' as c_int {
        return false;
    }
    // The matching `>` must not be preceded by a `/`.
    loop {
        // SAFETY: `pos` is a position in the current buffer, which is what
        // `inc` steps through; it reports the end of the buffer itself.
        if unsafe { inc(&mut pos) } < 0 {
            return false;
        }
        // SAFETY: `inc` left `pos` on a character of the current buffer, so
        // `ml_get_pos` hands back a pointer to it.
        let c = unsafe { *ml_get_pos(&raw mut pos) } as u8 as c_int;
        if c == '>' as c_int {
            break;
        }
        lc = c;
    }
    lc != '/' as c_int
}

/// `do_searchpair` with the arguments this file always passes: no middle
/// pattern, no skip expression, no match position, no stop line and no
/// time limit.
///
/// # Safety
/// `spat` and `epat` must be NUL-terminated patterns, and there must be a
/// current window and buffer with the cursor on a line of it.
unsafe fn search_tag_pair(spat: *const c_char, epat: *const c_char, dir: c_int) -> c_int {
    // SAFETY: the patterns come from the caller; the rest are the nulls and
    // zeroes `do_searchpair` reads as "no skip, no limit".
    unsafe {
        do_searchpair(
            spat,
            c"".as_ptr(),
            epat,
            dir,
            ptr::null(),
            0,
            ptr::null_mut(),
            0,
            0,
        )
    }
}

/// `it` / `at`: the region between an enclosing HTML start and end tag,
/// cursor left at the end.
///
/// # Safety
/// `oap` must be a live operator argument, and there must be a current line.
pub unsafe fn current_tagblock(oap: *mut oparg_T, count_arg: c_int, include: bool) -> c_int {
    let mut count = count_arg;
    let mut do_include = include;
    let save_p_ws = p_ws.get() != 0;
    let mut retval = FAIL;
    let mut is_inclusive = true;
    p_ws.set(0);

    let old_pos = cur_win().w_cursor;
    let mut old_end = cur_win().w_cursor; // where we started
    let mut old_start = old_end;
    // SAFETY: `p_sel` holds the NUL-terminated 'selection' value, set before
    // any mapping can run.
    if !visual_active() || unsafe { *p_sel.get() } as c_int == 'e' as c_int {
        // SAFETY: `old_end` is a position in the current buffer, which is
        // what `decl` steps back through.
        unsafe { decl(&mut old_end) }; // `old_end` is inclusive
    }

    // Starting on a `<aaa>` selects that block.
    if !visual_active() || equalpos(visual_anchor(), cur_win().w_cursor) {
        // SAFETY: on the main thread with a current window and buffer.
        setpcmark();
        // Ignore the indent.
        // SAFETY: there is a current line; `inindent` only measures its
        // leading white space and `inc_cursor` only advances within it.
        while unsafe { inindent(1) } {
            if inc_cursor() != 0 {
                break;
            }
        }
        // SAFETY: there is a current line with the cursor on it, which is
        // all `in_html_tag` needs; `get_cursor_pos_ptr` hands back a pointer
        // into that NUL-terminated line and `inc_cursor`/`dec_cursor` report
        // running off its ends themselves.
        if unsafe { in_html_tag(false) } {
            // On a start tag: move to its `>`.
            while unsafe { *get_cursor_pos_ptr() } as c_int != '>' as c_int {
                if inc_cursor() < 0 {
                    break;
                }
            }
        } else if unsafe { in_html_tag(true) } {
            // On an end tag: move to just before it.
            while unsafe { *get_cursor_pos_ptr() } as c_int != '<' as c_int {
                if dec_cursor() < 0 {
                    break;
                }
            }
            dec_cursor();
            old_end = cur_win().w_cursor;
        }
    } else if lt(visual_anchor(), cur_win().w_cursor) {
        old_start = visual_anchor();
        cur_win().w_cursor = visual_anchor(); // cursor at the low end
    } else {
        old_end = visual_anchor();
    }

    let mut start_pos;
    let mut end_pos;
    // Upstream's `again:` label. Two things jump back here: an end tag
    // found before where we started (so the start tag has no match and
    // the one further out is tried), and a Visual selection that came out
    // exactly as it already was (so the tags themselves are taken in).
    'again: loop {
        // Search backwards for the unclosed `<aaa>`.
        for _ in 0..count {
            // SAFETY: both patterns are NUL-terminated constants, and the
            // cursor is on a line of the current buffer.
            if unsafe { search_tag_pair(ANY_START_TAG.as_ptr(), ANY_END_TAG.as_ptr(), BACKWARD) }
                <= 0
            {
                cur_win().w_cursor = old_pos;
                p_ws.set(save_p_ws as c_int);
                return retval;
            }
        }
        start_pos = cur_win().w_cursor;

        // Isolate the `aaa` so the matching `</aaa>` can be searched for.
        // SAFETY: there is a current line with the cursor on it, so
        // `get_cursor_pos_ptr` hands back a pointer into it.
        inc_cursor();
        let p = get_cursor_pos_ptr();
        let mut cp = p;
        // SAFETY: `cp` walks the same NUL-terminated line from `p`; the NUL
        // test leads the chain, so the later reads are of a character that
        // is really there, and `utfc_ptr2len` stops at the NUL as well.
        while unsafe {
            *cp as c_int != NUL && *cp as c_int != '>' as c_int && !ascii_iswhite(*cp as c_int)
        } {
            cp = unsafe { cp.offset(utfc_ptr2len(cp) as isize) };
        }
        let len = unsafe { cp.offset_from(p) } as c_int;
        if len == 0 {
            cur_win().w_cursor = old_pos;
            p_ws.set(save_p_ws as c_int);
            return retval;
        }
        let spat_len = len as size_t + 39;
        // SAFETY: `xmalloc` aborts rather than answering null, and the two
        // lengths leave room for the format plus the `len` bytes of name.
        let spat = unsafe { xmalloc(spat_len) } as *mut c_char;
        let epat_len = len as size_t + 9;
        let epat = unsafe { xmalloc(epat_len) } as *mut c_char;
        // SAFETY: each buffer is `*_len` bytes long, the formats are
        // NUL-terminated constants, and `%.*s` is given the matching
        // `c_int` length and a pointer to that many bytes of the line.
        unsafe { snprintf(spat, spat_len, NAMED_START_TAG.as_ptr(), len, p) };
        unsafe { snprintf(epat, epat_len, NAMED_END_TAG.as_ptr(), len, p) };
        // SAFETY: `snprintf` NUL-terminated both patterns above.
        let r = unsafe { search_tag_pair(spat, epat, FORWARD) };
        // SAFETY: both came from `xmalloc` above and are dead from here on.
        unsafe { xfree(spat as *mut c_void) };
        unsafe { xfree(epat as *mut c_void) };

        if r < 1 || lt(cur_win().w_cursor, old_end) {
            // No other end, or it is before the previous one: this could
            // be an HTML tag with no matching end. Search backwards for
            // another start tag.
            count = 1;
            cur_win().w_cursor = start_pos;
            continue 'again;
        }

        if do_include {
            // Include up to the `>`.
            // SAFETY: there is a current line with the cursor on it, so
            // `get_cursor_pos_ptr` hands back a pointer into it, and
            // `inc_cursor` reports the end of the line itself.
            while unsafe { *get_cursor_pos_ptr() } as c_int != '>' as c_int {
                if inc_cursor() < 0 {
                    break;
                }
            }
        } else {
            // SAFETY: as above.
            let c = get_cursor_pos_ptr();
            // Exclude the `<` of the end tag. With the closing tag on a
            // new line, leave the cursor where it is and make the
            // operation exclusive instead, so the line feed is selected.
            // SAFETY: `c` points into the NUL-terminated cursor line.
            if unsafe { *c } as c_int == '<' as c_int
                && !visual_active()
                && cur_win().w_cursor.col == 0
            {
                is_inclusive = false;
            } else if unsafe { *c } as c_int == '<' as c_int {
                // SAFETY: the cursor is on a line of the current buffer.
                dec_cursor();
            }
        }
        end_pos = cur_win().w_cursor;

        if !do_include {
            // Exclude the start tag, stepping over any `>` inside quotes.
            let mut in_quotes = false;
            cur_win().w_cursor = start_pos;
            // SAFETY: `inc_cursor` reports running off the buffer itself, so
            // the cursor is on a character of a line of the current buffer
            // whenever the body runs, and `get_cursor_pos_ptr` points at it.
            while inc_cursor() >= 0 {
                let q = unsafe { *get_cursor_pos_ptr() } as c_int;
                if q == '>' as c_int && !in_quotes {
                    inc_cursor();
                    start_pos = cur_win().w_cursor;
                    break;
                } else if q == '"' as c_int || q == '\'' as c_int {
                    in_quotes = !in_quotes;
                }
            }
            cur_win().w_cursor = end_pos;

            // In Visual mode with exactly the text we already had, take
            // the tags in and try again.
            if visual_active() && equalpos(start_pos, old_start) && equalpos(end_pos, old_end) {
                do_include = true;
                cur_win().w_cursor = old_start;
                count = count_arg;
                continue 'again;
            }
        }
        break;
    }

    if visual_active() {
        // An end before the start means there is no text between the
        // tags: select the character under the cursor.
        if lt(end_pos, start_pos) {
            cur_win().w_cursor = start_pos;
        // SAFETY: `p_sel` holds the NUL-terminated 'selection' value.
        } else if unsafe { *p_sel.get() } as c_int == 'e' as c_int {
            // SAFETY: the cursor is on a line of the current buffer.
            inc_cursor();
        }
        set_visual_anchor(start_pos);
        set_visual_mode(VisualMode::CHAR);
        // SAFETY: on the main thread with a current window; both only mark
        // the screen dirty and reprint the mode message.
        redraw_curbuf_later(UPD_INVERTED); // update the inversion
        unsafe { showmode() };
    } else {
        // SAFETY: the caller passes a live operator argument, and nothing
        // reached from here reads it back through the raw pointer.
        let oap = unsafe { &mut *oap };
        oap.start = start_pos;
        oap.motion_type = kMTCharWise;
        if lt(end_pos, start_pos) {
            // No text between the tags: operate on an empty area.
            cur_win().w_cursor = start_pos;
            oap.inclusive = false;
        } else {
            oap.inclusive = is_inclusive;
        }
    }
    retval = OK;

    p_ws.set(save_p_ws as c_int);
    retval
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
