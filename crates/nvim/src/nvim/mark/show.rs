use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::buffer::{bt_prompt, buflist_findnr, buflist_nr2name};
use crate::src::nvim::charset::{ptr2cells, skipwhite};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    Columns, IObuff, curbuf, curwin, e_argreq, e_invarg, e_invarg2, got_int, namedfm,
};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memline::ml_get;
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::{
    emsg, message_filtered, msg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts_title, semsg,
};
use crate::src::nvim::os::libc::{gettext, snprintf};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::pos::lt;
use crate::src::nvim::strings::{vim_strchr, xstrnsave};
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use super::*;

/// print the marks
pub unsafe fn ex_marks(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    let mut name: *mut c_char = ptr::null_mut();
    let mut posp: *mut pos_T = ptr::null_mut();
    if !arg.is_null() && *arg as c_int == NUL {
        arg = ptr::null_mut();
    }
    msg_ext_set_kind(c"list_cmd".as_ptr());
    show_one_mark(
        '\'' as c_int,
        arg,
        &raw mut (*curwin.get()).w_pcmark,
        ptr::null_mut(),
        true_0,
    );
    let mut i: c_int = 0;
    while i < NMARKS {
        show_one_mark(
            i + 'a' as c_int,
            arg,
            &raw mut (*(&raw mut (*curbuf.get()).b_namedm as *mut fmark_T).offset(i as isize)).mark,
            ptr::null_mut(),
            true_0,
        );
        i += 1;
    }
    let mut i_0: c_int = 0;
    while i_0 < NGLOBALMARKS {
        if (*namedfm.ptr())[i_0 as usize].fmark.fnum != 0 {
            name = fm_getname(
                &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(i_0 as isize)).fmark,
                15,
            );
        } else {
            name = (*namedfm.ptr())[i_0 as usize].fname;
        }
        if !name.is_null() {
            show_one_mark(
                if i_0 >= NMARKS {
                    i_0 - NMARKS + '0' as c_int
                } else {
                    i_0 + 'A' as c_int
                },
                arg,
                &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(i_0 as isize))
                    .fmark
                    .mark,
                name,
                ((*namedfm.ptr())[i_0 as usize].fmark.fnum == (*curbuf.get()).handle) as c_int,
            );
            if (*namedfm.ptr())[i_0 as usize].fmark.fnum != 0 {
                xfree(name as *mut c_void);
            }
        }
        i_0 += 1;
    }
    show_one_mark(
        '"' as c_int,
        arg,
        &raw mut (*curbuf.get()).b_last_cursor.mark,
        ptr::null_mut(),
        true_0,
    );
    show_one_mark(
        '[' as c_int,
        arg,
        &raw mut (*curbuf.get()).b_op_start,
        ptr::null_mut(),
        true_0,
    );
    show_one_mark(
        ']' as c_int,
        arg,
        &raw mut (*curbuf.get()).b_op_end,
        ptr::null_mut(),
        true_0,
    );
    show_one_mark(
        '^' as c_int,
        arg,
        &raw mut (*curbuf.get()).b_last_insert.mark,
        ptr::null_mut(),
        true_0,
    );
    show_one_mark(
        '.' as c_int,
        arg,
        &raw mut (*curbuf.get()).b_last_change.mark,
        ptr::null_mut(),
        true_0,
    );
    if bt_prompt(curbuf.get()) {
        show_one_mark(
            ':' as c_int,
            arg,
            &raw mut (*curbuf.get()).b_prompt_start.mark,
            ptr::null_mut(),
            true_0,
        );
    }
    let mut startp: *mut pos_T = &raw mut (*curbuf.get()).b_visual.vi_start;
    let mut endp: *mut pos_T = &raw mut (*curbuf.get()).b_visual.vi_end;
    if (lt(*startp, *endp) || (*endp).lnum == 0) && (*startp).lnum != 0 {
        posp = startp;
    } else {
        posp = endp;
    }
    show_one_mark('<' as c_int, arg, posp, ptr::null_mut(), true_0);
    show_one_mark(
        '>' as c_int,
        arg,
        if posp == startp { endp } else { startp },
        ptr::null_mut(),
        true_0,
    );
    show_one_mark(-1, arg, ptr::null_mut(), ptr::null_mut(), false_0);
}

/// `current` — in current file
pub(super) unsafe extern "C" fn show_one_mark(
    mut c: c_int,
    mut arg: *mut c_char,
    mut p: *mut pos_T,
    mut name_arg: *mut c_char,
    mut current: c_int,
) {
    static did_title: GlobalCell<bool> = GlobalCell::new(false);
    let mut mustfree: bool = false;
    let mut name: *mut c_char = name_arg;
    if c == -1 {
        if did_title.get() {
            did_title.set(false);
        } else if arg.is_null() {
            msg(gettext(c"No marks set".as_ptr()), 0);
        } else {
            semsg(gettext(c"E283: No marks matching \"%s\"".as_ptr()), arg);
        }
    } else if !got_int.get() && (arg.is_null() || !vim_strchr(arg, c).is_null()) && (*p).lnum != 0 {
        if name.is_null() && current != 0 {
            name = mark_line(p, 15);
            mustfree = true;
        }
        if !message_filtered(name) {
            if !did_title.get() {
                msg_puts_title(gettext(c"\nmark line  col file/text".as_ptr()));
                did_title.set(true);
            }
            msg_putchar('\n' as c_int);
            if !got_int.get() {
                snprintf(
                    IObuff.ptr() as *mut c_char,
                    IOSIZE as size_t,
                    c" %c %6d %4d ".as_ptr(),
                    c,
                    (*p).lnum,
                    (*p).col,
                );
                msg_outtrans(IObuff.ptr() as *mut c_char, 0, false);
                if !name.is_null() {
                    msg_outtrans(name, if current != 0 { HLF_D as c_int } else { 0 }, false);
                }
            }
        }
        if mustfree {
            xfree(name as *mut c_void);
        }
    }
}

/// ":delmarks[!] [marks]"
pub unsafe fn ex_delmarks(mut eap: *mut exarg_T) {
    let mut from: c_int = 0;
    let mut to: c_int = 0;
    let mut n: c_int = 0;
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    if *(*eap).arg as c_int == NUL && (*eap).forceit != 0 {
        let mut i: size_t = 0;
        while i < NMARKS as size_t {
            if (*curbuf.get()).b_namedm[i as usize].mark.lnum != 0 {
                do_markset_autocmd(
                    i.wrapping_add('a' as size_t) as c_char,
                    &raw mut pos,
                    curbuf.get(),
                );
            }
            i = i.wrapping_add(1);
        }
        if (*curbuf.get()).b_last_cursor.mark.lnum != 0 {
            do_markset_autocmd('"' as c_char, &raw mut pos, curbuf.get());
        }
        if (*curbuf.get()).b_last_insert.mark.lnum != 0 {
            do_markset_autocmd('^' as c_char, &raw mut pos, curbuf.get());
        }
        if (*curbuf.get()).b_last_change.mark.lnum != 0 {
            do_markset_autocmd('.' as c_char, &raw mut pos, curbuf.get());
        }
        if (*curbuf.get()).b_op_start.lnum != 0 {
            do_markset_autocmd('[' as c_char, &raw mut pos, curbuf.get());
        }
        if (*curbuf.get()).b_op_end.lnum != 0 {
            do_markset_autocmd(']' as c_char, &raw mut pos, curbuf.get());
        }
        clrallmarks(curbuf.get(), os_time());
    } else if (*eap).forceit != 0 {
        emsg(gettext(&raw const e_invarg as *const c_char));
    } else if *(*eap).arg as c_int == NUL {
        emsg(gettext(&raw const e_argreq as *const c_char));
    } else {
        let timestamp: Timestamp = os_time();
        let mut p: *mut c_char = (*eap).arg;
        while *p as c_int != NUL {
            let mut lower: bool = *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint;
            let mut digit: bool = ascii_isdigit(*p as c_int);
            if lower || digit || *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint {
                if *p.offset(1) as c_int == '-' as c_int {
                    from = *p as uint8_t as c_int;
                    to = *p.offset(2) as uint8_t as c_int;
                    if (if lower {
                        (*p.offset(2) as c_uint >= 'a' as c_uint
                            && *p.offset(2) as c_uint <= 'z' as c_uint)
                            as c_int
                    } else {
                        if digit {
                            ascii_isdigit(*p.offset(2) as c_int) as c_int
                        } else {
                            (*p.offset(2) as c_uint >= 'A' as c_uint
                                && *p.offset(2) as c_uint <= 'Z' as c_uint)
                                as c_int
                        }
                    }) == 0
                        || to < from
                    {
                        semsg(gettext(&raw const e_invarg2 as *const c_char), p);
                        return;
                    }
                    p = p.offset(2);
                } else {
                    to = *p as uint8_t as c_int;
                    from = to;
                }
                let mut i_0: c_int = from;
                while i_0 <= to {
                    if lower {
                        if (*curbuf.get()).b_namedm[(i_0 - 'a' as c_int) as usize]
                            .mark
                            .lnum
                            != 0
                        {
                            do_markset_autocmd(i_0 as c_char, &raw mut pos, curbuf.get());
                        }
                        (*curbuf.get()).b_namedm[(i_0 - 'a' as c_int) as usize]
                            .mark
                            .lnum = 0;
                        (*curbuf.get()).b_namedm[(i_0 - 'a' as c_int) as usize].timestamp =
                            timestamp;
                    } else {
                        if digit {
                            n = i_0 - '0' as c_int + NMARKS;
                        } else {
                            n = i_0 - 'A' as c_int;
                        }
                        if (*namedfm.ptr())[n as usize].fmark.mark.lnum != 0 {
                            let mut buf: *mut buf_T =
                                buflist_findnr((*namedfm.ptr())[n as usize].fmark.fnum);
                            if buf.is_null() {
                                buf = curbuf.get();
                            }
                            do_markset_autocmd(i_0 as c_char, &raw mut pos, buf);
                        }
                        (*namedfm.ptr())[n as usize].fmark.mark.lnum = 0;
                        (*namedfm.ptr())[n as usize].fmark.fnum = 0;
                        (*namedfm.ptr())[n as usize].fmark.timestamp = timestamp;
                        let mut ptr_: *mut *mut c_void =
                            &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(n as isize)).fname
                                as *mut *mut c_void;
                        xfree(*ptr_);
                        *ptr_ = ptr::null_mut();
                        let _ = *ptr_;
                    }
                    i_0 += 1;
                }
            } else {
                match *p as c_int {
                    34 => {
                        if (*curbuf.get()).b_last_cursor.mark.lnum != 0 {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        clear_fmark(&raw mut (*curbuf.get()).b_last_cursor, timestamp);
                    }
                    94 => {
                        if (*curbuf.get()).b_last_insert.mark.lnum != 0 {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        clear_fmark(&raw mut (*curbuf.get()).b_last_insert, timestamp);
                    }
                    46 => {
                        if (*curbuf.get()).b_last_change.mark.lnum != 0 {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        clear_fmark(&raw mut (*curbuf.get()).b_last_change, timestamp);
                    }
                    91 => {
                        if (*curbuf.get()).b_op_start.lnum != 0 {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        (*curbuf.get()).b_op_start.lnum = 0;
                    }
                    93 => {
                        if (*curbuf.get()).b_op_end.lnum != 0 {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        (*curbuf.get()).b_op_end.lnum = 0;
                    }
                    60 => {
                        if (*curbuf.get()).b_visual.vi_start.lnum != 0 {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        (*curbuf.get()).b_visual.vi_start.lnum = 0;
                    }
                    62 => {
                        if (*curbuf.get()).b_visual.vi_end.lnum != 0 {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        (*curbuf.get()).b_visual.vi_end.lnum = 0;
                    }
                    58 | 32 => {}
                    _ => {
                        semsg(gettext(&raw const e_invarg2 as *const c_char), p);
                        return;
                    }
                }
            }
            p = p.offset(1);
        }
    };
}

/// Return the line at mark "mp".  Truncate to fit in window.
/// The returned string has been allocated.
pub(super) unsafe extern "C" fn mark_line(mut mp: *mut pos_T, mut lead_len: c_int) -> *mut c_char {
    let mut p: *mut c_char = ptr::null_mut();
    if (*mp).lnum == 0 || (*mp).lnum > (*curbuf.get()).b_ml.ml_line_count {
        return xstrdup(c"-invalid-".as_ptr());
    }
    assert!(Columns.get() >= 0, "Columns >= 0");
    let mut s: *mut c_char = xstrnsave(
        skipwhite(ml_get((*mp).lnum)),
        (Columns.get() as size_t).wrapping_mul(5),
    );
    let mut len: c_int = 0;
    p = s;
    while *p as c_int != NUL {
        len += ptr2cells(p);
        if len >= Columns.get() - lead_len {
            break;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    *p = NUL as c_char;
    return s;
}

/// Get name of file from a filemark.
/// When it's in the current buffer, return the text at the mark.
/// Returns an allocated string.
pub unsafe extern "C" fn fm_getname(mut fmark: *mut fmark_T, mut lead_len: c_int) -> *mut c_char {
    if (*fmark).fnum == (*curbuf.get()).handle {
        return mark_line(&raw mut (*fmark).mark, lead_len);
    }
    return buflist_nr2name((*fmark).fnum, false_0, true_0);
}
