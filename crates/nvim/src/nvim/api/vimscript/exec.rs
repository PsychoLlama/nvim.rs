//! Running Vimscript: `nvim_exec2()` and `nvim_command()`.
//!
//! `exec_impl` is the shared body -- it sources the string as an anonymous
//! script with `do_source_str`, optionally capturing the messages it prints so
//! `opts.output` can hand them back -- and the two entry points differ only in
//! whether they take that option.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_exec2(
    mut channel_id: uint64_t,
    mut src: String_0,
    mut opts: *mut KeyDict_exec_opts,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        let mut result: Dict = ARRAY_DICT_INIT;
        let mut output: String_0 = exec_impl(channel_id, src, opts, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return result;
        }
        if (*opts).output {
            if result.size == result.capacity {
                result.capacity = if result.capacity != 0 {
                    result.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                result.items = xrealloc(
                    result.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<KeyValuePair>().wrapping_mul(result.capacity),
                ) as *mut KeyValuePair;
            } else {
            };
            let c2rust_fresh0 = result.size;
            result.size = result.size.wrapping_add(1);
            *result.items.offset(c2rust_fresh0 as isize) = key_value_pair {
                key: cstr_to_string(c"output".as_ptr()),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed { string: output },
                },
            };
        }
        return result;
    }
}

pub unsafe extern "C" fn exec_impl(
    mut channel_id: uint64_t,
    mut src: String_0,
    mut opts: *mut KeyDict_exec_opts,
    mut err: *mut Error,
) -> String_0 {
    unsafe {
        let save_msg_silent: ::core::ffi::c_int = msg_silent.get();
        let save_redir_off: bool = redir_off.get();
        let save_capture_ga: *mut garray_T = capture_ga.get();
        let save_msg_col: ::core::ffi::c_int = msg_col.get();
        let mut capture_local: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
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
        do_source_str(
            src.data,
            c"nvim_exec2()".as_ptr() as *mut ::core::ffi::c_char,
        );
        if (*opts).output {
            capture_ga.set(save_capture_ga);
            msg_silent.set(save_msg_silent);
            redir_off.set(save_redir_off);
            msg_col.set(save_msg_col);
        }
        current_sctx.set(save_current_sctx);
        try_leave(&raw mut tstate, err);
        if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            if (*opts).output as ::core::ffi::c_int != 0
                && capture_local.ga_len > 1 as ::core::ffi::c_int
            {
                let mut s: String_0 = String_0 {
                    data: capture_local.ga_data as *mut ::core::ffi::c_char,
                    size: capture_local.ga_len as size_t,
                };
                if *s.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\n' as ::core::ffi::c_int
                {
                    memmove(
                        s.data as *mut ::core::ffi::c_void,
                        s.data.offset(1 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        s.size.wrapping_sub(1 as size_t),
                    );
                    *s.data.offset(s.size.wrapping_sub(1 as size_t) as isize) =
                        NUL as ::core::ffi::c_char;
                    s.size = s.size.wrapping_sub(1 as size_t);
                }
                return s;
            }
        }
        if (*opts).output {
            ga_clear(&raw mut capture_local);
        }
        return STRING_INIT;
    }
}

pub unsafe extern "C" fn nvim_command(mut cmd: String_0, mut err: *mut Error) {
    unsafe {
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
        do_cmdline_cmd(cmd.data);
        try_leave(&raw mut tstate, err);
    }
}
