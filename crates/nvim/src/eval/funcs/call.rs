//! Calling things: `call()`, `function()`, `eval()`, `execute()` and the
//! bridges to the script hosts.
#![forbid(unsafe_code)]

use super::wrappers::arg_number;
use super::{AUTOLOAD_CHAR, MAX_FUNC_ARGS, TFN_INT, TFN_NO_AUTOLOAD, TFN_NO_DEREF, TFN_QUIET};
use crate::ascii::ascii_isdigit;
use crate::autocmd::{au_exists, autocmd_supported};
use crate::eval::gc::{garbage_collect_at_exit, want_garbage_collect};
use crate::eval::typval::{
    ListRef, NumBuf, PartialRef, list_iter, list_len, tv_check_for_dict_arg, tv_check_for_list_arg,
};
use crate::eval::userfunc::{
    emsg_funcname, find_func, func_call, func_exists, func_ptr_ref, func_ref_name, func_unref_name,
    function_exists, save_function_name, scriptlocal_funcname, trans_function_name,
    translated_function_exists,
};
use crate::eval::vars::var_exists;
use crate::eval::{Cursor, eval_option, eval1, partial_name, script_host_eval};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::{DoCmdOpts, cmd_exists, do_cmdline_cmd, do_cmdline_getter};
use crate::ex_eval::aborting;
use crate::global_cell::GlobalCell;
use crate::guard::Suppress;
use crate::lua::executor::{
    nlua_func_exists, nlua_is_table_from_lua, nlua_typval_eval, register_table_as_callable,
};
use crate::memory::ThinCString;
use crate::message::emsg;
use crate::message::state::{emsg_noredir, emsg_silent, msg_col, need_clr_eos, redir_off};
use crate::message::{capture_finish, capture_start, e_toomanyarg, e_unknown_function_str};
use crate::message_fmt::{msg_bytes, msg_cstr, msg_cstr_opt};
use crate::os::cshim::gettext;
use crate::os::dl::{LibcallArg, LibcallResult, LibcallReturn, os_libcall};
use crate::os::env::env_exists;
use crate::os::env::expand::expand_env_save_opt_of;
use crate::semsg;
use crate::strings::has_char;
use crate::types::{
    EvalFuncData, List, NUL, Partial, TypVal, VAR_DICT, VAR_FUNC, VAR_LIST, VAR_NUMBER,
    VAR_PARTIAL, VAR_STRING, VarNumber, VarType,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// `call({func}, {arglist} [, {dict}])`
pub fn f_call(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if tv_check_for_list_arg(args, 1).is_err() {
        return;
    }
    // A null List is v:_null_list, which calls nothing.
    if args[1].list_shared().is_none() {
        return;
    }

    let mut partial = None;
    // Only the Lua-table arm registers a name; the others borrow.
    let mut owned = false;
    let lua_name;
    let func = match args[0].v_type() {
        VAR_FUNC => args[0].func_name().map(ThinCString::as_cstr),
        VAR_PARTIAL => {
            partial = args[0].partial_shared();
            Some(args[0].partial_ref().map_or(c"", partial_name))
        }
        _ if nlua_is_table_from_lua(&args[0]) => {
            owned = true;
            lua_name = register_table_as_callable(&args[0]);
            lua_name.as_ref().map(ThinCString::as_cstr)
        }
        _ => Some(numbuf.string(&args[0])),
    };
    let Some(mut func) = func.filter(|func| !func.is_empty()) else {
        // Upstream returns here without releasing an owned name.
        return;
    };

    // A String name is resolved through the function-name translator,
    // which is what turns `s:`/`<SID>` into the real name.
    let tofree;
    if args[0].v_type() == VAR_STRING {
        let flags = TFN_INT as c_int | TFN_QUIET as c_int;
        let written = func.to_bytes();
        let Some(translated) = trans_function_name(written, false, flags, false).name else {
            emsg_funcname(e_unknown_function_str, written);
            return;
        };
        tofree = translated;
        func = tofree.as_cstr();
    }

    // A bad {dict} skips the call but still runs the cleanup below.
    let selfdict = if args.len() <= 2 {
        Some(None)
    } else if tv_check_for_dict_arg(args, 2).is_err() {
        None
    } else {
        Some(args[2].dict_shared())
    };
    if let Some(selfdict) = selfdict {
        let _ = func_call(func, &args[1], partial, selfdict, result);
    }

    if owned {
        func_unref_name(func);
    }
}

/// `eval({string})`
pub fn f_eval(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let Some(text) = numbuf.string_chk(&args[0]) else {
        need_clr_eos.set(false);
        result.write_number(0);
        return;
    };
    let mut cursor = Cursor::new(text.to_bytes());
    cursor.skip_white();
    // Kept for the message: `eval1` advances the cursor past what it read.
    let expr_start = cursor.offset();
    if eval1(&mut cursor, result, true).is_err() {
        if !aborting() {
            let expr_start = msg_bytes(&cursor.text()[expr_start..]);
            semsg!("E15: Invalid expression: \"{expr_start}\"");
        }
        need_clr_eos.set(false);
        result.write_number(0);
    } else if cursor.byte() != 0 {
        let rest = msg_bytes(cursor.rest());
        semsg!("E488: Trailing characters: {rest}");
    }
}

/// Where an `execute([...])` List walk is up to.
struct ListLines {
    /// The list being walked, held across the run: a command may drop the
    /// variable holding it.
    list: ListRef,
    /// Where the walk is: an index, because a command in the list can edit
    /// the very list it is being read from.
    at: usize,
}

/// The `execute([...])` walks in progress, innermost last. A walk's line
/// getter is handed its depth here as its cookie: `execute()` nests when a
/// command in the list calls it again.
static LIST_WALKS: GlobalCell<Vec<ListLines>> = GlobalCell::new(Vec::new());

/// `do_cmdline`'s line getter for `execute([...])`: one allocated line per
/// List item, and null when the List runs out or an item is no String.
fn get_list_line(_c: c_int, cookie: *mut c_void, _indent: c_int, _do_concat: bool) -> *mut c_char {
    let depth = cookie.addr();
    // The item is copied out of the walk's cell: reading it as a String may
    // report an error, which must not happen under the cell's borrow.
    let item = LIST_WALKS.with_mut(|walks| {
        let walk = &mut walks[depth];
        let item = walk.list.items().get(walk.at)?.li_tv.clone();
        walk.at += 1;
        Some(item)
    });
    let Some(item) = item else {
        return ptr::null_mut();
    };
    let mut buf = NumBuf::new();
    let line = buf.string_chk(&item).map(ThinCString::from_cstr);
    line.map_or(ptr::null_mut(), ThinCString::into_raw)
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
    let save_msg_col = msg_col.get();
    let mut echo_output = false;
    let mut silence = true;

    if check_secure() {
        return;
    }

    if args.len() > silent_idx {
        let mut buf = NumBuf::new();
        let Some(s) = buf.bytes_chk(&args[silent_idx]) else {
            return;
        };
        // An explicit empty {silent} means "not silent", and is the
        // only spelling that leaves the cursor column alone.
        if s.is_empty() {
            echo_output = true;
        }
        // Any prefix of "silent" silences; only the exact "silent!"
        // also silences errors.
        silence = s.starts_with(b"silent");
        if s == b"silent!" {
            emsg_silent.set(1);
            emsg_noredir.set(true);
        }
    }
    // Restored either way: an explicit empty {silent} asks for output
    // and still resets what the commands below leave behind.
    let _silenced = Suppress::messages_saved_when(silence);

    let outer_capture = capture_start();
    redir_off.set(false);
    if !echo_output {
        msg_col.set(0);
    }

    if !args
        .get(cmd_idx)
        .is_some_and(|arg| arg.v_type() == VAR_LIST)
    {
        let _ = do_cmdline_cmd(numbuf.string(&args[cmd_idx]));
    } else if let Some(list) = args[cmd_idx].list_handle() {
        let depth = LIST_WALKS.with_mut(|walks| {
            walks.push(ListLines { list, at: 0 });
            walks.len() - 1
        });
        let opts = DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE | DoCmdOpts::REPEAT | DoCmdOpts::KEYTYPED;
        let _ = do_cmdline_getter(get_list_line, ptr::without_provenance_mut(depth), opts);
        let walk = LIST_WALKS.with_mut(|walks| walks.pop());
        debug_assert!(walks_matched(walk.as_ref(), depth), "execute(): walks nest");
        // The list's reference goes once the cell is no longer borrowed.
        drop(walk);
    }

    emsg_silent.set(save_emsg_silent);
    emsg_noredir.set(save_emsg_noredir);
    redir_off.set(save_redir_off);
    msg_col.set(if echo_output { 0 } else { save_msg_col });

    let captured = capture_finish(outer_capture);
    result.write_string(Some(ThinCString::from_bytes(&captured)));
}

/// Whether the walk popped is the one pushed at `depth`: the stack's own
/// length says so, since nothing else pushes onto it.
fn walks_matched(walk: Option<&ListLines>, depth: usize) -> bool {
    walk.is_some() && LIST_WALKS.with(Vec::len) == depth
}

/// `execute({command} [, {silent}])`
pub fn f_execute(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    execute_common(args, result, 0);
}

/// `exists({expr})` — the sigil in front of the name picks the namespace.
pub fn f_exists(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let p = numbuf.string(&args[0]);
    let bytes = p.to_bytes();
    // Not a bool: the `:` arm answers 2 for an exact command name, and
    // that grading is part of `exists()`'s contract.
    let found: c_int = match bytes.first().copied().unwrap_or(0) {
        b'$' => {
            // The environment, or a name that expands to something
            // other than itself.
            c_int::from(
                env_exists(&p[1..], false)
                    || expand_env_save_opt_of(p, false)
                        .as_cstr()
                        .to_bytes()
                        .first()
                        != Some(&b'$'),
            )
        }
        b'&' | b'+' => {
            // An option, and nothing may follow it.
            let mut cursor = Cursor::new(bytes);
            let found = eval_option(&mut cursor, None, true).is_ok() && {
                cursor.skip_white();
                cursor.byte() == NUL as u8
            };
            c_int::from(found)
        }
        b'*' => {
            if bytes.starts_with(b"*v:lua.") {
                c_int::from(nlua_func_exists(&p[7..]))
            } else {
                c_int::from(function_exists(&bytes[1..], false))
            }
        }
        b':' => cmd_exists(&p[1..]),
        // `##event` asks whether the event name is known at all;
        // `#event` asks whether an autocommand is defined for it.
        b'#' if bytes.get(1) == Some(&b'#') => c_int::from(autocmd_supported(&p[2..])),
        b'#' => c_int::from(au_exists(&p[1..])),
        _ => c_int::from(var_exists(bytes)),
    };
    result.write_number(VarNumber::from(found));
}

/// `function()` and `funcref()`.
///
/// `funcref()` binds the function the name resolves to *now*; `function()`
/// keeps the name and resolves it at call time.
fn common_function(args: &[TypVal], result: &mut TypVal, is_funcref: bool) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut arg_pt: Option<&Partial> = None;
    let mut use_string = false;
    // The name as given; `None` once it has proved not to be one.
    let mut given: Option<&CStr> = match args[0].v_type() {
        // function(MyFunc, [arg], dict)
        VAR_FUNC => args[0].func_name().map(ThinCString::as_cstr),
        // function(dict.MyFunc, [arg])
        VAR_PARTIAL if args[0].partial_ref().is_some() => {
            arg_pt = args[0].partial_ref();
            Some(arg_pt.map_or(c"", partial_name))
        }
        // function('MyFunc', [arg], dict)
        _ => {
            use_string = true;
            Some(numbuf.string(&args[0]))
        }
    };

    // An autoload name is left alone: it may not be loaded yet, and
    // checking would load it.
    let mut trans_name = None;
    if let Some(written) = given
        && ((use_string && !has_char(written, AUTOLOAD_CHAR)) || is_funcref)
    {
        let flags = TFN_INT as c_int
            | TFN_QUIET as c_int
            | TFN_NO_AUTOLOAD as c_int
            | TFN_NO_DEREF as c_int;
        let written = written.to_bytes();
        let found = save_function_name(written, false, flags, false);
        trans_name = found.name;
        // Anything left over means the name was not a name.
        if found.end < written.len() {
            given = None;
        }
    }

    let first = given.and_then(|name| name.to_bytes().first().copied());
    let s = match given {
        Some(s)
            if first.is_some()
                && !(use_string && first.is_some_and(|c| ascii_isdigit(c_int::from(c))))
                && !(is_funcref && trans_name.is_none()) =>
        {
            s
        }
        _ => {
            let what = if use_string {
                msg_cstr(numbuf2.string(&args[0]))
            } else {
                msg_cstr_opt(given)
            };
            semsg!("E475: Invalid argument: {what}");
            return;
        }
    };
    if let Some(trans_name) = &trans_name
        && if is_funcref {
            !func_exists(trans_name)
        } else {
            !translated_function_exists(trans_name)
        }
    {
        let s = msg_cstr(s);
        semsg!("E700: Unknown function: {s}");
        return;
    }

    // Expand `s:` and `<SID>` into `<SNR>nr_` so the result can be
    // called from another script. `trans_function_name` would do it
    // too, but some plugins depend on the name staying printable.
    let written = s.to_bytes();
    let name = if written.starts_with(b"s:") || written.starts_with(b"<SID>") {
        scriptlocal_funcname(written).map(ThinCString::from)
    } else {
        Some(ThinCString::from_cstr(s))
    };

    // The second argument may be either the argument list or the dict;
    // a third settles it.
    let mut dict_idx = 0;
    let mut arg_idx = 0;
    let mut list: Option<&List> = None;
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
                return;
            }
            // v:_null_dict binds nothing.
            if args[dict_idx].dict_shared().is_none() {
                dict_idx = 0;
            }
        }
        if arg_idx > 0 {
            if args[arg_idx].v_type() != VAR_LIST {
                let msg = c"E923: Second argument of function() must be a list or a dict";
                emsg(gettext(msg));
                return;
            }
            list = args[arg_idx].list_ref();
            if list_len(list) == 0 {
                arg_idx = 0;
            } else if list_len(list) > MAX_FUNC_ARGS as c_int {
                emsg_funcname(e_toomanyarg, written);
                return;
            }
        }
    }

    // Nothing bound and nothing to bind: a plain Funcref will do.
    if dict_idx == 0 && arg_idx == 0 && arg_pt.is_none() && !is_funcref {
        if let Some(name) = &name {
            func_ref_name(name.as_cstr());
        }
        result.write_func_name(name);
        return;
    }

    let mut pt = Partial::EMPTY;
    if arg_idx > 0 || arg_pt.is_some_and(|bound| !bound.pt_argv.is_empty()) {
        // The bound arguments of the partial being extended come
        // first, then this call's.
        let bound = arg_pt.map_or(&[][..], |bound| &bound.pt_argv);
        pt.pt_argv
            .reserve_exact(bound.len() + list.map_or(0, List::len));
        pt.pt_argv.extend(bound.iter().cloned());
        pt.pt_argv
            .extend(list_iter(list).map(|li| li.li_tv.clone()));
    }

    if dict_idx > 0 {
        // Bound explicitly, so `pt_auto` stays false.
        pt.pt_dict = args[dict_idx].dict_handle();
    } else if let Some(bound) = arg_pt {
        // A dict bound automatically stays bound automatically. This
        // is what makes `function(dict.func, [], dict)` keep `dict`.
        pt.pt_dict = bound.pt_dict.clone();
        pt.pt_auto = bound.pt_auto;
    }

    if let Some(func) = arg_pt.and_then(|bound| bound.pt_func.as_ref()) {
        func_ptr_ref(func);
        pt.pt_func = Some(func.clone());
    } else if is_funcref {
        let trans_name = trans_name.as_deref().unwrap_or_default();
        pt.pt_func = find_func(trans_name);
        if let Some(func) = &pt.pt_func {
            func_ptr_ref(func);
        }
    } else {
        pt.pt_name = name;
        if let Some(name) = &pt.pt_name {
            func_ref_name(name.as_cstr());
        }
    }
    result.write_partial(Some(PartialRef::new(pt)));
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
        result.write_string(None);
    }
    if check_secure() {
        return;
    }
    if !args.first().is_some_and(|arg| arg.v_type() == VAR_STRING)
        || !args.get(1).is_some_and(|arg| arg.v_type() == VAR_STRING)
    {
        return;
    }
    let libname = args[0].string_cstr();
    let funcname = args[1].string_cstr();
    let arg3 = &args[2];
    // A VAR_STRING third argument with a NULL v_string falls through to
    // the int-taking prototype, reading the same union as a number.
    // Upstream quirk, preserved.
    let arg = match arg3.string_cstr() {
        Some(text) => LibcallArg::Str(text),
        None => LibcallArg::Int(arg3.number_or_zero() as c_int),
    };
    let want = if out_type == VAR_STRING {
        LibcallReturn::Str
    } else {
        LibcallReturn::Int
    };
    let answer = match (libname, funcname) {
        (Some(libname), Some(funcname)) => os_libcall(libname, funcname, arg, want),
        _ => None,
    };
    match answer {
        None => {
            let funcname = msg_cstr_opt(funcname);
            semsg!("E364: Library call failed for \"{funcname}()\"");
        }
        Some(LibcallResult::Str(s)) => {
            result.write_string(s.map(ThinCString::from));
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
    let Some(chunk) = numbuf.bytes_chk(&args[0]) else {
        return;
    };
    // Lua sees `_A`; with no second argument that is upstream's empty slot,
    // which `nlua_push_typval` reads as nil.
    let absent = TypVal::Unknown;
    let arg = args.get(1).unwrap_or(&absent);
    nlua_typval_eval(chunk, arg, result);
}

/// `py3eval({expr})`
pub fn f_py3eval(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    script_host_eval(c"python3", args, result);
}

/// `perleval({expr})`
pub fn f_perleval(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    script_host_eval(c"perl", args, result);
}

/// `rubyeval({expr})`
pub fn f_rubyeval(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    script_host_eval(c"ruby", args, result);
}
