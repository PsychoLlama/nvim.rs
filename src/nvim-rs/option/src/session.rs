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

use std::ffi::{c_char, c_int};

use crate::{OptFlags, OptInt, OptValType};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // optval helpers
    fn nvim_optval_from_varp(opt_idx: c_int, varp: *mut std::ffi::c_void)
        -> crate::storage::OptVal;
    fn nvim_get_option_unset_value(opt_idx: c_int) -> crate::storage::OptVal;
    fn nvim_option_get_def_val(opt_idx: c_int) -> crate::storage::OptVal;
    fn nvim_option_is_global_local(opt_idx: c_int) -> c_int;
    fn nvim_opt_is_hidden(opt_idx: c_int) -> c_int;
    fn nvim_option_has_type(opt_idx: c_int, type_: c_int) -> c_int;
    fn rs_optval_equal(a: crate::storage::OptVal, b: crate::storage::OptVal) -> c_int;

    // wc_use_keyname
    fn nvim_option_get_p_wc_ptr() -> *const std::ffi::c_void;
    fn nvim_option_get_p_wcm_ptr() -> *const std::ffi::c_void;
    fn find_special_key_in_table(c: c_int) -> c_int;

    // option_value2string
    fn nvim_get_varp_scope_by_idx(opt_idx: c_int, opt_flags: c_int) -> *mut std::ffi::c_void;
    fn nvim_get_namebuff() -> *mut c_char;
    fn nvim_get_namebuff_size() -> usize;
    fn get_special_key_name(c: c_int, modifiers: c_int) -> *const c_char;
    fn transchar(c: c_int) -> *const c_char;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn nvim_option_home_replace(src: *const c_char, dst: *mut c_char, dstlen: usize);

    // put_set
    fn nvim_put_set_get_opt_name_flags(opt_idx: c_int, name: *mut *const c_char, flags: *mut u64);
    fn nvim_option_get_var_ptr(opt_idx: c_int) -> *mut std::ffi::c_void;
    fn nvim_call_put_escstr(fd: *mut libc::FILE, str_: *const c_char, what: c_int) -> c_int;
    fn nvim_call_put_eol(fd: *mut libc::FILE) -> c_int;
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
    fn strlen(s: *const c_char) -> usize;
    fn vim_strchr(s: *const c_char, c: c_int) -> *const c_char;

    // makefoldset - curwin fold option varp pointers
    fn nvim_curwin_p_fdm_varp() -> *mut std::ffi::c_void;
    fn nvim_curwin_p_fde_varp() -> *mut std::ffi::c_void;
    fn nvim_curwin_p_fmr_varp() -> *mut std::ffi::c_void;
    fn nvim_curwin_p_fdi_varp() -> *mut std::ffi::c_void;
    fn nvim_curwin_p_fdl_varp() -> *mut std::ffi::c_void;
    fn nvim_curwin_p_fml_varp() -> *mut std::ffi::c_void;
    fn nvim_curwin_p_fdn_varp() -> *mut std::ffi::c_void;
    fn nvim_curwin_p_fen_varp() -> *mut std::ffi::c_void;
    fn nvim_get_opt_idx_foldmethod() -> c_int;
    fn nvim_get_opt_idx_foldexpr() -> c_int;
    fn nvim_get_opt_idx_foldmarker() -> c_int;
    fn nvim_get_opt_idx_foldignore() -> c_int;
    fn nvim_get_opt_idx_foldlevel() -> c_int;
    fn nvim_get_opt_idx_foldminlines() -> c_int;
    fn nvim_get_opt_idx_foldnestmax() -> c_int;
    fn nvim_get_opt_idx_foldenable() -> c_int;
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
    let current_val = nvim_optval_from_varp(opt_idx, varp);
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
    let p_wc = nvim_option_get_p_wc_ptr();
    let p_wcm = nvim_option_get_p_wcm_ptr();
    if varp == p_wc || varp == p_wcm {
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
    let namebuff_size = nvim_get_namebuff_size();

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
        let mut name: *const c_char = std::ptr::null();
        let mut flags: u64 = 0;
        nvim_put_set_get_opt_name_flags(opt_idx, &mut name, &mut flags);
        if (flags & OptFlags::EXPAND.0 as u64) != 0 {
            nvim_option_home_replace(str_varp, namebuff, namebuff_size);
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
    let value = nvim_optval_from_varp(opt_idx, varp);

    let mut name: *const c_char = std::ptr::null();
    let mut flags: u64 = 0;
    nvim_put_set_get_opt_name_flags(opt_idx, &mut name, &mut flags);

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
                    nvim_option_home_replace(value_str, buf, size);

                    let maxpathl = nvim_get_namebuff_size(); // MAXPATHL
                    if size >= maxpathl
                        && (flags & OptFlags::COMMA.0 as u64) != 0
                        && !vim_strchr(buf as *const c_char, b',' as c_int).is_null()
                    {
                        // Write each comma-separated part as a separate +=
                        let part = xmalloc(size);
                        if nvim_call_put_eol(fd) == FAIL {
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
                            if nvim_call_put_escstr(fd, part, 2) == FAIL
                                || nvim_call_put_eol(fd) == FAIL
                            {
                                xfree(buf);
                                xfree(part);
                                return FAIL;
                            }
                        }
                        xfree(buf);
                        xfree(part);
                        return OK;
                    }
                    let r = nvim_call_put_escstr(fd, buf as *const c_char, 2);
                    xfree(buf);
                    if r == FAIL {
                        return FAIL;
                    }
                } else if nvim_call_put_escstr(fd, value_str, 2) == FAIL {
                    return FAIL;
                }
            }
        }
    }

    if nvim_call_put_eol(fd) < 0 {
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
#[no_mangle]
pub unsafe extern "C" fn rs_makefoldset(fd: *mut libc::FILE) -> c_int {
    let setlocal = c"setlocal".as_ptr();

    if rs_put_set(
        fd,
        setlocal,
        nvim_get_opt_idx_foldmethod(),
        nvim_curwin_p_fdm_varp(),
    ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            nvim_get_opt_idx_foldexpr(),
            nvim_curwin_p_fde_varp(),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            nvim_get_opt_idx_foldmarker(),
            nvim_curwin_p_fmr_varp(),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            nvim_get_opt_idx_foldignore(),
            nvim_curwin_p_fdi_varp(),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            nvim_get_opt_idx_foldlevel(),
            nvim_curwin_p_fdl_varp(),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            nvim_get_opt_idx_foldminlines(),
            nvim_curwin_p_fml_varp(),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            nvim_get_opt_idx_foldnestmax(),
            nvim_curwin_p_fdn_varp(),
        ) == FAIL
        || rs_put_set(
            fd,
            setlocal,
            nvim_get_opt_idx_foldenable(),
            nvim_curwin_p_fen_varp(),
        ) == FAIL
    {
        return FAIL;
    }
    OK
}
