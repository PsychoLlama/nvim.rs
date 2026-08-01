use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::charset::{skiptowhite, skipwhite};
use crate::src::nvim::drawscreen::{
    UPD_SOME_VALID, UPD_VALID, redraw_later, redraw_win_range_later,
};
use crate::src::nvim::eval::funcs::get_optional_window;
use crate::src::nvim::eval::typval::{
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find,
    tv_dict_get_number, tv_dict_get_string, tv_dict_get_string_buf, tv_get_number,
    tv_get_number_chk, tv_get_string, tv_get_string_buf_chk, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_dict, tv_list_append_number, tv_list_append_string, tv_list_append_tv,
    tv_list_idx_of_item, tv_list_unref,
};
use crate::src::nvim::eval::typval::{tv_list_first, tv_list_len, tv_list_ref};
use crate::src::nvim::eval::window::find_win_by_nr_or_id;
use crate::src::nvim::ex_docmd::{ends_excmd, ex_errmsg, find_nextcmd, set_no_hlsearch};
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::highlight::win_hl_attr;

use crate::src::nvim::highlight_group::{
    HLF_L, HLF_LC, syn_check_group, syn_id2attr, syn_id2name, syn_name2id,
};
use crate::src::nvim::main::{
    called_emsg, curwin, e_dictreq, e_invalwindow, e_invarg2, e_invcmd, e_listarg, e_listreq,
    e_trailing_arg, got_int, p_cpo, p_rdt, search_first_line, search_hl_has_cursor_lnum,
    search_last_line,
};
use crate::src::nvim::mbyte::{utf_char2bytes, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::ml_get_buf;
use crate::src::nvim::memory::{xcalloc, xfree, xmemdupz, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::libc::{__assert_fail, gettext, snprintf, strlen, strncasecmp};
use crate::src::nvim::profile::{profile_passed_limit, profile_setlimit};
use crate::src::nvim::regexp::skip_regexp;
use crate::src::nvim::regexp::vim_regexec_multi;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    EvalFuncData, ListLenSpecials, VarType, colnr_T, dict_T, dictitem_T, exarg_T, int64_t,
    linenr_T, list_T, listitem_T, llpos_T, match_T, matchitem_T, ptrdiff_t, regprog_T, size_t,
    ssize_t, typval_T, uint8_t, varnumber_T, win_T,
};
unsafe extern "C" {
    fn re_multiline(prog: *const regprog_T) -> ::core::ffi::c_int;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed = 2147483647;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 56] = unsafe {
    ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
        *b"void f_getmatches(typval_T *, typval_T *, EvalFuncData)\0",
    )
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_SEARCH: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
pub const SEARCH_HL_PRIORITY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn match_add(
    mut wp: *mut win_T,
    grp: *const ::core::ffi::c_char,
    pat: *const ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
    mut id: ::core::ffi::c_int,
    mut pos_list: *mut list_T,
    conceal_char: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut cur_0: *mut matchitem_T = ::core::ptr::null_mut::<matchitem_T>();
    let mut prev: *mut matchitem_T = ::core::ptr::null_mut::<matchitem_T>();
    let mut hlg_id: ::core::ffi::c_int = 0;
    let mut regprog: *mut regprog_T = ::core::ptr::null_mut::<regprog_T>();
    let mut rtype: ::core::ffi::c_int = UPD_SOME_VALID;
    if *grp as ::core::ffi::c_int == NUL || !pat.is_null() && *pat as ::core::ffi::c_int == NUL {
        return -1 as ::core::ffi::c_int;
    }
    if id < -1 as ::core::ffi::c_int || id == 0 as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E799: Invalid ID: %ld (must be greater than or equal to 1)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            id as int64_t,
        );
        return -1 as ::core::ffi::c_int;
    }
    if id == -1 as ::core::ffi::c_int {
        let c2rust_fresh0 = (*wp).w_next_match_id;
        (*wp).w_next_match_id = (*wp).w_next_match_id + 1;
        id = c2rust_fresh0;
    } else {
        let mut cur: *mut matchitem_T = (*wp).w_match_head;
        while !cur.is_null() {
            if (*cur).mit_id == id {
                semsg(
                    gettext(b"E801: ID already taken: %ld\0".as_ptr() as *const ::core::ffi::c_char),
                    id as int64_t,
                );
                return -1 as ::core::ffi::c_int;
            }
            cur = (*cur).mit_next;
        }
        if (*wp).w_next_match_id < id + 100 as ::core::ffi::c_int {
            (*wp).w_next_match_id = id + 100 as ::core::ffi::c_int;
        }
    }
    hlg_id = syn_check_group(grp, strlen(grp));
    if hlg_id == 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if !pat.is_null() && {
        regprog = vim_regcomp(pat, RE_MAGIC);
        regprog.is_null()
    } {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            pat,
        );
        return -1 as ::core::ffi::c_int;
    }
    let mut m: *mut matchitem_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<matchitem_T>()) as *mut matchitem_T;
    if tv_list_len(pos_list) > 0 as ::core::ffi::c_int {
        (*m).mit_pos_array = xcalloc(
            tv_list_len(pos_list) as size_t,
            ::core::mem::size_of::<llpos_T>(),
        ) as *mut llpos_T;
        (*m).mit_pos_count = tv_list_len(pos_list);
    }
    (*m).mit_id = id;
    (*m).mit_priority = prio;
    (*m).mit_pattern = if pat.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup(pat)
    };
    (*m).mit_hlg_id = hlg_id;
    (*m).mit_match.regprog = regprog;
    (*m).mit_match.rmm_ic = false_0;
    (*m).mit_match.rmm_maxcol = 0 as ::core::ffi::c_int as colnr_T;
    (*m).mit_conceal_char = 0 as ::core::ffi::c_int;
    if !conceal_char.is_null() {
        (*m).mit_conceal_char = utf_ptr2char(conceal_char);
    }
    if !pos_list.is_null() {
        let mut toplnum: linenr_T = 0 as linenr_T;
        let mut botlnum: linenr_T = 0 as linenr_T;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let l_: *mut list_T = pos_list;
        's_369: {
            if !l_.is_null() {
                let mut li: *mut listitem_T = (*l_).lv_first;
                '_fail: loop {
                    if li.is_null() {
                        break 's_369;
                    }
                    let mut lnum: linenr_T = 0 as linenr_T;
                    let mut col: colnr_T = 0 as colnr_T;
                    let mut len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    let mut error: bool = false;
                    's_183: {
                        if (*li).li_tv.v_type as ::core::ffi::c_uint
                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            let subl: *const list_T = (*li).li_tv.vval.v_list;
                            let mut subli: *const listitem_T = tv_list_first(subl);
                            if subli.is_null() {
                                semsg(
                                    gettext(b"E5030: Empty list at position %d\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    tv_list_idx_of_item(pos_list, li),
                                );
                                break '_fail;
                            } else {
                                lnum = tv_get_number_chk(&raw const (*subli).li_tv, &raw mut error)
                                    as linenr_T;
                                if error {
                                    break '_fail;
                                }
                                if lnum <= 0 as linenr_T {
                                    break 's_183;
                                } else {
                                    (*(*m).mit_pos_array.offset(i as isize)).lnum = lnum;
                                    subli = (*subli).li_next;
                                    if !subli.is_null() {
                                        col = tv_get_number_chk(
                                            &raw const (*subli).li_tv,
                                            &raw mut error,
                                        ) as colnr_T;
                                        if error {
                                            break '_fail;
                                        }
                                        if col < 0 as ::core::ffi::c_int {
                                            break 's_183;
                                        } else {
                                            subli = (*subli).li_next;
                                            if !subli.is_null() {
                                                len = tv_get_number_chk(
                                                    &raw const (*subli).li_tv,
                                                    &raw mut error,
                                                )
                                                    as colnr_T
                                                    as ::core::ffi::c_int;
                                                if len < 0 as ::core::ffi::c_int {
                                                    break 's_183;
                                                } else if error {
                                                    break '_fail;
                                                }
                                            }
                                        }
                                    }
                                    (*(*m).mit_pos_array.offset(i as isize)).col = col;
                                    (*(*m).mit_pos_array.offset(i as isize)).len = len;
                                }
                            }
                        } else if (*li).li_tv.v_type as ::core::ffi::c_uint
                            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if (*li).li_tv.vval.v_number <= 0 as varnumber_T {
                                break 's_183;
                            } else {
                                (*(*m).mit_pos_array.offset(i as isize)).lnum =
                                    (*li).li_tv.vval.v_number as linenr_T;
                                (*(*m).mit_pos_array.offset(i as isize)).col =
                                    0 as ::core::ffi::c_int as colnr_T;
                                (*(*m).mit_pos_array.offset(i as isize)).len =
                                    0 as ::core::ffi::c_int;
                            }
                        } else {
                            semsg(
                                gettext(
                                    b"E5031: List or number required at position %d\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ),
                                tv_list_idx_of_item(pos_list, li),
                            );
                            break '_fail;
                        }
                        if toplnum == 0 as linenr_T || lnum < toplnum {
                            toplnum = lnum;
                        }
                        if botlnum == 0 as linenr_T || lnum >= botlnum {
                            botlnum = lnum + 1 as linenr_T;
                        }
                        i += 1;
                    }
                    li = (*li).li_next;
                }
                vim_regfree(regprog);
                xfree((*m).mit_pattern as *mut ::core::ffi::c_void);
                xfree((*m).mit_pos_array as *mut ::core::ffi::c_void);
                xfree(m as *mut ::core::ffi::c_void);
                return -1 as ::core::ffi::c_int;
            }
        }
        if toplnum != 0 as linenr_T {
            redraw_win_range_later(wp, toplnum, botlnum);
            (*m).mit_toplnum = toplnum;
            (*m).mit_botlnum = botlnum;
            rtype = UPD_VALID;
        }
    }
    cur_0 = (*wp).w_match_head;
    prev = cur_0;
    while !cur_0.is_null() && prio >= (*cur_0).mit_priority {
        prev = cur_0;
        cur_0 = (*cur_0).mit_next;
    }
    if cur_0 == prev {
        (*wp).w_match_head = m;
    } else {
        (*prev).mit_next = m;
    }
    (*m).mit_next = cur_0;
    redraw_later(wp, rtype);
    return id;
}
unsafe extern "C" fn match_delete(
    mut wp: *mut win_T,
    mut id: ::core::ffi::c_int,
    mut perr: bool,
) -> ::core::ffi::c_int {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut prev: *mut matchitem_T = cur;
    let mut rtype: ::core::ffi::c_int = UPD_SOME_VALID;
    if id < 1 as ::core::ffi::c_int {
        if perr {
            semsg(
                gettext(
                    b"E802: Invalid ID: %ld (must be greater than or equal to 1)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                id as int64_t,
            );
        }
        return -1 as ::core::ffi::c_int;
    }
    while !cur.is_null() && (*cur).mit_id != id {
        prev = cur;
        cur = (*cur).mit_next;
    }
    if cur.is_null() {
        if perr {
            semsg(
                gettext(b"E803: ID not found: %ld\0".as_ptr() as *const ::core::ffi::c_char),
                id as int64_t,
            );
        }
        return -1 as ::core::ffi::c_int;
    }
    if cur == prev {
        (*wp).w_match_head = (*cur).mit_next;
    } else {
        (*prev).mit_next = (*cur).mit_next;
    }
    vim_regfree((*cur).mit_match.regprog);
    xfree((*cur).mit_pattern as *mut ::core::ffi::c_void);
    if (*cur).mit_toplnum != 0 as linenr_T {
        redraw_win_range_later(wp, (*cur).mit_toplnum, (*cur).mit_botlnum);
        rtype = UPD_VALID;
    }
    xfree((*cur).mit_pos_array as *mut ::core::ffi::c_void);
    xfree(cur as *mut ::core::ffi::c_void);
    redraw_later(wp, rtype);
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn clear_matches(mut wp: *mut win_T) {
    while !(*wp).w_match_head.is_null() {
        let mut m: *mut matchitem_T = (*(*wp).w_match_head).mit_next;
        vim_regfree((*(*wp).w_match_head).mit_match.regprog);
        xfree((*(*wp).w_match_head).mit_pattern as *mut ::core::ffi::c_void);
        xfree((*(*wp).w_match_head).mit_pos_array as *mut ::core::ffi::c_void);
        xfree((*wp).w_match_head as *mut ::core::ffi::c_void);
        (*wp).w_match_head = m;
    }
    redraw_later(wp, UPD_SOME_VALID);
}
unsafe extern "C" fn get_match(mut wp: *mut win_T, mut id: ::core::ffi::c_int) -> *mut matchitem_T {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    while !cur.is_null() && (*cur).mit_id != id {
        cur = (*cur).mit_next;
    }
    return cur;
}
pub unsafe extern "C" fn init_search_hl(mut wp: *mut win_T, mut search_hl: *mut match_T) {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    while !cur.is_null() {
        (*cur).mit_hl.rm = (*cur).mit_match;
        if (*cur).mit_hlg_id == 0 as ::core::ffi::c_int {
            (*cur).mit_hl.attr = 0 as ::core::ffi::c_int;
        } else {
            (*cur).mit_hl.attr = syn_id2attr((*cur).mit_hlg_id);
        }
        (*cur).mit_hl.buf = (*wp).w_buffer;
        (*cur).mit_hl.lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*cur).mit_hl.first_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*cur).mit_hl.tm = profile_setlimit(p_rdt.get() as int64_t);
        cur = (*cur).mit_next;
    }
    (*search_hl).buf = (*wp).w_buffer;
    (*search_hl).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*search_hl).first_lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*search_hl).attr = win_hl_attr(wp, HLF_L);
}
unsafe extern "C" fn next_search_hl_pos(
    mut shl: *mut match_T,
    mut lnum: linenr_T,
    mut match_0: *mut matchitem_T,
    mut mincol: colnr_T,
) -> ::core::ffi::c_int {
    let mut found: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
    let mut i: ::core::ffi::c_int = (*match_0).mit_pos_cur;
    while i < (*match_0).mit_pos_count {
        let mut pos: *mut llpos_T = (*match_0).mit_pos_array.offset(i as isize);
        if (*pos).lnum == 0 as linenr_T {
            break;
        }
        if !((*pos).len == 0 as ::core::ffi::c_int && (*pos).col < mincol) {
            if (*pos).lnum == lnum {
                if found >= 0 as ::core::ffi::c_int {
                    if (*pos).col < (*(*match_0).mit_pos_array.offset(found as isize)).col {
                        let mut tmp: llpos_T = *pos;
                        *pos = *(*match_0).mit_pos_array.offset(found as isize);
                        *(*match_0).mit_pos_array.offset(found as isize) = tmp;
                    }
                } else {
                    found = i;
                }
            }
        }
        i += 1;
    }
    (*match_0).mit_pos_cur = 0 as ::core::ffi::c_int;
    if found >= 0 as ::core::ffi::c_int {
        let mut start: colnr_T =
            if (*(*match_0).mit_pos_array.offset(found as isize)).col == 0 as ::core::ffi::c_int {
                0 as colnr_T
            } else {
                (*(*match_0).mit_pos_array.offset(found as isize)).col - 1 as colnr_T
            };
        let mut end: colnr_T =
            if (*(*match_0).mit_pos_array.offset(found as isize)).col == 0 as ::core::ffi::c_int {
                MAXCOL as ::core::ffi::c_int
            } else {
                start + (*(*match_0).mit_pos_array.offset(found as isize)).len as colnr_T
            };
        (*shl).lnum = lnum;
        (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum =
            0 as ::core::ffi::c_int as linenr_T;
        (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col = start;
        (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum =
            0 as ::core::ffi::c_int as linenr_T;
        (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col = end;
        (*shl).is_addpos = true_0 != 0;
        (*shl).has_cursor = false_0 != 0;
        (*match_0).mit_pos_cur = found + 1 as ::core::ffi::c_int;
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn next_search_hl(
    mut win: *mut win_T,
    mut search_hl: *mut match_T,
    mut shl: *mut match_T,
    mut lnum: linenr_T,
    mut mincol: colnr_T,
    mut cur: *mut matchitem_T,
) {
    let mut matchcol: colnr_T = 0;
    let mut nmatched: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    if (lnum < search_first_line.get() || lnum > search_last_line.get()) && cur.is_null() {
        (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
        return;
    }
    if (*shl).lnum != 0 as linenr_T {
        let mut l: linenr_T = (*shl).lnum + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
            - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
        if lnum > l {
            (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
        } else if lnum < l || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col > mincol {
            return;
        }
    }
    loop {
        if profile_passed_limit((*shl).tm) {
            (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
            break;
        } else {
            if (*shl).lnum == 0 as linenr_T {
                matchcol = 0 as ::core::ffi::c_int as colnr_T;
            } else if vim_strchr(p_cpo.get(), CPO_SEARCH).is_null()
                || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum == 0 as linenr_T
                    && (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col
                        <= (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col
            {
                matchcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
                let mut ml: *mut ::core::ffi::c_char =
                    ml_get_buf((*shl).buf, lnum).offset(matchcol as isize);
                if *ml as ::core::ffi::c_int == NUL {
                    matchcol += 1;
                    (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                    break;
                } else {
                    matchcol += utfc_ptr2len(ml);
                }
            } else {
                matchcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
            }
            (*shl).lnum = lnum;
            if !(*shl).rm.regprog.is_null() {
                let mut regprog_is_copy: bool = shl != search_hl
                    && !cur.is_null()
                    && shl == &raw mut (*cur).mit_hl
                    && ::core::ptr::addr_eq((*cur).mit_match.regprog, (*cur).mit_hl.rm.regprog);
                let mut timed_out: ::core::ffi::c_int = false_0;
                nmatched = vim_regexec_multi(
                    &raw mut (*shl).rm,
                    win,
                    (*shl).buf,
                    lnum,
                    matchcol,
                    &raw mut (*shl).tm,
                    &raw mut timed_out,
                );
                if regprog_is_copy {
                    (*cur).mit_match.regprog = (*cur).mit_hl.rm.regprog;
                }
                if called_emsg.get() > called_emsg_before
                    || got_int.get() as ::core::ffi::c_int != 0
                    || timed_out != 0
                {
                    if shl == search_hl {
                        vim_regfree((*shl).rm.regprog);
                        set_no_hlsearch(true_0 != 0);
                    }
                    (*shl).rm.regprog = ::core::ptr::null_mut::<regprog_T>();
                    (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                    got_int.set(false_0 != 0);
                    break;
                }
            } else if !cur.is_null() {
                nmatched = next_search_hl_pos(shl, lnum, cur, matchcol);
            }
            if nmatched == 0 as ::core::ffi::c_int {
                (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                break;
            } else {
                if !((*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum > 0 as linenr_T
                    || (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col >= mincol
                    || nmatched > 1 as ::core::ffi::c_int
                    || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col > mincol)
                {
                    continue;
                }
                (*shl).lnum += (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
                break;
            }
        }
    }
}
pub unsafe extern "C" fn prepare_search_hl(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut lnum: linenr_T,
) {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    while !cur.is_null() || shl_flag as ::core::ffi::c_int == false_0 {
        if shl_flag as ::core::ffi::c_int == false_0 {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if !(*shl).rm.regprog.is_null()
            && (*shl).lnum == 0 as linenr_T
            && re_multiline((*shl).rm.regprog) != 0
        {
            if (*shl).first_lnum == 0 as linenr_T {
                (*shl).first_lnum = lnum;
                while (*shl).first_lnum > (*wp).w_topline {
                    if hasFolding(
                        wp,
                        (*shl).first_lnum - 1 as linenr_T,
                        ::core::ptr::null_mut::<linenr_T>(),
                        ::core::ptr::null_mut::<linenr_T>(),
                    ) {
                        break;
                    }
                    (*shl).first_lnum -= 1;
                }
            }
            if !cur.is_null() {
                (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
            }
            let mut pos_inprogress: bool = true_0 != 0;
            let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while (*shl).first_lnum < lnum
                && (!(*shl).rm.regprog.is_null()
                    || !cur.is_null() && pos_inprogress as ::core::ffi::c_int != 0)
            {
                next_search_hl(
                    wp,
                    search_hl,
                    shl,
                    (*shl).first_lnum,
                    n,
                    if shl == search_hl {
                        ::core::ptr::null_mut::<matchitem_T>()
                    } else {
                        cur
                    },
                );
                pos_inprogress = !(cur.is_null() || (*cur).mit_pos_cur == 0 as ::core::ffi::c_int);
                if (*shl).lnum != 0 as linenr_T {
                    (*shl).first_lnum = (*shl).lnum
                        + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
                        - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
                    n = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col
                        as ::core::ffi::c_int;
                } else {
                    (*shl).first_lnum += 1;
                    n = 0 as ::core::ffi::c_int;
                }
            }
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
}
unsafe extern "C" fn check_cur_search_hl(mut wp: *mut win_T, mut shl: *mut match_T) {
    let mut linecount: linenr_T = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
        - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
    if (*wp).w_cursor.lnum >= (*shl).lnum
        && (*wp).w_cursor.lnum <= (*shl).lnum + linecount
        && ((*wp).w_cursor.lnum > (*shl).lnum
            || (*wp).w_cursor.col >= (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col)
        && ((*wp).w_cursor.lnum < (*shl).lnum + linecount
            || (*wp).w_cursor.col < (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col)
    {
        (*shl).has_cursor = true_0 != 0;
    } else {
        (*shl).has_cursor = false_0 != 0;
    };
}
pub unsafe extern "C" fn prepare_search_hl_line(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut mincol: colnr_T,
    mut line: *mut *mut ::core::ffi::c_char,
    mut search_hl: *mut match_T,
    mut search_attr: *mut ::core::ffi::c_int,
    mut search_attr_from_match: *mut bool,
) -> bool {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    let mut area_highlighting: bool = false_0 != 0;
    while !cur.is_null() || !shl_flag {
        if !shl_flag {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        (*shl).startcol = MAXCOL as ::core::ffi::c_int as colnr_T;
        (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
        (*shl).attr_cur = 0 as ::core::ffi::c_int;
        (*shl).is_addpos = false_0 != 0;
        (*shl).has_cursor = false_0 != 0;
        if !cur.is_null() {
            (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
        }
        next_search_hl(
            wp,
            search_hl,
            shl,
            lnum,
            mincol,
            if shl == search_hl {
                ::core::ptr::null_mut::<matchitem_T>()
            } else {
                cur
            },
        );
        *line = ml_get_buf((*wp).w_buffer, lnum);
        if (*shl).lnum != 0 as linenr_T && (*shl).lnum <= lnum {
            if (*shl).lnum == lnum {
                (*shl).startcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
            } else {
                (*shl).startcol = 0 as ::core::ffi::c_int as colnr_T;
            }
            if lnum
                == (*shl).lnum + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
                    - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum
            {
                (*shl).endcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
            } else {
                (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
            }
            if shl == search_hl {
                check_cur_search_hl(wp, shl);
            }
            if (*shl).startcol == (*shl).endcol {
                if *(*line).offset((*shl).endcol as isize) as ::core::ffi::c_int != NUL {
                    (*shl).endcol += utfc_ptr2len((*line).offset((*shl).endcol as isize));
                } else {
                    (*shl).endcol += 1;
                }
            }
            if (*shl).startcol < mincol {
                (*shl).attr_cur = (*shl).attr;
                *search_attr = (*shl).attr;
                *search_attr_from_match = shl != search_hl;
            }
            area_highlighting = true_0 != 0;
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
    return area_highlighting;
}
pub unsafe extern "C" fn update_search_hl(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut line: *mut *mut ::core::ffi::c_char,
    mut search_hl: *mut match_T,
    mut has_match_conc: *mut ::core::ffi::c_int,
    mut match_conc: *mut ::core::ffi::c_int,
    mut lcs_eol_todo: bool,
    mut on_last_col: *mut bool,
    mut search_attr_from_match: *mut bool,
) -> ::core::ffi::c_int {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    let mut search_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !cur.is_null() || !shl_flag {
        if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if !cur.is_null() {
            (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
        }
        let mut pos_inprogress: bool = true_0 != 0;
        while !(*shl).rm.regprog.is_null()
            || !cur.is_null() && pos_inprogress as ::core::ffi::c_int != 0
        {
            if (*shl).startcol != MAXCOL as ::core::ffi::c_int
                && col >= (*shl).startcol
                && col < (*shl).endcol
            {
                let mut next_col: ::core::ffi::c_int =
                    col as ::core::ffi::c_int + utfc_ptr2len((*line).offset(col as isize));
                if (*shl).endcol < next_col {
                    (*shl).endcol = next_col as colnr_T;
                }
                if shl == search_hl && (*shl).has_cursor as ::core::ffi::c_int != 0 {
                    (*shl).attr_cur = win_hl_attr(wp, HLF_LC);
                    if (*shl).attr_cur != (*shl).attr {
                        search_hl_has_cursor_lnum.set(lnum);
                    }
                } else {
                    (*shl).attr_cur = (*shl).attr;
                }
                if !cur.is_null()
                    && shl != search_hl
                    && syn_name2id(b"Conceal\0".as_ptr() as *const ::core::ffi::c_char)
                        == (*cur).mit_hlg_id
                {
                    *has_match_conc = if col == (*shl).startcol {
                        2 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    };
                    *match_conc = (*cur).mit_conceal_char;
                } else {
                    *has_match_conc = 0 as ::core::ffi::c_int;
                }
                break;
            } else {
                if col != (*shl).endcol {
                    break;
                }
                (*shl).attr_cur = 0 as ::core::ffi::c_int;
                next_search_hl(
                    wp,
                    search_hl,
                    shl,
                    lnum,
                    col,
                    if shl == search_hl {
                        ::core::ptr::null_mut::<matchitem_T>()
                    } else {
                        cur
                    },
                );
                pos_inprogress = !(cur.is_null() || (*cur).mit_pos_cur == 0 as ::core::ffi::c_int);
                *line = ml_get_buf((*wp).w_buffer, lnum);
                if (*shl).lnum != lnum {
                    break;
                }
                (*shl).startcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
                if (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum == 0 as linenr_T {
                    (*shl).endcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
                } else {
                    (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
                }
                if shl == search_hl {
                    check_cur_search_hl(wp, shl);
                }
                if (*shl).startcol == (*shl).endcol {
                    let mut p: *mut ::core::ffi::c_char = (*line).offset((*shl).endcol as isize);
                    if *p as ::core::ffi::c_int == NUL {
                        (*shl).endcol += 1;
                    } else {
                        (*shl).endcol += utfc_ptr2len(p);
                    }
                }
            }
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
    *search_attr_from_match = false_0 != 0;
    search_attr = (*search_hl).attr_cur;
    cur = (*wp).w_match_head;
    shl_flag = false_0 != 0;
    while !cur.is_null() || !shl_flag {
        if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if (*shl).attr_cur != 0 as ::core::ffi::c_int {
            search_attr = (*shl).attr_cur;
            *on_last_col = col as ::core::ffi::c_int + 1 as ::core::ffi::c_int >= (*shl).endcol;
            *search_attr_from_match = shl != search_hl;
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
    if *(*line).offset(col as isize) as ::core::ffi::c_int == NUL
        && ((*wp).w_onebuf_opt.wo_list != 0 && !lcs_eol_todo)
    {
        search_attr = 0 as ::core::ffi::c_int;
    }
    return search_attr;
}
pub unsafe extern "C" fn get_prevcol_hl_flag(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut curcol: colnr_T,
) -> bool {
    let mut prevcol: colnr_T = curcol;
    if (if (*wp).w_onebuf_opt.wo_wrap != 0 {
        (*wp).w_skipcol
    } else {
        (*wp).w_leftcol
    }) > prevcol
    {
        prevcol += 1;
    }
    if !(*search_hl).is_addpos
        && (prevcol == (*search_hl).startcol
            || prevcol > (*search_hl).startcol
                && (*search_hl).endcol == MAXCOL as ::core::ffi::c_int)
    {
        return true_0 != 0;
    }
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    while !cur.is_null() {
        if !(*cur).mit_hl.is_addpos
            && (prevcol == (*cur).mit_hl.startcol
                || prevcol > (*cur).mit_hl.startcol
                    && (*cur).mit_hl.endcol == MAXCOL as ::core::ffi::c_int)
        {
            return true_0 != 0;
        }
        cur = (*cur).mit_next;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn get_search_match_hl(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut col: colnr_T,
    mut char_attr: *mut ::core::ffi::c_int,
) {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    while !cur.is_null() || !shl_flag {
        if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if col as ::core::ffi::c_int - 1 as ::core::ffi::c_int == (*shl).startcol
            && (shl == search_hl || !(*shl).is_addpos)
        {
            *char_attr = (*shl).attr;
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
}
unsafe extern "C" fn matchadd_dict_arg(
    mut tv: *mut typval_T,
    mut conceal_char: *mut *const ::core::ffi::c_char,
    mut win: *mut *mut win_T,
) -> ::core::ffi::c_int {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    if (*tv).v_type as ::core::ffi::c_uint != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
        return FAIL;
    }
    di = tv_dict_find(
        (*tv).vval.v_dict,
        b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        *conceal_char = tv_get_string(&raw mut (*di).di_tv);
    }
    di = tv_dict_find(
        (*tv).vval.v_dict,
        b"window\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if di.is_null() {
        return OK;
    }
    *win = find_win_by_nr_or_id(&raw mut (*di).di_tv);
    if (*win).is_null() {
        emsg(gettext(
            &raw const e_invalwindow as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn f_clearmatches(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut win: *mut win_T = get_optional_window(argvars, 0 as ::core::ffi::c_int);
    if !win.is_null() {
        clear_matches(win);
    }
}
pub unsafe extern "C" fn f_getmatches(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut win: *mut win_T = get_optional_window(argvars, 0 as ::core::ffi::c_int);
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if win.is_null() {
        return;
    }
    let mut cur: *mut matchitem_T = (*win).w_match_head;
    while !cur.is_null() {
        let mut dict: *mut dict_T = tv_dict_alloc();
        if (*cur).mit_match.regprog.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*cur).mit_pos_count {
                let mut llpos: *mut llpos_T = ::core::ptr::null_mut::<llpos_T>();
                let mut buf: [::core::ffi::c_char; 30] = [0; 30];
                llpos = (*cur).mit_pos_array.offset(i as isize);
                if (*llpos).lnum == 0 as linenr_T {
                    break;
                }
                let l: *mut list_T = tv_list_alloc(
                    (1 as ::core::ffi::c_int
                        + (if (*llpos).col > 0 as ::core::ffi::c_int {
                            2 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        })) as ptrdiff_t,
                );
                tv_list_append_number(l, (*llpos).lnum as varnumber_T);
                if (*llpos).col > 0 as ::core::ffi::c_int {
                    tv_list_append_number(l, (*llpos).col as varnumber_T);
                    tv_list_append_number(l, (*llpos).len as varnumber_T);
                }
                let mut len: ::core::ffi::c_int = snprintf(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                    b"pos%d\0".as_ptr() as *const ::core::ffi::c_char,
                    i + 1 as ::core::ffi::c_int,
                );
                '_c2rust_label: {
                    if (len as size_t) < ::core::mem::size_of::<[::core::ffi::c_char; 30]>() {
                    } else {
                        __assert_fail(
                            b"(size_t)len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/match.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            898 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                tv_dict_add_list(
                    dict,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    len as size_t,
                    l,
                );
                i += 1;
            }
        } else {
            tv_dict_add_str(
                dict,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                (*cur).mit_pattern,
            );
        }
        tv_dict_add_str(
            dict,
            b"group\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            syn_id2name((*cur).mit_hlg_id),
        );
        tv_dict_add_nr(
            dict,
            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            (*cur).mit_priority as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            (*cur).mit_id as varnumber_T,
        );
        if (*cur).mit_conceal_char != 0 {
            let mut buf_0: [::core::ffi::c_char; 7] = [0; 7];
            buf_0[utf_char2bytes(
                (*cur).mit_conceal_char,
                &raw mut buf_0 as *mut ::core::ffi::c_char,
            ) as usize] = NUL as ::core::ffi::c_char;
            tv_dict_add_str(
                dict,
                b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                &raw mut buf_0 as *mut ::core::ffi::c_char,
            );
        }
        tv_list_append_dict((*rettv).vval.v_list, dict);
        cur = (*cur).mit_next;
    }
}
pub unsafe extern "C" fn f_setmatches(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut s: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut win: *mut win_T = get_optional_window(argvars, 1 as ::core::ffi::c_int);
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    }
    if win.is_null() {
        return;
    }
    let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let mut li_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                || {
                    d = (*li).li_tv.vval.v_dict;
                    d.is_null()
                }
            {
                semsg(
                    gettext(
                        b"E474: List item %d is either not a dictionary or an empty one\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    li_idx,
                );
                return;
            }
            if !(!tv_dict_find(
                d,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            )
            .is_null()
                && (!tv_dict_find(
                    d,
                    b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                )
                .is_null()
                    || !tv_dict_find(
                        d,
                        b"pos1\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                            as ptrdiff_t,
                    )
                    .is_null())
                && !tv_dict_find(
                    d,
                    b"priority\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                )
                .is_null()
                && !tv_dict_find(
                    d,
                    b"id\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                )
                .is_null())
            {
                semsg(
                    gettext(
                        b"E474: List item %d is missing one of the required keys\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    li_idx,
                );
                return;
            }
            li_idx += 1;
            li = (*li).li_next;
        }
    }
    clear_matches(win);
    let mut match_add_failed: bool = false_0 != 0;
    let l__0: *const list_T = l;
    if !l__0.is_null() {
        let mut li_0: *const listitem_T = (*l__0).lv_first;
        while !li_0.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            d = (*li_0).li_tv.vval.v_dict;
            let di: *mut dictitem_T = tv_dict_find(
                d,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if di.is_null() {
                if s.is_null() {
                    s = tv_list_alloc(9 as ptrdiff_t);
                }
                i = 1 as ::core::ffi::c_int;
                while i < 9 as ::core::ffi::c_int {
                    let mut buf: [::core::ffi::c_char; 30] = [0; 30];
                    snprintf(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                        b"pos%d\0".as_ptr() as *const ::core::ffi::c_char,
                        i,
                    );
                    let pos_di: *mut dictitem_T =
                        tv_dict_find(d, &raw mut buf as *mut ::core::ffi::c_char, -1 as ptrdiff_t);
                    if pos_di.is_null() {
                        break;
                    }
                    if (*pos_di).di_tv.v_type as ::core::ffi::c_uint
                        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        return;
                    }
                    tv_list_append_tv(s, &raw mut (*pos_di).di_tv);
                    tv_list_ref(s);
                    i += 1;
                }
            }
            let mut group_buf: [::core::ffi::c_char; 65] = [0; 65];
            let group: *const ::core::ffi::c_char = tv_dict_get_string_buf(
                d,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut group_buf as *mut ::core::ffi::c_char,
            );
            let priority: ::core::ffi::c_int =
                tv_dict_get_number(d, b"priority\0".as_ptr() as *const ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            let id: ::core::ffi::c_int =
                tv_dict_get_number(d, b"id\0".as_ptr() as *const ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            let conceal_di: *mut dictitem_T = tv_dict_find(
                d,
                b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            let conceal: *const ::core::ffi::c_char = if !conceal_di.is_null() {
                tv_get_string(&raw mut (*conceal_di).di_tv)
            } else {
                ::core::ptr::null::<::core::ffi::c_char>()
            };
            if i == 0 as ::core::ffi::c_int {
                if match_add(
                    win,
                    group,
                    tv_dict_get_string(
                        d,
                        b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                        false,
                    ),
                    priority,
                    id,
                    ::core::ptr::null_mut::<list_T>(),
                    conceal,
                ) != id
                {
                    match_add_failed = true;
                }
            } else {
                if match_add(
                    win,
                    group,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    priority,
                    id,
                    s,
                    conceal,
                ) != id
                {
                    match_add_failed = true;
                }
                tv_list_unref(s);
                s = ::core::ptr::null_mut::<list_T>();
            }
            li_0 = (*li_0).li_next;
        }
    }
    if !match_add_failed {
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
}
pub unsafe extern "C" fn f_matchadd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut grpbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let grp: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut grpbuf as *mut ::core::ffi::c_char,
    );
    let pat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut patbuf as *mut ::core::ffi::c_char,
    );
    let mut prio: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    let mut conceal_char: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut win: *mut win_T = curwin.get();
    (*rettv).vval.v_number = -1 as varnumber_T;
    if grp.is_null() || pat.is_null() {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        prio = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            id = tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && matchadd_dict_arg(
                    argvars.offset(4 as ::core::ffi::c_int as isize),
                    &raw mut conceal_char,
                    &raw mut win,
                ) == FAIL
            {
                return;
            }
        }
    }
    if error {
        return;
    }
    if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E798: ID is reserved for \":match\": %d\0".as_ptr() as *const ::core::ffi::c_char
            ),
            id,
        );
        return;
    }
    (*rettv).vval.v_number = match_add(
        win,
        grp,
        pat,
        prio,
        id,
        ::core::ptr::null_mut::<list_T>(),
        conceal_char,
    ) as varnumber_T;
}
pub unsafe extern "C" fn f_matchaddpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let group: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    if group.is_null() {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            b"matchaddpos()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    l = (*argvars.offset(1 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if tv_list_len(l) == 0 as ::core::ffi::c_int {
        return;
    }
    let mut error: bool = false_0 != 0;
    let mut prio: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut conceal_char: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut win: *mut win_T = curwin.get();
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        prio = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            id = tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && matchadd_dict_arg(
                    argvars.offset(4 as ::core::ffi::c_int as isize),
                    &raw mut conceal_char,
                    &raw mut win,
                ) == FAIL
            {
                return;
            }
        }
    }
    if error as ::core::ffi::c_int == true_0 {
        return;
    }
    if id == 1 as ::core::ffi::c_int || id == 2 as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E798: ID is reserved for \"match\": %d\0".as_ptr() as *const ::core::ffi::c_char
            ),
            id,
        );
        return;
    }
    (*rettv).vval.v_number = match_add(
        win,
        group,
        ::core::ptr::null::<::core::ffi::c_char>(),
        prio,
        id,
        l,
        conceal_char,
    ) as varnumber_T;
}
pub unsafe extern "C" fn f_matcharg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    tv_list_alloc_ret(
        rettv,
        (if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
            2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as ptrdiff_t,
    );
    if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
        let m: *mut matchitem_T = get_match(curwin.get(), id);
        if !m.is_null() {
            tv_list_append_string(
                (*rettv).vval.v_list,
                syn_id2name((*m).mit_hlg_id),
                -1 as ssize_t,
            );
            tv_list_append_string((*rettv).vval.v_list, (*m).mit_pattern, -1 as ssize_t);
        } else {
            tv_list_append_string(
                (*rettv).vval.v_list,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ssize_t,
            );
            tv_list_append_string(
                (*rettv).vval.v_list,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ssize_t,
            );
        }
    }
}
pub unsafe extern "C" fn f_matchdelete(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut win: *mut win_T = get_optional_window(argvars, 1 as ::core::ffi::c_int);
    if win.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = match_delete(
            win,
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
            true_0 != 0,
        ) as varnumber_T;
    };
}
pub unsafe fn ex_match(mut eap: *mut exarg_T) {
    let mut g: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = 0;
    if (*eap).line2 <= 3 as linenr_T {
        id = (*eap).line2 as ::core::ffi::c_int;
    } else {
        emsg(&raw const e_invcmd as *const ::core::ffi::c_char);
        return;
    }
    if (*eap).skip == 0 {
        match_delete(curwin.get(), id, false_0 != 0);
    }
    if ends_excmd(*(*eap).arg as ::core::ffi::c_int) != 0 {
        end = (*eap).arg;
    } else if strncasecmp(
        (*eap).arg,
        b"none\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        4 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && (ascii_iswhite(*(*eap).arg.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
            || ends_excmd(
                *(*eap).arg.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) != 0)
    {
        end = (*eap).arg.offset(4 as ::core::ffi::c_int as isize);
    } else {
        let mut p: *mut ::core::ffi::c_char = skiptowhite((*eap).arg);
        if (*eap).skip == 0 {
            g = xmemdupz(
                (*eap).arg as *const ::core::ffi::c_void,
                p.offset_from((*eap).arg) as size_t,
            ) as *mut ::core::ffi::c_char;
        }
        p = skipwhite(p);
        if *p as ::core::ffi::c_int == NUL {
            xfree(g as *mut ::core::ffi::c_void);
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
            return;
        }
        end = skip_regexp(
            p.offset(1 as ::core::ffi::c_int as isize),
            *p as ::core::ffi::c_int,
            true_0,
        );
        if (*eap).skip == 0 {
            if *end as ::core::ffi::c_int != NUL
                && ends_excmd(
                    *skipwhite(end.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                ) == 0
            {
                xfree(g as *mut ::core::ffi::c_void);
                (*eap).errmsg =
                    ex_errmsg(&raw const e_trailing_arg as *const ::core::ffi::c_char, end);
                return;
            }
            if *end as ::core::ffi::c_int != *p as ::core::ffi::c_int {
                xfree(g as *mut ::core::ffi::c_void);
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    p,
                );
                return;
            }
            let mut c: ::core::ffi::c_int = *end as uint8_t as ::core::ffi::c_int;
            *end = NUL as ::core::ffi::c_char;
            match_add(
                curwin.get(),
                g,
                p.offset(1 as ::core::ffi::c_int as isize),
                10 as ::core::ffi::c_int,
                id,
                ::core::ptr::null_mut::<list_T>(),
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            xfree(g as *mut ::core::ffi::c_void);
            *end = c as ::core::ffi::c_char;
        }
    }
    (*eap).nextcmd = find_nextcmd(end);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
