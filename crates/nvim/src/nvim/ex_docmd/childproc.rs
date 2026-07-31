//! `:terminal` and `:lsp`, which both hand a buffer to a process.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use crate::src::nvim::api::private::helpers::{api_clear_error, cstr_as_string};
use crate::src::nvim::ex_docmd::cmdline::do_cmdline_cmd;
use crate::src::nvim::ex_docmd::{HLF_E, NUL, kRetNilBool};
use crate::src::nvim::lua::executor::nlua_exec;
use crate::src::nvim::main::{cmdmod, e_shellempty, p_sh};
use crate::src::nvim::memory::{xfree, xstrlcat};
use crate::src::nvim::message::{emsg, emsg_multiline};
use crate::src::nvim::os::libc::{gettext, snprintf};
use crate::src::nvim::os::shell::{shell_build_argv, shell_free_argv};
use crate::src::nvim::strings::vim_strsave_escaped;
use crate::src::nvim::types::api::kErrorTypeNone;
use crate::src::nvim::types::{Array, Error, Object, String_0, exarg_T, kObjectTypeString, size_t};
use crate::src::nvim::usercmd::add_win_cmd_modifiers;

/// `:terminal` — spelled as a command line, not as a call.
///
/// The whole command is built as text and handed back to `do_cmdline_cmd`,
/// because `jobstart(…, {'term': v:true})` is the only entry point that
/// attaches a terminal to a buffer, and it is a vimscript function. The
/// argument therefore has to survive being read as a vimscript string
/// literal, which is what the `"` and `\` escaping is for.
pub(crate) unsafe fn ex_terminal(eap: *mut exarg_T) {
    unsafe {
        const CMD_LEN: usize = 1024;
        let mut ex_cmd: [c_char; CMD_LEN] = [0; CMD_LEN];
        let mut len: size_t = 0;

        // With a window modifier, the modifier plus `new` makes the window;
        // without one, `enew` reuses the current window.
        if (*cmdmod.ptr()).cmod_tab > 0 || (*cmdmod.ptr()).cmod_split != 0 {
            let mut multi_mods = false;
            len = add_win_cmd_modifiers(
                &raw mut ex_cmd as *mut c_char,
                cmdmod.ptr(),
                &raw mut multi_mods,
            );
            debug_assert!(len < CMD_LEN);
            let written = snprintf(
                (&raw mut ex_cmd as *mut c_char).add(len as usize),
                CMD_LEN - len as usize,
                c" new".as_ptr(),
            );
            debug_assert!(written > 0);
            len += written as size_t;
        } else {
            let written = snprintf(
                &raw mut ex_cmd as *mut c_char,
                CMD_LEN,
                c"enew%s".as_ptr(),
                if (*eap).forceit != 0 {
                    c"!".as_ptr()
                } else {
                    c"".as_ptr()
                },
            );
            debug_assert!(written > 0);
            len += written as size_t;
        }
        debug_assert!(len < CMD_LEN);

        if *(*eap).arg as c_int != NUL {
            let name = vim_strsave_escaped((*eap).arg, c"\"\\".as_ptr());
            snprintf(
                (&raw mut ex_cmd as *mut c_char).add(len as usize),
                CMD_LEN - len as usize,
                c" | call jobstart(\"%s\",{'term':v:true})".as_ptr(),
                name,
            );
            xfree(name as *mut core::ffi::c_void);
        } else {
            if *p_sh.get() as c_int == NUL {
                emsg(gettext(&raw const e_shellempty as *const c_char));
                return;
            }
            // No argument: run 'shell', as a *list* so that the shell's own
            // arguments are not re-split.
            let mut shell_argv: [c_char; 512] = [0; 512];
            let argv = shell_build_argv(ptr::null(), ptr::null());
            let mut p = argv;
            while !(*p).is_null() {
                let escaped = vim_strsave_escaped(*p, c"\"\\".as_ptr());
                let mut one: [c_char; 512] = [0; 512];
                snprintf(
                    &raw mut one as *mut c_char,
                    512,
                    c",\"%s\"".as_ptr(),
                    escaped,
                );
                xfree(escaped as *mut core::ffi::c_void);
                xstrlcat(
                    &raw mut shell_argv as *mut c_char,
                    &raw mut one as *mut c_char,
                    512,
                );
                p = p.add(1);
            }
            shell_free_argv(argv);
            // Every element was written with a leading comma, so the list
            // starts one byte in.
            snprintf(
                (&raw mut ex_cmd as *mut c_char).add(len as usize),
                CMD_LEN - len as usize,
                c" | call jobstart([%s], {'term':v:true})".as_ptr(),
                (&raw mut shell_argv as *mut c_char).add(1),
            );
        }
        do_cmdline_cmd(&raw mut ex_cmd as *mut c_char);
    }
}

/// `:lsp` — a Lua entry point that takes the whole argument as one string.
pub(crate) unsafe fn ex_lsp(eap: *mut exarg_T) {
    unsafe {
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        let mut items: [Object; 1] = [Object {
            type_0: kObjectTypeString,
            data: crate::src::nvim::types::object_data {
                string: cstr_as_string((*eap).arg),
            },
        }];
        let args = Array {
            size: 1,
            capacity: 1,
            items: &raw mut items as *mut Object,
        };
        const CHUNK: &core::ffi::CStr = c"require'vim._core.ex_cmd'.ex_lsp(...)";
        nlua_exec(
            String_0 {
                data: CHUNK.as_ptr() as *mut c_char,
                size: CHUNK.to_bytes().len() as size_t,
            },
            ptr::null(),
            args,
            kRetNilBool,
            ptr::null_mut(),
            &raw mut err,
        );
        if err.type_0 as c_int != kErrorTypeNone as c_int {
            emsg_multiline(err.msg, c"lua_error".as_ptr(), HLF_E as c_int, true);
        }
        api_clear_error(&raw mut err);
    }
}
