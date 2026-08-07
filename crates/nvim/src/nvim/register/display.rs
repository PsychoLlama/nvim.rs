//! `:registers` and `:display`.
//!
//! One command, and `dis_msg`, the escaping every register's preview goes
//! through -- a control character is shown as `^X`, and `skip_esc` drops the
//! trailing `<Esc>` an Insert-mode recording always ends with so that the
//! common case reads as the text that was typed.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn dis_msg(mut p: *const ::core::ffi::c_char, mut skip_esc: bool) {
    unsafe {
        let mut n: ::core::ffi::c_int = Columns.get() - 6 as ::core::ffi::c_int;
        while *p as ::core::ffi::c_int != NUL
            && !(*p as ::core::ffi::c_int == ESC
                && skip_esc as ::core::ffi::c_int != 0
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
            && {
                n -= ptr2cells(p);
                n >= 0 as ::core::ffi::c_int
            }
        {
            let mut l: ::core::ffi::c_int = 0;
            l = utfc_ptr2len(p);
            if l > 1 as ::core::ffi::c_int {
                msg_outtrans_len(p, l, 0 as ::core::ffi::c_int, false_0 != 0);
                p = p.offset(l as isize);
            } else {
                let c2rust_fresh4 = p;
                p = p.offset(1);
                msg_outtrans_len(
                    c2rust_fresh4,
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            }
        }
        os_breakcheck();
    }
}

pub unsafe fn ex_display(mut eap: *mut exarg_T) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut yb: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut type_0: ::core::ffi::c_int = 0;
        if !arg.is_null() && *arg as ::core::ffi::c_int == NUL {
            arg = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut hl_id: ::core::ffi::c_int = HLF_8;
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        msg_ext_skip_flush.set(true_0 != 0);
        msg_puts_title(gettext(
            b"\nType Name Content\0".as_ptr() as *const ::core::ffi::c_char
        ));
        let mut i: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        while i < NUM_REGISTERS as ::core::ffi::c_int && !got_int.get() {
            let mut name: ::core::ffi::c_int = get_register_name(i);
            if !(!arg.is_null() && vim_strchr(arg, name).is_null()) {
                match get_reg_type(name, ::core::ptr::null_mut::<colnr_T>()) as ::core::ffi::c_int {
                    1 => {
                        type_0 = 'l' as ::core::ffi::c_int;
                    }
                    0 => {
                        type_0 = 'c' as ::core::ffi::c_int;
                    }
                    _ => {
                        type_0 = 'b' as ::core::ffi::c_int;
                    }
                }
                if i == -1 as ::core::ffi::c_int {
                    if !(*y_previous.ptr()).is_null() {
                        yb = y_previous.get();
                    } else {
                        yb = (y_regs.ptr() as *mut yankreg_T)
                            .offset(0 as ::core::ffi::c_int as isize);
                    }
                } else {
                    yb = (y_regs.ptr() as *mut yankreg_T).offset(i as isize);
                }
                clipboard::get_clipboard(name, &mut yb, true);
                if !(name == mb_tolower(redir_reg.get())
                    || redir_reg.get() == '"' as ::core::ffi::c_int && yb == y_previous.get())
                {
                    if !(*yb).y_array.is_null() {
                        let mut do_show: bool = false_0 != 0;
                        let mut j: size_t = 0 as size_t;
                        while !do_show && j < (*yb).y_size {
                            do_show = !message_filtered((*(*yb).y_array.offset(j as isize)).data);
                            j = j.wrapping_add(1);
                        }
                        if do_show as ::core::ffi::c_int != 0 || (*yb).y_size == 0 as size_t {
                            msg_putchar('\n' as ::core::ffi::c_int);
                            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                            msg_putchar(type_0);
                            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                            msg_putchar('"' as ::core::ffi::c_int);
                            msg_putchar(name);
                            msg_puts(b"   \0".as_ptr() as *const ::core::ffi::c_char);
                            let mut n: ::core::ffi::c_int =
                                Columns.get() - 11 as ::core::ffi::c_int;
                            let mut j_0: size_t = 0 as size_t;
                            while j_0 < (*yb).y_size && n > 1 as ::core::ffi::c_int {
                                if j_0 != 0 {
                                    msg_puts_hl(
                                        b"^J\0".as_ptr() as *const ::core::ffi::c_char,
                                        hl_id,
                                        false_0 != 0,
                                    );
                                    n -= 2 as ::core::ffi::c_int;
                                }
                                p = (*(*yb).y_array.offset(j_0 as isize)).data;
                                while *p as ::core::ffi::c_int != NUL && {
                                    n -= ptr2cells(p);
                                    n >= 0 as ::core::ffi::c_int
                                } {
                                    let mut clen: ::core::ffi::c_int = utfc_ptr2len(p);
                                    msg_outtrans_len(
                                        p,
                                        clen,
                                        0 as ::core::ffi::c_int,
                                        false_0 != 0,
                                    );
                                    p = p.offset((clen - 1 as ::core::ffi::c_int) as isize);
                                    p = p.offset(1);
                                }
                                j_0 = j_0.wrapping_add(1);
                            }
                            if n > 1 as ::core::ffi::c_int
                                && (*yb).y_type as ::core::ffi::c_int
                                    == kMTLineWise as ::core::ffi::c_int
                            {
                                msg_puts_hl(
                                    b"^J\0".as_ptr() as *const ::core::ffi::c_char,
                                    hl_id,
                                    false_0 != 0,
                                );
                            }
                        }
                        os_breakcheck();
                    }
                }
            }
            i += 1;
        }
        let mut insert: String_0 = get_last_insert();
        p = insert.data;
        if !p.is_null()
            && (arg.is_null() || !vim_strchr(arg, '.' as ::core::ffi::c_int).is_null())
            && !got_int.get()
            && !message_filtered(p)
        {
            msg_puts(b"\n  c  \".   \0".as_ptr() as *const ::core::ffi::c_char);
            dis_msg(p, true_0 != 0);
        }
        if !(*last_cmdline.ptr()).is_null()
            && (arg.is_null() || !vim_strchr(arg, ':' as ::core::ffi::c_int).is_null())
            && !got_int.get()
            && !message_filtered(last_cmdline.get())
        {
            msg_puts(b"\n  c  \":   \0".as_ptr() as *const ::core::ffi::c_char);
            dis_msg(last_cmdline.get(), false_0 != 0);
        }
        if !(*curbuf.get()).b_fname.is_null()
            && (arg.is_null() || !vim_strchr(arg, '%' as ::core::ffi::c_int).is_null())
            && !got_int.get()
            && !message_filtered((*curbuf.get()).b_fname)
        {
            msg_puts(b"\n  c  \"%   \0".as_ptr() as *const ::core::ffi::c_char);
            dis_msg((*curbuf.get()).b_fname, false_0 != 0);
        }
        if (arg.is_null() || !vim_strchr(arg, '%' as ::core::ffi::c_int).is_null())
            && !got_int.get()
        {
            let mut fname: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut dummy: linenr_T = 0;
            if buflist_name_nr(0 as ::core::ffi::c_int, &raw mut fname, &raw mut dummy) != FAIL
                && !message_filtered(fname)
            {
                msg_puts(b"\n  c  \"#   \0".as_ptr() as *const ::core::ffi::c_char);
                dis_msg(fname, false_0 != 0);
            }
        }
        if !last_search_pat().is_null()
            && (arg.is_null() || !vim_strchr(arg, '/' as ::core::ffi::c_int).is_null())
            && !got_int.get()
            && !message_filtered(last_search_pat())
        {
            msg_puts(b"\n  c  \"/   \0".as_ptr() as *const ::core::ffi::c_char);
            dis_msg(last_search_pat(), false_0 != 0);
        }
        if !(*expr_line.ptr()).is_null()
            && (arg.is_null() || !vim_strchr(arg, '=' as ::core::ffi::c_int).is_null())
            && !got_int.get()
            && !message_filtered(expr_line.get())
        {
            msg_puts(b"\n  c  \"=   \0".as_ptr() as *const ::core::ffi::c_char);
            dis_msg(expr_line.get(), false_0 != 0);
        }
        msg_ext_skip_flush.set(false_0 != 0);
    }
}
