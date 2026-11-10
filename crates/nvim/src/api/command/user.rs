//! User commands: `:command` from the API.
//!
//! `create_user_command` is the shared implementation -- it validates the
//! name, decodes the `nargs`/`range`/`count`/`addr`/`complete` keyset into
//! the `uc_add_command` flags, and accepts either a command string or a
//! `LuaRef` -- and the four `nvim_*_user_command` entry points differ only
//! in whether they are buffer-local.  The two `get_commands` spellings
//! render the table back.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::{
    Bad, err_bad_number, err_bad_value, err_expected, err_invalid,
};
use crate::api_error;
use crate::cmdexpand::defs::ExpandContext;
use crate::cstr;
use crate::message_fmt::msg_cstr;
use crate::types::ExArgt;
use crate::types::Handle;
use crate::usercmd::AttrError;
use crate::winlayer::graph::switch_buffer;
use std::ffi::CString;

pub fn nvim_create_user_command(
    channel_id: uint64_t,
    name: ApiString,
    cmd: Object,
    opts: &mut KeyDict_user_command,
) -> Result<(), Error> {
    create_user_command(channel_id, name, cmd, opts, false)
}

pub fn nvim_del_user_command(name: ApiString) -> Result<(), Error> {
    nvim_buf_del_user_command(Handle::new(-1), name)
}

pub fn nvim_buf_create_user_command(
    channel_id: uint64_t,
    buf: BufferHandle,
    name: ApiString,
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

pub fn nvim_buf_del_user_command(buf: BufferHandle, name: ApiString) -> Result<(), Error> {
    let table = if buf == Handle::new(-1) {
        Table::Global
    } else {
        let Some(b) = find_buffer_by_handle(buf)? else {
            return Ok(());
        };
        Table::Buffer(b)
    };
    if uc_del_named(table, name.as_cstr()) {
        return Ok(());
    }
    let name = msg_cstr(name.as_cstr());
    Err(api_error!(
        ERROR_TYPE_EXCEPTION,
        "Invalid command (not found): {name}"
    ))
}

pub fn create_user_command(
    channel_id: uint64_t,
    name: ApiString,
    cmd: Object,
    opts: &mut KeyDict_user_command,
    buffer_local: bool,
) -> Result<(), Error> {
    let cmd_name = name.as_bytes();
    check_command_name(&name)?;
    if opts.range.is_some() && opts.count.is_some() {
        return Err(Error::validation(c"Cannot use both 'range' and 'count'"));
    }
    let mut argt = nargs_flags(opts.nargs.as_ref())?;
    if opts.complete.is_some() && argt == ExArgt::NONE {
        return Err(Error::validation(c"'complete' used without 'nargs'"));
    }
    let mut range = CommandRange {
        addr_type: CmdAddr::NoRange,
        default_count: -1,
    };
    range.read_range(opts.range.as_ref(), &mut argt)?;
    range.read_count(opts.count.as_ref(), &mut argt)?;
    range.read_addr(opts.addr.as_ref(), &mut argt)?;
    argt |= switch_flags(opts);
    // An unsupplied `force` defaults to true: `nvim_create_user_command`
    // replaces an existing command unless told otherwise.
    let force = opts.force.unwrap_or(true);

    // From here the call takes Lua references out of `opts`, and gives them
    // back if it refuses before the command owns them.
    let mut completion = Completion {
        context: ExpandContext::Nothing,
        arg: None,
        luaref: LUA_NOREF,
    };
    completion.read(opts, &mut argt)?;
    let preview_luaref = match take_preview(opts, &mut argt) {
        Ok(reference) => reference,
        Err(refused) => {
            release_luaref(completion.luaref);
            return Err(refused);
        }
    };
    let (luaref, rep) = match command_body(&cmd, opts) {
        Ok(body) => body,
        Err(refused) => {
            release_luaref(completion.luaref);
            release_luaref(preview_luaref);
            return Err(refused);
        }
    };

    let _sctx = api_set_sctx(channel_id);
    // The definition owns the parsed argument and the three references
    // from here on, and gives them back if it is never installed.
    let def = Definition {
        argt,
        default_count: range.default_count,
        complete: completion.context,
        complete_arg: completion.arg,
        addr_type: range.addr_type,
        luaref,
        complete_luaref: completion.luaref,
        preview_luaref,
    };
    uc_add_command(cmd_name, name.len(), rep, def, buffer_local, force).map_err(|refused| {
        // Upstream printed the message as well as refusing.
        refused.report();
        Error::exception(c"Failed to create user command")
    })
}

/// Refuse a name that is not a user command's: invalid, or not starting with
/// an uppercase letter.
fn check_command_name(name: &ApiString) -> Result<(), Error> {
    let cmd_name = name.as_bytes();
    if uc_validate_name(cmd_name).is_none() {
        return Err(err_bad_value(c"command name", name.as_cstr()));
    }
    if mb_islower(::core::ffi::c_int::from(cstr::byte_at(cmd_name, 0))) {
        let what = c"command name (must start with uppercase)";
        return Err(err_bad_value(what, name.as_cstr()));
    }
    Ok(())
}

/// The argument flags `nargs` asks for: a number (0 or 1) or one of `*`,
/// `?`, `+`.
fn nargs_flags(given: Option<&Object>) -> Result<ExArgt, Error> {
    let Some(given) = given else {
        return Ok(ExArgt::NONE);
    };
    if let Some(nargs) = given.as_integer() {
        return match nargs {
            0 => Ok(ExArgt::NONE),
            1 => Ok(ExArgt::EXTRA | ExArgt::NOSPC | ExArgt::NEEDARG),
            _ => Err(err_bad_number(c"nargs", nargs)),
        };
    }
    let Some(nargs) = given.as_string() else {
        return Err(err_invalid(c"nargs", Bad::Unsaid));
    };
    if nargs.len() > 1 {
        return Err(err_bad_value(c"nargs", nargs.as_cstr()));
    }
    // The empty string has no byte 0, and falls to the arm that rejects it.
    match nargs.as_bytes().first().copied().unwrap_or(0) {
        b'*' => Ok(ExArgt::EXTRA),
        b'?' => Ok(ExArgt::EXTRA | ExArgt::NOSPC),
        b'+' => Ok(ExArgt::EXTRA | ExArgt::NEEDARG),
        _ => Err(err_bad_value(c"nargs", nargs.as_cstr())),
    }
}

/// What `range`, `count` and `addr` say about the command's range.
#[derive(Debug)]
struct CommandRange {
    addr_type: CmdAddr,
    /// The default count or line, -1 when none was given.
    default_count: int64_t,
}

impl CommandRange {
    /// `range`: `true`, `"%"` (default the whole file) or a default count.
    fn read_range(&mut self, given: Option<&Object>, argt: &mut ExArgt) -> Result<(), Error> {
        let Some(given) = given else {
            return Ok(());
        };
        if let Some(range) = given.as_boolean() {
            if range {
                *argt |= ExArgt::RANGE;
                self.addr_type = CmdAddr::Lines;
            }
        } else if let Some(range) = given.as_string() {
            let percent = range.as_bytes().first() == Some(&b'%');
            if !(percent && range.len() == 1) {
                return Err(err_invalid(c"range", Bad::Unsaid));
            }
            *argt |= ExArgt::RANGE | ExArgt::DFLALL;
            self.addr_type = CmdAddr::Lines;
        } else if let Some(range) = given.as_integer() {
            *argt |= ExArgt::RANGE | ExArgt::ZEROR;
            self.default_count = range;
            self.addr_type = CmdAddr::Lines;
        } else {
            return Err(err_invalid(c"range", Bad::Unsaid));
        }
        Ok(())
    }

    /// `count`: `true` (default 0) or a default count.
    fn read_count(&mut self, given: Option<&Object>, argt: &mut ExArgt) -> Result<(), Error> {
        let Some(given) = given else {
            return Ok(());
        };
        if let Some(count) = given.as_boolean() {
            if count {
                *argt |= ExArgt::COUNT | ExArgt::ZEROR | ExArgt::RANGE;
                self.addr_type = CmdAddr::Other;
                self.default_count = 0;
            }
        } else if let Some(count) = given.as_integer() {
            *argt |= ExArgt::COUNT | ExArgt::ZEROR | ExArgt::RANGE;
            self.addr_type = CmdAddr::Other;
            self.default_count = count;
        } else {
            return Err(err_invalid(c"count", Bad::Unsaid));
        }
        Ok(())
    }

    /// `addr`: the name of what the range counts.
    fn read_addr(&mut self, given: Option<&Object>, argt: &mut ExArgt) -> Result<(), Error> {
        let Some(given) = given else {
            return Ok(());
        };
        let Some(addr) = given.as_string() else {
            let expected = api_typename(OBJECT_TYPE_STRING);
            let actual = api_typename(given.kind());
            return Err(err_expected(c"addr", expected, Some(actual)));
        };
        match parse_addr_type_arg(addr.as_bytes(), addr.as_cstr().to_bytes()) {
            Ok(parsed) => self.addr_type = parsed,
            Err(refused) => {
                // Upstream printed the message as well as refusing, cut the
                // caller's string at the word the message named, and quotes
                // what is left of it.
                refused.report();
                let AttrError::AddrType(word) = refused else {
                    unreachable!("an address type is refused by its word")
                };
                return Err(err_bad_value(c"addr", word.as_cstr()));
            }
        }
        *argt |= ExArgt::RANGE;
        if self.addr_type != CmdAddr::Lines {
            *argt |= ExArgt::ZEROR;
        }
        Ok(())
    }
}

/// The flags the four boolean keys switch on.
fn switch_flags(opts: &KeyDict_user_command) -> ExArgt {
    let mut argt = ExArgt::NONE;
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
    argt
}

/// How the command's argument completes: `complete` as a Lua function or a
/// `-complete=` name.
#[derive(Debug)]
struct Completion {
    context: ExpandContext,
    arg: Option<CString>,
    /// The Lua function's reference, taken out of the keyset; `LUA_NOREF`
    /// for none.
    luaref: LuaRef,
}

impl Completion {
    fn read(&mut self, opts: &mut KeyDict_user_command, argt: &mut ExArgt) -> Result<(), Error> {
        if opts.complete.as_ref().and_then(Object::as_luaref).is_some() {
            self.context = ExpandContext::UserLua;
            // The reference is this call's now, so the keyset must not
            // release it: the value moves out of the field.
            self.luaref = opts
                .complete
                .take()
                .and_then(Object::into_luaref)
                .expect("the arm above matched a LuaRef");
        } else if let Some(complete) = opts.complete.as_ref().and_then(Object::as_string) {
            let (value, shown) = (complete.as_bytes(), complete.as_cstr().to_bytes());
            let parsed = parse_compl_arg(value, shown, &mut self.context, argt, &mut self.arg);
            if let Err(refused) = parsed {
                // Upstream printed the message as well as refusing.
                refused.report();
                return Err(err_bad_value(c"complete", complete.as_cstr()));
            }
        } else if opts.complete.is_some() {
            let expected = c"Function or String";
            return Err(err_expected(c"complete", expected, None));
        }
        Ok(())
    }
}

/// `preview`: the Lua function `'inccommand'` calls, its reference taken
/// out of the keyset; `LUA_NOREF` for none.
fn take_preview(opts: &mut KeyDict_user_command, argt: &mut ExArgt) -> Result<LuaRef, Error> {
    let Some(given) = opts.preview.as_ref() else {
        return Ok(LUA_NOREF);
    };
    if given.as_luaref().is_none() {
        let expected = api_typename(OBJECT_TYPE_LUA_REF);
        let actual = api_typename(given.kind());
        return Err(err_expected(c"preview", expected, Some(actual)));
    }
    *argt |= ExArgt::PREVIEW;
    // As `complete`: the reference is this call's now.
    Ok(opts
        .preview
        .take()
        .and_then(Object::into_luaref)
        .expect("the check above matched a LuaRef"))
}

/// What the command runs: a Lua function (a new reference, shown as `desc`)
/// or a command string.
fn command_body<'a>(
    cmd: &'a Object,
    opts: &'a KeyDict_user_command,
) -> Result<(LuaRef, &'a [u8]), Error> {
    if let Some(body) = cmd.as_luaref() {
        let rep = match opts.desc.as_ref().and_then(Object::as_string) {
            Some(desc) => desc.as_bytes(),
            None => b"",
        };
        Ok((api_new_luaref(body), rep))
    } else if let Some(body) = cmd.as_string() {
        Ok((LUA_NOREF, body.as_bytes()))
    } else {
        Err(err_expected(c"command", c"Function or String", None))
    }
}

pub fn nvim_get_commands(opts: &mut KeyDict_get_commands) -> Result<ApiDict, Error> {
    nvim_buf_get_commands(Handle::new(-1), opts)
}

pub fn nvim_buf_get_commands(
    buf: BufferHandle,
    opts: &mut KeyDict_get_commands,
) -> Result<ApiDict, Error> {
    let builtin = opts.builtin.unwrap_or(false);
    if buf == Handle::new(-1) {
        if builtin {
            return Err(Error::validation(c"builtin=true not implemented"));
        }
        return Ok(commands_array(None));
    }
    let b = find_buffer_by_handle(buf)?;
    let (false, Some(b)) = (builtin, b) else {
        return Ok(ApiDict::new());
    };
    Ok(commands_array(Some(b)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nargs_is_a_count_of_zero_or_one() {
        assert_eq!(nargs_flags(None).ok(), Some(ExArgt::NONE));
        assert_eq!(
            nargs_flags(Some(&Object::integer(0))).ok(),
            Some(ExArgt::NONE)
        );
        let one = ExArgt::EXTRA | ExArgt::NOSPC | ExArgt::NEEDARG;
        assert_eq!(nargs_flags(Some(&Object::integer(1))).ok(), Some(one));
        assert!(nargs_flags(Some(&Object::integer(2))).is_err());
        assert!(nargs_flags(Some(&Object::boolean(true))).is_err());
    }

    #[test]
    fn a_count_range_defaults_its_count_and_counts_lines() {
        let mut range = CommandRange {
            addr_type: CmdAddr::NoRange,
            default_count: -1,
        };
        let mut argt = ExArgt::NONE;
        assert!(
            range
                .read_range(Some(&Object::integer(5)), &mut argt)
                .is_ok()
        );
        assert_eq!(argt, ExArgt::RANGE | ExArgt::ZEROR);
        assert_eq!(range.default_count, 5);
        assert_eq!(range.addr_type, CmdAddr::Lines);

        // `count = true` then counts something other than lines, from 0.
        assert!(
            range
                .read_count(Some(&Object::boolean(true)), &mut argt)
                .is_ok()
        );
        assert_eq!(range.default_count, 0);
        assert_eq!(range.addr_type, CmdAddr::Other);
        assert!(argt.has(ExArgt::COUNT));

        assert!(
            range
                .read_range(Some(&Object::float(1.0)), &mut argt)
                .is_err()
        );
    }
}
