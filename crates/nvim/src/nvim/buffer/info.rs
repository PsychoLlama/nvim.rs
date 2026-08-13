//! Describing a buffer to the user -- `:ls`, CTRL-G and the title.
//!
//! [`buflist_list`] is `:ls`/`:buffers`, including the flag column and the
//! `'columns'`-aware truncation of each name; [`fileinfo`] is CTRL-G and the
//! message a `:edit` prints; [`get_rel_pos`] is the ruler's "Top"/"Bot"/"NN%"
//! and [`append_arg_number`] the `(2 of 5)` suffix.  [`maketitle`] builds
//! `'title'` and `'icon'` -- the same information again, for the window
//! manager.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::charset::{trans_characters, vim_strsize};
use crate::src::nvim::drawscreen::redrawing;
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::main::{
    Columns, IObuff, NameBuff, curbuf, curwin, firstbuf, got_int, msg_col, msg_scroll,
    msg_scrolled, need_maketitle, need_wait_return, no_lines_msg, p_icon, p_iconstring, p_ru,
    p_title, p_titlelen, p_titlestring, restart_edit, stl_syntax,
};
use crate::src::nvim::mbyte::utf_cp_bounds;
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    message_filtered, msg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_start, msg_trunc,
    set_keep_msg,
};
use crate::src::nvim::r#move::validate_virtcol;
use crate::src::nvim::option::shortmess;
use crate::src::nvim::options::{kOptIconstring, kOptTitlestring};
use crate::src::nvim::os::env::home_replace;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{gettext, ngettext, qsort, strcmp, strcpy, strlen};
use crate::src::nvim::path::path_tail;
use crate::src::nvim::plines::win_get_fill;
use crate::src::nvim::statusline::build_stl_str_hl;
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_safelen, vim_strchr};
use crate::src::nvim::terminal::terminal_running;
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    OptInt, StlClickRecord, buf_T, exarg_T, garray_T, int64_t, linenr_T, schar_T, size_t,
    statuscol_T, stl_hlrec_t, win_T,
};
use crate::src::nvim::ui::{ui_call_set_icon, ui_call_set_title, ui_has};
use crate::src::nvim::undo::{bufIsChanged, curbufIsChanged, undo_fmt_time};

pub unsafe fn buflist_list(mut eap: *mut exarg_T) {
    unsafe {
        let mut buf: *mut buf_T = firstbuf.get();
        let mut buflist: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut buflist_data: *mut *mut buf_T = ::core::ptr::null_mut::<*mut buf_T>();
        msg_ext_set_kind(c"list_cmd".as_ptr());
        if !vim_strchr((*eap).arg, 't' as ::core::ffi::c_int).is_null() {
            ga_init(
                &raw mut buflist,
                ::core::mem::size_of::<*mut buf_T>() as ::core::ffi::c_int,
                50 as ::core::ffi::c_int,
            );
            buf = firstbuf.get();
            while !buf.is_null() {
                ga_grow(&raw mut buflist, 1 as ::core::ffi::c_int);
                let c2rust_fresh4 = buflist.ga_len;
                buflist.ga_len = buflist.ga_len + 1;
                let c2rust_lvalue_ptr =
                    &raw mut *(buflist.ga_data as *mut *mut buf_T).offset(c2rust_fresh4 as isize);
                *c2rust_lvalue_ptr = buf;
                buf = (*buf).b_next;
            }
            qsort(
                buflist.ga_data,
                buflist.ga_len as size_t,
                ::core::mem::size_of::<*mut buf_T>(),
                Some(
                    buf_time_compare
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
            buflist_data = buflist.ga_data as *mut *mut buf_T;
            buf = *buflist_data;
        }
        let mut p: *mut *mut buf_T = buflist_data;
        while !buf.is_null() && !got_int.get() {
            let is_terminal: bool = !(*buf).terminal.is_null();
            let job_running: bool = !(*buf).terminal.is_null()
                && terminal_running((*buf).terminal) as ::core::ffi::c_int != 0;
            if !((*buf).b_p_bl == 0
                && (*eap).forceit == 0
                && vim_strchr((*eap).arg, 'u' as ::core::ffi::c_int).is_null()
                || !vim_strchr((*eap).arg, 'u' as ::core::ffi::c_int).is_null()
                    && (*buf).b_p_bl != 0
                || !vim_strchr((*eap).arg, '+' as ::core::ffi::c_int).is_null()
                    && ((*buf).b_flags & BF_READERR != 0 || !bufIsChanged(buf))
                || !vim_strchr((*eap).arg, 'a' as ::core::ffi::c_int).is_null()
                    && ((*buf).b_ml.ml_mfp.is_null()
                        || (*buf).b_nwindows == 0 as ::core::ffi::c_int)
                || !vim_strchr((*eap).arg, 'h' as ::core::ffi::c_int).is_null()
                    && ((*buf).b_ml.ml_mfp.is_null()
                        || (*buf).b_nwindows != 0 as ::core::ffi::c_int)
                || !vim_strchr((*eap).arg, 'R' as ::core::ffi::c_int).is_null()
                    && (!is_terminal || !job_running)
                || !vim_strchr((*eap).arg, 'F' as ::core::ffi::c_int).is_null()
                    && (!is_terminal || job_running as ::core::ffi::c_int != 0)
                || !vim_strchr((*eap).arg, '-' as ::core::ffi::c_int).is_null()
                    && (*buf).b_p_ma != 0
                || !vim_strchr((*eap).arg, '=' as ::core::ffi::c_int).is_null()
                    && (*buf).b_p_ro == 0
                || !vim_strchr((*eap).arg, 'x' as ::core::ffi::c_int).is_null()
                    && (*buf).b_flags & BF_READERR == 0
                || !vim_strchr((*eap).arg, '%' as ::core::ffi::c_int).is_null()
                    && buf != curbuf.get()
                || !vim_strchr((*eap).arg, '#' as ::core::ffi::c_int).is_null()
                    && (buf == curbuf.get() || (*curwin.get()).w_alt_fnum != (*buf).handle))
            {
                let mut name: *mut ::core::ffi::c_char = buf_spname(buf);
                if !name.is_null() {
                    xstrlcpy(
                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                        name,
                        MAXPATHL as size_t,
                    );
                } else {
                    home_replace(
                        buf,
                        (*buf).b_fname,
                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                        true_0 != 0,
                    );
                }
                if !message_filtered(NameBuff.ptr() as *mut ::core::ffi::c_char) {
                    let changed_char: ::core::ffi::c_int = if (*buf).b_flags & BF_READERR != 0 {
                        'x' as ::core::ffi::c_int
                    } else if bufIsChanged(buf) as ::core::ffi::c_int != 0 {
                        '+' as ::core::ffi::c_int
                    } else {
                        ' ' as ::core::ffi::c_int
                    };
                    let mut ro_char: ::core::ffi::c_int = if (*buf).b_p_ma == 0 {
                        '-' as ::core::ffi::c_int
                    } else if (*buf).b_p_ro != 0 {
                        '=' as ::core::ffi::c_int
                    } else {
                        ' ' as ::core::ffi::c_int
                    };
                    if !(*buf).terminal.is_null() {
                        ro_char = if terminal_running((*buf).terminal) as ::core::ffi::c_int != 0 {
                            'R' as ::core::ffi::c_int
                        } else {
                            'F' as ::core::ffi::c_int
                        };
                    }
                    if !ui_has(kUIMessages) || msg_col.get() > 0 as ::core::ffi::c_int {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    }
                    let mut len: ::core::ffi::c_int = vim_snprintf_safelen(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        (IOSIZE - 20 as ::core::ffi::c_int) as size_t,
                        c"%3d%c%c%c%c%c \"%s\"".as_ptr(),
                        (*buf).handle,
                        if (*buf).b_p_bl != 0 {
                            ' ' as ::core::ffi::c_int
                        } else {
                            'u' as ::core::ffi::c_int
                        },
                        if buf == curbuf.get() {
                            '%' as ::core::ffi::c_int
                        } else if (*curwin.get()).w_alt_fnum == (*buf).handle {
                            '#' as ::core::ffi::c_int
                        } else {
                            ' ' as ::core::ffi::c_int
                        },
                        if (*buf).b_ml.ml_mfp.is_null() {
                            ' ' as ::core::ffi::c_int
                        } else if (*buf).b_nwindows == 0 as ::core::ffi::c_int {
                            'h' as ::core::ffi::c_int
                        } else {
                            'a' as ::core::ffi::c_int
                        },
                        ro_char,
                        changed_char,
                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                    ) as ::core::ffi::c_int;
                    len = if len
                        < 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                            - 20 as ::core::ffi::c_int
                    {
                        len
                    } else {
                        1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                            - 20 as ::core::ffi::c_int
                    };
                    let mut i: ::core::ffi::c_int = 40 as ::core::ffi::c_int
                        - vim_strsize(IObuff.ptr() as *mut ::core::ffi::c_char);
                    loop {
                        let c2rust_fresh5 = len;
                        len = len + 1;
                        (*IObuff.ptr())[c2rust_fresh5 as usize] = ' ' as ::core::ffi::c_char;
                        i -= 1;
                        if !(i > 0 as ::core::ffi::c_int && len < IOSIZE - 18 as ::core::ffi::c_int)
                        {
                            break;
                        }
                    }
                    if !vim_strchr((*eap).arg, 't' as ::core::ffi::c_int).is_null()
                        && (*buf).b_last_used != 0
                    {
                        undo_fmt_time(
                            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                            (IOSIZE - len) as size_t,
                            (*buf).b_last_used,
                        );
                    } else {
                        vim_snprintf(
                            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                            (IOSIZE - len) as size_t,
                            gettext(c"line %ld".as_ptr()),
                            if buf == curbuf.get() {
                                (*curwin.get()).w_cursor.lnum as int64_t
                            } else {
                                buflist_findlnum(buf) as int64_t
                            },
                        );
                    }
                    msg_outtrans(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                    line_breakcheck();
                }
            }
            buf = if !buflist_data.is_null() {
                p = p.offset(1);
                if p < buflist_data.offset(buflist.ga_len as isize) {
                    *p
                } else {
                    ::core::ptr::null_mut::<buf_T>()
                }
            } else {
                (*buf).b_next
            };
        }
        if !buflist_data.is_null() {
            ga_clear(&raw mut buflist);
        }
    }
}

pub unsafe extern "C" fn fileinfo(
    mut fullname: ::core::ffi::c_int,
    mut shorthelp: ::core::ffi::c_int,
    mut dont_truncate: bool,
) {
    unsafe {
        let mut buffer: *mut ::core::ffi::c_char =
            xmalloc(IOSIZE as size_t) as *mut ::core::ffi::c_char;
        let mut bufferlen: size_t = 0 as size_t;
        if fullname > 1 as ::core::ffi::c_int {
            bufferlen = vim_snprintf_safelen(
                buffer,
                IOSIZE as size_t,
                c"buf %d: ".as_ptr(),
                (*curbuf.get()).handle,
            );
        }
        let c2rust_fresh6 = bufferlen;
        bufferlen = bufferlen.wrapping_add(1);
        *buffer.add(c2rust_fresh6) = '"' as ::core::ffi::c_char;
        let mut name: *mut ::core::ffi::c_char = buf_spname(curbuf.get());
        if !name.is_null() {
            bufferlen = bufferlen.wrapping_add(vim_snprintf_safelen(
                buffer.add(bufferlen),
                (IOSIZE as size_t).wrapping_sub(bufferlen),
                c"%s".as_ptr(),
                name,
            ));
        } else {
            name = if fullname == 0 && !(*curbuf.get()).b_fname.is_null() {
                (*curbuf.get()).b_fname
            } else {
                (*curbuf.get()).b_ffname
            };
            home_replace(
                if shorthelp != 0 {
                    curbuf.get()
                } else {
                    ::core::ptr::null_mut::<buf_T>()
                },
                name,
                buffer.add(bufferlen),
                (IOSIZE as size_t).wrapping_sub(bufferlen),
                true_0 != 0,
            );
            bufferlen = bufferlen.wrapping_add(strlen(buffer.add(bufferlen)));
        }
        let mut dontwrite: bool = bt_dontwrite(curbuf.get());
        bufferlen = bufferlen.wrapping_add(vim_snprintf_safelen(
            buffer.add(bufferlen),
            (IOSIZE as size_t).wrapping_sub(bufferlen),
            c"\"%s%s%s%s%s%s".as_ptr(),
            if curbufIsChanged() as ::core::ffi::c_int != 0 {
                if shortmess(SHM_MOD as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                    c" [+]".as_ptr()
                } else {
                    gettext(c" [Modified]".as_ptr()) as *const ::core::ffi::c_char
                }
            } else {
                c" ".as_ptr()
            },
            if (*curbuf.get()).b_flags & BF_NOTEDITED != 0 && !dontwrite {
                gettext(c"[Not edited]".as_ptr()) as *const ::core::ffi::c_char
            } else {
                c"".as_ptr()
            },
            if (*curbuf.get()).b_flags & BF_NEW != 0 && !dontwrite {
                gettext(c"[New]".as_ptr()) as *const ::core::ffi::c_char
            } else {
                c"".as_ptr()
            },
            if (*curbuf.get()).b_flags & BF_READERR != 0 {
                gettext(c"[Read errors]".as_ptr()) as *const ::core::ffi::c_char
            } else {
                c"".as_ptr()
            },
            if (*curbuf.get()).b_p_ro != 0 {
                (if shortmess(SHM_RO as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                    gettext(c"[RO]".as_ptr())
                } else {
                    gettext(c"[readonly]".as_ptr())
                }) as *const ::core::ffi::c_char
            } else {
                c"".as_ptr()
            },
            if curbufIsChanged() as ::core::ffi::c_int != 0
                || (*curbuf.get()).b_flags & BF_WRITE_MASK != 0
                || (*curbuf.get()).b_p_ro != 0
            {
                c" ".as_ptr()
            } else {
                c"".as_ptr()
            },
        ));
        if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
            bufferlen = bufferlen.wrapping_add(vim_snprintf_safelen(
                buffer.add(bufferlen),
                (IOSIZE as size_t).wrapping_sub(bufferlen),
                c"%s".as_ptr(),
                gettext(no_lines_msg.ptr() as *mut ::core::ffi::c_char),
            ));
        } else if p_ru.get() != 0 {
            bufferlen = bufferlen.wrapping_add(vim_snprintf_safelen(
                buffer.add(bufferlen),
                (IOSIZE as size_t).wrapping_sub(bufferlen),
                ngettext(
                    c"%ld line --%d%%--".as_ptr(),
                    c"%ld lines --%d%%--".as_ptr(),
                    (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_ulong,
                ),
                (*curbuf.get()).b_ml.ml_line_count as int64_t,
                calc_percentage(
                    (*curwin.get()).w_cursor.lnum as int64_t,
                    (*curbuf.get()).b_ml.ml_line_count as int64_t,
                ),
            ));
        } else {
            bufferlen = bufferlen.wrapping_add(vim_snprintf_safelen(
                buffer.add(bufferlen),
                (IOSIZE as size_t).wrapping_sub(bufferlen),
                gettext(c"line %ld of %ld --%d%%-- col ".as_ptr()),
                (*curwin.get()).w_cursor.lnum as int64_t,
                (*curbuf.get()).b_ml.ml_line_count as int64_t,
                calc_percentage(
                    (*curwin.get()).w_cursor.lnum as int64_t,
                    (*curbuf.get()).b_ml.ml_line_count as int64_t,
                ),
            ));
            validate_virtcol(curwin.get());
            bufferlen = bufferlen.wrapping_add(col_print(
                buffer.add(bufferlen),
                (IOSIZE as size_t).wrapping_sub(bufferlen),
                (*curwin.get()).w_cursor.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                (*curwin.get()).w_virtcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            ) as size_t);
        }
        append_arg_number(
            curwin.get(),
            buffer.add(bufferlen),
            (IOSIZE as size_t).wrapping_sub(bufferlen),
        );
        if dont_truncate {
            msg_start();
            let mut n: ::core::ffi::c_int = msg_scroll.get();
            msg_scroll.set(true_0);
            msg(buffer, 0 as ::core::ffi::c_int);
            msg_scroll.set(n);
        } else {
            let mut p: *mut ::core::ffi::c_char =
                msg_trunc(buffer, false_0 != 0, 0 as ::core::ffi::c_int);
            if restart_edit.get() != 0 as ::core::ffi::c_int
                || msg_scrolled.get() != 0 && !need_wait_return.get()
            {
                set_keep_msg(p, 0 as ::core::ffi::c_int);
            }
        }
        xfree(buffer as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn col_print(
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: size_t,
    mut col: ::core::ffi::c_int,
    mut vcol: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if col == vcol {
            return vim_snprintf_safelen(buf, buflen, c"%d".as_ptr(), col) as ::core::ffi::c_int;
        }
        return vim_snprintf_safelen(buf, buflen, c"%d-%d".as_ptr(), col, vcol)
            as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn maketitle() {
    unsafe {
        let mut title_str: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut icon_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut buf: [::core::ffi::c_char; 1025] = [0; 1025];
        if !redrawing() {
            need_maketitle.set(true_0 != 0);
            return;
        }
        need_maketitle.set(false_0 != 0);
        if p_title.get() == 0
            && p_icon.get() == 0
            && (*lasttitle.ptr()).is_null()
            && (*lasticon.ptr()).is_null()
        {
            return;
        }
        if p_title.get() != 0 {
            let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if p_titlelen.get() > 0 as OptInt {
                maxlen = if (p_titlelen.get() * Columns.get() as OptInt / 100 as OptInt)
                    as ::core::ffi::c_int
                    > 10 as ::core::ffi::c_int
                {
                    (p_titlelen.get() * Columns.get() as OptInt / 100 as OptInt)
                        as ::core::ffi::c_int
                } else {
                    10 as ::core::ffi::c_int
                };
            }
            if *p_titlestring.get() as ::core::ffi::c_int != NUL {
                if stl_syntax.get() & STL_IN_TITLE != 0 {
                    build_stl_str_hl(
                        curwin.get(),
                        &raw mut buf as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                        p_titlestring.get(),
                        kOptTitlestring,
                        0 as ::core::ffi::c_int,
                        0 as schar_T,
                        maxlen,
                        ::core::ptr::null_mut::<*mut stl_hlrec_t>(),
                        ::core::ptr::null_mut::<size_t>(),
                        ::core::ptr::null_mut::<*mut StlClickRecord>(),
                        ::core::ptr::null_mut::<statuscol_T>(),
                    );
                    title_str = &raw mut buf as *mut ::core::ffi::c_char;
                } else {
                    title_str = p_titlestring.get();
                }
            } else {
                let mut default_titlestring: *mut ::core::ffi::c_char =
                    c"%t%( %M%)%( (%{expand('%:p:~:h')})%)%a - Nvim".as_ptr()
                        as *mut ::core::ffi::c_char;
                build_stl_str_hl(
                    curwin.get(),
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                    default_titlestring,
                    kOptTitlestring,
                    0 as ::core::ffi::c_int,
                    0 as schar_T,
                    maxlen,
                    ::core::ptr::null_mut::<*mut stl_hlrec_t>(),
                    ::core::ptr::null_mut::<size_t>(),
                    ::core::ptr::null_mut::<*mut StlClickRecord>(),
                    ::core::ptr::null_mut::<statuscol_T>(),
                );
                title_str = &raw mut buf as *mut ::core::ffi::c_char;
            }
        }
        let mut mustset: bool = value_change(title_str, lasttitle.ptr());
        if p_icon.get() != 0 {
            icon_str = &raw mut buf as *mut ::core::ffi::c_char;
            if *p_iconstring.get() as ::core::ffi::c_int != NUL {
                if stl_syntax.get() & STL_IN_ICON != 0 {
                    build_stl_str_hl(
                        curwin.get(),
                        icon_str,
                        ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                        p_iconstring.get(),
                        kOptIconstring,
                        0 as ::core::ffi::c_int,
                        0 as schar_T,
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<*mut stl_hlrec_t>(),
                        ::core::ptr::null_mut::<size_t>(),
                        ::core::ptr::null_mut::<*mut StlClickRecord>(),
                        ::core::ptr::null_mut::<statuscol_T>(),
                    );
                } else {
                    icon_str = p_iconstring.get();
                }
            } else {
                let mut name: *mut ::core::ffi::c_char = buf_spname(curbuf.get());
                if name.is_null() {
                    name = path_tail((*curbuf.get()).b_ffname);
                }
                let mut namelen: ::core::ffi::c_int = strlen(name) as ::core::ffi::c_int;
                if namelen > 100 as ::core::ffi::c_int {
                    namelen -= 100 as ::core::ffi::c_int;
                    namelen += utf_cp_bounds(name, name.offset(namelen as isize)).end_off
                        as ::core::ffi::c_int;
                    name = name.offset(namelen as isize);
                }
                strcpy(&raw mut buf as *mut ::core::ffi::c_char, name);
                trans_characters(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1025]>() as ::core::ffi::c_int,
                );
            }
        }
        mustset = mustset as ::core::ffi::c_int
            | value_change(icon_str, lasticon.ptr()) as ::core::ffi::c_int
            != 0;
        if mustset {
            resettitle();
        }
    }
}

unsafe extern "C" fn value_change(
    mut str: *mut ::core::ffi::c_char,
    mut last: *mut *mut ::core::ffi::c_char,
) -> bool {
    unsafe {
        if str.is_null() as ::core::ffi::c_int != (*last).is_null() as ::core::ffi::c_int
            || !str.is_null() && !(*last).is_null() && strcmp(str, *last) != 0 as ::core::ffi::c_int
        {
            xfree(*last as *mut ::core::ffi::c_void);
            if str.is_null() {
                *last = ::core::ptr::null_mut::<::core::ffi::c_char>();
                resettitle();
            } else {
                *last = xstrdup(str);
                return true_0 != 0;
            }
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn resettitle() {
    unsafe {
        ui_call_set_icon(cstr_as_string(lasticon.get()));
        ui_call_set_title(cstr_as_string(lasttitle.get()));
    }
}

pub unsafe extern "C" fn get_rel_pos(
    mut wp: *mut win_T,
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if buflen < 3 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let mut above: linenr_T = 0;
        let mut below: linenr_T = 0;
        above = (*wp).w_topline - 1 as linenr_T;
        above = (above as ::core::ffi::c_int
            + (win_get_fill(wp, (*wp).w_topline) - (*wp).w_topfill)) as linenr_T;
        if (*wp).w_topline == 1 as linenr_T && (*wp).w_topfill >= 1 as ::core::ffi::c_int {
            above = 0 as ::core::ffi::c_int as linenr_T;
        }
        below = (*(*wp).w_buffer).b_ml.ml_line_count - (*wp).w_botline + 1 as linenr_T;
        if below <= 0 as linenr_T {
            return vim_snprintf_safelen(
                buf,
                buflen as size_t,
                c"%s".as_ptr(),
                if above == 0 as linenr_T {
                    gettext(c"All".as_ptr())
                } else {
                    gettext(c"Bot".as_ptr())
                },
            ) as ::core::ffi::c_int;
        }
        if above <= 0 as linenr_T {
            return vim_snprintf_safelen(
                buf,
                buflen as size_t,
                c"%s".as_ptr(),
                gettext(c"Top".as_ptr()),
            ) as ::core::ffi::c_int;
        }
        let mut perc: ::core::ffi::c_int =
            calc_percentage(above as int64_t, (above + below) as int64_t);
        let mut tmp: [::core::ffi::c_char; 8] = [0; 8];
        vim_snprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
            gettext(c"%d%%".as_ptr()),
            perc,
        );
        return vim_snprintf_safelen(
            buf,
            buflen as size_t,
            gettext(c"%3s".as_ptr()),
            &raw mut tmp as *mut ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn append_arg_number(
    mut wp: *mut win_T,
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        if (*(*curwin.get()).w_alist).al_ga.ga_len <= 1 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let mut msg_0: *const ::core::ffi::c_char = if (*wp).w_arg_idx_invalid != 0 {
            gettext(c" ((%d) of %d)".as_ptr())
        } else {
            gettext(c" (%d of %d)".as_ptr())
        };
        return vim_snprintf_safelen(
            buf,
            buflen,
            msg_0,
            (*wp).w_arg_idx + 1 as ::core::ffi::c_int,
            (*(*curwin.get()).w_alist).al_ga.ga_len,
        ) as ::core::ffi::c_int;
    }
}
