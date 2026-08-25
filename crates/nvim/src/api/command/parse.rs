//! `nvim_parse_cmd()`: an Ex command line as a Dict.
//!
//! It runs the real parser (`parse_cmdline`) over the string and renders every
//! field the caller could need to rebuild it -- the command name, the bang, the
//! range, the count, the register, the arguments, the magic characters and the
//! whole `cmod_*` modifier set.  [`parse_map_cmd`] is the `:map`-family special
//! case, whose arguments the generic splitter would mangle.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, array_add, dict_put};
use crate::types::builders::static_cstring;
use crate::types::{ExArgt, NUL};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// A `:map`-family right-hand side is one opaque string, however much
/// whitespace it contains, so the arguments are exactly "lhs" and "rhs".
unsafe fn parse_map_cmd(arg_str: *const c_char, arena: *mut Arena) -> Array {
    unsafe {
        let mut args: Array = arena_array(arena, 2);
        let lhs_start: *mut c_char = arg_str.cast_mut();
        let lhs_end: *mut c_char = skiptowhite(lhs_start);
        let lhs_len = lhs_end.offset_from(lhs_start) as size_t;
        array_add(
            &mut args,
            Object::string(cstrn_as_string(lhs_start, lhs_len)),
        );
        let rhs_start: *mut c_char = skipwhite(lhs_end);
        if *rhs_start != NUL as c_char {
            let rhs_len = strlen(rhs_start);
            array_add(
                &mut args,
                Object::string(cstrn_as_string(rhs_start, rhs_len)),
            );
        }
        args
    }
}

/// The command's arguments, split the way the command itself would have them.
///
/// # Safety
/// As [`parse_map_cmd`]; `arena` must be the dispatcher's.
unsafe fn parse_args(ea: &exarg_T, arena: *mut Arena) -> Array {
    // SAFETY: caller contract.
    let (length, empty) = unsafe { (strlen(ea.arg), *ea.arg == NUL as c_char) };

    // `is_map_cmd` indexes the command table by `cmdidx`, so the `CMD_SIZE`
    // guard has to stay in front of it rather than be hoisted alongside.
    // SAFETY: `cmdidx` is in range, checked immediately to its left.
    if ea.cmdidx != CMD_SIZE && unsafe { is_map_cmd(ea.cmdidx) } && !empty {
        // SAFETY: caller contract.
        return unsafe { parse_map_cmd(ea.arg, arena) };
    }
    if ea.argt.has(ExArgt::NOSPC) {
        // One argument, whitespace and all.
        if empty {
            return Array::EMPTY;
        }
        let mut args: Array = arena_array(arena, 1);
        // SAFETY: room for the one item was just reserved.
        unsafe { array_add(&mut args, Object::string(cstrn_as_string(ea.arg, length))) };
        return args;
    }

    // `uc_split_args_iter` unescapes into `buf`, one NUL-separated argument
    // per call, so the whole split fits in one `length + 1` block.
    // SAFETY: `args` is reserved for the upper bound the splitter itself
    // computes, and `buf` advances by exactly what each call wrote.
    unsafe {
        let mut buf: *mut c_char = arena_alloc(arena, length + 1, false).cast::<c_char>();
        let mut args: Array = arena_array(arena, uc_nargs_upper_bound(ea.arg, length));
        let (mut end, mut len): (size_t, size_t) = (0, 0);
        let mut done = false;
        while !done {
            done = uc_split_args_iter(ea.arg, length, &raw mut end, buf, &raw mut len);
            if len > 0 {
                array_add(&mut args, Object::string(cstrn_as_string(buf, len)));
                buf = buf.add(len + 1);
            }
        }
        args
    }
}

/// The name of the command `ea` names: a user command's own spelling, the
/// built-in table's, or the empty string where the line named no command.
///
/// # Safety
/// `cmd` must be null or point at a live `ucmd_T`.
unsafe fn command_name(ea: &exarg_T, cmd: *const ucmd_T) -> *const c_char {
    if ea.cmdidx == CMD_SIZE {
        return c"".as_ptr();
    }
    if !cmd.is_null() {
        // SAFETY: caller contract.
        return unsafe { (*cmd).uc_name };
    }
    // SAFETY: `cmdidx` is a built-in index, checked against `CMD_SIZE` above.
    unsafe { get_command_name(ptr::null_mut::<expand_T>(), ea.cmdidx as c_int) }
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
/// Sizing from the same array that is then drained is what makes the puts
/// sound: `dict_put`'s only requirement is room, and `entries.len()` is it.
fn dict_of<const N: usize>(arena: *mut Arena, entries: [(&'static CStr, Object); N]) -> Dict {
    let mut dict = arena_dict(arena, N);
    // SAFETY: the dict was reserved for exactly `N` pairs and this is the
    // only thing that writes to it.
    unsafe {
        for (key, value) in entries {
            dict_put(&mut dict, key, value);
        }
    }
    dict
}

/// The `mods` sub-dictionary: every command modifier the line carried.
unsafe fn parse_mods(cmdmod: &cmdmod_T, arena: *mut Arena) -> Dict {
    // SAFETY: `cmod_filter_pat` is null or a NUL-terminated pattern, and the
    // arena copy outlives the Dict.
    let pattern = unsafe { arena_string(arena, cstr_as_string(cmdmod.cmod_filter_pat)) };
    let filter = dict_of(
        arena,
        [
            (c"pattern", Object::string(pattern)),
            (c"force", Object::boolean(cmdmod.cmod_filter_force)),
        ],
    );

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

    dict_of(
        arena,
        [
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
            (c"split", Object::string(static_cstring(split))),
        ],
    )
}

pub unsafe fn nvim_parse_cmd(
    str: String_0,
    _opts: *mut KeyDict_empty,
    arena: *mut Arena,
) -> Result<KeyDict_cmd, Error> {
    let mut error = ERROR_INIT;
    let err = &mut error;
    // SAFETY: all three are plain C aggregates whose all-zero state is the
    // valid "nothing parsed yet" one, as the C original's CLEAR_FIELD relies
    // on.
    let (mut result, mut ea, mut cmdinfo) = unsafe {
        (
            ::core::mem::zeroed::<KeyDict_cmd>(),
            ::core::mem::zeroed::<exarg_T>(),
            ::core::mem::zeroed::<CmdParseInfo>(),
        )
    };

    let mut errormsg: *const c_char = ptr::null();
    // SAFETY: `arena` is the dispatcher's and `str` is `size` readable bytes;
    // the arena copy outlives everything `parse_cmdline` leaves pointing into
    // it, including `ea.arg` and `ea.nextcmd`.
    let parsed = unsafe {
        let mut cmdline = arena_memdupz(arena, str.data(), str.len());
        parse_cmdline(
            &raw mut cmdline,
            &raw mut ea,
            &raw mut cmdinfo,
            &raw mut errormsg,
        )
    };
    if !parsed {
        // SAFETY: `err` is live; `errormsg`, when set, is the parser's own
        // NUL-terminated literal.
        unsafe {
            if errormsg.is_null() {
                api_set_error(err, kErrorTypeException, c"Parsing command-line".as_ptr());
            } else {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Parsing command-line: %s".as_ptr(),
                    errormsg,
                );
            }
        }
        return Err(error);
    }

    // SAFETY: `useridx` indexes the garray the matching `cmdidx` names, and
    // `parse_args` reads the arguments `parse_cmdline` left in `ea`.
    let (args, cmd) = unsafe {
        let args = parse_args(&ea, arena);
        let nth = |ga_data: *mut c_void| ga_data.cast::<ucmd_T>().add(ea.useridx as usize);
        let cmd: *mut ucmd_T = match ea.cmdidx {
            CMD_USER => ucmds.with(|ga| nth(ga.ga_data)),
            CMD_USER_BUF => nth((*curbuf.get()).b_ucmds.ga_data),
            _ => ptr::null_mut(),
        };
        (args, cmd)
    };
    // A user command carries its own default count.
    // SAFETY: `cmd`, when non-null, points at a live `ucmd_T`.
    let uc_def = (!cmd.is_null()).then(|| unsafe { (*cmd).uc_def });

    result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__cmd;
    // SAFETY: both names are NUL-terminated, and outlive the reply.
    result.cmd = unsafe { cstr_as_string(command_name(&ea, cmd)) };

    if ea.argt.has(ExArgt::RANGE) && ea.addr_count > 0 {
        // Two addresses give both bounds, one gives only `line2`.
        let mut range: Array = arena_array(arena, 2);
        // SAFETY: at most the two items just reserved are added.
        unsafe {
            if ea.addr_count > 1 {
                array_add(&mut range, Object::integer(ea.line1 as Integer));
            }
            array_add(&mut range, Object::integer(ea.line2 as Integer));
        }
        result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__range;
        result.range = range;
    }

    if ea.argt.has(ExArgt::COUNT) {
        let count: Integer = if ea.addr_count > 0 {
            ea.line2 as Integer
        } else {
            uc_def.unwrap_or(0) as Integer
        };
        // A zero count that nothing asked for is left unset.
        if ea.addr_count > 0 || uc_def.is_some_and(|def| def != 0) || count != 0 {
            result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__count;
            result.count = count;
        }
    }

    if ea.argt.has(ExArgt::REGSTR) {
        let mut reg: [c_char; 2] = [ea.regname as c_char, NUL as c_char];
        result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__reg;
        // SAFETY: `reg` is NUL-terminated and alive until the copy is made.
        result.reg = unsafe { arena_string(arena, cstr_as_string(reg.as_mut_ptr())) };
    }

    result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__bang;
    result.bang = ea.forceit != 0;
    result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__args;
    result.args = args;

    // `:command -nargs=` spelling of how many arguments the command takes.
    let nargs: &CStr = if !ea.argt.has(ExArgt::EXTRA) {
        c"0"
    } else if ea.argt.has(ExArgt::NOSPC) {
        if ea.argt.has(ExArgt::NEEDARG) {
            c"1"
        } else {
            c"?"
        }
    } else if ea.argt.has(ExArgt::NEEDARG) {
        c"+"
    } else {
        c"*"
    };
    result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__nargs;
    // SAFETY: the arena copy is what the reply keeps; `nargs` is a literal.
    result.nargs = Object::string(unsafe { arena_string(arena, static_cstring(nargs)) });

    result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__addr;
    result.addr = static_cstring(addr_type_name(ea.addr_type));
    result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__nextcmd;
    // SAFETY: `ea.nextcmd` points into the arena copy of the command line.
    result.nextcmd = unsafe { cstr_as_string(ea.nextcmd) };

    result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__mods;
    // SAFETY: `cmdinfo.cmdmod` is what `parse_cmdline` filled in.
    result.mods = unsafe { parse_mods(&cmdinfo.cmdmod, arena) };

    result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__magic;
    result.magic = dict_of(
        arena,
        [
            (c"file", Object::boolean(cmdinfo.magic.file)),
            (c"bar", Object::boolean(cmdinfo.magic.bar)),
        ],
    );

    // The `:filter` pattern `parse_mods` copied out is freed here, not before.
    // SAFETY: paired with the `parse_cmdline` above.
    unsafe { undo_cmdmod(&mut cmdinfo.cmdmod) };
    result.reported(error)
}
