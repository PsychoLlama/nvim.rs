//! Per-context match generators for the contexts with no module of their own.
//!
//! [`ExpandOther`] is the table of `(context, generator)` pairs that
//! [`super::fromcontext::ExpandFromContext`] dispatches through, plus the
//! generators small enough to live next to it — `:breakadd`, `:scriptnames`,
//! `:retab`, `:messages`, `:mapclear`, `:filetype`, `:checkhealth` and the
//! LSP list.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn expand_files_and_dirs(
    mut xp: *mut expand_T,
    mut pat: *mut ::core::ffi::c_char,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut free_pat: bool = false_0 != 0;
        if (*xp).xp_backslash != XP_BS_NONE as ::core::ffi::c_int {
            free_pat = true_0 != 0;
            let mut pat_len: size_t = strlen(pat);
            pat = xstrnsave(pat, pat_len);
            let mut pat_end: *mut ::core::ffi::c_char = pat.offset(pat_len as isize);
            let mut p: *mut ::core::ffi::c_char = pat;
            while *p as ::core::ffi::c_int != NUL {
                if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                    if (*xp).xp_backslash & XP_BS_THREE as ::core::ffi::c_int != 0
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                    {
                        let mut from: *mut ::core::ffi::c_char =
                            p.offset(3 as ::core::ffi::c_int as isize);
                        memmove(
                            p as *mut ::core::ffi::c_void,
                            from as *const ::core::ffi::c_void,
                            (pat_end.offset_from(from) as size_t).wrapping_add(1 as size_t),
                        );
                        pat_end = pat_end.offset(-(3 as ::core::ffi::c_int as isize));
                    } else if (*xp).xp_backslash & XP_BS_ONE as ::core::ffi::c_int != 0
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                    {
                        let mut from_0: *mut ::core::ffi::c_char =
                            p.offset(1 as ::core::ffi::c_int as isize);
                        memmove(
                            p as *mut ::core::ffi::c_void,
                            from_0 as *const ::core::ffi::c_void,
                            (pat_end.offset_from(from_0) as size_t).wrapping_add(1 as size_t),
                        );
                        pat_end = pat_end.offset(-1);
                    } else if (*xp).xp_backslash & XP_BS_COMMA as ::core::ffi::c_int != 0 {
                        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                            && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == ',' as ::core::ffi::c_int
                        {
                            let mut from_1: *mut ::core::ffi::c_char =
                                p.offset(2 as ::core::ffi::c_int as isize);
                            memmove(
                                p as *mut ::core::ffi::c_void,
                                from_1 as *const ::core::ffi::c_void,
                                (pat_end.offset_from(from_1) as size_t).wrapping_add(1 as size_t),
                            );
                            pat_end = pat_end.offset(-(2 as ::core::ffi::c_int as isize));
                        }
                    }
                }
                p = p.offset(1);
            }
        }
        let mut ret: ::core::ffi::c_int = FAIL;
        if (*xp).xp_context == EXPAND_FINDFUNC as ::core::ffi::c_int {
            ret = expand_findfunc(pat, matches, numMatches);
        } else {
            if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int {
                flags |= EW_FILE as ::core::ffi::c_int;
            } else if (*xp).xp_context == EXPAND_FILES_IN_PATH as ::core::ffi::c_int {
                flags |= EW_FILE as ::core::ffi::c_int | EW_PATH as ::core::ffi::c_int;
            } else if (*xp).xp_context == EXPAND_DIRS_IN_CDPATH as ::core::ffi::c_int {
                flags = (flags | EW_DIR as ::core::ffi::c_int | EW_CDPATH as ::core::ffi::c_int)
                    & !(EW_FILE as ::core::ffi::c_int);
            } else {
                flags = (flags | EW_DIR as ::core::ffi::c_int) & !(EW_FILE as ::core::ffi::c_int);
            }
            if options & WILD_ICASE as ::core::ffi::c_int != 0 {
                flags |= EW_ICASE as ::core::ffi::c_int;
            }
            ret = expand_wildcards_eval(&raw mut pat, numMatches, matches, flags);
        }
        if free_pat {
            xfree(pat as *mut ::core::ffi::c_void);
        }
        return ret;
    }
}

pub(crate) unsafe extern "C" fn get_filetypecmd_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx < 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if filetype_expand_what.get() as ::core::ffi::c_uint
        == EXP_FILETYPECMD_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
        && idx < 4 as ::core::ffi::c_int
    {
        let mut opts_all: [*mut ::core::ffi::c_char; 4] = [
            b"indent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"plugin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        return opts_all[idx as usize];
    }
    if filetype_expand_what.get() as ::core::ffi::c_uint
        == EXP_FILETYPECMD_PLUGIN as ::core::ffi::c_int as ::core::ffi::c_uint
        && idx < 3 as ::core::ffi::c_int
    {
        let mut opts_plugin: [*mut ::core::ffi::c_char; 3] = [
            b"plugin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        return opts_plugin[idx as usize];
    }
    if filetype_expand_what.get() as ::core::ffi::c_uint
        == EXP_FILETYPECMD_INDENT as ::core::ffi::c_int as ::core::ffi::c_uint
        && idx < 3 as ::core::ffi::c_int
    {
        let mut opts_indent: [*mut ::core::ffi::c_char; 3] = [
            b"indent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        return opts_indent[idx as usize];
    }
    if filetype_expand_what.get() as ::core::ffi::c_uint
        == EXP_FILETYPECMD_ONOFF as ::core::ffi::c_int as ::core::ffi::c_uint
        && idx < 2 as ::core::ffi::c_int
    {
        let mut opts_onoff: [*mut ::core::ffi::c_char; 2] = [
            b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        return opts_onoff[idx as usize];
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}

pub(crate) unsafe extern "C" fn get_breakadd_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx >= 0 as ::core::ffi::c_int && idx <= 3 as ::core::ffi::c_int {
        let mut opts: [*mut ::core::ffi::c_char; 4] = [
            b"expr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"file\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"func\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"here\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        if breakpt_expand_what.get() as ::core::ffi::c_uint
            == EXP_BREAKPT_ADD as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return opts[idx as usize];
        } else if breakpt_expand_what.get() as ::core::ffi::c_uint
            == EXP_BREAKPT_DEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if idx <= 2 as ::core::ffi::c_int {
                return opts[(idx + 1 as ::core::ffi::c_int) as usize];
            }
        } else if idx <= 1 as ::core::ffi::c_int {
            return opts[(idx + 1 as ::core::ffi::c_int) as usize];
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}

pub(crate) unsafe extern "C" fn get_scriptnames_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if !(idx + 1 as ::core::ffi::c_int > 0 as ::core::ffi::c_int
            && idx + 1 as ::core::ffi::c_int <= (*script_items.ptr()).ga_len)
        {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut si: *mut scriptitem_T = *((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
            .offset((idx + 1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize);
        home_replace(
            ::core::ptr::null::<buf_T>(),
            (*si).sn_name,
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            true_0 != 0,
        );
        return NameBuff.ptr() as *mut ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn get_retab_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx == 0 as ::core::ffi::c_int {
        return b"-indentonly\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}

pub(crate) unsafe extern "C" fn get_messages_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx == 0 as ::core::ffi::c_int {
        return b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}

pub(crate) unsafe extern "C" fn get_mapclear_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx == 0 as ::core::ffi::c_int {
        return b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}

pub(crate) unsafe extern "C" fn get_healthcheck_names(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static names: GlobalCell<Object> = GlobalCell::new(object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        });
        static last_gen: GlobalCell<::core::ffi::c_uint> =
            GlobalCell::new(0 as ::core::ffi::c_uint);
        if last_gen.get() != get_cmdline_last_prompt_id()
            || last_gen.get() == 0 as ::core::ffi::c_uint
        {
            let mut a: Array = ARRAY_DICT_INIT;
            let mut err: Error = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            let mut res: Object = nlua_exec(
                String_0 {
                    data: b"return vim.health._complete()\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 30]>()
                        .wrapping_sub(1 as size_t),
                },
                ::core::ptr::null::<::core::ffi::c_char>(),
                a,
                kRetObject,
                ::core::ptr::null_mut::<Arena>(),
                &raw mut err,
            );
            api_clear_error(&raw mut err);
            api_free_object(names.get());
            names.set(res);
            last_gen.set(get_cmdline_last_prompt_id());
        }
        if (*names.ptr()).type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            && idx < (*names.ptr()).data.array.size as ::core::ffi::c_int
            && (*(*names.ptr()).data.array.items.offset(idx as isize)).type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return (*(*names.ptr()).data.array.items.offset(idx as isize))
                .data
                .string
                .data;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn get_lsp_arg(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static names: GlobalCell<Object> = GlobalCell::new(object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        });
        static last_xp_line: GlobalCell<*mut ::core::ffi::c_char> =
            GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
        static last_gen: GlobalCell<::core::ffi::c_uint> =
            GlobalCell::new(0 as ::core::ffi::c_uint);
        if (*last_xp_line.ptr()).is_null()
            || strcmp(last_xp_line.get(), (*xp).xp_line) != 0 as ::core::ffi::c_int
            || last_gen.get() != get_cmdline_last_prompt_id()
        {
            xfree(last_xp_line.get() as *mut ::core::ffi::c_void);
            last_xp_line.set(xstrdup((*xp).xp_line));
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 1] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 1];
            args.capacity = 1 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let mut err: Error = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            let c2rust_fresh0 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh0 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*xp).xp_line),
                },
            };
            let mut res: Object = nlua_exec(
                String_0 {
                    data: b"return require'vim._core.ex_cmd'.lsp_complete(...)\0".as_ptr()
                        as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                        .wrapping_sub(1 as size_t),
                },
                ::core::ptr::null::<::core::ffi::c_char>(),
                args,
                kRetObject,
                ::core::ptr::null_mut::<Arena>(),
                &raw mut err,
            );
            api_clear_error(&raw mut err);
            api_free_object(names.get());
            names.set(res);
            last_gen.set(get_cmdline_last_prompt_id());
        }
        if (*names.ptr()).type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            && idx < (*names.ptr()).data.array.size as ::core::ffi::c_int
            && (*(*names.ptr()).data.array.items.offset(idx as isize)).type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return (*(*names.ptr()).data.array.items.offset(idx as isize))
                .data
                .string
                .data;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn ExpandOther(
    mut pat: *mut ::core::ffi::c_char,
    mut xp: *mut expand_T,
    mut rmp: *mut regmatch_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        static tab: GlobalCell<[expgen; 33]> = GlobalCell::new([
            expgen {
                context: EXPAND_COMMANDS as ::core::ffi::c_int,
                func: Some(
                    get_command_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_FILETYPECMD as ::core::ffi::c_int,
                func: Some(
                    get_filetypecmd_arg
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_MAPCLEAR as ::core::ffi::c_int,
                func: Some(
                    get_mapclear_arg
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_MESSAGES as ::core::ffi::c_int,
                func: Some(
                    get_messages_arg
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_HISTORY as ::core::ffi::c_int,
                func: Some(
                    get_history_arg
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_USER_COMMANDS as ::core::ffi::c_int,
                func: Some(
                    get_user_commands
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_USER_ADDR_TYPE as ::core::ffi::c_int,
                func: Some(
                    get_user_cmd_addr_type
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_USER_CMD_FLAGS as ::core::ffi::c_int,
                func: Some(
                    get_user_cmd_flags
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_USER_NARGS as ::core::ffi::c_int,
                func: Some(
                    get_user_cmd_nargs
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_USER_COMPLETE as ::core::ffi::c_int,
                func: Some(
                    get_user_cmd_complete
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_USER_VARS as ::core::ffi::c_int,
                func: Some(
                    get_user_var_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_FUNCTIONS as ::core::ffi::c_int,
                func: Some(
                    get_function_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_USER_FUNC as ::core::ffi::c_int,
                func: Some(
                    get_user_func_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_EXPRESSION as ::core::ffi::c_int,
                func: Some(
                    get_expr_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_MENUS as ::core::ffi::c_int,
                func: Some(
                    get_menu_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_MENUNAMES as ::core::ffi::c_int,
                func: Some(
                    get_menu_names
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: false_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_SYNTAX as ::core::ffi::c_int,
                func: Some(
                    get_syntax_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_SYNTIME as ::core::ffi::c_int,
                func: Some(
                    get_syntime_arg
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_HIGHLIGHT as ::core::ffi::c_int,
                func: Some(
                    get_highlight_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: false_0,
            },
            expgen {
                context: EXPAND_EVENTS as ::core::ffi::c_int,
                func: Some(
                    expand_get_event_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: false_0,
            },
            expgen {
                context: EXPAND_AUGROUP as ::core::ffi::c_int,
                func: Some(
                    expand_get_augroup_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: false_0,
            },
            expgen {
                context: EXPAND_SIGN as ::core::ffi::c_int,
                func: Some(
                    get_sign_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_PROFILE as ::core::ffi::c_int,
                func: Some(
                    get_profile_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_LANGUAGE as ::core::ffi::c_int,
                func: Some(
                    get_lang_arg
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: false_0,
            },
            expgen {
                context: EXPAND_LOCALES as ::core::ffi::c_int,
                func: Some(
                    get_locales
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: false_0,
            },
            expgen {
                context: EXPAND_ENV_VARS as ::core::ffi::c_int,
                func: Some(
                    get_env_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_USER as ::core::ffi::c_int,
                func: Some(
                    get_users
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: false_0,
            },
            expgen {
                context: EXPAND_ARGLIST as ::core::ffi::c_int,
                func: Some(
                    get_arglist_name
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: false_0,
            },
            expgen {
                context: EXPAND_BREAKPOINT as ::core::ffi::c_int,
                func: Some(
                    get_breakadd_arg
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_SCRIPTNAMES as ::core::ffi::c_int,
                func: Some(
                    get_scriptnames_arg
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: false_0,
            },
            expgen {
                context: EXPAND_RETAB as ::core::ffi::c_int,
                func: Some(
                    get_retab_arg
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: true_0,
            },
            expgen {
                context: EXPAND_CHECKHEALTH as ::core::ffi::c_int,
                func: Some(
                    get_healthcheck_names
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: false_0,
            },
            expgen {
                context: EXPAND_LSP as ::core::ffi::c_int,
                func: Some(
                    get_lsp_arg
                        as unsafe extern "C" fn(
                            *mut expand_T,
                            ::core::ffi::c_int,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                ic: true_0,
                escaped: false_0,
            },
        ]);
        let mut ret: ::core::ffi::c_int = FAIL;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < ::core::mem::size_of::<[expgen; 33]>()
            .wrapping_div(::core::mem::size_of::<expgen>())
            .wrapping_div(
                (::core::mem::size_of::<[expgen; 33]>()
                    .wrapping_rem(::core::mem::size_of::<expgen>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
        {
            if (*xp).xp_context == (*tab.ptr())[i as usize].context {
                if (*tab.ptr())[i as usize].ic != 0 {
                    (*rmp).rm_ic = true_0 != 0;
                }
                ExpandGeneric(
                    pat,
                    xp,
                    rmp,
                    matches,
                    numMatches,
                    (*tab.ptr())[i as usize].func as CompleteListItemGetter,
                    (*tab.ptr())[i as usize].escaped != 0,
                );
                ret = OK;
                break;
            } else {
                i += 1;
            }
        }
        return ret;
    }
}
