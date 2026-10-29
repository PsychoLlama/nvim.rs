//! User commands: `:command` from the API.
//!
//! `create_user_command` is the shared implementation -- it validates the
//! name, decodes the `nargs`/`range`/`count`/`addr`/`complete` keyset into
//! the `uc_add_command` flags, and accepts either a command string or a
//! `LuaRef` -- and the four `nvim_*_user_command` entry points differ only
//! in whether they are buffer-local.  The two `get_commands` spellings
//! render the table back.

#![forbid(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{
    Bad, err_bad_number, err_bad_value, err_expected, err_invalid,
};
use crate::api_error;
use crate::cstr;
use crate::message_fmt::msg_cstr;
use crate::types::{ExArgt, ExpandContext};
use crate::winlayer::graph::switch_buffer;

pub fn nvim_create_user_command(
    channel_id: uint64_t,
    name: String_0,
    cmd: Object,
    opts: &mut KeyDict_user_command,
) -> Result<(), Error> {
    create_user_command(channel_id, name, cmd, opts, false)
}

pub fn nvim_del_user_command(name: String_0) -> Result<(), Error> {
    nvim_buf_del_user_command(-1, name)
}

pub fn nvim_buf_create_user_command(
    channel_id: uint64_t,
    buf: BufferHandle,
    name: String_0,
    cmd: Object,
    opts: &mut KeyDict_user_command,
) -> Result<(), Error> {
    let Some(target_buf) = find_buffer_by_handle(buf)? else {
        return Ok(());
    };
    // The command is added to whichever buffer is current, so the lookup's
    // answer stands in for the caller's for the length of the call.
    let saved = switch_buffer(target_buf);
    let made = create_user_command(channel_id, name, cmd, opts, true);
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
    if uc_del_named(table, name.as_cstr()) {
        return ().reported(error);
    }
    let name = msg_cstr(name.as_cstr());
    error = api_error!(kErrorTypeException, "Invalid command (not found): {name}");
    ().reported(error)
}

pub fn create_user_command(
    channel_id: uint64_t,
    name: String_0,
    cmd: Object,
    opts: &mut KeyDict_user_command,
    buffer_local: bool,
) -> Result<(), Error> {
    let force: bool;
    let mut argt = ExArgt::NONE;
    let mut def: int64_t = -1;
    let mut addr_type_arg = CmdAddr::NoRange;
    let mut context = ExpandContext::Nothing;
    let mut compl_arg = None;
    let rep: &[u8];
    let mut luaref: LuaRef = LUA_NOREF;
    let mut compl_luaref: LuaRef = LUA_NOREF;
    let mut preview_luaref: LuaRef = LUA_NOREF;
    let cmd_name = name.as_bytes();
    // The refusal is held rather than returned: the cleanup below has to run
    // whichever way the block left.
    let mut failed = None;
    '_err: {
        if uc_validate_name(cmd_name).is_none() {
            failed = Some(err_bad_value(c"command name", name.as_cstr()));
            break '_err;
        }
        if mb_islower(::core::ffi::c_int::from(cstr::byte_at(cmd_name, 0))) {
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
            if nargs.len() > 1 {
                failed = Some(err_bad_value(c"nargs", nargs.as_cstr()));
                break '_err;
            }
            // The empty string has no byte 0, and falls to the arm that
            // rejects it.
            match nargs.as_bytes().first().copied().unwrap_or(0) {
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
            let percent = range.as_bytes().first() == Some(&b'%');
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
            match parse_addr_type_arg(addr.as_bytes(), addr.as_cstr().to_bytes()) {
                Ok(parsed) => addr_type_arg = parsed,
                Err(word) => {
                    // Upstream cut the caller's string at the word the
                    // message named, and quotes what is left of it.
                    failed = Some(err_bad_value(c"addr", &cstr::owned(word)));
                    break '_err;
                }
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
            let (value, shown) = (complete.as_bytes(), complete.as_cstr().to_bytes());
            let parsed = parse_compl_arg(value, shown, &mut context, &mut argt, &mut compl_arg);
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
                Some(desc) => desc.as_bytes(),
                None => b"",
            };
        } else if let Some(body) = cmd.as_string() {
            rep = body.as_bytes();
        } else {
            let expected = c"Function or String";
            failed = Some(err_expected(c"command", expected, None));
            break '_err;
        }

        let _sctx = api_set_sctx(channel_id);
        // The definition owns the parsed argument and the three references
        // from here on, and gives them back if it is never installed.
        let def = Definition {
            argt,
            default_count: def,
            complete: context,
            complete_arg: compl_arg.take(),
            addr_type: addr_type_arg,
            luaref: ::core::mem::replace(&mut luaref, LUA_NOREF),
            complete_luaref: ::core::mem::replace(&mut compl_luaref, LUA_NOREF),
            preview_luaref: ::core::mem::replace(&mut preview_luaref, LUA_NOREF),
        };
        if uc_add_command(cmd_name, name.len(), rep, def, buffer_local, force).is_err() {
            failed = Some(Error::exception(c"Failed to create user command"));
        }
        return match failed {
            Some(failed) => Err(failed),
            None => Ok(()),
        };
    }
    // Only reached when the command was never added, so this call still owns
    // the references it took.
    for reference in [luaref, compl_luaref, preview_luaref] {
        release_luaref(reference);
    }
    match failed {
        Some(failed) => Err(failed),
        None => Ok(()),
    }
}

pub fn nvim_get_commands(opts: &mut KeyDict_get_commands) -> Result<ApiDict, Error> {
    nvim_buf_get_commands(-1, opts)
}

pub fn nvim_buf_get_commands(
    buf: BufferHandle,
    opts: &mut KeyDict_get_commands,
) -> Result<ApiDict, Error> {
    let mut error = Error::none();
    let builtin = opts.builtin.unwrap_or(false);
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
