//! The `match*()` Vimscript functions.
//!
//! `matchadd()`/`matchaddpos()`/`matchdelete()`/`clearmatches()` are thin
//! wrappers over the list operations in the parent; `getmatches()` and
//! `setmatches()` are the dictionary round trip that lets a match list be
//! saved and restored, including the `pos1`..`pos8` keys a position match
//! is described by.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::eval::typval::{ListRef, NumBuf, tv_list_items, tv_list_iter};
use crate::semsg;
use crate::types::{Failed, MB_MAXCHAR, VAR_DICT, VAR_LIST, kListLenMayKnow};
use crate::winlayer::Win;

/// How many `posN` keys a saved position match can carry.
///
/// `matchaddpos()` itself takes any number of positions, but only the first
/// eight survive a `getmatches()`/`setmatches()` round trip — the reader
/// stops at `pos8`, so this is upstream's limit and not an arbitrary one.
const MAX_SAVED_POS: c_int = 8;

/// `tv_dict_add_str` with a Rust key: the dictionary copies exactly the
/// length it is given, so the transpile's `b"group\0"` plus
/// `size_of::<[c_char; 6]>() - 1` collapses to a `&str`.
///
/// # Safety
/// `d` must be live and `val` null or NUL-terminated.
unsafe fn put_str(d: *mut Dict, key: &str, val: *const c_char) {
    // SAFETY: the caller's dictionary and value.
    let _ = unsafe { tv_dict_add_str(d, key.as_ptr().cast(), key.len(), val) };
}

/// `tv_dict_add_nr` with a Rust key; see [`put_str`].
///
/// # Safety
/// `d` must be live.
unsafe fn put_nr(d: *mut Dict, key: &str, nr: VarNumber) {
    // SAFETY: the caller's dictionary.
    let _ = unsafe { tv_dict_add_nr(d, key.as_ptr().cast(), key.len(), nr) };
}

/// `tv_dict_find` with a Rust key; null when absent.
///
/// # Safety
/// `d` must be null or live.
unsafe fn find(d: *const Dict, key: &str) -> *mut DictItem {
    // SAFETY: the caller's dictionary.
    unsafe { tv_dict_find(d, key.as_ptr().cast(), key.len() as ptrdiff_t) }
}

/// Reads `matchadd()`'s and `matchaddpos()`' optional fifth argument, the
/// `{'conceal': c, 'window': w}` dictionary.
///
/// # Safety
/// `tv` must be live; `conceal_char` and `win` must be writable.
unsafe fn matchadd_dict_arg(
    tv: &TypVal,
    conceal_char: *mut *const c_char,
    win: &mut Win,
    numbuf: &mut NumBuf,
) -> Result<(), Failed> {
    // SAFETY: the caller's typval and out-parameters.
    if (*tv).v_type() != VAR_DICT {
        emsg(gettext(e_dictreq));
        return Err(Failed);
    }
    let dict = (*tv).dict_or_null();

    let di = unsafe { find(dict, "conceal") };
    if !di.is_null() {
        unsafe { *conceal_char = numbuf.string(&(*di).di_tv) };
    }

    let di = unsafe { find(dict, "window") };
    if di.is_null() {
        return Ok(());
    }
    let Some(found) = (unsafe { find_win_by_nr_or_id(&(*di).di_tv) }) else {
        emsg(gettext(e_invalwindow));
        return Err(Failed);
    };
    *win = found;
    Ok(())
}

/// `clearmatches([win])`.
pub(crate) fn f_clearmatches(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    if let Some(win) = get_optional_window(args, 0) {
        clear_matches(win);
    }
}

/// `getmatches([win])`.
pub(crate) fn f_getmatches(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    let win = get_optional_window(args, 0);
    let l = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    let Some(win) = win else {
        return;
    };

    let mut cur = win.w_match_head;
    while !cur.is_null() {
        let dict_held = tv_dict_alloc();
        let dict = dict_held.as_ptr();
        if unsafe { (*cur).mit_match.regprog }.is_null() {
            // Added with matchaddpos(): one `posN` key per position.
            for i in 0..unsafe { (*cur).mit_pos_count } {
                let llpos = unsafe { (*cur).mit_pos_array.offset(i as isize) };
                if unsafe { (*llpos).lnum } == 0 {
                    break;
                }
                // A column of zero means the whole line, and is reported
                // as a one-element list.
                let held = unsafe { tv_list_alloc(1 + if (*llpos).col > 0 { 2 } else { 0 }) };
                let sub = held.as_ptr();
                unsafe { tv_list_append_number(sub, (*llpos).lnum as VarNumber) };
                if unsafe { (*llpos).col } > 0 {
                    unsafe { tv_list_append_number(sub, (*llpos).col as VarNumber) };
                    unsafe { tv_list_append_number(sub, (*llpos).len as VarNumber) };
                }
                let key = format!("pos{}", i + 1);
                let _ =
                    unsafe { tv_dict_add_list(dict, key.as_ptr().cast(), key.len(), Some(held)) };
            }
        } else {
            unsafe { put_str(dict, "pattern", (*cur).mit_pattern) };
        }
        unsafe { put_str(dict, "group", syn_id2name((*cur).mit_hlg_id)) };
        unsafe { put_nr(dict, "priority", (*cur).mit_priority as VarNumber) };
        unsafe { put_nr(dict, "id", (*cur).mit_id as VarNumber) };

        if unsafe { (*cur).mit_conceal_char } != 0 {
            let mut buf = [0 as c_char; MB_MAXCHAR + 1];
            let len = unsafe { utf_char2bytes((*cur).mit_conceal_char, buf.as_mut_ptr()) };
            buf[len as usize] = 0;
            unsafe { put_str(dict, "conceal", buf.as_ptr()) };
        }

        unsafe { tv_list_append_dict(l, Some(dict_held)) };
        cur = unsafe { (*cur).mit_next };
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
    // SAFETY: the evaluator's slots.
    let win = get_optional_window(args, 1);

    result.write_number(-1);
    if args[0].v_type() != VAR_LIST {
        emsg(gettext(e_listreq));
        return;
    }
    let Some(win) = win else {
        return;
    };
    let l = args[0].list_or_null();

    // To some extent make sure this really came from getmatches().
    for (li_idx, li) in tv_list_iter(unsafe { l.as_ref() }).enumerate() {
        let tv = &li.li_tv;
        if tv.v_type() != VAR_DICT || tv.dict_or_null().is_null() {
            semsg!(
                "E474: List item {} is either not a dictionary or an empty one",
                li_idx
            );
            return;
        }
        let d = tv.dict_or_null();
        let ok = !unsafe { find(d, "group") }.is_null()
            && (!unsafe { find(d, "pattern") }.is_null() || !unsafe { find(d, "pos1") }.is_null())
            && !unsafe { find(d, "priority") }.is_null()
            && !unsafe { find(d, "id") }.is_null();
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
    // SAFETY: a live list, or NULL, which reads as empty.
    while at < unsafe { tv_list_items(l) }.len() {
        let d = unsafe { tv_list_items(l) }[at].li_tv.dict_or_null();

        // A match with no `pattern` is a position match: collect
        // pos1..pos8 into the list `match_add` wants.
        let mut held = None;
        if unsafe { find(d, "pattern") }.is_null() {
            held = Some(tv_list_alloc(MAX_SAVED_POS as ptrdiff_t + 1));
            let positions = held.as_ref().expect("just built").as_ptr();
            for i in 1..MAX_SAVED_POS + 1 {
                let key = format!("pos{i}");
                let pos_di =
                    unsafe { tv_dict_find(d, key.as_ptr().cast(), key.len() as ptrdiff_t) };
                if pos_di.is_null() {
                    break;
                }
                if unsafe { (*pos_di).di_tv.v_type() } != VAR_LIST {
                    // The earlier entries stay restored, as upstream's do:
                    // the validation above does not look inside a `posN`
                    // key. Upstream *also* leaked the position list here --
                    // it took a reference per entry and gave one back --
                    // which the handle no longer permits.
                    return;
                }
                unsafe { tv_list_append_tv(positions, &(*pos_di).di_tv) };
            }
        }

        // Three scratches are in play here — this one and the two the
        // frame lends below — and none may be reused before its value is.
        let group = unsafe { group_buf.dict_string(d, c"group".as_ptr()) };
        let priority = unsafe { tv_dict_get_number(d, c"priority".as_ptr()) } as c_int;
        let id = unsafe { tv_dict_get_number(d, c"id".as_ptr()) } as c_int;
        let conceal_di = unsafe { find(d, "conceal") };
        let conceal = if conceal_di.is_null() {
            ::core::ptr::null()
        } else {
            unsafe { numbuf.string(&(*conceal_di).di_tv) }
        };

        let positions = held
            .as_ref()
            .map_or(::core::ptr::null_mut(), ListRef::as_ptr);
        let added = if positions.is_null() {
            let pattern = unsafe { numbuf2.dict_string(d, c"pattern".as_ptr()) };
            let no_pos = ::core::ptr::null_mut();
            // SAFETY: the caller's window and the arguments checked above.
            unsafe { match_add(win, group, pattern, priority, id, no_pos, conceal) }
        } else {
            let no_pat = ::core::ptr::null();
            // SAFETY: as above, with the positions list instead of a pattern.
            unsafe { match_add(win, group, no_pat, priority, id, positions, conceal) }
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
/// `matchaddpos()` share, read from `argvars[2..=4]`.
///
/// Answers `None` once something has been diagnosed.
///
/// # Safety
/// The evaluator's argument slots.
unsafe fn optional_args(
    args: &[TypVal],
    numbuf: &mut NumBuf,
) -> Option<(c_int, c_int, *const c_char, Win)> {
    let mut prio = DEFAULT_PRIORITY;
    let mut id = -1;
    let mut conceal_char: *const c_char = ::core::ptr::null();
    let mut win = Win::current();
    let mut error = false;

    // Nested, not sequential: an `id` is only read when a `priority` was
    // given, and the dictionary only when an `id` was.
    if args.len() > 2 {
        prio = unsafe { tv_get_number_chk(&args[2], &raw mut error) } as c_int;
        if args.len() > 3 {
            id = unsafe { tv_get_number_chk(&args[3], &raw mut error) } as c_int;
            if args.len() > 4
                && unsafe { matchadd_dict_arg(&args[4], &raw mut conceal_char, &mut win, numbuf) }
                    .is_err()
            {
                return None;
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
    // The group, the pattern and the `conceal` option are all held at once,
    // so each is given a scratch of its own.
    let mut grpbuf = NumBuf::new();
    let mut patbuf = NumBuf::new();
    let mut concealbuf = NumBuf::new();
    // SAFETY: the evaluator's slots.
    let grp = unsafe { grpbuf.string_chk(&args[0]) };
    let pat = unsafe { patbuf.string_chk(&args[1]) };

    result.write_number(-1);
    if grp.is_null() || pat.is_null() {
        return;
    }
    let Some((prio, id, conceal_char, win)) = (unsafe { optional_args(args, &mut concealbuf) })
    else {
        return;
    };
    if (1..=3).contains(&id) {
        semsg!("E798: ID is reserved for \":match\": {}", id);
        return;
    }

    let no_pos = ::core::ptr::null_mut();
    let added = unsafe { match_add(win, grp, pat, prio, id, no_pos, conceal_char) };
    result.write_number(added as VarNumber);
}

/// `matchaddpos(group, positions [, priority [, id [, options]]])`.
pub(crate) fn f_matchaddpos(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut buf = NumBuf::new();
    let mut concealbuf = NumBuf::new();
    // SAFETY: the evaluator's slots.
    result.write_number(-1);

    let group = unsafe { buf.string_chk(&args[0]) };
    if group.is_null() {
        return;
    }
    if args[1].v_type() != VAR_LIST {
        semsg!("E686: Argument of {} must be a List", "matchaddpos()");
        return;
    }
    let l = args[1].list_or_null();
    if unsafe { tv_list_len(l) } == 0 {
        return;
    }

    let Some((prio, id, conceal_char, win)) = (unsafe { optional_args(args, &mut concealbuf) })
    else {
        return;
    };
    // 3 is allowed: matchaddpos() is meant to stand in for `:3match`.
    if id == 1 || id == 2 {
        semsg!("E798: ID is reserved for \"match\": {}", id);
        return;
    }

    let no_pat = ::core::ptr::null();
    let added = unsafe { match_add(win, group, no_pat, prio, id, l, conceal_char) };
    result.write_number(added as VarNumber);
}

/// `matcharg(id)` — the `[group, pattern]` of `:match`, `:2match` or
/// `:3match`, or `["", ""]` when that one is not set.
pub(crate) fn f_matcharg(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    let id = tv_get_number(&args[0]) as c_int;
    let is_excmd = (1..=3).contains(&id);
    // Any other id answers an empty list, not an error.
    let l = tv_list_alloc_ret(result, if is_excmd { 2 } else { 0 });
    if !is_excmd {
        return;
    }
    let m = unsafe { get_match(Win::current(), id) };
    if m.is_null() {
        unsafe { tv_list_append_string(l, ::core::ptr::null(), 0) };
        unsafe { tv_list_append_string(l, ::core::ptr::null(), 0) };
    } else {
        unsafe { tv_list_append_string(l, syn_id2name((*m).mit_hlg_id), -1) };
        unsafe { tv_list_append_string(l, (*m).mit_pattern, -1) };
    }
}

/// `matchdelete(id [, win])`.
pub(crate) fn f_matchdelete(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    let win = get_optional_window(args, 1);
    let deleted = match win {
        None => -1,
        Some(win) => {
            let id = tv_get_number(&args[0]) as c_int;
            match_delete(win, id, true) as VarNumber
        }
    };
    result.write_number(deleted);
}
