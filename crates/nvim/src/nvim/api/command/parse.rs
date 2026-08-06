//! `nvim_parse_cmd()`: an Ex command line as a Dict.
//!
//! It runs the real parser (`parse_cmdline`) over the string and renders every
//! field the caller could need to rebuild it -- the command name, the bang, the
//! range, the count, the register, the arguments, the magic characters and the
//! whole `cmod_*` modifier set.  [`parse_map_cmd`] is the `:map`-family special
//! case, whose arguments the generic splitter would mangle.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::{array_add, dict_put};
use crate::src::nvim::types::builders::static_cstring;
use core::ffi::{CStr, c_char, c_int};
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
unsafe fn parse_args(ea: &exarg_T, arena: *mut Arena) -> Array {
    unsafe {
        let length = strlen(ea.arg);
        if ea.cmdidx != CMD_SIZE && is_map_cmd(ea.cmdidx) && *ea.arg != NUL as c_char {
            return parse_map_cmd(ea.arg, arena);
        }
        if ea.argt & EX_NOSPC as uint32_t != 0 {
            // One argument, whitespace and all.
            if *ea.arg == NUL as c_char {
                return Array::EMPTY;
            }
            let mut args: Array = arena_array(arena, 1);
            array_add(&mut args, Object::string(cstrn_as_string(ea.arg, length)));
            return args;
        }
        // `uc_split_args_iter` unescapes into `buf`, one NUL-separated argument
        // per call, so the whole split fits in one `length + 1` block.
        let mut buf: *mut c_char = arena_alloc(arena, length + 1, false).cast::<c_char>();
        let mut args: Array = arena_array(arena, uc_nargs_upper_bound(ea.arg, length));
        let mut end: size_t = 0;
        let mut len: size_t = 0;
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
unsafe fn command_name(ea: &exarg_T, cmd: *const ucmd_T) -> *const c_char {
    unsafe {
        if ea.cmdidx == CMD_SIZE {
            c"".as_ptr()
        } else if !cmd.is_null() {
            (*cmd).uc_name
        } else {
            get_command_name(ptr::null_mut::<expand_T>(), ea.cmdidx as c_int)
        }
    }
}

/// How the command's range is counted, as the `addr` field's string.
fn addr_type_name(addr_type: cmd_addr_T) -> &'static CStr {
    match addr_type {
        ADDR_LINES => c"line",
        ADDR_ARGUMENTS => c"arg",
        ADDR_BUFFERS => c"buf",
        ADDR_LOADED_BUFFERS => c"load",
        ADDR_WINDOWS => c"win",
        ADDR_TABS => c"tab",
        ADDR_QUICKFIX => c"qf",
        ADDR_NONE => c"none",
        // ADDR_OTHER and ADDR_UNSIGNED have no name of their own.
        _ => c"?",
    }
}

/// The `mods` sub-dictionary: every command modifier the line carried.
unsafe fn parse_mods(cmdmod: &cmdmod_T, arena: *mut Arena) -> Dict {
    unsafe {
        let mut filter: Dict = arena_dict(arena, 2);
        dict_put(
            &mut filter,
            c"pattern",
            Object::string(arena_string(arena, cstr_as_string(cmdmod.cmod_filter_pat))),
        );
        dict_put(
            &mut filter,
            c"force",
            Object::boolean(cmdmod.cmod_filter_force),
        );

        let flag = |mask: uint32_t| Object::boolean(cmdmod.cmod_flags & mask as c_int != 0);
        let mut mods: Dict = arena_dict(arena, 20);
        dict_put(&mut mods, c"filter", Object::dict(filter));
        dict_put(&mut mods, c"silent", flag(CMOD_SILENT));
        dict_put(&mut mods, c"emsg_silent", flag(CMOD_ERRSILENT));
        dict_put(&mut mods, c"unsilent", flag(CMOD_UNSILENT));
        dict_put(&mut mods, c"sandbox", flag(CMOD_SANDBOX));
        dict_put(&mut mods, c"noautocmd", flag(CMOD_NOAUTOCMD));
        // Both are stored one higher than they read, so that zero means
        // "not given".
        dict_put(
            &mut mods,
            c"tab",
            Object::integer((cmdmod.cmod_tab - 1) as Integer),
        );
        dict_put(
            &mut mods,
            c"verbose",
            Object::integer((cmdmod.cmod_verbose - 1) as Integer),
        );
        dict_put(&mut mods, c"browse", flag(CMOD_BROWSE));
        dict_put(&mut mods, c"confirm", flag(CMOD_CONFIRM));
        dict_put(&mut mods, c"hide", flag(CMOD_HIDE));
        dict_put(&mut mods, c"keepalt", flag(CMOD_KEEPALT));
        dict_put(&mut mods, c"keepjumps", flag(CMOD_KEEPJUMPS));
        dict_put(&mut mods, c"keepmarks", flag(CMOD_KEEPMARKS));
        dict_put(&mut mods, c"keeppatterns", flag(CMOD_KEEPPATTERNS));
        dict_put(&mut mods, c"lockmarks", flag(CMOD_LOCKMARKS));
        dict_put(&mut mods, c"noswapfile", flag(CMOD_NOSWAPFILE));
        dict_put(
            &mut mods,
            c"vertical",
            Object::boolean(cmdmod.cmod_split & WSP_VERT as c_int != 0),
        );
        dict_put(
            &mut mods,
            c"horizontal",
            Object::boolean(cmdmod.cmod_split & WSP_HOR as c_int != 0),
        );
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
        dict_put(&mut mods, c"split", Object::string(static_cstring(split)));
        mods
    }
}

pub unsafe extern "C" fn nvim_parse_cmd(
    str: String_0,
    _opts: *mut KeyDict_empty,
    arena: *mut Arena,
    err: *mut Error,
) -> KeyDict_cmd {
    unsafe {
        let mut result: KeyDict_cmd = ::core::mem::zeroed();
        let mut ea: exarg_T = ::core::mem::zeroed();
        let mut cmdinfo: CmdParseInfo = ::core::mem::zeroed();
        let mut cmdline: *mut c_char = arena_memdupz(arena, str.data, str.size);
        let mut errormsg: *const c_char = ptr::null::<c_char>();
        if !parse_cmdline(
            &raw mut cmdline,
            &raw mut ea,
            &raw mut cmdinfo,
            &raw mut errormsg,
        ) {
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
            return result;
        }

        let args = parse_args(&ea, arena);
        let cmd: *mut ucmd_T = if ea.cmdidx == CMD_USER {
            ((*ucmds.ptr()).ga_data as *mut ucmd_T).add(ea.useridx as usize)
        } else if ea.cmdidx == CMD_USER_BUF {
            ((*curbuf.get()).b_ucmds.ga_data as *mut ucmd_T).add(ea.useridx as usize)
        } else {
            ptr::null_mut::<ucmd_T>()
        };

        result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__cmd;
        result.cmd = cstr_as_string(command_name(&ea, cmd));

        if ea.argt & EX_RANGE as uint32_t != 0 && ea.addr_count > 0 {
            // Two addresses give both bounds, one gives only `line2`.
            let mut range: Array = arena_array(arena, 2);
            if ea.addr_count > 1 {
                array_add(&mut range, Object::integer(ea.line1 as Integer));
            }
            array_add(&mut range, Object::integer(ea.line2 as Integer));
            result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__range;
            result.range = range;
        }

        if ea.argt & EX_COUNT as uint32_t != 0 {
            let count: Integer = if ea.addr_count > 0 {
                ea.line2 as Integer
            } else if !cmd.is_null() {
                (*cmd).uc_def as Integer
            } else {
                0
            };
            // A zero count that nothing asked for is left unset.
            if ea.addr_count > 0 || (!cmd.is_null() && (*cmd).uc_def != 0) || count != 0 {
                result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__count;
                result.count = count;
            }
        }

        if ea.argt & EX_REGSTR as uint32_t != 0 {
            let mut reg: [c_char; 2] = [ea.regname as c_char, NUL as c_char];
            result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__reg;
            result.reg = arena_string(arena, cstr_as_string(reg.as_mut_ptr()));
        }

        result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__bang;
        result.bang = ea.forceit != 0;
        result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__args;
        result.args = args;

        // `:command -nargs=` spelling of how many arguments the command takes.
        let nargs: &CStr = if ea.argt & EX_EXTRA as uint32_t == 0 {
            c"0"
        } else if ea.argt & EX_NOSPC as uint32_t != 0 {
            if ea.argt & EX_NEEDARG as uint32_t != 0 {
                c"1"
            } else {
                c"?"
            }
        } else if ea.argt & EX_NEEDARG as uint32_t != 0 {
            c"+"
        } else {
            c"*"
        };
        result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__nargs;
        result.nargs = Object::string(arena_string(arena, static_cstring(nargs)));

        result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__addr;
        result.addr = static_cstring(addr_type_name(ea.addr_type));
        result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__nextcmd;
        result.nextcmd = cstr_as_string(ea.nextcmd);

        result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__mods;
        result.mods = parse_mods(&cmdinfo.cmdmod, arena);

        let mut magic: Dict = arena_dict(arena, 2);
        dict_put(&mut magic, c"file", Object::boolean(cmdinfo.magic.file));
        dict_put(&mut magic, c"bar", Object::boolean(cmdinfo.magic.bar));
        result.is_set__cmd_ |= 1 << KEYSET_OPTIDX_cmd__magic;
        result.magic = magic;

        undo_cmdmod(&raw mut cmdinfo.cmdmod);
        result
    }
}
