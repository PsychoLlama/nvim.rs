//! Making a buffer current -- `set_curbuf()` and `enter_buffer()`.
//!
//! [`set_curbuf`] leaves the old buffer (`BufLeave`, remembering the cursor
//! position for the window) and [`enter_buffer`] arrives in the new one:
//! apply the window's remembered position, load the buffer if it is not
//! loaded, re-apply the local options and folds, and fire
//! `BufEnter`/`BufWinEnter`.  The `no_write_message*` trio is the "no write
//! since last change" error every caller of these has to be able to
//! raise.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::semsg_c;
use crate::src::nvim::arglist::check_arg_idx;
use crate::src::nvim::autocmd::{
    EVENT_BUFENTER, EVENT_BUFLEAVE, EVENT_BUFWINENTER, apply_autocmds,
};
use crate::src::nvim::channel::channel_job_running;
use crate::src::nvim::diff::diff_buf_add;
use crate::src::nvim::digraph::keymap_init;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::src::nvim::eval::typval::tv_dict_add;
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::file_search::vim_chdirfile;
use crate::src::nvim::fileio::{buf_check_timestamp, shorten_fnames};
use crate::src::nvim::fold::{clearFolding, foldUpdateAll};
use crate::src::nvim::indent::inindent;
use crate::src::nvim::main::{
    State, VIsual_active, VIsual_reselect, cmdmod, curbuf, curwin, e_job_still_running,
    e_job_still_running_add_bang_to_end_the_job, e_no_write_since_last_change,
    e_no_write_since_last_change_add_bang_to_override,
    e_no_write_since_last_change_for_buffer_nr_add_bang_to_override, last_chdir_reason, lastbuf,
    msg_silent, need_fileinfo, p_acd, starting,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::message::emsg;
use crate::src::nvim::r#move::scroll_cursor_halfway;
use crate::src::nvim::normal::end_visual_mode;
use crate::src::nvim::option::{buf_copy_options, shortmess};
use crate::src::nvim::os::libc::{gettext, time};
use crate::src::nvim::spell::parse_spelllang;
use crate::src::nvim::state::MODE_INSERT;
use crate::src::nvim::syntax::reset_synblock;
use crate::src::nvim::terminal::terminal_check_size;
use crate::src::nvim::types::{
    CMOD_KEEPALT, ChangedtickDictItem, OptInt, VAR_FIXED, VAR_NUMBER, buf_T, bufref_T, colnr_T,
    dictitem_T, exarg_T, linenr_T, time_t, typval_T, typval_vval_union, uint8_t, uint64_t, win_T,
};
use crate::src::nvim::undo::{bufIsChanged, u_sync};
use crate::src::nvim::window::{check_colorcolumn, close_windows, get_last_winid, win_valid};

pub unsafe extern "C" fn set_curbuf(
    mut buf: *mut buf_T,
    mut action: ::core::ffi::c_int,
    mut update_jumplist: bool,
) {
    unsafe {
        let mut prevbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut unload: ::core::ffi::c_int = (action == DOBUF_UNLOAD as ::core::ffi::c_int
            || action == DOBUF_DEL as ::core::ffi::c_int
            || action == DOBUF_WIPE as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        let mut old_tw: OptInt = (*curbuf.get()).b_p_tw;
        let last_winid: ::core::ffi::c_int = get_last_winid();
        if update_jumplist {
            setpcmark();
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curwin.get()).w_alt_fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
        }
        buflist_altfpos(curwin.get());
        VIsual_reselect.set(false_0);
        prevbuf = curbuf.get();
        let mut newbufref: bufref_T = bufref_T::default();
        let mut prevbufref: bufref_T = bufref_T::default();
        set_bufref(&raw mut prevbufref, prevbuf);
        set_bufref(&raw mut newbufref, buf);
        let prev_nwindows: ::core::ffi::c_int = (*prevbuf).b_nwindows;
        if !apply_autocmds(
            EVENT_BUFLEAVE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        ) || bufref_valid(&raw mut prevbufref) as ::core::ffi::c_int != 0
            && bufref_valid(&raw mut newbufref) as ::core::ffi::c_int != 0
            && !aborting()
        {
            if prevbuf == (*curwin.get()).w_buffer {
                reset_synblock(curwin.get());
            }
            if unload != 0
                || prev_nwindows <= 1 as ::core::ffi::c_int
                    && last_winid != get_last_winid()
                    && action == DOBUF_GOTO as ::core::ffi::c_int
                    && !buf_hide(prevbuf)
            {
                close_windows(prevbuf, false_0 != 0);
            }
            if bufref_valid(&raw mut prevbufref) as ::core::ffi::c_int != 0 && !aborting() {
                let mut previouswin: *mut win_T = curwin.get();
                if prevbuf == curbuf.get()
                    && (State.get() & MODE_INSERT == 0 as ::core::ffi::c_int
                        || (*curbuf.get()).b_nwindows <= 1 as ::core::ffi::c_int)
                {
                    u_sync(false_0 != 0);
                }
                close_buffer(
                    if prevbuf == (*curwin.get()).w_buffer {
                        curwin.get()
                    } else {
                        ::core::ptr::null_mut::<win_T>()
                    },
                    prevbuf,
                    if unload != 0 {
                        action
                    } else if action == DOBUF_GOTO as ::core::ffi::c_int
                        && !buf_hide(prevbuf)
                        && !bufIsChanged(prevbuf)
                    {
                        DOBUF_UNLOAD as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                    false_0 != 0,
                    false_0 != 0,
                );
                if curwin.get() != previouswin && win_valid(previouswin) as ::core::ffi::c_int != 0
                {
                    curwin.set(previouswin);
                }
            }
        }
        let mut valid: bool = buf_valid(buf);
        if valid as ::core::ffi::c_int != 0 && buf != curbuf.get() && !aborting()
            || (*curwin.get()).w_buffer.is_null()
        {
            if !(*curbuf.ptr()).is_null() && prevbuf != curbuf.get() {
                (*curbuf.get()).b_nwindows -= 1;
            }
            enter_buffer(if valid as ::core::ffi::c_int != 0 {
                buf
            } else {
                lastbuf.get()
            });
            if old_tw != (*curbuf.get()).b_p_tw {
                check_colorcolumn(::core::ptr::null_mut::<::core::ffi::c_char>(), curwin.get());
            }
        }
        if bufref_valid(&raw mut prevbufref) as ::core::ffi::c_int != 0
            && !(*prevbuf).terminal.is_null()
        {
            terminal_check_size((*prevbuf).terminal);
        }
    }
}

pub(crate) unsafe extern "C" fn enter_buffer(mut buf: *mut buf_T) {
    unsafe {
        if VIsual_active.get() {
            end_visual_mode();
        }
        (*curwin.get()).w_buffer = buf;
        curbuf.set(buf);
        (*curbuf.get()).b_nwindows += 1;
        buf_copy_options(
            buf,
            BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
        );
        if !(*buf).b_help {
            get_winopts(buf);
        } else {
            clearFolding(curwin.get());
        }
        foldUpdateAll(curwin.get());
        if (*curwin.get()).w_onebuf_opt.wo_diff != 0 {
            diff_buf_add(curbuf.get());
        }
        (*curwin.get()).w_s = &raw mut (*curbuf.get()).b_s;
        (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_set_curswant = true_0;
        (*curwin.get()).w_topline_was_set = false_0 as ::core::ffi::c_char;
        (*curwin.get()).w_valid = 0 as ::core::ffi::c_int;
        if (*curbuf.get()).b_ml.ml_mfp.is_null() {
            if *(*curbuf.get()).b_p_ft as ::core::ffi::c_int == NUL {
                (*curbuf.get()).b_did_filetype = false_0 != 0;
            }
            open_buffer(
                false_0 != 0,
                ::core::ptr::null_mut::<exarg_T>(),
                0 as ::core::ffi::c_int,
            );
        } else {
            if msg_silent.get() == 0 && !shortmess(SHM_FILEINFO as ::core::ffi::c_int) {
                need_fileinfo.set(true_0 != 0);
            }
            buf_check_timestamp(curbuf.get());
            (*curwin.get()).w_topline = 1 as ::core::ffi::c_int as linenr_T;
            (*curwin.get()).w_topfill = 0 as ::core::ffi::c_int;
            apply_autocmds(
                EVENT_BUFENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            apply_autocmds(
                EVENT_BUFWINENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
        if (*curwin.get()).w_cursor.lnum == 1 as linenr_T
            && inindent(0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            buflist_getfpos();
        }
        check_arg_idx(curwin.get());
        maketitle();
        if (*curwin.get()).w_topline == 1 as linenr_T && (*curwin.get()).w_topline_was_set == 0 {
            scroll_cursor_halfway(curwin.get(), false_0 != 0, false_0 != 0);
        }
        do_autochdir();
        if (*curbuf.get()).b_kmap_state as ::core::ffi::c_int & KEYMAP_INIT != 0 {
            keymap_init();
        }
        if !(*curbuf.get()).b_help
            && (*curwin.get()).w_onebuf_opt.wo_spell != 0
            && *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int != NUL
        {
            parse_spelllang(curwin.get());
        }
        (*curbuf.get()).b_last_used = time(::core::ptr::null_mut::<time_t>());
        if !(*curbuf.get()).terminal.is_null() {
            terminal_check_size((*curbuf.get()).terminal);
        }
        redraw_later(curwin.get(), UPD_NOT_VALID);
    }
}

pub unsafe extern "C" fn do_autochdir() {
    unsafe {
        if p_acd.get() != 0 {
            if starting.get() == 0 as ::core::ffi::c_int
                && !(*curbuf.get()).b_ffname.is_null()
                && vim_chdirfile((*curbuf.get()).b_ffname, kCdCauseAuto) == OK
            {
                last_chdir_reason.set(c"autochdir".as_ptr() as *mut ::core::ffi::c_char);
                shorten_fnames(true_0);
            }
        }
    }
}

pub unsafe extern "C" fn no_write_message_buf(mut buf: *mut buf_T) {
    unsafe {
        if !(*buf).terminal.is_null()
            && channel_job_running((*buf).b_p_channel as uint64_t) as ::core::ffi::c_int != 0
        {
            emsg(gettext(
                &raw const e_job_still_running_add_bang_to_end_the_job
                    as *const ::core::ffi::c_char,
            ));
        } else {
            semsg_c!(
                gettext(
                    &raw const e_no_write_since_last_change_for_buffer_nr_add_bang_to_override
                        as *const ::core::ffi::c_char,
                ),
                (*buf).handle,
            );
        };
    }
}

pub unsafe extern "C" fn no_write_message() {
    unsafe {
        if !(*curbuf.get()).terminal.is_null()
            && channel_job_running((*curbuf.get()).b_p_channel as uint64_t) as ::core::ffi::c_int
                != 0
        {
            emsg(gettext(
                &raw const e_job_still_running_add_bang_to_end_the_job
                    as *const ::core::ffi::c_char,
            ));
        } else {
            emsg(gettext(
                &raw const e_no_write_since_last_change_add_bang_to_override
                    as *const ::core::ffi::c_char,
            ));
        };
    }
}

pub unsafe extern "C" fn no_write_message_nobang(buf: *const buf_T) {
    unsafe {
        if !(*buf).terminal.is_null()
            && channel_job_running((*buf).b_p_channel as uint64_t) as ::core::ffi::c_int != 0
        {
            emsg(gettext(
                &raw const e_job_still_running as *const ::core::ffi::c_char,
            ));
        } else {
            emsg(gettext(
                &raw const e_no_write_since_last_change as *const ::core::ffi::c_char,
            ));
        };
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn buf_init_changedtick(buf: *mut buf_T) {
    unsafe {
        (*buf).changedtick_di = ChangedtickDictItem {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_FIXED,
                vval: typval_vval_union {
                    v_number: buf_get_changedtick(buf),
                },
            },
            di_flags: (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int)
                as uint8_t,
            di_key: ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(
                *b"changedtick\0",
            ),
        };
        tv_dict_add(
            (*buf).b_vars,
            &raw mut (*buf).changedtick_di as *mut dictitem_T,
        );
    }
}
