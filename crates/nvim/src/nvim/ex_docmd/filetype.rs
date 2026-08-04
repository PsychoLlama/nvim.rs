//! Autocommands, `:filetype` and `:setfiletype` — the commands that decide
//! what a buffer is. Plus `:checkhealth`, which is a Lua entry point.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use crate::src::nvim::api::private::helpers::{api_clear_error, cstr_as_string};
use crate::src::nvim::autocmd::{check_nomodeline, do_augroup, do_autocmd, do_doautocmd};
use crate::src::nvim::buffer::do_modelines;
use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::ex_docmd::{
    CMD_autocmd, FILETYPE_FILE, FTOFF_FILE, FTPLUGIN_FILE, FTPLUGOF_FILE, INDENT_FILE, INDOFF_FILE,
    NUL, OPT_LOCAL, filetype_detect, filetype_indent, filetype_plugin, kOptValTypeString,
    kRetNilBool,
};
use crate::src::nvim::lua::executor::nlua_exec;
use crate::src::nvim::main::{cmdmod, curbuf, e_curdir, e_invarg2, p_rtp, secure};
use crate::src::nvim::message::{emsg, semsg, semsg_multiline, smsg};
use crate::src::nvim::option::set_option_value_give_err;
use crate::src::nvim::options::kOptFiletype;
use crate::src::nvim::os::env::os_getenv_noalloc;
use crate::src::nvim::os::libc::{gettext, strcmp, strncmp, strstr};
use crate::src::nvim::runtime::{DIP_ALL, source_runtime};
use crate::src::nvim::types::api::kErrorTypeNone;
use crate::src::nvim::types::{
    Array, Error, Object, OptVal, OptValData, String_0, exarg_T, kFalse, kNone, kObjectTypeString,
    kTrue, size_t,
};
use crate::src::nvim::usercmd::add_win_cmd_modifiers;

/// `:autocmd` and `:augroup`.
///
/// Both are refused in a 'secure' context — a modeline or an untrusted
/// config — because an autocommand can run anything later.
pub(crate) unsafe fn ex_autocmd(eap: *mut exarg_T) {
    unsafe {
        if secure.get() != 0 {
            // 2 means "an error was already reported for this".
            secure.set(2);
            (*eap).errmsg = gettext(&raw const e_curdir as *const c_char);
        } else if (*eap).cmdidx as c_int == CMD_autocmd as c_int {
            do_autocmd(eap, (*eap).arg, (*eap).forceit);
        } else {
            do_augroup((*eap).arg, (*eap).forceit != 0);
        }
    }
}

/// `:doautocmd` — and the modelines that a `<nomodeline>` argument
/// suppresses.
pub(crate) unsafe fn ex_doautocmd(eap: *mut exarg_T) {
    unsafe {
        let mut arg = (*eap).arg;
        let call_do_modelines = check_nomodeline(&raw mut arg);
        let mut did_aucmd = false;
        do_doautocmd(arg, false, &raw mut did_aucmd);
        if call_do_modelines && did_aucmd {
            do_modelines(0);
        }
    }
}

/// `:filetype [plugin] [indent] on|off|detect`.
pub(crate) unsafe fn ex_filetype(eap: *mut exarg_T) {
    unsafe {
        if *(*eap).arg as c_int == NUL {
            report_filetype_state();
            return;
        }

        let mut arg = (*eap).arg;
        let mut plugin = false;
        let mut indent = false;
        loop {
            if strncmp(arg, c"plugin".as_ptr(), 6) == 0 {
                plugin = true;
                arg = skipwhite(arg.add(6));
            } else if strncmp(arg, c"indent".as_ptr(), 6) == 0 {
                indent = true;
                arg = skipwhite(arg.add(6));
            } else {
                break;
            }
        }

        if strcmp(arg, c"on".as_ptr()) == 0 || strcmp(arg, c"detect".as_ptr()) == 0 {
            // `:filetype detect` only re-sources the scripts when detection
            // was off; `:filetype on` always does.
            if *arg as c_int == 'o' as c_int || filetype_detect.get() as c_int != kTrue as c_int {
                source_runtime(FILETYPE_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                filetype_detect.set(kTrue);
                if plugin {
                    source_runtime(FTPLUGIN_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                    filetype_plugin.set(kTrue);
                }
                if indent {
                    source_runtime(INDENT_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                    filetype_indent.set(kTrue);
                }
            }
            if *arg as c_int == 'd' as c_int {
                // `detect` also applies the result to the buffers already
                // open.
                do_doautocmd(
                    c"filetypedetect BufRead".as_ptr() as *mut c_char,
                    true,
                    ptr::null_mut(),
                );
                do_modelines(0);
            }
        } else if strcmp(arg, c"off".as_ptr()) == 0 {
            if plugin || indent {
                // Only what was named is turned off; detection stays on.
                if plugin {
                    source_runtime(FTPLUGOF_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                    filetype_plugin.set(kFalse);
                }
                if indent {
                    source_runtime(INDOFF_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                    filetype_indent.set(kFalse);
                }
            } else {
                source_runtime(FTOFF_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
                filetype_detect.set(kFalse);
            }
        } else {
            semsg(gettext(&raw const e_invarg2 as *const c_char), arg);
        }
    }
}

/// `:filetype` with no argument.
///
/// Plugin and indent report `(on)` rather than `ON` when detection itself
/// is off, because nothing will ever ask them to run.
unsafe fn report_filetype_state() {
    unsafe {
        let detecting = filetype_detect.get() as c_int == kTrue as c_int;
        let state = |on: bool| -> *const c_char {
            if !on {
                c"OFF".as_ptr()
            } else if detecting {
                c"ON".as_ptr()
            } else {
                c"(on)".as_ptr()
            }
        };
        smsg(
            0,
            c"filetype detection:%s  plugin:%s  indent:%s".as_ptr(),
            if detecting {
                c"ON".as_ptr()
            } else {
                c"OFF".as_ptr()
            },
            state(filetype_plugin.get() as c_int == kTrue as c_int),
            state(filetype_indent.get() as c_int == kTrue as c_int),
        );
    }
}

/// Turn the filetype plugin and indent scripts on, unless they were
/// explicitly turned off.
pub unsafe fn filetype_plugin_enable() {
    unsafe {
        if filetype_plugin.get() as c_int == kNone as c_int {
            source_runtime(FTPLUGIN_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
            filetype_plugin.set(kTrue);
        }
        if filetype_indent.get() as c_int == kNone as c_int {
            source_runtime(INDENT_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
            filetype_indent.set(kTrue);
        }
    }
}

/// The same for detection.
pub unsafe fn filetype_maybe_enable() {
    unsafe {
        if filetype_detect.get() as c_int == kNone as c_int {
            source_runtime(FILETYPE_FILE.as_ptr() as *mut c_char, DIP_ALL as c_int);
            filetype_detect.set(kTrue);
        }
    }
}

/// `:setfiletype` — set 'filetype' unless something already did.
///
/// A `FALLBACK ` prefix means "only if nothing better is found later", and
/// is spelled by leaving `b_did_filetype` clear so that a later
/// `:setfiletype` still applies.
pub(crate) unsafe fn ex_setfiletype(eap: *mut exarg_T) {
    unsafe {
        if (*curbuf.get()).b_did_filetype {
            return;
        }
        let mut arg = (*eap).arg;
        if strncmp(arg, c"FALLBACK ".as_ptr(), 9) == 0 {
            arg = arg.add(9);
        }
        set_option_value_give_err(
            kOptFiletype,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(arg),
                },
            },
            OPT_LOCAL as c_int,
        );
        if arg != (*eap).arg {
            (*curbuf.get()).b_did_filetype = false;
        }
    }
}

/// `:checkhealth` — hand the window modifiers and the argument to
/// `vim.health._check`.
pub(crate) unsafe fn ex_checkhealth(eap: *mut exarg_T) {
    unsafe {
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        let mut items: [Object; 2] = core::mem::zeroed();
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
        if (*cmdmod.ptr()).cmod_tab > 0 || (*cmdmod.ptr()).cmod_split != 0 {
            let mut multi_mods = false;
            mods_len = add_win_cmd_modifiers(
                &raw mut mods as *mut c_char,
                cmdmod.ptr(),
                &raw mut multi_mods,
            );
            debug_assert!(mods_len < size_of::<[c_char; 1024]>());
        }

        items[0] = Object {
            type_0: kObjectTypeString,
            data: crate::src::nvim::types::object_data {
                string: String_0 {
                    data: &raw mut mods as *mut c_char,
                    size: mods_len,
                },
            },
        };
        items[1] = Object {
            type_0: kObjectTypeString,
            data: crate::src::nvim::types::object_data {
                string: cstr_as_string((*eap).arg),
            },
        };
        args.size = 2;

        nlua_exec(
            lua_chunk(c"vim.health._check(...)"),
            ptr::null(),
            args,
            kRetNilBool,
            ptr::null_mut(),
            &raw mut err,
        );
        if err.type_0 as c_int == kErrorTypeNone as c_int {
            return;
        }

        // The check failed to load at all, which almost always means the
        // runtime files are not where the editor thinks.
        let vimruntime = os_getenv_noalloc(c"VIMRUNTIME".as_ptr());
        if vimruntime.is_null() {
            emsg(gettext(c"E5009: $VIMRUNTIME is empty or unset".as_ptr()));
        } else if !strstr(p_rtp.get(), vimruntime).is_null() {
            // Upstream's, and it reads backwards: finding $VIMRUNTIME
            // *inside* 'runtimepath' is what makes it report $VIMRUNTIME as
            // the invalid one. Left alone — it is a message, not behaviour.
            semsg(
                gettext(c"E5009: Invalid $VIMRUNTIME: %s".as_ptr()),
                vimruntime,
            );
        } else {
            emsg(gettext(c"E5009: Invalid 'runtimepath'".as_ptr()));
        }
        semsg_multiline(c"emsg".as_ptr(), err.msg);
        api_clear_error(&raw mut err);
    }
}

/// A `'static` Lua source string as the API's counted string.
fn lua_chunk(src: &'static CStr) -> String_0 {
    String_0 {
        data: src.as_ptr() as *mut c_char,
        size: src.to_bytes().len() as size_t,
    }
}
