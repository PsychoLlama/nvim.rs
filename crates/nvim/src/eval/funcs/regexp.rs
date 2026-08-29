//! Matching a pattern against a string: the `match*()` family.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::wrappers::{arg_number_chk, arg_string, arg_string_chk, check_arg, list_alloc_ret};
use super::{
    NSUBEXP, SomeMatchType, kSomeMatch, kSomeMatchEnd, kSomeMatchList, kSomeMatchStr,
    kSomeMatchStrPos, tv_get_buf,
};
use crate::eval::encode::encode_tv2echo;
use crate::eval::typval::{
    NumBuf, tv_check_for_buffer_arg, tv_check_for_list_arg, tv_check_for_lnum_arg,
    tv_check_for_opt_dict_arg, tv_check_for_string_arg, tv_copy, tv_dict_add_list, tv_dict_add_nr,
    tv_dict_add_str_len, tv_dict_alloc, tv_dict_find, tv_get_bool, tv_get_lnum_buf, tv_list_alloc,
    tv_list_append_dict, tv_list_append_number, tv_list_append_string, tv_list_find, tv_list_first,
    tv_list_item_remove, tv_list_uidx,
};
use crate::main::{
    did_emsg, e_buffer_is_not_loaded, e_invalid_buffer_name_str, e_invargval, p_cpo, p_ic,
};
use crate::mbyte::utfc_ptr2len;
use crate::memline::ml_get_buf;
use crate::memory::{xfree, xmemdupz};
use crate::message::emsg;
use crate::optionstr::empty_option;
use crate::os::cshim::gettext;
use crate::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec_nl, vim_regfree};
use crate::semsg_c;
use crate::types::{
    EvalFuncData, FAIL, VAR_BOOL, VAR_LIST, VAR_STRING, buf_T, colnr_T, dict_T, kListLenMayKnow,
    kListLenUnknown, linenr_T, list_T, listitem_T, regmatch_T, regprog_T, typval_T, varnumber_T,
};
use ::libc::strlen;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// A cleared `regmatch_T`, which `vim_regcomp`'s result is dropped into.
const EMPTY_REGMATCH: regmatch_T = regmatch_T {
    regprog: ptr::null_mut::<regprog_T>(),
    startp: [ptr::null_mut(); 10],
    endp: [ptr::null_mut(); 10],
    rm_matchcol: 0,
    rm_ic: false,
};

/// The `match*()` family compiles its pattern with 'cpoptions' emptied, so
/// that a `cpo-l` or `cpo-\` setting cannot change what a pattern means.
/// Restores the caller's value on drop, which is what makes the early
/// returns below safe to write.
struct EmptyCpo(*mut c_char);

impl EmptyCpo {
    fn new() -> Self {
        let saved = p_cpo.get();
        p_cpo.set(empty_option());
        EmptyCpo(saved)
    }
}

impl Drop for EmptyCpo {
    fn drop(&mut self) {
        p_cpo.set(self.0);
    }
}

/// A compiled pattern, freed on drop.
struct Regprog(regmatch_T);

impl Regprog {
    /// Compile `pat` the way the whole family does. `None` when it did not
    /// compile — `vim_regcomp` has already reported why.
    ///
    /// # Safety
    /// `pat` is NUL-terminated.
    unsafe fn compile(pat: *const c_char) -> Option<Self> {
        let mut rm = EMPTY_REGMATCH;
        // SAFETY: the caller's obligation.
        rm.regprog = unsafe { vim_regcomp(pat, RE_MAGIC + RE_STRING) };
        if rm.regprog.is_null() {
            return None;
        }
        rm.rm_ic = p_ic.get() != 0;
        Some(Regprog(rm))
    }
}

impl Drop for Regprog {
    fn drop(&mut self) {
        // SAFETY: the program was compiled here and is not shared.
        unsafe { vim_regfree(self.0.regprog) }
    }
}

/// An owned string the walk over a List allocates per item.
struct Echoed(*mut c_char);

impl Drop for Echoed {
    fn drop(&mut self) {
        // SAFETY: `encode_tv2echo` returned it, or it is null.
        unsafe { xfree(self.0 as *mut c_void) }
    }
}

/// The shared body of `match()`, `matchend()`, `matchlist()`, `matchstr()`
/// and `matchstrpos()`.
///
/// # Safety
/// `args` is the call frame and `rettv` its cleared return value.
unsafe fn find_some_match(args: Args<'_>, rettv: &mut typval_T, kind: SomeMatchType) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the caller's obligation. Every pointer below either points
    // into an argument (which outlives the call), into `patbuf`, or into
    // the string `tofree` owns.
    let _cpo = EmptyCpo::new();
    rettv.vval.v_number = -1;
    match kind {
        kSomeMatchList => {
            list_alloc_ret(rettv, kListLenMayKnow as isize);
        }
        kSomeMatchStrPos => {
            // Seeded with the "no match" answer, which the tail of this
            // function trims back to three items for a String subject.
            list_alloc_ret(rettv, 4);
            unsafe { tv_list_append_string(rettv.vval.v_list, c"".as_ptr(), 0) };
            unsafe { tv_list_append_number(rettv.vval.v_list, -1) };
            unsafe { tv_list_append_number(rettv.vval.v_list, -1) };
            unsafe { tv_list_append_number(rettv.vval.v_list, -1) };
        }
        kSomeMatchStr => {
            rettv.v_type = VAR_STRING;
            rettv.vval.v_string = ptr::null_mut();
        }
        _ => {}
    }

    let mut l: *mut list_T = ptr::null_mut();
    let mut li: *mut listitem_T = ptr::null_mut();
    let mut str: *mut c_char = ptr::null_mut();
    let mut expr: *mut c_char = ptr::null_mut();
    let mut len: i64 = 0;
    let mut start: i64 = 0;
    let mut nth: i64 = 1;
    let mut startcol: colnr_T = 0;
    let mut idx: c_int = 0;
    let mut matched = false;
    // Owns whatever the List walk echoed most recently.
    let mut tofree = Echoed(ptr::null_mut());

    // Nothing below this point may return early without running the
    // trailing fixup, so the body is one labelled block as the C's
    // `goto theend` was.
    'theend: {
        if args.ty(0) == VAR_LIST {
            l = unsafe { args.get(0).vval.v_list };
            if l.is_null() {
                break 'theend;
            }
            li = unsafe { tv_list_first(l) };
        } else {
            str = arg_string(&mut numbuf, args.get(0)) as *mut c_char;
            expr = str;
            len = unsafe { strlen(str) } as i64;
        }

        let mut patbuf = NumBuf::new();
        let pat = arg_string_chk(&mut patbuf, args.get(1));
        if pat.is_null() {
            break 'theend;
        }

        if args.has(2) {
            let mut error = false;
            start = arg_number_chk(args.get(2), Some(&mut error)) as i64;
            if error {
                break 'theend;
            }
            if !l.is_null() {
                idx = unsafe { tv_list_uidx(l, start as c_int) };
                if idx == -1 {
                    break 'theend;
                }
                li = unsafe { tv_list_find(l, idx) };
            } else {
                if start < 0 {
                    start = 0;
                }
                if start > len {
                    break 'theend;
                }
                // With a `{count}` the start is a column the matcher is
                // told about, so that `^` still anchors to the real
                // start of the string; without one the string itself
                // moves forward.
                if args.has(3) {
                    startcol = start as colnr_T;
                } else {
                    str = unsafe { str.offset(start as isize) };
                    len -= start;
                }
            }
            if args.has(3) {
                nth = arg_number_chk(args.get(3), Some(&mut error)) as i64;
            }
            if error {
                break 'theend;
            }
        }

        let Some(mut prog) = (unsafe { Regprog::compile(pat) }) else {
            break 'theend;
        };
        let regmatch = &mut prog.0;
        loop {
            if !l.is_null() {
                if li.is_null() {
                    matched = false;
                    break;
                }
                tofree = Echoed(unsafe { encode_tv2echo(&raw mut (*li).li_tv, ptr::null_mut()) });
                str = tofree.0;
                expr = str;
                if str.is_null() {
                    break;
                }
            }
            matched = unsafe { vim_regexec_nl(regmatch, str, startcol) };
            // `nth` counts down only on a match: the C spells this as
            // `match && (--nth <= 0)`, and the short circuit is what
            // stops a non-matching List item from consuming a count.
            if matched {
                nth -= 1;
                if nth <= 0 {
                    break;
                }
            }
            if l.is_null() && !matched {
                break;
            }
            if !l.is_null() {
                li = unsafe { (*li).li_next };
                idx += 1;
                continue;
            }
            // Same string, next match: step past the character the
            // match started on. A match that did not advance, or one
            // past the end, ends the search.
            let hit = regmatch.startp[0];
            startcol = unsafe { hit.add(utfc_ptr2len(hit) as usize).offset_from(str) } as colnr_T;
            if startcol > len as colnr_T || unsafe { str.offset(startcol as isize) } <= hit {
                matched = false;
                break;
            }
        }

        if !matched {
            break 'theend;
        }
        match kind {
            kSomeMatchStrPos => {
                // The four items seeded above, overwritten in place.
                let ret_l = unsafe { rettv.vval.v_list };
                let li1 = unsafe { tv_list_first(ret_l) };
                let li2 = unsafe { (*li1).li_next };
                let li3 = unsafe { (*li2).li_next };
                let li4 = unsafe { (*li3).li_next };
                unsafe { xfree((*li1).li_tv.vval.v_string as *mut c_void) };
                let rd = unsafe { regmatch.endp[0].offset_from(regmatch.startp[0]) } as usize;
                let text = unsafe { xmemdupz(regmatch.startp[0].cast(), rd) };
                unsafe { (*li1).li_tv.vval.v_string = text as *mut c_char };
                let start = unsafe { regmatch.startp[0].offset_from(expr) };
                unsafe { (*li3).li_tv.vval.v_number = start as varnumber_T };
                let end = unsafe { regmatch.endp[0].offset_from(expr) };
                unsafe { (*li4).li_tv.vval.v_number = end as varnumber_T };
                if !l.is_null() {
                    unsafe { (*li2).li_tv.vval.v_number = idx as varnumber_T };
                }
            }
            kSomeMatchList => {
                for i in 0..NSUBEXP as usize {
                    if regmatch.endp[i].is_null() {
                        unsafe { tv_list_append_string(rettv.vval.v_list, ptr::null(), 0) };
                    } else {
                        let (start, end) = (regmatch.startp[i], regmatch.endp[i]);
                        let list = unsafe { rettv.vval.v_list };
                        let len = unsafe { end.offset_from(start) };
                        unsafe { tv_list_append_string(list, start, len) };
                    }
                }
            }
            kSomeMatchStr => {
                if !l.is_null() {
                    // A List subject answers with the whole item, not
                    // with the part that matched.
                    unsafe { tv_copy(&raw mut (*li).li_tv, rettv) };
                } else {
                    let rd = unsafe { regmatch.endp[0].offset_from(regmatch.startp[0]) } as usize;
                    rettv.vval.v_string =
                        unsafe { xmemdupz(regmatch.startp[0] as *const c_void, rd) } as *mut c_char;
                }
            }
            _ => {
                if !l.is_null() {
                    rettv.vval.v_number = idx as varnumber_T;
                } else {
                    let edge = if kind == kSomeMatch {
                        regmatch.startp[0]
                    } else {
                        regmatch.endp[0]
                    };
                    // Two offsets, because a `{start}` without a
                    // `{count}` moved `str` forward.
                    rettv.vval.v_number = (unsafe { edge.offset_from(str) }
                        + unsafe { str.offset_from(expr) })
                        as varnumber_T;
                }
            }
        }
    }

    // `matchstrpos()` on a String has no index to report, so the
    // placeholder seeded above comes back out.
    if kind == kSomeMatchStrPos && l.is_null() && !unsafe { rettv.vval.v_list }.is_null() {
        let ret_l = unsafe { rettv.vval.v_list };
        unsafe { tv_list_item_remove(ret_l, (*tv_list_first(ret_l)).li_next) };
    }
}

/// Append one dict per match of `rmp` in `str` to `mlist`.
///
/// # Safety
/// `str` is NUL-terminated, `rmp` holds a compiled program, `mlist` is a
/// live list.
unsafe fn get_matches_in_str(
    str: *const c_char,
    rmp: *mut regmatch_T,
    mlist: *mut list_T,
    idx: c_int,
    submatches: bool,
    matchbuf: bool,
) {
    // SAFETY: the caller's obligation; every pointer written below comes
    // back from the matcher and points into `str`.
    let len = unsafe { strlen(str) };
    let mut startidx: colnr_T = 0;
    loop {
        if !unsafe { vim_regexec_nl(rmp, str, startidx) } {
            return;
        }
        let d: *mut dict_T = unsafe { tv_dict_alloc() };
        unsafe { tv_list_append_dict(mlist, d) };
        // A buffer's matches are keyed by line number, a List's by the
        // index of the item they came from.
        if matchbuf {
            unsafe { tv_dict_add_nr(d, c"lnum".as_ptr(), 4, idx as varnumber_T) };
        } else {
            unsafe { tv_dict_add_nr(d, c"idx".as_ptr(), 3, idx as varnumber_T) };
        }
        let (start, end) = unsafe { ((*rmp).startp[0], (*rmp).endp[0]) };
        let byteidx = unsafe { start.offset_from(str) } as colnr_T as varnumber_T;
        unsafe { tv_dict_add_nr(d, c"byteidx".as_ptr(), 7, byteidx) };
        let matchlen = unsafe { end.offset_from(start) } as c_int;
        unsafe { tv_dict_add_str_len(d, c"text".as_ptr(), 4, start, matchlen) };
        if submatches {
            let sml = unsafe { tv_list_alloc(NSUBEXP as isize - 1) };
            unsafe { tv_dict_add_list(d, c"submatches".as_ptr(), 10, sml) };
            for i in 1..NSUBEXP as usize {
                if unsafe { (*rmp).endp[i] }.is_null() {
                    unsafe { tv_list_append_string(sml, c"".as_ptr(), 0) };
                } else {
                    let (start, end) = unsafe { ((*rmp).startp[i], (*rmp).endp[i]) };
                    let len = unsafe { end.offset_from(start) };
                    unsafe { tv_list_append_string(sml, start, len) };
                }
            }
        }
        // Resume past this match; stop at the end of the string, and
        // stop on a match that did not advance.
        startidx = unsafe { (*rmp).endp[0].offset_from(str) } as colnr_T;
        if startidx >= len as colnr_T
            || unsafe { str.offset(startidx as isize) }
                <= unsafe { (*rmp).startp[0] } as *const c_char
        {
            return;
        }
    }
}

/// `matchbufline({buf}, {pat}, {lnum}, {end} [, {dict}])`.
pub unsafe fn f_matchbufline(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: the buffer comes from the buffer list and is checked for a
    // memfile before any line is read.
    rettv.vval.v_number = -1;
    list_alloc_ret(rettv, kListLenUnknown as isize);
    let retlist = unsafe { rettv.vval.v_list };
    if check_arg(args, 0, tv_check_for_buffer_arg) == FAIL
        || check_arg(args, 1, tv_check_for_string_arg) == FAIL
        || check_arg(args, 2, tv_check_for_lnum_arg) == FAIL
        || check_arg(args, 3, tv_check_for_lnum_arg) == FAIL
        || check_arg(args, 4, tv_check_for_opt_dict_arg) == FAIL
    {
        return;
    }
    let prev_did_emsg = did_emsg.get();
    let buf: *mut buf_T = unsafe { tv_get_buf(args.ptr(0), 0) };
    if buf.is_null() {
        // Only report the name when `tv_get_buf` was silent about it.
        if did_emsg.get() == prev_did_emsg {
            let what = arg_string(&mut numbuf, args.get(0));
            unsafe { semsg_c!(gettext(e_invalid_buffer_name_str), what) };
        }
        return;
    }
    if unsafe { (*buf).b_ml.ml_mfp }.is_null() {
        emsg(gettext(e_buffer_is_not_loaded));
        return;
    }
    let mut patbuf = NumBuf::new();
    let pat = arg_string(&mut patbuf, args.get(1));

    let did_emsg_before = did_emsg.get();
    let mut slnum: linenr_T = unsafe { tv_get_lnum_buf(args.ptr(2), buf) };
    if did_emsg.get() > did_emsg_before {
        return;
    }
    if slnum < 1 {
        unsafe { semsg_c!(gettext(e_invargval), c"lnum".as_ptr(),) };
        return;
    }
    let mut elnum: linenr_T = unsafe { tv_get_lnum_buf(args.ptr(3), buf) };
    if did_emsg.get() > did_emsg_before {
        return;
    }
    if elnum < 1 || elnum < slnum {
        unsafe { semsg_c!(gettext(e_invargval), c"end_lnum".as_ptr(),) };
        return;
    }
    elnum = elnum.min(unsafe { (*buf).b_ml.ml_line_count });

    let Some(submatches) = (unsafe { want_submatches(args, 4) }) else {
        return;
    };

    let _cpo = EmptyCpo::new();
    let Some(mut prog) = (unsafe { Regprog::compile(pat) }) else {
        return;
    };
    while slnum <= elnum {
        let str = unsafe { ml_get_buf(buf, slnum) };
        unsafe { get_matches_in_str(str, &raw mut prog.0, retlist, slnum, submatches, true) };
        slnum += 1;
    }
}

/// The `{dict}` argument the two list-shaped matchers share: `submatches`
/// must be a Boolean if it is there at all. `None` means the argument was
/// rejected and the caller must stop.
///
/// # Safety
/// `args` is the call frame.
unsafe fn want_submatches(args: Args<'_>, i: usize) -> Option<bool> {
    if !args.has(i) {
        return Some(false);
    }
    // SAFETY: the caller's obligation; the type tag was checked by
    // `tv_check_for_opt_dict_arg` before this runs.
    let d = unsafe { args.get(i).vval.v_dict };
    if d.is_null() {
        return Some(false);
    }
    let di = unsafe { tv_dict_find(d, c"submatches".as_ptr(), 10) };
    if di.is_null() {
        return Some(false);
    }
    if unsafe { (*di).di_tv.v_type } != VAR_BOOL {
        unsafe { semsg_c!(gettext(e_invargval), c"submatches".as_ptr(),) };
        return None;
    }
    Some(unsafe { tv_get_bool(&raw mut (*di).di_tv) } != 0)
}

/// `match({expr}, {pat} [, {start} [, {count}]])`.
pub unsafe fn f_match(argvars: *mut typval_T, rettv: *mut typval_T, _f: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame's.
    unsafe { find_some_match(args, rettv, kSomeMatch) }
}

/// `matchend({expr}, {pat} [, {start} [, {count}]])`.
pub unsafe fn f_matchend(argvars: *mut typval_T, rettv: *mut typval_T, _f: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame's.
    unsafe { find_some_match(args, rettv, kSomeMatchEnd) }
}

/// `matchlist({expr}, {pat} [, {start} [, {count}]])`.
pub unsafe fn f_matchlist(argvars: *mut typval_T, rettv: *mut typval_T, _f: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame's.
    unsafe { find_some_match(args, rettv, kSomeMatchList) }
}

/// `matchstr({expr}, {pat} [, {start} [, {count}]])`.
pub unsafe fn f_matchstr(argvars: *mut typval_T, rettv: *mut typval_T, _f: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame's.
    unsafe { find_some_match(args, rettv, kSomeMatchStr) }
}

/// `matchstrpos({expr}, {pat} [, {start} [, {count}]])`.
pub unsafe fn f_matchstrpos(argvars: *mut typval_T, rettv: *mut typval_T, _f: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame's.
    unsafe { find_some_match(args, rettv, kSomeMatchStrPos) }
}

/// `matchstrlist({list}, {pat} [, {dict}])`.
pub unsafe fn f_matchstrlist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: the List and its items outlive the call.
    rettv.vval.v_number = -1;
    list_alloc_ret(rettv, kListLenUnknown as isize);
    let retlist = unsafe { rettv.vval.v_list };
    if check_arg(args, 0, tv_check_for_list_arg) == FAIL
        || check_arg(args, 1, tv_check_for_string_arg) == FAIL
        || check_arg(args, 2, tv_check_for_opt_dict_arg) == FAIL
    {
        return;
    }
    let l = unsafe { args.get(0).vval.v_list };
    if l.is_null() {
        return;
    }
    let mut patbuf = NumBuf::new();
    let pat = arg_string_chk(&mut patbuf, args.get(1));
    if pat.is_null() {
        return;
    }
    let _cpo = EmptyCpo::new();
    let Some(mut prog) = (unsafe { Regprog::compile(pat) }) else {
        return;
    };
    // The `{dict}` is only read once the pattern compiled, as upstream
    // has it: a bad pattern is reported before a bad option.
    let Some(submatches) = (unsafe { want_submatches(args, 2) }) else {
        return;
    };
    let mut li = unsafe { tv_list_first(l) };
    let mut idx: c_int = 0;
    while !li.is_null() {
        let li_tv = unsafe { &(*li).li_tv };
        // A non-String item, and the null String, contribute nothing.
        if li_tv.v_type == VAR_STRING && !unsafe { li_tv.vval.v_string }.is_null() {
            let str = unsafe { li_tv.vval.v_string };
            let rmp = &raw mut prog.0;
            unsafe { get_matches_in_str(str, rmp, retlist, idx, submatches, false) };
        }
        idx += 1;
        li = unsafe { (*li).li_next };
    }
}
