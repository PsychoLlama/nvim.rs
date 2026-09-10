//! `nvim_*_option_*`: reading and writing options through the API.
//!
//! Every entry point here takes the same `opts` dictionary -- `scope`, `buf`,
//! `win`, `filetype` -- and the first thing each does is turn it into the
//! three things the option layer actually wants: which option, at what scope,
//! and on which buffer or window. That is [`OptionTarget`], and
//! [`option_target`] is the one place the dictionary's rules are enforced.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::api::private::helpers::{
    Reported, api_set_sctx, api_try, api_typename, find_buffer_by_handle, find_window_by_handle,
    has_key,
};
use crate::autocmd::{
    aucmd_prepbuf, aucmd_restbuf, block_autocmds, do_filetype_autocmd, has_event, unblock_autocmds,
};
use crate::buffer::{BufFlags, BufRef, buflist_new, wipe_buffer};
use crate::options::{kOptBufhidden, kOptBuftype, kOptInvalid};
use crate::types::AutoEvent;
use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int, c_void};

use crate::api::private::validate::{err_bad_value, err_expected};
use crate::api_error;
use crate::memline::ml_open;
use crate::memory::xstrdup;
use crate::message_fmt::{c_str, msg_cstr};
use crate::option::{
    find_option, get_all_vimoptions, get_option_value_for, get_vimoption, object_as_optval,
    option_has_scope, optval_as_object, optval_free, set_option_direct, set_option_value_for,
};
use crate::types::{
    AcoSave, ApiDict, Arena, Buffer, Error, KeyDict_option, LineNr, Object, OptIndex, OptScope,
    OptVal, OptionSetFlags, String_0, kErrorTypeValidation, uint64_t,
};
use crate::window::close_windows;
use crate::winlayer::Buf;
use core::ptr;

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
    /// The `Buffer` or `Window` the scope names -- `get_option_value_for` and
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
/// string.
unsafe fn option_target(
    opts: *mut KeyDict_option,
    name: *mut c_char,
) -> Result<OptionTarget, Error> {
    // `opts`' keys, by their index in its `is_set` mask. Function-local so
    // that they cannot collide in the flat namespace `tools/ffigen` renders
    // module-level constants into.
    const OPTIDX_BUF: c_int = 1;
    const OPTIDX_WIN: c_int = 2;
    const OPTIDX_SCOPE: c_int = 3;
    const OPTIDX_FILETYPE: c_int = 4;

    // SAFETY: `opts` is the caller's, per this function's contract.
    let set = move |key| unsafe { has_key((*opts).is_set__option_, key) };
    let mut opt_flags = OptionSetFlags::NONE;
    if set(OPTIDX_SCOPE) {
        // SAFETY: as above; `scope` is a NUL-terminated key of `opts`.
        let scope = unsafe { CStr::from_ptr((*opts).scope.data()) };
        opt_flags = match scope.to_bytes() {
            b"local" => OptionSetFlags::LOCAL,
            b"global" => OptionSetFlags::GLOBAL,
            _ => return Err(err_expected(c"scope", c"'local' or 'global'", None)),
        };
    }

    let mut scope = kOptScopeGlobal;
    let mut from = ptr::null_mut::<c_void>();
    if set(OPTIDX_WIN) {
        scope = kOptScopeWin;
        // SAFETY: the handle is an integer.
        let win = unsafe { find_window_by_handle((*opts).win) }?;
        from = win.map_or(ptr::null_mut(), |w| w.raw().cast());
    }
    if set(OPTIDX_BUF) {
        if set(OPTIDX_SCOPE) && opt_flags == OptionSetFlags::GLOBAL {
            let why = c"cannot use both global 'scope' and 'buf'";
            return Err(Error::validation(why));
        }
        opt_flags = OptionSetFlags::LOCAL;
        scope = kOptScopeBuf;
        // SAFETY: as the window lookup above.
        let buf = unsafe { find_buffer_by_handle((*opts).buf) }?;
        from = buf.map_or(ptr::null_mut(), |b| b.raw().cast());
    }
    if set(OPTIDX_FILETYPE) && (set(OPTIDX_BUF) || set(OPTIDX_SCOPE) || set(OPTIDX_WIN)) {
        let why = c"cannot use 'filetype' with 'scope', 'buf' or 'win'";
        return Err(Error::validation(why));
    }
    if set(OPTIDX_WIN) && set(OPTIDX_BUF) {
        return Err(Error::validation(c"cannot use both 'buf' and 'win'"));
    }

    // SAFETY: `name` is the caller's C string.
    let opt_idx = find_option(unsafe { CStr::from_ptr(name) });
    if opt_idx == kOptInvalid {
        // SAFETY: `name` is the caller's C string.
        let name = unsafe { c_str(name) };
        return Err(api_error!(kErrorTypeValidation, "Unknown option '{name}'"));
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
        let (tgt, global, req) = (msg_cstr(tgt), msg_cstr(global), msg_cstr(req));
        // SAFETY: `name` is the caller's C string.
        let name = unsafe { c_str(name) };
        return Err(api_error!(
            kErrorTypeValidation,
            "'{tgt}' cannot be passed for {global}{req}option '{name}'"
        ));
    }

    // SAFETY: `opts` is the caller's; the key borrows its bytes.
    let filetype = match set(OPTIDX_FILETYPE) {
        true => unsafe { (*opts).filetype.data() },
        false => ptr::null_mut(),
    };
    Ok(OptionTarget {
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
/// caller's.
unsafe fn do_ft_buf(
    filetype: *const c_char,
    aco: *mut AcoSave,
    aco_used: *mut bool,
) -> (Option<Buf>, Result<(), Error>) {
    // SAFETY: `aco_used` is the caller's out-parameter.
    unsafe { *aco_used = false };
    if filetype.is_null() {
        return (None, Ok(()));
    }
    // SAFETY: a dummy buffer of no name, which owns everything it holds.
    let made = unsafe { buflist_new(ptr::null_mut(), ptr::null_mut(), 1 as LineNr, BLN_DUMMY) };
    let Some(mut ftbuf) = made else {
        let why = Error::exception(c"Could not create internal buffer");
        return (None, Err(why));
    };
    // SAFETY: `ftbuf` is the buffer just created.
    if unsafe { ml_open(ftbuf) }.is_err() {
        let why = Error::exception(c"Could not load internal buffer");
        return (Some(ftbuf), Err(why));
    }
    // SAFETY: `aco` is the caller's and `ftbuf` is live until it is wiped.
    let bufref = BufRef::of(ftbuf);
    unsafe { aucmd_prepbuf(aco, ftbuf) };
    unsafe { *aco_used = true };
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
    debug_assert!(
        unsafe { (*ftbuf.b_ml.ml_mfp).mf_fd } < 0,
        "ftbuf->b_ml.ml_mfp->mf_fd < 0"
    );
    ftbuf.b_p_swf = 0;
    ftbuf.b_p_ml = 0;
    unsafe { ftbuf.b_p_ft = xstrdup(filetype) };
    // SAFETY: the autocommand tables are the editor's own.
    if !has_event(AutoEvent::FileType) {
        return (Some(ftbuf), Ok(()));
    }
    // The autocommands may delete `ftbuf`, which the `bufref` re-checks.
    let ran = api_try(|| do_filetype_autocmd(ftbuf, true));
    let thrown = ran.as_ref().err().is_some();
    if !bufref.valid() {
        let why = match ran {
            Err(e) => e,
            Ok(_) => Error::exception(c"Internal buffer was deleted"),
        };
        return (None, Err(why));
    }
    let why = match ran {
        Err(e) => Err(e),
        Ok(false) if !thrown => Err(Error::exception(c"Could not execute FileType autocommands")),
        Ok(_) => Ok(()),
    };
    (Some(ftbuf), why)
}

/// An `OptVal` borrowing the static string `text`, for the two option writes
/// `do_ft_buf` makes: `set_option_direct` copies what it is given.
fn static_option(text: &'static CStr) -> OptVal {
    OptVal::String(String_0::from_cstr(text))
}

/// Take the scratch buffer `do_ft_buf` made back out of existence.
///
/// # Safety
/// `buffer` must be a live buffer.
unsafe fn wipe_ft_buf(mut buffer: Buf) {
    block_autocmds();
    let bufref = BufRef::of(buffer);
    close_windows(buffer, false);
    if bufref.valid() && buffer != Buf::current() && buffer.b_nwindows == 0 {
        wipe_buffer(buffer, false);
    }
    if bufref.valid() {
        buffer.b_flags.clear(BufFlags::DUMMY);
    }
    unblock_autocmds();
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
    // SAFETY: `name` and `opts` are the caller's, per this function's
    // contract.
    let target = unsafe { option_target(opts, name.data()) }?;

    let mut aco: AcoSave = AcoSave::default();
    let mut aco_used: bool = false;
    let (paco, pused) = (&raw mut aco, &raw mut aco_used);
    // SAFETY: `aco` and `aco_used` are this frame's own; `target.filetype`
    // borrows `opts`, which outlives the call. The buffer comes back even
    // when the call refused, because it still has to be wiped.
    let (ftbuf, made) = unsafe { do_ft_buf(target.filetype, paco, pused) };
    // SAFETY: `aco` is this frame's own and `ftbuf` is the scratch buffer.
    let mut leave_ft_buf = |ftbuf: Option<Buf>| unsafe {
        if aco_used {
            aucmd_restbuf(&raw mut aco);
        }
        if let Some(ftbuf) = ftbuf {
            wipe_ft_buf(ftbuf);
        }
    };
    if let Err(e) = made {
        leave_ft_buf(ftbuf);
        return Err(e);
    }

    // A filetype cannot be combined with `buf` or `win`, so `from` is null
    // wherever the scratch buffer exists.
    let from = match ftbuf {
        None => target.from,
        Some(ftbuf) => {
            debug_assert!(target.from.is_null(), "!from");
            ftbuf.raw().cast::<c_void>()
        }
    };
    let (idx, flags, scope) = (target.opt_idx, target.opt_flags, target.scope);
    // SAFETY: `from` is null or the live object `scope` names.
    let read = unsafe { get_option_value_for(idx, flags, scope, from) };
    if ftbuf.is_some() {
        leave_ft_buf(ftbuf);
    }
    let value = read?;
    if !value.is_nil() {
        return Ok(optval_as_object(value));
    }
    optval_free(value);
    // SAFETY: the caller's option name is NUL-terminated.
    Err(err_bad_value(c"option", unsafe { name.as_cstr() }))
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
    // SAFETY: as `nvim_get_option_value`.
    let target = unsafe { option_target(opts, name.data()) }?;
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
        let got = api_typename(value.kind());
        return Err(err_expected(c"value", c"valid option type", Some(got)));
    };
    // Whoever made this API call owns the write, so that `:verbose set` names
    // them rather than whatever ran last.
    let _sctx = api_set_sctx(channel_id);
    let (key, idx) = (name.data(), target.opt_idx);
    let (scope, from) = (target.scope, target.from);
    // SAFETY: `name` is the caller's, and `target.from` is null or the live
    // object `scope` names.
    unsafe { set_option_value_for(key, idx, optval, opt_flags, scope, from) }
}

/// Every option's metadata, keyed by name.
///
/// # Safety
/// `arena` must be the caller's, and live for as long as the answer is.
pub unsafe fn nvim_get_all_options_info(arena: *mut Arena) -> ApiDict {
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
) -> Result<ApiDict, Error> {
    let mut err = Error::none();
    // SAFETY: as `nvim_get_option_value`.
    let target = unsafe { option_target(opts, name.data()) }?;
    // The metadata is read off a buffer and a window whatever the scope, so
    // the two the caller did not name default to the current ones.
    // SAFETY: `option_target` answers the live buffer or window the scope
    // names, and `curbuf`/`curwin` stand in for the other.
    let buf = match target.scope == kOptScopeBuf {
        true => unsafe { Buf::new(target.from.cast::<Buffer>()) },
        false => Buf::current(),
    };
    // SAFETY: as above.
    let win = match target.scope == kOptScopeWin {
        true => unsafe { Win::new(target.from.cast()) },
        false => Win::current(),
    };
    // SAFETY: `buf` and `win` are live, `name` and `arena` are the caller's,
    // and `err` is this frame's own.
    let info = unsafe { get_vimoption(name, target.opt_flags, buf, win, arena, &mut err) };
    info.reported(err)
}
