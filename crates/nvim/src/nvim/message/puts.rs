//! `msg_puts` and its display half: text onto the message grid.
//!
//! [`msg_puts_len`] is the funnel every message eventually reaches; it feeds
//! the redirection sinks and then [`msg_puts_display`], which lays the text
//! out cell by cell, scrolls when it runs off the bottom and raises the pager
//! when `'more'` says to. [`msg_puts_printf`] is the same job for a process
//! with no UI at all.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn msg_start() {
    unsafe {
        let mut did_return: bool = false_0 != 0;
        msg_row.set(if msg_row.get() > cmdline_row.get() {
            msg_row.get()
        } else {
            cmdline_row.get()
        });
        if msg_silent.get() == 0 {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                keep_msg.ptr() as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            need_fileinfo.set(false_0 != 0);
        }
        if need_highlight_changed.get() {
            highlight_changed();
        }
        if need_clr_eos.get() as ::core::ffi::c_int != 0
            || p_ch.get() == 0 as OptInt && redrawing_cmdline.get() as ::core::ffi::c_int != 0
        {
            need_clr_eos.set(false_0 != 0);
            msg_clr_eos();
        }
        if p_ch.get() == 0 as OptInt && !ui_has(kUIMessages) && msg_scrolled.get() == 0 {
            msg_grid_validate();
            msg_scroll_up(false_0 != 0, true_0 != 0);
            (*msg_scrolled.ptr()) += 1;
            cmdline_row.set(Rows.get() - 1 as ::core::ffi::c_int);
        }
        if msg_scroll.get() == 0 && full_screen.get() as ::core::ffi::c_int != 0 {
            msg_row.set(cmdline_row.get());
            msg_col.set(0 as ::core::ffi::c_int);
        } else if (msg_didout.get() as ::core::ffi::c_int != 0 || p_ch.get() == 0 as OptInt)
            && !ui_has(kUIMessages)
        {
            if p_ch.get() == 0 as OptInt && !msg_didout.get() && msg_use_printf() != 0 {
                msg_puts_display(
                    b"\n\0".as_ptr() as *const ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    false_0,
                );
            } else {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            did_return = true_0 != 0;
            cmdline_row.set(msg_row.get());
        }
        if !msg_didany.get() || lines_left.get() < 0 as ::core::ffi::c_int {
            msg_starthere();
        }
        if msg_silent.get() == 0 as ::core::ffi::c_int {
            msg_didout.set(false_0 != 0);
        }
        if ui_has(kUIMessages) {
            msg_ext_ui_flush();
        }
        if !did_return {
            redir_write(
                b"\n\0".as_ptr() as *const ::core::ffi::c_char,
                1 as ptrdiff_t,
            );
        }
    }
}

pub unsafe extern "C" fn msg_starthere() {
    lines_left.set(cmdline_row.get());
    msg_didany.set(false_0 != 0);
}

pub unsafe extern "C" fn msg_puts(mut s: *const ::core::ffi::c_char) {
    unsafe {
        msg_puts_hl(s, 0 as ::core::ffi::c_int, false_0 != 0);
    }
}

pub unsafe extern "C" fn msg_puts_title(mut s: *const ::core::ffi::c_char) {
    unsafe {
        s = s.offset(
            (ui_has(kUIMessages) as ::core::ffi::c_int != 0
                && *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int)
                as ::core::ffi::c_int as isize,
        );
        msg_puts_hl(s, HLF_T, false_0 != 0);
    }
}

pub unsafe extern "C" fn msg_puts_hl(
    s: *const ::core::ffi::c_char,
    hl_id: ::core::ffi::c_int,
    hist: bool,
) {
    unsafe {
        msg_puts_len(s, -1 as ptrdiff_t, hl_id, hist);
    }
}

pub unsafe extern "C" fn msg_puts_len(
    str: *const ::core::ffi::c_char,
    len: ptrdiff_t,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) {
    unsafe {
        '_c2rust_label: {
            if len < 0 as ptrdiff_t
                || memchr(
                    str as *const ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    len as size_t,
                )
                .is_null()
            {
            } else {
                __assert_fail(
                    b"len < 0 || memchr(str, 0, (size_t)len) == NULL\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2367 as ::core::ffi::c_uint,
                    b"void msg_puts_len(const char *const, const ptrdiff_t, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        redir_write(str, len);
        if msg_silent.get() != 0 as ::core::ffi::c_int || *str as ::core::ffi::c_int == NUL {
            if *str as ::core::ffi::c_int == NUL && ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
                msg_ext_ui_flush();
                ui_call_msg_show(
                    cstr_as_string(b"empty\0".as_ptr() as *const ::core::ffi::c_char),
                    Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    },
                    false_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                    object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed_11 {
                            integer: -1 as Integer,
                        },
                    },
                    String_0 {
                        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        size: 0 as size_t,
                    },
                );
                cmdline_was_last_drawn.set(false_0 != 0);
            }
            return;
        }
        if hist {
            msg_hist_add(str, len as ::core::ffi::c_int, hl_id);
        }
        let mut overflow: bool = !ui_has(kUIMessages)
            && msg_scrolled.get()
                > (if p_ch.get() == 0 as OptInt {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                });
        if overflow as ::core::ffi::c_int != 0
            && !msg_scrolled_ign.get()
            && strcmp(str, b"\r\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as ::core::ffi::c_int
        {
            need_wait_return.set(true_0 != 0);
        }
        msg_didany.set(true_0 != 0);
        if msg_use_printf() != 0 {
            let mut saved_msg_col: ::core::ffi::c_int = msg_col.get();
            msg_puts_printf(str, len);
            if headless_mode.get() {
                msg_col.set(saved_msg_col);
            }
        }
        if msg_use_printf() == 0
            || headless_mode.get() as ::core::ffi::c_int != 0
                && !(*default_grid.ptr()).chars.is_null()
        {
            msg_puts_display(str, len as ::core::ffi::c_int, hl_id, false_0);
        }
        need_fileinfo.set(false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn msg_puts_display(
    mut str: *const ::core::ffi::c_char,
    mut maxlen: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut recurse: ::core::ffi::c_int,
) {
    unsafe {
        let mut s: *const ::core::ffi::c_char = str;
        let mut sb_str: *const ::core::ffi::c_char = str;
        let mut sb_col: ::core::ffi::c_int = msg_col.get();
        let mut attr: ::core::ffi::c_int = if hl_id != 0 {
            syn_id2attr(hl_id)
        } else {
            0 as ::core::ffi::c_int
        };
        did_wait_return.set(false_0 != 0);
        if ui_has(kUIMessages) {
            if attr as sattr_T != msg_ext_last_attr.get() {
                msg_ext_emit_chunk();
                msg_ext_last_attr.set(attr as sattr_T);
                msg_ext_last_hl_id.set(hl_id);
            }
            let mut len: size_t = if maxlen < 0 as ::core::ffi::c_int {
                strlen(str)
            } else {
                strnlen(str, maxlen as size_t)
            };
            ga_concat_len(msg_ext_last_chunk.ptr(), str, len);
            let mut lastline: *const ::core::ffi::c_char =
                xmemrchr(str as *const ::core::ffi::c_void, '\n' as uint8_t, len)
                    as *const ::core::ffi::c_char;
            maxlen -= (if !lastline.is_null() {
                lastline.offset_from(str)
            } else {
                0 as isize
            }) as ::core::ffi::c_int;
            let mut p: *const ::core::ffi::c_char = if !lastline.is_null() {
                lastline.offset(1 as ::core::ffi::c_int as isize)
            } else {
                str
            };
            let mut col: ::core::ffi::c_int = (if maxlen < 0 as ::core::ffi::c_int {
                mb_string2cells(p)
            } else {
                mb_string2cells_len(p, maxlen as size_t)
            }) as ::core::ffi::c_int;
            msg_col.set(
                (if !lastline.is_null() {
                    0 as ::core::ffi::c_int
                } else {
                    msg_col.get()
                }) + col,
            );
            return;
        }
        let mut print_attr: ::core::ffi::c_int =
            hl_combine_attr(*(*hl_attr_active.ptr()).offset(HLF_MSG as isize), attr);
        msg_grid_validate();
        cmdline_was_last_drawn.set(redrawing_cmdline.get());
        let mut msg_row_pending: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        loop {
            if msg_col.get() >= Columns.get() {
                if p_more.get() != 0 && recurse == 0 {
                    store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, true_0);
                }
                if msg_no_more.get() as ::core::ffi::c_int != 0
                    && lines_left.get() == 0 as ::core::ffi::c_int
                {
                    break;
                }
                msg_col.set(0 as ::core::ffi::c_int);
                (*msg_row.ptr()) += 1;
                msg_didout.set(false_0 != 0);
            }
            if msg_row.get() >= Rows.get() {
                msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
                if msg_no_more.get() as ::core::ffi::c_int != 0
                    && lines_left.get() == 0 as ::core::ffi::c_int
                {
                    break;
                }
                if recurse == 0 {
                    if msg_row_pending >= 0 as ::core::ffi::c_int {
                        msg_line_flush();
                        msg_row_pending = -1 as ::core::ffi::c_int;
                    }
                    msg_scroll_up(true_0 != 0, false_0 != 0);
                    inc_msg_scrolled();
                    need_wait_return.set(true_0 != 0);
                    redraw_cmdline.set(true_0 != 0);
                    if cmdline_row.get() > 0 as ::core::ffi::c_int && !exmode_active.get() {
                        (*cmdline_row.ptr()) -= 1;
                    }
                    if lines_left.get() > 0 as ::core::ffi::c_int {
                        (*lines_left.ptr()) -= 1;
                    }
                    if p_more.get() != 0
                        && lines_left.get() == 0 as ::core::ffi::c_int
                        && State.get() != MODE_HITRETURN
                        && !msg_no_more.get()
                        && !exmode_active.get()
                    {
                        if do_more_prompt(NUL) {
                            s = confirm_buttons.get();
                        }
                        if quit_more.get() {
                            return;
                        }
                    }
                }
            }
            if !((maxlen < 0 as ::core::ffi::c_int
                || (s.offset_from(str) as ::core::ffi::c_int) < maxlen)
                && *s as ::core::ffi::c_int != NUL)
            {
                break;
            }
            if msg_row.get() != msg_row_pending
                && (*s as uint8_t as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
                    || *s as ::core::ffi::c_int == TAB)
            {
                if msg_row_pending >= 0 as ::core::ffi::c_int {
                    msg_line_flush();
                }
                grid_line_start(msg_grid_adj.ptr(), msg_row.get());
                msg_row_pending = msg_row.get();
            }
            if *s as uint8_t as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int {
                let mut cw: ::core::ffi::c_int = utf_ptr2cells(s);
                let mut l: ::core::ffi::c_int = if maxlen >= 0 as ::core::ffi::c_int {
                    utfc_ptr2len_len(
                        s,
                        str.offset(maxlen as isize).offset_from(s) as ::core::ffi::c_int,
                    )
                } else {
                    utfc_ptr2len(s)
                };
                if cw > 1 as ::core::ffi::c_int
                    && msg_col.get() == Columns.get() - 1 as ::core::ffi::c_int
                {
                    grid_line_puts(
                        msg_col.get(),
                        b">\0".as_ptr() as *const ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                        *(*hl_attr_active.ptr()).offset(HLF_AT as isize),
                    );
                    cw = 1 as ::core::ffi::c_int;
                } else {
                    grid_line_puts(msg_col.get(), s, l, print_attr);
                    s = s.offset(l as isize);
                }
                msg_didout.set(true_0 != 0);
                (*msg_col.ptr()) += cw;
            } else {
                let c2rust_fresh5 = s;
                s = s.offset(1);
                let mut c: ::core::ffi::c_char = *c2rust_fresh5;
                if c as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                    msg_didout.set(false_0 != 0);
                    msg_col.set(0 as ::core::ffi::c_int);
                    (*msg_row.ptr()) += 1;
                    if p_more.get() != 0 && recurse == 0 {
                        store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, true_0);
                    }
                } else if c as ::core::ffi::c_int == '\r' as ::core::ffi::c_int {
                    msg_col.set(0 as ::core::ffi::c_int);
                } else if c as ::core::ffi::c_int == '\u{8}' as ::core::ffi::c_int {
                    if msg_col.get() != 0 {
                        (*msg_col.ptr()) -= 1;
                    }
                } else if c as ::core::ffi::c_int == TAB {
                    loop {
                        grid_line_puts(
                            msg_col.get(),
                            b" \0".as_ptr() as *const ::core::ffi::c_char,
                            1 as ::core::ffi::c_int,
                            print_attr,
                        );
                        (*msg_col.ptr()) += 1 as ::core::ffi::c_int;
                        if msg_col.get() == Columns.get() {
                            break;
                        }
                        if msg_col.get() & 7 as ::core::ffi::c_int == 0 {
                            break;
                        }
                    }
                } else if c as ::core::ffi::c_int == BELL {
                    vim_beep(kOptBoFlagShell as ::core::ffi::c_int as ::core::ffi::c_uint);
                }
            }
        }
        if msg_row_pending >= 0 as ::core::ffi::c_int {
            msg_line_flush();
        }
        msg_cursor_goto(msg_row.get(), msg_col.get());
        if p_more.get() != 0 && recurse == 0 {
            store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, false_0);
        }
        msg_check();
    }
}

pub unsafe extern "C" fn message_filtered(mut msg_0: *const ::core::ffi::c_char) -> bool {
    unsafe {
        if (*cmdmod.ptr()).cmod_filter_regmatch.regprog.is_null() {
            return false_0 != 0;
        }
        let mut match_0: bool = vim_regexec(
            &raw mut (*cmdmod.ptr()).cmod_filter_regmatch,
            msg_0,
            0 as colnr_T,
        );
        return if (*cmdmod.ptr()).cmod_filter_force as ::core::ffi::c_int != 0 {
            match_0 as ::core::ffi::c_int
        } else {
            !match_0 as ::core::ffi::c_int
        } != 0;
    }
}

pub unsafe extern "C" fn msg_use_printf() -> ::core::ffi::c_int {
    return (!embedded_mode.get() && ui_active() == 0 && !ui_has(kUIMessages))
        as ::core::ffi::c_int;
}

pub(crate) unsafe extern "C" fn msg_puts_printf(
    mut str: *const ::core::ffi::c_char,
    maxlen: ptrdiff_t,
) {
    unsafe {
        let mut s: *const ::core::ffi::c_char = str;
        let mut buf: [::core::ffi::c_char; 7] = [0; 7];
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*on_print.ptr()).type_0 as ::core::ffi::c_uint
            != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut argv: [typval_T; 1] = [typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            }; 1];
            argv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
            argv[0 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
            argv[0 as ::core::ffi::c_int as usize].vval.v_string = str as *mut ::core::ffi::c_char;
            let mut rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            callback_call(
                on_print.ptr(),
                1 as ::core::ffi::c_int,
                &raw mut argv as *mut typval_T,
                &raw mut rettv,
            );
            tv_clear(&raw mut rettv);
            return;
        }
        while (maxlen < 0 as ptrdiff_t || s.offset_from(str) < maxlen)
            && *s as ::core::ffi::c_int != NUL
        {
            let mut len: ::core::ffi::c_int = utf_ptr2len(s);
            if !(silent_mode.get() as ::core::ffi::c_int != 0 && p_verbose.get() == 0 as OptInt) {
                p = (&raw mut buf as *mut ::core::ffi::c_char)
                    .offset(0 as ::core::ffi::c_int as isize);
                if *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                    && !info_message.get()
                    && !silent_mode.get()
                    && !headless_mode.get()
                {
                    let c2rust_fresh6 = p;
                    p = p.offset(1);
                    *c2rust_fresh6 = '\r' as ::core::ffi::c_char;
                }
                memcpy(
                    p as *mut ::core::ffi::c_void,
                    s as *const ::core::ffi::c_void,
                    len as size_t,
                );
                *p.offset(len as isize) = NUL as ::core::ffi::c_char;
                if info_message.get() {
                    printf(
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut buf as *mut ::core::ffi::c_char,
                    );
                } else {
                    fprintf(
                        stderr,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut buf as *mut ::core::ffi::c_char,
                    );
                }
            }
            let mut cw: ::core::ffi::c_int = utf_char2cells(utf_ptr2char(s));
            if *s as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
            {
                msg_col.set(0 as ::core::ffi::c_int);
                msg_didout.set(false_0 != 0);
            } else {
                (*msg_col.ptr()) += cw;
                msg_didout.set(true_0 != 0);
            }
            s = s.offset(len as isize);
        }
    }
}

pub unsafe extern "C" fn msg_end() -> bool {
    unsafe {
        if !exiting.get()
            && need_wait_return.get() as ::core::ffi::c_int != 0
            && State.get() & MODE_CMDLINE == 0
        {
            wait_return(false_0);
            return false_0 != 0;
        }
        msg_ext_ui_flush();
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn msg_check() {
    if ui_has(kUIMessages) {
        return;
    }
    if msg_row.get() == Rows.get() - 1 as ::core::ffi::c_int && msg_col.get() >= sc_col.get() {
        need_wait_return.set(true_0 != 0);
        redraw_cmdline.set(true_0 != 0);
    }
}

pub unsafe extern "C" fn msg_advance(mut col: ::core::ffi::c_int) {
    unsafe {
        if msg_silent.get() != 0 as ::core::ffi::c_int {
            msg_col.set(col);
            return;
        }
        col = if col < Columns.get() - 1 as ::core::ffi::c_int {
            col
        } else {
            Columns.get() - 1 as ::core::ffi::c_int
        };
        while msg_col.get() < col {
            msg_putchar(' ' as ::core::ffi::c_int);
        }
    }
}
