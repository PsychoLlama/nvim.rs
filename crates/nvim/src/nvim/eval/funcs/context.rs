//! The editor context stack: the `ctx*()` family.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_ctxget(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut index: size_t = 0 as size_t;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        index = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_number as size_t;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected nothing or a Number as an argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut ctx: *mut Context = ctx_get(index);
    if ctx.is_null() {
        semsg(
            gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
            b"index\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arena: Arena = ARENA_EMPTY;
    let mut ctx_dict: Dict = ctx_to_dict(ctx, &raw mut arena);
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    object_to_vim(
        object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed_16 { dict: ctx_dict },
        },
        rettv,
        &raw mut err,
    );
    arena_mem_free(arena_finish(&raw mut arena));
    api_clear_error(&raw mut err);
}
pub unsafe extern "C" fn f_ctxpop(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if !ctx_restore(::core::ptr::null_mut::<Context>(), kCtxAll.get()) {
        emsg(gettext(
            b"Context stack is empty\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
}
pub unsafe extern "C" fn f_ctxpush(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut types: ::core::ffi::c_int = kCtxAll.get();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        types = 0 as ::core::ffi::c_int;
        let l_: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let mut tv_li: *mut typval_T = &raw mut (*li).li_tv;
                if (*tv_li).v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if strequal(
                        (*tv_li).vval.v_string,
                        b"regs\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxRegs as ::core::ffi::c_int;
                    } else if strequal(
                        (*tv_li).vval.v_string,
                        b"jumps\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxJumps as ::core::ffi::c_int;
                    } else if strequal(
                        (*tv_li).vval.v_string,
                        b"bufs\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxBufs as ::core::ffi::c_int;
                    } else if strequal(
                        (*tv_li).vval.v_string,
                        b"gvars\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxGVars as ::core::ffi::c_int;
                    } else if strequal(
                        (*tv_li).vval.v_string,
                        b"sfuncs\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxSFuncs as ::core::ffi::c_int;
                    } else if strequal(
                        (*tv_li).vval.v_string,
                        b"funcs\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        types |= kCtxFuncs as ::core::ffi::c_int;
                    }
                }
                li = (*li).li_next;
            }
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected nothing or a List as an argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    ctx_save(::core::ptr::null_mut::<Context>(), types);
}
pub unsafe extern "C" fn f_ctxset(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected dictionary as first argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut index: size_t = 0 as size_t;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        index = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_number as size_t;
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            b"expected nothing or a Number as second argument\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut ctx: *mut Context = ctx_get(index);
    if ctx.is_null() {
        semsg(
            gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
            b"index\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(false_0);
    let mut arena: Arena = ARENA_EMPTY;
    let mut dict: Dict = vim_to_object(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut arena,
        true_0 != 0,
    )
    .data
    .dict;
    let mut tmp: Context = CONTEXT_INIT;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    ctx_from_dict(dict, &raw mut tmp, &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        semsg(b"%s\0".as_ptr() as *const ::core::ffi::c_char, err.msg);
        ctx_free(&raw mut tmp);
    } else {
        ctx_free(ctx);
        *ctx = tmp;
    }
    arena_mem_free(arena_finish(&raw mut arena));
    api_clear_error(&raw mut err);
    did_emsg.set(save_did_emsg);
}
pub unsafe extern "C" fn f_ctxsize(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = ctx_size() as varnumber_T;
}
