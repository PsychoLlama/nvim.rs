//! The text a Visual selection covers: `getregion()` and
//! `getregionpos()`.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

unsafe extern "C" fn block_def2str(mut bd: *mut block_def) -> String_0 {
    let mut size: size_t = ((*bd).startspaces as size_t)
        .wrapping_add((*bd).endspaces as size_t)
        .wrapping_add((*bd).textlen as size_t);
    let mut ret: String_0 = String_0 {
        data: xmalloc(size.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char,
        size: 0,
    };
    memset(
        ret.data as *mut ::core::ffi::c_void,
        ' ' as ::core::ffi::c_int,
        (*bd).startspaces as size_t,
    );
    ret.size = ret.size.wrapping_add((*bd).startspaces as size_t);
    memmove(
        ret.data.offset(ret.size as isize) as *mut ::core::ffi::c_void,
        (*bd).textstart as *const ::core::ffi::c_void,
        (*bd).textlen as size_t,
    );
    ret.size = ret.size.wrapping_add((*bd).textlen as size_t);
    memset(
        ret.data.offset(ret.size as isize) as *mut ::core::ffi::c_void,
        ' ' as ::core::ffi::c_int,
        (*bd).endspaces as size_t,
    );
    ret.size = ret.size.wrapping_add((*bd).endspaces as size_t);
    *ret.data.offset(ret.size as isize) = NUL as ::core::ffi::c_char;
    return ret;
}
unsafe extern "C" fn getregionpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut p1: *mut pos_T,
    mut p2: *mut pos_T,
    inclusive: *mut bool,
    mut region_type: *mut MotionType,
    mut oap: *mut oparg_T,
) -> ::core::ffi::c_int {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if tv_check_for_list_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_list_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
    {
        return FAIL;
    }
    let mut fnum1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut fnum2: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if list2fpos(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        p1,
        &raw mut fnum1,
        ::core::ptr::null_mut::<colnr_T>(),
        false_0 != 0,
    ) != OK
        || list2fpos(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            p2,
            &raw mut fnum2,
            ::core::ptr::null_mut::<colnr_T>(),
            false_0 != 0,
        ) != OK
        || fnum1 != fnum2
    {
        return FAIL;
    }
    let mut is_select_exclusive: bool = false;
    let mut type_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut default_type: [::core::ffi::c_char; 2] =
        ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"v\0");
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        is_select_exclusive = tv_dict_get_bool(
            (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"exclusive\0".as_ptr() as *const ::core::ffi::c_char,
            (*p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int) as ::core::ffi::c_int,
        ) != 0;
        type_0 = tv_dict_get_string(
            (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"type\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if type_0.is_null() {
            type_0 = &raw mut default_type as *mut ::core::ffi::c_char;
        }
    } else {
        is_select_exclusive = *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int;
        type_0 = &raw mut default_type as *mut ::core::ffi::c_char;
    }
    let mut block_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *type_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'v' as ::core::ffi::c_int
        && *type_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        *region_type = kMTCharWise;
    } else if *type_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'V' as ::core::ffi::c_int
        && *type_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        *region_type = kMTLineWise;
    } else if *type_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == Ctrl_V {
        let mut p: *mut ::core::ffi::c_char = type_0.offset(1 as ::core::ffi::c_int as isize);
        if *p as ::core::ffi::c_int != NUL && {
            block_width = getdigits_int(&raw mut p, false_0 != 0, 0 as ::core::ffi::c_int);
            block_width <= 0 as ::core::ffi::c_int || *p as ::core::ffi::c_int != NUL
        } {
            semsg(
                gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                b"type\0".as_ptr() as *const ::core::ffi::c_char,
                type_0,
            );
            return FAIL;
        }
        *region_type = kMTBlockWise;
    } else {
        semsg(
            gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
            b"type\0".as_ptr() as *const ::core::ffi::c_char,
            type_0,
        );
        return FAIL;
    }
    let mut findbuf: *mut buf_T = if fnum1 != 0 as ::core::ffi::c_int {
        buflist_findnr(fnum1)
    } else {
        curbuf.get()
    };
    if findbuf.is_null() || (*findbuf).b_ml.ml_mfp.is_null() {
        emsg(gettext(
            &raw const e_buffer_is_not_loaded as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if (*p1).lnum < 1 as linenr_T || (*p1).lnum > (*findbuf).b_ml.ml_line_count {
        semsg(
            gettext(&raw const e_invalid_line_number_nr as *const ::core::ffi::c_char),
            (*p1).lnum,
        );
        return FAIL;
    }
    if (*p1).col == MAXCOL as ::core::ffi::c_int {
        (*p1).col = (ml_get_buf_len(findbuf, (*p1).lnum) + 1 as ::core::ffi::c_int) as colnr_T;
    } else if (*p1).col < 1 as ::core::ffi::c_int
        || (*p1).col > ml_get_buf_len(findbuf, (*p1).lnum) + 1 as ::core::ffi::c_int
    {
        semsg(
            gettext(&raw const e_invalid_column_number_nr as *const ::core::ffi::c_char),
            (*p1).col,
        );
        return FAIL;
    }
    if (*p2).lnum < 1 as linenr_T || (*p2).lnum > (*findbuf).b_ml.ml_line_count {
        semsg(
            gettext(&raw const e_invalid_line_number_nr as *const ::core::ffi::c_char),
            (*p2).lnum,
        );
        return FAIL;
    }
    if (*p2).col == MAXCOL as ::core::ffi::c_int {
        (*p2).col = (ml_get_buf_len(findbuf, (*p2).lnum) + 1 as ::core::ffi::c_int) as colnr_T;
    } else if (*p2).col < 1 as ::core::ffi::c_int
        || (*p2).col > ml_get_buf_len(findbuf, (*p2).lnum) + 1 as ::core::ffi::c_int
    {
        semsg(
            gettext(&raw const e_invalid_column_number_nr as *const ::core::ffi::c_char),
            (*p2).col,
        );
        return FAIL;
    }
    curbuf.set(findbuf);
    (*curwin.get()).w_buffer = curbuf.get();
    virtual_op.set(virtual_active(curwin.get()) as TriState);
    (*p1).col -= 1;
    (*p2).col -= 1;
    if !lt(*p1, *p2) {
        let mut p_0: pos_T = *p1;
        *p1 = *p2;
        *p2 = p_0;
    }
    if *region_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
        if is_select_exclusive as ::core::ffi::c_int != 0 && !equalpos(*p1, *p2) {
            *inclusive = !unadjust_for_sel_inner(p2);
        }
        if *inclusive as ::core::ffi::c_int != 0
            && virtual_op.get() as u64 == 0
            && *ml_get_pos(p2) as ::core::ffi::c_int == NUL
        {
            *inclusive = false_0 != 0;
        }
    } else if *region_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        let mut sc1: colnr_T = 0;
        let mut ec1: colnr_T = 0;
        let mut sc2: colnr_T = 0;
        let mut ec2: colnr_T = 0;
        let lbr_saved: bool = reset_lbr();
        getvvcol(
            curwin.get(),
            p1,
            &raw mut sc1,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut ec1,
        );
        getvvcol(
            curwin.get(),
            p2,
            &raw mut sc2,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut ec2,
        );
        restore_lbr(lbr_saved);
        (*oap).motion_type = kMTBlockWise;
        (*oap).inclusive = true_0 != 0;
        (*oap).op_type = OP_NOP as ::core::ffi::c_int;
        (*oap).start = *p1;
        (*oap).end = *p2;
        (*oap).start_vcol = if sc1 < sc2 { sc1 } else { sc2 };
        if block_width > 0 as ::core::ffi::c_int {
            (*oap).end_vcol = ((*oap).start_vcol as ::core::ffi::c_int + block_width
                - 1 as ::core::ffi::c_int) as colnr_T;
        } else if is_select_exclusive as ::core::ffi::c_int != 0
            && ec1 < sc2
            && (0 as ::core::ffi::c_int) < sc2
            && ec2 > ec1
        {
            (*oap).end_vcol = (sc2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
        } else {
            (*oap).end_vcol = if ec1 > ec2 { ec1 } else { ec2 };
        }
    }
    let mut l: ::core::ffi::c_int = utfc_ptr2len(ml_get_pos(p2));
    if l > 1 as ::core::ffi::c_int {
        (*p2).col += l - 1 as ::core::ffi::c_int;
    }
    return OK;
}
pub unsafe extern "C" fn f_getregion(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let save_curbuf: *mut buf_T = curbuf.get();
    let save_virtual: TriState = virtual_op.get();
    let mut p1: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut p2: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut inclusive: bool = true_0 != 0;
    let mut region_type: MotionType = kMTUnknown;
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    if getregionpos(
        argvars,
        rettv,
        &raw mut p1,
        &raw mut p2,
        &raw mut inclusive,
        &raw mut region_type,
        &raw mut oa,
    ) == FAIL
    {
        return;
    }
    let mut lnum: linenr_T = p1.lnum;
    while lnum <= p2.lnum {
        let mut akt: String_0 = STRING_INIT;
        if region_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            let mut bd: block_def = block_def {
                startspaces: 0,
                endspaces: 0,
                textlen: 0,
                textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                textcol: 0,
                start_vcol: 0,
                end_vcol: 0,
                is_short: 0,
                is_MAX: 0,
                is_oneChar: 0,
                pre_whitesp: 0,
                pre_whitesp_c: 0,
                end_char_vcols: 0,
                start_char_vcols: 0,
            };
            block_prep(&raw mut oa, &raw mut bd, lnum, false_0 != 0);
            akt = block_def2str(&raw mut bd);
        } else if region_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
            || p1.lnum < lnum && lnum < p2.lnum
        {
            akt = cbuf_to_string(ml_get(lnum), ml_get_len(lnum) as size_t);
        } else {
            let mut bd_0: block_def = block_def {
                startspaces: 0,
                endspaces: 0,
                textlen: 0,
                textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                textcol: 0,
                start_vcol: 0,
                end_vcol: 0,
                is_short: 0,
                is_MAX: 0,
                is_oneChar: 0,
                pre_whitesp: 0,
                pre_whitesp_c: 0,
                end_char_vcols: 0,
                start_char_vcols: 0,
            };
            charwise_block_prep(p1, p2, &raw mut bd_0, lnum, inclusive);
            akt = block_def2str(&raw mut bd_0);
        }
        '_c2rust_label: {
            if !akt.data.is_null() {
            } else {
                __assert_fail(
                    b"akt.data != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2344 as ::core::ffi::c_uint,
                    b"void f_getregion(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        tv_list_append_allocated_string((*rettv).vval.v_list, akt.data);
        lnum += 1;
    }
    curbuf.set(save_curbuf);
    (*curwin.get()).w_buffer = curbuf.get();
    virtual_op.set(save_virtual);
}
unsafe extern "C" fn add_regionpos_range(mut rettv: *mut typval_T, mut p1: pos_T, mut p2: pos_T) {
    let mut l1: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
    tv_list_append_list((*rettv).vval.v_list, l1);
    let mut l2: *mut list_T = tv_list_alloc(4 as ptrdiff_t);
    tv_list_append_list(l1, l2);
    let mut l3: *mut list_T = tv_list_alloc(4 as ptrdiff_t);
    tv_list_append_list(l1, l3);
    tv_list_append_number(l2, (*curbuf.get()).handle as varnumber_T);
    tv_list_append_number(l2, p1.lnum as varnumber_T);
    tv_list_append_number(l2, p1.col as varnumber_T);
    tv_list_append_number(l2, p1.coladd as varnumber_T);
    tv_list_append_number(l3, (*curbuf.get()).handle as varnumber_T);
    tv_list_append_number(l3, p2.lnum as varnumber_T);
    tv_list_append_number(l3, p2.col as varnumber_T);
    tv_list_append_number(l3, p2.coladd as varnumber_T);
}
pub unsafe extern "C" fn f_getregionpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let save_curbuf: *mut buf_T = curbuf.get();
    let save_virtual: TriState = virtual_op.get();
    let mut p1: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut p2: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut inclusive: bool = true_0 != 0;
    let mut region_type: MotionType = kMTUnknown;
    let mut allow_eol: bool = false_0 != 0;
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    if getregionpos(
        argvars,
        rettv,
        &raw mut p1,
        &raw mut p2,
        &raw mut inclusive,
        &raw mut region_type,
        &raw mut oa,
    ) == FAIL
    {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        allow_eol = tv_dict_get_bool(
            (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"eol\0".as_ptr() as *const ::core::ffi::c_char,
            false_0,
        ) != 0;
    }
    let mut lnum: linenr_T = p1.lnum;
    while lnum <= p2.lnum {
        let mut ret_p1: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut ret_p2: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
        let mut line_len: colnr_T = ml_get_len(lnum);
        if region_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            ret_p1.col = 1 as ::core::ffi::c_int as colnr_T;
            ret_p1.coladd = 0 as ::core::ffi::c_int as colnr_T;
            ret_p2.col = MAXCOL as ::core::ffi::c_int as colnr_T;
            ret_p2.coladd = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            let mut bd: block_def = block_def {
                startspaces: 0,
                endspaces: 0,
                textlen: 0,
                textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                textcol: 0,
                start_vcol: 0,
                end_vcol: 0,
                is_short: 0,
                is_MAX: 0,
                is_oneChar: 0,
                pre_whitesp: 0,
                pre_whitesp_c: 0,
                end_char_vcols: 0,
                start_char_vcols: 0,
            };
            if region_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                block_prep(&raw mut oa, &raw mut bd, lnum, false_0 != 0);
            } else {
                charwise_block_prep(p1, p2, &raw mut bd, lnum, inclusive);
            }
            if bd.is_oneChar != 0 {
                if region_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    ret_p1.col = (mb_prevptr(line, bd.textstart).offset_from(line)
                        as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int) as colnr_T;
                    ret_p1.coladd = bd.start_char_vcols - (bd.start_vcol - oa.start_vcol);
                } else {
                    ret_p1.col =
                        (p1.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
                    ret_p1.coladd = p1.coladd;
                }
            } else if region_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int
                && oa.start_vcol > bd.start_vcol
            {
                ret_p1.col = MAXCOL as ::core::ffi::c_int as colnr_T;
                ret_p1.coladd = oa.start_vcol - bd.start_vcol;
                bd.is_oneChar = true_0;
            } else if bd.startspaces > 0 as ::core::ffi::c_int {
                ret_p1.col = (mb_prevptr(line, bd.textstart).offset_from(line)
                    as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as colnr_T;
                ret_p1.coladd =
                    (bd.start_char_vcols as ::core::ffi::c_int - bd.startspaces) as colnr_T;
            } else {
                ret_p1.col =
                    (bd.textcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
                ret_p1.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
            if bd.is_oneChar != 0 {
                ret_p2.col = ret_p1.col;
                ret_p2.coladd = (ret_p1.coladd as ::core::ffi::c_int
                    + bd.startspaces
                    + bd.endspaces) as colnr_T;
            } else if bd.endspaces > 0 as ::core::ffi::c_int {
                ret_p2.col = (bd.textcol as ::core::ffi::c_int
                    + bd.textlen
                    + 1 as ::core::ffi::c_int) as colnr_T;
                ret_p2.coladd = bd.endspaces as colnr_T;
            } else {
                ret_p2.col = (bd.textcol as ::core::ffi::c_int + bd.textlen) as colnr_T;
                ret_p2.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
        }
        if !allow_eol && ret_p1.col > line_len {
            ret_p1.col = 0 as ::core::ffi::c_int as colnr_T;
            ret_p1.coladd = 0 as ::core::ffi::c_int as colnr_T;
        } else if ret_p1.col > line_len as ::core::ffi::c_int + 1 as ::core::ffi::c_int {
            ret_p1.col = (line_len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
        }
        if !allow_eol && ret_p2.col > line_len {
            ret_p2.col = (if ret_p1.col == 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                line_len as ::core::ffi::c_int
            }) as colnr_T;
            ret_p2.coladd = 0 as ::core::ffi::c_int as colnr_T;
        } else if ret_p2.col > line_len as ::core::ffi::c_int + 1 as ::core::ffi::c_int {
            ret_p2.col = (line_len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
        }
        ret_p1.lnum = lnum;
        ret_p2.lnum = lnum;
        add_regionpos_range(rettv, ret_p1, ret_p2);
        lnum += 1;
    }
    curbuf.set(save_curbuf);
    (*curwin.get()).w_buffer = curbuf.get();
    virtual_op.set(save_virtual);
}
