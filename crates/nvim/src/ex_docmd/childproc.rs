//! `:terminal` and `:lsp`, which both hand a buffer to a process.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::api::private::helpers::{api_clear_error, cstr_as_string};
use crate::ex_docmd::cmdline::do_cmdline_cmd;
use crate::ex_docmd::{cmdmod_split, cmdmod_tab, kRetNilBool};
use crate::highlight_group::HLF_E;
use crate::lua::executor::nlua_exec;
use crate::main::{cmdmod, e_shellempty, p_sh};
use crate::memory::xstrlcat;

use crate::message::{emsg, emsg_multiline};
use crate::os::cshim::{gettext, snprintf};

use crate::os::shell::{shell_build_argv, shell_free_argv};

use crate::types::{
    Array, Error, NUL, Object, String_0, exarg_T, kErrorTypeNone, kObjectTypeString, size_t,
};
use crate::usercmd::add_win_cmd_modifiers;
use crate::winlayer::Ea;

/// `:terminal` — spelled as a command line, not as a call.
///
/// The whole command is built as text and handed back to `do_cmdline_cmd`,
/// because `jobstart(…, {'term': v:true})` is the only entry point that
/// attaches a terminal to a buffer, and it is a vimscript function. The
/// argument therefore has to survive being read as a vimscript string
/// literal, which is what the `"` and `\` escaping is for.
pub(crate) unsafe fn ex_terminal(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    const CMD_LEN: usize = 1024;
    let mut ex_cmd: [c_char; CMD_LEN] = [0; CMD_LEN];
    let mut len: size_t = 0;

    // With a window modifier, the modifier plus `new` makes the window;
    // without one, `enew` reuses the current window.
    if cmdmod_tab() > 0 || cmdmod_split() != 0 {
        let mut multi_mods = false;
        len = cmdmod.with(|cmod| unsafe {
            add_win_cmd_modifiers(&raw mut ex_cmd as *mut c_char, cmod, &raw mut multi_mods)
        });
        debug_assert!(len < CMD_LEN);
        let written = unsafe {
            snprintf(
                (&raw mut ex_cmd as *mut c_char).add(len as usize),
                CMD_LEN - len as usize,
                c" new".as_ptr(),
            )
        };
        debug_assert!(written > 0);
        len += written as size_t;
    } else {
        let written = unsafe {
            snprintf(
                &raw mut ex_cmd as *mut c_char,
                CMD_LEN,
                c"enew%s".as_ptr(),
                if eap.forceit != 0 {
                    c"!".as_ptr()
                } else {
                    c"".as_ptr()
                },
            )
        };
        debug_assert!(written > 0);
        len += written as size_t;
    }
    debug_assert!(len < CMD_LEN);

    if unsafe { *eap.arg } as c_int != NUL {
        let name = vim_strsave_escaped(eap.arg, c"\"\\".as_ptr());
        unsafe {
            snprintf(
                (&raw mut ex_cmd as *mut c_char).add(len as usize),
                CMD_LEN - len as usize,
                c" | call jobstart(\"%s\",{'term':v:true})".as_ptr(),
                name,
            )
        };
        xfree(name as *mut c_void);
    } else {
        if unsafe { *p_sh.get() } as c_int == NUL {
            unsafe { emsg(gettext(&raw const e_shellempty as *const c_char)) };
            return;
        }
        // No argument: run 'shell', as a *list* so that the shell's own
        // arguments are not re-split.
        let mut shell_argv: [c_char; 512] = [0; 512];
        let argv = unsafe { shell_build_argv(ptr::null(), ptr::null()) };
        let mut p = argv;
        while !unsafe { *p }.is_null() {
            let escaped = unsafe { vim_strsave_escaped(*p, c"\"\\".as_ptr()) };
            let mut one: [c_char; 512] = [0; 512];
            unsafe {
                snprintf(
                    &raw mut one as *mut c_char,
                    512,
                    c",\"%s\"".as_ptr(),
                    escaped,
                )
            };
            xfree(escaped as *mut c_void);
            unsafe {
                xstrlcat(
                    &raw mut shell_argv as *mut c_char,
                    &raw mut one as *mut c_char,
                    512,
                )
            };
            p = unsafe { p.add(1) };
        }
        unsafe { shell_free_argv(argv) };
        // Every element was written with a leading comma, so the list
        // starts one byte in.
        unsafe {
            snprintf(
                (&raw mut ex_cmd as *mut c_char).add(len as usize),
                CMD_LEN - len as usize,
                c" | call jobstart([%s], {'term':v:true})".as_ptr(),
                (&raw mut shell_argv as *mut c_char).add(1),
            )
        };
    }
    unsafe { do_cmdline_cmd(&raw mut ex_cmd as *mut c_char) };
}

/// `:lsp` — a Lua entry point that takes the whole argument as one string.
pub(crate) unsafe fn ex_lsp(eap: *mut exarg_T) {
    let mut eap = unsafe { Ea::new(eap) };
    let mut err = Error {
        type_0: kErrorTypeNone,
        msg: ptr::null_mut(),
    };
    let mut items: [Object; 1] = [Object {
        type_0: kObjectTypeString,
        data: crate::types::object_data {
            string: unsafe { cstr_as_string(eap.arg) },
        },
    }];
    let args = Array {
        size: 1,
        capacity: 1,
        items: &raw mut items as *mut Object,
    };
    const CHUNK: &core::ffi::CStr = c"require'vim._core.ex_cmd'.ex_lsp(...)";
    unsafe {
        nlua_exec(
            String_0::from_raw_parts(
                CHUNK.as_ptr() as *mut c_char,
                CHUNK.to_bytes().len() as size_t,
            ),
            ptr::null(),
            args,
            kRetNilBool,
            ptr::null_mut(),
            &raw mut err,
        )
    };
    if err.type_0 as c_int != kErrorTypeNone as c_int {
        unsafe { emsg_multiline(err.msg, c"lua_error".as_ptr(), HLF_E, true) };
    }
    unsafe { api_clear_error(&raw mut err) };
}

/// `vim_strsave_escaped()` as checked code.
fn vim_strsave_escaped(string: *const c_char, esc_chars: *const c_char) -> *mut c_char {
    // SAFETY: two NUL-terminated strings.
    unsafe { crate::strings::vim_strsave_escaped(string, esc_chars) }
}

/// `xfree()` as checked code.
fn xfree(ptr: *mut c_void) {
    // SAFETY: `xmalloc`ed, or null.
    unsafe { crate::memory::xfree(ptr) }
}
