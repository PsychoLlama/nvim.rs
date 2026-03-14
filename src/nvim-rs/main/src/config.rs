//! Config sourcing and environment functions
//!
//! Implements `rs_execute_env`, `rs_source_startup_scripts`,
//! `rs_do_system_initialization`, `rs_do_user_initialization`,
//! `rs_do_exrc_initialization` replacing static C functions in main.c.

use crate::setup::MparmT;
use std::ffi::{c_char, c_int};

// SID_* constants (from globals.h)
const SID_ENV: c_int = -4; // for sourcing environment variable

// ETYPE_ENV value (from runtime_defs.h EtypeEnum, 0-indexed)
// ETYPE_TOP=0, SCRIPT=1, UFUNC=2, AUCMD=3, MODELINE=4, EXCEPT=5, ARGS=6, ENV=7
const ETYPE_ENV: c_int = 7;

// DOSO_* constants (from runtime.h)
const DOSO_NONE: c_int = 0;
const DOSO_VIMRC: c_int = 1;

// Return value constants (from vim_defs.h)
const OK: c_int = 1;
const FAIL: c_int = 0;

// kEqualFiles value (from path.h FileComparison enum, kEqualFiles = 1)
const K_EQUAL_FILES: c_int = 1;

// SYS_VIMRC_FILE default (matches C default when not defined by build system)
const SYS_VIMRC_FILE: &[u8] = b"$VIM/sysinit.vim\0";

// XDG config dirs enum value (from os/stdpaths_defs.h)
// kXDGConfigDirs = 4 (counting from 0: kXDGDataHome=0, kXDGConfigHome=1,
//                      kXDGStateHome=2, kXDGRuntimeDir=3, kXDGDataDirs=4 -- wait, check)
const K_XDG_CONFIG_DIRS: c_int = 5; // to be validated

// sctx_T fields (for save/restore of current_sctx)
/// Mirror of `sctx_T` from eval/typval_defs.h
#[repr(C)]
#[derive(Clone, Copy)]
struct SctxT {
    sc_sid: c_int, // scid_T = int
    sc_seq: c_int,
    sc_lnum: i32, // linenr_T = int32_t
    sc_chan: u64,
}

// Extern C declarations
unsafe extern "C" {
    fn os_getenv(name: *const c_char) -> *mut c_char;
    fn xfree(p: *mut std::ffi::c_void);
    fn xmalloc(size: usize) -> *mut std::ffi::c_void;
    fn estack_push(etype: c_int, name: *const c_char, lnum: i32);
    fn estack_pop();
    fn do_cmdline_cmd(cmd: *const c_char) -> c_int;
    fn do_source(
        fname: *const c_char,
        check_other: bool,
        flags: c_int,
        ret_sid: *mut c_int,
    ) -> c_int;
    fn do_source_str(cmd: *const c_char, name: *const c_char) -> c_int;
    fn stdpaths_get_xdg_var(idx: c_int) -> *mut c_char;
    fn stdpaths_user_conf_subpath(fname: *const c_char) -> *mut c_char;
    fn os_path_exists(fname: *const c_char) -> bool;
    fn path_full_compare(
        s1: *const c_char,
        s2: *const c_char,
        checkname: bool,
        expandenv: bool,
    ) -> c_int;
    fn nlua_read_secure(path: *const c_char) -> *mut c_char;
    fn nlua_exec(
        str_: NvimString,
        name: *const c_char,
        args: NvimArray,
        ret_type: c_int,
        ret_value: *mut std::ffi::c_void,
        err: *mut NvimError,
    ) -> NvimObject;
    fn vim_env_iter(
        delim: u8,
        val: *const c_char,
        iter: *const std::ffi::c_void,
        dir: *mut *const c_char,
        dir_len: *mut usize,
    ) -> *const std::ffi::c_void;
    fn semsg(msg: *const c_char, ...);
    fn semsg_multiline(category: *const c_char, msg: *const c_char);
    fn api_clear_error(err: *mut NvimError);

    static mut current_sctx: SctxT;
    static mut silent_mode: bool;
    static mut p_exrc: bool;
}

/// Opaque types for Nvim API objects passed by value (pointers only on our side)
#[repr(C)]
struct NvimString {
    data: *mut c_char,
    size: usize,
}

#[repr(C)]
struct NvimArray {
    items: *mut std::ffi::c_void,
    size: usize,
    capacity: usize,
}

#[repr(C)]
struct NvimObject {
    type_: c_int,
    data: [u8; 16], // Largest variant union member
}

#[repr(C)]
struct NvimError {
    type_: c_int,
    msg: *mut c_char,
}

impl NvimError {
    const fn init() -> Self {
        NvimError {
            type_: 0, // kErrorTypeNone = 0
            msg: std::ptr::null_mut(),
        }
    }

    fn is_set(&self) -> bool {
        self.type_ != 0
    }
}

impl NvimArray {
    const EMPTY: Self = NvimArray {
        items: std::ptr::null_mut(),
        size: 0,
        capacity: 0,
    };
}

// kRetNilBool return type (from api/private/defs.h)
const K_RET_NIL_BOOL: c_int = 2;

/// Get and execute an environment variable as Ex commands.
///
/// Returns OK if the variable was found and executed, FAIL otherwise.
///
/// # Safety
/// `env` must be a valid C string.
#[export_name = "execute_env"]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_execute_env(env: *mut c_char) -> c_int {
    let initstr = os_getenv(env);
    if initstr.is_null() {
        return FAIL;
    }

    estack_push(ETYPE_ENV, env, 0);
    let save_sctx = current_sctx;
    current_sctx.sc_sid = SID_ENV;
    current_sctx.sc_seq = 0;
    current_sctx.sc_lnum = 0;

    do_cmdline_cmd(initstr);

    estack_pop();
    current_sctx = save_sctx;

    xfree(initstr as *mut std::ffi::c_void);
    OK
}

/// Source system-wide vimrc from $XDG_CONFIG_DIRS/nvim/sysinit.vim or $VIM.
///
/// # Safety
/// Calls C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_do_system_initialization() {
    let config_dirs = stdpaths_get_xdg_var(K_XDG_CONFIG_DIRS);
    if !config_dirs.is_null() {
        // Construct "nvim/sysinit.vim" with PATHSEP
        let suffix = b"nvim\x2fsysinit.vim\0"; // "nvim/sysinit.vim\0", '/' = 0x2f
        let suffix_len = suffix.len() - 1; // exclude nul

        let mut iter: *const std::ffi::c_void = std::ptr::null();
        loop {
            let mut dir: *const c_char = std::ptr::null();
            let mut dir_len: usize = 0;
            iter = vim_env_iter(b':', config_dirs, iter, &mut dir, &mut dir_len);
            if dir.is_null() || dir_len == 0 {
                break;
            }

            // Build: dir + "/" + "nvim/sysinit.vim"
            let needs_sep = dir_len > 0 && *dir.add(dir_len - 1) as u8 != b'/';
            let total = dir_len + (if needs_sep { 1 } else { 0 }) + suffix_len + 1;
            let vimrc = xmalloc(total) as *mut c_char;
            std::ptr::copy_nonoverlapping(dir, vimrc, dir_len);
            let mut pos = dir_len;
            if needs_sep {
                *vimrc.add(pos) = b'/' as i8;
                pos += 1;
            }
            std::ptr::copy_nonoverlapping(
                suffix.as_ptr() as *const c_char,
                vimrc.add(pos),
                suffix_len + 1,
            );

            let result = do_source(vimrc, false, DOSO_NONE, std::ptr::null_mut());
            xfree(vimrc as *mut std::ffi::c_void);
            if result != FAIL {
                xfree(config_dirs as *mut std::ffi::c_void);
                return;
            }

            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut std::ffi::c_void);
    }

    // Fallback: system-wide SYS_VIMRC_FILE
    do_source(
        SYS_VIMRC_FILE.as_ptr() as *const c_char,
        false,
        DOSO_NONE,
        std::ptr::null_mut(),
    );
}

/// Source user vimrc or execute VIMINIT/EXINIT environment variable.
///
/// Returns true if .exrc sourcing should be attempted.
///
/// # Safety
/// Calls C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_do_user_initialization() -> bool {
    let mut do_exrc = p_exrc;

    if rs_execute_env(c"VIMINIT".as_ptr() as *mut c_char) == OK {
        do_exrc = p_exrc;
        return do_exrc;
    }

    let init_lua_path = stdpaths_user_conf_subpath(c"init.lua".as_ptr());
    let user_vimrc = stdpaths_user_conf_subpath(c"init.vim".as_ptr());

    // Try init.lua
    if os_path_exists(init_lua_path)
        && do_source(init_lua_path, true, DOSO_VIMRC, std::ptr::null_mut()) != 0
    {
        if os_path_exists(user_vimrc) {
            semsg(
                c"E5422: Conflicting configs: \"%s\" \"%s\"".as_ptr(),
                init_lua_path,
                user_vimrc,
            );
        }
        xfree(user_vimrc as *mut std::ffi::c_void);
        xfree(init_lua_path as *mut std::ffi::c_void);
        do_exrc = p_exrc;
        return do_exrc;
    }
    xfree(init_lua_path as *mut std::ffi::c_void);

    // Try init.vim
    if do_source(user_vimrc, true, DOSO_VIMRC, std::ptr::null_mut()) != FAIL {
        do_exrc = p_exrc;
        if do_exrc {
            do_exrc =
                path_full_compare(c".nvimrc".as_ptr(), user_vimrc, false, true) != K_EQUAL_FILES;
        }
        xfree(user_vimrc as *mut std::ffi::c_void);
        return do_exrc;
    }
    xfree(user_vimrc as *mut std::ffi::c_void);

    // Try XDG_CONFIG_DIRS init.vim files
    let config_dirs = stdpaths_get_xdg_var(K_XDG_CONFIG_DIRS);
    if !config_dirs.is_null() {
        let suffix = b"nvim\x2finit.vim\0"; // "nvim/init.vim\0"
        let suffix_len = suffix.len() - 1;

        let mut iter: *const std::ffi::c_void = std::ptr::null();
        loop {
            let mut dir: *const c_char = std::ptr::null();
            let mut dir_len: usize = 0;
            iter = vim_env_iter(b':', config_dirs, iter, &mut dir, &mut dir_len);
            if dir.is_null() || dir_len == 0 {
                break;
            }

            // Build: dir + "/" + "nvim/init.vim"
            let total = dir_len + 1 + suffix_len + 1;
            let vimrc = xmalloc(total) as *mut c_char;
            std::ptr::copy_nonoverlapping(dir, vimrc, dir_len);
            *vimrc.add(dir_len) = b'/' as i8;
            std::ptr::copy_nonoverlapping(
                suffix.as_ptr() as *const c_char,
                vimrc.add(dir_len + 1),
                suffix_len + 1,
            );

            if do_source(vimrc, true, DOSO_VIMRC, std::ptr::null_mut()) != FAIL {
                do_exrc = p_exrc;
                if do_exrc {
                    do_exrc =
                        path_full_compare(c".nvimrc".as_ptr(), vimrc, false, true) != K_EQUAL_FILES;
                }
                xfree(vimrc as *mut std::ffi::c_void);
                xfree(config_dirs as *mut std::ffi::c_void);
                return do_exrc;
            }
            xfree(vimrc as *mut std::ffi::c_void);

            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut std::ffi::c_void);
    }

    if rs_execute_env(c"EXINIT".as_ptr() as *mut c_char) == OK {
        do_exrc = p_exrc;
        return do_exrc;
    }

    do_exrc
}

/// Source .nvim.lua, .nvimrc, or .exrc in current directory (when 'exrc' is set).
///
/// # Safety
/// Calls C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_do_exrc_initialization() {
    let vimrc_lua = c".nvim.lua".as_ptr();
    let vimrc = c".nvimrc".as_ptr();
    let exrc = c".exrc".as_ptr();

    if os_path_exists(vimrc_lua) {
        let str_ = nlua_read_secure(vimrc_lua);
        if !str_.is_null() {
            let mut err = NvimError::init();
            let name = c"@.nvim.lua".as_ptr();
            nlua_exec(
                NvimString {
                    data: str_,
                    size: strlen_c(str_),
                },
                name,
                NvimArray::EMPTY,
                K_RET_NIL_BOOL,
                std::ptr::null_mut(),
                &mut err,
            );
            xfree(str_ as *mut std::ffi::c_void);
            if err.is_set() {
                semsg(c"Error in %s:".as_ptr(), vimrc_lua);
                semsg_multiline(c"emsg".as_ptr(), err.msg);
                api_clear_error(&mut err);
            }
        }
    } else if os_path_exists(vimrc) {
        let str_ = nlua_read_secure(vimrc);
        if !str_.is_null() {
            do_source_str(str_, vimrc);
            xfree(str_ as *mut std::ffi::c_void);
        }
    } else if os_path_exists(exrc) {
        let str_ = nlua_read_secure(exrc);
        if !str_.is_null() {
            do_source_str(str_, exrc);
            xfree(str_ as *mut std::ffi::c_void);
        }
    }
}

/// Source startup scripts based on mparm_T settings.
///
/// # Safety
/// `parmp` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_source_startup_scripts(parmp: *const MparmT) {
    let p = &*parmp;

    if !p.use_vimrc.is_null() {
        // -u given: use only this file
        if cstr_eq(p.use_vimrc, c"NONE".as_ptr()) || cstr_eq(p.use_vimrc, c"NORC".as_ptr()) {
            // Do nothing
        } else {
            let result = do_source(p.use_vimrc, false, DOSO_NONE, std::ptr::null_mut());
            if result != OK {
                // e_cannot_read_from_str_2 message
                semsg(c"E484: Can't open file %s".as_ptr(), p.use_vimrc);
            }
        }
    } else if !silent_mode {
        rs_do_system_initialization();

        if rs_do_user_initialization() {
            rs_do_exrc_initialization();
        }
    }
    // TIME_MSG("sourcing vimrc file(s)") is a no-op in Rust
}

/// C string length.
///
/// # Safety
/// `s` must be a valid nul-terminated C string.
unsafe fn strlen_c(s: *const c_char) -> usize {
    let mut n = 0usize;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// Compare two C strings for equality.
///
/// # Safety
/// Both must be valid nul-terminated C strings.
unsafe fn cstr_eq(a: *const c_char, b: *const c_char) -> bool {
    let mut i = 0usize;
    loop {
        let ac = *a.add(i);
        let bc = *b.add(i);
        if ac != bc {
            return false;
        }
        if ac == 0 {
            return true;
        }
        i += 1;
    }
}
