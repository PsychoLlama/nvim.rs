//! Match highlighting: the per-window match list.
//!
//! A *match* is a pattern (or a list of positions) that a window paints in a
//! highlight group, independently of `'hlsearch'`. Matches live in a
//! priority-ordered singly-linked list hanging off `win_T::w_match_head`,
//! high priority first, and are addressed by an id: 1, 2 and 3 belong to the
//! `:match`, `:2match` and `:3match` commands, everything above to
//! `matchadd()` and `matchaddpos()`.
//!
//! The drawing side lives in [`searchhl`], the `match*()` Vimscript
//! functions in [`vimscript`].

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int};

use crate::ascii::ascii_iswhite;
use crate::charset::{skiptowhite, skipwhite};
use crate::drawscreen::{UPD_SOME_VALID, UPD_VALID, redraw_later, redraw_win_range_later};
use crate::eval::funcs::get_optional_window;
use crate::eval::typval::{
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find,
    tv_dict_get_number, tv_dict_get_string, tv_dict_get_string_buf, tv_get_number,
    tv_get_number_chk, tv_get_string, tv_get_string_buf_chk, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_dict, tv_list_append_number, tv_list_append_string, tv_list_append_tv,
    tv_list_first, tv_list_idx_of_item, tv_list_len, tv_list_ref, tv_list_unref,
};
use crate::eval::window::find_win_by_nr_or_id;
use crate::ex_docmd::{ends_excmd, ex_errmsg, find_nextcmd, set_no_hlsearch};
use crate::fold::hasFolding;
use crate::highlight::win_hl_attr;
use crate::highlight_group::{
    HLF_L, HLF_LC, syn_check_group, syn_id2attr, syn_id2name, syn_name2id,
};
use crate::main::{
    called_emsg, curwin, e_dictreq, e_invalwindow, e_invarg2, e_invcmd, e_listarg, e_listreq,
    e_trailing_arg, got_int, p_cpo, p_rdt, search_first_line, search_hl_has_cursor_lnum,
    search_last_line,
};
use crate::mbyte::{utf_char2bytes, utf_ptr2char, utfc_ptr2len};
use crate::memline::ml_get_buf;
use crate::memory::{xcalloc, xfree, xmemdupz, xstrdup};
use crate::message::emsg;
use crate::os::cshim::{gettext, strncasecmp};
use crate::profile::{profile_passed_limit, profile_setlimit};
use crate::regexp::{RE_MAGIC, skip_regexp, vim_regcomp, vim_regexec_multi, vim_regfree};
use crate::strings::vim_strchr;
use crate::types::{
    EvalFuncData, VAR_LIST, VAR_NUMBER, colnr_T, dict_T, dictitem_T, exarg_T, int64_t, linenr_T,
    list_T, llpos_T, match_T, matchitem_T, ptrdiff_t, regprog_T, size_t, typval_T, uint8_t,
    varnumber_T, win_T,
};
use ::libc::strlen;

mod searchhl;
pub use self::searchhl::*;
mod vimscript;
pub use self::vimscript::*;

use crate::regexp::re_multiline;

pub const NUL: c_int = '\0' as c_int;
/// The `'cpoptions'` flag that makes a search continue at the end of the
/// previous match rather than one character past its start.
pub const CPO_SEARCH: c_int = 'c' as c_int;

/// The scratch buffer `tv_get_string_buf_chk` needs to render a non-string
/// argument into.
pub(crate) const NUMBUFLEN: usize = 65;

/// `matchadd()`'s and `:match`'s default priority.
const DEFAULT_PRIORITY: c_int = 10;

/// Adds a match to `wp`'s match list, and answers its id or `-1`.
///
/// Exactly one of `pat` and `pos_list` describes what to highlight: a
/// pattern, or a list of `[lnum]` / `[lnum, col]` / `[lnum, col, len]`
/// positions. `id` of `-1` allocates the next free one.
///
/// # Safety
/// `wp` must be live and `grp` NUL-terminated; `pat` and `conceal_char` must
/// be null or NUL-terminated; `pos_list` must be null or a live list.
#[allow(clippy::too_many_arguments)]
unsafe fn match_add(
    wp: *mut win_T,
    grp: *const c_char,
    pat: *const c_char,
    prio: c_int,
    id: c_int,
    pos_list: *mut list_T,
    conceal_char: *const c_char,
) -> c_int {
    // SAFETY: the caller's window, strings and list.
    unsafe {
        let mut id = id;
        let mut rtype = UPD_SOME_VALID;

        if *grp == 0 || (!pat.is_null() && *pat == 0) {
            return -1;
        }
        if id < -1 || id == 0 {
            semsg_c!(
                gettext(c"E799: Invalid ID: %ld (must be greater than or equal to 1)".as_ptr()),
                id as int64_t,
            );
            return -1;
        }
        if id == -1 {
            id = (*wp).w_next_match_id;
            (*wp).w_next_match_id += 1;
        } else {
            let mut cur = (*wp).w_match_head;
            while !cur.is_null() {
                if (*cur).mit_id == id {
                    semsg_c!(
                        gettext(c"E801: ID already taken: %ld".as_ptr()),
                        id as int64_t,
                    );
                    return -1;
                }
                cur = (*cur).mit_next;
            }
            // Keep the auto-allocated ids above every hand-picked one, with
            // room for a few more to be picked soon.
            if (*wp).w_next_match_id < id + 100 {
                (*wp).w_next_match_id = id + 100;
            }
        }

        let hlg_id = syn_check_group(grp, strlen(grp));
        if hlg_id == 0 {
            return -1;
        }
        let mut regprog: *mut regprog_T = ::core::ptr::null_mut();
        if !pat.is_null() {
            regprog = vim_regcomp(pat, RE_MAGIC);
            if regprog.is_null() {
                semsg_c!(gettext(&raw const e_invarg2 as *const c_char), pat);
                return -1;
            }
        }

        let m: *mut matchitem_T =
            xcalloc(1, ::core::mem::size_of::<matchitem_T>()).cast::<matchitem_T>();
        if tv_list_len(pos_list) > 0 {
            (*m).mit_pos_array = xcalloc(
                tv_list_len(pos_list) as size_t,
                ::core::mem::size_of::<llpos_T>(),
            )
            .cast::<llpos_T>();
            (*m).mit_pos_count = tv_list_len(pos_list);
        }
        (*m).mit_id = id;
        (*m).mit_priority = prio;
        (*m).mit_pattern = if pat.is_null() {
            ::core::ptr::null_mut()
        } else {
            xstrdup(pat)
        };
        (*m).mit_hlg_id = hlg_id;
        (*m).mit_match.regprog = regprog;
        (*m).mit_match.rmm_ic = 0;
        (*m).mit_match.rmm_maxcol = 0;
        (*m).mit_conceal_char = if conceal_char.is_null() {
            0
        } else {
            utf_ptr2char(conceal_char)
        };

        if !pos_list.is_null() {
            match fill_pos_array(m, pos_list) {
                Some((toplnum, botlnum)) if toplnum != 0 => {
                    redraw_win_range_later(wp, toplnum, botlnum);
                    (*m).mit_toplnum = toplnum;
                    (*m).mit_botlnum = botlnum;
                    rtype = UPD_VALID;
                }
                Some(_) => {}
                None => {
                    vim_regfree(regprog);
                    xfree((*m).mit_pattern.cast());
                    xfree((*m).mit_pos_array.cast());
                    xfree(m.cast());
                    return -1;
                }
            }
        }

        // Insert into the list, which is in ascending priority order — so a
        // new match goes *after* every existing one of equal priority.
        let mut cur = (*wp).w_match_head;
        let mut prev = cur;
        while !cur.is_null() && prio >= (*cur).mit_priority {
            prev = cur;
            cur = (*cur).mit_next;
        }
        if cur == prev {
            (*wp).w_match_head = m;
        } else {
            (*prev).mit_next = m;
        }
        (*m).mit_next = cur;

        redraw_later(wp, rtype);
        id
    }
}

/// Fills `m`'s position array from a `matchaddpos()` list.
///
/// Answers the `(toplnum, botlnum)` redraw range, or `None` after
/// diagnosing a malformed entry. A *rejected but not fatal* entry — a
/// non-positive line, a negative column or length — is skipped without
/// taking a slot, which is why the array can end up shorter than the list.
///
/// # Safety
/// `m` must be live with `mit_pos_array` sized for the list, and `pos_list`
/// must be a live list.
unsafe fn fill_pos_array(
    m: *mut matchitem_T,
    pos_list: *mut list_T,
) -> Option<(linenr_T, linenr_T)> {
    // SAFETY: the caller's match and list.
    unsafe {
        let mut toplnum: linenr_T = 0;
        let mut botlnum: linenr_T = 0;
        let mut i = 0;

        let mut li = tv_list_first(pos_list);
        while !li.is_null() {
            let tv = &raw mut (*li).li_tv;
            let mut lnum: linenr_T = 0;
            let mut col: colnr_T = 0;
            let mut len: c_int = 1;
            let mut error = false;
            let mut skip = false;

            if (*tv).v_type == VAR_LIST {
                let subl = (*tv).vval.v_list;
                let mut subli = tv_list_first(subl);
                if subli.is_null() {
                    semsg_c!(
                        gettext(c"E5030: Empty list at position %d".as_ptr()),
                        tv_list_idx_of_item(pos_list, li),
                    );
                    return None;
                }
                lnum = tv_get_number_chk(&raw const (*subli).li_tv, &raw mut error) as linenr_T;
                if error {
                    return None;
                }
                if lnum <= 0 {
                    skip = true;
                } else {
                    (*(*m).mit_pos_array.offset(i as isize)).lnum = lnum;
                    subli = (*subli).li_next;
                    if !subli.is_null() {
                        col =
                            tv_get_number_chk(&raw const (*subli).li_tv, &raw mut error) as colnr_T;
                        if error {
                            return None;
                        }
                        if col < 0 {
                            skip = true;
                        } else {
                            subli = (*subli).li_next;
                            if !subli.is_null() {
                                len = tv_get_number_chk(&raw const (*subli).li_tv, &raw mut error)
                                    as colnr_T;
                                // Note the order: a negative length is
                                // skipped before `error` is even looked at.
                                if len < 0 {
                                    skip = true;
                                } else if error {
                                    return None;
                                }
                            }
                        }
                    }
                    if !skip {
                        (*(*m).mit_pos_array.offset(i as isize)).col = col;
                        (*(*m).mit_pos_array.offset(i as isize)).len = len;
                    }
                }
            } else if (*tv).v_type == VAR_NUMBER {
                if (*tv).vval.v_number <= 0 {
                    skip = true;
                } else {
                    lnum = (*tv).vval.v_number as linenr_T;
                    (*(*m).mit_pos_array.offset(i as isize)).lnum = lnum;
                    (*(*m).mit_pos_array.offset(i as isize)).col = 0;
                    (*(*m).mit_pos_array.offset(i as isize)).len = 0;
                }
            } else {
                semsg_c!(
                    gettext(c"E5031: List or number required at position %d".as_ptr()),
                    tv_list_idx_of_item(pos_list, li),
                );
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
            li = (*li).li_next;
        }
        Some((toplnum, botlnum))
    }
}

/// Removes the match `id` from `wp`'s list; `-1` when there is no such match.
///
/// # Safety
/// `wp` must be live.
unsafe fn match_delete(wp: *mut win_T, id: c_int, perr: bool) -> c_int {
    // SAFETY: the caller's window.
    unsafe {
        let mut rtype = UPD_SOME_VALID;

        if id < 1 {
            if perr {
                semsg_c!(
                    gettext(c"E802: Invalid ID: %ld (must be greater than or equal to 1)".as_ptr()),
                    id as int64_t,
                );
            }
            return -1;
        }

        let mut cur = (*wp).w_match_head;
        let mut prev = cur;
        while !cur.is_null() && (*cur).mit_id != id {
            prev = cur;
            cur = (*cur).mit_next;
        }
        if cur.is_null() {
            if perr {
                semsg_c!(gettext(c"E803: ID not found: %ld".as_ptr()), id as int64_t);
            }
            return -1;
        }

        if cur == prev {
            (*wp).w_match_head = (*cur).mit_next;
        } else {
            (*prev).mit_next = (*cur).mit_next;
        }
        vim_regfree((*cur).mit_match.regprog);
        xfree((*cur).mit_pattern.cast());
        if (*cur).mit_toplnum != 0 {
            redraw_win_range_later(wp, (*cur).mit_toplnum, (*cur).mit_botlnum);
            rtype = UPD_VALID;
        }
        xfree((*cur).mit_pos_array.cast());
        xfree(cur.cast());
        redraw_later(wp, rtype);
        0
    }
}

/// Removes every match from `wp`'s list.
///
/// # Safety
/// `wp` must be live.
pub unsafe fn clear_matches(wp: *mut win_T) {
    // SAFETY: the caller's window.
    unsafe {
        while !(*wp).w_match_head.is_null() {
            let m = (*wp).w_match_head;
            (*wp).w_match_head = (*m).mit_next;
            vim_regfree((*m).mit_match.regprog);
            xfree((*m).mit_pattern.cast());
            xfree((*m).mit_pos_array.cast());
            xfree(m.cast());
        }
        redraw_later(wp, UPD_SOME_VALID);
    }
}

/// The match `id` in `wp`'s list, or null.
///
/// # Safety
/// `wp` must be live.
unsafe fn get_match(wp: *mut win_T, id: c_int) -> *mut matchitem_T {
    // SAFETY: the caller's window.
    unsafe {
        let mut cur = (*wp).w_match_head;
        while !cur.is_null() && (*cur).mit_id != id {
            cur = (*cur).mit_next;
        }
        cur
    }
}

/// `:[N]match {group} {pattern}`, `:[N]match none` and `:[N]match`.
///
/// Also runs while commands are being *skipped* (inside a false `:if`), in
/// which case nothing is added and only `eap->nextcmd` is set — which is why
/// it has to parse the pattern either way.
///
/// # Safety
/// `eap` must be a live Ex-command argument block with a writable `arg`.
pub unsafe fn ex_match(eap: *mut exarg_T) {
    // SAFETY: the caller's command.
    unsafe {
        // The command's count is the match id: `:match`, `:2match`, `:3match`.
        if (*eap).line2 > 3 {
            emsg(&raw const e_invcmd as *const c_char);
            return;
        }
        let id = (*eap).line2 as c_int;
        let skip = (*eap).skip != 0;

        // Whatever happens next, the old pattern for this id goes.
        if !skip {
            match_delete(curwin.get(), id, false);
        }

        let arg = (*eap).arg;
        let end;
        if ends_excmd(*arg as c_int) != 0 {
            // `:match` on its own: just clear.
            end = arg;
        } else if strncasecmp(arg, c"none".as_ptr(), 4) == 0
            && (ascii_iswhite(*arg.offset(4) as c_int) || ends_excmd(*arg.offset(4) as c_int) != 0)
        {
            end = arg.offset(4);
        } else {
            let mut p = skiptowhite(arg);
            // The group name, up to the first whitespace.
            let g = if skip {
                ::core::ptr::null_mut()
            } else {
                xmemdupz(arg.cast(), p.offset_from(arg) as size_t).cast::<c_char>()
            };
            p = skipwhite(p);
            if *p == 0 {
                // There must be two arguments.
                xfree(g.cast());
                semsg_c!(gettext(&raw const e_invarg2 as *const c_char), arg);
                return;
            }
            // `*p` is the pattern's delimiter, whatever character it is.
            end = skip_regexp(p.offset(1), *p as c_int, 1);
            if !skip {
                if *end != 0 && ends_excmd(*skipwhite(end.offset(1)) as c_int) == 0 {
                    xfree(g.cast());
                    (*eap).errmsg = ex_errmsg(&raw const e_trailing_arg as *const c_char, end);
                    return;
                }
                if *end != *p {
                    // The closing delimiter is missing.
                    xfree(g.cast());
                    semsg_c!(gettext(&raw const e_invarg2 as *const c_char), p);
                    return;
                }
                // Terminate the pattern in place for the compile, then put
                // the delimiter back so `find_nextcmd` sees the whole line.
                let c = *end as uint8_t;
                *end = 0;
                match_add(
                    curwin.get(),
                    g,
                    p.offset(1),
                    DEFAULT_PRIORITY,
                    id,
                    ::core::ptr::null_mut(),
                    ::core::ptr::null(),
                );
                xfree(g.cast());
                *end = c as c_char;
            }
        }
        (*eap).nextcmd = find_nextcmd(end);
    }
}
