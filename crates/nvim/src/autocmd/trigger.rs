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
use crate::buffer::BufRef;
use crate::message_fmt::c_str;
use crate::smsg;
use crate::types::{FAIL, OK, OptionSetFlags};
use crate::winlayer::{Buf, first_buffer};

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
    let mut arg = arg_start;
    let mut nothing_done = true;

    if !did_something.is_null() {
        // SAFETY: a writable `bool`, by the contract above.
        unsafe { *did_something = false };
    }

    // A leading word that is not a group name stays part of the events.
    // SAFETY: `arg` is the caller's NUL-terminated argument, and
    // `arg_augroup_get` advances it past any group name.
    let group = unsafe { arg_augroup_get(&raw mut arg) };

    // SAFETY: still inside the caller's NUL-terminated argument.
    if unsafe { *arg } == b'*' as ::core::ffi::c_char {
        emsg(gettext(c"E217: Can't execute autocommands for ALL events"));
        return FAIL;
    }

    // Validate every event name before running any of them.
    // SAFETY: as above.
    let fname = unsafe { arg_event_skip(arg, group != AUGROUP_ALL) };
    if fname.is_null() {
        return FAIL;
    }
    // SAFETY: `arg_event_skip` answered a pointer into the same string.
    let fname = unsafe { skipwhite(fname) };

    // SAFETY: `arg` stays inside the caller's NUL-terminated argument, and
    // `event_name2nr` is what advances it to the next event.
    while unsafe { *arg } != 0
        && unsafe { ends_excmd(*arg as ::core::ffi::c_int) } == 0
        && !unsafe { ascii_iswhite(*arg as ::core::ffi::c_int) }
    {
        // SAFETY: the event name at `arg`, the file name beside it, and
        // `curbuf`, which is live from startup to exit.
        let ran = unsafe {
            apply_autocmds_group(
                event_name2nr(arg, &raw mut arg),
                fname,
                ::core::ptr::null_mut(),
                true,
                group,
                curbuf.get(),
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
            )
        };
        if ran {
            nothing_done = false;
        }
    }

    if nothing_done && do_msg && !aborting() {
        // SAFETY: a NUL-terminated format literal and the caller's argument.
        let arg_start = unsafe { c_str(arg_start) };
        smsg!(0, "No matching autocommands: {arg_start}");
    }
    if !did_something.is_null() {
        // SAFETY: a writable `bool`, by the contract above.
        unsafe { *did_something = !nothing_done };
    }

    if aborting() { FAIL } else { OK }
}

/// `:doautoall`: run the event in every loaded buffer, the current one
/// last.
///
/// Buffers without a window are given one for the duration
/// ([`aucmd_prepbuf`]), because commands expect `curwin->w_buffer ==
/// curbuf`.  An autocommand that deletes the buffer under us stops the
/// sweep, which is what the `bufref` is for.
pub unsafe fn ex_doautoall(eap: *mut exarg_T) {
    let mut aco = aco_save_T::default();
    // SAFETY: a live command block, by the contract above, and
    // `check_nomodeline` only advances `arg` inside its own argument.
    let mut arg = unsafe { (*eap).arg };
    let call_do_modelines = unsafe { check_nomodeline(&raw mut arg) };
    let mut did_aucmd = false;

    let mut retval = OK;
    let mut next = first_buffer();
    while let Some(mut buf) = next {
        // Loaded buffers only, and the current one is done last. The step
        // is at the bottom, on a buffer `bufref` has just proved this pass
        // did not delete -- which is why this is not `buffers()`.
        if !buf.b_ml.ml_mfp.is_null() && buf.raw() != curbuf.get() {
            // SAFETY: `aco` is this frame's own storage and `buf` is live.
            unsafe { aucmd_prepbuf(&raw mut aco, buf.raw()) };
            let bufref = BufRef::of(buf);

            // SAFETY: `arg` is the command's own argument and `did_aucmd`
            // this frame's own `bool`.
            retval = unsafe { do_doautocmd(arg, false, &raw mut did_aucmd) };

            if call_do_modelines && did_aucmd {
                // Don't set window-local options when the window we are
                // in belongs to another buffer.
                do_modelines(if is_aucmd_win(curwin.get()) {
                    OptionSetFlags::NOWIN
                } else {
                    OptionSetFlags::NONE
                });
            }
            // SAFETY: the `aucmd_prepbuf` above opened this pair.
            unsafe { aucmd_restbuf(&raw mut aco) };

            // Stop on an error, or if the buffer was deleted under us.
            if retval == FAIL || !bufref.valid() {
                retval = FAIL;
                break;
            }
        }
        // `buf` survived this pass, as the `break` above ensures.
        next = buf.next();
    }

    if retval == OK {
        // SAFETY: as above.
        unsafe { do_doautocmd(arg, false, &raw mut did_aucmd) };
        if call_do_modelines && did_aucmd {
            do_modelines(OptionSetFlags::NONE);
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
    buf: Buf,
    eap: *mut exarg_T,
    data: *mut Object,
) {
    // SAFETY: `fname`/`fname_io` are the caller's NUL-terminated names or
    // NULL, so each is either copied or passed through as NULL.
    let dup = |s: *mut ::core::ffi::c_char| {
        if s.is_null() {
            ::core::ptr::null_mut()
        } else {
            unsafe { xstrdup(s) }
        }
    };

    // SAFETY: a fresh allocation, every field written before anything reads
    // it, and handed straight to the queue that owns it from here on.
    let evdata = unsafe { xmalloc(::core::mem::size_of::<AutoCmdEvent>()) }.cast::<AutoCmdEvent>();
    unsafe { (*evdata).event = event };
    unsafe { (*evdata).fname = dup(fname) };
    unsafe { (*evdata).fname_io = dup(fname_io) };
    unsafe { (*evdata).group = group };
    // The *handle* is stored, not the pointer: the buffer may be gone by the
    // time the queued event runs, and `deferred_event` looks it up again.
    unsafe { (*evdata).buf = buf.handle as Buffer };
    unsafe { (*evdata).eap = eap };
    // SAFETY: `data` is the caller's object or NULL; the copy is owned by
    // the event from here on.
    unsafe {
        (*evdata).data = if data.is_null() {
            ::core::ptr::null_mut()
        } else {
            let copy = xmalloc(::core::mem::size_of::<Object>()).cast::<Object>();
            *copy = copy_object(*data, ::core::ptr::null_mut());
            copy
        }
    };

    let mut argv = NO_ARGV;
    argv[0] = evdata.cast::<::core::ffi::c_void>();
    // SAFETY: the queue is the editor's own and takes ownership of `evdata`.
    unsafe {
        multiqueue_put_event(
            deferred_events.get(),
            Event {
                handler: Some(deferred_event),
                argv,
            },
        )
    };
}

/// Run a queued [`aucmd_defer`] event, and free everything it copied.
unsafe extern "C" fn deferred_event(argv: *mut *mut ::core::ffi::c_void) {
    // SAFETY: the event `aucmd_defer` queued, whose payload it owns until
    // this call frees it below.
    let e = unsafe { *argv }.cast::<AutoCmdEvent>();
    let event = unsafe { (*e).event };
    let fname = unsafe { (*e).fname };
    let fname_io = unsafe { (*e).fname_io };
    let group = unsafe { (*e).group };
    let eap = unsafe { (*e).eap };
    let data = unsafe { (*e).data };

    let mut err = Error::none();
    // The buffer may well have gone since the event was queued, which is
    // why the *handle* was stored and is resolved here.
    // SAFETY: `err` is this frame's own.
    let buf = unsafe { find_buffer_by_handle((*e).buf, &mut err) };
    if !buf.is_null() {
        let mut save_v_event = save_v_event_T::default();
        // SAFETY: `save_v_event` is this frame's own storage, and the
        // dictionary is `v:event`, live until `restore_v_event` below.
        let v_event = unsafe { get_v_event(&raw mut save_v_event) };
        if !data.is_null() && unsafe { (*data).type_0 } == kObjectTypeDict {
            // SAFETY: the tag above says the dict arm of the union is live,
            // and `size` is that dict's own item count.
            let items = unsafe { (*data).data.dict };
            for i in 0..items.size {
                let item = unsafe { *items.items.add(i) };
                let mut tv = TV_INITIAL_VALUE;
                // SAFETY: `tv` and `err` are this frame's own.
                unsafe { object_to_vim(item.value, &raw mut tv) };
                // A value `v:event` cannot hold is dropped, not fatal.
                if !err.is_set() {
                    // SAFETY: `v_event` is that dictionary and `item.key` is
                    // the dict entry's own name of the length given.
                    unsafe {
                        tv_dict_add_tv(v_event, item.key.data(), item.key.len(), &raw mut tv)
                    };
                    unsafe { tv_clear(&raw mut tv) };
                } else {
                    err.clear();
                }
            }
        }
        // SAFETY: `v_event` is that dictionary.
        unsafe { tv_dict_set_keys_readonly(v_event) };

        let mut aco = aco_save_T::default();
        // SAFETY: `aco` is this frame's own, `buf` was just proved live, and
        // the `prepbuf`/`restbuf` pair brackets the firing.
        unsafe { aucmd_prepbuf(&raw mut aco, buf) };
        unsafe { apply_autocmds_group(event, fname, fname_io, false, group, buf, eap, data) };
        unsafe { aucmd_restbuf(&raw mut aco) };
        // SAFETY: the pair `get_v_event` above opened.
        unsafe { restore_v_event(v_event, &raw mut save_v_event) };
    }

    // SAFETY: everything `aucmd_defer` copied, owned by this event alone.
    unsafe { xfree(fname.cast::<::core::ffi::c_void>()) };
    unsafe { xfree(fname_io.cast::<::core::ffi::c_void>()) };
    if !data.is_null() {
        unsafe { api_free_object(*data) };
        unsafe { xfree(data.cast::<::core::ffi::c_void>()) };
    }
    unsafe { xfree(e.cast::<::core::ffi::c_void>()) };
}

/// Fire `TermResponse` with the terminal's reply in `v:event.sequence`.
pub unsafe fn do_termresponse_autocmd(sequence: String_0) {
    let mut data = DictBuf::<1>::new();
    let mut event_data = data.insert(c"sequence", Object::string(sequence)).object();
    // SAFETY: no file name and no buffer; `event_data` is this frame's own
    // and outlives the call.
    unsafe {
        apply_autocmds_group(
            EVENT_TERMRESPONSE,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            true,
            AUGROUP_ALL,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            &raw mut event_data,
        )
    };
    termresponse_changed.set(true);
}

/// The queued half of [`may_trigger_vim_suspend_resume`]: `VimResume` has
/// to fire from the event loop, not from the signal handler's caller.
unsafe extern "C" fn vimresume_event(_argv: *mut *mut ::core::ffi::c_void) {
    // SAFETY: no file name and no buffer, so there is nothing for the event
    // to read but the editor's own autocommand tables.
    unsafe {
        apply_autocmds(
            EVENT_VIMRESUME,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            ::core::ptr::null_mut(),
        )
    };
    pending_vimresume.set(SuspendLatch::Idle);
}

/// Fire `VimSuspend`/`VimResume`, at most once per suspension.
///
/// [`SuspendLatch`] is what makes that true.
pub fn may_trigger_vim_suspend_resume(suspend: bool) {
    if suspend && pending_vimresume.get() == SuspendLatch::Idle {
        pending_vimresume.set(SuspendLatch::Firing);
        // SAFETY: no file name and no buffer, so there is nothing for the
        // event to read but the editor's own autocommand tables.
        unsafe {
            apply_autocmds(
                EVENT_VIMSUSPEND,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                false,
                ::core::ptr::null_mut(),
            )
        };
        pending_vimresume.set(SuspendLatch::ResumeOwed);
    } else if !suspend && pending_vimresume.get() == SuspendLatch::ResumeOwed {
        pending_vimresume.set(SuspendLatch::Firing);
        // SAFETY: `main_loop` is the editor's own loop, live from startup to
        // exit, and the event carries no arguments to outlive it.
        unsafe {
            multiqueue_put_event(
                (*main_loop.ptr()).events,
                Event {
                    handler: Some(vimresume_event),
                    argv: NO_ARGV,
                },
            )
        };
    }
}

/// Fire `UIEnter`/`UILeave` for the channel that attached or detached.
pub fn do_autocmd_uienter(chanid: uint64_t, attached: bool) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false);

    if starting.get() == NO_SCREEN || recursive.get() {
        return;
    }
    recursive.set(true);

    let mut save_v_event = save_v_event_T::default();
    // SAFETY: `save_v_event` is this frame's own storage, and the dictionary
    // `get_v_event` hands back is `v:event`, live until `restore_v_event`.
    let dict = unsafe { get_v_event(&raw mut save_v_event) };
    debug_assert!(chanid < varnumber_T::MAX as uint64_t);
    // SAFETY: `dict` is that dictionary and the key is a NUL-terminated
    // literal of the length given.
    unsafe { tv_dict_add_nr(dict, c"chan".as_ptr(), 4, chanid as varnumber_T) };
    // SAFETY: as above.
    unsafe { tv_dict_set_keys_readonly(dict) };

    // SAFETY: no file name, and `curbuf` is live from startup to exit.
    unsafe {
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
        )
    };
    // SAFETY: the pair `get_v_event` above opened.
    unsafe { restore_v_event(dict, &raw mut save_v_event) };

    recursive.set(false);
}

/// Fire `FocusGained`/`FocusLost`, and re-check file timestamps on a gain
/// -- but not more often than every two seconds.
pub fn do_autocmd_focusgained(gained: bool) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false);
    static last_time: GlobalCell<Timestamp> = GlobalCell::new(0 as Timestamp);

    if recursive.get() {
        return;
    }
    recursive.set(true);

    // SAFETY: no file name, and `curbuf` is live from startup to exit.
    unsafe {
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
        )
    };
    if gained && last_time.get().wrapping_add(2000 as Timestamp) < os_now() {
        // SAFETY: re-checks the buffer list's timestamps; nothing here holds
        // a borrow of editor state across it.
        unsafe { check_timestamps(1) };
        last_time.set(os_now());
    }

    recursive.set(false);
}

/// Fire `FileType` for `buf`, with `secure` cleared and recursion counted.
///
/// A nested `FileType` only fires when `force` says so; the *inner* one
/// then does not `force` the autocommands themselves, which is what the
/// `ft_recursive == 1` test says.
pub fn do_filetype_autocmd(mut buf: Buf, force: bool) -> bool {
    static ft_recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

    if ft_recursive.get() > 0 && !force {
        return false;
    }

    let secure_save = secure.get();
    secure.set(0);
    ft_recursive.set(ft_recursive.get() + 1);

    buf.b_did_filetype = true;
    // SAFETY: `b_p_ft` and `b_fname` are that buffer's own NUL-terminated
    // names, and the buffer is live by `Buf`'s contract.
    let ret = unsafe {
        apply_autocmds(
            EVENT_FILETYPE,
            buf.b_p_ft,
            buf.b_fname,
            force || ft_recursive.get() == 1,
            buf.raw(),
        )
    };

    ft_recursive.set(ft_recursive.get() - 1);
    secure.set(secure_save);
    ret
}
