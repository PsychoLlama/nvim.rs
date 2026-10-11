//! `nvim_parse_cmd()`: an Ex command line as a Dict.
//!
//! It runs the real parser (`parse_cmdline`) over the string and renders every
//! field the caller could need to rebuild it -- the command name, the bang, the
//! range, the count, the register, the arguments, the magic characters and the
//! whole `cmod_*` modifier set.  [`parse_map_cmd`] is the `:map`-family special
//! case, whose arguments the generic splitter would mangle.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api_error;
use crate::charset::skip;
use crate::message_fmt::msg_cstr;
use crate::types::CmdIdx;
use crate::types::{CmdLine, ExArgt, NUL};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int};
use core::slice;

/// A `:map`-family right-hand side is one opaque string, however much
/// whitespace it contains, so the arguments are exactly "lhs" and "rhs".
fn parse_map_cmd(arg: &[u8]) -> Array {
    let mut args: Array = Array::with_capacity(2);
    let lhs_len = skip::to_white(arg);
    args.push(Object::string(String_0::from_bytes(&arg[..lhs_len])));
    let rhs = &arg[lhs_len + skip::white(&arg[lhs_len..])..];
    if !rhs.is_empty() {
        // The rest of the line is one opaque argument.
        args.push(Object::string(String_0::from_bytes(rhs)));
    }
    args
}

/// The command's arguments, split the way the command itself would have them.
/// The line `excmd` holds is NUL-terminated, which is what the splitters
/// below walk to.
fn parse_args(excmd: &mut ExArg) -> Array {
    let (length, empty) = (
        excmd.line.arg().len(),
        excmd.line.byte_at(excmd.line.arg) == 0,
    );

    // `is_map_cmd` indexes the command table by `cmdidx`, so the `CmdIdx::SIZE`
    // guard has to stay in front of it rather than be hoisted alongside.
    // SAFETY: `cmdidx` is in range, checked immediately to its left.
    if excmd.cmdidx != CmdIdx::SIZE && is_map_cmd(excmd.cmdidx) && !empty {
        return parse_map_cmd(excmd.line.arg());
    }
    if excmd.argt.has(ExArgt::NOSPC) {
        // One argument, whitespace and all.
        if empty {
            return Array::EMPTY;
        }
        let mut args: Array = Array::with_capacity(1);
        args.push(Object::string(String_0::from_bytes(excmd.line.arg())));
        return args;
    }

    // `uc_split_args_iter` unescapes one argument per call.
    let arg = excmd.line.arg();
    debug_assert_eq!(arg.len(), length);
    let mut args = Array::with_capacity(uc_nargs_upper_bound(arg));
    let mut end = 0;
    let mut word = Vec::with_capacity(length);
    let mut done = false;
    while !done {
        done = uc_split_args_iter(arg, &mut end, &mut word);
        if !word.is_empty() {
            args.push(Object::string(String_0::from_bytes(&word)));
        }
    }
    args
}

/// The name of the command `excmd` names: a user command's own spelling
/// (`user`), the built-in table's, or the empty string where the line named
/// no command.
fn command_name(excmd: &ExArg, user: Option<&CStr>) -> String_0 {
    if excmd.cmdidx == CmdIdx::SIZE {
        return String_0::from_cstr(c"");
    }
    if let Some(name) = user {
        return String_0::from_cstr(name);
    }
    // `cmdidx` is a built-in index, checked against `CmdIdx::SIZE` above.
    String_0::from_cstr(builtin_command_cstr(excmd.cmdidx))
}

/// How the command's range is counted, as the `addr` field's string.
fn addr_type_name(addr_type: CmdAddr) -> &'static CStr {
    match addr_type {
        CmdAddr::Lines => c"line",
        CmdAddr::Arguments => c"arg",
        CmdAddr::Buffers => c"buf",
        CmdAddr::LoadedBuffers => c"load",
        CmdAddr::Windows => c"win",
        CmdAddr::Tabs => c"tab",
        CmdAddr::Quickfix => c"qf",
        CmdAddr::NoRange => c"none",
        // CmdAddr::Other and CmdAddr::Unsigned have no name of their own.
        _ => c"?",
    }
}

/// Collect `entries` into an arena Dict sized to hold exactly them.
///
/// Sizing from the same array that is then drained is what keeps the
/// dictionary to one allocation.
fn dict_of<const N: usize>(entries: [(&'static CStr, Object); N]) -> ApiDict {
    let mut dict = ApiDict::with_capacity(N);
    for (key, value) in entries {
        dict.insert(key, value);
    }
    dict
}

/// The `mods` sub-dictionary: every command modifier the line carried.
fn parse_mods(cmdmod: &CmdMod) -> ApiDict {
    let pattern = cmdmod
        .cmod_filter
        .as_ref()
        .map_or(String_0::NULL, |filter| String_0::from_cstr(&filter.pat));
    let filter = dict_of([
        (c"pattern", Object::string(pattern)),
        (c"force", Object::boolean(cmdmod.cmod_filter_force)),
    ]);

    let flag = |mask: CmdModFlags| Object::boolean(cmdmod.cmod_flags.has(mask));
    let split_flag = |mask: c_int| Object::boolean(cmdmod.cmod_split & mask != 0);
    // The four placements are mutually exclusive; no modifier is "".
    let split: &CStr = if cmdmod.cmod_split & WSP_BOT as c_int != 0 {
        c"botright"
    } else if cmdmod.cmod_split & WSP_TOP as c_int != 0 {
        c"topleft"
    } else if cmdmod.cmod_split & WSP_BELOW as c_int != 0 {
        c"belowright"
    } else if cmdmod.cmod_split & WSP_ABOVE as c_int != 0 {
        c"aboveleft"
    } else {
        c""
    };

    dict_of([
        (c"filter", Object::dict(filter)),
        (c"silent", flag(CmdModFlags::SILENT)),
        (c"emsg_silent", flag(CmdModFlags::ERRSILENT)),
        (c"unsilent", flag(CmdModFlags::UNSILENT)),
        (c"sandbox", flag(CmdModFlags::SANDBOX)),
        (c"noautocmd", flag(CmdModFlags::NOAUTOCMD)),
        // Both counts are stored one higher than they read, so that zero
        // means "not given".
        (c"tab", Object::integer((cmdmod.cmod_tab - 1) as Integer)),
        (
            c"verbose",
            Object::integer((cmdmod.cmod_verbose - 1) as Integer),
        ),
        (c"browse", flag(CmdModFlags::BROWSE)),
        (c"confirm", flag(CmdModFlags::CONFIRM)),
        (c"hide", flag(CmdModFlags::HIDE)),
        (c"keepalt", flag(CmdModFlags::KEEPALT)),
        (c"keepjumps", flag(CmdModFlags::KEEPJUMPS)),
        (c"keepmarks", flag(CmdModFlags::KEEPMARKS)),
        (c"keeppatterns", flag(CmdModFlags::KEEPPATTERNS)),
        (c"lockmarks", flag(CmdModFlags::LOCKMARKS)),
        (c"noswapfile", flag(CmdModFlags::NOSWAPFILE)),
        (c"vertical", split_flag(WSP_VERT as c_int)),
        (c"horizontal", split_flag(WSP_HOR as c_int)),
        (c"split", Object::string(String_0::from_cstr(split))),
    ])
}

pub fn nvim_parse_cmd(str: String_0, _opts: &mut KeyDict_empty) -> Result<KeyDict_cmd, Error> {
    let mut error = Error::none();
    // Every key unset; the answer names only what the parse found.
    let mut result = KeyDict_cmd::default();
    let mut excmd = ExArg::default();
    // SAFETY: a plain C aggregate whose all-zero state is the valid
    // "nothing parsed yet" one, as the C original's CLEAR_FIELD relies on.
    let mut cmdinfo: CmdParseInfo = crate::ex_getln::CMD_PARSE_INFO_INIT;

    let mut errormsg = None;
    // SAFETY: `str` names `len` readable bytes, copied into the line the
    // parse takes over.
    let line = CmdLine::from_bytes(unsafe { slice::from_raw_parts(str.data().cast(), str.len()) });
    let parsed = parse_cmdline(line, &mut excmd, &mut cmdinfo, &mut errormsg);
    if !parsed {
        match &errormsg {
            // SAFETY: `err` is live; the message takes no argument.
            None => {
                error = Error::exception(c"Parsing command-line");
            }
            Some(msg) => {
                let msg = msg_cstr(msg);
                error = api_error!(kErrorTypeException, "Parsing command-line: {msg}");
            }
        }
        return Err(error);
    }

    // The `useridx`th entry of one of the two user-command tables: its
    // name and its own default count, copied out.
    let useridx = usize::try_from(excmd.useridx).ok();
    let facts = |table: Table| {
        table.with(|cmds| {
            let cmd = cmds.get(useridx?)?;
            Some((cmd.name.clone(), cmd.default_count))
        })
    };
    // The arguments `parse_cmdline` left in `excmd`.
    let args = parse_args(&mut excmd);
    let user = match excmd.cmdidx {
        CmdIdx::USER => facts(Table::Global),
        CmdIdx::USER_BUF => facts(Table::Buffer(Buf::current())),
        _ => None,
    };
    // A user command carries its own default count.
    let default_count = user.as_ref().map(|(_, def)| *def);
    result.cmd = Some(command_name(&excmd, user.as_ref().map(|(name, _)| &**name)));

    if excmd.argt.has(ExArgt::RANGE) && excmd.addr_count > 0 {
        // Two addresses give both bounds, one gives only `line2`.
        let mut range: Array = Array::with_capacity(2);
        if excmd.addr_count > 1 {
            range.push(Object::integer(excmd.line1 as Integer));
        }
        range.push(Object::integer(excmd.line2 as Integer));
        result.range = Some(range);
    }

    if excmd.argt.has(ExArgt::COUNT) {
        let count: Integer = if excmd.addr_count > 0 {
            excmd.line2 as Integer
        } else {
            default_count.unwrap_or(0) as Integer
        };
        // A zero count that nothing asked for is left unset.
        if excmd.addr_count > 0 || default_count.is_some_and(|def| def != 0) || count != 0 {
            result.count = Some(count);
        }
    }

    if excmd.argt.has(ExArgt::REGSTR) {
        let reg: [c_char; 2] = [excmd.regname as c_char, NUL as c_char];
        // SAFETY: `reg` is NUL-terminated and alive for the copy.
        result.reg = Some(unsafe { cstr_to_string(reg.as_ptr()) });
    }

    result.bang = Some(excmd.forceit);
    result.args = Some(args);

    // `:command -nargs=` spelling of how many arguments the command takes.
    let nargs: &CStr = if !excmd.argt.has(ExArgt::EXTRA) {
        c"0"
    } else if excmd.argt.has(ExArgt::NOSPC) {
        if excmd.argt.has(ExArgt::NEEDARG) {
            c"1"
        } else {
            c"?"
        }
    } else if excmd.argt.has(ExArgt::NEEDARG) {
        c"+"
    } else {
        c"*"
    };
    let nargs = String_0::from_cstr(nargs);
    result.nargs = Some(Object::string(nargs));
    result.addr = Some(String_0::from_cstr(addr_type_name(excmd.addr_type)));
    let next = excmd.line.next_cmd().unwrap_or_default();
    result.nextcmd = Some(String_0::from_bytes(next));
    result.mods = Some(parse_mods(&cmdinfo.cmdmod));
    result.magic = Some(dict_of([
        (c"file", Object::boolean(cmdinfo.magic.file)),
        (c"bar", Object::boolean(cmdinfo.magic.bar)),
    ]));
    // The `:filter` pattern `parse_mods` copied out is freed here, not before.
    undo_cmdmod(&mut cmdinfo.cmdmod);
    result.reported(error)
}
