//! `b:`, `w:` and `t:` from somewhere else.
//!
//! [`get_var_from`] and [`setwinvar`] switch to the requested buffer, window
//! or tabpage, do the lookup there and switch back; the `f_*` entries below
//! them are the Vimscript builtins that call them.  The `&option` spelling
//! of a name lands in [`tv_to_optval`]/[`optval_as_tv`] instead.

#![forbid(unsafe_code)]

use crate::autocmd::AucmdBuf;
use crate::cstr;
use crate::eval::window::WinSwitch;
use crate::guard::Suppress;
use crate::message_fmt::msg_bytes;
use crate::option::set_option_value_handle_tty_named;
use crate::semsg;
use crate::types::String_0;
use core::ffi::c_int;

use super::*;
use crate::eval::typval::NumBuf;
use crate::option::boolean_optval;
use crate::types::OptionSetFlags;
use crate::window::valid_tab;
use crate::winlayer::graph::switch_buffer;
use crate::winlayer::{Buf, TabPage, Win, WinId, first_window};

/// Which scope [`get_var_from`] reads.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    Buffer,
    Window,
    Tab,
}

/// `getbufvar()`, `getwinvar()`, `gettabvar()` and `gettabwinvar()`: read
/// `varname` in `scope`, falling back to `deftv`.
///
/// An empty `varname` is the whole scope as a dictionary, and one starting
/// with `&` is an option rather than a variable.  Errors are suppressed
/// throughout: these functions answer the default instead.
fn get_var_from(
    varname: Option<&[u8]>,
    result: &mut TypVal,
    deftv: Option<&TypVal>,
    scope: Scope,
    tabpage: Option<TabPage>,
    win: Option<Win>,
    buffer: Option<Buf>,
) {
    let mut done = false;
    let do_change_curbuf = buffer.is_some() && scope == Scope::Buffer;

    let _no_emsg = Suppress::emsg();
    result.write_string(None);

    if let (Some(varname), Some(tp), Some(w)) = (varname, tabpage, win)
        && (scope != Scope::Buffer || buffer.is_some())
    {
        // Make `win` current, and its tab page with it, or the window is
        // not valid. Only when needed, since it blocks autocommands --
        // and not at all with a buffer in hand, where `curbuf` is saved
        // and restored directly instead.
        let need_switch_win = !(tabpage == TabPage::current_or_none()
            && win == Win::current_or_none())
            && !do_change_curbuf;
        let switched = need_switch_win.then(|| WinSwitch::enter(w, Some(tp), true));
        if switched.as_ref().is_none_or(|&(_, entered)| entered) {
            if varname.first() == Some(&b'&') && scope != Scope::Tab {
                // An option: read it from the right buffer.
                let scoped = do_change_curbuf.then(|| {
                    // `do_change_curbuf` is exactly "the caller handed a
                    // buffer", and the caller's are live.
                    switch_buffer(buffer.expect("`do_change_curbuf` means there is one"))
                });
                if varname.len() == 1 {
                    // A bare "&": every window- or buffer-local option.
                    let opts = get_winbuf_options(scope == Scope::Buffer);
                    result.write_dict(Some(opts));
                    done = true;
                } else if eval_option(&mut Cursor::new(varname), Some(result), true).is_ok() {
                    done = true;
                }
                if let Some(scoped) = scoped {
                    scoped.restore();
                }
            } else if varname.is_empty() {
                // An empty name: the whole scope as a dictionary.
                match scope {
                    Scope::Buffer => {
                        let buffer = buffer.expect("a `b:` scope");
                        tv_copy(&buffer.b_bufvar.di_tv, result);
                    }
                    Scope::Window => tv_copy(&w.w_winvar.di_tv, result),
                    Scope::Tab => tv_copy(&tp.tp_winvar.di_tv, result),
                }
                done = true;
            } else {
                let dict = match scope {
                    Scope::Buffer => buffer.expect("a `b:` scope").b_bufvar.di_tv.dict_handle(),
                    Scope::Window => w.w_winvar.di_tv.dict_handle(),
                    Scope::Tab => tp.tp_winvar.di_tv.dict_handle(),
                };
                if let Some(item) = dict.as_ref().and_then(|dict| dict.find(varname)) {
                    tv_copy(&item.di_tv, result);
                    done = true;
                }
            }
        }
        drop(switched);
    }

    if let Some(deftv) = deftv.filter(|_| !done) {
        tv_copy(deftv, result);
    }
}

/// `getwinvar()`, and `gettabwinvar()` with `off` 1 -- which is where the
/// extra leading tab-page argument goes.
fn getwinvar(args: &[TypVal], result: &mut TypVal, off: usize) {
    let mut numbuf = NumBuf::new();
    let tp = if off == 1 {
        find_tabpage(tv_get_number_chk(&args[0]).unwrap_or(-1) as c_int)
    } else {
        TabPage::current_or_none()
    };
    let win = find_win_by_nr(&args[off], tp);
    let varname = numbuf.string_chk(&args[off + 1]).map(CStr::to_bytes);
    let deftv = args.get(off + 2);
    get_var_from(varname, result, deftv, Scope::Window, tp, win, None);
}

/// `tv` as the value of option `opt_idx` (spelt `name`), and whether the
/// conversion failed -- in which case the value is [`OptVal::Nil`].
///
/// The option's declared types decide the conversion; a Funcref is accepted
/// (as its name) only by an option that takes one.
pub(crate) fn tv_to_optval(tv: &TypVal, opt_idx: OptIndex, name: &[u8]) -> (OptVal, bool) {
    let mut nbuf = NumBuf::new();
    let mut err = false;
    let is_tty_opt = is_tty_option(name);
    let option_has_bool = !is_tty_opt && option_has_type(opt_idx, kOptValTypeBoolean);
    let option_has_num = !is_tty_opt && option_has_type(opt_idx, kOptValTypeNumber);
    let option_has_str = is_tty_opt || option_has_type(opt_idx, kOptValTypeString);

    let value =
        if !is_tty_opt && get_option(opt_idx).flags & kOptFlagFunc as uint32_t != 0 && tv.is_func()
        {
            // An option that takes a function reference or a lambda stores
            // the name of one.
            OptVal::string(String_0::from_bytes(encode_tv2string(tv).as_bytes()))
        } else if option_has_bool || option_has_num {
            let read = if option_has_num {
                tv_get_number_chk(tv)
            } else {
                tv_get_bool_chk(tv)
            };
            err = read.is_err();
            let n = read.unwrap_or(0);
            // A String answers 0 both when it *is* zero and when it is not a
            // number at all, so a zero from a String has to be re-read: it
            // is only honest if the string is all '0's and nothing else.
            if !err && tv.v_type() == VAR_STRING && n == 0 {
                let s = tv.string_bytes();
                let zeros = s.iter().take_while(|&&byte| byte == b'0').count();
                if zeros == 0 || zeros != s.len() {
                    err = true;
                    let option = msg_bytes(name);
                    let arg1 = msg_bytes(s);
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
            if tv.v_type() != VAR_BOOL && tv.v_type() != VAR_SPECIAL {
                let strval = nbuf.string_chk(tv);
                err = strval.is_none();
                OptVal::string(
                    strval.map_or(String_0::NULL, |text| String_0::from_bytes(text.to_bytes())),
                )
            } else {
                if !is_tty_opt {
                    err = true;
                    emsg_static(e_string_required);
                }
                OptVal::Nil
            }
        } else {
            unreachable!("every option has at least one type");
        };
    (value, err)
}

/// `setbufvar()`/`setwinvar()`'s `&option` spelling: set the local value of
/// the option `name` from `value`.
fn set_option_from_tv(name: &[u8], value: &TypVal) {
    let opt_idx = cstr::with_terminated(name, find_option);
    if opt_idx == kOptInvalid {
        let name = msg_bytes(name);
        semsg!("E355: Unknown option: {name}");
        return;
    }
    let (optval, error) = tv_to_optval(value, opt_idx, name);
    if !error {
        let local = OptionSetFlags::LOCAL;
        let set = set_option_value_handle_tty_named(name, opt_idx, optval, local);
        if let Err(errmsg) = set {
            emsg(errmsg.as_cstr());
        }
    }
    optval_free(optval);
}

/// `setwinvar()`, and `settabwinvar()` with `off` 1.
fn setwinvar(args: &[TypVal], off: usize) {
    let mut numbuf = NumBuf::new();
    if check_secure() {
        return;
    }
    let tp = if off == 1 {
        find_tabpage(tv_get_number_chk(&args[0]).unwrap_or(-1) as c_int)
    } else {
        TabPage::current_or_none()
    };
    let win = find_win_by_nr(&args[off], tp);
    let varname = numbuf.string_chk(&args[off + 1]).map(CStr::to_bytes);
    let varp = &args[off + 2];
    let (Some(w), Some(varname)) = (win, varname) else {
        return;
    };

    let need_switch_win = !(tp == TabPage::current_or_none() && w.is_current());
    // `tp` is `None` for "wherever it is", which is what `switch_win` reads
    // an absent tab page as.
    let switched = need_switch_win.then(|| WinSwitch::enter(w, tp, true));
    if switched.as_ref().is_none_or(|&(_, entered)| entered) {
        match varname.split_first() {
            Some((b'&', option)) => set_option_from_tv(option, varp),
            _ => set_scoped_var(b"w:", varname, varp),
        }
    }
    drop(switched);
}

/// Set `<scope><varname>` from `varp`, in whatever buffer, window or tab
/// page is current.
///
/// `set_var` takes a name with its scope prefix, so the two are joined
/// first; `scope` is `"b:"`, `"w:"` or `"t:"`.
fn set_scoped_var(scope: &[u8], varname: &[u8], varp: &TypVal) {
    // The store either copies or takes; this one only ever borrows, so it
    // hands over a copy of its own and lets the store take that.
    let mut value = varp.clone();
    let mut name = scope.to_vec();
    name.extend_from_slice(varname);
    set_var(&name, &mut value, false);
}

/// `gettabvar()`.
pub fn f_gettabvar(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let varname = numbuf.string_chk(&args[1]).map(CStr::to_bytes);
    let tp = find_tabpage(tv_get_number_chk(&args[0]).unwrap_or(-1) as c_int);
    // Any window of that tab page will do: only its `t:` scope is read.
    let win = any_window_of(tp);
    get_var_from(varname, result, args.get(2), Scope::Tab, tp, win, None);
}

/// `gettabwinvar()`.
pub fn f_gettabwinvar(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    getwinvar(args, result, 1)
}

/// `getwinvar()`.
pub fn f_getwinvar(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    getwinvar(args, result, 0)
}

/// `getbufvar()`.
pub fn f_getbufvar(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let varname = numbuf.string_chk(&args[1]).map(CStr::to_bytes);
    let buf = tv_get_buf_from_arg(&args[0]);
    let deftv = args.get(2);
    let (tp, win) = (TabPage::current_or_none(), Win::current_or_none());
    get_var_from(varname, result, deftv, Scope::Buffer, tp, win, buf);
}

/// `settabvar()`.
pub fn f_settabvar(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if check_secure() {
        return;
    }
    let tp = find_tabpage(tv_get_number_chk(&args[0]).unwrap_or(-1) as c_int);
    let varname = numbuf.string_chk(&args[1]).map(CStr::to_bytes);
    let varp = &args[2];
    let (Some(varname), Some(tp)) = (varname, tp) else {
        return;
    };

    // The identity, taken while it is live: `set_scoped_var` below can run
    // autocommands that close this tab page.
    let save_curtab = TabPage::current().id();
    let save_lu_tp = lastused_tabpage.get();
    goto_tabpage_tp(tp, false, false);

    set_scoped_var(b"t:", varname, varp);

    if let Some(save_curtab) = valid_tab(save_curtab) {
        goto_tabpage_tp(save_curtab, false, false);
        // Going back must not count as a use of the previous tab page.
        if save_lu_tp.is_some_and(valid_tabpage) {
            lastused_tabpage.set(save_lu_tp);
        }
    }
}

/// `settabwinvar()`.
pub fn f_settabwinvar(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    setwinvar(args, 1)
}

/// `setwinvar()`.
pub fn f_setwinvar(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    setwinvar(args, 0)
}

/// `setbufvar()`.
pub fn f_setbufvar(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if check_secure() || !tv_check_str_or_nr(&args[0]) {
        return;
    }
    let varname = numbuf.string_chk(&args[1]).map(CStr::to_bytes);
    let buf = tv_get_buf(&args[0], 0);
    let varp = &args[2];
    let (Some(buf), Some(varname)) = (buf, varname) else {
        return;
    };

    match varname.split_first() {
        Some((b'&', option)) => {
            // An option: the buffer has to be current for the autocommands
            // the change fires, which `aucmd_prepbuf` arranges.
            let aco = AucmdBuf::enter(buf);
            set_option_from_tv(option, varp);
            drop(aco);
        }
        _ => {
            let saved = switch_buffer(buf);
            set_scoped_var(b"b:", varname, varp);
            saved.restore();
        }
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
