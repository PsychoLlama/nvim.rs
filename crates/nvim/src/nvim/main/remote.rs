//! `--server` and the `--remote` family: connect to another Nvim
//! and hand it the rest of the command line.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn server_connect(
    mut server_addr: *mut c_char,
    mut errmsg: *mut *const c_char,
) -> uint64_t {
    if server_addr.is_null() {
        *errmsg = b"no address specified\0".as_ptr() as *const c_char;
        return 0 as uint64_t;
    }
    let mut on_data: CallbackReader = CallbackReader {
        cb: Callback {
            data: Callback_data {
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
    let mut error: *const c_char = ::core::ptr::null::<c_char>();
    let mut is_tcp: bool = socket_address_is_tcp(CStr::from_ptr(server_addr));
    let mut chan: uint64_t = channel_connect(
        is_tcp,
        server_addr,
        true_0 != 0,
        on_data,
        500 as c_int,
        &raw mut error,
    );
    if !error.is_null() {
        *errmsg = error;
        return 0 as uint64_t;
    }
    return chan;
}

pub(crate) unsafe extern "C" fn remote_request(
    mut params: *mut mparm_T,
    mut remote_args: c_int,
    mut server_addr: *mut c_char,
    mut argc: c_int,
    mut argv: *mut *mut c_char,
    mut ui_only: bool,
) {
    let mut is_ui: bool = strequal(
        *argv.offset(remote_args as isize),
        b"--remote-ui\0".as_ptr() as *const c_char,
    );
    if ui_only as c_int != 0 && !is_ui {
        return;
    }
    let mut connect_error: *const c_char = ::core::ptr::null::<c_char>();
    let mut chan: uint64_t = server_connect(server_addr, &raw mut connect_error);
    let mut rvobj: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if is_ui {
        if chan == 0 {
            fprintf(
                stderr,
                b"Remote ui failed to start: %s\n\0".as_ptr() as *const c_char,
                connect_error,
            );
            os_exit(1 as c_int);
        } else if strequal(
            server_addr,
            os_getenv_noalloc(b"NVIM\0".as_ptr() as *const c_char),
        ) {
            fprintf(
                stderr,
                b"%s\0".as_ptr() as *const c_char,
                b"Cannot attach UI of :terminal child to its parent. \0".as_ptr() as *const c_char,
            );
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const c_char,
                b"(Unset $NVIM to skip this check)\0".as_ptr() as *const c_char,
            );
            os_exit(1 as c_int);
        }
        ui_client_channel_id.set(chan);
        return;
    }
    let mut args: Array = ARRAY_DICT_INIT;
    args.capacity = (argc - remote_args) as size_t;
    args.items = xrealloc(
        args.items as *mut c_void,
        ::core::mem::size_of::<Object>().wrapping_mul(args.capacity),
    ) as *mut Object;
    let mut t_argc: c_int = remote_args;
    while t_argc < argc {
        let c2rust_fresh1 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(*argv.offset(t_argc as isize)),
            },
        };
        t_argc += 1;
    }
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<c_char>(),
    };
    let mut a: Array = ARRAY_DICT_INIT;
    let mut a__items: [Object; 4] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 4];
    a.capacity = 4 as size_t;
    a.items = &raw mut a__items as *mut Object;
    let c2rust_fresh2 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh2 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: chan as c_int as Integer,
        },
    };
    let c2rust_fresh3 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh3 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(server_addr),
        },
    };
    let c2rust_fresh4 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh4 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(connect_error),
        },
    };
    let c2rust_fresh5 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh5 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed { array: args },
    };
    let mut s: String_0 = String_0 {
        data: b"return vim._cs_remote(...)\0".as_ptr() as *const c_char as *mut c_char,
        size: ::core::mem::size_of::<[c_char; 27]>().wrapping_sub(1 as size_t),
    };
    let mut o: Object = nlua_exec(
        s,
        ::core::ptr::null::<c_char>(),
        a,
        kRetObject,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    xfree(args.items as *mut c_void);
    args.capacity = 0 as size_t;
    args.size = args.capacity;
    args.items = ::core::ptr::null_mut::<Object>();
    if err.type_0 as c_int != kErrorTypeNone as c_int {
        fprintf(stderr, b"%s\n\0".as_ptr() as *const c_char, err.msg);
        os_exit(2 as c_int);
    }
    if o.type_0 as c_uint == kObjectTypeDict as c_int as c_uint {
        rvobj.data.dict = o.data.dict;
    } else {
        fprintf(
            stderr,
            b"vim._cs_remote returned unexpected value\n\0".as_ptr() as *const c_char,
        );
        os_exit(2 as c_int);
    }
    let mut should_exit: TriState = kNone;
    let mut tabbed: TriState = kNone;
    let mut i: size_t = 0 as size_t;
    while i < rvobj.data.dict.size {
        if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"errmsg\0".as_ptr() as *const c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as c_uint
                != kObjectTypeString as c_int as c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'errmsg'\n\0".as_ptr()
                        as *const c_char,
                );
                os_exit(2 as c_int);
            }
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const c_char,
                (*rvobj.data.dict.items.offset(i as isize))
                    .value
                    .data
                    .string
                    .data,
            );
            os_exit(2 as c_int);
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"result\0".as_ptr() as *const c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as c_uint
                != kObjectTypeString as c_int as c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'result'\n\0".as_ptr()
                        as *const c_char,
                );
                os_exit(2 as c_int);
            }
            printf(
                b"%s\0".as_ptr() as *const c_char,
                (*rvobj.data.dict.items.offset(i as isize))
                    .value
                    .data
                    .string
                    .data,
            );
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"tabbed\0".as_ptr() as *const c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as c_uint
                != kObjectTypeBoolean as c_int as c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'tabbed'\n\0".as_ptr()
                        as *const c_char,
                );
                os_exit(2 as c_int);
            }
            tabbed = (if (*rvobj.data.dict.items.offset(i as isize))
                .value
                .data
                .boolean as c_int
                != 0
            {
                kTrue as c_int
            } else {
                kFalse as c_int
            }) as TriState;
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"should_exit\0".as_ptr() as *const c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as c_uint
                != kObjectTypeBoolean as c_int as c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'should_exit'\n\0".as_ptr()
                        as *const c_char,
                );
                os_exit(2 as c_int);
            }
            should_exit = (if (*rvobj.data.dict.items.offset(i as isize))
                .value
                .data
                .boolean as c_int
                != 0
            {
                kTrue as c_int
            } else {
                kFalse as c_int
            }) as TriState;
        }
        i = i.wrapping_add(1);
    }
    if should_exit as c_int == kNone as c_int || tabbed as c_int == kNone as c_int {
        fprintf(
            stderr,
            b"vim._cs_remote didn't return a value for should_exit or tabbed, bailing\n\0".as_ptr()
                as *const c_char,
        );
        os_exit(2 as c_int);
    }
    api_free_object(o);
    if should_exit as c_int == kTrue as c_int {
        os_exit(0 as c_int);
    }
    if tabbed as c_int == kTrue as c_int {
        (*params).window_count = argc - remote_args - 1 as c_int;
        (*params).window_layout = WIN_TABS as c_int;
    }
}
