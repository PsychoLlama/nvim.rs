//! `:function` itself -- defining, and the header a listing prints.
//!
//! `ex_function` decides which of the four things the command is (define,
//! list one, list a pattern, list everything), builds the `ufunc_T` and
//! installs it in the table.  `list_func_head` prints the `function
//! Name(a, b = 1, ...) dict abort range` line, which is the same text in a
//! listing and in a `:verbose` report.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of_val;
use core::ptr;

use super::*;
use crate::types::{FAIL, NUL, OK, Refcount};

/// Whether the function table changed under a listing, which means the
/// `ufunc_T` the caller is holding may be gone.  Reports E454 when it did.
pub(crate) unsafe fn function_list_modified(prev_ht_changed: c_int) -> c_int {
    if prev_ht_changed != func_table().changed() {
        emsg(gettext(E_FUNCTION_LIST_WAS_MODIFIED));
        return 1;
    }
    0
}

/// Print `function Name(a, b = 1, ...) range dict abort closure`, the head of
/// a listing and of a `:verbose` report.
///
/// # Safety
/// `fp` is a live function.
pub(crate) unsafe fn list_func_head(fp: *mut ufunc_T, indent: bool, force: bool) -> c_int {
    // SAFETY: the caller's promise -- `fp` is a live function.
    let mut f = unsafe { Uf::new(fp) };
    let prev_ht_changed = func_table().changed();

    unsafe { msg_start() };

    // Check no function was added or removed from a callback, as
    // `msg_start` may have invoked a redraw.
    if unsafe { function_list_modified(prev_ht_changed) } != 0 {
        return FAIL;
    }

    if indent {
        unsafe { msg_puts(c"   ".as_ptr()) };
    }
    let intro = if force {
        c"function! ".as_ptr()
    } else {
        c"function ".as_ptr()
    };
    unsafe { msg_puts(intro) };
    unsafe { msg_puts(printable_func_name(fp)) };
    unsafe { msg_putchar(b'(' as c_int) };

    let args = ga_strings(&f.uf_args);
    let defaults = ga_strings(&f.uf_def_args);
    // The defaults are right-aligned with the arguments: the last
    // `defaults.len()` arguments are the ones that have one.
    // Upstream computes this in `int`, where a (impossible) surplus of
    // defaults would go negative and give every argument one.
    let first_default = args.len().saturating_sub(defaults.len());
    for (j, &arg) in args.iter().enumerate() {
        if j != 0 {
            unsafe { msg_puts(c", ".as_ptr()) };
        }
        unsafe { msg_puts(arg) };
        if j >= first_default {
            unsafe { msg_puts(c" = ".as_ptr()) };
            unsafe { msg_puts(defaults[j - first_default]) };
        }
    }
    if f.uf_varargs != 0 {
        if !args.is_empty() {
            unsafe { msg_puts(c", ".as_ptr()) };
        }
        unsafe { msg_puts(c"...".as_ptr()) };
    }
    unsafe { msg_putchar(b')' as c_int) };

    for (flag, text) in [
        (FC_ABORT, c" abort"),
        (FC_RANGE, c" range"),
        (FC_DICT, c" dict"),
        (FC_CLOSURE, c" closure"),
    ] {
        if f.uf_flags & flag != 0 {
            unsafe { msg_puts(text.as_ptr()) };
        }
    }

    unsafe { msg_clr_eos() };
    if p_verbose.get() > 0 {
        unsafe { last_set_msg(f.uf_script_ctx) };
    }
    OK
}

/// `:function`.
///
/// Four commands in one: with no argument it lists everything, with a
/// `/pattern/` it lists the matches, with a bare name it lists that one, and
/// with a `(` it defines.
///
/// # Safety
/// `eap` is a live `:function` command.
pub unsafe fn ex_function(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- `eap` is the Ex command being run.
    let mut ea = unsafe { Ea::new(eap) };
    let mut line_to_free: *mut c_char = ptr::null_mut();
    let mut line_arg: *mut c_char = ptr::null_mut();
    let mut newargs = GARRAY_EMPTY;
    let mut default_args = GARRAY_EMPTY;
    let mut newlines = GARRAY_EMPTY;
    let mut varargs = 0;
    let mut flags = 0;
    let mut fp: *mut ufunc_T = ptr::null_mut();
    let mut free_fp = false;
    let mut overwrite = false;
    let mut fudi = FUNCDICT_INIT;
    // The number the nameless (dictionary) functions are given.
    static func_nr: GlobalCell<c_int> = GlobalCell::new(0);
    let mut show_block = false;

    // ":function" without argument: list functions.
    if ends_excmd(unsafe { *ea.arg } as c_int) != 0 {
        if ea.skip == 0 {
            unsafe { list_functions(ptr::null_mut()) };
        }
        ea.nextcmd = unsafe { check_nextcmd(ea.arg) };
        return;
    }

    // ":function /pat": list functions matching the pattern.
    if unsafe { *ea.arg } == b'/' as c_char {
        let p = unsafe { list_functions_matching_pat(eap) };
        ea.nextcmd = unsafe { check_nextcmd(p) };
        return;
    }

    // Get the function name.  There are these situations:
    //   func       a normal function name: "name" == func, no dict
    //   dict.func  a new dictionary entry: "name" == NULL, fd_dict set,
    //              fd_di == NULL, fd_newkey == func
    //   dict.func  an existing entry holding a Funcref: "name" == func,
    //              fd_dict and fd_di set, fd_newkey == NULL
    //   dict.func  an existing entry that is not a Funcref:
    //              "name" == NULL, fd_dict and fd_di set
    //   s:func     a script-local name; g:func is the same as func
    let mut p = ea.arg;
    let mut name =
        unsafe { save_function_name(&raw mut p, ea.skip != 0, TFN_NO_AUTOLOAD, &raw mut fudi) };
    let paren = !unsafe { vim_strchr(p, b'(' as c_int) }.is_null();
    if name.is_null() && (fudi.fd_dict.is_null() || !paren) && ea.skip == 0 {
        // Return on an invalid expression in braces, unless the
        // evaluation was cancelled by an aborting error, an interrupt or
        // an exception.
        if !aborting() {
            if !fudi.fd_newkey.is_null() {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let fd_newkey = unsafe { c_str(fudi.fd_newkey) };
                semsg!("E716: Key not present in Dictionary: \"{fd_newkey}\"");
            }
            unsafe { xfree(fudi.fd_newkey as *mut c_void) };
            return;
        }
        ea.skip = 1;
    }

    // An error in a function call while evaluating an expression in magic
    // braces should not stop the function being defined.
    let saved_did_emsg = did_emsg.get();
    did_emsg.set(0);

    'ret_free: {
        if !paren {
            // ":function func": list that one function.
            fp = unsafe { list_one_function(eap, name, p) };
            break 'ret_free;
        }

        p = unsafe { skipwhite(p) };
        if unsafe { *p } != b'(' as c_char {
            if ea.skip == 0 {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg = unsafe { c_str(ea.arg) };
                semsg!("E124: Missing '(': {arg}");
                break 'ret_free;
            }
            // Attempt to carry on by skipping some text.
            if !unsafe { vim_strchr(p, b'(' as c_int) }.is_null() {
                p = unsafe { vim_strchr(p, b'(' as c_int) };
            }
        }
        p = unsafe { skipwhite(p.add(1)) };

        unsafe { ga_init(&raw mut newargs, size_of::<*mut c_char>() as c_int, 3) };
        unsafe { ga_init(&raw mut newlines, size_of::<*mut c_char>() as c_int, 3) };

        if ea.skip == 0 {
            // Check the name of the function, unless it is a dictionary
            // function that is being overwritten.
            let arg = if name.is_null() { fudi.fd_newkey } else { name };
            // A dictionary function defined with bracket notation
            // (`obj['foo-bar']()`) is named by a *dictionary key*, which
            // need not follow the function naming rules, so the
            // identifier check is skipped for it.
            if !arg.is_null()
                && (fudi.fd_di.is_null() || !tv_is_func(unsafe { (*fudi.fd_di).di_tv }))
                && arg != fudi.fd_newkey
            {
                let mut name_base = arg;
                if unsafe { *arg } as u8 as c_int == K_SPECIAL {
                    // Skip the "<SNR>123_" mangling.
                    name_base = unsafe { vim_strchr(arg, b'_' as c_int) };
                    name_base = if name_base.is_null() {
                        unsafe { arg.add(3) }
                    } else {
                        unsafe { name_base.add(1) }
                    };
                }
                let mut i = 0;
                while unsafe { *name_base.offset(i) } != NUL as c_char
                    && (if i == 0 {
                        eval_isnamec1(unsafe { *name_base.offset(i) } as c_int)
                    } else {
                        eval_isnamec(unsafe { *name_base.offset(i) } as c_int)
                    })
                {
                    i += 1;
                }
                if unsafe { *name_base.offset(i) } != NUL as c_char {
                    unsafe { emsg_funcname(e_invarg2.as_ptr(), arg) };
                    break 'ret_free;
                }
            }
            // Disallow using the g: dict.
            if !fudi.fd_dict.is_null() && unsafe { (*fudi.fd_dict).dv_scope } == VAR_DEF_SCOPE {
                emsg(gettext(c"E862: Cannot use g: here"));
                break 'ret_free;
            }
        }

        'errret_keep: {
            let (argp, args) = (&raw mut p, &raw mut newargs);
            let (varp, defs) = (&raw mut varargs, &raw mut default_args);
            let close = b')' as c_char;
            // SAFETY: `p` walks the caller's command line and the three
            // out-parameters are this frame's locals.
            if unsafe { get_function_args(argp, close, args, varp, defs, ea.skip != 0) } != FAIL {
                if KeyTyped.get() && ui_has(kUICmdline) {
                    show_block = true;
                    unsafe { ui_ext_cmdline_block_append(0, ea.cmd) };
                }

                'erret: {
                    // Find the trailing attributes.
                    loop {
                        p = unsafe { skipwhite(p) };
                        if unsafe { strncmp(p, c"range".as_ptr(), 5) } == 0 {
                            flags |= FC_RANGE;
                            p = unsafe { p.add(5) };
                        } else if unsafe { strncmp(p, c"dict".as_ptr(), 4) } == 0 {
                            flags |= FC_DICT;
                            p = unsafe { p.add(4) };
                        } else if unsafe { strncmp(p, c"abort".as_ptr(), 5) } == 0 {
                            flags |= FC_ABORT;
                            p = unsafe { p.add(5) };
                        } else if unsafe { strncmp(p, c"closure".as_ptr(), 7) } == 0 {
                            flags |= FC_CLOSURE;
                            p = unsafe { p.add(7) };
                            if current_funccal.get().is_null() {
                                let what = if name.is_null() {
                                    c"".as_ptr()
                                } else {
                                    name as *const c_char
                                };
                                let fmt = c"E932: Closure function should not be at top level: %s";
                                unsafe { emsg_funcname(fmt.as_ptr(), what) };
                                break 'erret;
                            }
                        } else {
                            break;
                        }
                    }

                    // A line break means the body follows in the same
                    // string, which is what makes
                    // `exe "func T()\n...\nendfunc"` work.
                    if unsafe { *p } == b'\n' as c_char {
                        line_arg = unsafe { p.add(1) };
                    } else if unsafe { *p } != NUL as c_char
                        && unsafe { *p } != b'"' as c_char
                        && ea.skip == 0
                        && did_emsg.get() == 0
                    {
                        // SAFETY: a message argument the caller holds as a NUL-terminated string.
                        let p = unsafe { c_str(p) };
                        semsg!("E488: Trailing characters: {p}");
                    }

                    if KeyTyped.get() {
                        // Check whether the function already exists.
                        if ea.skip == 0 && ea.forceit == 0 {
                            if !fudi.fd_dict.is_null() && fudi.fd_newkey.is_null() {
                                emsg(gettext(E_FUNCDICT));
                            } else if !name.is_null() && !unsafe { find_func(name) }.is_null() {
                                unsafe { emsg_funcname(E_FUNCEXTS.as_ptr(), name) };
                            }
                        }
                        if ea.skip == 0 && did_emsg.get() != 0 {
                            break 'erret;
                        }
                        if !ui_has(kUICmdline) {
                            // Don't overwrite the function name.
                            unsafe { msg_putchar(b'\n' as c_int) };
                        }
                        cmdline_row.set(msg_row.get());
                    }

                    // Save the starting line number.
                    let sourcing_lnum_top = sourcing_lnum();

                    // Do not define the function when reading the body
                    // fails, and not when skipping.
                    let (lines, freep) = (&raw mut newlines, &raw mut line_to_free);
                    // SAFETY: `eap` is the live `:function` and both
                    // out-parameters are this frame's locals.
                    let read =
                        unsafe { get_function_body(eap, lines, line_arg, freep, show_block) };
                    if read == FAIL || ea.skip != 0 {
                        break 'erret;
                    }

                    let mut namelen: size_t = 0;
                    if fudi.fd_dict.is_null() {
                        let mut ht: *mut hashtab_T = ptr::null_mut();
                        let v = unsafe { find_var(name, strlen(name), &raw mut ht, false) };
                        if !v.is_null() && unsafe { (*v).di_tv.v_type } == VAR_FUNC {
                            let clash = c"E707: Function name conflicts with variable: %s";
                            unsafe { emsg_funcname(clash.as_ptr(), name) };
                            break 'erret;
                        }
                        fp = unsafe { find_func(name) };
                        if !fp.is_null() {
                            // A function can be replaced with "function!"
                            // and when sourcing the same script again,
                            // but only once.
                            if ea.forceit == 0
                                && (unsafe { (*fp).uf_script_ctx.sc_sid }
                                    != current_sctx.get().sc_sid
                                    || unsafe { (*fp).uf_script_ctx.sc_seq }
                                        == current_sctx.get().sc_seq)
                            {
                                unsafe { emsg_funcname(E_FUNCEXTS.as_ptr(), name) };
                                break 'errret_keep;
                            }
                            if unsafe { (*fp).uf_calls } > 0 {
                                let busy = c"E127: Cannot redefine function %s: It is in use";
                                unsafe { emsg_funcname(busy.as_ptr(), name) };
                                break 'errret_keep;
                            }
                            if unsafe { (*fp).uf_refcount }.is_shared() {
                                // Referenced somewhere: don't redefine
                                // it, create a new one beside it.
                                unsafe { (*fp).uf_refcount.release() };
                                unsafe { (*fp).uf_flags |= FC_REMOVED };
                                fp = ptr::null_mut();
                                overwrite = true;
                            } else {
                                // Redefine the existing function, keeping
                                // its expanded name.
                                let exp_name = unsafe { (*fp).uf_name_exp };
                                unsafe { xfree(name as *mut c_void) };
                                name = ptr::null_mut();
                                unsafe { (*fp).uf_name_exp = ptr::null_mut() };
                                unsafe { func_clear_items(fp) };
                                unsafe { (*fp).uf_name_exp = exp_name };
                                unsafe { (*fp).uf_profiling = 0 };
                                unsafe { (*fp).uf_prof_initialized = 0 };
                            }
                        }
                    } else {
                        fp = ptr::null_mut();
                        if fudi.fd_newkey.is_null() && ea.forceit == 0 {
                            emsg(gettext(E_FUNCDICT));
                            break 'erret;
                        }
                        let locked = if fudi.fd_di.is_null() {
                            // Can't add a function to a locked dictionary.
                            unsafe { value_check_lock((*fudi.fd_dict).dv_lock, ea.arg, TV_CSTRING) }
                        } else {
                            // Can't change an existing function if it is
                            // locked.
                            unsafe {
                                value_check_lock((*fudi.fd_di).di_tv.v_lock, ea.arg, TV_CSTRING)
                            }
                        };
                        if locked {
                            break 'erret;
                        }

                        // Give the function a sequential number.  It can
                        // only be used through a Funcref.
                        let mut numbuf: [c_char; 65] = [0; 65];
                        unsafe { xfree(name as *mut c_void) };
                        func_nr.set(func_nr.get() + 1);
                        let (into, cap) = (numbuf.as_mut_ptr(), size_of_val(&numbuf));
                        let nr = func_nr.get();
                        namelen = unsafe { snprintf(into, cap, c"%d".as_ptr(), nr) } as size_t;
                        name = unsafe { xmemdupz(numbuf.as_ptr() as *const c_void, namelen) }
                            as *mut c_char;
                    }

                    if fp.is_null() {
                        if fudi.fd_dict.is_null()
                            && !unsafe { vim_strchr(name, AUTOLOAD_CHAR) }.is_null()
                        {
                            // Check that the autoload name matches the
                            // script name.
                            let mut j = FAIL;
                            let sourcing_name = sourcing_entry().es_name;
                            if !sourcing_name.is_null() {
                                let scriptname = unsafe { autoload_name(name, strlen(name)) };
                                p = unsafe { vim_strchr(scriptname, b'/' as c_int) };
                                let plen = unsafe { strlen(p) } as isize;
                                let slen = unsafe { strlen(sourcing_name) } as isize;
                                if slen > plen
                                    && unsafe {
                                        path_fnamecmp(p, sourcing_name.offset(slen - plen))
                                    } == 0
                                {
                                    j = OK;
                                }
                                unsafe { xfree(scriptname as *mut c_void) };
                            }
                            if j == FAIL {
                                // SAFETY: the function name being defined.
                                let shown = unsafe { c_str(name) };
                                semsg!(
                                    "E746: Function name does not match script file name: {shown}"
                                );
                                break 'erret;
                            }
                        }

                        if namelen == 0 {
                            namelen = unsafe { strlen(name) };
                        }
                        fp = unsafe { alloc_ufunc(name, namelen) };

                        if !fudi.fd_dict.is_null() {
                            if fudi.fd_di.is_null() {
                                // Add a new dict entry.
                                fudi.fd_di = unsafe { tv_dict_item_alloc(fudi.fd_newkey) };
                                if unsafe { tv_dict_add(fudi.fd_dict, fudi.fd_di) } == FAIL {
                                    unsafe { xfree(fudi.fd_di as *mut c_void) };
                                    unsafe { xfree(fp as *mut c_void) };
                                    fp = ptr::null_mut();
                                    break 'erret;
                                }
                            } else {
                                // Overwrite the existing dict entry.
                                unsafe { tv_clear(&raw mut (*fudi.fd_di).di_tv) };
                            }
                            unsafe { (*fudi.fd_di).di_tv.v_type = VAR_FUNC };
                            unsafe {
                                (*fudi.fd_di).di_tv.vval.v_string =
                                    xmemdupz(name as *const c_void, namelen) as *mut c_char
                            };

                            // Behave as though "dict" had been used.
                            flags |= FC_DICT;
                        }

                        // Insert the new function in the function list.
                        if overwrite {
                            let hi = unsafe { func_table().find(name) };
                            unsafe { (*hi).hi_key = uf_name_ptr(fp) };
                        } else if unsafe { func_table().add(uf_name_ptr(fp)) } == FAIL {
                            free_fp = true;
                            break 'erret;
                        }
                        unsafe { (*fp).uf_refcount = Refcount::ONE };
                    }

                    unsafe { (*fp).uf_args = newargs };
                    unsafe { (*fp).uf_def_args = default_args };
                    unsafe { (*fp).uf_lines = newlines };
                    if flags & FC_CLOSURE != 0 {
                        unsafe { register_closure(fp) };
                    } else {
                        unsafe { (*fp).uf_scoped = ptr::null_mut() };
                    }

                    if unsafe { prof_def_func() } {
                        unsafe { func_do_profile(fp) };
                    }
                    unsafe { (*fp).uf_varargs = varargs };
                    if sandbox.get() != 0 {
                        flags |= FC_SANDBOX;
                    }
                    unsafe { (*fp).uf_flags = flags };
                    unsafe { (*fp).uf_calls = 0 };
                    unsafe { (*fp).uf_script_ctx = current_sctx.get() };
                    unsafe { (*fp).uf_script_ctx.sc_lnum += sourcing_lnum_top };
                    unsafe { nlua_set_sctx(&raw mut (*fp).uf_script_ctx) };
                    break 'ret_free;
                }

                // erret: the garrays below were handed to `fp`, and are
                // cleared at `errret_keep`, so give it empty ones.
                if !fp.is_null() {
                    // SAFETY: `fp` is the function just allocated.
                    let slot = size_of::<*mut c_char>() as c_int;
                    unsafe { ga_init(&raw mut (*fp).uf_args, slot, 1) };
                    unsafe { ga_init(&raw mut (*fp).uf_def_args, slot, 1) };
                }
            }

            // errret_2:
            if !fp.is_null() {
                unsafe { xfree((*fp).uf_name_exp as *mut c_void) };
                unsafe { (*fp).uf_name_exp = ptr::null_mut() };
            }
            if free_fp {
                unsafe { xfree(fp as *mut c_void) };
                fp = ptr::null_mut();
            }
        }

        // errret_keep:
        unsafe { ga_clear_strings(&raw mut newargs) };
        unsafe { ga_clear_strings(&raw mut default_args) };
        unsafe { ga_clear_strings(&raw mut newlines) };
    }

    // ret_free:
    unsafe { xfree(line_to_free as *mut c_void) };
    unsafe { xfree(fudi.fd_newkey as *mut c_void) };
    unsafe { xfree(name as *mut c_void) };
    did_emsg.set(did_emsg.get() | saved_did_emsg);
    if show_block {
        ui_ext_cmdline_block_leave();
    }
}
