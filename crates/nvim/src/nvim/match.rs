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

// The carve of the transpiled module; see each child's docs.
mod searchhl;
pub use self::searchhl::*;
mod vimscript;
pub use self::vimscript::*;
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
