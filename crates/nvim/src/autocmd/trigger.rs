//! Everything that fires an event on purpose.
//!
//! [`do_doautocmd`] is `:doautocmd` and [`ex_doautoall`] is `:doautoall`,
//! which runs the event in every loaded buffer.  Below them are the
//! editor's own triggers: the deferred queue
//! ([`aucmd_defer`]/[`deferred_event`], for events that must not fire inside
//! the code that noticed them), [`do_termresponse_autocmd`], the
//! UIEnter/UILeave pair, FocusGained/Lost, VimSuspend/VimResume and
//! FileType.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::buffer::BufRef;
use crate::charset::skip;
use crate::event::multiqueue::{put_job_in, put_job_on_main_loop};
use crate::guard::Depth;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::optionstr::OptString;
use crate::smsg;
use crate::types::TypVal;
use crate::types::{Failed, OptionSetFlags};
use crate::winlayer::Win;
use crate::winlayer::{Buf, first_buffer};

/// `:doautocmd [group] {event} [fname]`, for each of the comma-separated
/// events named.
///
/// `Ok` unless the argument was malformed or an autocommand aborted;
/// `did_something` (when given) says whether any autocommand ran.
pub fn do_doautocmd(
    arg_start: &CStr,
    do_msg: bool,
    mut did_something: Option<&mut bool>,
) -> Result<(), Failed> {
    let arg = arg_start.to_bytes();
    // The byte at `at`, a NUL past the end, as the C walk read it.
    let byte = |at: usize| arg.get(at).copied().unwrap_or(0);
    let mut nothing_done = true;

    if let Some(did) = did_something.as_deref_mut() {
        *did = false;
    }

    // A leading word that is not a group name stays part of the events.
    let (group, mut at) = arg_augroup_get(arg);

    if byte(at) == b'*' {
        emsg(gettext(c"E217: Can't execute autocommands for ALL events"));
        return Err(Failed);
    }

    // Validate every event name before running any of them.
    let Some(events_len) = arg_event_skip(&arg[at..], group != AUGROUP_ALL) else {
        return Err(Failed);
    };
    let mut fname_at = at + events_len;
    fname_at += skip::white(&arg[fname_at..]);
    let fname = CStr::from_bytes_with_nul(&arg_start.to_bytes_with_nul()[fname_at..])
        .expect("the rest of a C string is one");

    while byte(at) != 0
        && ends_excmd(::core::ffi::c_int::from(byte(at))) == 0
        && !ascii_iswhite(::core::ffi::c_int::from(byte(at)))
    {
        let (event, consumed) = event_name2nr(&arg[at..]);
        at += consumed;
        // An unknown name fires nothing, and the walk has already stepped
        // past it.
        let Some(event) = event else {
            continue;
        };
        let ran = apply_autocmds_group(
            event,
            Some(fname),
            None,
            true,
            group,
            Buf::current_or_none(),
            None,
            None,
        );
        if ran {
            nothing_done = false;
        }
    }

    if nothing_done && do_msg && !aborting() {
        let arg_start = msg_bytes(arg_start.to_bytes());
        smsg!(0, "No matching autocommands: {arg_start}");
    }
    if let Some(did) = did_something {
        *did = !nothing_done;
    }

    if aborting() { Err(Failed) } else { Ok(()) }
}

/// `:doautoall`: run the event in every loaded buffer, the current one
/// last.
///
/// Buffers without a window are given one for the duration
/// ([`aucmd_prepbuf`]), because commands expect `curwin->w_buffer ==
/// curbuf`.  An autocommand that deletes the buffer under us stops the
/// sweep, which is what the `bufref` is for.
pub fn ex_doautoall(excmd: &mut ExArg) {
    let mut aco = AcoSave::default();
    let (call_do_modelines, skip) = check_nomodeline(excmd.line.arg());
    let arg = excmd.line.cstr_from(excmd.line.arg + skip).to_owned();
    let arg = arg.as_c_str();
    let mut did_aucmd = false;

    let mut retval = Ok(());
    let mut next = first_buffer();
    while let Some(buf) = next {
        // Loaded buffers only, and the current one is done last. The step
        // is at the bottom, on a buffer `bufref` has just proved this pass
        // did not delete -- which is why this is not `buffers()`.
        if buf.b_ml.ml_mfp.is_some() && buf.raw() != Buf::current_raw() {
            aucmd_prepbuf(&mut aco, buf);
            let bufref = BufRef::of(buf);

            retval = do_doautocmd(arg, false, Some(&mut did_aucmd));

            if call_do_modelines && did_aucmd {
                // Don't set window-local options when the window we are
                // in belongs to another buffer.
                do_modelines(if is_aucmd_win(Win::current()) {
                    OptionSetFlags::NOWIN
                } else {
                    OptionSetFlags::NONE
                });
            }
            aucmd_restbuf(&mut aco);

            // Stop on an error, or if the buffer was deleted under us.
            if retval.is_err() || !bufref.valid() {
                retval = Err(Failed);
                break;
            }
        }
        // `buf` survived this pass, as the `break` above ensures.
        next = buf.next();
    }

    if retval.is_ok() {
        let _ = do_doautocmd(arg, false, Some(&mut did_aucmd));
        if call_do_modelines && did_aucmd {
            do_modelines(OptionSetFlags::NONE);
        }
    }
}

/// Queue `event` to fire at the next event-loop tick, rather than inside
/// the code that noticed it.
///
/// Everything is copied: `fname`, `fname_io` and `data` are the caller's.
/// The *handle* of `buffer` is kept, not the buffer: it may be gone by the
/// time the queued event runs, and [`deferred_event`] looks it up again.
pub fn aucmd_defer(
    event: AutoEvent,
    fname: Option<&CStr>,
    fname_io: Option<&CStr>,
    group: ::core::ffi::c_int,
    buffer: Buf,
    data: Option<&Object>,
) {
    let fname = fname.map(XString::from_cstr);
    let fname_io = fname_io.map(XString::from_cstr);
    let handle: BufferHandle = buffer.handle;
    let data = data.cloned();
    put_job_in(
        &deferred_events,
        Box::new(move || deferred_event(event, fname, fname_io, group, handle, data)),
    );
}

/// Run a queued [`aucmd_defer`] event.
fn deferred_event(
    event: AutoEvent,
    fname: Option<XString>,
    fname_io: Option<XString>,
    group: ::core::ffi::c_int,
    handle: BufferHandle,
    data: Option<Object>,
) {
    let mut err = Error::none();
    // The buffer may well have gone since the event was queued, which is
    // why the *handle* was stored and is resolved here.
    let Some(buf) = find_buffer_by_handle(handle).unwrap_or_default() else {
        return;
    };
    let mut save_v_event = SaveVEvent::default();
    let v_event = get_v_event(&mut save_v_event);
    if let Some(items) = data.as_ref().and_then(Object::as_dict) {
        for item in items {
            let mut tv = TypVal::from(&item.value);
            // A value `v:event` cannot hold is dropped, not fatal.
            if !err.is_set() {
                let _ = v_event.edit().add_tv(item.key.bytes(), &tv);
                tv_clear(&mut tv);
            } else {
                err.clear();
            }
        }
    }
    v_event.edit().set_keys_readonly();

    let mut aco = AcoSave::default();
    aucmd_prepbuf(&mut aco, buf);
    apply_autocmds_group(
        event,
        fname.as_ref().map(XString::as_cstr),
        fname_io.as_ref().map(XString::as_cstr),
        false,
        group,
        Some(buf),
        None,
        data.as_ref(),
    );
    aucmd_restbuf(&mut aco);
    restore_v_event(v_event, &mut save_v_event);
}

/// Fire `TermResponse` with the terminal's reply in `v:event.sequence`.
pub fn do_termresponse_autocmd(sequence: String_0) {
    let mut data = DictBuf::<1>::new();
    let event_data = data.insert(c"sequence", Object::string(sequence)).object();
    apply_autocmds_group(
        AutoEvent::TermResponse,
        None,
        None,
        true,
        AUGROUP_ALL,
        None,
        None,
        Some(&event_data),
    );
    termresponse_changed.set(true);
}

/// Fire `VimSuspend`/`VimResume`, at most once per suspension.
///
/// [`SuspendLatch`] is what makes that true. `VimResume` fires from the
/// event loop, not from the signal handler's caller.
pub fn may_trigger_vim_suspend_resume(suspend: bool) {
    if suspend && pending_vimresume.get() == SuspendLatch::Idle {
        pending_vimresume.set(SuspendLatch::Firing);
        fire_autocmds(AutoEvent::VimSuspend, false, None);
        pending_vimresume.set(SuspendLatch::ResumeOwed);
    } else if !suspend && pending_vimresume.get() == SuspendLatch::ResumeOwed {
        pending_vimresume.set(SuspendLatch::Firing);
        put_job_on_main_loop(Box::new(|| {
            fire_autocmds(AutoEvent::VimResume, false, None);
            pending_vimresume.set(SuspendLatch::Idle);
        }));
    }
}

/// Fire `UIEnter`/`UILeave` for the channel that attached or detached.
pub fn do_autocmd_uienter(chanid: uint64_t, attached: bool) {
    if starting.get() == NO_SCREEN || uienter_busy.get() {
        return;
    }
    uienter_busy.set(true);

    let mut save_v_event = SaveVEvent::default();
    let dict = get_v_event(&mut save_v_event);
    debug_assert!(chanid < VarNumber::MAX as uint64_t);
    let _ = dict.edit().add_number(b"chan", chanid.cast_signed());
    dict.edit().set_keys_readonly();

    fire_autocmds(
        if attached {
            AutoEvent::UIEnter
        } else {
            AutoEvent::UILeave
        },
        false,
        Buf::current_or_none(),
    );
    restore_v_event(dict, &mut save_v_event);

    uienter_busy.set(false);
}

/// Fire `FocusGained`/`FocusLost`, and re-check file timestamps on a gain
/// -- but not more often than every two seconds.
pub fn do_autocmd_focusgained(gained: bool) {
    if focusgained_busy.get() {
        return;
    }
    focusgained_busy.set(true);

    fire_autocmds(
        if gained {
            AutoEvent::FocusGained
        } else {
            AutoEvent::FocusLost
        },
        false,
        Buf::current_or_none(),
    );
    if gained && focusgained_last_time.get().wrapping_add(2000 as Timestamp) < os_now() {
        check_timestamps(1);
        focusgained_last_time.set(os_now());
    }

    focusgained_busy.set(false);
}

/// Fire `FileType` for `buffer`, with `secure` cleared and recursion counted.
///
/// A nested `FileType` only fires when `force` says so; the *inner* one
/// then does not `force` the autocommands themselves, which is what the
/// `ft_recursive == 1` test says.
pub fn do_filetype_autocmd(mut buffer: Buf, force: bool) -> bool {
    if ft_recursive.get() > 0 && !force {
        return false;
    }

    let secure_save = secure.get();
    secure.set(0);
    let ft_recursing = Depth::of(ft_recursive);

    buffer.b_did_filetype = true;
    // Copies: a handler may set 'filetype' or rename the buffer while the
    // names are still the call's.
    let filetype = buffer.b_p_ft.get();
    let shown = buffer.name.shown().map(XString::from_cstr);
    let ret = apply_autocmds(
        AutoEvent::FileType,
        Some(filetype.as_cstr()),
        shown.as_ref().map(XString::as_cstr),
        force || ft_recursive.get() == 1,
        Some(buffer),
    );

    drop(ft_recursing);
    secure.set(secure_save);
    ret
}
