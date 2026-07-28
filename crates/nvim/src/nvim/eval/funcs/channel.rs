//! Channels and servers: `chansend()`, `rpcrequest()`, `serverstart()`
//! and the rest of the RPC and socket surface.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_chanclose(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut part: ChannelPart = kChannelPartAll;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut stream: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
        if strcmp(stream, b"stdin\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
            part = kChannelPartStdin;
        } else if strcmp(stream, b"stdout\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
            part = kChannelPartStdout;
        } else if strcmp(stream, b"stderr\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
            part = kChannelPartStderr;
        } else if strcmp(stream, b"rpc\0".as_ptr() as *const ::core::ffi::c_char) == 0 {
            part = kChannelPartRpc;
        } else {
            semsg(
                gettext(b"Invalid channel stream \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                stream,
            );
            return;
        }
    }
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    (*rettv).vval.v_number = channel_close(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint64_t,
        part,
        &raw mut error,
    ) as varnumber_T;
    if (*rettv).vval.v_number == 0 {
        emsg(error);
    }
}
pub unsafe extern "C" fn f_chansend(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut input_len: ptrdiff_t = 0 as ptrdiff_t;
    let mut input: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: uint64_t = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_number as uint64_t;
    let mut crlf: bool = false_0 != 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let b: *const blob_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_blob;
        input_len = tv_blob_len(b) as ptrdiff_t;
        if input_len > 0 as ptrdiff_t {
            input = xmemdup((*b).bv_ga.ga_data, input_len as size_t) as *mut ::core::ffi::c_char;
        }
    } else {
        input = save_tv_as_string(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut input_len,
            false_0 != 0,
            crlf,
        );
    }
    if input.is_null() {
        return;
    }
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    (*rettv).vval.v_number =
        channel_send(id, input, input_len as size_t, true_0 != 0, &raw mut error) as varnumber_T;
    if !error.is_null() {
        emsg(error);
    }
}
pub unsafe extern "C" fn f_rpcnotify(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number
            < 0 as varnumber_T
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"Channel id must be a positive integer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"Event type must be a string\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 20] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_16 { boolean: false },
    }; 20];
    args.capacity = MAX_FUNC_ARGS as ::core::ffi::c_int as size_t;
    args.items = &raw mut args__items as *mut Object;
    let mut arena: Arena = ARENA_EMPTY;
    let mut tv: *mut typval_T = argvars.offset(2 as ::core::ffi::c_int as isize);
    while (*tv).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let c2rust_fresh5 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh5 as isize) = vim_to_object(tv, &raw mut arena, true);
        tv = tv.offset(1);
    }
    let mut ok: bool = rpc_send_event(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as uint64_t,
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
        args,
    );
    arena_mem_free(arena_finish(&raw mut arena));
    if !ok {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"Channel doesn't exist\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    (*rettv).vval.v_number = 1 as varnumber_T;
}
pub unsafe extern "C" fn f_rpcrequest(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    let l_provider_call_nesting: ::core::ffi::c_int = provider_call_nesting.get();
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number
            <= 0 as varnumber_T
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"Channel id must be a positive integer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"Method name must be a string\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 20] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_16 { boolean: false },
    }; 20];
    args.capacity = MAX_FUNC_ARGS as ::core::ffi::c_int as size_t;
    args.items = &raw mut args__items as *mut Object;
    let mut arena: Arena = ARENA_EMPTY;
    let mut tv: *mut typval_T = argvars.offset(2 as ::core::ffi::c_int as isize);
    while (*tv).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let c2rust_fresh3 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh3 as isize) = vim_to_object(tv, &raw mut arena, true);
        tv = tv.offset(1);
    }
    let mut save_current_sctx: sctx_T = sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    let mut save_autocmd_fname: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_autocmd_match: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_autocmd_fname_full: bool = false;
    let mut save_autocmd_bufnr: ::core::ffi::c_int = 0;
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    if l_provider_call_nesting != 0 {
        save_current_sctx = current_sctx.get();
        save_autocmd_fname = autocmd_fname.get();
        save_autocmd_match = autocmd_match.get();
        save_autocmd_fname_full = autocmd_fname_full.get();
        save_autocmd_bufnr = autocmd_bufnr.get();
        save_funccal(&raw mut funccal_entry);
        current_sctx.set((*provider_caller_scope.ptr()).script_ctx);
        ga_grow(exestack.ptr(), 1 as ::core::ffi::c_int);
        let c2rust_fresh4 = (*exestack.ptr()).ga_len;
        (*exestack.ptr()).ga_len = (*exestack.ptr()).ga_len + 1;
        *((*exestack.ptr()).ga_data as *mut estack_T).offset(c2rust_fresh4 as isize) =
            (*provider_caller_scope.ptr()).es_entry;
        autocmd_fname.set((*provider_caller_scope.ptr()).autocmd_fname);
        autocmd_match.set((*provider_caller_scope.ptr()).autocmd_match);
        autocmd_fname_full.set((*provider_caller_scope.ptr()).autocmd_fname_full);
        autocmd_bufnr.set((*provider_caller_scope.ptr()).autocmd_bufnr);
        set_current_funccal((*provider_caller_scope.ptr()).funccalp as *mut funccall_T);
    }
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut chan_id: uint64_t = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_number as uint64_t;
    let mut method: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut res_mem: ArenaMem = ::core::ptr::null_mut::<consumed_blk>();
    let mut result: Object = rpc_send_call(chan_id, method, args, &raw mut res_mem, &raw mut err);
    arena_mem_free(arena_finish(&raw mut arena));
    if l_provider_call_nesting != 0 {
        current_sctx.set(save_current_sctx);
        (*exestack.ptr()).ga_len -= 1;
        autocmd_fname.set(save_autocmd_fname);
        autocmd_match.set(save_autocmd_match);
        autocmd_fname_full.set(save_autocmd_fname_full);
        autocmd_bufnr.set(save_autocmd_bufnr);
        restore_funccal();
    }
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut chan: *mut Channel = find_channel(chan_id);
        if !chan.is_null() {
            name = get_client_info(chan, b"name\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if !name.is_null() {
            semsg_multiline(
                b"rpc_error\0".as_ptr() as *const ::core::ffi::c_char,
                b"Invoking '%s' on channel %lu (%s):\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                method,
                chan_id,
                name,
                err.msg,
            );
        } else {
            semsg_multiline(
                b"rpc_error\0".as_ptr() as *const ::core::ffi::c_char,
                b"Invoking '%s' on channel %lu:\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                method,
                chan_id,
                err.msg,
            );
        }
    } else {
        object_to_vim(result, rettv, &raw mut err);
    }
    arena_mem_free(res_mem);
    api_clear_error(&raw mut err);
}
pub unsafe extern "C" fn f_serverlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_16 { boolean: false },
    }; 1];
    let mut err: Error = Error {
        type_0: kErrorTypeException,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut rv: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_16 { boolean: false },
    };
    let mut n: size_t = 0;
    let mut addrs: *mut *mut ::core::ffi::c_char = server_address_list(&raw mut n);
    let mut arena: Arena = ARENA_EMPTY;
    let mut addrs_arr: Array = arena_array(&raw mut arena, n);
    let l: *mut list_T = tv_list_alloc_ret(rettv, n as ptrdiff_t);
    let mut i: size_t = 0 as size_t;
    while i < n {
        tv_list_append_allocated_string(l, *addrs.offset(i as isize));
        let c2rust_fresh1 = addrs_arr.size;
        addrs_arr.size = addrs_arr.size.wrapping_add(1);
        *addrs_arr.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_16 {
                string: cstr_as_string(*addrs.offset(i as isize)),
            },
        };
        i = i.wrapping_add(1);
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_dict_get_bool(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"peer\0".as_ptr() as *const ::core::ffi::c_char,
            false_0,
        ) != 0
    {
        args = ARRAY_DICT_INIT;
        args__items = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_16 { boolean: false },
        }; 1];
        args.capacity = 1 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh2 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh2 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_16 { array: addrs_arr },
        };
        err = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        rv = nlua_exec(
            String_0 {
                data: b"return require('vim._core.server').serverlist(...)\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 51]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetObject,
            &raw mut arena,
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"f_serverlist\0".as_ptr() as *const ::core::ffi::c_char,
                6338 as ::core::ffi::c_int,
                true_0 != 0,
                b"vim._core.serverlist failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
                err.msg,
            );
        } else {
            let mut i_0: size_t = 0 as size_t;
            while i_0 < rv.data.array.size {
                let mut curr_server: *mut ::core::ffi::c_char =
                    (*rv.data.array.items.offset(i_0 as isize)).data.string.data;
                tv_list_append_string(l, curr_server, -1 as ssize_t);
                i_0 = i_0.wrapping_add(1);
            }
        }
    }
    xfree(addrs as *mut ::core::ffi::c_void);
    arena_mem_free(arena_finish(&raw mut arena));
}
pub unsafe extern "C" fn f_serverstart(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if check_secure() {
        return;
    }
    let mut address: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        address = xstrdup(tv_get_string(argvars));
    } else {
        address = server_address_new(::core::ptr::null::<::core::ffi::c_char>());
    }
    let mut result: ::core::ffi::c_int = server_start(address);
    xfree(address as *mut ::core::ffi::c_void);
    if result != 0 as ::core::ffi::c_int {
        semsg(
            b"Failed to start server: %s\0".as_ptr() as *const ::core::ffi::c_char,
            if result > 0 as ::core::ffi::c_int {
                b"Unknown system error\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                uv_strerror(result)
            },
        );
        return;
    }
    let mut n: size_t = 0;
    let mut addrs: *mut *mut ::core::ffi::c_char = server_address_list(&raw mut n);
    (*rettv).vval.v_string = *addrs.offset(n.wrapping_sub(1 as size_t) as isize);
    n = n.wrapping_sub(1);
    let mut i: size_t = 0 as size_t;
    while i < n {
        xfree(*addrs.offset(i as isize) as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    xfree(addrs as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_serverstop(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if !(*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_string
        .is_null()
    {
        let mut rv: bool = server_stop(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_string,
            false_0 != 0,
        );
        (*rettv).vval.v_number = (if rv as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as varnumber_T;
    }
}
pub unsafe extern "C" fn f_sockconnect(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected dictionary\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut mode: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut address: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut tcp: bool = false;
    if strcmp(mode, b"tcp\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        tcp = true_0 != 0;
    } else if strcmp(mode, b"pipe\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        tcp = false_0 != 0;
    } else {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"invalid mode\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut rpc: bool = false_0 != 0;
    let mut on_data: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_22 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut opts: *mut dict_T = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        rpc = tv_dict_get_number(opts, b"rpc\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T;
        if !tv_dict_get_callback(
            opts,
            b"on_data\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
            &raw mut on_data.cb,
        ) {
            return;
        }
        on_data.buffered = tv_dict_get_number(
            opts,
            b"data_buffered\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0;
        if on_data.buffered as ::core::ffi::c_int != 0
            && on_data.cb.type_0 as ::core::ffi::c_uint
                == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            on_data.self_0 = opts;
        }
    }
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: uint64_t = channel_connect(
        tcp,
        address,
        rpc,
        on_data,
        50 as ::core::ffi::c_int,
        &raw mut error,
    );
    if !error.is_null() {
        semsg(
            gettext(b"connection failed: %s\0".as_ptr() as *const ::core::ffi::c_char),
            error,
        );
    }
    (*rettv).vval.v_number = id as varnumber_T;
    (*rettv).v_type = VAR_NUMBER;
}
pub unsafe extern "C" fn f_stdioopen(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut on_stdin: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_22 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut opts: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    let mut rpc: bool = tv_dict_get_number(opts, b"rpc\0".as_ptr() as *const ::core::ffi::c_char)
        != 0 as varnumber_T;
    if !tv_dict_get_callback(
        opts,
        b"on_stdin\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
        &raw mut on_stdin.cb,
    ) {
        return;
    }
    if !tv_dict_get_callback(
        opts,
        b"on_print\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
        on_print.ptr(),
    ) {
        return;
    }
    on_stdin.buffered = tv_dict_get_number(
        opts,
        b"stdin_buffered\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0;
    if on_stdin.buffered as ::core::ffi::c_int != 0
        && on_stdin.cb.type_0 as ::core::ffi::c_uint
            == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        on_stdin.self_0 = opts;
    }
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: uint64_t = channel_from_stdio(rpc, on_stdin, &raw mut error);
    if id == 0 {
        semsg(&raw const e_stdiochan2 as *const ::core::ffi::c_char, error);
    }
    (*rettv).vval.v_number = id as varnumber_T;
    (*rettv).v_type = VAR_NUMBER;
}
