//! `:autocmd` itself -- parsing it, and installing what it says.
//!
//! [`do_autocmd`] splits the command into group, event list, pattern and
//! command, then loops [`do_autocmd_event`] over the events;
//! [`autocmd_register`] is the one place an `AutoPat`/`AutoCmd` pair is
//! created, and the same entry point the API's `nvim_create_autocmd`
//! reaches.  The `arg_*` helpers are the pieces of the parse the API
//! shares, and [`autocmd_delete_id`] is the deletion by id that
//! `nvim_del_autocmd` is.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::semsg_c;
use crate::types::{FAIL, OK, RefcountSize};
use crate::winlayer::{Live, Win, tabs};

/// A `Callback` that holds nothing: `CALLBACK_INIT`.
const CALLBACK_INIT: Callback = Callback {
    data: Callback_data {
        funcref: ::core::ptr::null_mut(),
    },
    type_0: kCallbackNone,
};

/// `:autocmd [group] {event} {pat} [++once] [++nested] {cmd}`, and every
/// shorter spelling of it: listing, deleting, and `:autocmd *`.
pub unsafe fn do_autocmd(
    eap: *mut exarg_T,
    arg_in: *mut ::core::ffi::c_char,
    forceit: ::core::ffi::c_int,
) {
    let mut arg = arg_in;
    let mut envpat: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut cmd: *mut ::core::ffi::c_char;
    let mut need_free = false;
    let mut nested = false;
    let mut once = false;

    let group = if unsafe { *arg } == b'|' as ::core::ffi::c_char {
        unsafe { (*eap).nextcmd = arg.add(1) };
        arg = c"".as_ptr().cast_mut();
        AUGROUP_ALL
    } else {
        unsafe { arg_augroup_get(&raw mut arg) }
    };

    // Validate every event name before doing anything.
    let mut pat = unsafe { arg_event_skip(arg, group != AUGROUP_ALL) };
    if pat.is_null() {
        return;
    }

    pat = unsafe { skipwhite(pat) };
    if unsafe { *pat } == b'|' as ::core::ffi::c_char {
        unsafe { (*eap).nextcmd = pat.add(1) };
        pat = c"".as_ptr().cast_mut();
        cmd = c"".as_ptr().cast_mut();
    } else {
        // Scan over the pattern, whose whitespace may be
        // backslash-escaped, and terminate it in place.
        cmd = pat;
        while unsafe { *cmd } != 0
            && (!ascii_iswhite(unsafe { *cmd } as ::core::ffi::c_int)
                || unsafe { *cmd.sub(1) } == b'\\' as ::core::ffi::c_char)
        {
            cmd = unsafe { cmd.add(1) };
        }
        if unsafe { *cmd } != 0 {
            unsafe { *cmd = 0 };
            cmd = unsafe { cmd.add(1) };
        }

        if !unsafe { vim_strchr(pat, '$' as ::core::ffi::c_int) }.is_null()
            || !unsafe { vim_strchr(pat, '~' as ::core::ffi::c_int) }.is_null()
        {
            envpat = unsafe { expand_env_save(pat) };
            if !envpat.is_null() {
                pat = envpat;
            }
        }

        cmd = unsafe { skipwhite(cmd) };

        // Two passes, so the flags may be given in either order.
        let mut invalid_flags = false;
        for _ in 0..2 {
            if unsafe { *cmd } == 0 {
                continue;
            }
            invalid_flags |=
                unsafe { arg_autocmd_flag_get(&raw mut once, &raw mut cmd, c"++once", 6) };
            invalid_flags |=
                unsafe { arg_autocmd_flag_get(&raw mut nested, &raw mut cmd, c"++nested", 8) };
            // The deprecated spelling of `++nested`.
            invalid_flags |=
                unsafe { arg_autocmd_flag_get(&raw mut nested, &raw mut cmd, c"nested", 6) };
        }
        if invalid_flags {
            return;
        }

        if unsafe { *cmd } != 0 {
            cmd = unsafe { expand_sfile(cmd) };
            if cmd.is_null() {
                return;
            }
            need_free = true;
        }
    }

    // No command and no `!`: this is a listing.
    let is_showing = forceit == 0 && unsafe { *cmd } == 0;
    let all_events = unsafe { *arg } == b'*' as ::core::ffi::c_char
        || unsafe { *arg } == b'|' as ::core::ffi::c_char
        || unsafe { *arg } == 0;

    if is_showing {
        unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
        unsafe { msg_puts_title(gettext(c"\n--- Autocommands ---").as_ptr()) };

        if all_events {
            unsafe { au_show_for_all_events(group, pat) };
        } else {
            let event = unsafe { event_name2nr(arg, &raw mut arg) };
            debug_assert!(event < NUM_EVENTS);
            unsafe { au_show_for_event(group, event, pat) };
        }
    } else if all_events {
        if unsafe { *cmd } != 0 {
            emsg(gettext(e_cannot_define_autocommands_for_all_events));
        } else {
            unsafe {
                do_all_autocmd_events(
                    pat,
                    once,
                    nested as ::core::ffi::c_int,
                    cmd,
                    forceit != 0,
                    group,
                )
            };
        }
    } else {
        while unsafe { *arg } != 0
            && unsafe { *arg } != b'|' as ::core::ffi::c_char
            && !ascii_iswhite(unsafe { *arg } as ::core::ffi::c_int)
        {
            let event = unsafe { event_name2nr(arg, &raw mut arg) };
            debug_assert!(event < NUM_EVENTS);
            if unsafe {
                do_autocmd_event(
                    event,
                    pat,
                    once,
                    nested as ::core::ffi::c_int,
                    cmd,
                    forceit != 0,
                    group,
                )
            } == FAIL
            {
                break;
            }
        }
    }

    if need_free {
        unsafe { xfree(cmd.cast::<::core::ffi::c_void>()) };
    }
    unsafe { xfree(envpat.cast::<::core::ffi::c_void>()) };
}

/// [`do_autocmd_event`] for every event: what `:autocmd *` means.
pub unsafe fn do_all_autocmd_events(
    pat: *const ::core::ffi::c_char,
    once: bool,
    nested: ::core::ffi::c_int,
    cmd: *mut ::core::ffi::c_char,
    del: bool,
    group: ::core::ffi::c_int,
) {
    for event in 0..NUM_EVENTS {
        if unsafe { do_autocmd_event(event, pat, once, nested, cmd, del, group) } == FAIL {
            return;
        }
    }
}

/// `:autocmd` for one event, over each of the comma-separated patterns.
///
/// An empty `cmd` with `del` deletes; a non-empty one adds.  Both together
/// are `:autocmd! {event} {pat} {cmd}`, which deletes the existing
/// autocommands on the pattern and then appends to the same `AutoPat`.
pub unsafe fn do_autocmd_event(
    event: event_T,
    mut pat: *const ::core::ffi::c_char,
    once: bool,
    nested: ::core::ffi::c_int,
    cmd: *const ::core::ffi::c_char,
    del: bool,
    group: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    // Listing all patterns goes through `au_show_for_event` instead.
    debug_assert!(unsafe { *pat } != 0 || del);

    let mut buflocal_pat = [0 as ::core::ffi::c_char; BUFLOCAL_PAT_LEN as usize];
    let is_adding_cmd = unsafe { *cmd } != 0;
    let findgroup = if group == AUGROUP_ALL {
        current_augroup.get()
    } else {
        group
    };

    // `:autocmd! {event}`: every pattern goes.
    if unsafe { *pat } == 0 && del {
        aucmd_del_for_event_and_group(event, findgroup);
        return OK;
    }

    let mut patlen = unsafe { aucmd_span_pattern(pat, &raw mut pat) } as ::core::ffi::c_int;
    while patlen != 0 {
        let endpat = unsafe { pat.offset(patlen as isize) };

        // `<buffer[=X]>` is normalised on the way in, so every later
        // comparison against a stored pattern is a plain `strncmp`.
        if unsafe { aupat_is_buflocal(pat, patlen) } {
            unsafe {
                aupat_normalize_buflocal_pat(
                    buflocal_pat.as_mut_ptr(),
                    pat,
                    patlen,
                    aupat_get_buflocal_nr(pat, patlen),
                )
            };
            pat = buflocal_pat.as_ptr();
            patlen = unsafe { strlen(buflocal_pat.as_ptr()) } as ::core::ffi::c_int;
        }

        if del {
            debug_assert!(unsafe { *pat } != 0);
            // Only the commands go; the `AutoPat` survives when a new
            // command is about to be appended to it below.
            let acs = au_event_vec(event);
            let mut i: usize = 0;
            while i < unsafe { (*acs).size } {
                let ac = unsafe { (*acs).items.add(i) };
                let ap = unsafe { (*ac).pat };
                if !ap.is_null()
                    && unsafe { (*ap).group } == findgroup
                    && unsafe { (*ap).patlen } == patlen
                    && unsafe { strncmp(pat, (*ap).pat, patlen as size_t) } == 0
                {
                    unsafe { aucmd_del(ac) };
                }
                i = i.wrapping_add(1);
            }
        }

        if is_adding_cmd {
            let mut handler_fn = CALLBACK_INIT;
            unsafe {
                autocmd_register(
                    0,
                    event,
                    pat,
                    patlen,
                    group,
                    once,
                    nested != 0,
                    ::core::ptr::null_mut(),
                    cmd,
                    &raw mut handler_fn,
                )
            };
        }

        patlen = unsafe { aucmd_span_pattern(endpat, &raw mut pat) } as ::core::ffi::c_int;
    }

    // The patterns and commands marked above can really go now.
    au_cleanup();
    OK
}

/// Create one autocommand: the only place an `AutoPat`/`AutoCmd` pair is
/// made, reached from both `:autocmd` and `nvim_create_autocmd`.
///
/// The handler is `handler_cmd` when that is non-null and `handler_fn`
/// otherwise.
pub unsafe fn autocmd_register(
    id: int64_t,
    event: event_T,
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
    group: ::core::ffi::c_int,
    once: bool,
    nested: bool,
    desc: *mut ::core::ffi::c_char,
    handler_cmd: *const ::core::ffi::c_char,
    handler_fn: *mut Callback,
) -> ::core::ffi::c_int {
    // 0 is not a valid group.
    debug_assert!(group != 0);

    if patlen > unsafe { strlen(pat) } as ::core::ffi::c_int {
        return FAIL;
    }

    let findgroup = if group == AUGROUP_ALL {
        current_augroup.get()
    } else {
        group
    };

    let is_buflocal = unsafe { aupat_is_buflocal(pat, patlen) };
    let mut buflocal_nr = 0;
    let mut buflocal_pat = [0 as ::core::ffi::c_char; BUFLOCAL_PAT_LEN as usize];
    if is_buflocal {
        buflocal_nr = unsafe { aupat_get_buflocal_nr(pat, patlen) };
        unsafe {
            aupat_normalize_buflocal_pat(buflocal_pat.as_mut_ptr(), pat, patlen, buflocal_nr)
        };
        pat = buflocal_pat.as_ptr();
        patlen = unsafe { strlen(buflocal_pat.as_ptr()) } as ::core::ffi::c_int;
    }

    // Reuse the pattern of the last live autocommand when it is the
    // same one, so `:autocmd` twice on a pattern compiles one regexp.
    let acs = au_event_vec(event);
    let mut ap: *mut AutoPat = ::core::ptr::null_mut();
    for i in (0..unsafe { (*acs).size }).rev() {
        ap = unsafe { (*(*acs).items.add(i)).pat };
        // Skip deleted autocommands.
        if ap.is_null() {
            continue;
        }
        if unsafe { (*ap).group } != findgroup
            || unsafe { (*ap).patlen } != patlen
            || unsafe { strncmp(pat, (*ap).pat, patlen as size_t) } != 0
        {
            ap = ::core::ptr::null_mut();
        }
        break;
    }

    if ap.is_null() {
        // A buffer-local pattern needs a buffer that exists.
        if is_buflocal && (buflocal_nr == 0 || find_buf(buflocal_nr).is_none()) {
            semsg!("E680: <buffer={}>: invalid buffer number ", buflocal_nr);
            return FAIL;
        }

        ap = unsafe { xmalloc(::core::mem::size_of::<AutoPat>()) }.cast::<AutoPat>();
        // SAFETY: the allocation above, live for the rest of this call.
        let mut new_pat = unsafe { Live::new(ap) };
        if is_buflocal {
            new_pat.buflocal_nr = buflocal_nr;
            new_pat.reg_prog = ::core::ptr::null_mut();
        } else {
            new_pat.buflocal_nr = 0;
            let reg_pat = unsafe {
                file_pat_to_reg_pat(
                    pat,
                    pat.offset(patlen as isize),
                    &raw mut (*ap).allow_dirs,
                    1,
                )
            };
            if !reg_pat.is_null() {
                new_pat.reg_prog = unsafe { vim_regcomp(reg_pat, RE_MAGIC) };
            }
            unsafe { xfree(reg_pat.cast::<::core::ffi::c_void>()) };
            if reg_pat.is_null() || new_pat.reg_prog.is_null() {
                unsafe { xfree(ap.cast::<::core::ffi::c_void>()) };
                return FAIL;
            }
        }

        new_pat.refcount = RefcountSize::ZERO;
        new_pat.pat = unsafe { xmemdupz(pat.cast::<::core::ffi::c_void>(), patlen as size_t) }
            .cast::<::core::ffi::c_char>();
        new_pat.patlen = patlen;

        // The events below compare against state sampled the last time
        // they fired.  The *first* autocommand for one has to seed that
        // state, or it fires immediately on a difference that predates
        // it.
        if event == EVENT_MODECHANGED && !has_event(EVENT_MODECHANGED) {
            last_mode.set(unsafe { get_mode() });
        }
        if (event == EVENT_CURSORMOVED && !has_event(EVENT_CURSORMOVED))
            || (event == EVENT_CURSORMOVEDI && !has_event(EVENT_CURSORMOVEDI))
        {
            last_cursormoved_win.set(curwin.get());
            last_cursormoved.set(cur_win().w_cursor);
        }
        if (event == EVENT_WINSCROLLED || event == EVENT_WINRESIZED)
            && !(has_event(EVENT_WINSCROLLED) || has_event(EVENT_WINRESIZED))
        {
            let save_curtab = curtab.get();
            for tp in tabs() {
                unsafe { unuse_tabpage(curtab.get()) };
                unsafe { use_tabpage(tp.raw()) };
                snapshot_windows_scroll_size();
            }
            unsafe { unuse_tabpage(curtab.get()) };
            unsafe { use_tabpage(save_curtab) };
        }

        // Spelled out rather than reusing `findgroup`: they are the
        // same value (nothing above can change `current_augroup`), but
        // upstream asks the question twice and a mutation aimed at
        // either one should still land where it lands.
        new_pat.group = if group == AUGROUP_ALL {
            current_augroup.get()
        } else {
            group
        };
    }

    // SAFETY: either the pattern found above or the one just allocated;
    // both are live for the rest of this call.
    let mut pat_entry = unsafe { Live::new(ap) };
    pat_entry.refcount.retain();

    // `kv_pushp`: append an `AutoCmd` at the end of the event's vector.
    // SAFETY: `acs` is the event's vector, whose array and size are grown
    // together here -- every step of the run reads or writes through it, so
    // one region is as tight as it gets.
    let ac = unsafe {
        if (*acs).size == (*acs).capacity {
            (*acs).capacity = if (*acs).capacity != 0 {
                (*acs).capacity << 1
            } else {
                8
            };
            (*acs).items = xrealloc(
                (*acs).items.cast::<::core::ffi::c_void>(),
                ::core::mem::size_of::<AutoCmd>().wrapping_mul((*acs).capacity),
            )
            .cast::<AutoCmd>();
        }
        let ac = (*acs).items.add((*acs).size);
        (*acs).size = (*acs).size.wrapping_add(1);
        // SAFETY: the slot `kv_pushp` just made, live until the vector next
        // grows -- which nothing below does.
        Live::new(ac)
    };
    let mut ac = ac;

    ac.pat = ap;
    ac.id = id;
    if handler_cmd.is_null() {
        ac.handler_cmd = ::core::ptr::null_mut();
        unsafe { callback_copy(&raw mut (*ac.raw()).handler_fn, handler_fn) };
    } else {
        ac.handler_cmd = unsafe { xstrdup(handler_cmd) };
    }
    ac.script_ctx = current_sctx.get();
    // `SOURCING_LNUM`: the line of the innermost execution-stack frame.
    ac.script_ctx.sc_lnum += crate::runtime::innermost_frame().es_lnum;
    unsafe { nlua_set_sctx(&raw mut (*ac.raw()).script_ctx) };
    ac.once = once;
    ac.nested = nested;
    ac.desc = if desc.is_null() {
        ::core::ptr::null_mut()
    } else {
        unsafe { xstrdup(desc) }
    };

    OK
}

/// The length of the first pattern in a comma-separated list, with `start`
/// left at where it begins.
///
/// Leading commas are skipped, and a comma inside braces or after a
/// backslash (`*.\{obj,o\}`) does not end a pattern.
pub unsafe fn aucmd_span_pattern(
    mut pat: *const ::core::ffi::c_char,
    start: *mut *const ::core::ffi::c_char,
) -> size_t {
    while unsafe { *pat } == b',' as ::core::ffi::c_char {
        pat = unsafe { pat.add(1) };
    }

    let mut p = pat;
    let mut brace_level = 0;
    while unsafe { *p } != 0
        && (unsafe { *p } != b',' as ::core::ffi::c_char
            || brace_level != 0
            || (p > pat && unsafe { *p.sub(1) } == b'\\' as ::core::ffi::c_char))
    {
        if unsafe { *p } == b'{' as ::core::ffi::c_char {
            brace_level += 1;
        } else if unsafe { *p } == b'}' as ::core::ffi::c_char {
            brace_level -= 1;
        }
        p = unsafe { p.add(1) };
    }

    unsafe { *start = pat };
    unsafe { p.offset_from(pat) as size_t }
}

/// Whether `do_modelines` should be called: false when `*argp` begins with
/// `<nomodeline>`, which is then skipped.
pub unsafe fn check_nomodeline(argp: *mut *mut ::core::ffi::c_char) -> bool {
    if unsafe { strncmp(*argp, c"<nomodeline>".as_ptr(), 12) } == 0 {
        unsafe { *argp = skipwhite((*argp).add(12)) };
        return false;
    }
    true
}

/// Delete the autocommand with this id, wherever it is.
///
/// Only autocommands created through the API have one.
///
/// Safe: it takes an id, not a pointer, and reaches the rows only through
/// the events' own vectors.
pub fn autocmd_delete_id(id: int64_t) -> bool {
    debug_assert!(id > 0);

    let mut success = false;
    for event in 0..NUM_EVENTS {
        let acs = au_event_vec(event);
        let mut i: usize = 0;
        while i < unsafe { (*acs).size } {
            let ac = unsafe { (*acs).items.add(i) };
            if unsafe { (*ac).id } == id {
                unsafe { aucmd_del(ac) };
                success = true;
            }
            i = i.wrapping_add(1);
        }
    }
    success
}

/// An autocommand's handler as an allocated string, whichever kind it is.
pub unsafe fn aucmd_handler_to_string(ac: *mut AutoCmd) -> *mut ::core::ffi::c_char {
    if unsafe { (*ac).handler_cmd.is_null() } {
        unsafe { callback_to_string(&raw mut (*ac).handler_fn, ::core::ptr::null_mut()) }
    } else {
        unsafe { xstrdup((*ac).handler_cmd) }
    }
}

/// Skip over `:autocmd`'s comma-separated event list, answering what
/// follows it -- or null, having raised an error, when a name is not an
/// event.
///
/// `have_group` only picks the wording: without a group, the leading word
/// could have been meant as one.
pub(crate) unsafe fn arg_event_skip(
    arg: *mut ::core::ffi::c_char,
    have_group: bool,
) -> *mut ::core::ffi::c_char {
    if unsafe { *arg } == b'*' as ::core::ffi::c_char {
        if unsafe { *arg.add(1) } != 0
            && !ascii_iswhite(unsafe { *arg.add(1) } as ::core::ffi::c_int)
        {
            // SAFETY: the message macros expand to a `vim_snprintf` over the // format literal above and the editor's message buffers.
            let arg = unsafe { c_str(arg) };
            semsg!("E215: Illegal character after *: {arg}");
            return ::core::ptr::null_mut();
        }
        return unsafe { arg.add(1) };
    }

    let mut pat = arg;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    while unsafe { *pat } != 0
        && unsafe { *pat } != b'|' as ::core::ffi::c_char
        && !ascii_iswhite(unsafe { *pat } as ::core::ffi::c_int)
    {
        if unsafe { event_name2nr(pat, &raw mut p) } >= NUM_EVENTS {
            // SAFETY: the message macros expand to a `vim_snprintf` over the
            // format literal above and the editor's message buffers.
            unsafe {
                semsg_c!(
                    gettext(if have_group {
                        c"E216: No such event: %s"
                    } else {
                        c"E216: No such group or event: %s"
                    }),
                    pat,
                )
            };
            return ::core::ptr::null_mut();
        }
        pat = p;
    }
    pat
}

/// Take a leading `++once`/`++nested`/`nested` off `*cmd_ptr`, setting
/// `*flag`.
///
/// Answers *true* on the error case -- the flag given twice -- so the
/// caller can `|=` the three calls together.
unsafe fn arg_autocmd_flag_get(
    flag: *mut bool,
    cmd_ptr: *mut *mut ::core::ffi::c_char,
    pattern: &CStr,
    len: ::core::ffi::c_int,
) -> bool {
    if unsafe { strncmp(*cmd_ptr, pattern.as_ptr(), len as size_t) } == 0
        && ascii_iswhite(unsafe { *(*cmd_ptr).offset(len as isize) } as ::core::ffi::c_int)
    {
        if unsafe { *flag } {
            // SAFETY: the message macros expand to a `vim_snprintf` over the // format literal above and the editor's message buffers.
            let pattern = unsafe { c_str(pattern.as_ptr()) };
            semsg!("E983: Duplicate argument: {pattern}");
            return true;
        }
        unsafe { *flag = true };
        unsafe { *cmd_ptr = skipwhite((*cmd_ptr).offset(len as isize)) };
    }
    false
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
