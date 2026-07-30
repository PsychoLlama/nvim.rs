//! Autocommands, `:filetype` and `:setfiletype` — the commands
//! that decide what a buffer is.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ex_autocmd(mut eap: *mut exarg_T) {
    if secure.get() != 0 {
        secure.set(2 as c_int);
        (*eap).errmsg = gettext(&raw const e_curdir as *const c_char);
    } else if (*eap).cmdidx as c_int == CMD_autocmd as c_int {
        do_autocmd(eap, (*eap).arg, (*eap).forceit);
    } else {
        do_augroup((*eap).arg, (*eap).forceit != 0);
    };
}

pub(crate) unsafe extern "C" fn ex_doautocmd(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    let mut call_do_modelines: c_int = check_nomodeline(&raw mut arg) as c_int;
    let mut did_aucmd: bool = false;
    do_doautocmd(arg, false_0 != 0, &raw mut did_aucmd);
    if call_do_modelines != 0 && did_aucmd as c_int != 0 {
        do_modelines(0 as c_int);
    }
}

pub(crate) unsafe extern "C" fn ex_filetype(mut eap: *mut exarg_T) {
    if *(*eap).arg as c_int == NUL {
        smsg(
            0 as c_int,
            b"filetype detection:%s  plugin:%s  indent:%s\0".as_ptr() as *const c_char,
            if filetype_detect.get() as c_int == kTrue as c_int {
                b"ON\0".as_ptr() as *const c_char
            } else {
                b"OFF\0".as_ptr() as *const c_char
            },
            if filetype_plugin.get() as c_int == kTrue as c_int {
                if filetype_detect.get() as c_int == kTrue as c_int {
                    b"ON\0".as_ptr() as *const c_char
                } else {
                    b"(on)\0".as_ptr() as *const c_char
                }
            } else {
                b"OFF\0".as_ptr() as *const c_char
            },
            if filetype_indent.get() as c_int == kTrue as c_int {
                if filetype_detect.get() as c_int == kTrue as c_int {
                    b"ON\0".as_ptr() as *const c_char
                } else {
                    b"(on)\0".as_ptr() as *const c_char
                }
            } else {
                b"OFF\0".as_ptr() as *const c_char
            },
        );
        return;
    }
    let mut arg: *mut c_char = (*eap).arg;
    let mut plugin: bool = false_0 != 0;
    let mut indent: bool = false_0 != 0;
    loop {
        if strncmp(arg, b"plugin\0".as_ptr() as *const c_char, 6 as size_t) == 0 as c_int {
            plugin = true_0 != 0;
            arg = skipwhite(arg.offset(6 as c_int as isize));
        } else {
            if strncmp(arg, b"indent\0".as_ptr() as *const c_char, 6 as size_t) != 0 as c_int {
                break;
            }
            indent = true_0 != 0;
            arg = skipwhite(arg.offset(6 as c_int as isize));
        }
    }
    if strcmp(arg, b"on\0".as_ptr() as *const c_char) == 0 as c_int
        || strcmp(arg, b"detect\0".as_ptr() as *const c_char) == 0 as c_int
    {
        if *arg as c_int == 'o' as c_int || filetype_detect.get() as c_int != kTrue as c_int {
            source_runtime(FILETYPE_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
            filetype_detect.set(kTrue);
            if plugin {
                source_runtime(FTPLUGIN_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                filetype_plugin.set(kTrue);
            }
            if indent {
                source_runtime(INDENT_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                filetype_indent.set(kTrue);
            }
        }
        if *arg as c_int == 'd' as c_int {
            do_doautocmd(
                b"filetypedetect BufRead\0".as_ptr() as *const c_char as *mut c_char,
                true_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            do_modelines(0 as c_int);
        }
    } else if strcmp(arg, b"off\0".as_ptr() as *const c_char) == 0 as c_int {
        if plugin as c_int != 0 || indent as c_int != 0 {
            if plugin {
                source_runtime(FTPLUGOF_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                filetype_plugin.set(kFalse);
            }
            if indent {
                source_runtime(INDOFF_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                filetype_indent.set(kFalse);
            }
        } else {
            source_runtime(FTOFF_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
            filetype_detect.set(kFalse);
        }
    } else {
        semsg(gettext(&raw const e_invarg2 as *const c_char), arg);
    };
}

pub unsafe extern "C" fn filetype_plugin_enable() {
    if filetype_plugin.get() as c_int == kNone as c_int {
        source_runtime(FTPLUGIN_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
        filetype_plugin.set(kTrue);
    }
    if filetype_indent.get() as c_int == kNone as c_int {
        source_runtime(INDENT_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
        filetype_indent.set(kTrue);
    }
}

pub unsafe extern "C" fn filetype_maybe_enable() {
    if filetype_detect.get() as c_int == kNone as c_int {
        source_runtime(FILETYPE_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
        filetype_detect.set(kTrue);
    }
}

pub(crate) unsafe extern "C" fn ex_setfiletype(mut eap: *mut exarg_T) {
    if (*curbuf.get()).b_did_filetype {
        return;
    }
    let mut arg: *mut c_char = (*eap).arg;
    if strncmp(arg, b"FALLBACK \0".as_ptr() as *const c_char, 9 as size_t) == 0 as c_int {
        arg = arg.offset(9 as c_int as isize);
    }
    set_option_value_give_err(
        kOptFiletype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(arg),
            },
        },
        OPT_LOCAL as c_int,
    );
    if arg != (*eap).arg {
        (*curbuf.get()).b_did_filetype = false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn ex_checkhealth(mut eap: *mut exarg_T) {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 2];
    args.capacity = 2 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let mut mods: [c_char; 1024] = [0; 1024];
    let mut mods_len: size_t = 0 as size_t;
    mods[0 as c_int as usize] = NUL as c_char;
    if (*cmdmod.ptr()).cmod_tab > 0 as c_int || (*cmdmod.ptr()).cmod_split != 0 as c_int {
        let mut multi_mods: bool = false_0 != 0;
        mods_len = add_win_cmd_modifiers(
            &raw mut mods as *mut c_char,
            cmdmod.ptr(),
            &raw mut multi_mods,
        );
        '_c2rust_label: {
            if mods_len < ::core::mem::size_of::<[c_char; 1024]>() {
            } else {
                __assert_fail(
                    b"mods_len < sizeof(mods)\0".as_ptr() as *const c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                    8263 as c_uint,
                    b"void ex_checkhealth(exarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
    }
    let c2rust_fresh23 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh23 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_14 {
            string: String_0 {
                data: &raw mut mods as *mut c_char,
                size: mods_len,
            },
        },
    };
    let c2rust_fresh24 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh24 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_14 {
            string: cstr_as_string((*eap).arg),
        },
    };
    nlua_exec(
        String_0 {
            data: b"vim.health._check(...)\0".as_ptr() as *const c_char as *mut c_char,
            size: ::core::mem::size_of::<[c_char; 23]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<c_char>(),
        args,
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if !(err.type_0 as c_int != kErrorTypeNone as c_int) {
        return;
    }
    let mut vimruntime_env: *mut c_char =
        os_getenv_noalloc(b"VIMRUNTIME\0".as_ptr() as *const c_char);
    if vimruntime_env.is_null() {
        emsg(gettext(
            b"E5009: $VIMRUNTIME is empty or unset\0".as_ptr() as *const c_char
        ));
    } else {
        let mut rtp_ok: bool = !strstr(p_rtp.get(), vimruntime_env).is_null();
        if rtp_ok {
            semsg(
                gettext(b"E5009: Invalid $VIMRUNTIME: %s\0".as_ptr() as *const c_char),
                vimruntime_env,
            );
        } else {
            emsg(gettext(
                b"E5009: Invalid 'runtimepath'\0".as_ptr() as *const c_char
            ));
        }
    }
    semsg_multiline(b"emsg\0".as_ptr() as *const c_char, err.msg);
    api_clear_error(&raw mut err);
}
