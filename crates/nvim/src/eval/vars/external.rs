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
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::eval::Parsed;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::eval::typval::{ListRef, NumBuf};
use crate::guard::Suppress;
use crate::narrow::number_as_int;
use crate::types::{FAIL, OK};

/// Publish the `v:` strings an expression is meant to read.
///
/// One promise for the whole list rather than one per variable: every caller
/// below has the same handful of NUL-terminated arguments to put in place.
///
/// # Safety
/// Every value is NULL or NUL-terminated.
unsafe fn set_vim_var_strings(vars: &[(Vv, *const c_char)]) {
    for &(idx, val) in vars {
        // SAFETY: the caller's obligation.
        unsafe { set_vim_var_string(idx, val, -1) };
    }
}

/// Blank the `v:` strings [`set_vim_var_strings`] put in place.
///
/// Safe: the null string reads no bytes.
fn clear_vim_var_strings(vars: &[Vv]) {
    for &idx in vars {
        // SAFETY: the null string, which needs nothing readable.
        unsafe { set_vim_var_string(idx, ptr::null(), -1) };
    }
}

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
    const VARS: [Vv; 4] = [
        Vv::CharconvertFrom,
        Vv::CharconvertTo,
        Vv::FnameIn,
        Vv::FnameOut,
    ];
    let saved_sctx = current_sctx.get();
    let named = [
        (Vv::CharconvertFrom, enc_from),
        (Vv::CharconvertTo, enc_to),
        (Vv::FnameIn, fname_from),
        (Vv::FnameOut, fname_to),
    ];
    // SAFETY: the caller's obligation -- four NUL-terminated strings.
    unsafe { set_vim_var_strings(&named) };
    current_sctx.set(option_last_set(kOptCharconvert));

    // A copy: the expression may set the option and free its text.
    let expr = p_ccv(XString::from_cstr);
    let err = eval_to_bool(&expr, true) != Ok(false);

    clear_vim_var_strings(&VARS);
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
    const VARS: [Vv; 3] = [Vv::FnameIn, Vv::FnameNew, Vv::FnameOut];
    let saved_sctx = current_sctx.get();
    let named = [
        (Vv::FnameIn, origfile),
        (Vv::FnameNew, newfile),
        (Vv::FnameOut, outfile),
    ];
    // SAFETY: the caller's obligation -- three NUL-terminated strings.
    unsafe { set_vim_var_strings(&named) };
    current_sctx.set(option_last_set(kOptDiffexpr));

    // A copy: the expression may set the option and free its text.
    drop(eval_expr_ext(&p_dex(XString::from_cstr), true));

    clear_vim_var_strings(&VARS);
    current_sctx.set(saved_sctx);
}

/// Evaluate `'patchexpr'` to apply `difffile` to `origfile`, writing the
/// result to `outfile`.  Errors are ignored, as in [`eval_diff`].
///
/// # Safety
/// The three arguments are NUL-terminated strings.
pub unsafe fn eval_patch(origfile: *const c_char, difffile: *const c_char, outfile: *const c_char) {
    const VARS: [Vv; 3] = [Vv::FnameIn, Vv::FnameDiff, Vv::FnameOut];
    let saved_sctx = current_sctx.get();
    let named = [
        (Vv::FnameIn, origfile),
        (Vv::FnameDiff, difffile),
        (Vv::FnameOut, outfile),
    ];
    // SAFETY: the caller's obligation -- three NUL-terminated strings.
    unsafe { set_vim_var_strings(&named) };
    current_sctx.set(option_last_set(kOptPatchexpr));

    // A copy: the expression may set the option and free its text.
    drop(eval_expr_ext(&p_pex(XString::from_cstr), true));

    clear_vim_var_strings(&VARS);
    current_sctx.set(saved_sctx);
}

/// Evaluate the `expr:` part of `'spellsuggest'` over `badword`, which the
/// expression reads as `v:val`.
///
/// Answers the suggestion list, or `None` when the expression failed or did
/// not answer a List.  Errors are suppressed unless `'verbose'` is on.
///
/// # Safety
/// `badword` and `expr` are NUL-terminated strings.
pub unsafe fn eval_spell_expr(badword: *mut c_char, expr: *mut c_char) -> Option<ListRef> {
    // SAFETY: the caller's promise -- `expr` is NUL-terminated.
    let text = unsafe { cstr::bytes_at(expr) };
    let text = &text[skip::white(text)..];
    let saved_sctx = current_sctx.get();

    // `v:val` is the bad word; it has no type of its own, so it has to
    // be added to the `v:` dictionary and taken out again.
    let mut save_val = TV_INITIAL_VALUE;
    prepare_vimvar(Vv::Val, &mut save_val);
    unsafe { set_vim_var_string(Vv::Val, badword, -1) };
    let no_emsg = (p_verbose() == 0).then(Suppress::emsg);
    current_sctx.set(option_last_set(kOptSpellsuggest));

    let mut rettv = TV_INITIAL_VALUE;
    // A bare `Func(v:val)` call is evaluated without the expression
    // parser; anything else goes through it.
    let r = match may_call_simple_func(text, &mut rettv) {
        Ok(Parsed::NotThis) => eval1(&mut Cursor::new(text), &mut rettv, true),
        other => other.map(|_| ()),
    };
    let mut list = None;
    if r.is_ok() {
        if rettv.v_type() == VAR_LIST {
            // The reference goes to the caller with the handle.
            list = rettv.take_list();
        } else {
            clear_local(&mut rettv);
        }
    }

    drop(no_emsg);
    unsafe { tv_clear(&mut *get_vim_var_tv(Vv::Val)) };
    restore_vimvar(Vv::Val, &mut save_val);
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
    list: *mut List,
    ret_word: *mut *const c_char,
    numbuf: &mut NumBuf,
) -> c_int {
    if list_len(unsafe { list.as_ref() }) != 2 {
        let msg = c"E5700: Expression from 'spellsuggest' must yield lists with exactly two values";
        // SAFETY: a NUL-terminated literal.
        emsg_static(msg);
        return -1;
    }
    unsafe {
        *ret_word = list_find_str(list.as_ref(), 0, numbuf).map_or(ptr::null(), CStr::as_ptr)
    };
    if unsafe { (*ret_word).is_null() } {
        return -1;
    }
    number_as_int(list_find_nr(unsafe { list.as_ref() }, -1, None))
}
