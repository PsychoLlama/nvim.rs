//! The plumbing every `nvim_*` function shares. Nothing here is API surface
//! of its own.
//!
//! This file holds the constants the whole family reads, plus the two jobs
//! that are only a few functions each:
//!
//! - **Handles.** Turning a `Buffer`/`Window`/`Tabpage` id from the wire back
//!   into a pointer, or reporting that it names nothing.
//! - **Errors.** An API call reports failure through an `Error`
//!   out-parameter rather than by throwing, so [`try_enter`]/[`try_leave`]
//!   bracket a call that runs Vimscript and turn whatever it threw — an
//!   exception, an `:echoerr`, a `CTRL-C` — into one.
//!
//! The rest is one module per kind of value: [`value`] for the `Object` tree
//! and who owns it, [`text`] for strings and buffer text, [`vimdict`] for
//! Vimscript dictionaries, and [`keydict`] for the generated option structs.
//! They are re-exported here, so callers name `helpers::` as they always
//! did.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::ex_eval::{discard_current_exception, free_global_msglist, get_exception_string};
use crate::guard::{SavedSctx, Script};
use crate::highlight_group::syn_id2name;
use crate::main::{
    curbuf, current_exception, current_sctx, curtab, curwin, did_emsg, did_throw, force_abort,
    got_int, msg_list, need_rethrow, trylevel,
};
use crate::mark::setmark_pos;
use crate::memory::xfree;
use crate::pos::MAXCOL;
use crate::runtime::script_is_lua;
use crate::types::{
    Buffer, Dict, Error, HlMessage, Integer, NUL, Object, String_0, Tabpage, TryState, Window,
    buf_T, colnr_T, except_type_T, fmarkv_T, handle_T, int64_t, kErrorTypeException,
    kObjectTypeNil, linenr_T, msglist_T, object, object_data, pos_T, scid_T, tabpage_T, uint64_t,
    win_T,
};
use crate::winlayer::{self, Buf, TabPage, Win};

mod keydict;
mod text;
mod value;
mod vimdict;

pub(crate) use self::keydict::*;
pub(crate) use self::text::*;
pub(crate) use self::value::*;
// Reached by name from `crates/nvim/tests/unit`, which links the library from
// outside; the rest of `value` stays in-crate.
pub use self::value::api_free_object;
pub(crate) use self::vimdict::*;

const ET_ERROR: except_type_T = 1;

/// `dictitem_T.di_flags`: the key cannot be changed, cannot be changed right
/// now, and cannot be removed.
const DI_FLAGS_RO: c_int = 1;
const DI_FLAGS_FIX: c_int = 4;
const DI_FLAGS_LOCK: c_int = 8;

const NL: c_char = b'\n' as c_char;
const CAR: c_char = b'\r' as c_char;

/// `current_sctx.sc_sid` for a call that came from Lua, and for one that came
/// from an RPC client.
const SID_LUA: scid_T = -8;
const SID_API_CLIENT: scid_T = -9;

/// Channel ids with the top bit set are not channels at all: they mark a call
/// nvim made of itself, from Vimscript or from Lua.
const INTERNAL_CALL_MASK: uint64_t = 1 << (uint64_t::BITS - 1);
const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL + 1;

/// The `Object` that says "no value": upstream's `NIL`.
pub(crate) const NIL: Object = object {
    type_0: kObjectTypeNil,
    data: object_data { boolean: false },
};

const EMPTY_DICT: Dict = Dict {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};

const EMPTY_HL_MESSAGE: HlMessage = HlMessage {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};
use crate::api::private::validate::{Bad, err_bad_number, err_invalid};
use crate::api_error;
use crate::message_fmt::{c_str, msg_bytes};
// -- Handles ---------------------------------------------------------------

/// The buffer with this id, or null. Unlike [`find_buffer_by_handle`] it has
/// no "0 means current" rule and reports nothing: it is upstream's
/// `handle_get_buffer()`, the registry lookup.
pub(crate) fn handle_get_buffer(handle: handle_T) -> *mut buf_T {
    winlayer::buffer(handle).map_or(ptr::null_mut(), Buf::raw)
}

/// [`handle_get_buffer`] for a window.
pub(crate) fn handle_get_window(handle: handle_T) -> *mut win_T {
    winlayer::window(handle).map_or(ptr::null_mut(), Win::raw)
}

/// The buffer `buffer` names, or the current one for 0. Null — with `err`
/// set — when it names nothing.
pub(crate) unsafe fn find_buffer_by_handle(buffer: Buffer, err: &mut Error) -> *mut buf_T {
    if buffer == 0 {
        return curbuf.get();
    }
    let rv = handle_get_buffer(buffer);
    if rv.is_null() {
        let id = buffer as int64_t;
        // SAFETY: the names and values are NUL-terminated strings.
        *err = err_bad_number(c"buffer id", id);
    }
    rv
}

/// [`find_buffer_by_handle`] for a window.
pub unsafe fn find_window_by_handle(window: Window, err: &mut Error) -> *mut win_T {
    if window == 0 {
        return curwin.get();
    }
    let rv = handle_get_window(window);
    if rv.is_null() {
        let id = window as int64_t;
        // SAFETY: the names and values are NUL-terminated strings.
        *err = err_bad_number(c"window id", id);
    }
    rv
}

/// [`find_buffer_by_handle`] for a tab page.
pub(crate) unsafe fn find_tab_by_handle(tabpage: Tabpage, err: &mut Error) -> *mut tabpage_T {
    if tabpage == 0 {
        return curtab.get();
    }
    let rv = winlayer::tabpage(tabpage).map_or(ptr::null_mut(), TabPage::raw);
    if rv.is_null() {
        let id = tabpage as int64_t;
        // SAFETY: the names and values are NUL-terminated strings.
        *err = err_bad_number(c"tabpage id", id);
    }
    rv
}

// -- Handles, as the entry points take them --------------------------------
//
// The three `find_*_by_handle` functions above answer a raw pointer and are
// what the FFI edge still calls. An `nvim_*` entry point wants the same
// lookup as a value it can then use without an `unsafe` block per field, and
// that is what these three give it: the handle is an integer off the wire, so
// nothing about the *call* is unsafe, and the answer is a `winlayer` wrapper
// whose construction discharged the liveness promise once.
//
// `err` is `&mut` rather than a pointer so that the wrappers are safe to call
// and do not trip `clippy::not_unsafe_ptr_arg_deref`.

/// The window `handle` names, or the current one for 0. `None` -- with `err`
/// set, unless there is no current window -- when it names nothing.
pub(crate) fn window_by_handle(handle: Window, err: &mut Error) -> Option<Win> {
    // SAFETY: `err` is the caller's own slot, and the lookup answers a live
    // window or null.
    unsafe { Win::from_raw(find_window_by_handle(handle, err)) }
}

/// [`window_by_handle`] for a buffer.
pub(crate) fn buffer_by_handle(handle: Buffer, err: &mut Error) -> Option<Buf> {
    // SAFETY: as [`window_by_handle`].
    unsafe { Buf::from_raw(find_buffer_by_handle(handle, err)) }
}

/// [`window_by_handle`] for a tab page.
pub(crate) fn tabpage_by_handle(handle: Tabpage, err: &mut Error) -> Option<TabPage> {
    // SAFETY: as [`window_by_handle`].
    unsafe { TabPage::from_raw(find_tab_by_handle(handle, err)) }
}

// -- Errors and the try/catch bracket --------------------------------------

/// Start catching what Vimscript throws, saving the state to put back into
/// `tstate`. Pairs with [`try_leave`].
pub(crate) unsafe fn try_enter(tstate: *mut TryState) {
    let saved = TryState {
        current_exception: current_exception.get(),
        private_msg_list: ptr::null_mut(),
        msg_list: msg_list.get() as *const *const msglist_T,
        got_int: got_int.get() as c_int,
        did_throw: did_throw.get(),
        need_rethrow: need_rethrow.get() as c_int,
        did_emsg: did_emsg.get(),
    };
    // SAFETY: `tstate` is the caller's, and lives until `try_leave`.
    unsafe { *tstate = saved };
    // Errors go to the caller's own list from here on, so that an
    // `:echoerr` inside the call does not reach an enclosing `:try`.
    // SAFETY: as above -- the field's address is the caller's slot plus a
    // constant, live for as long as the state is.
    msg_list.set(unsafe { &raw mut (*tstate).private_msg_list });
    current_exception.set(ptr::null_mut());
    got_int.set(false);
    did_throw.set(false);
    need_rethrow.set(false);
    did_emsg.set(0);
    trylevel.set(trylevel.get() + 1);
}

/// Stop catching, report whatever was caught through `err`, and restore what
/// [`try_enter`] saved into `tstate`.
pub(crate) unsafe fn try_leave(tstate: *const TryState, err: &mut Error) {
    debug_assert!(trylevel.get() > 0);
    trylevel.set(trylevel.get() - 1);
    did_emsg.set(0);
    force_abort.set(false);

    let list = msg_list.get();
    // SAFETY: `msg_list` names a live slot or is null.
    let pending = !list.is_null() && !unsafe { *list }.is_null();

    if got_int.get() {
        // An interrupt outranks anything that was thrown along the way.
        if did_throw.get() {
            // SAFETY: `did_throw` says there is a current exception.
            unsafe { discard_current_exception() };
        }
        *err = Error::exception(c"Keyboard interrupt");
        got_int.set(false);
    } else if pending {
        let mut should_free = false;
        // SAFETY: the slot holds a live message list, and `should_free` is
        // this frame's.
        let msg = unsafe {
            let head = (*list).cast::<c_void>();
            get_exception_string(head, ET_ERROR, ptr::null_mut(), &raw mut should_free)
        };
        // SAFETY: the message is a NUL-terminated string.
        *err = Error::from_message(kErrorTypeException, unsafe { cstr::at(msg) });
        // SAFETY: the list has been rendered into `err`.
        unsafe { free_global_msglist() };
        if should_free {
            // SAFETY: `msg` is the allocation `get_exception_string` made.
            unsafe { xfree(msg.cast()) };
        }
    } else if did_throw.get() || need_rethrow.get() {
        let ex = current_exception.get();
        // SAFETY: either flag says `ex` is the live exception being unwound.
        let (name, lnum, value) = unsafe { ((*ex).throw_name, (*ex).throw_lnum, (*ex).value) };
        // SAFETY: `throw_name` is NUL-terminated, empty for a throw with no
        // script to name.
        let named = unsafe { *name } != NUL as c_char;
        if !named {
            // SAFETY: the message is a NUL-terminated string.
            *err = Error::from_message(kErrorTypeException, unsafe { cstr::at(value) });
        } else {
            // SAFETY: both are the exception's own NUL-terminated strings.
            let (name, value) = unsafe { (c_str(name), c_str(value)) };
            *err = if lnum != 0 {
                api_error!(kErrorTypeException, "{name}, line {lnum}: {value}")
            } else {
                api_error!(kErrorTypeException, "{name}: {value}")
            };
        }
        // SAFETY: the exception has been rendered into `err`.
        unsafe { discard_current_exception() };
    }

    // SAFETY: `tstate` is what the matching `try_enter` filled in.
    let saved = unsafe { *tstate };
    msg_list.set(saved.msg_list as *mut *mut msglist_T);
    current_exception.set(saved.current_exception);
    got_int.set(saved.got_int != 0);
    did_throw.set(saved.did_throw);
    need_rethrow.set(saved.need_rethrow != 0);
    did_emsg.set(saved.did_emsg);
}

/// Run `body` with whatever it throws caught, and report that through `err`:
/// the [`try_enter`]/[`try_leave`] pair as one safe call.
///
/// The `TryState` the two halves communicate through is this frame's own and
/// nothing else can name it, which is the whole of what they ask of a caller
/// -- so every call site that spelled the bracket out by hand was writing
/// eleven lines of `unsafe` to say what this says in one. `try_enter`
/// overwrites the state before `try_leave` reads it, so the value handed in
/// is never read.
///
/// `body` runs between the two, exactly as the statements between a
/// hand-written pair did. It is handed the same slot so that a callee which
/// reports through one of its own can still be given it -- `try_leave` writes
/// over whatever the body left there, which is what a hand-written pair did
/// too.
pub(crate) fn api_try<T>(err: &mut Error, body: impl FnOnce(&mut Error) -> T) -> T {
    let mut tstate = TryState::default();
    // SAFETY: `tstate` is this frame's local, live until `try_leave` below.
    unsafe { try_enter(&raw mut tstate) };
    let value = body(err);
    // SAFETY: `tstate` is what the `try_enter` above filled in, and `err` is
    // the caller's own slot.
    unsafe { try_leave(&raw const tstate, err) };
    value
}

/// Answering with what a helper that still reports through an error
/// out-parameter produced.
pub(crate) trait Reported: Sized {
    /// `self`, unless the lent [`Error`] slot `err` says why the call that
    /// produced it could not answer.
    ///
    /// This is deliberately more forgiving than `?`. A `find_*_by_handle`
    /// miss answers null *without* setting the error when the handle is `0`
    /// and there is no current buffer/window/tabpage, and upstream answers
    /// that with a default value rather than a failure; reading "null means
    /// `Err`" would show the client nil where it used to see `0`. Handing
    /// the value on keeps a converted function exactly as forgiving as the
    /// one it replaced.
    ///
    /// It is postfix, not `reported(value, err)`, for two reasons. Order:
    /// the receiver is evaluated before the argument, so
    /// `f(err).reported(error)` reads the slot *after* the call that
    /// fills it — the prefix spelling reads it before, and silently throws
    /// the failure away. Shape: the value is usually the transpiled body's
    /// existing multi-line call, and a suffix leaves its formatting alone
    /// where a wrapper would re-indent and re-wrap the whole thing.
    fn reported(self, err: Error) -> Result<Self, Error>;
}

impl<T> Reported for T {
    fn reported(self, mut err: Error) -> Result<Self, Error> {
        match err.take() {
            Some(err) => Err(err),
            None => Ok(self),
        }
    }
}

// -- Odds and ends ---------------------------------------------------------

/// Set the mark `name` in `buf` to line/column, or delete it when `line` is
/// 0. False, with `err` set, when the position is out of range or the mark
/// name is not one that can be set.
pub(crate) unsafe fn set_mark(
    buf: *mut buf_T,
    name: String_0,
    line: Integer,
    col: Integer,
    err: &mut Error,
) -> bool {
    let buf = if buf.is_null() { curbuf.get() } else { buf };
    let mut col = col;
    let mut deleting = false;
    let out_of_range = c"out of range".as_ptr();
    if line == 0 {
        col = 0;
        deleting = true;
    } else {
        if col > MAXCOL as Integer {
            // SAFETY: the names and values are NUL-terminated strings.
            *err = err_invalid(c"column", Bad::Bare(unsafe { cstr::at(out_of_range) }));
            return false;
        }
        // SAFETY: `buf` is the caller's buffer, or the current one.
        let line_count = unsafe { (*buf).b_ml.ml_line_count } as Integer;
        if line < 1 || line > line_count {
            // SAFETY: the names and values are NUL-terminated strings.
            *err = err_invalid(c"line", Bad::Bare(unsafe { cstr::at(out_of_range) }));
            return false;
        }
    }
    debug_assert!((i32::MIN as Integer..=i32::MAX as Integer).contains(&line));

    let mut pos = pos_T {
        lnum: line as linenr_T,
        col: col as colnr_T,
        coladd: 0,
    };
    // SAFETY: `name` names one character, per this function's contract.
    let mark = unsafe { *name.data() } as c_int;
    // SAFETY: `buf` is live.
    let handle = unsafe { (*buf).handle };
    let (at, no_view) = (&raw mut pos, ptr::null_mut::<fmarkv_T>());
    // SAFETY: `pos` is this frame's, and the mark is set in `handle`.
    let res = unsafe { setmark_pos(mark, at, handle, no_view) }.is_ok();
    if !res {
        // `%c` wrote the one byte, whatever it was.
        let byte = mark as u8;
        let mark = msg_bytes(core::slice::from_ref(&byte));
        *err = if deleting {
            api_error!(kErrorTypeException, "Failed to delete named mark: {mark}")
        } else {
            api_error!(kErrorTypeException, "Failed to set named mark: {mark}")
        };
    }
    res
}

/// The highlight group a status line, window bar or status column defaults
/// to when its 'statusline' text names none. A null window is the tab line.
pub(crate) fn get_default_stl_hl(
    wp: *mut win_T,
    use_winbar: bool,
    stc_hl_id: c_int,
) -> *const c_char {
    // `wp` is only compared, never followed.
    if wp.is_null() {
        c"TabLineFill".as_ptr()
    } else if use_winbar {
        if wp == curwin.get() {
            c"WinBar".as_ptr()
        } else {
            c"WinBarNC".as_ptr()
        }
    } else if stc_hl_id > 0 {
        syn_id2name(stc_hl_id)
    } else if wp == curwin.get() {
        c"StatusLine".as_ptr()
    } else {
        c"StatusLineNC".as_ptr()
    }
}

/// Point `current_sctx` at whoever made this API call, so that `:verbose`
/// and `<sfile>` name them, for as long as the returned guard lives.
///
/// This is the C's `WITH_SCRIPT_CONTEXT`, whose trailing `current_sctx =
/// save_current_sctx` every caller had to write out by hand. Holding the
/// restore in a [`SavedSctx`] means no way out of the call -- an early
/// return, a `?`, a panic -- can leave the editor believing the API client
/// is still the one running. `nvim_create_augroup` returns early on a bad
/// group name and upstream leaks the client's context out of the call that
/// way; here it cannot.
pub(crate) fn api_set_sctx(channel_id: uint64_t) -> SavedSctx {
    let caller = current_sctx.get();
    // A call from Vimscript is already running in the right context.
    if channel_id == VIML_INTERNAL_CALL {
        return Script::saved();
    }
    let mut sctx = caller.with_lnum(0);
    if channel_id == LUA_INTERNAL_CALL {
        // Unless the caller is a Lua script, which keeps its own id.
        // SAFETY: `script_is_lua` takes a script id, not a pointer.
        if !unsafe { script_is_lua(sctx.sc_sid) } {
            sctx.sc_sid = SID_LUA;
        }
    } else {
        sctx.sc_sid = SID_API_CLIENT;
        sctx.sc_chan = channel_id;
    }
    Script::context(sctx)
}
