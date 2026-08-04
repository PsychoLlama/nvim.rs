use crate::src::nvim::api::private::helpers::{
    api_set_error, api_set_sctx, api_typename, find_buffer_by_handle, find_window_by_handle,
    try_enter, try_leave,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid};
use crate::src::nvim::autocmd::{
    EVENT_FILETYPE, aucmd_prepbuf, aucmd_restbuf, block_autocmds, do_filetype_autocmd, has_event,
    unblock_autocmds,
};
use crate::src::nvim::buffer::{buflist_new, bufref_valid, set_bufref, wipe_buffer};
use crate::src::nvim::options::{kOptAleph, kOptBufhidden, kOptBuftype, kOptInvalid};

use crate::src::nvim::main::{curbuf, current_sctx, curwin};
use crate::src::nvim::memline::ml_open;
use crate::src::nvim::memory::xstrdup;
use crate::src::nvim::option::{
    find_option, get_all_vimoptions, get_option_value_for, get_vimoption, object_as_optval,
    option_has_scope, optval_as_object, optval_free, set_option_direct, set_option_value_for,
};
use crate::src::nvim::os::libc::{__assert_fail, strcmp};
use crate::src::nvim::types::api::{kErrorTypeException, kErrorTypeNone, kErrorTypeValidation};
use crate::src::nvim::types::{
    Arena, BoolVarValue, Dict, Error, KeyDict_option, KeyValuePair, Object, OptIndex, OptScope,
    OptVal, OptValData, OptValType, ScopeType, SpecialVarValue, String_0, TryState, VarLockStatus,
    VarType, aco_save_T, bln_values, buf_T, bufref_T, except_T, int64_t, kObjectTypeNil, linenr_T,
    msglist_T, object, object_data as C2Rust_Unnamed, sctx_T, size_t, uint64_t, win_T,
};
use crate::src::nvim::window::close_windows;
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
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeNil: OptValType = -1;
pub const kOptScopeBuf: OptScope = 2;
pub const kOptScopeWin: OptScope = 1;
pub const kOptScopeGlobal: OptScope = 0;
pub const BLN_DUMMY: bln_values = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_13 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_13 = 1;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Dict = Dict {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<KeyValuePair>(),
};
pub const ARRAY_DICT_INIT: Dict = KV_INITIAL_VALUE;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn validate_option_value_args(
    mut opts: *mut KeyDict_option,
    mut name: *mut ::core::ffi::c_char,
    mut opt_idxp: *mut OptIndex,
    mut opt_flags: *mut ::core::ffi::c_int,
    mut scope: *mut OptScope,
    mut from: *mut *mut ::core::ffi::c_void,
    mut filetype: *mut *mut ::core::ffi::c_char,
    mut err: *mut Error,
) -> ::core::ffi::c_int {
    if (*opts).is_set__option_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_option__scope
        != 0 as ::core::ffi::c_ulonglong
    {
        if strcmp(
            (*opts).scope.data,
            b"local\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0
        {
            *opt_flags = OPT_LOCAL as ::core::ffi::c_int;
        } else if strcmp(
            (*opts).scope.data,
            b"global\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0
        {
            *opt_flags = OPT_GLOBAL as ::core::ffi::c_int;
        } else if true {
            api_err_exp(
                err,
                b"scope\0".as_ptr() as *const ::core::ffi::c_char,
                b"'local' or 'global'\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return 0 as ::core::ffi::c_int;
        }
    }
    *scope = kOptScopeGlobal;
    if !filetype.is_null()
        && (*opts).is_set__option_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_option__filetype
            != 0 as ::core::ffi::c_ulonglong
    {
        *filetype = (*opts).filetype.data;
    }
    if (*opts).is_set__option_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_option__win
        != 0 as ::core::ffi::c_ulonglong
    {
        *scope = kOptScopeWin;
        *from = find_window_by_handle((*opts).win, err) as *mut ::core::ffi::c_void;
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return FAIL;
        }
    }
    if (*opts).is_set__option_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_option__buf
        != 0 as ::core::ffi::c_ulonglong
    {
        if (*opts).is_set__option_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong
            && *opt_flags == OPT_GLOBAL as ::core::ffi::c_int
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"cannot use both global 'scope' and 'buf'\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            return 0 as ::core::ffi::c_int;
        }
        *opt_flags = OPT_LOCAL as ::core::ffi::c_int;
        *scope = kOptScopeBuf;
        *from = find_buffer_by_handle((*opts).buf, err) as *mut ::core::ffi::c_void;
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return FAIL;
        }
    }
    if !(!((*opts).is_set__option_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_ulonglong)
        || !((*opts).is_set__option_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong
            || (*opts).is_set__option_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong
            || (*opts).is_set__option_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong))
    {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"cannot use 'filetype' with 'scope', 'buf' or 'win'\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return 0 as ::core::ffi::c_int;
    }
    if !(!((*opts).is_set__option_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_ulonglong)
        || !((*opts).is_set__option_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong))
    {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"cannot use both 'buf' and 'win'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 0 as ::core::ffi::c_int;
    }
    *opt_idxp = find_option(name);
    if *opt_idxp as ::core::ffi::c_int == kOptInvalid as ::core::ffi::c_int {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Unknown option '%s'\0".as_ptr() as *const ::core::ffi::c_char,
            name,
        );
    } else if *scope as ::core::ffi::c_uint
        == kOptScopeBuf as ::core::ffi::c_int as ::core::ffi::c_uint
        || *scope as ::core::ffi::c_uint
            == kOptScopeWin as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !option_has_scope(*opt_idxp, *scope) {
            let mut tgt: *mut ::core::ffi::c_char = (if *scope as ::core::ffi::c_uint
                == kOptScopeBuf as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                b"buf\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"win\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            let mut global: *mut ::core::ffi::c_char =
                (if option_has_scope(*opt_idxp, kOptScopeGlobal) as ::core::ffi::c_int != 0 {
                    b"global \0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char;
            let mut req: *mut ::core::ffi::c_char =
                (if option_has_scope(*opt_idxp, kOptScopeBuf) as ::core::ffi::c_int != 0 {
                    b"buffer-local \0".as_ptr() as *const ::core::ffi::c_char
                } else if option_has_scope(*opt_idxp, kOptScopeWin) as ::core::ffi::c_int != 0 {
                    b"window-local \0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char;
            api_set_error(
                err,
                kErrorTypeValidation,
                b"'%s' cannot be passed for %s%soption '%s'\0".as_ptr()
                    as *const ::core::ffi::c_char,
                tgt,
                global,
                req,
                name,
            );
        }
    }
    return if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        FAIL
    } else {
        OK
    };
}
unsafe extern "C" fn do_ft_buf(
    mut filetype: *const ::core::ffi::c_char,
    mut aco: *mut aco_save_T,
    mut aco_used: *mut bool,
    mut err: *mut Error,
) -> *mut buf_T {
    *aco_used = false_0 != 0;
    if filetype.is_null() {
        return ::core::ptr::null_mut::<buf_T>();
    }
    let mut ftbuf: *mut buf_T = buflist_new(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        1 as linenr_T,
        BLN_DUMMY as ::core::ffi::c_int,
    );
    if ftbuf.is_null() {
        api_set_error(
            err,
            kErrorTypeException,
            b"Could not create internal buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<buf_T>();
    }
    if ml_open(ftbuf) == FAIL {
        api_set_error(
            err,
            kErrorTypeException,
            b"Could not load internal buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return ftbuf;
    }
    let mut bufref: bufref_T = bufref_T::default();
    set_bufref(&raw mut bufref, ftbuf);
    aucmd_prepbuf(aco, ftbuf);
    *aco_used = true_0 != 0;
    set_option_direct(
        kOptBufhidden,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"hide\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
        SID_NONE,
    );
    set_option_direct(
        kOptBuftype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"nofile\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
        SID_NONE,
    );
    '_c2rust_label: {
        if (*(*ftbuf).b_ml.ml_mfp).mf_fd < 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"ftbuf->b_ml.ml_mfp->mf_fd < 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/options.rs\0".as_ptr() as *const ::core::ffi::c_char,
                134 as ::core::ffi::c_uint,
                b"buf_T *do_ft_buf(const char *, aco_save_T *, _Bool *, Error *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*ftbuf).b_p_swf = false_0;
    (*ftbuf).b_p_ml = false_0;
    (*ftbuf).b_p_ft = xstrdup(filetype);
    if !has_event(EVENT_FILETYPE) {
        return ftbuf;
    }
    let mut did_au_ft: bool = false_0 != 0;
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
    did_au_ft = do_filetype_autocmd(ftbuf, true);
    try_leave(&raw mut tstate, err);
    if !bufref_valid(&raw mut bufref) {
        if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            api_set_error(
                err,
                kErrorTypeException,
                b"Internal buffer was deleted\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return ::core::ptr::null_mut::<buf_T>();
    }
    if !did_au_ft && !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"Could not execute FileType autocommands\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return ftbuf;
}
unsafe extern "C" fn wipe_ft_buf(mut buf: *mut buf_T) {
    block_autocmds();
    let mut bufref: bufref_T = bufref_T::default();
    set_bufref(&raw mut bufref, buf);
    close_windows(buf, false_0 != 0);
    if bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
        && buf != curbuf.get()
        && (*buf).b_nwindows == 0 as ::core::ffi::c_int
    {
        wipe_buffer(buf, false_0 != 0);
    }
    if bufref_valid(&raw mut bufref) {
        (*buf).b_flags &= !BF_DUMMY;
    }
    unblock_autocmds();
}
pub unsafe extern "C" fn nvim_get_option_value(
    mut name: String_0,
    mut opts: *mut KeyDict_option,
    mut err: *mut Error,
) -> Object {
    let mut opt_idx: OptIndex = kOptAleph;
    let mut opt_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut scope: OptScope = kOptScopeGlobal;
    let mut from: *mut ::core::ffi::c_void = NULL;
    let mut filetype: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if validate_option_value_args(
        opts,
        name.data,
        &raw mut opt_idx,
        &raw mut opt_flags,
        &raw mut scope,
        &raw mut from,
        &raw mut filetype,
        err,
    ) == 0
    {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    let mut aco: aco_save_T = aco_save_T::default();
    let mut aco_used: bool = false;
    let mut ftbuf: *mut buf_T = do_ft_buf(filetype, &raw mut aco, &raw mut aco_used, err);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        if aco_used {
            aucmd_restbuf(&raw mut aco);
        }
        if !ftbuf.is_null() {
            wipe_ft_buf(ftbuf);
        }
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    if !ftbuf.is_null() {
        '_c2rust_label: {
            if from.is_null() {
            } else {
                __assert_fail(
                    b"!from\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/api/options.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    230 as ::core::ffi::c_uint,
                    b"Object nvim_get_option_value(String, KeyDict_option *, Error *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        from = ftbuf as *mut ::core::ffi::c_void;
    }
    let mut value: OptVal = get_option_value_for(opt_idx, opt_flags, scope, from, err);
    if !ftbuf.is_null() {
        if aco_used {
            aucmd_restbuf(&raw mut aco);
        }
        wipe_ft_buf(ftbuf);
    }
    if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
        if !(value.type_0 as ::core::ffi::c_int != kOptValTypeNil as ::core::ffi::c_int) {
            api_err_invalid(
                err,
                b"option\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
        } else {
            return optval_as_object(value);
        }
    }
    optval_free(value);
    return object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
}
pub unsafe extern "C" fn nvim_set_option_value(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut value: Object,
    mut opts: *mut KeyDict_option,
    mut err: *mut Error,
) {
    let mut opt_idx: OptIndex = kOptAleph;
    let mut opt_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut scope: OptScope = kOptScopeGlobal;
    let mut to: *mut ::core::ffi::c_void = NULL;
    if validate_option_value_args(
        opts,
        name.data,
        &raw mut opt_idx,
        &raw mut opt_flags,
        &raw mut scope,
        &raw mut to,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        err,
    ) == 0
    {
        return;
    }
    if scope as ::core::ffi::c_uint == kOptScopeWin as ::core::ffi::c_int as ::core::ffi::c_uint
        && opt_flags == 0 as ::core::ffi::c_int
    {
        if option_has_scope(opt_idx, kOptScopeGlobal) {
            opt_flags = OPT_LOCAL as ::core::ffi::c_int;
        }
    }
    let Some(optval) = object_as_optval(value) else {
        api_err_exp(
            err,
            b"value\0".as_ptr() as *const ::core::ffi::c_char,
            b"valid option type\0".as_ptr() as *const ::core::ffi::c_char,
            api_typename(value.type_0),
        );
        return;
    };
    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
    set_option_value_for(name.data, opt_idx, optval, opt_flags, scope, to, err);
    current_sctx.set(save_current_sctx);
}
pub unsafe extern "C" fn nvim_get_all_options_info(
    mut arena: *mut Arena,
    mut _err: *mut Error,
) -> Dict {
    return get_all_vimoptions(arena);
}
pub unsafe extern "C" fn nvim_get_option_info2(
    mut name: String_0,
    mut opts: *mut KeyDict_option,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut opt_idx: OptIndex = kOptAleph;
    let mut opt_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut scope: OptScope = kOptScopeGlobal;
    let mut from: *mut ::core::ffi::c_void = NULL;
    if validate_option_value_args(
        opts,
        name.data,
        &raw mut opt_idx,
        &raw mut opt_flags,
        &raw mut scope,
        &raw mut from,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        err,
    ) == 0
    {
        return ARRAY_DICT_INIT;
    }
    let mut buf: *mut buf_T = if scope as ::core::ffi::c_uint
        == kOptScopeBuf as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        from as *mut buf_T
    } else {
        curbuf.get()
    };
    let mut win: *mut win_T = if scope as ::core::ffi::c_uint
        == kOptScopeWin as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        from as *mut win_T
    } else {
        curwin.get()
    };
    return get_vimoption(name, opt_flags, buf, win, arena, err);
}
pub const KEYSET_OPTIDX_option__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_option__win: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_option__scope: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_option__filetype: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const BF_DUMMY: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
