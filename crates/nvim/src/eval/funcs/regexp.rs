//! Matching a pattern against a string: the `match*()` family.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::wrappers::{arg_number_chk, arg_string, arg_string_chk, list_alloc_ret};
use super::{
    NSUBEXP, SomeMatchType, kSomeMatch, kSomeMatchEnd, kSomeMatchList, kSomeMatchStr,
    kSomeMatchStrPos, tv_get_buf,
};
use crate::cstr;
use crate::eval::callback_call;
use crate::eval::encode::encode_tv2echo;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{
    DictRef, ListRef, NumBuf, callback_free, dict_find, dict_get_callback, index_of, list_find,
    list_items, list_items_mut, list_uidx, tv_check_for_buffer_arg, tv_check_for_list_arg,
    tv_check_for_lnum_arg, tv_check_for_nonnull_dict_arg, tv_check_for_opt_dict_arg,
    tv_check_for_string_arg, tv_clear, tv_copy, tv_dict_alloc, tv_get_bool, tv_get_lnum_buf,
    tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret,
};
use crate::fuzzy::{FUZZY_MATCH_MAX_LEN, fuzzy_match, matched_char_count};
use crate::mbyte::utfc_ptr2len;
use crate::memline::ml_get_buf;
use crate::memory::{xfree, xmemdupz};
use crate::message::e_buffer_is_not_loaded;
use crate::message::emsg;
use crate::message::state::did_emsg;
use crate::message_fmt::c_str;
use crate::option::SavedCpo;
use crate::option::vars::p_ic;
use crate::os::cshim::gettext;
use crate::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec_nl, vim_regfree};
use crate::semsg;
use crate::types::{
    Callback, ColNr, EvalFuncData, LineNr, List, RegMatch, RegProg, TypVal, VAR_BOOL, VAR_DICT,
    VAR_LIST, VAR_NUMBER, VAR_STRING, VarNumber, kListLenMayKnow, kListLenUnknown,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// An unset typval, as `VAR_UNKNOWN` spells it.
const TV_UNKNOWN: TypVal = TV_INITIAL_VALUE;

/// A cleared `RegMatch`, which `vim_regcomp`'s result is dropped into.
const EMPTY_REGMATCH: RegMatch = RegMatch::new(ptr::null_mut::<RegProg>(), false);

/// A compiled pattern, freed on drop.
struct Regprog(RegMatch);

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
        rm.rm_ic = p_ic();
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
fn find_some_match(args: &[TypVal], result: &mut TypVal, kind: SomeMatchType) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the caller's obligation. Every pointer below either points
    // into an argument (which outlives the call), into `patbuf`, or into
    // the string `tofree` owns.
    let _cpo = SavedCpo::empty();
    result.write_number(-1);
    match kind {
        kSomeMatchList => {
            list_alloc_ret(result, kListLenMayKnow as isize);
        }
        kSomeMatchStrPos => {
            // Seeded with the "no match" answer, which the tail of this
            // function trims back to three items for a String subject.
            list_alloc_ret(result, 4);
            unsafe { (*result.list_or_null()).push_string(c"".as_ptr(), 0) };
            unsafe { (*result.list_or_null()).push_number(-1) };
            unsafe { (*result.list_or_null()).push_number(-1) };
            unsafe { (*result.list_or_null()).push_number(-1) };
        }
        kSomeMatchStr => {
            result.write_string(ptr::null_mut());
        }
        _ => {}
    }

    let mut l: *mut List = ptr::null_mut();
    // Where the List walk is; `encode_tv2echo` below runs the evaluator, so
    // this cannot be an address into the item store.
    let mut at: usize = 0;
    let mut str: *mut c_char = ptr::null_mut();
    let mut expr: *mut c_char = ptr::null_mut();
    let mut len: i64 = 0;
    let mut start: i64;
    let mut nth: i64 = 1;
    let mut startcol: ColNr = 0;
    let mut idx: c_int = 0;
    let mut matched = false;
    // Owns whatever the List walk echoed most recently.
    let mut tofree;

    // Nothing below this point may return early without running the
    // trailing fixup, so the body is one labelled block as the C's
    // `goto theend` was.
    'theend: {
        if args.first().is_some_and(|arg| arg.v_type() == VAR_LIST) {
            l = args[0].list_or_null();
            if l.is_null() {
                break 'theend;
            }
        } else {
            str = arg_string(&mut numbuf, &args[0]) as *mut c_char;
            expr = str;
            len = unsafe { cstr::bytes_at(str) }.len() as i64;
        }

        let mut patbuf = NumBuf::new();
        let pat = arg_string_chk(&mut patbuf, &args[1]);
        if pat.is_null() {
            break 'theend;
        }

        if args.len() > 2 {
            let mut error = false;
            start = arg_number_chk(&args[2], Some(&mut error)) as i64;
            if error {
                break 'theend;
            }
            if !l.is_null() {
                idx = list_uidx(unsafe { l.as_ref() }, start as c_int);
                let Ok(start_at) = usize::try_from(idx) else {
                    break 'theend;
                };
                at = start_at;
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
                if args.len() > 3 {
                    startcol = start as ColNr;
                } else {
                    str = unsafe { str.offset(start as isize) };
                    len -= start;
                }
            }
            if args.len() > 3 {
                nth = arg_number_chk(&args[3], Some(&mut error)) as i64;
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
                // SAFETY: a live list, re-read each step.
                let Some(item) = (list_items(unsafe { l.as_ref() })).get(at) else {
                    matched = false;
                    break;
                };
                tofree = Echoed(unsafe { encode_tv2echo(&item.li_tv, ptr::null_mut()) });
                str = tofree.0;
                expr = str;
                if str.is_null() {
                    break;
                }
            }
            // SAFETY: `str` is the subject, NUL-terminated.
            matched = vim_regexec_nl(regmatch, unsafe { cstr::at(str) }, startcol as usize);
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
                at += 1;
                idx += 1;
                continue;
            }
            // Same string, next match: step past the character the
            // match started on. A match that did not advance, or one
            // past the end, ends the search.
            let start = regmatch.starts[0].unwrap_or(0);
            // SAFETY: an offset into the subject.
            let hit = unsafe { str.add(start) };
            startcol = (start + unsafe { utfc_ptr2len(hit) } as usize) as ColNr;
            if startcol > len as ColNr || startcol as usize <= start {
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
                let ret_l = result.list_or_null();
                // SAFETY: the four items seeded above.
                let seeded = list_items_mut(unsafe { ret_l.as_mut() });
                unsafe { xfree(seeded[0].li_tv.string_or_null() as *mut c_void) };
                let span = regmatch.group(0).unwrap_or(0..0);
                // SAFETY: the span is an offset range into `str`.
                let text = unsafe { xmemdupz(str.add(span.start).cast(), span.len()) };
                seeded[0].li_tv.write_string(text as *mut c_char);
                // `str` may have moved on from `expr`, and the answer is
                // counted from where the subject began.
                let skipped = unsafe { str.offset_from(expr) } as usize;
                seeded[2]
                    .li_tv
                    .write_number((skipped + span.start) as VarNumber);
                seeded[3]
                    .li_tv
                    .write_number((skipped + span.end) as VarNumber);
                if !l.is_null() {
                    seeded[1].li_tv.write_number(VarNumber::from(idx));
                }
            }
            kSomeMatchList => {
                for i in 0..NSUBEXP as usize {
                    let list = result.list_or_null();
                    match regmatch.group(i) {
                        None => unsafe { (*list).push_string(ptr::null(), 0) },
                        // SAFETY: the span is an offset range into `str`.
                        Some(span) => unsafe {
                            (*list).push_string(str.add(span.start), span.len() as isize)
                        },
                    }
                }
            }
            kSomeMatchStr => {
                if !l.is_null() {
                    // A List subject answers with the whole item, not
                    // with the part that matched.
                    // SAFETY: a live list and the index the walk matched at.
                    tv_copy(&list_items(unsafe { l.as_ref() })[at].li_tv, result);
                } else {
                    let span = regmatch.group(0).unwrap_or(0..0);
                    // SAFETY: the span is an offset range into `str`.
                    let text =
                        unsafe { xmemdupz(str.add(span.start) as *const c_void, span.len()) };
                    result.write_string(text as *mut c_char);
                }
            }
            _ => {
                if !l.is_null() {
                    result.write_number(idx as VarNumber);
                } else {
                    let edge = if kind == kSomeMatch {
                        regmatch.starts[0]
                    } else {
                        regmatch.ends[0]
                    };
                    // Two offsets, because a `{start}` without a
                    // `{count}` moved `str` forward.
                    let skipped = unsafe { str.offset_from(expr) } as usize;
                    result.write_number((skipped + edge.unwrap_or(0)) as VarNumber);
                }
            }
        }
    }

    // `matchstrpos()` on a String has no index to report, so the
    // placeholder seeded above comes back out.
    if kind == kSomeMatchStrPos && l.is_null() && !result.list_or_null().is_null() {
        let ret_l = result.list_or_null();
        // SAFETY: the placeholder is the second of the four items seeded
        // above.
        unsafe { (*ret_l).remove_at(1) };
    }
}

/// Append one dict per match of `rmp` in `str` to `mlist`.
///
/// # Safety
/// `rmp` holds a compiled program and `mlist` is a live list.
unsafe fn get_matches_in_str(
    str: &CStr,
    rmp: &mut RegMatch,
    mlist: *mut List,
    idx: c_int,
    submatches: bool,
    matchbuf: bool,
) {
    let len = str.count_bytes();
    let mut startidx = 0;
    loop {
        if !vim_regexec_nl(rmp, str, startidx) {
            return;
        }
        let d_held = tv_dict_alloc();
        let d = d_held.as_ptr();
        unsafe { (*mlist).push_dict(Some(d_held)) };
        // A buffer's matches are keyed by line number, a List's by the
        // index of the item they came from.
        if matchbuf {
            let _ = unsafe { (*d).add_number(b"lnum", idx as VarNumber) };
        } else {
            let _ = unsafe { (*d).add_number(b"idx", idx as VarNumber) };
        }
        let span = rmp.group(0).unwrap_or(0..0);
        let _ = unsafe { (*d).add_number(b"byteidx", span.start as VarNumber) };
        // SAFETY: the span is an offset range into `str`.
        let start = unsafe { str.as_ptr().add(span.start) };
        let _ = unsafe { (*d).add_str_len(b"text", start, span.len() as c_int) };
        if submatches {
            let submatch_list = tv_list_alloc(NSUBEXP as isize - 1);
            // A borrow of the list the dictionary owns from here on.
            let sml = submatch_list.as_ptr();
            let _ = unsafe { (*d).add_list(b"submatches", Some(submatch_list)) };
            for i in 1..NSUBEXP as usize {
                match rmp.group(i) {
                    None => unsafe { (*sml).push_string(c"".as_ptr(), 0) },
                    // SAFETY: the span is an offset range into `str`.
                    Some(span) => unsafe {
                        (*sml).push_string(str.as_ptr().add(span.start), span.len() as isize)
                    },
                }
            }
        }
        // Resume past this match; stop at the end of the string, and
        // stop on a match that did not advance.
        startidx = span.end;
        if startidx >= len || startidx <= span.start {
            return;
        }
    }
}

/// `matchbufline({buf}, {pat}, {lnum}, {end} [, {dict}])`.
pub fn f_matchbufline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the buffer comes from the buffer list and is checked for a
    // memfile before any line is read.
    result.write_number(-1);
    list_alloc_ret(result, kListLenUnknown as isize);
    let retlist = result.list_or_null();
    if tv_check_for_buffer_arg(args, 0).is_err()
        || tv_check_for_string_arg(args, 1).is_err()
        || tv_check_for_lnum_arg(args, 2).is_err()
        || tv_check_for_lnum_arg(args, 3).is_err()
        || tv_check_for_opt_dict_arg(args, 4).is_err()
    {
        return;
    }
    let prev_did_emsg = did_emsg.get();
    let buf = tv_get_buf(&args[0], 0);
    let Some(buf) = buf else {
        // Only report the name when `tv_get_buf` was silent about it.
        if did_emsg.get() == prev_did_emsg {
            let what = arg_string(&mut numbuf, &args[0]);
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let what = unsafe { c_str(what) };
            semsg!("E158: Invalid buffer name: {what}");
        }
        return;
    };
    if buf.b_ml.ml_mfp.is_null() {
        emsg(gettext(e_buffer_is_not_loaded));
        return;
    }
    let mut patbuf = NumBuf::new();
    let pat = arg_string(&mut patbuf, &args[1]);

    let did_emsg_before = did_emsg.get();
    let mut slnum: LineNr = tv_get_lnum_buf(&args[2], Some(buf));
    if did_emsg.get() > did_emsg_before {
        return;
    }
    if slnum < 1 {
        let arg0 = "lnum";
        semsg!("E475: Invalid value for argument {arg0}");
        return;
    }
    let mut elnum: LineNr = tv_get_lnum_buf(&args[3], Some(buf));
    if did_emsg.get() > did_emsg_before {
        return;
    }
    if elnum < 1 || elnum < slnum {
        let arg0 = "end_lnum";
        semsg!("E475: Invalid value for argument {arg0}");
        return;
    }
    elnum = elnum.min(buf.b_ml.ml_line_count);

    let Some(submatches) = want_submatches(args, 4) else {
        return;
    };

    let _cpo = SavedCpo::empty();
    let Some(mut prog) = (unsafe { Regprog::compile(pat) }) else {
        return;
    };
    while slnum <= elnum {
        // SAFETY: a buffer line, NUL-terminated.
        let str = unsafe { cstr::at(ml_get_buf(buf, slnum)) };
        unsafe { get_matches_in_str(str, &mut prog.0, retlist, slnum, submatches, true) };
        slnum += 1;
    }
}

/// The `{dict}` argument the two list-shaped matchers share: `submatches`
/// must be a Boolean if it is there at all. `None` means the argument was
/// rejected and the caller must stop.
fn want_submatches(args: &[TypVal], i: usize) -> Option<bool> {
    if args.len() <= i {
        return Some(false);
    }
    let Some(di) = dict_find(args[i].dict_ref(), b"submatches") else {
        return Some(false);
    };
    if di.di_tv.v_type() != VAR_BOOL {
        let arg0 = "submatches";
        semsg!("E475: Invalid value for argument {arg0}");
        return None;
    }
    Some(tv_get_bool(&di.di_tv) != 0)
}

/// `match({expr}, {pat} [, {start} [, {count}]])`.
pub fn f_match(args: &[TypVal], result: &mut TypVal, _f: EvalFuncData) {
    find_some_match(args, result, kSomeMatch)
}

/// `matchend({expr}, {pat} [, {start} [, {count}]])`.
pub fn f_matchend(args: &[TypVal], result: &mut TypVal, _f: EvalFuncData) {
    find_some_match(args, result, kSomeMatchEnd)
}

/// `matchlist({expr}, {pat} [, {start} [, {count}]])`.
pub fn f_matchlist(args: &[TypVal], result: &mut TypVal, _f: EvalFuncData) {
    find_some_match(args, result, kSomeMatchList)
}

/// `matchstr({expr}, {pat} [, {start} [, {count}]])`.
pub fn f_matchstr(args: &[TypVal], result: &mut TypVal, _f: EvalFuncData) {
    find_some_match(args, result, kSomeMatchStr)
}

/// `matchstrpos({expr}, {pat} [, {start} [, {count}]])`.
pub fn f_matchstrpos(args: &[TypVal], result: &mut TypVal, _f: EvalFuncData) {
    find_some_match(args, result, kSomeMatchStrPos)
}

/// `matchstrlist({list}, {pat} [, {dict}])`.
pub fn f_matchstrlist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the List and its items outlive the call.
    result.write_number(-1);
    list_alloc_ret(result, kListLenUnknown as isize);
    let retlist = result.list_or_null();
    if tv_check_for_list_arg(args, 0).is_err()
        || tv_check_for_string_arg(args, 1).is_err()
        || tv_check_for_opt_dict_arg(args, 2).is_err()
    {
        return;
    }
    let l = args[0].list_or_null();
    if l.is_null() {
        return;
    }
    let mut patbuf = NumBuf::new();
    let pat = arg_string_chk(&mut patbuf, &args[1]);
    if pat.is_null() {
        return;
    }
    let _cpo = SavedCpo::empty();
    let Some(mut prog) = (unsafe { Regprog::compile(pat) }) else {
        return;
    };
    // The `{dict}` is only read once the pattern compiled, as upstream
    // has it: a bad pattern is reported before a bad option.
    let Some(submatches) = want_submatches(args, 2) else {
        return;
    };
    let mut at = 0;
    // By index: `get_matches_in_str` fills a result list and can re-enter.
    // SAFETY: a live list, or NULL, which reads as empty.
    while at < list_items(unsafe { l.as_ref() }).len() {
        let li_tv = &list_items(unsafe { l.as_ref() })[at].li_tv;
        // A non-String item, and the null String, contribute nothing.
        if li_tv.v_type() == VAR_STRING && !li_tv.string_or_null().is_null() {
            // SAFETY: a live String typval's text is NUL-terminated.
            let str = unsafe { cstr::at(li_tv.string_or_null()) };
            let idx = index_of(at);
            unsafe { get_matches_in_str(str, &mut prog.0, retlist, idx, submatches, false) };
        }
        at += 1;
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
    /// Where the item is in the input list, which is where the result
    /// copies it from.  An index and not an address: the callback that
    /// produced the string may have edited the list.
    at: usize,
    score: c_int,
    /// Whether the pattern occurs literally at the first matched position.
    exact: bool,
    /// The matching positions, for `matchfuzzypos()`.
    positions: Option<ListRef>,
}

/// The item's string, as `Request::source` says to find it. A callback's
/// answer lands in `result`, which the caller clears; the string is only
/// borrowed until then.
///
/// # Safety
///
/// `tv` must point at an initialized typval. `result` must point at the
/// caller's return slot: an initialized typval it owns and will clear.
unsafe fn item_string(
    request: &Request,
    tv: &TypVal,
    result: &mut TypVal,
    numbuf: &mut NumBuf,
) -> *const c_char {
    if (*tv).v_type() == VAR_STRING {
        return (*tv).string_or_null();
    }
    if (*tv).v_type() != VAR_DICT {
        return ptr::null();
    }
    match request.source {
        Source::Item => ptr::null(),
        // SAFETY: the key the option dictionary held, NUL-terminated.
        Source::Key(key) => numbuf.dict_string((*tv).dict_ref(), unsafe { cstr::bytes_at(key) }),
        Source::Callback(cb) => {
            // The callback is handed the dict, which it must not be able
            // to free out from under this loop.
            // SAFETY: the value's own dictionary; the argument holds a
            // reference of its own for the length of the call.
            let held = unsafe { DictRef::retained((*tv).dict_or_null()) };
            let argv = [TypVal::dict(held)];
            // SAFETY: `result` is the caller's return value.
            let rv = &mut *result;
            let called = unsafe { callback_call(cb, &argv, rv) };
            drop(argv);
            if called && (*result).v_type() == VAR_STRING {
                (*result).string_or_null()
            } else {
                ptr::null()
            }
        }
    }
}

/// The list held by item `idx` of `list`, which the caller has just built.
///
/// # Safety
///
/// `list` must point at a live list, unaliased for the call.
unsafe fn nested_list(list: *mut List, idx: c_int) -> *mut List {
    let li = list_find(unsafe { list.as_mut() }, idx);
    debug_assert!(!li.is_null(), "fuzzy: result list is short");
    let nested = unsafe { (*li).li_tv.list_or_null() };
    debug_assert!(!nested.is_null(), "fuzzy: result item is not a list");
    nested
}

/// Fuzzy match `request`'s pattern against the strings of `list`, appending
/// the matches to `fmatchlist` in descending score order. For `matchfuzzy()`
/// that is a list of strings; for `matchfuzzypos()` `fmatchlist` already
/// holds three lists — the matched strings, the matching positions of each,
/// and the scores — which are filled in turn.
///
/// # Safety
///
/// `list` must point at a live list, unaliased for the call. `fmatchlist`
/// must point at a live list, unaliased for the call.
unsafe fn fuzzy_match_in_list(list: *mut List, request: &Request, fmatchlist: *mut List) {
    let mut numbuf = NumBuf::new();
    let pattern = unsafe { CStr::from_ptr(request.pattern) };
    let mut found: Vec<FuzzyItem> = Vec::new();
    let mut matches = [0u32; FUZZY_MATCH_MAX_LEN];
    let mut at = 0;
    // SAFETY: the caller's promise: a live list.
    while at < list_items(unsafe { list.as_ref() }).len() {
        if request.limit > 0 && found.len() >= request.limit as usize {
            break;
        }
        let mut rettv = TV_UNKNOWN;
        // SAFETY: as above, and an index of it; re-read each step because
        // `item_string` may run the user's `text_cb`.
        let item_tv = &raw const list_items(unsafe { list.as_ref() })[at].li_tv;
        let item_tv = unsafe { &*item_tv };
        let itemstr = unsafe { item_string(request, item_tv, &mut rettv, &mut numbuf) };
        if !itemstr.is_null() {
            let itemstr = unsafe { CStr::from_ptr(itemstr) };
            let (score, filled) = fuzzy_match(itemstr, pattern, request.matchseq, &mut matches);
            if filled != 0 {
                // Upstream reads the string at the first *character*
                // position as if it were a byte offset. Preserved: it is
                // only a tie-break between two equally scored items.
                let first = matches[0] as usize;
                let exact = itemstr
                    .to_bytes()
                    .get(first..)
                    .is_some_and(|tail| tail.starts_with(pattern.to_bytes()));
                let positions = request.retmatchpos.then(|| {
                    let positions = tv_list_alloc(kListLenMayKnow as isize);
                    // One position per pattern character that took part
                    // in the match, i.e. all but the word separators.
                    let placed = matched_char_count(pattern, request.matchseq);
                    for at in matches.iter().take(placed) {
                        unsafe { (*positions.as_ptr()).push_number(*at as VarNumber) };
                    }
                    positions
                });
                found.push(FuzzyItem {
                    idx: found.len(),
                    at,
                    score,
                    exact,
                    positions,
                });
            }
        }
        tv_clear(&mut rettv);
        at += 1;
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
        // SAFETY: the caller's live list; a callback above may have
        // shortened it, in which case the item it matched is simply gone.
        let Some(li) = (list_items(unsafe { list.as_ref() })).get(item.at) else {
            continue;
        };
        unsafe { (*strings).push_copy(&li.li_tv) };
    }
    if request.retmatchpos {
        let positions = unsafe { nested_list(fmatchlist, -2) };
        for item in &mut found {
            let list = item.positions.take().expect("fuzzy: positions were kept");
            unsafe { (*positions).push_list(Some(list)) };
        }
        let scores = unsafe { nested_list(fmatchlist, -1) };
        for item in &found {
            unsafe { (*scores).push_number(item.score as VarNumber) };
        }
    }
}

/// The body of `matchfuzzy()` and, with `retmatchpos`, `matchfuzzypos()`.
fn do_fuzzymatch(args: &[TypVal], result: &mut TypVal, retmatchpos: bool) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let list = &args[0];
    if list.v_type() != VAR_LIST || list.list_or_null().is_null() {
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
    let pat = &args[1];
    if pat.v_type() != VAR_STRING || pat.string_or_null().is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg0 = unsafe { c_str(numbuf.string_ptr(pat)) };
        semsg!("E475: Invalid argument: {arg0}");
        return;
    }

    // The optional third argument says where to find the string of a
    // dict item, and how much of the list to bother with.
    let mut cb = Callback::None;
    let mut key = ptr::null();
    let mut matchseq = false;
    let mut limit = 0;
    if args.len() > 2 {
        if tv_check_for_nonnull_dict_arg(args, 2).is_err() {
            return;
        }
        // SAFETY: the argument's own dictionary, which the check above
        // says is there.
        let d = unsafe { &mut *args[2].dict_or_null() };
        if let Some(di) = d.find(b"key") {
            if di.di_tv.v_type() != VAR_STRING
                || di.di_tv.string_or_null().is_null()
                // SAFETY: a non-null string of the item's own.
                || unsafe { *di.di_tv.string_or_null() } == 0
            {
                let got = numbuf2.string_ptr(&di.di_tv);
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let got = unsafe { c_str(got) };
                semsg!("E475: Invalid value for argument {}: {got}", "key");
                return;
            }
            key = numbuf3.string_ptr(&di.di_tv);
        } else if !dict_get_callback(Some(d), b"text_cb", &mut cb) {
            semsg!("E475: Invalid value for argument {}", "text_cb");
            return;
        }
        if let Some(di) = d.find(b"limit") {
            if di.di_tv.v_type() != VAR_NUMBER {
                semsg!("E475: Invalid value for argument {}", "limit");
                return;
            }
            limit = tv_get_number_chk(&di.di_tv).unwrap_or(-1) as c_int;
        }
        matchseq = d.has_key(b"matchseq");
    }

    // matchfuzzypos() answers three lists: the matching strings, their
    // matching positions, and their scores.
    let len = if retmatchpos {
        3
    } else {
        kListLenUnknown as isize
    };
    let result = tv_list_alloc_ret(result, len);
    if retmatchpos {
        for _ in 0..3 {
            (*result).push_list(Some(tv_list_alloc(kListLenUnknown as isize)));
        }
    }
    let request = Request {
        pattern: numbuf4.string_ptr(pat),
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
    unsafe { fuzzy_match_in_list(list.list_or_null(), &request, result) };
    unsafe { callback_free(&raw mut cb) };
}

/// `matchfuzzy()`: the items of a list that fuzzy match a pattern.
pub(crate) fn f_matchfuzzy(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    do_fuzzymatch(args, result, false)
}

/// `matchfuzzypos()`: as [`f_matchfuzzy`], plus where each match landed and
/// what it scored.
pub(crate) fn f_matchfuzzypos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    do_fuzzymatch(args, result, true)
}
