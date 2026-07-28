//! The plumbing every `nvim_*` function shares.
//!
//! Four jobs live here, and nothing in this file is API surface of its own:
//!
//! - **Errors.** An API call reports failure through an `Error` out-parameter
//!   rather than by throwing, so [`try_enter`]/[`try_leave`] bracket a call
//!   that runs Vimscript and turn whatever it threw — an exception, an
//!   `:echoerr`, a `CTRL-C` — into one.
//! - **Conversion.** Between the API's `Object` tree and C strings, buffer
//!   text, highlight ids, and the generated "keydict" structs the typed
//!   `nvim_*` signatures take their options as.
//! - **Ownership.** `Object`s are either arena-allocated, and freed in one
//!   go when the arena is, or malloc'd, and freed member by member. The
//!   `arena_*`, `copy_*` and `api_free_*` families are the two halves of
//!   that, and passing a value to the wrong one is a leak or a double free.
//! - **Handles.** Turning a `Buffer`/`Window`/`Tabpage` id from the wire back
//!   into a pointer, or reporting that it names nothing.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{VaList, c_char, c_int, c_void};
use core::{mem, ptr};

use crate::src::nvim::api::private::converter::{object_to_vim, vim_to_object};
use crate::src::nvim::api::private::metadata::PACKED_API_METADATA;
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid};
use crate::src::nvim::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_find, tv_dict_is_watched, tv_dict_item_alloc_len,
    tv_dict_item_remove, tv_dict_watcher_notify,
};
use crate::src::nvim::eval::vars::{before_set_vvar, get_vimvar_dict};
use crate::src::nvim::ex_eval::{
    discard_current_exception, free_global_msglist, get_exception_string,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{highlight_num_groups, syn_check_group, syn_id2name};
use crate::src::nvim::kvec::InitVec;
use crate::src::nvim::lua::executor::{api_free_luaref, api_new_luaref};
use crate::src::nvim::main::{
    buffer_handles, curbuf, current_exception, current_sctx, curtab, curwin, did_emsg, did_throw,
    force_abort, got_int, msg_list, need_rethrow, tabpage_handles, trylevel, window_handles,
};
use crate::src::nvim::map::mh_get_int;
use crate::src::nvim::mark::setmark_pos;
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{
    ARENA_EMPTY, arena_alloc, arena_finish, arena_memdupz, memchrsub, xfree, xmalloc, xmemdupz,
    xrealloc, xstrdup, xstrndup,
};
use crate::src::nvim::message::hl_msg_free;
use crate::src::nvim::msgpack_rpc::unpacker::unpack;
use crate::src::nvim::os::libc::{abort, memcpy, strlen, strnlen, vsnprintf};
use crate::src::nvim::runtime::script_is_lua;
use crate::src::nvim::types::{
    Arena, ArenaMem, Array, ArrayBuilder, Boolean, Buffer, Dict, Error, ErrorType, FieldHashfn,
    Float, HlMessage, HlMessageChunk, Integer, KeySetLink, KeyValuePair, LuaRef, Map_int_ptr_t,
    Object, ObjectType, OptKeySet, OptionalKeys, String_0, Tabpage, TryState, VarLockStatus,
    VarType, Window, buf_T, colnr_T, consumed_blk, dict_T, dictitem_T, except_type_T, fmarkv_T,
    garray_T, handle_T, int64_t, key_value_pair, linenr_T, msglist_T, object, object_data, pos_T,
    ptr_t, ptrdiff_t, scid_T, sctx_T, size_t, tabpage_T, typval_T, typval_vval_union, uint32_t,
    uint64_t, win_T,
};

const kErrorTypeNone: ErrorType = -1;
const kErrorTypeException: ErrorType = 0;
const kErrorTypeValidation: ErrorType = 1;

const kObjectTypeNil: ObjectType = 0;
const kObjectTypeBoolean: ObjectType = 1;
const kObjectTypeInteger: ObjectType = 2;
const kObjectTypeFloat: ObjectType = 3;
const kObjectTypeString: ObjectType = 4;
const kObjectTypeArray: ObjectType = 5;
const kObjectTypeDict: ObjectType = 6;
const kObjectTypeLuaRef: ObjectType = 7;
const kObjectTypeBuffer: ObjectType = 8;
const kObjectTypeWindow: ObjectType = 9;
const kObjectTypeTabpage: ObjectType = 10;

const VAR_UNKNOWN: VarType = 0;
const VAR_UNLOCKED: VarLockStatus = 0;
const ET_ERROR: except_type_T = 1;

/// `dictitem_T.di_flags`: the key cannot be changed, cannot be changed right
/// now, and cannot be removed.
const DI_FLAGS_RO: c_int = 1;
const DI_FLAGS_FIX: c_int = 4;
const DI_FLAGS_LOCK: c_int = 8;

/// The highlight group an error chunk gets when the caller named none.
const HLF_E: c_int = 6;

/// The hash slot `mh_get_int` reports for a key it did not find.
const MH_TOMBSTONE: uint32_t = u32::MAX;

const NUL: c_char = 0;
const NL: c_char = b'\n' as c_char;
const CAR: c_char = b'\r' as c_char;
const MAXLNUM: int64_t = 2147483647;
const MAXCOL: Integer = 2147483647;

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

// -- Vimscript dictionaries ------------------------------------------------

/// The value `key` has in `dict`, as an API object. Nil — with `err` set —
/// when the key is absent.
pub(crate) unsafe fn dict_get_value(
    dict: *mut dict_T,
    key: String_0,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    // SAFETY: `dict` is a live Vimscript dictionary and `key` borrows the
    // caller's text.
    unsafe {
        let di = tv_dict_find(dict, key.data, key.size as ptrdiff_t);
        if di.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Key not found: %s".as_ptr(),
                key.data,
            );
            return NIL;
        }
        vim_to_object(&raw mut (*di).di_tv, arena, true)
    }
}

/// The item `key` names, having first reported through `err` any reason it
/// could not be assigned to (or, with `del`, removed).
///
/// A null return does not mean failure: an absent key is fine for an
/// assignment. Callers check `err`.
pub(crate) unsafe fn dict_check_writable(
    dict: *mut dict_T,
    key: String_0,
    del: bool,
    err: *mut Error,
) -> *mut dictitem_T {
    // SAFETY: as `dict_get_value`.
    unsafe {
        let di = tv_dict_find(dict, key.data, key.size as ptrdiff_t);
        if !di.is_null() {
            let flags = (*di).di_flags as c_int;
            if flags & DI_FLAGS_RO != 0 {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Key is read-only: %s".as_ptr(),
                    key.data,
                );
            } else if flags & DI_FLAGS_LOCK != 0 {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Key is locked: %s".as_ptr(),
                    key.data,
                );
            } else if del && flags & DI_FLAGS_FIX != 0 {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Key is fixed: %s".as_ptr(),
                    key.data,
                );
            }
        } else if (*dict).dv_lock as u64 != 0 {
            api_set_error(err, kErrorTypeException, c"Dict is locked".as_ptr());
        } else if key.size == 0 {
            api_set_error(err, kErrorTypeValidation, c"Key name is empty".as_ptr());
        } else if key.size > c_int::MAX as size_t {
            api_set_error(err, kErrorTypeValidation, c"Key name is too long".as_ptr());
        }
        di
    }
}

/// Set or remove `key` in `dict`. With `retval` the previous value comes
/// back, otherwise nil. Fires the dictionary's watchers either way.
pub(crate) unsafe fn dict_set_var(
    dict: *mut dict_T,
    key: String_0,
    value: Object,
    del: bool,
    retval: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    // SAFETY: as `dict_get_value`.
    unsafe {
        let mut rv = NIL;
        let mut di = dict_check_writable(dict, key, del, err);
        if (*err).type_0 != kErrorTypeNone {
            return rv;
        }
        let watched = tv_dict_is_watched(dict);

        if del {
            if di.is_null() {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Key not found: %s".as_ptr(),
                    key.data,
                );
                return rv;
            }
            if watched {
                tv_dict_watcher_notify(dict, key.data, ptr::null_mut(), &raw mut (*di).di_tv);
            }
            if retval {
                rv = vim_to_object(&raw mut (*di).di_tv, arena, false);
            }
            tv_dict_item_remove(dict, di);
            return rv;
        }

        let mut tv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        object_to_vim(value, &raw mut tv, err);
        // Only filled in for a key that already existed; the watchers see an
        // unset value for a key that did not.
        let mut oldtv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };

        if di.is_null() {
            di = tv_dict_item_alloc_len(key.data, key.size);
            tv_dict_add(dict, di);
        } else {
            if retval {
                rv = vim_to_object(&raw mut (*di).di_tv, arena, false);
            }
            // `v:` keys are typed, and some of them run a hook on assignment.
            let mut type_error = false;
            if dict == get_vimvar_dict()
                && !before_set_vvar(
                    key.data,
                    di,
                    &raw mut tv,
                    true,
                    watched,
                    &raw mut type_error,
                )
            {
                tv_clear(&raw mut tv);
                if type_error {
                    let fmt = c"Setting v:%s to value with wrong type".as_ptr();
                    api_set_error(err, kErrorTypeValidation, fmt, key.data);
                }
                return rv;
            }
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            }
            tv_clear(&raw mut (*di).di_tv);
        }

        tv_copy(&raw mut tv, &raw mut (*di).di_tv);
        if watched {
            tv_dict_watcher_notify(dict, key.data, &raw mut tv, &raw mut oldtv);
            tv_clear(&raw mut oldtv);
        }
        tv_clear(&raw mut tv);
        rv
    }
}

// -- Strings ---------------------------------------------------------------

/// A copy of the C string `str`, owned by the caller.
pub(crate) unsafe fn cstr_to_string(str: *const c_char) -> String_0 {
    // SAFETY: `str` is null or NUL-terminated.
    unsafe {
        if str.is_null() {
            return STRING_INIT;
        }
        cbuf_to_string(str, strlen(str))
    }
}

/// A copy of `size` bytes of `buf`, owned by the caller and NUL-terminated
/// however many NULs the bytes themselves hold.
pub(crate) unsafe fn cbuf_to_string(buf: *const c_char, size: size_t) -> String_0 {
    // SAFETY: `buf` has `size` readable bytes.
    unsafe {
        String_0 {
            data: xmemdupz(buf.cast(), size).cast(),
            size,
        }
    }
}

/// A NUL-terminated copy of `str`'s bytes, owned by the caller.
pub(crate) unsafe fn string_to_cstr(str: String_0) -> *mut c_char {
    // SAFETY: `str` has `size` readable bytes.
    unsafe { xstrndup(str.data, str.size) }
}

/// `str` viewed as an API string, borrowing rather than copying.
pub(crate) unsafe fn cstr_as_string(str: *const c_char) -> String_0 {
    // SAFETY: `str` is null or NUL-terminated.
    unsafe {
        if str.is_null() {
            return STRING_INIT;
        }
        String_0 {
            data: str as *mut c_char,
            size: strlen(str),
        }
    }
}

/// [`cstr_as_string`] for a buffer that need not be NUL-terminated within
/// `maxsize` bytes.
pub(crate) unsafe fn cstrn_as_string(str: *mut c_char, maxsize: size_t) -> String_0 {
    // SAFETY: `str` has `maxsize` readable bytes.
    unsafe {
        String_0 {
            data: str,
            size: strnlen(str, maxsize),
        }
    }
}

/// Take `ga`'s buffer as an API string, leaving the growarray empty.
pub(crate) unsafe fn ga_take_string(ga: *mut garray_T) -> String_0 {
    // SAFETY: `ga` is the caller's growarray of bytes.
    unsafe {
        let str = String_0 {
            data: (*ga).ga_data.cast(),
            size: (*ga).ga_len as size_t,
        };
        (*ga).ga_data = ptr::null_mut();
        (*ga).ga_len = 0;
        (*ga).ga_maxlen = 0;
        str
    }
}

/// Split `input` into one array item per line, arena-allocating the lines.
///
/// Line breaks are `\n`, or `\r` and `\r\n` as well with `crlf`. A NUL in
/// the text stands for a newline, as it does everywhere a buffer line is
/// passed as a C string, and is turned back into one. Text that ends *with*
/// a break gets a trailing empty item, so that the array round-trips.
pub(crate) unsafe fn string_to_array(input: String_0, crlf: bool, arena: *mut Arena) -> Array {
    // SAFETY: `input` has `size` readable bytes.
    unsafe {
        let mut ret: ArrayBuilder = mem::zeroed();
        let mut items = InitVec::new(
            &mut ret.size,
            &mut ret.capacity,
            &mut ret.items,
            &mut ret.init_array,
        );
        items.init();

        let mut i: size_t = 0;
        while i < input.size {
            let start = input.data.add(i);
            let mut end = start;
            let mut line_len: size_t = 0;
            while line_len < input.size - i {
                end = start.add(line_len);
                if *end == NL || (crlf && *end == CAR) {
                    break;
                }
                line_len += 1;
            }
            i += line_len;
            let ends_line = *end == NL || (crlf && *end == CAR);
            if crlf && *end == CAR && i + 1 < input.size && *end.add(1) == NL {
                i += 1;
            }

            let s = arena_string(
                arena,
                String_0 {
                    data: start,
                    size: line_len,
                },
            );
            memchrsub(s.data.cast(), NUL, NL, line_len);
            items.push(object {
                type_0: kObjectTypeString,
                data: object_data { string: s },
            });
            if i + 1 == input.size && ends_line {
                items.push(object {
                    type_0: kObjectTypeString,
                    data: object_data {
                        string: STRING_INIT,
                    },
                });
            }
            i += 1;
        }
        arena_take_arraybuilder(arena, &raw mut ret)
    }
}

// -- Buffer text -----------------------------------------------------------

/// Turn a signed, end-relative line index into a 1-based line number,
/// clamping it into the buffer and reporting through `oob` that it had to.
///
/// `end_exclusive` allows one past the last line, which is what an
/// end-of-range index means.
pub(crate) unsafe fn normalize_index(
    buf: *mut buf_T,
    index: int64_t,
    end_exclusive: bool,
    oob: *mut bool,
) -> int64_t {
    // SAFETY: `buf` is a loaded buffer and `oob` the caller's flag.
    unsafe {
        assert!((*buf).b_ml.ml_line_count > 0);
        let max_index = ((*buf).b_ml.ml_line_count + end_exclusive as linenr_T - 1) as int64_t;
        let mut index = if index < 0 {
            max_index + index + 1
        } else {
            index
        };
        if index > max_index {
            *oob = true;
            index = max_index;
        } else if index < 0 {
            *oob = true;
            index = 0;
        }
        index + 1
    }
}

/// The text of line `lnum` between the two columns, as a *borrowed* string
/// into the buffer's own line. Negative columns count back from the end.
pub(crate) unsafe fn buf_get_text(
    buf: *mut buf_T,
    lnum: int64_t,
    start_col: int64_t,
    end_col: int64_t,
    err: *mut Error,
) -> String_0 {
    // SAFETY: `buf` is a loaded buffer and `err` the caller's error slot.
    unsafe {
        if lnum >= MAXLNUM {
            api_err_invalid(
                err,
                c"line index".as_ptr(),
                c"out of range".as_ptr(),
                0,
                false,
            );
            return STRING_INIT;
        }
        let bufstr = ml_get_buf(buf, lnum as linenr_T);
        let line_length = ml_get_buf_len(buf, lnum as linenr_T) as int64_t;

        let relative = |col: int64_t| if col < 0 { line_length + col + 1 } else { col };
        let start_col = relative(start_col).clamp(0, line_length);
        let end_col = relative(end_col).clamp(0, line_length);
        if start_col > end_col {
            let msg = c"start_col must be less than or equal to end_col".as_ptr();
            api_set_error(err, kErrorTypeValidation, msg);
            return STRING_INIT;
        }
        String_0 {
            data: bufstr.offset(start_col as isize),
            size: (end_col - start_col) as size_t,
        }
    }
}

// -- Arena allocation ------------------------------------------------------

/// An empty array with room for `max_size` items, taken from `arena` — or
/// from the heap when `arena` is null.
pub(crate) fn arena_array(arena: *mut Arena, max_size: size_t) -> Array {
    // SAFETY: `arena_alloc` accepts a null arena and falls back to `xmalloc`.
    let items = unsafe { arena_alloc(arena, mem::size_of::<Object>() * max_size, true) };
    Array {
        size: 0,
        capacity: max_size,
        items: items.cast(),
    }
}

/// [`arena_array`] for a dictionary.
pub(crate) fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict {
    // SAFETY: as `arena_array`.
    let items = unsafe { arena_alloc(arena, mem::size_of::<KeyValuePair>() * max_size, true) };
    Dict {
        size: 0,
        capacity: max_size,
        items: items.cast(),
    }
}

/// A copy of `str` in `arena`, NUL-terminated. The empty string is a shared
/// literal rather than an allocation — but only when there is an arena to
/// outlive it; without one the caller frees what it gets.
pub(crate) unsafe fn arena_string(arena: *mut Arena, str: String_0) -> String_0 {
    // SAFETY: `str` has `size` readable bytes.
    unsafe {
        if str.size != 0 {
            return String_0 {
                data: arena_memdupz(arena, str.data, str.size),
                size: str.size,
            };
        }
        let empty = if arena.is_null() {
            xstrdup(c"".as_ptr())
        } else {
            c"".as_ptr() as *mut c_char
        };
        String_0 {
            data: empty,
            size: 0,
        }
    }
}

/// Move a builder's items into an arena-allocated array of exactly the right
/// size, freeing the builder's own buffer if it had grown onto the heap.
pub(crate) unsafe fn arena_take_arraybuilder(arena: *mut Arena, arr: *mut ArrayBuilder) -> Array {
    // SAFETY: `arr` is the caller's builder, live for the call.
    unsafe {
        let mut items = InitVec::new(
            &mut (*arr).size,
            &mut (*arr).capacity,
            &mut (*arr).items,
            &mut (*arr).init_array,
        );
        let mut ret = arena_array(arena, items.len());
        ret.size = items.len();
        memcpy(
            ret.items.cast(),
            items.as_slice().as_ptr().cast(),
            mem::size_of::<Object>() * ret.size,
        );
        let heap = items.take_heap();
        xfree(heap);
        ret
    }
}

// -- Freeing ---------------------------------------------------------------

pub(crate) unsafe fn api_free_string(value: String_0) {
    // SAFETY: `value` owns its allocation.
    unsafe { xfree(value.data.cast()) };
}

/// Free `value` and everything below it. Only for objects that were built on
/// the heap; an arena-allocated object is freed with its arena.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn api_free_object(value: Object) {
    // SAFETY: `value` owns whatever it points at.
    unsafe {
        match value.type_0 {
            kObjectTypeString => api_free_string(value.data.string),
            kObjectTypeArray => api_free_array(value.data.array),
            kObjectTypeDict => api_free_dict(value.data.dict),
            kObjectTypeLuaRef => api_free_luaref(value.data.luaref),
            _ => {}
        }
    }
}

pub(crate) unsafe fn api_free_array(value: Array) {
    // SAFETY: as `api_free_object`.
    unsafe {
        for i in 0..value.size {
            api_free_object(*value.items.add(i));
        }
        xfree(value.items.cast());
    }
}

pub(crate) unsafe fn api_free_dict(value: Dict) {
    // SAFETY: as `api_free_object`.
    unsafe {
        for i in 0..value.size {
            api_free_string((*value.items.add(i)).key);
            api_free_object((*value.items.add(i)).value);
        }
        xfree(value.items.cast());
    }
}

/// Release the Lua references `value` holds, without freeing `value` itself.
/// For arena-allocated objects, whose memory the arena reclaims but whose
/// references the Lua registry does not.
pub(crate) unsafe fn api_luarefs_free_object(value: Object) {
    // SAFETY: `value` owns the references it names.
    unsafe {
        match value.type_0 {
            kObjectTypeLuaRef => api_free_luaref(value.data.luaref),
            kObjectTypeArray => api_luarefs_free_array(value.data.array),
            kObjectTypeDict => api_luarefs_free_dict(value.data.dict),
            _ => {}
        }
    }
}

pub(crate) unsafe fn api_luarefs_free_array(value: Array) {
    // SAFETY: as `api_luarefs_free_object`.
    unsafe {
        for i in 0..value.size {
            api_luarefs_free_object(*value.items.add(i));
        }
    }
}

pub(crate) unsafe fn api_luarefs_free_dict(value: Dict) {
    // SAFETY: as `api_luarefs_free_object`.
    unsafe {
        for i in 0..value.size {
            api_luarefs_free_object((*value.items.add(i)).value);
        }
    }
}

/// [`api_luarefs_free_object`] over a keydict, walking `table` to find which
/// of its fields can hold a reference.
pub(crate) unsafe fn api_luarefs_free_keydict(dict: *mut c_void, table: *mut KeySetLink) {
    // SAFETY: `table` is the generated table for `dict`'s type, so its
    // offsets and types describe `dict`'s fields; it ends with a null name.
    unsafe {
        for field in keyset_fields(table) {
            let mem = (dict as *mut c_char).add((*field).ptr_off);
            match (*field).type_0 as ObjectType {
                kObjectTypeNil => api_luarefs_free_object(*(mem as *mut Object)),
                kObjectTypeLuaRef => api_free_luaref(*(mem as *mut LuaRef)),
                kObjectTypeDict => api_luarefs_free_dict(*(mem as *mut Dict)),
                _ => {}
            }
        }
    }
}

// -- Copying ---------------------------------------------------------------

/// A copy of `str` in `arena`. Unlike [`arena_string`] a null string stays
/// null rather than becoming the empty one.
pub(crate) unsafe fn copy_string(str: String_0, arena: *mut Arena) -> String_0 {
    // SAFETY: `str` is null or has `size` readable bytes.
    unsafe {
        if str.data.is_null() {
            return STRING_INIT;
        }
        String_0 {
            data: arena_memdupz(arena, str.data, str.size),
            size: str.size,
        }
    }
}

pub(crate) unsafe fn copy_array(array: Array, arena: *mut Arena) -> Array {
    // SAFETY: `array` is live for the call.
    unsafe {
        // Sized for exactly this many items, so it cannot need to grow.
        let mut rv = arena_array(arena, array.size);
        for i in 0..array.size {
            *rv.items.add(i) = copy_object(*array.items.add(i), arena);
        }
        rv.size = array.size;
        rv
    }
}

pub(crate) unsafe fn copy_dict(dict: Dict, arena: *mut Arena) -> Dict {
    // SAFETY: `dict` is live for the call.
    unsafe {
        let mut rv = arena_dict(arena, dict.size);
        for i in 0..dict.size {
            let item = *dict.items.add(i);
            *rv.items.add(i) = key_value_pair {
                // The key's length is re-derived rather than copied, so a
                // key holding a NUL comes back truncated. Upstream's shape.
                key: cstr_as_string(copy_string(item.key, arena).data),
                value: copy_object(item.value, arena),
            };
        }
        rv.size = dict.size;
        rv
    }
}

/// A deep copy of `obj` in `arena`. Handles and scalars copy as they stand;
/// a Lua reference gets a second registry reference of its own.
pub(crate) unsafe fn copy_object(obj: Object, arena: *mut Arena) -> Object {
    // SAFETY: `obj` is live for the call.
    unsafe {
        match obj.type_0 {
            kObjectTypeString => object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: copy_string(obj.data.string, arena),
                },
            },
            kObjectTypeArray => object {
                type_0: kObjectTypeArray,
                data: object_data {
                    array: copy_array(obj.data.array, arena),
                },
            },
            kObjectTypeDict => object {
                type_0: kObjectTypeDict,
                data: object_data {
                    dict: copy_dict(obj.data.dict, arena),
                },
            },
            kObjectTypeLuaRef => object {
                type_0: kObjectTypeLuaRef,
                data: object_data {
                    luaref: api_new_luaref(obj.data.luaref),
                },
            },
            _ => obj,
        }
    }
}

// -- Metadata --------------------------------------------------------------

/// The arena `api_metadata`'s unpacked tree lives in, kept alive for the
/// process's lifetime because the tree is handed out by reference.
static METADATA_ARENA: GlobalCell<ArenaMem> = GlobalCell::new(ptr::null_mut::<consumed_blk>());

/// The API description, as the `nvim_get_api_info` reply carries it. Unpacked
/// from the blob on first use and then shared.
pub(crate) unsafe fn api_metadata() -> Object {
    static METADATA: GlobalCell<Object> = GlobalCell::new(NIL);
    // SAFETY: the blob is a compile-time constant and a valid msgpack map.
    unsafe {
        if (*METADATA.ptr()).type_0 == kObjectTypeNil {
            let mut arena = ARENA_EMPTY;
            let mut err = Error {
                type_0: kErrorTypeNone,
                msg: ptr::null_mut(),
            };
            METADATA.set(unpack(
                PACKED_API_METADATA.as_ptr() as *mut c_char,
                PACKED_API_METADATA.len(),
                &raw mut arena,
                &raw mut err,
            ));
            if err.type_0 != kErrorTypeNone || (*METADATA.ptr()).type_0 != kObjectTypeDict {
                abort();
            }
            METADATA_ARENA.set(arena_finish(&raw mut arena));
        }
        METADATA.get()
    }
}

/// [`api_metadata`] still packed, for a caller that is going to forward it
/// over the wire unchanged.
pub(crate) fn api_metadata_raw() -> String_0 {
    String_0 {
        data: PACKED_API_METADATA.as_ptr() as *mut c_char,
        size: PACKED_API_METADATA.len(),
    }
}

// -- Object conversion -----------------------------------------------------

/// The name of `t` as the API's documentation and error messages spell it.
pub(crate) fn api_typename(t: ObjectType) -> *mut c_char {
    let name = match t {
        kObjectTypeNil => c"nil",
        kObjectTypeBoolean => c"Boolean",
        kObjectTypeInteger => c"Integer",
        kObjectTypeFloat => c"Float",
        kObjectTypeString => c"String",
        kObjectTypeArray => c"Array",
        kObjectTypeDict => c"Dict",
        kObjectTypeLuaRef => c"Function",
        kObjectTypeBuffer => c"Buffer",
        kObjectTypeWindow => c"Window",
        kObjectTypeTabpage => c"Tabpage",
        _ => unreachable!(),
    };
    name.as_ptr() as *mut c_char
}

/// `obj` as a boolean. An integer is true when nonzero and nil takes
/// `nil_value`; anything else is an error naming `what`.
pub(crate) unsafe fn api_object_to_bool(
    obj: Object,
    what: *const c_char,
    nil_value: bool,
    err: *mut Error,
) -> bool {
    // SAFETY: `obj` is live and `what`/`err` are the caller's.
    unsafe {
        match obj.type_0 {
            kObjectTypeBoolean => obj.data.boolean,
            kObjectTypeInteger => obj.data.integer != 0,
            kObjectTypeNil => nil_value,
            _ => {
                api_err_exp(err, what, c"boolean".as_ptr(), ptr::null());
                false
            }
        }
    }
}

/// `obj` as a highlight group id, defining the group if it was named and does
/// not exist yet. Zero for the empty name and for an id out of range.
pub(crate) unsafe fn object_to_hl_id(obj: Object, what: *const c_char, err: *mut Error) -> c_int {
    // SAFETY: `obj` is live and `what`/`err` are the caller's.
    unsafe {
        match obj.type_0 {
            kObjectTypeString => {
                let str = obj.data.string;
                if str.size != 0 {
                    syn_check_group(str.data, str.size)
                } else {
                    0
                }
            }
            kObjectTypeInteger => {
                let id = obj.data.integer as c_int;
                if (1..=highlight_num_groups()).contains(&id) {
                    id
                } else {
                    0
                }
            }
            _ => {
                api_err_invalid(err, c"hl_group".as_ptr(), what, 0, true);
                0
            }
        }
    }
}

/// `kv_push` for a plain kvec, which starts empty and doubles from 8.
unsafe fn push_chunk(msg: &mut HlMessage, chunk: HlMessageChunk) {
    // SAFETY: `items` is null with a zero capacity, or an allocation of
    // `capacity` chunks.
    unsafe {
        if msg.size == msg.capacity {
            msg.capacity = if msg.capacity != 0 {
                msg.capacity * 2
            } else {
                8
            };
            let bytes = mem::size_of::<HlMessageChunk>() * msg.capacity;
            msg.items = xrealloc(msg.items.cast(), bytes).cast();
        }
        *msg.items.add(msg.size) = chunk;
        msg.size += 1;
    }
}

/// Parse `[[text, hl], …]` — the shape `nvim_echo` and friends take — into a
/// highlighted message. Empty, with `err` set, on the first bad chunk.
pub(crate) unsafe fn parse_hl_msg(chunks: Array, is_err: bool, err: *mut Error) -> HlMessage {
    // SAFETY: `chunks` is live for the call and `err` is the caller's.
    unsafe {
        let mut hl_msg = EMPTY_HL_MESSAGE;
        for i in 0..chunks.size {
            let item = *chunks.items.add(i);
            if item.type_0 != kObjectTypeArray {
                api_err_exp(
                    err,
                    c"chunk".as_ptr(),
                    api_typename(kObjectTypeArray),
                    api_typename(item.type_0),
                );
                hl_msg_free(hl_msg);
                return EMPTY_HL_MESSAGE;
            }
            let chunk = item.data.array;
            if !((1..=2).contains(&chunk.size) && (*chunk.items).type_0 == kObjectTypeString) {
                let msg = c"Invalid chunk: expected Array with 1 or 2 Strings".as_ptr();
                api_set_error(err, kErrorTypeValidation, c"%s".as_ptr(), msg);
                hl_msg_free(hl_msg);
                return EMPTY_HL_MESSAGE;
            }
            // Heap-allocated: the message outlives the caller's arena.
            let text = copy_string((*chunk.items).data.string, ptr::null_mut());
            let hl_id = if chunk.size == 2 {
                object_to_hl_id(*chunk.items.add(1), c"text highlight".as_ptr(), err)
            } else if is_err {
                HLF_E
            } else {
                0
            };
            push_chunk(&mut hl_msg, HlMessageChunk { text, hl_id });
        }
        hl_msg
    }
}

// -- Keydicts --------------------------------------------------------------

/// The fields of a keydict, as its generated `KeySetLink` table lists them.
/// The table ends with a null name.
unsafe fn keyset_fields(table: *mut KeySetLink) -> impl Iterator<Item = *mut KeySetLink> {
    // SAFETY: `table` is one of the generated tables, which are
    // null-terminated by construction.
    let len = unsafe {
        let mut n = 0;
        while !(*table.add(n)).str.is_null() {
            n += 1;
        }
        n
    };
    (0..len).map(move |i| unsafe { table.add(i) })
}

/// Fill the keydict `retval` from `dict`, type-checking each value against
/// what the field it names holds. False, with `err` set, on the first
/// unknown key or wrong type.
///
/// `retval` is untyped because there is one such struct per API function;
/// `hashy` is that struct's generated perfect-hash lookup and the
/// `KeySetLink` it returns is what says where and what the field is.
pub(crate) unsafe fn api_dict_to_keydict(
    retval: *mut c_void,
    hashy: FieldHashfn,
    dict: Dict,
    err: *mut Error,
) -> bool {
    // SAFETY: `hashy` is the generated lookup for `retval`'s type, so the
    // offsets it hands back are inside `retval`.
    unsafe {
        for i in 0..dict.size {
            let k = (*dict.items.add(i)).key;
            let field = hashy.expect("non-null function pointer")(k.data, k.size);
            if field.is_null() {
                let fmt = c"Invalid key: '%.*s'".as_ptr();
                api_set_error(err, kErrorTypeValidation, fmt, k.size as c_int, k.data);
                return false;
            }
            // Optional fields record that they were given, so that the API
            // function can tell "absent" from "set to the default".
            if (*field).opt_index >= 0 {
                let ks = retval as *mut OptKeySet;
                (*ks).is_set_ |= (1 as OptionalKeys) << (*field).opt_index;
            }

            let mem = (retval as *mut c_char).add((*field).ptr_off);
            let value = &raw mut (*dict.items.add(i)).value;
            let expected = (*field).type_0 as ObjectType;
            // A mismatch reports the field's name, not the key's: they are
            // the same string.
            let mut wrong_type = |want: ObjectType| {
                api_err_exp(
                    err,
                    (*field).str,
                    api_typename(want),
                    api_typename((*value).type_0),
                );
            };

            match expected {
                // A nil-typed field takes the object as it stands.
                kObjectTypeNil => *(mem as *mut Object) = *value,
                kObjectTypeInteger if (*field).is_hlgroup => {
                    let mut hl_id = 0;
                    if (*value).type_0 != kObjectTypeNil {
                        hl_id = object_to_hl_id(*value, k.data, err);
                        if (*err).type_0 != kErrorTypeNone {
                            return false;
                        }
                    }
                    *(mem as *mut Integer) = hl_id as Integer;
                }
                kObjectTypeInteger => {
                    if (*value).type_0 != kObjectTypeInteger {
                        wrong_type(kObjectTypeInteger);
                        return false;
                    }
                    *(mem as *mut Integer) = (*value).data.integer;
                }
                // A float field takes an integer too.
                kObjectTypeFloat => match (*value).type_0 {
                    kObjectTypeInteger => *(mem as *mut Float) = (*value).data.integer as Float,
                    kObjectTypeFloat => *(mem as *mut Float) = (*value).data.floating,
                    _ => {
                        wrong_type(kObjectTypeFloat);
                        return false;
                    }
                },
                kObjectTypeBoolean => {
                    *(mem as *mut Boolean) = api_object_to_bool(*value, (*field).str, false, err);
                    if (*err).type_0 != kErrorTypeNone {
                        return false;
                    }
                }
                kObjectTypeString => {
                    if (*value).type_0 != kObjectTypeString {
                        wrong_type(kObjectTypeString);
                        return false;
                    }
                    *(mem as *mut String_0) = (*value).data.string;
                }
                kObjectTypeArray => {
                    if (*value).type_0 != kObjectTypeArray {
                        wrong_type(kObjectTypeArray);
                        return false;
                    }
                    *(mem as *mut Array) = (*value).data.array;
                }
                // An empty array is how msgpack spells an empty map.
                kObjectTypeDict => match (*value).type_0 {
                    kObjectTypeArray if (*value).data.array.size == 0 => {
                        *(mem as *mut Dict) = EMPTY_DICT;
                    }
                    kObjectTypeDict => *(mem as *mut Dict) = (*value).data.dict,
                    _ => {
                        wrong_type(kObjectTypeDict);
                        return false;
                    }
                },
                kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage => {
                    if (*value).type_0 != kObjectTypeInteger && (*value).type_0 != expected {
                        wrong_type(expected);
                        return false;
                    }
                    *(mem as *mut handle_T) = (*value).data.integer as handle_T;
                }
                kObjectTypeLuaRef => {
                    let fmt = c"Invalid key: '%.*s' is only allowed from Lua".as_ptr();
                    api_set_error(err, kErrorTypeValidation, fmt, k.size as c_int, k.data);
                    return false;
                }
                _ => abort(),
            }
        }
        true
    }
}

/// The reverse of [`api_dict_to_keydict`]: the keydict `value` as a plain
/// dictionary, holding only the fields that were set. Lua references are
/// skipped — they mean nothing outside the Lua state.
pub(crate) unsafe fn api_keydict_to_dict(
    value: *mut c_void,
    table: *mut KeySetLink,
    max_size: size_t,
    arena: *mut Arena,
) -> Dict {
    // SAFETY: as `api_dict_to_keydict`; `max_size` is the table's length.
    unsafe {
        let mut rv = arena_dict(arena, max_size);
        for field in keyset_fields(table) {
            if (*field).opt_index >= 0 {
                let ks = value as *mut OptKeySet;
                if (*ks).is_set_ & ((1 as OptionalKeys) << (*field).opt_index) == 0 {
                    continue;
                }
            }
            let mem = (value as *mut c_char).add((*field).ptr_off);
            let mut val = NIL;
            match (*field).type_0 as ObjectType {
                kObjectTypeNil => val = *(mem as *mut Object),
                kObjectTypeInteger => {
                    val = object {
                        type_0: kObjectTypeInteger,
                        data: object_data {
                            integer: *(mem as *mut Integer),
                        },
                    };
                }
                kObjectTypeFloat => {
                    val = object {
                        type_0: kObjectTypeFloat,
                        data: object_data {
                            floating: *(mem as *mut Float),
                        },
                    };
                }
                kObjectTypeBoolean => {
                    val = object {
                        type_0: kObjectTypeBoolean,
                        data: object_data {
                            boolean: *(mem as *mut Boolean),
                        },
                    };
                }
                kObjectTypeString => {
                    val = object {
                        type_0: kObjectTypeString,
                        data: object_data {
                            string: *(mem as *mut String_0),
                        },
                    };
                }
                kObjectTypeArray => {
                    val = object {
                        type_0: kObjectTypeArray,
                        data: object_data {
                            array: *(mem as *mut Array),
                        },
                    };
                }
                kObjectTypeDict => {
                    val = object {
                        type_0: kObjectTypeDict,
                        data: object_data {
                            dict: *(mem as *mut Dict),
                        },
                    };
                }
                handle @ (kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage) => {
                    val.data.integer = *(mem as *mut handle_T) as Integer;
                    val.type_0 = handle;
                }
                // A Lua reference is still counted as a key, with a nil value.
                kObjectTypeLuaRef => {}
                _ => abort(),
            }
            *rv.items.add(rv.size) = key_value_pair {
                key: cstr_as_string((*field).str),
                value: val,
            };
            rv.size += 1;
        }
        rv
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
            if col > MAXCOL {
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
    // SAFETY: `syn_id2name` takes an id, not a pointer; `wp` is only
    // compared, never followed.
    unsafe {
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
