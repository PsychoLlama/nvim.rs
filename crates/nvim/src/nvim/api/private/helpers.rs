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

use core::ffi::{VaList, c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::api::private::validate::api_err_invalid;
use crate::src::nvim::ex_eval::{
    discard_current_exception, free_global_msglist, get_exception_string,
};
use crate::src::nvim::highlight_group::syn_id2name;
use crate::src::nvim::main::{
    buffer_handles, curbuf, current_exception, current_sctx, curtab, curwin, did_emsg, did_throw,
    force_abort, got_int, msg_list, need_rethrow, tabpage_handles, trylevel, window_handles,
};
use crate::src::nvim::map::mh_get_int;
use crate::src::nvim::mark::setmark_pos;
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::os::libc::vsnprintf;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::runtime::script_is_lua;
use crate::src::nvim::types::{
    Buffer, Dict, Error, ErrorType, HlMessage, Integer, Map_int_ptr_t, Object, String_0, Tabpage,
    TryState, Window, buf_T, colnr_T, except_type_T, fmarkv_T, int64_t, kErrorTypeException,
    kErrorTypeNone, kObjectTypeNil, linenr_T, msglist_T, object, object_data, pos_T, ptr_t, scid_T,
    sctx_T, size_t, tabpage_T, uint32_t, uint64_t, win_T,
};

mod keydict;
mod text;
mod value;
mod vimdict;

pub(crate) use self::keydict::*;
pub(crate) use self::text::*;
pub(crate) use self::value::*;
pub(crate) use self::vimdict::*;

const ET_ERROR: except_type_T = 1;

/// `dictitem_T.di_flags`: the key cannot be changed, cannot be changed right
/// now, and cannot be removed.
const DI_FLAGS_RO: c_int = 1;
const DI_FLAGS_FIX: c_int = 4;
const DI_FLAGS_LOCK: c_int = 8;

/// The highlight group an error chunk gets when the caller named none.

/// The hash slot `mh_get_int` reports for a key it did not find.
const MH_TOMBSTONE: uint32_t = u32::MAX;

const NUL: c_char = 0;
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

const STRING_INIT: String_0 = String_0 {
    data: ptr::null_mut(),
    size: 0,
};

const NIL: Object = object {
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
// -- Handles ---------------------------------------------------------------

/// `map_get(int, ptr_t)`: what `key` maps to, or null when it maps to
/// nothing. The C macro reads a per-value-type "default" global that nothing
/// ever writes, so a miss is always null.
unsafe fn map_get_ptr(map: *mut Map_int_ptr_t, key: c_int) -> ptr_t {
    // SAFETY: the caller passes one of the three handle maps, which are
    // initialised before any API call can run.
    unsafe {
        let slot = mh_get_int(&raw mut (*map).set, key);
        if slot == MH_TOMBSTONE {
            ptr::null_mut()
        } else {
            *(*map).values.add(slot as usize)
        }
    }
}

/// The buffer `buffer` names, or the current one for 0. Null — with `err`
/// set — when it names nothing.
pub(crate) unsafe fn find_buffer_by_handle(buffer: Buffer, err: *mut Error) -> *mut buf_T {
    // SAFETY: `err` is the caller's out-parameter.
    unsafe {
        if buffer == 0 {
            return curbuf.get();
        }
        let rv = map_get_ptr(buffer_handles.ptr(), buffer) as *mut buf_T;
        if rv.is_null() {
            api_err_invalid(
                err,
                c"buffer id".as_ptr(),
                ptr::null(),
                buffer as int64_t,
                false,
            );
        }
        rv
    }
}

/// [`find_buffer_by_handle`] for a window.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn find_window_by_handle(window: Window, err: *mut Error) -> *mut win_T {
    // SAFETY: `err` is the caller's out-parameter.
    unsafe {
        if window == 0 {
            return curwin.get();
        }
        let rv = map_get_ptr(window_handles.ptr(), window) as *mut win_T;
        if rv.is_null() {
            api_err_invalid(
                err,
                c"window id".as_ptr(),
                ptr::null(),
                window as int64_t,
                false,
            );
        }
        rv
    }
}

/// [`find_buffer_by_handle`] for a tab page.
pub(crate) unsafe fn find_tab_by_handle(tabpage: Tabpage, err: *mut Error) -> *mut tabpage_T {
    // SAFETY: `err` is the caller's out-parameter.
    unsafe {
        if tabpage == 0 {
            return curtab.get();
        }
        let rv = map_get_ptr(tabpage_handles.ptr(), tabpage) as *mut tabpage_T;
        if rv.is_null() {
            api_err_invalid(
                err,
                c"tabpage id".as_ptr(),
                ptr::null(),
                tabpage as int64_t,
                false,
            );
        }
        rv
    }
}

// -- Errors and the try/catch bracket --------------------------------------

/// Start catching what Vimscript throws, saving the state to put back into
/// `tstate`. Pairs with [`try_leave`].
pub(crate) unsafe fn try_enter(tstate: *mut TryState) {
    // SAFETY: `tstate` is the caller's, and lives until `try_leave`.
    unsafe {
        *tstate = TryState {
            current_exception: current_exception.get(),
            private_msg_list: ptr::null_mut(),
            msg_list: msg_list.get() as *const *const msglist_T,
            got_int: got_int.get() as c_int,
            did_throw: did_throw.get(),
            need_rethrow: need_rethrow.get() as c_int,
            did_emsg: did_emsg.get(),
        };
        // Errors go to the caller's own list from here on, so that an
        // `:echoerr` inside the call does not reach an enclosing `:try`.
        msg_list.set(&raw mut (*tstate).private_msg_list);
        current_exception.set(ptr::null_mut());
        got_int.set(false);
        did_throw.set(false);
        need_rethrow.set(false);
        did_emsg.set(0);
        (*trylevel.ptr()) += 1;
    }
}

/// Stop catching, report whatever was caught through `err`, and restore what
/// [`try_enter`] saved into `tstate`.
pub(crate) unsafe fn try_leave(tstate: *const TryState, err: *mut Error) {
    // SAFETY: `tstate` is what the matching `try_enter` filled in.
    unsafe {
        assert!(trylevel.get() > 0);
        (*trylevel.ptr()) -= 1;
        did_emsg.set(0);
        force_abort.set(false);

        if got_int.get() {
            // An interrupt outranks anything that was thrown along the way.
            if did_throw.get() {
                discard_current_exception();
            }
            api_set_error(err, kErrorTypeException, c"Keyboard interrupt".as_ptr());
            got_int.set(false);
        } else if !msg_list.get().is_null() && !(*msg_list.get()).is_null() {
            let mut should_free = false;
            let msg = get_exception_string(
                *msg_list.get() as *mut c_void,
                ET_ERROR,
                ptr::null_mut(),
                &raw mut should_free,
            );
            api_set_error(err, kErrorTypeException, c"%s".as_ptr(), msg);
            free_global_msglist();
            if should_free {
                xfree(msg.cast());
            }
        } else if did_throw.get() || need_rethrow.get() {
            let ex = current_exception.get();
            if *(*ex).throw_name != NUL {
                if (*ex).throw_lnum != 0 {
                    let fmt = c"%s, line %d: %s".as_ptr();
                    api_set_error(
                        err,
                        kErrorTypeException,
                        fmt,
                        (*ex).throw_name,
                        (*ex).throw_lnum,
                        (*ex).value,
                    );
                } else {
                    let fmt = c"%s: %s".as_ptr();
                    api_set_error(err, kErrorTypeException, fmt, (*ex).throw_name, (*ex).value);
                }
            } else {
                api_set_error(err, kErrorTypeException, c"%s".as_ptr(), (*ex).value);
            }
            discard_current_exception();
        }

        msg_list.set((*tstate).msg_list as *mut *mut msglist_T);
        current_exception.set((*tstate).current_exception);
        got_int.set((*tstate).got_int != 0);
        did_throw.set((*tstate).did_throw);
        need_rethrow.set((*tstate).need_rethrow != 0);
        did_emsg.set((*tstate).did_emsg);
    }
}

/// Set `err` from a printf-style message. The message is measured first and
/// then formatted, so it is never truncated below 1 MiB.
pub(crate) unsafe extern "C" fn api_set_error(
    err: *mut Error,
    err_type: ErrorType,
    format: *const c_char,
    mut args: ...
) {
    // SAFETY: `format` and the variadic arguments are the caller's, and are
    // a valid printf call by construction — every call site is in-tree.
    unsafe {
        assert!(err_type != kErrorTypeNone);
        let measure: VaList = args.clone();
        let write: VaList = args.clone();
        let len = vsnprintf(ptr::null_mut(), 0, format, measure);
        assert!(len >= 0);
        let bufsize = (len as size_t + 1).min(1024 * 1024);
        (*err).msg = xmalloc(bufsize).cast();
        vsnprintf((*err).msg, bufsize, format, write);
        (*err).type_0 = err_type;
    }
}

/// Free `err`'s message and mark it as carrying no error.
pub(crate) unsafe fn api_clear_error(value: *mut Error) {
    // SAFETY: `value` is the caller's error slot.
    unsafe {
        if (*value).type_0 == kErrorTypeNone {
            return;
        }
        xfree((*value).msg.cast());
        (*value).msg = ptr::null_mut();
        (*value).type_0 = kErrorTypeNone;
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
    err: *mut Error,
) -> bool {
    // SAFETY: `name` names one character, and `buf`/`err` are the caller's.
    unsafe {
        let buf = if buf.is_null() { curbuf.get() } else { buf };
        let mut col = col;
        let mut deleting = false;
        if line == 0 {
            col = 0;
            deleting = true;
        } else {
            if col > MAXCOL as Integer {
                api_err_invalid(err, c"column".as_ptr(), c"out of range".as_ptr(), 0, false);
                return false;
            }
            if line < 1 || line > (*buf).b_ml.ml_line_count as Integer {
                api_err_invalid(err, c"line".as_ptr(), c"out of range".as_ptr(), 0, false);
                return false;
            }
        }
        assert!((i32::MIN as Integer..=i32::MAX as Integer).contains(&line));

        let mut pos = pos_T {
            lnum: line as linenr_T,
            col: col as colnr_T,
            coladd: 0,
        };
        let mark = *name.data as c_int;
        let res = setmark_pos(
            mark,
            &raw mut pos,
            (*buf).handle,
            ptr::null_mut::<fmarkv_T>(),
        ) != 0;
        if !res {
            let fmt = if deleting {
                c"Failed to delete named mark: %c".as_ptr()
            } else {
                c"Failed to set named mark: %c".as_ptr()
            };
            api_set_error(err, kErrorTypeException, fmt, mark);
        }
        res
    }
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
/// and `<sfile>` name them, and return what it was pointing at.
pub(crate) fn api_set_sctx(channel_id: uint64_t) -> sctx_T {
    let old_current_sctx = current_sctx.get();
    // SAFETY: `script_is_lua` takes a script id, not a pointer.
    unsafe {
        // A call from Vimscript is already running in the right context.
        if channel_id != VIML_INTERNAL_CALL {
            (*current_sctx.ptr()).sc_lnum = 0;
            if channel_id == LUA_INTERNAL_CALL {
                // Unless the caller is a Lua script, which keeps its own id.
                if !script_is_lua((*current_sctx.ptr()).sc_sid) {
                    (*current_sctx.ptr()).sc_sid = SID_LUA;
                }
            } else {
                (*current_sctx.ptr()).sc_sid = SID_API_CLIENT;
                (*current_sctx.ptr()).sc_chan = channel_id;
            }
        }
    }
    old_current_sctx
}
