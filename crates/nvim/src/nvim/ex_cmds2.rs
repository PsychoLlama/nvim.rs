use crate::semsg_c;
use crate::src::nvim::arglist::{do_argfile, editing_arg_idx, ex_all, ex_rewind, set_arglist};
use crate::src::nvim::autocmd::{
    EVENT_SYNTAX, apply_autocmds, au_event_disable, au_event_restore, aucmd_prepbuf, aucmd_restbuf,
};
use crate::src::nvim::buffer::{
    bt_dontwrite, buf_hide, buf_set_name, buf_spname, buflist_findnr, bufref_valid, goto_buffer,
    no_write_message, no_write_message_nobang, set_bufref, set_curbuf,
};
use crate::src::nvim::bufwrite::{WriteRequest, buf_write};
use crate::src::nvim::change::unchanged;
use crate::src::nvim::channel::channel_job_running;
use crate::src::nvim::eval::eval_call_provider;
use crate::src::nvim::eval::typval::{
    tv_list_alloc, tv_list_append_allocated_string, tv_list_append_number, tv_list_append_string,
};
use crate::src::nvim::eval::vars::{
    do_unlet, get_var_value, set_internal_string_var, set_vim_var_string,
};
use crate::src::nvim::ex_cmds::{check_overwrite, set_swapcommand};
use crate::src::nvim::ex_docmd::{dialog_msg, do_cmdline, do_cmdline_cmd};
use crate::src::nvim::ex_getln::script_get;
use crate::src::nvim::fileio::{buf_check_timestamp, check_timestamps};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::HLF_W;
use crate::src::nvim::main::{
    cmdline_row, cmdmod, curbuf, curtab, curwin, e_noname, e_winfixbuf_cannot_go_to_buffer,
    emsg_off, exiting, first_tabpage, firstbuf, firstwin, got_int, listcmd_busy, msg_col,
    msg_didany, msg_didout, msg_listdo_overwrite, msg_row, no_check_timestamps, no_wait_return,
    p_aw, p_awa, p_confirm, p_write, prevwin, vgetc_busy,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup};
use crate::src::nvim::message::{
    emsg, msg, msg_source, vim_dialog_yesnoallcancel, vim_dialog_yesnocancel, wait_return,
};
use crate::src::nvim::r#move::validate_cursor;
use crate::src::nvim::normal::do_check_scrollbind;
use crate::src::nvim::os::libc::{gettext, snprintf, strlen};
use crate::src::nvim::path::vim_FullName;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::quickfix::{ex_cc, ex_cnext, qf_get_cur_idx, qf_get_valid_size};
use crate::src::nvim::runtime::{DIP_ALL, source_runtime_vim_lua};
use crate::src::nvim::search::FORWARD;
use crate::src::nvim::types::{
    CMD_append, CMD_argdo, CMD_bufdo, CMD_cdo, CMD_cfdo, CMD_first, CMD_ldo, CMD_lfdo, CMD_sfirst,
    CMD_tabdo, CMD_windo, CMOD_CONFIRM, VV_SWAPCOMMAND, aco_save_T, aentry_T, buf_T, bufref_T,
    cmd_addr_T, cstack_T, dobuf_action_values, dobuf_start_values, exarg, exarg_T, linenr_T,
    list_T, ptrdiff_t, size_t, ssize_t, tabpage_T, uint8_t, uint64_t, varnumber_T, win_T,
};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::window::{
    goto_tabpage_tp, goto_tabpage_win, valid_tabpage, win_goto, win_split, win_valid,
};
pub const ADDR_LINES: cmd_addr_T = 0;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const VIM_QUESTION: C2Rust_Unnamed_17 = 4;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const VIM_DISCARDALL: C2Rust_Unnamed_18 = 6;
pub const VIM_ALL: C2Rust_Unnamed_18 = 5;
pub const VIM_NO: C2Rust_Unnamed_18 = 3;
pub const VIM_YES: C2Rust_Unnamed_18 = 2;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const CCGD_EXCMD: C2Rust_Unnamed_19 = 16;
pub const CCGD_ALLBUF: C2Rust_Unnamed_19 = 8;
pub const CCGD_FORCEIT: C2Rust_Unnamed_19 = 4;
pub const CCGD_MULTWIN: C2Rust_Unnamed_19 = 2;
pub const CCGD_AW: C2Rust_Unnamed_19 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_20 = 2;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BF_SYN_SET: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static e_compiler_not_supported_str: GlobalCell<[::core::ffi::c_char; 33]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
            *b"E666: Compiler not supported: %s\0",
        )
    });
pub unsafe fn ex_ruby(mut eap: *mut exarg_T) {
    script_host_execute(
        b"ruby\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
pub unsafe fn ex_rubyfile(mut eap: *mut exarg_T) {
    script_host_execute_file(
        b"ruby\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
pub unsafe fn ex_rubydo(mut eap: *mut exarg_T) {
    script_host_do_range(
        b"ruby\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
pub unsafe fn ex_python3(mut eap: *mut exarg_T) {
    script_host_execute(
        b"python3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
pub unsafe fn ex_py3file(mut eap: *mut exarg_T) {
    script_host_execute_file(
        b"python3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
pub unsafe fn ex_pydo3(mut eap: *mut exarg_T) {
    script_host_do_range(
        b"python3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
pub unsafe fn ex_perl(mut eap: *mut exarg_T) {
    script_host_execute(
        b"perl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
pub unsafe fn ex_perlfile(mut eap: *mut exarg_T) {
    script_host_execute_file(
        b"perl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
pub unsafe fn ex_perldo(mut eap: *mut exarg_T) {
    script_host_do_range(
        b"perl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        eap,
    );
}
pub unsafe extern "C" fn autowrite(mut buf: *mut buf_T, mut forceit: bool) -> ::core::ffi::c_int {
    let mut bufref: bufref_T = bufref_T::default();
    if !(p_aw.get() != 0 || p_awa.get() != 0)
        || p_write.get() == 0
        || bt_dontwrite(buf) as ::core::ffi::c_int != 0
        || !forceit && (*buf).b_p_ro != 0
        || (*buf).b_ffname.is_null()
    {
        return FAIL;
    }
    set_bufref(&raw mut bufref, buf);
    let mut r: ::core::ffi::c_int = buf_write_all(buf, forceit);
    if bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
        && bufIsChanged(buf) as ::core::ffi::c_int != 0
    {
        r = FAIL;
    }
    return r;
}
pub unsafe extern "C" fn autowrite_all() {
    if !(p_aw.get() != 0 || p_awa.get() != 0) || p_write.get() == 0 {
        return;
    }
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if bufIsChanged(buf) as ::core::ffi::c_int != 0 && (*buf).b_p_ro == 0 && !bt_dontwrite(buf)
        {
            let mut bufref: bufref_T = bufref_T::default();
            set_bufref(&raw mut bufref, buf);
            buf_write_all(buf, false_0 != 0);
            if !bufref_valid(&raw mut bufref) {
                buf = firstbuf.get();
            }
        }
        buf = (*buf).b_next;
    }
}
pub unsafe extern "C" fn check_changed(mut buf: *mut buf_T, mut flags: ::core::ffi::c_int) -> bool {
    let mut forceit: bool = flags & CCGD_FORCEIT as ::core::ffi::c_int != 0;
    let mut bufref: bufref_T = bufref_T::default();
    set_bufref(&raw mut bufref, buf);
    if !forceit
        && bufIsChanged(buf) as ::core::ffi::c_int != 0
        && (flags & CCGD_MULTWIN as ::core::ffi::c_int != 0
            || (*buf).b_nwindows <= 1 as ::core::ffi::c_int)
        && (flags & CCGD_AW as ::core::ffi::c_int == 0 || autowrite(buf, forceit) == FAIL)
    {
        if (p_confirm.get() != 0
            || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
            && p_write.get() != 0
        {
            let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if flags & CCGD_ALLBUF as ::core::ffi::c_int != 0 {
                let mut buf2: *mut buf_T = firstbuf.get();
                while !buf2.is_null() {
                    if bufIsChanged(buf2) as ::core::ffi::c_int != 0 && !(*buf2).b_ffname.is_null()
                    {
                        count += 1;
                    }
                    buf2 = (*buf2).b_next;
                }
            }
            if !bufref_valid(&raw mut bufref) {
                return false_0 != 0;
            }
            dialog_changed(buf, count > 1 as ::core::ffi::c_int);
            if !bufref_valid(&raw mut bufref) {
                return false_0 != 0;
            }
            return bufIsChanged(buf);
        }
        if flags & CCGD_EXCMD as ::core::ffi::c_int != 0 {
            no_write_message();
        } else {
            no_write_message_nobang(curbuf.get());
        }
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn dialog_changed(mut buf: *mut buf_T, mut checkall: bool) {
    let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
    let mut ret: ::core::ffi::c_int = 0;
    let mut ea: exarg_T = exarg {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: false_0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: false_0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    dialog_msg(
        &raw mut buff as *mut ::core::ffi::c_char,
        gettext(b"Save changes to \"%s\"?\0".as_ptr() as *const ::core::ffi::c_char),
        (*buf).b_fname,
    );
    if checkall {
        ret = vim_dialog_yesnoallcancel(
            VIM_QUESTION as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            &raw mut buff as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    } else {
        ret = vim_dialog_yesnocancel(
            VIM_QUESTION as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            &raw mut buff as *mut ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    }
    if ret == VIM_YES as ::core::ffi::c_int {
        let mut empty_bufname: bool = (*buf).b_fname.is_null();
        if empty_bufname {
            buf_set_name(
                (*buf).handle as ::core::ffi::c_int,
                b"Untitled\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        }
        if check_overwrite(
            &raw mut ea,
            buf,
            (*buf).b_fname,
            (*buf).b_ffname,
            false_0 != 0,
        ) == OK
        {
            if buf_write_all(buf, false_0 != 0) == OK {
                return;
            }
        }
        if empty_bufname {
            (*buf).b_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*buf).b_ffname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*buf).b_sfname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL;
            let _ = *ptr__0;
        }
    } else if ret == VIM_NO as ::core::ffi::c_int {
        unchanged(buf, true_0 != 0, false_0 != 0);
    } else if ret == VIM_ALL as ::core::ffi::c_int {
        let mut buf2: *mut buf_T = firstbuf.get();
        while !buf2.is_null() {
            if bufIsChanged(buf2) as ::core::ffi::c_int != 0
                && !(*buf2).b_ffname.is_null()
                && (*buf2).b_p_ro == 0
            {
                let mut bufref: bufref_T = bufref_T::default();
                set_bufref(&raw mut bufref, buf2);
                if !(*buf2).b_fname.is_null()
                    && check_overwrite(
                        &raw mut ea,
                        buf2,
                        (*buf2).b_fname,
                        (*buf2).b_ffname,
                        false_0 != 0,
                    ) == OK
                {
                    buf_write_all(buf2, false_0 != 0);
                }
                if !bufref_valid(&raw mut bufref) {
                    buf2 = firstbuf.get();
                }
            }
            buf2 = (*buf2).b_next;
        }
    } else if ret == VIM_DISCARDALL as ::core::ffi::c_int {
        let mut buf2_0: *mut buf_T = firstbuf.get();
        while !buf2_0.is_null() {
            unchanged(buf2_0, true_0 != 0, false_0 != 0);
            buf2_0 = (*buf2_0).b_next;
        }
    }
}
pub unsafe extern "C" fn dialog_close_terminal(mut buf: *mut buf_T) -> bool {
    let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
    dialog_msg(
        &raw mut buff as *mut ::core::ffi::c_char,
        gettext(b"Close \"%s\"?\0".as_ptr() as *const ::core::ffi::c_char),
        (if !(*buf).b_fname.is_null() {
            (*buf).b_fname as *const ::core::ffi::c_char
        } else {
            b"?\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char,
    );
    let mut ret: ::core::ffi::c_int = vim_dialog_yesnocancel(
        VIM_QUESTION as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        &raw mut buff as *mut ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    return ret == VIM_YES as ::core::ffi::c_int;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn can_abandon(mut buf: *mut buf_T, mut forceit: bool) -> bool {
    return buf_hide(buf) as ::core::ffi::c_int != 0
        || !bufIsChanged(buf)
        || (*buf).b_nwindows > 1 as ::core::ffi::c_int
        || autowrite(buf, forceit) == OK
        || forceit as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn add_bufnum(
    mut bufnrs: *mut ::core::ffi::c_int,
    mut bufnump: *mut ::core::ffi::c_int,
    mut nr: ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < *bufnump {
        if *bufnrs.offset(i as isize) == nr {
            return;
        }
        i += 1;
    }
    *bufnrs.offset(*bufnump as isize) = nr;
    *bufnump = *bufnump + 1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn check_changed_any(mut hidden: bool, mut unload: bool) -> bool {
    let mut ret: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut bufnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut bufcount: size_t = 0 as size_t;
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        bufcount = bufcount.wrapping_add(1);
        buf = (*buf).b_next;
    }
    if bufcount == 0 as size_t {
        return false_0 != 0;
    }
    let mut bufnrs: *mut ::core::ffi::c_int =
        xmalloc(::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(bufcount))
            as *mut ::core::ffi::c_int;
    let c2rust_fresh0 = bufnum;
    bufnum = bufnum + 1;
    *bufnrs.offset(c2rust_fresh0 as isize) = (*curbuf.get()).handle as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer != curbuf.get() {
            add_bufnum(
                bufnrs,
                &raw mut bufnum,
                (*(*wp).w_buffer).handle as ::core::ffi::c_int,
            );
        }
        wp = (*wp).w_next;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        if tp != curtab.get() {
            let mut wp_0: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp_0.is_null() {
                add_bufnum(
                    bufnrs,
                    &raw mut bufnum,
                    (*(*wp_0).w_buffer).handle as ::core::ffi::c_int,
                );
                wp_0 = (*wp_0).w_next;
            }
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    let mut buf_0: *mut buf_T = firstbuf.get();
    while !buf_0.is_null() {
        add_bufnum(
            bufnrs,
            &raw mut bufnum,
            (*buf_0).handle as ::core::ffi::c_int,
        );
        buf_0 = (*buf_0).b_next;
    }
    let mut buf_1: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    i = 0 as ::core::ffi::c_int;
    while i < bufnum {
        buf_1 = buflist_findnr(*bufnrs.offset(i as isize));
        if !buf_1.is_null() {
            if (!hidden || (*buf_1).b_nwindows == 0 as ::core::ffi::c_int)
                && bufIsChanged(buf_1) as ::core::ffi::c_int != 0
            {
                let mut bufref: bufref_T = bufref_T::default();
                set_bufref(&raw mut bufref, buf_1);
                if check_changed(
                    buf_1,
                    (if p_awa.get() != 0 {
                        CCGD_AW as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) | CCGD_MULTWIN as ::core::ffi::c_int
                        | CCGD_ALLBUF as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
                    && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                {
                    break;
                }
            }
        }
        i += 1;
    }
    '_theend: {
        if i < bufnum {
            ret = true_0 != 0;
            exiting.set(false_0 != 0);
            if !(p_confirm.get() != 0
                || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
            {
                if vgetc_busy.get() > 0 as ::core::ffi::c_int {
                    msg_row.set(cmdline_row.get());
                    msg_col.set(0 as ::core::ffi::c_int);
                    msg_didout.set(false_0 != 0);
                }
                if (if !(*buf_1).terminal.is_null()
                    && channel_job_running((*buf_1).b_p_channel as uint64_t) as ::core::ffi::c_int
                        != 0
                {
                    semsg_c!(
                        gettext(b"E947: Job still running in buffer \"%s\"\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*buf_1).b_fname,
                    ) as ::core::ffi::c_int
                } else {
                    semsg_c!(
                        gettext(
                            b"E162: No write since last change for buffer \"%s\"\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        if !buf_spname(buf_1).is_null() {
                            buf_spname(buf_1)
                        } else {
                            (*buf_1).b_fname
                        },
                    ) as ::core::ffi::c_int
                }) != 0
                    && msg_didany.get() as ::core::ffi::c_int != 0
                {
                    let mut save: ::core::ffi::c_int = no_wait_return.get();
                    no_wait_return.set(false_0);
                    wait_return(false_0);
                    no_wait_return.set(save);
                }
            }
            '_buf_found: {
                if buf_1 != curbuf.get() {
                    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                    loop {
                        if tp_0.is_null() {
                            break '_buf_found;
                        }
                        let mut wp_1: *mut win_T = if tp_0 == curtab.get() {
                            firstwin.get()
                        } else {
                            (*tp_0).tp_firstwin
                        };
                        while !wp_1.is_null() {
                            if (*wp_1).w_buffer == buf_1 {
                                let mut bufref_0: bufref_T = bufref_T::default();
                                set_bufref(&raw mut bufref_0, buf_1);
                                goto_tabpage_win(tp_0 as *mut tabpage_T, wp_1);
                                if !bufref_valid(&raw mut bufref_0) {
                                    break '_theend;
                                } else {
                                    break '_buf_found;
                                }
                            } else {
                                wp_1 = (*wp_1).w_next;
                            }
                        }
                        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
                    }
                }
            }
            if buf_1 != curbuf.get() {
                set_curbuf(
                    buf_1,
                    if unload as ::core::ffi::c_int != 0 {
                        DOBUF_UNLOAD as ::core::ffi::c_int
                    } else {
                        DOBUF_GOTO as ::core::ffi::c_int
                    },
                    true_0 != 0,
                );
            }
        }
    }
    xfree(bufnrs as *mut ::core::ffi::c_void);
    return ret;
}
pub unsafe extern "C" fn check_fname() -> ::core::ffi::c_int {
    if (*curbuf.get()).b_ffname.is_null() {
        emsg(gettext(&raw const e_noname as *const ::core::ffi::c_char));
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn buf_write_all(
    mut buf: *mut buf_T,
    mut forceit: bool,
) -> ::core::ffi::c_int {
    let mut old_curbuf: *mut buf_T = curbuf.get();
    let mut retval: ::core::ffi::c_int = buf_write(
        buf,
        (*buf).b_ffname,
        (*buf).b_fname,
        1 as linenr_T,
        (*buf).b_ml.ml_line_count,
        ::core::ptr::null_mut::<exarg_T>(),
        WriteRequest {
            append: false,
            forceit,
            reset_changed: true,
            filtering: false,
        },
    );
    if curbuf.get() != old_curbuf {
        msg_source(HLF_W);
        msg(
            gettext(
                b"Warning: Entered other buffer unexpectedly (check autocommands)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            0 as ::core::ffi::c_int,
        );
    }
    return retval;
}
pub unsafe fn ex_listdo(mut eap: *mut exarg_T) {
    if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
        if ((*eap).cmdidx as ::core::ffi::c_int == CMD_ldo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int)
            && (*eap).forceit == 0
        {
            emsg(gettext(
                &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
            ));
            return;
        }
        if win_valid(prevwin.get()) as ::core::ffi::c_int != 0
            && (*prevwin.get()).w_onebuf_opt.wo_wfb == 0
        {
            win_goto(prevwin.get());
        }
        if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
            win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            if (*curwin.get()).w_onebuf_opt.wo_wfb != 0 {
                emsg(gettext(
                    &raw const e_winfixbuf_cannot_go_to_buffer as *const ::core::ffi::c_char,
                ));
                return;
            }
        }
    }
    let mut save_ei: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*msg_listdo_overwrite.ptr()) += 1;
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_windo as ::core::ffi::c_int
        && (*eap).cmdidx as ::core::ffi::c_int != CMD_tabdo as ::core::ffi::c_int
    {
        save_ei = au_event_disable(
            b",Syntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char
        );
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            (*buf).b_flags &= !BF_SYN_SET;
            buf = (*buf).b_next;
        }
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_windo as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_tabdo as ::core::ffi::c_int
        || buf_hide(curbuf.get()) as ::core::ffi::c_int != 0
        || !check_changed(
            curbuf.get(),
            CCGD_AW as ::core::ffi::c_int
                | (if (*eap).forceit != 0 {
                    CCGD_FORCEIT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                | CCGD_EXCMD as ::core::ffi::c_int,
        )
    {
        let mut next_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wp: *mut win_T = firstwin.get();
        let mut tp: *mut tabpage_T = first_tabpage.get();
        match (*eap).cmdidx as ::core::ffi::c_int {
            528 => {
                while !wp.is_null() && (i as linenr_T + 1 as linenr_T) < (*eap).line1 {
                    i += 1;
                    wp = (*wp).w_next;
                }
            }
            455 => {
                while !tp.is_null() && (i as linenr_T + 1 as linenr_T) < (*eap).line1 {
                    i += 1;
                    tp = (*tp).tp_next;
                }
            }
            10 => {
                i = (*eap).line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            }
            _ => {}
        }
        let mut buf_0: *mut buf_T = curbuf.get();
        let mut qf_size: size_t = 0 as size_t;
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_bufdo as ::core::ffi::c_int {
            buf_0 = firstbuf.get();
            while !buf_0.is_null()
                && (((*buf_0).handle as linenr_T) < (*eap).line1 || (*buf_0).b_p_bl == 0)
            {
                if (*buf_0).handle as linenr_T > (*eap).line2 {
                    buf_0 = ::core::ptr::null_mut::<buf_T>();
                    break;
                } else {
                    buf_0 = (*buf_0).b_next;
                }
            }
            if !buf_0.is_null() {
                goto_buffer(
                    eap,
                    DOBUF_FIRST as ::core::ffi::c_int,
                    FORWARD as ::core::ffi::c_int,
                    (*buf_0).handle as ::core::ffi::c_int,
                );
            }
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_cdo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_ldo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_cfdo as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int
        {
            qf_size = qf_get_valid_size(eap);
            debug_assert!((*eap).line1 >= 0 as linenr_T, "eap->line1 >= 0");
            if qf_size == 0 as size_t || (*eap).line1 as size_t > qf_size {
                buf_0 = ::core::ptr::null_mut::<buf_T>();
            } else {
                ex_cc(eap);
                buf_0 = curbuf.get();
                i = (*eap).line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                if (*eap).addr_count <= 0 as ::core::ffi::c_int {
                    debug_assert!(
                        qf_size < MAXLNUM as ::core::ffi::c_int as size_t,
                        "qf_size < MAXLNUM"
                    );
                    (*eap).line2 = qf_size as linenr_T;
                }
            }
        } else {
            setpcmark();
        }
        listcmd_busy.set(true_0 != 0);
        while !got_int.get() && !buf_0.is_null() {
            let mut execute: bool = true_0 != 0;
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_argdo as ::core::ffi::c_int {
                if i == (*(*curwin.get()).w_alist).al_ga.ga_len {
                    break;
                }
                if (*curwin.get()).w_arg_idx != i || !editing_arg_idx(curwin.get()) {
                    do_argfile(eap, i);
                }
                if (*curwin.get()).w_arg_idx != i {
                    break;
                }
            } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_windo as ::core::ffi::c_int {
                if !win_valid(wp) {
                    break;
                }
                debug_assert!(!wp.is_null(), "wp");
                execute = !(*wp).w_floating
                    || !(*wp).w_config.hide && (*wp).w_config.focusable as ::core::ffi::c_int != 0;
                if execute {
                    win_goto(wp);
                    if curwin.get() != wp {
                        break;
                    }
                }
                wp = (*wp).w_next;
            } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_tabdo as ::core::ffi::c_int {
                if !valid_tabpage(tp) {
                    break;
                }
                debug_assert!(!tp.is_null(), "tp");
                goto_tabpage_tp(tp, true_0 != 0, true_0 != 0);
                tp = (*tp).tp_next;
            } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_bufdo as ::core::ffi::c_int {
                next_fnum = -1 as ::core::ffi::c_int;
                let mut bp: *mut buf_T = (*curbuf.get()).b_next;
                while !bp.is_null() {
                    if (*bp).b_p_bl != 0 {
                        next_fnum = (*bp).handle as ::core::ffi::c_int;
                        break;
                    } else {
                        bp = (*bp).b_next;
                    }
                }
            }
            i += 1;
            if execute {
                do_cmdline(
                    (*eap).arg,
                    (*eap).ea_getline,
                    (*eap).cookie,
                    DOCMD_VERBOSE as ::core::ffi::c_int + DOCMD_NOWAIT as ::core::ffi::c_int,
                );
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_bufdo as ::core::ffi::c_int {
                if next_fnum < 0 as ::core::ffi::c_int || next_fnum as linenr_T > (*eap).line2 {
                    break;
                }
                let mut buf_still_exists: bool = false_0 != 0;
                let mut bp_0: *mut buf_T = firstbuf.get();
                while !bp_0.is_null() {
                    if (*bp_0).handle == next_fnum {
                        buf_still_exists = true_0 != 0;
                        break;
                    } else {
                        bp_0 = (*bp_0).b_next;
                    }
                }
                if !buf_still_exists {
                    break;
                }
                goto_buffer(
                    eap,
                    DOBUF_FIRST as ::core::ffi::c_int,
                    FORWARD as ::core::ffi::c_int,
                    next_fnum,
                );
                if (*curbuf.get()).handle != next_fnum {
                    break;
                }
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_cdo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_ldo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_cfdo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_lfdo as ::core::ffi::c_int
            {
                debug_assert!(i >= 0 as ::core::ffi::c_int, "i >= 0");
                if i as size_t >= qf_size || i as linenr_T >= (*eap).line2 {
                    break;
                }
                let mut qf_idx: size_t = qf_get_cur_idx(eap);
                ex_cnext(eap);
                if qf_get_cur_idx(eap) == qf_idx {
                    break;
                }
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_windo as ::core::ffi::c_int
                && execute as ::core::ffi::c_int != 0
            {
                validate_cursor(curwin.get());
                if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
                    do_check_scrollbind(true_0 != 0);
                }
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_windo as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_tabdo as ::core::ffi::c_int
            {
                if i as linenr_T + 1 as linenr_T > (*eap).line2 {
                    break;
                }
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_argdo as ::core::ffi::c_int
                && i as linenr_T >= (*eap).line2
            {
                break;
            }
        }
        listcmd_busy.set(false_0 != 0);
    }
    (*msg_listdo_overwrite.ptr()) -= 1;
    if !save_ei.is_null() {
        let mut bnext: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut aco: aco_save_T = aco_save_T::default();
        au_event_restore(save_ei);
        let mut buf_1: *mut buf_T = firstbuf.get();
        while !buf_1.is_null() {
            bnext = (*buf_1).b_next;
            if (*buf_1).b_nwindows > 0 as ::core::ffi::c_int && (*buf_1).b_flags & BF_SYN_SET != 0 {
                (*buf_1).b_flags &= !BF_SYN_SET;
                if buf_1 == curbuf.get() {
                    apply_autocmds(
                        EVENT_SYNTAX,
                        (*curbuf.get()).b_p_syn,
                        (*curbuf.get()).b_fname,
                        true_0 != 0,
                        curbuf.get(),
                    );
                } else {
                    aucmd_prepbuf(&raw mut aco, buf_1);
                    apply_autocmds(
                        EVENT_SYNTAX,
                        (*buf_1).b_p_syn,
                        (*buf_1).b_fname,
                        true_0 != 0,
                        buf_1,
                    );
                    aucmd_restbuf(&raw mut aco);
                }
                bnext = firstbuf.get();
            }
            buf_1 = bnext;
        }
    }
}
pub unsafe fn ex_compiler(mut eap: *mut exarg_T) {
    let mut old_cur_comp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        do_cmdline_cmd(
            b"echo globpath(&rtp, 'compiler/*.vim')\0".as_ptr() as *const ::core::ffi::c_char
        );
        do_cmdline_cmd(
            b"echo globpath(&rtp, 'compiler/*.lua')\0".as_ptr() as *const ::core::ffi::c_char
        );
        return;
    }
    let mut bufsize: size_t = strlen((*eap).arg).wrapping_add(14 as size_t);
    let mut buf: *mut ::core::ffi::c_char = xmalloc(bufsize) as *mut ::core::ffi::c_char;
    if (*eap).forceit != 0 {
        do_cmdline_cmd(
            b"command -nargs=* -keepscript CompilerSet set <args>\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    } else {
        old_cur_comp =
            get_var_value(b"g:current_compiler\0".as_ptr() as *const ::core::ffi::c_char);
        if !old_cur_comp.is_null() {
            old_cur_comp = xstrdup(old_cur_comp);
        }
        do_cmdline_cmd(
            b"command -nargs=* -keepscript CompilerSet setlocal <args>\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
    do_unlet(
        b"g:current_compiler\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
        true_0 != 0,
    );
    do_unlet(
        b"b:current_compiler\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
        true_0 != 0,
    );
    snprintf(
        buf,
        bufsize,
        b"compiler/%s.*\0".as_ptr() as *const ::core::ffi::c_char,
        (*eap).arg,
    );
    if source_runtime_vim_lua(buf, DIP_ALL as ::core::ffi::c_int) == FAIL {
        semsg_c!(
            gettext((e_compiler_not_supported_str.ptr() as *const _) as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    }
    xfree(buf as *mut ::core::ffi::c_void);
    do_cmdline_cmd(b":delcommand CompilerSet\0".as_ptr() as *const ::core::ffi::c_char);
    let mut p: *mut ::core::ffi::c_char =
        get_var_value(b"g:current_compiler\0".as_ptr() as *const ::core::ffi::c_char);
    if !p.is_null() {
        set_internal_string_var(
            b"b:current_compiler\0".as_ptr() as *const ::core::ffi::c_char,
            p,
        );
    }
    if (*eap).forceit == 0 {
        if !old_cur_comp.is_null() {
            set_internal_string_var(
                b"g:current_compiler\0".as_ptr() as *const ::core::ffi::c_char,
                old_cur_comp,
            );
            xfree(old_cur_comp as *mut ::core::ffi::c_void);
        } else {
            do_unlet(
                b"g:current_compiler\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
                true_0 != 0,
            );
        }
    }
}
pub unsafe fn ex_checktime(mut eap: *mut exarg_T) {
    let mut save_no_check_timestamps: ::core::ffi::c_int = no_check_timestamps.get();
    no_check_timestamps.set(0 as ::core::ffi::c_int);
    if (*eap).addr_count == 0 as ::core::ffi::c_int {
        check_timestamps(false_0);
    } else {
        let mut buf: *mut buf_T = buflist_findnr((*eap).line2 as ::core::ffi::c_int);
        if !buf.is_null() {
            buf_check_timestamp(buf);
        }
    }
    no_check_timestamps.set(save_no_check_timestamps);
}
unsafe extern "C" fn script_host_execute(
    mut name: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
) {
    let mut len: size_t = 0;
    let script: *mut ::core::ffi::c_char = script_get(eap, &raw mut len);
    if !script.is_null() {
        let args: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_allocated_string(args, script);
        tv_list_append_number(args, (*eap).line1 as ::core::ffi::c_int as varnumber_T);
        tv_list_append_number(args, (*eap).line2 as ::core::ffi::c_int as varnumber_T);
        eval_call_provider(
            name,
            b"execute\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            args,
            true_0 != 0,
        );
    }
}
unsafe extern "C" fn script_host_execute_file(
    mut name: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
) {
    if (*eap).skip == 0 {
        let mut buffer: [uint8_t; 4096] = [0; 4096];
        vim_FullName(
            (*eap).arg,
            &raw mut buffer as *mut uint8_t as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[uint8_t; 4096]>(),
            false_0 != 0,
        );
        let mut args: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_string(
            args,
            &raw mut buffer as *mut uint8_t as *const ::core::ffi::c_char,
            -1 as ssize_t,
        );
        tv_list_append_number(args, (*eap).line1 as ::core::ffi::c_int as varnumber_T);
        tv_list_append_number(args, (*eap).line2 as ::core::ffi::c_int as varnumber_T);
        eval_call_provider(
            name,
            b"execute_file\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            args,
            true_0 != 0,
        );
    }
}
unsafe extern "C" fn script_host_do_range(
    mut name: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
) {
    if (*eap).skip == 0 {
        let mut args: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
        tv_list_append_number(args, (*eap).line1 as ::core::ffi::c_int as varnumber_T);
        tv_list_append_number(args, (*eap).line2 as ::core::ffi::c_int as varnumber_T);
        tv_list_append_string(args, (*eap).arg, -1 as ssize_t);
        eval_call_provider(
            name,
            b"do_range\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            args,
            true_0 != 0,
        );
    }
}
pub unsafe fn ex_drop(mut eap: *mut exarg_T) {
    let mut split: bool = false_0 != 0;
    set_arglist((*eap).arg);
    if (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as ::core::ffi::c_int {
        return;
    }
    if (*cmdmod.ptr()).cmod_tab != 0 {
        ex_all(eap);
        (*cmdmod.ptr()).cmod_tab = 0 as ::core::ffi::c_int;
        ex_rewind(eap);
        return;
    }
    let mut buf: *mut buf_T = buflist_findnr(
        (*((*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T)
            .offset(0 as ::core::ffi::c_int as isize))
        .ae_fnum,
    );
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                goto_tabpage_win(tp as *mut tabpage_T, wp);
                (*curwin.get()).w_arg_idx = 0 as ::core::ffi::c_int;
                if !bufIsChanged(curbuf.get()) {
                    let save_ar: ::core::ffi::c_int = (*curbuf.get()).b_p_ar;
                    (*curbuf.get()).b_p_ar = true_0;
                    buf_check_timestamp(curbuf.get());
                    (*curbuf.get()).b_p_ar = save_ar;
                }
                if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                    ex_rewind(eap);
                }
                if !(*eap).do_ecmd_cmd.is_null() {
                    let mut did_set_swapcommand: bool =
                        set_swapcommand((*eap).do_ecmd_cmd, 0 as linenr_T);
                    do_cmdline(
                        (*eap).do_ecmd_cmd,
                        None,
                        NULL,
                        DOCMD_VERBOSE as ::core::ffi::c_int,
                    );
                    if did_set_swapcommand {
                        set_vim_var_string(
                            VV_SWAPCOMMAND,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                            -1 as ptrdiff_t,
                        );
                    }
                }
                return;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    if !buf_hide(curbuf.get()) {
        (*emsg_off.ptr()) += 1;
        split = check_changed(
            curbuf.get(),
            CCGD_AW as ::core::ffi::c_int | CCGD_EXCMD as ::core::ffi::c_int,
        );
        (*emsg_off.ptr()) -= 1;
    }
    if split {
        (*eap).cmdidx = CMD_sfirst;
        *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize) = 's' as ::core::ffi::c_char;
    } else {
        (*eap).cmdidx = CMD_first;
    }
    ex_rewind(eap);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
