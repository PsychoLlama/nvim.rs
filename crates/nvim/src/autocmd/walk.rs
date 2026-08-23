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
use crate::smsg_c;

/// Advance `apc` to the next autocommand whose pattern matches, updating
/// the execution-stack entry when the pattern changes.
///
/// When there is no next one, `lastpat` is left null and `auidx` at
/// `SIZE_MAX`, which is how [`getnextac`] and its caller see the end.
pub(crate) unsafe fn aucmd_next(apc: *mut AutoPatCmd) {
    unsafe {
        let entry = ((*exestack.ptr()).ga_data.cast::<estack_T>())
            .offset(((*exestack.ptr()).ga_len - 1) as isize);
        let acs = au_event_vec((*apc).event);
        debug_assert!((*apc).ausize <= (*acs).size);

        // `ausize` is the snapshot, not `(*acs).size`: a handler that
        // defines more autocommands for its own event must not extend the
        // walk it is running under.
        for i in (*apc).auidx..(*apc).ausize {
            if got_int.get() {
                break;
            }
            let ap = (*(*acs).items.add(i)).pat;

            // Skip deleted autocommands.
            if ap.is_null() {
                continue;
            }
            // Skip the matching when the pattern did not change.
            if ap != (*apc).lastpat {
                // Skip the ones that don't match the group...
                if (*apc).group != AUGROUP_ALL && (*apc).group != (*ap).group {
                    continue;
                }
                // ...or the pattern, or the buffer number.
                let matched = if (*ap).buflocal_nr == 0 {
                    match_file_pat(
                        ::core::ptr::null_mut(),
                        &raw mut (*ap).reg_prog,
                        (*apc).fname,
                        (*apc).sfname,
                        (*apc).tail,
                        (*ap).allow_dirs as ::core::ffi::c_int,
                    )
                } else {
                    (*ap).buflocal_nr == (*apc).arg_bufnr
                };
                if !matched {
                    continue;
                }

                let name = event_nr2name((*apc).event);
                let s = gettext(c"%s Autocommands for \"%s\"".as_ptr());
                let sourcing_name_len = strlen(s)
                    .wrapping_add(strlen(name))
                    .wrapping_add((*ap).patlen as size_t)
                    .wrapping_add(1);
                let namep = xmalloc(sourcing_name_len).cast::<::core::ffi::c_char>();
                snprintf(namep, sourcing_name_len, s, name, (*ap).pat);
                if p_verbose.get() >= 8 {
                    verbose_enter();
                    smsg_c!(0, gettext(c"Executing %s".as_ptr()), namep);
                    verbose_leave();
                }

                // Point the execution stack at this autocommand.
                xfree((*entry).es_name.cast::<::core::ffi::c_void>());
                (*entry).es_name = namep;
                (*entry).es_info.aucmd = apc;
            }

            (*apc).lastpat = ap;
            (*apc).auidx = i;

            line_breakcheck();
            return;
        }

        // Nothing left: clear the ETYPE_AUCMD stack entry.
        xfree((*entry).es_name.cast::<::core::ffi::c_void>());
        (*entry).es_name = ::core::ptr::null_mut();
        (*entry).es_info.aucmd = ::core::ptr::null_mut();

        (*apc).lastpat = ::core::ptr::null_mut();
        (*apc).auidx = SIZE_MAX as size_t;
    }
}

/// Run an autocommand whose handler is a callback rather than an Ex
/// command.
///
/// Answers whether the callback asked to be deleted, which only a Lua one
/// can do (by returning `true`).
unsafe fn au_callback(ac: *const AutoCmd, apc: *const AutoPatCmd) -> bool {
    unsafe {
        let mut callback = (*ac).handler_fn.clone();
        if callback.type_0 != kCallbackLua {
            let mut argsin = TV_INITIAL_VALUE;
            let mut rettv = TV_INITIAL_VALUE;
            callback_call(&raw mut callback, 0, &raw mut argsin, &raw mut rettv);
            return false;
        }

        let mut data = DictBuf::<7>::new();
        data.insert(c"id", Object::integer((*ac).id));
        data.insert(
            c"event",
            Object::string(cstr_as_string(event_nr2name((*apc).event))),
        );
        data.insert(c"file", Object::string(cstr_as_string((*apc).afile_orig)));
        data.insert(
            c"match",
            Object::string(cstr_as_string(autocmd_match.get())),
        );
        data.insert(c"buf", Object::integer(autocmd_bufnr.get() as Integer));
        if !(*apc).data.is_null() {
            data.insert(c"data", *(*apc).data);
        }
        match (*(*ac).pat).group {
            AUGROUP_ERROR => abort(),
            // The pseudo-groups are not something a handler can be told.
            AUGROUP_DEFAULT | AUGROUP_ALL | AUGROUP_DELETED => {}
            group => {
                data.insert(c"group", Object::integer(group as Integer));
            }
        }

        let mut args = ArrayBuf::<1>::new();
        args.push(data.object());

        let result = nlua_call_ref(
            callback.data.luaref,
            ::core::ptr::null(),
            args.array(),
            kRetNilBool,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
        );
        result.type_0 == kObjectTypeBoolean && result.data.boolean
    }
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
    unsafe {
        let apc = cookie.cast::<AutoPatCmd>();
        let acs = au_event_vec((*apc).event);

        aucmd_next(apc);
        if (*apc).lastpat.is_null() {
            return ::core::ptr::null_mut();
        }

        debug_assert!((*apc).auidx < (*acs).size);
        let ac = (*acs).items.add((*apc).auidx);
        debug_assert!(!(*ac).pat.is_null());
        let mut oneshot = (*ac).once;

        if p_verbose.get() >= 9 {
            verbose_enter_scroll();
            let handler_str = aucmd_handler_to_string(ac);
            smsg_c!(0, gettext(c"autocommand %s".as_ptr()), handler_str);
            // Don't overwrite this either.
            msg_puts(c"\n".as_ptr());
            xfree(handler_str.cast::<::core::ffi::c_void>());
            verbose_leave_scroll();
        }

        // `autocmd_nested` has to be set before any Lua runs, or a nested
        // event fired from the callback sees the wrong value.
        autocmd_nested.set((*ac).nested);
        current_sctx.set((*ac).script_ctx);
        (*apc).script_ctx = current_sctx.get();

        let retval;
        if (*ac).handler_cmd.is_null() {
            let mut ac_copy = (*ac).clone();
            // Mark a `++once` handler removed *before* running it, so a
            // `:doautocmd` from inside it cannot run it again (#25526).
            (*ac).pat = if oneshot {
                ::core::ptr::null_mut()
            } else {
                (*ac).pat
            };
            let rv = au_callback(&raw mut ac_copy, apc);
            if oneshot {
                // Through `acs`: the callback may have defined an
                // autocommand, which reallocates `items` and invalidates
                // `ac`.
                (*(*acs).items.add((*apc).auidx)).pat = ac_copy.pat;
            }
            // A callback returning true asks to be deleted.
            oneshot = oneshot || rv;

            // HACK(tjdevries): we just return "not-null" and keep going.
            // Fixing it means either teaching `do_cmdline` to take
            // something other than a string, or looping over the matches
            // here instead of being pulled.
            retval = xcalloc(1, 1).cast::<::core::ffi::c_char>();
        } else {
            retval = xstrdup((*ac).handler_cmd);
        }

        // Delete a one-shot autocommand in anticipation of its execution.
        if oneshot {
            aucmd_del((*acs).items.add((*apc).auidx));
        }

        if (*apc).auidx < (*apc).ausize {
            (*apc).auidx = (*apc).auidx.wrapping_add(1);
        } else {
            (*apc).auidx = SIZE_MAX as size_t;
        }

        retval
    }
}
