//! Autocommands, `:filetype` and `:setfiletype` — the commands that decide
//! what a buffer is. Plus `:checkhealth`, which is a Lua entry point.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use crate::semsg_multiline;
use crate::smsg;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use crate::autocmd::{check_nomodeline, do_augroup, do_autocmd};

use crate::buffer::do_modelines;

use crate::ex_docmd::{
    FILETYPE_FILE, FTOFF_FILE, FTPLUGIN_FILE, FTPLUGOF_FILE, INDENT_FILE, INDOFF_FILE,
    cmdmod_split, cmdmod_tab, ex_msg, filetype_detect, filetype_indent, filetype_plugin,
    kOptValTypeString, kRetNilBool,
};
use crate::lua::executor::nlua_exec;
use crate::main::{cmdmod, e_curdir, p_rtp, secure};

use crate::option::set_option_value_give_err;
use crate::options::kOptFiletype;
use crate::os::cshim::strstr;

use crate::os::env::{env_buf, os_getenv_into};
use crate::runtime::RuntimeOpts;

use crate::types::{
    Array, CMD_autocmd, Error, Failed, NUL, Object, OptVal, OptValData, OptionSetFlags, String_0,
    exarg_T, kObjectTypeString, size_t,
};
use crate::usercmd::add_win_cmd_modifiers;
use crate::winlayer::{Buf, Ea};

/// `:autocmd` and `:augroup`.
///
/// Both are refused in a 'secure' context — a modeline or an untrusted
/// config — because an autocommand can run anything later.
pub(crate) unsafe fn ex_autocmd(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if secure.get() != 0 {
        // 2 means "an error was already reported for this".
        secure.set(2);
        eap.errmsg = Some(unsafe { ex_msg(e_curdir.as_ptr()) });
    } else if eap.cmdidx as c_int == CMD_autocmd as c_int {
        unsafe { do_autocmd(eap.raw(), eap.arg, eap.forceit) };
    } else {
        unsafe { do_augroup(eap.arg, eap.forceit != 0) };
    }
}

/// `:doautocmd` — and the modelines that a `<nomodeline>` argument
/// suppresses.
pub(crate) unsafe fn ex_doautocmd(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    let mut arg = eap.arg;
    let call_do_modelines = unsafe { check_nomodeline(&raw mut arg) };
    let mut did_aucmd = false;
    let _ = do_doautocmd(arg, false, &raw mut did_aucmd);
    if call_do_modelines && did_aucmd {
        do_modelines(OptionSetFlags::NONE);
    }
}

/// `:filetype [plugin] [indent] on|off|detect`.
pub(crate) unsafe fn ex_filetype(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if byte(eap.arg) == NUL {
        unsafe { report_filetype_state() };
        return;
    }

    let mut arg = eap.arg;
    let mut plugin = false;
    let mut indent = false;
    loop {
        if strncmp(arg, c"plugin".as_ptr(), 6) == 0 {
            plugin = true;
            arg = unsafe { skipwhite(arg.add(6)) };
        } else if strncmp(arg, c"indent".as_ptr(), 6) == 0 {
            indent = true;
            arg = unsafe { skipwhite(arg.add(6)) };
        } else {
            break;
        }
    }

    if strcmp(arg, c"on".as_ptr()) == 0 || strcmp(arg, c"detect".as_ptr()) == 0 {
        // `:filetype detect` only re-sources the scripts when detection
        // was off; `:filetype on` always does.
        if byte(arg) == 'o' as c_int || filetype_detect.get() != Some(true) {
            let _ = source_runtime(FILETYPE_FILE.as_ptr() as *mut c_char, RuntimeOpts::ALL);
            filetype_detect.set(Some(true));
            if plugin {
                let _ = source_runtime(FTPLUGIN_FILE.as_ptr() as *mut c_char, RuntimeOpts::ALL);
                filetype_plugin.set(Some(true));
            }
            if indent {
                let _ = source_runtime(INDENT_FILE.as_ptr() as *mut c_char, RuntimeOpts::ALL);
                filetype_indent.set(Some(true));
            }
        }
        if byte(arg) == 'd' as c_int {
            // `detect` also applies the result to the buffers already
            // open.
            let _ = do_doautocmd(
                c"filetypedetect BufRead".as_ptr() as *mut c_char,
                true,
                ptr::null_mut(),
            );
            do_modelines(OptionSetFlags::NONE);
        }
    } else if strcmp(arg, c"off".as_ptr()) == 0 {
        if plugin || indent {
            // Only what was named is turned off; detection stays on.
            if plugin {
                let _ = source_runtime(FTPLUGOF_FILE.as_ptr() as *mut c_char, RuntimeOpts::ALL);
                filetype_plugin.set(Some(false));
            }
            if indent {
                let _ = source_runtime(INDOFF_FILE.as_ptr() as *mut c_char, RuntimeOpts::ALL);
                filetype_indent.set(Some(false));
            }
        } else {
            let _ = source_runtime(FTOFF_FILE.as_ptr() as *mut c_char, RuntimeOpts::ALL);
            filetype_detect.set(Some(false));
        }
    } else {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str(arg) };
        semsg!("E475: Invalid argument: {arg}");
    }
}

/// `:filetype` with no argument.
///
/// Plugin and indent report `(on)` rather than `ON` when detection itself
/// is off, because nothing will ever ask them to run.
unsafe fn report_filetype_state() {
    let detecting = filetype_detect.get() == Some(true);
    let state = |on: bool| -> *const c_char {
        if !on {
            c"OFF".as_ptr()
        } else if detecting {
            c"ON".as_ptr()
        } else {
            c"(on)".as_ptr()
        }
    };
    // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
    let (arg0, arg1, arg2) = unsafe {
        (
            c_str(if detecting {
                c"ON".as_ptr()
            } else {
                c"OFF".as_ptr()
            }),
            c_str(state(filetype_plugin.get() == Some(true))),
            c_str(state(filetype_indent.get() == Some(true))),
        )
    };
    smsg!(0, "filetype detection:{arg0}  plugin:{arg1}  indent:{arg2}");
}

/// Turn the filetype plugin and indent scripts on, unless they were
/// explicitly turned off.
pub unsafe fn filetype_plugin_enable() {
    if filetype_plugin.get().is_none() {
        let _ = source_runtime(FTPLUGIN_FILE.as_ptr() as *mut c_char, RuntimeOpts::ALL);
        filetype_plugin.set(Some(true));
    }
    if filetype_indent.get().is_none() {
        let _ = source_runtime(INDENT_FILE.as_ptr() as *mut c_char, RuntimeOpts::ALL);
        filetype_indent.set(Some(true));
    }
}

/// The same for detection.
pub unsafe fn filetype_maybe_enable() {
    if filetype_detect.get().is_none() {
        let _ = source_runtime(FILETYPE_FILE.as_ptr() as *mut c_char, RuntimeOpts::ALL);
        filetype_detect.set(Some(true));
    }
}

/// `:setfiletype` — set 'filetype' unless something already did.
///
/// A `FALLBACK ` prefix means "only if nothing better is found later", and
/// is spelled by leaving `b_did_filetype` clear so that a later
/// `:setfiletype` still applies.
pub(crate) unsafe fn ex_setfiletype(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    if cur_buf().b_did_filetype {
        return;
    }
    let mut arg = eap.arg;
    if strncmp(arg, c"FALLBACK ".as_ptr(), 9) == 0 {
        arg = unsafe { arg.add(9) };
    }
    set_option_value_give_err(
        kOptFiletype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(arg),
            },
        },
        OptionSetFlags::LOCAL,
    );
    if arg != eap.arg {
        cur_buf().b_did_filetype = false;
    }
}

/// `:checkhealth` — hand the window modifiers and the argument to
/// `vim.health._check`.
pub(crate) unsafe fn ex_checkhealth(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    let mut env = env_buf();
    let mut err = Error::none();
    let mut items: [Object; 2] = unsafe { core::mem::zeroed() };
    let mut args = Array {
        size: 0,
        capacity: 2,
        items: &raw mut items as *mut Object,
    };

    // The modifiers are passed as text, because the health check opens
    // its own window and has to reproduce `:vertical`, `:tab` and the
    // rest itself.
    let mut mods: [c_char; 1024] = [0; 1024];
    let mut mods_len: size_t = 0;
    if cmdmod_tab() > 0 || cmdmod_split() != 0 {
        let mut multi_mods = false;
        mods_len = cmdmod.with(|cmod| unsafe {
            add_win_cmd_modifiers(&raw mut mods as *mut c_char, cmod, &raw mut multi_mods)
        });
        debug_assert!(mods_len < size_of::<[c_char; 1024]>());
    }

    items[0] = Object {
        type_0: kObjectTypeString,
        data: crate::types::object_data {
            string: String_0::from_raw_parts(&raw mut mods as *mut c_char, mods_len),
        },
    };
    items[1] = Object {
        type_0: kObjectTypeString,
        data: crate::types::object_data {
            string: cstr_as_string(eap.arg),
        },
    };
    args.size = 2;

    unsafe {
        nlua_exec(
            lua_chunk(c"vim.health._check(...)"),
            ptr::null(),
            args,
            kRetNilBool,
            ptr::null_mut(),
            &mut err,
        )
    };
    if !err.is_set() {
        return;
    }

    // The check failed to load at all, which almost always means the
    // runtime files are not where the editor thinks.
    let vimruntime = unsafe { os_getenv_into(c"VIMRUNTIME".as_ptr(), &mut env) };
    if vimruntime.is_null() {
        emsg(gettext(c"E5009: $VIMRUNTIME is empty or unset".as_ptr()));
    } else if !unsafe { strstr(p_rtp.get(), vimruntime) }.is_null() {
        // Upstream's, and it reads backwards: finding $VIMRUNTIME
        // *inside* 'runtimepath' is what makes it report $VIMRUNTIME as
        // the invalid one. Left alone — it is a message, not behaviour.
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let vimruntime = unsafe { c_str(vimruntime) };
        semsg!("E5009: Invalid $VIMRUNTIME: {vimruntime}");
    } else {
        emsg(gettext(c"E5009: Invalid 'runtimepath'".as_ptr()));
    }
    // SAFETY: the API error's own NUL-terminated message.
    let msg = unsafe { c_str(err.message_or_empty().as_ptr()) };
    semsg_multiline!(c"emsg", "{msg}");
    err.clear();
}

/// A `'static` Lua source string as the API's counted string.
fn lua_chunk(src: &'static CStr) -> String_0 {
    String_0::from_raw_parts(src.as_ptr() as *mut c_char, src.to_bytes().len() as size_t)
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// `cstr_as_string()` as checked code.
fn cstr_as_string(str: *const c_char) -> String_0 {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::api::private::helpers::cstr_as_string(str) }
}

/// `do_doautocmd()` as checked code.
fn do_doautocmd(
    arg_start: *mut ::core::ffi::c_char,
    do_msg: bool,
    did_something: *mut bool,
) -> Result<(), Failed> {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::autocmd::do_doautocmd(arg_start, do_msg, did_something) }
}

/// `emsg()` as checked code.
fn emsg(s: *const c_char) -> bool {
    // SAFETY: a NUL-terminated message.
    unsafe { crate::message::emsg_ptr(s) }
}

/// `gettext()` as checked code.
fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    // SAFETY: a NUL-terminated message; `gettext` answers one too.
    unsafe { crate::os::cshim::gettext_ptr(__msgid).as_ptr().cast_mut() }
}

/// `skipwhite()` as checked code.
fn skipwhite(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::charset::skipwhite(p) }
}

/// `source_runtime()` as checked code.
fn source_runtime(name: *mut c_char, flags: RuntimeOpts) -> Result<(), Failed> {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::runtime::source_runtime(name, flags) }
}

/// `strncmp()` as checked code.
fn strncmp(
    __s1: *const ::core::ffi::c_char,
    __s2: *const ::core::ffi::c_char,
    __n: size_t,
) -> ::core::ffi::c_int {
    // SAFETY: two NUL-terminated strings, and a length within both.
    unsafe { crate::os::cshim::strncmp(__s1, __s2, __n) }
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as c_int }
}

/// `strcmp()` as checked code.
fn strcmp(a: *const c_char, b: *const c_char) -> c_int {
    // SAFETY: two NUL-terminated strings.
    unsafe { ::libc::strcmp(a, b) }
}
