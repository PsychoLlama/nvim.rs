//! `build_stl_str_hl()` -- the `'statusline'` format language.
//!
//! One 1,751-line item: the parser and evaluator for the whole `%` alphabet
//! (`%f`, `%l`, `%{expr}`, `%(...%)` groups, `%=` separators, `%<` truncation,
//! `%N.Mx` widths, `%#Hl#` highlights, `%@Func@` click definitions), together
//! with the six function-local arenas it fills -- the item list, the highlight
//! and click tables it hands back to the caller as raw out-parameters, and the
//! scratch buffers the group and truncation passes rewrite in place.
//!
//! It is over the 1,000-line file cap on purpose: the item is not carvable
//! without decomposing it, which is its own slice.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::buffer::{
    append_arg_number, bt_quickfix, buf_spname, calc_percentage, get_rel_pos,
};
use crate::src::nvim::charset::{
    getdigits_int, ptr2cells, skipdigits, trans_characters, vim_strsize,
};
use crate::src::nvim::decoration::{SCL_NUM, SIGN_WIDTH};
use crate::src::nvim::digraph::keymap_str;
use crate::src::nvim::drawline::{fill_foldcolumn, use_cursor_line_highlight};
use crate::src::nvim::drawscreen::compute_foldcolumn;
use crate::src::nvim::eval::eval_to_string_safe;
use crate::src::nvim::eval::vars::{do_unlet, get_vim_var_nr, set_internal_string_var, set_var};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{schar_get, schar_get_adv, schar_len};
use crate::src::nvim::highlight_group::{HLF_CLF, HLF_FC, syn_name2id_len};
use crate::src::nvim::main::{
    KeyTyped, NameBuff, State, VIsual_active, curbuf, curwin, did_emsg, msg_loclist, msg_qflist,
    p_sc, p_sloc, redraw_not_allowed, showcmd_buf, updating_screen,
};
use crate::src::nvim::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::{ml_find_line_or_offset, ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{xfree, xmalloc, xmemdupz, xrealloc, xstrlcpy};
use crate::src::nvim::option::{
    find_option, get_fileformat, get_option_default, set_option_direct, was_set_insecurely,
};
use crate::src::nvim::options::kOptInvalid;
use crate::src::nvim::os::env::home_replace;
use crate::src::nvim::os::libc::{abs, atoi, gettext, memcpy, memmove, strchr, strlen, toupper};
use crate::src::nvim::path::path_tail;
use crate::src::nvim::sign::describe_sign_text;
use crate::src::nvim::state::MODE_INSERT;
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_safelen, vim_strchr};
use crate::src::nvim::types::{
    OptIndex, SignTextAttrs, StlClickRecord, StlFlag, VAR_NUMBER, VAR_UNLOCKED, VV_LNUM, VV_RELNUM,
    VV_VIRTNUM, buf_T, colnr_T, int64_t, linenr_T, ptrdiff_t, schar_T, size_t, statuscol_T,
    stl_hlrec_t, typval_T, typval_vval_union, uint8_t, varnumber_T, win_T,
};
use crate::src::nvim::undo::bufIsChanged;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn build_stl_str_hl(
    mut wp: *mut win_T,
    mut out: *mut ::core::ffi::c_char,
    mut outlen: size_t,
    mut fmt: *mut ::core::ffi::c_char,
    mut opt_idx: OptIndex,
    mut opt_scope: ::core::ffi::c_int,
    mut fillchar: schar_T,
    mut maxwidth: ::core::ffi::c_int,
    mut hltab: *mut *mut stl_hlrec_t,
    mut hltab_len: *mut size_t,
    mut tabtab: *mut *mut StlClickRecord,
    mut stcp: *mut statuscol_T,
) -> ::core::ffi::c_int {
    unsafe {
        static stl_items_len: GlobalCell<size_t> = GlobalCell::new(20 as size_t);
        static stl_items: GlobalCell<*mut stl_item_t> =
            GlobalCell::new(::core::ptr::null_mut::<stl_item_t>());
        static stl_groupitems: GlobalCell<*mut ::core::ffi::c_int> =
            GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_int>());
        static stl_hltab: GlobalCell<*mut stl_hlrec_t> =
            GlobalCell::new(::core::ptr::null_mut::<stl_hlrec_t>());
        static stl_tabtab: GlobalCell<*mut StlClickRecord> =
            GlobalCell::new(::core::ptr::null_mut::<StlClickRecord>());
        static stl_separator_locations: GlobalCell<*mut ::core::ffi::c_int> =
            GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_int>());
        static curitem: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        let mut buf_tmp: [::core::ffi::c_char; 70] = [0; 70];
        let mut usefmt: *mut ::core::ffi::c_char = fmt;
        let save_redraw_not_allowed: bool = redraw_not_allowed.get();
        let save_KeyTyped: bool = KeyTyped.get();
        let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
        if updating_screen.get() {
            redraw_not_allowed.set(true_0 != 0);
        }
        if (*stl_items.ptr()).is_null() {
            stl_items.set(xmalloc(
                ::core::mem::size_of::<stl_item_t>().wrapping_mul(stl_items_len.get()),
            ) as *mut stl_item_t);
            stl_groupitems.set(xmalloc(
                ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(stl_items_len.get()),
            ) as *mut ::core::ffi::c_int);
            stl_hltab.set(xmalloc(
                ::core::mem::size_of::<stl_hlrec_t>()
                    .wrapping_mul((*stl_items_len.ptr()).wrapping_add(1 as size_t)),
            ) as *mut stl_hlrec_t);
            stl_tabtab.set(xmalloc(
                ::core::mem::size_of::<StlClickRecord>()
                    .wrapping_mul((*stl_items_len.ptr()).wrapping_add(1 as size_t)),
            ) as *mut StlClickRecord);
            stl_separator_locations.set(xmalloc(
                ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(stl_items_len.get()),
            ) as *mut ::core::ffi::c_int);
        }
        let use_sandbox: bool = opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int
            && was_set_insecurely(wp, opt_idx, opt_scope);
        if *fmt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '%' as ::core::ffi::c_int
            && *fmt.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '!' as ::core::ffi::c_int
        {
            let mut tv: typval_T = typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_number: (*wp).handle as varnumber_T,
                },
            };
            set_var(
                c"g:statusline_winid".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
                &raw mut tv,
                false_0 != 0,
            );
            usefmt = eval_to_string_safe(
                fmt.offset(2 as ::core::ffi::c_int as isize),
                use_sandbox,
                false_0 != 0,
            );
            if usefmt.is_null() {
                usefmt = fmt;
            }
            do_unlet(
                c"g:statusline_winid".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
                true_0 != 0,
            );
        }
        if fillchar == 0 as schar_T {
            fillchar = ' ' as ::core::ffi::c_int as schar_T;
        }
        let mut lnum: linenr_T = (*wp).w_cursor.lnum;
        if lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
            lnum = (*(*wp).w_buffer).b_ml.ml_line_count;
            (*wp).w_cursor.lnum = lnum;
        }
        let mut line_ptr: *const ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
        let mut empty_line: bool = *line_ptr as ::core::ffi::c_int == NUL;
        let mut byteval: ::core::ffi::c_int = 0;
        let len: colnr_T = ml_get_buf_len((*wp).w_buffer, lnum);
        if (*wp).w_cursor.col > len {
            (*wp).w_cursor.col = len;
            (*wp).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            byteval = 0 as ::core::ffi::c_int;
        } else {
            byteval = utf_ptr2char(line_ptr.offset((*wp).w_cursor.col as isize));
        }
        let mut groupdepth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut evaldepth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut evalstart: ::core::ffi::c_int = curitem.get();
        let mut prevchar_isflag: bool = true_0 != 0;
        let mut prevchar_isitem: bool = false_0 != 0;
        let mut out_p: *mut ::core::ffi::c_char = out;
        let mut out_end_p: *mut ::core::ffi::c_char =
            out.add(outlen).offset(-(1 as ::core::ffi::c_int as isize));
        let mut fmt_p: *mut ::core::ffi::c_char = usefmt;
        's_2297: while *fmt_p as ::core::ffi::c_int != NUL {
            if curitem.get() == stl_items_len.get() as ::core::ffi::c_int {
                let mut new_len: size_t = (*stl_items_len.ptr())
                    .wrapping_mul(3 as size_t)
                    .wrapping_div(2 as size_t);
                stl_items.set(xrealloc(
                    stl_items.get() as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<stl_item_t>().wrapping_mul(new_len),
                ) as *mut stl_item_t);
                stl_groupitems.set(xrealloc(
                    stl_groupitems.get() as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(new_len),
                ) as *mut ::core::ffi::c_int);
                stl_hltab.set(xrealloc(
                    stl_hltab.get() as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<stl_hlrec_t>()
                        .wrapping_mul(new_len.wrapping_add(1 as size_t)),
                ) as *mut stl_hlrec_t);
                stl_tabtab.set(xrealloc(
                    stl_tabtab.get() as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<StlClickRecord>()
                        .wrapping_mul(new_len.wrapping_add(1 as size_t)),
                ) as *mut StlClickRecord);
                stl_separator_locations.set(xrealloc(
                    stl_separator_locations.get() as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(new_len),
                ) as *mut ::core::ffi::c_int);
                stl_items_len.set(new_len);
            }
            if *fmt_p as ::core::ffi::c_int != '%' as ::core::ffi::c_int {
                prevchar_isitem = false_0 != 0;
                prevchar_isflag = prevchar_isitem;
            }
            while *fmt_p as ::core::ffi::c_int != NUL
                && *fmt_p as ::core::ffi::c_int != '%' as ::core::ffi::c_int
                && out_p < out_end_p
            {
                let c2rust_fresh7 = fmt_p;
                fmt_p = fmt_p.offset(1);
                let c2rust_fresh8 = out_p;
                out_p = out_p.offset(1);
                *c2rust_fresh8 = *c2rust_fresh7;
            }
            if *fmt_p as ::core::ffi::c_int == NUL || out_p >= out_end_p {
                break;
            }
            fmt_p = fmt_p.offset(1);
            if *fmt_p as ::core::ffi::c_int == NUL {
                break;
            }
            if *fmt_p as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
                let c2rust_fresh9 = fmt_p;
                fmt_p = fmt_p.offset(1);
                let c2rust_fresh10 = out_p;
                out_p = out_p.offset(1);
                *c2rust_fresh10 = *c2rust_fresh9;
                prevchar_isitem = false_0 != 0;
                prevchar_isflag = prevchar_isitem;
            } else if *fmt_p as ::core::ffi::c_int == STL_SEPARATE as ::core::ffi::c_int {
                fmt_p = fmt_p.offset(1);
                if groupdepth > 0 as ::core::ffi::c_int {
                    continue;
                }
                (*(*stl_items.ptr()).offset(curitem.get() as isize)).type_0 = Separate;
                let c2rust_fresh11 = curitem.get();
                curitem.set(curitem.get() + 1);
                let c2rust_lvalue_ptr =
                    &raw mut (*(*stl_items.ptr()).offset(c2rust_fresh11 as isize)).start;
                *c2rust_lvalue_ptr = out_p;
            } else if *fmt_p as ::core::ffi::c_int == STL_TRUNCMARK as ::core::ffi::c_int {
                fmt_p = fmt_p.offset(1);
                (*(*stl_items.ptr()).offset(curitem.get() as isize)).type_0 = Trunc;
                let c2rust_fresh12 = curitem.get();
                curitem.set(curitem.get() + 1);
                let c2rust_lvalue_ptr_0 =
                    &raw mut (*(*stl_items.ptr()).offset(c2rust_fresh12 as isize)).start;
                *c2rust_lvalue_ptr_0 = out_p;
            } else if *fmt_p as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
                fmt_p = fmt_p.offset(1);
                if groupdepth < 1 as ::core::ffi::c_int {
                    continue;
                }
                groupdepth -= 1;
                let mut t: *mut ::core::ffi::c_char = (*(*stl_items.ptr())
                    .offset(*(*stl_groupitems.ptr()).offset(groupdepth as isize) as isize))
                .start;
                *out_p = NUL as ::core::ffi::c_char;
                let mut group_len: ptrdiff_t = vim_strsize(t) as ptrdiff_t;
                if curitem.get()
                    > *(*stl_groupitems.ptr()).offset(groupdepth as isize) + 1 as ::core::ffi::c_int
                    && (*(*stl_items.ptr())
                        .offset(*(*stl_groupitems.ptr()).offset(groupdepth as isize) as isize))
                    .minwid
                        == 0 as ::core::ffi::c_int
                {
                    let mut group_start_userhl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut group_end_userhl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut n: ::core::ffi::c_int = 0;
                    n = *(*stl_groupitems.ptr()).offset(groupdepth as isize)
                        - 1 as ::core::ffi::c_int;
                    while n >= 0 as ::core::ffi::c_int {
                        if (*(*stl_items.ptr()).offset(n as isize)).type_0 as ::core::ffi::c_uint
                            == Highlight as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*(*stl_items.ptr()).offset(n as isize)).type_0
                                as ::core::ffi::c_uint
                                == HighlightCombining as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            group_end_userhl = (*(*stl_items.ptr()).offset(n as isize)).minwid;
                            group_start_userhl = group_end_userhl;
                            break;
                        } else {
                            n -= 1;
                        }
                    }
                    n = *(*stl_groupitems.ptr()).offset(groupdepth as isize)
                        + 1 as ::core::ffi::c_int;
                    while n < curitem.get() {
                        if (*(*stl_items.ptr()).offset(n as isize)).type_0 as ::core::ffi::c_uint
                            == Normal as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            break;
                        }
                        if (*(*stl_items.ptr()).offset(n as isize)).type_0 as ::core::ffi::c_uint
                            == Highlight as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*(*stl_items.ptr()).offset(n as isize)).type_0
                                as ::core::ffi::c_uint
                                == HighlightCombining as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            group_end_userhl = (*(*stl_items.ptr()).offset(n as isize)).minwid;
                        }
                        n += 1;
                    }
                    if n == curitem.get() && group_start_userhl == group_end_userhl {
                        out_p = t;
                        group_len = 0 as ptrdiff_t;
                        n = *(*stl_groupitems.ptr()).offset(groupdepth as isize)
                            + 1 as ::core::ffi::c_int;
                        while n < curitem.get() {
                            if (*(*stl_items.ptr()).offset(n as isize)).type_0
                                as ::core::ffi::c_uint
                                == Highlight as ::core::ffi::c_int as ::core::ffi::c_uint
                                || (*(*stl_items.ptr()).offset(n as isize)).type_0
                                    as ::core::ffi::c_uint
                                    == HighlightCombining as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                            {
                                (*(*stl_items.ptr()).offset(n as isize)).type_0 = Empty;
                            }
                            if (*(*stl_items.ptr()).offset(n as isize)).type_0
                                as ::core::ffi::c_uint
                                == TabPage as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                (*(*stl_items.ptr()).offset(n as isize)).start = out_p;
                            }
                            n += 1;
                        }
                    }
                }
                let mut minwid: ::core::ffi::c_int = (*(*stl_items.ptr())
                    .offset(*(*stl_groupitems.ptr()).offset(groupdepth as isize) as isize))
                .minwid;
                if group_len
                    > (*(*stl_items.ptr())
                        .offset(*(*stl_groupitems.ptr()).offset(groupdepth as isize) as isize))
                    .maxwid as ptrdiff_t
                    && (*(*stl_items.ptr())
                        .offset(*(*stl_groupitems.ptr()).offset(groupdepth as isize) as isize))
                    .type_0 as ::core::ffi::c_uint
                        != HighlightFold as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut maxwid: ::core::ffi::c_int = (*(*stl_items.ptr())
                        .offset(*(*stl_groupitems.ptr()).offset(groupdepth as isize) as isize))
                    .maxwid;
                    let mut n_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while group_len >= maxwid as ptrdiff_t {
                        group_len -= ptr2cells(t.offset(n_0 as isize)) as ptrdiff_t;
                        n_0 += utfc_ptr2len(t.offset(n_0 as isize));
                    }
                    *t = '<' as ::core::ffi::c_char;
                    memmove(
                        t.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                        t.offset(n_0 as isize) as *const ::core::ffi::c_void,
                        out_p.offset_from(t.offset(n_0 as isize)) as size_t,
                    );
                    out_p = out_p
                        .offset(-(n_0 as isize))
                        .offset(1 as ::core::ffi::c_int as isize);
                    minwid = if minwid < maxwid { minwid } else { maxwid };
                    loop {
                        group_len += 1;
                        if group_len >= minwid as ptrdiff_t {
                            break;
                        }
                        schar_get_adv(&raw mut out_p, fillchar);
                    }
                    let mut idx: ::core::ffi::c_int = *(*stl_groupitems.ptr())
                        .offset(groupdepth as isize)
                        + 1 as ::core::ffi::c_int;
                    while idx < curitem.get() {
                        (*(*stl_items.ptr()).offset(idx as isize)).start = (*(*stl_items.ptr())
                            .offset(idx as isize))
                        .start
                        .offset(-((n_0 - 1 as ::core::ffi::c_int) as isize));
                        (*(*stl_items.ptr()).offset(idx as isize)).start =
                            if (*(*stl_items.ptr()).offset(idx as isize)).start > t {
                                (*(*stl_items.ptr()).offset(idx as isize)).start
                            } else {
                                t
                            };
                        idx += 1;
                    }
                } else if abs(minwid) as ptrdiff_t > group_len {
                    let mut fillchar_bytes: ptrdiff_t = schar_len(fillchar) as ptrdiff_t;
                    if minwid < 0 as ::core::ffi::c_int {
                        minwid = 0 as ::core::ffi::c_int - minwid;
                        loop {
                            let c2rust_fresh13 = group_len;
                            group_len = group_len + 1;
                            if !(c2rust_fresh13 < minwid as ptrdiff_t
                                && out_p.offset(fillchar_bytes as isize) <= out_end_p)
                            {
                                break;
                            }
                            schar_get_adv(&raw mut out_p, fillchar);
                        }
                    } else {
                        let mut added_cells: ptrdiff_t = minwid as ptrdiff_t - group_len;
                        let mut added_bytes: ptrdiff_t = added_cells * fillchar_bytes;
                        if out_p.offset(added_bytes as isize) > out_end_p {
                            added_cells = (out_end_p.offset_from(out_p) / fillchar_bytes as isize)
                                as ptrdiff_t;
                            added_bytes = added_cells * fillchar_bytes;
                        }
                        memmove(
                            t.offset(added_bytes as isize) as *mut ::core::ffi::c_void,
                            t as *const ::core::ffi::c_void,
                            out_p.offset_from(t) as size_t,
                        );
                        out_p = out_p.offset(added_bytes as isize);
                        let mut n_1: ::core::ffi::c_int = *(*stl_groupitems.ptr())
                            .offset(groupdepth as isize)
                            + 1 as ::core::ffi::c_int;
                        while n_1 < curitem.get() {
                            (*(*stl_items.ptr()).offset(n_1 as isize)).start =
                                (*(*stl_items.ptr()).offset(n_1 as isize))
                                    .start
                                    .offset(added_bytes as isize);
                            n_1 += 1;
                        }
                        while added_cells > 0 as ptrdiff_t {
                            schar_get_adv(&raw mut t, fillchar);
                            added_cells -= 1;
                        }
                    }
                }
            } else {
                let mut minwid_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut maxwid_0: ::core::ffi::c_int = 9999 as ::core::ffi::c_int;
                let mut foldsignitem: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                let mut left_align_num: bool = false_0 != 0;
                let mut left_align: bool = false_0 != 0;
                let mut zeropad: bool = *fmt_p as ::core::ffi::c_int == '0' as ::core::ffi::c_int;
                if zeropad {
                    fmt_p = fmt_p.offset(1);
                }
                if *fmt_p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                    fmt_p = fmt_p.offset(1);
                    left_align = true_0 != 0;
                }
                if ascii_isdigit(*fmt_p as ::core::ffi::c_int) {
                    minwid_0 = getdigits_int(&raw mut fmt_p, false_0 != 0, 0 as ::core::ffi::c_int);
                }
                if *fmt_p as ::core::ffi::c_int == STL_USER_HL as ::core::ffi::c_int {
                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).type_0 = Highlight;
                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).start = out_p;
                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).minwid =
                        if minwid_0 > 9 as ::core::ffi::c_int {
                            1 as ::core::ffi::c_int
                        } else {
                            minwid_0
                        };
                    fmt_p = fmt_p.offset(1);
                    (*curitem.ptr()) += 1;
                } else if *fmt_p as ::core::ffi::c_int == STL_TABPAGENR as ::core::ffi::c_int
                    || *fmt_p as ::core::ffi::c_int == STL_TABCLOSENR as ::core::ffi::c_int
                {
                    if *fmt_p as ::core::ffi::c_int == STL_TABCLOSENR as ::core::ffi::c_int {
                        if minwid_0 == 0 as ::core::ffi::c_int {
                            let mut n_2: ::core::ffi::c_int =
                                curitem.get() - 1 as ::core::ffi::c_int;
                            while n_2 >= 0 as ::core::ffi::c_int {
                                if (*(*stl_items.ptr()).offset(n_2 as isize)).type_0
                                    as ::core::ffi::c_uint
                                    == TabPage as ::core::ffi::c_int as ::core::ffi::c_uint
                                    && (*(*stl_items.ptr()).offset(n_2 as isize)).minwid
                                        >= 0 as ::core::ffi::c_int
                                {
                                    minwid_0 = (*(*stl_items.ptr()).offset(n_2 as isize)).minwid;
                                    break;
                                } else {
                                    n_2 -= 1;
                                }
                            }
                        } else {
                            minwid_0 = -minwid_0;
                        }
                    }
                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).type_0 = TabPage;
                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).start = out_p;
                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).minwid = minwid_0;
                    fmt_p = fmt_p.offset(1);
                    (*curitem.ptr()) += 1;
                } else if *fmt_p as ::core::ffi::c_int == STL_CLICK_FUNC as ::core::ffi::c_int {
                    fmt_p = fmt_p.offset(1);
                    let mut t_0: *mut ::core::ffi::c_char = fmt_p;
                    while *fmt_p as ::core::ffi::c_int != STL_CLICK_FUNC as ::core::ffi::c_int
                        && *fmt_p as ::core::ffi::c_int != 0
                    {
                        fmt_p = fmt_p.offset(1);
                    }
                    if *fmt_p as ::core::ffi::c_int != STL_CLICK_FUNC as ::core::ffi::c_int {
                        break;
                    }
                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).type_0 = ClickFunc;
                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).start = out_p;
                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).cmd =
                        (if !tabtab.is_null() {
                            xmemdupz(
                                t_0 as *const ::core::ffi::c_void,
                                fmt_p.offset_from(t_0) as size_t,
                            )
                        } else {
                            NULL
                        }) as *mut ::core::ffi::c_char;
                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).minwid = minwid_0;
                    fmt_p = fmt_p.offset(1);
                    (*curitem.ptr()) += 1;
                } else {
                    if *fmt_p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                        fmt_p = fmt_p.offset(1);
                        if ascii_isdigit(*fmt_p as ::core::ffi::c_int) {
                            maxwid_0 = getdigits_int(
                                &raw mut fmt_p,
                                false_0 != 0,
                                50 as ::core::ffi::c_int,
                            );
                        }
                    }
                    minwid_0 = (if minwid_0 > 50 as ::core::ffi::c_int {
                        50 as ::core::ffi::c_int
                    } else {
                        minwid_0
                    }) * (if left_align as ::core::ffi::c_int != 0 {
                        -1 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    });
                    if *fmt_p as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
                        let c2rust_fresh14 = groupdepth;
                        groupdepth = groupdepth + 1;
                        *(*stl_groupitems.ptr()).offset(c2rust_fresh14 as isize) = curitem.get();
                        (*(*stl_items.ptr()).offset(curitem.get() as isize)).type_0 = Group;
                        (*(*stl_items.ptr()).offset(curitem.get() as isize)).start = out_p;
                        (*(*stl_items.ptr()).offset(curitem.get() as isize)).minwid = minwid_0;
                        (*(*stl_items.ptr()).offset(curitem.get() as isize)).maxwid = maxwid_0;
                        fmt_p = fmt_p.offset(1);
                        (*curitem.ptr()) += 1;
                    } else if *fmt_p as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                        && evaldepth > 0 as ::core::ffi::c_int
                    {
                        fmt_p = fmt_p.offset(1);
                        evaldepth -= 1;
                    } else {
                        let mut c2rust_lvalue: [::core::ffi::c_char; 45] = [
                            STL_FILEPATH as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_FULLPATH as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_FILENAME as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_COLUMN as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_VIRTCOL as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_VIRTCOL_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_LINE as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_NUMLINES as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_BUFNO as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_KEYMAP as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_OFFSET as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_OFFSET_X as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_BYTEVAL as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_BYTEVAL_X as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_ROFLAG as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_ROFLAG_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_HELPFLAG as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_HELPFLAG_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_FILETYPE as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_FILETYPE_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_PREVIEWFLAG as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_PREVIEWFLAG_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_MODIFIED as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_MODIFIED_ALT as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_PERCENTAGE as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_ALTPERCENT as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_ARGLISTSTAT as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_PAGENUM as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_SHOWCMD as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_FOLDCOL as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_SIGNCOL as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_VIM_EXPR as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_SEPARATE as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_TRUNCMARK as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_USER_HL as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_HIGHLIGHT as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_HIGHLIGHT_COMB as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_TABPAGENR as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_TABCLOSENR as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_CLICK_FUNC as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_TABPAGENR as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_TABCLOSENR as ::core::ffi::c_int as ::core::ffi::c_char,
                            STL_CLICK_FUNC as ::core::ffi::c_int as ::core::ffi::c_char,
                            0 as ::core::ffi::c_char,
                        ];
                        if vim_strchr(
                            &raw mut c2rust_lvalue as *mut ::core::ffi::c_char,
                            *fmt_p as uint8_t as ::core::ffi::c_int,
                        )
                        .is_null()
                        {
                            if *fmt_p as ::core::ffi::c_int == NUL {
                                break;
                            } else {
                                fmt_p = fmt_p.offset(1);
                            }
                        } else {
                            let c2rust_fresh15 = fmt_p;
                            fmt_p = fmt_p.offset(1);
                            let mut opt: ::core::ffi::c_char = *c2rust_fresh15;
                            let mut base: NumberBase = kNumBaseDecimal;
                            let mut itemisflag: bool = false_0 != 0;
                            let mut fillable: bool = true_0 != 0;
                            let mut num: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                            let mut str: *mut ::core::ffi::c_char =
                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                            's_1848: {
                                '_stcsign: {
                                    's_1418: {
                                        match opt as ::core::ffi::c_int {
                                            102 | 70 | 116 => {
                                                fillable = false_0 != 0;
                                                let mut name: *mut ::core::ffi::c_char =
                                                    buf_spname((*wp).w_buffer);
                                                if !name.is_null() {
                                                    xstrlcpy(
                                                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                                                        name,
                                                        MAXPATHL as size_t,
                                                    );
                                                } else {
                                                    let mut t_1: *mut ::core::ffi::c_char = if opt
                                                        as ::core::ffi::c_int
                                                        == STL_FULLPATH as ::core::ffi::c_int
                                                    {
                                                        (*(*wp).w_buffer).b_ffname
                                                    } else {
                                                        (*(*wp).w_buffer).b_fname
                                                    };
                                                    home_replace(
                                                        (*wp).w_buffer,
                                                        t_1,
                                                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                                                        MAXPATHL as size_t,
                                                        true_0 != 0,
                                                    );
                                                }
                                                trans_characters(
                                                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                                                    MAXPATHL,
                                                );
                                                if opt as ::core::ffi::c_int
                                                    != STL_FILENAME as ::core::ffi::c_int
                                                {
                                                    str =
                                                        NameBuff.ptr() as *mut ::core::ffi::c_char;
                                                } else {
                                                    str =
                                                        path_tail(NameBuff.ptr()
                                                            as *mut ::core::ffi::c_char);
                                                }
                                                break 's_1848;
                                            }
                                            123 => {
                                                let mut block_start: *mut ::core::ffi::c_char =
                                                    fmt_p.offset(
                                                        -(1 as ::core::ffi::c_int as isize),
                                                    );
                                                let mut reevaluate: bool = *fmt_p
                                                    as ::core::ffi::c_int
                                                    == '%' as ::core::ffi::c_int;
                                                itemisflag = true_0 != 0;
                                                if reevaluate {
                                                    fmt_p = fmt_p.offset(1);
                                                }
                                                let mut t_2: *mut ::core::ffi::c_char = out_p;
                                                while (*fmt_p as ::core::ffi::c_int
                                                    != '}' as ::core::ffi::c_int
                                                    || reevaluate as ::core::ffi::c_int != 0
                                                        && *fmt_p.offset(
                                                            -1 as ::core::ffi::c_int as isize,
                                                        )
                                                            as ::core::ffi::c_int
                                                            != '%' as ::core::ffi::c_int)
                                                    && *fmt_p as ::core::ffi::c_int != NUL
                                                    && out_p < out_end_p
                                                {
                                                    let c2rust_fresh16 = fmt_p;
                                                    fmt_p = fmt_p.offset(1);
                                                    let c2rust_fresh17 = out_p;
                                                    out_p = out_p.offset(1);
                                                    *c2rust_fresh17 = *c2rust_fresh16;
                                                }
                                                if *fmt_p as ::core::ffi::c_int
                                                    != '}' as ::core::ffi::c_int
                                                {
                                                    break 's_1848;
                                                } else {
                                                    fmt_p = fmt_p.offset(1);
                                                    if reevaluate as ::core::ffi::c_int != 0
                                                        && out_p > out
                                                    {
                                                        *out_p.offset(
                                                            -1 as ::core::ffi::c_int as isize,
                                                        ) = NUL as ::core::ffi::c_char;
                                                    } else {
                                                        *out_p = NUL as ::core::ffi::c_char;
                                                    }
                                                    out_p = t_2;
                                                    vim_snprintf(
                                                        &raw mut buf_tmp
                                                            as *mut ::core::ffi::c_char,
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 70],
                                                        >(
                                                        ),
                                                        c"%d".as_ptr(),
                                                        (*curbuf.get()).handle,
                                                    );
                                                    set_internal_string_var(
                                                        c"g:actual_curbuf".as_ptr(),
                                                        &raw mut buf_tmp
                                                            as *mut ::core::ffi::c_char,
                                                    );
                                                    vim_snprintf(
                                                        &raw mut buf_tmp
                                                            as *mut ::core::ffi::c_char,
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 70],
                                                        >(
                                                        ),
                                                        c"%d".as_ptr(),
                                                        (*curwin.get()).handle,
                                                    );
                                                    set_internal_string_var(
                                                        c"g:actual_curwin".as_ptr(),
                                                        &raw mut buf_tmp
                                                            as *mut ::core::ffi::c_char,
                                                    );
                                                    let save_curbuf: *mut buf_T = curbuf.get();
                                                    let save_curwin: *mut win_T = curwin.get();
                                                    let save_VIsual_active: ::core::ffi::c_int =
                                                        VIsual_active.get() as ::core::ffi::c_int;
                                                    curwin.set(wp);
                                                    curbuf.set((*wp).w_buffer);
                                                    if curwin.get() != save_curwin {
                                                        VIsual_active.set(false_0 != 0);
                                                    }
                                                    str = eval_to_string_safe(
                                                        out_p,
                                                        use_sandbox,
                                                        false_0 != 0,
                                                    );
                                                    curwin.set(save_curwin);
                                                    curbuf.set(save_curbuf);
                                                    VIsual_active.set(save_VIsual_active != 0);
                                                    do_unlet(
                                                        c"g:actual_curbuf".as_ptr(),
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 16],
                                                        >(
                                                        )
                                                        .wrapping_sub(1 as size_t),
                                                        true_0 != 0,
                                                    );
                                                    do_unlet(
                                                        c"g:actual_curwin".as_ptr(),
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 16],
                                                        >(
                                                        )
                                                        .wrapping_sub(1 as size_t),
                                                        true_0 != 0,
                                                    );
                                                    if !str.is_null()
                                                        && *str as ::core::ffi::c_int != NUL
                                                    {
                                                        if *skipdigits(str) as ::core::ffi::c_int
                                                            == NUL
                                                        {
                                                            num = atoi(str);
                                                            let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut str
                                                            as *mut *mut ::core::ffi::c_void;
                                                            xfree(*ptr_);
                                                            *ptr_ = NULL;
                                                            let _ = *ptr_;
                                                            itemisflag = false_0 != 0;
                                                        }
                                                    }
                                                    if reevaluate as ::core::ffi::c_int != 0
                                                        && !str.is_null()
                                                        && *str as ::core::ffi::c_int != NUL
                                                        && !strchr(str, '%' as ::core::ffi::c_int)
                                                            .is_null()
                                                        && evaldepth < MAX_STL_EVAL_DEPTH
                                                    {
                                                        let mut parsed_usefmt: size_t = block_start
                                                            .offset_from(usefmt)
                                                            as size_t;
                                                        let mut str_length: size_t = strlen(str);
                                                        let mut fmt_length: size_t = strlen(fmt_p);
                                                        let mut new_fmt_len: size_t = parsed_usefmt
                                                            .wrapping_add(str_length)
                                                            .wrapping_add(fmt_length)
                                                            .wrapping_add(3 as size_t);
                                                        let mut new_fmt: *mut ::core::ffi::c_char =
                                                            xmalloc(new_fmt_len.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    ::core::ffi::c_char,
                                                                >(
                                                                ),
                                                            ))
                                                                as *mut ::core::ffi::c_char;
                                                        let mut new_fmt_p: *mut ::core::ffi::c_char =
                                                        new_fmt;
                                                        new_fmt_p = (memcpy(
                                                            new_fmt_p as *mut ::core::ffi::c_void,
                                                            usefmt as *const ::core::ffi::c_void,
                                                            parsed_usefmt,
                                                        )
                                                            as *mut ::core::ffi::c_char)
                                                            .add(parsed_usefmt);
                                                        new_fmt_p = (memcpy(
                                                            new_fmt_p as *mut ::core::ffi::c_void,
                                                            str as *const ::core::ffi::c_void,
                                                            str_length,
                                                        )
                                                            as *mut ::core::ffi::c_char)
                                                            .add(str_length);
                                                        new_fmt_p = (memcpy(
                                                            new_fmt_p as *mut ::core::ffi::c_void,
                                                            c"%}".as_ptr()
                                                                as *const ::core::ffi::c_void,
                                                            2 as size_t,
                                                        )
                                                            as *mut ::core::ffi::c_char)
                                                            .offset(
                                                                2 as ::core::ffi::c_int as isize,
                                                            );
                                                        new_fmt_p = (memcpy(
                                                            new_fmt_p as *mut ::core::ffi::c_void,
                                                            fmt_p as *const ::core::ffi::c_void,
                                                            fmt_length,
                                                        )
                                                            as *mut ::core::ffi::c_char)
                                                            .add(fmt_length);
                                                        *new_fmt_p = 0 as ::core::ffi::c_char;
                                                        new_fmt_p = ::core::ptr::null_mut::<
                                                            ::core::ffi::c_char,
                                                        >(
                                                        );
                                                        if usefmt != fmt {
                                                            xfree(
                                                                usefmt as *mut ::core::ffi::c_void,
                                                            );
                                                        }
                                                        let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                                        &raw mut str
                                                            as *mut *mut ::core::ffi::c_void;
                                                        xfree(*ptr__0);
                                                        *ptr__0 = NULL;
                                                        let _ = *ptr__0;
                                                        usefmt = new_fmt;
                                                        fmt_p = usefmt.add(parsed_usefmt);
                                                        evaldepth += 1;
                                                        continue 's_2297;
                                                    } else {
                                                        break 's_1848;
                                                    }
                                                }
                                            }
                                            108 => {
                                                if !stcp.is_null()
                                                    && ((*wp).w_onebuf_opt.wo_nu != 0
                                                        || (*wp).w_onebuf_opt.wo_rnu != 0)
                                                    && get_vim_var_nr(VV_VIRTNUM)
                                                        == 0 as varnumber_T
                                                {
                                                    if (*wp).w_maxscwidth == SCL_NUM
                                                        && (*(*stcp).sattrs.offset(
                                                            0 as ::core::ffi::c_int as isize,
                                                        ))
                                                        .text
                                                            [0 as ::core::ffi::c_int as usize]
                                                            != 0
                                                    {
                                                        break '_stcsign;
                                                    } else {
                                                        let mut relnum: ::core::ffi::c_int =
                                                            get_vim_var_nr(VV_RELNUM)
                                                                as ::core::ffi::c_int;
                                                        num = if (*wp).w_onebuf_opt.wo_rnu == 0
                                                            || (*wp).w_onebuf_opt.wo_nu != 0
                                                                && relnum == 0 as ::core::ffi::c_int
                                                        {
                                                            get_vim_var_nr(VV_LNUM)
                                                                as ::core::ffi::c_int
                                                        } else {
                                                            relnum
                                                        };
                                                        left_align_num = (*wp).w_onebuf_opt.wo_rnu
                                                            != 0
                                                            && (*wp).w_onebuf_opt.wo_nu != 0
                                                            && relnum == 0 as ::core::ffi::c_int;
                                                        if !left_align_num {
                                                            (*(*stl_items.ptr())
                                                                .offset(curitem.get() as isize))
                                                            .type_0 = Separate;
                                                            let c2rust_fresh18 = curitem.get();
                                                            curitem.set(curitem.get() + 1);
                                                            let c2rust_lvalue_ptr_1 =
                                                                &raw mut (*(*stl_items.ptr())
                                                                    .offset(
                                                                        c2rust_fresh18 as isize,
                                                                    ))
                                                                .start;
                                                            *c2rust_lvalue_ptr_1 = out_p;
                                                        }
                                                        break 's_1848;
                                                    }
                                                } else {
                                                    if stcp.is_null() {
                                                        num = (if (*(*wp).w_buffer).b_ml.ml_flags
                                                            & ML_EMPTY
                                                            != 0
                                                        {
                                                            0 as linenr_T
                                                        } else {
                                                            (*wp).w_cursor.lnum
                                                        })
                                                            as ::core::ffi::c_int;
                                                    }
                                                    break 's_1848;
                                                }
                                            }
                                            76 => {
                                                num = (*(*wp).w_buffer).b_ml.ml_line_count
                                                    as ::core::ffi::c_int;
                                                break 's_1848;
                                            }
                                            99 => {
                                                num = if State.get() & MODE_INSERT
                                                    == 0 as ::core::ffi::c_int
                                                    && empty_line as ::core::ffi::c_int != 0
                                                {
                                                    0 as ::core::ffi::c_int
                                                } else {
                                                    (*wp).w_cursor.col + 1 as ::core::ffi::c_int
                                                };
                                                break 's_1848;
                                            }
                                            118 | 86 => {
                                                let mut virtcol: colnr_T =
                                                    (*wp).w_virtcol + 1 as colnr_T;
                                                if opt as ::core::ffi::c_int
                                                    == STL_VIRTCOL_ALT as ::core::ffi::c_int
                                                    && virtcol
                                                        == (if State.get() & MODE_INSERT
                                                            == 0 as ::core::ffi::c_int
                                                            && empty_line as ::core::ffi::c_int != 0
                                                        {
                                                            0 as ::core::ffi::c_int
                                                        } else {
                                                            (*wp).w_cursor.col
                                                                + 1 as ::core::ffi::c_int
                                                        })
                                                {
                                                    break 's_1848;
                                                } else {
                                                    num = virtcol as ::core::ffi::c_int;
                                                    break 's_1848;
                                                }
                                            }
                                            112 => {
                                                num = calc_percentage(
                                                    (*wp).w_cursor.lnum as int64_t,
                                                    (*(*wp).w_buffer).b_ml.ml_line_count as int64_t,
                                                );
                                                break 's_1848;
                                            }
                                            80 => {
                                                get_rel_pos(
                                                    wp,
                                                    &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                    TMPLEN,
                                                );
                                                str = &raw mut buf_tmp as *mut ::core::ffi::c_char;
                                                break 's_1848;
                                            }
                                            83 => {
                                                if p_sc.get() != 0
                                                    && (opt_idx as ::core::ffi::c_int
                                                        == kOptInvalid as ::core::ffi::c_int
                                                        || find_option(p_sloc.get())
                                                            as ::core::ffi::c_int
                                                            == opt_idx as ::core::ffi::c_int)
                                                {
                                                    str = showcmd_buf.ptr()
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                break 's_1848;
                                            }
                                            97 => {
                                                fillable = false_0 != 0;
                                                buf_tmp[0] = NUL as ::core::ffi::c_char;
                                                if append_arg_number(
                                                    wp,
                                                    &raw mut buf_tmp as *mut ::core::ffi::c_char,
                                                    ::core::mem::size_of::<[::core::ffi::c_char; 70]>(
                                                    ),
                                                ) > 0 as ::core::ffi::c_int
                                                {
                                                    str = &raw mut buf_tmp
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                break 's_1848;
                                            }
                                            107 => {
                                                fillable = false_0 != 0;
                                                if let Some(keymap_name) = keymap_str(wp) {
                                                    let plen = vim_snprintf(
                                                        buf_tmp.as_mut_ptr(),
                                                        TMPLEN as size_t,
                                                        c"<%s>".as_ptr() as *const _,
                                                        keymap_name.as_ptr(),
                                                    );
                                                    if plen > 0 && plen <= TMPLEN - 1 {
                                                        str = buf_tmp.as_mut_ptr();
                                                    }
                                                }
                                                break 's_1848;
                                            }
                                            78 => {
                                                num = 0 as ::core::ffi::c_int;
                                                break 's_1848;
                                            }
                                            110 => {
                                                num =
                                                    (*(*wp).w_buffer).handle as ::core::ffi::c_int;
                                                break 's_1848;
                                            }
                                            79 => {
                                                base = kNumBaseHexadecimal;
                                                break 's_1418;
                                            }
                                            111 => {
                                                break 's_1418;
                                            }
                                            66 => {
                                                base = kNumBaseHexadecimal;
                                            }
                                            98 => {}
                                            114 | 82 => {
                                                itemisflag = true_0 != 0;
                                                if (*(*wp).w_buffer).b_p_ro != 0 {
                                                    str = (if opt as ::core::ffi::c_int
                                                        == STL_ROFLAG_ALT as ::core::ffi::c_int
                                                    {
                                                        c",RO".as_ptr()
                                                    } else {
                                                        gettext(c"[RO]".as_ptr())
                                                            as *const ::core::ffi::c_char
                                                    })
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                break 's_1848;
                                            }
                                            104 | 72 => {
                                                itemisflag = true_0 != 0;
                                                if (*(*wp).w_buffer).b_help {
                                                    str = (if opt as ::core::ffi::c_int
                                                        == STL_HELPFLAG_ALT as ::core::ffi::c_int
                                                    {
                                                        c",HLP".as_ptr()
                                                    } else {
                                                        gettext(c"[Help]".as_ptr())
                                                            as *const ::core::ffi::c_char
                                                    })
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                break 's_1848;
                                            }
                                            67 => {
                                                break '_stcsign;
                                            }
                                            115 => {
                                                break '_stcsign;
                                            }
                                            121 => {
                                                if *(*(*wp).w_buffer).b_p_ft as ::core::ffi::c_int
                                                    != NUL
                                                    && strlen((*(*wp).w_buffer).b_p_ft)
                                                        < (TMPLEN - 3 as ::core::ffi::c_int)
                                                            as size_t
                                                {
                                                    vim_snprintf(
                                                        &raw mut buf_tmp
                                                            as *mut ::core::ffi::c_char,
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 70],
                                                        >(
                                                        ),
                                                        c"[%s]".as_ptr(),
                                                        (*(*wp).w_buffer).b_p_ft,
                                                    );
                                                    str = &raw mut buf_tmp
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                break 's_1848;
                                            }
                                            89 => {
                                                itemisflag = true_0 != 0;
                                                if *(*(*wp).w_buffer).b_p_ft as ::core::ffi::c_int
                                                    != NUL
                                                    && strlen((*(*wp).w_buffer).b_p_ft)
                                                        < (TMPLEN - 2 as ::core::ffi::c_int)
                                                            as size_t
                                                {
                                                    vim_snprintf(
                                                        &raw mut buf_tmp
                                                            as *mut ::core::ffi::c_char,
                                                        ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 70],
                                                        >(
                                                        ),
                                                        c",%s".as_ptr(),
                                                        (*(*wp).w_buffer).b_p_ft,
                                                    );
                                                    let mut t_3: *mut ::core::ffi::c_char =
                                                        &raw mut buf_tmp
                                                            as *mut ::core::ffi::c_char;
                                                    while *t_3 as ::core::ffi::c_int
                                                        != 0 as ::core::ffi::c_int
                                                    {
                                                        *t_3 = toupper(
                                                            *t_3 as uint8_t as ::core::ffi::c_int,
                                                        )
                                                            as ::core::ffi::c_char;
                                                        t_3 = t_3.offset(1);
                                                    }
                                                    str = &raw mut buf_tmp
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                break 's_1848;
                                            }
                                            119 | 87 => {
                                                itemisflag = true_0 != 0;
                                                if (*wp).w_onebuf_opt.wo_pvw != 0 {
                                                    str = (if opt as ::core::ffi::c_int
                                                        == STL_PREVIEWFLAG_ALT as ::core::ffi::c_int
                                                    {
                                                        c",PRV".as_ptr()
                                                    } else {
                                                        gettext(c"[Preview]".as_ptr())
                                                            as *const ::core::ffi::c_char
                                                    })
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                break 's_1848;
                                            }
                                            113 => {
                                                if bt_quickfix((*wp).w_buffer) {
                                                    str = if !(*wp).w_llist_ref.is_null() {
                                                        gettext(msg_loclist.get())
                                                    } else {
                                                        gettext(msg_qflist.get())
                                                    };
                                                }
                                                break 's_1848;
                                            }
                                            109 | 77 => {
                                                itemisflag = true_0 != 0;
                                                match (opt as ::core::ffi::c_int
                                                    == STL_MODIFIED_ALT as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                                    + bufIsChanged((*wp).w_buffer)
                                                        as ::core::ffi::c_int
                                                        * 2 as ::core::ffi::c_int
                                                    + ((*(*wp).w_buffer).b_p_ma == 0)
                                                        as ::core::ffi::c_int
                                                        * 4 as ::core::ffi::c_int
                                                {
                                                    2 => {
                                                        str = c"[+]".as_ptr()
                                                            as *mut ::core::ffi::c_char;
                                                    }
                                                    3 => {
                                                        str = c",+".as_ptr()
                                                            as *mut ::core::ffi::c_char;
                                                    }
                                                    4 => {
                                                        str = c"[-]".as_ptr()
                                                            as *mut ::core::ffi::c_char;
                                                    }
                                                    5 => {
                                                        str = c",-".as_ptr()
                                                            as *mut ::core::ffi::c_char;
                                                    }
                                                    6 => {
                                                        str = c"[+-]".as_ptr()
                                                            as *mut ::core::ffi::c_char;
                                                    }
                                                    7 => {
                                                        str = c",+-".as_ptr()
                                                            as *mut ::core::ffi::c_char;
                                                    }
                                                    _ => {}
                                                }
                                                break 's_1848;
                                            }
                                            36 | 35 => {
                                                let mut t_4: *mut ::core::ffi::c_char = fmt_p;
                                                while *fmt_p as ::core::ffi::c_int
                                                    != opt as ::core::ffi::c_int
                                                    && *fmt_p as ::core::ffi::c_int != NUL
                                                {
                                                    fmt_p = fmt_p.offset(1);
                                                }
                                                if *fmt_p as ::core::ffi::c_int
                                                    == opt as ::core::ffi::c_int
                                                {
                                                    (*(*stl_items.ptr())
                                                        .offset(curitem.get() as isize))
                                                    .type_0 = (if opt as ::core::ffi::c_int
                                                        == STL_HIGHLIGHT_COMB as ::core::ffi::c_int
                                                    {
                                                        HighlightCombining as ::core::ffi::c_int
                                                    } else {
                                                        Highlight as ::core::ffi::c_int
                                                    })
                                                        as C2Rust_Unnamed_15;
                                                    (*(*stl_items.ptr())
                                                        .offset(curitem.get() as isize))
                                                    .start = out_p;
                                                    (*(*stl_items.ptr())
                                                        .offset(curitem.get() as isize))
                                                    .minwid = -syn_name2id_len(
                                                        t_4,
                                                        fmt_p.offset_from(t_4) as size_t,
                                                    );
                                                    (*curitem.ptr()) += 1;
                                                    fmt_p = fmt_p.offset(1);
                                                }
                                                continue 's_2297;
                                            }
                                            _ => {
                                                break 's_1848;
                                            }
                                        }
                                        num = byteval;
                                        if num == NL {
                                            num = 0 as ::core::ffi::c_int;
                                        } else if num == CAR
                                            && get_fileformat((*wp).w_buffer) == EOL_MAC
                                        {
                                            num = NL;
                                        }
                                        break 's_1848;
                                    }
                                    let mut l: ::core::ffi::c_int = ml_find_line_or_offset(
                                        (*wp).w_buffer,
                                        (*wp).w_cursor.lnum,
                                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                        false_0 != 0,
                                    );
                                    num = if (*(*wp).w_buffer).b_ml.ml_flags & ML_EMPTY != 0
                                        || l < 0 as ::core::ffi::c_int
                                    {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        l + 1 as ::core::ffi::c_int
                                            + (if State.get() & MODE_INSERT
                                                == 0 as ::core::ffi::c_int
                                                && empty_line as ::core::ffi::c_int != 0
                                            {
                                                0 as ::core::ffi::c_int
                                            } else {
                                                (*wp).w_cursor.col
                                            })
                                    };
                                    break 's_1848;
                                }
                                if !stcp.is_null() {
                                    let mut fdc: ::core::ffi::c_int = if opt as ::core::ffi::c_int
                                        == STL_FOLDCOL as ::core::ffi::c_int
                                    {
                                        compute_foldcolumn(wp, 0 as ::core::ffi::c_int)
                                    } else {
                                        0 as ::core::ffi::c_int
                                    };
                                    let mut width: ::core::ffi::c_int = if opt as ::core::ffi::c_int
                                        == STL_FOLDCOL as ::core::ffi::c_int
                                    {
                                        (fdc > 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                                    } else if opt as ::core::ffi::c_int
                                        == STL_SIGNCOL as ::core::ffi::c_int
                                    {
                                        (*wp).w_scwidth
                                    } else {
                                        1 as ::core::ffi::c_int
                                    };
                                    if width > 0 as ::core::ffi::c_int {
                                        foldsignitem = curitem.get();
                                        lnum = get_vim_var_nr(VV_LNUM) as linenr_T;
                                        if fdc > 0 as ::core::ffi::c_int {
                                            let mut fold_buf: [schar_T; 9] = [0; 9];
                                            fill_foldcolumn(
                                                wp,
                                                (*stcp).foldinfo,
                                                (*stcp).lnum,
                                                fdc,
                                                get_vim_var_nr(VV_VIRTNUM) < 0 as varnumber_T,
                                                &raw mut (*stcp).fold_vcol as *mut colnr_T,
                                                &raw mut fold_buf as *mut schar_T,
                                            );
                                            (*(*stl_items.ptr()).offset(curitem.get() as isize))
                                                .minwid = -if use_cursor_line_highlight(wp, lnum)
                                                as ::core::ffi::c_int
                                                != 0
                                            {
                                                HLF_CLF
                                            } else {
                                                HLF_FC
                                            };
                                            let mut buflen: size_t = 0 as size_t;
                                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                            while i < fdc {
                                                buflen = buflen.wrapping_add(schar_get(
                                                    (&raw mut buf_tmp as *mut ::core::ffi::c_char)
                                                        .add(buflen),
                                                    fold_buf[i as usize],
                                                ));
                                                i += 1;
                                            }
                                        }
                                        let mut signlen: size_t = 0 as size_t;
                                        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                        while i_0 < width {
                                            (*(*stl_items.ptr()).offset(curitem.get() as isize))
                                                .start = out_p.add(signlen);
                                            if fdc == 0 as ::core::ffi::c_int {
                                                let mut sattr: SignTextAttrs =
                                                    *(*stcp).sattrs.offset(i_0 as isize);
                                                if sattr.text[0 as ::core::ffi::c_int as usize] != 0
                                                    && get_vim_var_nr(VV_VIRTNUM)
                                                        == 0 as varnumber_T
                                                {
                                                    signlen =
                                                        signlen.wrapping_add(describe_sign_text(
                                                            (&raw mut buf_tmp
                                                                as *mut ::core::ffi::c_char)
                                                                .add(signlen),
                                                            &raw mut sattr.text as *mut schar_T,
                                                        ));
                                                    (*(*stl_items.ptr())
                                                        .offset(curitem.get() as isize))
                                                    .minwid = -if (*stcp).sign_cul_id != 0 {
                                                        (*stcp).sign_cul_id
                                                    } else {
                                                        sattr.hl_id
                                                    };
                                                } else {
                                                    let c2rust_fresh19 = signlen;
                                                    signlen = signlen.wrapping_add(1);
                                                    buf_tmp[c2rust_fresh19 as usize] =
                                                        ' ' as ::core::ffi::c_char;
                                                    let c2rust_fresh20 = signlen;
                                                    signlen = signlen.wrapping_add(1);
                                                    buf_tmp[c2rust_fresh20 as usize] =
                                                        ' ' as ::core::ffi::c_char;
                                                    buf_tmp[signlen as usize] =
                                                        NUL as ::core::ffi::c_char;
                                                    (*(*stl_items.ptr())
                                                        .offset(curitem.get() as isize))
                                                    .minwid = 0 as ::core::ffi::c_int;
                                                }
                                            }
                                            let c2rust_fresh21 = curitem.get();
                                            curitem.set(curitem.get() + 1);
                                            (*(*stl_items.ptr()).offset(c2rust_fresh21 as isize))
                                                .type_0 = (if fdc > 0 as ::core::ffi::c_int {
                                                HighlightFold as ::core::ffi::c_int
                                            } else {
                                                HighlightSign as ::core::ffi::c_int
                                            })
                                                as C2Rust_Unnamed_15;
                                            i_0 += 1;
                                        }
                                        str = &raw mut buf_tmp as *mut ::core::ffi::c_char;
                                    }
                                }
                            }
                            (*(*stl_items.ptr()).offset(curitem.get() as isize)).start = out_p;
                            (*(*stl_items.ptr()).offset(curitem.get() as isize)).type_0 = Normal;
                            if !str.is_null() && *str as ::core::ffi::c_int != 0 {
                                let mut t_5: *mut ::core::ffi::c_char = str;
                                if itemisflag {
                                    if *t_5.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        != 0
                                        && *t_5.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            != 0
                                        && (!prevchar_isitem
                                            && *t_5 as ::core::ffi::c_int
                                                == ',' as ::core::ffi::c_int
                                            || prevchar_isflag as ::core::ffi::c_int != 0
                                                && *t_5 as ::core::ffi::c_int
                                                    == ' ' as ::core::ffi::c_int)
                                    {
                                        t_5 = t_5.offset(1);
                                    }
                                    prevchar_isflag = true_0 != 0;
                                }
                                let mut l_0: ::core::ffi::c_int = vim_strsize(t_5);
                                if l_0 > 0 as ::core::ffi::c_int {
                                    prevchar_isitem = true_0 != 0;
                                }
                                if l_0 > maxwid_0 {
                                    while l_0 >= maxwid_0 {
                                        l_0 -= ptr2cells(t_5);
                                        t_5 = t_5.offset(utfc_ptr2len(t_5) as isize);
                                    }
                                    if out_p >= out_end_p {
                                        break;
                                    }
                                    let c2rust_fresh22 = out_p;
                                    out_p = out_p.offset(1);
                                    *c2rust_fresh22 = '<' as ::core::ffi::c_char;
                                }
                                if minwid_0 > 0 as ::core::ffi::c_int {
                                    while l_0 < minwid_0 && out_p < out_end_p {
                                        if l_0 + 1 as ::core::ffi::c_int == minwid_0
                                            && fillchar == '-' as schar_T
                                            && ascii_isdigit(*t_5 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                                != 0
                                        {
                                            let c2rust_fresh23 = out_p;
                                            out_p = out_p.offset(1);
                                            *c2rust_fresh23 = ' ' as ::core::ffi::c_char;
                                        } else {
                                            schar_get_adv(&raw mut out_p, fillchar);
                                        }
                                        l_0 += 1;
                                    }
                                    minwid_0 = 0 as ::core::ffi::c_int;
                                    if foldsignitem >= 0 as ::core::ffi::c_int {
                                        let mut offset: ptrdiff_t = out_p.offset_from(
                                            (*(*stl_items.ptr()).offset(foldsignitem as isize))
                                                .start,
                                        );
                                        let mut i_1: ::core::ffi::c_int = foldsignitem;
                                        while i_1 < curitem.get() {
                                            (*(*stl_items.ptr()).offset(i_1 as isize)).start =
                                                (*(*stl_items.ptr()).offset(i_1 as isize))
                                                    .start
                                                    .offset(offset as isize);
                                            i_1 += 1;
                                        }
                                    }
                                } else {
                                    minwid_0 *= -1 as ::core::ffi::c_int;
                                }
                                while *t_5 as ::core::ffi::c_int != 0 && out_p < out_end_p {
                                    if fillable as ::core::ffi::c_int != 0
                                        && *t_5 as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                                        && (!ascii_isdigit(
                                            *t_5.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        ) || fillchar != '-' as schar_T)
                                    {
                                        schar_get_adv(&raw mut out_p, fillchar);
                                    } else {
                                        let c2rust_fresh24 = out_p;
                                        out_p = out_p.offset(1);
                                        *c2rust_fresh24 = *t_5;
                                    }
                                    t_5 = t_5.offset(1);
                                }
                                if foldsignitem >= 0 as ::core::ffi::c_int {
                                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).type_0 =
                                        Highlight;
                                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).start =
                                        out_p;
                                    (*(*stl_items.ptr()).offset(curitem.get() as isize)).minwid =
                                        0 as ::core::ffi::c_int;
                                }
                                while l_0 < minwid_0 && out_p < out_end_p {
                                    schar_get_adv(&raw mut out_p, fillchar);
                                    l_0 += 1;
                                }
                            } else if num >= 0 as ::core::ffi::c_int {
                                if out_p.offset(20 as ::core::ffi::c_int as isize) > out_end_p {
                                    break;
                                }
                                prevchar_isitem = true_0 != 0;
                                let mut nstr: [::core::ffi::c_char; 20] = [0; 20];
                                let mut t_6: *mut ::core::ffi::c_char =
                                    &raw mut nstr as *mut ::core::ffi::c_char;
                                if opt as ::core::ffi::c_int
                                    == STL_VIRTCOL_ALT as ::core::ffi::c_int
                                {
                                    let c2rust_fresh25 = t_6;
                                    t_6 = t_6.offset(1);
                                    *c2rust_fresh25 = '-' as ::core::ffi::c_char;
                                    minwid_0 -= 1;
                                }
                                let c2rust_fresh26 = t_6;
                                t_6 = t_6.offset(1);
                                *c2rust_fresh26 = '%' as ::core::ffi::c_char;
                                if zeropad {
                                    let c2rust_fresh27 = t_6;
                                    t_6 = t_6.offset(1);
                                    *c2rust_fresh27 = '0' as ::core::ffi::c_char;
                                }
                                let c2rust_fresh28 = t_6;
                                t_6 = t_6.offset(1);
                                *c2rust_fresh28 = '*' as ::core::ffi::c_char;
                                let c2rust_fresh29 = t_6;
                                t_6 = t_6.offset(1);
                                *c2rust_fresh29 = (if base as ::core::ffi::c_uint
                                    == kNumBaseHexadecimal as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                {
                                    'X' as ::core::ffi::c_int
                                } else {
                                    'd' as ::core::ffi::c_int
                                })
                                    as ::core::ffi::c_char;
                                *t_6 = NUL as ::core::ffi::c_char;
                                let mut num_chars: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                                let mut n_3: ::core::ffi::c_int = num;
                                while n_3 >= base as ::core::ffi::c_int {
                                    num_chars += 1;
                                    n_3 /= base as ::core::ffi::c_int;
                                }
                                if opt as ::core::ffi::c_int
                                    == STL_VIRTCOL_ALT as ::core::ffi::c_int
                                {
                                    num_chars += 1;
                                }
                                debug_assert!(out_end_p >= out_p, "out_end_p >= out_p");
                                let mut remaining_buf_len: size_t = (out_end_p.offset_from(out_p)
                                    as size_t)
                                    .wrapping_add(1 as size_t);
                                if num_chars > maxwid_0 {
                                    num_chars += 2 as ::core::ffi::c_int;
                                    let mut n_4: ::core::ffi::c_int = num_chars - maxwid_0;
                                    loop {
                                        let c2rust_fresh30 = num_chars;
                                        num_chars = num_chars - 1;
                                        if c2rust_fresh30 <= maxwid_0 {
                                            break;
                                        }
                                        num /= base as ::core::ffi::c_int;
                                    }
                                    let c2rust_fresh31 = t_6;
                                    t_6 = t_6.offset(1);
                                    *c2rust_fresh31 = '>' as ::core::ffi::c_char;
                                    let c2rust_fresh32 = t_6;
                                    t_6 = t_6.offset(1);
                                    *c2rust_fresh32 = '%' as ::core::ffi::c_char;
                                    *t_6 = *t_6.offset(-3 as ::core::ffi::c_int as isize);
                                    t_6 = t_6.offset(1);
                                    *t_6 = NUL as ::core::ffi::c_char;
                                    out_p = out_p.add(vim_snprintf_safelen(
                                        out_p,
                                        remaining_buf_len,
                                        &raw mut nstr as *mut ::core::ffi::c_char,
                                        0 as ::core::ffi::c_int,
                                        num,
                                        n_4,
                                    ));
                                } else {
                                    out_p = out_p.add(vim_snprintf_safelen(
                                        out_p,
                                        remaining_buf_len,
                                        &raw mut nstr as *mut ::core::ffi::c_char,
                                        minwid_0,
                                        num,
                                    ));
                                }
                            } else {
                                (*(*stl_items.ptr()).offset(curitem.get() as isize)).type_0 = Empty;
                            }
                            if num >= 0 as ::core::ffi::c_int
                                || !itemisflag && !str.is_null() && *str as ::core::ffi::c_int != 0
                            {
                                prevchar_isflag = false_0 != 0;
                            }
                            if opt as ::core::ffi::c_int == STL_VIM_EXPR as ::core::ffi::c_int {
                                let mut ptr__1: *mut *mut ::core::ffi::c_void =
                                    &raw mut str as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr__1);
                                *ptr__1 = NULL;
                                let _ = *ptr__1;
                            }
                            (*curitem.ptr()) += 1;
                            if left_align_num {
                                (*(*stl_items.ptr()).offset(curitem.get() as isize)).type_0 =
                                    Separate;
                                let c2rust_fresh33 = curitem.get();
                                curitem.set(curitem.get() + 1);
                                let c2rust_lvalue_ptr_2 = &raw mut (*(*stl_items.ptr())
                                    .offset(c2rust_fresh33 as isize))
                                .start;
                                *c2rust_lvalue_ptr_2 = out_p;
                            }
                        }
                    }
                }
            }
        }
        *out_p = NUL as ::core::ffi::c_char;
        let mut outputlen: size_t = out_p.offset_from(out) as size_t;
        let mut itemcnt: ::core::ffi::c_int = curitem.get() - evalstart;
        curitem.set(evalstart);
        if usefmt != fmt {
            xfree(usefmt as *mut ::core::ffi::c_void);
        }
        let mut width_0: ::core::ffi::c_int = vim_strsize(out);
        if maxwidth > 0 as ::core::ffi::c_int
            && width_0 > maxwidth
            && (stcp.is_null()
                || width_0
                    > MAX_NUMBERWIDTH
                        + SIGN_SHOW_MAX * SIGN_WIDTH as ::core::ffi::c_int
                        + 9 as ::core::ffi::c_int)
        {
            let mut item_idx: ::core::ffi::c_int = evalstart;
            let mut trunc_p: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            if itemcnt == 0 as ::core::ffi::c_int {
                trunc_p = out;
            } else {
                trunc_p = (*(*stl_items.ptr()).offset(item_idx as isize)).start;
                let mut i_2: ::core::ffi::c_int = evalstart;
                while i_2 < itemcnt + evalstart {
                    if (*(*stl_items.ptr()).offset(i_2 as isize)).type_0 as ::core::ffi::c_uint
                        == Trunc as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        trunc_p = (*(*stl_items.ptr()).offset(i_2 as isize)).start;
                        item_idx = i_2;
                        break;
                    } else {
                        i_2 += 1;
                    }
                }
            }
            if width_0 - vim_strsize(trunc_p) >= maxwidth {
                trunc_p = out;
                width_0 = 0 as ::core::ffi::c_int;
                loop {
                    width_0 += ptr2cells(trunc_p);
                    if width_0 >= maxwidth {
                        break;
                    }
                    trunc_p = trunc_p.offset(utfc_ptr2len(trunc_p) as isize);
                }
                let mut i_3: ::core::ffi::c_int = evalstart;
                while i_3 < itemcnt + evalstart {
                    if (*(*stl_items.ptr()).offset(i_3 as isize)).start > trunc_p {
                        let mut j: ::core::ffi::c_int = i_3;
                        while j < itemcnt + evalstart {
                            if (*(*stl_items.ptr()).offset(j as isize)).type_0
                                as ::core::ffi::c_uint
                                == ClickFunc as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                let mut ptr__2: *mut *mut ::core::ffi::c_void =
                                    &raw mut (*(*stl_items.ptr()).offset(j as isize)).cmd
                                        as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr__2);
                                *ptr__2 = NULL;
                                let _ = *ptr__2;
                            }
                            j += 1;
                        }
                        itemcnt = i_3;
                        break;
                    } else {
                        i_3 += 1;
                    }
                }
                let c2rust_fresh34 = trunc_p;
                trunc_p = trunc_p.offset(1);
                *c2rust_fresh34 = '>' as ::core::ffi::c_char;
                *trunc_p = NUL as ::core::ffi::c_char;
            } else {
                let mut end: *mut ::core::ffi::c_char = out.add(outputlen);
                let mut trunc_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while width_0 >= maxwidth {
                    width_0 -= ptr2cells(trunc_p.offset(trunc_len as isize));
                    trunc_len += utfc_ptr2len(trunc_p.offset(trunc_len as isize));
                }
                let mut trunc_end_p: *mut ::core::ffi::c_char = trunc_p.offset(trunc_len as isize);
                memmove(
                    trunc_p.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    trunc_end_p as *const ::core::ffi::c_void,
                    (end.offset_from(trunc_end_p) as size_t).wrapping_add(1 as size_t),
                );
                end = end.offset(
                    -(trunc_end_p.offset_from(trunc_p.offset(1 as ::core::ffi::c_int as isize))
                        as size_t as isize),
                );
                *trunc_p = '<' as ::core::ffi::c_char;
                let mut item_offset: ::core::ffi::c_int = trunc_len - 1 as ::core::ffi::c_int;
                let mut i_4: ::core::ffi::c_int = item_idx;
                while i_4 < itemcnt + evalstart {
                    if (*(*stl_items.ptr()).offset(i_4 as isize)).start >= trunc_end_p {
                        (*(*stl_items.ptr()).offset(i_4 as isize)).start = (*(*stl_items.ptr())
                            .offset(i_4 as isize))
                        .start
                        .offset(-(item_offset as isize));
                    } else {
                        (*(*stl_items.ptr()).offset(i_4 as isize)).start = trunc_p;
                    }
                    i_4 += 1;
                }
                if (width_0 + 1 as ::core::ffi::c_int) < maxwidth {
                    trunc_p = end;
                }
                loop {
                    width_0 += 1;
                    if width_0 >= maxwidth {
                        break;
                    }
                    schar_get_adv(&raw mut trunc_p, fillchar);
                    end = trunc_p;
                }
            }
            width_0 = maxwidth;
        } else if width_0 < maxwidth
            && outputlen
                .wrapping_add(((maxwidth - width_0) as size_t).wrapping_mul(schar_len(fillchar)))
                .wrapping_add(1 as size_t)
                < outlen
        {
            let mut num_separators: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut i_5: ::core::ffi::c_int = evalstart;
            while i_5 < itemcnt + evalstart {
                if (*(*stl_items.ptr()).offset(i_5 as isize)).type_0 as ::core::ffi::c_uint
                    == Separate as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    *(*stl_separator_locations.ptr()).offset(num_separators as isize) = i_5;
                    num_separators += 1;
                }
                i_5 += 1;
            }
            if num_separators != 0 {
                let mut standard_spaces: ::core::ffi::c_int = (maxwidth - width_0) / num_separators;
                let mut final_spaces: ::core::ffi::c_int = maxwidth
                    - width_0
                    - standard_spaces * (num_separators - 1 as ::core::ffi::c_int);
                let mut l_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while l_1 < num_separators {
                    let mut dislocation: ::core::ffi::c_int =
                        if l_1 == num_separators - 1 as ::core::ffi::c_int {
                            final_spaces
                        } else {
                            standard_spaces
                        };
                    dislocation *= schar_len(fillchar) as ::core::ffi::c_int;
                    let mut start: *mut ::core::ffi::c_char = (*(*stl_items.ptr())
                        .offset(*(*stl_separator_locations.ptr()).offset(l_1 as isize) as isize))
                    .start;
                    let mut seploc: *mut ::core::ffi::c_char = start.offset(dislocation as isize);
                    memmove(
                        seploc as *mut ::core::ffi::c_void,
                        start as *const ::core::ffi::c_void,
                        strlen(start).wrapping_add(1 as size_t),
                    );
                    let mut s: *mut ::core::ffi::c_char = start;
                    while s < seploc {
                        schar_get_adv(&raw mut s, fillchar);
                    }
                    let mut item_idx_0: ::core::ffi::c_int = *(*stl_separator_locations.ptr())
                        .offset(l_1 as isize)
                        + 1 as ::core::ffi::c_int;
                    while item_idx_0 < itemcnt + evalstart {
                        (*(*stl_items.ptr()).offset(item_idx_0 as isize)).start =
                            (*(*stl_items.ptr()).offset(item_idx_0 as isize))
                                .start
                                .offset(dislocation as isize);
                        item_idx_0 += 1;
                    }
                    l_1 += 1;
                }
                width_0 = maxwidth;
            }
        }
        if !hltab.is_null() {
            *hltab = stl_hltab.get();
            let mut sp: *mut stl_hlrec_t = stl_hltab.get();
            let mut l_2: ::core::ffi::c_int = evalstart;
            while l_2 < itemcnt + evalstart {
                if (*(*stl_items.ptr()).offset(l_2 as isize)).type_0 as ::core::ffi::c_uint
                    == Highlight as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*(*stl_items.ptr()).offset(l_2 as isize)).type_0 as ::core::ffi::c_uint
                        == HighlightCombining as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*(*stl_items.ptr()).offset(l_2 as isize)).type_0 as ::core::ffi::c_uint
                        == HighlightFold as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*(*stl_items.ptr()).offset(l_2 as isize)).type_0 as ::core::ffi::c_uint
                        == HighlightSign as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    (*sp).start = (*(*stl_items.ptr()).offset(l_2 as isize)).start;
                    (*sp).userhl = (*(*stl_items.ptr()).offset(l_2 as isize)).minwid;
                    let mut type_0: ::core::ffi::c_uint =
                        (*(*stl_items.ptr()).offset(l_2 as isize)).type_0 as ::core::ffi::c_uint;
                    (*sp).item = (if type_0
                        == HighlightSign as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        STL_SIGNCOL as ::core::ffi::c_int
                    } else if type_0 == HighlightFold as ::core::ffi::c_int as ::core::ffi::c_uint {
                        STL_FOLDCOL as ::core::ffi::c_int
                    } else if type_0
                        == HighlightCombining as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        STL_HIGHLIGHT_COMB as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as StlFlag;
                    sp = sp.offset(1);
                }
                l_2 += 1;
            }
            (*sp).start = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*sp).userhl = 0 as ::core::ffi::c_int;
        }
        if !hltab_len.is_null() {
            *hltab_len = itemcnt as size_t;
        }
        if !tabtab.is_null() {
            *tabtab = stl_tabtab.get();
            let mut cur_tab_rec: *mut StlClickRecord = stl_tabtab.get();
            let mut l_3: ::core::ffi::c_int = evalstart;
            while l_3 < itemcnt + evalstart {
                if (*(*stl_items.ptr()).offset(l_3 as isize)).type_0 as ::core::ffi::c_uint
                    == TabPage as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    (*cur_tab_rec).start = (*(*stl_items.ptr()).offset(l_3 as isize)).start;
                    if (*(*stl_items.ptr()).offset(l_3 as isize)).minwid == 0 as ::core::ffi::c_int
                    {
                        (*cur_tab_rec).def.type_0 = kStlClickDisabled;
                        (*cur_tab_rec).def.tabnr = 0 as ::core::ffi::c_int;
                    } else {
                        let mut tabnr: ::core::ffi::c_int =
                            (*(*stl_items.ptr()).offset(l_3 as isize)).minwid;
                        if (*(*stl_items.ptr()).offset(l_3 as isize)).minwid
                            > 0 as ::core::ffi::c_int
                        {
                            (*cur_tab_rec).def.type_0 = kStlClickTabSwitch;
                        } else {
                            (*cur_tab_rec).def.type_0 = kStlClickTabClose;
                            tabnr = -tabnr;
                        }
                        (*cur_tab_rec).def.tabnr = tabnr;
                    }
                    (*cur_tab_rec).def.func = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    cur_tab_rec = cur_tab_rec.offset(1);
                } else if (*(*stl_items.ptr()).offset(l_3 as isize)).type_0 as ::core::ffi::c_uint
                    == ClickFunc as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    (*cur_tab_rec).start = (*(*stl_items.ptr()).offset(l_3 as isize)).start;
                    (*cur_tab_rec).def.type_0 = kStlClickFuncRun;
                    (*cur_tab_rec).def.tabnr = (*(*stl_items.ptr()).offset(l_3 as isize)).minwid;
                    (*cur_tab_rec).def.func = (*(*stl_items.ptr()).offset(l_3 as isize)).cmd;
                    cur_tab_rec = cur_tab_rec.offset(1);
                }
                l_3 += 1;
            }
            (*cur_tab_rec).start = ::core::ptr::null::<::core::ffi::c_char>();
            (*cur_tab_rec).def.type_0 = kStlClickDisabled;
            (*cur_tab_rec).def.tabnr = 0 as ::core::ffi::c_int;
            (*cur_tab_rec).def.func = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        redraw_not_allowed.set(save_redraw_not_allowed);
        if opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int
            && did_emsg.get() > did_emsg_before
        {
            set_option_direct(
                opt_idx,
                get_option_default(opt_idx, opt_scope),
                opt_scope,
                SID_ERROR,
            );
        }
        KeyTyped.set(save_KeyTyped);
        return width_0;
    }
}
