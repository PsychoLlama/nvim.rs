//! Matching a pattern against a string: the `match*()` family.
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::wrappers::arg_number_chk;
use super::{
    NSUBEXP, SomeMatchType, kSomeMatch, kSomeMatchEnd, kSomeMatchList, kSomeMatchStr,
    kSomeMatchStrPos, tv_get_buf,
};
use crate::eval::callback_call;
use crate::eval::encode::encode_tv2echo;
use crate::eval::typval::{
    ListRef, NumBuf, callback_free, dict_find, dict_get_callback, index_of, list_items,
    list_items_mut, list_uidx, tv_check_for_buffer_arg, tv_check_for_list_arg,
    tv_check_for_lnum_arg, tv_check_for_nonnull_dict_arg, tv_check_for_opt_dict_arg,
    tv_check_for_string_arg, tv_clear, tv_copy, tv_dict_alloc, tv_get_bool, tv_get_lnum_buf,
    tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret,
};
use crate::fuzzy::{FUZZY_MATCH_MAX_LEN, fuzzy_match, matched_char_count};
use crate::mbyte::cluster_len;
use crate::memline::Lines;
use crate::memory::ThinCString;
use crate::message::e_buffer_is_not_loaded;
use crate::message::emsg;
use crate::message::state::did_emsg;
use crate::message_fmt::msg_cstr;
use crate::option::SavedCpo;
use crate::option::vars::p_ic;
use crate::os::cshim::gettext;
use crate::regexp::{OwnedProg, RE_MAGIC, RE_STRING};
use crate::semsg;
use crate::types::{
    Callback, EvalFuncData, LineNr, List, RegMatch, TypVal, VAR_BOOL, VAR_DICT, VAR_LIST,
    VAR_NUMBER, VAR_STRING, VarNumber, kListLenMayKnow, kListLenUnknown,
};
use core::ffi::{CStr, c_int};

/// A pattern compiled the way the whole family compiles one, with the
/// 'ignorecase' it was compiled under. `None` when it did not compile --
/// `vim_regcomp` has already reported why.
fn compile(pat: &CStr) -> Option<(OwnedProg, bool)> {
    OwnedProg::compile(pat, RE_MAGIC + RE_STRING).map(|prog| (prog, p_ic()))
}

/// The shared body of `match()`, `matchend()`, `matchlist()`, `matchstr()`
/// and `matchstrpos()`.
fn find_some_match(args: &[TypVal], result: &mut TypVal, kind: SomeMatchType) {
    let mut numbuf = NumBuf::new();
    let _cpo = SavedCpo::empty();
    result.write_number(-1);
    match kind {
        kSomeMatchList => {
            tv_list_alloc_ret(result, kListLenMayKnow as isize);
        }
        kSomeMatchStrPos => {
            // Seeded with the "no match" answer, which the tail of this
            // function trims back to three items for a String subject.
            let seeded = tv_list_alloc_ret(result, 4);
            seeded.push_bytes(Some(b""));
            seeded.push_number(-1);
            seeded.push_number(-1);
            seeded.push_number(-1);
        }
        kSomeMatchStr => {
            result.write_string(None);
        }
        _ => {}
    }

    // A List subject; `v:_null_list` is no subject at all.
    let is_list = args.first().is_some_and(|arg| arg.v_type() == VAR_LIST);
    let list: Option<&ListRef> = if is_list { args[0].list_shared() } else { None };
    // Nothing below this point may return early without running the
    // trailing fixup, so the body is one labelled block as the C's
    // `goto theend` was.
    'theend: {
        if is_list && list.is_none() {
            break 'theend;
        }
        // A String subject, and how far a `{start}` without a `{count}`
        // moved it on: the answers are counted from where it began.
        let whole = if list.is_none() {
            numbuf.string(&args[0])
        } else {
            c""
        };
        let mut skipped = 0;
        let mut len = whole.count_bytes();
        // Where the List walk is; an index, because the echo below is made
        // afresh for every item.
        let mut at = 0;
        let mut idx: c_int = 0;
        let mut nth: VarNumber = 1;
        let mut startcol = 0;

        let mut patbuf = NumBuf::new();
        let Some(pat) = patbuf.string_chk(&args[1]) else {
            break 'theend;
        };

        if args.len() > 2 {
            let mut error = false;
            let start = arg_number_chk(&args[2], Some(&mut error));
            if error {
                break 'theend;
            }
            if let Some(list) = list {
                idx = list_uidx(Some(list), start as c_int);
                let Ok(start_at) = usize::try_from(idx) else {
                    break 'theend;
                };
                at = start_at;
            } else {
                let start = usize::try_from(start).unwrap_or(0);
                if start > len {
                    break 'theend;
                }
                // With a `{count}` the start is a column the matcher is
                // told about, so that `^` still anchors to the real start
                // of the string; without one the string itself moves on.
                if args.len() > 3 {
                    startcol = start;
                } else {
                    skipped = start;
                    len -= start;
                }
            }
            if args.len() > 3 {
                nth = arg_number_chk(&args[3], Some(&mut error));
            }
            if error {
                break 'theend;
            }
        }

        let Some((mut prog, ignore_case)) = compile(pat) else {
            break 'theend;
        };
        // The echo of the List item the walk is on.
        let mut echoed = ThinCString::empty();
        let mut found: Option<RegMatch>;
        loop {
            let subject = if let Some(list) = list {
                let Some(item) = list_items(Some(list)).get(at) else {
                    found = None;
                    break;
                };
                echoed = encode_tv2echo(&item.li_tv);
                echoed.as_cstr()
            } else {
                &whole[skipped..]
            };
            found = prog.exec_nl(subject, startcol, ignore_case);
            // `nth` counts down only on a match: the C spells this as
            // `match && (--nth <= 0)`, and the short circuit is what stops
            // a non-matching List item from consuming a count.
            if found.is_some() {
                nth -= 1;
                if nth <= 0 {
                    break;
                }
            }
            if list.is_some() {
                at += 1;
                idx += 1;
                continue;
            }
            let Some(hit) = &found else {
                break;
            };
            // Same string, next match: step past the character the match
            // started on. A match that did not advance, or one past the
            // end, ends the search.
            let start = hit.starts[0].unwrap_or(0);
            startcol = start + cluster_len(&subject.to_bytes()[start..]);
            if startcol > len || startcol <= start {
                found = None;
                break;
            }
        }

        let Some(hit) = found else {
            break 'theend;
        };
        let subject = if list.is_some() {
            echoed.as_bytes()
        } else {
            whole[skipped..].to_bytes()
        };
        match kind {
            kSomeMatchStrPos => {
                // The four items seeded above, overwritten in place.
                let seeded = list_items_mut(result.list_mut());
                drop(seeded[0].li_tv.take_string());
                let span = hit.group(0).unwrap_or(0..0);
                seeded[0]
                    .li_tv
                    .write_string(Some(ThinCString::from_bytes(&subject[span.clone()])));
                seeded[2]
                    .li_tv
                    .write_number((skipped + span.start) as VarNumber);
                seeded[3]
                    .li_tv
                    .write_number((skipped + span.end) as VarNumber);
                if list.is_some() {
                    seeded[1].li_tv.write_number(VarNumber::from(idx));
                }
            }
            kSomeMatchList => {
                let out = result.list_mut().expect("the list allocated above");
                for i in 0..NSUBEXP as usize {
                    out.push_bytes(hit.group_bytes(i, subject));
                }
            }
            kSomeMatchStr => {
                if let Some(list) = list {
                    // A List subject answers with the whole item, not with
                    // the part that matched.
                    tv_copy(&list_items(Some(list))[at].li_tv, result);
                } else {
                    let span = hit.group(0).unwrap_or(0..0);
                    result.write_string(Some(ThinCString::from_bytes(&subject[span])));
                }
            }
            _ => {
                if list.is_some() {
                    result.write_number(VarNumber::from(idx));
                } else {
                    let edge = if kind == kSomeMatch {
                        hit.starts[0]
                    } else {
                        hit.ends[0]
                    };
                    result.write_number((skipped + edge.unwrap_or(0)) as VarNumber);
                }
            }
        }
    }

    // `matchstrpos()` on a String has no index to report, so the
    // placeholder seeded above comes back out.
    if kind == kSomeMatchStrPos
        && list.is_none()
        && let Some(seeded) = result.list_mut()
    {
        drop(seeded.take_range(1, 1));
    }
}

/// Append one dict per match of `prog` in `text` to `out`.
fn get_matches_in_str(
    text: &CStr,
    (prog, ignore_case): (&mut OwnedProg, bool),
    out: &mut List,
    idx: c_int,
    submatches: bool,
    matchbuf: bool,
) {
    let bytes = text.to_bytes();
    let mut startidx = 0;
    while let Some(hit) = prog.exec_nl(text, startidx, ignore_case) {
        let held = tv_dict_alloc();
        let found = held.edit();
        // A buffer's matches are keyed by line number, a List's by the
        // index of the item they came from.
        let key: &[u8] = if matchbuf { b"lnum" } else { b"idx" };
        let _ = found.add_number(key, VarNumber::from(idx));
        let span = hit.group(0).unwrap_or(0..0);
        let _ = found.add_number(b"byteidx", span.start as VarNumber);
        let _ = found.add_str_len(b"text", Some(&bytes[span.clone()]));
        if submatches {
            let groups = tv_list_alloc(NSUBEXP as isize - 1);
            for i in 1..NSUBEXP as usize {
                groups
                    .edit()
                    .push_bytes(Some(hit.group_bytes(i, bytes).unwrap_or(b"")));
            }
            let _ = found.add_list(b"submatches", Some(groups));
        }
        out.push_dict(Some(held));
        // Resume past this match; stop at the end of the string, and
        // stop on a match that did not advance.
        startidx = span.end;
        if startidx >= bytes.len() || startidx <= span.start {
            return;
        }
    }
}

/// `matchbufline({buf}, {pat}, {lnum}, {end} [, {dict}])`.
pub fn f_matchbufline(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);
    tv_list_alloc_ret(result, kListLenUnknown as isize);
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
            let what = msg_cstr(numbuf.string(&args[0]));
            semsg!("E158: Invalid buffer name: {what}");
        }
        return;
    };
    if buf.b_ml.ml_mfp.is_null() {
        emsg(gettext(e_buffer_is_not_loaded));
        return;
    }
    let mut patbuf = NumBuf::new();
    let pat = patbuf.string(&args[1]);

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
    let Some((mut prog, ignore_case)) = compile(pat) else {
        return;
    };
    let out = result.list_mut().expect("the list allocated above");
    let mut lines = Lines::in_buffer(buf);
    while slnum <= elnum {
        let text = lines.line_cstr(slnum, 0);
        get_matches_in_str(text, (&mut prog, ignore_case), out, slnum, submatches, true);
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
    result.write_number(-1);
    tv_list_alloc_ret(result, kListLenUnknown as isize);
    if tv_check_for_list_arg(args, 0).is_err()
        || tv_check_for_string_arg(args, 1).is_err()
        || tv_check_for_opt_dict_arg(args, 2).is_err()
    {
        return;
    }
    let Some(list) = args[0].list_ref() else {
        return;
    };
    let mut patbuf = NumBuf::new();
    let Some(pat) = patbuf.string_chk(&args[1]) else {
        return;
    };
    let _cpo = SavedCpo::empty();
    let Some((mut prog, ignore_case)) = compile(pat) else {
        return;
    };
    // The `{dict}` is only read once the pattern compiled, as upstream
    // has it: a bad pattern is reported before a bad option.
    let Some(submatches) = want_submatches(args, 2) else {
        return;
    };
    let out = result.list_mut().expect("the list allocated above");
    // Matching runs no user code, so the items can be walked in place.
    for (at, item) in list.items().iter().enumerate() {
        // A non-String item, and the null String, contribute nothing.
        if let Some(text) = item.li_tv.string_cstr() {
            let idx = index_of(at);
            get_matches_in_str(text, (&mut prog, ignore_case), out, idx, submatches, false);
        }
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
enum Source<'a> {
    Item,
    Key(&'a CStr),
    Callback(&'a Callback),
}

/// What one `matchfuzzy()`/`matchfuzzypos()` call was asked for: the pattern,
/// where each item's string comes from, whether the words of a multi-word
/// pattern have to match in sequence, whether the matching positions are
/// wanted too (that is `matchfuzzypos()`), and how many matches are enough.
struct Request<'a> {
    pattern: &'a CStr,
    source: Source<'a>,
    matchseq: bool,
    retmatchpos: bool,
    limit: c_int,
}

/// One list item that matched.
struct FuzzyItem {
    /// Where it sat among the matches, which is how ties are broken.
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

/// Fuzzy match `request`'s pattern against the strings of `list`: the
/// matches, best first.
///
/// The list is reached through its handle afresh for every item, because a
/// `text_cb` runs user code that may edit it.
fn fuzzy_match_in_list(list: &ListRef, request: &Request) -> Vec<FuzzyItem> {
    let mut numbuf = NumBuf::new();
    let pattern = request.pattern;
    let mut found: Vec<FuzzyItem> = Vec::new();
    let mut matches = [0u32; FUZZY_MATCH_MAX_LEN];
    let mut at = 0;
    while at < list.items().len() {
        if request.limit > 0 && found.len() >= request.limit as usize {
            break;
        }
        let mut rettv = TypVal::Unknown;
        let kind = list.items()[at].li_tv.v_type();
        let itemstr = if kind == VAR_STRING {
            list.items()[at].li_tv.string_cstr()
        } else if kind != VAR_DICT {
            None
        } else {
            match request.source {
                Source::Item => None,
                Source::Key(key) => {
                    numbuf.dict_string(list.items()[at].li_tv.dict_ref(), key.to_bytes())
                }
                Source::Callback(cb) => {
                    // The callback is handed the dict, which it must not be
                    // able to free out from under this loop: the argument
                    // holds a reference of its own for the call.
                    let argv = [TypVal::dict(list.items()[at].li_tv.dict_handle())];
                    let called = callback_call(cb, &argv, &mut rettv);
                    drop(argv);
                    if called && rettv.v_type() == VAR_STRING {
                        rettv.string_cstr()
                    } else {
                        None
                    }
                }
            }
        };
        if let Some(itemstr) = itemstr {
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
                        positions.edit().push_number(VarNumber::from(*at));
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

    // Best score first; an exact match wins a tie, and the input order
    // settles the rest. No two items share an `idx`, so this is a total
    // order and the sort needs no stability of its own.
    found.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(b.exact.cmp(&a.exact))
            .then(a.idx.cmp(&b.idx))
    });
    found
}

/// The body of `matchfuzzy()` and, with `retmatchpos`, `matchfuzzypos()`.
fn do_fuzzymatch(args: &[TypVal], result: &mut TypVal, retmatchpos: bool) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut numbuf4 = NumBuf::new();
    let Some(list) = args[0].list_shared() else {
        let who = if retmatchpos {
            "matchfuzzypos()"
        } else {
            "matchfuzzy()"
        };
        semsg!("E686: Argument of {who} must be a List");
        return;
    };
    let pat = &args[1];
    if pat.string_ref().is_none() {
        let arg0 = msg_cstr(numbuf.string(pat));
        semsg!("E475: Invalid argument: {arg0}");
        return;
    }

    // The optional third argument says where to find the string of a
    // dict item, and how much of the list to bother with.
    let mut cb = Callback::None;
    let mut key = None;
    let mut matchseq = false;
    let mut limit = 0;
    if args.len() > 2 {
        if tv_check_for_nonnull_dict_arg(args, 2).is_err() {
            return;
        }
        let d = args[2].dict_ref().expect("checked to be a dictionary");
        if let Some(di) = d.find(b"key") {
            if di.di_tv.string_ref().is_none_or(ThinCString::is_empty) {
                let got = msg_cstr(numbuf2.string(&di.di_tv));
                semsg!("E475: Invalid value for argument {}: {got}", "key");
                return;
            }
            key = Some(numbuf3.string(&di.di_tv));
        } else if !dict_get_callback(args[2].dict_shared(), b"text_cb", &mut cb) {
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

    let request = Request {
        pattern: numbuf4.string(pat),
        source: if let Some(key) = key {
            Source::Key(key)
        } else if cb.is_set() {
            Source::Callback(&cb)
        } else {
            Source::Item
        },
        matchseq,
        retmatchpos,
        limit,
    };
    let found = fuzzy_match_in_list(list, &request);
    callback_free(&mut cb);

    // matchfuzzy() answers just the strings; matchfuzzypos() answers them
    // in the first of three lists, the matching positions in the second and
    // the scores in the third.
    let strings = tv_list_alloc(kListLenUnknown as isize);
    for item in &found {
        // A callback above may have shortened the list, in which case the
        // item it matched is simply gone.
        if let Some(li) = list.items().get(item.at) {
            strings.edit().push_copy(&li.li_tv);
        }
    }
    if !retmatchpos {
        result.write_list(Some(strings));
        return;
    }
    let out = tv_list_alloc_ret(result, 3);
    let positions = tv_list_alloc(kListLenUnknown as isize);
    let scores = tv_list_alloc(kListLenUnknown as isize);
    for item in found {
        positions
            .edit()
            .push_list(Some(item.positions.expect("fuzzy: positions were kept")));
        scores.edit().push_number(VarNumber::from(item.score));
    }
    out.push_list(Some(strings));
    out.push_list(Some(positions));
    out.push_list(Some(scores));
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
