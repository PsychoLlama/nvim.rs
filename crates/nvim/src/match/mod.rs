//! Match highlighting: the per-window match list.
//!
//! A *match* is a pattern (or a list of positions) that a window paints in a
//! highlight group, independently of `'hlsearch'`. Matches live in a
//! priority-ordered singly-linked list hanging off `Window::w_match_head`,
//! high priority first, and are addressed by an id: 1, 2 and 3 belong to the
//! `:match`, `:2match` and `:3match` commands, everything above to
//! `matchadd()` and `matchaddpos()`.
//!
//! The drawing side lives in [`searchhl`], the `match*()` Vimscript
//! functions in [`vimscript`].

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::semsg;
use core::ffi::{c_char, c_int};

use crate::ascii::ascii_iswhite;
use crate::charset::{skiptowhite, skipwhite};
use crate::drawscreen::state::search_hl_has_cursor_lnum;
use crate::drawscreen::{UPD_SOME_VALID, UPD_VALID, redraw_later, redraw_win_range_later};
use crate::eval::funcs::get_optional_window;
use crate::eval::typval::{
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find,
    tv_dict_get_number, tv_get_number, tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_dict, tv_list_append_number, tv_list_append_string, tv_list_append_tv,
    tv_list_first, tv_list_idx_of_item, tv_list_len, tv_list_ref, tv_list_unref,
};
use crate::eval::window::find_win_by_nr_or_id;
use crate::ex_docmd::{ends_excmd, ex_errmsg, find_nextcmd, set_no_hlsearch};
use crate::fold::has_folding;
use crate::getchar::state::got_int;
use crate::highlight::win_hl_attr;
use crate::highlight_group::{
    HLF_L, HLF_LC, syn_check_group, syn_id2attr, syn_id2name, syn_name2id,
};
use crate::mbyte::{utf_char2bytes, utf_ptr2char, utfc_ptr2len};
use crate::memline::ml_get_buf;
use crate::memory::{xcalloc, xfree, xmemdupz, xstrdup};
use crate::message::emsg;
use crate::message::state::called_emsg;
use crate::message::{e_dictreq, e_invalwindow, e_invcmd, e_listreq, e_trailing_arg};
use crate::message_fmt::c_str;
use crate::option::vars::p_rdt;
use crate::os::cshim::{gettext, strncasecmp};
use crate::profile::{profile_passed_limit, profile_setlimit};
use crate::regexp::{RE_MAGIC, skip_regexp, vim_regcomp, vim_regexec_multi, vim_regfree};
use crate::search::state::{search_first_line, search_last_line};
use crate::types::{
    ColNr, Dict, DictItem, EvalFuncData, ExArg, LLPos, LineNr, List, MatchItem, MatchState,
    RegProg, TypVal, VAR_LIST, VAR_NUMBER, VarNumber, int64_t, ptrdiff_t, size_t, uint8_t,
};
use crate::winlayer::{Live, Win};

mod searchhl;
pub(crate) use self::searchhl::*;
mod vimscript;
pub(crate) use self::vimscript::*;

use crate::regexp::re_multiline;

/// The highlight state of one matcher — a window's `'hlsearch'` state or one
/// match's — whose holder has promised it outlives the value.
///
/// The `'hlsearch'` one is a `static` in `drawscreen`; a match's is a field of
/// the `MatchItem` the window's list owns, so both live as long as the
/// redraw that reads them.
pub(crate) type Shl = Live<MatchState>;

/// One entry of a window's match list, on [`Shl`]'s terms: the window owns it
/// until `:call matchdelete()` frees it.
pub(crate) type Mi = Live<MatchItem>;

/// `matchadd()`'s and `:match`'s default priority.
const DEFAULT_PRIORITY: c_int = 10;

/// Adds a match to `window`'s match list, and answers its id or `-1`.
///
/// Exactly one of `pat` and `pos_list` describes what to highlight: a
/// pattern, or a list of `[lnum]` / `[lnum, col]` / `[lnum, col, len]`
/// positions. `id` of `-1` allocates the next free one.
///
/// # Safety
/// `window` must be live and `grp` NUL-terminated; `pat` and `conceal_char` must
/// be null or NUL-terminated; `pos_list` must be null or a live list.
#[allow(clippy::too_many_arguments)]
unsafe fn match_add(
    mut window: Win,
    grp: *const c_char,
    pat: *const c_char,
    prio: c_int,
    id: c_int,
    pos_list: *mut List,
    conceal_char: *const c_char,
) -> c_int {
    // SAFETY: the caller's window, strings and list.
    let mut id = id;
    let mut rtype = UPD_SOME_VALID;

    if unsafe { *grp } == 0 || (!pat.is_null() && unsafe { *pat } == 0) {
        return -1;
    }
    if id < -1 || id == 0 {
        semsg!(
            "E799: Invalid ID: {} (must be greater than or equal to 1)",
            id as int64_t
        );
        return -1;
    }
    if id == -1 {
        id = window.w_next_match_id;
        window.w_next_match_id += 1;
    } else {
        let mut cur = window.w_match_head;
        while !cur.is_null() {
            if unsafe { (*cur).mit_id } == id {
                semsg!("E801: ID already taken: {}", id as int64_t);
                return -1;
            }
            cur = unsafe { (*cur).mit_next };
        }
        // Keep the auto-allocated ids above every hand-picked one, with
        // room for a few more to be picked soon.
        if window.w_next_match_id < id + 100 {
            window.w_next_match_id = id + 100;
        }
    }

    let hlg_id = unsafe { syn_check_group(grp, cstr::bytes_at(grp).len()) };
    if hlg_id == 0 {
        return -1;
    }
    let mut regprog: *mut RegProg = ::core::ptr::null_mut();
    if !pat.is_null() {
        regprog = unsafe { vim_regcomp(pat, RE_MAGIC) };
        if regprog.is_null() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let pat = unsafe { c_str(pat) };
            semsg!("E475: Invalid argument: {pat}");
            return -1;
        }
    }

    // SAFETY: a fresh allocation, live until this frame hands it to the
    // window's match list.
    let mut m =
        unsafe { Mi::new(xcalloc(1, ::core::mem::size_of::<MatchItem>()).cast::<MatchItem>()) };
    if unsafe { tv_list_len(pos_list) } > 0 {
        unsafe {
            m.mit_pos_array = xcalloc(
                tv_list_len(pos_list) as size_t,
                ::core::mem::size_of::<LLPos>(),
            )
            .cast::<LLPos>()
        };
        unsafe { m.mit_pos_count = tv_list_len(pos_list) };
    }
    m.mit_id = id;
    m.mit_priority = prio;
    unsafe {
        m.mit_pattern = if pat.is_null() {
            ::core::ptr::null_mut()
        } else {
            xstrdup(pat)
        }
    };
    m.mit_hlg_id = hlg_id;
    m.mit_match.regprog = regprog;
    m.mit_match.rmm_ic = 0;
    m.mit_match.rmm_maxcol = 0;
    unsafe {
        m.mit_conceal_char = if conceal_char.is_null() {
            0
        } else {
            utf_ptr2char(conceal_char)
        }
    };

    if !pos_list.is_null() {
        match unsafe { fill_pos_array(m.raw(), pos_list) } {
            Some((toplnum, botlnum)) if toplnum != 0 => {
                redraw_win_range_later(window, toplnum, botlnum);
                m.mit_toplnum = toplnum;
                m.mit_botlnum = botlnum;
                rtype = UPD_VALID;
            }
            Some(_) => {}
            None => {
                unsafe { vim_regfree(regprog) };
                unsafe { xfree(m.mit_pattern.cast()) };
                unsafe { xfree(m.mit_pos_array.cast()) };
                unsafe { xfree(m.raw().cast()) };
                return -1;
            }
        }
    }

    // Insert into the list, which is in ascending priority order — so a
    // new match goes *after* every existing one of equal priority.
    let mut cur = window.w_match_head;
    let mut prev = cur;
    while !cur.is_null() && prio >= unsafe { (*cur).mit_priority } {
        prev = cur;
        cur = unsafe { (*cur).mit_next };
    }
    if cur == prev {
        window.w_match_head = m.raw();
    } else {
        unsafe { (*prev).mit_next = m.raw() };
    }
    m.mit_next = cur;

    redraw_later(window, rtype);
    id
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
unsafe fn fill_pos_array(m: *mut MatchItem, pos_list: *mut List) -> Option<(LineNr, LineNr)> {
    // SAFETY: the caller's promise -- see this function's `# Safety`.
    let m = unsafe { Mi::new(m) };
    // SAFETY: the caller's match and list.
    let mut toplnum: LineNr = 0;
    let mut botlnum: LineNr = 0;
    let mut i = 0;

    let mut li = unsafe { tv_list_first(pos_list) };
    while !li.is_null() {
        let tv = unsafe { &raw mut (*li).li_tv };
        let mut lnum: LineNr = 0;
        let mut col: ColNr = 0;
        let mut len: c_int = 1;
        let mut error = false;
        let mut skip = false;

        if unsafe { (*tv).v_type } == VAR_LIST {
            let subl = unsafe { (*tv).list_or_null() };
            let mut subli = unsafe { tv_list_first(subl) };
            if subli.is_null() {
                // SAFETY: `pos_list` holds `li`, as the caller established.
                let at = unsafe { tv_list_idx_of_item(pos_list, li) };
                semsg!("E5030: Empty list at position {at}");
                return None;
            }
            lnum =
                unsafe { tv_get_number_chk(&raw const (*subli).li_tv, &raw mut error) } as LineNr;
            if error {
                return None;
            }
            if lnum <= 0 {
                skip = true;
            } else {
                unsafe { (*m.mit_pos_array.offset(i as isize)).lnum = lnum };
                subli = unsafe { (*subli).li_next };
                if !subli.is_null() {
                    col = unsafe { tv_get_number_chk(&raw const (*subli).li_tv, &raw mut error) }
                        as ColNr;
                    if error {
                        return None;
                    }
                    if col < 0 {
                        skip = true;
                    } else {
                        subli = unsafe { (*subli).li_next };
                        if !subli.is_null() {
                            len = unsafe {
                                tv_get_number_chk(&raw const (*subli).li_tv, &raw mut error)
                            } as ColNr;
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
                    unsafe { (*m.mit_pos_array.offset(i as isize)).col = col };
                    unsafe { (*m.mit_pos_array.offset(i as isize)).len = len };
                }
            }
        } else if unsafe { (*tv).v_type } == VAR_NUMBER {
            if unsafe { (*tv).number_or_zero() } <= 0 {
                skip = true;
            } else {
                lnum = unsafe { (*tv).number_or_zero() } as LineNr;
                unsafe { (*m.mit_pos_array.offset(i as isize)).lnum = lnum };
                unsafe { (*m.mit_pos_array.offset(i as isize)).col = 0 };
                unsafe { (*m.mit_pos_array.offset(i as isize)).len = 0 };
            }
        } else {
            // SAFETY: `pos_list` holds `li`, as the caller established.
            let at = unsafe { tv_list_idx_of_item(pos_list, li) };
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
        li = unsafe { (*li).li_next };
    }
    Some((toplnum, botlnum))
}

/// Removes the match `id` from `window`'s list; `-1` when there is no such match.
///
/// # Safety
/// `window` must be live.
unsafe fn match_delete(mut window: Win, id: c_int, perr: bool) -> c_int {
    let mut rtype = UPD_SOME_VALID;

    if id < 1 {
        if perr {
            semsg!(
                "E802: Invalid ID: {} (must be greater than or equal to 1)",
                id as int64_t
            );
        }
        return -1;
    }

    let mut cur = window.w_match_head;
    let mut prev = cur;
    while !cur.is_null() && unsafe { (*cur).mit_id } != id {
        prev = cur;
        cur = unsafe { (*cur).mit_next };
    }
    if cur.is_null() {
        if perr {
            semsg!("E803: ID not found: {}", id as int64_t);
        }
        return -1;
    }

    if cur == prev {
        unsafe { window.w_match_head = (*cur).mit_next };
    } else {
        unsafe { (*prev).mit_next = (*cur).mit_next };
    }
    unsafe { vim_regfree((*cur).mit_match.regprog) };
    unsafe { xfree((*cur).mit_pattern.cast()) };
    if unsafe { (*cur).mit_toplnum } != 0 {
        unsafe { redraw_win_range_later(window, (*cur).mit_toplnum, (*cur).mit_botlnum) };
        rtype = UPD_VALID;
    }
    unsafe { xfree((*cur).mit_pos_array.cast()) };
    unsafe { xfree(cur.cast()) };
    redraw_later(window, rtype);
    0
}

/// Removes every match from `window`'s list.
///
/// # Safety
/// `window` must be live.
pub(crate) unsafe fn clear_matches(mut window: Win) {
    // SAFETY: the caller's promise -- see this function's `# Safety`.
    // SAFETY: the caller's window.
    while !window.w_match_head.is_null() {
        // SAFETY: the window owns every entry of its list until it is
        // unlinked, which is what the line below does.
        let m = unsafe { Mi::new(window.w_match_head) };
        window.w_match_head = m.mit_next;
        unsafe { vim_regfree(m.mit_match.regprog) };
        unsafe { xfree(m.mit_pattern.cast()) };
        unsafe { xfree(m.mit_pos_array.cast()) };
        unsafe { xfree(m.raw().cast()) };
    }
    redraw_later(window, UPD_SOME_VALID);
}

/// The match `id` in `window`'s list, or null.
///
/// # Safety
/// `window` must be live.
unsafe fn get_match(window: Win, id: c_int) -> *mut MatchItem {
    let mut cur = window.w_match_head;
    while !cur.is_null() && unsafe { (*cur).mit_id } != id {
        cur = unsafe { (*cur).mit_next };
    }
    cur
}

/// `:[N]match {group} {pattern}`, `:[N]match none` and `:[N]match`.
///
/// Also runs while commands are being *skipped* (inside a false `:if`), in
/// which case nothing is added and only `args.nextcmd` is set — which is why
/// it has to parse the pattern either way.
///
/// # Safety
/// `args` must be a live Ex-command argument block with a writable `arg`.
pub(crate) unsafe fn ex_match(args: *mut ExArg) {
    // SAFETY: the caller's command.
    // The command's count is the match id: `:match`, `:2match`, `:3match`.
    if unsafe { (*args).line2 } > 3 {
        emsg(e_invcmd);
        return;
    }
    let id = unsafe { (*args).line2 } as c_int;
    let skip = unsafe { (*args).skip } != 0;

    // Whatever happens next, the old pattern for this id goes.
    if !skip {
        unsafe { match_delete(Win::current(), id, false) };
    }

    let arg = unsafe { (*args).arg };
    let end;
    if ends_excmd(unsafe { *arg } as c_int) != 0 {
        // `:match` on its own: just clear.
        end = arg;
    } else if unsafe { strncasecmp(arg, c"none".as_ptr(), 4) } == 0
        && (ascii_iswhite(unsafe { *arg.offset(4) } as c_int)
            || ends_excmd(unsafe { *arg.offset(4) } as c_int) != 0)
    {
        end = unsafe { arg.offset(4) };
    } else {
        let mut p = unsafe { skiptowhite(arg) };
        // The group name, up to the first whitespace.
        let g = if skip {
            ::core::ptr::null_mut()
        } else {
            unsafe { xmemdupz(arg.cast(), p.offset_from(arg) as size_t) }.cast::<c_char>()
        };
        p = unsafe { skipwhite(p) };
        if unsafe { *p } == 0 {
            // There must be two arguments.
            unsafe { xfree(g.cast()) };
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str(arg) };
            semsg!("E475: Invalid argument: {arg}");
            return;
        }
        // `*p` is the pattern's delimiter, whatever character it is.
        end = unsafe { skip_regexp(p.offset(1), *p as c_int, 1) };
        if !skip {
            if unsafe { *end } != 0
                && ends_excmd(unsafe { *skipwhite(end.offset(1)) } as c_int) == 0
            {
                unsafe { xfree(g.cast()) };
                unsafe { (*args).errmsg = Some(ex_errmsg(e_trailing_arg.as_ptr(), end)) };
                return;
            }
            if unsafe { *end } != unsafe { *p } {
                // The closing delimiter is missing.
                unsafe { xfree(g.cast()) };
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let p = unsafe { c_str(p) };
                semsg!("E475: Invalid argument: {p}");
                return;
            }
            // Terminate the pattern in place for the compile, then put
            // the delimiter back so `find_nextcmd` sees the whole line.
            let c = unsafe { *end } as uint8_t;
            unsafe { *end = 0 };
            // SAFETY: the pattern is NUL-terminated in place just above.
            let pat = unsafe { p.offset(1) };
            let win = Win::current();
            let no_pos = ::core::ptr::null_mut();
            let no_conceal = ::core::ptr::null();
            // SAFETY: the editor's own window and the group checked above.
            unsafe { match_add(win, g, pat, DEFAULT_PRIORITY, id, no_pos, no_conceal) };
            unsafe { xfree(g.cast()) };
            unsafe { *end = c as c_char };
        }
    }
    unsafe { (*args).nextcmd = find_nextcmd(end) };
}
