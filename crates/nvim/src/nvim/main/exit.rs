//! Leaving: the orderly path through `VimLeavePre`/`VimLeave`, and the
//! two that skip it.
//!
//! `getout` is the only path that runs autocommands; `os_exit` is what every
//! path funnels into, and `preserve_exit` is for when the process is already
//! too broken to do either properly.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn os_exit(mut r: c_int) -> ! {
    exiting.set(true_0 != 0);
    if ui_client_channel_id.get() != 0 {
        ui_client_stop();
        if r == 0 as c_int {
            r = ui_client_exit_status.get();
        }
    } else {
        ui_flush();
        ui_call_stop();
    }
    if !event_teardown() && r == 0 as c_int {
        r = 1 as c_int;
    }
    if ui_client_channel_id.get() != 0 {
        if stdout_isatty.get() {
            tcdrain(STDOUT_FILENO);
        }
        if stderr_isatty.get() {
            tcdrain(STDERR_FILENO);
        }
    } else {
        ml_close_all(true_0 != 0);
    }
    if used_stdin.get() {
        stream_set_blocking(STDIN_FILENO, true_0 != 0);
    }
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<c_char>(),
        b"os_exit\0".as_ptr() as *const c_char,
        737 as c_int,
        true_0 != 0,
        b"Nvim exit: %d\0".as_ptr() as *const c_char,
        r,
    );
    exit(r);
}

pub unsafe extern "C" fn getout(mut exitval: c_int) -> ! {
    '_c2rust_label: {
        if ui_client_channel_id.get() == 0 {
        } else {
            __assert_fail(
                b"!ui_client_channel_id\0".as_ptr() as *const c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const c_char,
                750 as c_uint,
                b"void getout(int)\0".as_ptr() as *const c_char,
            );
        }
    };
    exiting.set(true_0 != 0);
    time_finish();
    if exmode_active.get() {
        exitval += ex_exitval.get();
    }
    set_vim_var_type(VV_EXITING, VAR_NUMBER);
    set_vim_var_nr(VV_EXITING, exitval as varnumber_T);
    if *get_vim_var_str(VV_EXITREASON) as c_int == NUL {
        set_vim_var_string(
            VV_EXITREASON,
            b"quit\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
    }
    invoke_all_defer();
    if v_dying.get() <= 1 as c_int {
        let mut next_tp: *const tabpage_T = ::core::ptr::null::<tabpage_T>();
        let mut tp: *const tabpage_T = first_tabpage.get();
        while !tp.is_null() {
            next_tp = (*tp).tp_next;
            let mut wp: *mut win_T = if tp == curtab.get() as *const tabpage_T {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if !((*wp).w_buffer.is_null() || !buf_valid((*wp).w_buffer)) {
                    let mut buf: *mut buf_T = (*wp).w_buffer;
                    if buf_get_changedtick(buf) != -1 as varnumber_T {
                        let mut bufref: bufref_T = bufref_T {
                            br_buf: ::core::ptr::null_mut::<buf_T>(),
                            br_fnum: 0,
                            br_buf_free_count: 0,
                        };
                        set_bufref(&raw mut bufref, buf);
                        apply_autocmds(
                            EVENT_BUFWINLEAVE,
                            (*buf).b_fname,
                            (*buf).b_fname,
                            false_0 != 0,
                            buf,
                        );
                        if bufref_valid(&raw mut bufref) {
                            buf_set_changedtick(buf, -1 as varnumber_T);
                        }
                        next_tp = first_tabpage.get();
                        break;
                    }
                }
                wp = (*wp).w_next;
            }
            tp = next_tp;
        }
        let mut buf_0: *mut buf_T = firstbuf.get();
        while !buf_0.is_null() {
            if !(*buf_0).b_ml.ml_mfp.is_null() {
                let mut bufref_0: bufref_T = bufref_T {
                    br_buf: ::core::ptr::null_mut::<buf_T>(),
                    br_fnum: 0,
                    br_buf_free_count: 0,
                };
                set_bufref(&raw mut bufref_0, buf_0);
                apply_autocmds(
                    EVENT_BUFUNLOAD,
                    (*buf_0).b_fname,
                    (*buf_0).b_fname,
                    false_0 != 0,
                    buf_0,
                );
                if !bufref_valid(&raw mut bufref_0) {
                    break;
                }
            }
            buf_0 = (*buf_0).b_next;
        }
        let mut unblock: c_int = 0 as c_int;
        if is_autocmd_blocked() {
            unblock_autocmds();
            unblock += 1;
        }
        apply_autocmds(
            EVENT_VIMLEAVEPRE,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if unblock != 0 {
            block_autocmds();
        }
    }
    if !(*p_shada.ptr()).is_null() && *p_shada.get() as c_int != NUL {
        shada_write_file(::core::ptr::null::<c_char>(), false_0 != 0);
    }
    if v_dying.get() <= 1 as c_int {
        let mut unblock_0: c_int = 0 as c_int;
        if is_autocmd_blocked() {
            unblock_autocmds();
            unblock_0 += 1;
        }
        apply_autocmds(
            EVENT_VIMLEAVE,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if unblock_0 != 0 {
            block_autocmds();
        }
    }
    profile_dump();
    if did_emsg.get() != 0 {
        no_wait_return.set(false_0);
        wait_return(false_0);
    }
    if p_title.get() != 0 && *p_titleold.get() as c_int != NUL {
        ui_call_set_title(cstr_as_string(p_titleold.get()));
    }
    if garbage_collect_at_exit.get() {
        garbage_collect(false_0 != 0);
    }
    os_exit(exitval);
}

pub unsafe extern "C" fn preserve_exit(mut errmsg: *const c_char) -> ! {
    static really_exiting: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if really_exiting.get() {
        if used_stdin.get() {
            stream_set_blocking(STDIN_FILENO, true_0 != 0);
        }
        exit(2 as c_int);
    }
    really_exiting.set(true_0 != 0);
    signal_reject_deadly();
    if ui_client_channel_id.get() != 0 {
        ui_client_stop();
    }
    if !errmsg.is_null() && *errmsg.offset(0 as c_int as isize) as c_int != NUL {
        let mut has_eol: bool = '\n' as c_int
            == *errmsg.offset(strlen(errmsg).wrapping_sub(1 as size_t) as isize) as c_int;
        fprintf(
            stderr,
            if has_eol as c_int != 0 {
                b"%s\0".as_ptr() as *const c_char
            } else {
                b"%s\n\0".as_ptr() as *const c_char
            },
            errmsg,
        );
    }
    if ui_client_channel_id.get() != 0 {
        os_exit(1 as c_int);
    }
    ml_close_notmod();
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(*buf).b_ml.ml_mfp.is_null() && !(*(*buf).b_ml.ml_mfp).mf_fname.is_null() {
            if !errmsg.is_null() {
                fprintf(
                    stderr,
                    b"Nvim: preserving files...\n\0".as_ptr() as *const c_char,
                );
            }
            ml_sync_all(false_0, false_0, true_0 != 0);
            break;
        } else {
            buf = (*buf).b_next;
        }
    }
    ml_close_all(false_0 != 0);
    if !errmsg.is_null() {
        fprintf(stderr, b"Nvim: Finished.\n\0".as_ptr() as *const c_char);
    }
    getout(1 as c_int);
}
