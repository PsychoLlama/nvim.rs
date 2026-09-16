//! User commands: `:command` from the API.
//!
//! `create_user_command` is the shared implementation -- it validates the
//! name, decodes the `nargs`/`range`/`count`/`addr`/`complete` keyset into
//! the `uc_add_command` flags, and accepts either a command string or a
//! `LuaRef` -- and the four `nvim_*_user_command` entry points differ only
//! in whether they are buffer-local.  The two `get_commands` spellings
//! render the table back.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{
    Bad, err_bad_number, err_bad_value, err_expected, err_invalid,
};
use crate::api_error;
use crate::cstr;
use crate::message_fmt::c_str;
use crate::types::{ExArgt, ExpandContext};
use crate::winlayer::Live;
use crate::winlayer::graph::switch_buffer;

/// The options keyset this family decodes, with checked field access.
///
/// Construction is the one unsafe step: the caller's `*mut
/// KeyDict_user_command` outlives the call that was handed it, and every
/// `opts.field` after the wrap is ordinary code.
type UserCmdOpts = Live<KeyDict_user_command>;

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `cmd` must be a well-formed API object the caller owns
/// for the call. `opts` must point at the `KeyDict_user_command` the
/// dispatcher filled in, live for the call.
pub unsafe fn nvim_create_user_command(
    channel_id: uint64_t,
    name: String_0,
    cmd: Object,
    opts: *mut KeyDict_user_command,
) -> Result<(), Error> {
    // SAFETY: `opts` is the caller's keydict.
    unsafe { create_user_command(channel_id, name, cmd, opts, 0) }
}

pub fn nvim_del_user_command(name: String_0) -> Result<(), Error> {
    nvim_buf_del_user_command(-1, name)
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `cmd` must be a well-formed API object the caller owns
/// for the call. `opts` must point at the `KeyDict_user_command` the
/// dispatcher filled in, live for the call.
pub unsafe fn nvim_buf_create_user_command(
    channel_id: uint64_t,
    buf: BufferHandle,
    name: String_0,
    cmd: Object,
    opts: *mut KeyDict_user_command,
) -> Result<(), Error> {
    let Some(target_buf) = find_buffer_by_handle(buf)? else {
        return Ok(());
    };
    // The command is added to whichever buffer is current, so the lookup's
    // answer stands in for the caller's for the length of the call.
    let saved = switch_buffer(target_buf);
    let flags = UC_BUFFER as ::core::ffi::c_int;
    // SAFETY: `opts` is the caller's keydict.
    let made = unsafe { create_user_command(channel_id, name, cmd, opts, flags) };
    saved.restore();
    made
}

pub fn nvim_buf_del_user_command(buf: BufferHandle, name: String_0) -> Result<(), Error> {
    let mut error = Error::none();
    let table = if buf == -1 {
        Table::Global
    } else {
        let Some(b) = find_buffer_by_handle(buf)? else {
            return Ok(());
        };
        Table::Buffer(b)
    };
    // SAFETY: `table` names the global table or a live buffer's, the borrow
    // does not outlive the search, and `name` is the caller's C string.
    let found = unsafe {
        table
            .list()
            .iter()
            .position(|cmd| cstr::eq(name.data(), cmd.uc_name))
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

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `cmd` must be a well-formed API object the caller owns
/// for the call. `opts` must point at the `KeyDict_user_command` the
/// dispatcher filled in, live for the call.
pub unsafe fn create_user_command(
    channel_id: uint64_t,
    name: String_0,
    cmd: Object,
    opts: *mut KeyDict_user_command,
    flags: ::core::ffi::c_int,
) -> Result<(), Error> {
    // SAFETY: `opts` is the caller's keydict, live for the call.
    let mut opts = unsafe { UserCmdOpts::new(opts) };
    let force: bool;
    let mut argt = ExArgt::NONE;
    let mut def: int64_t = -1;
    let mut addr_type_arg = CmdAddr::NoRange;
    let mut context = ExpandContext::Nothing;
    let mut compl_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let rep: *const ::core::ffi::c_char;
    let mut luaref: LuaRef = LUA_NOREF;
    let mut compl_luaref: LuaRef = LUA_NOREF;
    let mut preview_luaref: LuaRef = LUA_NOREF;
    let cmd_name = name.data();
    // The refusal is held rather than returned: the cleanup below has to run
    // whichever way the block left.
    let mut failed = None;
    '_err: {
        // SAFETY: `cmd_name` is the caller's NUL-terminated command name.
        let named = !unsafe { uc_validate_name(cmd_name) }.is_null();
        if !named {
            failed = Some(err_bad_value(c"command name", name.as_cstr()));
            break '_err;
        }
        // SAFETY: the name validated, so it has at least one byte.
        if mb_islower(unsafe { *cmd_name } as ::core::ffi::c_int) {
            let what = c"command name (must start with uppercase)";
            failed = Some(err_bad_value(what, name.as_cstr()));
            break '_err;
        }
        // Every key, borrowed: the keyset owns its values, and the one
        // this function *takes* -- `complete`'s Lua reference -- says so.
        let given_nargs = opts.nargs.as_ref();
        let given_range = opts.range.as_ref();
        let given_count = opts.count.as_ref();
        let given_addr = opts.addr.as_ref();
        if given_range.is_some() && given_count.is_some() {
            failed = Some(Error::validation(c"Cannot use both 'range' and 'count'"));
            break '_err;
        }

        if let Some(nargs) = given_nargs.and_then(Object::as_integer) {
            match nargs {
                0 => {}
                1 => argt |= ExArgt::EXTRA | ExArgt::NOSPC | ExArgt::NEEDARG,
                _ => {
                    failed = Some(err_bad_number(c"nargs", nargs));
                    break '_err;
                }
            }
        } else if let Some(nargs) = given_nargs.and_then(Object::as_string) {
            let value = nargs.data();
            if nargs.len() > 1 {
                failed = Some(err_bad_value(c"nargs", nargs.as_cstr()));
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
                    failed = Some(err_bad_value(c"nargs", nargs.as_cstr()));
                    break '_err;
                }
            }
        } else if given_nargs.is_some() {
            failed = Some(err_invalid(c"nargs", Bad::Unsaid));
            break '_err;
        }

        if opts.complete.is_some() && argt == ExArgt::NONE {
            failed = Some(Error::validation(c"'complete' used without 'nargs'"));
            break '_err;
        }

        if let Some(range) = given_range.and_then(Object::as_boolean) {
            if range {
                argt |= ExArgt::RANGE;
                addr_type_arg = CmdAddr::Lines;
            }
        } else if let Some(range) = given_range.and_then(Object::as_string) {
            // SAFETY: an API string is NUL-terminated, so byte 0 is readable.
            let percent = unsafe { *range.data() } as u8 == b'%';
            if !(percent && range.len() == 1) {
                failed = Some(err_invalid(c"range", Bad::Unsaid));
                break '_err;
            }
            argt |= ExArgt::RANGE | ExArgt::DFLALL;
            addr_type_arg = CmdAddr::Lines;
        } else if let Some(range) = given_range.and_then(Object::as_integer) {
            argt |= ExArgt::RANGE | ExArgt::ZEROR;
            def = range;
            addr_type_arg = CmdAddr::Lines;
        } else if given_range.is_some() {
            failed = Some(err_invalid(c"range", Bad::Unsaid));
            break '_err;
        }

        if let Some(count) = given_count.and_then(Object::as_boolean) {
            if count {
                argt |= ExArgt::COUNT | ExArgt::ZEROR | ExArgt::RANGE;
                addr_type_arg = CmdAddr::Other;
                def = 0;
            }
        } else if let Some(count) = given_count.and_then(Object::as_integer) {
            argt |= ExArgt::COUNT | ExArgt::ZEROR | ExArgt::RANGE;
            addr_type_arg = CmdAddr::Other;
            def = count;
        } else if given_count.is_some() {
            failed = Some(err_invalid(c"count", Bad::Unsaid));
            break '_err;
        }

        if let Some(given) = given_addr {
            let Some(addr) = given.as_string() else {
                let expected = api_typename(kObjectTypeString);
                let actual = api_typename(given.kind());
                failed = Some(err_expected(c"addr", expected, Some(actual)));
                break '_err;
            };
            let value = addr.data();
            let vallen = addr.len() as ::core::ffi::c_int;
            let slot = &raw mut addr_type_arg;
            // SAFETY: `addr` is the caller's string, NUL-terminated with
            // `vallen` readable bytes, and `slot` is this frame's.
            let parsed = unsafe { parse_addr_type_arg(value, vallen, slot) };
            if parsed.is_err() {
                failed = Some(err_bad_value(c"addr", addr.as_cstr()));
                break '_err;
            }
            argt |= ExArgt::RANGE;
            if addr_type_arg != CmdAddr::Lines {
                argt |= ExArgt::ZEROR;
            }
        }

        if opts.bang.unwrap_or(false) {
            argt |= ExArgt::BANG;
        }
        if opts.bar.unwrap_or(false) {
            argt |= ExArgt::TRLBAR;
        }
        if opts.register_.unwrap_or(false) {
            argt |= ExArgt::REGSTR;
        }
        if opts.keepscript.unwrap_or(false) {
            argt |= ExArgt::KEEPSCRIPT;
        }
        // An unsupplied `force` defaults to true: `nvim_create_user_command`
        // replaces an existing command unless told otherwise.
        force = opts.force.unwrap_or(true);

        // Everything above reports through `err` without stopping, so a
        // failure that fell through to here still has to skip the rest.
        if failed.is_some() {
            break '_err;
        }

        if opts.complete.as_ref().and_then(Object::as_luaref).is_some() {
            context = ExpandContext::UserLua;
            // The reference is this call's now, so the keyset must not
            // release it: the value moves out of the field.
            compl_luaref = opts
                .complete
                .take()
                .and_then(Object::into_luaref)
                .expect("the arm above matched a LuaRef");
        } else if let Some(complete) = opts.complete.as_ref().and_then(Object::as_string) {
            let value = complete.data();
            let vallen = complete.len() as ::core::ffi::c_int;
            // SAFETY: `complete` is the caller's string, NUL-terminated with
            // `vallen` readable bytes; the three out-parameters are this
            // frame's.
            let parsed =
                unsafe { parse_compl_arg(value, vallen, &mut context, &mut argt, &mut compl_arg) };
            if parsed.is_err() {
                failed = Some(err_bad_value(c"complete", complete.as_cstr()));
                break '_err;
            }
        } else if opts.complete.is_some() {
            let expected = c"Function or String";
            failed = Some(err_expected(c"complete", expected, None));
            break '_err;
        }

        if let Some(given) = opts.preview.as_ref() {
            let Some(preview) = given.as_luaref() else {
                let expected = api_typename(kObjectTypeLuaRef);
                let actual = api_typename(given.kind());
                failed = Some(err_expected(c"preview", expected, Some(actual)));
                break '_err;
            };
            argt |= ExArgt::PREVIEW;
            let _ = preview;
            // As `complete`: the reference is this call's now.
            preview_luaref = opts
                .preview
                .take()
                .and_then(Object::into_luaref)
                .expect("the check above matched a LuaRef");
        }

        if let Some(body) = cmd.as_luaref() {
            luaref = api_new_luaref(body);
            rep = match opts.desc.as_ref().and_then(Object::as_string) {
                Some(desc) => desc.data().cast_const(),
                None => c"".as_ptr(),
            };
        } else if let Some(body) = cmd.as_string() {
            rep = body.data().cast_const();
        } else {
            let expected = c"Function or String";
            failed = Some(err_expected(c"command", expected, None));
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
        if added.is_err() {
            failed = Some(Error::exception(c"Failed to create user command"));
        }
        // `uc_add_command` owns what it was handed, so nothing below runs.
        return match failed {
            Some(failed) => Err(failed),
            None => Ok(()),
        };
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
    match failed {
        Some(failed) => Err(failed),
        None => Ok(()),
    }
}

/// # Safety
///
/// `opts` must point at the `KeyDict_get_commands` the dispatcher filled in,
/// live for the call. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
pub unsafe fn nvim_get_commands(opts: *mut KeyDict_get_commands) -> Result<ApiDict, Error> {
    // SAFETY: `opts` and `arena` are the caller's.
    unsafe { nvim_buf_get_commands(-1, opts) }
}

/// # Safety
///
/// `opts` must point at the `KeyDict_get_commands` the dispatcher filled in,
/// live for the call. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
pub unsafe fn nvim_buf_get_commands(
    buf: BufferHandle,
    opts: *mut KeyDict_get_commands,
) -> Result<ApiDict, Error> {
    let mut error = Error::none();
    // SAFETY: `opts` is the caller's keydict, live for the call.
    let builtin = unsafe { (*opts).builtin }.unwrap_or(false);
    if buf == -1 {
        if builtin {
            error = Error::validation(c"builtin=true not implemented");
            return ApiDict::EMPTY.reported(error);
        }
        return commands_array(None).reported(error);
    }
    let b = find_buffer_by_handle(buf)?;
    let (false, Some(b)) = (builtin, b) else {
        return Ok(ApiDict::EMPTY);
    };
    commands_array(Some(b)).reported(error)
}
