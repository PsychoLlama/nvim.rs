//! Rust implementations of `search_cmn`, `searchpair_cmn`, and `do_searchpair`.
//!
//! Migrated from `src/nvim/eval/funcs_shim.c` Phase 3.
//! Also provides `get_search_arg` helper.

#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clashing_extern_declarations)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::similar_names)]

use std::ffi::{c_char, c_int, c_void};

// ─── Type aliases ─────────────────────────────────────────────────────────────

/// Opaque `typval_T *`
type Tv = *mut c_void;
/// Opaque `win_T *`
type WinHandle = *mut c_void;
/// Opaque `list_T *`
type ListPtr = *mut c_void;

// ─── Constants ────────────────────────────────────────────────────────────────

const VAR_UNKNOWN: c_int = 0;

const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;
const FAIL: c_int = 0;

// Search options (from search.h / nvim-rs/search/src/helpers.rs)
const SEARCH_KEEP: c_int = 0x400;
const SEARCH_START: c_int = 0x100;
const SEARCH_END: c_int = 0x40;
const SEARCH_COL: c_int = 0x1000;
const RE_SEARCH: c_int = 0;

// SP_* flags (from funcs_shim.c Phase 33 defines)
const SP_NOMOVE: c_int = 0x01;
const SP_REPEAT: c_int = 0x02;
const SP_RETCOUNT: c_int = 0x04;
const SP_SETPCMARK: c_int = 0x08;
const SP_START: c_int = 0x10;
const SP_SUBPAT: c_int = 0x20;
const SP_END: c_int = 0x40;
const SP_COLUMN: c_int = 0x80;

const NUMBUFLEN: usize = 65;

// ─── C struct mirrors ─────────────────────────────────────────────────────────

/// Mirror of C `pos_T` (12 bytes).
/// Reuse from search::searchit crate via extern; define locally for this module.
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct PosT {
    pub lnum: i32,
    pub col: i32,
    pub coladd: i32,
}

/// Mirror of C `searchit_arg_T`.
/// `sa_tm` is `proftime_T *` (opaque `*mut c_void`).
#[repr(C)]
pub struct SearchitArgT {
    pub sa_stop_lnum: i32,
    pub sa_tm: *mut c_void,
    pub sa_timed_out: c_int,
    pub sa_wrapped: c_int,
}

// ─── FFI imports ──────────────────────────────────────────────────────────────

extern "C" {
    // Typval accessors
    fn nvim_tv_idx(argvars: Tv, i: c_int) -> Tv;
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;
    fn tv_get_string(tv: *const c_void) -> *const c_char;
    fn tv_get_string_chk(tv: *const c_void) -> *const c_char;
    fn tv_get_string_buf_chk(tv: *const c_void, buf: *mut c_char) -> *const c_char;

    // p_ws (wrapscan)
    fn nvim_get_p_ws() -> c_int;
    fn nvim_set_p_ws(val: c_int);

    // p_cpo (cpoptions) — save and empty in one call; restore via nvim_restore_cpo
    fn nvim_eval_save_cpo() -> *mut c_char;

    // profile_setlimit: returns proftime_T (u64)
    fn profile_setlimit(msec: i64) -> u64;

    // searchit (Rust export from search crate)
    fn searchit(
        win: WinHandle,
        buf: *mut c_void,
        pos: *mut PosT,
        end_pos: *mut PosT,
        dir: c_int,
        pat: *mut c_char,
        patlen: usize,
        count: c_int,
        options: c_int,
        pat_use: c_int,
        extra_arg: *mut SearchitArgT,
    ) -> c_int;

    // equalpos / clearpos (Rust exports from mark crate)
    fn rs_equalpos(a: PosT, b: PosT) -> c_int;
    fn rs_clearpos(a: *mut PosT);

    // decl/incl position
    fn nvim_search_decl_pos(lnum: *mut c_int, col: *mut c_int, coladd: *mut c_int) -> c_int;
    fn nvim_search_incl_pos(lnum: *mut c_int, col: *mut c_int, coladd: *mut c_int) -> c_int;

    // cursor
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_curbuf() -> *mut c_void;
    fn nvim_win_get_cursor_ptr(wp: WinHandle) -> *mut PosT;
    fn nvim_curwin_set_curswant(val: bool);

    // setpcmark / check_cursor
    fn setpcmark();
    fn check_cursor(win: WinHandle);

    // eval_expr_to_bool (Rust export from eval_exec crate)
    fn eval_expr_to_bool(expr: *const c_void, error: *mut bool) -> bool;
    fn rs_eval_expr_valid_arg(tv: *const c_void) -> c_int;

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn snprintf(s: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
    fn strlen(s: *const c_char) -> usize;

    // List API for searchpairpos / searchpos return values
    #[link_name = "tv_list_alloc_ret"]
    fn tv_list_alloc_ret(rettv: Tv, len: isize) -> ListPtr;
    #[link_name = "tv_list_append_number"]
    fn tv_list_append_number(list: ListPtr, nr: i64);

    // nvim_restore_cpo from quickfix_shim.c — handles the complex restore
    fn nvim_restore_cpo(saved_cpo: *mut c_void);

    // Error messages
    fn semsg(fmt: *const c_char, ...);
    static e_invarg2: [c_char; 0];

    // nvim_tv_set_number
    fn nvim_tv_set_number(tv: Tv, n: i64);
}

// ─── get_search_arg ───────────────────────────────────────────────────────────

/// Parse the optional flags string argument for search() / searchpos() / searchpair().
///
/// Returns direction (FORWARD=1, BACKWARD=-1) or 0 on error.
/// Sets `*flagsp` with SP_* bits based on the flags string.
///
/// Mirrors C `get_search_arg`.
///
/// # Safety
///
/// `varp` must be a valid `typval_T *`. `flagsp` must be valid if non-null.
pub unsafe fn get_search_arg(varp: *const c_void, flagsp: *mut c_int) -> c_int {
    if nvim_tv_get_type(varp) == VAR_UNKNOWN {
        return FORWARD;
    }

    let mut nbuf = [0u8; NUMBUFLEN];
    let flags_ptr = tv_get_string_buf_chk(varp, nbuf.as_mut_ptr().cast::<c_char>());
    if flags_ptr.is_null() {
        return 0; // type error, errmsg already given
    }

    let mut dir = FORWARD;
    let mut p = flags_ptr;
    loop {
        let c = *p as u8;
        if c == 0 {
            break;
        }
        match c {
            b'b' => dir = BACKWARD,
            b'w' => nvim_set_p_ws(1),
            b'W' => nvim_set_p_ws(0),
            _ => {
                let mask: c_int = if flagsp.is_null() {
                    0
                } else {
                    match c {
                        b'c' => SP_START,
                        b'e' => SP_END,
                        b'm' => SP_RETCOUNT,
                        b'n' => SP_NOMOVE,
                        b'p' => SP_SUBPAT,
                        b'r' => SP_REPEAT,
                        b's' => SP_SETPCMARK,
                        b'z' => SP_COLUMN,
                        _ => 0,
                    }
                };
                if mask == 0 {
                    semsg(e_invarg2.as_ptr(), p);
                    dir = 0;
                } else {
                    *flagsp |= mask;
                }
            }
        }
        if dir == 0 {
            break;
        }
        p = p.add(1);
    }
    dir
}

// ─── search_cmn ───────────────────────────────────────────────────────────────

/// Shared implementation for `search()` and `searchpos()`.
///
/// Returns the matching line number (or subpattern number if SP_SUBPAT), or 0/FAIL.
/// Sets `*match_pos` if non-null and a match was found.
/// Sets SP_* bits in `*flagsp`.
///
/// Mirrors C `search_cmn`.
///
/// # Safety
///
/// `argvars` must point to a valid array of at least 5 `typval_T`.
/// `match_pos` and `flagsp` must be valid (or null for match_pos).
pub unsafe fn search_cmn(argvars: Tv, match_pos: *mut PosT, flagsp: *mut c_int) -> c_int {
    let save_p_ws = nvim_get_p_ws();
    let mut retval = 0;
    let mut lnum_stop: i32 = 0;
    let mut time_limit: i64 = 0;
    let mut options = SEARCH_KEEP;
    let mut use_skip = false;

    let pat = tv_get_string(argvars.cast::<c_void>());

    let arg1 = nvim_tv_idx(argvars, 1);
    let dir = get_search_arg(arg1.cast::<c_void>(), flagsp);
    if dir == 0 {
        nvim_set_p_ws(save_p_ws);
        return retval;
    }

    let flags = *flagsp;
    if flags & SP_START != 0 {
        options |= SEARCH_START;
    }
    if flags & SP_END != 0 {
        options |= SEARCH_END;
    }
    if flags & SP_COLUMN != 0 {
        options |= SEARCH_COL;
    }

    // Optional args: lnum_stop (arg[2]), time_limit (arg[3]), skip (arg[4])
    let arg1_type = nvim_tv_get_type(arg1);
    if arg1_type != VAR_UNKNOWN {
        let arg2 = nvim_tv_idx(argvars, 2);
        if nvim_tv_get_type(arg2) != VAR_UNKNOWN {
            let mut chk_err = false;
            let n = nvim_tv_get_number_chk(arg2.cast::<c_void>(), &raw mut chk_err);
            if chk_err || n < 0 {
                nvim_set_p_ws(save_p_ws);
                return retval;
            }
            lnum_stop = n as i32;

            let arg3 = nvim_tv_idx(argvars, 3);
            if nvim_tv_get_type(arg3) != VAR_UNKNOWN {
                let mut chk_err2 = false;
                let t = nvim_tv_get_number_chk(arg3.cast::<c_void>(), &raw mut chk_err2);
                if chk_err2 || t < 0 {
                    nvim_set_p_ws(save_p_ws);
                    return retval;
                }
                time_limit = t;
                let arg4 = nvim_tv_idx(argvars, 4);
                use_skip = rs_eval_expr_valid_arg(arg4.cast::<c_void>()) != 0;
            }
        }
    }

    // Validate flags: SP_REPEAT and SP_RETCOUNT not accepted.
    // SP_NOMOVE and SP_SETPCMARK cannot both be set.
    if (flags & (SP_REPEAT | SP_RETCOUNT)) != 0
        || ((flags & SP_NOMOVE) != 0 && (flags & SP_SETPCMARK) != 0)
    {
        let arg1_str = tv_get_string(arg1.cast::<c_void>());
        semsg(e_invarg2.as_ptr(), arg1_str);
        nvim_set_p_ws(save_p_ws);
        return retval;
    }

    let mut tm = profile_setlimit(time_limit);
    let curwin = nvim_get_curwin();
    let cursor_ptr = nvim_win_get_cursor_ptr(curwin);
    let save_cursor = *cursor_ptr;
    let mut pos = save_cursor;
    let mut firstpos = PosT::default();

    let patlen = strlen(pat);
    let mut subpatnum;

    loop {
        let mut sia = SearchitArgT {
            sa_stop_lnum: lnum_stop,
            sa_tm: (&raw mut tm).cast::<c_void>(),
            sa_timed_out: 0,
            sa_wrapped: 0,
        };

        subpatnum = searchit(
            curwin,
            nvim_get_curbuf(),
            &raw mut pos,
            std::ptr::null_mut(),
            dir,
            pat.cast_mut().cast::<c_char>(),
            patlen,
            1,
            options,
            RE_SEARCH,
            &raw mut sia,
        );

        // found first match again → no match where skip evaluates to false
        if firstpos.lnum != 0 && rs_equalpos(pos, firstpos) != 0 {
            subpatnum = FAIL;
        }

        if subpatnum == FAIL || !use_skip {
            break;
        }

        if firstpos.lnum == 0 {
            firstpos = pos;
        }

        // Evaluate skip expression
        let arg4 = nvim_tv_idx(argvars, 4);
        let save_pos = *cursor_ptr;
        *cursor_ptr = pos;
        let mut err = false;
        let do_skip = eval_expr_to_bool(arg4.cast::<c_void>(), &raw mut err);
        *cursor_ptr = save_pos;

        if err {
            subpatnum = FAIL;
            break;
        }
        if !do_skip {
            break;
        }

        // clear start flag to avoid getting stuck
        options &= !SEARCH_START;
    }

    if subpatnum != FAIL {
        if flags & SP_SUBPAT != 0 {
            retval = subpatnum;
        } else {
            retval = pos.lnum;
        }
        if flags & SP_SETPCMARK != 0 {
            setpcmark();
        }
        *cursor_ptr = pos;
        if !match_pos.is_null() {
            (*match_pos).lnum = pos.lnum;
            (*match_pos).col = pos.col + 1;
        }
        // check_cursor in case pos is past end of line
        check_cursor(curwin);
    }

    // If 'n' flag: restore cursor
    if flags & SP_NOMOVE != 0 {
        *cursor_ptr = save_cursor;
    } else {
        nvim_curwin_set_curswant(true);
    }

    nvim_set_p_ws(save_p_ws);
    retval
}

// ─── do_searchpair ────────────────────────────────────────────────────────────

/// Search for a start/middle/end pattern pair.
///
/// Mirrors C `do_searchpair`. Exported with `#[export_name]` so C callers
/// in textobject_shim.c can still link to it.
///
/// # Safety
///
/// All string pointers must be valid NUL-terminated strings.
/// `skip` may be null (no skip expression).
/// `match_pos` may be null.
#[export_name = "do_searchpair"]
pub unsafe extern "C" fn rs_do_searchpair(
    spat: *const c_char,
    mpat: *const c_char,
    epat: *const c_char,
    dir: c_int,
    skip: *const c_void,
    flags: c_int,
    match_pos: *mut PosT,
    lnum_stop: i32,
    time_limit: i64,
) -> c_int {
    let mut retval = 0;
    let mut nest: c_int = 1;
    let use_skip = !skip.is_null() && rs_eval_expr_valid_arg(skip) != 0;
    let mut options = SEARCH_KEEP;

    // Save p_cpo and set it to empty_string_option.
    let save_cpo = nvim_eval_save_cpo();

    let mut tm = profile_setlimit(time_limit);

    // Build pat2 = \m\(spat\m\)\|\(epat\m\)
    let spatlen = strlen(spat);
    let epatlen = strlen(epat);
    let mpatlen = strlen(mpat);

    let pat2size = spatlen + epatlen + 17;
    let pat3size = spatlen + mpatlen + epatlen + 25;
    let pat2 = xmalloc(pat2size).cast::<c_char>();
    let pat3 = xmalloc(pat3size).cast::<c_char>();

    let pat2fmt = c"\\m\\(%s\\m\\)\\|\\(%s\\m\\)".as_ptr();
    let pat2len = snprintf(pat2, pat2size, pat2fmt, spat, epat);

    let pat3len: c_int = if *mpat == 0 {
        // Copy pat2 into pat3
        let mut i = 0;
        while i < pat2len as usize {
            *pat3.add(i) = *pat2.add(i);
            i += 1;
        }
        *pat3.add(pat2len as usize) = 0;
        pat2len
    } else {
        let pat3fmt = c"\\m\\(%s\\m\\)\\|\\(%s\\m\\)\\|\\(%s\\m\\)".as_ptr();
        snprintf(pat3, pat3size, pat3fmt, spat, epat, mpat)
    };

    if flags & SP_START != 0 {
        options |= SEARCH_START;
    }

    let curwin = nvim_get_curwin();
    let cursor_ptr = nvim_win_get_cursor_ptr(curwin);
    let save_cursor = *cursor_ptr;
    let mut pos = *cursor_ptr;
    let mut firstpos = PosT::default();
    rs_clearpos(&raw mut firstpos);
    let mut foundpos = PosT::default();
    rs_clearpos(&raw mut foundpos);

    let mut pat = pat3;
    let mut patlen = pat3len as usize;

    'search: loop {
        let mut sia = SearchitArgT {
            sa_stop_lnum: lnum_stop,
            sa_tm: (&raw mut tm).cast::<c_void>(),
            sa_timed_out: 0,
            sa_wrapped: 0,
        };

        let n = searchit(
            curwin,
            nvim_get_curbuf(),
            &raw mut pos,
            std::ptr::null_mut(),
            dir,
            pat,
            patlen,
            1,
            options,
            RE_SEARCH,
            &raw mut sia,
        );

        if n == FAIL || (firstpos.lnum != 0 && rs_equalpos(pos, firstpos) != 0) {
            break;
        }

        if firstpos.lnum == 0 {
            firstpos = pos;
        }

        if rs_equalpos(pos, foundpos) != 0 {
            // Same position again — advance one char to avoid infinite loop
            let (mut lnum, mut col, mut coladd) = (pos.lnum, pos.col, pos.coladd);
            if dir == BACKWARD {
                nvim_search_decl_pos(&raw mut lnum, &raw mut col, &raw mut coladd);
            } else {
                nvim_search_incl_pos(&raw mut lnum, &raw mut col, &raw mut coladd);
            }
            pos.lnum = lnum;
            pos.col = col;
            pos.coladd = coladd;
        }
        foundpos = pos;

        // clear start flag
        options &= !SEARCH_START;

        if use_skip {
            let save_pos = *cursor_ptr;
            *cursor_ptr = pos;
            let mut err = false;
            let r = eval_expr_to_bool(skip, &raw mut err);
            *cursor_ptr = save_pos;
            if err {
                *cursor_ptr = save_cursor;
                retval = -1;
                break;
            }
            if r {
                continue;
            }
        }

        // n==3: found end (backward) or start (forward) → nested pair
        if (dir == BACKWARD && n == 3) || (dir == FORWARD && n == 2) {
            nest += 1;
            pat = pat2;
            patlen = pat2len as usize;
        } else {
            nest -= 1;
            if nest == 1 {
                pat = pat3;
                patlen = pat3len as usize;
            }
        }

        if nest == 0 {
            if flags & SP_RETCOUNT != 0 {
                retval += 1;
            } else {
                retval = pos.lnum;
            }
            if flags & SP_SETPCMARK != 0 {
                setpcmark();
            }
            *cursor_ptr = pos;
            if flags & SP_REPEAT == 0 {
                break 'search;
            }
            nest = 1;
        }
    }

    if !match_pos.is_null() {
        (*match_pos).lnum = (*cursor_ptr).lnum;
        (*match_pos).col = (*cursor_ptr).col + 1;
    }

    if (flags & SP_NOMOVE) != 0 || retval == 0 {
        *cursor_ptr = save_cursor;
    }

    xfree(pat2.cast::<c_void>());
    xfree(pat3.cast::<c_void>());

    // Restore p_cpo — nvim_restore_cpo handles both the simple and complex case.
    nvim_restore_cpo(save_cpo.cast::<c_void>());

    retval
}

// ─── searchpair_cmn ───────────────────────────────────────────────────────────

/// Shared implementation for `searchpair()` and `searchpairpos()`.
///
/// Mirrors C `searchpair_cmn`.
///
/// # Safety
///
/// `argvars` must point to valid `typval_T` array of at least 7 elements.
/// `match_pos` may be null.
pub unsafe fn searchpair_cmn(argvars: Tv, match_pos: *mut PosT) -> c_int {
    let save_p_ws = nvim_get_p_ws();
    let mut flags: c_int = 0;
    let mut retval = 0;
    let mut lnum_stop: i32 = 0;
    let mut time_limit: i64 = 0;

    let mut nbuf1 = [0u8; NUMBUFLEN];
    let mut nbuf2 = [0u8; NUMBUFLEN];

    let spat = tv_get_string_chk(argvars.cast::<c_void>());
    let arg1 = nvim_tv_idx(argvars, 1);
    let mpat = tv_get_string_buf_chk(arg1.cast::<c_void>(), nbuf1.as_mut_ptr().cast::<c_char>());
    let arg2 = nvim_tv_idx(argvars, 2);
    let epat = tv_get_string_buf_chk(arg2.cast::<c_void>(), nbuf2.as_mut_ptr().cast::<c_char>());

    if spat.is_null() || mpat.is_null() || epat.is_null() {
        nvim_set_p_ws(save_p_ws);
        return retval; // type error
    }

    let arg3 = nvim_tv_idx(argvars, 3);
    let dir = get_search_arg(arg3.cast::<c_void>(), &raw mut flags);
    if dir == 0 {
        nvim_set_p_ws(save_p_ws);
        return retval;
    }

    // Don't accept SP_END or SP_SUBPAT.
    // Only one of SP_NOMOVE or SP_SETPCMARK.
    if (flags & (SP_END | SP_SUBPAT)) != 0
        || ((flags & SP_NOMOVE) != 0 && (flags & SP_SETPCMARK) != 0)
    {
        semsg(e_invarg2.as_ptr(), tv_get_string(arg3.cast::<c_void>()));
        nvim_set_p_ws(save_p_ws);
        return retval;
    }

    // 'r' implies 'W'
    if flags & SP_REPEAT != 0 {
        nvim_set_p_ws(0);
    }

    // Optional 5th arg: skip expression
    let skip: *const c_void;
    let arg4 = nvim_tv_idx(argvars, 4);
    if nvim_tv_get_type(arg3) == VAR_UNKNOWN || nvim_tv_get_type(arg4) == VAR_UNKNOWN {
        skip = std::ptr::null();
    } else {
        skip = arg4.cast::<c_void>();

        let arg5 = nvim_tv_idx(argvars, 5);
        if nvim_tv_get_type(arg5) != VAR_UNKNOWN {
            let mut chk_err = false;
            let n = nvim_tv_get_number_chk(arg5.cast::<c_void>(), &raw mut chk_err);
            if n < 0 {
                semsg(e_invarg2.as_ptr(), tv_get_string(arg5.cast::<c_void>()));
                nvim_set_p_ws(save_p_ws);
                return retval;
            }
            lnum_stop = n as i32;

            let arg6 = nvim_tv_idx(argvars, 6);
            if nvim_tv_get_type(arg6) != VAR_UNKNOWN {
                let mut chk_err2 = false;
                let t = nvim_tv_get_number_chk(arg6.cast::<c_void>(), &raw mut chk_err2);
                if t < 0 {
                    semsg(e_invarg2.as_ptr(), tv_get_string(arg6.cast::<c_void>()));
                    nvim_set_p_ws(save_p_ws);
                    return retval;
                }
                time_limit = t;
            }
        }
    }

    retval = rs_do_searchpair(spat, mpat, epat, dir, skip, flags, match_pos, lnum_stop, time_limit);

    nvim_set_p_ws(save_p_ws);
    retval
}

// ─── VimL f_* wrappers ────────────────────────────────────────────────────────
//
// These replace the nvim_eval_* wrappers that misc.rs was calling.
// misc.rs f_search/f_searchpos/f_searchpair/f_searchpairpos now call these directly.

/// Implementation of search() and searchpos() shared logic, returning line number.
/// Called from f_search in misc.rs.
///
/// # Safety
/// `argvars` and `rettv` must be valid.
pub unsafe fn do_search(argvars: Tv, rettv: Tv) {
    let mut flags = 0;
    let result = search_cmn(argvars, std::ptr::null_mut(), &raw mut flags);
    nvim_tv_set_number(rettv, i64::from(result));
}

/// Implementation of searchpairpos() — builds a [lnum, col] list.
/// Called from f_searchpairpos in misc.rs.
///
/// # Safety
/// `argvars` and `rettv` must be valid.
pub unsafe fn do_searchpairpos(argvars: Tv, rettv: Tv) {
    let mut match_pos = PosT::default();
    let mut lnum = 0;
    let mut col = 0;

    let list = tv_list_alloc_ret(rettv, 2);
    if searchpair_cmn(argvars, &raw mut match_pos) > 0 {
        lnum = match_pos.lnum;
        col = match_pos.col;
    }
    tv_list_append_number(list, i64::from(lnum));
    tv_list_append_number(list, i64::from(col));
}

/// Implementation of searchpos() — builds a [lnum, col[, subpat]] list.
/// Called from f_searchpos in misc.rs.
///
/// # Safety
/// `argvars` and `rettv` must be valid.
pub unsafe fn do_searchpos(argvars: Tv, rettv: Tv) {
    let mut match_pos = PosT::default();
    let mut flags = 0;

    let n = search_cmn(argvars, &raw mut match_pos, &raw mut flags);
    let extra = isize::from(flags & SP_SUBPAT != 0);

    let list = tv_list_alloc_ret(rettv, 2 + extra);
    let lnum = if n > 0 { match_pos.lnum } else { 0 };
    let col = if n > 0 { match_pos.col } else { 0 };
    tv_list_append_number(list, i64::from(lnum));
    tv_list_append_number(list, i64::from(col));
    if flags & SP_SUBPAT != 0 {
        tv_list_append_number(list, i64::from(n));
    }
}

/// `searchpair()` result: just returns the integer result of searchpair_cmn.
/// Called from f_searchpair in misc.rs.
///
/// # Safety
/// `argvars` must be valid.
pub unsafe fn do_searchpair_fn(argvars: Tv) -> c_int {
    searchpair_cmn(argvars, std::ptr::null_mut())
}
