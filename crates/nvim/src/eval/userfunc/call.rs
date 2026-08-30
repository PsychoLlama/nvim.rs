//! Calling a user function: the funccall_T's whole life.
//!
//! `call_user_func` builds the `a:` and `l:` scopes in the funccall's
//! embedded storage, evaluates the default arguments in order, runs the
//! body through `do_cmdline` and tears the scopes down again.
//! `call_user_func_check` is the guard in front of it ('maxfuncdepth',
//! the `dict` attribute, deleted functions) and `user_func_error` turns an
//! `FCERR_*` code into the message the user sees.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::eval::Parsed;
use crate::ex_docmd::DoCmdOpts;
use crate::guard::{Lock, Suppress};
use crate::message_fmt::c_str;
use crate::semsg;
use crate::smsg;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of_val;
use core::ptr;

use super::*;
use crate::types::{Failed, Refcount};

/// Run `body` inside a `:verbose` report frame: no wait-return, scrolled,
/// and terminated with a newline.
///
/// Safe: the frame is the message module's own state and the terminator is a
/// literal, so the only obligation left is `body`'s, which is its own.
fn verbose_report(body: impl FnOnce()) {
    let _no_prompt = Suppress::wait_return();
    // SAFETY: the scroll frame is the message module's own state, and the
    // terminator is a NUL-terminated literal.
    unsafe { verbose_enter_scroll() };
    body();
    unsafe { msg_puts(c"\n".as_ptr()) };
    unsafe { verbose_leave_scroll() };
}

/// Call the user function `fp`.
///
/// # Safety
/// `fp` is a live function, `argvars` holds `argcount` values, and `rettv`
/// is an uninitialised return value.
pub unsafe fn call_user_func(
    fp: *mut ufunc_T,
    argcount: c_int,
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    firstline: linenr_T,
    lastline: linenr_T,
    selfdict: *mut dict_T,
) {
    // SAFETY: the caller's promise -- `fp` is a live function.
    let mut f = unsafe { Uf::new(fp) };
    // SAFETY: the caller's promise -- `rettv` is the return value being built.
    let mut rv = unsafe { Tv::new(rettv) };
    let mut evalarg = EVALARG_EVALUATE;
    static depth: GlobalCell<c_int> = GlobalCell::new(0);

    // Don't execute the function when the call depth is getting too high.
    if depth.get() as OptInt >= p_mfd.get() {
        let deep = c"E132: Function call depth is higher than 'maxfuncdepth'";
        emsg(gettext(deep));
        rv.v_type = VAR_NUMBER;
        rv.vval.v_number = -1;
        return;
    }
    depth.set(depth.get() + 1);

    // Save the search patterns and the redo buffer.
    let mut save_redo = save_redo_T::default();
    let mut did_save_redo = false;
    save_search_patterns();
    if !ins_compl_active() {
        unsafe { save_redobuff(&raw mut save_redo) };
        did_save_redo = true;
    }
    f.uf_calls += 1;
    line_breakcheck(); // check for CTRL-C hit

    // Prepare the funccall_T.
    let fc = unsafe { create_funccal(fp, rettv) };
    // SAFETY: `create_funccal` answers the live funccall this call owns.
    let mut frame = unsafe { Fc::new(fc) };
    frame.fc_level = ex_nesting_level.get();
    // SAFETY: `fp` is live, so its inline name is a NUL-terminated string.
    frame.fc_breakpoint = unsafe { dbg_find_breakpoint(false, uf_name_ptr(fp), 0) };
    frame.fc_dbg_tick = debug_tick.get();
    let slot = size_of::<*mut ufunc_T>() as c_int;
    unsafe { ga_init(&raw mut (*fc).fc_ufuncs, slot, 1) };

    let islambda = unsafe { strncmp(uf_name_ptr(fp), c"<lambda>".as_ptr(), 8) } == 0;

    // `fc_fixvar` is an array of FIXVAR_CNT variables with names up to
    // VAR_SHORT_LEN long.  Handing out slots of it rather than allocating
    // each argument variable saves a lot of time -- and the *address* of
    // a slot is what goes into the hashtab, which is why the array lives
    // in the funccall_T and cannot be a `Vec`.
    let mut fixvar_idx = 0;
    let fixvar_base = unsafe { &raw mut (*fc).fc_fixvar } as *mut funccall_S_fc_fixvar;
    let take_fixvar = |idx: &mut c_int| -> *mut dictitem_T {
        let v = unsafe { fixvar_base.offset(*idx as isize) } as *mut dictitem_T;
        *idx += 1;
        v
    };
    // A fixvar holding one of the two scope-level names, `l:self` and
    // `a:000`; the value is filled in by the caller.
    let add_fix_var = |v: *mut dictitem_T, ht: *mut hashtab_T, key: &CStr| {
        unsafe { strcpy(tv_dict_item_key(v), key.as_ptr()) };
        unsafe { (*v).di_flags = DI_FLAGS_RO | DI_FLAGS_FIX };
        let _ = unsafe { hash_add(ht, tv_dict_item_key(v)) };
    };

    // Init the l: variables.
    let (vars, vars_var) = unsafe { (&raw mut (*fc).fc_l_vars, &raw mut (*fc).fc_l_vars_var) };
    unsafe { init_var_dict(vars, vars_var, VAR_DEF_SCOPE) };
    if !selfdict.is_null() {
        // Set l:self to "selfdict".
        let v = take_fixvar(&mut fixvar_idx);
        add_fix_var(v, unsafe { &raw mut (*fc).fc_l_vars.dv_hashtab }, c"self");
        unsafe { (*v).di_tv.v_type = VAR_DICT };
        unsafe { (*v).di_tv.v_lock = VarLock::Unlocked };
        unsafe { (*v).di_tv.vval.v_dict = selfdict };
        unsafe { (*selfdict).dv_refcount.retain() };
    }

    // Init the a: variables, unless the function body is known to use
    // none of them.
    let (avars, avars_var) = unsafe { (&raw mut (*fc).fc_l_avars, &raw mut (*fc).fc_l_avars_var) };
    unsafe { init_var_dict(avars, avars_var, VAR_SCOPE) };
    let has_args = f.uf_flags & FC_NOARGS == 0;
    if has_args {
        // Set a:0 to the number of arguments past the declared ones.
        let v = take_fixvar(&mut fixvar_idx);
        let extra = (argcount - f.uf_args.ga_len).max(0) as varnumber_T;
        unsafe { add_nr_var(avars, v, c"0".as_ptr() as *mut c_char, extra) };
    }
    frame.fc_l_avars.dv_lock = VarLock::Fixed;
    if has_args {
        // Set a:000 to the list of the extra arguments, whose items are
        // the funccall's own `fc_l_listitems` slots.
        let v = take_fixvar(&mut fixvar_idx);
        add_fix_var(v, unsafe { &raw mut (*fc).fc_l_avars.dv_hashtab }, c"000");
        unsafe { (*v).di_tv.v_type = VAR_LIST };
        unsafe { (*v).di_tv.v_lock = VarLock::Fixed };
        unsafe { (*v).di_tv.vval.v_list = &raw mut (*fc).fc_l_varlist };
    }
    unsafe { tv_list_init_static(&raw mut (*fc).fc_l_varlist) };
    unsafe { tv_list_set_lock(&raw mut (*fc).fc_l_varlist, VarLock::Fixed) };
    if has_args {
        // Set a:firstline and a:lastline.
        let avars = unsafe { &raw mut (*fc).fc_l_avars };
        let (first, last) = (c"firstline".as_ptr(), c"lastline".as_ptr());
        let v = take_fixvar(&mut fixvar_idx);
        unsafe { add_nr_var(avars, v, first as *mut c_char, firstline as varnumber_T) };
        let v = take_fixvar(&mut fixvar_idx);
        unsafe { add_nr_var(avars, v, last as *mut c_char, lastline as varnumber_T) };
    }

    // Set the argument variables.  The order is important here: the
    // parameters are named first, so that a default expression may refer
    // to the parameter to its left.
    let mut numbuf: [c_char; 65] = [0; 65];
    let mut tv_to_free: [*mut typval_T; MAX_FUNC_ARGS as usize] =
        [ptr::null_mut(); MAX_FUNC_ARGS as usize];
    let mut tv_to_free_len = 0;
    let mut default_arg_err = false;
    // Hoisted out of the loop: neither garray moves while it runs, and
    // this is the hottest loop in the family.
    let declared = ga_strings(&f.uf_args);
    let defaults = ga_strings(&f.uf_def_args);
    let mut i = 0;
    while i < argcount || i < f.uf_args.ga_len {
        let mut addlocal = false;
        let mut isdefault = false;
        let mut def_rettv = TV_INITIAL_VALUE;
        let name: *mut c_char;
        let namelen: size_t;

        // "ai" is the index in "argvars" past the declared arguments.
        let ai = i - f.uf_args.ga_len;
        if ai < 0 {
            // A declared argument: use its name.
            name = declared[i as usize];
            if islambda {
                addlocal = true;
            }
            // Evaluate the default expression when there is one and no
            // argument was given for it.
            isdefault = ai + f.uf_def_args.ga_len >= 0 && i >= argcount;
            if isdefault {
                def_rettv.v_type = VAR_NUMBER;
                def_rettv.vval.v_number = -1;
                let mut default_expr = defaults[(ai + defaults.len() as c_int) as usize];
                if unsafe { eval1(&raw mut default_expr, &raw mut def_rettv, &raw mut evalarg) }
                    .is_err()
                {
                    default_arg_err = true;
                    break;
                }
            }
            namelen = unsafe { strlen(name) };
        } else {
            if !has_args {
                break;
            }
            // An extra argument: a:1, a:2, ...
            let (into, cap) = (numbuf.as_mut_ptr(), size_of_val(&numbuf));
            namelen = unsafe { snprintf(into, cap, c"%d".as_ptr(), ai + 1) } as size_t;
            name = numbuf.as_mut_ptr();
        }

        let v = if fixvar_idx < FIXVAR_CNT && namelen <= VAR_SHORT_LEN as size_t {
            let v = take_fixvar(&mut fixvar_idx);
            unsafe { (*v).di_flags = DI_FLAGS_RO | DI_FLAGS_FIX };
            unsafe { strcpy(tv_dict_item_key(v), name) };
            v
        } else {
            let v = unsafe { tv_dict_item_alloc_len(name, namelen) };
            unsafe { (*v).di_flags |= DI_FLAGS_RO | DI_FLAGS_FIX };
            v
        };

        // Note: the argument is not copied, so its value is shared with
        // the caller's.  A default's value is this call's own, and is
        // cleared at the end.
        let value = if isdefault {
            def_rettv
        } else {
            // SAFETY: `i` is inside the caller's argument array.
            unsafe { *argvars.offset(i as isize) }
        };
        unsafe { (*v).di_tv = value };
        unsafe { (*v).di_tv.v_lock = VarLock::Fixed };
        if isdefault {
            tv_to_free[tv_to_free_len] = unsafe { &raw mut (*v).di_tv };
            tv_to_free_len += 1;
        }

        if addlocal {
            // A lambda sees its arguments as l: variables too, so the
            // value has to be reference-counted twice.
            unsafe { tv_copy(&raw mut (*v).di_tv, &raw mut (*v).di_tv) };
            let _ = unsafe { hash_add(&raw mut (*fc).fc_l_vars.dv_hashtab, tv_dict_item_key(v)) };
        } else {
            let _ = unsafe { hash_add(&raw mut (*fc).fc_l_avars.dv_hashtab, tv_dict_item_key(v)) };
        }

        if (0..MAX_FUNC_ARGS).contains(&ai) {
            // Add the extra argument to a:000, through the funccall's own
            // listitem storage.
            let li =
                unsafe { (&raw mut (*fc).fc_l_listitems as *mut listitem_T).offset(ai as isize) };
            unsafe { (*li).li_tv = *argvars.offset(i as isize) };
            unsafe { (*li).li_tv.v_lock = VarLock::Fixed };
            unsafe { tv_list_append(&raw mut (*fc).fc_l_varlist, li) };
        }
        i += 1;
    }

    // Don't redraw while executing the function.
    let redraw_off = Suppress::redraw();

    let sandboxed = (f.uf_flags & FC_SANDBOX != 0).then(Lock::sandbox);

    unsafe { estack_push_ufunc(fp, 1) };
    if p_verbose.get() >= 12 {
        verbose_report(|| {
            let called = sourcing_entry().es_name;
            // SAFETY: the message texts are literals and `es_name` is the // NUL-terminated name of the innermost exec-stack entry.
            let called = unsafe { c_str(called) };
            smsg!(0, "calling {called}");
            if p_verbose.get() >= 14 {
                unsafe { msg_puts(c"(".as_ptr()) };
                for i in 0..argcount {
                    if i > 0 {
                        unsafe { msg_puts(c", ".as_ptr()) };
                    }
                    // SAFETY: `i` is inside the caller's argument array.
                    let tv = unsafe { Tv::new(argvars.offset(i as isize)) };
                    if tv.v_type == VAR_NUMBER {
                        // SAFETY: the tag says the union holds a Number.
                        unsafe { msg_outnum(tv.vval.v_number as c_int) };
                    } else {
                        // Do not want errors such as E724 here.
                        let tofree = {
                            let _no_emsg = Suppress::emsg();
                            unsafe { encode_tv2string(tv.raw(), ptr::null_mut()) }
                        };
                        if !tofree.is_null() {
                            let mut buf: [c_char; MSG_BUF_LEN as usize] = [0; MSG_BUF_LEN as usize];
                            let mut s = tofree;
                            // SAFETY: `s` is the rendering just made and
                            // `buf` this frame's own scratch.
                            if unsafe { vim_strsize(s) } > MSG_BUF_CLEN {
                                let (into, cap) = (buf.as_mut_ptr(), size_of_val(&buf) as c_int);
                                unsafe { trunc_string(s, into, MSG_BUF_CLEN, cap) };
                                s = buf.as_mut_ptr();
                            }
                            unsafe { msg_puts(s) };
                            unsafe { xfree(tofree as *mut c_void) };
                        }
                    }
                }
                unsafe { msg_puts(c")".as_ptr()) };
            }
        });
    }

    let do_profiling_yes = do_profiling.get() == PROF_YES;
    let mut started_profiling = false;
    if do_profiling_yes
        && f.uf_profiling == 0
        && unsafe { has_profiling(false, uf_name_ptr(fp), ptr::null_mut()) }
    {
        started_profiling = true;
        unsafe { func_do_profile(fp) };
    }
    let func_or_func_caller_profiling = do_profiling_yes
        && (f.uf_profiling != 0
            || (!frame.fc_caller.is_null()
                && unsafe { (*(*(*fc).fc_caller).fc_func).uf_profiling } != 0));
    let mut call_start = 0;
    let mut wait_start = 0;
    if func_or_func_caller_profiling {
        f.uf_tm_count += 1;
        call_start = profile_start();
        f.uf_tm_children = profile_zero();
    }
    if do_profiling_yes {
        wait_start = unsafe { script_prof_save() };
    }

    let save_current_sctx = current_sctx.get();
    current_sctx.set(f.uf_script_ctx);
    let save_did_emsg = did_emsg.get();
    did_emsg.set(0);

    if default_arg_err && (f.uf_flags & FC_ABORT != 0 || trylevel.get() > 0) {
        did_emsg.set(1);
    } else if islambda {
        // A lambda's body is one line, "return <expr>"; evaluate the
        // expression straight rather than going through `do_cmdline`.
        let mut p = unsafe { ga_strings(&f.uf_lines)[0].add(c"return ".count_bytes()) };
        ex_nesting_level.set(ex_nesting_level.get() + 1);
        let _ = unsafe { eval1(&raw mut p, rettv, &raw mut evalarg) };
        ex_nesting_level.set(ex_nesting_level.get() - 1);
    } else {
        // Call do_cmdline() to execute the lines.
        type Getline = unsafe fn(c_int, *mut c_void, c_int, bool) -> *mut c_char;
        let getline = Some(get_func_line as Getline);
        let opts = DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE | DoCmdOpts::REPEAT;
        let _ = unsafe { do_cmdline(ptr::null_mut(), getline, fc as *mut c_void, opts) };
    }

    // Invoke functions added with `:defer`.
    unsafe { handle_defer_one(current_funccal.get()) };

    drop(redraw_off);

    // When the function was aborted because of an error, return -1.
    if (did_emsg.get() != 0 && f.uf_flags & FC_ABORT != 0) || rv.v_type == VAR_UNKNOWN {
        unsafe { tv_clear(rettv) };
        rv.v_type = VAR_NUMBER;
        rv.vval.v_number = -1;
    }

    if func_or_func_caller_profiling {
        call_start = profile_end(call_start);
        call_start = profile_sub_wait(wait_start, call_start);
        f.uf_tm_total = profile_add(f.uf_tm_total, call_start);
        f.uf_tm_self = profile_self(f.uf_tm_self, call_start, f.uf_tm_children);
        if !frame.fc_caller.is_null() && unsafe { (*(*(*fc).fc_caller).fc_func).uf_profiling } != 0
        {
            let caller = unsafe { (*(*fc).fc_caller).fc_func };
            unsafe { (*caller).uf_tm_children = profile_add((*caller).uf_tm_children, call_start) };
            unsafe {
                (*caller).uf_tml_children = profile_add((*caller).uf_tml_children, call_start)
            };
        }
        if started_profiling {
            // Make a `:profdel func` stop profiling the function.
            f.uf_profiling = 0;
        }
    }

    if p_verbose.get() >= 12 {
        verbose_report(|| {
            let name = sourcing_entry().es_name;
            // SAFETY: `fc_rettv` is the return value the caller handed in,
            // and the message texts are literals.
            let ret = unsafe { Tv::new(frame.fc_rettv) };
            if aborting() {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let name = unsafe { c_str(name) };
                smsg!(0, "{name} aborted");
            } else if ret.v_type == VAR_NUMBER {
                // SAFETY: the tag says the union holds a Number.
                let n = unsafe { ret.vval.v_number };
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let name = unsafe { c_str(name) };
                smsg!(0, "{name} returning #{}", n);
            } else {
                // Do not want errors such as E724 here.
                let tofree = {
                    let _no_emsg = Suppress::emsg();
                    unsafe { encode_tv2string(ret.raw(), ptr::null_mut()) }
                };
                let mut s = tofree;
                if !s.is_null() {
                    let mut buf: [c_char; MSG_BUF_LEN as usize] = [0; MSG_BUF_LEN as usize];
                    // SAFETY: `s` is the rendering just made and `buf` this
                    // frame's own scratch.
                    if unsafe { vim_strsize(s) } > MSG_BUF_CLEN {
                        let into = buf.as_mut_ptr();
                        unsafe { trunc_string(s, into, MSG_BUF_CLEN, MSG_BUF_LEN) };
                        s = buf.as_mut_ptr();
                    }
                    // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                    let (name, s) = unsafe { (c_str(name), c_str(s)) };
                    smsg!(0, "{name} returning {s}");
                    unsafe { xfree(tofree as *mut c_void) };
                }
            }
        });
    }

    estack_pop();
    current_sctx.set(save_current_sctx);
    if do_profiling_yes {
        unsafe { script_prof_restore(wait_start) };
    }
    drop(sandboxed);

    if p_verbose.get() >= 12 && !sourcing_entry().es_name.is_null() {
        verbose_report(|| {
            let name = sourcing_entry().es_name;
            // SAFETY: a literal text and the exec-stack entry's own name.
            let name = unsafe { c_str(name) };
            smsg!(0, "continuing in {name}");
        });
    }

    did_emsg.set(did_emsg.get() | save_did_emsg);
    depth.set(depth.get() - 1);
    for tv in &tv_to_free[..tv_to_free_len] {
        unsafe { tv_clear(*tv) };
    }
    unsafe { cleanup_function_call(fc) };

    f.uf_calls -= 1;
    // Free the function when it was deleted while it was running.
    if f.uf_calls <= 0 && f.uf_refcount <= Refcount::ZERO {
        unsafe { func_clear_free(fp, false) };
    }

    if did_save_redo {
        unsafe { restore_redobuff(&raw mut save_redo) };
    }
    restore_search_patterns();
}

/// The guard in front of [`call_user_func`]: a Lua reference is called
/// directly, and a wrong argument count or a missing `self` dictionary is
/// answered as an `FCERR_*` code rather than a call.
///
/// # Safety
/// `fp` is a live function and `funcexe` describes the call.
pub(crate) unsafe fn call_user_func_check(
    fp: *mut ufunc_T,
    argcount: c_int,
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    funcexe: *mut funcexe_T,
    selfdict: *mut dict_T,
) -> c_int {
    // SAFETY: the caller's promise -- `fp` is a live function.
    let mut f = unsafe { Uf::new(fp) };
    if f.uf_flags & FC_LUAREF != 0 {
        return unsafe { typval_exec_lua_callable(f.uf_luaref, argcount, argvars, rettv) };
    }

    if f.uf_flags & FC_RANGE != 0 && !unsafe { (*funcexe).fe_doesrange }.is_null() {
        unsafe { *(*funcexe).fe_doesrange = true };
    }
    let error = unsafe { check_user_func_argcount(fp, argcount) };
    if error != FCERR_UNKNOWN {
        return error;
    }
    if f.uf_flags & FC_DICT != 0 && selfdict.is_null() {
        return FCERR_DICT;
    }

    // SAFETY: the caller's promise -- `funcexe` describes the call.
    let (first, last) = unsafe { ((*funcexe).fe_firstline, (*funcexe).fe_lastline) };
    let dict = if f.uf_flags & FC_DICT != 0 {
        selfdict
    } else {
        ptr::null_mut()
    };
    unsafe { call_user_func(fp, argcount, argvars, rettv, first, last, dict) };
    FCERR_NONE
}

/// Report why a call could not be made.
///
/// # Safety
/// `name` is NUL-terminated.
pub(crate) unsafe fn user_func_error(error: c_int, name: *const c_char, found_var: bool) {
    match error {
        FCERR_UNKNOWN => {
            if found_var {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let name = unsafe { c_str(name) };
                semsg!("E1085: Not a callable type: {name}");
            } else {
                unsafe { emsg_funcname(e_unknown_function_str.as_ptr(), name) };
            }
        }
        FCERR_NOTMETHOD => {
            unsafe { emsg_funcname(c"E276: Cannot use function as a method: %s".as_ptr(), name) };
        }
        FCERR_DELETED => {
            unsafe { emsg_funcname(c"E933: Function was deleted: %s".as_ptr(), name) };
        }
        FCERR_TOOMANY => {
            unsafe { emsg_funcname(gettext(e_toomanyarg).as_ptr(), name) };
        }
        FCERR_TOOFEW => {
            unsafe { emsg_funcname(gettext(e_toofewarg).as_ptr(), name) };
        }
        FCERR_SCRIPT => {
            let fmt = c"E120: Using <SID> not in a script context: %s";
            unsafe { emsg_funcname(fmt.as_ptr(), name) };
        }
        FCERR_DICT => {
            let fmt = c"E725: Calling dict function without Dictionary: %s";
            unsafe { emsg_funcname(fmt.as_ptr(), name) };
        }
        _ => {}
    }
}

/// Call a Lua function by name with no arguments.
///
/// # Safety
/// `funcname` has `len` readable bytes.
pub unsafe fn call_simple_luafunc(
    funcname: *const c_char,
    len: size_t,
    rettv: *mut typval_T,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- `rettv` is the return value.
    let mut rv = unsafe { Tv::new(rettv) };
    rv.v_type = VAR_NUMBER; // the default is number zero
    rv.vval.v_number = 0;

    let mut argvars = [TV_INITIAL_VALUE; 1];
    argvars[0].v_type = VAR_UNKNOWN;
    unsafe { nlua_typval_call(funcname, len, argvars.as_mut_ptr(), 0, rettv) };
    Ok(())
}

/// Call a user function by name with no arguments, for the internal callers
/// that know there is nothing else to pass.  Answers [`Parsed::NotThis`]
/// when there is no such function.
///
/// # Safety
/// `funcname` has `len` readable bytes.
pub unsafe fn call_simple_func(
    funcname: *const c_char,
    len: size_t,
    rettv: *mut typval_T,
) -> Result<Parsed, Failed> {
    // SAFETY: the caller's promise -- `rettv` is the return value.
    let mut rv = unsafe { Tv::new(rettv) };
    let mut ret = Err(Failed);
    rv.v_type = VAR_NUMBER; // the default is number zero
    rv.vval.v_number = 0;

    let name = unsafe { xstrnsave(funcname, len) };
    let mut error = FCERR_NONE;
    let mut tofree: *mut c_char = ptr::null_mut();
    let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
    let buf = fname_buf.as_mut_ptr();
    let (freep, errp) = (&raw mut tofree, &raw mut error);
    // SAFETY: `buf` has `FLEN_FIXED + 1` bytes and the two out-parameters
    // are this frame's locals.
    let fname = unsafe { fname_trans_sid(name, buf, freep, errp) };

    // Skip "g:" before a function name.
    let is_global =
        unsafe { *fname } == b'g' as c_char && unsafe { *fname.add(1) } == b':' as c_char;
    let rfname = if is_global {
        unsafe { fname.add(2) }
    } else {
        fname
    };

    let fp = unsafe { find_func(rfname) };
    if fp.is_null() {
        ret = Ok(Parsed::NotThis);
    } else if unsafe { (*fp).uf_flags } & FC_DELETED != 0 {
        error = FCERR_DELETED;
    } else {
        let mut argvars = [TV_INITIAL_VALUE; 1];
        argvars[0].v_type = VAR_UNKNOWN;
        let mut funcexe = FUNCEXE_INIT;
        funcexe.fe_evaluate = true;
        let (args, exe) = (argvars.as_mut_ptr(), &raw mut funcexe);
        error = unsafe { call_user_func_check(fp, 0, args, rettv, exe, ptr::null_mut()) };
        if error == FCERR_NONE {
            ret = Ok(Parsed::Done);
        }
    }

    unsafe { user_func_error(error, name, false) };
    unsafe { xfree(tofree as *mut c_void) };
    unsafe { xfree(name as *mut c_void) };
    ret
}
