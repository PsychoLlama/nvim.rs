//! CTRL-A and CTRL-X -- incrementing the number under the cursor.
//!
//! `op_addsub` is the operator wrapper (`g CTRL-A` over a Visual region
//! increments each line by a growing multiple); `do_addsub` is the whole of
//! the per-line work: find a number at or after the cursor under the
//! 'nrformats' in force (decimal, hex, octal, binary, or a single
//! alphabetic character), parse it, add, and write it back preserving the
//! original's width and leading zeros.  Signs, `0x` prefixes and the
//! 64-bit overflow clamp are all its business.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn op_addsub(mut oap: *mut oparg_T, mut Prenum1: linenr_T, mut g_cmd: bool) {
    unsafe {
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
        let mut change_cnt: ssize_t = 0 as ssize_t;
        let mut amount: linenr_T = Prenum1;
        (*disable_fold_update.ptr()) += 1;
        if !VIsual_active.get() {
            let mut pos: pos_T = (*curwin.get()).w_cursor;
            if u_save_cursor() == FAIL {
                (*disable_fold_update.ptr()) -= 1;
                return;
            }
            change_cnt = do_addsub(
                (*oap).op_type,
                &raw mut pos,
                0 as ::core::ffi::c_int,
                amount,
            ) as ssize_t;
            (*disable_fold_update.ptr()) -= 1;
            if change_cnt != 0 {
                changed_lines(
                    curbuf.get(),
                    pos.lnum,
                    0 as colnr_T,
                    pos.lnum + 1 as linenr_T,
                    0 as linenr_T,
                    true_0 != 0,
                );
            }
        } else {
            let mut length: ::core::ffi::c_int = 0;
            let mut startpos: pos_T = pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            };
            if u_save(
                (*oap).start.lnum - 1 as linenr_T,
                (*oap).end.lnum + 1 as linenr_T,
            ) == FAIL
            {
                (*disable_fold_update.ptr()) -= 1;
                return;
            }
            let mut pos_0: pos_T = (*oap).start;
            while pos_0.lnum <= (*oap).end.lnum {
                if (*oap).motion_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    block_prep(oap, &raw mut bd, pos_0.lnum, false_0 != 0);
                    pos_0.col = bd.textcol;
                    length = bd.textlen;
                } else if (*oap).motion_type as ::core::ffi::c_int
                    == kMTLineWise as ::core::ffi::c_int
                {
                    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                    pos_0.col = 0 as ::core::ffi::c_int as colnr_T;
                    length = ml_get_len(pos_0.lnum) as ::core::ffi::c_int;
                } else {
                    if pos_0.lnum == (*oap).start.lnum && !(*oap).inclusive {
                        dec(&raw mut (*oap).end);
                    }
                    length = ml_get_len(pos_0.lnum) as ::core::ffi::c_int;
                    pos_0.col = 0 as ::core::ffi::c_int as colnr_T;
                    if pos_0.lnum == (*oap).start.lnum {
                        pos_0.col += (*oap).start.col;
                        length -= (*oap).start.col as ::core::ffi::c_int;
                    }
                    if pos_0.lnum == (*oap).end.lnum {
                        length = ml_get_len((*oap).end.lnum) as ::core::ffi::c_int;
                        (*oap).end.col = (if (*oap).end.col < length - 1 as ::core::ffi::c_int {
                            (*oap).end.col as ::core::ffi::c_int
                        } else {
                            length - 1 as ::core::ffi::c_int
                        }) as colnr_T;
                        length = (*oap).end.col as ::core::ffi::c_int
                            - pos_0.col as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int;
                    }
                }
                let mut one_change: bool =
                    do_addsub((*oap).op_type, &raw mut pos_0, length, amount);
                if one_change {
                    if change_cnt == 0 as ssize_t {
                        startpos = (*curbuf.get()).b_op_start;
                    }
                    change_cnt += 1;
                }
                if g_cmd as ::core::ffi::c_int != 0 && one_change as ::core::ffi::c_int != 0 {
                    amount += Prenum1;
                }
                pos_0.lnum += 1;
            }
            (*disable_fold_update.ptr()) -= 1;
            if change_cnt != 0 {
                changed_lines(
                    curbuf.get(),
                    (*oap).start.lnum,
                    0 as colnr_T,
                    (*oap).end.lnum + 1 as linenr_T,
                    0 as linenr_T,
                    true_0 != 0,
                );
            }
            if change_cnt == 0 && (*oap).is_VIsual as ::core::ffi::c_int != 0 {
                redraw_curbuf_later(UPD_INVERTED);
            }
            if change_cnt > 0 as ssize_t
                && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                (*curbuf.get()).b_op_start = startpos;
            }
            if change_cnt > p_report.get() as ssize_t {
                smsg(
                    0 as ::core::ffi::c_int,
                    ngettext(
                        b"%ld lines changed\0".as_ptr() as *const ::core::ffi::c_char,
                        b"%ld lines changed\0".as_ptr() as *const ::core::ffi::c_char,
                        change_cnt as ::core::ffi::c_ulong,
                    ),
                    change_cnt as int64_t,
                );
            }
        };
    }
}

pub unsafe extern "C" fn do_addsub(
    mut op_type: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut length: ::core::ffi::c_int,
    mut Prenum1: linenr_T,
) -> bool {
    unsafe {
        let mut firstdigit: ::core::ffi::c_int = 0;
        let mut pre: ::core::ffi::c_int = 0;
        static hexupper: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut n: uvarnumber_T = 0;
        let mut blank_unsigned: bool = false_0 != 0;
        let mut negative: bool = false_0 != 0;
        let mut was_positive: bool = true_0 != 0;
        let mut visual: bool = VIsual_active.get();
        let mut did_change: bool = false_0 != 0;
        let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
        let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut startpos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut endpos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut save_coladd: colnr_T = 0 as colnr_T;
        let do_hex: bool = !vim_strchr((*curbuf.get()).b_p_nf, 'x' as ::core::ffi::c_int).is_null();
        let do_oct: bool = !vim_strchr((*curbuf.get()).b_p_nf, 'o' as ::core::ffi::c_int).is_null();
        let do_bin: bool = !vim_strchr((*curbuf.get()).b_p_nf, 'b' as ::core::ffi::c_int).is_null();
        let do_alpha: bool =
            !vim_strchr((*curbuf.get()).b_p_nf, 'p' as ::core::ffi::c_int).is_null();
        let do_unsigned: bool =
            !vim_strchr((*curbuf.get()).b_p_nf, 'u' as ::core::ffi::c_int).is_null();
        let do_blank: bool =
            !vim_strchr((*curbuf.get()).b_p_nf, 'k' as ::core::ffi::c_int).is_null();
        if virtual_active(curwin.get()) {
            save_coladd = (*pos).coladd;
            (*pos).coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
        (*curwin.get()).w_cursor = *pos;
        let mut ptr: *mut ::core::ffi::c_char = ml_get((*pos).lnum);
        let mut linelen: ::core::ffi::c_int = ml_get_len((*pos).lnum);
        let mut col: ::core::ffi::c_int = (*pos).col as ::core::ffi::c_int;
        '_theend: {
            if (col + (save_coladd != 0) as ::core::ffi::c_int) < linelen {
                if !VIsual_active.get() {
                    if do_bin {
                        while col > 0 as ::core::ffi::c_int
                            && ascii_isbdigit(*ptr.offset(col as isize) as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                        {
                            col -= 1;
                            col -= utf_head_off(ptr, ptr.offset(col as isize));
                        }
                    }
                    if do_hex {
                        while col > 0 as ::core::ffi::c_int
                            && ascii_isxdigit(*ptr.offset(col as isize) as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                        {
                            col -= 1;
                            col -= utf_head_off(ptr, ptr.offset(col as isize));
                        }
                    }
                    if do_bin as ::core::ffi::c_int != 0
                        && do_hex as ::core::ffi::c_int != 0
                        && !(col > 0 as ::core::ffi::c_int
                            && (*ptr.offset(col as isize) as ::core::ffi::c_int
                                == 'X' as ::core::ffi::c_int
                                || *ptr.offset(col as isize) as ::core::ffi::c_int
                                    == 'x' as ::core::ffi::c_int)
                            && *ptr.offset((col - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int
                                == '0' as ::core::ffi::c_int
                            && utf_head_off(
                                ptr,
                                ptr.offset(col as isize)
                                    .offset(-(1 as ::core::ffi::c_int as isize)),
                            ) == 0
                            && ascii_isxdigit(*ptr.offset((col + 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0)
                    {
                        col = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                        while col > 0 as ::core::ffi::c_int
                            && ascii_isdigit(*ptr.offset(col as isize) as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                        {
                            col -= 1;
                            col -= utf_head_off(ptr, ptr.offset(col as isize));
                        }
                    }
                    if do_hex as ::core::ffi::c_int != 0
                        && col > 0 as ::core::ffi::c_int
                        && (*ptr.offset(col as isize) as ::core::ffi::c_int
                            == 'X' as ::core::ffi::c_int
                            || *ptr.offset(col as isize) as ::core::ffi::c_int
                                == 'x' as ::core::ffi::c_int)
                        && *ptr.offset((col - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == '0' as ::core::ffi::c_int
                        && utf_head_off(
                            ptr,
                            ptr.offset(col as isize)
                                .offset(-(1 as ::core::ffi::c_int as isize)),
                        ) == 0
                        && ascii_isxdigit(*ptr.offset((col + 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int) as ::core::ffi::c_int
                            != 0
                        || do_bin as ::core::ffi::c_int != 0
                            && col > 0 as ::core::ffi::c_int
                            && (*ptr.offset(col as isize) as ::core::ffi::c_int
                                == 'B' as ::core::ffi::c_int
                                || *ptr.offset(col as isize) as ::core::ffi::c_int
                                    == 'b' as ::core::ffi::c_int)
                            && *ptr.offset((col - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int
                                == '0' as ::core::ffi::c_int
                            && utf_head_off(
                                ptr,
                                ptr.offset(col as isize)
                                    .offset(-(1 as ::core::ffi::c_int as isize)),
                            ) == 0
                            && ascii_isbdigit(*ptr.offset((col + 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                    {
                        col -= 1;
                        col -= utf_head_off(ptr, ptr.offset(col as isize));
                    } else {
                        col = (*pos).col as ::core::ffi::c_int;
                        while *ptr.offset(col as isize) as ::core::ffi::c_int != NUL
                            && !ascii_isdigit(*ptr.offset(col as isize) as ::core::ffi::c_int)
                            && !(do_alpha as ::core::ffi::c_int != 0
                                && (*ptr.offset(col as isize) as ::core::ffi::c_uint
                                    >= 'A' as ::core::ffi::c_uint
                                    && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                        <= 'Z' as ::core::ffi::c_uint
                                    || *ptr.offset(col as isize) as ::core::ffi::c_uint
                                        >= 'a' as ::core::ffi::c_uint
                                        && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                            <= 'z' as ::core::ffi::c_uint))
                        {
                            col += 1;
                        }
                        while col > 0 as ::core::ffi::c_int
                            && ascii_isdigit(*ptr.offset((col - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                            && !(do_alpha as ::core::ffi::c_int != 0
                                && (*ptr.offset(col as isize) as ::core::ffi::c_uint
                                    >= 'A' as ::core::ffi::c_uint
                                    && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                        <= 'Z' as ::core::ffi::c_uint
                                    || *ptr.offset(col as isize) as ::core::ffi::c_uint
                                        >= 'a' as ::core::ffi::c_uint
                                        && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                            <= 'z' as ::core::ffi::c_uint))
                        {
                            col -= 1;
                        }
                    }
                }
                if visual {
                    while *ptr.offset(col as isize) as ::core::ffi::c_int != NUL
                        && length > 0 as ::core::ffi::c_int
                        && !ascii_isdigit(*ptr.offset(col as isize) as ::core::ffi::c_int)
                        && !(do_alpha as ::core::ffi::c_int != 0
                            && (*ptr.offset(col as isize) as ::core::ffi::c_uint
                                >= 'A' as ::core::ffi::c_uint
                                && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                    <= 'Z' as ::core::ffi::c_uint
                                || *ptr.offset(col as isize) as ::core::ffi::c_uint
                                    >= 'a' as ::core::ffi::c_uint
                                    && *ptr.offset(col as isize) as ::core::ffi::c_uint
                                        <= 'z' as ::core::ffi::c_uint))
                    {
                        let mut mb_len: ::core::ffi::c_int = utfc_ptr2len(ptr.offset(col as isize));
                        col += mb_len;
                        length -= mb_len;
                    }
                    if length == 0 as ::core::ffi::c_int {
                        break '_theend;
                    } else if col > (*pos).col
                        && *ptr.offset((col - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == '-' as ::core::ffi::c_int
                        && utf_head_off(
                            ptr,
                            ptr.offset(col as isize)
                                .offset(-(1 as ::core::ffi::c_int as isize)),
                        ) == 0
                        && !do_unsigned
                    {
                        if do_blank as ::core::ffi::c_int != 0
                            && col >= 2 as ::core::ffi::c_int
                            && !ascii_iswhite(*ptr.offset((col - 2 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int)
                        {
                            blank_unsigned = true_0 != 0;
                        } else {
                            negative = true_0 != 0;
                            was_positive = false_0 != 0;
                        }
                    }
                }
                firstdigit = *ptr.offset(col as isize) as uint8_t as ::core::ffi::c_int;
                if !ascii_isdigit(firstdigit)
                    && !(do_alpha as ::core::ffi::c_int != 0
                        && (firstdigit as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                            && firstdigit as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                            || firstdigit as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                                && firstdigit as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint))
                {
                    beep_flush();
                } else {
                    if do_alpha as ::core::ffi::c_int != 0
                        && (firstdigit as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                            && firstdigit as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                            || firstdigit as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                                && firstdigit as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                    {
                        if op_type == OP_NR_SUB as ::core::ffi::c_int {
                            if (if (firstdigit as uint8_t as ::core::ffi::c_int)
                                < 'a' as ::core::ffi::c_int
                            {
                                firstdigit as uint8_t as linenr_T - 'A' as linenr_T
                            } else {
                                firstdigit as uint8_t as linenr_T - 'a' as linenr_T
                            }) < Prenum1
                            {
                                firstdigit = if *(*__ctype_b_loc()).offset(firstdigit as isize)
                                    as ::core::ffi::c_int
                                    & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort
                                        as ::core::ffi::c_int
                                    != 0
                                {
                                    'A' as ::core::ffi::c_int
                                } else {
                                    'a' as ::core::ffi::c_int
                                };
                            } else {
                                firstdigit -= Prenum1 as ::core::ffi::c_int;
                            }
                        } else if (26 as linenr_T
                            - (if (firstdigit as uint8_t as ::core::ffi::c_int)
                                < 'a' as ::core::ffi::c_int
                            {
                                firstdigit as uint8_t as linenr_T - 'A' as linenr_T
                            } else {
                                firstdigit as uint8_t as linenr_T - 'a' as linenr_T
                            })
                            - 1 as linenr_T)
                            < Prenum1
                        {
                            firstdigit = if *(*__ctype_b_loc()).offset(firstdigit as isize)
                                as ::core::ffi::c_int
                                & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort
                                    as ::core::ffi::c_int
                                != 0
                            {
                                'Z' as ::core::ffi::c_int
                            } else {
                                'z' as ::core::ffi::c_int
                            };
                        } else {
                            firstdigit += Prenum1 as ::core::ffi::c_int;
                        }
                        (*curwin.get()).w_cursor.col = col as colnr_T;
                        startpos = (*curwin.get()).w_cursor;
                        did_change = true_0 != 0;
                        del_char(false_0 != 0);
                        ins_char(firstdigit);
                        endpos = (*curwin.get()).w_cursor;
                        (*curwin.get()).w_cursor.col = col as colnr_T;
                    } else {
                        if col > 0 as ::core::ffi::c_int
                            && *ptr.offset((col - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int
                                == '-' as ::core::ffi::c_int
                            && utf_head_off(
                                ptr,
                                ptr.offset(col as isize)
                                    .offset(-(1 as ::core::ffi::c_int as isize)),
                            ) == 0
                            && !visual
                            && !do_unsigned
                        {
                            if do_blank as ::core::ffi::c_int != 0
                                && col >= 2 as ::core::ffi::c_int
                                && !ascii_iswhite(
                                    *ptr.offset((col - 2 as ::core::ffi::c_int) as isize)
                                        as ::core::ffi::c_int,
                                )
                            {
                                blank_unsigned = true_0 != 0;
                            } else {
                                col -= 1;
                                negative = true_0 != 0;
                            }
                        }
                        if visual as ::core::ffi::c_int != 0
                            && VIsual_mode.get() != 'V' as ::core::ffi::c_int
                        {
                            maxlen = if (*curbuf.get()).b_visual.vi_curswant
                                == MAXCOL as ::core::ffi::c_int
                            {
                                linelen - col
                            } else {
                                length
                            };
                        }
                        let mut overflow: bool = false_0 != 0;
                        vim_str2nr(
                            ptr.offset(col as isize),
                            &raw mut pre,
                            &raw mut length,
                            0 as ::core::ffi::c_int
                                + (if do_bin as ::core::ffi::c_int != 0 {
                                    STR2NR_BIN as ::core::ffi::c_int
                                } else {
                                    0 as ::core::ffi::c_int
                                })
                                + (if do_oct as ::core::ffi::c_int != 0 {
                                    STR2NR_OCT as ::core::ffi::c_int
                                } else {
                                    0 as ::core::ffi::c_int
                                })
                                + (if do_hex as ::core::ffi::c_int != 0 {
                                    STR2NR_HEX as ::core::ffi::c_int
                                } else {
                                    0 as ::core::ffi::c_int
                                }),
                            ::core::ptr::null_mut::<varnumber_T>(),
                            &raw mut n,
                            maxlen,
                            false_0 != 0,
                            &raw mut overflow,
                        );
                        if pre != 0 && negative as ::core::ffi::c_int != 0 {
                            col += 1;
                            length -= 1;
                            negative = false_0 != 0;
                        }
                        let mut subtract: bool = false_0 != 0;
                        if op_type == OP_NR_SUB as ::core::ffi::c_int {
                            subtract = subtract as ::core::ffi::c_int ^ true_0 != 0;
                        }
                        if negative {
                            subtract = subtract as ::core::ffi::c_int ^ true_0 != 0;
                        }
                        let mut oldn: uvarnumber_T = n;
                        if !overflow {
                            n = if subtract as ::core::ffi::c_int != 0 {
                                n.wrapping_sub(Prenum1 as uvarnumber_T)
                            } else {
                                n.wrapping_add(Prenum1 as uvarnumber_T)
                            };
                        }
                        if pre == 0 {
                            if subtract {
                                if n > oldn {
                                    n = (1 as uvarnumber_T)
                                        .wrapping_add(n ^ -1 as ::core::ffi::c_int as uvarnumber_T);
                                    negative = negative as ::core::ffi::c_int ^ true_0 != 0;
                                }
                            } else if n < oldn {
                                n = n ^ -1 as ::core::ffi::c_int as uvarnumber_T;
                                negative = negative as ::core::ffi::c_int ^ true_0 != 0;
                            }
                            if n == 0 as uvarnumber_T {
                                negative = false_0 != 0;
                            }
                        }
                        if (do_unsigned as ::core::ffi::c_int != 0
                            || blank_unsigned as ::core::ffi::c_int != 0)
                            && negative as ::core::ffi::c_int != 0
                        {
                            if subtract {
                                n = 0 as uvarnumber_T;
                            } else {
                                n = -1 as ::core::ffi::c_int as uvarnumber_T;
                            }
                            negative = false_0 != 0;
                        }
                        if visual as ::core::ffi::c_int != 0
                            && !was_positive
                            && !negative
                            && col > 0 as ::core::ffi::c_int
                        {
                            col -= 1;
                            length += 1;
                        }
                        (*curwin.get()).w_cursor.col = col as colnr_T;
                        startpos = (*curwin.get()).w_cursor;
                        did_change = true_0 != 0;
                        let mut todel: ::core::ffi::c_int = length;
                        let mut c: ::core::ffi::c_int = gchar_cursor();
                        if c == '-' as ::core::ffi::c_int {
                            length -= 1;
                        }
                        loop {
                            let c2rust_fresh0 = todel;
                            todel = todel - 1;
                            if c2rust_fresh0 <= 0 as ::core::ffi::c_int {
                                break;
                            }
                            if c < 0x100 as ::core::ffi::c_int
                                && *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                                    & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort
                                        as ::core::ffi::c_int
                                    != 0
                            {
                                hexupper.set(
                                    *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                                        & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort
                                            as ::core::ffi::c_int
                                        != 0,
                                );
                            }
                            del_char(false_0 != 0);
                            c = gchar_cursor();
                        }
                        let mut buf1: *mut ::core::ffi::c_char = xmalloc(
                            (length as size_t)
                                .wrapping_add(NUMBUFLEN as ::core::ffi::c_int as size_t),
                        )
                            as *mut ::core::ffi::c_char;
                        ptr = buf1;
                        if negative as ::core::ffi::c_int != 0
                            && (!visual || was_positive as ::core::ffi::c_int != 0)
                        {
                            let c2rust_fresh1 = ptr;
                            ptr = ptr.offset(1);
                            *c2rust_fresh1 = '-' as ::core::ffi::c_char;
                        }
                        if pre != 0 {
                            let c2rust_fresh2 = ptr;
                            ptr = ptr.offset(1);
                            *c2rust_fresh2 = '0' as ::core::ffi::c_char;
                            length -= 1;
                        }
                        if pre == 'b' as ::core::ffi::c_int
                            || pre == 'B' as ::core::ffi::c_int
                            || pre == 'x' as ::core::ffi::c_int
                            || pre == 'X' as ::core::ffi::c_int
                        {
                            let c2rust_fresh3 = ptr;
                            ptr = ptr.offset(1);
                            *c2rust_fresh3 = pre as ::core::ffi::c_char;
                            length -= 1;
                        }
                        let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
                        let mut buf2len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if pre == 'b' as ::core::ffi::c_int || pre == 'B' as ::core::ffi::c_int {
                            let mut bits: size_t = 0 as size_t;
                            bits = (8 as usize).wrapping_mul(::core::mem::size_of::<uvarnumber_T>())
                                as size_t;
                            while bits > 0 as size_t {
                                if n >> bits.wrapping_sub(1 as size_t) & 0x1 as uvarnumber_T != 0 {
                                    break;
                                }
                                bits = bits.wrapping_sub(1);
                            }
                            while bits > 0 as size_t
                                && buf2len
                                    < NUMBUFLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                            {
                                bits = bits.wrapping_sub(1);
                                let c2rust_fresh4 = buf2len;
                                buf2len = buf2len + 1;
                                buf2[c2rust_fresh4 as usize] =
                                    (if n >> bits & 0x1 as uvarnumber_T != 0 {
                                        '1' as ::core::ffi::c_int
                                    } else {
                                        '0' as ::core::ffi::c_int
                                    }) as ::core::ffi::c_char;
                            }
                            buf2[buf2len as usize] = NUL as ::core::ffi::c_char;
                        } else if pre == 0 as ::core::ffi::c_int {
                            buf2len = vim_snprintf(
                                &raw mut buf2 as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                            .wrapping_rem(::core::mem::size_of::<
                                                ::core::ffi::c_char,
                                            >(
                                            ))
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    ),
                                b"%lu\0".as_ptr() as *const ::core::ffi::c_char,
                                n,
                            );
                        } else if pre == '0' as ::core::ffi::c_int {
                            buf2len = vim_snprintf(
                                &raw mut buf2 as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                            .wrapping_rem(::core::mem::size_of::<
                                                ::core::ffi::c_char,
                                            >(
                                            ))
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    ),
                                b"%lo\0".as_ptr() as *const ::core::ffi::c_char,
                                n,
                            );
                        } else if hexupper.get() {
                            buf2len = vim_snprintf(
                                &raw mut buf2 as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                            .wrapping_rem(::core::mem::size_of::<
                                                ::core::ffi::c_char,
                                            >(
                                            ))
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    ),
                                b"%lX\0".as_ptr() as *const ::core::ffi::c_char,
                                n,
                            );
                        } else {
                            buf2len = vim_snprintf(
                                &raw mut buf2 as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                    .wrapping_div(
                                        (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                                            .wrapping_rem(::core::mem::size_of::<
                                                ::core::ffi::c_char,
                                            >(
                                            ))
                                            == 0)
                                            as ::core::ffi::c_int
                                            as size_t,
                                    ),
                                b"%lx\0".as_ptr() as *const ::core::ffi::c_char,
                                n,
                            );
                        }
                        length -= buf2len;
                        if firstdigit == '0' as ::core::ffi::c_int
                            && !(do_oct as ::core::ffi::c_int != 0
                                && pre == 0 as ::core::ffi::c_int)
                        {
                            loop {
                                let c2rust_fresh5 = length;
                                length = length - 1;
                                if c2rust_fresh5 <= 0 as ::core::ffi::c_int {
                                    break;
                                }
                                let c2rust_fresh6 = ptr;
                                ptr = ptr.offset(1);
                                *c2rust_fresh6 = '0' as ::core::ffi::c_char;
                            }
                        }
                        *ptr = NUL as ::core::ffi::c_char;
                        let mut buf1len: ::core::ffi::c_int =
                            ptr.offset_from(buf1) as ::core::ffi::c_int;
                        strcpy(
                            buf1.offset(buf1len as isize),
                            &raw mut buf2 as *mut ::core::ffi::c_char,
                        );
                        buf1len += buf2len;
                        ins_str(buf1, buf1len as size_t);
                        xfree(buf1 as *mut ::core::ffi::c_void);
                        endpos = (*curwin.get()).w_cursor;
                        if (*curwin.get()).w_cursor.col != 0 {
                            (*curwin.get()).w_cursor.col -= 1;
                        }
                    }
                    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        (*curbuf.get()).b_op_start = startpos;
                        (*curbuf.get()).b_op_end = endpos;
                        if (*curbuf.get()).b_op_end.col > 0 as ::core::ffi::c_int {
                            (*curbuf.get()).b_op_end.col -= 1;
                        }
                    }
                }
            }
        }
        if visual {
            (*curwin.get()).w_cursor = save_cursor;
        } else if did_change {
            (*curwin.get()).w_set_curswant = true_0;
        } else if virtual_active(curwin.get()) {
            (*curwin.get()).w_cursor.coladd = save_coladd;
        }
        return did_change;
    }
}
