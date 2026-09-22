//! Autocommands, `:filetype` and `:setfiletype` — the commands that decide
//! what a buffer is. Plus `:checkhealth`, which is a Lua entry point.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::message::emsg;
use crate::message_fmt::{c_str, msg_bytes};
use crate::os::cshim::gettext;
use crate::runtime::source_runtime;
use crate::semsg;
use crate::semsg_multiline;
use crate::smsg;
use crate::strings::has_bytes;
use crate::types::CmdIdx;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use crate::autocmd::{check_nomodeline, do_augroup, do_autocmd};

use crate::buffer::do_modelines;

use crate::ex_docmd::state::cmdmod;
use crate::ex_docmd::{
    FILETYPE_FILE, FTOFF_FILE, FTPLUGIN_FILE, FTPLUGOF_FILE, INDENT_FILE, INDOFF_FILE,
    cmdmod_split, cmdmod_tab, ex_msg, filetype_detect, filetype_indent, filetype_plugin,
    kRetNilBool,
};
use crate::guard::secure;
use crate::lua::executor::nlua_exec;
use crate::message::e_curdir;
use crate::option::vars::p_rtp;

use crate::option::set_option_value_give_err;
use crate::options::kOptFiletype;

use crate::os::env::{env_buf, os_getenv_into};
use crate::runtime::RuntimeOpts;

use crate::types::{Array, ExArg, Failed, Object, OptVal, OptionSetFlags, String_0, size_t};
use crate::usercmd::add_win_cmd_modifiers;
use crate::winlayer::Buf;

/// `:autocmd` and `:augroup`.
///
/// Both are refused in a 'secure' context — a modeline or an untrusted
/// config — because an autocommand can run anything later.
pub(crate) fn ex_autocmd(excmd: &mut ExArg) {
    if secure.get() != 0 {
        // 2 means "an error was already reported for this".
        secure.set(2);
        excmd.errmsg = Some(ex_msg(e_curdir));
    } else if excmd.cmdidx == CmdIdx::autocmd {
        let (arg, forceit) = (excmd.arg_ptr(), c_int::from(excmd.forceit));
        unsafe { do_autocmd(excmd, arg, forceit) };
    } else {
        do_augroup(excmd.line.cstr_from(excmd.line.arg), excmd.forceit);
    }
}

/// `:doautocmd` — and the modelines that a `<nomodeline>` argument
/// suppresses.
pub(crate) fn ex_doautocmd(excmd: &mut ExArg) {
    let (call_do_modelines, skip) = check_nomodeline(excmd.line.arg());
    let arg = excmd.line.cstr_from(excmd.line.arg + skip);
    let mut did_aucmd = false;
    let _ = do_doautocmd(arg, false, &raw mut did_aucmd);
    if call_do_modelines && did_aucmd {
        do_modelines(OptionSetFlags::NONE);
    }
}

/// `:filetype [plugin] [indent] on|off|detect`.
pub(crate) fn ex_filetype(excmd: &mut ExArg) {
    if excmd.line.byte_at(excmd.line.arg) == 0 {
        report_filetype_state();
        return;
    }

    let mut at = excmd.line.arg;
    let mut plugin = false;
    let mut indent = false;
    loop {
        if excmd.line.starts_with(at, b"plugin") {
            plugin = true;
            at = excmd.line.skip_white(at + 6);
        } else if excmd.line.starts_with(at, b"indent") {
            indent = true;
            at = excmd.line.skip_white(at + 6);
        } else {
            break;
        }
    }

    let arg = excmd.line.rest_of(at);
    if arg == b"on" || arg == b"detect" {
        // `:filetype detect` only re-sources the scripts when detection
        // was off; `:filetype on` always does.
        if arg[0] == b'o' || filetype_detect.get() != Some(true) {
            let _ = source_runtime(FILETYPE_FILE, RuntimeOpts::ALL);
            filetype_detect.set(Some(true));
            if plugin {
                let _ = source_runtime(FTPLUGIN_FILE, RuntimeOpts::ALL);
                filetype_plugin.set(Some(true));
            }
            if indent {
                let _ = source_runtime(INDENT_FILE, RuntimeOpts::ALL);
                filetype_indent.set(Some(true));
            }
        }
        if arg[0] == b'd' {
            // `detect` also applies the result to the buffers already
            // open.
            let _ = do_doautocmd(c"filetypedetect BufRead", true, ptr::null_mut());
            do_modelines(OptionSetFlags::NONE);
        }
    } else if arg == b"off" {
        if plugin || indent {
            // Only what was named is turned off; detection stays on.
            if plugin {
                let _ = source_runtime(FTPLUGOF_FILE, RuntimeOpts::ALL);
                filetype_plugin.set(Some(false));
            }
            if indent {
                let _ = source_runtime(INDOFF_FILE, RuntimeOpts::ALL);
                filetype_indent.set(Some(false));
            }
        } else {
            let _ = source_runtime(FTOFF_FILE, RuntimeOpts::ALL);
            filetype_detect.set(Some(false));
        }
    } else {
        let arg = msg_bytes(arg);
        semsg!("E475: Invalid argument: {arg}");
    }
}

/// `:filetype` with no argument.
///
/// Plugin and indent report `(on)` rather than `ON` when detection itself
/// is off, because nothing will ever ask them to run.
fn report_filetype_state() {
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
pub fn filetype_plugin_enable() {
    if filetype_plugin.get().is_none() {
        let _ = source_runtime(FTPLUGIN_FILE, RuntimeOpts::ALL);
        filetype_plugin.set(Some(true));
    }
    if filetype_indent.get().is_none() {
        let _ = source_runtime(INDENT_FILE, RuntimeOpts::ALL);
        filetype_indent.set(Some(true));
    }
}

/// The same for detection.
pub fn filetype_maybe_enable() {
    if filetype_detect.get().is_none() {
        let _ = source_runtime(FILETYPE_FILE, RuntimeOpts::ALL);
        filetype_detect.set(Some(true));
    }
}

/// `:setfiletype` — set 'filetype' unless something already did.
///
/// A `FALLBACK ` prefix means "only if nothing better is found later", and
/// is spelled by leaving `b_did_filetype` clear so that a later
/// `:setfiletype` still applies.
pub(crate) fn ex_setfiletype(excmd: &mut ExArg) {
    if Buf::current().b_did_filetype {
        return;
    }
    let fallback = excmd.line.starts_with(excmd.line.arg, b"FALLBACK ");
    let at = excmd.line.arg + if fallback { 9 } else { 0 };
    set_option_value_give_err(
        kOptFiletype,
        OptVal::string(String_0::from_bytes(excmd.line.rest_of(at))),
        OptionSetFlags::LOCAL,
    );
    if fallback {
        Buf::current().b_did_filetype = false;
    }
}

/// `:checkhealth` — hand the window modifiers and the argument to
/// `vim.health._check`.
pub(crate) fn ex_checkhealth(excmd: &mut ExArg) {
    let mut env = env_buf();

    // The modifiers are passed as text, because the health check opens
    // its own window and has to reproduce `:vertical`, `:tab` and the
    // rest itself.
    let mut mods: [c_char; 1024] = [0; 1024];
    let mut mods_len: size_t = 0;
    if cmdmod_tab() > 0 || cmdmod_split() != 0 {
        let mut multi_mods = false;
        mods_len = cmdmod.with(|cmod| unsafe {
            add_win_cmd_modifiers((&raw mut mods).cast::<c_char>(), cmod, &raw mut multi_mods)
        });
        debug_assert!(mods_len < size_of::<[c_char; 1024]>());
    }

    // SAFETY: `add_win_cmd_modifiers` wrote `mods_len` bytes into `mods`.
    let mods = unsafe { core::slice::from_raw_parts(mods.as_ptr().cast::<u8>(), mods_len) };
    let argv = Array::from(vec![
        Object::string(String_0::from_bytes(mods)),
        Object::string(String_0::from_bytes(excmd.line.arg())),
    ]);

    let ran = unsafe {
        nlua_exec(
            &lua_chunk(c"vim.health._check(...)"),
            ptr::null(),
            argv,
            kRetNilBool,
            ptr::null_mut(),
        )
    };
    let Err(err) = ran else {
        return;
    };

    // The check failed to load at all, which almost always means the
    // runtime files are not where the editor thinks.
    let vimruntime = unsafe { os_getenv_into(c"VIMRUNTIME".as_ptr(), &mut env) };
    if vimruntime.is_null() {
        emsg(gettext(c"E5009: $VIMRUNTIME is empty or unset"));
    } else if p_rtp(|value| unsafe {
        has_bytes(
            cstr::at(value.as_ptr().cast_mut()),
            cstr::bytes_at(vimruntime),
        )
    }) {
        // Upstream's, and it reads backwards: finding $VIMRUNTIME
        // *inside* 'runtimepath' is what makes it report $VIMRUNTIME as
        // the invalid one. Left alone — it is a message, not behaviour.
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let vimruntime = unsafe { c_str(vimruntime) };
        semsg!("E5009: Invalid $VIMRUNTIME: {vimruntime}");
    } else {
        emsg(gettext(c"E5009: Invalid 'runtimepath'"));
    }
    // SAFETY: the refusal's own NUL-terminated message.
    let msg = unsafe { c_str(err.message_or_empty().as_ptr()) };
    semsg_multiline!(c"emsg", "{msg}");
}

/// A `'static` Lua source string as the API's counted string.
fn lua_chunk(src: &'static CStr) -> String_0 {
    String_0::from_cstr(src)
}

/// `do_doautocmd()` as checked code.
fn do_doautocmd(arg_start: &CStr, do_msg: bool, did_something: *mut bool) -> Result<(), Failed> {
    // SAFETY: `did_something` is the caller's own slot, or null.
    unsafe { crate::autocmd::do_doautocmd(arg_start, do_msg, did_something) }
}
