//! The objects delimited by a matched pair: `i(`/`a{`/... and `it`/`at`.
//!
//! Both halves answer "where does the region enclosing the cursor start and
//! end", and differ only in how the pair is found: [`current_block`] hands
//! the bracket to `findmatch`, [`current_tagblock`] hands a generated
//! start/end pattern to `do_searchpair`. The retry loops in both are
//! Visual-mode extension -- an object already selected whole grows outwards.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::normal::{VisualMode, set_visual_anchor, set_visual_mode, visual_active, visual_anchor};
use crate::winlayer::Win;
use core::ffi::c_int;

use super::*;
use crate::ascii::ascii_iswhite;
use crate::cstr::byte_at;
use crate::cursor::{dec_cursor, gchar_cursor, inc_cursor};
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later, showmode};
use crate::eval::funcs::do_searchpair;
use crate::indent::inindent;
use crate::mark::setpcmark;
use crate::mbyte::{c_cluster_len, head_off};
use crate::memline::{Lines, decl, inc, incl};
use crate::option::vars::{P_SEL, P_WS, p_ws};
use crate::option::{SavedCpo, cpo_has};
use crate::pos::{equalpos, lt, ltoreq};
use crate::search::{BACKWARD, FORWARD, findmatch, findmatchlimit};
use crate::types::{ColNr, CpoFlag, FAIL, Failed, NUL, OK, OpArg, Pos};

/// The `do_searchpair` pattern that matches any HTML start tag, used to find
/// the one enclosing the cursor before its name is known.
const ANY_START_TAG: &::core::ffi::CStr =
    c"<[^ \t>/!]\\+\\%(\\_s\\_[^>]\\{-}[^/]>\\|$\\|\\_s\\=>\\)";
/// The matching end tag, likewise before the name is known.
const ANY_END_TAG: &::core::ffi::CStr = c"</[^>]*>";
/// What follows the name in the pattern for the start tag of a *named*
/// element, once the name has been isolated (`<` comes before it): the
/// attributes up to the `>`.
const NAMED_START_TAG_REST: &[u8] = b"\\>\\%(\\_s\\_[^>]\\{-}\\_[^/]>\\|\\_s\\?>\\)\\c";
/// What follows the name in the matching end tag's pattern (`</` comes
/// before it).
const NAMED_END_TAG_REST: &[u8] = b">\\c";

/// `i(` / `a{` / ... : the region between the `count`th enclosing `what` and
/// its matching `other`, cursor left at the end.
///
/// `include` takes the brackets themselves; without it a closing bracket
/// preceded only by indent gives the object the whole line break, which is
/// what `sol` records.
pub fn current_block(
    op: &mut OpArg,
    mut count: c_int,
    include: bool,
    what: c_int,
    other: c_int,
) -> Result<(), Failed> {
    let mut pos: Option<Pos>;
    let mut start_pos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    // `{` at the start of a line, so the object claims the line break.
    let mut sol = false;
    let old_pos = Win::current().w_cursor;
    let mut old_end = Win::current().w_cursor; // where we started
    let mut old_start = old_end;

    // Starting on a bracket takes the whole block, brackets included.
    if !visual_active() || equalpos(visual_anchor(), Win::current().w_cursor) {
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
            Win::current().w_cursor.col += 1;
        }
    } else if lt(visual_anchor(), Win::current().w_cursor) {
        old_start = visual_anchor();
        Win::current().w_cursor = visual_anchor(); // cursor at the low end
    } else {
        old_end = visual_anchor();
    }

    // Search backwards for the unclosed bracket. Quotes are ignored here,
    // but 'cpoptions' `M` is kept because that is the user's choice.
    let cpo = SavedCpo::held(if cpo_has(CpoFlag::MATCHBSL) {
        c"%M"
    } else {
        c"%"
    });
    // `findmatch` answering null means the cursor is not inside a pair
    // at all, and `findmatchlimit` with no limit is the fallback that
    // finds one anyway. Upstream's probe *is* the first `pos`, which is
    // why it is kept rather than discarded.
    //
    pos = findmatch(None, what);
    let unbounded = pos.is_none();
    loop {
        let this = count;
        count -= 1;
        if this <= 0 {
            break;
        }
        pos = if unbounded {
            findmatchlimit(None, what, FM_FORWARD.cast_signed(), 0)
        } else {
            findmatch(None, what)
        };
        let Some(found) = pos else {
            break;
        };
        Win::current().w_cursor = found;
        start_pos = found;
    }
    drop(cpo);

    // Then the matching closing bracket.
    if pos.is_none() {
        Win::current().w_cursor = old_pos;
        return Err(Failed);
    }
    let Some(mut end_pos) = findmatch(None, other) else {
        Win::current().w_cursor = old_pos;
        return Err(Failed);
    };
    Win::current().w_cursor = end_pos;

    // Without `include`, leave the brackets out. A closing bracket
    // preceded only by indent takes that indent with it -- but only if
    // what is left is not smaller than what we started with, which is
    // what the retry is for. (Upstream spells this `while (!include)`,
    // whose condition never changes: the loop exists only for the retry.)
    if !include {
        loop {
            incl(&mut start_pos);
            sol = Win::current().w_cursor.col == 0;
            decl(&mut Win::current().cursor());
            while inindent(1) {
                sol = true;
                if decl(&mut Win::current().cursor()) != 0 {
                    break;
                }
            }

            // In Visual mode, an empty result means there is no inner block.
            if equalpos(start_pos, end_pos) && visual_active() {
                Win::current().w_cursor = old_pos;
                return Err(Failed);
            }
            // In Visual mode, a result no bigger than what we started with
            // extends to the next block out and excludes again. An empty area
            // is not expanded.
            if lt(start_pos, old_start)
                || lt(old_end, Win::current().w_cursor)
                || equalpos(start_pos, Win::current().w_cursor)
                || !visual_active()
            {
                break;
            }
            Win::current().w_cursor = old_start;
            decl(&mut Win::current().cursor());
            pos = findmatch(None, what);
            let Some(found) = pos else {
                Win::current().w_cursor = old_pos;
                return Err(Failed);
            };
            start_pos = found;
            Win::current().w_cursor = found;
            let Some(found_end) = findmatch(None, other) else {
                Win::current().w_cursor = old_pos;
                return Err(Failed);
            };
            end_pos = found_end;
            Win::current().w_cursor = end_pos;
        }
    }

    if visual_active() {
        if P_SEL.first_byte() == b'e' {
            inc(&mut Win::current().cursor());
        }
        if sol && gchar_cursor() != NUL {
            inc(&mut Win::current().cursor()); // include the line break
        }
        set_visual_anchor(start_pos);
        set_visual_mode(VisualMode::CHAR);
        redraw_curbuf_later(UPD_INVERTED); // update the inversion
        showmode();
    } else {
        op.start = start_pos;
        op.motion_type = kMTCharWise;
        op.inclusive = false;
        if sol {
            incl(&mut Win::current().cursor());
        } else if ltoreq(start_pos, Win::current().w_cursor) {
            // Include the character under the cursor.
            op.inclusive = true;
        } else {
            // The end is before the start -- nothing between `<>`, `[]`
            // and so on -- so operate on no text at all.
            Win::current().w_cursor = start_pos;
        }
    }
    Ok(())
}

/// Whether the cursor is inside a `<aaa>` tag, or with `end_tag` a `</aaa>`
/// one. A self-closing `<aaa/>` is neither.
fn in_html_tag(end_tag: bool) -> bool {
    let cursor = Win::current().w_cursor;
    let mut lc = NUL;
    let at = {
        let mut lines = Lines::current();
        let line = lines.line(cursor.lnum);

        // Back to the `<` under or before the cursor, giving up at a `>`.
        let mut p = usize::try_from(cursor.col).unwrap_or(0).min(line.len());
        while p > 0 {
            if byte_at(line, p) == b'<' {
                break;
            }
            p -= head_off(line, p - 1) + 1;
            if byte_at(line, p) == b'>' {
                break;
            }
        }
        if byte_at(line, p) != b'<' {
            return false;
        }

        // The `/` an end tag needs after its `<`, and a start tag must not
        // have.
        let slash = byte_at(line, p + c_cluster_len(&line[p..])) == b'/';
        if slash != end_tag {
            return false;
        }
        if end_tag {
            return true;
        }
        p
    };
    let mut pos = Pos {
        lnum: cursor.lnum,
        col: ColNr::try_from(at).unwrap_or(ColNr::MAX),
        coladd: 0,
    };
    // The matching `>` must not be preceded by a `/`.
    loop {
        if inc(&mut pos) < 0 {
            return false;
        }
        // `inc` left `pos` on a character of the current buffer.
        let c = c_int::from(byte_at(
            Lines::current().line(pos.lnum),
            usize::try_from(pos.col).unwrap_or(usize::MAX),
        ));
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
fn search_tag_pair(spat: &[u8], epat: &[u8], dir: c_int) -> c_int {
    do_searchpair(spat, b"", epat, dir, None, 0, None, 0, 0)
}

/// `it` / `at`: the region between an enclosing HTML start and end tag,
/// cursor left at the end.
pub fn current_tagblock(op: &mut OpArg, count_arg: c_int, include: bool) -> c_int {
    let mut count = count_arg;
    let mut do_include = include;
    let save_p_ws = p_ws();
    let mut retval = FAIL;
    let mut is_inclusive = true;
    P_WS.set(false);

    let old_pos = Win::current().w_cursor;
    let mut old_end = Win::current().w_cursor; // where we started
    let mut old_start = old_end;
    if !visual_active() || P_SEL.first_byte() == b'e' {
        decl(&mut old_end); // `old_end` is inclusive
    }

    // Starting on a `<aaa>` selects that block.
    if !visual_active() || equalpos(visual_anchor(), Win::current().w_cursor) {
        setpcmark();
        // Ignore the indent.
        while inindent(1) {
            if inc_cursor() != 0 {
                break;
            }
        }
        // `inc_cursor`/`dec_cursor` report running off the line's ends
        // themselves.
        if in_html_tag(false) {
            // On a start tag: move to its `>`.
            while gchar_cursor() != '>' as c_int {
                if inc_cursor() < 0 {
                    break;
                }
            }
        } else if in_html_tag(true) {
            // On an end tag: move to just before it.
            while gchar_cursor() != '<' as c_int {
                if dec_cursor() < 0 {
                    break;
                }
            }
            dec_cursor();
            old_end = Win::current().w_cursor;
        }
    } else if lt(visual_anchor(), Win::current().w_cursor) {
        old_start = visual_anchor();
        Win::current().w_cursor = visual_anchor(); // cursor at the low end
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
            if search_tag_pair(ANY_START_TAG.to_bytes(), ANY_END_TAG.to_bytes(), BACKWARD) <= 0 {
                Win::current().w_cursor = old_pos;
                P_WS.set(save_p_ws);
                return retval;
            }
        }
        start_pos = Win::current().w_cursor;

        // Isolate the `aaa` so the matching `</aaa>` can be searched for.
        inc_cursor();
        let name = {
            let cursor = Win::current().w_cursor;
            let mut lines = Lines::current();
            let line = lines.line(cursor.lnum);
            let start = usize::try_from(cursor.col).unwrap_or(0).min(line.len());
            let mut end = start;
            while !matches!(byte_at(line, end), 0 | b'>') && !ascii_iswhite(c_int::from(line[end]))
            {
                end += c_cluster_len(&line[end..]);
            }
            line[start..end].to_vec()
        };
        if name.is_empty() {
            Win::current().w_cursor = old_pos;
            P_WS.set(save_p_ws);
            return retval;
        }
        let spat = [&b"<"[..], &name, NAMED_START_TAG_REST].concat();
        let epat = [&b"</"[..], &name, NAMED_END_TAG_REST].concat();
        let r = search_tag_pair(&spat, &epat, FORWARD);

        if r < 1 || lt(Win::current().w_cursor, old_end) {
            // No other end, or it is before the previous one: this could
            // be an HTML tag with no matching end. Search backwards for
            // another start tag.
            count = 1;
            Win::current().w_cursor = start_pos;
            continue 'again;
        }

        if do_include {
            // Include up to the `>`; `inc_cursor` reports the end of the
            // line itself.
            while gchar_cursor() != '>' as c_int {
                if inc_cursor() < 0 {
                    break;
                }
            }
        } else {
            let c = gchar_cursor();
            // Exclude the `<` of the end tag. With the closing tag on a
            // new line, leave the cursor where it is and make the
            // operation exclusive instead, so the line feed is selected.
            if c == '<' as c_int && !visual_active() && Win::current().w_cursor.col == 0 {
                is_inclusive = false;
            } else if c == '<' as c_int {
                dec_cursor();
            }
        }
        end_pos = Win::current().w_cursor;

        if !do_include {
            // Exclude the start tag, stepping over any `>` inside quotes.
            let mut in_quotes = false;
            Win::current().w_cursor = start_pos;
            // `inc_cursor` reports running off the buffer itself, so the
            // cursor is on a character whenever the body runs.
            while inc_cursor() >= 0 {
                let q = gchar_cursor();
                if q == '>' as c_int && !in_quotes {
                    inc_cursor();
                    start_pos = Win::current().w_cursor;
                    break;
                } else if q == '"' as c_int || q == '\'' as c_int {
                    in_quotes = !in_quotes;
                }
            }
            Win::current().w_cursor = end_pos;

            // In Visual mode with exactly the text we already had, take
            // the tags in and try again.
            if visual_active() && equalpos(start_pos, old_start) && equalpos(end_pos, old_end) {
                do_include = true;
                Win::current().w_cursor = old_start;
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
            Win::current().w_cursor = start_pos;
        } else if P_SEL.first_byte() == b'e' {
            inc_cursor();
        }
        set_visual_anchor(start_pos);
        set_visual_mode(VisualMode::CHAR);
        redraw_curbuf_later(UPD_INVERTED); // update the inversion
        showmode();
    } else {
        op.start = start_pos;
        op.motion_type = kMTCharWise;
        if lt(end_pos, start_pos) {
            // No text between the tags: operate on an empty area.
            Win::current().w_cursor = start_pos;
            op.inclusive = false;
        } else {
            op.inclusive = is_inclusive;
        }
    }
    retval = OK;

    P_WS.set(save_p_ws);
    retval
}
