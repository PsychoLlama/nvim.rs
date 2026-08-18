//! `b:`, `w:` and `t:` from somewhere else.
//!
//! [`get_var_from`] and [`setwinvar`] switch to the requested buffer, window
//! or tabpage, do the lookup there and switch back; the `f_*` entries below
//! them are the Vimscript builtins that call them.  The `&option` spelling
//! of a name lands in [`tv_to_optval`]/[`optval_as_tv`] instead.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;

/// The zeroed `switchwin_T` [`switch_win`] fills in.
const SWITCHWIN_INITIAL_VALUE: switchwin_T = switchwin_T {
    sw_curwin: ptr::null_mut(),
    sw_curtab: ptr::null_mut(),
    sw_same_win: false,
    sw_visual_active: false,
};

/// `getbufvar()`, `getwinvar()`, `gettabvar()` and `gettabwinvar()`: read
/// `varname` in the scope `htname` names, falling back to `deftv`.
///
/// An empty `varname` is the whole scope as a dictionary, and one starting
/// with `&` is an option rather than a variable.  Errors are suppressed
/// throughout: these functions answer the default instead.
///
/// # Safety
/// `rettv` is writable and holds nothing; `deftv` is a live value;
/// `tp`/`win`/`buf` are live or NULL.
unsafe fn get_var_from(
    mut varname: *const c_char,
    rettv: *mut typval_T,
    deftv: *mut typval_T,
    htname: c_int,
    tp: *mut tabpage_T,
    win: *mut win_T,
    buf: *mut buf_T,
) {
    unsafe {
        let mut done = false;
        let do_change_curbuf = !buf.is_null() && htname == b'b' as c_int;

        (*emsg_off.ptr()) += 1;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ptr::null_mut();

        if !varname.is_null()
            && !tp.is_null()
            && !win.is_null()
            && (htname != b'b' as c_int || !buf.is_null())
        {
            // Make `win` current, and its tab page with it, or the window is
            // not valid. Only when needed, since it blocks autocommands --
            // and not at all with a buffer in hand, where `curbuf` is saved
            // and restored directly instead.
            let need_switch_win = !(tp == curtab.get() && win == curwin.get()) && !do_change_curbuf;
            let mut switchwin = SWITCHWIN_INITIAL_VALUE;
            if !need_switch_win || switch_win(&raw mut switchwin, win, tp, true) == OK {
                if *varname == b'&' as c_char && htname != b't' as c_int {
                    // An option: read it from the right buffer.
                    let save_curbuf = curbuf.get();
                    if do_change_curbuf {
                        curbuf.set(buf);
                    }
                    if *varname.add(1) == NUL {
                        // A bare "&": every window- or buffer-local option.
                        let opts = get_winbuf_options(c_int::from(htname == b'b' as c_int));
                        if !opts.is_null() {
                            tv_dict_set_ret(rettv, opts);
                            done = true;
                        }
                    } else if eval_option(&raw mut varname, rettv, true) == OK {
                        done = true;
                    }
                    curbuf.set(save_curbuf);
                } else if *varname == NUL {
                    // An empty name: the whole scope as a dictionary.
                    let v: *const ScopeDictDictItem = match htname as u8 {
                        b'b' => &raw mut (*buf).b_bufvar,
                        b'w' => &raw mut (*win).w_winvar,
                        _ => &raw mut (*tp).tp_winvar,
                    };
                    tv_copy(&raw const (*v).di_tv, rettv);
                    done = true;
                } else {
                    let ht = match htname as u8 {
                        b'b' => &raw mut (*(*buf).b_vars).dv_hashtab,
                        b'w' => &raw mut (*(*win).w_vars).dv_hashtab,
                        _ => &raw mut (*(*tp).tp_vars).dv_hashtab,
                    };
                    let v = find_var_in_ht(ht, htname, varname, strlen(varname), false);
                    if !v.is_null() {
                        tv_copy(&raw const (*v).di_tv, rettv);
                        done = true;
                    }
                }
            }
            if need_switch_win {
                restore_win(&raw mut switchwin, true);
            }
        }

        if !done && (*deftv).v_type != VAR_UNKNOWN {
            tv_copy(deftv, rettv);
        }
        (*emsg_off.ptr()) -= 1;
    }
}

/// `getwinvar()`, and `gettabwinvar()` with `off` 1 -- which is where the
/// extra leading tab-page argument goes.
///
/// # Safety
/// `argvars` holds at least `off + 3` values; `rettv` is writable.
unsafe fn getwinvar(argvars: *mut typval_T, rettv: *mut typval_T, off: c_int) {
    unsafe {
        let tp = if off == 1 {
            find_tabpage(tv_get_number_chk(argvars, ptr::null_mut()) as c_int)
        } else {
            curtab.get()
        };
        let win = find_win_by_nr(argvars.offset(off as isize), tp);
        let varname = tv_get_string_chk(argvars.offset((off + 1) as isize));
        get_var_from(
            varname,
            rettv,
            argvars.offset((off + 2) as isize),
            b'w' as c_int,
            tp,
            win,
            ptr::null_mut(),
        );
    }
}

/// `tv` as the value of option `opt_idx`, or `NIL_OPTVAL` with `error` set.
///
/// The option's declared types decide the conversion; a Funcref is accepted
/// (as its name) only by an option that takes one.
///
/// # Safety
/// `tv` is a live value, `option` a NUL-terminated name matching `opt_idx`,
/// and `error` writable or NULL.
pub(crate) unsafe fn tv_to_optval(
    tv: *mut typval_T,
    opt_idx: OptIndex,
    option: *const c_char,
    error: *mut bool,
) -> OptVal {
    unsafe {
        let mut nbuf = [0 as c_char; 65];
        let mut err = false;
        let is_tty_opt = is_tty_option(option);
        let option_has_bool = !is_tty_opt && option_has_type(opt_idx, kOptValTypeBoolean);
        let option_has_num = !is_tty_opt && option_has_type(opt_idx, kOptValTypeNumber);
        let option_has_str = is_tty_opt || option_has_type(opt_idx, kOptValTypeString);

        let value = if !is_tty_opt
            && (*get_option(opt_idx)).flags & kOptFlagFunc as uint32_t != 0
            && tv_is_func(*tv)
        {
            // An option that takes a function reference or a lambda stores
            // the name of one.
            let strval = encode_tv2string(tv, ptr::null_mut());
            err = strval.is_null();
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(strval),
                },
            }
        } else if option_has_bool || option_has_num {
            let n = if option_has_num {
                tv_get_number_chk(tv, &raw mut err)
            } else {
                tv_get_bool_chk(tv, &raw mut err)
            };
            // A String answers 0 both when it *is* zero and when it is not a
            // number at all, so a zero from a String has to be re-read: it
            // is only honest if the string is all '0's and nothing else.
            if !err && (*tv).v_type == VAR_STRING && n == 0 {
                let s = (*tv).vval.v_string;
                let mut idx = 0;
                while !s.is_null() && *s.add(idx) == b'0' as c_char {
                    idx += 1;
                }
                if idx == 0 || *s.add(idx) != NUL {
                    err = true;
                    semsg_c!(
                        gettext(c"E521: Number required: &%s = '%s'".as_ptr()),
                        option,
                        if s.is_null() { c"".as_ptr() } else { s },
                    );
                }
            }
            if option_has_num {
                OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData { number: n },
                }
            } else {
                OptVal {
                    type_0: kOptValTypeBoolean,
                    data: OptValData {
                        boolean: tristate_from_int(n),
                    },
                }
            }
        } else if option_has_str {
            // Never set a string option to `v:true` or `v:null`.
            if (*tv).v_type != VAR_BOOL && (*tv).v_type != VAR_SPECIAL {
                let strval = tv_get_string_buf_chk(tv, nbuf.as_mut_ptr());
                err = strval.is_null();
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_to_string(strval),
                    },
                }
            } else {
                if !is_tty_opt {
                    err = true;
                    emsg(gettext(&raw const e_string_required as *const c_char));
                }
                NIL_OPTVAL
            }
        } else {
            // Every option has at least one type.
            abort();
        };

        if !error.is_null() {
            *error = err;
        }
        value
    }
}

/// An option's value as a typval.  `numbool` renders a Boolean option as a
/// Number, which is what the old spelling of the accessors answered.
///
/// # Safety
/// `value` is a live option value; the String case hands its buffer over.
pub unsafe fn optval_as_tv(value: OptVal, numbool: bool) -> typval_T {
    unsafe {
        let mut rettv = typval_T {
            v_type: VAR_SPECIAL,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_special: kSpecialVarNull,
            },
        };
        match value.type_0 {
            kOptValTypeBoolean => {
                if numbool {
                    rettv.v_type = VAR_NUMBER;
                    rettv.vval.v_number = value.data.boolean as varnumber_T;
                } else if value.data.boolean != kNone {
                    // A `kNone` boolean has no Vimscript spelling and stays
                    // the `v:null` this started as.
                    rettv.v_type = VAR_BOOL;
                    rettv.vval.v_bool = c_int::from(value.data.boolean == kTrue) as BoolVarValue;
                }
            }
            kOptValTypeNumber => {
                rettv.v_type = VAR_NUMBER;
                rettv.vval.v_number = value.data.number as varnumber_T;
            }
            kOptValTypeString => {
                rettv.v_type = VAR_STRING;
                rettv.vval.v_string = value.data.string.data;
            }
            // `kOptValTypeNil` is the remaining arm.
            _ => {}
        }
        rettv
    }
}

/// `setbufvar()`/`setwinvar()`'s `&option` spelling: set the local value of
/// `varname` from `varp`.
///
/// # Safety
/// `varname` is a NUL-terminated name and `varp` a live value.
unsafe fn set_option_from_tv(varname: *const c_char, varp: *mut typval_T) {
    unsafe {
        let opt_idx = find_option(varname);
        if opt_idx == kOptInvalid {
            semsg_c!(
                gettext(&raw const e_unknown_option2 as *const c_char),
                varname,
            );
            return;
        }
        let mut error = false;
        let value = tv_to_optval(varp, opt_idx, varname, &raw mut error);
        if !error {
            let errmsg = set_option_value_handle_tty(varname, opt_idx, value, OPT_LOCAL);
            if !errmsg.is_null() {
                emsg(errmsg);
            }
        }
        optval_free(value);
    }
}

/// `setwinvar()`, and `settabwinvar()` with `off` 1.
///
/// # Safety
/// `argvars` holds at least `off + 3` values.
unsafe fn setwinvar(argvars: *mut typval_T, off: c_int) {
    unsafe {
        if check_secure() {
            return;
        }
        let tp = if off == 1 {
            find_tabpage(tv_get_number_chk(argvars, ptr::null_mut()) as c_int)
        } else {
            curtab.get()
        };
        let win = find_win_by_nr(argvars.offset(off as isize), tp);
        let varname = tv_get_string_chk(argvars.offset((off + 1) as isize));
        let varp = argvars.offset((off + 2) as isize);
        if win.is_null() || varname.is_null() {
            return;
        }

        let need_switch_win = !(tp == curtab.get() && win == curwin.get());
        let mut switchwin = SWITCHWIN_INITIAL_VALUE;
        if !need_switch_win || switch_win(&raw mut switchwin, win, tp, true) == OK {
            if *varname == b'&' as c_char {
                set_option_from_tv(varname.add(1), varp);
            } else {
                set_scoped_var(c"w:", varname, varp);
            }
        }
        if need_switch_win {
            restore_win(&raw mut switchwin, true);
        }
    }
}

/// Set `<scope><varname>` from `varp`, in whatever buffer, window or tab
/// page is current.
///
/// `set_var` takes a name with its scope prefix, so the two are joined into
/// a scratch buffer first; `scope` is `"b:"`, `"w:"` or `"t:"`.
///
/// # Safety
/// `varname` is a NUL-terminated name and `varp` a live value.
unsafe fn set_scoped_var(scope: &CStr, varname: *const c_char, varp: *mut typval_T) {
    unsafe {
        let varname_len = strlen(varname);
        let name = xmalloc(varname_len + 3) as *mut c_char;
        memcpy(name.cast(), scope.as_ptr().cast(), 2);
        memcpy(name.add(2).cast(), varname.cast(), varname_len + 1);
        set_var(name, varname_len + 2, varp, true);
        xfree(name.cast());
    }
}

/// `gettabvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_gettabvar(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        let varname = tv_get_string_chk(argvars.add(1));
        let tp = find_tabpage(tv_get_number_chk(argvars, ptr::null_mut()) as c_int);
        // Any window of that tab page will do: only its `t:` scope is read.
        let win = if tp.is_null() {
            ptr::null_mut()
        } else if tp == curtab.get() || (*tp).tp_firstwin.is_null() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        get_var_from(
            varname,
            rettv,
            argvars.add(2),
            b't' as c_int,
            tp,
            win,
            ptr::null_mut(),
        );
    }
}

/// `gettabwinvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_gettabwinvar(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { getwinvar(argvars, rettv, 1) }
}

/// `getwinvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_getwinvar(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { getwinvar(argvars, rettv, 0) }
}

/// `getbufvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_getbufvar(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        let varname = tv_get_string_chk(argvars.add(1));
        let buf = tv_get_buf_from_arg(argvars);
        get_var_from(
            varname,
            rettv,
            argvars.add(2),
            b'b' as c_int,
            curtab.get(),
            curwin.get(),
            buf,
        );
    }
}

/// `settabvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_settabvar(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        if check_secure() {
            return;
        }
        let tp = find_tabpage(tv_get_number_chk(argvars, ptr::null_mut()) as c_int);
        let varname = tv_get_string_chk(argvars.add(1));
        let varp = argvars.add(2);
        if varname.is_null() || tp.is_null() {
            return;
        }

        let save_curtab = curtab.get();
        let save_lu_tp = lastused_tabpage.get();
        goto_tabpage_tp(tp, false, false);

        set_scoped_var(c"t:", varname, varp);

        if valid_tabpage(save_curtab) {
            goto_tabpage_tp(save_curtab, false, false);
            // Going back must not count as a use of the previous tab page.
            if valid_tabpage(save_lu_tp) {
                lastused_tabpage.set(save_lu_tp);
            }
        }
    }
}

/// `settabwinvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_settabwinvar(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { setwinvar(argvars, 1) }
}

/// `setwinvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_setwinvar(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { setwinvar(argvars, 0) }
}

/// `setbufvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_setbufvar(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        if check_secure() || !tv_check_str_or_nr(argvars) {
            return;
        }
        let varname = tv_get_string_chk(argvars.add(1));
        let buf = tv_get_buf(argvars, false_0);
        let varp = argvars.add(2);
        if buf.is_null() || varname.is_null() {
            return;
        }

        if *varname == b'&' as c_char {
            // An option: the buffer has to be current for the autocommands
            // the change fires, which `aucmd_prepbuf` arranges.
            let mut aco = aco_save_T::default();
            aucmd_prepbuf(&raw mut aco, buf);
            set_option_from_tv(varname.add(1), varp);
            aucmd_restbuf(&raw mut aco);
        } else {
            let save_curbuf = curbuf.get();
            curbuf.set(buf);
            set_scoped_var(c"b:", varname, varp);
            curbuf.set(save_curbuf);
        }
    }
}
