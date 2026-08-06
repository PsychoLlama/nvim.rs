//! `nvim_parse_cmd()`: an Ex command line as a Dict.
//!
//! It runs the real parser (`parse_cmd_line`) over the string and renders
//! every field the caller could need to rebuild it -- the command name, the
//! bang, the range, the count, the register, the arguments, the magic
//! characters and the whole `cmod_*` modifier set.  `parse_map_cmd` is the
//! `:map`-family special case, whose arguments the generic splitter would
//! mangle.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn parse_map_cmd(
    mut arg_str: *const ::core::ffi::c_char,
    mut arena: *mut Arena,
) -> Array {
    unsafe {
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
}

pub unsafe extern "C" fn nvim_parse_cmd(
    mut str: String_0,
    mut _opts: *mut KeyDict_empty,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> KeyDict_cmd {
    unsafe {
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
                    addr = b"line\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
                2 => {
                    addr =
                        b"arg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                }
                4 => {
                    addr =
                        b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                }
                3 => {
                    addr = b"load\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
                1 => {
                    addr =
                        b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                }
                5 => {
                    addr =
                        b"tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                }
                8 => {
                    addr =
                        b"qf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                }
                11 => {
                    addr = b"none\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
                _ => {
                    addr =
                        b"?\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
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
                        boolean: cmdinfo.cmdmod.cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int
                            != 0,
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
                        boolean: cmdinfo.cmdmod.cmod_flags & CMOD_UNSILENT as ::core::ffi::c_int
                            != 0,
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
                        boolean: cmdinfo.cmdmod.cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int
                            != 0,
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
                        boolean: cmdinfo.cmdmod.cmod_flags & CMOD_NOAUTOCMD as ::core::ffi::c_int
                            != 0,
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
                        boolean: cmdinfo.cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int
                            != 0,
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
                        boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                            != 0,
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
                        boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int
                            != 0,
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
                        boolean: cmdinfo.cmdmod.cmod_flags & CMOD_KEEPMARKS as ::core::ffi::c_int
                            != 0,
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
                        boolean: cmdinfo.cmdmod.cmod_flags
                            & CMOD_KEEPPATTERNS as ::core::ffi::c_int
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
                        boolean: cmdinfo.cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                            != 0,
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
                        boolean: cmdinfo.cmdmod.cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int
                            != 0,
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
                split = b"botright\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if cmdinfo.cmdmod.cmod_split & WSP_TOP as ::core::ffi::c_int != 0 {
                split =
                    b"topleft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else if cmdinfo.cmdmod.cmod_split & WSP_BELOW as ::core::ffi::c_int != 0 {
                split = b"belowright\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if cmdinfo.cmdmod.cmod_split & WSP_ABOVE as ::core::ffi::c_int != 0 {
                split = b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
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
}
