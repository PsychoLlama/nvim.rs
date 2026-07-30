//! `:restart`, `:detach` and `:connect` — the commands that hand
//! the session to another process or take it back.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ex_restart(mut eap: *mut exarg_T) {
    let mut servername_args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut servername_args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    let mut result: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    };
    let mut listen_addr: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut quit_cmd: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut quit_cmd_copy: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut result_mem: ArenaMem = ::core::ptr::null_mut::<consumed_blk>();
    let mut detach_args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut detach_args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    let mut chanclose_expr_args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut chanclose_expr_args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    let no_ui: bool = ui_active() == 0;
    let mut exepath: *const c_char = get_vim_var_str(VV_PROGPATH);
    let mut l: *const list_T = get_vim_var_list(VV_ARGV);
    let mut argc: c_int = tv_list_len(l);
    let mut argv: *mut *mut c_char = xcalloc(
        (argc as size_t).wrapping_add(3 as size_t),
        ::core::mem::size_of::<*mut c_char>(),
    ) as *mut *mut c_char;
    let mut i: size_t = 0 as size_t;
    let mut listen_arg: *const c_char = ::core::ptr::null::<c_char>();
    let mut li: *const listitem_T = (*l).lv_first;
    while !li.is_null() {
        let mut arg: *const c_char = tv_get_string(&raw const (*li).li_tv);
        if i > 0 as size_t && strequal(arg, b"--\0".as_ptr() as *const c_char) as c_int != 0 {
            break;
        }
        if i > 0 as size_t && strequal(arg, b"-s\0".as_ptr() as *const c_char) as c_int != 0 {
            li = (*li).li_next;
        } else {
            if i > 0 as size_t
                && strequal(arg, b"--listen\0".as_ptr() as *const c_char) as c_int != 0
            {
                let mut next_li: *const listitem_T = (*li).li_next;
                if !next_li.is_null() {
                    let mut addr: *const c_char = tv_get_string(&raw const (*next_li).li_tv);
                    if !strstr(addr, b":\0".as_ptr() as *const c_char).is_null()
                        || !strstr(addr, b"/\0".as_ptr() as *const c_char).is_null()
                        || !strstr(addr, b"\\\0".as_ptr() as *const c_char).is_null()
                    {
                        listen_arg = addr;
                    }
                }
            }
            if i == 0 as size_t
                || !strequal(arg, b"--embed\0".as_ptr() as *const c_char)
                    && !strequal(arg, b"--headless\0".as_ptr() as *const c_char)
                    && !strequal(arg, b"-\0".as_ptr() as *const c_char)
            {
                let c2rust_fresh4 = i;
                i = i.wrapping_add(1);
                let c2rust_lvalue_ptr = &raw mut *argv.offset(c2rust_fresh4 as isize);
                *c2rust_lvalue_ptr = xstrdup(arg);
                if i == 1 as size_t {
                    let c2rust_fresh5 = i;
                    i = i.wrapping_add(1);
                    let c2rust_lvalue_ptr_0 = &raw mut *argv.offset(c2rust_fresh5 as isize);
                    *c2rust_lvalue_ptr_0 = xstrdup(b"--embed\0".as_ptr() as *const c_char);
                    if no_ui {
                        let c2rust_fresh6 = i;
                        i = i.wrapping_add(1);
                        let c2rust_lvalue_ptr_1 = &raw mut *argv.offset(c2rust_fresh6 as isize);
                        *c2rust_lvalue_ptr_1 = xstrdup(b"--headless\0".as_ptr() as *const c_char);
                    }
                }
            }
        }
        li = (*li).li_next;
    }
    let mut server_stopped: bool = if !listen_arg.is_null() {
        server_stop(listen_arg, true_0 != 0) as c_int
    } else {
        false_0
    } != 0;
    let mut on_err: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_20 {
                funcref: ::core::ptr::null_mut::<c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<c_char>(),
    };
    on_err.fwd_err = true_0 != 0;
    let mut detach: bool = true_0 != 0;
    let mut exit_status: varnumber_T = 0;
    let mut channel: *mut Channel = channel_job_start(
        argv,
        exepath,
        CallbackReader {
            cb: Callback {
                data: C2Rust_Unnamed_20 {
                    funcref: ::core::ptr::null_mut::<c_char>(),
                },
                type_0: kCallbackNone,
            },
            self_0: ::core::ptr::null_mut::<dict_T>(),
            buffer: GA_EMPTY_INIT_VALUE,
            eof: false,
            buffered: false_0 != 0,
            fwd_err: false_0 != 0,
            type_0: ::core::ptr::null::<c_char>(),
        },
        on_err,
        Callback {
            data: C2Rust_Unnamed_20 {
                funcref: ::core::ptr::null_mut::<c_char>(),
            },
            type_0: kCallbackNone,
        },
        false_0 != 0,
        true_0 != 0,
        true_0 != 0,
        detach,
        kChannelStdinPipe,
        ::core::ptr::null::<c_char>(),
        0 as uint16_t,
        0 as uint16_t,
        ::core::ptr::null_mut::<dict_T>(),
        &raw mut exit_status,
    );
    if channel.is_null() {
        emsg(b"cannot create a channel job\0".as_ptr() as *const c_char);
    } else {
        result_mem = ::core::ptr::null_mut::<consumed_blk>();
        detach_args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        detach_args__items = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_14 { boolean: false },
        }; 1];
        detach_args.capacity = 1 as size_t;
        detach_args.items = &raw mut detach_args__items as *mut Object;
        let c2rust_fresh7 = detach_args.size;
        detach_args.size = detach_args.size.wrapping_add(1);
        *detach_args.items.offset(c2rust_fresh7 as isize) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed_14 { boolean: true },
        };
        rpc_send_call(
            (*channel).id,
            b"nvim__chan_set_detach\0".as_ptr() as *const c_char,
            detach_args,
            &raw mut result_mem,
            &raw mut err,
        );
        '_fail_2: {
            if err.type_0 as c_int == kErrorTypeNone as c_int {
                arena_mem_free(result_mem);
                result_mem = ::core::ptr::null_mut::<consumed_blk>();
                if *(*eap).arg as c_int != NUL {
                    let mut autocmd_opts: Dict = Dict {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<KeyValuePair>(),
                    };
                    let mut autocmd_opts__items: [KeyValuePair; 3] = [KeyValuePair {
                        key: String_0 {
                            data: ::core::ptr::null_mut::<c_char>(),
                            size: 0,
                        },
                        value: Object {
                            type_0: kObjectTypeNil,
                            data: C2Rust_Unnamed_14 { boolean: false },
                        },
                    }; 3];
                    autocmd_opts.capacity = 3 as size_t;
                    autocmd_opts.items = &raw mut autocmd_opts__items as *mut KeyValuePair;
                    let c2rust_fresh8 = autocmd_opts.size;
                    autocmd_opts.size = autocmd_opts.size.wrapping_add(1);
                    *autocmd_opts.items.offset(c2rust_fresh8 as isize) = key_value_pair {
                        key: cstr_as_string(b"once\0".as_ptr() as *const c_char),
                        value: object {
                            type_0: kObjectTypeBoolean,
                            data: C2Rust_Unnamed_14 { boolean: true },
                        },
                    };
                    let c2rust_fresh9 = autocmd_opts.size;
                    autocmd_opts.size = autocmd_opts.size.wrapping_add(1);
                    *autocmd_opts.items.offset(c2rust_fresh9 as isize) = key_value_pair {
                        key: cstr_as_string(b"nested\0".as_ptr() as *const c_char),
                        value: object {
                            type_0: kObjectTypeBoolean,
                            data: C2Rust_Unnamed_14 { boolean: true },
                        },
                    };
                    let c2rust_fresh10 = autocmd_opts.size;
                    autocmd_opts.size = autocmd_opts.size.wrapping_add(1);
                    *autocmd_opts.items.offset(c2rust_fresh10 as isize) = key_value_pair {
                        key: cstr_as_string(b"command\0".as_ptr() as *const c_char),
                        value: object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed_14 {
                                string: cstr_as_string((*eap).arg),
                            },
                        },
                    };
                    let mut autocmd_args: Array = Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    };
                    let mut autocmd_args__items: [Object; 2] = [Object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed_14 { boolean: false },
                    }; 2];
                    autocmd_args.capacity = 2 as size_t;
                    autocmd_args.items = &raw mut autocmd_args__items as *mut Object;
                    let c2rust_fresh11 = autocmd_args.size;
                    autocmd_args.size = autocmd_args.size.wrapping_add(1);
                    *autocmd_args.items.offset(c2rust_fresh11 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_14 {
                            string: cstr_as_string(b"UIEnter\0".as_ptr() as *const c_char),
                        },
                    };
                    let c2rust_fresh12 = autocmd_args.size;
                    autocmd_args.size = autocmd_args.size.wrapping_add(1);
                    *autocmd_args.items.offset(c2rust_fresh12 as isize) = object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed_14 { dict: autocmd_opts },
                    };
                    rpc_send_call(
                        (*channel).id,
                        b"nvim_create_autocmd\0".as_ptr() as *const c_char,
                        autocmd_args,
                        &raw mut result_mem,
                        &raw mut err,
                    );
                    if err.type_0 as c_int != kErrorTypeNone as c_int {
                        break '_fail_2;
                    } else {
                        arena_mem_free(result_mem);
                        result_mem = ::core::ptr::null_mut::<consumed_blk>();
                    }
                }
                servername_args = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                servername_args__items = [Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed_14 { boolean: false },
                }; 1];
                servername_args.capacity = 1 as size_t;
                servername_args.items = &raw mut servername_args__items as *mut Object;
                let c2rust_fresh13 = servername_args.size;
                servername_args.size = servername_args.size.wrapping_add(1);
                *servername_args.items.offset(c2rust_fresh13 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(b"servername\0".as_ptr() as *const c_char),
                    },
                };
                result = rpc_send_call(
                    (*channel).id,
                    b"nvim_get_vvar\0".as_ptr() as *const c_char,
                    servername_args,
                    &raw mut result_mem,
                    &raw mut err,
                );
                if err.type_0 as c_int == kErrorTypeNone as c_int {
                    if result.type_0 as c_uint != kObjectTypeString as c_int as c_uint
                        || result.data.string.size == 0 as size_t
                    {
                        emsg(
                            b"restart failed: could not get listen address from new server\0"
                                .as_ptr() as *const c_char,
                        );
                    } else {
                        listen_addr = xmemdupz(
                            result.data.string.data as *const c_void,
                            result.data.string.size,
                        ) as *mut c_char;
                        arena_mem_free(result_mem);
                        result_mem = ::core::ptr::null_mut::<consumed_blk>();
                        ui_call_restart(cstr_as_string(listen_addr));
                        ui_flush();
                        xfree(listen_addr as *mut c_void);
                        set_vim_var_string(
                            VV_EXITREASON,
                            b"restart\0".as_ptr() as *const c_char,
                            ::core::mem::size_of::<[c_char; 8]>().wrapping_sub(1 as usize)
                                as ptrdiff_t,
                        );
                        quit_cmd = (if !(*eap).do_ecmd_cmd.is_null() {
                            (*eap).do_ecmd_cmd as *const c_char
                        } else {
                            b"qall\0".as_ptr() as *const c_char
                        }) as *mut c_char;
                        quit_cmd_copy = ::core::ptr::null_mut::<c_char>();
                        if (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as c_int != 0 {
                            quit_cmd_copy =
                                concat_str(b"confirm \0".as_ptr() as *const c_char, quit_cmd);
                            quit_cmd = quit_cmd_copy;
                        }
                        nvim_command(cstr_as_string(quit_cmd), &raw mut err);
                        xfree(quit_cmd_copy as *mut c_void);
                        if err.type_0 as c_int != kErrorTypeNone as c_int {
                            emsg(err.msg);
                            api_clear_error(&raw mut err);
                        } else if !exiting.get() {
                            emsg(b"restart failed: +cmd did not quit the server\0".as_ptr()
                                as *const c_char);
                        }
                    }
                }
            }
        }
        set_vim_var_string(
            VV_EXITREASON,
            ::core::ptr::null::<c_char>(),
            -1 as ptrdiff_t,
        );
        if err.type_0 as c_int != kErrorTypeNone as c_int {
            emsg(err.msg);
            api_clear_error(&raw mut err);
        }
        arena_mem_free(result_mem);
        result_mem = ::core::ptr::null_mut::<consumed_blk>();
        chanclose_expr_args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        chanclose_expr_args__items = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_14 { boolean: false },
        }; 1];
        chanclose_expr_args.capacity = 1 as size_t;
        chanclose_expr_args.items = &raw mut chanclose_expr_args__items as *mut Object;
        let c2rust_fresh14 = chanclose_expr_args.size;
        chanclose_expr_args.size = chanclose_expr_args.size.wrapping_add(1);
        *chanclose_expr_args.items.offset(c2rust_fresh14 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_14 {
                string: cstr_as_string(b"chanclose(v:stderr)\0".as_ptr() as *const c_char),
            },
        };
        rpc_send_call(
            (*channel).id,
            b"nvim_eval\0".as_ptr() as *const c_char,
            chanclose_expr_args,
            &raw mut result_mem,
            &raw mut err,
        );
        api_clear_error(&raw mut err);
        arena_mem_free(result_mem);
        proc_stop(channel_proc(channel));
        if proc_wait(
            channel_proc(channel),
            -1 as c_int,
            ::core::ptr::null_mut::<MultiQueue>(),
        ) < 0 as c_int
        {
            emsg(b"killing new nvim server failed\0".as_ptr() as *const c_char);
        }
    }
    if server_stopped as c_int != 0 && server_start(listen_arg) != 0 as c_int {
        semsg(
            b"couldn't resume listening on %s\0".as_ptr() as *const c_char,
            listen_arg,
        );
    }
}

pub(crate) unsafe extern "C" fn ex_detach(mut eap: *mut exarg_T) {
    if !eap.is_null() && (*eap).forceit != 0 {
        emsg(b"bang (!) not supported yet\0".as_ptr() as *const c_char);
    } else {
        if current_ui.get() == 0 {
            emsg(b"UI not attached\0".as_ptr() as *const c_char);
            return;
        }
        let mut chan: *mut Channel = find_channel(current_ui.get());
        if chan.is_null() {
            emsg(&raw const e_invchan as *const c_char);
            return;
        }
        let mut detach_err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<c_char>(),
        };
        nvim__chan_set_detach((*chan).id, true_0 != 0, &raw mut detach_err);
        api_clear_error(&raw mut detach_err);
        let mut err2: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<c_char>(),
        };
        remote_ui_disconnect((*chan).id, &raw mut err2, true_0 != 0);
        if err2.type_0 as c_int != kErrorTypeNone as c_int {
            emsg(err2.msg);
            api_clear_error(&raw mut err2);
            return;
        }
        let mut err: *const c_char = ::core::ptr::null::<c_char>();
        let mut rv: bool = channel_close((*chan).id, kChannelPartAll, &raw mut err);
        if !rv && !err.is_null() {
            emsg(err);
            return;
        }
        logmsg(
            LOGLVL_INF,
            ::core::ptr::null::<c_char>(),
            b"ex_detach\0".as_ptr() as *const c_char,
            6019 as c_int,
            true_0 != 0,
            b"detach current_ui=%ld\0".as_ptr() as *const c_char,
            (*chan).id,
        );
    };
}

pub(crate) unsafe extern "C" fn ex_connect(mut eap: *mut exarg_T) {
    let mut stop_server: bool = if (*eap).forceit != 0 {
        (ui_active() == 1 as size_t) as c_int
    } else {
        false_0
    } != 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    remote_ui_connect(current_ui.get(), (*eap).arg, &raw mut err);
    if err.type_0 as c_int != kErrorTypeNone as c_int {
        emsg(err.msg);
        api_clear_error(&raw mut err);
        return;
    }
    ex_detach(::core::ptr::null_mut::<exarg_T>());
    if stop_server {
        exiting.set(true_0 != 0);
        getout(0 as c_int);
    }
}
