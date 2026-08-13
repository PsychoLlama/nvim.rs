//! `:buffer`, `:bnext`, `:bdelete` and friends -- `do_buffer()`.
//!
//! [`do_buffer_ext`] is the whole `:buffer`-family command: resolve the
//! argument to a buffer (by number, by name, relative to the current one, or
//! by the alternate file), decide whether the current buffer may be
//! abandoned, and then either switch to the target, unload it, delete it or
//! wipe it.  [`do_bufdel`] is the range form, [`goto_buffer`] the split/hide
//! wrapper, and [`empty_curbuf`] the "there is nothing left to show" fallback
//! when the last listed buffer goes away.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::autocmd::{block_autocmds, is_aucmd_win, unblock_autocmds};
use crate::src::nvim::charset::{getdigits_int, skiptowhite_esc, skipwhite};
use crate::src::nvim::ex_cmds::do_ecmd;
use crate::src::nvim::ex_cmds2::{can_abandon, dialog_changed, dialog_close_terminal};
use crate::src::nvim::ex_docmd::ex_errmsg;
use crate::src::nvim::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::src::nvim::main::{
    IObuff, VIsual_active, au_new_curbuf, cmdline_row, cmdmod, curbuf, curwin,
    e_cannot_switch_to_a_closing_buffer,
    e_no_write_since_last_change_for_buffer_nr_add_bang_to_override, e_nobufnr, e_trailing_arg,
    firstbuf, firstwin, got_int, jop_flags, lastbuf, lastwin, msg_row, msg_scroll, need_fileinfo,
    p_confirm, p_report, p_write, swap_exists_action, swap_exists_did_quit,
};
use crate::src::nvim::mark::{mark_jumplist_forget_file, setpcmark};
use crate::src::nvim::memline::ml_recover;
use crate::src::nvim::memory::xstrlcpy;
use crate::src::nvim::message::{emsg, msg_puts};
use crate::src::nvim::normal::end_visual_mode;
use crate::src::nvim::options::kOptJopFlagClean;
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{gettext, ngettext};
use crate::src::nvim::search::FORWARD;
use crate::src::nvim::terminal::terminal_running;
use crate::src::nvim::types::{
    CMOD_CONFIRM, OptInt, buf_T, bufref_T, cleanup_T, exarg_T, except_T, int64_t, linenr_T, size_t,
    win_T,
};
use crate::src::nvim::undo::bufIsChanged;
use crate::src::nvim::window::{
    check_can_set_curbuf_forceit, check_colorcolumn, close_windows, last_window,
    swbuf_goto_win_with_buf, win_close, win_locked, win_split,
};
use crate::{semsg_c, smsg_c};

pub unsafe extern "C" fn goto_buffer(
    mut eap: *mut exarg_T,
    mut start: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
) {
    unsafe {
        let save_sea: ::core::ffi::c_int = swap_exists_action.get();
        let mut skip_help_buf: bool = false;
        match (*eap).cmdidx as ::core::ffi::c_int {
            30 | 394 | 21 | 32 | 389 | 395 => {
                skip_help_buf = true_0 != 0;
            }
            _ => {
                skip_help_buf = false_0 != 0;
            }
        }
        let mut old_curbuf: bufref_T = bufref_T::default();
        set_bufref(&raw mut old_curbuf, curbuf.get());
        if swap_exists_action.get() == SEA_NONE {
            swap_exists_action.set(SEA_DIALOG);
        }
        do_buffer_ext(
            if *(*eap).cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int {
                DOBUF_SPLIT as ::core::ffi::c_int
            } else {
                DOBUF_GOTO as ::core::ffi::c_int
            },
            start,
            dir,
            count,
            (if (*eap).forceit != 0 {
                DOBUF_FORCEIT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | (if skip_help_buf as ::core::ffi::c_int != 0 {
                DOBUF_SKIPHELP as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
        );
        if swap_exists_action.get() == SEA_QUIT
            && *(*eap).cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int
        {
            let mut cs: cleanup_T = cleanup_T {
                pending: 0,
                exception: ::core::ptr::null_mut::<except_T>(),
            };
            enter_cleanup(&raw mut cs);
            win_close(curwin.get(), true_0 != 0, false_0 != 0);
            swap_exists_action.set(save_sea);
            swap_exists_did_quit.set(true_0 != 0);
            leave_cleanup(&raw mut cs);
        } else {
            handle_swap_exists(&raw mut old_curbuf);
        };
    }
}

pub unsafe extern "C" fn handle_swap_exists(mut old_curbuf: *mut bufref_T) {
    unsafe {
        let mut cs: cleanup_T = cleanup_T {
            pending: 0,
            exception: ::core::ptr::null_mut::<except_T>(),
        };
        let mut old_tw: OptInt = (*curbuf.get()).b_p_tw;
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if swap_exists_action.get() == SEA_QUIT {
            enter_cleanup(&raw mut cs);
            swap_exists_action.set(SEA_NONE);
            swap_exists_did_quit.set(true_0 != 0);
            close_buffer(
                curwin.get(),
                curbuf.get(),
                DOBUF_UNLOAD as ::core::ffi::c_int,
                false_0 != 0,
                false_0 != 0,
            );
            if old_curbuf.is_null()
                || !bufref_valid(old_curbuf)
                || (*old_curbuf).br_buf == curbuf.get()
            {
                block_autocmds();
                buf = buflist_new(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    1 as linenr_T,
                    BLN_CURBUF as ::core::ffi::c_int | BLN_LISTED as ::core::ffi::c_int,
                );
                unblock_autocmds();
            } else {
                buf = (*old_curbuf).br_buf;
            }
            if !buf.is_null() {
                enter_buffer(buf);
                if old_tw != (*curbuf.get()).b_p_tw {
                    check_colorcolumn(::core::ptr::null_mut::<::core::ffi::c_char>(), curwin.get());
                }
            }
            leave_cleanup(&raw mut cs);
        } else if swap_exists_action.get() == SEA_RECOVER {
            enter_cleanup(&raw mut cs);
            msg_scroll.set(true_0);
            ml_recover(false_0 != 0);
            msg_puts(c"\n".as_ptr());
            cmdline_row.set(msg_row.get());
            do_modelines(0 as ::core::ffi::c_int);
            leave_cleanup(&raw mut cs);
        }
        swap_exists_action.set(SEA_NONE);
    }
}

pub unsafe extern "C" fn do_bufdel(
    mut command: ::core::ffi::c_int,
    mut arg: *mut ::core::ffi::c_char,
    mut addr_count: ::core::ffi::c_int,
    mut start_bnr: ::core::ffi::c_int,
    mut end_bnr: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut do_current: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut deleted: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut errormsg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut bnr: ::core::ffi::c_int = 0;
        if addr_count == 0 as ::core::ffi::c_int {
            do_buffer(
                command,
                DOBUF_CURRENT as ::core::ffi::c_int,
                FORWARD as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                forceit,
            );
        } else {
            if addr_count == 2 as ::core::ffi::c_int {
                if *arg != 0 {
                    return ex_errmsg(&raw const e_trailing_arg as *const ::core::ffi::c_char, arg);
                }
                bnr = start_bnr;
            } else {
                bnr = end_bnr;
            }
            while !got_int.get() {
                if bnr == (*curbuf.get()).handle {
                    do_current = bnr;
                } else if do_buffer(
                    command,
                    DOBUF_FIRST as ::core::ffi::c_int,
                    FORWARD as ::core::ffi::c_int,
                    bnr,
                    forceit,
                ) == OK
                {
                    deleted += 1;
                }
                if addr_count == 2 as ::core::ffi::c_int {
                    bnr += 1;
                    if bnr > end_bnr {
                        break;
                    }
                } else {
                    arg = skipwhite(arg);
                    if *arg as ::core::ffi::c_int == NUL {
                        break;
                    }
                    if !ascii_isdigit(*arg as ::core::ffi::c_int) {
                        let mut p: *mut ::core::ffi::c_char = skiptowhite_esc(arg);
                        bnr = buflist_findpat(
                            arg,
                            p,
                            command == DOBUF_WIPE as ::core::ffi::c_int,
                            false_0 != 0,
                            false_0 != 0,
                        );
                        if bnr < 0 as ::core::ffi::c_int {
                            break;
                        }
                        arg = p;
                    } else {
                        bnr = getdigits_int(&raw mut arg, false_0 != 0, 0 as ::core::ffi::c_int);
                    }
                }
                os_breakcheck();
            }
            if !got_int.get()
                && do_current != 0
                && do_buffer(
                    command,
                    DOBUF_FIRST as ::core::ffi::c_int,
                    FORWARD as ::core::ffi::c_int,
                    do_current,
                    forceit,
                ) == OK
            {
                deleted += 1;
            }
            if deleted == 0 as ::core::ffi::c_int {
                if command == DOBUF_UNLOAD as ::core::ffi::c_int {
                    xstrlcpy(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        gettext(c"E515: No buffers were unloaded".as_ptr()),
                        IOSIZE as size_t,
                    );
                } else if command == DOBUF_DEL as ::core::ffi::c_int {
                    xstrlcpy(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        gettext(c"E516: No buffers were deleted".as_ptr()),
                        IOSIZE as size_t,
                    );
                } else {
                    xstrlcpy(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        gettext(c"E517: No buffers were wiped out".as_ptr()),
                        IOSIZE as size_t,
                    );
                }
                errormsg = IObuff.ptr() as *mut ::core::ffi::c_char;
            } else if deleted as OptInt >= p_report.get() {
                if command == DOBUF_UNLOAD as ::core::ffi::c_int {
                    smsg_c!(
                        0 as ::core::ffi::c_int,
                        ngettext(
                            c"%d buffer unloaded".as_ptr(),
                            c"%d buffers unloaded".as_ptr(),
                            deleted as ::core::ffi::c_ulong,
                        ),
                        deleted,
                    );
                } else if command == DOBUF_DEL as ::core::ffi::c_int {
                    smsg_c!(
                        0 as ::core::ffi::c_int,
                        ngettext(
                            c"%d buffer deleted".as_ptr(),
                            c"%d buffers deleted".as_ptr(),
                            deleted as ::core::ffi::c_ulong,
                        ),
                        deleted,
                    );
                } else {
                    smsg_c!(
                        0 as ::core::ffi::c_int,
                        ngettext(
                            c"%d buffer wiped out".as_ptr(),
                            c"%d buffers wiped out".as_ptr(),
                            deleted as ::core::ffi::c_ulong,
                        ),
                        deleted,
                    );
                }
            }
        }
        return errormsg;
    }
}

unsafe extern "C" fn empty_curbuf(
    mut close_others: bool,
    mut forceit: ::core::ffi::c_int,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut buf_T = curbuf.get();
        if action == DOBUF_UNLOAD as ::core::ffi::c_int {
            emsg(gettext(c"E90: Cannot unload last buffer".as_ptr()));
            return FAIL;
        }
        let mut bufref: bufref_T = bufref_T::default();
        set_bufref(&raw mut bufref, buf);
        if close_others {
            let mut can_close_all_others: bool = true_0 != 0;
            if (*curwin.get()).w_floating {
                can_close_all_others = false_0 != 0;
                let mut wp: *mut win_T = firstwin.get();
                while !(*wp).w_floating {
                    if (*wp).w_buffer != curbuf.get() {
                        can_close_all_others = true_0 != 0;
                        break;
                    } else {
                        wp = (*wp).w_next;
                    }
                }
            }
            close_windows(buf, can_close_all_others);
        }
        setpcmark();
        let mut retval: ::core::ffi::c_int = do_ecmd(
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<exarg_T>(),
            ECMD_ONE as ::core::ffi::c_int as linenr_T,
            if forceit != 0 {
                ECMD_FORCEIT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            curwin.get(),
        );
        if buf != curbuf.get()
            && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
            && (*buf).b_nwindows == 0 as ::core::ffi::c_int
        {
            close_buffer(
                ::core::ptr::null_mut::<win_T>(),
                buf,
                action,
                false_0 != 0,
                false_0 != 0,
            );
        }
        if !close_others {
            need_fileinfo.set(false_0 != 0);
        }
        return retval;
    }
}

unsafe extern "C" fn do_buffer_ext(
    mut action: ::core::ffi::c_int,
    mut start: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut bp: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut update_jumplist: bool = true_0 != 0;
        let mut unload: bool = action == DOBUF_UNLOAD as ::core::ffi::c_int
            || action == DOBUF_DEL as ::core::ffi::c_int
            || action == DOBUF_WIPE as ::core::ffi::c_int;
        match start {
            1 => {
                buf = firstbuf.get();
            }
            2 => {
                buf = lastbuf.get();
            }
            _ => {
                buf = curbuf.get();
            }
        }
        if start == DOBUF_MOD as ::core::ffi::c_int {
            loop {
                let c2rust_fresh2 = count;
                count = count - 1;
                if c2rust_fresh2 <= 0 as ::core::ffi::c_int {
                    break;
                }
                loop {
                    buf = (*buf).b_next;
                    if buf.is_null() {
                        buf = firstbuf.get();
                    }
                    if !(buf != curbuf.get() && !bufIsChanged(buf)) {
                        break;
                    }
                }
            }
            if !bufIsChanged(buf) {
                emsg(gettext(c"E84: No modified buffer found".as_ptr()));
                return FAIL;
            }
        } else if start == DOBUF_FIRST as ::core::ffi::c_int && count != 0 {
            while !buf.is_null() && (*buf).handle != count {
                buf = (*buf).b_next;
            }
        } else {
            let help_only: bool = flags & DOBUF_SKIPHELP as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
                && (*buf).b_help as ::core::ffi::c_int != 0;
            bp = ::core::ptr::null_mut::<buf_T>();
            while count > 0 as ::core::ffi::c_int
                || bp != buf
                    && !unload
                    && (if help_only as ::core::ffi::c_int != 0 {
                        (*buf).b_help as ::core::ffi::c_int
                    } else {
                        (*buf).b_p_bl
                    }) == 0
            {
                if bp.is_null() {
                    bp = buf;
                }
                buf = if dir == FORWARD as ::core::ffi::c_int {
                    if !(*buf).b_next.is_null() {
                        (*buf).b_next
                    } else {
                        firstbuf.get()
                    }
                } else if !(*buf).b_prev.is_null() {
                    (*buf).b_prev
                } else {
                    lastbuf.get()
                };
                if unload as ::core::ffi::c_int != 0
                    || (if help_only as ::core::ffi::c_int != 0 {
                        (*buf).b_help as ::core::ffi::c_int
                    } else {
                        ((*buf).b_p_bl != 0
                            && (flags & DOBUF_SKIPHELP as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                                || !(*buf).b_help)) as ::core::ffi::c_int
                    }) != 0
                {
                    count -= 1;
                    bp = ::core::ptr::null_mut::<buf_T>();
                }
                if bp == buf {
                    emsg(gettext(c"E85: There is no listed buffer".as_ptr()));
                    return FAIL;
                }
            }
        }
        if buf.is_null() {
            if start == DOBUF_FIRST as ::core::ffi::c_int {
                if !unload {
                    semsg_c!(
                        gettext(&raw const e_nobufnr as *const ::core::ffi::c_char),
                        count as int64_t,
                    );
                }
            } else if dir == FORWARD as ::core::ffi::c_int {
                emsg(gettext(c"E87: Cannot go beyond last buffer".as_ptr()));
            } else {
                emsg(gettext(c"E88: Cannot go before first buffer".as_ptr()));
            }
            return FAIL;
        }
        if action == DOBUF_GOTO as ::core::ffi::c_int
            && buf != curbuf.get()
            && !check_can_set_curbuf_forceit(
                (flags & DOBUF_FORCEIT as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int,
            )
        {
            return FAIL;
        }
        if (action == DOBUF_GOTO as ::core::ffi::c_int
            || action == DOBUF_SPLIT as ::core::ffi::c_int)
            && (*buf).b_flags & BF_DUMMY != 0
        {
            semsg_c!(
                gettext(&raw const e_nobufnr as *const ::core::ffi::c_char),
                count,
            );
            return FAIL;
        }
        if unload {
            let mut forward: ::core::ffi::c_int = 0;
            let mut bufref: bufref_T = bufref_T::default();
            if !can_unload_buffer(buf) {
                return FAIL;
            }
            set_bufref(&raw mut bufref, buf);
            if action != DOBUF_WIPE as ::core::ffi::c_int
                && (*buf).b_ml.ml_mfp.is_null()
                && (*buf).b_p_bl == 0
            {
                return FAIL;
            }
            if flags & DOBUF_FORCEIT as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                && bufIsChanged(buf) as ::core::ffi::c_int != 0
            {
                if (p_confirm.get() != 0
                    || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
                    && p_write.get() != 0
                {
                    dialog_changed(buf, false_0 != 0);
                    if !bufref_valid(&raw mut bufref) {
                        return FAIL;
                    }
                    if bufIsChanged(buf) {
                        return FAIL;
                    }
                } else {
                    semsg_c!(
                    gettext(
                        &raw const e_no_write_since_last_change_for_buffer_nr_add_bang_to_override
                            as *const ::core::ffi::c_char,
                    ),
                    (*buf).handle,
                );
                    return FAIL;
                }
            }
            if flags & DOBUF_FORCEIT as ::core::ffi::c_int == 0
                && !(*buf).terminal.is_null()
                && terminal_running((*buf).terminal) as ::core::ffi::c_int != 0
            {
                if p_confirm.get() != 0
                    || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
                {
                    if !dialog_close_terminal(buf) {
                        return FAIL;
                    }
                } else {
                    semsg_c!(
                        gettext(c"E89: %s will be killed (add ! to override)".as_ptr()),
                        (*buf).b_fname,
                    );
                    return FAIL;
                }
            }
            let mut buf_fnum: ::core::ffi::c_int = (*buf).handle as ::core::ffi::c_int;
            if buf == curbuf.get() && VIsual_active.get() as ::core::ffi::c_int != 0 {
                end_visual_mode();
            }
            bp = ::core::ptr::null_mut::<buf_T>();
            let mut bp2: *mut buf_T = firstbuf.get();
            while !bp2.is_null() {
                if (*bp2).b_p_bl != 0 && bp2 != buf {
                    bp = bp2;
                    break;
                } else {
                    bp2 = (*bp2).b_next;
                }
            }
            if bp.is_null() && buf == curbuf.get() {
                return empty_curbuf(
                    true_0 != 0,
                    flags & DOBUF_FORCEIT as ::core::ffi::c_int,
                    action,
                );
            }
            while buf == curbuf.get()
                && !(win_locked(curwin.get()) != 0
                    || (*(*curwin.get()).w_buffer).b_locked > 0 as ::core::ffi::c_int)
                && (is_aucmd_win(lastwin.get()) as ::core::ffi::c_int != 0
                    || !last_window(curwin.get()))
            {
                if win_close(curwin.get(), false_0 != 0, false_0 != 0) == FAIL {
                    break;
                }
            }
            if buf != curbuf.get() {
                if jop_flags.get() & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                {
                    mark_jumplist_forget_file(curwin.get(), buf_fnum);
                }
                close_windows(buf, false_0 != 0);
                if buf != curbuf.get()
                    && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                    && (*buf).b_nwindows <= 0 as ::core::ffi::c_int
                {
                    close_buffer(
                        ::core::ptr::null_mut::<win_T>(),
                        buf,
                        action,
                        false_0 != 0,
                        false_0 != 0,
                    );
                }
                return OK;
            }
            buf = ::core::ptr::null_mut::<buf_T>();
            bp = ::core::ptr::null_mut::<buf_T>();
            if !(*au_new_curbuf.ptr()).br_buf.is_null()
                && bufref_valid(au_new_curbuf.ptr()) as ::core::ffi::c_int != 0
                && (*(*au_new_curbuf.ptr()).br_buf).b_locked_split == 0
            {
                buf = (*au_new_curbuf.ptr()).br_buf;
            } else if (*curwin.get()).w_jumplistlen > 0 as ::core::ffi::c_int {
                if jop_flags.get() & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                {
                    mark_jumplist_forget_file(curwin.get(), buf_fnum);
                }
                if (*curwin.get()).w_jumplistlen > 0 as ::core::ffi::c_int {
                    let mut jumpidx: ::core::ffi::c_int = (*curwin.get()).w_jumplistidx;
                    if jop_flags.get()
                        & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        if jumpidx == (*curwin.get()).w_jumplistlen {
                            (*curwin.get()).w_jumplistidx =
                                (*curwin.get()).w_jumplistlen - 1 as ::core::ffi::c_int;
                            jumpidx = (*curwin.get()).w_jumplistidx;
                        }
                    } else {
                        jumpidx -= 1;
                        if jumpidx < 0 as ::core::ffi::c_int {
                            jumpidx = (*curwin.get()).w_jumplistlen - 1 as ::core::ffi::c_int;
                        }
                    }
                    forward = jumpidx;
                    while jop_flags.get()
                        & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                        || jumpidx != (*curwin.get()).w_jumplistidx
                    {
                        buf =
                            buflist_findnr((*curwin.get()).w_jumplist[jumpidx as usize].fmark.fnum);
                        if !buf.is_null() {
                            if buf == curbuf.get()
                                || (*buf).b_p_bl == 0
                                || bt_quickfix(buf) as ::core::ffi::c_int != 0
                                || (*buf).b_locked_split != 0
                            {
                                buf = ::core::ptr::null_mut::<buf_T>();
                            } else if (*buf).b_ml.ml_mfp.is_null() {
                                if bp.is_null() {
                                    bp = buf;
                                }
                                buf = ::core::ptr::null_mut::<buf_T>();
                            }
                        }
                        if !buf.is_null() {
                            if jop_flags.get()
                                & kOptJopFlagClean as ::core::ffi::c_int as ::core::ffi::c_uint
                                != 0
                            {
                                (*curwin.get()).w_jumplistidx = jumpidx;
                                update_jumplist = false_0 != 0;
                            }
                            break;
                        } else {
                            if jumpidx == 0
                                && (*curwin.get()).w_jumplistidx == (*curwin.get()).w_jumplistlen
                            {
                                break;
                            }
                            jumpidx -= 1;
                            if jumpidx < 0 as ::core::ffi::c_int {
                                jumpidx = (*curwin.get()).w_jumplistlen - 1 as ::core::ffi::c_int;
                            }
                            if jumpidx == forward {
                                break;
                            }
                        }
                    }
                }
            }
            if buf.is_null() {
                forward = true_0;
                buf = (*curbuf.get()).b_next;
                loop {
                    if buf.is_null() {
                        if forward == 0 {
                            break;
                        }
                        buf = (*curbuf.get()).b_prev;
                        forward = false_0;
                    } else {
                        if (*buf).b_help as ::core::ffi::c_int
                            == (*curbuf.get()).b_help as ::core::ffi::c_int
                            && (*buf).b_p_bl != 0
                            && !bt_quickfix(buf)
                            && (*buf).b_locked_split == 0
                        {
                            if !(*buf).b_ml.ml_mfp.is_null() {
                                break;
                            }
                            if bp.is_null() {
                                bp = buf;
                            }
                        }
                        buf = if forward != 0 {
                            (*buf).b_next
                        } else {
                            (*buf).b_prev
                        };
                    }
                }
            }
            if buf.is_null() {
                buf = bp;
            }
            if buf.is_null() {
                let mut buf2: *mut buf_T = firstbuf.get();
                while !buf2.is_null() {
                    if (*buf2).b_p_bl != 0
                        && buf2 != curbuf.get()
                        && !bt_quickfix(buf2)
                        && (*buf2).b_locked_split == 0
                    {
                        buf = buf2;
                        break;
                    } else {
                        buf2 = (*buf2).b_next;
                    }
                }
            }
            if buf.is_null() {
                buf = if !(*curbuf.get()).b_next.is_null() {
                    (*curbuf.get()).b_next
                } else {
                    (*curbuf.get()).b_prev
                };
                if bt_quickfix(buf) as ::core::ffi::c_int != 0
                    || buf != curbuf.get() && (*buf).b_locked_split != 0
                {
                    buf = ::core::ptr::null_mut::<buf_T>();
                }
            }
        }
        if buf.is_null() {
            return empty_curbuf(
                false_0 != 0,
                flags & DOBUF_FORCEIT as ::core::ffi::c_int,
                action,
            );
        }
        if action == DOBUF_SPLIT as ::core::ffi::c_int && !swbuf_goto_win_with_buf(buf).is_null() {
            return OK;
        }
        if buf != curbuf.get() && (*buf).b_locked_split != 0 {
            emsg(gettext(
                &raw const e_cannot_switch_to_a_closing_buffer as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if action == DOBUF_SPLIT as ::core::ffi::c_int
            && win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) == FAIL
        {
            return FAIL;
        }
        if buf == curbuf.get() {
            return OK;
        }
        if action == DOBUF_GOTO as ::core::ffi::c_int
            && !can_abandon(
                curbuf.get(),
                flags & DOBUF_FORCEIT as ::core::ffi::c_int != 0,
            )
        {
            if (p_confirm.get() != 0
                || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
                && p_write.get() != 0
            {
                let mut bufref_0: bufref_T = bufref_T::default();
                set_bufref(&raw mut bufref_0, buf);
                dialog_changed(curbuf.get(), false_0 != 0);
                if !bufref_valid(&raw mut bufref_0) {
                    return FAIL;
                }
            }
            if bufIsChanged(curbuf.get()) {
                no_write_message();
                return FAIL;
            }
        }
        set_curbuf(buf, action, update_jumplist);
        if action == DOBUF_SPLIT as ::core::ffi::c_int {
            (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
            (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
        }
        if aborting() {
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn do_buffer(
    mut action: ::core::ffi::c_int,
    mut start: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return do_buffer_ext(
            action,
            start,
            dir,
            count,
            if forceit != 0 {
                DOBUF_FORCEIT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        );
    }
}
