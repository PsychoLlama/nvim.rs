//! `:let` -- parsing the targets and performing the assignment.
//!
//! [`ex_let`] splits the command, [`ex_let_vars`] deals with the
//! `[a, b; rest]` unpack, and the four `ex_let_*` below it are one per kind
//! of target: a variable, an environment variable, an option and a register.
//! The last three implement the compound operators themselves and never
//! reach `set_var_lval`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

#[allow(unused_imports)]
use super::*;

/// The compound assignment operators, as they appear before the `=`.
const OPERATORS: &CStr = c"+-*/%.";

/// The arithmetic ones, which an environment variable and a register refuse.
const ARITHMETIC: &CStr = c"+-*/%";

/// `:let`, `:const` and (with no `=`) the listing forms.
///
/// # Safety
/// `eap` is a live `:let`/`:const` command.
pub unsafe fn ex_let(eap: *mut exarg_T) {
    unsafe {
        let is_const = (*eap).cmdidx as c_int == CMD_const as c_int;
        let mut arg = (*eap).arg;
        let mut var_count = 0;
        let mut semicolon = 0;
        let mut first = true_0;

        let argend = skip_var_list(arg, &raw mut var_count, &raw mut semicolon, false);
        if argend.is_null() {
            return;
        }
        let mut expr = skipwhite(argend);
        let concat = strncmp(expr, c"..=".as_ptr(), 3) == 0;
        let has_assign = *expr == b'=' as c_char
            || (!vim_strchr(OPERATORS.as_ptr(), *expr as uint8_t as c_int).is_null()
                && *expr.add(1) == b'=' as c_char);

        if !has_assign && !concat {
            // ":let" with no "=": list variables.
            if *arg == b'[' as c_char {
                emsg(gettext(&raw const e_invarg as *const c_char));
            } else if ends_excmd(*arg as c_int) == 0 {
                // ":let var1 var2"
                arg = list_arg_vars(eap, arg, &raw mut first) as *mut c_char;
            } else if (*eap).skip == 0 {
                // ":let" on its own.
                list_glob_vars(&raw mut first);
                list_buf_vars(&raw mut first);
                list_win_vars(&raw mut first);
                list_tab_vars(&raw mut first);
                list_script_vars(&raw mut first);
                list_func_vars(&raw mut first);
                list_vim_vars(&raw mut first);
            }
            (*eap).nextcmd = check_nextcmd(arg);
            return;
        }

        let mut rettv = TV_INITIAL_VALUE;
        if *expr == b'=' as c_char
            && *expr.add(1) == b'<' as c_char
            && *expr.add(2) == b'<' as c_char
        {
            // A here-document.
            let l = heredoc_get(eap, expr.add(3), false);
            if !l.is_null() {
                tv_list_set_ret(&raw mut rettv, l);
                if (*eap).skip == 0 {
                    let op = [b'=' as c_char, NUL];
                    ex_let_vars(
                        (*eap).arg,
                        &raw mut rettv,
                        false,
                        semicolon,
                        var_count,
                        is_const,
                        op.as_ptr(),
                    );
                }
                tv_clear(&raw mut rettv);
            }
            return;
        }

        // The operator, if any, and the expression past it.
        let mut op = [b'=' as c_char, NUL];
        if *expr != b'=' as c_char {
            if !vim_strchr(OPERATORS.as_ptr(), *expr as uint8_t as c_int).is_null() {
                // "+=", "-=", "*=", "/=", "%=" or ".="
                op[0] = *expr;
                if *expr == b'.' as c_char && *expr.add(1) == b'.' as c_char {
                    // "..=" -- one character longer than the rest.
                    expr = expr.add(1);
                }
            }
            expr = expr.add(2);
        } else {
            expr = expr.add(1);
        }
        expr = skipwhite(expr);

        if (*eap).skip != 0 {
            (*emsg_skip.ptr()) += 1;
        }
        let mut evalarg = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ptr::null_mut(),
            eval_tofree: ptr::null_mut(),
        };
        fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
        let eval_res = eval0(expr, &raw mut rettv, eap, &raw mut evalarg);
        if (*eap).skip != 0 {
            (*emsg_skip.ptr()) -= 1;
        }
        clear_evalarg(&raw mut evalarg, eap);

        if (*eap).skip == 0 && eval_res != FAIL {
            ex_let_vars(
                (*eap).arg,
                &raw mut rettv,
                false,
                semicolon,
                var_count,
                is_const,
                op.as_ptr(),
            );
        }
        if eval_res != FAIL {
            tv_clear(&raw mut rettv);
        }
    }
}

/// Assign `tv` to the target or targets at `arg_start`: one name, or the
/// `[v1, v2]` / `[v1, v2; rest]` unpack of a List.
///
/// `op` points at the characters that must follow the target(s), and names
/// the operator: `"+"`, `"-"` or `"."` for add, subtract or concatenate.
/// `semicolon` and `var_count` come from [`skip_var_list`].
///
/// # Safety
/// `arg_start` is a NUL-terminated string and `tv` a live value.
pub unsafe fn ex_let_vars(
    arg_start: *mut c_char,
    tv: *mut typval_T,
    copy: bool,
    semicolon: c_int,
    var_count: c_int,
    is_const: bool,
    op: *const c_char,
) -> c_int {
    unsafe {
        let mut arg = arg_start;
        if *arg != b'[' as c_char {
            // ":let var = expr" or ":for var in list"
            if ex_let_one(arg, tv, copy, is_const, op, op).is_null() {
                return FAIL;
            }
            return OK;
        }

        // ":let [v1, v2] = list" or ":for [v1, v2] in listlist"
        if (*tv).v_type != VAR_LIST {
            emsg(gettext(&raw const e_listreq as *const c_char));
            return FAIL;
        }
        let l = (*tv).vval.v_list;
        let len = tv_list_len(l);
        if semicolon == 0 && var_count < len {
            emsg(gettext(c"E687: Less targets than List items".as_ptr()));
            return FAIL;
        }
        if var_count - semicolon > len {
            emsg(gettext(c"E688: More targets than List items".as_ptr()));
            return FAIL;
        }
        // `l` may really be NULL, but `:let [] = v:_null_list` fails with
        // E688 or earlier before it can get here.
        debug_assert!(!l.is_null());

        let mut item = tv_list_first(l);
        let mut rest_len = tv_list_len(l) as size_t;
        while *arg != b']' as c_char {
            // Skip the whitespace after the '[', ',' or ';'.
            arg = skipwhite(arg.add(1));
            arg = ex_let_one(
                arg,
                &raw mut (*item).li_tv,
                true,
                is_const,
                c",;]".as_ptr(),
                op,
            );
            if arg.is_null() {
                return FAIL;
            }
            rest_len -= 1;
            item = (*item).li_next;

            arg = skipwhite(arg);
            if *arg == b';' as c_char {
                // The rest of the list, which may be empty, goes to the
                // variable after the ';', as a list of its own.
                let rest_list = tv_list_alloc(rest_len as ptrdiff_t);
                while !item.is_null() {
                    tv_list_append_tv(rest_list, &raw mut (*item).li_tv);
                    item = (*item).li_next;
                }
                let mut ltv = typval_T {
                    v_type: VAR_LIST,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_list: rest_list },
                };
                tv_list_ref(rest_list);

                arg = ex_let_one(
                    skipwhite(arg.add(1)),
                    &raw mut ltv,
                    false,
                    is_const,
                    c"]".as_ptr(),
                    op,
                );
                tv_clear(&raw mut ltv);
                if arg.is_null() {
                    return FAIL;
                }
                break;
            } else if *arg != b',' as c_char && *arg != b']' as c_char {
                internal_error(c"ex_let_vars()".as_ptr());
                return FAIL;
            }
        }
        OK
    }
}

/// Skip an assignable variable, or the `[var, var]` list of them, answering
/// the character past it or NULL on an error.
///
/// `var_count` counts the variables in a list and `semicolon` records
/// whether one carried a `;`.  `silent` suppresses E475.
///
/// # Safety
/// `arg` is a NUL-terminated string; `var_count` and `semicolon` are
/// writable.
pub unsafe fn skip_var_list(
    arg: *const c_char,
    var_count: *mut c_int,
    semicolon: *mut c_int,
    silent: bool,
) -> *const c_char {
    unsafe {
        if *arg != b'[' as c_char {
            return skip_var_one(arg);
        }
        // "[var, var]": find the matching ']'.
        let mut p = arg;
        loop {
            // Skip the whitespace after the '[', ';' or ','.
            p = skipwhite(p.add(1));
            let s = skip_var_one(p);
            if s == p {
                if !silent {
                    semsg(gettext(&raw const e_invarg2 as *const c_char), p);
                }
                return ptr::null();
            }
            *var_count += 1;

            p = skipwhite(s);
            if *p == b']' as c_char {
                return p.add(1);
            }
            if *p == b';' as c_char {
                if *semicolon == 1 {
                    if !silent {
                        emsg(gettext(e_double_semicolon_in_list_of_variables.as_ptr()));
                    }
                    return ptr::null();
                }
                *semicolon = 1;
            } else if *p != b',' as c_char {
                if !silent {
                    semsg(gettext(&raw const e_invarg2 as *const c_char), p);
                }
                return ptr::null();
            }
        }
    }
}

/// Skip one assignable name, including `@r`, `$VAR`, `&option`, `d.key` and
/// `l[idx]`.
///
/// # Safety
/// `arg` is a NUL-terminated string.
unsafe fn skip_var_one(arg: *const c_char) -> *const c_char {
    unsafe {
        if *arg == b'@' as c_char && *arg.add(1) != NUL {
            return arg.add(2);
        }
        let name = if *arg == b'$' as c_char || *arg == b'&' as c_char {
            arg.add(1)
        } else {
            arg
        };
        find_name_end(
            name,
            ptr::null_mut(),
            ptr::null_mut(),
            FNE_INCL_BR | FNE_CHECK_START,
        )
    }
}

/// `:let $VAR = …`.  Answers the character past the name, or NULL.
///
/// # Safety
/// `arg` points at the `$`; `tv` is a live value.
unsafe fn ex_let_env(
    mut arg: *mut c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const c_char,
    op: *const c_char,
) -> *mut c_char {
    unsafe {
        if is_const {
            emsg(gettext(
                c"E996: Cannot lock an environment variable".as_ptr(),
            ));
            return ptr::null_mut();
        }

        // Find the end of the name.
        let mut arg_end: *mut c_char = ptr::null_mut();
        arg = arg.add(1);
        let name = arg;
        let len = get_env_len(&raw mut arg as *mut *const c_char);
        if len == 0 {
            semsg(gettext(&raw const e_invarg2 as *const c_char), name.sub(1));
        } else if !op.is_null()
            && !vim_strchr(ARITHMETIC.as_ptr(), *op as uint8_t as c_int).is_null()
        {
            semsg(gettext(&raw const e_letwrong as *const c_char), op);
        } else if !endchars.is_null()
            && vim_strchr(endchars, *skipwhite(arg) as uint8_t as c_int).is_null()
        {
            emsg(gettext(e_letunexp.as_ptr()));
        } else if !check_secure() {
            // Terminate the name in place: `arg` has already moved past it.
            let mut tofree: *mut c_char = ptr::null_mut();
            let c1 = *name.offset(len as isize);
            *name.offset(len as isize) = NUL;

            let mut p = tv_get_string_chk(tv);
            if !p.is_null() && !op.is_null() && *op == b'.' as c_char {
                let s = vim_getenv(name);
                if !s.is_null() {
                    tofree = concat_str(s, p);
                    p = tofree;
                    xfree(s.cast());
                }
            }
            if !p.is_null() {
                vim_setenv_ext(name, p);
                arg_end = arg;
            }
            *name.offset(len as isize) = c1;
            xfree(tofree.cast());
        }
        arg_end
    }
}

/// `:let &opt = …`.  Answers the character past the name, or NULL.
///
/// The compound operators are implemented here rather than through
/// `eexe_mod_op`, because an option's value is an `OptVal` and not a
/// `typval_T`: the current value is read, combined, and set back.
///
/// # Safety
/// `arg` points at the `&`; `tv` is a live value.
unsafe fn ex_let_option(
    mut arg: *mut c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const c_char,
    op: *const c_char,
) -> *mut c_char {
    unsafe {
        if is_const {
            emsg(gettext(c"E996: Cannot lock an option".as_ptr()));
            return ptr::null_mut();
        }

        // Find the end of the name.
        let mut opt_idx: OptIndex = kOptAleph;
        let mut opt_flags: c_int = 0;
        let p = find_option_var_end(
            &raw mut arg as *mut *const c_char,
            &raw mut opt_idx,
            &raw mut opt_flags,
        ) as *mut c_char;
        if p.is_null()
            || (!endchars.is_null()
                && vim_strchr(endchars, *skipwhite(p) as uint8_t as c_int).is_null())
        {
            emsg(gettext(e_letunexp.as_ptr()));
            return ptr::null_mut();
        }

        // Terminate the name in place; every exit below puts it back.
        let c1 = *p;
        *p = NUL;

        let is_tty_opt = is_tty_option(arg);
        let hidden = is_option_hidden(opt_idx);
        let curval = if is_tty_opt {
            get_tty_option(arg)
        } else {
            get_option_value(opt_idx, opt_flags)
        };
        let mut newval = NIL_OPTVAL;
        let mut arg_end: *mut c_char = ptr::null_mut();

        'theend: {
            if curval.type_0 == kOptValTypeNil {
                semsg(gettext(&raw const e_unknown_option2 as *const c_char), arg);
                break 'theend;
            }
            if !op.is_null()
                && *op != b'=' as c_char
                && ((curval.type_0 != kOptValTypeString && *op == b'.' as c_char)
                    || (curval.type_0 == kOptValTypeString && *op != b'.' as c_char))
            {
                semsg(gettext(&raw const e_letwrong as *const c_char), op);
                break 'theend;
            }

            let mut error = false;
            newval = tv_to_optval(tv, opt_idx, arg, &raw mut error);
            if error {
                break 'theend;
            }
            // The current and the new value must have the same type.
            debug_assert!(curval.type_0 == newval.type_0);

            if !op.is_null() && *op != b'=' as c_char && !hidden {
                // A Number or Boolean `OptVal` as a number; a closure, so
                // that reading the union stays inside this function's one
                // `unsafe` block.
                let as_int = |v: OptVal| -> OptInt {
                    if v.type_0 == kOptValTypeNumber {
                        v.data.number
                    } else {
                        v.data.boolean as OptInt
                    }
                };
                if curval.type_0 == kOptValTypeNumber || curval.type_0 == kOptValTypeBoolean {
                    let cur_n = as_int(curval);
                    let new_n = as_int(newval);
                    let new_n = match *op as u8 {
                        b'+' => cur_n + new_n,
                        b'-' => cur_n - new_n,
                        b'*' => cur_n * new_n,
                        b'/' => num_divide(cur_n as varnumber_T, new_n as varnumber_T) as OptInt,
                        b'%' => num_modulus(cur_n as varnumber_T, new_n as varnumber_T) as OptInt,
                        // No other operator reaches here: `.` was refused
                        // above for a non-String option.
                        _ => new_n,
                    };
                    newval = if curval.type_0 == kOptValTypeNumber {
                        OptVal {
                            type_0: kOptValTypeNumber,
                            data: OptValData { number: new_n },
                        }
                    } else {
                        OptVal {
                            type_0: kOptValTypeBoolean,
                            data: OptValData {
                                boolean: tristate_from_int(new_n),
                            },
                        }
                    };
                } else if curval.type_0 == kOptValTypeString {
                    let curval_data = curval.data.string.data;
                    let newval_data = newval.data.string.data;
                    if !curval_data.is_null() && !newval_data.is_null() {
                        let newval_old = newval;
                        newval = OptVal {
                            type_0: kOptValTypeString,
                            data: OptValData {
                                string: cstr_as_string(concat_str(curval_data, newval_data)),
                            },
                        };
                        optval_free(newval_old);
                    }
                }
            }

            let err = set_option_value_handle_tty(arg, opt_idx, newval, opt_flags);
            arg_end = p;
            if !err.is_null() {
                emsg(gettext(err));
            }
        }

        *p = c1;
        optval_free(curval);
        optval_free(newval);
        arg_end
    }
}

/// Upstream's `TRISTATE_FROM_INT`: anything positive is true, zero is false,
/// and a negative number is "unset".
fn tristate_from_int(n: OptInt) -> TriState {
    if n == 0 {
        kFalse
    } else if n >= 1 {
        kTrue
    } else {
        kNone
    }
}

/// `:let @r = …`.  Answers the character past the register, or NULL.
///
/// # Safety
/// `arg` points at the `@`; `tv` is a live value.
unsafe fn ex_let_register(
    mut arg: *mut c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const c_char,
    op: *const c_char,
) -> *mut c_char {
    unsafe {
        if is_const {
            emsg(gettext(c"E996: Cannot lock a register".as_ptr()));
            return ptr::null_mut();
        }

        let mut arg_end: *mut c_char = ptr::null_mut();
        arg = arg.add(1);
        if !op.is_null() && !vim_strchr(ARITHMETIC.as_ptr(), *op as uint8_t as c_int).is_null() {
            semsg(gettext(&raw const e_letwrong as *const c_char), op);
            return arg_end;
        }
        if !endchars.is_null()
            && vim_strchr(endchars, *skipwhite(arg.add(1)) as uint8_t as c_int).is_null()
        {
            emsg(gettext(e_letunexp.as_ptr()));
            return arg_end;
        }

        // A bare "@" is the unnamed register.
        let regname = if *arg == b'@' as c_char {
            b'"' as c_int
        } else {
            *arg as c_int
        };
        let mut ptofree: *mut c_char = ptr::null_mut();
        let mut p = tv_get_string_chk(tv);
        if !p.is_null() && !op.is_null() && *op == b'.' as c_char {
            let s = get_reg_contents(regname, kGRegExprSrc as c_int) as *mut c_char;
            if !s.is_null() {
                ptofree = concat_str(s, p);
                p = ptofree;
                xfree(s.cast());
            }
        }
        if !p.is_null() {
            write_reg_contents(regname, p, strlen(p) as ssize_t, false_0);
            arg_end = arg.add(1);
        }
        xfree(ptofree.cast());
        arg_end
    }
}

/// One assignment target, dispatched on what it starts with.  Answers the
/// character past it, or NULL on an error.
///
/// # Safety
/// `arg` is a NUL-terminated string; `tv` is a live value.
unsafe fn ex_let_one(
    arg: *mut c_char,
    tv: *mut typval_T,
    copy: bool,
    is_const: bool,
    endchars: *const c_char,
    op: *const c_char,
) -> *mut c_char {
    unsafe {
        if *arg == b'$' as c_char {
            return ex_let_env(arg, tv, is_const, endchars, op);
        }
        if *arg == b'&' as c_char {
            return ex_let_option(arg, tv, is_const, endchars, op);
        }
        if *arg == b'@' as c_char {
            return ex_let_register(arg, tv, is_const, endchars, op);
        }
        if !eval_isnamec1(*arg as c_int) && *arg != b'{' as c_char {
            semsg(gettext(&raw const e_invarg2 as *const c_char), arg);
            return ptr::null_mut();
        }

        // A variable, a List or Dict item, or a Blob byte.
        let mut arg_end: *mut c_char = ptr::null_mut();
        let mut lv = LVAL_INITIAL_VALUE;
        let p = get_lval(arg, tv, &raw mut lv, false, false, 0, FNE_CHECK_START);
        if !p.is_null() && !lv.ll_name.is_null() {
            if !endchars.is_null()
                && vim_strchr(endchars, *skipwhite(p) as uint8_t as c_int).is_null()
            {
                emsg(gettext(e_letunexp.as_ptr()));
            } else {
                set_var_lval(&raw mut lv, p, tv, copy, is_const, op);
                arg_end = p;
            }
        }
        clear_lval(&raw mut lv);
        arg_end
    }
}
