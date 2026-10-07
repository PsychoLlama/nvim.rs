//! The environment and the paths around it: `environ()`, `expand()`,
//! `stdpath()` and the swap-file queries.
#![forbid(unsafe_code)]

use super::wrappers::{arg_number_chk, dict_alloc_ret};
use super::{
    ENV_SEPCHAR, kXDGCacheHome, kXDGConfigDirs, kXDGConfigHome, kXDGDataDirs, kXDGDataHome,
    kXDGRuntimeDir, kXDGStateHome, tv_get_buf,
};
use crate::cmdexpand::{WildMode, WildOpts, expand_cleanup, expand_one};
use crate::eval::typval::{NumBuf, dict_get_bool, dict_has_key, tv_list_alloc_ret};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::{eval_vars_leading, expand_filename};
use crate::guard::Suppress;
use crate::memline::{list_swap_files, swapfile_dict};
use crate::memory::ThinCString;
use crate::message::emsg;
use crate::message_fmt::msg_cstr;
use crate::option::vars::{p_verbose, p_wic};
use crate::os::env::{os_copy_fullenv, vim_getenv_owned, vim_setenv_named, vim_unsetenv_named};
use crate::os::fs::os_setperm;
use crate::os::stdpaths::{get_appname, xdg_home, xdg_var};
use crate::path::join_fnames;
use crate::semsg;
use crate::types::CmdIdx;
use crate::types::CmdLine;
use crate::types::{
    CmdAddr, EvalFuncData, ExArg, ExArgt, Expand, ExpandContext, OK, OptInt, TypVal, VAR_DICT,
    VAR_LIST, VAR_STRING, VarNumber, XDGVarType, kBoolVarFalse, kListLenShouldKnow,
    kListLenUnknown, kSpecialVarNull,
};
use core::ffi::c_int;

/// `environ()` — the process environment as a Dictionary.
pub fn f_environ(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    dict_alloc_ret(result);
    let env = os_copy_fullenv();
    // Walked backwards, so that when a name appears twice the *first*
    // entry is the one that survives the duplicate check below.
    for entry in env.iter().rev() {
        // A leading '=' is part of the name on the platforms that allow
        // it, so the separator search starts past it.
        let skip = usize::from(entry.first() == Some(&b'='));
        let len = entry[skip..]
            .iter()
            .position(|&byte| byte == b'=')
            .map(|at| skip + at)
            .expect("an environment entry holds a '='");
        debug_assert!(len > 0);
        let (key, value) = (&entry[..len], &entry[len + 1..]);
        if !dict_has_key(result.dict_ref(), key) {
            let dict = result.dict_mut().expect("just allocated");
            let _ = dict.add_str_len(key, Some(value));
        }
    }
}

/// `getenv({name})` — the variable's value, or `v:null` when it is unset.
pub fn f_getenv(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    match vim_getenv_owned(numbuf.string(&args[0])) {
        None => result.write_special(kSpecialVarNull),
        Some(value) => result.write_string(Some(ThinCString::from(value))),
    }
}

/// `expand({string} [, {nosuf} [, {list}]])`.
pub fn f_expand(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut options = WildOpts::SILENT | WildOpts::USE_NL | WildOpts::LIST_NOTFOUND;
    let mut error = false;
    result.write_empty(VAR_STRING);
    // The `{list}` argument is only honoured when `{nosuf}` was given
    // too, because it is the third.
    if args.len() > 1 && args.len() > 2 && arg_number_chk(&args[2], Some(&mut error)) != 0 && !error
    {
        result.write_list(None);
    }
    let s = numbuf.string(&args[0]);
    if matches!(s.to_bytes().first(), Some(b'%' | b'#' | b'<')) {
        // A `%`/`#`/`<` item is resolved by the Ex-command machinery,
        // whose own errors are suppressed unless 'verbose' is set.
        let quiet = p_verbose() == 0 as OptInt;
        let no_emsg = quiet.then(Suppress::emsg);
        let (expanded, errormsg) = eval_vars_leading(s);
        drop(no_emsg);
        if !quiet && let Some(msg) = &errormsg {
            emsg(msg);
        }
        if result.v_type() == VAR_LIST {
            let list = tv_list_alloc_ret(result, isize::from(expanded.is_some()));
            if let Some(expanded) = expanded {
                list.push(TypVal::string(Some(expanded)));
            }
        } else {
            result.write_string(expanded);
        }
        return;
    }
    if args.len() > 1 && arg_number_chk(&args[1], Some(&mut error)) != 0 {
        options |= WildOpts::KEEP_ALL;
    }
    if error {
        // `{list}` may already have made the answer a List; the empty
        // answer keeps whichever tag was chosen.
        if result.v_type() == VAR_LIST {
            result.write_list(None);
        } else {
            result.write_string(None);
        }
        return;
    }
    let mut xpc = Expand::new();
    xpc.context = ExpandContext::Files;
    if p_wic() {
        options |= WildOpts::ICASE;
    }
    let pat = s;
    if result.v_type() == VAR_STRING {
        let all = expand_one(&mut xpc, Some(pat), None, options, WildMode::All);
        result.write_string(all.map(ThinCString::from));
    } else {
        expand_one(&mut xpc, Some(pat), None, options, WildMode::AllKeep);
        let list = tv_list_alloc_ret(result, xpc.match_count() as isize);
        for name in xpc.matches() {
            list.push_str(Some(name.as_cstr()));
        }
        expand_cleanup(&mut xpc);
    }
}

/// `expandcmd({string} [, {options}])` — expand the `%`, `#` and wildcard
/// items in a command line.
pub fn f_expandcmd(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_empty(VAR_STRING);
    // {'errmsg': v:true} asks for the expansion's own error instead of
    // silence.
    let errmsg = args.get(1).is_some_and(|arg| arg.v_type() == VAR_DICT)
        && dict_get_bool(args[1].dict_ref(), b"errmsg", kBoolVarFalse as c_int) != 0;
    let quiet = !errmsg;
    let line = numbuf.bytes(&args[0]).to_vec();
    let mut eap = ExArg {
        line: CmdLine::from_bytes(&line),
        cmdidx: CmdIdx::USER,
        addr_type: CmdAddr::Lines,
        argt: ExArgt::NOSPC,
        ..ExArg::default()
    };
    let mut errormsg = None;
    let _no_emsg = quiet.then(Suppress::emsg);
    if expand_filename(&mut eap, &mut errormsg).is_err()
        && !quiet
        && let Some(msg) = &errormsg
        && !msg.is_empty()
    {
        emsg(msg);
    }
    result.write_string(Some(ThinCString::from_bytes(eap.line.line())));
}

/// `setenv({name}, {val})` — `v:null` unsets.
pub fn f_setenv(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let mut namebuf = NumBuf::new();
    let mut valbuf = NumBuf::new();
    // Coerced before the sandbox check, as upstream has it: the
    // coercion can report an error of its own.
    let name = namebuf.string(&args[0]);
    if check_secure() {
        return;
    }
    if args[1].as_special() == Some(kSpecialVarNull) {
        vim_unsetenv_named(name);
    } else {
        vim_setenv_named(name, valbuf.string(&args[1]));
    }
}

/// `setfperm({fname}, {mode})` — `{mode}` is nine "rwxrwxrwx" characters,
/// any of which is "off" only when it is a `-`.
pub fn f_setfperm(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(0);
    let Some(fname) = numbuf.string_chk(&args[0]) else {
        return;
    };
    let mut modebuf = NumBuf::new();
    let Some(mode_str) = modebuf.string_chk(&args[1]) else {
        return;
    };
    let Ok(mode_bytes) = <&[u8; 9]>::try_from(mode_str.to_bytes()) else {
        let mode_str = msg_cstr(mode_str);
        semsg!("E475: Invalid argument: {mode_str}");
        return;
    };
    let mut mode: c_int = 0;
    for (i, &byte) in mode_bytes.iter().enumerate().rev() {
        if byte != b'-' {
            mode |= 1 << (8 - i);
        }
    }
    result.write_number((os_setperm(fname, mode) == OK) as VarNumber);
}

/// The `config_dirs`/`data_dirs` answer: every directory in the XDG search
/// path, each with the application name appended.
fn get_xdg_var_list(xdg: XDGVarType, result: &mut TypVal) {
    let appname = get_appname(false);
    let list = tv_list_alloc_ret(result, kListLenShouldKnow as isize);
    let Some(dirs) = xdg_var(xdg) else {
        return;
    };
    // An empty entry names no directory.
    for dir in dirs.as_bytes().split(|&byte| byte == ENV_SEPCHAR as u8) {
        if !dir.is_empty() {
            let path = join_fnames(&ThinCString::from_bytes(dir), &appname, true);
            list.push(TypVal::string(Some(ThinCString::from(path))));
        }
    }
}

/// `stdpath({what})`.
pub fn f_stdpath(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_string(None);
    let Some(p) = numbuf.string_chk(&args[0]) else {
        return;
    };
    let dir = match p.to_bytes() {
        b"config" => xdg_home(kXDGConfigHome),
        b"data" => xdg_home(kXDGDataHome),
        b"cache" => xdg_home(kXDGCacheHome),
        // "log" is deliberately the state directory: the log file lives
        // there and there is no XDG log home.
        b"state" | b"log" => xdg_home(kXDGStateHome),
        b"run" => xdg_var(kXDGRuntimeDir).map(ThinCString::into_xstring),
        b"config_dirs" => return get_xdg_var_list(kXDGConfigDirs, result),
        b"data_dirs" => return get_xdg_var_list(kXDGDataDirs, result),
        _ => {
            let p = msg_cstr(p);
            semsg!("E6100: \"{p}\" is not a valid stdpath");
            return;
        }
    };
    result.write_string(dir.map(ThinCString::from));
}

/// `swapfilelist()` — every swap file in 'directory'.
pub fn f_swapfilelist(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    list_swap_files(tv_list_alloc_ret(result, kListLenUnknown as isize));
}

/// `swapinfo({fname})` — what a swap file says about its buffer.
pub fn f_swapinfo(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // The dict is allocated into the return value first, so `swapfile_dict`
    // has somewhere to write.
    dict_alloc_ret(result);
    let fname = numbuf.string(&args[0]);
    swapfile_dict(fname, result.dict_mut().expect("just allocated"));
}

/// `swapname({buf})` — the swap file a buffer is using, if any.
pub fn f_swapname(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_empty(VAR_STRING);
    let name = tv_get_buf(&args[0], 0).and_then(|buffer| buffer.swap_file_name());
    result.write_string(name.map(ThinCString::from));
}
