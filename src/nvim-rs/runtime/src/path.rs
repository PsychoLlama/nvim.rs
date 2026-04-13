//! Path manipulation and expansion
//!
//! This module handles path manipulation for runtime file searching,
//! runtimepath default computation, and autoload script resolution.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use nvim_memory::xstrdup;

use crate::constants::{IOSIZE, MAXPATHL};

// =============================================================================
// Constants
// =============================================================================

/// Path separator character (Unix: '/')
const PATHSEP: u8 = b'/';

/// Autoload character '#' used in function/variable names
const AUTOLOAD_CHAR: u8 = b'#';

/// XDG variable types (must match kXDG* enum in stdpaths_defs.h)
mod xdg {
    pub const NONE: i32 = -1;
    pub const CONFIG_HOME: i32 = 0;
    pub const DATA_HOME: i32 = 1;
    // CACHE_HOME = 2
    // STATE_HOME = 3
    // RUNTIME_DIR = 4
    pub const CONFIG_DIRS: i32 = 5;
    pub const DATA_DIRS: i32 = 6;
}

/// sizeof("site") - 1
const SITE_SIZE: usize = 4;

/// sizeof("after") - 1
const AFTER_SIZE: usize = 5;

/// OK return value from C
const OK: c_int = 1;

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);

    // Phase 3 accessors from runtime_ffi.c
    fn nvim_rt_vim_env_iter(
        val: *const c_char,
        iter: *const c_void,
        dir: *mut *const c_char,
        len: *mut usize,
    ) -> *const c_void;
    fn nvim_rt_vim_env_iter_rev(
        val: *const c_char,
        iter: *const c_void,
        dir: *mut *const c_char,
        len: *mut usize,
    ) -> *const c_void;
    #[link_name = "after_pathsep"]
    fn nvim_rt_after_pathsep(b: *const c_char, s: *const c_char) -> bool;
    #[link_name = "memcnt"]
    fn nvim_rt_memcnt(s: *const c_void, c: c_int, n: usize) -> usize;
    fn nvim_rt_get_appname() -> *const c_char;
    fn nvim_rt_stdpaths_get_xdg_var(xdg_type: c_int) -> *mut c_char;
    #[link_name = "vim_getenv"]
    fn nvim_rt_vim_getenv(name: *const c_char) -> *mut c_char;
    #[link_name = "os_isdir"]
    fn nvim_rt_os_isdir(name: *const c_char) -> bool;
    fn nvim_rt_get_default_lib_dir() -> *const c_char;
    fn nvim_rt_vim_get_prefix_from_exepath(buf: *mut c_char);
    fn nvim_rt_append_path(path: *mut c_char, to_append: *const c_char, max_len: usize) -> c_int;
    #[link_name = "vim_ispathsep"]
    fn nvim_rt_vim_ispathsep(c: c_int) -> bool;
    fn xmemcpyz(dst: *mut c_void, src: *const c_void, len: usize);

    // Phase 3 accessors from runtime.c (ga_loaded)
    fn nvim_rt_ga_loaded_len() -> c_int;
    fn nvim_rt_ga_loaded_get(idx: c_int) -> *const c_char;
    fn nvim_rt_ga_loaded_append(name: *mut c_char);
    fn nvim_rt_do_in_runtimepath_source(
        name: *const c_char,
        flags: c_int,
        cookie: *mut c_void,
    ) -> c_int;

    fn nvim_rt_get_iobuff() -> *mut c_char;
}

// =============================================================================
// Path Component Checking
// =============================================================================

/// Check if a path ends with a path separator.
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_ends_with_sep(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    // Find the end of the string
    let mut p = path;
    while !(*p == 0) {
        p = p.add(1);
    }

    // Check if previous char is a separator
    if p > path {
        let last = *p.sub(1) as u8;
        return last == b'/' || last == b'\\';
    }

    false
}

/// Check if a character is a path separator.
pub fn rs_is_path_sep(c: c_int) -> bool {
    let c = c as u8;
    c == b'/' || c == b'\\'
}

/// Check if a path is absolute.
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_is_absolute(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    let first = *path as u8;

    // Unix absolute path
    if first == b'/' {
        return true;
    }

    // Windows absolute path (drive letter)
    if first.is_ascii_alphabetic() {
        let second = *path.add(1) as u8;
        if second == b':' {
            return true;
        }
    }

    // Windows UNC path
    if first == b'\\' {
        let second = *path.add(1) as u8;
        if second == b'\\' {
            return true;
        }
    }

    false
}

/// Get the length of a path string.
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_strlen(path: *const c_char) -> usize {
    if path.is_null() {
        return 0;
    }

    let mut len = 0usize;
    let mut p = path;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }
    len
}

// =============================================================================
// Path Pattern Matching
// =============================================================================

/// Check if a path contains a wildcard character.
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_has_wildcard(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    let mut p = path;
    while *p != 0 {
        let c = *p as u8;
        if c == b'*' || c == b'?' || c == b'[' {
            return true;
        }
        p = p.add(1);
    }

    false
}

/// Check if a pattern matches the "after" directory.
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_is_after_dir(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }

    // Check for "after" at start or after separator
    static AFTER: &[u8] = b"after";

    let mut p = path;
    let mut i = 0;

    while i < AFTER.len() {
        if *p == 0 || (*p as u8) != AFTER[i] {
            return false;
        }
        p = p.add(1);
        i += 1;
    }

    // Must be followed by separator or end
    let c = *p as u8;
    c == 0 || c == b'/' || c == b'\\'
}

// =============================================================================
// Path Extension Checking
// =============================================================================

/// Check if a path has a Vim script extension (.vim).
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_has_vim_ext(path: *const c_char) -> bool {
    rs_path_has_ext(path, b".vim\0".as_ptr().cast())
}

/// Check if a path has a Lua script extension (.lua).
///
/// # Safety
///
/// `path` must be null or a valid null-terminated C string.
pub unsafe fn rs_path_has_lua_ext(path: *const c_char) -> bool {
    rs_path_has_ext(path, b".lua\0".as_ptr().cast())
}

/// Check if a path ends with a given extension (case-insensitive).
///
/// # Safety
///
/// Both `path` and `ext` must be null or valid null-terminated C strings.
pub unsafe fn rs_path_has_ext(path: *const c_char, ext: *const c_char) -> bool {
    if path.is_null() || ext.is_null() {
        return false;
    }

    let path_len = rs_path_strlen(path);
    let ext_len = rs_path_strlen(ext);

    if ext_len == 0 || path_len < ext_len {
        return false;
    }

    // Compare from the end
    let path_suffix = path.add(path_len - ext_len);
    let mut p = path_suffix;
    let mut e = ext;

    while *e != 0 {
        let pc = (*p as u8).to_ascii_lowercase();
        let ec = (*e as u8).to_ascii_lowercase();
        if pc != ec {
            return false;
        }
        p = p.add(1);
        e = e.add(1);
    }

    true
}

// =============================================================================
// Phase 3: Internal helper functions
// =============================================================================

/// Copy string with escaped commas. Returns pointer past end of written data.
///
/// Equivalent to C's `strcpy_comma_escaped(dest, src, len)`.
///
/// # Safety
///
/// `dest` must have enough space for `len + number_of_commas_in_src` bytes.
/// `src` must point to at least `len` valid bytes.
unsafe fn strcpy_comma_escaped(dest: *mut c_char, src: *const c_char, len: usize) -> *mut c_char {
    let mut shift: usize = 0;
    for i in 0..len {
        if *src.add(i) as u8 == b',' {
            *dest.add(i + shift) = b'\\' as c_char;
            shift += 1;
        }
        *dest.add(i + shift) = *src.add(i);
    }
    dest.add(len + shift)
}

/// Compute length of a ENV_SEPCHAR-separated value, doubled and with suffixes.
///
/// Each item appears twice with `common_suf_len` appended, and once with
/// `single_suf_len` appended, plus commas and path separators as needed.
/// Commas inside values are escaped (counted).
///
/// # Safety
///
/// `val` must be null or a valid null-terminated C string.
unsafe fn compute_double_env_sep_len(
    val: *const c_char,
    common_suf_len: usize,
    single_suf_len: usize,
) -> usize {
    if val.is_null() || *val == 0 {
        return 0;
    }
    let mut ret: usize = 0;
    let mut iter: *const c_void = ptr::null();
    loop {
        let mut dir_len: usize = 0;
        let mut dir: *const c_char = ptr::null();
        iter = nvim_rt_vim_env_iter(val, iter, &raw mut dir, &raw mut dir_len);
        if !dir.is_null() && dir_len > 0 {
            let comma_count = nvim_rt_memcnt(dir.cast(), c_int::from(b','), dir_len);
            let needs_sep = !nvim_rt_after_pathsep(dir, dir.add(dir_len));
            ret += ((dir_len + comma_count + common_suf_len + usize::from(needs_sep)) * 2)
                + single_suf_len;
        }
        if iter.is_null() {
            break;
        }
    }
    ret
}

/// Add ENV_SEPCHAR-separated dirs to a comma-separated buffer.
///
/// To each item, PATHSEP + appname is appended, then optionally suf1 and suf2.
///
/// # Safety
///
/// `dest` must have enough space. `val` must be null-terminated or null.
unsafe fn add_env_sep_dirs(
    mut dest: *mut c_char,
    val: *const c_char,
    suf1: *const c_char,
    len1: usize,
    suf2: *const c_char,
    len2: usize,
    forward: bool,
) -> *mut c_char {
    if val.is_null() || *val == 0 {
        return dest;
    }
    let mut iter: *const c_void = ptr::null();
    let appname = nvim_rt_get_appname();
    let appname_len = libc::strlen(appname);
    loop {
        let mut dir_len: usize = 0;
        let mut dir: *const c_char = ptr::null();
        if forward {
            iter = nvim_rt_vim_env_iter(val, iter, &raw mut dir, &raw mut dir_len);
        } else {
            iter = nvim_rt_vim_env_iter_rev(val, iter, &raw mut dir, &raw mut dir_len);
        }
        if !dir.is_null() && dir_len > 0 {
            dest = strcpy_comma_escaped(dest, dir, dir_len);
            if !nvim_rt_after_pathsep(dest.sub(1), dest) {
                *dest = PATHSEP as c_char;
                dest = dest.add(1);
            }
            ptr::copy_nonoverlapping(appname, dest, appname_len);
            dest = dest.add(appname_len);
            if !suf1.is_null() {
                *dest = PATHSEP as c_char;
                dest = dest.add(1);
                ptr::copy_nonoverlapping(suf1, dest, len1);
                dest = dest.add(len1);
                if !suf2.is_null() {
                    *dest = PATHSEP as c_char;
                    dest = dest.add(1);
                    ptr::copy_nonoverlapping(suf2, dest, len2);
                    dest = dest.add(len2);
                }
            }
            *dest = b',' as c_char;
            dest = dest.add(1);
        }
        if iter.is_null() {
            break;
        }
    }
    dest
}

/// Add a single directory to a comma-separated buffer with XDG handling.
///
/// If `xdg_type` is `kXDGDataHome` or `kXDGConfigHome`, appname is appended
/// after the directory. On Windows, "-data" is also appended for data/state dirs.
///
/// # Safety
///
/// `dest` must have enough space. `dir` must point to `dir_len` valid bytes.
#[allow(clippy::too_many_arguments)]
unsafe fn add_dir(
    mut dest: *mut c_char,
    dir: *const c_char,
    dir_len: usize,
    xdg_type: i32,
    suf1: *const c_char,
    len1: usize,
    suf2: *const c_char,
    len2: usize,
) -> *mut c_char {
    if dir.is_null() || dir_len == 0 {
        return dest;
    }
    dest = strcpy_comma_escaped(dest, dir, dir_len);
    let append_nvim = xdg_type == xdg::DATA_HOME || xdg_type == xdg::CONFIG_HOME;
    if append_nvim {
        if !nvim_rt_after_pathsep(dest.sub(1), dest) {
            *dest = PATHSEP as c_char;
            dest = dest.add(1);
        }
        let appname = nvim_rt_get_appname();
        let appname_len = libc::strlen(appname);
        let iosize = IOSIZE;
        assert!(appname_len < iosize - 5); // sizeof("-data")
        let iobuff = nvim_rt_get_iobuff();
        xmemcpyz(iobuff.cast(), appname.cast(), appname_len);
        // On MSWIN, "-data" would be appended for kXDGDataHome/kXDGStateHome.
        // Not applicable on Linux.
        xmemcpyz(dest.cast(), iobuff.cast(), appname_len);
        dest = dest.add(appname_len);
        if !suf1.is_null() {
            *dest = PATHSEP as c_char;
            dest = dest.add(1);
            ptr::copy_nonoverlapping(suf1, dest, len1);
            dest = dest.add(len1);
            if !suf2.is_null() {
                *dest = PATHSEP as c_char;
                dest = dest.add(1);
                ptr::copy_nonoverlapping(suf2, dest, len2);
                dest = dest.add(len2);
            }
        }
    }
    *dest = b',' as c_char;
    dest = dest.add(1);
    dest
}

// =============================================================================
// Phase 3: Exported functions
// =============================================================================

/// Check if path ends with "after" directory component.
///
/// Equivalent to C's `path_is_after(buf, buflen)`.
#[export_name = "path_is_after"]
pub unsafe extern "C" fn rs_path_is_after(buf: *const c_char, buflen: usize) -> bool {
    // buflen >= 5
    //   && (!(buflen >= 6) || vim_ispathsep(buf[buflen - 6]))
    //   && strcmp(buf + buflen - 5, "after") == 0
    if buflen < AFTER_SIZE {
        return false;
    }
    if buflen > AFTER_SIZE
        && !nvim_rt_vim_ispathsep(c_int::from(*buf.add(buflen - AFTER_SIZE - 1) as u8))
    {
        return false;
    }
    let suffix = buf.add(buflen - AFTER_SIZE);
    libc::strncmp(suffix, b"after\0".as_ptr().cast(), AFTER_SIZE) == 0
}

/// Find library dir relative to binary.
///
/// Returns allocated string or NULL.
#[export_name = "get_lib_dir"]
pub unsafe extern "C" fn rs_get_lib_dir() -> *mut c_char {
    let default_lib = nvim_rt_get_default_lib_dir();
    if !default_lib.is_null() && libc::strlen(default_lib) != 0 && nvim_rt_os_isdir(default_lib) {
        return xstrdup(default_lib);
    }

    let maxpathl = MAXPATHL;
    let exe_name = xmalloc(maxpathl).cast::<c_char>();
    nvim_rt_vim_get_prefix_from_exepath(exe_name);
    if nvim_rt_append_path(exe_name, b"lib/nvim\0".as_ptr().cast(), maxpathl) == OK {
        // exe_name is already allocated by xmalloc, reuse it
        return exe_name;
    }
    xfree(exe_name.cast());
    ptr::null_mut()
}

/// Build the default &runtimepath value.
///
/// Returns allocated string or NULL.
#[export_name = "runtimepath_default"]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_runtimepath_default(clean_arg: bool) -> *mut c_char {
    let mut rtp_size: usize = 0;

    let data_home: *mut c_char = if clean_arg {
        ptr::null_mut()
    } else {
        nvim_rt_stdpaths_get_xdg_var(xdg::DATA_HOME)
    };
    let config_home: *mut c_char = if clean_arg {
        ptr::null_mut()
    } else {
        nvim_rt_stdpaths_get_xdg_var(xdg::CONFIG_HOME)
    };
    let vimruntime: *mut c_char = nvim_rt_vim_getenv(b"VIMRUNTIME\0".as_ptr().cast());
    let libdir: *mut c_char = rs_get_lib_dir();
    let data_dirs: *mut c_char = nvim_rt_stdpaths_get_xdg_var(xdg::DATA_DIRS);
    let config_dirs: *mut c_char = nvim_rt_stdpaths_get_xdg_var(xdg::CONFIG_DIRS);

    let mut data_len: usize = 0;
    let mut config_len: usize = 0;
    let mut vimruntime_len: usize = 0;
    let mut libdir_len: usize = 0;
    let appname_len = libc::strlen(nvim_rt_get_appname());

    if !data_home.is_null() {
        data_len = libc::strlen(data_home);
        let nvim_data_size = appname_len;
        // On MSWIN: nvim_data_size += sizeof("-data") - 1; (not applicable on Linux)
        if data_len != 0 {
            let comma_count = nvim_rt_memcnt(data_home.cast(), c_int::from(b','), data_len);
            let needs_sep = !nvim_rt_after_pathsep(data_home, data_home.add(data_len));
            rtp_size += ((data_len
                + comma_count
                + nvim_data_size
                + 1
                + SITE_SIZE
                + 1
                + usize::from(needs_sep))
                * 2)
                + AFTER_SIZE
                + 1;
        }
    }
    if !config_home.is_null() {
        config_len = libc::strlen(config_home);
        if config_len != 0 {
            let comma_count = nvim_rt_memcnt(config_home.cast(), c_int::from(b','), config_len);
            let needs_sep = !nvim_rt_after_pathsep(config_home, config_home.add(config_len));
            rtp_size += ((config_len + comma_count + appname_len + 1 + usize::from(needs_sep)) * 2)
                + AFTER_SIZE
                + 1;
        }
    }
    if !vimruntime.is_null() {
        vimruntime_len = libc::strlen(vimruntime);
        if vimruntime_len != 0 {
            rtp_size += vimruntime_len
                + nvim_rt_memcnt(vimruntime.cast(), c_int::from(b','), vimruntime_len)
                + 1;
        }
    }
    if !libdir.is_null() {
        libdir_len = libc::strlen(libdir);
        if libdir_len != 0 {
            rtp_size +=
                libdir_len + nvim_rt_memcnt(libdir.cast(), c_int::from(b','), libdir_len) + 1;
        }
    }
    rtp_size +=
        compute_double_env_sep_len(data_dirs, appname_len + 1 + SITE_SIZE + 1, AFTER_SIZE + 1);
    rtp_size += compute_double_env_sep_len(config_dirs, appname_len + 1, AFTER_SIZE + 1);

    let rtp: *mut c_char;
    if rtp_size == 0 {
        rtp = ptr::null_mut();
    } else {
        rtp = xmalloc(rtp_size).cast();
        let mut cur = rtp;

        let site = b"site\0".as_ptr().cast();
        let after = b"after\0".as_ptr().cast();

        cur = add_dir(
            cur,
            config_home,
            config_len,
            xdg::CONFIG_HOME,
            ptr::null(),
            0,
            ptr::null(),
            0,
        );
        cur = add_env_sep_dirs(cur, config_dirs, ptr::null(), 0, ptr::null(), 0, true);
        cur = add_dir(
            cur,
            data_home,
            data_len,
            xdg::DATA_HOME,
            site,
            SITE_SIZE,
            ptr::null(),
            0,
        );
        cur = add_env_sep_dirs(cur, data_dirs, site, SITE_SIZE, ptr::null(), 0, true);
        cur = add_dir(
            cur,
            vimruntime,
            vimruntime_len,
            xdg::NONE,
            ptr::null(),
            0,
            ptr::null(),
            0,
        );
        cur = add_dir(
            cur,
            libdir,
            libdir_len,
            xdg::NONE,
            ptr::null(),
            0,
            ptr::null(),
            0,
        );
        cur = add_env_sep_dirs(cur, data_dirs, site, SITE_SIZE, after, AFTER_SIZE, false);
        cur = add_dir(
            cur,
            data_home,
            data_len,
            xdg::DATA_HOME,
            site,
            SITE_SIZE,
            after,
            AFTER_SIZE,
        );
        cur = add_env_sep_dirs(cur, config_dirs, after, AFTER_SIZE, ptr::null(), 0, false);
        cur = add_dir(
            cur,
            config_home,
            config_len,
            xdg::CONFIG_HOME,
            after,
            AFTER_SIZE,
            ptr::null(),
            0,
        );

        // Strip trailing comma
        *cur.sub(1) = 0;
        debug_assert_eq!(cur.offset_from(rtp) as usize, rtp_size);
    }

    // Free temporaries
    xfree(data_dirs.cast());
    xfree(config_dirs.cast());
    xfree(data_home.cast());
    xfree(config_home.cast());
    xfree(vimruntime.cast());
    xfree(libdir.cast());

    rtp
}

/// Convert a function/variable name to an autoload script path.
///
/// Replaces '#' with '/' and appends ".vim" after the last '#'.
/// The name must contain at least one AUTOLOAD_CHAR ('#').
///
/// Returns allocated string: "autoload/<name_with_slashes>.vim"
#[export_name = "autoload_name"]
pub unsafe extern "C" fn rs_autoload_name(name: *const c_char, name_len: usize) -> *mut c_char {
    // Allocate: "autoload/" + name + ".vim" + NUL
    // sizeof("autoload/.vim") = 13 + 1(NUL) = 14, but C uses sizeof which includes NUL
    let prefix = b"autoload/";
    let suffix = b".vim";
    let alloc_size = prefix.len() + name_len + suffix.len() + 1;
    let scriptname = xmalloc(alloc_size).cast::<c_char>();

    // Copy "autoload/"
    ptr::copy_nonoverlapping(prefix.as_ptr().cast(), scriptname, prefix.len());

    // Copy name after "autoload/"
    ptr::copy_nonoverlapping(name, scriptname.add(prefix.len()), name_len);

    // Replace '#' with '/' and track last '#' position
    let mut auchar_idx: usize = 0;
    for i in prefix.len()..(prefix.len() + name_len) {
        if *scriptname.add(i) as u8 == AUTOLOAD_CHAR {
            *scriptname.add(i) = b'/' as c_char;
            auchar_idx = i;
        }
    }

    // Append ".vim" at the last '#' position (overwriting last slash)
    ptr::copy_nonoverlapping(
        suffix.as_ptr().cast(),
        scriptname.add(auchar_idx),
        suffix.len() + 1, // includes NUL
    );

    scriptname
}

/// If name has a package name (contains '#'), try autoloading the script for it.
///
/// Returns true if a package was loaded.
#[export_name = "script_autoload"]
pub unsafe extern "C" fn rs_script_autoload(
    name: *const c_char,
    name_len: usize,
    reload: bool,
) -> bool {
    // If there is no '#' after name[0] there is no package name.
    let p = libc::memchr(name.cast(), c_int::from(AUTOLOAD_CHAR), name_len);
    if p.is_null() || p == name as *mut c_void {
        return false;
    }

    let mut ret = false;
    let tofree = rs_autoload_name(name, name_len);
    let scriptname = tofree;

    // Find the name in the list of previously loaded package names.
    // Skip "autoload/" prefix (9 bytes), it's always the same.
    let mut i: c_int = 0;
    let ga_len = nvim_rt_ga_loaded_len();
    while i < ga_len {
        let loaded = nvim_rt_ga_loaded_get(i);
        if libc::strcmp(loaded.add(9), scriptname.add(9)) == 0 {
            break;
        }
        i += 1;
    }

    if !reload && i < ga_len {
        // Was loaded already.
        ret = false;
    } else {
        // Remember the name if it wasn't loaded already.
        if i == ga_len {
            // Duplicate scriptname since ga_loaded takes ownership
            let owned = xstrdup(scriptname);
            nvim_rt_ga_loaded_append(owned);
        }

        // Try loading the package from $VIMRUNTIME/autoload/<name>.vim
        // Use "ret_sid" to avoid loading the same script again.
        let mut ret_sid: c_int = 0;
        if nvim_rt_do_in_runtimepath_source(
            scriptname,
            super::dip::START,
            (&raw mut ret_sid).cast(),
        ) == OK
        {
            ret = true;
        }
    }

    xfree(tofree.cast());
    ret
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_is_path_sep() {
        assert!(rs_is_path_sep(b'/' as c_int));
        assert!(rs_is_path_sep(b'\\' as c_int));
        assert!(!rs_is_path_sep(b'a' as c_int));
        assert!(!rs_is_path_sep(0));
    }

    #[test]
    fn test_path_ends_with_sep() {
        unsafe {
            let path1 = CString::new("/home/user/").unwrap();
            assert!(rs_path_ends_with_sep(path1.as_ptr()));

            let path2 = CString::new("/home/user").unwrap();
            assert!(!rs_path_ends_with_sep(path2.as_ptr()));

            let path3 = CString::new("C:\\Users\\").unwrap();
            assert!(rs_path_ends_with_sep(path3.as_ptr()));

            assert!(!rs_path_ends_with_sep(std::ptr::null()));
        }
    }

    #[test]
    fn test_path_is_absolute() {
        unsafe {
            let unix = CString::new("/home/user").unwrap();
            assert!(rs_path_is_absolute(unix.as_ptr()));

            let win = CString::new("C:\\Users").unwrap();
            assert!(rs_path_is_absolute(win.as_ptr()));

            let unc = CString::new("\\\\server\\share").unwrap();
            assert!(rs_path_is_absolute(unc.as_ptr()));

            let relative = CString::new("home/user").unwrap();
            assert!(!rs_path_is_absolute(relative.as_ptr()));

            assert!(!rs_path_is_absolute(std::ptr::null()));
        }
    }

    #[test]
    fn test_path_has_wildcard() {
        unsafe {
            let wild1 = CString::new("*.vim").unwrap();
            assert!(rs_path_has_wildcard(wild1.as_ptr()));

            let wild2 = CString::new("file?.txt").unwrap();
            assert!(rs_path_has_wildcard(wild2.as_ptr()));

            let wild3 = CString::new("file[0-9].txt").unwrap();
            assert!(rs_path_has_wildcard(wild3.as_ptr()));

            let plain = CString::new("file.txt").unwrap();
            assert!(!rs_path_has_wildcard(plain.as_ptr()));
        }
    }

    #[test]
    fn test_path_has_ext() {
        unsafe {
            let vim = CString::new("file.vim").unwrap();
            assert!(rs_path_has_vim_ext(vim.as_ptr()));
            assert!(!rs_path_has_lua_ext(vim.as_ptr()));

            let lua = CString::new("file.lua").unwrap();
            assert!(rs_path_has_lua_ext(lua.as_ptr()));
            assert!(!rs_path_has_vim_ext(lua.as_ptr()));

            // Case insensitive
            let vim_upper = CString::new("file.VIM").unwrap();
            assert!(rs_path_has_vim_ext(vim_upper.as_ptr()));
        }
    }

    #[test]
    fn test_path_is_after_dir() {
        unsafe {
            let after = CString::new("after").unwrap();
            assert!(rs_path_is_after_dir(after.as_ptr()));

            let after_slash = CString::new("after/").unwrap();
            assert!(rs_path_is_after_dir(after_slash.as_ptr()));

            let not_after = CString::new("before").unwrap();
            assert!(!rs_path_is_after_dir(not_after.as_ptr()));

            let afterx = CString::new("afterx").unwrap();
            assert!(!rs_path_is_after_dir(afterx.as_ptr()));
        }
    }

    #[test]
    fn test_strcpy_comma_escaped() {
        unsafe {
            let mut buf = [0u8; 32];
            let src = CString::new("hello,world").unwrap();
            let end = strcpy_comma_escaped(buf.as_mut_ptr().cast(), src.as_ptr(), 11);
            let written = end.offset_from(buf.as_ptr().cast()) as usize;
            // "hello" + "\," + "world" = 12 chars
            assert_eq!(written, 12);
            assert_eq!(&buf[..12], b"hello\\,world");
        }
    }

    #[test]
    fn test_autoload_name_pure() {
        // Test the autoload name logic without FFI by checking the algorithm
        let name = b"foo#bar#baz";
        let prefix = b"autoload/";
        let suffix = b".vim";
        let mut scriptname = Vec::new();
        scriptname.extend_from_slice(prefix);
        scriptname.extend_from_slice(name);

        let mut auchar_idx = 0;
        for (i, byte) in scriptname
            .iter_mut()
            .enumerate()
            .skip(prefix.len())
            .take(name.len())
        {
            if *byte == b'#' {
                *byte = b'/';
                auchar_idx = i;
            }
        }
        // Replace from auchar_idx with ".vim"
        scriptname.truncate(auchar_idx);
        scriptname.extend_from_slice(suffix);
        assert_eq!(scriptname, b"autoload/foo/bar.vim");
    }
}
