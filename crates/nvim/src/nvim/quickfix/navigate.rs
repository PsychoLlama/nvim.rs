//! Choosing which entry to go to next.
//!
//! [`ex_cc`] is `:cc`/`:ll`, [`ex_cnext`] the `:cnext`/`:cprev`/`:cfirst`
//! family, and [`ex_cbelow`] the position-relative
//! `:cabove`/`:cbelow`/`:cbefore`/`:cafter`, which need the adjacent-entry
//! finders in this file to decide what "next" means relative to the
//! cursor.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe fn ex_cc(mut eap: *mut exarg_T) {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, true_0 != 0);
        if qi.is_null() {
            return;
        }
        let mut errornr: ::core::ffi::c_int = 0;
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            errornr = (*eap).line2 as ::core::ffi::c_int;
        } else {
            match (*eap).cmdidx as ::core::ffi::c_int {
                59 | 243 => {
                    errornr = 0 as ::core::ffi::c_int;
                }
                104 | 261 | 67 | 235 => {
                    errornr = 1 as ::core::ffi::c_int;
                }
                _ => {
                    errornr = 32767 as ::core::ffi::c_int;
                }
            }
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_cdo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_ldo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_cfdo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int
        {
            let mut n: size_t = 0;
            if (*eap).addr_count > 0 as ::core::ffi::c_int {
                '_c2rust_label: {
                    if (*eap).line1 >= 0 as linenr_T {
                    } else {
                        __assert_fail(
                            b"eap->line1 >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            4917 as ::core::ffi::c_uint,
                            b"void ex_cc(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                n = (*eap).line1 as size_t;
            } else {
                n = 1 as size_t;
            }
            let mut valid_entry: size_t = qf_get_nth_valid_entry(
                qf_get_curlist(qi),
                n,
                (*eap).cmdidx as ::core::ffi::c_int == CMD_cfdo as ::core::ffi::c_int
                    || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int,
            );
            '_c2rust_label_0: {
                if valid_entry <= 2147483647 as ::core::ffi::c_int as size_t {
                } else {
                    __assert_fail(
                        b"valid_entry <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        4924 as ::core::ffi::c_uint,
                        b"void ex_cc(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            errornr = valid_entry as ::core::ffi::c_int;
        }
        qf_jump(qi, 0 as ::core::ffi::c_int, errornr, (*eap).forceit);
    }
}

pub unsafe fn ex_cnext(mut eap: *mut exarg_T) {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, true_0 != 0);
        if qi.is_null() {
            return;
        }
        let mut errornr: ::core::ffi::c_int = 0;
        if (*eap).addr_count > 0 as ::core::ffi::c_int
            && ((*eap).cmdidx as ::core::ffi::c_int != CMD_cdo as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_ldo as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_cfdo as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_lfdo as ::core::ffi::c_int)
        {
            errornr = (*eap).line2 as ::core::ffi::c_int;
        } else {
            errornr = 1 as ::core::ffi::c_int;
        }
        let mut dir: Direction = kDirectionNotSet;
        match (*eap).cmdidx as ::core::ffi::c_int {
            101 | 259 | 44 | 211 => {
                dir = BACKWARD;
            }
            86 | 252 | 66 | 234 => {
                dir = FORWARD_FILE;
            }
            102 | 260 | 45 | 212 => {
                dir = BACKWARD_FILE;
            }
            84 | 250 | 62 | 228 | _ => {
                dir = FORWARD;
            }
        }
        qf_jump(qi, dir as ::core::ffi::c_int, errornr, (*eap).forceit);
    }
}

pub(crate) unsafe extern "C" fn qf_find_first_entry_in_buf(
    mut qfl: *mut qf_list_T,
    mut bnr: ::core::ffi::c_int,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        idx = 1 as ::core::ffi::c_int;
        qfp = (*qfl).qf_start;
        while !got_int.get() && idx <= (*qfl).qf_count && !qfp.is_null() {
            if (*qfp).qf_fnum == bnr {
                break;
            }
            idx += 1;
            qfp = (*qfp).qf_next;
        }
        *errornr = idx;
        return qfp;
    }
}

pub(crate) unsafe extern "C" fn qf_find_first_entry_on_line(
    mut entry: *mut qfline_T,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        while !got_int.get()
            && !(*entry).qf_prev.is_null()
            && (*entry).qf_fnum == (*(*entry).qf_prev).qf_fnum
            && (*entry).qf_lnum == (*(*entry).qf_prev).qf_lnum
        {
            entry = (*entry).qf_prev;
            *errornr -= 1;
        }
        return entry;
    }
}

pub(crate) unsafe extern "C" fn qf_find_last_entry_on_line(
    mut entry: *mut qfline_T,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        while !got_int.get()
            && !(*entry).qf_next.is_null()
            && (*entry).qf_fnum == (*(*entry).qf_next).qf_fnum
            && (*entry).qf_lnum == (*(*entry).qf_next).qf_lnum
        {
            entry = (*entry).qf_next;
            *errornr += 1;
        }
        return entry;
    }
}

pub(crate) unsafe extern "C" fn qf_entry_after_pos(
    mut qfp: *const qfline_T,
    mut pos: *const pos_T,
    mut linewise: bool,
) -> bool {
    unsafe {
        if linewise {
            return (*qfp).qf_lnum > (*pos).lnum;
        }
        return (*qfp).qf_lnum > (*pos).lnum
            || (*qfp).qf_lnum == (*pos).lnum && (*qfp).qf_col > (*pos).col;
    }
}

pub(crate) unsafe extern "C" fn qf_entry_before_pos(
    mut qfp: *const qfline_T,
    mut pos: *const pos_T,
    mut linewise: bool,
) -> bool {
    unsafe {
        if linewise {
            return (*qfp).qf_lnum < (*pos).lnum;
        }
        return (*qfp).qf_lnum < (*pos).lnum
            || (*qfp).qf_lnum == (*pos).lnum && (*qfp).qf_col < (*pos).col;
    }
}

pub(crate) unsafe extern "C" fn qf_entry_on_or_after_pos(
    mut qfp: *const qfline_T,
    mut pos: *const pos_T,
    mut linewise: bool,
) -> bool {
    unsafe {
        if linewise {
            return (*qfp).qf_lnum >= (*pos).lnum;
        }
        return (*qfp).qf_lnum > (*pos).lnum
            || (*qfp).qf_lnum == (*pos).lnum && (*qfp).qf_col >= (*pos).col;
    }
}

pub(crate) unsafe extern "C" fn qf_entry_on_or_before_pos(
    mut qfp: *const qfline_T,
    mut pos: *const pos_T,
    mut linewise: bool,
) -> bool {
    unsafe {
        if linewise {
            return (*qfp).qf_lnum <= (*pos).lnum;
        }
        return (*qfp).qf_lnum < (*pos).lnum
            || (*qfp).qf_lnum == (*pos).lnum && (*qfp).qf_col <= (*pos).col;
    }
}

pub(crate) unsafe extern "C" fn qf_find_entry_after_pos(
    mut bnr: ::core::ffi::c_int,
    mut pos: *const pos_T,
    mut linewise: bool,
    mut qfp: *mut qfline_T,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        if qf_entry_after_pos(qfp, pos, linewise) {
            return qfp;
        }
        while !(*qfp).qf_next.is_null()
            && (*(*qfp).qf_next).qf_fnum == bnr
            && qf_entry_on_or_before_pos((*qfp).qf_next, pos, linewise) as ::core::ffi::c_int != 0
        {
            qfp = (*qfp).qf_next;
            *errornr += 1;
        }
        if (*qfp).qf_next.is_null() || (*(*qfp).qf_next).qf_fnum != bnr {
            return ::core::ptr::null_mut::<qfline_T>();
        }
        qfp = (*qfp).qf_next;
        *errornr += 1;
        return qfp;
    }
}

pub(crate) unsafe extern "C" fn qf_find_entry_before_pos(
    mut bnr: ::core::ffi::c_int,
    mut pos: *const pos_T,
    mut linewise: bool,
    mut qfp: *mut qfline_T,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        while !(*qfp).qf_next.is_null()
            && (*(*qfp).qf_next).qf_fnum == bnr
            && qf_entry_before_pos((*qfp).qf_next, pos, linewise) as ::core::ffi::c_int != 0
        {
            qfp = (*qfp).qf_next;
            *errornr += 1;
        }
        if qf_entry_on_or_after_pos(qfp, pos, linewise) {
            return ::core::ptr::null_mut::<qfline_T>();
        }
        if linewise {
            qfp = qf_find_first_entry_on_line(qfp, errornr);
        }
        return qfp;
    }
}

pub(crate) unsafe extern "C" fn qf_find_closest_entry(
    mut qfl: *mut qf_list_T,
    mut bnr: ::core::ffi::c_int,
    mut pos: *const pos_T,
    mut dir: Direction,
    mut linewise: bool,
    mut errornr: *mut ::core::ffi::c_int,
) -> *mut qfline_T {
    unsafe {
        *errornr = 0 as ::core::ffi::c_int;
        let mut qfp: *mut qfline_T = qf_find_first_entry_in_buf(qfl, bnr, errornr);
        if qfp.is_null() {
            return ::core::ptr::null_mut::<qfline_T>();
        }
        if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            qfp = qf_find_entry_after_pos(bnr, pos, linewise, qfp, errornr);
        } else {
            qfp = qf_find_entry_before_pos(bnr, pos, linewise, qfp, errornr);
        }
        return qfp;
    }
}

pub(crate) unsafe extern "C" fn qf_get_nth_below_entry(
    mut entry_arg: *mut qfline_T,
    mut n: linenr_T,
    mut linewise: bool,
    mut errornr: *mut ::core::ffi::c_int,
) {
    unsafe {
        let mut entry: *mut qfline_T = entry_arg;
        loop {
            let c2rust_fresh25 = n;
            n = n - 1;
            if !(c2rust_fresh25 > 0 as linenr_T && !got_int.get()) {
                break;
            }
            let mut first_errornr: ::core::ffi::c_int = *errornr;
            if linewise {
                entry = qf_find_last_entry_on_line(entry, errornr);
            }
            if (*entry).qf_next.is_null() || (*(*entry).qf_next).qf_fnum != (*entry).qf_fnum {
                if linewise {
                    *errornr = first_errornr;
                }
                break;
            } else {
                entry = (*entry).qf_next;
                *errornr += 1;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn qf_get_nth_above_entry(
    mut entry: *mut qfline_T,
    mut n: linenr_T,
    mut linewise: bool,
    mut errornr: *mut ::core::ffi::c_int,
) {
    unsafe {
        loop {
            let c2rust_fresh24 = n;
            n = n - 1;
            if !(c2rust_fresh24 > 0 as linenr_T && !got_int.get()) {
                break;
            }
            if (*entry).qf_prev.is_null() || (*(*entry).qf_prev).qf_fnum != (*entry).qf_fnum {
                break;
            }
            entry = (*entry).qf_prev;
            *errornr -= 1;
            if linewise {
                entry = qf_find_first_entry_on_line(entry, errornr);
            }
        }
    }
}

pub(crate) unsafe extern "C" fn qf_find_nth_adj_entry(
    mut qfl: *mut qf_list_T,
    mut bnr: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut n: linenr_T,
    mut dir: Direction,
    mut linewise: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut errornr: ::core::ffi::c_int = 0;
        let adj_entry: *mut qfline_T =
            qf_find_closest_entry(qfl, bnr, pos, dir, linewise, &raw mut errornr);
        if adj_entry.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        n -= 1;
        if n > 0 as linenr_T {
            if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                qf_get_nth_below_entry(adj_entry, n, linewise, &raw mut errornr);
            } else {
                qf_get_nth_above_entry(adj_entry, n, linewise, &raw mut errornr);
            }
        }
        return errornr;
    }
}

pub unsafe fn ex_cbelow(mut eap: *mut exarg_T) {
    unsafe {
        if (*eap).addr_count > 0 as ::core::ffi::c_int && (*eap).line2 <= 0 as linenr_T {
            emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
            return;
        }
        let mut buf_has_flag: ::core::ffi::c_int = if (*eap).cmdidx as ::core::ffi::c_int
            == CMD_cabove as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_cbelow as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_cbefore as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_cafter as ::core::ffi::c_int
        {
            BUF_HAS_QF_ENTRY
        } else {
            BUF_HAS_LL_ENTRY
        };
        if (*curbuf.get()).b_has_qf_entry & buf_has_flag == 0 {
            emsg(gettext(
                &raw const e_no_errors as *const ::core::ffi::c_char,
            ));
            return;
        }
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, true_0 != 0);
        if qi.is_null() {
            return;
        }
        let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
        if !qf_list_has_valid_entries(qfl) {
            emsg(gettext(
                &raw const e_no_errors as *const ::core::ffi::c_char,
            ));
            return;
        }
        let mut dir: ::core::ffi::c_int = if (*eap).cmdidx as ::core::ffi::c_int
            == CMD_cbelow as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lbelow as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_cafter as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lafter as ::core::ffi::c_int
        {
            FORWARD as ::core::ffi::c_int
        } else {
            BACKWARD as ::core::ffi::c_int
        };
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        pos.col += 1;
        let errornr: ::core::ffi::c_int = qf_find_nth_adj_entry(
            qfl,
            (*curbuf.get()).handle as ::core::ffi::c_int,
            &raw mut pos,
            if (*eap).addr_count > 0 as ::core::ffi::c_int {
                (*eap).line2
            } else {
                0 as linenr_T
            },
            dir as Direction,
            (*eap).cmdidx as ::core::ffi::c_int == CMD_cbelow as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_lbelow as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_cabove as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_labove as ::core::ffi::c_int,
        );
        if errornr > 0 as ::core::ffi::c_int {
            qf_jump(qi, 0 as ::core::ffi::c_int, errornr, false_0);
        } else {
            emsg(gettext(e_no_more_items.get()));
        };
    }
}
