//! `b:`, `w:` and `t:` from somewhere else.
//!
//! [`get_var_from`] and [`setwinvar`] switch to the requested buffer, window
//! or tabpage, do the lookup there and switch back; the `f_*` entries below
//! them are the Vimscript builtins that call them.  The `&option` spelling
//! of a name lands in [`tv_to_optval`]/[`optval_as_tv`] instead.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::guard::Suppress;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::eval::typval::NumBuf;
use crate::option::boolean_optval;
use crate::types::{NUL, OptionSetFlags};
use crate::winlayer::graph::switch_buffer;
use crate::winlayer::{Buf, TabPage, Win, WinId, first_window};

/// The zeroed `SwitchWin` [`switch_win`] fills in.
const SWITCHWIN_INITIAL_VALUE: SwitchWin = SwitchWin {
    sw_curwin: None,
    sw_curtab: None,
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
/// `result` is writable and holds nothing; `deftv` is a live value;
/// `tabpage`/`win`/`buffer` are live or NULL.
unsafe fn get_var_from(
    mut varname: *const c_char,
    result: *mut TypVal,
    deftv: *mut TypVal,
    htname: c_int,
    tabpage: Option<TabPage>,
    win: Option<Win>,
    buffer: Option<Buf>,
) {
    let mut done = false;
    let do_change_curbuf = buffer.is_some() && htname == b'b' as c_int;

    let _no_emsg = Suppress::emsg();
    // SAFETY: the caller's obligation -- a writable value holding nothing.
    let mut ret = unsafe { Tv::new(result) };
    ret.v_type = VAR_STRING;
    ret.vval.v_string = ptr::null_mut();

    if let (false, Some(mut tp), Some(mut w)) = (varname.is_null(), tabpage, win)
        && (htname != b'b' as c_int || buffer.is_some())
    {
        // Make `win` current, and its tab page with it, or the window is
        // not valid. Only when needed, since it blocks autocommands --
        // and not at all with a buffer in hand, where `curbuf` is saved
        // and restored directly instead.
        let need_switch_win = !(tabpage == TabPage::current_or_none()
            && win == Win::current_or_none())
            && !do_change_curbuf;
        let mut switchwin = SWITCHWIN_INITIAL_VALUE;
        // SAFETY: `varname` is NUL-terminated and the handles are live.
        let lead = unsafe { *varname } as u8;
        if !need_switch_win || unsafe { switch_win(&raw mut switchwin, w, Some(tp), true) }.is_ok()
        {
            if lead == b'&' && htname != b't' as c_int {
                // An option: read it from the right buffer.
                let scoped = do_change_curbuf.then(|| {
                    // SAFETY: `do_change_curbuf` is exactly "the caller
                    // handed a buffer", and the caller's are live.
                    switch_buffer(buffer.expect("`do_change_curbuf` means there is one"))
                });
                if unsafe { *varname.add(1) } == NUL as c_char {
                    // A bare "&": every window- or buffer-local option.
                    let opts = get_winbuf_options(c_int::from(htname == b'b' as c_int));
                    if !opts.is_null() {
                        unsafe { tv_dict_set_ret(result, opts) };
                        done = true;
                    }
                } else if unsafe { eval_option(&raw mut varname, result, true) }.is_ok() {
                    done = true;
                }
                if let Some(scoped) = scoped {
                    scoped.restore();
                }
            } else if lead == NUL as u8 {
                // An empty name: the whole scope as a dictionary.
                let scope = buffer.map(|b| b.raw());
                let v: *const ScopeDictDictItem = match htname as u8 {
                    b'b' => &raw mut unsafe { Buf::new(scope.expect("a `b:` scope")) }.b_bufvar,
                    b'w' => &raw mut w.w_winvar,
                    _ => &raw mut tp.tp_winvar,
                };
                unsafe { tv_copy(&raw const (*v).di_tv, result) };
                done = true;
            } else {
                // SAFETY: each scope's own variable dictionary is live.
                let ht = unsafe {
                    match htname as u8 {
                        b'b' => &raw mut (*buffer.expect("a `b:` scope").b_vars).dv_hashtab,
                        b'w' => &raw mut (*w.w_vars).dv_hashtab,
                        _ => &raw mut (*tp.tp_vars).dv_hashtab,
                    }
                };
                let varname_len = unsafe { cstr::bytes_at(varname) }.len();
                let v = unsafe { find_var_in_ht(ht, htname, varname, varname_len, false) };
                if !v.is_null() {
                    unsafe { tv_copy(&raw const (*v).di_tv, result) };
                    done = true;
                }
            }
        }
        if need_switch_win {
            unsafe { restore_win(&raw mut switchwin, true) };
        }
    }

    // SAFETY: the caller's obligation -- a live default value.
    if !done && unsafe { (*deftv).v_type } != VAR_UNKNOWN {
        unsafe { tv_copy(deftv, result) };
    }
}

/// `getwinvar()`, and `gettabwinvar()` with `off` 1 -- which is where the
/// extra leading tab-page argument goes.
///
/// # Safety
/// `args` holds at least `off + 3` values; `result` is writable.
unsafe fn getwinvar(args: *mut TypVal, result: *mut TypVal, off: c_int) {
    let mut numbuf = NumBuf::new();
    let tp = if off == 1 {
        find_tabpage(unsafe { tv_get_number_chk(args, ptr::null_mut()) } as c_int)
    } else {
        TabPage::current_or_none()
    };
    let win = unsafe { find_win_by_nr(args.offset(off as isize), tp) };
    let varname = unsafe { numbuf.string_chk(args.offset((off + 1) as isize)) };
    let deftv = unsafe { args.offset((off + 2) as isize) };
    let nil = None;
    // SAFETY: the caller's obligation -- `off + 3` live values -- and the
    // window and tab page the resolver answered.
    unsafe { get_var_from(varname, result, deftv, b'w' as c_int, tp, win, nil) };
}

/// `tv` as the value of option `opt_idx`, or [`OptVal::Nil`] with `error` set.
///
/// The option's declared types decide the conversion; a Funcref is accepted
/// (as its name) only by an option that takes one.
///
/// # Safety
/// `tv` is a live value, `option` a NUL-terminated name matching `opt_idx`,
/// and `error` writable or NULL.
pub(crate) unsafe fn tv_to_optval(
    tv: *mut TypVal,
    opt_idx: OptIndex,
    option: *const c_char,
    error: *mut bool,
) -> OptVal {
    let mut nbuf = [0 as c_char; 65];
    let mut err = false;
    // SAFETY: the caller's obligation -- a live value and a NUL-terminated
    // option name.
    let tvh = unsafe { Tv::new(tv) };
    let is_tty_opt = is_tty_option(unsafe { CStr::from_ptr(option) });
    let option_has_bool = !is_tty_opt && option_has_type(opt_idx, kOptValTypeBoolean);
    let option_has_num = !is_tty_opt && option_has_type(opt_idx, kOptValTypeNumber);
    let option_has_str = is_tty_opt || option_has_type(opt_idx, kOptValTypeString);

    let value = if !is_tty_opt
        && get_option(opt_idx).flags & kOptFlagFunc as uint32_t != 0
        && tv_is_func(*tvh)
    {
        // An option that takes a function reference or a lambda stores
        // the name of one.
        let strval = unsafe { encode_tv2string(tv, ptr::null_mut()) };
        err = strval.is_null();
        OptVal::String(unsafe { cstr_as_string(strval) })
    } else if option_has_bool || option_has_num {
        let n = if option_has_num {
            unsafe { tv_get_number_chk(tv, &raw mut err) }
        } else {
            unsafe { tv_get_bool_chk(tv, &raw mut err) }
        };
        // A String answers 0 both when it *is* zero and when it is not a
        // number at all, so a zero from a String has to be re-read: it
        // is only honest if the string is all '0's and nothing else.
        if !err && tvh.v_type == VAR_STRING && n == 0 {
            // SAFETY: the type tag says the union holds the string arm.
            let s = tvh.string_or_null();
            let mut idx = 0;
            while !s.is_null() && unsafe { *s.add(idx) } == b'0' as c_char {
                idx += 1;
            }
            if idx == 0 || unsafe { *s.add(idx) } != NUL as c_char {
                err = true;
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (option, arg1) = unsafe {
                    (
                        c_str(option),
                        c_str(if s.is_null() { c"".as_ptr() } else { s }),
                    )
                };
                semsg!("E521: Number required: &{option} = '{arg1}'");
            }
        }
        if option_has_num {
            OptVal::Number(n)
        } else {
            boolean_optval(tristate_from_int(n))
        }
    } else if option_has_str {
        // Never set a string option to `v:true` or `v:null`.
        if tvh.v_type != VAR_BOOL && tvh.v_type != VAR_SPECIAL {
            let strval = unsafe { tv_get_string_buf_chk(tv, nbuf.as_mut_ptr()) };
            err = strval.is_null();
            OptVal::String(unsafe { cstr_to_string(strval) })
        } else {
            if !is_tty_opt {
                err = true;
                emsg_static(e_string_required);
            }
            OptVal::Nil
        }
    } else {
        // Every option has at least one type.
        unsafe { abort() };
    };

    if !error.is_null() {
        unsafe { *error = err };
    }
    value
}

/// An option's value as a typval.  `numbool` renders a Boolean option as a
/// Number, which is what the old spelling of the accessors answered.
///
/// # Safety
/// `value` is a live option value; the String case hands its buffer over.
pub unsafe fn optval_as_tv(value: OptVal, numbool: bool) -> TypVal {
    let mut rettv = TypVal {
        v_type: VAR_SPECIAL,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union {
            v_special: kSpecialVarNull,
        },
    };
    match value {
        OptVal::Boolean(word) => {
            if numbool {
                rettv.v_type = VAR_NUMBER;
                rettv.vval.v_number = word as VarNumber;
            } else if let Some(boolean) = value.as_boolean() {
                // An unset global-local boolean has no Vimscript
                // spelling and stays the `v:null` this started as.
                rettv.v_type = VAR_BOOL;
                rettv.vval.v_bool = c_int::from(boolean) as BoolVarValue;
            }
        }
        OptVal::Number(number) => {
            rettv.v_type = VAR_NUMBER;
            rettv.vval.v_number = number as VarNumber;
        }
        OptVal::String(string) => {
            rettv.v_type = VAR_STRING;
            rettv.vval.v_string = string.data();
        }
        OptVal::Nil => {}
    }
    rettv
}

/// `setbufvar()`/`setwinvar()`'s `&option` spelling: set the local value of
/// `varname` from `varp`.
///
/// # Safety
/// `varname` is a NUL-terminated name and `varp` a live value.
unsafe fn set_option_from_tv(varname: *const c_char, varp: *mut TypVal) {
    let opt_idx = find_option(unsafe { CStr::from_ptr(varname) });
    if opt_idx == kOptInvalid {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let varname = unsafe { c_str(varname) };
        semsg!("E355: Unknown option: {varname}");
        return;
    }
    let mut error = false;
    let value = unsafe { tv_to_optval(varp, opt_idx, varname, &raw mut error) };
    if !error {
        let local = OptionSetFlags::LOCAL;
        // SAFETY: the caller's obligation -- a NUL-terminated name matching
        // the index the lookup above answered.
        let errmsg = unsafe { set_option_value_handle_tty(varname, opt_idx, value, local) };
        if let Some(errmsg) = errmsg {
            emsg(&errmsg);
        }
    }
    optval_free(value);
}

/// `setwinvar()`, and `settabwinvar()` with `off` 1.
///
/// # Safety
/// `args` holds at least `off + 3` values.
unsafe fn setwinvar(args: *mut TypVal, off: c_int) {
    let mut numbuf = NumBuf::new();
    if check_secure() {
        return;
    }
    let tp = if off == 1 {
        find_tabpage(unsafe { tv_get_number_chk(args, ptr::null_mut()) } as c_int)
    } else {
        TabPage::current_or_none()
    };
    let win =
        unsafe { find_win_by_nr(args.offset(off as isize), tp) }.map_or(ptr::null_mut(), Win::raw);
    let varname = unsafe { numbuf.string_chk(args.offset((off + 1) as isize)) };
    let varp = unsafe { args.offset((off + 2) as isize) };
    if win.is_null() || varname.is_null() {
        return;
    }

    let need_switch_win = !(tp == TabPage::current_or_none() && win == Win::current_raw());
    let mut switchwin = SWITCHWIN_INITIAL_VALUE;
    // SAFETY: a live window and its tab page.
    let (w, t) = unsafe {
        (
            Win::new(win),
            TabPage::new(tp.map_or(ptr::null_mut(), TabPage::raw)),
        )
    };
    if !need_switch_win || unsafe { switch_win(&raw mut switchwin, w, Some(t), true) }.is_ok() {
        if unsafe { *varname } == b'&' as c_char {
            unsafe { set_option_from_tv(varname.add(1), varp) };
        } else {
            unsafe { set_scoped_var(c"w:", varname, varp) };
        }
    }
    if need_switch_win {
        unsafe { restore_win(&raw mut switchwin, true) };
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
unsafe fn set_scoped_var(scope: &CStr, varname: *const c_char, varp: *mut TypVal) {
    let varname_len = unsafe { cstr::bytes_at(varname) }.len();
    let name = unsafe { xmalloc(varname_len + 3) } as *mut c_char;
    let into = name.cast::<u8>();
    unsafe { into.copy_from_nonoverlapping(scope.as_ptr().cast(), 2) };
    let into = unsafe { name.add(2) }.cast::<u8>();
    unsafe { into.copy_from_nonoverlapping(varname.cast(), varname_len + 1) };
    unsafe { set_var(name, varname_len + 2, varp, true) };
    unsafe { xfree(name.cast()) };
}

/// `gettabvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_gettabvar(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let varname = unsafe { numbuf.string_chk(args.add(1)) };
    let tp = find_tabpage(unsafe { tv_get_number_chk(args, ptr::null_mut()) } as c_int);
    // Any window of that tab page will do: only its `t:` scope is read.
    let win = any_window_of(tp);
    let (deftv, nil) = (unsafe { args.add(2) }, None);

    let __hoisted_1 = win;

    unsafe { get_var_from(varname, result, deftv, b't' as c_int, tp, __hoisted_1, nil) };
}

/// `gettabwinvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_gettabwinvar(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    unsafe { getwinvar(args, result, 1) }
}

/// `getwinvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_getwinvar(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    unsafe { getwinvar(args, result, 0) }
}

/// `getbufvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_getbufvar(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let varname = unsafe { numbuf.string_chk(args.add(1)) };
    let buf = unsafe { tv_get_buf_from_arg(args) };
    let deftv = unsafe { args.add(2) };
    let (tp, win) = (TabPage::current_or_none(), Win::current_or_none());

    let __hoisted_2 = buf;

    unsafe { get_var_from(varname, result, deftv, b'b' as c_int, tp, win, __hoisted_2) };
}

/// `settabvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_settabvar(args: *mut TypVal, _result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if check_secure() {
        return;
    }
    let tp = find_tabpage(unsafe { tv_get_number_chk(args, ptr::null_mut()) } as c_int);
    let varname = unsafe { numbuf.string_chk(args.add(1)) };
    let varp = unsafe { args.add(2) };
    if varname.is_null() || tp.is_none() {
        return;
    }

    let save_curtab = TabPage::current();
    let save_lu_tp = lastused_tabpage.get();
    unsafe { goto_tabpage_tp(tp.expect("a live handle"), false, false) };

    unsafe { set_scoped_var(c"t:", varname, varp) };

    if valid_tabpage(save_curtab.id()) {
        unsafe { goto_tabpage_tp(save_curtab, false, false) };
        // Going back must not count as a use of the previous tab page.
        if save_lu_tp.is_some_and(valid_tabpage) {
            lastused_tabpage.set(save_lu_tp);
        }
    }
}

/// `settabwinvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_settabwinvar(args: *mut TypVal, _result: *mut TypVal, _fptr: EvalFuncData) {
    unsafe { setwinvar(args, 1) }
}

/// `setwinvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_setwinvar(args: *mut TypVal, _result: *mut TypVal, _fptr: EvalFuncData) {
    unsafe { setwinvar(args, 0) }
}

/// `setbufvar()`.
///
/// # Safety
/// As a `VimLFunc`.
pub unsafe fn f_setbufvar(args: *mut TypVal, _result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if check_secure() || !unsafe { tv_check_str_or_nr(args) } {
        return;
    }
    let varname = unsafe { numbuf.string_chk(args.add(1)) };
    let buf = unsafe { tv_get_buf(args, 0) };
    let varp = unsafe { args.add(2) };
    if buf.is_none() || varname.is_null() {
        return;
    }

    if unsafe { *varname } == b'&' as c_char {
        // An option: the buffer has to be current for the autocommands
        // the change fires, which `aucmd_prepbuf` arranges.
        let mut aco = AcoSave::default();
        unsafe { aucmd_prepbuf(&raw mut aco, buf.expect("a live handle")) };
        unsafe { set_option_from_tv(varname.add(1), varp) };
        unsafe { aucmd_restbuf(&raw mut aco) };
    } else {
        // SAFETY: `tv_get_buf` answers a live buffer or null, and the null
        // was ruled out above.
        let saved = switch_buffer(buf.expect("a live handle"));
        unsafe { set_scoped_var(c"b:", varname, varp) };
        saved.restore();
    }
}

/// Any window of `tab`, for the sake of its `t:` scope; a null when there is
/// no such tab page. The current tab page's list hangs off `firstwin`, which
/// upstream spells out here rather than reaching for the macro.
fn any_window_of(tab: Option<TabPage>) -> Option<Win> {
    let tab = tab?;
    match tab.is_current() || tab.tp_firstwin.is_none() {
        true => first_window(),
        false => tab.tp_firstwin.and_then(WinId::get),
    }
}
