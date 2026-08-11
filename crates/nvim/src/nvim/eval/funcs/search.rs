//! Searching the buffer: `search()`, `searchpos()`, `searchpair()`,
//! `searchpairpos()` and `searchdecl()`.
//!
//! The three real entry points are [`search_cmn`], [`searchpair_cmn`] and
//! [`do_searchpair`]; the `f_*` bodies are thin. Every one of them lets the
//! flag parser write 'wrapscan' and puts the caller's value back on the way
//! out, which [`SavedWrapScan`] does here instead of the C's `goto theend`.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::{FAIL, NUL, false_0, true_0};
use crate::semsg_c;
use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::eval::typval::{
    tv_get_number_chk, tv_get_string, tv_get_string_buf_chk, tv_get_string_chk, tv_list_alloc_ret,
    tv_list_append_number,
};
use crate::src::nvim::eval::{eval_expr_to_bool, eval_expr_valid_arg};
use crate::src::nvim::main::{curbuf, curwin, e_invarg2, empty_string_option, p_cpo, p_ws};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::memline::{decl, incl};
use crate::src::nvim::normal::find_decl;
use crate::src::nvim::option::{kOptValTypeString, set_option_value_give_err};
use crate::src::nvim::options::kOptCpoptions;
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::pos::equalpos;
use crate::src::nvim::profile::profile_setlimit;
use crate::src::nvim::regexp::RE_SEARCH;
use crate::src::nvim::search::{
    BACKWARD, FORWARD, SEARCH_COL, SEARCH_END, SEARCH_KEEP, SEARCH_START, searchit,
};
use crate::src::nvim::types::{
    Direction, EvalFuncData, OptVal, OptValData, VAR_UNKNOWN, int64_t, linenr_T, pos_T,
    searchit_arg_T, typval_T, varnumber_T,
};
use core::ffi::{c_char, c_int};
use core::ptr;

/// The size of a `tv_get_string_buf_chk` scratch buffer. `NUMBUFLEN` in the
/// C: enough for the decimal spelling of any Number.
const NUMBUFLEN: usize = 65;

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
struct SavedWrapScan(c_int);

impl SavedWrapScan {
    fn new() -> Self {
        SavedWrapScan(p_ws.get())
    }
}

impl Drop for SavedWrapScan {
    fn drop(&mut self) {
        p_ws.set(self.0);
    }
}

/// Parse a `{flags}` argument.
///
/// Returns [`FORWARD`], [`BACKWARD`], or 0 for an error already reported.
/// Sets the bits it recognises in `flags`, and may write 'wrapscan'.
///
/// # Safety
/// `varp` is a live typval.
unsafe fn search_direction(varp: *mut typval_T, flags: &mut c_int) -> c_int {
    let mut dir = FORWARD as c_int;
    // SAFETY: the caller's obligation; `nbuf` outlives the string
    // `tv_get_string_buf_chk` may park in it.
    unsafe {
        if (*varp).v_type == VAR_UNKNOWN {
            return FORWARD as c_int;
        }
        let mut nbuf = [0 as c_char; NUMBUFLEN];
        let mut p = tv_get_string_buf_chk(varp, nbuf.as_mut_ptr());
        if p.is_null() {
            // Type error; the message is already out.
            return 0;
        }
        while *p as c_int != NUL {
            match *p as u8 {
                b'b' => dir = BACKWARD as c_int,
                b'w' => p_ws.set(true_0),
                b'W' => p_ws.set(false_0),
                letter => match FLAG_BITS.iter().find(|&&(l, _)| l == letter) {
                    Some(&(_, mask)) => *flags |= mask,
                    None => {
                        // The message quotes the rest of the flag string
                        // from the offending letter on, not just the
                        // letter, and those are arbitrary user bytes.
                        semsg_c!(gettext(e_invarg2.as_ptr()), p);
                        dir = 0;
                    }
                },
            }
            if dir == 0 {
                break;
            }
            p = p.add(1);
        }
        dir
    }
}

/// Shared by `search()` and `searchpos()`.
///
/// Answers the matched line (or the sub-pattern number under `p`), 0 for no
/// match, and writes the one-based match position through `match_pos`.
///
/// # Safety
/// `args` is a live call frame.
unsafe fn search_cmn(args: Args, match_pos: Option<&mut pos_T>, flagsp: &mut c_int) -> c_int {
    let _wrapscan = SavedWrapScan::new();
    let mut lnum_stop: linenr_T = 0;
    let mut time_limit: int64_t = 0;
    let mut options = SEARCH_KEEP as c_int;
    let mut use_skip = false;

    // SAFETY: the frame's arguments and the current window are live for the
    // whole call; `pos`/`firstpos`/`tm` are locals handed to `searchit` by
    // pointer and outlive it.
    unsafe {
        let pat = tv_get_string(args.ptr(0));
        // May set 'wrapscan'.
        let dir = search_direction(args.ptr(1), flagsp);
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
        if args.has(1) && args.has(2) {
            lnum_stop = tv_get_number_chk(args.ptr(2), ptr::null_mut()) as linenr_T;
            if lnum_stop < 0 {
                return 0;
            }
            if args.has(3) {
                time_limit = tv_get_number_chk(args.ptr(3), ptr::null_mut()) as int64_t;
                if time_limit < 0 {
                    return 0;
                }
                use_skip = eval_expr_valid_arg(args.ptr(4));
            }
        }
        let mut tm = profile_setlimit(time_limit);

        // `m` and `r` belong to searchpair(); `n` and `s` contradict each
        // other.
        if flags & (SP_REPEAT | SP_RETCOUNT) != 0
            || (flags & SP_NOMOVE != 0 && flags & SP_SETPCMARK != 0)
        {
            semsg_c!(gettext(e_invarg2.as_ptr()), tv_get_string(args.ptr(1)),);
            return 0;
        }

        let save_cursor = (*curwin.get()).w_cursor;
        let mut pos = save_cursor;
        let mut firstpos = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut sia = searchit_arg_T {
            sa_stop_lnum: lnum_stop,
            sa_tm: &raw mut tm,
            sa_timed_out: 0,
            sa_wrapped: 0,
        };
        let patlen = strlen(pat);

        // Repeat until {skip} answers false.
        let mut subpatnum;
        loop {
            subpatnum = searchit(
                curwin.get(),
                curbuf.get(),
                &raw mut pos,
                ptr::null_mut(),
                dir as Direction,
                pat as *mut c_char,
                patlen,
                1,
                options,
                RE_SEARCH as c_int,
                &raw mut sia,
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
            let save_pos = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = pos;
            let mut err = false;
            let do_skip = eval_expr_to_bool(args.ptr(4), &raw mut err);
            (*curwin.get()).w_cursor = save_pos;
            if err {
                subpatnum = FAIL;
                break;
            }
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
            (*curwin.get()).w_cursor = pos;
            if let Some(match_pos) = match_pos {
                match_pos.lnum = pos.lnum;
                match_pos.col = pos.col + 1;
            }
            // A `/$` match leaves the cursor past the end of the line.
            check_cursor(curwin.get());
        }

        if flags & SP_NOMOVE != 0 {
            (*curwin.get()).w_cursor = save_cursor;
        } else {
            (*curwin.get()).w_set_curswant = true_0;
        }
        retval
    }
}

/// `search({pattern} [, {flags} [, {stopline} [, {timeout} [, {skip}]]]])`
pub unsafe extern "C" fn f_search(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut flags = 0;
    // SAFETY: the frame is live.
    rettv.vval.v_number = unsafe { search_cmn(args, None, &mut flags) } as varnumber_T;
}

/// `searchpos()` — as `search()`, but answering `[lnum, col]`, plus the
/// sub-pattern number under the `p` flag.
pub unsafe extern "C" fn f_searchpos(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut match_pos = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut flags = 0;
    // SAFETY: the frame is live and `rettv` is the cleared return value.
    unsafe {
        let n = search_cmn(args, Some(&mut match_pos), &mut flags);
        let list = tv_list_alloc_ret(rettv, 2 + (flags & SP_SUBPAT != 0) as isize);
        let (lnum, col) = if n > 0 {
            (match_pos.lnum as c_int, match_pos.col as c_int)
        } else {
            (0, 0)
        };
        tv_list_append_number(list, lnum as varnumber_T);
        tv_list_append_number(list, col as varnumber_T);
        if flags & SP_SUBPAT != 0 {
            tv_list_append_number(list, n as varnumber_T);
        }
    }
}

/// `searchdecl({name} [, {global} [, {thisblock}]])` — 0 when the
/// declaration was found, 1 otherwise.
pub unsafe extern "C" fn f_searchdecl(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut locally = true;
    let mut thisblock = false;
    let mut error = false;
    // Default: FAIL.
    rettv.vval.v_number = 1;

    // SAFETY: the frame's arguments are live typvals and `name` is the
    // string one of them owns, which outlives the `find_decl` call.
    unsafe {
        let name = tv_get_string_chk(args.ptr(0));
        if args.has(1) {
            locally = tv_get_number_chk(args.ptr(1), &raw mut error) == 0;
            if !error && args.has(2) {
                thisblock = tv_get_number_chk(args.ptr(2), &raw mut error) != 0;
            }
        }
        if !error && !name.is_null() {
            let found = find_decl(
                name as *mut c_char,
                strlen(name),
                locally,
                thisblock,
                SEARCH_KEEP as c_int,
            );
            rettv.vval.v_number = (found as c_int == FAIL) as varnumber_T;
        }
    }
}

/// Shared by `searchpair()` and `searchpairpos()`: parse the arguments and
/// hand them to [`do_searchpair`].
///
/// # Safety
/// `args` is a live call frame.
unsafe fn searchpair_cmn(args: Args, match_pos: Option<&mut pos_T>) -> c_int {
    let _wrapscan = SavedWrapScan::new();
    let mut flags = 0;
    let mut lnum_stop: linenr_T = 0;
    let mut time_limit: int64_t = 0;

    // SAFETY: the frame's arguments are live typvals; the two scratch
    // buffers outlive the strings `tv_get_string_buf_chk` may park in them,
    // and the three patterns outlive the `do_searchpair` call.
    unsafe {
        let mut nbuf1 = [0 as c_char; NUMBUFLEN];
        let mut nbuf2 = [0 as c_char; NUMBUFLEN];
        let spat = tv_get_string_chk(args.ptr(0));
        let mpat = tv_get_string_buf_chk(args.ptr(1), nbuf1.as_mut_ptr());
        let epat = tv_get_string_buf_chk(args.ptr(2), nbuf2.as_mut_ptr());
        if spat.is_null() || mpat.is_null() || epat.is_null() {
            // Type error, already reported.
            return 0;
        }

        // May set 'wrapscan'.
        let dir = search_direction(args.ptr(3), &mut flags);
        if dir == 0 {
            return 0;
        }

        // `e` and `p` belong to search(); `n` and `s` contradict each other.
        if flags & (SP_END | SP_SUBPAT) != 0
            || (flags & SP_NOMOVE != 0 && flags & SP_SETPCMARK != 0)
        {
            semsg_c!(gettext(e_invarg2.as_ptr()), tv_get_string(args.ptr(3)),);
            return 0;
        }

        // `r` implies `W`; without it the repeat would wrap forever.
        if flags & SP_REPEAT != 0 {
            p_ws.set(false_0);
        }

        // The optional {skip}, {stopline} and {timeout}. As in search(),
        // each is only read when the one before it was supplied.
        let skip = if !args.has(3) || !args.has(4) {
            ptr::null()
        } else {
            // The type is checked later, when the expression is evaluated.
            if args.has(5) {
                lnum_stop = tv_get_number_chk(args.ptr(5), ptr::null_mut()) as linenr_T;
                if lnum_stop < 0 {
                    semsg_c!(gettext(e_invarg2.as_ptr()), tv_get_string(args.ptr(5)),);
                    return 0;
                }
                if args.has(6) {
                    time_limit = tv_get_number_chk(args.ptr(6), ptr::null_mut()) as int64_t;
                    if time_limit < 0 {
                        semsg_c!(gettext(e_invarg2.as_ptr()), tv_get_string(args.ptr(6)),);
                        return 0;
                    }
                }
            }
            args.ptr(4) as *const typval_T
        };

        do_searchpair(
            spat,
            mpat,
            epat,
            dir,
            skip,
            flags,
            match_pos.map_or(ptr::null_mut(), |p| p as *mut pos_T),
            lnum_stop,
            time_limit,
        )
    }
}

/// `searchpair({start}, {middle}, {end} [, {flags} [, {skip} [, {stopline}
/// [, {timeout}]]]])`
pub unsafe extern "C" fn f_searchpair(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live.
    rettv.vval.v_number = unsafe { searchpair_cmn(args, None) } as varnumber_T;
}

/// `searchpairpos()` — as `searchpair()`, answering `[lnum, col]`.
pub unsafe extern "C" fn f_searchpairpos(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut match_pos = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let (mut lnum, mut col) = (0, 0);
    // SAFETY: the frame is live and `rettv` is the cleared return value.
    unsafe {
        let list = tv_list_alloc_ret(rettv, 2);
        if searchpair_cmn(args, Some(&mut match_pos)) > 0 {
            lnum = match_pos.lnum as c_int;
            col = match_pos.col as c_int;
        }
        tv_list_append_number(list, lnum as varnumber_T);
        tv_list_append_number(list, col as varnumber_T);
    }
}

/// The alternation `do_searchpair` hands to `searchit`, NUL-terminated.
///
/// Each pattern becomes its own `\(…\)` group with `\m` forced on around
/// it, so that neither a caller's 'magic' setting nor one pattern's magic
/// escapes can change what the next one means. The group a match landed in
/// is what `searchit`'s answer names, and that is how the walk below tells
/// a start from an end from a middle.
fn alternation(pats: &[&[u8]]) -> Vec<c_char> {
    let mut out: Vec<u8> = Vec::new();
    for (i, pat) in pats.iter().enumerate() {
        out.extend_from_slice(if i == 0 { b"\\m\\(" } else { b"\\|\\(" });
        out.extend_from_slice(pat);
        out.extend_from_slice(b"\\m\\)");
    }
    out.push(0);
    out.into_iter().map(|b| b as c_char).collect()
}

/// 'cpoptions' emptied for the duration of a pair search, so that a `cpo-l`
/// setting cannot change what the patterns mean.
///
/// Restoring is not a plain assignment: the {skip} expression is arbitrary
/// vimscript and may have set the option itself. If it left our own empty
/// string in place nothing happened and the saved value goes straight back;
/// if it left some *other* empty string the option was set and restored
/// behind our back, and the saved value has to go through the option
/// machinery so that everything watching 'cpoptions' hears about it.
struct EmptyCpo(*mut c_char);

impl EmptyCpo {
    fn new() -> Self {
        let saved = p_cpo.get();
        p_cpo.set(empty_string_option.ptr() as *mut c_char);
        EmptyCpo(saved)
    }
}

impl Drop for EmptyCpo {
    fn drop(&mut self) {
        if p_cpo.get() == empty_string_option.ptr() as *mut c_char {
            p_cpo.set(self.0);
            return;
        }
        // SAFETY: `self.0` is the string the option owned on entry and is
        // still live; `set_option_value_give_err` copies it and
        // `free_string_option` then releases our claim on it.
        unsafe {
            if *p_cpo.get() == 0 {
                set_option_value_give_err(
                    kOptCpoptions,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_as_string(self.0),
                        },
                    },
                    0,
                );
            }
            free_string_option(self.0);
        }
    }
}

/// Search for a start/middle/end triple, honouring nesting.
///
/// Also used by the `it`/`at` text objects, which pass a null `skip` and no
/// flags. Answers the matched line, the match count under `m`, 0 for no
/// match, or -1 when evaluating `skip` failed.
///
/// # Safety
/// `spat`, `mpat` and `epat` are non-null C strings; `skip` is null or a
/// live typval; `match_pos` is null or writable.
pub unsafe extern "C" fn do_searchpair(
    spat: *const c_char,
    mpat: *const c_char,
    epat: *const c_char,
    dir: c_int,
    skip: *const typval_T,
    flags: c_int,
    match_pos: *mut pos_T,
    lnum_stop: linenr_T,
    time_limit: int64_t,
) -> c_int {
    let _cpo = EmptyCpo::new();
    let mut retval = 0;
    let mut nest = 1;
    let mut options = SEARCH_KEEP as c_int;

    // SAFETY: the caller's obligation on the three patterns and `skip`; the
    // current window is live for the whole call, and `pos`/`tm`/the two
    // pattern buffers are locals that outlive every `searchit` call.
    unsafe {
        let mut tm = profile_setlimit(time_limit);

        // Without a middle pattern the nested search is the same as the
        // outer one.
        let outer = alternation(&[
            core::slice::from_raw_parts(spat as *const u8, strlen(spat)),
            core::slice::from_raw_parts(epat as *const u8, strlen(epat)),
        ]);
        let full = if *mpat == 0 {
            outer.clone()
        } else {
            alternation(&[
                core::slice::from_raw_parts(spat as *const u8, strlen(spat)),
                core::slice::from_raw_parts(epat as *const u8, strlen(epat)),
                core::slice::from_raw_parts(mpat as *const u8, strlen(mpat)),
            ])
        };

        if flags & SP_START != 0 {
            options |= SEARCH_START as c_int;
        }
        let use_skip = !skip.is_null() && eval_expr_valid_arg(skip);

        let save_cursor = (*curwin.get()).w_cursor;
        let mut pos = save_cursor;
        let mut firstpos = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut foundpos = firstpos;

        // Start on the full alternation; drop the middle pattern while
        // nested, since a middle only counts at the outermost level.
        let mut pat = &full;
        loop {
            let mut sia = searchit_arg_T {
                sa_stop_lnum: lnum_stop,
                sa_tm: &raw mut tm,
                sa_timed_out: 0,
                sa_wrapped: 0,
            };
            let n = searchit(
                curwin.get(),
                curbuf.get(),
                &raw mut pos,
                ptr::null_mut(),
                dir as Direction,
                pat.as_ptr() as *mut c_char,
                pat.len() - 1,
                1,
                options,
                RE_SEARCH as c_int,
                &raw mut sia,
            );
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
                    decl(&raw mut pos);
                } else {
                    incl(&raw mut pos);
                }
            }
            foundpos = pos;

            // Clear the start flag so that the next round moves on.
            options &= !(SEARCH_START as c_int);

            if use_skip {
                let save_pos = (*curwin.get()).w_cursor;
                (*curwin.get()).w_cursor = pos;
                let mut err = false;
                let skipped = eval_expr_to_bool(skip, &raw mut err);
                (*curwin.get()).w_cursor = save_pos;
                if err {
                    (*curwin.get()).w_cursor = save_cursor;
                    retval = -1;
                    break;
                }
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
            (*curwin.get()).w_cursor = pos;
            if flags & SP_REPEAT == 0 {
                break;
            }
            nest = 1;
        }

        if !match_pos.is_null() {
            (*match_pos).lnum = (*curwin.get()).w_cursor.lnum;
            (*match_pos).col = (*curwin.get()).w_cursor.col + 1;
        }
        if flags & SP_NOMOVE != 0 || retval == 0 {
            (*curwin.get()).w_cursor = save_cursor;
        }
    }
    retval
}
