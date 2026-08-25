//! `:function` itself -- defining, and the header a listing prints.
//!
//! `ex_function` decides which of the four things the command is (define,
//! list one, list a pattern, list everything), builds the `ufunc_T` and
//! installs it in the table.  `list_func_head` prints the `function
//! Name(a, b = 1, ...) dict abort range` line, which is the same text in a
//! listing and in a `:verbose` report.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of_val;
use core::ptr;

use super::*;
use crate::types::{FAIL, NUL, OK};

/// Whether the function table changed under a listing, which means the
/// `ufunc_T` the caller is holding may be gone.  Reports E454 when it did.
pub(crate) unsafe fn function_list_modified(prev_ht_changed: c_int) -> c_int {
    unsafe {
        if prev_ht_changed != func_table().changed() {
            emsg(gettext(E_FUNCTION_LIST_WAS_MODIFIED.as_ptr()));
            return 1;
        }
        0
    }
}

/// Print `function Name(a, b = 1, ...) range dict abort closure`, the head of
/// a listing and of a `:verbose` report.
///
/// # Safety
/// `fp` is a live function.
pub(crate) unsafe fn list_func_head(fp: *mut ufunc_T, indent: bool, force: bool) -> c_int {
    unsafe {
        let prev_ht_changed = func_table().changed();

        msg_start();

        // Check no function was added or removed from a callback, as
        // `msg_start` may have invoked a redraw.
        if function_list_modified(prev_ht_changed) != 0 {
            return FAIL;
        }

        if indent {
            msg_puts(c"   ".as_ptr());
        }
        msg_puts(if force {
            c"function! ".as_ptr()
        } else {
            c"function ".as_ptr()
        });
        msg_puts(printable_func_name(fp));
        msg_putchar(b'(' as c_int);

        let args = ga_strings(&(*fp).uf_args);
        let defaults = ga_strings(&(*fp).uf_def_args);
        // The defaults are right-aligned with the arguments: the last
        // `defaults.len()` arguments are the ones that have one.
        // Upstream computes this in `int`, where a (impossible) surplus of
        // defaults would go negative and give every argument one.
        let first_default = args.len().saturating_sub(defaults.len());
        for (j, &arg) in args.iter().enumerate() {
            if j != 0 {
                msg_puts(c", ".as_ptr());
            }
            msg_puts(arg);
            if j >= first_default {
                msg_puts(c" = ".as_ptr());
                msg_puts(defaults[j - first_default]);
            }
        }
        if (*fp).uf_varargs != 0 {
            if !args.is_empty() {
                msg_puts(c", ".as_ptr());
            }
            msg_puts(c"...".as_ptr());
        }
        msg_putchar(b')' as c_int);

        for (flag, text) in [
            (FC_ABORT, c" abort"),
            (FC_RANGE, c" range"),
            (FC_DICT, c" dict"),
            (FC_CLOSURE, c" closure"),
        ] {
            if (*fp).uf_flags & flag != 0 {
                msg_puts(text.as_ptr());
            }
        }

        msg_clr_eos();
        if p_verbose.get() > 0 {
            last_set_msg((*fp).uf_script_ctx);
        }
        OK
    }
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
    unsafe {
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
        if ends_excmd(*(*eap).arg as c_int) != 0 {
            if (*eap).skip == 0 {
                list_functions(ptr::null_mut());
            }
            (*eap).nextcmd = check_nextcmd((*eap).arg);
            return;
        }

        // ":function /pat": list functions matching the pattern.
        if *(*eap).arg == b'/' as c_char {
            let p = list_functions_matching_pat(eap);
            (*eap).nextcmd = check_nextcmd(p);
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
        let mut p = (*eap).arg;
        let mut name =
            save_function_name(&raw mut p, (*eap).skip != 0, TFN_NO_AUTOLOAD, &raw mut fudi);
        let paren = !vim_strchr(p, b'(' as c_int).is_null();
        if name.is_null() && (fudi.fd_dict.is_null() || !paren) && (*eap).skip == 0 {
            // Return on an invalid expression in braces, unless the
            // evaluation was cancelled by an aborting error, an interrupt or
            // an exception.
            if !aborting() {
                if !fudi.fd_newkey.is_null() {
                    semsg_c!(
                        gettext(&raw const e_dictkey as *const c_char),
                        fudi.fd_newkey,
                    );
                }
                xfree(fudi.fd_newkey as *mut c_void);
                return;
            }
            (*eap).skip = 1;
        }

        // An error in a function call while evaluating an expression in magic
        // braces should not stop the function being defined.
        let saved_did_emsg = did_emsg.get();
        did_emsg.set(0);

        'ret_free: {
            if !paren {
                // ":function func": list that one function.
                fp = list_one_function(eap, name, p);
                break 'ret_free;
            }

            p = skipwhite(p);
            if *p != b'(' as c_char {
                if (*eap).skip == 0 {
                    semsg_c!(gettext(c"E124: Missing '(': %s".as_ptr()), (*eap).arg);
                    break 'ret_free;
                }
                // Attempt to carry on by skipping some text.
                if !vim_strchr(p, b'(' as c_int).is_null() {
                    p = vim_strchr(p, b'(' as c_int);
                }
            }
            p = skipwhite(p.add(1));

            ga_init(&raw mut newargs, size_of::<*mut c_char>() as c_int, 3);
            ga_init(&raw mut newlines, size_of::<*mut c_char>() as c_int, 3);

            if (*eap).skip == 0 {
                // Check the name of the function, unless it is a dictionary
                // function that is being overwritten.
                let arg = if name.is_null() { fudi.fd_newkey } else { name };
                // A dictionary function defined with bracket notation
                // (`obj['foo-bar']()`) is named by a *dictionary key*, which
                // need not follow the function naming rules, so the
                // identifier check is skipped for it.
                if !arg.is_null()
                    && (fudi.fd_di.is_null() || !tv_is_func((*fudi.fd_di).di_tv))
                    && arg != fudi.fd_newkey
                {
                    let mut name_base = arg;
                    if *arg as u8 as c_int == K_SPECIAL {
                        // Skip the "<SNR>123_" mangling.
                        name_base = vim_strchr(arg, b'_' as c_int);
                        name_base = if name_base.is_null() {
                            arg.add(3)
                        } else {
                            name_base.add(1)
                        };
                    }
                    let mut i = 0;
                    while *name_base.offset(i) != NUL as c_char
                        && (if i == 0 {
                            eval_isnamec1(*name_base.offset(i) as c_int)
                        } else {
                            eval_isnamec(*name_base.offset(i) as c_int)
                        })
                    {
                        i += 1;
                    }
                    if *name_base.offset(i) != NUL as c_char {
                        emsg_funcname(&raw const e_invarg2 as *const c_char, arg);
                        break 'ret_free;
                    }
                }
                // Disallow using the g: dict.
                if !fudi.fd_dict.is_null() && (*fudi.fd_dict).dv_scope == VAR_DEF_SCOPE {
                    emsg(gettext(c"E862: Cannot use g: here".as_ptr()));
                    break 'ret_free;
                }
            }

            'errret_keep: {
                if get_function_args(
                    &raw mut p,
                    b')' as c_char,
                    &raw mut newargs,
                    &raw mut varargs,
                    &raw mut default_args,
                    (*eap).skip != 0,
                ) != FAIL
                {
                    if KeyTyped.get() && ui_has(kUICmdline) {
                        show_block = true;
                        ui_ext_cmdline_block_append(0, (*eap).cmd);
                    }

                    'erret: {
                        // Find the trailing attributes.
                        loop {
                            p = skipwhite(p);
                            if strncmp(p, c"range".as_ptr(), 5) == 0 {
                                flags |= FC_RANGE;
                                p = p.add(5);
                            } else if strncmp(p, c"dict".as_ptr(), 4) == 0 {
                                flags |= FC_DICT;
                                p = p.add(4);
                            } else if strncmp(p, c"abort".as_ptr(), 5) == 0 {
                                flags |= FC_ABORT;
                                p = p.add(5);
                            } else if strncmp(p, c"closure".as_ptr(), 7) == 0 {
                                flags |= FC_CLOSURE;
                                p = p.add(7);
                                if current_funccal.get().is_null() {
                                    emsg_funcname(
                                        c"E932: Closure function should not be at top level: %s"
                                            .as_ptr(),
                                        if name.is_null() {
                                            c"".as_ptr()
                                        } else {
                                            name as *const c_char
                                        },
                                    );
                                    break 'erret;
                                }
                            } else {
                                break;
                            }
                        }

                        // A line break means the body follows in the same
                        // string, which is what makes
                        // `exe "func T()\n...\nendfunc"` work.
                        if *p == b'\n' as c_char {
                            line_arg = p.add(1);
                        } else if *p != NUL as c_char
                            && *p != b'"' as c_char
                            && (*eap).skip == 0
                            && did_emsg.get() == 0
                        {
                            semsg_c!(gettext(&raw const e_trailing_arg as *const c_char), p);
                        }

                        if KeyTyped.get() {
                            // Check whether the function already exists.
                            if (*eap).skip == 0 && (*eap).forceit == 0 {
                                if !fudi.fd_dict.is_null() && fudi.fd_newkey.is_null() {
                                    emsg(gettext(E_FUNCDICT.as_ptr()));
                                } else if !name.is_null() && !find_func(name).is_null() {
                                    emsg_funcname(E_FUNCEXTS.as_ptr(), name);
                                }
                            }
                            if (*eap).skip == 0 && did_emsg.get() != 0 {
                                break 'erret;
                            }
                            if !ui_has(kUICmdline) {
                                // Don't overwrite the function name.
                                msg_putchar(b'\n' as c_int);
                            }
                            cmdline_row.set(msg_row.get());
                        }

                        // Save the starting line number.
                        let sourcing_lnum_top = sourcing_lnum();

                        // Do not define the function when reading the body
                        // fails, and not when skipping.
                        if get_function_body(
                            eap,
                            &raw mut newlines,
                            line_arg,
                            &raw mut line_to_free,
                            show_block,
                        ) == FAIL
                            || (*eap).skip != 0
                        {
                            break 'erret;
                        }

                        let mut namelen: size_t = 0;
                        if fudi.fd_dict.is_null() {
                            let mut ht: *mut hashtab_T = ptr::null_mut();
                            let v = find_var(name, strlen(name), &raw mut ht, false);
                            if !v.is_null() && (*v).di_tv.v_type == VAR_FUNC {
                                emsg_funcname(
                                    c"E707: Function name conflicts with variable: %s".as_ptr(),
                                    name,
                                );
                                break 'erret;
                            }
                            fp = find_func(name);
                            if !fp.is_null() {
                                // A function can be replaced with "function!"
                                // and when sourcing the same script again,
                                // but only once.
                                if (*eap).forceit == 0
                                    && ((*fp).uf_script_ctx.sc_sid != current_sctx.get().sc_sid
                                        || (*fp).uf_script_ctx.sc_seq == current_sctx.get().sc_seq)
                                {
                                    emsg_funcname(E_FUNCEXTS.as_ptr(), name);
                                    break 'errret_keep;
                                }
                                if (*fp).uf_calls > 0 {
                                    emsg_funcname(
                                        c"E127: Cannot redefine function %s: It is in use".as_ptr(),
                                        name,
                                    );
                                    break 'errret_keep;
                                }
                                if (*fp).uf_refcount > 1 {
                                    // Referenced somewhere: don't redefine
                                    // it, create a new one beside it.
                                    (*fp).uf_refcount -= 1;
                                    (*fp).uf_flags |= FC_REMOVED;
                                    fp = ptr::null_mut();
                                    overwrite = true;
                                } else {
                                    // Redefine the existing function, keeping
                                    // its expanded name.
                                    let exp_name = (*fp).uf_name_exp;
                                    xfree(name as *mut c_void);
                                    name = ptr::null_mut();
                                    (*fp).uf_name_exp = ptr::null_mut();
                                    func_clear_items(fp);
                                    (*fp).uf_name_exp = exp_name;
                                    (*fp).uf_profiling = 0;
                                    (*fp).uf_prof_initialized = 0;
                                }
                            }
                        } else {
                            fp = ptr::null_mut();
                            if fudi.fd_newkey.is_null() && (*eap).forceit == 0 {
                                emsg(gettext(E_FUNCDICT.as_ptr()));
                                break 'erret;
                            }
                            let locked = if fudi.fd_di.is_null() {
                                // Can't add a function to a locked dictionary.
                                value_check_lock((*fudi.fd_dict).dv_lock, (*eap).arg, TV_CSTRING)
                            } else {
                                // Can't change an existing function if it is
                                // locked.
                                value_check_lock((*fudi.fd_di).di_tv.v_lock, (*eap).arg, TV_CSTRING)
                            };
                            if locked {
                                break 'erret;
                            }

                            // Give the function a sequential number.  It can
                            // only be used through a Funcref.
                            let mut numbuf: [c_char; 65] = [0; 65];
                            xfree(name as *mut c_void);
                            func_nr.set(func_nr.get() + 1);
                            namelen = snprintf(
                                numbuf.as_mut_ptr(),
                                size_of_val(&numbuf),
                                c"%d".as_ptr(),
                                func_nr.get(),
                            ) as size_t;
                            name =
                                xmemdupz(numbuf.as_ptr() as *const c_void, namelen) as *mut c_char;
                        }

                        if fp.is_null() {
                            if fudi.fd_dict.is_null() && !vim_strchr(name, AUTOLOAD_CHAR).is_null()
                            {
                                // Check that the autoload name matches the
                                // script name.
                                let mut j = FAIL;
                                let sourcing_name = sourcing_entry().es_name;
                                if !sourcing_name.is_null() {
                                    let scriptname = autoload_name(name, strlen(name));
                                    p = vim_strchr(scriptname, b'/' as c_int);
                                    let plen = strlen(p) as isize;
                                    let slen = strlen(sourcing_name) as isize;
                                    if slen > plen
                                        && path_fnamecmp(p, sourcing_name.offset(slen - plen)) == 0
                                    {
                                        j = OK;
                                    }
                                    xfree(scriptname as *mut c_void);
                                }
                                if j == FAIL {
                                    semsg_c!(
                                        gettext(
                                            c"E746: Function name does not match script file name: %s"
                                                .as_ptr(),
                                        ),
                                        name,
                                    );
                                    break 'erret;
                                }
                            }

                            if namelen == 0 {
                                namelen = strlen(name);
                            }
                            fp = alloc_ufunc(name, namelen);

                            if !fudi.fd_dict.is_null() {
                                if fudi.fd_di.is_null() {
                                    // Add a new dict entry.
                                    fudi.fd_di = tv_dict_item_alloc(fudi.fd_newkey);
                                    if tv_dict_add(fudi.fd_dict, fudi.fd_di) == FAIL {
                                        xfree(fudi.fd_di as *mut c_void);
                                        xfree(fp as *mut c_void);
                                        fp = ptr::null_mut();
                                        break 'erret;
                                    }
                                } else {
                                    // Overwrite the existing dict entry.
                                    tv_clear(&raw mut (*fudi.fd_di).di_tv);
                                }
                                (*fudi.fd_di).di_tv.v_type = VAR_FUNC;
                                (*fudi.fd_di).di_tv.vval.v_string =
                                    xmemdupz(name as *const c_void, namelen) as *mut c_char;

                                // Behave as though "dict" had been used.
                                flags |= FC_DICT;
                            }

                            // Insert the new function in the function list.
                            if overwrite {
                                let hi = func_table().find(name);
                                (*hi).hi_key = uf_name_ptr(fp);
                            } else if func_table().add(uf_name_ptr(fp)) == FAIL {
                                free_fp = true;
                                break 'erret;
                            }
                            (*fp).uf_refcount = 1;
                        }

                        (*fp).uf_args = newargs;
                        (*fp).uf_def_args = default_args;
                        (*fp).uf_lines = newlines;
                        if flags & FC_CLOSURE != 0 {
                            register_closure(fp);
                        } else {
                            (*fp).uf_scoped = ptr::null_mut();
                        }

                        if prof_def_func() {
                            func_do_profile(fp);
                        }
                        (*fp).uf_varargs = varargs;
                        if sandbox.get() != 0 {
                            flags |= FC_SANDBOX;
                        }
                        (*fp).uf_flags = flags;
                        (*fp).uf_calls = 0;
                        (*fp).uf_script_ctx = current_sctx.get();
                        (*fp).uf_script_ctx.sc_lnum += sourcing_lnum_top;
                        nlua_set_sctx(&raw mut (*fp).uf_script_ctx);
                        break 'ret_free;
                    }

                    // erret: the garrays below were handed to `fp`, and are
                    // cleared at `errret_keep`, so give it empty ones.
                    if !fp.is_null() {
                        ga_init(&raw mut (*fp).uf_args, size_of::<*mut c_char>() as c_int, 1);
                        ga_init(
                            &raw mut (*fp).uf_def_args,
                            size_of::<*mut c_char>() as c_int,
                            1,
                        );
                    }
                }

                // errret_2:
                if !fp.is_null() {
                    xfree((*fp).uf_name_exp as *mut c_void);
                    (*fp).uf_name_exp = ptr::null_mut();
                }
                if free_fp {
                    xfree(fp as *mut c_void);
                    fp = ptr::null_mut();
                }
            }

            // errret_keep:
            ga_clear_strings(&raw mut newargs);
            ga_clear_strings(&raw mut default_args);
            ga_clear_strings(&raw mut newlines);
        }

        // ret_free:
        xfree(line_to_free as *mut c_void);
        xfree(fudi.fd_newkey as *mut c_void);
        xfree(name as *mut c_void);
        did_emsg.set(did_emsg.get() | saved_did_emsg);
        if show_block {
            ui_ext_cmdline_block_leave();
        }
    }
}
