//! Calling out to a user expression from a C caller.
//!
//! `'charconvert'`, `'diffexpr'`, `'patchexpr'` and `'spellsuggest'` are
//! options holding Vimscript, and each of these evaluates one of them with
//! the relevant `v:` variables in place.  They live in vars.c because that
//! is where `prepare_vimvar`/`restore_vimvar` do.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn eval_charconvert(
    enc_from: *const ::core::ffi::c_char,
    enc_to: *const ::core::ffi::c_char,
    fname_from: *const ::core::ffi::c_char,
    fname_to: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let saved_sctx: sctx_T = current_sctx.get();
        set_vim_var_string(VV_CC_FROM, enc_from, -1 as ptrdiff_t);
        set_vim_var_string(VV_CC_TO, enc_to, -1 as ptrdiff_t);
        set_vim_var_string(VV_FNAME_IN, fname_from, -1 as ptrdiff_t);
        set_vim_var_string(VV_FNAME_OUT, fname_to, -1 as ptrdiff_t);
        let mut ctx: *mut sctx_T = get_option_sctx(kOptCharconvert);
        if !ctx.is_null() {
            current_sctx.set(*ctx);
        }
        let mut err: bool = false_0 != 0;
        if eval_to_bool(
            p_ccv.get(),
            &raw mut err,
            ::core::ptr::null_mut::<exarg_T>(),
            false_0 != 0,
            true_0 != 0,
        ) {
            err = true_0 != 0;
        }
        set_vim_var_string(
            VV_CC_FROM,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_CC_TO,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_FNAME_IN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_FNAME_OUT,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        current_sctx.set(saved_sctx);
        if err {
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn eval_diff(
    origfile: *const ::core::ffi::c_char,
    newfile: *const ::core::ffi::c_char,
    outfile: *const ::core::ffi::c_char,
) {
    unsafe {
        let saved_sctx: sctx_T = current_sctx.get();
        set_vim_var_string(VV_FNAME_IN, origfile, -1 as ptrdiff_t);
        set_vim_var_string(VV_FNAME_NEW, newfile, -1 as ptrdiff_t);
        set_vim_var_string(VV_FNAME_OUT, outfile, -1 as ptrdiff_t);
        let mut ctx: *mut sctx_T = get_option_sctx(kOptDiffexpr);
        if !ctx.is_null() {
            current_sctx.set(*ctx);
        }
        let mut tv: *mut typval_T =
            eval_expr_ext(p_dex.get(), ::core::ptr::null_mut::<exarg_T>(), true_0 != 0);
        tv_free(tv);
        set_vim_var_string(
            VV_FNAME_IN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_FNAME_NEW,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_FNAME_OUT,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        current_sctx.set(saved_sctx);
    }
}

pub unsafe extern "C" fn eval_patch(
    origfile: *const ::core::ffi::c_char,
    difffile: *const ::core::ffi::c_char,
    outfile: *const ::core::ffi::c_char,
) {
    unsafe {
        let saved_sctx: sctx_T = current_sctx.get();
        set_vim_var_string(VV_FNAME_IN, origfile, -1 as ptrdiff_t);
        set_vim_var_string(VV_FNAME_DIFF, difffile, -1 as ptrdiff_t);
        set_vim_var_string(VV_FNAME_OUT, outfile, -1 as ptrdiff_t);
        let mut ctx: *mut sctx_T = get_option_sctx(kOptPatchexpr);
        if !ctx.is_null() {
            current_sctx.set(*ctx);
        }
        let mut tv: *mut typval_T =
            eval_expr_ext(p_pex.get(), ::core::ptr::null_mut::<exarg_T>(), true_0 != 0);
        tv_free(tv);
        set_vim_var_string(
            VV_FNAME_IN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_FNAME_DIFF,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_FNAME_OUT,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        current_sctx.set(saved_sctx);
    }
}

pub unsafe extern "C" fn eval_spell_expr(
    mut badword: *mut ::core::ffi::c_char,
    mut expr: *mut ::core::ffi::c_char,
) -> *mut list_T {
    unsafe {
        let mut save_val: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut list: *mut list_T = ::core::ptr::null_mut::<list_T>();
        let mut p: *mut ::core::ffi::c_char = skipwhite(expr);
        let saved_sctx: sctx_T = current_sctx.get();
        prepare_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
        set_vim_var_string(VV_VAL, badword, -1 as ptrdiff_t);
        if p_verbose.get() == 0 as OptInt {
            (*emsg_off.ptr()) += 1;
        }
        let mut ctx: *mut sctx_T = get_option_sctx(kOptSpellsuggest);
        if !ctx.is_null() {
            current_sctx.set(*ctx);
        }
        let mut r: ::core::ffi::c_int = may_call_simple_func(p, &raw mut rettv);
        if r == NOTDONE {
            r = eval1(&raw mut p, &raw mut rettv, EVALARG_EVALUATE.ptr());
        }
        if r == OK {
            if rettv.v_type as ::core::ffi::c_uint
                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                tv_clear(&raw mut rettv);
            } else {
                list = rettv.vval.v_list;
            }
        }
        if p_verbose.get() == 0 as OptInt {
            (*emsg_off.ptr()) -= 1;
        }
        tv_clear(get_vim_var_tv(VV_VAL));
        restore_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
        current_sctx.set(saved_sctx);
        return list;
    }
}

pub unsafe extern "C" fn get_spellword(
    list: *mut list_T,
    mut ret_word: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if tv_list_len(list) != 2 as ::core::ffi::c_int {
            emsg(gettext(
                b"E5700: Expression from 'spellsuggest' must yield lists with exactly two values\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ));
            return -1 as ::core::ffi::c_int;
        }
        *ret_word = tv_list_find_str(list, 0 as ::core::ffi::c_int);
        if (*ret_word).is_null() {
            return -1 as ::core::ffi::c_int;
        }
        return tv_list_find_nr(
            list,
            -1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<bool>(),
        ) as ::core::ffi::c_int;
    }
}
