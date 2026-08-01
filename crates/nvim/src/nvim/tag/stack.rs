//! The tag stack.
//!
//! Each window remembers where it jumped from, so `CTRL-T` and `:pop` can
//! walk back. [`do_tags`] prints the stack, [`get_tagstack`] and
//! [`set_tagstack`] are the Vimscript views, and the `tagstack_*` family is
//! the pushing, shifting and freeing underneath them.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe fn do_tags(mut _eap: *mut exarg_T) {
    unsafe {
        let mut tagstack: *mut taggy_T = &raw mut (*curwin.get()).w_tagstack as *mut taggy_T;
        let mut tagstackidx: ::core::ffi::c_int = (*curwin.get()).w_tagstackidx;
        let mut tagstacklen: ::core::ffi::c_int = (*curwin.get()).w_tagstacklen;
        msg_puts_title(gettext(
            b"\n  # TO tag         FROM line  in file/text\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < tagstacklen {
            if !(*tagstack.offset(i as isize)).tagname.is_null() {
                let mut name: *mut ::core::ffi::c_char = fm_getname(
                    &raw mut (*tagstack.offset(i as isize)).fmark,
                    30 as ::core::ffi::c_int,
                );
                if !name.is_null() {
                    msg_putchar('\n' as ::core::ffi::c_int);
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        b"%c%2d %2d %-15s %5d  \0".as_ptr() as *const ::core::ffi::c_char,
                        if i == tagstackidx {
                            '>' as ::core::ffi::c_int
                        } else {
                            ' ' as ::core::ffi::c_int
                        },
                        i + 1 as ::core::ffi::c_int,
                        (*tagstack.offset(i as isize)).cur_match + 1 as ::core::ffi::c_int,
                        (*tagstack.offset(i as isize)).tagname,
                        (*tagstack.offset(i as isize)).fmark.mark.lnum,
                    );
                    msg_outtrans(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                    msg_outtrans(
                        name,
                        if (*tagstack.offset(i as isize)).fmark.fnum == (*curbuf.get()).handle {
                            HLF_D as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        },
                        false_0 != 0,
                    );
                    xfree(name as *mut ::core::ffi::c_void);
                }
            }
            i += 1;
        }
        if tagstackidx == tagstacklen {
            msg_puts(b"\n>\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
}

pub unsafe extern "C" fn tagstack_clear_entry(mut item: *mut taggy_T) {
    unsafe {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*item).tagname as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*item).user_data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
    }
}

pub unsafe extern "C" fn get_tagstack(mut wp: *mut win_T, mut retdict: *mut dict_T) {
    unsafe {
        tv_dict_add_nr(
            retdict,
            b"length\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            (*wp).w_tagstacklen as varnumber_T,
        );
        tv_dict_add_nr(
            retdict,
            b"curidx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            ((*wp).w_tagstackidx + 1 as ::core::ffi::c_int) as varnumber_T,
        );
        let mut l: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
        tv_dict_add_list(
            retdict,
            b"items\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            l,
        );
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*wp).w_tagstacklen {
            let mut d: *mut dict_T = tv_dict_alloc();
            tv_list_append_dict(l, d);
            get_tag_details(
                (&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize),
                d,
            );
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn tagstack_clear(mut wp: *mut win_T) {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*wp).w_tagstacklen {
            tagstack_clear_entry((&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize));
            i += 1;
        }
        (*wp).w_tagstacklen = 0 as ::core::ffi::c_int;
        (*wp).w_tagstackidx = 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn tagstack_shift(mut wp: *mut win_T) {
    unsafe {
        let mut tagstack: *mut taggy_T = &raw mut (*wp).w_tagstack as *mut taggy_T;
        tagstack_clear_entry(tagstack.offset(0 as ::core::ffi::c_int as isize));
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i < (*wp).w_tagstacklen {
            *tagstack.offset((i - 1 as ::core::ffi::c_int) as isize) = *tagstack.offset(i as isize);
            i += 1;
        }
        (*wp).w_tagstacklen -= 1;
    }
}

pub(crate) unsafe extern "C" fn tagstack_push_item(
    mut wp: *mut win_T,
    mut tagname: *mut ::core::ffi::c_char,
    mut cur_fnum: ::core::ffi::c_int,
    mut cur_match: ::core::ffi::c_int,
    mut mark: pos_T,
    mut fnum: ::core::ffi::c_int,
    mut user_data: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut tagstack: *mut taggy_T = &raw mut (*wp).w_tagstack as *mut taggy_T;
        let mut idx: ::core::ffi::c_int = (*wp).w_tagstacklen;
        if idx >= TAGSTACKSIZE {
            tagstack_shift(wp);
            idx = TAGSTACKSIZE - 1 as ::core::ffi::c_int;
        }
        (*wp).w_tagstacklen += 1;
        (*tagstack.offset(idx as isize)).tagname = tagname;
        (*tagstack.offset(idx as isize)).cur_fnum = cur_fnum;
        (*tagstack.offset(idx as isize)).cur_match = cur_match;
        (*tagstack.offset(idx as isize)).cur_match =
            if (*tagstack.offset(idx as isize)).cur_match > 0 as ::core::ffi::c_int {
                (*tagstack.offset(idx as isize)).cur_match
            } else {
                0 as ::core::ffi::c_int
            };
        (*tagstack.offset(idx as isize)).fmark.mark = mark;
        (*tagstack.offset(idx as isize)).fmark.fnum = fnum;
        (*tagstack.offset(idx as isize)).fmark.view = fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        };
        (*tagstack.offset(idx as isize)).user_data = user_data;
    }
}

pub(crate) unsafe extern "C" fn tagstack_push_items(mut wp: *mut win_T, mut l: *mut list_T) {
    unsafe {
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut tagname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut mark: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut fnum: ::core::ffi::c_int = 0;
        let mut li: *mut listitem_T = tv_list_first(l);
        while !li.is_null() {
            if !((*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*li).li_tv.vval.v_dict.is_null())
            {
                let mut itemdict: *mut dict_T = (*li).li_tv.vval.v_dict;
                di = tv_dict_find(
                    itemdict,
                    b"from\0".as_ptr() as *const ::core::ffi::c_char,
                    -1 as ptrdiff_t,
                );
                if !di.is_null() {
                    if list2fpos(
                        &raw mut (*di).di_tv,
                        &raw mut mark,
                        &raw mut fnum,
                        ::core::ptr::null_mut::<colnr_T>(),
                        false_0 != 0,
                    ) == OK
                    {
                        tagname = tv_dict_get_string(
                            itemdict,
                            b"tagname\0".as_ptr() as *const ::core::ffi::c_char,
                            true_0 != 0,
                        );
                        if !tagname.is_null() {
                            if mark.col > 0 as ::core::ffi::c_int {
                                mark.col -= 1;
                            }
                            tagstack_push_item(
                                wp,
                                tagname,
                                tv_dict_get_number(
                                    itemdict,
                                    b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
                                ) as ::core::ffi::c_int,
                                tv_dict_get_number(
                                    itemdict,
                                    b"matchnr\0".as_ptr() as *const ::core::ffi::c_char,
                                ) as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int,
                                mark,
                                fnum,
                                tv_dict_get_string(
                                    itemdict,
                                    b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                                    true_0 != 0,
                                ),
                            );
                        }
                    }
                }
            }
            li = (*li).li_next;
        }
    }
}

pub(crate) unsafe extern "C" fn tagstack_set_curidx(
    mut wp: *mut win_T,
    mut curidx: ::core::ffi::c_int,
) {
    unsafe {
        (*wp).w_tagstackidx = curidx;
        (*wp).w_tagstackidx = if (if (*wp).w_tagstackidx > 0 as ::core::ffi::c_int {
            (*wp).w_tagstackidx
        } else {
            0 as ::core::ffi::c_int
        }) < (*wp).w_tagstacklen
        {
            if (*wp).w_tagstackidx > 0 as ::core::ffi::c_int {
                (*wp).w_tagstackidx
            } else {
                0 as ::core::ffi::c_int
            }
        } else {
            (*wp).w_tagstacklen
        };
    }
}

pub unsafe extern "C" fn set_tagstack(
    mut wp: *mut win_T,
    mut d: *const dict_T,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if tfu_in_use.get() {
            emsg(gettext(
                (e_cannot_modify_tag_stack_within_tagfunc.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
        di = tv_dict_find(
            d,
            b"items\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            if (*di).di_tv.v_type as ::core::ffi::c_uint
                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
                return FAIL;
            }
            l = (*di).di_tv.vval.v_list;
        }
        di = tv_dict_find(
            d,
            b"curidx\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            tagstack_set_curidx(
                wp,
                tv_get_number(&raw mut (*di).di_tv) as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            );
        }
        if action == 't' as ::core::ffi::c_int {
            let tagstack: *mut taggy_T = &raw mut (*wp).w_tagstack as *mut taggy_T;
            let tagstackidx: ::core::ffi::c_int = (*wp).w_tagstackidx;
            let mut tagstacklen: ::core::ffi::c_int = (*wp).w_tagstacklen;
            while tagstackidx < tagstacklen {
                tagstacklen -= 1;
                tagstack_clear_entry(tagstack.offset(tagstacklen as isize));
            }
            (*wp).w_tagstacklen = tagstacklen;
        }
        if !l.is_null() {
            if action == 'r' as ::core::ffi::c_int {
                tagstack_clear(wp);
            }
            tagstack_push_items(wp, l);
            (*wp).w_tagstackidx = (*wp).w_tagstacklen;
        }
        return OK;
    }
}
