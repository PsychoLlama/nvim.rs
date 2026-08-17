//! The objects delimited by a matched pair: `i(`/`a{`/... and `it`/`at`.
//!
//! Both halves answer "where does the region enclosing the cursor start and
//! end", and differ only in how the pair is found: [`current_block`] hands
//! the bracket to `findmatch`, [`current_tagblock`] hands a generated
//! start/end pattern to `do_searchpair`. The retry loops in both are
//! Visual-mode extension -- an object already selected whole grows outwards.

#![deny(unsafe_op_in_unsafe_fn)]

use ::core::ffi::{c_char, c_int};

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cursor::{
    dec_cursor, gchar_cursor, get_cursor_line_ptr, get_cursor_pos_ptr, inc_cursor,
};
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later, showmode};
use crate::eval::funcs::do_searchpair;
use crate::indent::inindent;
use crate::main::{VIsual, VIsual_active, VIsual_mode, curwin, p_cpo, p_sel, p_ws};
use crate::mark::setpcmark;
use crate::mbyte::{utf_head_off, utfc_ptr2len};
use crate::memline::{decl, inc, incl, ml_get_pos};
use crate::memory::{xfree, xmalloc};
use crate::os::libc::snprintf;
use crate::pos::{equalpos, lt, ltoreq};
use crate::search::{BACKWARD, FORWARD, findmatch, findmatchlimit};
use crate::strings::vim_strchr;
use crate::types::{colnr_T, int64_t, linenr_T, oparg_T, pos_T, size_t, typval_T};

/// The `do_searchpair` pattern that matches any HTML start tag, used to find
/// the one enclosing the cursor before its name is known.
const ANY_START_TAG: &::core::ffi::CStr =
    c"<[^ \t>/!]\\+\\%(\\_s\\_[^>]\\{-}[^/]>\\|$\\|\\_s\\=>\\)";
/// The matching end tag, likewise before the name is known.
const ANY_END_TAG: &::core::ffi::CStr = c"</[^>]*>";

/// `i(` / `a{` / ... : the region between the `count`th enclosing `what` and
/// its matching `other`, cursor left at the end.
///
/// `include` takes the brackets themselves; without it a closing bracket
/// preceded only by indent gives the object the whole line break, which is
/// what `sol` records.
///
/// # Safety
/// `oap` must be a live operator argument, and there must be a current line.
pub unsafe extern "C" fn current_block(
    oap: *mut oparg_T,
    mut count: c_int,
    include: bool,
    what: c_int,
    other: c_int,
) -> c_int {
    unsafe {
        let mut pos = ::core::ptr::null_mut::<pos_T>();
        let mut start_pos = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        // `{` at the start of a line, so the object claims the line break.
        let mut sol = false;
        let old_pos = (*curwin.get()).w_cursor;
        let mut old_end = (*curwin.get()).w_cursor; // where we started
        let mut old_start = old_end;

        // Starting on a bracket takes the whole block, brackets included.
        if !VIsual_active.get() || equalpos(VIsual.get(), (*curwin.get()).w_cursor) {
            setpcmark();
            if what == '{' as c_int {
                // Ignore the indent.
                while inindent(1) {
                    if inc_cursor() != 0 {
                        break;
                    }
                }
            }
            if gchar_cursor() == what {
                // On the opening bracket: move just past it.
                (*curwin.get()).w_cursor.col += 1;
            }
        } else if lt(VIsual.get(), (*curwin.get()).w_cursor) {
            old_start = VIsual.get();
            (*curwin.get()).w_cursor = VIsual.get(); // cursor at the low end
        } else {
            old_end = VIsual.get();
        }

        // Search backwards for the unclosed bracket. Quotes are ignored here,
        // but 'cpoptions' `M` is kept because that is the user's choice.
        let save_cpo = p_cpo.get();
        p_cpo.set(if vim_strchr(p_cpo.get(), CPO_MATCHBSL).is_null() {
            c"%".as_ptr() as *mut c_char
        } else {
            c"%M".as_ptr() as *mut c_char
        });
        // `findmatch` answering null means the cursor is not inside a pair
        // at all, and `findmatchlimit` with no limit is the fallback that
        // finds one anyway. Upstream's probe *is* the first `pos`, which is
        // why it is kept rather than discarded.
        pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
        let unbounded = pos.is_null();
        loop {
            let this = count;
            count -= 1;
            if this <= 0 {
                break;
            }
            pos = if unbounded {
                findmatchlimit(
                    ::core::ptr::null_mut::<oparg_T>(),
                    what,
                    FM_FORWARD as c_int,
                    0 as int64_t,
                )
            } else {
                findmatch(::core::ptr::null_mut::<oparg_T>(), what)
            };
            if pos.is_null() {
                break;
            }
            (*curwin.get()).w_cursor = *pos;
            // The `findmatch` for `end_pos` overwrites what `pos` points at.
            start_pos = *pos;
        }
        p_cpo.set(save_cpo);

        // Then the matching closing bracket.
        if pos.is_null() {
            (*curwin.get()).w_cursor = old_pos;
            return FAIL;
        }
        let mut end_pos = findmatch(::core::ptr::null_mut::<oparg_T>(), other);
        if end_pos.is_null() {
            (*curwin.get()).w_cursor = old_pos;
            return FAIL;
        }
        (*curwin.get()).w_cursor = *end_pos;

        // Without `include`, leave the brackets out. A closing bracket
        // preceded only by indent takes that indent with it -- but only if
        // what is left is not smaller than what we started with, which is
        // what the retry is for. (Upstream spells this `while (!include)`,
        // whose condition never changes: the loop exists only for the retry.)
        if !include {
            loop {
                incl(&raw mut start_pos);
                sol = (*curwin.get()).w_cursor.col == 0;
                decl(&raw mut (*curwin.get()).w_cursor);
                while inindent(1) {
                    sol = true;
                    if decl(&raw mut (*curwin.get()).w_cursor) != 0 {
                        break;
                    }
                }

                // In Visual mode, an empty result means there is no inner block.
                if equalpos(start_pos, *end_pos) && VIsual_active.get() {
                    (*curwin.get()).w_cursor = old_pos;
                    return FAIL;
                }
                // In Visual mode, a result no bigger than what we started with
                // extends to the next block out and excludes again. An empty area
                // is not expanded.
                if lt(start_pos, old_start)
                    || lt(old_end, (*curwin.get()).w_cursor)
                    || equalpos(start_pos, (*curwin.get()).w_cursor)
                    || !VIsual_active.get()
                {
                    break;
                }
                (*curwin.get()).w_cursor = old_start;
                decl(&raw mut (*curwin.get()).w_cursor);
                pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
                if pos.is_null() {
                    (*curwin.get()).w_cursor = old_pos;
                    return FAIL;
                }
                start_pos = *pos;
                (*curwin.get()).w_cursor = *pos;
                end_pos = findmatch(::core::ptr::null_mut::<oparg_T>(), other);
                if end_pos.is_null() {
                    (*curwin.get()).w_cursor = old_pos;
                    return FAIL;
                }
                (*curwin.get()).w_cursor = *end_pos;
            }
        }

        if VIsual_active.get() {
            if *p_sel.get() as c_int == 'e' as c_int {
                inc(&raw mut (*curwin.get()).w_cursor);
            }
            if sol && gchar_cursor() != NUL {
                inc(&raw mut (*curwin.get()).w_cursor); // include the line break
            }
            VIsual.set(start_pos);
            VIsual_mode.set('v' as c_int);
            redraw_curbuf_later(UPD_INVERTED); // update the inversion
            showmode();
        } else {
            (*oap).start = start_pos;
            (*oap).motion_type = kMTCharWise;
            (*oap).inclusive = false;
            if sol {
                incl(&raw mut (*curwin.get()).w_cursor);
            } else if ltoreq(start_pos, (*curwin.get()).w_cursor) {
                // Include the character under the cursor.
                (*oap).inclusive = true;
            } else {
                // The end is before the start -- nothing between `<>`, `[]`
                // and so on -- so operate on no text at all.
                (*curwin.get()).w_cursor = start_pos;
            }
        }
        OK
    }
}

/// Whether the cursor is inside a `<aaa>` tag, or with `end_tag` a `</aaa>`
/// one. A self-closing `<aaa/>` is neither.
///
/// # Safety
/// There must be a current line and the cursor must be on it.
unsafe fn in_html_tag(end_tag: bool) -> bool {
    unsafe {
        let line = get_cursor_line_ptr();
        let mut lc = NUL;

        // Back to the `<` under or before the cursor, giving up at a `>`.
        let mut p = line.offset((*curwin.get()).w_cursor.col as isize);
        while p > line {
            if *p as c_int == '<' as c_int {
                break;
            }
            p = p.offset(-((utf_head_off(line, p.offset(-1)) + 1) as isize));
            if *p as c_int == '>' as c_int {
                break;
            }
        }
        if *p as c_int != '<' as c_int {
            return false;
        }

        let mut pos = pos_T {
            lnum: (*curwin.get()).w_cursor.lnum,
            col: p.offset_from(line) as colnr_T,
            coladd: 0,
        };
        p = p.offset(utfc_ptr2len(p) as isize);
        if end_tag {
            // There must be a `/` after the `<`.
            return *p as c_int == '/' as c_int;
        }
        if *p as c_int == '/' as c_int {
            return false;
        }
        // The matching `>` must not be preceded by a `/`.
        loop {
            if inc(&raw mut pos) < 0 {
                return false;
            }
            let c = *ml_get_pos(&raw mut pos) as u8 as c_int;
            if c == '>' as c_int {
                break;
            }
            lc = c;
        }
        lc != '/' as c_int
    }
}

/// `it` / `at`: the region between an enclosing HTML start and end tag,
/// cursor left at the end.
///
/// # Safety
/// `oap` must be a live operator argument, and there must be a current line.
pub unsafe extern "C" fn current_tagblock(
    oap: *mut oparg_T,
    count_arg: c_int,
    include: bool,
) -> c_int {
    unsafe {
        let mut count = count_arg;
        let mut do_include = include;
        let save_p_ws = p_ws.get() != 0;
        let mut retval = FAIL;
        let mut is_inclusive = true;
        p_ws.set(0);

        let old_pos = (*curwin.get()).w_cursor;
        let mut old_end = (*curwin.get()).w_cursor; // where we started
        let mut old_start = old_end;
        if !VIsual_active.get() || *p_sel.get() as c_int == 'e' as c_int {
            decl(&raw mut old_end); // `old_end` is inclusive
        }

        // Starting on a `<aaa>` selects that block.
        if !VIsual_active.get() || equalpos(VIsual.get(), (*curwin.get()).w_cursor) {
            setpcmark();
            // Ignore the indent.
            while inindent(1) {
                if inc_cursor() != 0 {
                    break;
                }
            }
            if in_html_tag(false) {
                // On a start tag: move to its `>`.
                while *get_cursor_pos_ptr() as c_int != '>' as c_int {
                    if inc_cursor() < 0 {
                        break;
                    }
                }
            } else if in_html_tag(true) {
                // On an end tag: move to just before it.
                while *get_cursor_pos_ptr() as c_int != '<' as c_int {
                    if dec_cursor() < 0 {
                        break;
                    }
                }
                dec_cursor();
                old_end = (*curwin.get()).w_cursor;
            }
        } else if lt(VIsual.get(), (*curwin.get()).w_cursor) {
            old_start = VIsual.get();
            (*curwin.get()).w_cursor = VIsual.get(); // cursor at the low end
        } else {
            old_end = VIsual.get();
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
                if do_searchpair(
                    ANY_START_TAG.as_ptr(),
                    c"".as_ptr(),
                    ANY_END_TAG.as_ptr(),
                    BACKWARD as c_int,
                    ::core::ptr::null::<typval_T>(),
                    0,
                    ::core::ptr::null_mut::<pos_T>(),
                    0 as linenr_T,
                    0 as int64_t,
                ) <= 0
                {
                    (*curwin.get()).w_cursor = old_pos;
                    p_ws.set(save_p_ws as c_int);
                    return retval;
                }
            }
            start_pos = (*curwin.get()).w_cursor;

            // Isolate the `aaa` so the matching `</aaa>` can be searched for.
            inc_cursor();
            let mut p = get_cursor_pos_ptr();
            let mut cp = p;
            while *cp as c_int != NUL
                && *cp as c_int != '>' as c_int
                && !ascii_iswhite(*cp as c_int)
            {
                cp = cp.offset(utfc_ptr2len(cp) as isize);
            }
            let len = cp.offset_from(p) as c_int;
            if len == 0 {
                (*curwin.get()).w_cursor = old_pos;
                p_ws.set(save_p_ws as c_int);
                return retval;
            }
            let spat_len = len as size_t + 39;
            let spat = xmalloc(spat_len) as *mut c_char;
            let epat_len = len as size_t + 9;
            let epat = xmalloc(epat_len) as *mut c_char;
            snprintf(
                spat,
                spat_len,
                c"<%.*s\\>\\%%(\\_s\\_[^>]\\{-}\\_[^/]>\\|\\_s\\?>\\)\\c".as_ptr(),
                len,
                p,
            );
            snprintf(epat, epat_len, c"</%.*s>\\c".as_ptr(), len, p);
            let r = do_searchpair(
                spat,
                c"".as_ptr(),
                epat,
                FORWARD as c_int,
                ::core::ptr::null::<typval_T>(),
                0,
                ::core::ptr::null_mut::<pos_T>(),
                0 as linenr_T,
                0 as int64_t,
            );
            xfree(spat as *mut ::core::ffi::c_void);
            xfree(epat as *mut ::core::ffi::c_void);

            if r < 1 || lt((*curwin.get()).w_cursor, old_end) {
                // No other end, or it is before the previous one: this could
                // be an HTML tag with no matching end. Search backwards for
                // another start tag.
                count = 1;
                (*curwin.get()).w_cursor = start_pos;
                continue 'again;
            }

            if do_include {
                // Include up to the `>`.
                while *get_cursor_pos_ptr() as c_int != '>' as c_int {
                    if inc_cursor() < 0 {
                        break;
                    }
                }
            } else {
                let c = get_cursor_pos_ptr();
                // Exclude the `<` of the end tag. With the closing tag on a
                // new line, leave the cursor where it is and make the
                // operation exclusive instead, so the line feed is selected.
                if *c as c_int == '<' as c_int
                    && !VIsual_active.get()
                    && (*curwin.get()).w_cursor.col == 0
                {
                    is_inclusive = false;
                } else if *c as c_int == '<' as c_int {
                    dec_cursor();
                }
            }
            end_pos = (*curwin.get()).w_cursor;

            if !do_include {
                // Exclude the start tag, stepping over any `>` inside quotes.
                let mut in_quotes = false;
                (*curwin.get()).w_cursor = start_pos;
                while inc_cursor() >= 0 {
                    p = get_cursor_pos_ptr();
                    if *p as c_int == '>' as c_int && !in_quotes {
                        inc_cursor();
                        start_pos = (*curwin.get()).w_cursor;
                        break;
                    } else if *p as c_int == '"' as c_int || *p as c_int == '\'' as c_int {
                        in_quotes = !in_quotes;
                    }
                }
                (*curwin.get()).w_cursor = end_pos;

                // In Visual mode with exactly the text we already had, take
                // the tags in and try again.
                if VIsual_active.get()
                    && equalpos(start_pos, old_start)
                    && equalpos(end_pos, old_end)
                {
                    do_include = true;
                    (*curwin.get()).w_cursor = old_start;
                    count = count_arg;
                    continue 'again;
                }
            }
            break;
        }

        if VIsual_active.get() {
            // An end before the start means there is no text between the
            // tags: select the character under the cursor.
            if lt(end_pos, start_pos) {
                (*curwin.get()).w_cursor = start_pos;
            } else if *p_sel.get() as c_int == 'e' as c_int {
                inc_cursor();
            }
            VIsual.set(start_pos);
            VIsual_mode.set('v' as c_int);
            redraw_curbuf_later(UPD_INVERTED); // update the inversion
            showmode();
        } else {
            (*oap).start = start_pos;
            (*oap).motion_type = kMTCharWise;
            if lt(end_pos, start_pos) {
                // No text between the tags: operate on an empty area.
                (*curwin.get()).w_cursor = start_pos;
                (*oap).inclusive = false;
            } else {
                (*oap).inclusive = is_inclusive;
            }
        }
        retval = OK;

        p_ws.set(save_p_ws as c_int);
        retval
    }
}
