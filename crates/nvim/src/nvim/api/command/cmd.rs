//! `nvim_cmd()`: executing a command given as a Dict.
//!
//! The inverse of [`super::parse`]: every field is validated against the
//! command's `argt` flags (which arguments it accepts, whether it takes a
//! range, a count, a register or a bang), the `mods` sub-keyset is unpacked
//! into an `cmdmod_T`, and the result is handed to `execute_cmd` -- with
//! the output captured when `opts.output` is set.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::{array_add, has_key};

pub unsafe extern "C" fn nvim_cmd(
    mut channel_id: uint64_t,
    mut cmd: *mut KeyDict_cmd,
    mut opts: *mut KeyDict_cmd_opts,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    unsafe {
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
            if !(has_key((*cmd).is_set__cmd_, 1 as ::core::ffi::c_int)) {
                api_err_required(err, c"cmd".as_ptr());
            } else {
                if *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == NUL
                {
                    if !(has_key((*cmd).is_set__cmd_, 10 as ::core::ffi::c_int)
                        && (*cmd).range.size > 0 as size_t
                        || has_key((*cmd).is_set__cmd_, 5 as ::core::ffi::c_int))
                    {
                        api_err_exp(
                            err,
                            c"cmd".as_ptr(),
                            c"non-empty String".as_ptr(),
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
                        true,
                        ::core::ptr::null_mut::<buf_T>(),
                    ) as ::core::ffi::c_int;
                    p = if ret != 0 && !aborting() {
                        find_ex_command(&raw mut ea, ::core::ptr::null_mut::<::core::ffi::c_int>())
                    } else {
                        ea.cmd
                    };
                }
                range_only = ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                    && *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == NUL
                    && has_key((*cmd).is_set__cmd_, KEYSET_OPTIDX_cmd__range)
                    && (*cmd).range.size > 0 as size_t;
                if !(ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                    && *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == NUL
                    && (!(has_key((*cmd).is_set__cmd_, KEYSET_OPTIDX_cmd__range))
                        || (*cmd).range.size == 0 as size_t)
                    && has_key((*cmd).is_set__cmd_, KEYSET_OPTIDX_cmd__mods))
                {
                    if !(!p.is_null()
                        && ea.cmdidx as ::core::ffi::c_int != CMD_SIZE as ::core::ffi::c_int
                        || range_only as ::core::ffi::c_int != 0)
                    {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            c"Command not found: %s".as_ptr(),
                            cmdname,
                        );
                    } else if !(range_only as ::core::ffi::c_int != 0 || !is_cmd_ni(ea.cmdidx)) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            c"Command not implemented: %s".as_ptr(),
                            cmdname,
                        );
                    } else {
                        if !range_only {
                            let mut fullname: *const ::core::ffi::c_char = if (ea.cmdidx
                                as ::core::ffi::c_int)
                                < 0 as ::core::ffi::c_int
                            {
                                get_user_command_name(ea.useridx, ea.cmdidx as ::core::ffi::c_int)
                            } else {
                                get_command_name(
                                    ::core::ptr::null_mut::<expand_T>(),
                                    ea.cmdidx as ::core::ffi::c_int,
                                )
                            };
                            if !(strncmp(fullname, cmdname, strlen(cmdname))
                                == 0 as ::core::ffi::c_int)
                            {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    c"Invalid command: \"%s\"".as_ptr(),
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
                        count_from_first_arg = false;
                        if has_key((*cmd).is_set__cmd_, KEYSET_OPTIDX_cmd__args) {
                            if (*cmd).args.size == 1 as size_t
                                && ea.argt & EX_COUNT as uint32_t != 0
                                && ea.argt & EX_EXTRA as uint32_t == 0
                            {
                                let mut first_arg: Object =
                                    *(*cmd).args.items.offset(0 as ::core::ffi::c_int as isize);
                                let mut is_numeric: bool = false;
                                let mut count_value: int64_t = 0 as int64_t;
                                if first_arg.type_0 as ::core::ffi::c_uint
                                    == kObjectTypeInteger as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                {
                                    is_numeric = true;
                                    count_value = first_arg.data.integer as int64_t;
                                } else if first_arg.type_0 as ::core::ffi::c_uint
                                    == kObjectTypeString as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
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
                                        is_numeric = true;
                                        count_value = val as int64_t;
                                    }
                                }
                                if is_numeric as ::core::ffi::c_int != 0
                                    && count_value >= 0 as int64_t
                                {
                                    count_from_first_arg = true;
                                    ea.addr_count = 1 as ::core::ffi::c_int;
                                    ea.line2 = count_value as linenr_T;
                                    ea.line1 = ea.line2;
                                    args = arena_array(arena, 0 as size_t);
                                }
                            }
                            if !count_from_first_arg {
                                args = arena_array(arena, (*cmd).args.size);
                                for i in 0..(*cmd).args.size {
                                    let elem: Object = *(*cmd).args.items.add(i);
                                    match elem.type_0 {
                                        // A boolean argument is spelled to the
                                        // command as "0" or "1".
                                        kObjectTypeBoolean => {
                                            let data_str: *mut ::core::ffi::c_char =
                                                arena_alloc(arena, 2, false).cast();
                                            *data_str = if elem.data.boolean { b'1' } else { b'0' }
                                                as ::core::ffi::c_char;
                                            *data_str.add(1) = NUL as ::core::ffi::c_char;
                                            array_add(
                                                &mut args,
                                                Object::string(cstr_as_string(data_str)),
                                            );
                                        }
                                        // A handle is its id, like any integer.
                                        kObjectTypeBuffer | kObjectTypeWindow
                                        | kObjectTypeTabpage | kObjectTypeInteger => {
                                            let data_str: *mut ::core::ffi::c_char =
                                                arena_alloc(arena, NUMBUFLEN as size_t, false)
                                                    .cast();
                                            snprintf(
                                                data_str,
                                                NUMBUFLEN as size_t,
                                                c"%ld".as_ptr(),
                                                elem.data.integer,
                                            );
                                            array_add(
                                                &mut args,
                                                Object::string(cstr_as_string(data_str)),
                                            );
                                        }
                                        kObjectTypeString => {
                                            // An all-whitespace argument would
                                            // vanish into the separators.
                                            if string_iswhite(elem.data.string) {
                                                api_err_exp(
                                                    err,
                                                    c"command arg".as_ptr(),
                                                    c"non-whitespace".as_ptr(),
                                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                                );
                                                break '_end;
                                            }
                                            array_add(&mut args, elem);
                                        }
                                        _ => {
                                            api_err_exp(
                                                err,
                                                c"command arg".as_ptr(),
                                                c"valid type".as_ptr(),
                                                api_typename(elem.type_0),
                                            );
                                            break '_end;
                                        }
                                    }
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
                                        argc_valid = true;
                                    }
                                    _ => {
                                        argc_valid = args.size == 0 as size_t;
                                    }
                                }
                                if !argc_valid {
                                    api_set_error(
                                        err,
                                        kErrorTypeValidation,
                                        c"%s".as_ptr(),
                                        c"Wrong number of arguments".as_ptr(),
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
                        if has_key((*cmd).is_set__cmd_, KEYSET_OPTIDX_cmd__range) {
                            if ea.argt & 0x1 as uint32_t == 0 {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    c"Command cannot accept %s: %s".as_ptr(),
                                    c"range".as_ptr(),
                                    (*cmd).cmd.data,
                                );
                                break '_end;
                            } else if !((*cmd).range.size <= 2 as size_t) {
                                api_err_exp(
                                    err,
                                    c"range".as_ptr(),
                                    c"<=2 elements".as_ptr(),
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                                break '_end;
                            }
                            let range: Array = (*cmd).range;
                            ea.addr_count = range.size as ::core::ffi::c_int;
                            for i in 0..range.size {
                                let elem: Object = *range.items.add(i);
                                if elem.type_0 != kObjectTypeInteger || elem.data.integer < 0 {
                                    api_err_exp(
                                        err,
                                        c"range element".as_ptr(),
                                        c"non-negative Integer".as_ptr(),
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                    );
                                    break '_end;
                                }
                            }
                            // One element gives both bounds.
                            if range.size > 0 {
                                ea.line1 = (*range.items).data.integer as linenr_T;
                                ea.line2 =
                                    (*range.items.add(range.size - 1)).data.integer as linenr_T;
                            }
                            if !invalid_range(&raw mut ea).is_null() {
                                api_err_invalid(
                                    err,
                                    c"range".as_ptr(),
                                    c"".as_ptr(),
                                    0 as int64_t,
                                    true,
                                );
                                break '_end;
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
                        if has_key((*cmd).is_set__cmd_, KEYSET_OPTIDX_cmd__count) {
                            if count_from_first_arg {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    c"%s".as_ptr(),
                                    c"Cannot specify both 'count' and numeric argument".as_ptr(),
                                );
                                break '_end;
                            } else if ea.argt & 0x400 as uint32_t == 0 {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    c"Command cannot accept %s: %s".as_ptr(),
                                    c"count".as_ptr(),
                                    (*cmd).cmd.data,
                                );
                                break '_end;
                            } else if !((*cmd).count >= 0 as Integer) {
                                api_err_exp(
                                    err,
                                    c"count".as_ptr(),
                                    c"non-negative Integer".as_ptr(),
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                                break '_end;
                            }
                            set_cmd_count(&raw mut ea, (*cmd).count as linenr_T, true);
                        }
                        if has_key((*cmd).is_set__cmd_, KEYSET_OPTIDX_cmd__reg) {
                            if ea.argt & 0x200 as uint32_t == 0 {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    c"Command cannot accept %s: %s".as_ptr(),
                                    c"register".as_ptr(),
                                    (*cmd).cmd.data,
                                );
                                break '_end;
                            } else if !((*cmd).reg.size == 1 as size_t) {
                                api_err_exp(
                                    err,
                                    c"reg".as_ptr(),
                                    c"single character".as_ptr(),
                                    (*cmd).reg.data,
                                );
                                break '_end;
                            }
                            let mut regname: ::core::ffi::c_char =
                                *(*cmd).reg.data.offset(0 as ::core::ffi::c_int as isize);
                            if !(regname as ::core::ffi::c_int != '=' as ::core::ffi::c_int) {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    c"%s".as_ptr(),
                                    c"Cannot use register \"=".as_ptr(),
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
                                    c"Invalid register: \"%c".as_ptr(),
                                    regname as ::core::ffi::c_int,
                                );
                                break '_end;
                            }
                            ea.regname = regname as uint8_t as ::core::ffi::c_int;
                        }
                        ea.forceit = (*cmd).bang as ::core::ffi::c_int;
                        if !(ea.forceit == 0 || ea.argt & 0x2 as uint32_t != 0) {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                c"Command cannot accept %s: %s".as_ptr(),
                                c"bang".as_ptr(),
                                (*cmd).cmd.data,
                            );
                        } else {
                            if has_key((*cmd).is_set__cmd_, KEYSET_OPTIDX_cmd__magic) {
                                let mut magic: [KeyDict_cmd_magic; 1] = [::core::mem::zeroed()];
                                let magic_p: *mut KeyDict_cmd_magic =
                                    &raw mut magic as *mut KeyDict_cmd_magic;
                                if !api_dict_to_keydict(
                                    magic_p as *mut ::core::ffi::c_void,
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
                                }
                                cmdinfo.magic.file = if has_key(
                                    (*magic_p).is_set__cmd_magic_,
                                    KEYSET_OPTIDX_cmd_magic__file,
                                ) {
                                    (*magic_p).file as uint32_t
                                } else {
                                    ea.argt & EX_XFILE as uint32_t
                                } != 0;
                                cmdinfo.magic.bar = if has_key(
                                    (*magic_p).is_set__cmd_magic_,
                                    KEYSET_OPTIDX_cmd_magic__bar,
                                ) {
                                    (*magic_p).bar as uint32_t
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
                            } else {
                                cmdinfo.magic.file = ea.argt & EX_XFILE as uint32_t != 0;
                                cmdinfo.magic.bar = ea.argt & EX_TRLBAR as uint32_t != 0;
                            }
                            if has_key((*cmd).is_set__cmd_, KEYSET_OPTIDX_cmd__mods) {
                                let mut mods: [KeyDict_cmd_mods; 1] = [::core::mem::zeroed()];
                                let mods_p: *mut KeyDict_cmd_mods =
                                    &raw mut mods as *mut KeyDict_cmd_mods;
                                if !api_dict_to_keydict(
                                    mods_p as *mut ::core::ffi::c_void,
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
                                }
                                if has_key(
                                    (*mods_p).is_set__cmd_mods_,
                                    KEYSET_OPTIDX_cmd_mods__filter,
                                ) {
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
                                    let filter_p: *mut KeyDict_cmd_mods_filter =
                                        &raw mut filter as *mut KeyDict_cmd_mods_filter;
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
                                        (*mods_p).filter,
                                        err,
                                    ) {
                                        break '_end;
                                    } else if has_key(
                                        (*filter_p).is_set__cmd_mods_filter_,
                                        KEYSET_OPTIDX_cmd_mods_filter__pattern,
                                    ) {
                                        cmdinfo.cmdmod.cmod_filter_force =
                                            (*filter_p).force as bool;
                                        if *(*filter_p).pattern.data as ::core::ffi::c_int != NUL
                                            || cmdinfo.cmdmod.cmod_filter_force
                                                as ::core::ffi::c_int
                                                != 0
                                        {
                                            cmdinfo.cmdmod.cmod_filter_pat =
                                                string_to_cstr((*filter_p).pattern);
                                            cmdinfo.cmdmod.cmod_filter_regmatch.regprog =
                                                vim_regcomp(
                                                    cmdinfo.cmdmod.cmod_filter_pat,
                                                    RE_MAGIC,
                                                );
                                        }
                                    }
                                }
                                if has_key((*mods_p).is_set__cmd_mods_, KEYSET_OPTIDX_cmd_mods__tab)
                                {
                                    if (*mods_p).tab as ::core::ffi::c_int
                                        >= 0 as ::core::ffi::c_int
                                    {
                                        cmdinfo.cmdmod.cmod_tab = (*mods_p).tab
                                            as ::core::ffi::c_int
                                            + 1 as ::core::ffi::c_int;
                                    }
                                }
                                if has_key(
                                    (*mods_p).is_set__cmd_mods_,
                                    KEYSET_OPTIDX_cmd_mods__verbose,
                                ) {
                                    if (*mods_p).verbose as ::core::ffi::c_int
                                        >= 0 as ::core::ffi::c_int
                                    {
                                        cmdinfo.cmdmod.cmod_verbose = (*mods_p).verbose
                                            as ::core::ffi::c_int
                                            + 1 as ::core::ffi::c_int;
                                    }
                                }
                                cmdinfo.cmdmod.cmod_split |=
                                    if (*mods_p).vertical as ::core::ffi::c_int != 0 {
                                        WSP_VERT as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    };
                                cmdinfo.cmdmod.cmod_split |=
                                    if (*mods_p).horizontal as ::core::ffi::c_int != 0 {
                                        WSP_HOR as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    };
                                if has_key(
                                    (*mods_p).is_set__cmd_mods_,
                                    KEYSET_OPTIDX_cmd_mods__split,
                                ) {
                                    if *(*mods_p).split.data as ::core::ffi::c_int != NUL {
                                        if strcmp((*mods_p).split.data, c"aboveleft".as_ptr())
                                            == 0 as ::core::ffi::c_int
                                            || strcmp((*mods_p).split.data, c"leftabove".as_ptr())
                                                == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_ABOVE as ::core::ffi::c_int;
                                        } else if strcmp(
                                            (*mods_p).split.data,
                                            c"belowright".as_ptr(),
                                        ) == 0 as ::core::ffi::c_int
                                            || strcmp((*mods_p).split.data, c"rightbelow".as_ptr())
                                                == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_BELOW as ::core::ffi::c_int;
                                        } else if strcmp((*mods_p).split.data, c"topleft".as_ptr())
                                            == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_TOP as ::core::ffi::c_int;
                                        } else if strcmp((*mods_p).split.data, c"botright".as_ptr())
                                            == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |=
                                                WSP_BOT as ::core::ffi::c_int;
                                        } else if true {
                                            api_err_invalid(
                                                err,
                                                c"mods.split".as_ptr(),
                                                c"".as_ptr(),
                                                0 as int64_t,
                                                true,
                                            );
                                            break '_end;
                                        }
                                    }
                                }
                                if (*mods_p).silent {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_SILENT as ::core::ffi::c_int;
                                }
                                if (*mods_p).emsg_silent {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_ERRSILENT as ::core::ffi::c_int;
                                }
                                if (*mods_p).unsilent {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_UNSILENT as ::core::ffi::c_int;
                                }
                                if (*mods_p).sandbox {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_SANDBOX as ::core::ffi::c_int;
                                }
                                if (*mods_p).noautocmd {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_NOAUTOCMD as ::core::ffi::c_int;
                                }
                                if (*mods_p).browse {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_BROWSE as ::core::ffi::c_int;
                                }
                                if (*mods_p).confirm {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_CONFIRM as ::core::ffi::c_int;
                                }
                                if (*mods_p).hide {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_HIDE as ::core::ffi::c_int;
                                }
                                if (*mods_p).keepalt {
                                    cmdinfo.cmdmod.cmod_flags |= CMOD_KEEPALT as ::core::ffi::c_int;
                                }
                                if (*mods_p).keepjumps {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_KEEPJUMPS as ::core::ffi::c_int;
                                }
                                if (*mods_p).keepmarks {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_KEEPMARKS as ::core::ffi::c_int;
                                }
                                if (*mods_p).keeppatterns {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_KEEPPATTERNS as ::core::ffi::c_int;
                                }
                                if (*mods_p).lockmarks {
                                    cmdinfo.cmdmod.cmod_flags |=
                                        CMOD_LOCKMARKS as ::core::ffi::c_int;
                                }
                                if (*mods_p).noswapfile {
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
                                        c"%s".as_ptr(),
                                        c"Command cannot be run in sandbox".as_ptr(),
                                    );
                                    break '_end;
                                }
                            }
                            build_cmdline_str(
                                &raw mut cmdline,
                                &raw mut ea,
                                &raw mut cmdinfo,
                                args,
                            );
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
                                            c"argument ".as_ptr(),
                                            orig_arg,
                                            0 as int64_t,
                                            true,
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
}
