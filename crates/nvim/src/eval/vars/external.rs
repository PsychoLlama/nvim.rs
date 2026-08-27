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

use super::*;
use crate::eval::typval::NumBuf;
use crate::guard::Suppress;
use crate::types::{FAIL, OK};

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
    let saved_sctx = current_sctx.get();
    unsafe { set_vim_var_string(Vv::CharconvertFrom, enc_from, -1) };
    unsafe { set_vim_var_string(Vv::CharconvertTo, enc_to, -1) };
    unsafe { set_vim_var_string(Vv::FnameIn, fname_from, -1) };
    unsafe { set_vim_var_string(Vv::FnameOut, fname_to, -1) };
    current_sctx.set(option_last_set(kOptCharconvert));

    let mut err = false;
    if unsafe { eval_to_bool(p_ccv.get(), &raw mut err, ptr::null_mut(), false, true) } {
        err = true;
    }

    unsafe { set_vim_var_string(Vv::CharconvertFrom, ptr::null(), -1) };
    unsafe { set_vim_var_string(Vv::CharconvertTo, ptr::null(), -1) };
    unsafe { set_vim_var_string(Vv::FnameIn, ptr::null(), -1) };
    unsafe { set_vim_var_string(Vv::FnameOut, ptr::null(), -1) };
    current_sctx.set(saved_sctx);

    if err { FAIL } else { OK }
}

/// Evaluate `'diffexpr'` to write the difference between `origfile` and
/// `newfile` into `outfile`.  Errors are ignored: the caller notices by
/// finding no usable output.
///
/// # Safety
/// The three arguments are NUL-terminated strings.
pub unsafe fn eval_diff(origfile: *const c_char, newfile: *const c_char, outfile: *const c_char) {
    let saved_sctx = current_sctx.get();
    unsafe { set_vim_var_string(Vv::FnameIn, origfile, -1) };
    unsafe { set_vim_var_string(Vv::FnameNew, newfile, -1) };
    unsafe { set_vim_var_string(Vv::FnameOut, outfile, -1) };
    current_sctx.set(option_last_set(kOptDiffexpr));

    unsafe { tv_free(eval_expr_ext(p_dex.get(), ptr::null_mut(), true)) };

    unsafe { set_vim_var_string(Vv::FnameIn, ptr::null(), -1) };
    unsafe { set_vim_var_string(Vv::FnameNew, ptr::null(), -1) };
    unsafe { set_vim_var_string(Vv::FnameOut, ptr::null(), -1) };
    current_sctx.set(saved_sctx);
}

/// Evaluate `'patchexpr'` to apply `difffile` to `origfile`, writing the
/// result to `outfile`.  Errors are ignored, as in [`eval_diff`].
///
/// # Safety
/// The three arguments are NUL-terminated strings.
pub unsafe fn eval_patch(origfile: *const c_char, difffile: *const c_char, outfile: *const c_char) {
    let saved_sctx = current_sctx.get();
    unsafe { set_vim_var_string(Vv::FnameIn, origfile, -1) };
    unsafe { set_vim_var_string(Vv::FnameDiff, difffile, -1) };
    unsafe { set_vim_var_string(Vv::FnameOut, outfile, -1) };
    current_sctx.set(option_last_set(kOptPatchexpr));

    unsafe { tv_free(eval_expr_ext(p_pex.get(), ptr::null_mut(), true)) };

    unsafe { set_vim_var_string(Vv::FnameIn, ptr::null(), -1) };
    unsafe { set_vim_var_string(Vv::FnameDiff, ptr::null(), -1) };
    unsafe { set_vim_var_string(Vv::FnameOut, ptr::null(), -1) };
    current_sctx.set(saved_sctx);
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
    let mut evalarg = EVALARG_EVALUATE;
    let mut p = unsafe { skipwhite(expr) };
    let saved_sctx = current_sctx.get();

    // `v:val` is the bad word; it has no type of its own, so it has to
    // be added to the `v:` dictionary and taken out again.
    let mut save_val = TV_INITIAL_VALUE;
    unsafe { prepare_vimvar(Vv::Val, &raw mut save_val) };
    unsafe { set_vim_var_string(Vv::Val, badword, -1) };
    let no_emsg = (p_verbose.get() == 0).then(Suppress::emsg);
    current_sctx.set(option_last_set(kOptSpellsuggest));

    let mut rettv = TV_INITIAL_VALUE;
    // A bare `Func(v:val)` call is evaluated without the expression
    // parser; anything else goes through it.
    let mut r = unsafe { may_call_simple_func(p, &raw mut rettv) };
    if r == NOTDONE {
        r = unsafe { eval1(&raw mut p, &raw mut rettv, &raw mut evalarg) };
    }
    let mut list: *mut list_T = ptr::null_mut();
    if r == OK {
        if rettv.v_type == VAR_LIST {
            list = unsafe { rettv.vval.v_list };
        } else {
            unsafe { tv_clear(&raw mut rettv) };
        }
    }

    drop(no_emsg);
    unsafe { tv_clear(get_vim_var_tv(Vv::Val)) };
    unsafe { restore_vimvar(Vv::Val, &raw mut save_val) };
    current_sctx.set(saved_sctx);

    list
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
pub unsafe fn get_spellword(
    list: *mut list_T,
    ret_word: *mut *const c_char,
    numbuf: &mut NumBuf,
) -> c_int {
    if unsafe { tv_list_len(list) } != 2 {
        unsafe {
            emsg(gettext(
                c"E5700: Expression from 'spellsuggest' must yield lists with exactly two values"
                    .as_ptr(),
            ))
        };
        return -1;
    }
    unsafe { *ret_word = tv_list_find_str(list, 0, numbuf) };
    if unsafe { (*ret_word).is_null() } {
        return -1;
    }
    unsafe { tv_list_find_nr(list, -1, ptr::null_mut()) as c_int }
}
