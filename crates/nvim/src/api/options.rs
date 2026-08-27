//! `nvim_*_option_*`: reading and writing options through the API.
//!
//! Every entry point here takes the same `opts` dictionary -- `scope`, `buf`,
//! `win`, `filetype` -- and the first thing each does is turn it into the
//! three things the option layer actually wants: which option, at what scope,
//! and on which buffer or window. That is [`OptionTarget`], and
//! [`option_target`] is the one place the dictionary's rules are enforced.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::api::private::helpers::{
    ERROR_INIT, NIL, Reported, api_set_error, api_set_sctx, api_try, api_typename,
    buffer_by_handle, has_key, window_by_handle,
};
use crate::api::private::validate::{api_err_exp, api_err_invalid};
use crate::autocmd::{
    EVENT_FILETYPE, aucmd_prepbuf, aucmd_restbuf, block_autocmds, do_filetype_autocmd, has_event,
    unblock_autocmds,
};
use crate::buffer::{BufFlags, buflist_new, bufref_valid, set_bufref, wipe_buffer};
use crate::options::{kOptBufhidden, kOptBuftype, kOptInvalid};
use core::ffi::{CStr, c_char, c_int, c_void};

use crate::main::{curbuf, current_sctx, curwin};
use crate::memline::ml_open;
use crate::memory::xstrdup;
use crate::option::{
    find_option, get_all_vimoptions, get_option_value_for, get_vimoption, object_as_optval,
    option_has_scope, optval_as_object, optval_free, set_option_direct, set_option_value_for,
};
use crate::types::{
    Arena, Dict, Error, FAIL, KeyDict_option, Object, OptIndex, OptScope, OptVal, OptValData,
    OptValType, OptionSetFlags, String_0, aco_save_T, buf_T, bufref_T, int64_t,
    kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, linenr_T, sctx_T, uint64_t,
};
use crate::window::close_windows;
use crate::winlayer::Buf;
use core::ptr;

const kOptValTypeString: OptValType = 2;
const kOptValTypeNil: OptValType = -1;
const kOptScopeBuf: OptScope = 2;
const kOptScopeWin: OptScope = 1;
const kOptScopeGlobal: OptScope = 0;

/// `buflist_new`: a scratch buffer, not on the buffer list.
const BLN_DUMMY: c_int = 4;

/// `current_sctx.sc_sid` for a `:set` that came from nowhere in particular,
/// which is what the scratch buffer's two option writes are.
const SID_NONE: c_int = -6;

/// What an `opts` dictionary resolved to: the option itself, the scope to
/// read or write it at, and the buffer or window that scope names.
struct OptionTarget {
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    scope: OptScope,
    /// The `buf_T` or `win_T` the scope names -- `get_option_value_for` and
    /// `set_option_value_for` take it untyped and read `scope` to know which
    /// it is -- or null for the global scope.
    from: *mut c_void,
    /// `filetype`, when the caller asked for the option as a buffer of that
    /// type would see it. Null otherwise.
    filetype: *mut c_char,
}

/// Resolve `opts` against the option named `name`, or report why it cannot
/// be: an unknown option, a scope the option does not have, or a combination
/// of keys that contradict each other.
///
/// # Safety
/// `opts` must point at a filled-in `KeyDict_option`, `name` must be a C
/// string, and `err` must be the caller's error slot.
unsafe fn option_target(
    opts: *mut KeyDict_option,
    name: *mut c_char,
    err: *mut Error,
) -> Option<OptionTarget> {
    // `opts`' keys, by their index in its `is_set` mask. Function-local so
    // that they cannot collide in the flat namespace `tools/ffigen` renders
    // module-level constants into.
    const OPTIDX_BUF: c_int = 1;
    const OPTIDX_WIN: c_int = 2;
    const OPTIDX_SCOPE: c_int = 3;
    const OPTIDX_FILETYPE: c_int = 4;

    // SAFETY: `opts` is the caller's, per this function's contract.
    let set = move |key| unsafe { has_key((*opts).is_set__option_, key) };
    // SAFETY: `err` is the caller's, and `msg` is always a static string.
    let fail = move |msg: &CStr| unsafe {
        api_set_error(err, kErrorTypeValidation, c"%s".as_ptr(), msg.as_ptr());
    };

    let mut opt_flags = OptionSetFlags::NONE;
    if set(OPTIDX_SCOPE) {
        // SAFETY: as above; `scope` is a NUL-terminated key of `opts`.
        let scope = unsafe { CStr::from_ptr((*opts).scope.data()) };
        opt_flags = match scope.to_bytes() {
            b"local" => OptionSetFlags::LOCAL,
            b"global" => OptionSetFlags::GLOBAL,
            _ => {
                // SAFETY: `err` is the caller's; the strings are static.
                unsafe {
                    api_err_exp(
                        err,
                        c"scope".as_ptr(),
                        c"'local' or 'global'".as_ptr(),
                        ptr::null(),
                    );
                }
                return None;
            }
        };
    }

    let mut scope = kOptScopeGlobal;
    let mut from = ptr::null_mut::<c_void>();
    if set(OPTIDX_WIN) {
        scope = kOptScopeWin;
        // SAFETY: `err` is the caller's, and the handle is an integer.
        let win = unsafe { window_by_handle((*opts).win, &mut *err) };
        from = win.map_or(ptr::null_mut(), |w| w.raw().cast());
        // SAFETY: `err` is the caller's.
        if unsafe { (*err).type_0 } != kErrorTypeNone {
            return None;
        }
    }
    if set(OPTIDX_BUF) {
        if set(OPTIDX_SCOPE) && opt_flags == OptionSetFlags::GLOBAL {
            fail(c"cannot use both global 'scope' and 'buf'");
            return None;
        }
        opt_flags = OptionSetFlags::LOCAL;
        scope = kOptScopeBuf;
        // SAFETY: as the window lookup above.
        let buf = unsafe { buffer_by_handle((*opts).buf, &mut *err) };
        from = buf.map_or(ptr::null_mut(), |b| b.raw().cast());
        // SAFETY: `err` is the caller's.
        if unsafe { (*err).type_0 } != kErrorTypeNone {
            return None;
        }
    }
    if set(OPTIDX_FILETYPE) && (set(OPTIDX_BUF) || set(OPTIDX_SCOPE) || set(OPTIDX_WIN)) {
        fail(c"cannot use 'filetype' with 'scope', 'buf' or 'win'");
        return None;
    }
    if set(OPTIDX_WIN) && set(OPTIDX_BUF) {
        fail(c"cannot use both 'buf' and 'win'");
        return None;
    }

    // SAFETY: `name` is the caller's C string.
    let opt_idx = find_option(unsafe { CStr::from_ptr(name) });
    if opt_idx == kOptInvalid {
        // SAFETY: `err` is the caller's and the format takes `name`.
        unsafe {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Unknown option '%s'".as_ptr(),
                name,
            )
        };
    } else if (scope == kOptScopeBuf || scope == kOptScopeWin) && !option_has_scope(opt_idx, scope)
    {
        let tgt = if scope == kOptScopeBuf {
            c"buf"
        } else {
            c"win"
        };
        let global = if option_has_scope(opt_idx, kOptScopeGlobal) {
            c"global "
        } else {
            c""
        };
        let req = if option_has_scope(opt_idx, kOptScopeBuf) {
            c"buffer-local "
        } else if option_has_scope(opt_idx, kOptScopeWin) {
            c"window-local "
        } else {
            c""
        };
        // SAFETY: `err` is the caller's; three static strings and `name`.
        unsafe {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"'%s' cannot be passed for %s%soption '%s'".as_ptr(),
                tgt.as_ptr(),
                global.as_ptr(),
                req.as_ptr(),
                name,
            );
        }
    }
    // SAFETY: `err` is the caller's.
    if unsafe { (*err).type_0 } != kErrorTypeNone {
        return None;
    }

    // SAFETY: `opts` is the caller's; the key borrows its bytes.
    let filetype = match set(OPTIDX_FILETYPE) {
        true => unsafe { (*opts).filetype.data() },
        false => ptr::null_mut(),
    };
    Some(OptionTarget {
        opt_idx,
        opt_flags,
        scope,
        from,
        filetype,
    })
}

/// A scratch buffer of type `filetype`, with its `FileType` autocommands
/// already run, so that an option can be read as such a buffer would see it.
///
/// The caller gets the buffer back even when this fails part way, because it
/// still has to be wiped; `aco` says whether the autocommand window was
/// entered and so has to be left again.
///
/// # Safety
/// `filetype` must be null or a C string, `aco` and `aco_used` must be the
/// caller's, and `err` must be the caller's error slot.
unsafe fn do_ft_buf(
    filetype: *const c_char,
    aco: *mut aco_save_T,
    aco_used: *mut bool,
    err: *mut Error,
) -> *mut buf_T {
    // SAFETY: `aco_used` is the caller's out-parameter.
    unsafe { *aco_used = false };
    if filetype.is_null() {
        return ptr::null_mut::<buf_T>();
    }
    // SAFETY: `err` is the caller's; the format takes nothing.
    let fail = move |msg: &CStr| unsafe { api_set_error(err, kErrorTypeException, msg.as_ptr()) };

    // SAFETY: a dummy buffer of no name, which owns everything it holds.
    let ftbuf = unsafe { buflist_new(ptr::null_mut(), ptr::null_mut(), 1 as linenr_T, BLN_DUMMY) };
    if ftbuf.is_null() {
        fail(c"Could not create internal buffer");
        return ptr::null_mut::<buf_T>();
    }
    // SAFETY: `ftbuf` is the buffer just created.
    if unsafe { ml_open(ftbuf) } == FAIL {
        fail(c"Could not load internal buffer");
        return ftbuf;
    }
    let mut bufref: bufref_T = bufref_T::default();
    // SAFETY: `bufref` is this frame's own, `aco` the caller's, and `ftbuf`
    // is live until it is wiped.
    unsafe {
        set_bufref(&raw mut bufref, ftbuf);
        aucmd_prepbuf(aco, ftbuf);
        *aco_used = true;
    }
    // 'bufhidden' and 'buftype' keep the scratch buffer out of everything the
    // user can see; both are set without autocommands, as `:setlocal` would.
    set_option_direct(
        kOptBufhidden,
        static_option(c"hide"),
        OptionSetFlags::LOCAL,
        SID_NONE,
    );
    set_option_direct(
        kOptBuftype,
        static_option(c"nofile"),
        OptionSetFlags::LOCAL,
        SID_NONE,
    );
    // SAFETY: `ftbuf` is the live scratch buffer; `ml_open` gave it a memfile.
    unsafe {
        debug_assert!(
            (*(*ftbuf).b_ml.ml_mfp).mf_fd < 0,
            "ftbuf->b_ml.ml_mfp->mf_fd < 0"
        );
        (*ftbuf).b_p_swf = 0;
        (*ftbuf).b_p_ml = 0;
        (*ftbuf).b_p_ft = xstrdup(filetype);
    }
    // SAFETY: the autocommand tables are the editor's own.
    if !has_event(EVENT_FILETYPE) {
        return ftbuf;
    }
    // SAFETY: `ftbuf` is live; the autocommands may delete it, which the
    // `bufref` below re-checks.
    // SAFETY: `err` is the caller's.
    let did_au_ft = api_try(unsafe { &mut *err }, |_| {
        do_filetype_autocmd(unsafe { Buf::new(ftbuf) }, true)
    });
    // SAFETY: `bufref` is this frame's own.
    if !unsafe { bufref_valid(&raw mut bufref) } {
        // SAFETY: `err` is the caller's.
        if unsafe { (*err).type_0 } == kErrorTypeNone {
            fail(c"Internal buffer was deleted");
        }
        return ptr::null_mut::<buf_T>();
    }
    // SAFETY: `err` is the caller's.
    if !did_au_ft && unsafe { (*err).type_0 } == kErrorTypeNone {
        fail(c"Could not execute FileType autocommands");
    }
    ftbuf
}

/// An `OptVal` borrowing the static string `text`, for the two option writes
/// `do_ft_buf` makes: `set_option_direct` copies what it is given.
fn static_option(text: &'static CStr) -> OptVal {
    OptVal {
        type_0: kOptValTypeString,
        data: OptValData {
            string: String_0::from_cstr(text),
        },
    }
}

/// Take the scratch buffer `do_ft_buf` made back out of existence.
///
/// # Safety
/// `buf` must be a live buffer.
unsafe fn wipe_ft_buf(buf: *mut buf_T) {
    // SAFETY: `buf` is the caller's live buffer; `bufref` is this frame's own
    // and re-checks it after each step that can delete it.
    unsafe {
        block_autocmds();
        let mut bufref: bufref_T = bufref_T::default();
        set_bufref(&raw mut bufref, buf);
        close_windows(buf, false);
        if bufref_valid(&raw mut bufref) && buf != curbuf.get() && (*buf).b_nwindows == 0 {
            wipe_buffer(Buf::new(buf), false);
        }
        if bufref_valid(&raw mut bufref) {
            (*buf).b_flags.clear(BufFlags::DUMMY);
        }
        unblock_autocmds();
    }
}

/// The value of option `name`, at whatever scope `opts` names.
///
/// # Safety
/// `name` must point at its own bytes and `opts` at a filled-in
/// `KeyDict_option`.
pub unsafe fn nvim_get_option_value(
    name: String_0,
    opts: *mut KeyDict_option,
) -> Result<Object, Error> {
    let mut err = ERROR_INIT;
    // SAFETY: `name` and `opts` are the caller's, per this function's
    // contract, and `err` is this frame's own.
    let Some(target) = (unsafe { option_target(opts, name.data(), &raw mut err) }) else {
        return NIL.reported(err);
    };

    let mut aco: aco_save_T = aco_save_T::default();
    let mut aco_used: bool = false;
    // SAFETY: `aco` and `aco_used` are this frame's own; `target.filetype`
    // borrows `opts`, which outlives the call.
    let ftbuf = unsafe {
        do_ft_buf(
            target.filetype,
            &raw mut aco,
            &raw mut aco_used,
            &raw mut err,
        )
    };
    // SAFETY: `aco` is this frame's own and `ftbuf` is the scratch buffer.
    let mut leave_ft_buf = |ftbuf: *mut buf_T| unsafe {
        if aco_used {
            aucmd_restbuf(&raw mut aco);
        }
        if !ftbuf.is_null() {
            wipe_ft_buf(ftbuf);
        }
    };
    if err.type_0 != kErrorTypeNone {
        leave_ft_buf(ftbuf);
        return NIL.reported(err);
    }

    // A filetype cannot be combined with `buf` or `win`, so `from` is null
    // wherever the scratch buffer exists.
    let from = if ftbuf.is_null() {
        target.from
    } else {
        debug_assert!(target.from.is_null(), "!from");
        ftbuf.cast::<c_void>()
    };
    // SAFETY: `from` is null or the live object `scope` names, and `err` is
    // this frame's own.
    let value = unsafe {
        get_option_value_for(
            target.opt_idx,
            target.opt_flags,
            target.scope,
            from,
            &raw mut err,
        )
    };
    if !ftbuf.is_null() {
        leave_ft_buf(ftbuf);
    }
    if err.type_0 == kErrorTypeNone {
        if value.type_0 != kOptValTypeNil {
            return optval_as_object(value).reported(err);
        }
        // SAFETY: `err` is this frame's own and `name` is the caller's.
        unsafe {
            api_err_invalid(
                &raw mut err,
                c"option".as_ptr(),
                name.data(),
                0 as int64_t,
                true,
            )
        };
    }
    optval_free(value);
    NIL.reported(err)
}

/// Set option `name` to `value`, at whatever scope `opts` names.
///
/// # Safety
/// `name` and `value` must own their bytes, and `opts` must point at a
/// filled-in `KeyDict_option`.
pub unsafe fn nvim_set_option_value(
    channel_id: uint64_t,
    name: String_0,
    value: Object,
    opts: *mut KeyDict_option,
) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    // SAFETY: as `nvim_get_option_value`.
    let Some(target) = (unsafe { option_target(opts, name.data(), &raw mut err) }) else {
        return ().reported(err);
    };
    // Setting a window-local option without saying local or global writes the
    // local value, where *reading* one falls back to the global.
    let mut opt_flags = target.opt_flags;
    if target.scope == kOptScopeWin
        && opt_flags.is_empty()
        && option_has_scope(target.opt_idx, kOptScopeGlobal)
    {
        opt_flags = OptionSetFlags::LOCAL;
    }
    let Some(optval) = object_as_optval(value) else {
        // SAFETY: `err` is this frame's own; the strings are static.
        unsafe {
            let got = api_typename(value.type_0);
            api_err_exp(
                &raw mut err,
                c"value".as_ptr(),
                c"valid option type".as_ptr(),
                got,
            );
        }
        return ().reported(err);
    };
    // Whoever made this API call owns the write, so that `:verbose set` names
    // them rather than whatever ran last.
    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
    // SAFETY: `name` is the caller's, `target.from` is null or the live
    // object `scope` names, and `err` is this frame's own.
    unsafe {
        set_option_value_for(
            name.data(),
            target.opt_idx,
            optval,
            opt_flags,
            target.scope,
            target.from,
            &raw mut err,
        );
    }
    current_sctx.set(save_current_sctx);
    ().reported(err)
}

/// Every option's metadata, keyed by name.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_get_all_options_info(arena: *mut Arena) -> Dict {
    // SAFETY: `arena` is the caller's, per this function's contract.
    unsafe { get_all_vimoptions(arena) }
}

/// Option `name`'s metadata, as seen at whatever scope `opts` names.
///
/// # Safety
/// `name` must point at its own bytes, `opts` at a filled-in
/// `KeyDict_option`, and `arena` must be the caller's.
pub unsafe fn nvim_get_option_info2(
    name: String_0,
    opts: *mut KeyDict_option,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut err = ERROR_INIT;
    // SAFETY: as `nvim_get_option_value`.
    let Some(target) = (unsafe { option_target(opts, name.data(), &raw mut err) }) else {
        return Dict::EMPTY.reported(err);
    };
    // The metadata is read off a buffer and a window whatever the scope, so
    // the two the caller did not name default to the current ones.
    let buf = match target.scope == kOptScopeBuf {
        true => target.from.cast::<buf_T>(),
        false => curbuf.get(),
    };
    let win = match target.scope == kOptScopeWin {
        true => target.from.cast(),
        false => curwin.get(),
    };
    // SAFETY: `buf` and `win` are live, `name` and `arena` are the caller's,
    // and `err` is this frame's own.
    let info = unsafe { get_vimoption(name, target.opt_flags, buf, win, arena, &raw mut err) };
    info.reported(err)
}
