use crate::src::nvim::api::private::dispatch::{
    KeyDict_cmd_magic_get_field, KeyDict_cmd_mods_filter_get_field, KeyDict_cmd_mods_get_field,
};
use crate::src::nvim::api::private::helpers::{
    api_dict_to_keydict, api_set_error, api_set_sctx, api_typename, arena_array, arena_dict,
    arena_string, cstr_as_string, cstrn_as_string, find_buffer_by_handle, string_to_cstr,
    try_enter, try_leave,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid, api_err_required};
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::autocmd::{EVENT_CMDUNDEFINED, apply_autocmds, has_event};
use crate::src::nvim::charset::{skiptowhite, skipwhite};
use crate::src::nvim::ex_docmd::{
    excmd_get_argt, execute_cmd, find_ex_command, get_cmd_default_range, get_command_name,
    getargcmd, getargopt, invalid_range, is_cmd_ni, is_map_cmd, parse_cmdline, replace_makeprg,
    set_cmd_addr_type, set_cmd_count, set_cmd_dflall_range, undo_cmdmod,
};
use crate::src::nvim::ex_eval::aborting;

use crate::src::nvim::garray::{ga_clear, ga_init};
use crate::src::nvim::lua::executor::{api_free_luaref, api_new_luaref};
use crate::src::nvim::main::{capture_ga, curbuf, current_sctx, msg_col, msg_silent, redir_off};
use crate::src::nvim::mbyte::mb_islower;
use crate::src::nvim::memory::{arena_alloc, arena_memdupz, xcalloc, xfree, xrealloc};
use crate::src::nvim::os::libc::{memcpy, memmove, snprintf, strcmp, strlen, strncmp, strtol};
use crate::src::nvim::regexp::{RE_MAGIC, vim_regcomp};
use crate::src::nvim::register::valid_yank_reg;
use crate::src::nvim::strings::kv_do_printf;
use crate::src::nvim::types::{
    Arena, Array, Buffer, CMD_SIZE, CMD_USER, CMD_USER_BUF, CMD_append, CMD_iput, CMD_put,
    CMOD_BROWSE, CMOD_CONFIRM, CMOD_ERRSILENT, CMOD_HIDE, CMOD_KEEPALT, CMOD_KEEPJUMPS,
    CMOD_KEEPMARKS, CMOD_KEEPPATTERNS, CMOD_LOCKMARKS, CMOD_NOAUTOCMD, CMOD_NOSWAPFILE,
    CMOD_SANDBOX, CMOD_SILENT, CMOD_UNSILENT, CmdParseInfo,
    CmdParseInfo_magic as C2Rust_Unnamed_13, Dict, Direction, Error, Integer, KeyDict_cmd,
    KeyDict_cmd_magic, KeyDict_cmd_mods, KeyDict_cmd_mods_filter, KeyDict_cmd_opts, KeyDict_empty,
    KeyDict_get_commands, KeyDict_user_command, KeySetLink, KeyValuePair, LuaRef, Object,
    OptionalKeys, String_0, StringBuilder, TryState, buf_T, cmd_addr_T, cmdmod_T, cstack_T,
    exarg_T, except_T, expand_T, garray_T, int64_t, kErrorTypeException, kErrorTypeNone,
    kErrorTypeValidation, kObjectTypeBoolean, kObjectTypeDict, kObjectTypeInteger,
    kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString, key_value_pair, linenr_T, msglist_T,
    object, object_data as C2Rust_Unnamed, regmatch_T, regprog_T, sctx_T, size_t, ucmd_T, uint8_t,
    uint32_t, uint64_t,
};
use crate::src::nvim::usercmd::{
    commands_array, free_ucmd, get_user_command_name, parse_addr_type_arg, parse_compl_arg,
    uc_add_command, uc_nargs_upper_bound, uc_split_args_iter, uc_validate_name, ucmds,
};
use crate::src::nvim::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT};
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_LINES: cmd_addr_T = 0;
pub const kDirectionNotSet: Direction = 0;
pub const NUMBUFLEN: C2Rust_Unnamed_15 = 65;
pub const EXPAND_NOTHING: C2Rust_Unnamed_16 = 0;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_16 = 32;
pub const UC_BUFFER: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const EXPAND_SIGN: C2Rust_Unnamed_16 = 34;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_16 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_16 = 12;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_16 = 9;
pub const EXPAND_FILES: C2Rust_Unnamed_16 = 2;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__addr: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__count: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__force: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__nargs: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__range: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__preview: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__complete: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__cmd: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__reg: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__bang: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__addr: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__mods: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__args: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__count: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__magic: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__nargs: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__range: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__nextcmd: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_magic__bar: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_magic__file: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__tab: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__split: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__filter: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__verbose: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods_filter__pattern: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const EX_RANGE: ::core::ffi::c_uint = 0x1 as ::core::ffi::c_uint;
pub const EX_BANG: ::core::ffi::c_uint = 0x2 as ::core::ffi::c_uint;
pub const EX_EXTRA: ::core::ffi::c_uint = 0x4 as ::core::ffi::c_uint;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const EX_DFLALL: ::core::ffi::c_uint = 0x20 as ::core::ffi::c_uint;
pub const EX_NEEDARG: ::core::ffi::c_uint = 0x80 as ::core::ffi::c_uint;
pub const EX_TRLBAR: ::core::ffi::c_uint = 0x100 as ::core::ffi::c_uint;
pub const EX_REGSTR: ::core::ffi::c_uint = 0x200 as ::core::ffi::c_uint;
pub const EX_COUNT: ::core::ffi::c_uint = 0x400 as ::core::ffi::c_uint;
pub const EX_ZEROR: ::core::ffi::c_uint = 0x1000 as ::core::ffi::c_uint;
pub const EX_CMDARG: ::core::ffi::c_uint = 0x4000 as ::core::ffi::c_uint;
pub const EX_ARGOPT: ::core::ffi::c_uint = 0x20000 as ::core::ffi::c_uint;
pub const EX_SBOXOK: ::core::ffi::c_uint = 0x40000 as ::core::ffi::c_uint;
pub const EX_KEEPSCRIPT: ::core::ffi::c_uint = 0x4000000 as ::core::ffi::c_uint;
pub const EX_PREVIEW: ::core::ffi::c_uint = 0x8000000 as ::core::ffi::c_uint;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn parse_map_cmd(
    mut arg_str: *const ::core::ffi::c_char,
    mut arena: *mut Arena,
) -> Array {
    let mut args: Array = arena_array(arena, 2 as size_t);
    let mut lhs_start: *mut ::core::ffi::c_char = arg_str as *mut ::core::ffi::c_char;
    let mut lhs_end: *mut ::core::ffi::c_char = skiptowhite(lhs_start);
    let mut lhs_len: size_t = lhs_end.offset_from(lhs_start) as size_t;
    let c2rust_fresh28 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh28 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstrn_as_string(lhs_start, lhs_len),
        },
    };
    let mut rhs_start: *mut ::core::ffi::c_char = skipwhite(lhs_end);
    if *rhs_start as ::core::ffi::c_int != NUL {
        let mut rhs_len: size_t = strlen(rhs_start);
        let c2rust_fresh29 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh29 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstrn_as_string(rhs_start, rhs_len),
            },
        };
    }
    return args;
}
pub unsafe extern "C" fn nvim_parse_cmd(
    mut str: String_0,
    mut _opts: *mut KeyDict_empty,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> KeyDict_cmd {
    let mut args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut length: size_t = 0;
    let mut cmd: *mut ucmd_T = ::core::ptr::null_mut::<ucmd_T>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut nargs: [::core::ffi::c_char; 2] = [0; 2];
    let mut addr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mods: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut filter: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut split: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut magic: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut result: KeyDict_cmd = KeyDict_cmd {
        is_set__cmd_: 0 as OptionalKeys,
        cmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        range: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        count: 0,
        reg: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        bang: false,
        args: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        magic: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        mods: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        addr: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        nextcmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    let mut cmdinfo: CmdParseInfo = CmdParseInfo {
        cmdmod: cmdmod_T {
            cmod_flags: 0,
            cmod_split: 0,
            cmod_tab: 0,
            cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_filter_regmatch: regmatch_T {
                regprog: ::core::ptr::null_mut::<regprog_T>(),
                startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                rm_matchcol: 0,
                rm_ic: false,
            },
            cmod_filter_force: false,
            cmod_verbose: 0,
            cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_did_sandbox: 0,
            cmod_verbose_save: 0,
            cmod_save_msg_silent: 0,
            cmod_save_msg_scroll: 0,
            cmod_did_esilent: 0,
        },
        magic: C2Rust_Unnamed_13 {
            file: false,
            bar: false,
        },
    };
    let mut cmdline: *mut ::core::ffi::c_char = arena_memdupz(arena, str.data, str.size);
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !parse_cmdline(
        &raw mut cmdline,
        &raw mut ea,
        &raw mut cmdinfo,
        &raw mut errormsg,
    ) {
        if !errormsg.is_null() {
            api_set_error(
                err,
                kErrorTypeException,
                b"Parsing command-line: %s\0".as_ptr() as *const ::core::ffi::c_char,
                errormsg,
            );
        } else {
            api_set_error(
                err,
                kErrorTypeException,
                b"Parsing command-line\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    } else {
        args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        length = strlen(ea.arg);
        if ea.cmdidx as ::core::ffi::c_int != CMD_SIZE as ::core::ffi::c_int
            && is_map_cmd(ea.cmdidx) as ::core::ffi::c_int != 0
            && *ea.arg as ::core::ffi::c_int != NUL
        {
            args = parse_map_cmd(ea.arg, arena);
        } else if ea.argt & EX_NOSPC as uint32_t != 0 {
            if *ea.arg as ::core::ffi::c_int != NUL {
                args = arena_array(arena, 1 as size_t);
                let c2rust_fresh0 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh0 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstrn_as_string(ea.arg, length),
                    },
                };
            }
        } else {
            let mut end: size_t = 0 as size_t;
            let mut len: size_t = 0 as size_t;
            let mut buf: *mut ::core::ffi::c_char =
                arena_alloc(arena, length.wrapping_add(1 as size_t), false_0 != 0)
                    as *mut ::core::ffi::c_char;
            let mut done: bool = false_0 != 0;
            args = arena_array(arena, uc_nargs_upper_bound(ea.arg, length));
            while !done {
                done = uc_split_args_iter(ea.arg, length, &raw mut end, buf, &raw mut len);
                if len > 0 as size_t {
                    let c2rust_fresh1 = args.size;
                    args.size = args.size.wrapping_add(1);
                    *args.items.offset(c2rust_fresh1 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: cstrn_as_string(buf, len),
                        },
                    };
                    buf = buf.offset(len.wrapping_add(1 as size_t) as isize);
                }
            }
        }
        cmd = ::core::ptr::null_mut::<ucmd_T>();
        if ea.cmdidx as ::core::ffi::c_int == CMD_USER as ::core::ffi::c_int {
            cmd = ((*ucmds.ptr()).ga_data as *mut ucmd_T).offset(ea.useridx as isize);
        } else if ea.cmdidx as ::core::ffi::c_int == CMD_USER_BUF as ::core::ffi::c_int {
            cmd = ((*curbuf.get()).b_ucmds.ga_data as *mut ucmd_T).offset(ea.useridx as isize);
        }
        name = (if ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            (if !cmd.is_null() {
                (*cmd).uc_name
            } else {
                get_command_name(
                    ::core::ptr::null_mut::<expand_T>(),
                    ea.cmdidx as ::core::ffi::c_int,
                )
            }) as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__cmd)
            as OptionalKeys;
        result.cmd = cstr_as_string(name);
        if ea.argt & EX_RANGE as uint32_t != 0 && ea.addr_count > 0 as ::core::ffi::c_int {
            let mut range: Array = arena_array(arena, 2 as size_t);
            if ea.addr_count > 1 as ::core::ffi::c_int {
                let c2rust_fresh2 = range.size;
                range.size = range.size.wrapping_add(1);
                *range.items.offset(c2rust_fresh2 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: ea.line1 as Integer,
                    },
                };
            }
            let c2rust_fresh3 = range.size;
            range.size = range.size.wrapping_add(1);
            *range.items.offset(c2rust_fresh3 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: ea.line2 as Integer,
                },
            };
            result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range)
                as OptionalKeys;
            result.range = range;
        }
        if ea.argt & EX_COUNT as uint32_t != 0 {
            let mut count: Integer = if ea.addr_count > 0 as ::core::ffi::c_int {
                ea.line2 as Integer
            } else if !cmd.is_null() {
                (*cmd).uc_def as Integer
            } else {
                0 as Integer
            };
            if ea.addr_count > 0 as ::core::ffi::c_int
                || !cmd.is_null() && (*cmd).uc_def != 0 as int64_t
                || count != 0 as Integer
            {
                result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__count)
                    as OptionalKeys;
                result.count = count;
            }
        }
        if ea.argt & EX_REGSTR as uint32_t != 0 {
            let mut reg: [::core::ffi::c_char; 2] = [
                ea.regname as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
            ];
            result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__reg)
                as OptionalKeys;
            result.reg = arena_string(
                arena,
                cstr_as_string(&raw mut reg as *mut ::core::ffi::c_char),
            );
        }
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__bang)
            as OptionalKeys;
        result.bang = ea.forceit != 0;
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__args)
            as OptionalKeys;
        result.args = args;
        nargs = [0; 2];
        if ea.argt & EX_EXTRA as uint32_t != 0 {
            if ea.argt & EX_NOSPC as uint32_t != 0 {
                if ea.argt & EX_NEEDARG as uint32_t != 0 {
                    nargs[0 as ::core::ffi::c_int as usize] = '1' as ::core::ffi::c_char;
                } else {
                    nargs[0 as ::core::ffi::c_int as usize] = '?' as ::core::ffi::c_char;
                }
            } else if ea.argt & EX_NEEDARG as uint32_t != 0 {
                nargs[0 as ::core::ffi::c_int as usize] = '+' as ::core::ffi::c_char;
            } else {
                nargs[0 as ::core::ffi::c_int as usize] = '*' as ::core::ffi::c_char;
            }
        } else {
            nargs[0 as ::core::ffi::c_int as usize] = '0' as ::core::ffi::c_char;
        }
        nargs[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__nargs)
            as OptionalKeys;
        result.nargs = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: arena_string(
                    arena,
                    cstr_as_string(&raw mut nargs as *mut ::core::ffi::c_char),
                ),
            },
        };
        addr = ::core::ptr::null_mut::<::core::ffi::c_char>();
        match ea.addr_type as ::core::ffi::c_uint {
            0 => {
                addr = b"line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            2 => {
                addr = b"arg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            4 => {
                addr = b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            3 => {
                addr = b"load\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            1 => {
                addr = b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            5 => {
                addr = b"tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            8 => {
                addr = b"qf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            11 => {
                addr = b"none\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            _ => {
                addr = b"?\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
        }
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__addr)
            as OptionalKeys;
        result.addr = cstr_as_string(addr);
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__nextcmd)
            as OptionalKeys;
        result.nextcmd = cstr_as_string(ea.nextcmd);
        mods = arena_dict(arena, 20 as size_t);
        filter = arena_dict(arena, 2 as size_t);
        let c2rust_fresh4 = filter.size;
        filter.size = filter.size.wrapping_add(1);
        *filter.items.offset(c2rust_fresh4 as isize) = key_value_pair {
            key: cstr_as_string(b"pattern\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(arena, cstr_as_string(cmdinfo.cmdmod.cmod_filter_pat)),
                },
            },
        };
        let c2rust_fresh5 = filter.size;
        filter.size = filter.size.wrapping_add(1);
        *filter.items.offset(c2rust_fresh5 as isize) = key_value_pair {
            key: cstr_as_string(b"force\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_filter_force,
                },
            },
        };
        let c2rust_fresh6 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh6 as isize) = key_value_pair {
            key: cstr_as_string(b"filter\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: filter },
            },
        };
        let c2rust_fresh7 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh7 as isize) = key_value_pair {
            key: cstr_as_string(b"silent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_SILENT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh8 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh8 as isize) = key_value_pair {
            key: cstr_as_string(b"emsg_silent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh9 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"unsilent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_UNSILENT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh10 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"sandbox\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh11 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh11 as isize) = key_value_pair {
            key: cstr_as_string(b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_NOAUTOCMD as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh12 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh12 as isize) = key_value_pair {
            key: cstr_as_string(b"tab\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (cmdinfo.cmdmod.cmod_tab - 1 as ::core::ffi::c_int) as Integer,
                },
            },
        };
        let c2rust_fresh13 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh13 as isize) = key_value_pair {
            key: cstr_as_string(b"verbose\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (cmdinfo.cmdmod.cmod_verbose - 1 as ::core::ffi::c_int) as Integer,
                },
            },
        };
        let c2rust_fresh14 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh14 as isize) = key_value_pair {
            key: cstr_as_string(b"browse\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_BROWSE as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh15 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh15 as isize) = key_value_pair {
            key: cstr_as_string(b"confirm\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh16 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh16 as isize) = key_value_pair {
            key: cstr_as_string(b"hide\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_HIDE as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh17 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh17 as isize) = key_value_pair {
            key: cstr_as_string(b"keepalt\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh18 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh18 as isize) = key_value_pair {
            key: cstr_as_string(b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh19 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh19 as isize) = key_value_pair {
            key: cstr_as_string(b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPMARKS as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh20 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh20 as isize) = key_value_pair {
            key: cstr_as_string(b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        let c2rust_fresh21 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh22 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh22 as isize) = key_value_pair {
            key: cstr_as_string(b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh23 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"vertical\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_split & WSP_VERT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh24 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh24 as isize) = key_value_pair {
            key: cstr_as_string(b"horizontal\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_split & WSP_HOR as ::core::ffi::c_int != 0,
                },
            },
        };
        split = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if cmdinfo.cmdmod.cmod_split & WSP_BOT as ::core::ffi::c_int != 0 {
            split =
                b"botright\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if cmdinfo.cmdmod.cmod_split & WSP_TOP as ::core::ffi::c_int != 0 {
            split = b"topleft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if cmdinfo.cmdmod.cmod_split & WSP_BELOW as ::core::ffi::c_int != 0 {
            split =
                b"belowright\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if cmdinfo.cmdmod.cmod_split & WSP_ABOVE as ::core::ffi::c_int != 0 {
            split =
                b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            split = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        let c2rust_fresh25 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh25 as isize) = key_value_pair {
            key: cstr_as_string(b"split\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(split),
                },
            },
        };
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__mods)
            as OptionalKeys;
        result.mods = mods;
        magic = arena_dict(arena, 2 as size_t);
        let c2rust_fresh26 = magic.size;
        magic.size = magic.size.wrapping_add(1);
        *magic.items.offset(c2rust_fresh26 as isize) = key_value_pair {
            key: cstr_as_string(b"file\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.magic.file,
                },
            },
        };
        let c2rust_fresh27 = magic.size;
        magic.size = magic.size.wrapping_add(1);
        *magic.items.offset(c2rust_fresh27 as isize) = key_value_pair {
            key: cstr_as_string(b"bar\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.magic.bar,
                },
            },
        };
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__magic)
            as OptionalKeys;
        result.magic = magic;
        undo_cmdmod(&raw mut cmdinfo.cmdmod);
    }
    return result;
}
pub unsafe extern "C" fn nvim_cmd(
    mut channel_id: uint64_t,
    mut cmd: *mut KeyDict_cmd,
    mut opts: *mut KeyDict_cmd_opts,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut range_only: bool = false;
    let mut count_from_first_arg: bool = false;
    let mut capture_local: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut save_msg_silent: ::core::ffi::c_int = 0;
    let mut save_redir_off: bool = false;
    let mut save_capture_ga: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    let mut save_msg_col: ::core::ffi::c_int = 0;
    let mut ea: exarg_T = ::core::mem::zeroed();
    let mut cmdinfo: CmdParseInfo = ::core::mem::zeroed();
    let mut cmdline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cmdname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut retv: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    '_end: {
        if !((*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
        {
            api_err_required(err, b"cmd\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            if *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == NUL
            {
                if !((*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 10 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong
                    && (*cmd).range.size > 0 as size_t
                    || (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong)
                {
                    api_err_exp(
                        err,
                        b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                        b"non-empty String\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_end;
                }
            }
            cmdname = arena_string(arena, (*cmd).cmd).data;
            ea.cmd = cmdname;
            p = find_ex_command(&raw mut ea, ::core::ptr::null_mut::<::core::ffi::c_int>());
            if !p.is_null()
                && ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                && (*ea.cmd as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *ea.cmd as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint)
                && has_event(EVENT_CMDUNDEFINED) as ::core::ffi::c_int != 0
            {
                p = arena_string(arena, (*cmd).cmd).data;
                let mut ret: ::core::ffi::c_int = apply_autocmds(
                    EVENT_CMDUNDEFINED,
                    p,
                    p,
                    true_0 != 0,
                    ::core::ptr::null_mut::<buf_T>(),
                ) as ::core::ffi::c_int;
                p = if ret != 0 && !aborting() {
                    find_ex_command(&raw mut ea, ::core::ptr::null_mut::<::core::ffi::c_int>())
                } else {
                    ea.cmd
                };
            }
            range_only = ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                && *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == NUL
                && (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range
                    != 0 as ::core::ffi::c_ulonglong
                && (*cmd).range.size > 0 as size_t;
            if !(ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                && *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == NUL
                && (!((*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range
                    != 0 as ::core::ffi::c_ulonglong)
                    || (*cmd).range.size == 0 as size_t)
                && (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__mods
                    != 0 as ::core::ffi::c_ulonglong)
            {
                if !(!p.is_null()
                    && ea.cmdidx as ::core::ffi::c_int != CMD_SIZE as ::core::ffi::c_int
                    || range_only as ::core::ffi::c_int != 0)
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Command not found: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        cmdname,
                    );
                } else if !(range_only as ::core::ffi::c_int != 0 || !is_cmd_ni(ea.cmdidx)) {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Command not implemented: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        cmdname,
                    );
                } else {
                    if !range_only {
                        let mut fullname: *const ::core::ffi::c_char =
                            if (ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                                get_user_command_name(ea.useridx, ea.cmdidx as ::core::ffi::c_int)
                            } else {
                                get_command_name(
                                    ::core::ptr::null_mut::<expand_T>(),
                                    ea.cmdidx as ::core::ffi::c_int,
                                )
                            };
                        if !(strncmp(fullname, cmdname, strlen(cmdname)) == 0 as ::core::ffi::c_int)
                        {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Invalid command: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                                cmdname,
                            );
                            break '_end;
                        }
                    }
                    if range_only {
                        ea.argt = (EX_RANGE | EX_SBOXOK) as uint32_t;
                    } else if !((ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int) {
                        ea.argt = excmd_get_argt(ea.cmdidx);
                    }
                    count_from_first_arg = false_0 != 0;
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__args
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if (*cmd).args.size == 1 as size_t
                            && ea.argt & EX_COUNT as uint32_t != 0
                            && ea.argt & EX_EXTRA as uint32_t == 0
                        {
                            let mut first_arg: Object =
                                *(*cmd).args.items.offset(0 as ::core::ffi::c_int as isize);
                            let mut is_numeric: bool = false_0 != 0;
                            let mut count_value: int64_t = 0 as int64_t;
                            if first_arg.type_0 as ::core::ffi::c_uint
                                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                is_numeric = true_0 != 0;
                                count_value = first_arg.data.integer as int64_t;
                            } else if first_arg.type_0 as ::core::ffi::c_uint
                                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                let mut endptr: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                let mut val: ::core::ffi::c_long = strtol(
                                    first_arg.data.string.data,
                                    &raw mut endptr,
                                    10 as ::core::ffi::c_int,
                                );
                                if *endptr as ::core::ffi::c_int == '\0' as ::core::ffi::c_int
                                    && first_arg.data.string.size > 0 as size_t
                                {
                                    is_numeric = true_0 != 0;
                                    count_value = val as int64_t;
                                }
                            }
                            if is_numeric as ::core::ffi::c_int != 0 && count_value >= 0 as int64_t
                            {
                                count_from_first_arg = true_0 != 0;
                                ea.addr_count = 1 as ::core::ffi::c_int;
                                ea.line2 = count_value as linenr_T;
                                ea.line1 = ea.line2;
                                args = arena_array(arena, 0 as size_t);
                            }
                        }
                        if !count_from_first_arg {
                            args = arena_array(arena, (*cmd).args.size);
                            let mut i: size_t = 0 as size_t;
                            while i < (*cmd).args.size {
                                let mut elem: Object = *(*cmd).args.items.offset(i as isize);
                                let mut data_str: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                match elem.type_0 as ::core::ffi::c_uint {
                                    1 => {
                                        data_str = arena_alloc(arena, 2 as size_t, false_0 != 0)
                                            as *mut ::core::ffi::c_char;
                                        *data_str.offset(0 as ::core::ffi::c_int as isize) =
                                            (if elem.data.boolean as ::core::ffi::c_int != 0 {
                                                '1' as ::core::ffi::c_int
                                            } else {
                                                '0' as ::core::ffi::c_int
                                            })
                                                as ::core::ffi::c_char;
                                        *data_str.offset(1 as ::core::ffi::c_int as isize) =
                                            NUL as ::core::ffi::c_char;
                                        let c2rust_fresh30 = args.size;
                                        args.size = args.size.wrapping_add(1);
                                        *args.items.offset(c2rust_fresh30 as isize) = object {
                                            type_0: kObjectTypeString,
                                            data: C2Rust_Unnamed {
                                                string: cstr_as_string(data_str),
                                            },
                                        };
                                    }
                                    8 | 9 | 10 | 2 => {
                                        data_str = arena_alloc(
                                            arena,
                                            NUMBUFLEN as ::core::ffi::c_int as size_t,
                                            false_0 != 0,
                                        )
                                            as *mut ::core::ffi::c_char;
                                        snprintf(
                                            data_str,
                                            NUMBUFLEN as ::core::ffi::c_int as size_t,
                                            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                                            elem.data.integer,
                                        );
                                        let c2rust_fresh31 = args.size;
                                        args.size = args.size.wrapping_add(1);
                                        *args.items.offset(c2rust_fresh31 as isize) = object {
                                            type_0: kObjectTypeString,
                                            data: C2Rust_Unnamed {
                                                string: cstr_as_string(data_str),
                                            },
                                        };
                                    }
                                    4 => {
                                        if string_iswhite(elem.data.string) {
                                            api_err_exp(
                                                err,
                                                b"command arg\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"non-whitespace\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                            break '_end;
                                        } else {
                                            let c2rust_fresh32 = args.size;
                                            args.size = args.size.wrapping_add(1);
                                            *args.items.offset(c2rust_fresh32 as isize) = elem;
                                        }
                                    }
                                    _ => {
                                        if true {
                                            api_err_exp(
                                                err,
                                                b"command arg\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"valid type\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                api_typename(elem.type_0),
                                            );
                                            break '_end;
                                        }
                                    }
                                }
                                i = i.wrapping_add(1);
                            }
                            let mut argc_valid: bool = false;
                            match ea.argt
                                & (EX_EXTRA as uint32_t
                                    | EX_NOSPC as uint32_t
                                    | EX_NEEDARG as uint32_t)
                            {
                                148 => {
                                    argc_valid = args.size == 1 as size_t;
                                }
                                20 => {
                                    argc_valid = args.size <= 1 as size_t;
                                }
                                132 => {
                                    argc_valid = args.size >= 1 as size_t;
                                }
                                EX_EXTRA => {
                                    argc_valid = true_0 != 0;
                                }
                                _ => {
                                    argc_valid = args.size == 0 as size_t;
                                }
                            }
                            if !argc_valid {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Wrong number of arguments\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                                break '_end;
                            }
                        }
                    }
                    if !range_only {
                        set_cmd_addr_type(
                            &raw mut ea,
                            if args.size > 0 as size_t {
                                (*args.items.offset(0 as ::core::ffi::c_int as isize))
                                    .data
                                    .string
                                    .data
                            } else {
                                ::core::ptr::null_mut::<::core::ffi::c_char>()
                            },
                        );
                    }
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if ea.argt & 0x1 as uint32_t == 0 {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Command cannot accept %s: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).cmd.data,
                            );
                            break '_end;
                        } else if !((*cmd).range.size <= 2 as size_t) {
                            api_err_exp(
                                err,
                                b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                b"<=2 elements\0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                            break '_end;
                        } else {
                            let mut range: Array = (*cmd).range;
                            ea.addr_count = range.size as ::core::ffi::c_int;
                            let mut i_0: size_t = 0 as size_t;
                            while i_0 < range.size {
                                let mut elem_0: Object = *range.items.offset(i_0 as isize);
                                if !(elem_0.type_0 as ::core::ffi::c_uint
                                    == kObjectTypeInteger as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                    && elem_0.data.integer >= 0 as Integer)
                                {
                                    api_err_exp(
                                        err,
                                        b"range element\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"non-negative Integer\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                    );
                                    break '_end;
                                } else {
                                    i_0 = i_0.wrapping_add(1);
                                }
                            }
                            if range.size > 0 as size_t {
                                ea.line1 = (*range.items.offset(0 as ::core::ffi::c_int as isize))
                                    .data
                                    .integer as linenr_T;
                                ea.line2 = (*range
                                    .items
                                    .offset(range.size.wrapping_sub(1 as size_t) as isize))
                                .data
                                .integer as linenr_T;
                            }
                            if !invalid_range(&raw mut ea).is_null() {
                                api_err_invalid(
                                    err,
                                    b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_end;
                            }
                        }
                    }
                    if ea.addr_count == 0 as ::core::ffi::c_int {
                        if ea.argt & EX_DFLALL as uint32_t != 0 {
                            set_cmd_dflall_range(&raw mut ea);
                        } else {
                            ea.line2 = get_cmd_default_range(&raw mut ea);
                            ea.line1 = ea.line2;
                            if ea.addr_type as ::core::ffi::c_uint
                                == ADDR_OTHER as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                ea.line2 = 1 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                    }
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__count
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if count_from_first_arg {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                b"Cannot specify both 'count' and numeric argument\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_end;
                        } else if ea.argt & 0x400 as uint32_t == 0 {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Command cannot accept %s: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"count\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).cmd.data,
                            );
                            break '_end;
                        } else if !((*cmd).count >= 0 as Integer) {
                            api_err_exp(
                                err,
                                b"count\0".as_ptr() as *const ::core::ffi::c_char,
                                b"non-negative Integer\0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                            break '_end;
                        } else {
                            set_cmd_count(&raw mut ea, (*cmd).count as linenr_T, true_0 != 0);
                        }
                    }
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__reg
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if ea.argt & 0x200 as uint32_t == 0 {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Command cannot accept %s: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"register\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).cmd.data,
                            );
                            break '_end;
                        } else if !((*cmd).reg.size == 1 as size_t) {
                            api_err_exp(
                                err,
                                b"reg\0".as_ptr() as *const ::core::ffi::c_char,
                                b"single character\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).reg.data,
                            );
                            break '_end;
                        } else {
                            let mut regname: ::core::ffi::c_char =
                                *(*cmd).reg.data.offset(0 as ::core::ffi::c_int as isize);
                            if !(regname as ::core::ffi::c_int != '=' as ::core::ffi::c_int) {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Cannot use register \"=\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                                break '_end;
                            } else if !valid_yank_reg(
                                regname as ::core::ffi::c_int,
                                !((ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                                    && ea.cmdidx as ::core::ffi::c_int
                                        != CMD_put as ::core::ffi::c_int
                                    && ea.cmdidx as ::core::ffi::c_int
                                        != CMD_iput as ::core::ffi::c_int,
                            ) {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"Invalid register: \"%c\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    regname as ::core::ffi::c_int,
                                );
                                break '_end;
                            } else {
                                ea.regname = regname as uint8_t as ::core::ffi::c_int;
                            }
                        }
                    }
                    ea.forceit = (*cmd).bang as ::core::ffi::c_int;
                    if !(ea.forceit == 0 || ea.argt & 0x2 as uint32_t != 0) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Command cannot accept %s: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"bang\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).cmd.data,
                        );
                    } else {
                        if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__magic
                            != 0 as ::core::ffi::c_ulonglong
                        {
                            let mut magic: [KeyDict_cmd_magic; 1] = [KeyDict_cmd_magic {
                                is_set__cmd_magic_: 0 as OptionalKeys,
                                file: false,
                                bar: false,
                            }];
                            if !api_dict_to_keydict(
                                &raw mut magic as *mut KeyDict_cmd_magic
                                    as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict_cmd_magic_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        )
                                            -> *mut KeySetLink,
                                ),
                                (*cmd).magic,
                                err,
                            ) {
                                break '_end;
                            } else {
                                cmdinfo.magic.file = if (*(&raw mut magic
                                    as *mut KeyDict_cmd_magic))
                                    .is_set__cmd_magic_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_magic__file
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*(&raw mut magic as *mut KeyDict_cmd_magic)).file as uint32_t
                                } else {
                                    ea.argt & EX_XFILE as uint32_t
                                } != 0;
                                cmdinfo.magic.bar = if (*(&raw mut magic as *mut KeyDict_cmd_magic))
                                    .is_set__cmd_magic_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_magic__bar
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*(&raw mut magic as *mut KeyDict_cmd_magic)).bar as uint32_t
                                } else {
                                    ea.argt & EX_TRLBAR as uint32_t
                                } != 0;
                                if cmdinfo.magic.file {
                                    ea.argt =
                                        (ea.argt as ::core::ffi::c_uint | EX_XFILE) as uint32_t;
                                } else {
                                    ea.argt =
                                        (ea.argt as ::core::ffi::c_uint & !EX_XFILE) as uint32_t;
                                }
                            }
                        } else {
                            cmdinfo.magic.file = ea.argt & EX_XFILE as uint32_t != 0;
                            cmdinfo.magic.bar = ea.argt & EX_TRLBAR as uint32_t != 0;
                        }
                        if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__mods
                            != 0 as ::core::ffi::c_ulonglong
                        {
                            let mut mods: [KeyDict_cmd_mods; 1] = [KeyDict_cmd_mods {
                                is_set__cmd_mods_: 0 as OptionalKeys,
                                silent: false,
                                emsg_silent: false,
                                unsilent: false,
                                filter: Dict {
                                    size: 0,
                                    capacity: 0,
                                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                                },
                                sandbox: false,
                                noautocmd: false,
                                browse: false,
                                confirm: false,
                                hide: false,
                                horizontal: false,
                                keepalt: false,
                                keepjumps: false,
                                keepmarks: false,
                                keeppatterns: false,
                                lockmarks: false,
                                noswapfile: false,
                                tab: 0,
                                verbose: 0,
                                vertical: false,
                                split: String_0 {
                                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    size: 0,
                                },
                            }];
                            if !api_dict_to_keydict(
                                &raw mut mods as *mut KeyDict_cmd_mods as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict_cmd_mods_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        )
                                            -> *mut KeySetLink,
                                ),
                                (*cmd).mods,
                                err,
                            ) {
                                break '_end;
                            } else {
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).is_set__cmd_mods_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_mods__filter
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    let mut filter: [KeyDict_cmd_mods_filter; 1] =
                                        [KeyDict_cmd_mods_filter {
                                            is_set__cmd_mods_filter_: 0 as OptionalKeys,
                                            pattern: String_0 {
                                                data: ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                ),
                                                size: 0,
                                            },
                                            force: false,
                                        }];
                                    if !api_dict_to_keydict(
                                        &raw mut filter as *mut ::core::ffi::c_void,
                                        Some(
                                            KeyDict_cmd_mods_filter_get_field
                                                as unsafe extern "C" fn(
                                                    *const ::core::ffi::c_char,
                                                    size_t,
                                                )
                                                    -> *mut KeySetLink,
                                        ),
                                        (*(&raw mut mods as *mut KeyDict_cmd_mods)).filter,
                                        err,
                                    ) {
                                        break '_end;
                                    } else if (*(&raw mut filter as *mut KeyDict_cmd_mods_filter))
                                        .is_set__cmd_mods_filter_
                                        as ::core::ffi::c_ulonglong
                                        & (1 as ::core::ffi::c_ulonglong)
                                            << KEYSET_OPTIDX_cmd_mods_filter__pattern
                                        != 0 as ::core::ffi::c_ulonglong
                                    {
                                        cmdinfo.cmdmod.cmod_filter_force = (*(&raw mut filter
                                            as *mut KeyDict_cmd_mods_filter))
                                            .force
                                            as bool;
                                        if *(*(&raw mut filter as *mut KeyDict_cmd_mods_filter))
                                            .pattern
                                            .data
                                            as ::core::ffi::c_int
                                            != NUL
                                            || cmdinfo.cmdmod.cmod_filter_force
                                                as ::core::ffi::c_int
                                                != 0
                                        {
                                            cmdinfo.cmdmod.cmod_filter_pat = string_to_cstr(
                                                (*(&raw mut filter
                                                    as *mut KeyDict_cmd_mods_filter))
                                                    .pattern,
                                            );
                                            cmdinfo.cmdmod.cmod_filter_regmatch.regprog =
                                                vim_regcomp(
                                                    cmdinfo.cmdmod.cmod_filter_pat,
                                                    RE_MAGIC,
                                                );
                                        }
                                    }
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).is_set__cmd_mods_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd_mods__tab
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if (*(&raw mut mods as *mut KeyDict_cmd_mods)).tab
                                        as ::core::ffi::c_int
                                        >= 0 as ::core::ffi::c_int
                                    {
                                        cmdinfo.cmdmod.cmod_tab =
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).tab
                                                as ::core::ffi::c_int
                                                + 1 as ::core::ffi::c_int;
                                    }
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).is_set__cmd_mods_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_mods__verbose
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if (*(&raw mut mods as *mut KeyDict_cmd_mods)).verbose
                                        as ::core::ffi::c_int
                                        >= 0 as ::core::ffi::c_int
                                    {
                                        cmdinfo.cmdmod.cmod_verbose =
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).verbose
                                                as ::core::ffi::c_int
                                                + 1 as ::core::ffi::c_int;
                                    }
                                }
                                cmdinfo.cmdmod.cmod_split |=
                                    if (*(&raw mut mods as *mut KeyDict_cmd_mods)).vertical
                                        as ::core::ffi::c_int
                                        != 0
                                    {
                                        WSP_VERT as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    };
                                cmdinfo.cmdmod.cmod_split |=
                                    if (*(&raw mut mods as *mut KeyDict_cmd_mods)).horizontal
                                        as ::core::ffi::c_int
                                        != 0
                                    {
                                        WSP_HOR as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    };
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).is_set__cmd_mods_
                                    as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_mods__split
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if *(*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data
                                        as ::core::ffi::c_int
                                        != NUL
                                    {
                                        if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                            || strcmp(
                                                (*(&raw mut mods as *mut KeyDict_cmd_mods))
                                                    .split
                                                    .data,
                                                b"leftabove\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_ABOVE as ::core::ffi::c_int;
                                        } else if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"belowright\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                            || strcmp(
                                                (*(&raw mut mods as *mut KeyDict_cmd_mods))
                                                    .split
                                                    .data,
                                                b"rightbelow\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_BELOW as ::core::ffi::c_int;
                                        } else if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"topleft\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_TOP as ::core::ffi::c_int;
                                        } else if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"botright\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_BOT as ::core::ffi::c_int;
                                        } else if true {
                                            api_err_invalid(
                                                err,
                                                b"mods.split\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"\0".as_ptr() as *const ::core::ffi::c_char,
                                                0 as int64_t,
                                                true_0 != 0,
                                            );
                                            break '_end;
                                        }
                                    }
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).silent {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_SILENT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).emsg_silent {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_ERRSILENT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).unsilent {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_UNSILENT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).sandbox {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_SANDBOX as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).noautocmd {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_NOAUTOCMD as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).browse {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_BROWSE as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).confirm {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_CONFIRM as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).hide {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_HIDE as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keepalt {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_KEEPALT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keepjumps {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_KEEPJUMPS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keepmarks {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_KEEPMARKS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keeppatterns {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_KEEPPATTERNS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).lockmarks {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_LOCKMARKS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).noswapfile {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_NOSWAPFILE as ::core::ffi::c_int;
                                }
                                if cmdinfo.cmdmod.cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int
                                    != 0
                                {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_SILENT as ::core::ffi::c_int;
                                }
                                if cmdinfo.cmdmod.cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int
                                    != 0
                                    && ea.argt & 0x40000 as uint32_t == 0
                                {
                                    api_set_error(
                                        err,
                                        kErrorTypeValidation,
                                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"Command cannot be run in sandbox\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                    break '_end;
                                }
                            }
                        }
                        build_cmdline_str(&raw mut cmdline, &raw mut ea, &raw mut cmdinfo, args);
                        ea.cmdlinep = &raw mut cmdline;
                        's_1442: {
                            if ea.argt & EX_ARGOPT as uint32_t != 0 {
                                loop {
                                    if !(*ea.arg.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '+' as ::core::ffi::c_int
                                        && *ea.arg.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '+' as ::core::ffi::c_int)
                                    {
                                        break 's_1442;
                                    }
                                    let mut orig_arg: *mut ::core::ffi::c_char = ea.arg;
                                    let mut result: ::core::ffi::c_int = getargopt(&raw mut ea);
                                    if result != 0 as ::core::ffi::c_int
                                        || is_cmd_ni(ea.cmdidx) as ::core::ffi::c_int != 0
                                    {
                                        continue;
                                    }
                                    api_err_invalid(
                                        err,
                                        b"argument \0".as_ptr() as *const ::core::ffi::c_char,
                                        orig_arg,
                                        0 as int64_t,
                                        true_0 != 0,
                                    );
                                    break '_end;
                                }
                            }
                        }
                        if ea.argt & EX_CMDARG as uint32_t != 0 && ea.usefilter == 0 {
                            ea.do_ecmd_cmd = getargcmd(&raw mut ea.arg);
                        }
                        capture_local = garray_T {
                            ga_len: 0,
                            ga_maxlen: 0,
                            ga_itemsize: 0,
                            ga_growsize: 0,
                            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        };
                        save_msg_silent = msg_silent.get();
                        save_redir_off = redir_off.get();
                        save_capture_ga = capture_ga.get();
                        save_msg_col = msg_col.get();
                        if (*opts).output {
                            ga_init(
                                &raw mut capture_local,
                                1 as ::core::ffi::c_int,
                                80 as ::core::ffi::c_int,
                            );
                            capture_ga.set(&raw mut capture_local);
                        }
                        let mut tstate: TryState = TryState {
                            current_exception: ::core::ptr::null_mut::<except_T>(),
                            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                            msg_list: ::core::ptr::null::<*const msglist_T>(),
                            got_int: 0,
                            did_throw: false,
                            need_rethrow: 0,
                            did_emsg: 0,
                        };
                        try_enter(&raw mut tstate);
                        if (*opts).output {
                            (*msg_silent.ptr()) += 1;
                            redir_off.set(false);
                            msg_col.set(0 as ::core::ffi::c_int);
                        }
                        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
                        execute_cmd(&raw mut ea, &raw mut cmdinfo, false);
                        current_sctx.set(save_current_sctx);
                        if (*opts).output {
                            capture_ga.set(save_capture_ga);
                            msg_silent.set(save_msg_silent);
                            redir_off.set(save_redir_off);
                            msg_col.set(save_msg_col);
                        }
                        try_leave(&raw mut tstate, err);
                        if (*err).type_0 as ::core::ffi::c_int
                            == kErrorTypeNone as ::core::ffi::c_int
                        {
                            if (*opts).output as ::core::ffi::c_int != 0
                                && capture_local.ga_len > 1 as ::core::ffi::c_int
                            {
                                retv = arena_string(
                                    arena,
                                    String_0 {
                                        data: capture_local.ga_data as *mut ::core::ffi::c_char,
                                        size: capture_local.ga_len as size_t,
                                    },
                                );
                                if *retv.data.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '\n' as ::core::ffi::c_int
                                {
                                    retv.data = retv.data.offset(1);
                                    retv.size = retv.size.wrapping_sub(1);
                                }
                            }
                        }
                        if (*opts).output {
                            ga_clear(&raw mut capture_local);
                        }
                    }
                }
            }
        }
    }
    xfree(cmdline as *mut ::core::ffi::c_void);
    xfree(ea.args as *mut ::core::ffi::c_void);
    xfree(ea.arglens as *mut ::core::ffi::c_void);
    return retv;
}
unsafe extern "C" fn string_iswhite(mut str: String_0) -> bool {
    let mut i: size_t = 0 as size_t;
    while i < str.size {
        if !ascii_iswhite(*str.data.offset(i as isize) as ::core::ffi::c_int) {
            return false_0 != 0;
        } else {
            if *str.data.offset(i as isize) as ::core::ffi::c_int == NUL {
                break;
            }
            i = i.wrapping_add(1);
        }
    }
    return true_0 != 0;
}
/// Append `len` bytes to a [`StringBuilder`], growing it to the next power
/// of two when they do not fit: upstream's `kv_concat_len(cmdline, src,
/// len)`.  c2rust expanded that macro at all twenty-four of
/// [`build_cmdline_str`]'s call sites, ~40 lines apiece.
///
/// # Safety
/// `cmdline` points at a live builder and `src` at `len` readable bytes.
unsafe fn cmdline_concat(
    cmdline: *mut StringBuilder,
    src: *const ::core::ffi::c_char,
    len: size_t,
) {
    if len == 0 as size_t {
        return;
    }
    if (*cmdline).capacity < (*cmdline).size.wrapping_add(len) {
        let mut capacity: size_t = (*cmdline).size.wrapping_add(len);
        capacity = capacity.wrapping_sub(1);
        capacity |= capacity >> 1 as ::core::ffi::c_int;
        capacity |= capacity >> 2 as ::core::ffi::c_int;
        capacity |= capacity >> 4 as ::core::ffi::c_int;
        capacity |= capacity >> 8 as ::core::ffi::c_int;
        capacity |= capacity >> 16 as ::core::ffi::c_int;
        (*cmdline).capacity = capacity.wrapping_add(1);
        (*cmdline).items = xrealloc(
            (*cmdline).items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*cmdline).capacity),
        ) as *mut ::core::ffi::c_char;
    }
    debug_assert!(!(*cmdline).items.is_null());
    memcpy(
        (*cmdline).items.offset((*cmdline).size as isize) as *mut ::core::ffi::c_void,
        src as *const ::core::ffi::c_void,
        ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(len),
    );
    (*cmdline).size = (*cmdline).size.wrapping_add(len);
}

/// [`cmdline_concat`] for a string literal: upstream's `kv_concat`.
///
/// # Safety
/// `cmdline` points at a live builder.
unsafe fn cmdline_concat_str(cmdline: *mut StringBuilder, s: &::core::ffi::CStr) {
    cmdline_concat(cmdline, s.as_ptr(), s.count_bytes())
}

unsafe extern "C" fn build_cmdline_str(
    mut cmdlinep: *mut *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut cmdinfo: *mut CmdParseInfo,
    mut args: Array,
) {
    let mut argc: size_t = args.size;
    let mut cmdline: StringBuilder = StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    cmdline.capacity = 32 as size_t;
    cmdline.items = xrealloc(
        cmdline.items as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
    ) as *mut ::core::ffi::c_char;
    if (*cmdinfo).cmdmod.cmod_tab != 0 as ::core::ffi::c_int {
        kv_do_printf(
            &raw mut cmdline,
            b"%dtab \0".as_ptr() as *const ::core::ffi::c_char,
            (*cmdinfo).cmdmod.cmod_tab - 1 as ::core::ffi::c_int,
        );
    }
    if (*cmdinfo).cmdmod.cmod_verbose > 0 as ::core::ffi::c_int {
        kv_do_printf(
            &raw mut cmdline,
            b"%dverbose \0".as_ptr() as *const ::core::ffi::c_char,
            (*cmdinfo).cmdmod.cmod_verbose - 1 as ::core::ffi::c_int,
        );
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"silent! ");
    } else if (*cmdinfo).cmdmod.cmod_flags & CMOD_SILENT as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"silent ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_UNSILENT as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"unsilent ");
    }
    match (*cmdinfo).cmdmod.cmod_split
        & (WSP_ABOVE as ::core::ffi::c_int
            | WSP_BELOW as ::core::ffi::c_int
            | WSP_TOP as ::core::ffi::c_int
            | WSP_BOT as ::core::ffi::c_int)
    {
        128 => {
            cmdline_concat_str(&raw mut cmdline, c"aboveleft ");
        }
        64 => {
            cmdline_concat_str(&raw mut cmdline, c"belowright ");
        }
        8 => {
            cmdline_concat_str(&raw mut cmdline, c"topleft ");
        }
        16 => {
            cmdline_concat_str(&raw mut cmdline, c"botright ");
        }
        _ => {}
    }
    if (*cmdinfo).cmdmod.cmod_split & WSP_VERT as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"vertical ");
    }
    if (*cmdinfo).cmdmod.cmod_split & WSP_HOR as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"horizontal ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"sandbox ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_NOAUTOCMD as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"noautocmd ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_BROWSE as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"browse ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"confirm ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_HIDE as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"hide ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"keepalt ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"keepjumps ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPMARKS as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"keepmarks ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"keeppatterns ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"lockmarks ");
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0 {
        cmdline_concat_str(&raw mut cmdline, c"noswapfile ");
    }
    if (*eap).argt & EX_RANGE as uint32_t != 0 {
        if (*eap).addr_count == 1 as ::core::ffi::c_int {
            kv_do_printf(
                &raw mut cmdline,
                b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*eap).line2,
            );
        } else if (*eap).addr_count > 1 as ::core::ffi::c_int {
            kv_do_printf(
                &raw mut cmdline,
                b"%d,%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*eap).line1,
                (*eap).line2,
            );
            (*eap).addr_count = 2 as ::core::ffi::c_int;
        }
    }
    let mut cmdname_idx: size_t = cmdline.size;
    cmdline_concat(&raw mut cmdline, (*eap).cmd, strlen((*eap).cmd));
    if (*eap).argt & EX_BANG as uint32_t != 0 && (*eap).forceit != 0 {
        cmdline_concat_str(&raw mut cmdline, c"!");
    }
    if (*eap).argt & EX_REGSTR as uint32_t != 0 && (*eap).regname != 0 {
        kv_do_printf(
            &raw mut cmdline,
            b" %c\0".as_ptr() as *const ::core::ffi::c_char,
            (*eap).regname,
        );
    }
    (*eap).argc = argc;
    (*eap).arglens = (if (*eap).argc > 0 as size_t {
        xcalloc(argc, ::core::mem::size_of::<size_t>())
    } else {
        NULL
    }) as *mut size_t;
    let mut argstart_idx: size_t = cmdline.size;
    let mut i: size_t = 0 as size_t;
    while i < argc {
        let mut s: String_0 = (*args.items.offset(i as isize)).data.string;
        *(*eap).arglens.offset(i as isize) = s.size;
        cmdline_concat_str(&raw mut cmdline, c" ");
        cmdline_concat(&raw mut cmdline, s.data, s.size);
        i = i.wrapping_add(1);
    }
    if cmdline.size == cmdline.capacity {
        cmdline.capacity = if cmdline.capacity != 0 {
            cmdline.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        cmdline.items = xrealloc(
            cmdline.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
        ) as *mut ::core::ffi::c_char;
    } else {
    };
    let c2rust_fresh33 = cmdline.size;
    cmdline.size = cmdline.size.wrapping_add(1);
    *cmdline.items.offset(c2rust_fresh33 as isize) = '\0' as ::core::ffi::c_char;
    (*eap).cmd = cmdline.items.offset(cmdname_idx as isize);
    (*eap).args = (if (*eap).argc > 0 as size_t {
        xcalloc(argc, ::core::mem::size_of::<*mut ::core::ffi::c_char>())
    } else {
        NULL
    }) as *mut *mut ::core::ffi::c_char;
    let mut offset: size_t = argstart_idx;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < argc {
        offset = offset.wrapping_add(1);
        *(*eap).args.offset(i_0 as isize) = cmdline.items.offset(offset as isize);
        offset = offset.wrapping_add(*(*eap).arglens.offset(i_0 as isize));
        i_0 = i_0.wrapping_add(1);
    }
    (*eap).arg = if argc > 0 as size_t {
        *(*eap).args.offset(0 as ::core::ffi::c_int as isize)
    } else {
        cmdline
            .items
            .offset(cmdline.size as isize)
            .offset(-(1 as ::core::ffi::c_int as isize))
    };
    *cmdlinep = cmdline.items;
    let mut p: *mut ::core::ffi::c_char = replace_makeprg(eap, (*eap).arg, cmdlinep);
    if p != (*eap).arg {
        (*eap).arg = p;
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*eap).args as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*eap).arglens as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
        (*eap).argc = 0 as size_t;
    }
}
pub unsafe extern "C" fn nvim_create_user_command(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut err: *mut Error,
) {
    create_user_command(channel_id, name, cmd, opts, 0 as ::core::ffi::c_int, err);
}
pub unsafe extern "C" fn nvim_del_user_command(mut name: String_0, mut err: *mut Error) {
    nvim_buf_del_user_command(-1 as Buffer, name, err);
}
pub unsafe extern "C" fn nvim_buf_create_user_command(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut err: *mut Error,
) {
    let mut target_buf: *mut buf_T = find_buffer_by_handle(buf, err);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    let mut save_curbuf: *mut buf_T = curbuf.get();
    curbuf.set(target_buf);
    create_user_command(
        channel_id,
        name,
        cmd,
        opts,
        UC_BUFFER as ::core::ffi::c_int,
        err,
    );
    curbuf.set(save_curbuf);
}
pub unsafe extern "C" fn nvim_buf_del_user_command(
    mut buf: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) {
    let mut gap: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    if buf == -1 as ::core::ffi::c_int {
        gap = ucmds.ptr();
    } else {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
        gap = &raw mut (*b).b_ucmds;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut cmd: *mut ucmd_T = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
        if strcmp(name.data, (*cmd).uc_name) == 0 {
            free_ucmd(cmd);
            (*gap).ga_len -= 1 as ::core::ffi::c_int;
            if i < (*gap).ga_len {
                memmove(
                    cmd as *mut ::core::ffi::c_void,
                    cmd.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    (((*gap).ga_len - i) as size_t).wrapping_mul(::core::mem::size_of::<ucmd_T>()),
                );
            }
            return;
        }
        i += 1;
    }
    api_set_error(
        err,
        kErrorTypeException,
        b"Invalid command (not found): %s\0".as_ptr() as *const ::core::ffi::c_char,
        name.data,
    );
}
pub unsafe extern "C" fn create_user_command(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut flags: ::core::ffi::c_int,
    mut err: *mut Error,
) {
    let mut force: bool = false;
    let mut argt: uint32_t = 0 as uint32_t;
    let mut def: int64_t = -1 as int64_t;
    let mut addr_type_arg: cmd_addr_T = ADDR_NONE;
    let mut context: ::core::ffi::c_int = EXPAND_NOTHING as ::core::ffi::c_int;
    let mut compl_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut rep: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut luaref: LuaRef = LUA_NOREF;
    let mut compl_luaref: LuaRef = LUA_NOREF;
    let mut preview_luaref: LuaRef = LUA_NOREF;
    '_err: {
        if uc_validate_name(name.data).is_null() {
            api_err_invalid(
                err,
                b"command name\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
        } else if mb_islower(
            *name.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        ) {
            api_err_invalid(
                err,
                b"command name (must start with uppercase)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
        } else if !(!((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
            || !((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong))
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"Cannot use both 'range' and 'count'\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            if (*opts).nargs.type_0 as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                match (*opts).nargs.data.integer {
                    0 => {}
                    1 => {
                        argt = (argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NOSPC | EX_NEEDARG))
                            as uint32_t;
                    }
                    _ => {
                        if true {
                            api_err_invalid(
                                err,
                                b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                                (*opts).nargs.data.integer,
                                false_0 != 0,
                            );
                            break '_err;
                        }
                    }
                }
            } else if (*opts).nargs.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if !((*opts).nargs.data.string.size <= 1 as size_t) {
                    api_err_invalid(
                        err,
                        b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                        (*opts).nargs.data.string.data,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_err;
                } else {
                    match *(*opts)
                        .nargs
                        .data
                        .string
                        .data
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                    {
                        42 => {
                            argt = (argt as ::core::ffi::c_uint | EX_EXTRA) as uint32_t;
                        }
                        63 => {
                            argt =
                                (argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NOSPC)) as uint32_t;
                        }
                        43 => {
                            argt =
                                (argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NEEDARG)) as uint32_t;
                        }
                        _ => {
                            if true {
                                api_err_invalid(
                                    err,
                                    b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                                    (*opts).nargs.data.string.data,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_err;
                            }
                        }
                    }
                }
            } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__nargs
                != 0 as ::core::ffi::c_ulonglong
            {
                if true {
                    api_err_invalid(
                        err,
                        b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                        b"\0".as_ptr() as *const ::core::ffi::c_char,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_err;
                }
            }
            if !(!((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 10 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong)
                || argt != 0)
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"'complete' used without 'nargs'\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                if (*opts).range.type_0 as ::core::ffi::c_uint
                    == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if (*opts).range.data.boolean {
                        argt = (argt as ::core::ffi::c_uint | EX_RANGE) as uint32_t;
                        addr_type_arg = ADDR_LINES;
                    }
                } else if (*opts).range.type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if !(*(*opts)
                        .range
                        .data
                        .string
                        .data
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '%' as ::core::ffi::c_int
                        && (*opts).range.data.string.size == 1 as size_t)
                    {
                        api_err_invalid(
                            err,
                            b"range\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    } else {
                        argt = (argt as ::core::ffi::c_uint | (EX_RANGE | EX_DFLALL)) as uint32_t;
                        addr_type_arg = ADDR_LINES;
                    }
                } else if (*opts).range.type_0 as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    argt = (argt as ::core::ffi::c_uint | (EX_RANGE | EX_ZEROR)) as uint32_t;
                    def = (*opts).range.data.integer as int64_t;
                    addr_type_arg = ADDR_LINES;
                } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__range
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if true {
                        api_err_invalid(
                            err,
                            b"range\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    }
                }
                if (*opts).count.type_0 as ::core::ffi::c_uint
                    == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if (*opts).count.data.boolean {
                        argt = (argt as ::core::ffi::c_uint | (EX_COUNT | EX_ZEROR | EX_RANGE))
                            as uint32_t;
                        addr_type_arg = ADDR_OTHER;
                        def = 0 as int64_t;
                    }
                } else if (*opts).count.type_0 as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    argt = (argt as ::core::ffi::c_uint | (EX_COUNT | EX_ZEROR | EX_RANGE))
                        as uint32_t;
                    addr_type_arg = ADDR_OTHER;
                    def = (*opts).count.data.integer as int64_t;
                } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__count
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if true {
                        api_err_invalid(
                            err,
                            b"count\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    }
                }
                if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__addr
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                        != (*opts).addr.type_0 as ::core::ffi::c_uint
                    {
                        api_err_exp(
                            err,
                            b"addr\0".as_ptr() as *const ::core::ffi::c_char,
                            api_typename(kObjectTypeString),
                            api_typename((*opts).addr.type_0),
                        );
                        break '_err;
                    } else if !(1 as ::core::ffi::c_int
                        == parse_addr_type_arg(
                            (*opts).addr.data.string.data,
                            (*opts).addr.data.string.size as ::core::ffi::c_int,
                            &raw mut addr_type_arg,
                        ))
                    {
                        api_err_invalid(
                            err,
                            b"addr\0".as_ptr() as *const ::core::ffi::c_char,
                            (*opts).addr.data.string.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    } else {
                        argt = (argt as ::core::ffi::c_uint | EX_RANGE) as uint32_t;
                        if addr_type_arg as ::core::ffi::c_uint
                            != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            argt = (argt as ::core::ffi::c_uint | EX_ZEROR) as uint32_t;
                        }
                    }
                }
                if (*opts).bang {
                    argt = (argt as ::core::ffi::c_uint | EX_BANG) as uint32_t;
                }
                if (*opts).bar {
                    argt = (argt as ::core::ffi::c_uint | EX_TRLBAR) as uint32_t;
                }
                if (*opts).register_ {
                    argt = (argt as ::core::ffi::c_uint | EX_REGSTR) as uint32_t;
                }
                if (*opts).keepscript {
                    argt = (argt as ::core::ffi::c_uint | EX_KEEPSCRIPT) as uint32_t;
                }
                force = if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__force
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*opts).force as ::core::ffi::c_int
                } else {
                    true_0
                } != 0;
                if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                    if (*opts).complete.type_0 as ::core::ffi::c_uint
                        == kObjectTypeLuaRef as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        context = EXPAND_USER_LUA as ::core::ffi::c_int;
                        compl_luaref = (*opts).complete.data.luaref;
                        (*opts).complete.data.luaref = LUA_NOREF as LuaRef;
                    } else if (*opts).complete.type_0 as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if !(1 as ::core::ffi::c_int
                            == parse_compl_arg(
                                (*opts).complete.data.string.data,
                                (*opts).complete.data.string.size as ::core::ffi::c_int,
                                &raw mut context,
                                &raw mut argt,
                                &raw mut compl_arg,
                            ))
                        {
                            api_err_invalid(
                                err,
                                b"complete\0".as_ptr() as *const ::core::ffi::c_char,
                                (*opts).complete.data.string.data,
                                0 as int64_t,
                                true_0 != 0,
                            );
                            break '_err;
                        }
                    } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__complete
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if true {
                            api_err_exp(
                                err,
                                b"complete\0".as_ptr() as *const ::core::ffi::c_char,
                                b"Function or String\0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                            break '_err;
                        }
                    }
                    if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__preview
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if kObjectTypeLuaRef as ::core::ffi::c_int as ::core::ffi::c_uint
                            != (*opts).preview.type_0 as ::core::ffi::c_uint
                        {
                            api_err_exp(
                                err,
                                b"preview\0".as_ptr() as *const ::core::ffi::c_char,
                                api_typename(kObjectTypeLuaRef),
                                api_typename((*opts).preview.type_0),
                            );
                            break '_err;
                        } else {
                            argt = (argt as ::core::ffi::c_uint | EX_PREVIEW) as uint32_t;
                            preview_luaref = (*opts).preview.data.luaref;
                            (*opts).preview.data.luaref = LUA_NOREF as LuaRef;
                        }
                    }
                    match cmd.type_0 as ::core::ffi::c_uint {
                        7 => {
                            luaref = api_new_luaref(cmd.data.luaref);
                            if (*opts).desc.type_0 as ::core::ffi::c_uint
                                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                rep = (*opts).desc.data.string.data;
                            } else {
                                rep = b"\0".as_ptr() as *const ::core::ffi::c_char;
                            }
                        }
                        4 => {
                            rep = cmd.data.string.data;
                        }
                        _ => {
                            if true {
                                api_err_exp(
                                    err,
                                    b"command\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Function or String\0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                                break '_err;
                            }
                        }
                    }
                    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
                    if uc_add_command(
                        name.data,
                        name.size,
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
                    ) != 1 as ::core::ffi::c_int
                    {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Failed to create user command\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                    current_sctx.set(save_current_sctx);
                    return;
                }
            }
        }
    }
    if luaref != LUA_NOREF {
        api_free_luaref(luaref);
        luaref = LUA_NOREF as LuaRef;
    }
    if compl_luaref != LUA_NOREF {
        api_free_luaref(compl_luaref);
        compl_luaref = LUA_NOREF as LuaRef;
    }
    if preview_luaref != LUA_NOREF {
        api_free_luaref(preview_luaref);
        preview_luaref = LUA_NOREF as LuaRef;
    }
    xfree(compl_arg as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn nvim_get_commands(
    mut opts: *mut KeyDict_get_commands,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    return nvim_buf_get_commands(-1 as Buffer, opts, arena, err);
}
pub unsafe extern "C" fn nvim_buf_get_commands(
    mut buf: Buffer,
    mut opts: *mut KeyDict_get_commands,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut global: bool = buf == -1 as ::core::ffi::c_int;
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    if global {
        if (*opts).builtin {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"builtin=true not implemented\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            };
        }
        return commands_array(::core::ptr::null_mut::<buf_T>(), arena);
    }
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if (*opts).builtin as ::core::ffi::c_int != 0 || b.is_null() {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    return commands_array(b, arena);
}
