//! Calling things: `call()`, `function()`, `eval()`, `execute()` and the
//! bridges to the script hosts.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::{AUTOLOAD_CHAR, MAX_FUNC_ARGS, TFN_INT, TFN_NO_AUTOLOAD, TFN_NO_DEREF, TFN_QUIET};
use crate::api::private::helpers::cstr_as_string;
use crate::ascii::ascii_isdigit;
use crate::autocmd::{au_exists, autocmd_supported};
use crate::charset::skipwhite;
use crate::eval::EVALARG_EVALUATE;
use crate::eval::typval::{
    NumBuf, tv_check_for_dict_arg, tv_check_for_list_arg, tv_copy, tv_get_number,
    tv_get_string_buf_chk, tv_list_first, tv_list_len, tv_list_ref, tv_list_unref,
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
use crate::main::{
    capture_ga, e_invarg2, e_invexpr2, e_libcall, e_toomanyarg, e_trailing_arg,
    e_unknown_function_str, emsg_noredir, emsg_silent, garbage_collect_at_exit, msg_col,
    need_clr_eos, redir_off, want_garbage_collect,
};
use crate::memory::{strnequal, xcalloc, xfree, xmalloc, xstrdup};
use crate::message::emsg;
use crate::os::cshim::{gettext, strncmp};
use crate::os::dl::{LibcallArg, LibcallResult, LibcallReturn, os_libcall};
use crate::os::env::{expand_env_save, os_env_exists};
use crate::semsg_c;
use crate::strings::vim_strchr;
use crate::types::{
    EvalFuncData, FAIL, NUL, OK, Refcount, VAR_DICT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL,
    VAR_STRING, VarType, funcdict_T, garray_T, list_T, listitem_T, partial_T, typval_T, uint8_t,
    varnumber_T,
};
use ::libc::strcmp;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

/// The size of a `tv_get_string_buf_chk` scratch buffer. `NUMBUFLEN` in the
/// C.
const NUMBUFLEN: usize = 65;

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
pub unsafe fn f_call(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live; every pointer below either belongs to an
    // argument or is one this body allocated and releases.
    unsafe {
        if tv_check_for_list_arg(args.ptr(0), 1) == FAIL {
            return;
        }
        // A null List is v:_null_list, which calls nothing.
        if args.get(1).vval.v_list.is_null() {
            return;
        }

        let mut partial = ptr::null_mut::<partial_T>();
        // Only the Lua-table arm allocates; the others borrow.
        let mut owned = false;
        let mut func = match args.ty(0) {
            VAR_FUNC => args.get(0).vval.v_string,
            VAR_PARTIAL => {
                partial = args.get(0).vval.v_partial;
                partial_name(partial)
            }
            _ if nlua_is_table_from_lua(args.ptr(0)) => {
                owned = true;
                nlua_register_table_as_callable(args.ptr(0))
            }
            _ => numbuf.string(args.ptr(0)) as *mut c_char,
        };
        if func.is_null() || *func as c_int == NUL {
            // Upstream returns here without releasing an owned name.
            return;
        }

        // A String name is resolved through the function-name translator,
        // which is what turns `s:`/`<SID>` into the real name.
        let mut tofree = Owned(ptr::null_mut());
        if args.ty(0) == VAR_STRING {
            let mut p = func;
            tofree = Owned(trans_function_name(
                &raw mut p,
                false,
                TFN_INT as c_int | TFN_QUIET as c_int,
                ptr::null_mut::<funcdict_T>(),
                ptr::null_mut::<*mut partial_T>(),
            ));
            if tofree.0.is_null() {
                emsg_funcname(e_unknown_function_str.as_ptr(), func);
                return;
            }
            func = tofree.0;
        }

        // A bad {dict} skips the call but still runs the cleanup below.
        let selfdict = if !args.has(2) {
            Some(ptr::null_mut())
        } else if tv_check_for_dict_arg(args.ptr(0), 2) == FAIL {
            None
        } else {
            Some(args.get(2).vval.v_dict)
        };
        if let Some(selfdict) = selfdict {
            func_call(func, args.ptr(1), partial, selfdict, rettv);
        }

        if owned {
            func_unref(func);
        }
    }
}

/// `eval({string})`
pub unsafe fn f_eval(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut evalarg = EVALARG_EVALUATE;
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live and `s` walks a string an argument owns.
    unsafe {
        let mut s = numbuf.string_chk(args.ptr(0));
        if !s.is_null() {
            s = skipwhite(s);
        }
        // Kept for the message: `eval1` advances `s` past what it consumed.
        let expr_start = s;
        if s.is_null() || eval1(&raw mut s as *mut *mut c_char, rettv, &raw mut evalarg) == FAIL {
            if !expr_start.is_null() && !aborting() {
                semsg_c!(gettext(e_invexpr2.as_ptr()), expr_start);
            }
            need_clr_eos.set(false);
            rettv.v_type = VAR_NUMBER;
            rettv.vval.v_number = 0;
        } else if *s as c_int != NUL {
            semsg_c!(gettext(e_trailing_arg.as_ptr()), s);
        }
    }
}

/// Where the `:execute` List form is up to, as `do_cmdline`'s cookie.
struct ListLines {
    /// Held only to keep the reference count honest; the walk uses `item`.
    _list: *mut list_T,
    item: *const listitem_T,
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
    // SAFETY: the caller's obligation; `buf` outlives the string
    // `tv_get_string_buf_chk` may park in it, because the duplicate is made
    // before returning.
    unsafe {
        let state = &mut *(cookie as *mut ListLines);
        let Some(item) = state.item.as_ref() else {
            return ptr::null_mut();
        };
        let mut buf = [0 as c_char; NUMBUFLEN];
        let s = tv_get_string_buf_chk(&raw const item.li_tv, buf.as_mut_ptr());
        state.item = item.li_next;
        if s.is_null() {
            ptr::null_mut()
        } else {
            xstrdup(s)
        }
    }
}

/// `execute()` and `win_execute()`: run commands with output captured.
///
/// `arg_off` is where this caller's arguments start, since `win_execute()`
/// puts a window in front of them.
///
/// # Safety
/// `argvars` is a dispatcher argument array and `rettv` its return value.
pub unsafe fn execute_common(argvars: *mut typval_T, rettv: *mut typval_T, arg_off: c_int) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's obligation, which is `Args::new`'s.
    let args = unsafe { Args::new(argvars) };
    let cmd_idx = arg_off as usize;
    let silent_idx = cmd_idx + 1;

    let save_emsg_silent = emsg_silent.get();
    let save_emsg_noredir = emsg_noredir.get();
    let save_redir_off = redir_off.get();
    let save_capture_ga = capture_ga.get();
    let save_msg_col = msg_col.get();
    let mut echo_output = false;
    let mut silence = true;

    // SAFETY: the frame is live; `capture_local` outlives every command run
    // below, and `rettv` adopts its allocation at the end.
    unsafe {
        if check_secure() {
            return;
        }

        if args.has(silent_idx) {
            let mut buf = [0 as c_char; NUMBUFLEN];
            let s = tv_get_string_buf_chk(args.ptr(silent_idx), buf.as_mut_ptr());
            if s.is_null() {
                return;
            }
            // An explicit empty {silent} means "not silent", and is the
            // only spelling that leaves the cursor column alone.
            if *s as c_int == NUL {
                echo_output = true;
            }
            // Any prefix of "silent" silences; only the exact "silent!"
            // also silences errors.
            silence = strncmp(s, c"silent".as_ptr(), 6) == 0;
            if strcmp(s, c"silent!".as_ptr()) == 0 {
                emsg_silent.set(1);
                emsg_noredir.set(true);
            }
        }
        // Restored either way: an explicit empty {silent} asks for output
        // and still resets what the commands below leave behind.
        let _silenced = Suppress::messages_saved_when(silence);

        let mut capture_local = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ptr::null_mut(),
        };
        ga_init(&raw mut capture_local, size_of::<c_char>() as c_int, 80);
        capture_ga.set(&raw mut capture_local);
        redir_off.set(false);
        if !echo_output {
            msg_col.set(0);
        }

        if args.ty(cmd_idx) != VAR_LIST {
            do_cmdline_cmd(numbuf.string(args.ptr(cmd_idx)));
        } else if !args.get(cmd_idx).vval.v_list.is_null() {
            let list = args.get(cmd_idx).vval.v_list;
            // The List is held across the run: a command may drop the
            // variable holding it.
            tv_list_ref(list);
            let mut cookie = ListLines {
                _list: list,
                item: tv_list_first(list),
            };
            do_cmdline(
                ptr::null_mut(),
                Some(get_list_line as unsafe fn(c_int, *mut c_void, c_int, bool) -> *mut c_char),
                &raw mut cookie as *mut c_void,
                DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE | DoCmdOpts::REPEAT | DoCmdOpts::KEYTYPED,
            );
            tv_list_unref(list);
        }

        emsg_silent.set(save_emsg_silent);
        emsg_noredir.set(save_emsg_noredir);
        redir_off.set(save_redir_off);
        msg_col.set(if echo_output { 0 } else { save_msg_col });

        // Read `capture_ga` back rather than using `capture_local`: a
        // nested `execute()` restores the pointer, and it is the current
        // one that holds this run's output.
        ga_append(capture_ga.get(), NUL as uint8_t);
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = (*capture_ga.get()).ga_data as *mut c_char;
        capture_ga.set(save_capture_ga);
    }
}

/// `execute({command} [, {silent}])`
pub unsafe fn f_execute(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: this is the dispatcher's argument array, which is what
    // `execute_common` needs.
    unsafe { execute_common(argvars, rettv, 0) };
}

/// `exists({expr})` — the sigil in front of the name picks the namespace.
pub unsafe fn f_exists(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live and `p` walks a string an argument owns.
    unsafe {
        let mut p = numbuf.string(args.ptr(0));
        // Not a bool: the `:` arm answers 2 for an exact command name, and
        // that grading is part of `exists()`'s contract.
        let found: c_int = match *p as u8 {
            b'$' => {
                // The environment, or a name that expands to something
                // other than itself.
                (if os_env_exists(p.add(1), false) {
                    true
                } else {
                    let expanded = Owned(expand_env_save(p as *mut c_char));
                    !expanded.0.is_null() && *expanded.0 as u8 != b'$'
                }) as c_int
            }
            b'&' | b'+' => {
                // An option, and nothing may follow it.
                (eval_option(&raw mut p, ptr::null_mut(), true) == OK
                    && *skipwhite(p) as c_int == NUL) as c_int
            }
            b'*' => {
                if strnequal(p, c"*v:lua.".as_ptr(), 7) {
                    nlua_func_exists(p.add(7)) as c_int
                } else {
                    function_exists(p.add(1), false) as c_int
                }
            }
            b':' => cmd_exists(p.add(1)),
            // `##event` asks whether the event name is known at all;
            // `#event` asks whether an autocommand is defined for it.
            b'#' if *p.add(1) as u8 == b'#' => autocmd_supported(p.add(2)) as c_int,
            b'#' => au_exists(p.add(1)) as c_int,
            _ => var_exists(p) as c_int,
        };
        rettv.vval.v_number = found as varnumber_T;
    }
}

/// `function()` and `funcref()`.
///
/// `funcref()` binds the function the name resolves to *now*; `function()`
/// keeps the name and resolves it at call time.
///
/// # Safety
/// `args` is a live call frame.
unsafe fn common_function(args: Args, rettv: &mut typval_T, is_funcref: bool) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    // SAFETY: the frame is live; the partial built below owns every value
    // it copies, and `trans_name`/`name` are released on every path.
    unsafe {
        let mut arg_pt = ptr::null_mut::<partial_T>();
        let mut use_string = false;
        let mut s = match args.ty(0) {
            // function(MyFunc, [arg], dict)
            VAR_FUNC => args.get(0).vval.v_string,
            // function(dict.MyFunc, [arg])
            VAR_PARTIAL if !args.get(0).vval.v_partial.is_null() => {
                arg_pt = args.get(0).vval.v_partial;
                partial_name(arg_pt)
            }
            // function('MyFunc', [arg], dict)
            _ => {
                use_string = true;
                numbuf.string(args.ptr(0)) as *mut c_char
            }
        };

        // An autoload name is left alone: it may not be loaded yet, and
        // checking would load it.
        let mut trans_name = Owned(ptr::null_mut());
        if (use_string && vim_strchr(s, AUTOLOAD_CHAR).is_null()) || is_funcref {
            let mut name = s;
            trans_name = Owned(save_function_name(
                &raw mut name,
                false,
                TFN_INT as c_int
                    | TFN_QUIET as c_int
                    | TFN_NO_AUTOLOAD as c_int
                    | TFN_NO_DEREF as c_int,
                ptr::null_mut::<funcdict_T>(),
            ));
            // Anything left over means the name was not a name.
            if *name as c_int != NUL {
                s = ptr::null_mut();
            }
        }

        if s.is_null()
            || *s as c_int == NUL
            || (use_string && ascii_isdigit(*s as c_int))
            || (is_funcref && trans_name.0.is_null())
        {
            semsg_c!(
                gettext(e_invarg2.as_ptr()),
                if use_string {
                    numbuf2.string(args.ptr(0))
                } else {
                    s as *const c_char
                },
            );
            return;
        }
        if !trans_name.0.is_null()
            && if is_funcref {
                find_func(trans_name.0).is_null()
            } else {
                !translated_function_exists(trans_name.0)
            }
        {
            semsg_c!(gettext(c"E700: Unknown function: %s".as_ptr()), s);
            return;
        }

        // Expand `s:` and `<SID>` into `<SNR>nr_` so the result can be
        // called from another script. `trans_function_name` would do it
        // too, but some plugins depend on the name staying printable.
        let name = if strncmp(s, c"s:".as_ptr(), 2) == 0 || strncmp(s, c"<SID>".as_ptr(), 5) == 0 {
            get_scriptlocal_funcname(s)
        } else {
            xstrdup(s)
        };

        // The second argument may be either the argument list or the dict;
        // a third settles it.
        let mut dict_idx = 0;
        let mut arg_idx = 0;
        let mut list = ptr::null_mut::<list_T>();
        if args.has(1) {
            if args.has(2) {
                arg_idx = 1;
                dict_idx = 2;
            } else if args.ty(1) == VAR_DICT {
                dict_idx = 1;
            } else {
                arg_idx = 1;
            }
            if dict_idx > 0 {
                if tv_check_for_dict_arg(args.ptr(0), dict_idx) == FAIL {
                    xfree(name as *mut c_void);
                    return;
                }
                // v:_null_dict binds nothing.
                if args.get(dict_idx as usize).vval.v_dict.is_null() {
                    dict_idx = 0;
                }
            }
            if arg_idx > 0 {
                if args.ty(arg_idx as usize) != VAR_LIST {
                    emsg(gettext(
                        c"E923: Second argument of function() must be a list or a dict".as_ptr(),
                    ));
                    xfree(name as *mut c_void);
                    return;
                }
                list = args.get(arg_idx as usize).vval.v_list;
                if tv_list_len(list) == 0 {
                    arg_idx = 0;
                } else if tv_list_len(list) > MAX_FUNC_ARGS as c_int {
                    emsg_funcname(e_toomanyarg.as_ptr(), s);
                    xfree(name as *mut c_void);
                    return;
                }
            }
        }

        // Nothing bound and nothing to bind: a plain Funcref will do.
        if dict_idx == 0 && arg_idx == 0 && arg_pt.is_null() && !is_funcref {
            rettv.v_type = VAR_FUNC;
            rettv.vval.v_string = name;
            func_ref(name);
            return;
        }

        let pt = xcalloc(1, size_of::<partial_T>()) as *mut partial_T;
        if arg_idx > 0 || (!arg_pt.is_null() && (*arg_pt).pt_argc > 0) {
            // The bound arguments of the partial being extended come
            // first, then this call's.
            let arg_len = if arg_pt.is_null() {
                0
            } else {
                (*arg_pt).pt_argc
            };
            let lv_len = tv_list_len(list);
            (*pt).pt_argc = arg_len + lv_len;
            (*pt).pt_argv =
                xmalloc(size_of::<typval_T>() * (*pt).pt_argc as usize) as *mut typval_T;
            let mut i = 0;
            while i < arg_len {
                tv_copy(
                    (*arg_pt).pt_argv.add(i as usize),
                    (*pt).pt_argv.add(i as usize),
                );
                i += 1;
            }
            if lv_len > 0 && !list.is_null() {
                let mut li = (*list).lv_first;
                while !li.is_null() {
                    tv_copy(&raw mut (*li).li_tv, (*pt).pt_argv.add(i as usize));
                    i += 1;
                    li = (*li).li_next;
                }
            }
        }

        if dict_idx > 0 {
            // Bound explicitly, so `pt_auto` stays false.
            (*pt).pt_dict = args.get(dict_idx as usize).vval.v_dict;
            (*(*pt).pt_dict).dv_refcount.retain();
        } else if !arg_pt.is_null() {
            // A dict bound automatically stays bound automatically. This
            // is what makes `function(dict.func, [], dict)` keep `dict`.
            (*pt).pt_dict = (*arg_pt).pt_dict;
            (*pt).pt_auto = (*arg_pt).pt_auto;
            if !(*pt).pt_dict.is_null() {
                (*(*pt).pt_dict).dv_refcount.retain();
            }
        }

        (*pt).pt_refcount = Refcount::ONE;
        if !arg_pt.is_null() && !(*arg_pt).pt_func.is_null() {
            (*pt).pt_func = (*arg_pt).pt_func;
            func_ptr_ref((*pt).pt_func);
            xfree(name as *mut c_void);
        } else if is_funcref {
            (*pt).pt_func = find_func(trans_name.0);
            func_ptr_ref((*pt).pt_func);
            xfree(name as *mut c_void);
        } else {
            (*pt).pt_name = name;
            func_ref(name);
        }
        rettv.v_type = VAR_PARTIAL;
        rettv.vval.v_partial = pt;
    }
}

/// `funcref({name} [, {arglist}] [, {dict}])`
pub unsafe fn f_funcref(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live.
    unsafe { common_function(args, rettv, true) };
}

/// `function({name} [, {arglist}] [, {dict}])`
pub unsafe fn f_function(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live.
    unsafe { common_function(args, rettv, false) };
}

/// `garbagecollect([{atexit}])` — schedules a collection; the argument asks
/// for one on exit as well.
pub unsafe fn f_garbagecollect(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, _rettv) = frame!(argvars, rettv);
    want_garbage_collect.set(true);
    // SAFETY: the frame is live.
    unsafe {
        // Exactly 1, not "non-zero".
        if args.has(0) && tv_get_number(args.ptr(0)) == 1 {
            garbage_collect_at_exit.set(true);
        }
    }
}

/// `libcall()` and `libcallnr()`.
///
/// # Safety
/// `args` is a live call frame.
unsafe fn libcall_common(args: Args, rettv: &mut typval_T, out_type: VarType) {
    rettv.v_type = out_type;
    if out_type != VAR_NUMBER {
        rettv.vval.v_string = ptr::null_mut();
    }
    // SAFETY: the frame is live; the two names and the string argument are
    // owned by arguments and outlive the call.
    unsafe {
        if check_secure() {
            return;
        }
        if args.ty(0) != VAR_STRING || args.ty(1) != VAR_STRING {
            return;
        }
        let libname = args.get(0).vval.v_string;
        let funcname = args.get(1).vval.v_string;
        let arg3 = args.get(2);
        let str_in = if arg3.v_type == VAR_STRING {
            arg3.vval.v_string
        } else {
            ptr::null_mut()
        };
        // A VAR_STRING third argument with a NULL v_string falls through to
        // the int-taking prototype, reading the same union as a number.
        // Upstream quirk, preserved.
        let arg = if str_in.is_null() {
            LibcallArg::Int(arg3.vval.v_number as c_int)
        } else {
            LibcallArg::Str(CStr::from_ptr(str_in))
        };
        let want = if out_type == VAR_STRING {
            LibcallReturn::Str
        } else {
            LibcallReturn::Int
        };
        let result = if libname.is_null() || funcname.is_null() {
            None
        } else {
            os_libcall(CStr::from_ptr(libname), CStr::from_ptr(funcname), arg, want)
        };
        match result {
            None => {
                semsg_c!(gettext(e_libcall.as_ptr()), funcname);
            }
            Some(LibcallResult::Str(s)) => {
                rettv.vval.v_string = s.map_or(ptr::null_mut(), CString::into_raw);
            }
            Some(LibcallResult::Int(n)) => rettv.vval.v_number = n as varnumber_T,
        }
    }
}

/// `libcall({lib}, {func}, {arg})`
pub unsafe fn f_libcall(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live.
    unsafe { libcall_common(args, rettv, VAR_STRING) };
}

/// `libcallnr({lib}, {func}, {arg})`
pub unsafe fn f_libcallnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live.
    unsafe { libcall_common(args, rettv, VAR_NUMBER) };
}

/// `luaeval({expr} [, {expr}])`
pub unsafe fn f_luaeval(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live and the chunk outlives the call.
    unsafe {
        let chunk = numbuf.string_chk(args.ptr(0));
        if chunk.is_null() {
            return;
        }
        nlua_typval_eval(cstr_as_string(chunk), args.ptr(1), rettv);
    }
}

/// `py3eval({expr})`
pub unsafe fn f_py3eval(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value.
    unsafe { script_host_eval(c"python3".as_ptr() as *mut c_char, argvars, rettv) };
}

/// `perleval({expr})`
pub unsafe fn f_perleval(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value.
    unsafe { script_host_eval(c"perl".as_ptr() as *mut c_char, argvars, rettv) };
}

/// `rubyeval({expr})`
pub unsafe fn f_rubyeval(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the dispatcher's argument array and return value.
    unsafe { script_host_eval(c"ruby".as_ptr() as *mut c_char, argvars, rettv) };
}
