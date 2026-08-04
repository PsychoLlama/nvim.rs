use crate::src::nvim::channel::{channel_close, channel_create_event, channel_job_start};
use crate::src::nvim::eval::find_job;
use crate::src::nvim::eval::funcs::{f_jobstart, f_jobstop};
use crate::src::nvim::eval::typval::{kCallbackNone, tv_list_len};
use crate::src::nvim::eval::typval::{
    tv_dict_add_bool, tv_dict_alloc, tv_dict_free, tv_get_string,
};
use crate::src::nvim::ex_cmds::check_secure;

use crate::src::nvim::main::{e_api_spawn_failed, e_invarg, e_invarg2, firstbuf};
use crate::src::nvim::memory::{xmalloc, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::types::channel::kChannelStdinPipe;
use crate::src::nvim::types::{
    BoolVarValue, Callback, Callback_data as C2Rust_Unnamed_19, CallbackReader, Channel,
    ChannelPart, EvalFuncData, ScopeType, SpecialVarValue, VarLockStatus, VarType, buf_T, dict_T,
    garray_T, list_T, listitem_T, size_t, typval_T, uint16_t, uint64_t, varnumber_T,
};
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kChannelPartRpc: ChannelPart = 3;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub unsafe extern "C" fn f_rpcstart(
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
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut args: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut argsl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        args = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        argsl = tv_list_len(args);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let l_: *const list_T = args;
        if !l_.is_null() {
            let mut arg: *const listitem_T = (*l_).lv_first;
            while !arg.is_null() {
                if (*arg).li_tv.v_type as ::core::ffi::c_uint
                    != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    semsg(
                        gettext(
                            b"E5010: List item %d of the second argument is not a string\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        i,
                    );
                    return;
                }
                i += 1;
                arg = (*arg).li_next;
            }
        }
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_string
        .is_null()
        || *(*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == NUL
    {
        emsg(gettext(
            &raw const e_api_spawn_failed as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut argvl: ::core::ffi::c_int = argsl + 2 as ::core::ffi::c_int;
    let mut argv: *mut *mut ::core::ffi::c_char =
        xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>().wrapping_mul(argvl as size_t))
            as *mut *mut ::core::ffi::c_char;
    *argv.offset(0 as ::core::ffi::c_int as isize) = xstrdup(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string,
    );
    let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if argsl > 0 as ::core::ffi::c_int {
        let l__0: *const list_T = args;
        if !l__0.is_null() {
            let mut arg_0: *const listitem_T = (*l__0).lv_first;
            while !arg_0.is_null() {
                let c2rust_fresh0 = i_0;
                i_0 = i_0 + 1;
                let c2rust_lvalue_ptr = &raw mut *argv.offset(c2rust_fresh0 as isize);
                *c2rust_lvalue_ptr = xstrdup(tv_get_string(&raw const (*arg_0).li_tv));
                arg_0 = (*arg_0).li_next;
            }
        }
    }
    *argv.offset(i_0 as isize) = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut chan: *mut Channel = channel_job_start(
        argv,
        ::core::ptr::null::<::core::ffi::c_char>(),
        CallbackReader {
            cb: Callback {
                data: C2Rust_Unnamed_19 {
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
        },
        CallbackReader {
            cb: Callback {
                data: C2Rust_Unnamed_19 {
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
        },
        Callback {
            data: C2Rust_Unnamed_19 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        false_0 != 0,
        true_0 != 0,
        false_0 != 0,
        false_0 != 0,
        kChannelStdinPipe,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as uint16_t,
        0 as uint16_t,
        ::core::ptr::null_mut::<dict_T>(),
        &raw mut (*rettv).vval.v_number,
    );
    if !chan.is_null() {
        channel_create_event(chan, ::core::ptr::null::<::core::ffi::c_char>());
    }
}
pub unsafe extern "C" fn f_rpcstop(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut id: uint64_t = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_number as uint64_t;
    if !find_job(id, false_0 != 0).is_null() {
        f_jobstop(argvars, rettv, fptr);
    } else {
        let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        (*rettv).vval.v_number = channel_close(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_number as uint64_t,
            kChannelPartRpc,
            &raw mut error,
        ) as varnumber_T;
        if (*rettv).vval.v_number == 0 {
            emsg(error);
        }
    };
}
pub unsafe extern "C" fn f_last_buffer_nr(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if n < (*buf).handle {
            n = (*buf).handle as ::core::ffi::c_int;
        }
        buf = (*buf).b_next;
    }
    (*rettv).vval.v_number = n as varnumber_T;
}
pub unsafe extern "C" fn f_termopen(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    let mut must_free: bool = false_0 != 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        must_free = true_0 != 0;
        (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type = VAR_DICT;
        (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict = tv_dict_alloc();
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected dictionary\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    tv_dict_add_bool(
        (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict,
        b"term\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        kBoolVarTrue,
    );
    f_jobstart(argvars, rettv, fptr);
    if must_free {
        tv_dict_free(
            (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
        );
    }
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
