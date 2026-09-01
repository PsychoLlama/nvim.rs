//! One step of a firing walk.
//!
//! [`getnextac`] is the `do_cmdline` getline callback an autocommand body is
//! executed through: each call hands back the next matching command,
//! advancing [`aucmd_next`] over the pattern list until one matches the file
//! name being fired for.  `au_callback` is the same step for an autocommand
//! whose body is a Lua callback rather than a Vimscript command.
//!
//! Nothing here may cache a position.  The walk runs *while* the handler it
//! is running can define and delete autocommands, so `(*acs).items` is
//! re-read at every use (a define reallocates it), a row may go
//! `pat == NULL` under the index (`aucmd_del` only marks), and the walk's
//! own bound is the `ausize` snapshot `apply_autocmds_group` took -- which
//! is what keeps a handler that defines more autocommands for its own event
//! from looping forever.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::lua::executor::nlua_call_ref_quiet;
use crate::message_fmt::c_str;
use crate::smsg;
use crate::types::EstackInfo;

/// Advance `apc` to the next autocommand whose pattern matches, updating
/// the execution-stack entry when the pattern changes.
///
/// When there is no next one, `lastpat` is left null and `auidx` at
/// `SIZE_MAX`, which is how [`getnextac`] and its caller see the end.
pub(crate) unsafe fn aucmd_next(apc: *mut AutoPatCmd) {
    // SAFETY: `apc` is the caller's cursor, standing in the frame
    // `apply_autocmds_group` is still in, and `acs` is the event table's own
    // row for the event it is walking -- both live for this whole call.
    let acs = au_event_vec(unsafe { (*apc).event });
    let (auidx, ausize) = unsafe { ((*apc).auidx, (*apc).ausize) };
    debug_assert!(ausize <= unsafe { (*acs).size });

    // `ausize` is the snapshot, not `(*acs).size`: a handler that
    // defines more autocommands for its own event must not extend the
    // walk it is running under.
    for i in auidx..ausize {
        if got_int.get() {
            break;
        }
        // SAFETY: `i` is below `ausize`, which the assert above ties to the
        // vector's length, so this is one of its rows.
        let ap = unsafe { (*(*acs).items.add(i)).pat };

        // Skip deleted autocommands.
        if ap.is_null() {
            continue;
        }
        // Skip the matching when the pattern did not change.
        if ap != unsafe { (*apc).lastpat } {
            // Skip the ones that don't match the group...
            //
            // SAFETY: `ap` is non-null, so it is a pattern the vector owns,
            // and its `reg_prog` is the cached program `match_file_pat` may
            // compile into.  The names come from `apc`, which holds them for
            // the length of the walk.
            let group = unsafe { (*apc).group };
            if group != AUGROUP_ALL && group != unsafe { (*ap).group } {
                continue;
            }
            // ...or the pattern, or the buffer number.
            let matched = if unsafe { (*ap).buflocal_nr } == 0 {
                unsafe {
                    match_file_pat(
                        ::core::ptr::null_mut(),
                        &raw mut (*ap).reg_prog,
                        (*apc).fname,
                        (*apc).sfname,
                        (*apc).tail,
                        (*ap).allow_dirs as ::core::ffi::c_int,
                    )
                }
            } else {
                unsafe { (*ap).buflocal_nr == (*apc).arg_bufnr }
            };
            if !matched {
                continue;
            }

            let name = event_nr2name(unsafe { (*apc).event });
            let s = gettext(c"%s Autocommands for \"%s\"");
            let name_len = unsafe { cstr::bytes_at(name) }.len();
            let sourcing_name_len = unsafe {
                s.count_bytes()
                    .wrapping_add(name_len)
                    .wrapping_add((*ap).patlen as size_t)
                    .wrapping_add(1)
            };
            let namep = unsafe { xmalloc(sourcing_name_len) }.cast::<::core::ffi::c_char>();
            unsafe { snprintf(namep, sourcing_name_len, s.as_ptr(), name, (*ap).pat) };
            if p_verbose.get() >= 8 {
                unsafe { verbose_enter() };
                // SAFETY: `namep` is the NUL-terminated name just built.
                let namep = unsafe { c_str(namep) };
                smsg!(0, "Executing {namep}");
                unsafe { verbose_leave() };
            }

            // Point the execution stack at this autocommand.
            //
            // SAFETY: the stack takes `namep`, and hands back the name it
            // held, which is this walk's own to free.
            unsafe {
                xfree(crate::runtime::replace_sourcing_name(namep).cast::<::core::ffi::c_void>());
            };
            crate::runtime::with_innermost(|entry| entry.es_info = EstackInfo::Autocommand(apc));
        }

        // SAFETY: `apc` is the caller's live cursor.
        unsafe { (*apc).lastpat = ap };
        unsafe { (*apc).auidx = i };

        line_breakcheck();
        return;
    }

    // Nothing left: clear the ETYPE_AUCMD stack entry.
    //
    // SAFETY: as above -- the name the stack gives back is ours to free.
    unsafe {
        xfree(
            crate::runtime::replace_sourcing_name(::core::ptr::null_mut())
                .cast::<::core::ffi::c_void>(),
        );
    };
    crate::runtime::with_innermost(|entry| entry.es_info = EstackInfo::None);

    // SAFETY: `apc` is the caller's live cursor.
    unsafe { (*apc).lastpat = ::core::ptr::null_mut() };
    unsafe { (*apc).auidx = SIZE_MAX as size_t };
}

/// Run an autocommand whose handler is a callback rather than an Ex
/// command.
///
/// Answers whether the callback asked to be deleted, which only a Lua one
/// can do (by returning `true`).
unsafe fn au_callback(ac: *const AutoCmd, apc: *const AutoPatCmd) -> bool {
    // SAFETY: `ac` and `apc` are the caller's live row and walk cursor, and
    // the row owns the handler this clones.
    let mut callback = unsafe { (*ac).handler_fn.clone() };
    if callback.type_0 != kCallbackLua {
        let mut argsin = TV_INITIAL_VALUE;
        let mut rettv = TV_INITIAL_VALUE;
        // SAFETY: three locals of this frame, which outlive the call.
        unsafe { callback_call(&raw mut callback, 0, &raw mut argsin, &raw mut rettv) };
        return false;
    }

    // SAFETY: every name below is a NUL-terminated string of the row's or
    // the walk's, and each `String_0` borrows it only until the call that
    // copies it out.
    let mut data = DictBuf::<7>::new();
    data.insert(c"id", Object::integer(unsafe { (*ac).id }));
    data.insert(
        c"event",
        Object::string(unsafe { cstr_as_string(event_nr2name((*apc).event)) }),
    );
    data.insert(
        c"file",
        Object::string(unsafe { cstr_as_string((*apc).afile_orig) }),
    );
    data.insert(
        c"match",
        Object::string(unsafe { cstr_as_string(autocmd_match.get()) }),
    );
    data.insert(c"buf", Object::integer(autocmd_bufnr.get() as Integer));
    let event_data = unsafe { (*apc).data };
    if !event_data.is_null() {
        // SAFETY: non-null, so it is the object the caller published.
        data.insert(c"data", unsafe { *event_data });
    }
    // SAFETY: a row that is being run still has its pattern.
    let group = unsafe { (*(*ac).pat).group };
    match group {
        // SAFETY: `abort` never comes back.
        AUGROUP_ERROR => unsafe { abort() },
        // The pseudo-groups are not something a handler can be told.
        AUGROUP_DEFAULT | AUGROUP_ALL | AUGROUP_DELETED => {}
        group => {
            data.insert(c"group", Object::integer(group as Integer));
        }
    }

    let mut args = ArrayBuf::<1>::new();
    args.push(data.object());

    // SAFETY: `callback` is Lua-typed, so `luaref` is the live union field;
    // `args` stands for the length of the call.
    let result = unsafe {
        nlua_call_ref_quiet(
            callback.data.luaref,
            ::core::ptr::null(),
            args.array(),
            kRetNilBool,
            ::core::ptr::null_mut(),
        )
    };
    matches!(result, Object::Boolean(true))
}

/// The `do_cmdline` getline callback an autocommand body runs through:
/// one call per matching autocommand.
///
/// The `_c`/`_indent`/`_do_concat` parameters exist for `do_cmdline`'s
/// signature.  A callback handler has no line to give back, so this
/// answers an empty allocated string -- "not null, keep going".
pub unsafe fn getnextac(
    _c: ::core::ffi::c_int,
    cookie: *mut ::core::ffi::c_void,
    _indent: ::core::ffi::c_int,
    _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    // SAFETY: by the contract `cookie` is the `AutoPatCmd` that
    // `apply_autocmds_group` handed `do_cmdline`, and it stands for as long
    // as the walk does; `acs` is the event table's own row for its event.
    let apc = cookie.cast::<AutoPatCmd>();
    let acs = au_event_vec(unsafe { (*apc).event });

    unsafe { aucmd_next(apc) };
    if unsafe { (*apc).lastpat }.is_null() {
        return ::core::ptr::null_mut();
    }

    debug_assert!(unsafe { (*apc).auidx } < unsafe { (*acs).size });
    // SAFETY: the row the walk just stopped at, which `aucmd_next` only
    // stops at while it still has a pattern.
    let ac = unsafe { (*acs).items.add((*apc).auidx) };
    debug_assert!(!unsafe { (*ac).pat }.is_null());
    let mut oneshot = unsafe { (*ac).once };

    if p_verbose.get() >= 9 {
        // SAFETY: `aucmd_handler_to_string` answers an allocated
        // NUL-terminated string this owns and frees below.
        unsafe { verbose_enter_scroll() };
        let handler_str = unsafe { aucmd_handler_to_string(ac) };
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let shown = unsafe { c_str(handler_str) };
        smsg!(0, "autocommand {shown}");
        // Don't overwrite this either.
        unsafe { msg_puts(c"\n".as_ptr()) };
        unsafe { xfree(handler_str.cast::<::core::ffi::c_void>()) };
        unsafe { verbose_leave_scroll() };
    }

    // `autocmd_nested` has to be set before any Lua runs, or a nested
    // event fired from the callback sees the wrong value.
    //
    // SAFETY: `ac` is the row being run, `apc` the live cursor.
    autocmd_nested.set(unsafe { (*ac).nested });
    current_sctx.set(unsafe { (*ac).script_ctx });
    unsafe { (*apc).script_ctx = current_sctx.get() };

    let retval;
    if unsafe { (*ac).handler_cmd }.is_null() {
        let mut ac_copy = unsafe { (*ac).clone() };
        // Mark a `++once` handler removed *before* running it, so a
        // `:doautocmd` from inside it cannot run it again (#25526).
        //
        // SAFETY: the row is still the vector's own at this point -- only
        // the callback below can reallocate it.
        unsafe {
            (*ac).pat = if oneshot {
                ::core::ptr::null_mut()
            } else {
                (*ac).pat
            }
        };
        // SAFETY: `ac_copy` is a local, and `apc` the live cursor.
        let rv = unsafe { au_callback(&raw mut ac_copy, apc) };
        if oneshot {
            // Through `acs`: the callback may have defined an
            // autocommand, which reallocates `items` and invalidates
            // `ac`.
            //
            // SAFETY: `auidx` is the index the walk stopped at, which is
            // still in bounds of the vector however it was reallocated.
            unsafe { (*(*acs).items.add((*apc).auidx)).pat = ac_copy.pat };
        }
        // A callback returning true asks to be deleted.
        oneshot = oneshot || rv;

        // HACK(tjdevries): we just return "not-null" and keep going.
        // Fixing it means either teaching `do_cmdline` to take
        // something other than a string, or looping over the matches
        // here instead of being pulled.
        retval = unsafe { xcalloc(1, 1) }.cast::<::core::ffi::c_char>();
    } else {
        // SAFETY: the row's command text, which `xstrdup` copies.
        retval = unsafe { xstrdup((*ac).handler_cmd) };
    }

    // Delete a one-shot autocommand in anticipation of its execution.
    if oneshot {
        // SAFETY: the row at the index the walk stopped at, re-reached
        // through `acs` for the reason above.
        unsafe { aucmd_del((*acs).items.add((*apc).auidx)) };
    }

    // SAFETY: `apc` is the caller's live cursor.
    if unsafe { (*apc).auidx } < unsafe { (*apc).ausize } {
        unsafe { (*apc).auidx = (*apc).auidx.wrapping_add(1) };
    } else {
        unsafe { (*apc).auidx = SIZE_MAX as size_t };
    }

    retval
}
