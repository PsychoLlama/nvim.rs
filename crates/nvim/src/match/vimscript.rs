//! The `match*()` Vimscript functions.
//!
//! `matchadd()`/`matchaddpos()`/`matchdelete()`/`clearmatches()` are thin
//! wrappers over the list operations in the parent; `getmatches()` and
//! `setmatches()` are the dictionary round trip that lets a match list be
//! saved and restored, including the `pos1`..`pos8` keys a position match
//! is described by.

#![forbid(unsafe_code)]

use super::*;
use crate::eval::typval::{DictRef, NumBuf, list_items, list_iter};
use crate::semsg;
use crate::types::{Failed, VAR_DICT, VAR_LIST, kListLenMayKnow};
use crate::winlayer::Win;
use std::ffi::CString;

/// How many `posN` keys a saved position match can carry.
///
/// `matchaddpos()` itself takes any number of positions, but only the first
/// eight survive a `getmatches()`/`setmatches()` round trip — the reader
/// stops at `pos8`, so this is upstream's limit and not an arbitrary one.
const MAX_SAVED_POS: c_int = 8;

/// Reads `matchadd()`'s and `matchaddpos()`' optional fifth argument, the
/// `{'conceal': c, 'window': w}` dictionary: the conceal character's text,
/// copied, and the window, written to `win`.
fn matchadd_dict_arg(tv: &TypVal, win: &mut Win) -> Result<Option<CString>, Failed> {
    if tv.v_type() != VAR_DICT {
        emsg(gettext(e_dictreq));
        return Err(Failed);
    }
    let dict = tv.dict_ref();

    let mut numbuf = NumBuf::new();
    let conceal = dict_find(dict, b"conceal").map(|di| numbuf.string(&di.di_tv).to_owned());

    let Some(di) = dict_find(dict, b"window") else {
        return Ok(conceal);
    };
    let Some(found) = find_win_by_nr_or_id(&di.di_tv) else {
        emsg(gettext(e_invalwindow));
        return Err(Failed);
    };
    *win = found;
    Ok(conceal)
}

/// `clearmatches([win])`.
pub(crate) fn f_clearmatches(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    if let Some(win) = get_optional_window(args, 0) {
        clear_matches(win);
    }
}

/// One match as `getmatches()` describes it.
fn match_dict(item: &MatchItem) -> DictRef {
    let dict_held = tv_dict_alloc();
    let dict = dict_held.edit();
    if item.mit_match.regprog.is_none() {
        // Added with matchaddpos(): one `posN` key per position.
        for (i, llpos) in item.mit_pos_array.iter().enumerate() {
            if llpos.lnum == 0 {
                break;
            }
            // A column of zero means the whole line, and is reported
            // as a one-element list.
            let held = tv_list_alloc(1 + if llpos.col > 0 { 2 } else { 0 });
            let sub = held.edit();
            sub.push_number(VarNumber::from(llpos.lnum));
            if llpos.col > 0 {
                sub.push_number(VarNumber::from(llpos.col));
                sub.push_number(VarNumber::from(llpos.len));
            }
            let key = format!("pos{}", i + 1);
            let _ = dict.add_list(key.as_bytes(), Some(held));
        }
    } else {
        let pattern = item.mit_pattern.as_ref().map(XString::as_cstr);
        let _ = dict.add_str(b"pattern", pattern);
    }
    let _ = dict.add_str(b"group", Some(syn_id2name(item.mit_hlg_id)));
    let _ = dict.add_number(b"priority", VarNumber::from(item.mit_priority));
    let _ = dict.add_number(b"id", VarNumber::from(item.mit_id));

    if item.mit_conceal_char != 0 {
        let mut buf = [0u8; 6];
        let len = encode_char(item.mit_conceal_char, &mut buf);
        let _ = dict.add_str_len(b"conceal", Some(&buf[..len]));
    }
    dict_held
}

/// `getmatches([win])`.
pub(crate) fn f_getmatches(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let win = get_optional_window(args, 0);
    let l = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    let Some(win) = win else {
        return;
    };
    for item in &win.w_matches {
        l.push_dict(Some(match_dict(item)));
    }
}

/// `setmatches(list [, win])`.
///
/// Rebuilds a whole match list from `getmatches()`' answer. The list is
/// validated in full *before* anything is cleared, so a malformed entry
/// leaves the window's matches alone.
pub(crate) fn f_setmatches(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut group_buf = NumBuf::new();
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let win = get_optional_window(args, 1);

    result.write_number(-1);
    if args[0].v_type() != VAR_LIST {
        emsg(gettext(e_listreq));
        return;
    }
    let Some(win) = win else {
        return;
    };
    let l = args[0].list_ref();

    // To some extent make sure this really came from getmatches().
    for (li_idx, li) in list_iter(l).enumerate() {
        let tv = &li.li_tv;
        let Some(d) = tv.dict_ref().filter(|_| tv.v_type() == VAR_DICT) else {
            semsg!(
                "E474: List item {} is either not a dictionary or an empty one",
                li_idx
            );
            return;
        };
        let has = |key: &[u8]| dict_find(Some(d), key).is_some();
        let ok =
            has(b"group") && (has(b"pattern") || has(b"pos1")) && has(b"priority") && has(b"id");
        if !ok {
            semsg!(
                "E474: List item {} is missing one of the required keys",
                li_idx
            );
            return;
        }
    }

    clear_matches(win);
    let mut match_add_failed = false;
    // By index: `match_add` runs no user code, but the dictionary lookups
    // below do reach the evaluator, and the list is the caller's own.
    let mut at = 0;
    while at < list_items(l).len() {
        let d = list_items(l)[at].li_tv.dict_ref();

        // A match with no `pattern` is a position match: collect
        // pos1..pos8 into the list `match_add` wants.
        let mut held = None;
        if dict_find(d, b"pattern").is_none() {
            let positions = tv_list_alloc(MAX_SAVED_POS as ptrdiff_t + 1);
            for i in 1..MAX_SAVED_POS + 1 {
                let key = format!("pos{i}");
                let Some(pos_di) = dict_find(d, key.as_bytes()) else {
                    break;
                };
                if pos_di.di_tv.v_type() != VAR_LIST {
                    // The earlier entries stay restored, as upstream's do:
                    // the validation above does not look inside a `posN`
                    // key. Upstream *also* leaked the position list here --
                    // it took a reference per entry and gave one back --
                    // which the handle no longer permits.
                    return;
                }
                positions.edit().push_copy(&pos_di.di_tv);
            }
            held = Some(positions);
        }

        // Three scratches are in play here — this one and the two the
        // frame lends below — and none may be reused before its value is.
        let group = group_buf.dict_string(d, b"group").unwrap_or(c"");
        let priority = dict_get_number(d, b"priority") as c_int;
        let id = dict_get_number(d, b"id") as c_int;
        let conceal = dict_find(d, b"conceal").map(|di| numbuf.string(&di.di_tv));

        let added = match &held {
            None => {
                let pattern = numbuf2.dict_string(d, b"pattern");
                match_add(win, group, pattern, priority, id, None, conceal)
            }
            Some(positions) => {
                let positions = Some(&*positions.edit());
                match_add(win, group, None, priority, id, positions, conceal)
            }
        };
        if added != id {
            match_add_failed = true;
        }

        at += 1;
    }
    if !match_add_failed {
        result.write_number(0);
    }
}

/// The optional `priority`, `id` and dictionary arguments `matchadd()` and
/// `matchaddpos()` share, read from `argvars[2..=4]`: the priority, the id,
/// the conceal character's text and the window.
///
/// Answers `None` once something has been diagnosed.
fn optional_args(args: &[TypVal]) -> Option<(c_int, c_int, Option<CString>, Win)> {
    let mut prio = DEFAULT_PRIORITY;
    let mut id = -1;
    let mut conceal_char = None;
    let mut win = Win::current();
    let mut error = false;

    // Nested, not sequential: an `id` is only read when a `priority` was
    // given, and the dictionary only when an `id` was.
    if args.len() > 2 {
        prio = tv_get_number_chk(&args[2]).unwrap_or_else(|_| {
            error = true;
            0
        }) as c_int;
        if args.len() > 3 {
            id = tv_get_number_chk(&args[3]).unwrap_or_else(|_| {
                error = true;
                0
            }) as c_int;
            if args.len() > 4 {
                conceal_char = matchadd_dict_arg(&args[4], &mut win).ok()?;
            }
        }
    }
    if error {
        None
    } else {
        Some((prio, id, conceal_char, win))
    }
}

/// `matchadd(group, pattern [, priority [, id [, options]]])`.
pub(crate) fn f_matchadd(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // The group and the pattern are held at once, so each is given a
    // scratch of its own.
    let mut grpbuf = NumBuf::new();
    let mut patbuf = NumBuf::new();
    let grp = grpbuf.string_chk(&args[0]);
    let pat = patbuf.string_chk(&args[1]);

    result.write_number(-1);
    let (Some(grp), Some(pat)) = (grp, pat) else {
        return;
    };
    let Some((prio, id, conceal_char, win)) = optional_args(args) else {
        return;
    };
    if (1..=3).contains(&id) {
        semsg!("E798: ID is reserved for \":match\": {}", id);
        return;
    }

    let conceal = conceal_char.as_deref();
    let added = match_add(win, grp, Some(pat), prio, id, None, conceal);
    result.write_number(VarNumber::from(added));
}

/// `matchaddpos(group, positions [, priority [, id [, options]]])`.
pub(crate) fn f_matchaddpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut buf = NumBuf::new();
    result.write_number(-1);

    let Some(group) = buf.string_chk(&args[0]) else {
        return;
    };
    if args[1].v_type() != VAR_LIST {
        semsg!("E686: Argument of {} must be a List", "matchaddpos()");
        return;
    }
    let l = args[1].list_ref();
    if list_len(l) == 0 {
        return;
    }

    let Some((prio, id, conceal_char, win)) = optional_args(args) else {
        return;
    };
    // 3 is allowed: matchaddpos() is meant to stand in for `:3match`.
    if id == 1 || id == 2 {
        semsg!("E798: ID is reserved for \"match\": {}", id);
        return;
    }

    let conceal = conceal_char.as_deref();
    let added = match_add(win, group, None, prio, id, l, conceal);
    result.write_number(VarNumber::from(added));
}

/// `matcharg(id)` — the `[group, pattern]` of `:match`, `:2match` or
/// `:3match`, or `["", ""]` when that one is not set.
pub(crate) fn f_matcharg(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let id = tv_get_number(&args[0]) as c_int;
    let is_excmd = (1..=3).contains(&id);
    // Any other id answers an empty list, not an error.
    let l = tv_list_alloc_ret(result, if is_excmd { 2 } else { 0 });
    if !is_excmd {
        return;
    }
    let window = Win::current();
    match get_match(&window, id) {
        None => {
            l.push_bytes(None);
            l.push_bytes(None);
        }
        Some(item) => {
            l.push_str(Some(syn_id2name(item.mit_hlg_id)));
            l.push_str(item.mit_pattern.as_ref().map(XString::as_cstr));
        }
    }
}

/// `matchdelete(id [, win])`.
pub(crate) fn f_matchdelete(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let win = get_optional_window(args, 1);
    let deleted = match win {
        None => -1,
        Some(win) => {
            let id = tv_get_number(&args[0]) as c_int;
            VarNumber::from(match_delete(win, id, true))
        }
    };
    result.write_number(deleted);
}
