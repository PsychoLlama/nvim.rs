//! Yanking text into a register.
//!
//! `op_yank_reg` is the whole of `y`: it copies the operator's region into a
//! `yankreg_T` line by line, with the blockwise case going through
//! `block_prep` per line so that a short line is padded and a tab straddling
//! the edge is split.  `format_reg_type` renders the `v`/`V`/`CTRL-V width`
//! string the API and `:registers` both show, and `do_autocmd_textyankpost`
//! builds the `v:event` dictionary TextYankPost sees.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn yank_copy_line(
    mut reg: *mut yankreg_T,
    mut bd: *mut block_def,
    mut y_idx: size_t,
    mut exclude_trailing_space: bool,
) {
    unsafe {
        if exclude_trailing_space {
            (*bd).endspaces = 0 as ::core::ffi::c_int;
        }
        let mut size: ::core::ffi::c_int = (*bd).startspaces + (*bd).endspaces + (*bd).textlen;
        '_c2rust_label: {
            if size >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"size >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    985 as ::core::ffi::c_uint,
                    b"void yank_copy_line(yankreg_T *, struct block_def *, size_t, _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut pnew: *mut ::core::ffi::c_char =
            xmallocz(size as size_t) as *mut ::core::ffi::c_char;
        (*(*reg).y_array.offset(y_idx as isize)).data = pnew;
        memset(
            pnew as *mut ::core::ffi::c_void,
            ' ' as ::core::ffi::c_int,
            (*bd).startspaces as size_t,
        );
        pnew = pnew.offset((*bd).startspaces as isize);
        memmove(
            pnew as *mut ::core::ffi::c_void,
            (*bd).textstart as *const ::core::ffi::c_void,
            (*bd).textlen as size_t,
        );
        pnew = pnew.offset((*bd).textlen as isize);
        memset(
            pnew as *mut ::core::ffi::c_void,
            ' ' as ::core::ffi::c_int,
            (*bd).endspaces as size_t,
        );
        pnew = pnew.offset((*bd).endspaces as isize);
        if exclude_trailing_space {
            let mut s: ::core::ffi::c_int = (*bd).textlen + (*bd).endspaces;
            while s > 0 as ::core::ffi::c_int
                && ascii_iswhite(
                    *(*bd)
                        .textstart
                        .offset(s as isize)
                        .offset(-(1 as ::core::ffi::c_int as isize))
                        as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                s =
                    s - utf_head_off(
                        (*bd).textstart,
                        (*bd)
                            .textstart
                            .offset(s as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize)),
                    ) - 1 as ::core::ffi::c_int;
                pnew = pnew.offset(-1);
            }
        }
        *pnew = NUL as ::core::ffi::c_char;
        (*(*reg).y_array.offset(y_idx as isize)).size =
            pnew.offset_from((*(*reg).y_array.offset(y_idx as isize)).data) as size_t;
    }
}

pub unsafe extern "C" fn op_yank_reg(
    mut oap: *mut oparg_T,
    mut message: bool,
    mut reg: *mut yankreg_T,
    mut append: bool,
) {
    unsafe {
        let mut newreg: yankreg_T = yankreg_T {
            y_array: ::core::ptr::null_mut::<String_0>(),
            y_size: 0,
            y_type: kMTCharWise,
            y_width: 0,
            timestamp: 0,
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        };
        let mut yank_type: MotionType = (*oap).motion_type;
        let mut yanklines: size_t = (*oap).line_count as size_t;
        let mut yankendlnum: linenr_T = (*oap).end.lnum;
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
        let mut curr: *mut yankreg_T = reg;
        if append as ::core::ffi::c_int != 0 && !(*reg).y_array.is_null() {
            reg = &raw mut newreg;
        } else {
            free_register(reg);
        }
        if (*oap).motion_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            && (*oap).start.col == 0 as ::core::ffi::c_int
            && !(*oap).inclusive
            && (!(*oap).is_VIsual
                || *p_sel.get() as ::core::ffi::c_int == 'o' as ::core::ffi::c_int)
            && (*oap).end.col == 0 as ::core::ffi::c_int
            && yanklines > 1 as size_t
        {
            yank_type = kMTLineWise;
            yankendlnum -= 1;
            yanklines = yanklines.wrapping_sub(1);
        }
        (*reg).y_size = yanklines;
        (*reg).y_type = yank_type;
        (*reg).y_width = 0 as ::core::ffi::c_int as colnr_T;
        (*reg).y_array = xcalloc(yanklines, ::core::mem::size_of::<String_0>()) as *mut String_0;
        (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        (*reg).timestamp = os_time();
        let mut y_idx: size_t = 0 as size_t;
        let mut lnum: linenr_T = (*oap).start.lnum;
        if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
            (*reg).y_width = (*oap).end_vcol - (*oap).start_vcol;
            if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int
                && (*reg).y_width > 0 as ::core::ffi::c_int
            {
                (*reg).y_width -= 1;
            }
        }
        while lnum <= yankendlnum {
            let mut tmp: ::core::ffi::c_int = 0;
            match (*reg).y_type as ::core::ffi::c_int {
                2 => {
                    block_prep(oap, &raw mut bd, lnum, false_0 != 0);
                    yank_copy_line(reg, &raw mut bd, y_idx, (*oap).excl_tr_ws);
                }
                1 => {
                    *(*reg).y_array.offset(y_idx as isize) =
                        cbuf_to_string(ml_get(lnum), ml_get_len(lnum) as size_t);
                }
                0 => {
                    charwise_block_prep(
                        (*oap).start,
                        (*oap).end,
                        &raw mut bd,
                        lnum,
                        (*oap).inclusive,
                    );
                    tmp = strlen(bd.textstart) as ::core::ffi::c_int;
                    if tmp < bd.textlen {
                        bd.textlen = tmp;
                    }
                    yank_copy_line(reg, &raw mut bd, y_idx, false_0 != 0);
                }
                -1 => {
                    abort();
                }
                _ => {}
            }
            lnum += 1;
            y_idx = y_idx.wrapping_add(1);
        }
        if curr != reg {
            let mut j: size_t = 0;
            let mut new_ptr: *mut String_0 = xmalloc(
                ::core::mem::size_of::<String_0>()
                    .wrapping_mul((*curr).y_size.wrapping_add((*reg).y_size)),
            ) as *mut String_0;
            j = 0 as size_t;
            while j < (*curr).y_size {
                *new_ptr.offset(j as isize) = *(*curr).y_array.offset(j as isize);
                j = j.wrapping_add(1);
            }
            xfree((*curr).y_array as *mut ::core::ffi::c_void);
            (*curr).y_array = new_ptr;
            if yank_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                (*curr).y_type = kMTLineWise;
            }
            if (*curr).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                && vim_strchr(p_cpo.get(), CPO_REGAPPEND).is_null()
            {
                let mut pnew: *mut ::core::ffi::c_char = xmalloc(
                    (*(*curr)
                        .y_array
                        .offset((*curr).y_size.wrapping_sub(1 as size_t) as isize))
                    .size
                    .wrapping_add((*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size)
                    .wrapping_add(1 as size_t),
                )
                    as *mut ::core::ffi::c_char;
                j = j.wrapping_sub(1);
                strcpy(pnew, (*(*curr).y_array.offset(j as isize)).data);
                strcpy(
                    pnew.offset((*(*curr).y_array.offset(j as isize)).size as isize),
                    (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).data,
                );
                xfree((*(*curr).y_array.offset(j as isize)).data as *mut ::core::ffi::c_void);
                *(*curr).y_array.offset(j as isize) = String_0 {
                    data: pnew,
                    size: (*(*curr).y_array.offset(j as isize)).size.wrapping_add(
                        (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size,
                    ),
                };
                j = j.wrapping_add(1);
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).data
                        as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                let _ = *ptr_;
                (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size = 0 as size_t;
                y_idx = 1 as size_t;
            } else {
                y_idx = 0 as size_t;
            }
            while y_idx < (*reg).y_size {
                let c2rust_fresh2 = y_idx;
                y_idx = y_idx.wrapping_add(1);
                let c2rust_fresh3 = j;
                j = j.wrapping_add(1);
                *(*curr).y_array.offset(c2rust_fresh3 as isize) =
                    *(*reg).y_array.offset(c2rust_fresh2 as isize);
            }
            (*curr).y_size = j;
            xfree((*reg).y_array as *mut ::core::ffi::c_void);
        }
        if message {
            if yank_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                && yanklines == 1 as size_t
            {
                yanklines = 0 as size_t;
            }
            if yanklines > p_report.get() as size_t {
                let mut namebuf: [::core::ffi::c_char; 100] = [0; 100];
                if (*oap).regname == NUL {
                    *(&raw mut namebuf as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
                } else {
                    vim_snprintf(
                        &raw mut namebuf as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
                        gettext(b" into \"%c\0".as_ptr() as *const ::core::ffi::c_char),
                        (*oap).regname,
                    );
                }
                update_topline(curwin.get());
                if must_redraw.get() != 0 {
                    update_screen();
                }
                if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    smsg(
                        0 as ::core::ffi::c_int,
                        ngettext(
                            b"block of %ld line yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                            b"block of %ld lines yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                            yanklines as ::core::ffi::c_ulong,
                        ),
                        yanklines as int64_t,
                        &raw mut namebuf as *mut ::core::ffi::c_char,
                    );
                } else {
                    smsg(
                        0 as ::core::ffi::c_int,
                        ngettext(
                            b"%ld line yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                            b"%ld lines yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                            yanklines as ::core::ffi::c_ulong,
                        ),
                        yanklines as int64_t,
                        &raw mut namebuf as *mut ::core::ffi::c_char,
                    );
                }
            }
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_start = (*oap).start;
            (*curbuf.get()).b_op_end = (*oap).end;
            if yank_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                (*curbuf.get()).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
                (*curbuf.get()).b_op_end.col = MAXCOL as ::core::ffi::c_int as colnr_T;
            }
            if yank_type as ::core::ffi::c_int != kMTLineWise as ::core::ffi::c_int
                && !(*oap).inclusive
            {
                decl(&raw mut (*curbuf.get()).b_op_end);
            }
        }
    }
}

pub unsafe extern "C" fn format_reg_type(
    mut reg_type: MotionType,
    mut reg_width: colnr_T,
    mut buf: *mut ::core::ffi::c_char,
    mut buf_len: size_t,
) {
    unsafe {
        '_c2rust_label: {
            if buf_len > 1 as size_t {
            } else {
                __assert_fail(
                    b"buf_len > 1\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1176 as ::core::ffi::c_uint,
                    b"void format_reg_type(MotionType, colnr_T, char *, size_t)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        match reg_type as ::core::ffi::c_int {
            1 => {
                *buf.offset(0 as ::core::ffi::c_int as isize) = 'V' as ::core::ffi::c_char;
                *buf.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            }
            0 => {
                *buf.offset(0 as ::core::ffi::c_int as isize) = 'v' as ::core::ffi::c_char;
                *buf.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            }
            2 => {
                snprintf(
                    buf,
                    buf_len,
                    b"\x16%d\0".as_ptr() as *const ::core::ffi::c_char,
                    reg_width as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                );
            }
            -1 => {
                *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            }
            _ => {}
        };
    }
}

pub unsafe extern "C" fn do_autocmd_textyankpost(mut oap: *mut oparg_T, mut reg: *mut yankreg_T) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if recursive.get() as ::core::ffi::c_int != 0 || !has_event(EVENT_TEXTYANKPOST) {
            return;
        }
        recursive.set(true_0 != 0);
        let mut save_v_event: save_v_event_T = save_v_event_T {
            sve_did_save: false,
            sve_hashtab: hashtab_T {
                ht_mask: 0,
                ht_used: 0,
                ht_filled: 0,
                ht_changed: 0,
                ht_locked: 0,
                ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                ht_smallarray: [hashitem_T {
                    hi_hash: 0,
                    hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                }; 16],
            },
        };
        let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
        let list: *mut list_T = tv_list_alloc((*reg).y_size as ptrdiff_t);
        let mut i: size_t = 0 as size_t;
        while i < (*reg).y_size {
            tv_list_append_string(
                list,
                (*(*reg).y_array.offset(i as isize)).data,
                (*(*reg).y_array.offset(i as isize)).size as ::core::ffi::c_int as ssize_t,
            );
            i = i.wrapping_add(1);
        }
        tv_list_set_lock(list, VAR_FIXED);
        tv_dict_add_list(
            dict,
            b"regcontents\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
            list,
        );
        let mut buf: [::core::ffi::c_char; 67] = [0; 67];
        format_reg_type(
            (*reg).y_type,
            (*reg).y_width,
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 67]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 67]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
        );
        tv_dict_add_str(
            dict,
            b"regtype\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        buf[0 as ::core::ffi::c_int as usize] = (*oap).regname as ::core::ffi::c_char;
        buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        tv_dict_add_str(
            dict,
            b"regname\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        tv_dict_add_bool(
            dict,
            b"inclusive\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            (if (*oap).inclusive as ::core::ffi::c_int != 0 {
                kBoolVarTrue as ::core::ffi::c_int
            } else {
                kBoolVarFalse as ::core::ffi::c_int
            }) as BoolVarValue,
        );
        buf[0 as ::core::ffi::c_int as usize] = get_op_char((*oap).op_type) as ::core::ffi::c_char;
        buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        tv_dict_add_str(
            dict,
            b"operator\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        tv_dict_add_bool(
            dict,
            b"visual\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            (if (*oap).is_VIsual as ::core::ffi::c_int != 0 {
                kBoolVarTrue as ::core::ffi::c_int
            } else {
                kBoolVarFalse as ::core::ffi::c_int
            }) as BoolVarValue,
        );
        tv_dict_set_keys_readonly(dict);
        (*textlock.ptr()) += 1;
        apply_autocmds(
            EVENT_TEXTYANKPOST,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*textlock.ptr()) -= 1;
        restore_v_event(dict, &raw mut save_v_event);
        recursive.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn op_yank(mut oap: *mut oparg_T, mut message: bool) -> bool {
    unsafe {
        if (*oap).regname != 0 as ::core::ffi::c_int && !valid_yank_reg((*oap).regname, true_0 != 0)
        {
            beep_flush();
            return false_0 != 0;
        }
        if (*oap).regname == '_' as ::core::ffi::c_int {
            return true_0 != 0;
        }
        let mut reg: *mut yankreg_T =
            get_yank_register((*oap).regname, YREG_YANK as ::core::ffi::c_int);
        op_yank_reg(oap, message, reg, is_append_register((*oap).regname));
        clipboard::set_clipboard((*oap).regname, reg);
        do_autocmd_textyankpost(oap, reg);
        return true_0 != 0;
    }
}
