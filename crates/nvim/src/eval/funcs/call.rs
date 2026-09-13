//! Calling things: `call()`, `function()`, `eval()`, `execute()` and the
//! bridges to the script hosts.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::wrappers::{arg_number, arg_string, arg_string_chk};
use super::{AUTOLOAD_CHAR, MAX_FUNC_ARGS, TFN_INT, TFN_NO_AUTOLOAD, TFN_NO_DEREF, TFN_QUIET};
use crate::api::private::helpers::cstr_to_string;
use crate::ascii::ascii_isdigit;
use crate::autocmd::{au_exists, autocmd_supported};
use crate::charset::skipwhite;
use crate::cstr;
use crate::eval::EVALARG_EVALUATE;
use crate::eval::gc::{garbage_collect_at_exit, want_garbage_collect};
use crate::eval::typval::{
    ListRef, NumBuf, tv_check_for_dict_arg, tv_check_for_list_arg, tv_copy, tv_list_items,
    tv_list_iter, tv_list_len,
};
use crate::eval::userfunc::{
    emsg_funcname, find_func, func_call, func_ptr_ref, func_ref, func_unref, function_exists,
    get_scriptlocal_funcname, save_function_name, trans_function_name, translated_function_exists,
};
use crate::eval::vars::var_exists;
use crate::eval::{eval_option, eval1, partial_name, script_host_eval};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::{DoCmdOpts, cmd_exists, do_cmdline, do_cmdline_cmd};
use crate::ex_eval::aborting;
use crate::garray::{ga_append, ga_init};
use crate::guard::Suppress;
use crate::lua::executor::{
    nlua_func_exists, nlua_is_table_from_lua, nlua_register_table_as_callable, nlua_typval_eval,
};
use crate::memory::{strnequal, xcalloc, xfree, xmalloc, xstrdup};
use crate::message::emsg;
use crate::message::state::{
    capture_ga, emsg_noredir, emsg_silent, msg_col, need_clr_eos, redir_off,
};
use crate::message::{e_toomanyarg, e_unknown_function_str};
use crate::message_fmt::c_str;
use crate::os::cshim::gettext;
use crate::os::dl::{LibcallArg, LibcallResult, LibcallReturn, os_libcall};
use crate::os::env::{expand_env_save, os_env_exists};
use crate::semsg;
use crate::strings::has_char;
use crate::types::{
    EvalFuncData, FuncDict, GArray, List, NUL, Partial, Refcount, TypVal, VAR_DICT, VAR_FUNC,
    VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_STRING, VarNumber, VarType, uint8_t,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

/// A C string this module allocated and must release.
///
/// The C bodies below end in a `theend:` label whose only job is one
/// `xfree`; this is that label.
struct Owned(*mut c_char);

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: the pointer came from the allocator `xfree` releases, or
        // is null, which `xfree` accepts.
        unsafe { xfree(self.0 as *mut c_void) };
    }
}

/// `call({func}, {arglist} [, {dict}])`
pub fn f_call(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the frame is live; every pointer below either belongs to an
    // argument or is one this body allocated and releases.
    if tv_check_for_list_arg(args, 1).is_err() {
        return;
    }
    // A null List is v:_null_list, which calls nothing.
    if args[1].list_or_null().is_null() {
        return;
    }

    let mut partial = ptr::null_mut::<Partial>();
    // Only the Lua-table arm allocates; the others borrow.
    let mut owned = false;
    let mut func = match args[0].v_type() {
        VAR_FUNC => args[0].func_name_or_null(),
        VAR_PARTIAL => {
            partial = args[0].partial_or_null();
            unsafe { partial_name(partial) }
        }
        _ if nlua_is_table_from_lua(&args[0]) => {
            owned = true;
            unsafe { nlua_register_table_as_callable(&args[0]) }
        }
        _ => arg_string(&mut numbuf, &args[0]) as *mut c_char,
    };
    if func.is_null() || unsafe { *func } as c_int == NUL {
        // Upstream returns here without releasing an owned name.
        return;
    }

    // A String name is resolved through the function-name translator,
    // which is what turns `s:`/`<SID>` into the real name.
    let tofree;
    if args[0].v_type() == VAR_STRING {
        let mut p = func;
        let name = &raw mut p;
        let flags = TFN_INT as c_int | TFN_QUIET as c_int;
        let (fd, pt) = (ptr::null_mut::<FuncDict>(), ptr::null_mut());
        // SAFETY: `p` walks a NUL-terminated string the frame owns.
        tofree = Owned(unsafe { trans_function_name(name, false, flags, fd, pt) });
        if tofree.0.is_null() {
            unsafe { emsg_funcname(e_unknown_function_str.as_ptr(), func) };
            return;
        }
        func = tofree.0;
    }

    // A bad {dict} skips the call but still runs the cleanup below.
    let selfdict = if args.len() <= 2 {
        Some(ptr::null_mut())
    } else if tv_check_for_dict_arg(args, 2).is_err() {
        None
    } else {
        Some(args[2].dict_or_null())
    };
    if let Some(selfdict) = selfdict {
        let _ = unsafe { func_call(func, &args[1], partial, selfdict, result) };
    }

    if owned {
        unsafe { func_unref(func) };
    }
}

/// `eval({string})`
pub fn f_eval(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut evalarg = EVALARG_EVALUATE;
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the frame is live and `s` walks a string an argument owns.
    let mut s = arg_string_chk(&mut numbuf, &args[0]);
    if !s.is_null() {
        s = unsafe { skipwhite(s) };
    }
    // Kept for the message: `eval1` advances `s` past what it consumed.
    let expr_start = s;
    if s.is_null()
        || unsafe { eval1(&raw mut s as *mut *mut c_char, result, &raw mut evalarg) }.is_err()
    {
        if !expr_start.is_null() && !aborting() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let expr_start = unsafe { c_str(expr_start) };
            semsg!("E15: Invalid expression: \"{expr_start}\"");
        }
        need_clr_eos.set(false);
        result.write_number(0);
    } else if unsafe { *s } as c_int != NUL {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let s = unsafe { c_str(s) };
        semsg!("E488: Trailing characters: {s}");
    }
}

/// Where the `:execute` List form is up to, as `do_cmdline`'s cookie.
struct ListLines {
    /// The list being walked, whose reference this holds.
    list: *mut List,
    /// Where the walk is: an index, because a command in the list can edit
    /// the very list it is being read from.
    at: usize,
}

/// `do_cmdline`'s line getter for `execute([...])`: one allocated line per
/// List item, and null when the List runs out.
///
/// # Safety
/// `cookie` points at a live [`ListLines`].
unsafe fn get_list_line(
    _c: c_int,
    cookie: *mut c_void,
    _indent: c_int,
    _do_concat: bool,
) -> *mut c_char {
    let state = cookie.cast::<ListLines>();
    // SAFETY: the caller's obligation. `buf` outlives the string
    // `tv_get_string_buf_chk` may park in it, because the duplicate is made
    // before returning.
    let at = unsafe { (*state).at };
    let Some(item) = (unsafe { tv_list_items((*state).list) }).get(at) else {
        return ptr::null_mut();
    };
    let mut buf = NumBuf::new();
    let s = buf.string_ptr_chk(&item.li_tv);
    unsafe { (*state).at = at + 1 };
    if s.is_null() {
        ptr::null_mut()
    } else {
        unsafe { xstrdup(s) }
    }
}

/// `execute()` and `win_execute()`: run commands with output captured.
///
/// `arg_off` is where this caller's arguments start, since `win_execute()`
/// puts a window in front of them.
pub fn execute_common(args: &[TypVal], result: &mut TypVal, arg_off: c_int) {
    let mut numbuf = NumBuf::new();
    let cmd_idx = arg_off as usize;
    let silent_idx = cmd_idx + 1;

    let save_emsg_silent = emsg_silent.get();
    let save_emsg_noredir = emsg_noredir.get();
    let save_redir_off = redir_off.get();
    let save_capture_ga = capture_ga.get();
    let save_msg_col = msg_col.get();
    let mut echo_output = false;
    let mut silence = true;

    // SAFETY throughout: the frame is live; `capture_local` outlives every command run
    // below, and `result` adopts its allocation at the end.
    if check_secure() {
        return;
    }

    if args.len() > silent_idx {
        let mut buf = NumBuf::new();
        let s = arg_string_chk(&mut buf, &args[silent_idx]);
        if s.is_null() {
            return;
        }
        // An explicit empty {silent} means "not silent", and is the
        // only spelling that leaves the cursor column alone.
        if unsafe { *s } as c_int == NUL {
            echo_output = true;
        }
        // Any prefix of "silent" silences; only the exact "silent!"
        // also silences errors.
        silence = unsafe { cstr::starts_with(s, b"silent") };
        if unsafe { cstr::eq_bytes(s, b"silent!") } {
            emsg_silent.set(1);
            emsg_noredir.set(true);
        }
    }
    // Restored either way: an explicit empty {silent} asks for output
    // and still resets what the commands below leave behind.
    let _silenced = Suppress::messages_saved_when(silence);

    let mut capture_local = GArray {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    unsafe { ga_init(&raw mut capture_local, size_of::<c_char>() as c_int, 80) };
    capture_ga.set(&raw mut capture_local);
    redir_off.set(false);
    if !echo_output {
        msg_col.set(0);
    }

    if !args
        .get(cmd_idx)
        .is_some_and(|arg| arg.v_type() == VAR_LIST)
    {
        let _ = unsafe { do_cmdline_cmd(arg_string(&mut numbuf, &args[cmd_idx])) };
    } else if !args[cmd_idx].list_or_null().is_null() {
        let list = args[cmd_idx].list_or_null();
        // The List is held across the run: a command may drop the
        // variable holding it.
        // SAFETY: the argument's live list.
        let held = unsafe { ListRef::retained(list) };
        let mut cookie = ListLines { list, at: 0 };
        type GetLine = unsafe fn(c_int, *mut c_void, c_int, bool) -> *mut c_char;
        let getline = Some(get_list_line as GetLine);
        let cookie = (&raw mut cookie).cast::<c_void>();
        let opts = DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE | DoCmdOpts::REPEAT | DoCmdOpts::KEYTYPED;
        // SAFETY: `cookie` is the walk state this frame owns and outlives
        // the call, which is what `get_list_line` asks for.
        let _ = unsafe { do_cmdline(ptr::null_mut(), getline, cookie, opts) };
        drop(held);
    }

    emsg_silent.set(save_emsg_silent);
    emsg_noredir.set(save_emsg_noredir);
    redir_off.set(save_redir_off);
    msg_col.set(if echo_output { 0 } else { save_msg_col });

    // Read `capture_ga` back rather than using `capture_local`: a
    // nested `execute()` restores the pointer, and it is the current
    // one that holds this run's output.
    unsafe { ga_append(capture_ga.get(), NUL as uint8_t) };
    unsafe { (*result).write_string((*capture_ga.get()).ga_data as *mut c_char) };
    capture_ga.set(save_capture_ga);
}

/// `execute({command} [, {silent}])`
pub fn f_execute(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    execute_common(args, result, 0);
}

/// `exists({expr})` — the sigil in front of the name picks the namespace.
pub fn f_exists(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the frame is live and `p` walks a string an argument owns.
    let mut p = arg_string(&mut numbuf, &args[0]);
    // Not a bool: the `:` arm answers 2 for an exact command name, and
    // that grading is part of `exists()`'s contract.
    let found: c_int = match unsafe { *p } as u8 {
        b'$' => {
            // The environment, or a name that expands to something
            // other than itself.
            (if unsafe { os_env_exists(p.add(1), false) } {
                true
            } else {
                let expanded = Owned(unsafe { expand_env_save(p as *mut c_char) });
                !expanded.0.is_null() && unsafe { *expanded.0 } as u8 != b'$'
            }) as c_int
        }
        b'&' | b'+' => {
            // An option, and nothing may follow it.
            (unsafe { eval_option(&raw mut p, None, true) }.is_ok()
                && unsafe { *skipwhite(p) } as c_int == NUL) as c_int
        }
        b'*' => {
            if unsafe { strnequal(p, c"*v:lua.".as_ptr(), 7) } {
                unsafe { nlua_func_exists(p.add(7)) as c_int }
            } else {
                unsafe { function_exists(p.add(1), false) as c_int }
            }
        }
        b':' => unsafe { cmd_exists(p.add(1)) },
        // `##event` asks whether the event name is known at all;
        // `#event` asks whether an autocommand is defined for it.
        b'#' if unsafe { *p.add(1) } as u8 == b'#' => {
            (unsafe { autocmd_supported(p.add(2)) }) as c_int
        }
        b'#' => unsafe { au_exists(p.add(1)) as c_int },
        _ => unsafe { var_exists(p) as c_int },
    };
    result.write_number(found as VarNumber);
}

/// `function()` and `funcref()`.
///
/// `funcref()` binds the function the name resolves to *now*; `function()`
/// keeps the name and resolves it at call time.
fn common_function(args: &[TypVal], result: &mut TypVal, is_funcref: bool) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    // SAFETY throughout: the frame is live; the partial built below owns every value
    // it copies, and `trans_name`/`name` are released on every path.
    let mut arg_pt = ptr::null_mut::<Partial>();
    let mut use_string = false;
    let mut s = match args[0].v_type() {
        // function(MyFunc, [arg], dict)
        VAR_FUNC => args[0].func_name_or_null(),
        // function(dict.MyFunc, [arg])
        VAR_PARTIAL if !args[0].partial_or_null().is_null() => {
            arg_pt = args[0].partial_or_null();
            unsafe { partial_name(arg_pt) }
        }
        // function('MyFunc', [arg], dict)
        _ => {
            use_string = true;
            arg_string(&mut numbuf, &args[0]) as *mut c_char
        }
    };

    // An autoload name is left alone: it may not be loaded yet, and
    // checking would load it.
    let mut trans_name = Owned(ptr::null_mut());
    if (use_string && !has_char(unsafe { cstr::at(s) }, AUTOLOAD_CHAR)) || is_funcref {
        let mut name = s;
        let out = &raw mut name;
        let flags = TFN_INT as c_int
            | TFN_QUIET as c_int
            | TFN_NO_AUTOLOAD as c_int
            | TFN_NO_DEREF as c_int;
        let fd = ptr::null_mut::<FuncDict>();
        // SAFETY: `name` walks a NUL-terminated string the frame owns.
        trans_name = Owned(unsafe { save_function_name(out, false, flags, fd) });
        // Anything left over means the name was not a name.
        if unsafe { *name } as c_int != NUL {
            s = ptr::null_mut();
        }
    }

    if s.is_null()
        || unsafe { *s } as c_int == NUL
        || (use_string && ascii_isdigit(unsafe { *s } as c_int))
        || (is_funcref && trans_name.0.is_null())
    {
        let what = if use_string {
            arg_string(&mut numbuf2, &args[0])
        } else {
            s as *const c_char
        };
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let what = unsafe { c_str(what) };
        semsg!("E475: Invalid argument: {what}");
        return;
    }
    if !trans_name.0.is_null()
        && if is_funcref {
            unsafe { find_func(trans_name.0) }.is_null()
        } else {
            !unsafe { translated_function_exists(trans_name.0) }
        }
    {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let s = unsafe { c_str(s) };
        semsg!("E700: Unknown function: {s}");
        return;
    }

    // Expand `s:` and `<SID>` into `<SNR>nr_` so the result can be
    // called from another script. `trans_function_name` would do it
    // too, but some plugins depend on the name staying printable.
    let name =
        if unsafe { cstr::starts_with(s, b"s:") } || unsafe { cstr::starts_with(s, b"<SID>") } {
            unsafe { get_scriptlocal_funcname(s) }
        } else {
            unsafe { xstrdup(s) }
        };

    // The second argument may be either the argument list or the dict;
    // a third settles it.
    let mut dict_idx = 0;
    let mut arg_idx = 0;
    let mut list = ptr::null_mut::<List>();
    if args.len() > 1 {
        if args.len() > 2 {
            arg_idx = 1;
            dict_idx = 2;
        } else if args.get(1).is_some_and(|arg| arg.v_type() == VAR_DICT) {
            dict_idx = 1;
        } else {
            arg_idx = 1;
        }
        if dict_idx > 0 {
            if tv_check_for_dict_arg(args, dict_idx).is_err() {
                unsafe { xfree(name as *mut c_void) };
                return;
            }
            // v:_null_dict binds nothing.
            if args[dict_idx].dict_or_null().is_null() {
                dict_idx = 0;
            }
        }
        if arg_idx > 0 {
            if args[arg_idx as usize].v_type() != VAR_LIST {
                let msg = c"E923: Second argument of function() must be a list or a dict";
                emsg(gettext(msg));
                unsafe { xfree(name as *mut c_void) };
                return;
            }
            list = args[arg_idx as usize].list_or_null();
            if unsafe { tv_list_len(list) } == 0 {
                arg_idx = 0;
            } else if unsafe { tv_list_len(list) } > MAX_FUNC_ARGS as c_int {
                unsafe { emsg_funcname(e_toomanyarg.as_ptr(), s) };
                unsafe { xfree(name as *mut c_void) };
                return;
            }
        }
    }

    // Nothing bound and nothing to bind: a plain Funcref will do.
    if dict_idx == 0 && arg_idx == 0 && arg_pt.is_null() && !is_funcref {
        result.write_func_name(name);
        unsafe { func_ref(name) };
        return;
    }

    let pt = unsafe { xcalloc(1, size_of::<Partial>()) } as *mut Partial;
    if arg_idx > 0 || (!arg_pt.is_null() && unsafe { (*arg_pt).pt_argc } > 0) {
        // The bound arguments of the partial being extended come
        // first, then this call's.
        let arg_len = if arg_pt.is_null() {
            0
        } else {
            unsafe { (*arg_pt).pt_argc }
        };
        let lv_len = unsafe { tv_list_len(list) };
        unsafe { (*pt).pt_argc = arg_len + lv_len };
        let bytes = size_of::<TypVal>() * unsafe { (*pt).pt_argc } as usize;
        unsafe { (*pt).pt_argv = xmalloc(bytes) as *mut TypVal };
        let mut i = 0;
        while i < arg_len {
            let from = unsafe { (*arg_pt).pt_argv.add(i as usize) };
            let to = unsafe { (*pt).pt_argv.add(i as usize) };
            unsafe { tv_copy(&*from, &mut *to) };
            i += 1;
        }
        for li in tv_list_iter(unsafe { list.as_ref() }) {
            unsafe { tv_copy(&li.li_tv, &mut *(*pt).pt_argv.add(i as usize)) };
            i += 1;
        }
    }

    if dict_idx > 0 {
        // Bound explicitly, so `pt_auto` stays false.
        unsafe { (*pt).pt_dict = args[dict_idx].dict_or_null() };
        unsafe { (*(*pt).pt_dict).dv_refcount.retain() };
    } else if !arg_pt.is_null() {
        // A dict bound automatically stays bound automatically. This
        // is what makes `function(dict.func, [], dict)` keep `dict`.
        unsafe { (*pt).pt_dict = (*arg_pt).pt_dict };
        unsafe { (*pt).pt_auto = (*arg_pt).pt_auto };
        if !unsafe { (*pt).pt_dict }.is_null() {
            unsafe { (*(*pt).pt_dict).dv_refcount.retain() };
        }
    }

    unsafe { (*pt).pt_refcount = Refcount::ONE };
    if !arg_pt.is_null() && !unsafe { (*arg_pt).pt_func }.is_null() {
        unsafe { (*pt).pt_func = (*arg_pt).pt_func };
        unsafe { func_ptr_ref((*pt).pt_func) };
        unsafe { xfree(name as *mut c_void) };
    } else if is_funcref {
        unsafe { (*pt).pt_func = find_func(trans_name.0) };
        unsafe { func_ptr_ref((*pt).pt_func) };
        unsafe { xfree(name as *mut c_void) };
    } else {
        unsafe { (*pt).pt_name = name };
        unsafe { func_ref(name) };
    }
    result.write_partial(pt);
}

/// `funcref({name} [, {arglist}] [, {dict}])`
pub fn f_funcref(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    common_function(args, result, true);
}

/// `function({name} [, {arglist}] [, {dict}])`
pub fn f_function(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    common_function(args, result, false);
}

/// `garbagecollect([{atexit}])` — schedules a collection; the argument asks
/// for one on exit as well.
pub fn f_garbagecollect(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let _rettv = result;
    want_garbage_collect.set(true);
    if !args.is_empty() && arg_number(&args[0]) == 1 {
        garbage_collect_at_exit.set(true);
    }
}

/// `libcall()` and `libcallnr()`.
fn libcall_common(args: &[TypVal], result: &mut TypVal, out_type: VarType) {
    result.write_empty(out_type);
    if out_type != VAR_NUMBER {
        result.write_string(ptr::null_mut());
    }
    // SAFETY throughout: the frame is live; the two names and the string argument are
    // owned by arguments and outlive the call.
    if check_secure() {
        return;
    }
    if !args.first().is_some_and(|arg| arg.v_type() == VAR_STRING)
        || !args.get(1).is_some_and(|arg| arg.v_type() == VAR_STRING)
    {
        return;
    }
    let libname = args[0].string_or_null();
    let funcname = args[1].string_or_null();
    let arg3 = &args[2];
    let str_in = if arg3.v_type() == VAR_STRING {
        arg3.string_or_null()
    } else {
        ptr::null_mut()
    };
    // A VAR_STRING third argument with a NULL v_string falls through to
    // the int-taking prototype, reading the same union as a number.
    // Upstream quirk, preserved.
    let arg = if str_in.is_null() {
        LibcallArg::Int(arg3.number_or_zero() as c_int)
    } else {
        LibcallArg::Str(unsafe { CStr::from_ptr(str_in) })
    };
    let want = if out_type == VAR_STRING {
        LibcallReturn::Str
    } else {
        LibcallReturn::Int
    };
    let answer = if libname.is_null() || funcname.is_null() {
        None
    } else {
        unsafe { os_libcall(CStr::from_ptr(libname), CStr::from_ptr(funcname), arg, want) }
    };
    match answer {
        None => {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let funcname = unsafe { c_str(funcname) };
            semsg!("E364: Library call failed for \"{funcname}()\"");
        }
        Some(LibcallResult::Str(s)) => {
            result.write_string(s.map_or(ptr::null_mut(), CString::into_raw));
        }
        Some(LibcallResult::Int(n)) => result.write_number(n as VarNumber),
    }
}

/// `libcall({lib}, {func}, {arg})`
pub fn f_libcall(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    libcall_common(args, result, VAR_STRING);
}

/// `libcallnr({lib}, {func}, {arg})`
pub fn f_libcallnr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    libcall_common(args, result, VAR_NUMBER);
}

/// `luaeval({expr} [, {expr}])`
pub fn f_luaeval(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY throughout: the frame is live and the chunk outlives the call.
    let chunk = arg_string_chk(&mut numbuf, &args[0]);
    if chunk.is_null() {
        return;
    }
    // Lua sees `_A`; with no second argument that is upstream's empty slot,
    // which `nlua_push_typval` reads as nil.
    let absent = TypVal::Unknown;
    let arg = args.get(1).unwrap_or(&absent);
    unsafe { nlua_typval_eval(cstr_to_string(chunk), arg, result) };
}

/// `py3eval({expr})`
pub fn f_py3eval(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value.
    unsafe { script_host_eval(c"python3".as_ptr() as *mut c_char, args, result) };
}

/// `perleval({expr})`
pub fn f_perleval(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value.
    unsafe { script_host_eval(c"perl".as_ptr() as *mut c_char, args, result) };
}

/// `rubyeval({expr})`
pub fn f_rubyeval(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value.
    unsafe { script_host_eval(c"ruby".as_ptr() as *mut c_char, args, result) };
}
