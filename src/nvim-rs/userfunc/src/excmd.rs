//! Ex-command entry points for user functions: `:call` and `:function`.
//!
//! Wave 2 Phase 2: `ex_call` migrated here from `src/nvim/eval/userfunc.c`.
//! Wave 2 Phase 4: `ex_function` to be migrated here.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int, c_void};

use nvim_eval_exec::eval::EvalargT;
use nvim_eval_exec::funcexe::FuncExeT;

// ============================================================================
// Return value constants (matching C OK/FAIL)
// ============================================================================
const FAIL: c_int = 0;

// ============================================================================
// Function flags (matching C TFN_*)
// ============================================================================
const TFN_INT: c_int = 1;

// ============================================================================
// C extern declarations — raw pointer versions to avoid private Handle types
// ============================================================================

// trans_function_name: last param is `*mut *mut c_void` here (double pointer for partial_T**)
// but other modules use `*mut c_void`. Same ABI; allow the clash.
// aborting: some modules use `-> bool`, others `-> c_int`; same ABI on this platform.
#[allow(clashing_extern_declarations)]
extern "C" {
    // eap field accessors (existing in userfunc.c)
    fn nvim_eap_get_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_eap_get_skip(eap: *const c_void) -> c_int;
    fn nvim_eap_get_line1(eap: *const c_void) -> i32;
    fn nvim_eap_get_line2(eap: *const c_void) -> i32;
    fn nvim_eap_set_nextcmd(eap: *mut c_void, val: *mut c_char);
    fn nvim_eap_get_cstack(eap: *const c_void) -> *mut c_void;
    fn nvim_eap_get_cmdidx(eap: *const c_void) -> c_int;

    // Wave 2 Phase 2 new accessors (added to userfunc.c)
    fn nvim_cstack_get_trylevel(cstack: *const c_void) -> c_int;
    fn nvim_cmd_defer_idx() -> c_int;
    fn nvim_semsg_e_missingparen(name: *const c_char);

    // fudi (funcdict_T) field accessors (existing in userfunc.c)
    fn nvim_fudi_get_dict(fudi: *const c_void) -> *mut c_void;
    fn nvim_fudi_get_newkey(fudi: *const c_void) -> *mut c_char;

    // dict refcount + unref
    fn nvim_tv_dict_incr_refcount(dict: *mut c_void);
    fn tv_dict_unref(dict: *mut c_void);

    // message helpers
    fn nvim_semsg_dictkey(key: *const c_char);

    // emsg_skip global (inc/dec)
    fn nvim_syn_emsg_skip_inc();
    fn nvim_syn_emsg_skip_dec();

    // String/memory helpers
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn strlen(s: *const c_char) -> usize;

    // eval helpers — raw pointer signatures (these are already Rust symbols
    // via export_name; using raw *mut c_void avoids private handle types)
    fn fill_evalarg_from_eap(evalarg: *mut c_void, eap: *mut c_void, skip: bool);
    fn clear_evalarg(evalarg: *mut c_void, eap: *mut c_void);
    fn eval0(arg: *mut c_char, rettv: *mut c_void, eap: *mut c_void, evalarg: *mut c_void)
        -> c_int;
    fn tv_clear(tv: *mut c_void);

    // trans_function_name (will be Rust in Phase 3, currently C)
    fn trans_function_name(
        pp: *mut *mut c_char,
        skip: c_int,
        flags: c_int,
        fdp: *mut c_void,
        partial: *mut *mut c_void,
    ) -> *mut c_char;

    // deref_func_name (Rust, lookup.rs)
    fn deref_func_name(
        name: *const c_char,
        lenp: *mut c_int,
        partialp: *mut *mut c_void,
        no_autoload: c_int,
        found_var: *mut bool,
    ) -> *mut c_char;

    // rs_ex_defer_inner (Rust, defer.rs)
    fn rs_ex_defer_inner(
        name: *mut c_char,
        arg: *mut *mut c_char,
        partial: *const c_void,
        evalarg: *mut c_void,
    ) -> c_int;

    // ex_call_inner (Rust, funccal.rs; exported as "ex_call_inner")
    fn ex_call_inner(
        eap: *const c_void,
        name: *const c_char,
        arg: *mut *mut c_char,
        startarg: *mut c_char,
        funcexe_init: *const c_void,
        evalarg: *mut c_void,
    ) -> c_int;

    // check_nextcmd, ends_excmd, aborting (C)
    fn check_nextcmd(p: *const c_char) -> *mut c_char;
    fn ends_excmd(c: c_int) -> c_int;
    fn aborting() -> bool;

    // emsg_severe and did_throw globals
    static mut emsg_severe: bool;
    static did_throw: bool;

    // e_trailing_arg message accessor
    fn nvim_e_trailing_arg() -> *const c_char;
    fn nvim_semsg_with_name(errmsg: *const c_char, name: *const c_char);
}

// ============================================================================
// ex_call  (Wave 2 Phase 2: migrated from userfunc.c)
// ============================================================================

/// `:1,25call func(arg1, arg2)` function call.
/// `:defer func(arg1, arg2)`    deferred function call.
///
/// Replaces C `ex_call` — body is now here; C file has `extern` decl only.
///
/// # Safety
/// `eap` must be a valid `exarg_T *`.
#[unsafe(export_name = "ex_call")]
pub unsafe extern "C" fn rs_ex_call(eap: *mut c_void) {
    // funcdict_T fudi — 3 pointer fields (dict_T*, char*, dictitem_T*) = 24 bytes, zero-init
    let mut fudi = [0u8; 24usize];
    let mut partial: *mut c_void = std::ptr::null_mut();
    // evalarg_T on the stack, zero-initialized via EvalargT::new_skip()
    let mut evalarg = EvalargT::new_skip();

    let mut arg = unsafe { nvim_eap_get_arg(eap) };
    let skip = unsafe { nvim_eap_get_skip(eap) } != 0;

    unsafe { fill_evalarg_from_eap(std::ptr::addr_of_mut!(evalarg).cast::<c_void>(), eap, skip) };

    if skip {
        // trans_function_name() doesn't work well when skipping; use eval0()
        // to skip to any following command (e.g. `:if 0 | call dict.foo().bar() | endif`).
        // typval_T on stack: v_type (4 bytes int) + 4 pad + vval (8 bytes) + v_lock (4 bytes) + 4 pad = 24 bytes
        let mut rettv = [0u8; 24usize];
        unsafe { nvim_syn_emsg_skip_inc() };
        let eap_arg = unsafe { nvim_eap_get_arg(eap) };
        if unsafe {
            eval0(
                eap_arg,
                rettv.as_mut_ptr().cast::<c_void>(),
                eap,
                std::ptr::addr_of_mut!(evalarg).cast::<c_void>(),
            )
        } != FAIL
        {
            unsafe { tv_clear(rettv.as_mut_ptr().cast::<c_void>()) };
        }
        unsafe { nvim_syn_emsg_skip_dec() };
        unsafe { clear_evalarg(std::ptr::addr_of_mut!(evalarg).cast::<c_void>(), eap) };
        return;
    }

    let tofree = unsafe {
        trans_function_name(
            std::ptr::addr_of_mut!(arg),
            0, // skip = false
            TFN_INT,
            fudi.as_mut_ptr().cast::<c_void>(),
            std::ptr::addr_of_mut!(partial),
        )
    };

    let fudi_ptr: *mut c_void = fudi.as_mut_ptr().cast();
    let newkey = unsafe { nvim_fudi_get_newkey(fudi_ptr) };
    if !newkey.is_null() {
        // Still need to give an error message for missing key.
        unsafe { nvim_semsg_dictkey(newkey) };
        unsafe { xfree(newkey.cast()) };
    }

    let fd_dict = unsafe { nvim_fudi_get_dict(fudi_ptr) };

    if tofree.is_null() {
        unsafe { clear_evalarg(std::ptr::addr_of_mut!(evalarg).cast::<c_void>(), eap) };
        unsafe { tv_dict_unref(fd_dict) };
        return;
    }

    // Increase refcount on dictionary; it could get deleted when evaluating args.
    if !fd_dict.is_null() {
        unsafe { nvim_tv_dict_incr_refcount(fd_dict) };
    }

    // If it is a VAR_FUNC or VAR_PARTIAL variable, use its contents.
    // For VAR_PARTIAL get its partial, unless we already have one from trans_function_name().
    #[allow(clippy::cast_possible_truncation)]
    let mut len = unsafe { strlen(tofree) } as c_int;
    let mut found_var = false;
    let name = unsafe {
        deref_func_name(
            tofree,
            std::ptr::addr_of_mut!(len),
            if partial.is_null() {
                std::ptr::addr_of_mut!(partial)
            } else {
                std::ptr::null_mut()
            },
            0, // no_autoload = false
            std::ptr::addr_of_mut!(found_var),
        )
    };

    // Skip white space to allow ":call func ()".
    let startarg = unsafe { skipwhite(arg) };

    if unsafe { *startarg } != b'(' as c_char {
        let eap_arg = unsafe { nvim_eap_get_arg(eap) };
        unsafe { nvim_semsg_e_missingparen(eap_arg) };
        // goto end
        unsafe { tv_dict_unref(fd_dict) };
        unsafe { xfree(tofree.cast()) };
        unsafe { clear_evalarg(std::ptr::addr_of_mut!(evalarg).cast::<c_void>(), eap) };
        return;
    }

    let cmd_defer_idx = unsafe { nvim_cmd_defer_idx() };
    let cmdidx = unsafe { nvim_eap_get_cmdidx(eap) };
    let failed: c_int;

    if cmdidx == cmd_defer_idx {
        let mut arg_mut = startarg;
        // FAIL = 0, OK = 1; c_int::from(result == FAIL) = 1 if failed
        let result = unsafe {
            rs_ex_defer_inner(
                name,
                std::ptr::addr_of_mut!(arg_mut),
                partial,
                std::ptr::addr_of_mut!(evalarg).cast::<c_void>(),
            )
        };
        failed = c_int::from(result == FAIL);
        arg = arg_mut;
    } else {
        let mut funcexe = FuncExeT::new();
        funcexe.fe_partial = partial;
        funcexe.fe_selfdict = fd_dict;
        funcexe.fe_firstline = unsafe { nvim_eap_get_line1(eap) };
        funcexe.fe_lastline = unsafe { nvim_eap_get_line2(eap) };
        funcexe.fe_found_var = found_var;
        funcexe.fe_evaluate = true;

        let mut arg_mut = startarg;
        failed = unsafe {
            ex_call_inner(
                eap,
                name,
                std::ptr::addr_of_mut!(arg_mut),
                startarg,
                std::ptr::addr_of!(funcexe).cast::<c_void>(),
                std::ptr::addr_of_mut!(evalarg).cast::<c_void>(),
            )
        };
        arg = arg_mut;
    }

    // When inside :try we need to check for following "| catch" or "| endtry".
    // Not when there was an error, but do check if an exception was thrown.
    let cstack = unsafe { nvim_eap_get_cstack(eap) };
    let trylevel = unsafe { nvim_cstack_get_trylevel(cstack) };
    if (!unsafe { aborting() } || unsafe { did_throw }) && (failed == 0 || trylevel > 0) {
        // Check for trailing illegal characters and a following command.
        if unsafe { ends_excmd(c_int::from(*arg as u8)) } == 0 {
            if failed == 0 && !unsafe { aborting() } {
                unsafe { emsg_severe = true };
                unsafe { nvim_semsg_with_name(nvim_e_trailing_arg(), arg) };
            }
        } else {
            let next = unsafe { check_nextcmd(arg) };
            unsafe { nvim_eap_set_nextcmd(eap, next) };
        }
    }
    unsafe { clear_evalarg(std::ptr::addr_of_mut!(evalarg).cast::<c_void>(), eap) };

    // end:
    unsafe { tv_dict_unref(fd_dict) };
    unsafe { xfree(tofree.cast()) };
}
