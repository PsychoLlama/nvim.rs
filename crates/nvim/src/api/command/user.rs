//! User commands: `:command` from the API.
//!
//! `create_user_command` is the shared implementation -- it validates the
//! name, decodes the `nargs`/`range`/`count`/`addr`/`complete` keyset into
//! the `uc_add_command` flags, and accepts either a command string or a
//! `LuaRef` -- and the four `nvim_*_user_command` entry points differ only
//! in whether they are buffer-local.  The two `get_commands` spellings
//! render the table back.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, has_key};
use crate::api::private::validate::err_bad_number;
use crate::api::private::validate::err_expected_ptr;
use crate::api::private::validate::err_invalid_ptr;
use crate::api::private::validate::err_msg_ptr;
use crate::api_error;
use crate::message_fmt::c_str;
use crate::types::{ExArgt, ExpandContext};
use crate::winlayer::Live;

/// The options keyset this family decodes, with checked field access.
///
/// Construction is the one unsafe step: the caller's `*mut
/// KeyDict_user_command` outlives the call that was handed it, and every
/// `opts.field` after the wrap is ordinary code.
type UserCmdOpts = Live<KeyDict_user_command>;

pub unsafe fn nvim_create_user_command(
    channel_id: uint64_t,
    name: String_0,
    cmd: Object,
    opts: *mut KeyDict_user_command,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // SAFETY: `opts` is the caller's keydict and `err` this frame's slot.
    unsafe { create_user_command(channel_id, name, cmd, opts, 0, err) };
    ().reported(error)
}

pub unsafe fn nvim_del_user_command(name: String_0) -> Result<(), Error> {
    // SAFETY: `name` is the caller's command name.
    unsafe { nvim_buf_del_user_command(-1, name) }
}

pub unsafe fn nvim_buf_create_user_command(
    channel_id: uint64_t,
    buf: Buffer,
    name: String_0,
    cmd: Object,
    opts: *mut KeyDict_user_command,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // SAFETY: `err` is this frame's slot.
    let target_buf = unsafe { find_buffer_by_handle(buf, err) };
    if error.is_set() {
        return ().reported(error);
    }
    // The command is added to whichever buffer is current, so the lookup's
    // answer stands in for the caller's for the length of the call.
    let save_curbuf = curbuf.get();
    curbuf.set(target_buf);
    let flags = UC_BUFFER as ::core::ffi::c_int;
    // SAFETY: `opts` is the caller's keydict and `err` this frame's slot.
    unsafe { create_user_command(channel_id, name, cmd, opts, flags, err) };
    curbuf.set(save_curbuf);
    ().reported(error)
}

pub unsafe fn nvim_buf_del_user_command(buf: Buffer, name: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let table = if buf == -1 {
        Table::Global
    } else {
        // SAFETY: `err` is this frame's slot.
        let b = unsafe { find_buffer_by_handle(buf, err) };
        if error.is_set() {
            return ().reported(error);
        }
        Table::Buffer(b)
    };
    // SAFETY: `table` names the global table or a live buffer's, the borrow
    // does not outlive the search, and `name` is the caller's C string.
    let found = unsafe {
        table
            .list()
            .iter()
            .position(|cmd| strcmp(name.data(), cmd.uc_name) == 0)
    };
    if let Some(idx) = found {
        // SAFETY: `idx` indexes the table the search just walked.
        unsafe { uc_del_command(table, idx) };
        return ().reported(error);
    }
    // SAFETY: `name` names its own NUL-terminated bytes.
    let name = unsafe { c_str(name.data()) };
    error = api_error!(kErrorTypeException, "Invalid command (not found): {name}");
    ().reported(error)
}

pub unsafe fn create_user_command(
    channel_id: uint64_t,
    name: String_0,
    cmd: Object,
    opts: *mut KeyDict_user_command,
    flags: ::core::ffi::c_int,
    err: *mut Error,
) {
    // SAFETY: `opts` is the caller's keydict, live for the call.
    let mut opts = unsafe { UserCmdOpts::new(opts) };
    let mut force: bool = false;
    let mut argt = ExArgt::NONE;
    let mut def: int64_t = -1;
    let mut addr_type_arg = CmdAddr::NoRange;
    let mut context = ExpandContext::Nothing;
    let mut compl_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut rep: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut luaref: LuaRef = LUA_NOREF;
    let mut compl_luaref: LuaRef = LUA_NOREF;
    let mut preview_luaref: LuaRef = LUA_NOREF;
    let cmd_name = name.data();
    '_err: {
        // SAFETY: `cmd_name` is the caller's NUL-terminated command name.
        let named = !unsafe { uc_validate_name(cmd_name) }.is_null();
        if !named {
            // SAFETY: the caller's error slot.
            unsafe { *err = err_invalid_ptr(c"command name".as_ptr(), cmd_name, 0, true) };
            break '_err;
        }
        // SAFETY: the name validated, so it has at least one byte.
        if mb_islower(unsafe { *cmd_name } as ::core::ffi::c_int) {
            let what = c"command name (must start with uppercase)".as_ptr();
            // SAFETY: the caller's error slot.
            unsafe { *err = err_invalid_ptr(what, cmd_name, 0, true) };
            break '_err;
        }
        let is_set = opts.is_set__user_command_;
        if has_key(is_set, KEYSET_OPTIDX_user_command__range)
            && has_key(is_set, KEYSET_OPTIDX_user_command__count)
        {
            let msg = c"Cannot use both 'range' and 'count'".as_ptr();
            // SAFETY: the caller's error slot.
            unsafe { *err = err_msg_ptr(kErrorTypeValidation, msg) };
            break '_err;
        }

        if let Some(nargs) = opts.nargs.as_integer() {
            match nargs {
                0 => {}
                1 => argt |= ExArgt::EXTRA | ExArgt::NOSPC | ExArgt::NEEDARG,
                _ => {
                    // SAFETY: the caller's error slot.
                    unsafe { *err = err_bad_number(c"nargs", nargs) };
                    break '_err;
                }
            }
        } else if let Some(nargs) = opts.nargs.as_string() {
            let value = nargs.data();
            if nargs.len() > 1 {
                // SAFETY: the caller's error slot.
                unsafe { *err = err_invalid_ptr(c"nargs".as_ptr(), value, 0, true) };
                break '_err;
            }
            // SAFETY: an API string is NUL-terminated, so byte 0 is readable
            // even for the empty string -- where it is the terminator, and
            // falls to the arm that rejects it.
            match unsafe { *value } as u8 {
                b'*' => argt |= ExArgt::EXTRA,
                b'?' => argt |= ExArgt::EXTRA | ExArgt::NOSPC,
                b'+' => argt |= ExArgt::EXTRA | ExArgt::NEEDARG,
                _ => {
                    // SAFETY: the caller's error slot.
                    unsafe { *err = err_invalid_ptr(c"nargs".as_ptr(), value, 0, true) };
                    break '_err;
                }
            }
        } else if has_key(is_set, KEYSET_OPTIDX_user_command__nargs) {
            // SAFETY: the caller's error slot.
            unsafe { *err = err_invalid_ptr(c"nargs".as_ptr(), c"".as_ptr(), 0, true) };
            break '_err;
        }

        if has_key(is_set, KEYSET_OPTIDX_user_command__complete) && argt == ExArgt::NONE {
            let msg = c"'complete' used without 'nargs'".as_ptr();
            // SAFETY: the caller's error slot.
            unsafe { *err = err_msg_ptr(kErrorTypeValidation, msg) };
            break '_err;
        }

        if let Some(range) = opts.range.as_boolean() {
            if range {
                argt |= ExArgt::RANGE;
                addr_type_arg = CmdAddr::Lines;
            }
        } else if let Some(range) = opts.range.as_string() {
            // SAFETY: an API string is NUL-terminated, so byte 0 is readable.
            let percent = unsafe { *range.data() } as u8 == b'%';
            if !(percent && range.len() == 1) {
                // SAFETY: the caller's error slot.
                unsafe { *err = err_invalid_ptr(c"range".as_ptr(), c"".as_ptr(), 0, true) };
                break '_err;
            }
            argt |= ExArgt::RANGE | ExArgt::DFLALL;
            addr_type_arg = CmdAddr::Lines;
        } else if let Some(range) = opts.range.as_integer() {
            argt |= ExArgt::RANGE | ExArgt::ZEROR;
            def = range;
            addr_type_arg = CmdAddr::Lines;
        } else if has_key(is_set, KEYSET_OPTIDX_user_command__range) {
            // SAFETY: the caller's error slot.
            unsafe { *err = err_invalid_ptr(c"range".as_ptr(), c"".as_ptr(), 0, true) };
            break '_err;
        }

        if let Some(count) = opts.count.as_boolean() {
            if count {
                argt |= ExArgt::COUNT | ExArgt::ZEROR | ExArgt::RANGE;
                addr_type_arg = CmdAddr::Other;
                def = 0;
            }
        } else if let Some(count) = opts.count.as_integer() {
            argt |= ExArgt::COUNT | ExArgt::ZEROR | ExArgt::RANGE;
            addr_type_arg = CmdAddr::Other;
            def = count;
        } else if has_key(is_set, KEYSET_OPTIDX_user_command__count) {
            // SAFETY: the caller's error slot.
            unsafe { *err = err_invalid_ptr(c"count".as_ptr(), c"".as_ptr(), 0, true) };
            break '_err;
        }

        if has_key(is_set, KEYSET_OPTIDX_user_command__addr) {
            let Some(addr) = opts.addr.as_string() else {
                let expected = api_typename(kObjectTypeString);
                let actual = api_typename(opts.addr.type_0);
                // SAFETY: the caller's error slot.
                unsafe { *err = err_expected_ptr(c"addr".as_ptr(), expected, Some(actual)) };
                break '_err;
            };
            let value = addr.data();
            let vallen = addr.len() as ::core::ffi::c_int;
            let slot = &raw mut addr_type_arg;
            // SAFETY: `addr` is the caller's string, NUL-terminated with
            // `vallen` readable bytes, and `slot` is this frame's.
            let parsed = unsafe { parse_addr_type_arg(value, vallen, slot) };
            if parsed != 1 {
                // SAFETY: the caller's error slot.
                unsafe { *err = err_invalid_ptr(c"addr".as_ptr(), value, 0, true) };
                break '_err;
            }
            argt |= ExArgt::RANGE;
            if addr_type_arg != CmdAddr::Lines {
                argt |= ExArgt::ZEROR;
            }
        }

        if opts.bang {
            argt |= ExArgt::BANG;
        }
        if opts.bar {
            argt |= ExArgt::TRLBAR;
        }
        if opts.register_ {
            argt |= ExArgt::REGSTR;
        }
        if opts.keepscript {
            argt |= ExArgt::KEEPSCRIPT;
        }
        // An unsupplied `force` defaults to true: `nvim_create_user_command`
        // replaces an existing command unless told otherwise.
        force = !has_key(is_set, KEYSET_OPTIDX_user_command__force) || opts.force;

        // Everything above reports through `err` without stopping, so a
        // failure that fell through to here still has to skip the rest.
        // SAFETY: `err` is the caller's slot.
        if unsafe { (*err).kind() } != kErrorTypeNone {
            break '_err;
        }

        if let Some(complete) = opts.complete.as_luaref() {
            context = ExpandContext::UserLua;
            compl_luaref = complete;
            // The reference is this call's now, so the keyset must not free
            // it a second time.
            opts.complete.data.luaref = LUA_NOREF;
        } else if let Some(complete) = opts.complete.as_string() {
            let value = complete.data();
            let vallen = complete.len() as ::core::ffi::c_int;
            // SAFETY: `complete` is the caller's string, NUL-terminated with
            // `vallen` readable bytes; the three out-parameters are this
            // frame's.
            let parsed =
                unsafe { parse_compl_arg(value, vallen, &mut context, &mut argt, &mut compl_arg) };
            if parsed != 1 {
                // SAFETY: the caller's error slot.
                unsafe { *err = err_invalid_ptr(c"complete".as_ptr(), value, 0, true) };
                break '_err;
            }
        } else if has_key(is_set, KEYSET_OPTIDX_user_command__complete) {
            let expected = c"Function or String";
            // SAFETY: the caller's error slot.
            unsafe { *err = err_expected_ptr(c"complete".as_ptr(), expected, None) };
            break '_err;
        }

        if has_key(is_set, KEYSET_OPTIDX_user_command__preview) {
            let Some(preview) = opts.preview.as_luaref() else {
                let expected = api_typename(kObjectTypeLuaRef);
                let actual = api_typename(opts.preview.type_0);
                // SAFETY: the caller's error slot.
                unsafe { *err = err_expected_ptr(c"preview".as_ptr(), expected, Some(actual)) };
                break '_err;
            };
            argt |= ExArgt::PREVIEW;
            preview_luaref = preview;
            // As `complete`: the reference is this call's now.
            opts.preview.data.luaref = LUA_NOREF;
        }

        if let Some(body) = cmd.as_luaref() {
            // SAFETY: `body` is a registry index rather than a pointer, and
            // the object still holds the caller's own reference to it.
            luaref = unsafe { api_new_luaref(body) };
            rep = match opts.desc.as_string() {
                Some(desc) => desc.data().cast_const(),
                None => c"".as_ptr(),
            };
        } else if let Some(body) = cmd.as_string() {
            rep = body.data().cast_const();
        } else {
            let expected = c"Function or String";
            // SAFETY: the caller's error slot.
            unsafe { *err = err_expected_ptr(c"command".as_ptr(), expected, None) };
            break '_err;
        }

        let _sctx = api_set_sctx(channel_id);
        // SAFETY: `name` and `rep` are the caller's NUL-terminated strings,
        // `compl_arg` is this frame's allocation, and the three Lua
        // references are owned by this call -- `uc_add_command` takes all
        // four over whether it succeeds or fails.
        let added = unsafe {
            uc_add_command(
                name.data(),
                name.len(),
                rep,
                argt,
                def,
                flags,
                context,
                compl_arg,
                compl_luaref,
                preview_luaref,
                addr_type_arg,
                luaref,
                force,
            )
        };
        if added != 1 {
            let msg = c"Failed to create user command".as_ptr();
            // SAFETY: the caller's error slot.
            unsafe { *err = err_msg_ptr(kErrorTypeException, msg) };
        }
        // `uc_add_command` owns what it was handed, so nothing below runs.
        return;
    }
    // Only reached when the command was never added, so this call still owns
    // the references it took and the argument it parsed.
    if luaref != LUA_NOREF {
        // SAFETY: the reference is this call's, taken above.
        unsafe { api_free_luaref(luaref) };
    }
    if compl_luaref != LUA_NOREF {
        // SAFETY: as above.
        unsafe { api_free_luaref(compl_luaref) };
    }
    if preview_luaref != LUA_NOREF {
        // SAFETY: as above.
        unsafe { api_free_luaref(preview_luaref) };
    }
    // SAFETY: `compl_arg` is null or `parse_compl_arg`'s own allocation.
    unsafe { xfree(compl_arg.cast()) };
}

pub unsafe fn nvim_get_commands(
    opts: *mut KeyDict_get_commands,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    // SAFETY: `opts` and `arena` are the caller's.
    unsafe { nvim_buf_get_commands(-1, opts, arena) }
}

pub unsafe fn nvim_buf_get_commands(
    buf: Buffer,
    opts: *mut KeyDict_get_commands,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // SAFETY: `opts` is the caller's keydict, live for the call.
    let builtin = unsafe { (*opts).builtin };
    if buf == -1 {
        if builtin {
            error = Error::from_message(kErrorTypeValidation, c"builtin=true not implemented");
            return Dict::EMPTY.reported(error);
        }
        // SAFETY: a null buffer names the global table, and `arena` is the
        // caller's.
        let global = ::core::ptr::null_mut::<buf_T>();
        return unsafe { commands_array(global, arena) }.reported(error);
    }
    // SAFETY: `err` is this frame's slot.
    let b = unsafe { find_buffer_by_handle(buf, err) };
    if builtin || b.is_null() {
        return Dict::EMPTY.reported(error);
    }
    // SAFETY: `b` is a live buffer and `arena` is the caller's.
    unsafe { commands_array(b, arena) }.reported(error)
}
