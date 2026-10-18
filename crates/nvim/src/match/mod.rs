//! Match highlighting: the per-window match list.
//!
//! A *match* is a pattern (or a list of positions) that a window paints in a
//! highlight group, independently of `'hlsearch'`. Matches live in a list
//! the window owns, `Window::w_matches`, in ascending priority order (ties in
//! the order they were added), and are addressed by an id: 1, 2 and 3 belong to the
//! `:match`, `:2match` and `:3match` commands, everything above to
//! `matchadd()` and `matchaddpos()`.
//!
//! The drawing side lives in [`searchhl`], the `match*()` Vimscript
//! functions in [`vimscript`].

#![forbid(unsafe_code)]

use crate::cstr;
use crate::regexp::{char_delim, skip_regexp_at};
use crate::semsg;
use core::ffi::{CStr, c_int};

use crate::ascii::ascii_iswhite;
use crate::drawscreen::state::search_hl_has_cursor_lnum;
use crate::drawscreen::{UPD_SOME_VALID, UPD_VALID, redraw_later, redraw_win_range_later};
use crate::eval::funcs::get_optional_window;
use crate::eval::typval::{
    dict_find, dict_get_number, index_of, list_items, list_iter, list_len, tv_dict_alloc,
    tv_get_number, tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret,
};
use crate::eval::window::find_win_by_nr_or_id;
use crate::ex_docmd::{ends_excmd, ex_errmsg, set_no_hlsearch};
use crate::fold::has_folding;
use crate::getchar::state::got_int;
use crate::highlight::win_hl_attr;
use crate::highlight_group::{
    HLF_L, HLF_LC, syn_check_group, syn_id2attr, syn_id2name, syn_name2id,
};
use crate::mbyte::{char_at, encode_char};
use crate::memory::XString;
use crate::message::emsg;
use crate::message::state::called_emsg;
use crate::message::{e_dictreq, e_invalwindow, e_invcmd, e_listreq, e_trailing_arg};
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::option::vars::p_rdt;
use crate::os::cshim::gettext;
use crate::profile::{profile_passed_limit, profile_setlimit};
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regexec_multi};
use crate::search::state::{search_first_line, search_last_line};
use crate::types::{
    ColNr, EvalFuncData, ExArg, LLPos, LineNr, List, MatchItem, MatchState, RegMMatch, TypVal,
    VAR_LIST, VAR_NUMBER, VarNumber, Window, int64_t, ptrdiff_t,
};
use crate::winlayer::Win;

mod searchhl;
pub(crate) use self::searchhl::*;
mod vimscript;
pub(crate) use self::vimscript::*;

/// `matchadd()`'s and `:match`'s default priority.
const DEFAULT_PRIORITY: c_int = 10;

/// Adds a match to `window`'s match list, and answers its id or `-1`.
///
/// Exactly one of `pat` and `pos_list` describes what to highlight: a
/// pattern, or a list of `[lnum]` / `[lnum, col]` / `[lnum, col, len]`
/// positions. `id` of `-1` allocates the next free one.
fn match_add(
    mut window: Win,
    grp: &CStr,
    pat: Option<&CStr>,
    prio: c_int,
    id: c_int,
    pos_list: Option<&List>,
    conceal_char: Option<&CStr>,
) -> c_int {
    let mut id = id;
    let mut rtype = UPD_SOME_VALID;

    if grp.is_empty() || pat.is_some_and(CStr::is_empty) {
        return -1;
    }
    if id < -1 || id == 0 {
        semsg!(
            "E799: Invalid ID: {} (must be greater than or equal to 1)",
            int64_t::from(id)
        );
        return -1;
    }
    if id == -1 {
        id = window.w_next_match_id;
        window.w_next_match_id += 1;
    } else {
        if window.w_matches.iter().any(|item| item.mit_id == id) {
            semsg!("E801: ID already taken: {}", int64_t::from(id));
            return -1;
        }
        // Keep the auto-allocated ids above every hand-picked one, with
        // room for a few more to be picked soon.
        if window.w_next_match_id < id + 100 {
            window.w_next_match_id = id + 100;
        }
    }

    let hlg_id = syn_check_group(grp.to_bytes());
    if hlg_id == 0 {
        return -1;
    }
    let mut regprog = None;
    if let Some(pat) = pat {
        regprog = vim_regcomp(pat, RE_MAGIC);
        if regprog.is_none() {
            let pat = msg_cstr(pat);
            semsg!("E475: Invalid argument: {pat}");
            return -1;
        }
    }

    let mut item = MatchItem {
        mit_id: id,
        mit_priority: prio,
        mit_pattern: pat.map(XString::from_cstr),
        mit_match: RegMMatch {
            regprog,
            ..RegMMatch::default()
        },
        mit_pos_array: Vec::new(),
        mit_pos_cur: 0,
        mit_toplnum: 0,
        mit_botlnum: 0,
        mit_hl: MatchState {
            conceals: hlg_id == syn_name2id(c"Conceal"),
            ..MatchState::default()
        },
        mit_hlg_id: hlg_id,
        mit_conceal_char: conceal_char.map_or(0, |c| char_at(c.to_bytes())),
        mit_suspended: false,
    };

    if let Some(pos_list) = pos_list {
        item.mit_pos_array = vec![LLPos::default(); list_items(Some(pos_list)).len()];
        match fill_pos_array(&mut item.mit_pos_array, pos_list) {
            Some((toplnum, botlnum)) if toplnum != 0 => {
                redraw_win_range_later(window, toplnum, botlnum);
                item.mit_toplnum = toplnum;
                item.mit_botlnum = botlnum;
                rtype = UPD_VALID;
            }
            Some(_) => {}
            None => return -1,
        }
    }

    // Insert into the list, which is in ascending priority order — so a
    // new match goes *after* every existing one of equal priority.
    let at = window
        .w_matches
        .iter()
        .position(|item| prio < item.mit_priority)
        .unwrap_or(window.w_matches.len());
    window.w_matches.insert(at, item);

    redraw_later(window, rtype);
    id
}

/// Fills a match's position array from a `matchaddpos()` list.
///
/// Answers the `(toplnum, botlnum)` redraw range, or `None` after
/// diagnosing a malformed entry. A *rejected but not fatal* entry — a
/// non-positive line, a negative column or length — is skipped without
/// taking a slot, which is why the array can end up with unused slots at
/// its end (a zero line number ends it).
fn fill_pos_array(positions: &mut [LLPos], pos_list: &List) -> Option<(LineNr, LineNr)> {
    let mut toplnum: LineNr = 0;
    let mut botlnum: LineNr = 0;
    let mut i = 0;

    for (at, li) in list_iter(Some(pos_list)).enumerate() {
        let at = index_of(at);
        let tv = &li.li_tv;
        let mut lnum: LineNr = 0;
        let mut col: ColNr = 0;
        let mut len: c_int = 1;
        let mut skip = false;

        if tv.v_type() == VAR_LIST {
            let sub = list_items(tv.list_ref());
            if sub.is_empty() {
                semsg!("E5030: Empty list at position {at}");
                return None;
            }
            let Ok(first) = tv_get_number_chk(&sub[0].li_tv) else {
                return None;
            };
            lnum = first as LineNr;
            if lnum <= 0 {
                skip = true;
            } else {
                positions[i].lnum = lnum;
                if let Some(second) = sub.get(1) {
                    let Ok(second) = tv_get_number_chk(&second.li_tv) else {
                        return None;
                    };
                    col = second as ColNr;
                    if col < 0 {
                        skip = true;
                    } else if let Some(third) = sub.get(2) {
                        let read = tv_get_number_chk(&third.li_tv);
                        len = read.unwrap_or(0) as ColNr;
                        // Note the order: a negative length is skipped
                        // before the failure is even looked at.
                        if len < 0 {
                            skip = true;
                        } else if read.is_err() {
                            return None;
                        }
                    }
                }
                if !skip {
                    positions[i].col = col;
                    positions[i].len = len;
                }
            }
        } else if tv.v_type() == VAR_NUMBER {
            if tv.number_or_zero() <= 0 {
                skip = true;
            } else {
                lnum = tv.number_or_zero() as LineNr;
                positions[i] = LLPos {
                    lnum,
                    col: 0,
                    len: 0,
                };
            }
        } else {
            semsg!("E5031: List or number required at position {at}");
            return None;
        }

        if !skip {
            if toplnum == 0 || lnum < toplnum {
                toplnum = lnum;
            }
            if botlnum == 0 || lnum >= botlnum {
                botlnum = lnum + 1;
            }
            i += 1;
        }
    }
    Some((toplnum, botlnum))
}

/// Removes the match `id` from `window`'s list; `-1` when there is no such match.
fn match_delete(mut window: Win, id: c_int, perr: bool) -> c_int {
    let mut rtype = UPD_SOME_VALID;

    if id < 1 {
        if perr {
            semsg!(
                "E802: Invalid ID: {} (must be greater than or equal to 1)",
                int64_t::from(id)
            );
        }
        return -1;
    }

    let Some(at) = window.w_matches.iter().position(|item| item.mit_id == id) else {
        if perr {
            semsg!("E803: ID not found: {}", int64_t::from(id));
        }
        return -1;
    };
    let item = window.w_matches.remove(at);
    if item.mit_toplnum != 0 {
        redraw_win_range_later(window, item.mit_toplnum, item.mit_botlnum);
        rtype = UPD_VALID;
    }
    drop(item);
    redraw_later(window, rtype);
    0
}

/// Removes every match from `window`'s list, and the list's own storage
/// with it: a window's memory is released without dropping its fields.
pub(crate) fn clear_matches(mut window: Win) {
    drop(core::mem::take(&mut window.w_matches));
    redraw_later(window, UPD_SOME_VALID);
}

/// The match `id` in `window`'s list.
fn get_match(window: &Window, id: c_int) -> Option<&MatchItem> {
    window.w_matches.iter().find(|item| item.mit_id == id)
}

/// `:[N]match {group} {pattern}`, `:[N]match none` and `:[N]match`.
///
/// Also runs while commands are being *skipped* (inside a false `:if`), in
/// which case nothing is added and only `args.nextcmd` is set — which is why
/// it has to parse the pattern either way.
pub(crate) fn ex_match(excmd: &mut ExArg) {
    // SAFETY: the caller's command.
    // The command's count is the match id: `:match`, `:2match`, `:3match`.
    if excmd.line2 > 3 {
        emsg(e_invcmd);
        return;
    }
    let id = excmd.line2 as c_int;
    let skip = excmd.skip;

    // Whatever happens next, the old pattern for this id goes.
    if !skip {
        match_delete(Win::current(), id, false);
    }

    let line = &excmd.line;
    let arg = line.arg;
    let end;
    if ends_excmd(c_int::from(line.byte_at(arg))) != 0 {
        // `:match` on its own: just clear.
        end = arg;
    } else if line.slice_at(arg, 4).eq_ignore_ascii_case(b"none")
        && (ascii_iswhite(c_int::from(line.byte_at(arg + 4)))
            || ends_excmd(c_int::from(line.byte_at(arg + 4))) != 0)
    {
        end = arg + 4;
    } else {
        // The group name, up to the first whitespace.
        let group_end = line.skip_to_white(arg);
        let group = (!skip).then(|| cstr::owned(line.slice_at(arg, group_end - arg)));
        let p = line.skip_white(group_end);
        if line.byte_at(p) == 0 {
            // There must be two arguments.
            let arg = msg_bytes(line.rest_of(arg));
            semsg!("E475: Invalid argument: {arg}");
            return;
        }
        // The pattern's delimiter is whatever character comes first.
        let delim = line.byte_at(p);
        end = p + 1 + skip_regexp_at(line.tail(p + 1), char_delim(delim), 1);
        if let Some(group) = group {
            if line.byte_at(end) != 0
                && ends_excmd(c_int::from(line.byte_at(line.skip_white(end + 1)))) == 0
            {
                excmd.errmsg = Some(ex_errmsg(e_trailing_arg, line.cstr_from(end)));
                return;
            }
            if line.byte_at(end) != delim {
                // The closing delimiter is missing.
                let p = msg_bytes(line.rest_of(p));
                semsg!("E475: Invalid argument: {p}");
                return;
            }
            let pat = cstr::owned(line.slice_at(p + 1, end - (p + 1)));
            let win = Win::current();
            match_add(win, &group, Some(&pat), DEFAULT_PRIORITY, id, None, None);
        }
    }
    excmd.line.next = excmd.line.find_next(end);
}
