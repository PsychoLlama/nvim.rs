//! Searching the buffer: `search()`, `searchpos()`, `searchpair()`,
//! `searchpairpos()` and `searchdecl()`.
//!
//! The three real entry points are [`search_cmn`], [`searchpair_cmn`] and
//! [`do_searchpair`]; the `f_*` bodies are thin. Every one of them lets the
//! flag parser write 'wrapscan' and puts the caller's value back on the way
//! out, which [`SavedWrapScan`] does here instead of the C's `goto theend`.
#![forbid(unsafe_code)]

use super::wrappers::arg_number_chk;
use crate::option::SavedCpo;

use crate::cursor::check_cursor;
use crate::eval::typval::{NumBuf, tv_list_alloc_ret};
use crate::eval::{eval_expr_to_bool, eval_expr_valid_arg};
use crate::mark::setpcmark;
use crate::memline::{decl, incl};
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::normal::find_decl_of;
use crate::option::vars::{P_WS, p_ws};

use crate::pos::equalpos;
use crate::profile::profile_setlimit;
use crate::search::{
    BACKWARD, FORWARD, SEARCH_COL, SEARCH_END, SEARCH_KEEP, SEARCH_START, search_in_current,
};
use crate::semsg;
use crate::types::{Direction, EvalFuncData, FAIL, LineNr, Pos, TypVal, VarNumber, int64_t};
use crate::winlayer::Win;
use core::ffi::c_int;

/// Accept a match at the cursor's own position ('c').
const SP_START: c_int = 0x10;
/// Leave the cursor at the end of the match ('e').
const SP_END: c_int = 0x40;
/// Answer with the number of matches rather than a line ('m').
const SP_RETCOUNT: c_int = 0x04;
/// Do not move the cursor ('n').
const SP_NOMOVE: c_int = 0x01;
/// Answer with the number of the sub-pattern that matched ('p').
const SP_SUBPAT: c_int = 0x20;
/// Keep searching for the outer pair ('r').
const SP_REPEAT: c_int = 0x02;
/// Set the previous-context mark before moving ('s').
const SP_SETPCMARK: c_int = 0x08;
/// Start at the cursor's column rather than at the start of the line ('z').
const SP_COLUMN: c_int = 0x80;

/// The flag letters that contribute a bit. `b`, `w` and `W` are handled
/// separately: they steer the direction and 'wrapscan' instead.
const FLAG_BITS: [(u8, c_int); 8] = [
    (b'c', SP_START),
    (b'e', SP_END),
    (b'm', SP_RETCOUNT),
    (b'n', SP_NOMOVE),
    (b'p', SP_SUBPAT),
    (b'r', SP_REPEAT),
    (b's', SP_SETPCMARK),
    (b'z', SP_COLUMN),
];

/// A search entry point's saved 'wrapscan'.
///
/// The flag parser writes the option directly (that is what `w` and `W`
/// mean) and every caller restores it, including on the error paths the C
/// reaches with `goto theend`.
struct SavedWrapScan(bool);

impl SavedWrapScan {
    fn new() -> Self {
        SavedWrapScan(p_ws())
    }
}

impl Drop for SavedWrapScan {
    fn drop(&mut self) {
        P_WS.set(self.0);
    }
}

/// Parse a `{flags}` argument.
///
/// Returns [`FORWARD`], [`BACKWARD`], or 0 for an error already reported.
/// Sets the bits it recognises in `flags`, and may write 'wrapscan'.
fn search_direction(varp: Option<&TypVal>, flags: &mut c_int) -> c_int {
    let mut dir = FORWARD as c_int;
    let Some(varp) = varp else {
        return FORWARD as c_int;
    };
    let mut nbuf = NumBuf::new();
    let Some(text) = nbuf.bytes_chk(varp) else {
        // Type error; the message is already out.
        return 0;
    };
    for (at, &letter) in text.iter().enumerate() {
        match letter {
            b'b' => dir = BACKWARD as c_int,
            b'w' => P_WS.set(true),
            b'W' => P_WS.set(false),
            letter => match FLAG_BITS.iter().find(|&&(l, _)| l == letter) {
                Some(&(_, mask)) => *flags |= mask,
                None => {
                    // The message quotes the rest of the flag string
                    // from the offending letter on, not just the
                    // letter, and those are arbitrary user bytes.
                    let p = msg_bytes(&text[at..]);
                    semsg!("E475: Invalid argument: {p}");
                    dir = 0;
                }
            },
        }
        if dir == 0 {
            break;
        }
    }
    dir
}

/// Shared by `search()` and `searchpos()`.
///
/// Answers the matched line (or the sub-pattern number under `p`), 0 for no
/// match, and writes the one-based match position through `match_pos`.
fn search_cmn(args: &[TypVal], match_pos: Option<&mut Pos>, flagsp: &mut c_int) -> c_int {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let _wrapscan = SavedWrapScan::new();
    let mut lnum_stop: LineNr = 0;
    let mut time_limit: int64_t = 0;
    let mut options = SEARCH_KEEP as c_int;
    let mut use_skip = false;

    // A copy: {skip} runs user code between two searches, and the search
    // must not hold onto anything an argument owns across it.
    let pat = numbuf.string(&args[0]).to_bytes().to_vec();
    // May set 'wrapscan'.
    let dir = search_direction(args.get(1), flagsp);
    if dir == 0 {
        return 0;
    }
    let flags = *flagsp;
    if flags & SP_START != 0 {
        options |= SEARCH_START as c_int;
    }
    if flags & SP_END != 0 {
        options |= SEARCH_END as c_int;
    }
    if flags & SP_COLUMN != 0 {
        options |= SEARCH_COL as c_int;
    }

    // The optional {stopline}, {timeout} and {skip} arguments. Each is
    // only read when the one before it was supplied, so a {skip} passed
    // without a {flags} is silently ignored.
    if args.len() > 1 && args.len() > 2 {
        lnum_stop = arg_number_chk(&args[2], None) as LineNr;
        if lnum_stop < 0 {
            return 0;
        }
        if args.len() > 3 {
            time_limit = arg_number_chk(&args[3], None) as int64_t;
            if time_limit < 0 {
                return 0;
            }
            use_skip = args.get(4).is_some_and(eval_expr_valid_arg);
        }
    }
    let mut tm = profile_setlimit(time_limit);

    // `m` and `r` belong to searchpair(); `n` and `s` contradict each
    // other.
    if flags & (SP_REPEAT | SP_RETCOUNT) != 0
        || (flags & SP_NOMOVE != 0 && flags & SP_SETPCMARK != 0)
    {
        let what = msg_cstr(numbuf2.string(&args[1]));
        semsg!("E475: Invalid argument: {what}");
        return 0;
    }

    let save_cursor = Win::current().w_cursor;
    let mut pos = save_cursor;
    let mut firstpos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };

    // Repeat until {skip} answers false.
    let mut subpatnum;
    loop {
        subpatnum = search_in_current(
            &mut pos,
            dir as Direction,
            &pat,
            options,
            lnum_stop,
            &mut tm,
        );
        // Coming back to the first match means every match was skipped.
        if firstpos.lnum != 0 && equalpos(pos, firstpos) {
            subpatnum = FAIL;
        }
        if subpatnum == FAIL || !use_skip {
            break;
        }
        if firstpos.lnum == 0 {
            firstpos = pos;
        }

        // {skip} is evaluated with the cursor on the match.
        let save_pos = Win::current().w_cursor;
        Win::current().w_cursor = pos;
        let answer = eval_expr_to_bool(&args[4]);
        Win::current().w_cursor = save_pos;
        let Ok(do_skip) = answer else {
            subpatnum = FAIL;
            break;
        };
        if !do_skip {
            break;
        }
        // Clear the start flag so that the next round moves on.
        options &= !(SEARCH_START as c_int);
    }

    let mut retval = 0;
    if subpatnum != FAIL {
        retval = if flags & SP_SUBPAT != 0 {
            subpatnum
        } else {
            pos.lnum as c_int
        };
        if flags & SP_SETPCMARK != 0 {
            setpcmark();
        }
        Win::current().w_cursor = pos;
        if let Some(match_pos) = match_pos {
            match_pos.lnum = pos.lnum;
            match_pos.col = pos.col + 1;
        }
        // A `/$` match leaves the cursor past the end of the line.
        check_cursor(Win::current());
    }

    if flags & SP_NOMOVE != 0 {
        Win::current().w_cursor = save_cursor;
    } else {
        Win::current().w_set_curswant = true;
    }
    retval
}

/// `search({pattern} [, {flags} [, {stopline} [, {timeout} [, {skip}]]]])`
pub fn f_search(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut flags = 0;
    result.write_number(search_cmn(args, None, &mut flags) as VarNumber);
}

/// `searchpos()` — as `search()`, but answering `[lnum, col]`, plus the
/// sub-pattern number under the `p` flag.
pub fn f_searchpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut match_pos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut flags = 0;
    let n = search_cmn(args, Some(&mut match_pos), &mut flags);
    let list = tv_list_alloc_ret(result, 2 + (flags & SP_SUBPAT != 0) as isize);
    let (lnum, col) = if n > 0 {
        (match_pos.lnum as c_int, match_pos.col as c_int)
    } else {
        (0, 0)
    };
    list.push_number(lnum as VarNumber);
    list.push_number(col as VarNumber);
    if flags & SP_SUBPAT != 0 {
        list.push_number(n as VarNumber);
    }
}

/// `searchdecl({name} [, {global} [, {thisblock}]])` — 0 when the
/// declaration was found, 1 otherwise.
pub fn f_searchdecl(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut locally = true;
    let mut thisblock = false;
    let mut error = false;
    // Default: FAIL.
    result.write_number(1);

    let name = numbuf.string_chk(&args[0]);
    if args.len() > 1 {
        locally = arg_number_chk(&args[1], Some(&mut error)) == 0;
        if !error && args.len() > 2 {
            thisblock = arg_number_chk(&args[2], Some(&mut error)) != 0;
        }
    }
    if !error && let Some(name) = name {
        let found = find_decl_of(name, locally, thisblock, SEARCH_KEEP as c_int);
        result.write_number((found as c_int == FAIL) as VarNumber);
    }
}

/// Shared by `searchpair()` and `searchpairpos()`: parse the arguments and
/// hand them to [`do_searchpair`].
fn searchpair_cmn(args: &[TypVal], match_pos: Option<&mut Pos>) -> c_int {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let _wrapscan = SavedWrapScan::new();
    let mut flags = 0;
    let mut lnum_stop: LineNr = 0;
    let mut time_limit: int64_t = 0;

    let mut nbuf1 = NumBuf::new();
    let mut nbuf2 = NumBuf::new();
    let spat = numbuf.string_chk(&args[0]);
    let mpat = nbuf1.string_chk(&args[1]);
    let epat = nbuf2.string_chk(&args[2]);
    let (Some(spat), Some(mpat), Some(epat)) = (spat, mpat, epat) else {
        // Type error, already reported.
        return 0;
    };

    // May set 'wrapscan'.
    let dir = search_direction(args.get(3), &mut flags);
    if dir == 0 {
        return 0;
    }

    // `e` and `p` belong to search(); `n` and `s` contradict each other.
    if flags & (SP_END | SP_SUBPAT) != 0 || (flags & SP_NOMOVE != 0 && flags & SP_SETPCMARK != 0) {
        let what = msg_cstr(numbuf2.string(&args[3]));
        semsg!("E475: Invalid argument: {what}");
        return 0;
    }

    // `r` implies `W`; without it the repeat would wrap forever.
    if flags & SP_REPEAT != 0 {
        P_WS.set(false);
    }

    // The optional {skip}, {stopline} and {timeout}. As in search(),
    // each is only read when the one before it was supplied.
    let skip = if args.len() <= 3 || args.len() <= 4 {
        None
    } else {
        // The type is checked later, when the expression is evaluated.
        if args.len() > 5 {
            lnum_stop = arg_number_chk(&args[5], None) as LineNr;
            if lnum_stop < 0 {
                let what = msg_cstr(numbuf3.string(&args[5]));
                semsg!("E475: Invalid argument: {what}");
                return 0;
            }
            if args.len() > 6 {
                time_limit = arg_number_chk(&args[6], None) as int64_t;
                if time_limit < 0 {
                    let what = msg_cstr(numbuf4.string(&args[6]));
                    semsg!("E475: Invalid argument: {what}");
                    return 0;
                }
            }
        }
        Some(&args[4])
    };

    // `do_searchpair` builds its own patterns out of these before {skip}
    // can run.
    let (spat, mpat, epat) = (spat.to_bytes(), mpat.to_bytes(), epat.to_bytes());
    do_searchpair(
        spat, mpat, epat, dir, skip, flags, match_pos, lnum_stop, time_limit,
    )
}

/// `searchpair({start}, {middle}, {end} [, {flags} [, {skip} [, {stopline}
/// [, {timeout}]]]])`
pub fn f_searchpair(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(searchpair_cmn(args, None) as VarNumber);
}

/// `searchpairpos()` — as `searchpair()`, answering `[lnum, col]`.
pub fn f_searchpairpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut match_pos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let (mut lnum, mut col) = (0, 0);
    // The List is allocated after the search: {skip} is user code, and
    // the return value is not yet anything it could reach either way.
    if searchpair_cmn(args, Some(&mut match_pos)) > 0 {
        lnum = match_pos.lnum as c_int;
        col = match_pos.col as c_int;
    }
    let list = tv_list_alloc_ret(result, 2);
    list.push_number(lnum as VarNumber);
    list.push_number(col as VarNumber);
}

/// The alternation `do_searchpair` hands to `searchit`.
///
/// Each pattern becomes its own `\(…\)` group with `\m` forced on around
/// it, so that neither a caller's 'magic' setting nor one pattern's magic
/// escapes can change what the next one means. The group a match landed in
/// is what `searchit`'s answer names, and that is how the walk below tells
/// a start from an end from a middle.
fn alternation(pats: &[&[u8]]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for (i, pat) in pats.iter().enumerate() {
        out.extend_from_slice(if i == 0 { b"\\m\\(" } else { b"\\|\\(" });
        out.extend_from_slice(pat);
        out.extend_from_slice(b"\\m\\)");
    }
    out
}

/// Search for a start/middle/end triple, honouring nesting.
///
/// Also used by the `it`/`at` text objects, which pass a null `skip` and no
/// flags. Answers the matched line, the match count under `m`, 0 for no
/// match, or -1 when evaluating `skip` failed.
///
/// `skip` is evaluated with the cursor on each match and may run any user
/// code; nothing borrowed is held across it.
#[allow(clippy::too_many_arguments)]
pub fn do_searchpair(
    spat: &[u8],
    mpat: &[u8],
    epat: &[u8],
    dir: c_int,
    skip: Option<&TypVal>,
    flags: c_int,
    match_pos: Option<&mut Pos>,
    lnum_stop: LineNr,
    time_limit: int64_t,
) -> c_int {
    let _cpo = SavedCpo::empty_under_user_code();
    let mut retval = 0;
    let mut nest = 1;
    let mut options = SEARCH_KEEP as c_int;

    let mut tm = profile_setlimit(time_limit);

    // Without a middle pattern the nested search is the same as the
    // outer one.
    let outer = alternation(&[spat, epat]);
    let full = if mpat.is_empty() {
        outer.clone()
    } else {
        alternation(&[spat, epat, mpat])
    };

    if flags & SP_START != 0 {
        options |= SEARCH_START as c_int;
    }
    let use_skip = skip.is_some_and(eval_expr_valid_arg);

    let save_cursor = Win::current().w_cursor;
    let mut pos = save_cursor;
    let mut firstpos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut foundpos = firstpos;

    // Start on the full alternation; drop the middle pattern while
    // nested, since a middle only counts at the outermost level.
    let mut pat = &full;
    loop {
        let n = search_in_current(&mut pos, dir as Direction, pat, options, lnum_stop, &mut tm);
        // No match, or back at the first one: the walk is done.
        if n == FAIL || (firstpos.lnum != 0 && equalpos(pos, firstpos)) {
            break;
        }
        if firstpos.lnum == 0 {
            firstpos = pos;
        }
        // Landing on the same spot twice means a zero-width match; step
        // over it so that the walk makes progress.
        if equalpos(pos, foundpos) {
            if dir == BACKWARD as c_int {
                decl(&mut pos);
            } else {
                incl(&mut pos);
            }
        }
        foundpos = pos;

        // Clear the start flag so that the next round moves on.
        options &= !(SEARCH_START as c_int);

        if use_skip {
            let save_pos = Win::current().w_cursor;
            Win::current().w_cursor = pos;
            let expr = skip.expect("`use_skip` means there is one");
            let answer = eval_expr_to_bool(expr);
            Win::current().w_cursor = save_pos;
            let Ok(skipped) = answer else {
                Win::current().w_cursor = save_cursor;
                retval = -1;
                break;
            };
            if skipped {
                continue;
            }
        }

        // Group 2 is the end pattern and group 3 the middle one, so
        // searching backwards a middle opens a level and searching
        // forwards an end does.
        if (dir == BACKWARD as c_int && n == 3) || (dir == FORWARD as c_int && n == 2) {
            nest += 1;
            pat = &outer;
        } else {
            nest -= 1;
            if nest == 1 {
                pat = &full;
            }
        }
        if nest != 0 {
            continue;
        }

        // Back at the outermost level: this is a result.
        if flags & SP_RETCOUNT != 0 {
            retval += 1;
        } else {
            retval = pos.lnum as c_int;
        }
        if flags & SP_SETPCMARK != 0 {
            setpcmark();
        }
        Win::current().w_cursor = pos;
        if flags & SP_REPEAT == 0 {
            break;
        }
        nest = 1;
    }

    if let Some(match_pos) = match_pos {
        match_pos.lnum = Win::current().w_cursor.lnum;
        match_pos.col = Win::current().w_cursor.col + 1;
    }
    if flags & SP_NOMOVE != 0 || retval == 0 {
        Win::current().w_cursor = save_cursor;
    }
    retval
}
