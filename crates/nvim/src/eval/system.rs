//! `system()` and `systemlist()`: the argument vector and the captured
//! output.
//!
//! Both directions swap NUL and newline. A shell's stdin is a byte stream
//! with no way to carry a NUL, so `save_tv_as_string` writes a newline for
//! every NUL a List item held and vice versa; `get_system_output_as_rettv`
//! undoes it on the way back. That is why the two halves look asymmetric:
//! one builds a buffer, the other rewrites one in place.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{semsg_c, smsg_c};
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr::{null, null_mut};

use crate::buffer::buflist_findnr;
use crate::eval::encode::encode_list_write;
use crate::eval::typval::{
    tv_get_number, tv_get_string, tv_get_string_chk, tv_list_alloc, tv_list_alloc_ret,
    tv_list_first, tv_list_len, tv_list_ref,
};
use crate::eval::vars::set_vim_var_nr;
use crate::eval::{NL, PROF_YES};
use crate::ex_cmds::check_secure;
use crate::main::{do_profiling, e_invarg, e_invarg2, e_invargNval, e_nobufnr, p_verbose};
use crate::memline::ml_get_buf;
use crate::memory::{memchrsub, xcalloc, xfree, xmalloc, xmemdupz, xstrdup};
use crate::message::{emsg, msg_puts, verbose_enter_scroll, verbose_leave_scroll};
use crate::os::cshim::{gettext, snprintf};
use crate::os::fs::os_can_exe;
use crate::os::shell::{os_system, shell_argv_to_str, shell_build_argv, shell_free_argv};
use crate::profile::{prof_child_enter, prof_child_exit};
use crate::types::{
    EvalFuncData, IOSIZE, NUL, OptInt, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
    VV_SHELL_ERROR, buf_T, kListLenMayKnow, list_T, listitem_T, proftime_T, ptrdiff_t, size_t,
    typval_T, varnumber_T,
};
use ::libc::strlen;

/// Build a `NULL`-terminated argument vector out of a String (through the
/// shell) or a List (directly). `cmd`, when given, comes back naming the
/// executable; `executable` is cleared when the first item is not one.
///
/// # Safety
/// `cmd_tv` must be valid; `cmd` and `executable` null or valid.
pub unsafe fn tv_to_argv(
    cmd_tv: *mut typval_T,
    cmd: *mut *const c_char,
    executable: *mut bool,
) -> *mut *mut c_char {
    unsafe {
        if (*cmd_tv).v_type == VAR_STRING {
            let cmd_str = tv_get_string(cmd_tv);
            if !cmd.is_null() {
                *cmd = cmd_str;
            }
            return shell_build_argv(cmd_str, null::<c_char>());
        }
        if (*cmd_tv).v_type != VAR_LIST {
            semsg_c!(
                gettext(e_invarg2.as_ptr()),
                c"expected String or List".as_ptr(),
            );
            return null_mut();
        }

        let argl: *mut list_T = (*cmd_tv).vval.v_list;
        let argc = tv_list_len(argl);
        if argc == 0 {
            emsg(gettext(e_invarg.as_ptr()));
            return null_mut();
        }

        // The first item has to resolve to something runnable, and the
        // resolved path is what actually goes in slot 0.
        let arg0 = tv_get_string_chk(&raw mut (*tv_list_first(argl)).li_tv);
        let mut exe_resolved: *mut c_char = null_mut();
        if arg0.is_null() || !os_can_exe(arg0, &raw mut exe_resolved, true) {
            if !arg0.is_null() && !executable.is_null() {
                let mut buf: [c_char; IOSIZE as usize] = [0; IOSIZE as usize];
                snprintf(
                    buf.as_mut_ptr(),
                    size_of::<[c_char; IOSIZE as usize]>(),
                    c"'%s' is not executable".as_ptr(),
                    arg0,
                );
                semsg_c!(
                    gettext(e_invargNval.as_ptr()),
                    c"cmd".as_ptr(),
                    buf.as_mut_ptr(),
                );
                *executable = false;
            }
            return null_mut();
        }
        if !cmd.is_null() {
            *cmd = exe_resolved;
        }

        let argv = xcalloc(argc as size_t + 1, size_of::<*mut c_char>()) as *mut *mut c_char;
        let mut i = 0;
        if !argl.is_null() {
            let mut arg: *const listitem_T = (*argl).lv_first;
            while !arg.is_null() {
                let a = tv_get_string_chk(&raw const (*arg).li_tv);
                if a.is_null() {
                    shell_free_argv(argv);
                    xfree(exe_resolved as *mut c_void);
                    return null_mut();
                }
                *argv.offset(i) = xstrdup(a);
                i += 1;
                arg = (*arg).li_next;
            }
        }
        // Slot 0 holds the item's own spelling; swap in the resolved path.
        xfree(*argv as *mut c_void);
        *argv = exe_resolved;
        argv
    }
}

/// Split captured output into a List of lines, undoing the NUL/newline
/// swap on the way.
///
/// # Safety
/// `str` must hold `len` readable bytes.
pub(crate) unsafe fn string_to_list(
    str: *const c_char,
    mut len: size_t,
    keepempty: bool,
) -> *mut list_T {
    unsafe {
        // A trailing newline does not start an empty last line unless the
        // caller asked to keep one.
        if !keepempty && *str.add(len - 1) as c_int == NL {
            len -= 1;
        }
        let list = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
        encode_list_write(list as *mut c_void, str, len);
        list
    }
}

/// The shared body of `system()` and `systemlist()`.
///
/// # Safety
/// `argvars` must hold the builtin's arguments; `rettv` must be valid.
pub(crate) unsafe fn get_system_output_as_rettv(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    retlist: bool,
) {
    unsafe {
        let profiling = do_profiling.get() == PROF_YES;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = null_mut();
        if check_secure() {
            return;
        }

        let mut input_len: ptrdiff_t = 0;
        let input = save_tv_as_string(argvars.add(1), &raw mut input_len, false, false);
        if input_len < 0 {
            debug_assert!(input.is_null());
            return;
        }

        let mut executable = true;
        let argv = tv_to_argv(argvars, null_mut(), &raw mut executable);
        if argv.is_null() {
            // A command that does not exist reports -1 rather than a shell
            // exit status.
            if !executable {
                set_vim_var_nr(VV_SHELL_ERROR, -1);
            }
            xfree(input as *mut c_void);
            return;
        }

        if p_verbose.get() > 3 as OptInt {
            let cmdstr = shell_argv_to_str(argv);
            verbose_enter_scroll();
            smsg_c!(0, gettext(c"Executing command: \"%s\"".as_ptr()), cmdstr);
            msg_puts(c"\n\n".as_ptr());
            verbose_leave_scroll();
            xfree(cmdstr as *mut c_void);
        }

        let mut wait_time: proftime_T = 0;
        if profiling {
            wait_time = prof_child_enter();
        }
        let mut nread: size_t = 0;
        let mut res: *mut c_char = null_mut();
        let status = os_system(
            argv,
            input,
            input_len as size_t,
            &raw mut res,
            &raw mut nread,
        );
        if profiling {
            prof_child_exit(wait_time);
        }
        xfree(input as *mut c_void);
        set_vim_var_nr(VV_SHELL_ERROR, status as varnumber_T);

        if res.is_null() {
            if retlist {
                tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
            } else {
                (*rettv).vval.v_string = xstrdup(c"".as_ptr());
            }
            return;
        }

        if retlist {
            // The `keepempty` argument is the third, so it is only read
            // when the second was given too.
            let mut keepempty = 0;
            if (*argvars.add(1)).v_type != VAR_UNKNOWN && (*argvars.add(2)).v_type != VAR_UNKNOWN {
                keepempty = tv_get_number(argvars.add(2)) as c_int;
            }
            (*rettv).vval.v_list = string_to_list(res, nread, keepempty != 0);
            tv_list_ref((*rettv).vval.v_list);
            (*rettv).v_type = VAR_LIST;
            xfree(res as *mut c_void);
        } else {
            // Undo the swap in place; the buffer is handed over as it is.
            memchrsub(res as *mut c_void, NUL as c_char, 1 as c_char, nread);
            (*rettv).vval.v_string = res;
        }
    }
}

/// `system()`
///
/// # Safety
/// Called through the builtin table.
pub unsafe fn f_system(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { get_system_output_as_rettv(argvars, rettv, false) }
}

/// `systemlist()`
///
/// # Safety
/// Called through the builtin table.
pub unsafe fn f_systemlist(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { get_system_output_as_rettv(argvars, rettv, true) }
}

/// Render a typval as the byte stream a child process's stdin wants: a
/// String as it is, a Number as that buffer's whole text, a List one item
/// per line. `len` comes back -1 for a coercion that failed.
///
/// Newlines in the text become NULs and the line separators are newlines,
/// which is the convention the reading half undoes.
///
/// # Safety
/// `tv` and `len` must be valid.
pub unsafe fn save_tv_as_string(
    tv: *mut typval_T,
    len: *mut ptrdiff_t,
    endnl: bool,
    crlf: bool,
) -> *mut c_char {
    unsafe {
        *len = 0;
        if (*tv).v_type == VAR_UNKNOWN {
            return null_mut();
        }
        if (*tv).v_type != VAR_LIST && (*tv).v_type != VAR_NUMBER {
            let ret = tv_get_string_chk(tv);
            if ret.is_null() {
                *len = -1;
                return null_mut();
            }
            *len = strlen(ret) as ptrdiff_t;
            return xmemdupz(ret as *const c_void, *len as size_t) as *mut c_char;
        }
        if (*tv).v_type == VAR_NUMBER {
            return buffer_as_string(tv, len);
        }
        list_as_string((*tv).vval.v_list, len, endnl, crlf)
    }
}

/// A Number names a buffer; its whole text is the input.
///
/// # Safety
/// `tv` must be a `VAR_NUMBER`; `len` valid.
unsafe fn buffer_as_string(tv: *mut typval_T, len: *mut ptrdiff_t) -> *mut c_char {
    unsafe {
        let buf: *mut buf_T = buflist_findnr((*tv).vval.v_number as c_int);
        if buf.is_null() {
            semsg_c!(gettext(e_nobufnr.as_ptr()), (*tv).vval.v_number);
            *len = -1;
            return null_mut();
        }

        // Measure first: every line's bytes plus its terminator. The walk
        // is `strlen` on purpose — upstream counts bytes up to the NUL,
        // not whatever the memline records as the line's length.
        for lnum in 1..=(*buf).b_ml.ml_line_count {
            *len += strlen(ml_get_buf(buf, lnum)) as ptrdiff_t + 1;
        }
        if *len == 0 {
            return null_mut();
        }

        let ret = xmalloc(*len as size_t + 1) as *mut c_char;
        let mut end = ret;
        for lnum in 1..=(*buf).b_ml.ml_line_count {
            let mut p = ml_get_buf(buf, lnum);
            while *p as c_int != NUL {
                *end = if *p == b'\n' as c_char {
                    NUL as c_char
                } else {
                    *p
                };
                end = end.add(1);
                p = p.add(1);
            }
            *end = b'\n' as c_char;
            end = end.add(1);
        }
        *end = NUL as c_char;
        *len = end.offset_from(ret) as ptrdiff_t;
        ret
    }
}

/// A List is one line per item.
///
/// # Safety
/// `list` must be null or valid; `len` valid.
unsafe fn list_as_string(
    list: *mut list_T,
    len: *mut ptrdiff_t,
    endnl: bool,
    crlf: bool,
) -> *mut c_char {
    unsafe {
        let sep = if crlf { 2 } else { 1 };

        // Measure first, charging every item a separator.
        if !list.is_null() {
            let mut li: *const listitem_T = (*list).lv_first;
            while !li.is_null() {
                *len += strlen(tv_get_string(&raw const (*li).li_tv)) as ptrdiff_t + sep;
                li = (*li).li_next;
            }
        }
        if *len == 0 {
            return null_mut();
        }

        // The last item's separator is only written when `endnl`, so the
        // measured length already covers the terminator when it is not.
        let ret = xmalloc((*len + if endnl { sep } else { 0 }) as size_t) as *mut c_char;
        let mut end = ret;
        if !list.is_null() {
            let mut li: *const listitem_T = (*list).lv_first;
            while !li.is_null() {
                let mut s = tv_get_string(&raw const (*li).li_tv);
                while *s as c_int != NUL {
                    *end = if *s == b'\n' as c_char {
                        NUL as c_char
                    } else {
                        *s
                    };
                    end = end.add(1);
                    s = s.add(1);
                }
                if endnl || !(*li).li_next.is_null() {
                    if crlf {
                        *end = b'\r' as c_char;
                        end = end.add(1);
                    }
                    *end = b'\n' as c_char;
                    end = end.add(1);
                }
                li = (*li).li_next;
            }
        }
        *end = NUL as c_char;
        *len = end.offset_from(ret) as ptrdiff_t;
        ret
    }
}
