//! Option session persistence functions (Phase 4)
//!
//! Rust implementations of optval_default, wc_use_keyname, option_value2string,
//! put_set, and makefoldset from option_shim.c.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::missing_panics_doc)]

use std::ffi::{c_char, c_int, c_void};

use crate::{OptFlags, OptInt, OptValType};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // makeset - Phase 2
    #[link_name = "rs_get_option_flags"]
    fn nvim_get_option_flags(opt_idx: c_int) -> u32;
    #[link_name = "rs_option_is_global_only"]
    fn nvim_option_is_global_only(opt_idx: c_int) -> c_int;
    #[link_name = "rs_option_is_window_local"]
    fn nvim_option_is_window_local(opt_idx: c_int) -> c_int;
    fn put_line(fd: *mut libc::FILE, str_: *const c_char) -> c_int;
    fn nvim_option_get_fullname(opt_idx: c_int) -> *const c_char;
}

extern "C" {
    // optval helpers - nvim_optval_from_varp replaced by crate::value::rs_optval_from_varp
    #[link_name = "rs_get_option_unset_value"]
    fn nvim_get_option_unset_value(opt_idx: c_int) -> crate::storage::OptVal;
    fn nvim_option_get_def_val(opt_idx: c_int) -> crate::storage::OptVal;
    #[link_name = "rs_option_is_global_local"]
    fn nvim_option_is_global_local(opt_idx: c_int) -> c_int;
    #[link_name = "is_option_hidden"]
    fn nvim_opt_is_hidden(opt_idx: c_int) -> c_int;
    #[link_name = "option_has_type"]
    fn nvim_option_has_type(opt_idx: c_int, type_: c_int) -> c_int;
    fn rs_optval_equal(a: crate::storage::OptVal, b: crate::storage::OptVal) -> c_int;

    // wc_use_keyname
    static p_wc: crate::OptInt;
    static p_wcm: crate::OptInt;
    fn find_special_key_in_table(c: c_int) -> c_int;

    // option_value2string
    fn nvim_get_varp_scope_by_idx(opt_idx: c_int, opt_flags: c_int) -> *mut std::ffi::c_void;
    fn nvim_get_namebuff() -> *mut c_char;
    fn get_special_key_name(c: c_int, modifiers: c_int) -> *const c_char;
    fn transchar(c: c_int) -> *const c_char;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    #[link_name = "home_replace"]
    fn nvim_home_replace(
        buf: *const std::ffi::c_void,
        src: *const c_char,
        dst: *mut c_char,
        dstlen: usize,
        one: bool,
    ) -> usize;

    // put_set
    fn nvim_option_get_var_ptr(opt_idx: c_int) -> *mut std::ffi::c_void;
    fn put_escstr(fd: *mut libc::FILE, str_: *const c_char, what: c_int) -> c_int;
    fn put_eol(fd: *mut libc::FILE) -> c_int;
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
    fn strlen(s: *const c_char) -> usize;
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // makefoldset - get curwin and opt field addr directly
    static mut curwin: crate::WinHandle;
    fn nvim_win_get_opt_field_addr(win: crate::WinHandle, idx: c_int) -> *mut std::ffi::c_void;
}

// OptValType int values matching C enum
const K_OPT_VAL_TYPE_NUMBER: c_int = 1;

// TriState constants (from types_defs.h): kNone = -1, kFalse = 0, kTrue = 1
const K_NONE: c_int = -1;
const K_FALSE: c_int = 0;

// C return values
const OK: c_int = 1;
const FAIL: c_int = 0;

// IS_SPECIAL macro: c < 0
#[inline]
fn is_special(c: OptInt) -> bool {
    c < 0
}

/// Inline port of C `nvim_call_makeset_if_line`.
/// Writes `if &optname != 'val'` + EOL to fd; returns OK or FAIL.
unsafe fn makeset_if_line(
    fd: *mut libc::FILE,
    optname: *const c_char,
    val: *const c_char,
) -> c_int {
    if libc::fprintf(fd, c"if &%s != '%s'".as_ptr(), optname, val) < 0 {
        return FAIL;
    }
    if put_eol(fd) < 0 {
        FAIL
    } else {
        OK
    }
}

// =============================================================================
// optval_default
// =============================================================================

/// Return true (nonzero) if option `opt_idx` has its default value.
///
/// Corresponds to `optval_default()` in option_shim.c.
#[no_mangle]
pub unsafe extern "C" fn rs_optval_default(opt_idx: c_int, varp: *mut std::ffi::c_void) -> c_int {
    // Hidden options always use their default value.
    if nvim_opt_is_hidden(opt_idx) != 0 {
        return 1;
    }
    let current_val = crate::value::rs_optval_from_varp(opt_idx, varp);
    let default_val = nvim_option_get_def_val(opt_idx);
    c_int::from(rs_optval_equal(current_val, default_val) != 0)
}

// =============================================================================
// wc_use_keyname
// =============================================================================

/// Return true if `varp` points to 'wildchar' or 'wildcharm' and the value
/// can be printed as a keyname. Sets `*wcp` to the option value.
///
/// Corresponds to `wc_use_keyname()` in option_shim.c.
#[no_mangle]
pub unsafe extern "C" fn rs_wc_use_keyname(
    varp: *const std::ffi::c_void,
    wcp: *mut OptInt,
) -> c_int {
    if varp == std::ptr::addr_of!(p_wc).cast::<c_void>()
        || varp == std::ptr::addr_of!(p_wcm).cast::<c_void>()
    {
        let wc = *(varp as *const OptInt);
        *wcp = wc;
        if is_special(wc) || find_special_key_in_table(wc as c_int) >= 0 {
            return 1;
        }
    }
    0
}

// =============================================================================
// option_value2string
// =============================================================================

/// Write the formatted value of option `opt_idx` into NameBuff.
///
/// Corresponds to `option_value2string()` in option_shim.c.
/// Takes OptIndex (c_int) instead of vimoption_T* so the C thin wrapper calls
/// get_opt_idx(opt) first.
#[no_mangle]
pub unsafe extern "C" fn rs_option_value2string(opt_idx: c_int, opt_flags: c_int) {
    let varp = nvim_get_varp_scope_by_idx(opt_idx, opt_flags);
    assert!(!varp.is_null());

    let namebuff = nvim_get_namebuff();
    let namebuff_size = crate::defaults::MAXPATHL;

    if nvim_option_has_type(opt_idx, K_OPT_VAL_TYPE_NUMBER) != 0 {
        let mut wc: OptInt = 0;
        if rs_wc_use_keyname(varp, &mut wc) != 0 {
            let key_name = get_special_key_name(wc as c_int, 0);
            xstrlcpy(namebuff, key_name, namebuff_size);
        } else if wc != 0 {
            let tc = transchar(wc as c_int);
            xstrlcpy(namebuff, tc, namebuff_size);
        } else {
            let val = *(varp as *const OptInt);
            snprintf(namebuff, namebuff_size, c"%lld".as_ptr(), val as i64);
        }
    } else {
        // string option
        let str_varp = *(varp as *const *const c_char);
        let flags = nvim_get_option_flags(opt_idx) as u64;
        if (flags & OptFlags::EXPAND.0 as u64) != 0 {
            nvim_home_replace(std::ptr::null(), str_varp, namebuff, namebuff_size, false);
        } else {
            xstrlcpy(namebuff, str_varp, namebuff_size);
        }
    }
}

// =============================================================================
// put_set
// =============================================================================

/// Write a `:set opt=value` command for option `opt_idx` to file `fd`.
///
/// Corresponds to `put_set()` in option_shim.c.
#[no_mangle]
pub unsafe extern "C" fn rs_put_set(
    fd: *mut libc::FILE,
    cmd: *const c_char,
    opt_idx: c_int,
    varp: *mut std::ffi::c_void,
) -> c_int {
    let value = crate::value::rs_optval_from_varp(opt_idx, varp);

    let name = nvim_option_get_fullname(opt_idx);
    let flags = nvim_get_option_flags(opt_idx) as u64;

    // If this is a global-local option, varp is a local var (not opt->var),
    // and value equals the unset value, do nothing.
    let global_var = nvim_option_get_var_ptr(opt_idx);
    if nvim_option_is_global_local(opt_idx) != 0
        && varp != global_var
        && rs_optval_equal(value, nvim_get_option_unset_value(opt_idx)) != 0
    {
        return OK;
    }

    match value.type_ {
        OptValType::Nil => {
            // Should not happen (C uses abort())
            libc::abort();
        }
        OptValType::Boolean => {
            let bool_val = value.data.boolean;
            // kNone should not occur
            assert!(bool_val != K_NONE);
            let is_true = bool_val != K_FALSE;
            let prefix: &[u8] = if is_true { b"" } else { b"no" };
            // Write: "cmd prefix name\n" via fprintf equivalent
            // We build the string and use fputs
            let cmd_len = strlen(cmd);
            let prefix_len = prefix.len();
            let name_len = strlen(name);
            // total: cmd + " " + prefix + name + NUL
            let total = cmd_len + 1 + prefix_len + name_len + 1;
            let buf = xmalloc(total);
            let mut pos = 0usize;
            std::ptr::copy_nonoverlapping(cmd as *const u8, buf.add(pos) as *mut u8, cmd_len);
            pos += cmd_len;
            *buf.add(pos) = b' ' as c_char;
            pos += 1;
            if !prefix.is_empty() {
                std::ptr::copy_nonoverlapping(
                    prefix.as_ptr() as *const u8,
                    buf.add(pos) as *mut u8,
                    prefix_len,
                );
                pos += prefix_len;
            }
            std::ptr::copy_nonoverlapping(name as *const u8, buf.add(pos) as *mut u8, name_len);
            pos += name_len;
            *buf.add(pos) = 0;
            let r = libc::fputs(buf as *const c_char, fd);
            xfree(buf);
            if r < 0 {
                return FAIL;
            }
        }
        OptValType::Number => {
            // Write: "cmd name=" via fputs
            if write_str2(fd, cmd, name, b'=') < 0 {
                return FAIL;
            }
            let value_num = value.data.number;
            let mut wc: OptInt = 0;
            if rs_wc_use_keyname(varp, &mut wc) != 0 {
                // print wildchar/wildcharm as key name
                let key_name = get_special_key_name(wc as c_int, 0);
                if libc::fputs(key_name, fd) < 0 {
                    return FAIL;
                }
            } else {
                // Use nvim_call_fprintf_int to write the number
                if write_int64(fd, value_num as i64) < 0 {
                    return FAIL;
                }
            }
        }
        OptValType::String => {
            // Write: "cmd name=" via fputs
            if write_str2(fd, cmd, name, b'=') < 0 {
                return FAIL;
            }
            let value_str = value.data.string.data as *const c_char;
            if !value_str.is_null() {
                if (flags & OptFlags::EXPAND.0 as u64) != 0 {
                    let size = strlen(value_str) + 1;
                    let buf = xmalloc(size);
                    nvim_home_replace(std::ptr::null(), value_str, buf, size, false);

                    let maxpathl = crate::defaults::MAXPATHL;
                    if size >= maxpathl
                        && (flags & OptFlags::COMMA.0 as u64) != 0
                        && !vim_strchr(buf as *const c_char, b',' as c_int).is_null()
                    {
                        // Write each comma-separated part as a separate +=
                        let part = xmalloc(size);
                        if put_eol(fd) == FAIL {
                            xfree(buf);
                            xfree(part);
                            return FAIL;
                        }
                        let mut p: *mut c_char = buf;
                        while *p != 0 {
                            if write_str2(fd, cmd, name, b'+') < 0 {
                                xfree(buf);
                                xfree(part);
                                return FAIL;
                            }
                            // append "=" after "+"
                            if libc::fputc(b'=' as c_int, fd) < 0 {
                                xfree(buf);
                                xfree(part);
                                return FAIL;
                            }
                            crate::parsing::rs_copy_option_part(
                                &raw mut p,
                                part,
                                size,
                                c",".as_ptr() as *const c_char,
                            );
                            if put_escstr(fd, part, 2) == FAIL || put_eol(fd) == FAIL {
                                xfree(buf);
                                xfree(part);
                                return FAIL;
                            }
                        }
                        xfree(buf);
                        xfree(part);
                        return OK;
                    }
                    let r = put_escstr(fd, buf as *const c_char, 2);
                    xfree(buf);
                    if r == FAIL {
                        return FAIL;
                    }
                } else if put_escstr(fd, value_str, 2) == FAIL {
                    return FAIL;
                }
            }
        }
    }

    if put_eol(fd) < 0 {
        return FAIL;
    }
    OK
}

// Helper: write "cmd name<suffix_char>" to fd
// suffix_char is '=' or '+' (for "+=" we write '+' then '=' separately)
unsafe fn write_str2(
    fd: *mut libc::FILE,
    cmd: *const c_char,
    name: *const c_char,
    suffix: u8,
) -> c_int {
    let cmd_len = strlen(cmd);
    let name_len = strlen(name);
    // total: cmd + " " + name + suffix + NUL
    let total = cmd_len + 1 + name_len + 1 + 1;
    let buf = xmalloc(total);
    let mut pos = 0usize;
    std::ptr::copy_nonoverlapping(cmd as *const u8, buf.add(pos) as *mut u8, cmd_len);
    pos += cmd_len;
    *buf.add(pos) = b' ' as c_char;
    pos += 1;
    std::ptr::copy_nonoverlapping(name as *const u8, buf.add(pos) as *mut u8, name_len);
    pos += name_len;
    *buf.add(pos) = suffix as c_char;
    pos += 1;
    *buf.add(pos) = 0;
    let r = libc::fputs(buf as *const c_char, fd);
    xfree(buf);
    r
}

// Helper: write an i64 integer to fd using libc fprintf
unsafe fn write_int64(fd: *mut libc::FILE, val: i64) -> c_int {
    // Use a stack buffer to format the integer
    let mut buf = [0u8; 32];
    let len = snprintf(
        buf.as_mut_ptr() as *mut c_char,
        buf.len(),
        c"%lld".as_ptr(),
        val,
    );
    if len < 0 {
        return -1;
    }
    libc::fputs(buf.as_ptr() as *const c_char, fd)
}

// =============================================================================
// makefoldset
// =============================================================================

/// Write fold option set commands to file `fd`.
///
/// Corresponds to `makefoldset()` in option_shim.c.
#[allow(clippy::must_use_candidate)]
#[export_name = "makefoldset"]
pub unsafe extern "C" fn rs_makefoldset(fd: *mut libc::FILE) -> c_int {
    let setlocal = c"setlocal".as_ptr();

    if rs_put_set(
        fd,
        setlocal,
        K_OPT_FOLDMETHOD,
        nvim_win_get_opt_field_addr(curwin, K_OPT_FOLDMETHOD),
    ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            K_OPT_FOLDEXPR,
            nvim_win_get_opt_field_addr(curwin, K_OPT_FOLDEXPR),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            K_OPT_FOLDMARKER,
            nvim_win_get_opt_field_addr(curwin, K_OPT_FOLDMARKER),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            K_OPT_FOLDIGNORE,
            nvim_win_get_opt_field_addr(curwin, K_OPT_FOLDIGNORE),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            K_OPT_FOLDLEVEL,
            nvim_win_get_opt_field_addr(curwin, K_OPT_FOLDLEVEL),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            K_OPT_FOLDMINLINES,
            nvim_win_get_opt_field_addr(curwin, K_OPT_FOLDMINLINES),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            K_OPT_FOLDNESTMAX,
            nvim_win_get_opt_field_addr(curwin, K_OPT_FOLDNESTMAX),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            K_OPT_FOLDENABLE,
            nvim_win_get_opt_field_addr(curwin, K_OPT_FOLDENABLE),
        ) == FAIL
    {
        return FAIL;
    }
    OK
}

// =============================================================================
// makeset (Phase 5 of option_shim migration)
// =============================================================================

use crate::opt_index::{
    K_OPT_COUNT, K_OPT_FILETYPE, K_OPT_FOLDENABLE, K_OPT_FOLDEXPR, K_OPT_FOLDIGNORE,
    K_OPT_FOLDLEVEL, K_OPT_FOLDMARKER, K_OPT_FOLDMETHOD, K_OPT_FOLDMINLINES, K_OPT_FOLDNESTMAX,
    K_OPT_PACKPATH, K_OPT_RUNTIMEPATH, K_OPT_SYNTAX,
};

// Option flag constants (from OptFlags in lib.rs / option_defs.h)
const K_OPT_FLAG_NO_MKRC: u32 = 1 << 4;
const K_OPT_FLAG_PRI_MKRC: u32 = 1 << 19;
const K_OPT_FLAG_NO_GLOB: u32 = 1 << 16;

// OptionSetFlags constants (from lib.rs / option.h)
const OPT_GLOBAL: c_int = 0x01;
const OPT_LOCAL: c_int = 0x02;
const OPT_SKIPRTP: c_int = 0x80;

/// Write modified options as `:set` commands to a file.
///
/// Corresponds to `makeset()` in option_shim.c.
#[allow(clippy::must_use_candidate)]
#[export_name = "makeset"]
pub unsafe extern "C" fn rs_makeset(
    fd: *mut libc::FILE,
    opt_flags: c_int,
    local_only: c_int,
) -> c_int {
    // Get the var_ptr for runtimepath and packpath for skiprtp check.
    let rtp_var_ptr = nvim_option_get_var_ptr(K_OPT_RUNTIMEPATH);
    let pp_var_ptr = nvim_option_get_var_ptr(K_OPT_PACKPATH);

    // Do the loop over options[] twice: once for kOptFlagPriMkrc, once without.
    for pri in (0..=1i32).rev() {
        for opt_idx in 0..K_OPT_COUNT {
            let flags = nvim_get_option_flags(opt_idx);

            // Skip if kOptFlagNoMkrc is set
            if (flags & K_OPT_FLAG_NO_MKRC) != 0 {
                continue;
            }
            // Match priority: pri==1 means kOptFlagPriMkrc must be set
            let has_pri = (flags & K_OPT_FLAG_PRI_MKRC) != 0;
            if (pri == 1) != has_pri {
                continue;
            }

            // Skip global-only option when only doing locals
            if nvim_option_is_global_only(opt_idx) != 0 && (opt_flags & OPT_GLOBAL) == 0 {
                continue;
            }

            // Do not store buffer-specific options in a vimrc
            if (opt_flags & OPT_GLOBAL) != 0 && (flags & K_OPT_FLAG_NO_GLOB) != 0 {
                continue;
            }

            // Get currently used value
            let varp = nvim_get_varp_scope_by_idx(opt_idx, opt_flags);
            // Hidden options (varp == NULL) are never written
            if varp.is_null() {
                continue;
            }

            // Global values only written when not at default
            if (opt_flags & OPT_GLOBAL) != 0 && rs_optval_default(opt_idx, varp) != 0 {
                continue;
            }

            // OPT_SKIPRTP: skip runtimepath and packpath
            if (opt_flags & OPT_SKIPRTP) != 0 {
                let opt_var = nvim_option_get_var_ptr(opt_idx);
                if opt_var == rtp_var_ptr || opt_var == pp_var_ptr {
                    continue;
                }
            }

            let mut round = 2i32;
            let mut varp_local: *mut std::ffi::c_void = std::ptr::null_mut();
            let mut cur_varp = varp;

            if nvim_option_is_window_local(opt_idx) != 0 {
                // Skip window-local option when only doing globals
                if (opt_flags & OPT_LOCAL) == 0 {
                    continue;
                }
                // When fresh value is not at default, write it too
                if (opt_flags & OPT_GLOBAL) == 0 && local_only == 0 {
                    let varp_fresh = nvim_get_varp_scope_by_idx(opt_idx, OPT_GLOBAL);
                    if rs_optval_default(opt_idx, varp_fresh) == 0 {
                        round = 1;
                        varp_local = cur_varp;
                        cur_varp = varp_fresh;
                    }
                }
            }

            // Round 1: fresh value for window-local; Round 2: other values
            while round <= 2 {
                let cmd: *const c_char = if round == 1 || (opt_flags & OPT_GLOBAL) != 0 {
                    c"set".as_ptr()
                } else {
                    c"setlocal".as_ptr()
                };

                // syntax and filetype get an 'if' guard to avoid reloading
                let do_endif = if opt_idx == K_OPT_SYNTAX || opt_idx == K_OPT_FILETYPE {
                    let optname = nvim_option_get_fullname(opt_idx);
                    let str_val = *(cur_varp.cast::<*const c_char>());
                    if makeset_if_line(fd, optname, str_val) == FAIL {
                        return FAIL;
                    }
                    true
                } else {
                    false
                };

                if rs_put_set(fd, cmd, opt_idx, cur_varp) == FAIL {
                    return FAIL;
                }

                if do_endif && put_line(fd, c"endif".as_ptr()) == FAIL {
                    return FAIL;
                }

                round += 1;
                // Swap to varp_local for round 2
                if round == 2 {
                    cur_varp = varp_local;
                }
            }
        }
    }
    OK
}
