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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::narrow::number_as_col;
use crate::narrow::number_as_lnum;
use crate::regexp::{char_delim, skip_regexp_at};
use crate::semsg;
use core::ffi::{CStr, c_int};

use crate::ascii::ascii_iswhite;
use crate::drawscreen::state::SEARCH_HL_HAS_CURSOR_LNUM;
use crate::drawscreen::{UPD_SOME_VALID, UPD_VALID, redraw_later, redraw_win_range_later};
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::funcs::get_optional_window;
use crate::eval::typval::{
    dict_find, dict_get_number, index_of, list_items, list_iter, list_len, tv_dict_alloc,
    tv_get_number, tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret,
};
use crate::eval::window::find_win_by_nr_or_id;
use crate::ex_docmd::{ends_excmd, ex_errmsg, set_no_hlsearch};
use crate::getchar::state::GOT_INT;
use crate::highlight::win_hl_attr;
use crate::highlight_group::{
    HLF_L, HLF_LC, syn_check_group, syn_id2attr, syn_id2name, syn_name2id,
};
use crate::mbyte::{char_at, encode_char};
use crate::memory::{ThinCString, XString};
use crate::message::emsg;
use crate::message::state::CALLED_EMSG;
use crate::message::{E_DICTREQ, E_INVALWINDOW, E_INVCMD, E_LISTREQ, E_TRAILING_ARG};
use crate::message_fmt::msg_bytes;
use crate::option::vars::p_rdt;
use crate::os::cshim::gettext;
use crate::os::libc::ptrdiff_t;
use crate::profile::{profile_passed_limit, profile_setlimit};
use crate::regexp::RegMMatch;
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regexec_multi};
use crate::search::state::{SEARCH_FIRST_LINE, SEARCH_LAST_LINE};
use crate::types::{
    ColNr, ExArg, Failed, LLPos, LineNr, List, MatchItem, MatchState, TypVal, VAR_LIST, VAR_NUMBER,
    VarNumber, Window,
};
use crate::winlayer::Win;

mod error;
use self::error::MatchError;
mod searchhl;
pub(crate) use self::searchhl::*;
mod vimscript;
pub(crate) use self::vimscript::*;

/// `matchadd()`'s and `:match`'s default priority.
const DEFAULT_PRIORITY: c_int = 10;

impl Win {
    /// Adds a match to `self`'s match list, and answers its id.
    ///
    /// Exactly one of `pat` and `pos_list` describes what to highlight: a
    /// pattern, or a list of `[lnum]` / `[lnum, col]` / `[lnum, col, len]`
    /// positions. `id` of `-1` allocates the next free one.
    fn add_match(
        mut self,
        grp: &CStr,
        pat: Option<&CStr>,
        prio: c_int,
        id: c_int,
        pos_list: Option<&List>,
        conceal_char: Option<&CStr>,
    ) -> Result<c_int, MatchError> {
        let mut id = id;
        let mut rtype = UPD_SOME_VALID;

        if grp.is_empty() || pat.is_some_and(CStr::is_empty) {
            return Err(MatchError::Empty);
        }
        if id < -1 || id == 0 {
            return Err(MatchError::InvalidId(id));
        }
        if id == -1 {
            id = self.w_next_match_id;
            self.w_next_match_id += 1;
        } else {
            if self.w_matches.iter().any(|item| item.mit_id == id) {
                return Err(MatchError::IdTaken(id));
            }
            // Keep the auto-allocated ids above every hand-picked one, with
            // room for a few more to be picked soon.
            if self.w_next_match_id < id + 100 {
                self.w_next_match_id = id + 100;
            }
        }

        let hlg_id = syn_check_group(grp.to_bytes());
        if hlg_id == 0 {
            // The group's name was refused, which `syn_check_group` said.
            return Err(MatchError::Reported(Failed));
        }
        let mut regprog = None;
        if let Some(pat) = pat {
            regprog = vim_regcomp(pat, RE_MAGIC);
            if regprog.is_none() {
                return Err(MatchError::Pattern(ThinCString::from_cstr(pat)));
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
            mit_toplnum: LineNr::ZERO,
            mit_botlnum: LineNr::ZERO,
            mit_hl: MatchState {
                conceals: hlg_id == syn_name2id(c"Conceal"),
                ..MatchState::default()
            },
            mit_hlg_id: hlg_id,
            mit_conceal_char: conceal_char.map_or(0, |c| char_at(c.to_bytes()).get()),
            mit_suspended: false,
        };

        if let Some(pos_list) = pos_list {
            item.mit_pos_array = vec![LLPos::default(); list_items(Some(pos_list)).len()];
            let (toplnum, botlnum) = fill_pos_array(&mut item.mit_pos_array, pos_list)?;
            if toplnum != LineNr::ZERO {
                redraw_win_range_later(self, toplnum, botlnum);
                item.mit_toplnum = toplnum;
                item.mit_botlnum = botlnum;
                rtype = UPD_VALID;
            }
        }

        // Insert into the list, which is in ascending priority order — so a
        // new match goes *after* every existing one of equal priority.
        let at = self
            .w_matches
            .iter()
            .position(|item| prio < item.mit_priority)
            .unwrap_or(self.w_matches.len());
        self.w_matches.insert(at, item);

        redraw_later(self, rtype);
        Ok(id)
    }
}

/// Fills a match's position array from a `matchaddpos()` list.
///
/// Answers the `(toplnum, botlnum)` redraw range, or the malformed entry's
/// refusal. A *rejected but not fatal* entry — a
/// non-positive line, a negative column or length — is skipped without
/// taking a slot, which is why the array can end up with unused slots at
/// its end (a zero line number ends it).
fn fill_pos_array(
    positions: &mut [LLPos],
    pos_list: &List,
) -> Result<(LineNr, LineNr), MatchError> {
    let mut toplnum: LineNr = LineNr::ZERO;
    let mut botlnum: LineNr = LineNr::ZERO;
    let mut i = 0;

    for (at, li) in list_iter(Some(pos_list)).enumerate() {
        let at = index_of(at);
        let tv = &li.li_tv;
        let mut lnum: LineNr = LineNr::ZERO;
        let mut col: ColNr = ColNr::ZERO;
        let mut len: c_int = 1;
        let mut skip = false;

        if tv.v_type() == VAR_LIST {
            let sub = list_items(tv.list_ref());
            if sub.is_empty() {
                return Err(MatchError::EmptyPosition(at));
            }
            let first = tv_get_number_chk(&sub[0].li_tv)?;
            lnum = number_as_lnum(first);
            if lnum <= LineNr::ZERO {
                skip = true;
            } else {
                positions[i].lnum = lnum;
                if let Some(second) = sub.get(1) {
                    let second = tv_get_number_chk(&second.li_tv)?;
                    col = number_as_col(second);
                    if col < ColNr::ZERO {
                        skip = true;
                    } else if let Some(third) = sub.get(2) {
                        let read = tv_get_number_chk(&third.li_tv);
                        len = number_as_col(read.unwrap_or(0)).get();
                        // Note the order: a negative length is skipped
                        // before the failure is even looked at.
                        if len < 0 {
                            skip = true;
                        } else {
                            read?;
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
                lnum = number_as_lnum(tv.number_or_zero());
                positions[i] = LLPos {
                    lnum,
                    col: ColNr::ZERO,
                    len: 0,
                };
            }
        } else {
            return Err(MatchError::NotAPosition(at));
        }

        if !skip {
            if toplnum == LineNr::ZERO || lnum < toplnum {
                toplnum = lnum;
            }
            if botlnum == LineNr::ZERO || lnum >= botlnum {
                botlnum = lnum + 1;
            }
            i += 1;
        }
    }
    Ok((toplnum, botlnum))
}

/// Removes the match `id` from `window`'s list, or says why it cannot.
fn match_delete(mut window: Win, id: c_int) -> Result<(), MatchError> {
    let mut rtype = UPD_SOME_VALID;

    if id < 1 {
        return Err(MatchError::DeleteInvalidId(id));
    }

    let Some(at) = window.w_matches.iter().position(|item| item.mit_id == id) else {
        return Err(MatchError::NoSuchId(id));
    };
    let item = window.w_matches.remove(at);
    if item.mit_toplnum != LineNr::ZERO {
        redraw_win_range_later(window, item.mit_toplnum, item.mit_botlnum);
        rtype = UPD_VALID;
    }
    drop(item);
    redraw_later(window, rtype);
    Ok(())
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
    if excmd.number2 > 3 {
        emsg(E_INVCMD);
        return;
    }
    let id = excmd.number2;
    let skip = excmd.skip;

    // Whatever happens next, the old pattern for this id goes -- if there
    // is one: its absence is not worth a word.
    if !skip {
        let _ = match_delete(Win::current(), id);
    }

    let line = &excmd.line;
    let arg = line.arg;
    let end;
    if ends_excmd(line.byte_at(arg)) {
        // `:match` on its own: just clear.
        end = arg;
    } else if line.slice_at(arg, 4).eq_ignore_ascii_case(b"none")
        && (ascii_iswhite(c_int::from(line.byte_at(arg + 4))) || ends_excmd(line.byte_at(arg + 4)))
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
            if line.byte_at(end) != 0 && !ends_excmd(line.byte_at(line.skip_white(end + 1))) {
                excmd.errmsg = Some(ex_errmsg(E_TRAILING_ARG, line.cstr_from(end)));
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
            if let Err(err) = win.add_match(&group, Some(&pat), DEFAULT_PRIORITY, id, None, None) {
                err.report();
            }
        }
    }
    excmd.line.next = excmd.line.find_next(end);
}
