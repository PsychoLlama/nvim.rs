//! Everything that fires an event on purpose.
//!
//! [`do_doautocmd`] is `:doautocmd` and [`ex_doautoall`] is `:doautoall`,
//! which runs the event in every loaded buffer.  Below them are the
//! editor's own triggers: the deferred queue
//! ([`aucmd_defer`]/`deferred_event`, for events that must not fire inside
//! the code that noticed them), [`do_termresponse_autocmd`], the
//! UIEnter/UILeave pair, FocusGained/Lost, VimSuspend/VimResume and
//! FileType.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::smsg_c;
use crate::types::{FAIL, OK, OptionSetFlags};

/// A `multiqueue` event's argument vector with nothing in it.
const NO_ARGV: [*mut ::core::ffi::c_void; 10] = [::core::ptr::null_mut(); 10];

/// `:doautocmd [group] {event} [fname]`, for each of the comma-separated
/// events named.
///
/// `OK` unless the argument was malformed or an autocommand aborted;
/// `did_something` (when given) says whether any autocommand ran.
pub unsafe fn do_doautocmd(
    arg_start: *mut ::core::ffi::c_char,
    do_msg: bool,
    did_something: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut arg = arg_start;
        let mut nothing_done = true;

        if !did_something.is_null() {
            *did_something = false;
        }

        // A leading word that is not a group name stays part of the events.
        let group = arg_augroup_get(&raw mut arg);

        if *arg == b'*' as ::core::ffi::c_char {
            emsg(gettext(
                c"E217: Can't execute autocommands for ALL events".as_ptr(),
            ));
            return FAIL;
        }

        // Validate every event name before running any of them.
        let fname = arg_event_skip(arg, group != AUGROUP_ALL);
        if fname.is_null() {
            return FAIL;
        }
        let fname = skipwhite(fname);

        while *arg != 0
            && ends_excmd(*arg as ::core::ffi::c_int) == 0
            && !ascii_iswhite(*arg as ::core::ffi::c_int)
        {
            // `event_name2nr` is what advances `arg` to the next event.
            if apply_autocmds_group(
                event_name2nr(arg, &raw mut arg),
                fname,
                ::core::ptr::null_mut(),
                true,
                group,
                curbuf.get(),
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
            ) {
                nothing_done = false;
            }
        }

        if nothing_done && do_msg && !aborting() {
            smsg_c!(
                0,
                gettext(c"No matching autocommands: %s".as_ptr()),
                arg_start,
            );
        }
        if !did_something.is_null() {
            *did_something = !nothing_done;
        }

        if aborting() { FAIL } else { OK }
    }
}

/// `:doautoall`: run the event in every loaded buffer, the current one
/// last.
///
/// Buffers without a window are given one for the duration
/// ([`aucmd_prepbuf`]), because commands expect `curwin->w_buffer ==
/// curbuf`.  An autocommand that deletes the buffer under us stops the
/// sweep, which is what the `bufref` is for.
pub unsafe fn ex_doautoall(eap: *mut exarg_T) {
    unsafe {
        let mut aco = aco_save_T::default();
        let mut arg = (*eap).arg;
        let call_do_modelines = check_nomodeline(&raw mut arg);
        let mut bufref = bufref_T::default();
        let mut did_aucmd = false;

        let mut retval = OK;
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            // Loaded buffers only, and the current one is done last.
            if !(*buf).b_ml.ml_mfp.is_null() && buf != curbuf.get() {
                aucmd_prepbuf(&raw mut aco, buf);
                set_bufref(&raw mut bufref, buf);

                retval = do_doautocmd(arg, false, &raw mut did_aucmd);

                if call_do_modelines && did_aucmd {
                    // Don't set window-local options when the window we are
                    // in belongs to another buffer.
                    do_modelines(if is_aucmd_win(curwin.get()) {
                        OptionSetFlags::NOWIN
                    } else {
                        OptionSetFlags::NONE
                    });
                }
                aucmd_restbuf(&raw mut aco);

                // Stop on an error, or if the buffer was deleted under us.
                if retval == FAIL || !bufref_valid(&raw mut bufref) {
                    retval = FAIL;
                    break;
                }
            }
            buf = (*buf).b_next;
        }

        if retval == OK {
            do_doautocmd(arg, false, &raw mut did_aucmd);
            if call_do_modelines && did_aucmd {
                do_modelines(OptionSetFlags::NONE);
            }
        }
    }
}

/// Queue `event` to fire at the next event-loop tick, rather than inside
/// the code that noticed it.
///
/// Everything is copied: `fname`, `fname_io` and `data` are the caller's.
pub unsafe fn aucmd_defer(
    event: event_T,
    fname: *mut ::core::ffi::c_char,
    fname_io: *mut ::core::ffi::c_char,
    group: ::core::ffi::c_int,
    buf: *mut buf_T,
    eap: *mut exarg_T,
    data: *mut Object,
) {
    unsafe {
        let dup = |s: *mut ::core::ffi::c_char| {
            if s.is_null() {
                ::core::ptr::null_mut()
            } else {
                xstrdup(s)
            }
        };

        let evdata = xmalloc(::core::mem::size_of::<AutoCmdEvent>()).cast::<AutoCmdEvent>();
        (*evdata).event = event;
        (*evdata).fname = dup(fname);
        (*evdata).fname_io = dup(fname_io);
        (*evdata).group = group;
        (*evdata).buf = (*buf).handle as Buffer;
        (*evdata).eap = eap;
        (*evdata).data = if data.is_null() {
            ::core::ptr::null_mut()
        } else {
            let copy = xmalloc(::core::mem::size_of::<Object>()).cast::<Object>();
            *copy = copy_object(*data, ::core::ptr::null_mut());
            copy
        };

        let mut argv = NO_ARGV;
        argv[0] = evdata.cast::<::core::ffi::c_void>();
        multiqueue_put_event(
            deferred_events.get(),
            Event {
                handler: Some(deferred_event),
                argv,
            },
        );
    }
}

/// Run a queued [`aucmd_defer`] event, and free everything it copied.
unsafe extern "C" fn deferred_event(argv: *mut *mut ::core::ffi::c_void) {
    unsafe {
        let e = (*argv).cast::<AutoCmdEvent>();
        let event = (*e).event;
        let fname = (*e).fname;
        let fname_io = (*e).fname_io;
        let group = (*e).group;
        let eap = (*e).eap;
        let data = (*e).data;

        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut(),
        };
        // The buffer may well have gone since the event was queued.
        let buf = find_buffer_by_handle((*e).buf, &raw mut err);
        if !buf.is_null() {
            let mut save_v_event = save_v_event_T::default();
            let v_event = get_v_event(&raw mut save_v_event);
            if !data.is_null() && (*data).type_0 == kObjectTypeDict {
                let items = (*data).data.dict;
                for i in 0..items.size {
                    let item = *items.items.add(i);
                    let mut tv = TV_INITIAL_VALUE;
                    object_to_vim(item.value, &raw mut tv, &raw mut err);
                    // A value `v:event` cannot hold is dropped, not fatal.
                    if err.type_0 == kErrorTypeNone {
                        tv_dict_add_tv(v_event, item.key.data, item.key.size, &raw mut tv);
                        tv_clear(&raw mut tv);
                    } else {
                        api_clear_error(&raw mut err);
                    }
                }
            }
            tv_dict_set_keys_readonly(v_event);

            let mut aco = aco_save_T::default();
            aucmd_prepbuf(&raw mut aco, buf);
            apply_autocmds_group(event, fname, fname_io, false, group, buf, eap, data);
            aucmd_restbuf(&raw mut aco);
            restore_v_event(v_event, &raw mut save_v_event);
        }

        xfree(fname.cast::<::core::ffi::c_void>());
        xfree(fname_io.cast::<::core::ffi::c_void>());
        if !data.is_null() {
            api_free_object(*data);
            xfree(data.cast::<::core::ffi::c_void>());
        }
        xfree(e.cast::<::core::ffi::c_void>());
    }
}

/// Fire `TermResponse` with the terminal's reply in `v:event.sequence`.
pub unsafe fn do_termresponse_autocmd(sequence: String_0) {
    unsafe {
        let mut data = DictBuf::<1>::new();
        let mut event_data = data.insert(c"sequence", Object::string(sequence)).object();
        apply_autocmds_group(
            EVENT_TERMRESPONSE,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            true,
            AUGROUP_ALL,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            &raw mut event_data,
        );
        termresponse_changed.set(true);
    }
}

/// The queued half of [`may_trigger_vim_suspend_resume`]: `VimResume` has
/// to fire from the event loop, not from the signal handler's caller.
unsafe extern "C" fn vimresume_event(_argv: *mut *mut ::core::ffi::c_void) {
    unsafe {
        apply_autocmds(
            EVENT_VIMRESUME,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            ::core::ptr::null_mut(),
        );
        pending_vimresume.set(SuspendLatch::Idle);
    }
}

/// Fire `VimSuspend`/`VimResume`, at most once per suspension.
///
/// [`SuspendLatch`] is what makes that true.
pub unsafe fn may_trigger_vim_suspend_resume(suspend: bool) {
    unsafe {
        if suspend && pending_vimresume.get() == SuspendLatch::Idle {
            pending_vimresume.set(SuspendLatch::Firing);
            apply_autocmds(
                EVENT_VIMSUSPEND,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                false,
                ::core::ptr::null_mut(),
            );
            pending_vimresume.set(SuspendLatch::ResumeOwed);
        } else if !suspend && pending_vimresume.get() == SuspendLatch::ResumeOwed {
            pending_vimresume.set(SuspendLatch::Firing);
            multiqueue_put_event(
                (*main_loop.ptr()).events,
                Event {
                    handler: Some(vimresume_event),
                    argv: NO_ARGV,
                },
            );
        }
    }
}

/// Fire `UIEnter`/`UILeave` for the channel that attached or detached.
pub unsafe fn do_autocmd_uienter(chanid: uint64_t, attached: bool) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false);

        if starting.get() == NO_SCREEN || recursive.get() {
            return;
        }
        recursive.set(true);

        let mut save_v_event = save_v_event_T::default();
        let dict = get_v_event(&raw mut save_v_event);
        debug_assert!(chanid < varnumber_T::MAX as uint64_t);
        tv_dict_add_nr(dict, c"chan".as_ptr(), 4, chanid as varnumber_T);
        tv_dict_set_keys_readonly(dict);

        apply_autocmds(
            if attached {
                EVENT_UIENTER
            } else {
                EVENT_UILEAVE
            },
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            curbuf.get(),
        );
        restore_v_event(dict, &raw mut save_v_event);

        recursive.set(false);
    }
}

/// Fire `FocusGained`/`FocusLost`, and re-check file timestamps on a gain
/// -- but not more often than every two seconds.
pub unsafe fn do_autocmd_focusgained(gained: bool) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false);
        static last_time: GlobalCell<Timestamp> = GlobalCell::new(0 as Timestamp);

        if recursive.get() {
            return;
        }
        recursive.set(true);

        apply_autocmds(
            if gained {
                EVENT_FOCUSGAINED
            } else {
                EVENT_FOCUSLOST
            },
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            curbuf.get(),
        );
        if gained && last_time.get().wrapping_add(2000 as Timestamp) < os_now() {
            check_timestamps(1);
            last_time.set(os_now());
        }

        recursive.set(false);
    }
}

/// Fire `FileType` for `buf`, with `secure` cleared and recursion counted.
///
/// A nested `FileType` only fires when `force` says so; the *inner* one
/// then does not `force` the autocommands themselves, which is what the
/// `ft_recursive == 1` test says.
pub unsafe fn do_filetype_autocmd(buf: *mut buf_T, force: bool) -> bool {
    unsafe {
        static ft_recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

        if ft_recursive.get() > 0 && !force {
            return false;
        }

        let secure_save = secure.get();
        secure.set(0);
        *ft_recursive.ptr() += 1;

        (*buf).b_did_filetype = true;
        let ret = apply_autocmds(
            EVENT_FILETYPE,
            (*buf).b_p_ft,
            (*buf).b_fname,
            force || ft_recursive.get() == 1,
            buf,
        );

        *ft_recursive.ptr() -= 1;
        secure.set(secure_save);
        ret
    }
}
