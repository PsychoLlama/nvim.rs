//! Calling out to a user expression from a C caller.
//!
//! `'charconvert'`, `'diffexpr'`, `'patchexpr'` and `'spellsuggest'` are
//! options holding Vimscript, and each of these evaluates one of them with
//! the relevant `v:` variables in place.  They live here because
//! `prepare_vimvar`/`restore_vimvar` and the `v:fname_*` family do.
//!
//! All four share one shape: publish the `v:` variables the expression is
//! meant to read, evaluate it in the script context the *option* was set
//! from (so that a `<SID>` in it resolves where the user wrote it, not where
//! the file is being read), then blank the variables again and put the
//! context back.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

#[allow(unused_imports)]
use super::*;

/// Evaluate `'charconvert'` to convert `fname_from` into `fname_to`.
///
/// Answers `FAIL` both when the expression itself failed and when it
/// answered a true value, which is how it reports that the conversion did
/// not work.
///
/// # Safety
/// The four arguments are NUL-terminated strings.
pub unsafe fn eval_charconvert(
    enc_from: *const c_char,
    enc_to: *const c_char,
    fname_from: *const c_char,
    fname_to: *const c_char,
) -> c_int {
    unsafe {
        let saved_sctx = current_sctx.get();
        set_vim_var_string(VV_CC_FROM, enc_from, -1);
        set_vim_var_string(VV_CC_TO, enc_to, -1);
        set_vim_var_string(VV_FNAME_IN, fname_from, -1);
        set_vim_var_string(VV_FNAME_OUT, fname_to, -1);
        if let Some(ctx) = get_option_sctx(kOptCharconvert).as_ref() {
            current_sctx.set(*ctx);
        }

        let mut err = false;
        if eval_to_bool(p_ccv.get(), &raw mut err, ptr::null_mut(), false, true) {
            err = true;
        }

        set_vim_var_string(VV_CC_FROM, ptr::null(), -1);
        set_vim_var_string(VV_CC_TO, ptr::null(), -1);
        set_vim_var_string(VV_FNAME_IN, ptr::null(), -1);
        set_vim_var_string(VV_FNAME_OUT, ptr::null(), -1);
        current_sctx.set(saved_sctx);

        if err { FAIL } else { OK }
    }
}

/// Evaluate `'diffexpr'` to write the difference between `origfile` and
/// `newfile` into `outfile`.  Errors are ignored: the caller notices by
/// finding no usable output.
///
/// # Safety
/// The three arguments are NUL-terminated strings.
pub unsafe fn eval_diff(origfile: *const c_char, newfile: *const c_char, outfile: *const c_char) {
    unsafe {
        let saved_sctx = current_sctx.get();
        set_vim_var_string(VV_FNAME_IN, origfile, -1);
        set_vim_var_string(VV_FNAME_NEW, newfile, -1);
        set_vim_var_string(VV_FNAME_OUT, outfile, -1);
        if let Some(ctx) = get_option_sctx(kOptDiffexpr).as_ref() {
            current_sctx.set(*ctx);
        }

        tv_free(eval_expr_ext(p_dex.get(), ptr::null_mut(), true));

        set_vim_var_string(VV_FNAME_IN, ptr::null(), -1);
        set_vim_var_string(VV_FNAME_NEW, ptr::null(), -1);
        set_vim_var_string(VV_FNAME_OUT, ptr::null(), -1);
        current_sctx.set(saved_sctx);
    }
}

/// Evaluate `'patchexpr'` to apply `difffile` to `origfile`, writing the
/// result to `outfile`.  Errors are ignored, as in [`eval_diff`].
///
/// # Safety
/// The three arguments are NUL-terminated strings.
pub unsafe fn eval_patch(origfile: *const c_char, difffile: *const c_char, outfile: *const c_char) {
    unsafe {
        let saved_sctx = current_sctx.get();
        set_vim_var_string(VV_FNAME_IN, origfile, -1);
        set_vim_var_string(VV_FNAME_DIFF, difffile, -1);
        set_vim_var_string(VV_FNAME_OUT, outfile, -1);
        if let Some(ctx) = get_option_sctx(kOptPatchexpr).as_ref() {
            current_sctx.set(*ctx);
        }

        tv_free(eval_expr_ext(p_pex.get(), ptr::null_mut(), true));

        set_vim_var_string(VV_FNAME_IN, ptr::null(), -1);
        set_vim_var_string(VV_FNAME_DIFF, ptr::null(), -1);
        set_vim_var_string(VV_FNAME_OUT, ptr::null(), -1);
        current_sctx.set(saved_sctx);
    }
}

/// Evaluate the `expr:` part of `'spellsuggest'` over `badword`, which the
/// expression reads as `v:val`.
///
/// Answers the suggestion list, or NULL when the expression failed or did
/// not answer a List.  Errors are suppressed unless `'verbose'` is on.
///
/// # Safety
/// `badword` and `expr` are NUL-terminated strings.
pub unsafe fn eval_spell_expr(badword: *mut c_char, expr: *mut c_char) -> *mut list_T {
    unsafe {
        let mut p = skipwhite(expr);
        let saved_sctx = current_sctx.get();

        // `v:val` is the bad word; it has no type of its own, so it has to
        // be added to the `v:` dictionary and taken out again.
        let mut save_val = TV_INITIAL_VALUE;
        prepare_vimvar(VV_VAL as c_int, &raw mut save_val);
        set_vim_var_string(VV_VAL, badword, -1);
        if p_verbose.get() == 0 {
            (*emsg_off.ptr()) += 1;
        }
        if let Some(ctx) = get_option_sctx(kOptSpellsuggest).as_ref() {
            current_sctx.set(*ctx);
        }

        let mut rettv = TV_INITIAL_VALUE;
        // A bare `Func(v:val)` call is evaluated without the expression
        // parser; anything else goes through it.
        let mut r = may_call_simple_func(p, &raw mut rettv);
        if r == NOTDONE {
            r = eval1(&raw mut p, &raw mut rettv, EVALARG_EVALUATE.ptr());
        }
        let mut list: *mut list_T = ptr::null_mut();
        if r == OK {
            if rettv.v_type == VAR_LIST {
                list = rettv.vval.v_list;
            } else {
                tv_clear(&raw mut rettv);
            }
        }

        if p_verbose.get() == 0 {
            (*emsg_off.ptr()) -= 1;
        }
        tv_clear(get_vim_var_tv(VV_VAL));
        restore_vimvar(VV_VAL as c_int, &raw mut save_val);
        current_sctx.set(saved_sctx);

        list
    }
}

/// One suggestion from [`eval_spell_expr`]'s answer: the word into
/// `ret_word` and the score as the return value, or -1 on an error.
///
/// An entry has to be a two-element list of a word and a score; the score is
/// not checked for being unsigned, which upstream notes and does not fix.
///
/// # Safety
/// `list` is one entry of the suggestion list; `ret_word` is writable, and
/// is left alone when the answer is -1.
pub unsafe fn get_spellword(list: *mut list_T, ret_word: *mut *const c_char) -> c_int {
    unsafe {
        if tv_list_len(list) != 2 {
            emsg(gettext(
                c"E5700: Expression from 'spellsuggest' must yield lists with exactly two values"
                    .as_ptr(),
            ));
            return -1;
        }
        *ret_word = tv_list_find_str(list, 0);
        if (*ret_word).is_null() {
            return -1;
        }
        tv_list_find_nr(list, -1, ptr::null_mut()) as c_int
    }
}
