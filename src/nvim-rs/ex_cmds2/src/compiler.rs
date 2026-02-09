//! ex_compiler implementation
//!
//! Port of `ex_compiler`.

use std::ffi::{c_char, c_int};

const FAIL: c_int = 0;
const NUL: u8 = 0;
const DIP_ALL: c_int = 0x01;

type ExArgHandle = crate::script_host::ExArgHandle;

extern "C" {
    fn nvim_ex2_eap_get_arg(eap: *mut ExArgHandle) -> *mut c_char;
    fn nvim_ex2_eap_get_forceit(eap: *mut ExArgHandle) -> c_int;
    fn nvim_ex2_gettext(s: *const c_char) -> *const c_char;

    // compiler-specific accessors
    fn nvim_ex2_do_cmdline_cmd(cmd: *const c_char);
    fn nvim_ex2_get_var_value(name: *const c_char) -> *mut c_char;
    fn nvim_ex2_set_internal_string_var(name: *const c_char, val: *const c_char);
    fn nvim_ex2_do_unlet(name: *const c_char, name_len: usize, forceit: bool);
    fn nvim_ex2_source_runtime_vim_lua(name: *const c_char, flags: c_int) -> c_int;
    fn nvim_ex2_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_ex2_xfree(p: *mut c_char);
    fn nvim_ex2_xmalloc(size: usize) -> *mut c_char;
    fn nvim_ex2_semsg(fmt: *const c_char, arg: *const c_char) -> bool;
    fn nvim_ex2_snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, arg: *const c_char);
    fn nvim_ex2_strlen(s: *const c_char) -> usize;
}

/// Port of `ex_compiler`
#[no_mangle]
pub unsafe extern "C" fn rs_ex_compiler(eap: *mut ExArgHandle) {
    let arg = unsafe { nvim_ex2_eap_get_arg(eap) };

    // If no argument, list all compiler scripts
    if unsafe { *arg.cast::<u8>() } == NUL {
        unsafe {
            nvim_ex2_do_cmdline_cmd(b"echo globpath(&rtp, 'compiler/*.vim')\0".as_ptr().cast());
            nvim_ex2_do_cmdline_cmd(b"echo globpath(&rtp, 'compiler/*.lua')\0".as_ptr().cast());
        }
        return;
    }

    let arg_len = unsafe { nvim_ex2_strlen(arg) };
    let bufsize = arg_len + 14;
    let buf = unsafe { nvim_ex2_xmalloc(bufsize) };

    let forceit = unsafe { nvim_ex2_eap_get_forceit(eap) } != 0;
    let mut old_cur_comp: *mut c_char = std::ptr::null_mut();

    if forceit {
        // ":compiler! {name}" sets global options
        unsafe {
            nvim_ex2_do_cmdline_cmd(
                b"command -nargs=* -keepscript CompilerSet set <args>\0"
                    .as_ptr()
                    .cast(),
            );
        }
    } else {
        // ":compiler {name}" sets local options.
        old_cur_comp = unsafe { nvim_ex2_get_var_value(b"g:current_compiler\0".as_ptr().cast()) };
        if !old_cur_comp.is_null() {
            old_cur_comp = unsafe { nvim_ex2_xstrdup(old_cur_comp) };
        }
        unsafe {
            nvim_ex2_do_cmdline_cmd(
                b"command -nargs=* -keepscript CompilerSet setlocal <args>\0"
                    .as_ptr()
                    .cast(),
            );
        }
    }

    unsafe {
        nvim_ex2_do_unlet(
            b"g:current_compiler\0".as_ptr().cast(),
            b"g:current_compiler".len(),
            true,
        );
        nvim_ex2_do_unlet(
            b"b:current_compiler\0".as_ptr().cast(),
            b"b:current_compiler".len(),
            true,
        );
    }

    unsafe {
        nvim_ex2_snprintf(buf, bufsize, b"compiler/%s.*\0".as_ptr().cast(), arg);
    }
    if unsafe { nvim_ex2_source_runtime_vim_lua(buf, DIP_ALL) } == FAIL {
        static E_COMPILER: &[u8] = b"E666: Compiler not supported: %s\0";
        unsafe {
            nvim_ex2_semsg(nvim_ex2_gettext(E_COMPILER.as_ptr().cast()), arg);
        }
    }
    unsafe { nvim_ex2_xfree(buf) };

    unsafe {
        nvim_ex2_do_cmdline_cmd(b":delcommand CompilerSet\0".as_ptr().cast());
    }

    // Set "b:current_compiler" from "current_compiler".
    let p = unsafe { nvim_ex2_get_var_value(b"g:current_compiler\0".as_ptr().cast()) };
    if !p.is_null() {
        unsafe { nvim_ex2_set_internal_string_var(b"b:current_compiler\0".as_ptr().cast(), p) };
    }

    // Restore "current_compiler" for ":compiler {name}".
    if !forceit {
        if old_cur_comp.is_null() {
            unsafe {
                nvim_ex2_do_unlet(
                    b"g:current_compiler\0".as_ptr().cast(),
                    b"g:current_compiler".len(),
                    true,
                );
            }
        } else {
            unsafe {
                nvim_ex2_set_internal_string_var(
                    b"g:current_compiler\0".as_ptr().cast(),
                    old_cur_comp,
                );
                nvim_ex2_xfree(old_cur_comp);
            }
        }
    }
}
