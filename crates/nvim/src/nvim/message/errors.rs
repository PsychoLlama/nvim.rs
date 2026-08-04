//! The `emsg` family: errors, warnings and where they came from.
//!
//! [`emsg_multiline`] is the funnel -- it consults `'debug'`, the `:try`
//! stack and `v:errmsg` before anything is displayed -- and
//! [`get_emsg_source`] is what prefixes the message with the script and line
//! that raised it.
//!
//! The `s`-prefixed entry points ([`semsg`], [`siemsg`], [`swmsg`],
//! [`msg_schedule_semsg`] and friends) stay C variadics. They are called
//! ~650 times across the tree as `printf`-style forwarders; turning them into
//! Rust macros is a tree-wide change, not a rewrite of this file.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use core::ffi::{VaList, c_char, c_int, c_long, c_void};
use core::ptr;

/// The script/function the last error was reported from.
static last_sourcing_name: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// The line the last error was reported from.
static last_sourcing_lnum: GlobalCell<c_int> = GlobalCell::new(0);

/// Forget where the last error came from, so the next one names its source
/// again.
///
/// # Safety
/// Only that nothing else holds the stored name.
pub unsafe fn reset_last_sourcing() {
    unsafe {
        xfree(last_sourcing_name.get().cast());
        last_sourcing_name.set(ptr::null_mut());
        last_sourcing_lnum.set(0);
    }
}

/// Is the innermost script/function a different one from the last error's?
///
/// # Safety
/// Only that the exec stack is well formed.
unsafe fn other_sourcing_name() -> bool {
    unsafe {
        if exestack_has_name() {
            if !last_sourcing_name.get().is_null() {
                return strcmp((*sourcing_top()).es_name, last_sourcing_name.get()) != 0;
            }
            return true;
        }
        false
    }
}

/// Is there an innermost exec-stack entry, and does it name a source?
///
/// # Safety
/// Only that the exec stack is well formed.
unsafe fn exestack_has_name() -> bool {
    unsafe {
        !(*exestack.ptr()).ga_data.is_null()
            && (*exestack.ptr()).ga_len > 0
            && !(*sourcing_top()).es_name.is_null()
    }
}

/// An allocated "Error in <script>:" line, or null when the source has
/// already been reported.
///
/// # Safety
/// Only that the exec stack is well formed.
unsafe fn get_emsg_source() -> *mut c_char {
    unsafe {
        if !exestack_has_name() || !other_sourcing_name() {
            return ptr::null_mut();
        }
        let tofree = estack_sfile(ESTACK_NONE);
        let sname = if tofree.is_null() {
            (*sourcing_top()).es_name
        } else {
            tofree
        };
        let p = gettext(c"Error in %s:".as_ptr());
        let buf_len = strlen(sname) + strlen(p) + 1;
        let buf: *mut c_char = xmalloc(buf_len).cast();
        snprintf(buf, buf_len, p, sname);
        xfree(tofree.cast());
        buf
    }
}

/// An allocated "line NNNN:" line, or null when the line has already been
/// reported (or there is none).
///
/// # Safety
/// Only that the exec stack is well formed.
unsafe fn get_emsg_lnum() -> *mut c_char {
    unsafe {
        // Show the source of the error, but not if it is the same as the last
        // time.
        if (*sourcing_top()).es_name.is_null()
            || !(other_sourcing_name() || (*sourcing_top()).es_lnum != last_sourcing_lnum.get())
            || (*sourcing_top()).es_lnum == 0
        {
            return ptr::null_mut();
        }
        let p = gettext(c"line %4d:".as_ptr());
        let buf_len = 20 + strlen(p);
        let buf: *mut c_char = xmalloc(buf_len).cast();
        snprintf(buf, buf_len, p, (*sourcing_top()).es_lnum);
        buf
    }
}

/// Display the source of an error message, if it has not been shown already.
///
/// # Safety
/// Only that the exec stack is well formed.
pub unsafe fn msg_source(hl_id: c_int) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false);
        if recursive.get() {
            return;
        }
        recursive.set(true);

        no_wait_return.set(no_wait_return.get() + 1);
        let p = get_emsg_source();
        if !p.is_null() {
            msg_scroll.set(true_0);
            msg(p, hl_id);
            xfree(p.cast());
        }
        let p = get_emsg_lnum();
        if !p.is_null() {
            msg(p, HLF_N);
            xfree(p.cast());
            last_sourcing_lnum.set((*sourcing_top()).es_lnum as c_int);
        }

        // Remember the source name and line number, so we can tell when
        // the message changes.
        if (*sourcing_top()).es_name.is_null() || other_sourcing_name() {
            xfree(last_sourcing_name.get().cast());
            last_sourcing_name.set(ptr::null_mut());
            if !(*sourcing_top()).es_name.is_null() {
                last_sourcing_name.set(xstrdup((*sourcing_top()).es_name));
                if !redirecting() {
                    msg_putchar_hl(b'\n' as c_int, hl_id);
                }
            }
        }
        no_wait_return.set(no_wait_return.get() - 1);
        recursive.set(false);
    }
}

/// Is this a bad time to show an error?
///
/// # Safety
/// Only that `'debug'` holds a valid string.
pub(crate) unsafe fn emsg_not_now() -> bool {
    unsafe {
        (emsg_off.get() > 0
            && vim_strchr(p_debug.get(), b'm' as c_int).is_null()
            && vim_strchr(p_debug.get(), b't' as c_int).is_null())
            || emsg_skip.get() > 0
    }
}

/// Show an error message, possibly spanning several lines.
///
/// Answers true when the message was shown (or deliberately swallowed).
/// `kind` is the `ext_messages` kind; `multiline` keeps embedded newlines
/// rather than escaping them.
///
/// # Safety
/// `s` must be a valid C string; `kind` may be null.
pub unsafe fn emsg_multiline(
    s: *const c_char,
    kind: *const c_char,
    hl_id: c_int,
    multiline: bool,
) -> bool {
    unsafe {
        if emsg_not_now() {
            return true;
        }
        called_emsg.set(called_emsg.get() + 1);

        // Reset the "severe" flag: it applies to this message only.
        let severe = emsg_severe.get();
        emsg_severe.set(false);

        if emsg_off.get() == 0 || !vim_strchr(p_debug.get(), b't' as c_int).is_null() {
            // Cause a throw of an error exception if appropriate. Don't display
            // the error message in this case.
            let mut ignore = false;
            if cause_errthrow(s, multiline, is_multihl.get() > 1, severe, &raw mut ignore) {
                if !ignore {
                    did_emsg.set(did_emsg.get() + 1);
                }
                return true;
            }

            if in_assert_fails.get() && emsg_assert_fails_msg.get().is_null() {
                emsg_assert_fails_msg.set(xstrdup(s));
                emsg_assert_fails_lnum.set((*sourcing_top()).es_lnum as c_long);
                xfree(emsg_assert_fails_context.get().cast());
                emsg_assert_fails_context.set(xstrdup(if (*sourcing_top()).es_name.is_null() {
                    c"".as_ptr()
                } else {
                    (*sourcing_top()).es_name
                }));
            }

            // set "v:errmsg", also when using ":silent! cmd"
            set_vim_var_string(VV_ERRMSG, s, -1);

            // When using ":silent! cmd" don't display the error message, but
            // do write it to the redirection and the log.
            if emsg_silent.get() != 0 {
                if !emsg_noredir.get() {
                    msg_start();
                    // Each source line is redirected with a newline appended.
                    // Both helpers size their buffer for one byte more than
                    // the text, so the terminator's slot takes it.
                    //
                    // Called one at a time rather than collected first: the
                    // redirection in between can reach `:redir => var`, and
                    // `get_emsg_lnum`'s answer depends on state a redirection
                    // could in principle move.
                    let write_line = |line: *mut c_char| {
                        if !line.is_null() {
                            let len = strlen(line);
                            *line.add(len) = b'\n' as c_char;
                            redir_write(line, len as ptrdiff_t + 1);
                            xfree(line.cast());
                        }
                    };
                    write_line(get_emsg_source());
                    write_line(get_emsg_lnum());
                    redir_write(s, strlen(s) as ptrdiff_t);
                }
                if !(*sourcing_top()).es_name.is_null() && (*sourcing_top()).es_lnum != 0 {
                    logmsg(
                        LOGLVL_DBG,
                        ptr::null(),
                        c"emsg_multiline".as_ptr(),
                        845,
                        true,
                        c"(:silent) %s (%s (line %d))".as_ptr(),
                        s,
                        (*sourcing_top()).es_name,
                        (*sourcing_top()).es_lnum,
                    );
                } else {
                    logmsg(
                        LOGLVL_DBG,
                        ptr::null(),
                        c"emsg_multiline".as_ptr(),
                        847,
                        true,
                        c"(:silent) %s".as_ptr(),
                        s,
                    );
                }
                return true;
            }

            if !(*sourcing_top()).es_name.is_null() && (*sourcing_top()).es_lnum != 0 {
                logmsg(
                    LOGLVL_INF,
                    ptr::null(),
                    c"emsg_multiline".as_ptr(),
                    855,
                    true,
                    c"%s (%s (line %d))".as_ptr(),
                    s,
                    (*sourcing_top()).es_name,
                    (*sourcing_top()).es_lnum,
                );
            } else {
                logmsg(
                    LOGLVL_INF,
                    ptr::null(),
                    c"emsg_multiline".as_ptr(),
                    857,
                    true,
                    c"%s".as_ptr(),
                    s,
                );
            }

            ex_exitval.set(1);

            // Reset msg_silent, an error causes messages to be visible again.
            msg_silent.set(0);
            cmd_silent.set(false);

            if global_busy.get() != 0 {
                // Break out of the :global command.
                global_busy.set(global_busy.get() + 1);
            }

            // Now that we have a message, flush the input buffer or beep.
            if p_eb.get() != 0 {
                beep_flush();
            } else {
                flush_buffers(FLUSH_MINIMAL);
            }
            did_emsg.set(did_emsg.get() + 1);
        }

        emsg_on_display.set(true); // remember there is an error message
        if msg_scrolled.get() != 0 {
            need_wait_return.set(true); // needed in case emsg() is called after wait_return() has cleared it
        }
        msg_ext_set_kind(kind);
        msg_scroll.set(true_0); // don't overwrite a previous message

        // Skip the flush until the whole message has been written, so that the
        // source line and the error arrive as one ext_messages event.
        let save_msg_skip_flush = msg_ext_skip_flush.get();
        msg_ext_skip_flush.set(true);
        msg_source(hl_id);
        msg_nowait.set(false); // wait for this msg
        let rv = msg_keep(s, hl_id, false, multiline);
        msg_ext_skip_flush.set(save_msg_skip_flush);
        rv
    }
}

/// Show an error message. Exported for the unit specs.
///
/// # Safety
/// `s` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emsg(s: *const c_char) -> bool {
    unsafe { emsg_multiline(s, c"emsg".as_ptr(), HLF_E, false) }
}

/// "E354: Invalid register name" for register `name`.
///
/// # Safety
/// Only that `name` is a character code.
pub unsafe fn emsg_invreg(name: c_int) {
    unsafe {
        semsg(
            gettext(c"E354: Invalid register name: '%s'".as_ptr()),
            transchar_buf(ptr::null(), name),
        );
    }
}

/// [`emsg`] with `printf` formatting.
///
/// # Safety
/// `fmt` and the arguments must agree, as for `printf`.
pub unsafe extern "C" fn semsg(fmt: *const c_char, mut c2rust_args: ...) -> bool {
    unsafe { semsgv(fmt, c2rust_args.clone()) }
}

/// [`emsg_multiline`] with `printf` formatting.
///
/// # Safety
/// As [`semsg`].
pub unsafe extern "C" fn semsg_multiline(
    kind: *const c_char,
    fmt: *const c_char,
    mut c2rust_args: ...
) -> bool {
    unsafe {
        // A multiline error can be much longer than one line's worth.
        static errbuf: GlobalCell<[c_char; 8192]> = GlobalCell::new([0; 8192]);
        if emsg_not_now() {
            return true;
        }
        vim_vsnprintf(
            errbuf.ptr().cast(),
            ::core::mem::size_of::<[c_char; 8192]>(),
            fmt,
            c2rust_args.clone(),
        );
        emsg_multiline(errbuf.ptr().cast(), kind, HLF_E, true)
    }
}

/// The `va_list` half of [`semsg`], shared with [`siemsg`].
///
/// # Safety
/// `fmt` and `ap` must agree, as for `vprintf`.
pub(crate) unsafe fn semsgv(fmt: *const c_char, ap: VaList) -> bool {
    unsafe {
        static errbuf: GlobalCell<[c_char; 1025]> = GlobalCell::new([0; 1025]);
        if emsg_not_now() {
            return true;
        }
        vim_vsnprintf(
            errbuf.ptr().cast(),
            ::core::mem::size_of::<[c_char; 1025]>(),
            fmt,
            ap,
        );
        emsg(errbuf.ptr().cast())
    }
}

/// An internal error: same as [`emsg`], but skipped when errors are off.
///
/// # Safety
/// `s` must be a valid C string.
pub unsafe fn iemsg(s: *const c_char) {
    unsafe {
        if emsg_not_now() {
            return;
        }
        emsg(s);
    }
}

/// [`iemsg`] with `printf` formatting.
///
/// # Safety
/// As [`semsg`].
pub unsafe extern "C" fn siemsg(s: *const c_char, mut c2rust_args: ...) {
    unsafe {
        if emsg_not_now() {
            return;
        }
        semsgv(s, c2rust_args.clone());
    }
}

/// "E5555: API call: <where>", for a reached-the-unreachable case.
///
/// # Safety
/// `where_0` must be a valid C string.
pub unsafe fn internal_error(where_0: *const c_char) {
    unsafe {
        siemsg(gettext(&raw const e_intern2 as *const c_char), where_0);
    }
}

/// Deferred-event handler for [`msg_schedule_semsg`].
///
/// # Safety
/// `argv[0]` must be an allocated C string this call takes ownership of.
pub(crate) unsafe extern "C" fn msg_semsg_event(argv: *mut *mut c_void) {
    unsafe {
        let s: *mut c_char = (*argv).cast();
        emsg(s);
        xfree(s.cast());
    }
}

/// [`semsg`], but deferred to the main loop.
///
/// For the places that cannot show a message where they are -- inside a
/// libuv callback, or with the screen mid-update.
///
/// # Safety
/// As [`semsg`].
pub unsafe extern "C" fn msg_schedule_semsg(fmt: *const c_char, mut c2rust_args: ...) {
    unsafe {
        vim_vsnprintf(
            IObuff.ptr().cast(),
            IOSIZE as size_t,
            fmt,
            c2rust_args.clone(),
        );
        let s = xstrdup(IObuff.ptr().cast());
        loop_schedule_deferred(
            main_loop.ptr(),
            Event::new(Some(msg_semsg_event), [s.cast::<c_void>()]),
        );
    }
}

/// Deferred-event handler for [`msg_schedule_semsg_multiline`].
///
/// # Safety
/// As [`msg_semsg_event`].
pub(crate) unsafe extern "C" fn msg_semsg_multiline_event(argv: *mut *mut c_void) {
    unsafe {
        let s: *mut c_char = (*argv).cast();
        emsg_multiline(s, c"emsg".as_ptr(), HLF_E, true);
        xfree(s.cast());
    }
}

/// [`semsg_multiline`], but deferred to the main loop.
///
/// # Safety
/// As [`semsg`].
pub unsafe extern "C" fn msg_schedule_semsg_multiline(fmt: *const c_char, mut c2rust_args: ...) {
    unsafe {
        vim_vsnprintf(
            IObuff.ptr().cast(),
            IOSIZE as size_t,
            fmt,
            c2rust_args.clone(),
        );
        let s = xstrdup(IObuff.ptr().cast());
        loop_schedule_deferred(
            main_loop.ptr(),
            Event::new(Some(msg_semsg_multiline_event), [s.cast::<c_void>()]),
        );
    }
}

/// Show a warning, which `'warningmsg'` highlighting and `v:warningmsg` pick
/// up. Repeated after a redraw, unlike an error.
///
/// # Safety
/// `message` must be a valid C string.
pub unsafe fn give_warning(message: *const c_char, hl: bool, hist: bool) {
    unsafe {
        // Don't do this for ":silent".
        if msg_silent.get() != 0 {
            return;
        }
        let save_msg_hist_off = msg_hist_off.get();
        msg_hist_off.set(!hist);

        no_wait_return.set(no_wait_return.get() + 1);
        set_vim_var_string(VV_WARNINGMSG, message, -1);
        xfree(keep_msg.get().cast());
        keep_msg.set(ptr::null_mut());
        keep_msg_hl_id.set(if hl { HLF_W } else { 0 });

        if msg_ext_kind.get().is_null() {
            msg_ext_set_kind(c"wmsg".as_ptr());
        }
        if msg(message, keep_msg_hl_id.get()) && msg_scrolled.get() == 0 {
            set_keep_msg(message, keep_msg_hl_id.get());
        }
        msg_didout.set(false); // overwrite this message
        msg_nowait.set(true); // don't wait for this message
        msg_col.set(0);

        no_wait_return.set(no_wait_return.get() - 1);
        msg_hist_off.set(save_msg_hist_off);
    }
}

/// [`give_warning`] with `printf` formatting.
///
/// # Safety
/// As [`semsg`].
pub unsafe extern "C" fn swmsg(hl: bool, fmt: *const c_char, mut c2rust_args: ...) {
    unsafe {
        vim_vsnprintf(
            IObuff.ptr().cast(),
            IOSIZE as size_t,
            fmt,
            c2rust_args.clone(),
        );
        give_warning(IObuff.ptr().cast(), hl, true);
    }
}
