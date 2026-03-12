//! Environment variable operations
//!
//! Provides portable access to environment variables.
//!
//! Note: Functions that return allocated strings use nvim's `xmallocz`
//! allocator, so callers should free them with `xfree`.

use std::env;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;

use nvim_memory::NvimString;

extern "C" {
    #[link_name = "xfree"]
    fn nvim_xfree(ptr: *mut c_char);
    #[link_name = "xstrdup"]
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    #[link_name = "xmalloc"]
    fn nvim_xmalloc(size: usize) -> *mut c_char;
    #[link_name = "xstrlcpy"]
    fn nvim_xstrlcpy(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;
    #[link_name = "xstrlcat"]
    fn nvim_xstrlcat(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;
    #[link_name = "expand_env_esc"]
    fn c_expand_env_esc(
        srcp: *const c_char,
        dst: *mut c_char,
        dstlen: c_int,
        esc: bool,
        one: bool,
        prefix: *mut c_char,
    ) -> usize;
    #[link_name = "home_replace"]
    fn c_home_replace(
        buf: *const c_void,
        src: *const c_char,
        dst: *mut c_char,
        dstlen: usize,
        one: bool,
    ) -> usize;
    #[link_name = "path_is_absolute"]
    fn c_path_is_absolute(fname: *const c_char) -> bool;
    #[link_name = "path_tail_with_sep"]
    fn c_path_tail_with_sep(fname: *mut c_char) -> *mut c_char;
    #[link_name = "path_tail"]
    fn c_path_tail(fname: *const c_char) -> *mut c_char;
    #[link_name = "striequal"]
    fn c_striequal(a: *const c_char, b: *const c_char) -> bool;
    #[link_name = "os_getenv_noalloc"]
    fn c_os_getenv_noalloc(name: *const c_char) -> *mut c_char;
    #[link_name = "init_homedir"]
    fn c_init_homedir();
    #[link_name = "internal_error"]
    fn c_internal_error(where_: *const c_char);
    #[link_name = "nvim_xp_get_buf"]
    fn nvim_xp_get_buf(xp: *mut c_void) -> *mut c_char;
    // C globals accessed by Phase 2 functions
    static mut didset_vim: bool;
    static mut didset_vimruntime: bool;
}

/// Get the value of an environment variable.
///
/// Returns a newly allocated string (via `xmallocz`) that must be freed
/// with `xfree`, or NULL if the variable is not set or empty.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[export_name = "os_getenv"]
pub unsafe extern "C" fn rs_os_getenv(name: *const c_char) -> *mut c_char {
    if name.is_null() {
        return ptr::null_mut();
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    if name_str.is_empty() {
        return ptr::null_mut();
    }

    match env::var(name_str) {
        Ok(value) if !value.is_empty() => {
            // Use NvimString which allocates with xmallocz
            // The string contains no interior NULs (env::var returns valid UTF-8)
            NvimString::new(&value).into_raw()
        }
        _ => ptr::null_mut(),
    }
}

// Note: rs_os_getenv_free is no longer needed since rs_os_getenv now uses
// xmallocz. Callers should use xfree() directly to free returned strings.

/// Check if an environment variable exists (and optionally is non-empty).
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[export_name = "os_env_exists"]
pub unsafe extern "C" fn rs_os_env_exists(name: *const c_char, nonempty: bool) -> bool {
    if name.is_null() {
        return false;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if name_str.is_empty() {
        return false;
    }

    if nonempty {
        // Must be set and non-empty
        match env::var(name_str) {
            Ok(v) => !v.is_empty(),
            Err(_) => false,
        }
    } else {
        // Only needs to be set (even if empty)
        env::var_os(name_str).is_some()
    }
}

/// Set an environment variable.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `name` and `value` must be valid null-terminated C strings.
#[export_name = "os_setenv"]
pub unsafe extern "C" fn rs_os_setenv(
    name: *const c_char,
    value: *const c_char,
    overwrite: c_int,
) -> c_int {
    if name.is_null() || value.is_null() {
        return -1;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let value_cstr = unsafe { CStr::from_ptr(value) };

    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let value_str = match value_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    if name_str.is_empty() {
        return -1;
    }

    #[cfg(windows)]
    {
        if overwrite == 0 && !unsafe { rs_os_env_exists(name, true) } {
            return 0;
        }
        // Windows (Vim-compat): Empty string undefines the env var.
        if value_str.is_empty() {
            return unsafe { rs_os_unsetenv(name) };
        }
    }

    #[cfg(not(windows))]
    {
        if overwrite == 0 && unsafe { rs_os_env_exists(name, false) } {
            return 0;
        }
    }

    // Note: std::env::set_var is marked unsafe in newer Rust versions
    // because it's not thread-safe. For now, we proceed with caution.
    #[allow(deprecated)]
    env::set_var(name_str, value_str);
    0
}

/// Unset an environment variable.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[export_name = "os_unsetenv"]
pub unsafe extern "C" fn rs_os_unsetenv(name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    if name_str.is_empty() {
        return -1;
    }

    #[allow(deprecated)]
    env::remove_var(name_str);
    0
}

/// Get the current process ID.
#[export_name = "os_get_pid"]
pub extern "C" fn rs_os_get_pid() -> i64 {
    i64::from(std::process::id())
}

/// Gets the hostname of the current machine.
///
/// Writes the hostname to `buf` (max `size` bytes including NUL).
/// On failure, sets `buf[0]` to NUL.
///
/// # Safety
///
/// `buf` must point to a buffer of at least `size` bytes.
#[export_name = "os_get_hostname"]
pub unsafe extern "C" fn rs_os_get_hostname(buf: *mut c_char, size: usize) {
    if buf.is_null() || size == 0 {
        return;
    }

    #[cfg(unix)]
    {
        let mut hostname_buf = vec![0u8; size];
        let result = unsafe { libc::gethostname(hostname_buf.as_mut_ptr().cast(), size) };

        if result != 0 {
            unsafe {
                *buf = 0;
            }
            return;
        }

        // Ensure null termination
        hostname_buf[size - 1] = 0;

        // Copy to output buffer
        unsafe {
            ptr::copy_nonoverlapping(hostname_buf.as_ptr(), buf.cast(), size);
        }
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::SystemInformation::ComputerNameDnsHostname;
        use windows_sys::Win32::System::SystemInformation::GetComputerNameExA;

        let mut len = size as u32;
        let result = unsafe { GetComputerNameExA(ComputerNameDnsHostname, buf.cast(), &mut len) };

        if result == 0 {
            unsafe {
                *buf = 0;
            }
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        unsafe {
            *buf = 0;
        }
    }
}

/// Returns number of variables in the current environment variables block.
#[export_name = "os_get_fullenv_size"]
pub extern "C" fn rs_os_get_fullenv_size() -> usize {
    env::vars_os().count()
}

/// Copies the current environment variables into the given array `env`.
/// Each element is of the form "NAME=VALUE", allocated with xstrdup.
///
/// # Safety
///
/// `env` must point to an array of at least `env_size` pointers.
#[export_name = "os_copy_fullenv"]
pub unsafe extern "C" fn rs_os_copy_fullenv(env: *mut *mut c_char, env_size: usize) {
    if env.is_null() || env_size == 0 {
        return;
    }
    let mut i = 0usize;
    for (key, val) in env::vars() {
        if i >= env_size {
            break;
        }
        let entry = format!("{key}={val}");
        if let Ok(cstr) = CString::new(entry) {
            let ptr = unsafe { nvim_xstrdup(cstr.as_ptr()) };
            unsafe {
                *env.add(i) = ptr;
            }
            i += 1;
        }
    }
}

/// Frees an environment array allocated by `os_copy_fullenv`.
///
/// # Safety
///
/// `env` must be a null-terminated array of pointers allocated with xmalloc,
/// and the array itself must also be xmalloc-allocated.
#[export_name = "os_free_fullenv"]
pub unsafe extern "C" fn rs_os_free_fullenv(env: *mut *mut c_char) {
    if env.is_null() {
        return;
    }
    let mut p = env;
    while !unsafe { *p }.is_null() {
        unsafe {
            nvim_xfree(*p);
            *p = ptr::null_mut();
            p = p.add(1);
        }
    }
    unsafe {
        nvim_xfree(env.cast());
    }
}

/// Signals to the OS that Nvim is an application for "interactive work"
/// which should be prioritized similar to a GUI app.
#[export_name = "os_hint_priority"]
pub extern "C" fn rs_os_hint_priority() {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn mach_task_self_() -> u32;
            fn task_policy_set(task: u32, flavor: u32, policy_info: *const i32, count: u32) -> i32;
        }
        // TASK_CATEGORY_POLICY = 1
        // TASK_DEFAULT_APPLICATION = 4 (integer_t)
        let policy: i32 = 4;
        unsafe {
            task_policy_set(mach_task_self_(), 1, &policy, 1);
        }
    }
}

/// Gets the name of the environment variable at `index`.
///
/// Result must be freed by the caller with `xfree`.
///
/// # Safety
///
/// Always safe to call.
#[export_name = "os_getenvname_at_index"]
pub extern "C" fn rs_os_getenvname_at_index(index: usize) -> *mut c_char {
    match env::vars().nth(index) {
        Some((key, _)) => match CString::new(key) {
            Ok(cstr) => unsafe { nvim_xstrdup(cstr.as_ptr()) },
            Err(_) => ptr::null_mut(),
        },
        None => ptr::null_mut(),
    }
}

// MAXPATHL value (must match nvim's MAXPATHL)
const MAXPATHL: usize = 4096;

// EXPAND_BUF_LEN (must match cmdexpand_defs.h)
const EXPAND_BUF_LEN: usize = 256;

/// Iterates $PATH-like delimited list `val` forward.
///
/// # Safety
///
/// `val`, `dir`, and `len` must be valid pointers.
#[export_name = "vim_env_iter"]
pub unsafe extern "C" fn rs_vim_env_iter(
    delim: c_char,
    val: *const c_char,
    iter: *const c_void,
    dir: *mut *const c_char,
    len: *mut usize,
) -> *const c_void {
    let varval: *const c_char = if iter.is_null() { val } else { iter.cast() };

    unsafe {
        *dir = varval;
    }

    // Find next delimiter
    let slice_start = varval;
    let mut p = varval;
    while unsafe { *p } != 0 && unsafe { *p } != delim {
        p = unsafe { p.add(1) };
    }

    // p.offset_from is always non-negative since p >= slice_start
    let dist = unsafe { p.offset_from(slice_start) };
    #[allow(clippy::cast_sign_loss)]
    let entry_len = dist as usize;
    unsafe { *len = entry_len };

    if unsafe { *p } == 0 {
        ptr::null()
    } else {
        unsafe { p.add(1) }.cast()
    }
}

/// Iterates $PATH-like delimited list `val` in reverse.
///
/// # Safety
///
/// `val`, `dir`, and `len` must be valid pointers.
#[export_name = "vim_env_iter_rev"]
pub unsafe extern "C" fn rs_vim_env_iter_rev(
    delim: c_char,
    val: *const c_char,
    iter: *const c_void,
    dir: *mut *const c_char,
    len: *mut usize,
) -> *const c_void {
    // Calculate string length of val
    let val_len = unsafe {
        let mut p = val;
        while *p != 0 {
            p = p.add(1);
        }
        // p >= val always
        #[allow(clippy::cast_sign_loss)]
        let n = p.offset_from(val) as usize;
        n
    };

    let varend: *const c_char = if iter.is_null() {
        if val_len == 0 {
            // Empty string: no entries
            unsafe {
                *dir = val;
                *len = 0;
            }
            return ptr::null();
        }
        unsafe { val.add(val_len - 1) }
    } else {
        iter.cast()
    };

    // varend >= val always
    #[allow(clippy::cast_sign_loss)]
    let varlen = unsafe { varend.offset_from(val) as usize } + 1;

    // Find last occurrence of delim in val[0..varlen]
    let colon = {
        let mut found: *const c_char = ptr::null();
        let mut p = val;
        let end = unsafe { val.add(varlen) };
        while p < end {
            if unsafe { *p } == delim {
                found = p;
            }
            p = unsafe { p.add(1) };
        }
        found
    };

    if colon.is_null() {
        unsafe {
            *len = varlen;
            *dir = val;
        }
        ptr::null()
    } else {
        unsafe {
            *dir = colon.add(1);
            // varend >= colon always (colon is in val[0..varlen])
            #[allow(clippy::cast_sign_loss)]
            let entry_len = varend.offset_from(colon) as usize;
            *len = entry_len;
        }
        unsafe { colon.sub(1) }.cast()
    }
}

/// Call `expand_env()` and store the result in an allocated string.
///
/// # Safety
///
/// `src` must be a valid C string.
#[export_name = "expand_env_save"]
pub unsafe extern "C" fn rs_expand_env_save(src: *mut c_char) -> *mut c_char {
    unsafe { rs_expand_env_save_opt(src, false) }
}

/// Like `expand_env_save()` but when `one` is true, handle string as one filename.
///
/// # Safety
///
/// `src` must be a valid C string.
#[export_name = "expand_env_save_opt"]
pub unsafe extern "C" fn rs_expand_env_save_opt(src: *mut c_char, one: bool) -> *mut c_char {
    let p = unsafe { nvim_xmalloc(MAXPATHL) };
    #[allow(clippy::cast_possible_truncation)]
    unsafe {
        c_expand_env_esc(src, p, MAXPATHL as c_int, false, one, ptr::null_mut());
    }
    p
}

/// Expand environment variable with path name.
///
/// # Safety
///
/// `src` and `dst` must be valid C strings.
#[export_name = "expand_env"]
pub unsafe extern "C" fn rs_expand_env(src: *mut c_char, dst: *mut c_char, dstlen: c_int) -> usize {
    unsafe { c_expand_env_esc(src, dst, dstlen, false, false, ptr::null_mut()) }
}

/// Function given to `ExpandGeneric()` to obtain an environment variable name.
///
/// # Safety
///
/// `xp` must be a valid pointer to `expand_T`.
#[export_name = "get_env_name"]
pub unsafe extern "C" fn rs_get_env_name(xp: *mut c_void, idx: c_int) -> *mut c_char {
    assert!(idx >= 0);
    #[allow(clippy::cast_sign_loss)]
    let envname = rs_os_getenvname_at_index(idx as usize);
    if envname.is_null() {
        return ptr::null_mut();
    }
    let buf = unsafe { nvim_xp_get_buf(xp) };
    unsafe {
        nvim_xstrlcpy(buf, envname, EXPAND_BUF_LEN);
        nvim_xfree(envname);
    }
    buf
}

/// Appends the head of `fname` to $PATH and sets it in the environment.
///
/// # Safety
///
/// `fname` must be a valid null-terminated C string.
#[export_name = "os_setenv_append_path"]
pub unsafe extern "C" fn rs_os_setenv_append_path(fname: *const c_char) -> bool {
    #[cfg(windows)]
    const MAX_ENVPATHLEN: usize = 8192;
    #[cfg(not(windows))]
    const MAX_ENVPATHLEN: usize = usize::MAX;

    if !unsafe { c_path_is_absolute(fname) } {
        unsafe { c_internal_error(c"os_setenv_append_path()".as_ptr()) };
        return false;
    }

    let tail = unsafe { c_path_tail_with_sep(fname.cast_mut()) };
    // tail >= fname always (path_tail_with_sep returns ptr into fname)
    #[allow(clippy::cast_sign_loss)]
    let dirlen = unsafe { tail.offset_from(fname) as usize };

    // Copy the directory portion into a local buffer
    let mut dir_buf = [0u8; MAXPATHL];
    if dirlen >= MAXPATHL {
        return false;
    }
    unsafe {
        ptr::copy_nonoverlapping(fname.cast::<u8>(), dir_buf.as_mut_ptr(), dirlen);
    }
    dir_buf[dirlen] = 0;

    let path = unsafe { rs_os_getenv(c"PATH".as_ptr()) };
    let pathlen = if path.is_null() {
        0usize
    } else {
        unsafe {
            let mut p = path;
            while *p != 0 {
                p = p.add(1);
            }
            // p >= path always
            #[allow(clippy::cast_sign_loss)]
            let n = p.offset_from(path) as usize;
            n
        }
    };
    let newlen = pathlen + dirlen + 2;
    let retval = if newlen < MAX_ENVPATHLEN {
        let temp = unsafe { nvim_xmalloc(newlen) };
        if pathlen == 0 {
            unsafe { *temp = 0 };
        } else {
            unsafe {
                nvim_xstrlcpy(temp, path, newlen);
            }
            #[cfg(windows)]
            let sep = b';' as c_char;
            #[cfg(not(windows))]
            let sep = b':' as c_char;
            #[cfg(windows)]
            let sep_str = c";";
            #[cfg(not(windows))]
            let sep_str = c":";
            let last = unsafe { *path.add(pathlen - 1) };
            if last != sep {
                unsafe {
                    nvim_xstrlcat(temp, sep_str.as_ptr(), newlen);
                }
            }
        }
        unsafe {
            nvim_xstrlcat(temp, dir_buf.as_ptr().cast(), newlen);
            rs_os_setenv(c"PATH".as_ptr(), temp, 1);
            nvim_xfree(temp);
        }
        true
    } else {
        false
    };
    if !path.is_null() {
        unsafe { nvim_xfree(path) };
    }
    retval
}

/// Returns true if `sh` looks like it resolves to "cmd.exe".
///
/// # Safety
///
/// `sh` must be a valid null-terminated C string.
#[export_name = "os_shell_is_cmdexe"]
pub unsafe extern "C" fn rs_os_shell_is_cmdexe(sh: *const c_char) -> bool {
    if unsafe { *sh } == 0 {
        return false;
    }
    if unsafe { c_striequal(sh, c"$COMSPEC".as_ptr()) } {
        let comspec_val = unsafe { c_os_getenv_noalloc(c"COMSPEC".as_ptr()) };
        return unsafe { c_striequal(c"cmd.exe".as_ptr(), c_path_tail(comspec_val)) };
    }
    if unsafe { c_striequal(sh, c"cmd.exe".as_ptr()) }
        || unsafe { c_striequal(sh, c"cmd".as_ptr()) }
    {
        return true;
    }
    unsafe { c_striequal(c"cmd.exe".as_ptr(), c_path_tail(sh)) }
}

/// Removes environment variable and handles side effects.
///
/// # Safety
///
/// `var` must be a valid null-terminated C string.
#[export_name = "vim_unsetenv_ext"]
pub unsafe extern "C" fn rs_vim_unsetenv_ext(var: *const c_char) {
    unsafe { rs_os_unsetenv(var) };
    // Check if it's "VIM" or "VIMRUNTIME" (case-insensitive)
    let var_cstr = unsafe { CStr::from_ptr(var) };
    if let Ok(s) = var_cstr.to_str() {
        if s.eq_ignore_ascii_case("VIM") {
            unsafe { didset_vim = false };
        } else if s.eq_ignore_ascii_case("VIMRUNTIME") {
            unsafe { didset_vimruntime = false };
        }
    }
}

/// Sets environment variable and handles side effects.
///
/// # Safety
///
/// `name` and `val` must be valid null-terminated C strings.
#[export_name = "vim_setenv_ext"]
pub unsafe extern "C" fn rs_vim_setenv_ext(name: *const c_char, val: *const c_char) {
    unsafe { rs_os_setenv(name, val, 1) };
    let name_cstr = unsafe { CStr::from_ptr(name) };
    if let Ok(s) = name_cstr.to_str() {
        if s.eq_ignore_ascii_case("HOME") {
            unsafe { c_init_homedir() };
        } else if s.eq_ignore_ascii_case("VIM") && unsafe { didset_vim } {
            unsafe { didset_vim = false };
        } else if s.eq_ignore_ascii_case("VIMRUNTIME") && unsafe { didset_vimruntime } {
            unsafe { didset_vimruntime = false };
        }
    }
}

/// Like `home_replace`, but stores result in allocated memory.
///
/// # Safety
///
/// `buf` may be NULL. `src` may be NULL.
#[export_name = "home_replace_save"]
pub unsafe extern "C" fn rs_home_replace_save(
    buf: *const c_void,
    src: *const c_char,
) -> *mut c_char {
    let mut len: usize = 3; // space for "~/" and trailing NUL
    if !src.is_null() {
        let src_cstr = unsafe { CStr::from_ptr(src) };
        len += src_cstr.to_bytes().len();
    }
    let dst = unsafe { nvim_xmalloc(len) };
    unsafe {
        c_home_replace(buf, src, dst, len, true);
    }
    dst
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_get_pid() {
        let pid = rs_os_get_pid();
        assert!(pid > 0);
    }

    #[test]
    fn test_env_exists_nonempty() {
        // PATH should exist and be non-empty on all systems
        let name = CString::new("PATH").unwrap();
        let exists = unsafe { rs_os_env_exists(name.as_ptr(), true) };
        assert!(exists);

        // Non-existent variable
        let name = CString::new("NVIM_RS_TEST_NONEXISTENT_VAR_12345").unwrap();
        let exists = unsafe { rs_os_env_exists(name.as_ptr(), false) };
        assert!(!exists);
    }

    #[test]
    fn test_setenv_unsetenv() {
        let name = CString::new("NVIM_RS_TEST_VAR_PHASE1").unwrap();
        let value = CString::new("test_value").unwrap();

        // Set the variable
        let result = unsafe { rs_os_setenv(name.as_ptr(), value.as_ptr(), 1) };
        assert_eq!(result, 0);

        // Check it exists
        let exists = unsafe { rs_os_env_exists(name.as_ptr(), false) };
        assert!(exists);

        // Unset it
        let result = unsafe { rs_os_unsetenv(name.as_ptr()) };
        assert_eq!(result, 0);

        // Check it's gone
        let exists = unsafe { rs_os_env_exists(name.as_ptr(), false) };
        assert!(!exists);
    }

    // Note: test_getenvname_at_index is disabled because rs_os_getenvname_at_index
    // calls nvim's xstrdup allocator which requires linking with nvim.
    // The function will be tested via nvim's integration tests.

    #[test]
    fn test_fullenv_size() {
        let size = rs_os_get_fullenv_size();
        // There should always be at least one env var (e.g., PATH)
        assert!(size > 0);
    }

    // Note: test_getenv is disabled because rs_os_getenv now uses
    // nvim's xmallocz allocator which requires linking with nvim.
    // The function will be tested via nvim's integration tests.
}
