//! `:highlight` with no settings to apply: printing what is set.
//!
//! [`highlight_list_one`] prints one group as the `key=value` pairs that
//! would recreate it, [`highlight_list_arg`] formats one such pair and
//! [`syn_list_header`] does the column arithmetic that keeps the output in
//! line. The `get_highlight_name*` pair is command-line completion.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub const LIST_ATTR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

pub const LIST_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const LIST_INT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;

pub(crate) unsafe extern "C" fn highlight_list_one(id: ::core::ffi::c_int) {
    unsafe {
        let mut sgp: *const HlGroup = ((*highlight_ga.ptr()).ga_data as *mut HlGroup)
            .offset((id - 1 as ::core::ffi::c_int) as isize);
        let mut didh: bool = false_0 != 0;
        if message_filtered((*sgp).sg_name) {
            return;
        }
        if (*sgp).sg_parent != 0 && (*sgp).sg_cleared as ::core::ffi::c_int != 0 {
            return;
        }
        didh = highlight_list_arg(
            id,
            didh,
            LIST_ATTR,
            (*sgp).sg_cterm,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"cterm\0".as_ptr() as *const ::core::ffi::c_char,
        );
        didh = highlight_list_arg(
            id,
            didh,
            LIST_INT,
            (*sgp).sg_cterm_fg,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ctermfg\0".as_ptr() as *const ::core::ffi::c_char,
        );
        didh = highlight_list_arg(
            id,
            didh,
            LIST_INT,
            (*sgp).sg_cterm_bg,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ctermbg\0".as_ptr() as *const ::core::ffi::c_char,
        );
        didh = highlight_list_arg(
            id,
            didh,
            LIST_ATTR,
            (*sgp).sg_gui,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"gui\0".as_ptr() as *const ::core::ffi::c_char,
        );
        let mut hexbuf: HexBuf = [0; 8];
        didh = highlight_list_arg(
            id,
            didh,
            LIST_STRING,
            0 as ::core::ffi::c_int,
            coloridx_to_name(
                (*sgp).sg_rgb_fg_idx,
                (*sgp).sg_rgb_fg as ::core::ffi::c_int,
                &mut hexbuf,
            )
            .map_or(::core::ptr::null(), |s| s.as_ptr()),
            b"guifg\0".as_ptr() as *const ::core::ffi::c_char,
        );
        didh = highlight_list_arg(
            id,
            didh,
            LIST_STRING,
            0 as ::core::ffi::c_int,
            coloridx_to_name(
                (*sgp).sg_rgb_bg_idx,
                (*sgp).sg_rgb_bg as ::core::ffi::c_int,
                &mut hexbuf,
            )
            .map_or(::core::ptr::null(), |s| s.as_ptr()),
            b"guibg\0".as_ptr() as *const ::core::ffi::c_char,
        );
        didh = highlight_list_arg(
            id,
            didh,
            LIST_STRING,
            0 as ::core::ffi::c_int,
            coloridx_to_name(
                (*sgp).sg_rgb_sp_idx,
                (*sgp).sg_rgb_sp as ::core::ffi::c_int,
                &mut hexbuf,
            )
            .map_or(::core::ptr::null(), |s| s.as_ptr()),
            b"guisp\0".as_ptr() as *const ::core::ffi::c_char,
        );
        didh = highlight_list_arg(
            id,
            didh,
            LIST_INT,
            (*sgp).sg_blend + 1 as ::core::ffi::c_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"blend\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if (*sgp).sg_link != 0 && !got_int.get() {
            syn_list_header(didh, 0 as ::core::ffi::c_int, id, true_0 != 0);
            didh = true_0 != 0;
            msg_puts_hl(
                b"links to\0".as_ptr() as *const ::core::ffi::c_char,
                HLF_D,
                false_0 != 0,
            );
            msg_putchar(' ' as ::core::ffi::c_int);
            msg_outtrans(
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(
                    ((*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                        .offset((id - 1 as ::core::ffi::c_int) as isize))
                    .sg_link
                        - 1 as ::core::ffi::c_int) as isize,
                ))
                .sg_name,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        if !didh {
            highlight_list_arg(
                id,
                didh,
                LIST_STRING,
                0 as ::core::ffi::c_int,
                b"cleared\0".as_ptr() as *const ::core::ffi::c_char,
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if p_verbose.get() > 0 as OptInt {
            last_set_msg((*sgp).sg_script_ctx);
        }
    }
}

pub(crate) unsafe extern "C" fn highlight_list_arg(
    id: ::core::ffi::c_int,
    mut didh: bool,
    type_0: ::core::ffi::c_int,
    mut iarg: ::core::ffi::c_int,
    mut sarg: *const ::core::ffi::c_char,
    name: *const ::core::ffi::c_char,
) -> bool {
    unsafe {
        if got_int.get() {
            return false_0 != 0;
        }
        if if type_0 == LIST_STRING {
            sarg.is_null() as ::core::ffi::c_int
        } else {
            (iarg == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } != 0
        {
            return didh;
        }
        let mut buf: [::core::ffi::c_char; 100] = [0; 100];
        let mut ts: *const ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
        if type_0 == LIST_INT {
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
                b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                iarg - 1 as ::core::ffi::c_int,
            );
        } else if type_0 == LIST_STRING {
            ts = sarg;
        } else {
            buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while (*hl_attr_table.ptr())[i as usize] != 0 as ::core::ffi::c_int {
                if (*hl_attr_table.ptr())[i as usize] & HL_UNDERLINE_MASK != 0
                    && iarg & HL_UNDERLINE_MASK == (*hl_attr_table.ptr())[i as usize]
                    || (*hl_attr_table.ptr())[i as usize] & HL_UNDERLINE_MASK == 0
                        && iarg & (*hl_attr_table.ptr())[i as usize] != 0
                {
                    if buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL {
                        xstrlcat(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            b",\0".as_ptr() as *const ::core::ffi::c_char,
                            100 as size_t,
                        );
                    }
                    xstrlcat(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        (*hl_name_table.ptr())[i as usize] as *const ::core::ffi::c_char,
                        100 as size_t,
                    );
                    if (*hl_attr_table.ptr())[i as usize] & HL_UNDERLINE_MASK == 0 {
                        iarg &= !(*hl_attr_table.ptr())[i as usize];
                    }
                }
                i += 1;
            }
        }
        syn_list_header(
            didh,
            vim_strsize(ts) + strlen(name) as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            id,
            false_0 != 0,
        );
        didh = true_0 != 0;
        if !got_int.get() {
            if *name as ::core::ffi::c_int != NUL {
                msg_puts_hl(name, HLF_D, false_0 != 0);
                msg_puts_hl(
                    b"=\0".as_ptr() as *const ::core::ffi::c_char,
                    HLF_D,
                    false_0 != 0,
                );
            }
            msg_outtrans(ts, 0 as ::core::ffi::c_int, false_0 != 0);
        }
        return didh;
    }
}

pub unsafe extern "C" fn syn_list_header(
    did_header: bool,
    outlen: ::core::ffi::c_int,
    id: ::core::ffi::c_int,
    mut force_newline: bool,
) -> bool {
    unsafe {
        let mut endcol: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
        let mut newline: bool = true_0 != 0;
        let mut name_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut adjust: bool = true_0 != 0;
        if !did_header {
            if !ui_has(kUIMessages) || msg_col.get() > 0 as ::core::ffi::c_int {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            if got_int.get() {
                return true_0 != 0;
            }
            name_col = msg_outtrans(
                (*((*highlight_ga.ptr()).ga_data as *mut HlGroup)
                    .offset((id - 1 as ::core::ffi::c_int) as isize))
                .sg_name,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
            msg_col.set(name_col);
            endcol = 15 as ::core::ffi::c_int;
        } else if (ui_has(kUIMessages) as ::core::ffi::c_int != 0 || msg_silent.get() != 0)
            && !force_newline
        {
            msg_putchar(' ' as ::core::ffi::c_int);
            adjust = false_0 != 0;
        } else if msg_col.get() + outlen + 1 as ::core::ffi::c_int >= Columns.get()
            || force_newline as ::core::ffi::c_int != 0
        {
            msg_putchar('\n' as ::core::ffi::c_int);
            if got_int.get() {
                return true_0 != 0;
            }
        } else if msg_col.get() >= endcol {
            newline = false_0 != 0;
        }
        if adjust {
            if msg_col.get() >= endcol {
                endcol = msg_col.get() + 1 as ::core::ffi::c_int;
            }
            msg_advance(endcol);
        }
        if !did_header {
            if endcol == Columns.get() - 1 as ::core::ffi::c_int && endcol <= name_col {
                msg_putchar(' ' as ::core::ffi::c_int);
            }
            msg_puts_hl(
                b"xxx\0".as_ptr() as *const ::core::ffi::c_char,
                id,
                false_0 != 0,
            );
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        return newline;
    }
}

pub unsafe extern "C" fn set_context_in_highlight_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    unsafe {
        (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        include_link.set(2 as ::core::ffi::c_int);
        include_default.set(1 as ::core::ffi::c_int);
        if *arg as ::core::ffi::c_int == NUL {
            return;
        }
        let mut p: *const ::core::ffi::c_char = skiptowhite(arg);
        if *p as ::core::ffi::c_int == NUL {
            return;
        }
        include_default.set(0 as ::core::ffi::c_int);
        if strncmp(
            b"default\0".as_ptr() as *const ::core::ffi::c_char,
            arg,
            p.offset_from(arg) as ::core::ffi::c_uint as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = skipwhite(p);
            (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
            p = skiptowhite(arg);
        }
        if *p as ::core::ffi::c_int == NUL {
            return;
        }
        include_link.set(0 as ::core::ffi::c_int);
        if *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'i' as ::core::ffi::c_int
            && *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'N' as ::core::ffi::c_int
        {
            highlight_list();
        }
        if strncmp(
            b"link\0".as_ptr() as *const ::core::ffi::c_char,
            arg,
            p.offset_from(arg) as ::core::ffi::c_uint as size_t,
        ) == 0 as ::core::ffi::c_int
            || strncmp(
                b"clear\0".as_ptr() as *const ::core::ffi::c_char,
                arg,
                p.offset_from(arg) as ::core::ffi::c_uint as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            (*xp).xp_pattern = skipwhite(p);
            p = skiptowhite((*xp).xp_pattern);
            if *p as ::core::ffi::c_int != NUL {
                (*xp).xp_pattern = skipwhite(p);
                p = skiptowhite((*xp).xp_pattern);
            }
        }
        if *p as ::core::ffi::c_int != NUL {
            (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        }
    }
}

pub(crate) unsafe extern "C" fn highlight_list() {
    unsafe {
        let mut i: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
        loop {
            i -= 1;
            if i < 0 as ::core::ffi::c_int {
                break;
            }
            highlight_list_two(i, HLF_D);
        }
        let mut i_0: ::core::ffi::c_int = 40 as ::core::ffi::c_int;
        loop {
            i_0 -= 1;
            if i_0 < 0 as ::core::ffi::c_int {
                break;
            }
            highlight_list_two(99 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        }
    }
}

pub(crate) unsafe extern "C" fn highlight_list_two(
    mut cnt: ::core::ffi::c_int,
    mut id: ::core::ffi::c_int,
) {
    unsafe {
        msg_puts_hl(
            (b"N \x08I \x08!  \x08\0".as_ptr() as *const ::core::ffi::c_char)
                .offset((cnt / 11 as ::core::ffi::c_int) as isize),
            id,
            false_0 != 0,
        );
        msg_clr_eos();
        ui_flush();
        os_delay(
            if cnt == 99 as ::core::ffi::c_int {
                40 as uint64_t
            } else {
                (cnt as uint64_t).wrapping_mul(50 as uint64_t)
            },
            false_0 != 0,
        );
    }
}

pub unsafe extern "C" fn get_highlight_name(
    xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        return get_highlight_name_ext(xp, idx, true_0 != 0) as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn get_highlight_name_ext(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
    mut skip_cleared: bool,
) -> *const ::core::ffi::c_char {
    unsafe {
        if idx < 0 as ::core::ffi::c_int {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        if skip_cleared as ::core::ffi::c_int != 0
            && idx < (*highlight_ga.ptr()).ga_len
            && (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_cleared
                as ::core::ffi::c_int
                != 0
        {
            return b"\0".as_ptr() as *const ::core::ffi::c_char;
        }
        if idx == (*highlight_ga.ptr()).ga_len && include_none.get() != 0 as ::core::ffi::c_int {
            return b"none\0".as_ptr() as *const ::core::ffi::c_char;
        } else if idx == (*highlight_ga.ptr()).ga_len + include_none.get()
            && include_default.get() != 0 as ::core::ffi::c_int
        {
            return b"default\0".as_ptr() as *const ::core::ffi::c_char;
        } else if idx == (*highlight_ga.ptr()).ga_len + include_none.get() + include_default.get()
            && include_link.get() != 0 as ::core::ffi::c_int
        {
            return b"link\0".as_ptr() as *const ::core::ffi::c_char;
        } else if idx
            == (*highlight_ga.ptr()).ga_len
                + include_none.get()
                + include_default.get()
                + 1 as ::core::ffi::c_int
            && include_link.get() != 0 as ::core::ffi::c_int
        {
            return b"clear\0".as_ptr() as *const ::core::ffi::c_char;
        } else if idx >= (*highlight_ga.ptr()).ga_len {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        return (*((*highlight_ga.ptr()).ga_data as *mut HlGroup).offset(idx as isize)).sg_name;
    }
}
