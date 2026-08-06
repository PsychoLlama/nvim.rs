//! `nvim_cmd()`: executing a command given as a Dict.
//!
//! The inverse of [`super::parse`]: every field is validated against the
//! command's `argt` flags (which arguments it accepts, whether it takes a
//! range, a count, a register or a bang), the `mods` sub-keyset is unpacked
//! into an `cmdmod_T`, and the result is handed to `execute_cmd` -- with
//! the output captured when `opts.output` is set.

// One transpiled body of 900-odd lines: the four-space shift a wrapping
// block costs would put this file back over the 1,000-line cap.  Opt
// out until the rewrite shortens it.
#![allow(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

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
