//! The `sign_*()` Vimscript functions.
//!
//! The same operations the `:sign` command performs, addressed by
//! dictionary rather than by command line, plus the two report functions
//! (`sign_getdefined()`, `sign_getplaced()`) that answer with dictionaries
//! of their own. The `*_from_dict` helpers are shared between the single
//! and the `*list()` bulk forms.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn sign_get_info_dict(mut sp: *mut sign_T) -> *mut dict_T {
    unsafe {
        let mut d: *mut dict_T = tv_dict_alloc();
        tv_dict_add_str(
            d,
            b"name\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*sp).sn_name,
        );
        if !(*sp).sn_icon.is_null() {
            tv_dict_add_str(
                d,
                b"icon\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                (*sp).sn_icon,
            );
        }
        if (*sp).sn_text[0 as ::core::ffi::c_int as usize] != 0 {
            let mut buf: [::core::ffi::c_char; 64] = [0; 64];
            describe_sign_text(
                &raw mut buf as *mut ::core::ffi::c_char,
                &raw mut (*sp).sn_text as *mut schar_T,
            );
            tv_dict_add_str(
                d,
                b"text\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                &raw mut buf as *mut ::core::ffi::c_char,
            );
        }
        if (*sp).sn_priority > 0 as ::core::ffi::c_int {
            tv_dict_add_nr(
                d,
                b"priority\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                (*sp).sn_priority as varnumber_T,
            );
        }
        static arg: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
            b"linehl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"texthl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"culhl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"numhl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ]);
        let mut hl: [::core::ffi::c_int; 4] = [
            (*sp).sn_line_hl,
            (*sp).sn_text_hl,
            (*sp).sn_cul_hl,
            (*sp).sn_num_hl,
        ];
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 4 as ::core::ffi::c_int {
            if hl[i as usize] > 0 as ::core::ffi::c_int {
                let mut p: *const ::core::ffi::c_char = get_highlight_name_ext(
                    ::core::ptr::null_mut::<expand_T>(),
                    hl[i as usize] - 1 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                tv_dict_add_str(
                    d,
                    (*arg.ptr())[i as usize],
                    strlen((*arg.ptr())[i as usize]),
                    if !p.is_null() {
                        p
                    } else {
                        b"NONE\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
            }
            i += 1;
        }
        return d;
    }
}

pub(crate) unsafe extern "C" fn sign_get_placed_info_dict(mut mark: MTKey) -> *mut dict_T {
    unsafe {
        let mut d: *mut dict_T = tv_dict_alloc();
        let mut sh: *mut DecorSignHighlight = decor_find_sign(mt_decor(mark));
        tv_dict_add_str(
            d,
            b"name\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            sign_get_name(sh),
        );
        tv_dict_add_nr(
            d,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            mark.id as ::core::ffi::c_int as varnumber_T,
        );
        tv_dict_add_str(
            d,
            b"group\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            describe_ns(mark.ns as NS, b"\0".as_ptr() as *const ::core::ffi::c_char),
        );
        tv_dict_add_nr(
            d,
            b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (mark.pos.row + 1 as int32_t) as varnumber_T,
        );
        tv_dict_add_nr(
            d,
            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            (*sh).priority as varnumber_T,
        );
        return d;
    }
}

pub unsafe extern "C" fn get_buffer_signs(mut buf: *mut buf_T) -> *mut list_T {
    unsafe {
        let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1];
        marktree_itr_get(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            0 as int32_t,
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
        while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            if !mt_end(mark) && mt_decor_sign(mark) as ::core::ffi::c_int != 0 {
                tv_list_append_dict(l, sign_get_placed_info_dict(mark));
            }
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
            );
        }
        return l;
    }
}

pub(crate) unsafe extern "C" fn sign_get_placed_in_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut sign_id: ::core::ffi::c_int,
    mut group: *const ::core::ffi::c_char,
    mut retlist: *mut list_T,
) {
    unsafe {
        let mut d: *mut dict_T = tv_dict_alloc();
        tv_list_append_dict(retlist, d);
        tv_dict_add_nr(
            d,
            b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            (*buf).handle as varnumber_T,
        );
        let mut l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        tv_dict_add_list(
            d,
            b"signs\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            l,
        );
        let mut ns: int64_t = group_get_ns(group);
        if !buf_has_signs(buf) || ns < 0 as int64_t {
            return;
        }
        let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1];
        let mut signs: C2Rust_Unnamed_28 = C2Rust_Unnamed_28 {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<MTKey>(),
        };
        marktree_itr_get(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            if lnum != 0 {
                lnum as int32_t - 1 as int32_t
            } else {
                0 as int32_t
            },
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
        while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            if lnum != 0 && mark.pos.row >= lnum {
                break;
            }
            if !mt_end(mark)
                && (ns == UINT32_MAX as int64_t || ns == mark.ns as int64_t)
                && (lnum == 0 as linenr_T && sign_id == 0 as ::core::ffi::c_int
                    || sign_id == 0 as ::core::ffi::c_int && lnum == mark.pos.row + 1 as int32_t
                    || lnum == 0 as linenr_T && sign_id == mark.id as ::core::ffi::c_int
                    || lnum == mark.pos.row + 1 as int32_t
                        && sign_id == mark.id as ::core::ffi::c_int)
            {
                if mt_decor_sign(mark) {
                    if signs.size == signs.capacity {
                        signs.capacity = if signs.capacity != 0 {
                            signs.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        signs.items = xrealloc(
                            signs.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<MTKey>().wrapping_mul(signs.capacity),
                        ) as *mut MTKey;
                    } else {
                    };
                    let c2rust_fresh10 = signs.size;
                    signs.size = signs.size.wrapping_add(1);
                    *signs.items.offset(c2rust_fresh10 as isize) = mark;
                }
            }
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
            );
        }
        if signs.size != 0 {
            qsort(
                signs.items.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                signs.size,
                ::core::mem::size_of::<MTKey>(),
                Some(
                    sign_row_cmp
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
            let mut i: size_t = 0 as size_t;
            while i < signs.size {
                tv_list_append_dict(
                    l,
                    sign_get_placed_info_dict(*signs.items.offset(i as isize)),
                );
                i = i.wrapping_add(1);
            }
            xfree(signs.items as *mut ::core::ffi::c_void);
            signs.capacity = 0 as size_t;
            signs.size = signs.capacity;
            signs.items = ::core::ptr::null_mut::<MTKey>();
        }
    }
}

pub(crate) unsafe extern "C" fn sign_get_placed(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut id: ::core::ffi::c_int,
    mut group: *const ::core::ffi::c_char,
    mut retlist: *mut list_T,
) {
    unsafe {
        if !buf.is_null() {
            sign_get_placed_in_buf(buf, lnum, id, group, retlist);
        } else {
            let mut cbuf: *mut buf_T = firstbuf.get();
            while !cbuf.is_null() {
                if buf_has_signs(cbuf) {
                    sign_get_placed_in_buf(cbuf, 0 as linenr_T, id, group, retlist);
                }
                cbuf = (*cbuf).b_next;
            }
        };
    }
}

pub(crate) unsafe extern "C" fn sign_define_from_dict(
    mut name: *mut ::core::ffi::c_char,
    mut dict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        if name.is_null() {
            name = tv_dict_get_string(
                dict,
                b"name\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            if name.is_null()
                || *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            {
                return -1 as ::core::ffi::c_int;
            }
        }
        let mut icon: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut linehl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut texthl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut culhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut numhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if !dict.is_null() {
            icon = tv_dict_get_string(
                dict,
                b"icon\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            linehl = tv_dict_get_string(
                dict,
                b"linehl\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            text = tv_dict_get_string(
                dict,
                b"text\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            texthl = tv_dict_get_string(
                dict,
                b"texthl\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            culhl = tv_dict_get_string(
                dict,
                b"culhl\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            numhl = tv_dict_get_string(
                dict,
                b"numhl\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            prio = tv_dict_get_number_def(
                dict,
                b"priority\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int;
        }
        return sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio)
            - 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn sign_define_multiple(mut l: *mut list_T, mut retlist: *mut list_T) {
    unsafe {
        let l_: *const list_T = l;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                if (*li).li_tv.v_type as ::core::ffi::c_uint
                    == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    retval = sign_define_from_dict(
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        (*li).li_tv.vval.v_dict,
                    );
                } else {
                    emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
                }
                tv_list_append_number(retlist, retval as varnumber_T);
                li = (*li).li_next;
            }
        }
    }
}

pub unsafe extern "C" fn f_sign_define(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
            sign_define_multiple(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
                (*rettv).vval.v_list,
            );
            return;
        }
        (*rettv).vval.v_number = -1 as varnumber_T;
        let mut name: *mut ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_char;
        if name.is_null() {
            return;
        }
        if tv_check_for_opt_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let mut d: *mut dict_T = if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type
            as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_dict
        } else {
            ::core::ptr::null_mut::<dict_T>()
        };
        (*rettv).vval.v_number = sign_define_from_dict(name, d) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_sign_getdefined(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut sp: *mut sign_T = ::core::ptr::null_mut::<sign_T>();
            let mut __i: uint32_t = 0;
            __i = 0 as uint32_t;
            while __i < (*sign_map.ptr()).set.h.n_keys {
                sp = *(*sign_map.ptr()).values.offset(__i as isize) as *mut sign_T;
                tv_list_append_dict((*rettv).vval.v_list, sign_get_info_dict(sp));
                __i = __i.wrapping_add(1);
            }
        } else {
            let mut sp_0: *mut sign_T = map_get_cstr_t_ptr_t(
                sign_map.ptr(),
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            ) as *mut sign_T;
            if !sp_0.is_null() {
                tv_list_append_dict((*rettv).vval.v_list, sign_get_info_dict(sp_0));
            }
        };
    }
}

pub unsafe extern "C" fn f_sign_getplaced(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut lnum: linenr_T = 0 as linenr_T;
        let mut sign_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut group: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut notanum: bool = false_0 != 0;
        tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            buf = get_buf_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
            if buf.is_null() {
                return;
            }
            if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if tv_check_for_nonnull_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
                    return;
                }
                let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
                let mut dict: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict;
                di = tv_dict_find(
                    dict,
                    b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                    -1 as ptrdiff_t,
                );
                if !di.is_null() {
                    lnum = tv_get_lnum(&raw mut (*di).di_tv);
                    if lnum <= 0 as linenr_T {
                        return;
                    }
                }
                di = tv_dict_find(
                    dict,
                    b"id\0".as_ptr() as *const ::core::ffi::c_char,
                    -1 as ptrdiff_t,
                );
                if !di.is_null() {
                    sign_id = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut notanum)
                        as ::core::ffi::c_int;
                    if notanum {
                        return;
                    }
                }
                di = tv_dict_find(
                    dict,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    -1 as ptrdiff_t,
                );
                if !di.is_null() {
                    group = tv_get_string_chk(&raw mut (*di).di_tv);
                    if group.is_null() {
                        return;
                    }
                    if *group as ::core::ffi::c_int == NUL {
                        group = ::core::ptr::null::<::core::ffi::c_char>();
                    }
                }
            }
        }
        sign_get_placed(buf, lnum, sign_id, group, (*rettv).vval.v_list);
    }
}

pub unsafe extern "C" fn f_sign_jump(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        let mut notanum: bool = false_0 != 0;
        let mut id: ::core::ffi::c_int = tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut notanum,
        ) as ::core::ffi::c_int;
        if notanum {
            return;
        }
        if id <= 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        let mut group: *mut ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_char;
        if group.is_null() {
            return;
        }
        if *group.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            group = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut buf: *mut buf_T = get_buf_arg(argvars.offset(2 as ::core::ffi::c_int as isize));
        if buf.is_null() {
            return;
        }
        (*rettv).vval.v_number = sign_jump(id, group, buf) as varnumber_T;
    }
}

pub(crate) unsafe extern "C" fn sign_place_from_dict(
    mut id_tv: *mut typval_T,
    mut group_tv: *mut typval_T,
    mut name_tv: *mut typval_T,
    mut buf_tv: *mut typval_T,
    mut dict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut notanum: bool = false_0 != 0;
        if id_tv.is_null() {
            di = tv_dict_find(
                dict,
                b"id\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                id_tv = &raw mut (*di).di_tv;
            }
        }
        if !id_tv.is_null() {
            id = tv_get_number_chk(id_tv, &raw mut notanum) as ::core::ffi::c_int;
            if notanum {
                return -1 as ::core::ffi::c_int;
            }
            if id < 0 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return -1 as ::core::ffi::c_int;
            }
        }
        let mut group: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if group_tv.is_null() {
            di = tv_dict_find(
                dict,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                group_tv = &raw mut (*di).di_tv;
            }
        }
        if !group_tv.is_null() {
            group = tv_get_string_chk(group_tv) as *mut ::core::ffi::c_char;
            if group.is_null() {
                return -1 as ::core::ffi::c_int;
            }
            if *group.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                group = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        }
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if name_tv.is_null() {
            di = tv_dict_find(
                dict,
                b"name\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                name_tv = &raw mut (*di).di_tv;
            }
        }
        if name_tv.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        name = tv_get_string_chk(name_tv) as *mut ::core::ffi::c_char;
        if name.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        if buf_tv.is_null() {
            di = tv_dict_find(
                dict,
                b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                buf_tv = &raw mut (*di).di_tv;
            }
        }
        if buf_tv.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        let mut buf: *mut buf_T = get_buf_arg(buf_tv);
        if buf.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        let mut lnum: linenr_T = 0 as linenr_T;
        di = tv_dict_find(
            dict,
            b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            lnum = tv_get_lnum(&raw mut (*di).di_tv);
            if lnum <= 0 as linenr_T {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return -1 as ::core::ffi::c_int;
            }
        }
        let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        di = tv_dict_find(
            dict,
            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            prio = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut notanum) as ::core::ffi::c_int;
            if notanum {
                return -1 as ::core::ffi::c_int;
            }
        }
        let mut uid: uint32_t = id as uint32_t;
        if sign_place(&raw mut uid, group, name, buf, lnum, prio) == OK {
            return uid as ::core::ffi::c_int;
        }
        return -1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn f_sign_place(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        (*rettv).vval.v_number = -1 as varnumber_T;
        if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_check_for_nonnull_dict_arg(argvars, 4 as ::core::ffi::c_int) == FAIL {
                return;
            }
            dict = (*argvars.offset(4 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
        }
        (*rettv).vval.v_number = sign_place_from_dict(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            argvars.offset(1 as ::core::ffi::c_int as isize),
            argvars.offset(2 as ::core::ffi::c_int as isize),
            argvars.offset(3 as ::core::ffi::c_int as isize),
            dict,
        ) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_sign_placelist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return;
        }
        let l_: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let mut sign_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                if (*li).li_tv.v_type as ::core::ffi::c_uint
                    == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    sign_id = sign_place_from_dict(
                        ::core::ptr::null_mut::<typval_T>(),
                        ::core::ptr::null_mut::<typval_T>(),
                        ::core::ptr::null_mut::<typval_T>(),
                        ::core::ptr::null_mut::<typval_T>(),
                        (*li).li_tv.vval.v_dict,
                    );
                } else {
                    emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
                }
                tv_list_append_number((*rettv).vval.v_list, sign_id as varnumber_T);
                li = (*li).li_next;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn sign_undefine_multiple(
    mut l: *mut list_T,
    mut retlist: *mut list_T,
) {
    unsafe {
        let l_: *const list_T = l;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                let mut name: *mut ::core::ffi::c_char =
                    tv_get_string_chk(&raw const (*li).li_tv) as *mut ::core::ffi::c_char;
                if !name.is_null() && sign_undefine_by_name(name) == 1 as ::core::ffi::c_int {
                    retval = 0 as ::core::ffi::c_int;
                }
                tv_list_append_number(retlist, retval as varnumber_T);
                li = (*li).li_next;
            }
        }
    }
}

pub unsafe extern "C" fn f_sign_undefine(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
            sign_undefine_multiple(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
                (*rettv).vval.v_list,
            );
            return;
        }
        (*rettv).vval.v_number = -1 as varnumber_T;
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            free_signs();
            (*rettv).vval.v_number = 0 as varnumber_T;
        } else {
            let mut name: *const ::core::ffi::c_char =
                tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
            if name.is_null() {
                return;
            }
            if sign_undefine_by_name(name) == OK {
                (*rettv).vval.v_number = 0 as varnumber_T;
            }
        };
    }
}

pub(crate) unsafe extern "C" fn sign_unplace_from_dict(
    mut group_tv: *mut typval_T,
    mut dict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut group: *mut ::core::ffi::c_char = if !group_tv.is_null() {
            tv_get_string(group_tv) as *mut ::core::ffi::c_char
        } else {
            tv_dict_get_string(
                dict,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            )
        };
        if !group.is_null()
            && *group.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            group = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !dict.is_null() {
            di = tv_dict_find(
                dict,
                b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                buf = get_buf_arg(&raw mut (*di).di_tv);
                if buf.is_null() {
                    return -1 as ::core::ffi::c_int;
                }
            }
            if !tv_dict_find(
                dict,
                b"id\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            )
            .is_null()
            {
                id = tv_dict_get_number(dict, b"id\0".as_ptr() as *const ::core::ffi::c_char)
                    as ::core::ffi::c_int;
                if id <= 0 as ::core::ffi::c_int {
                    emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                    return -1 as ::core::ffi::c_int;
                }
            }
        }
        return sign_unplace(buf, id, group, 0 as linenr_T) - 1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn f_sign_unplace(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        (*rettv).vval.v_number = -1 as varnumber_T;
        if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
            || tv_check_for_opt_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        {
            return;
        }
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            dict = (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
        }
        (*rettv).vval.v_number =
            sign_unplace_from_dict(argvars.offset(0 as ::core::ffi::c_int as isize), dict)
                as varnumber_T;
    }
}

pub unsafe extern "C" fn f_sign_unplacelist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return;
        }
        let l_: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                if (*li).li_tv.v_type as ::core::ffi::c_uint
                    == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    retval = sign_unplace_from_dict(
                        ::core::ptr::null_mut::<typval_T>(),
                        (*li).li_tv.vval.v_dict,
                    );
                } else {
                    emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
                }
                tv_list_append_number((*rettv).vval.v_list, retval as varnumber_T);
                li = (*li).li_next;
            }
        }
    }
}
