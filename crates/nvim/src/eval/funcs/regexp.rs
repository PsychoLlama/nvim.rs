//! Matching a pattern against a string: the `match*()` family.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::wrappers::{arg_number_chk, arg_string, arg_string_chk, check_arg, list_alloc_ret};
use super::{
    NSUBEXP, SomeMatchType, kSomeMatch, kSomeMatchEnd, kSomeMatchList, kSomeMatchStr,
    kSomeMatchStrPos, tv_get_buf,
};
use crate::cstr;
use crate::eval::callback_call;
use crate::eval::encode::encode_tv2echo;
use crate::eval::typval::{
    NumBuf, callback_free, tv_check_for_buffer_arg, tv_check_for_list_arg, tv_check_for_lnum_arg,
    tv_check_for_nonnull_dict_arg, tv_check_for_opt_dict_arg, tv_check_for_string_arg, tv_clear,
    tv_copy, tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str_len, tv_dict_alloc, tv_dict_find,
    tv_dict_get_callback, tv_dict_has_key, tv_dict_unref, tv_get_bool, tv_get_lnum_buf,
    tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict, tv_list_append_list,
    tv_list_append_number, tv_list_append_string, tv_list_append_tv, tv_list_find, tv_list_first,
    tv_list_item_remove, tv_list_uidx,
};
use crate::fuzzy::{FUZZY_MATCH_MAX_LEN, fuzzy_match, matched_char_count};
use crate::main::{did_emsg, e_buffer_is_not_loaded, p_cpo, p_ic};
use crate::mbyte::utfc_ptr2len;
use crate::memline::ml_get_buf;
use crate::memory::{xfree, xmemdupz};
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::optionstr::empty_option;
use crate::os::cshim::gettext;
use crate::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec_nl, vim_regfree};
use crate::semsg;
use crate::types::{
    Callback, EvalFuncData, VAR_BOOL, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
    VarLock, buf_T, colnr_T, dict_T, kListLenMayKnow, kListLenUnknown, linenr_T, list_T,
    listitem_T, regmatch_T, regprog_T, typval_T, typval_vval_union, varnumber_T,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// An unset typval, as `VAR_UNKNOWN` spells it.
const TV_UNKNOWN: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

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
            unsafe { tv_list_append_string(rettv.list_or_null(), c"".as_ptr(), 0) };
            unsafe { tv_list_append_number(rettv.list_or_null(), -1) };
            unsafe { tv_list_append_number(rettv.list_or_null(), -1) };
            unsafe { tv_list_append_number(rettv.list_or_null(), -1) };
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
            l = args.get(0).list_or_null();
            if l.is_null() {
                break 'theend;
            }
            li = unsafe { tv_list_first(l) };
        } else {
            str = arg_string(&mut numbuf, args.get(0)) as *mut c_char;
            expr = str;
            len = unsafe { cstr::bytes_at(str) }.len() as i64;
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
                let ret_l = rettv.list_or_null();
                let li1 = unsafe { tv_list_first(ret_l) };
                let li2 = unsafe { (*li1).li_next };
                let li3 = unsafe { (*li2).li_next };
                let li4 = unsafe { (*li3).li_next };
                unsafe { xfree((*li1).li_tv.string_or_null() as *mut c_void) };
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
                        unsafe { tv_list_append_string(rettv.list_or_null(), ptr::null(), 0) };
                    } else {
                        let (start, end) = (regmatch.startp[i], regmatch.endp[i]);
                        let list = rettv.list_or_null();
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
    if kind == kSomeMatchStrPos && l.is_null() && !rettv.list_or_null().is_null() {
        let ret_l = rettv.list_or_null();
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
    let len = unsafe { cstr::bytes_at(str) }.len();
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
            let _ = unsafe { tv_dict_add_nr(d, c"lnum".as_ptr(), 4, idx as varnumber_T) };
        } else {
            let _ = unsafe { tv_dict_add_nr(d, c"idx".as_ptr(), 3, idx as varnumber_T) };
        }
        let (start, end) = unsafe { ((*rmp).startp[0], (*rmp).endp[0]) };
        let byteidx = unsafe { start.offset_from(str) } as colnr_T as varnumber_T;
        let _ = unsafe { tv_dict_add_nr(d, c"byteidx".as_ptr(), 7, byteidx) };
        let matchlen = unsafe { end.offset_from(start) } as c_int;
        let _ = unsafe { tv_dict_add_str_len(d, c"text".as_ptr(), 4, start, matchlen) };
        if submatches {
            let sml = unsafe { tv_list_alloc(NSUBEXP as isize - 1) };
            let _ = unsafe { tv_dict_add_list(d, c"submatches".as_ptr(), 10, sml) };
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
    let retlist = rettv.list_or_null();
    if check_arg(args, 0, tv_check_for_buffer_arg).is_err()
        || check_arg(args, 1, tv_check_for_string_arg).is_err()
        || check_arg(args, 2, tv_check_for_lnum_arg).is_err()
        || check_arg(args, 3, tv_check_for_lnum_arg).is_err()
        || check_arg(args, 4, tv_check_for_opt_dict_arg).is_err()
    {
        return;
    }
    let prev_did_emsg = did_emsg.get();
    let buf: *mut buf_T = unsafe { tv_get_buf(args.ptr(0), 0) };
    if buf.is_null() {
        // Only report the name when `tv_get_buf` was silent about it.
        if did_emsg.get() == prev_did_emsg {
            let what = arg_string(&mut numbuf, args.get(0));
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let what = unsafe { c_str(what) };
            semsg!("E158: Invalid buffer name: {what}");
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
        let arg0 = "lnum";
        semsg!("E475: Invalid value for argument {arg0}");
        return;
    }
    let mut elnum: linenr_T = unsafe { tv_get_lnum_buf(args.ptr(3), buf) };
    if did_emsg.get() > did_emsg_before {
        return;
    }
    if elnum < 1 || elnum < slnum {
        let arg0 = "end_lnum";
        semsg!("E475: Invalid value for argument {arg0}");
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
    let d = args.get(i).dict_or_null();
    if d.is_null() {
        return Some(false);
    }
    let di = unsafe { tv_dict_find(d, c"submatches".as_ptr(), 10) };
    if di.is_null() {
        return Some(false);
    }
    if unsafe { (*di).di_tv.v_type } != VAR_BOOL {
        let arg0 = "submatches";
        semsg!("E475: Invalid value for argument {arg0}");
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
    let retlist = rettv.list_or_null();
    if check_arg(args, 0, tv_check_for_list_arg).is_err()
        || check_arg(args, 1, tv_check_for_string_arg).is_err()
        || check_arg(args, 2, tv_check_for_opt_dict_arg).is_err()
    {
        return;
    }
    let l = args.get(0).list_or_null();
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
        if li_tv.v_type == VAR_STRING && !li_tv.string_or_null().is_null() {
            let str = li_tv.string_or_null();
            let rmp = &raw mut prog.0;
            unsafe { get_matches_in_str(str, rmp, retlist, idx, submatches, false) };
        }
        idx += 1;
        li = unsafe { (*li).li_next };
    }
}

// ---------------------------------------------------------------------------
// `matchfuzzy()` and `matchfuzzypos()`.
//
// The scorer they drive is `crate::fuzzy`, which is now nothing but the
// scorer; these two are `match*()` functions like the rest of this file, and
// they read a List, a Dict argument and a Callback the way its neighbours do.
/// Where the string to match comes from: the list items are strings, and a
/// dict item then contributes nothing — or they are dicts, to look a key up
/// in or to hand to a callback.
enum Source {
    Item,
    Key(*const c_char),
    Callback(*mut Callback),
}

/// What one `matchfuzzy()`/`matchfuzzypos()` call was asked for: the pattern,
/// where each item's string comes from, whether the words of a multi-word
/// pattern have to match in sequence, whether the matching positions are
/// wanted too (that is `matchfuzzypos()`), and how many matches are enough.
struct Request {
    pattern: *const c_char,
    source: Source,
    matchseq: bool,
    retmatchpos: bool,
    limit: c_int,
}

/// One list item that matched.
struct FuzzyItem {
    /// Where it sat in the input list, which is how ties are broken.
    idx: usize,
    /// The item itself, copied to the result list as it is.
    item: *mut listitem_T,
    score: c_int,
    /// Whether the pattern occurs literally at the first matched position.
    exact: bool,
    /// The matching positions, for `matchfuzzypos()`.
    positions: Option<*mut list_T>,
}

/// The item's string, as `Request::source` says to find it. A callback's
/// answer lands in `rettv`, which the caller clears; the string is only
/// borrowed until then.
unsafe fn item_string(
    request: &Request,
    tv: *const typval_T,
    rettv: *mut typval_T,
    numbuf: &mut NumBuf,
) -> *const c_char {
    if unsafe { (*tv).v_type } == VAR_STRING {
        return unsafe { (*tv).vval.v_string };
    }
    if unsafe { (*tv).v_type } != VAR_DICT {
        return ptr::null();
    }
    match request.source {
        Source::Item => ptr::null(),
        Source::Key(key) => unsafe { numbuf.dict_string((*tv).vval.v_dict, key) },
        Source::Callback(cb) => {
            // The callback is handed the dict, which it must not be able
            // to free out from under this loop.
            unsafe { (*(*tv).vval.v_dict).dv_refcount.retain() };
            let mut argv = [
                typval_T {
                    v_type: VAR_DICT,
                    v_lock: VarLock::Unlocked,
                    vval: typval_vval_union {
                        v_dict: unsafe { (*tv).vval.v_dict },
                    },
                },
                TV_UNKNOWN,
            ];
            let called = unsafe { callback_call(cb, 1, argv.as_mut_ptr(), rettv) };
            unsafe { tv_dict_unref((*tv).vval.v_dict) };
            if called && unsafe { (*rettv).v_type } == VAR_STRING {
                unsafe { (*rettv).vval.v_string }
            } else {
                ptr::null()
            }
        }
    }
}

/// The list held by item `idx` of `list`, which the caller has just built.
unsafe fn nested_list(list: *mut list_T, idx: c_int) -> *mut list_T {
    let li = unsafe { tv_list_find(list, idx) };
    debug_assert!(!li.is_null(), "fuzzy: result list is short");
    let nested = unsafe { (*li).li_tv.vval.v_list };
    debug_assert!(!nested.is_null(), "fuzzy: result item is not a list");
    nested
}

/// Fuzzy match `request`'s pattern against the strings of `list`, appending
/// the matches to `fmatchlist` in descending score order. For `matchfuzzy()`
/// that is a list of strings; for `matchfuzzypos()` `fmatchlist` already
/// holds three lists — the matched strings, the matching positions of each,
/// and the scores — which are filled in turn.
unsafe fn fuzzy_match_in_list(list: *mut list_T, request: &Request, fmatchlist: *mut list_T) {
    let mut numbuf = NumBuf::new();
    let pattern = unsafe { CStr::from_ptr(request.pattern) };
    let mut found: Vec<FuzzyItem> = Vec::new();
    let mut matches = [0u32; FUZZY_MATCH_MAX_LEN];
    let mut li = unsafe { (*list).lv_first };
    while !li.is_null() {
        if request.limit > 0 && found.len() >= request.limit as usize {
            break;
        }
        let mut rettv = TV_UNKNOWN;
        let itemstr =
            unsafe { item_string(request, &raw const (*li).li_tv, &raw mut rettv, &mut numbuf) };
        if !itemstr.is_null() {
            let itemstr = unsafe { CStr::from_ptr(itemstr) };
            let (score, filled) = fuzzy_match(itemstr, pattern, request.matchseq, &mut matches);
            if filled != 0 {
                // Upstream reads the string at the first *character*
                // position as if it were a byte offset. Preserved: it is
                // only a tie-break between two equally scored items.
                let at = matches[0] as usize;
                let exact = itemstr
                    .to_bytes()
                    .get(at..)
                    .is_some_and(|tail| tail.starts_with(pattern.to_bytes()));
                let positions = request.retmatchpos.then(|| {
                    let positions = unsafe { tv_list_alloc(kListLenMayKnow as isize) };
                    // One position per pattern character that took part
                    // in the match, i.e. all but the word separators.
                    let placed = matched_char_count(pattern, request.matchseq);
                    for at in matches.iter().take(placed) {
                        unsafe { tv_list_append_number(positions, *at as varnumber_T) };
                    }
                    positions
                });
                found.push(FuzzyItem {
                    idx: found.len(),
                    item: li,
                    score,
                    exact,
                    positions,
                });
            }
        }
        unsafe { tv_clear(&raw mut rettv) };
        li = unsafe { (*li).li_next };
    }
    if found.is_empty() {
        return;
    }

    // Best score first; an exact match wins a tie, and the input order
    // settles the rest. No two items share an `idx`, so this is a total
    // order and the sort needs no stability of its own.
    found.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(b.exact.cmp(&a.exact))
            .then(a.idx.cmp(&b.idx))
    });

    // matchfuzzy() answers just the strings; matchfuzzypos() answers
    // them in the first of its three lists.
    let strings = if request.retmatchpos {
        unsafe { nested_list(fmatchlist, 0) }
    } else {
        fmatchlist
    };
    for item in &found {
        unsafe { tv_list_append_tv(strings, &raw mut (*item.item).li_tv) };
    }
    if request.retmatchpos {
        let positions = unsafe { nested_list(fmatchlist, -2) };
        for item in &mut found {
            let list = item.positions.take().expect("fuzzy: positions were kept");
            unsafe { tv_list_append_list(positions, list) };
        }
        let scores = unsafe { nested_list(fmatchlist, -1) };
        for item in &found {
            unsafe { tv_list_append_number(scores, item.score as varnumber_T) };
        }
    }
}

/// The body of `matchfuzzy()` and, with `retmatchpos`, `matchfuzzypos()`.
unsafe fn do_fuzzymatch(argvars: *const typval_T, rettv: *mut typval_T, retmatchpos: bool) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let list = unsafe { &*argvars };
    if list.v_type != VAR_LIST || unsafe { list.vval.v_list }.is_null() {
        let who = if retmatchpos {
            c"matchfuzzypos()".as_ptr()
        } else {
            c"matchfuzzy()".as_ptr()
        };
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let who = unsafe { c_str(who) };
        semsg!("E686: Argument of {who} must be a List");
        return;
    }
    let pat = unsafe { &*argvars.add(1) };
    if pat.v_type != VAR_STRING || unsafe { pat.vval.v_string }.is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg0 = unsafe { c_str(numbuf.string(pat)) };
        semsg!("E475: Invalid argument: {arg0}");
        return;
    }

    // The optional third argument says where to find the string of a
    // dict item, and how much of the list to bother with.
    let mut cb = Callback::None;
    let mut key = ptr::null();
    let mut matchseq = false;
    let mut limit = 0;
    if unsafe { (*argvars.add(2)).v_type } != VAR_UNKNOWN {
        if unsafe { tv_check_for_nonnull_dict_arg(argvars, 2) }.is_err() {
            return;
        }
        let d: *mut dict_T = unsafe { (*argvars.add(2)).vval.v_dict };
        let di = unsafe { tv_dict_find(d, c"key".as_ptr(), -1) };
        if !di.is_null() {
            if unsafe { (*di).di_tv.v_type } != VAR_STRING
                || unsafe { (*di).di_tv.vval.v_string }.is_null()
                || unsafe { *(*di).di_tv.vval.v_string } == 0
            {
                let got = unsafe { numbuf2.string(&raw const (*di).di_tv) };
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let got = unsafe { c_str(got) };
                semsg!("E475: Invalid value for argument {}: {got}", "key");
                return;
            }
            key = unsafe { numbuf3.string(&raw const (*di).di_tv) };
        } else if !unsafe { tv_dict_get_callback(d, c"text_cb".as_ptr(), -1, &raw mut cb) } {
            semsg!("E475: Invalid value for argument {}", "text_cb");
            return;
        }
        let di = unsafe { tv_dict_find(d, c"limit".as_ptr(), -1) };
        if !di.is_null() {
            if unsafe { (*di).di_tv.v_type } != VAR_NUMBER {
                semsg!("E475: Invalid value for argument {}", "limit");
                return;
            }
            limit = unsafe { tv_get_number_chk(&raw const (*di).di_tv, ptr::null_mut()) } as c_int;
        }
        matchseq = unsafe { tv_dict_has_key(d, c"matchseq".as_ptr()) };
    }

    // matchfuzzypos() answers three lists: the matching strings, their
    // matching positions, and their scores.
    let len = if retmatchpos {
        3
    } else {
        kListLenUnknown as isize
    };
    let result = unsafe { tv_list_alloc_ret(rettv, len) };
    if retmatchpos {
        for _ in 0..3 {
            unsafe { tv_list_append_list(result, tv_list_alloc(kListLenUnknown as isize)) };
        }
    }
    let request = Request {
        pattern: unsafe { numbuf4.string(pat) },
        source: if !key.is_null() {
            Source::Key(key)
        } else if cb.is_set() {
            Source::Callback(&raw mut cb)
        } else {
            Source::Item
        },
        matchseq,
        retmatchpos,
        limit,
    };
    unsafe { fuzzy_match_in_list(list.vval.v_list, &request, result) };
    unsafe { callback_free(&raw mut cb) };
}

/// `matchfuzzy()`: the items of a list that fuzzy match a pattern.
///
/// # Safety
/// Called with a Vimscript function's arguments and result slot.
pub(crate) unsafe fn f_matchfuzzy(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe { do_fuzzymatch(argvars, rettv, false) }
}

/// `matchfuzzypos()`: as [`f_matchfuzzy`], plus where each match landed and
/// what it scored.
///
/// # Safety
/// Called with a Vimscript function's arguments and result slot.
pub(crate) unsafe fn f_matchfuzzypos(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe { do_fuzzymatch(argvars, rettv, true) }
}
