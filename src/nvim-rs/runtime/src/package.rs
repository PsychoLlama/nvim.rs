//! Package/plugin management
//!
//! This module handles loading plugins from 'packpath' directories.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::constants::MAXPATHL;
use crate::dip;
use crate::do_in_path::rs_do_in_path;
use crate::globals;
use nvim_ex_eval::ExargT;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Wildcard expansion (already declared in pathsearch.rs, repeated here for use)
    fn gen_expand_wildcards(
        num_pat: c_int,
        pat: *mut *mut c_char,
        num_file: *mut c_int,
        file: *mut *mut *mut c_char,
        flags: c_int,
    ) -> c_int;
    fn FreeWild(count: c_int, files: *mut *mut c_char);

    // path_fnamecmp: compare two paths (platform-correct; no add_pack_dir_to_rtp here, it's Rust now)
    fn path_fnamecmp(a: *const c_char, b: *const c_char) -> c_int;

    // copy_option_part: advance pointer through comma-separated value
    fn copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    );

    // Phase 3: add_pack_dir_to_rtp helpers
    #[link_name = "utfc_ptr2len"]
    fn nvim_rt_utfc_ptr2len(p: *const c_char) -> c_int;
    #[link_name = "vim_ispathsep_nocolon"]
    fn nvim_rt_vim_ispathsep_nocolon(c: c_int) -> bool;
    #[link_name = "vim_ispathsep"]
    fn nvim_rt_vim_ispathsep(c: c_int) -> bool;
    #[link_name = "os_isdir"]
    fn nvim_rt_os_isdir(name: *const c_char) -> bool;
    #[link_name = "add_pathsep"]
    fn nvim_rt_add_pathsep(p: *mut c_char);
    #[link_name = "path_fnamencmp"]
    fn nvim_rt_path_fnamencmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    #[link_name = "concat_fnames"]
    fn nvim_rt_concat_fnames(
        fname1: *const c_char,
        fname2: *const c_char,
        sep: bool,
    ) -> *mut c_char;
    fn try_malloc(n: usize) -> *mut c_void;
    fn nvim_rt_set_runtimepath(new_rtp: *const c_char);
    #[link_name = "get_past_head"]
    fn nvim_rt_get_past_head(path: *const c_char) -> *mut c_char;
    #[link_name = "fix_fname"]
    fn nvim_rt_fix_fname(fname: *const c_char) -> *mut c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

    // Global options
    static p_pp: *mut c_char; // packpath
    static p_rtp: *mut c_char; // runtimepath

    // Memory management
    fn xmallocz(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, maxlen: usize) -> usize;
    fn xstrlcat(dst: *mut c_char, src: *const c_char, maxlen: usize) -> usize;

    // Package management accessors (in runtime_ffi.c)
    // (nvim_rt_pkg_exarg_get_forceit replaced by ExargT.forceit direct access)
    #[link_name = "fix_fname"]
    fn nvim_rt_pkg_fix_fname(fname: *const c_char) -> *mut c_char;
    fn nvim_rt_pkg_snprintf(
        buf: *mut c_char,
        len: usize,
        fmt: *const c_char,
        arg: *const c_char,
    ) -> c_int;
    fn nvim_rt_pkg_eval_to_number(expr: *mut c_char) -> i64;
    #[link_name = "do_cmdline_cmd"]
    fn nvim_rt_pkg_do_cmdline_cmd(cmd: *const c_char);
    // nvim_rt_pkg_time_msg: implemented inline in Rust
    #[link_name = "time_msg"]
    fn nvim_rt_pkg_time_msg_inner(msg: *const c_char, start: *const c_void);

    // (nvim_rt_exarg_get_arg replaced by ExargT.arg direct access)
}

extern "C" {
    /// Timing log file pointer (FILE *time_fd). Non-null when timing is active.
    #[link_name = "time_fd"]
    static nvim_rt_pkg_time_fd: *mut c_void;
}

/// Call time_msg if time_fd is set (replaces the TIME_MSG macro / nvim_rt_pkg_time_msg).
///
/// # Safety
/// `msg` must be a valid NUL-terminated C string.
#[inline]
unsafe fn pkg_time_msg(msg: *const c_char) {
    if !nvim_rt_pkg_time_fd.is_null() {
        nvim_rt_pkg_time_msg_inner(msg, std::ptr::null());
    }
}

use crate::pathsearch::{
    rs_gen_expand_wildcards_and_cb, rs_source_callback_vim_lua, rs_source_in_path_vim_lua,
    rs_source_runtime_vim_lua,
};

// =============================================================================
// Constants
// =============================================================================

const FAIL: c_int = 0;
const OK: c_int = 1;

/// EW_DIR - expand directory names
const EW_DIR: c_int = 0x01;
/// EW_FILE - expand file names
const EW_FILE: c_int = 0x02;

// =============================================================================
// Package Loading State
// =============================================================================

/// Package load status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageStatus {
    /// Package not yet loaded
    NotLoaded = 0,
    /// Package currently loading
    Loading = 1,
    /// Package loaded successfully
    Loaded = 2,
    /// Package load failed
    Failed = 3,
}

impl PackageStatus {
    /// Convert from integer
    pub const fn from_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::NotLoaded),
            1 => Some(Self::Loading),
            2 => Some(Self::Loaded),
            3 => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Check if a package status indicates it can be loaded.
pub fn rs_package_can_load(status: c_int) -> bool {
    status == PackageStatus::NotLoaded as c_int
}

/// Check if a package status indicates it's already loaded.
pub fn rs_package_is_loaded(status: c_int) -> bool {
    status == PackageStatus::Loaded as c_int
}

// =============================================================================
// Package Type
// =============================================================================

/// Package type (start vs opt)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageType {
    /// Loaded at startup (pack/*/start/*)
    Start = 0,
    /// Loaded on demand (pack/*/opt/*)
    Opt = 1,
}

impl PackageType {
    /// Get the corresponding DIP flag
    pub const fn to_dip_flag(self) -> c_int {
        match self {
            Self::Start => dip::START,
            Self::Opt => dip::OPT,
        }
    }
}

/// Get DIP flag for start packages.
pub fn rs_package_start_flag() -> c_int {
    PackageType::Start.to_dip_flag()
}

/// Get DIP flag for opt packages.
pub fn rs_package_opt_flag() -> c_int {
    PackageType::Opt.to_dip_flag()
}

// =============================================================================
// Package Name Handling
// =============================================================================

/// Check if a package name is valid.
///
/// Package names must not be empty, contain path separators, or contain wildcards.
///
/// # Safety
///
/// `name` must be null or a valid null-terminated C string.
pub unsafe fn rs_package_name_valid(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }

    let first = *name as u8;
    if first == 0 {
        return false; // Empty name
    }

    let mut p = name;
    while *p != 0 {
        let c = *p as u8;
        // Reject path separators
        if c == b'/' || c == b'\\' {
            return false;
        }
        // Reject wildcards
        if c == b'*' || c == b'?' || c == b'[' {
            return false;
        }
        p = p.add(1);
    }

    true
}

// =============================================================================
// Plugin Directory Names
// =============================================================================

/// Well-known plugin subdirectories that should be searched.
pub const PLUGIN_DIRS: &[&[u8]] = &[
    b"plugin\0",
    b"autoload\0",
    b"colors\0",
    b"compiler\0",
    b"doc\0",
    b"ftdetect\0",
    b"ftplugin\0",
    b"indent\0",
    b"keymap\0",
    b"lang\0",
    b"syntax\0",
];

/// Number of plugin directories
pub const PLUGIN_DIR_COUNT: usize = PLUGIN_DIRS.len();

/// Get a plugin directory name by index.
///
/// Returns null if index is out of bounds.
pub fn rs_get_plugin_dir(idx: usize) -> *const c_char {
    if idx >= PLUGIN_DIR_COUNT {
        return std::ptr::null();
    }
    PLUGIN_DIRS[idx].as_ptr().cast::<c_char>()
}

/// Get the count of plugin directories.
pub fn rs_plugin_dir_count() -> usize {
    PLUGIN_DIR_COUNT
}

// =============================================================================
// Phase 3: add_pack_dir_to_rtp (migrated from C)
// =============================================================================

/// Add the package directory `fname` to 'runtimepath'.
///
/// `is_pack`: true if this is a "pack/*/start/*/" style package (uses pack_has_entries
/// to check for "after" dir), false otherwise (uses os_isdir).
///
/// # Safety
///
/// `fname` must be a valid null-terminated C string.
#[allow(clippy::too_many_lines)]
#[export_name = "add_pack_dir_to_rtp"]
pub unsafe extern "C" fn rs_add_pack_dir_to_rtp(fname: *mut c_char, is_pack: bool) -> c_int {
    // Find the path separator positions: walk from get_past_head
    // to find the 4 deepest separators.
    let p_start = nvim_rt_get_past_head(fname);
    let mut p1 = p_start;
    let mut p2 = p_start;
    let mut p3 = p_start;
    let mut p4 = p_start;

    let mut p = p_start;
    while *p != 0 {
        if nvim_rt_vim_ispathsep_nocolon(c_int::from(*p)) {
            p4 = p3;
            p3 = p2;
            p2 = p1;
            p1 = p;
        }
        let adv = nvim_rt_utfc_ptr2len(p);
        p = p.add(adv as usize);
    }
    // suppress unused variable warnings for p1-p3 (used for rolling window)
    let _ = (p1, p2, p3);

    // p4++ to append pathsep for symlink expansion
    p4 = p4.add(1);
    let c = *p4;
    *p4 = 0; // NUL-terminate temporarily
    let fixed_fname = nvim_rt_fix_fname(fname);
    *p4 = c; // restore

    if fixed_fname.is_null() {
        return FAIL;
    }

    // Find insertion point in p_rtp
    let fname_len = strlen(fixed_fname);
    let maxpathl = MAXPATHL;
    let buf = xmallocz(maxpathl).cast::<c_char>();

    let mut insp: *const c_char = ptr::null();
    let mut after_insp: *const c_char = ptr::null();
    let mut found_insp = false; // true when insp has been set
    let mut entry: *mut c_char = p_rtp;

    while *entry != 0 {
        let cur_entry: *const c_char = entry;
        copy_option_part(&raw mut entry, buf, maxpathl, c",".as_ptr());

        // Check if this entry is an "after" directory
        let pa = strstr(buf, c"after".as_ptr());
        let is_after = !pa.is_null()
            && pa > buf
            && nvim_rt_vim_ispathsep(c_int::from(*pa.sub(1)))
            && (nvim_rt_vim_ispathsep(c_int::from(*pa.add(5)))
                || *pa.add(5) == 0
                || *pa.add(5) == b',' as c_char);

        if is_after {
            if !found_insp {
                // Did not find fixed_fname before first "after" dir; insert before it
                insp = cur_entry;
                found_insp = true;
            }
            after_insp = cur_entry;
            break;
        }

        if !found_insp {
            // Add separator to buf, then fix_fname for comparison
            nvim_rt_add_pathsep(buf);
            let rtp_fixed = nvim_rt_fix_fname(buf);
            if rtp_fixed.is_null() {
                xfree(buf.cast());
                xfree(fixed_fname.cast());
                return FAIL;
            }
            if nvim_rt_path_fnamencmp(rtp_fixed, fixed_fname, fname_len) == 0 {
                // Insert fixed_fname after this entry (and comma)
                insp = entry;
                found_insp = true;
            }
            xfree(rtp_fixed.cast());
        }
    }

    xfree(buf.cast());

    if !found_insp {
        // Both fname and "after" not found; append at end
        insp = p_rtp.add(strlen(p_rtp));
    }

    // Check if rtp/pack/name/start/name/after exists
    let afterdir = nvim_rt_concat_fnames(fname, c"after".as_ptr(), true);
    let after_exists = if is_pack {
        rs_pack_has_entries(afterdir)
    } else {
        nvim_rt_os_isdir(afterdir)
    };
    let afterlen = if after_exists {
        strlen(afterdir) + 1
    } else {
        0
    }; // +1 for comma

    let oldlen = strlen(p_rtp);
    let addlen = strlen(fname) + 1; // +1 for comma
    let new_rtp_capacity = oldlen + addlen + afterlen + 1; // +1 for NUL
    let new_rtp = try_malloc(new_rtp_capacity).cast::<c_char>();
    if new_rtp.is_null() {
        xfree(fixed_fname.cast());
        xfree(afterdir.cast());
        return FAIL;
    }

    // Build new_rtp: {keep},{fname}
    let keep = insp as usize - p_rtp as usize;
    memmove(new_rtp.cast(), p_rtp.cast(), keep);
    let mut new_rtp_len = keep;

    if *insp == 0 {
        *new_rtp.add(new_rtp_len) = b',' as c_char; // comma before
        new_rtp_len += 1;
    }
    memmove(new_rtp.add(new_rtp_len).cast(), fname.cast(), addlen - 1);
    new_rtp_len += addlen - 1;
    if *insp != 0 {
        *new_rtp.add(new_rtp_len) = b',' as c_char; // comma after
        new_rtp_len += 1;
    }

    if afterlen > 0 && !after_insp.is_null() {
        let keep_after = after_insp as usize - p_rtp as usize;
        memmove(
            new_rtp.add(new_rtp_len).cast(),
            p_rtp.add(keep).cast(),
            keep_after - keep,
        );
        new_rtp_len += keep_after - keep;
        memmove(
            new_rtp.add(new_rtp_len).cast(),
            afterdir.cast(),
            afterlen - 1,
        );
        new_rtp_len += afterlen - 1;
        *new_rtp.add(new_rtp_len) = b',' as c_char;
        new_rtp_len += 1;
        if p_rtp.add(keep_after).cast::<u8>().read() != 0 {
            memmove(
                new_rtp.add(new_rtp_len).cast(),
                p_rtp.add(keep_after).cast(),
                oldlen - keep_after + 1,
            );
        } else {
            *new_rtp.add(new_rtp_len) = 0;
        }
    } else if p_rtp.add(keep).cast::<u8>().read() != 0 {
        memmove(
            new_rtp.add(new_rtp_len).cast(),
            p_rtp.add(keep).cast(),
            oldlen - keep + 1,
        );
    } else {
        *new_rtp.add(new_rtp_len) = 0;
    }

    if afterlen > 0 && after_insp.is_null() {
        xstrlcat(new_rtp, c",".as_ptr(), new_rtp_capacity);
        xstrlcat(new_rtp, afterdir, new_rtp_capacity);
    }

    nvim_rt_set_runtimepath(new_rtp);
    xfree(new_rtp.cast());
    xfree(fixed_fname.cast());
    xfree(afterdir.cast());
    OK
}

// =============================================================================
// Phase 2: Callback cookie sentinels (moved from C static vars)
// =============================================================================

/// Sentinel cookie: add directory to runtimepath only (do not load).
static APP_ADD_DIR: u8 = 0;
/// Sentinel cookie: load plugins only (do not add to runtimepath).
static APP_LOAD: u8 = 0;
/// Sentinel cookie: both add to runtimepath and load plugins.
static APP_BOTH: u8 = 0;

/// Helper: compare cookie against APP_ADD_DIR sentinel.
unsafe fn cookie_is_app_add_dir(cookie: *mut c_void) -> bool {
    cookie == (&raw const APP_ADD_DIR).cast_mut().cast()
}

/// Helper: compare cookie against APP_LOAD sentinel.
unsafe fn cookie_is_app_load(cookie: *mut c_void) -> bool {
    cookie == (&raw const APP_LOAD).cast_mut().cast()
}

// =============================================================================
// Phase 2: Pack callback functions (migrated from C)
// =============================================================================

/// Core pack plugin loading logic.
///
/// If cookie != APP_LOAD: add each fname to runtimepath if not already there.
/// If cookie != APP_ADD_DIR: load the plugin for each fname.
unsafe fn rs_add_pack_plugins_impl(
    opt: bool,
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) {
    let mut did_one = false;
    let maxpathl = MAXPATHL;

    if !cookie_is_app_load(cookie) {
        let buf = xmallocz(maxpathl).cast::<c_char>();
        let mut i = 0;
        while i < num_fnames {
            let fname = *fnames.add(i as usize);
            let mut found = false;

            // Scan p_rtp for fname
            let mut p = p_rtp;
            while *p != 0 {
                copy_option_part(&raw mut p, buf, maxpathl, c",".as_ptr());
                if path_fnamecmp(buf, fname) == 0 {
                    found = true;
                    break;
                }
            }

            if !found {
                // directory is not yet in 'runtimepath', add it
                if rs_add_pack_dir_to_rtp(fname, false) == FAIL {
                    xfree(buf.cast());
                    return;
                }
            }
            did_one = true;
            if !all {
                break;
            }
            i += 1;
        }
        xfree(buf.cast());
        if !all && did_one {
            return;
        }
    }

    if !cookie_is_app_add_dir(cookie) {
        let mut i = 0;
        while i < num_fnames {
            let fname = *fnames.add(i as usize);
            rs_load_pack_plugin(opt, fname);
            if !all {
                break;
            }
            i += 1;
        }
    }
}

/// Callback for start packages: add dir to rtp and optionally load plugins.
#[export_name = "add_start_pack_plugins"]
pub unsafe extern "C" fn rs_add_start_pack_plugins(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    rs_add_pack_plugins_impl(false, num_fnames, fnames, all, cookie);
    num_fnames > 0
}

/// Callback for opt packages: add dir to rtp and optionally load plugins.
#[export_name = "add_opt_pack_plugins"]
pub unsafe extern "C" fn rs_add_opt_pack_plugins(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    cookie: *mut c_void,
) -> bool {
    rs_add_pack_plugins_impl(true, num_fnames, fnames, all, cookie);
    num_fnames > 0
}

/// Callback for directory enumeration: adds start dirs to rtp.
#[export_name = "add_pack_start_dir"]
pub unsafe extern "C" fn rs_add_pack_start_dir(
    num_fnames: c_int,
    fnames: *mut *mut c_char,
    all: bool,
    _cookie: *mut c_void,
) -> bool {
    let maxpathl = MAXPATHL;
    let buf = xmallocz(maxpathl).cast::<c_char>();

    let start_pats = [c"/start/*".as_ptr(), c"/pack/*/start/*".as_ptr()];

    let mut i = 0;
    while i < num_fnames {
        let fname = *fnames.add(i as usize);
        let fname_len = strlen(fname);

        for pat in &start_pats {
            let pat_len = strlen(*pat);
            if fname_len + pat_len + 1 > maxpathl {
                continue;
            }
            xstrlcpy(buf, fname, maxpathl);
            xstrlcat(buf, *pat, maxpathl);
            if rs_pack_has_entries(buf) {
                rs_add_pack_dir_to_rtp(buf, true);
            }
        }

        if !all {
            break;
        }
        i += 1;
    }

    xfree(buf.cast());
    num_fnames > 1
}

// =============================================================================
// Phase 6: Migrated Package Management Functions
// =============================================================================

/// Check if a directory pattern has any matching entries.
///
/// Expands wildcards in `buf` looking for directories; returns true if any found.
///
/// # Safety
///
/// `buf` must be a valid null-terminated C string.
#[export_name = "pack_has_entries"]
pub unsafe extern "C" fn rs_pack_has_entries(buf: *mut c_char) -> bool {
    let mut num_files: c_int = 0;
    let mut files: *mut *mut c_char = ptr::null_mut();
    let mut pat = buf;

    if gen_expand_wildcards(1, &raw mut pat, &raw mut num_files, &raw mut files, EW_DIR) == OK {
        FreeWild(num_files, files);
    }

    num_files > 0
}

/// Load scripts in "plugin" directory of the package.
/// For opt packages, also load scripts in "ftdetect" (start packages already
/// load these from filetype.lua).
///
/// # Safety
///
/// `fname` must be a valid null-terminated C string.
#[export_name = "load_pack_plugin"]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn rs_load_pack_plugin(opt: bool, fname: *mut c_char) -> c_int {
    let ffname = nvim_rt_pkg_fix_fname(fname);
    if ffname.is_null() {
        return OK;
    }

    let plugpat = c"%s/plugin/**/*".as_ptr();
    let ftpat = c"%s/ftdetect/*".as_ptr();

    let ffname_len = strlen(ffname);
    // sizeof(plugpat) in C includes NUL; the longest pattern is "%s/plugin/**/*" = 15 bytes
    let len = ffname_len + 16;
    let mut pat = xmallocz(len).cast::<c_char>();

    nvim_rt_pkg_snprintf(pat, len, plugpat, ffname);
    rs_gen_expand_wildcards_and_cb(
        1,
        &raw mut pat,
        EW_FILE,
        true,
        Some(rs_source_callback_vim_lua),
        ptr::null_mut(),
    );

    let cmd = xstrdup(c"g:did_load_filetypes".as_ptr());

    // If runtime/filetype.lua wasn't loaded yet, the scripts will be
    // found when it loads.
    if opt && nvim_rt_pkg_eval_to_number(cmd) > 0 {
        nvim_rt_pkg_do_cmdline_cmd(c"augroup filetypedetect".as_ptr());
        nvim_rt_pkg_snprintf(pat, len, ftpat, ffname);
        rs_gen_expand_wildcards_and_cb(
            1,
            &raw mut pat,
            EW_FILE,
            true,
            Some(rs_source_callback_vim_lua),
            ptr::null_mut(),
        );
        nvim_rt_pkg_do_cmdline_cmd(c"augroup END".as_ptr());
    }

    xfree(cmd.cast());
    xfree(pat.cast());
    xfree(ffname.cast());

    OK
}

/// Add all packages in the "start" directory to 'runtimepath'.
///
/// # Safety
///
/// Accesses global C state (p_pp, callbacks).
#[export_name = "add_pack_start_dirs"]
pub unsafe extern "C" fn rs_add_pack_start_dirs() {
    rs_do_in_path(
        p_pp,
        c"".as_ptr(),
        ptr::null_mut(),
        dip::ALL + dip::DIR,
        Some(rs_add_pack_start_dir),
        ptr::null_mut(),
    );
}

/// Load plugins from all packages in the "start" directory.
///
/// # Safety
///
/// Accesses global C state.
#[export_name = "load_start_packages"]
pub unsafe extern "C" fn rs_load_start_packages() {
    globals::did_source_packages = true;

    let app_load = (&raw const APP_LOAD).cast_mut().cast::<c_void>();

    rs_do_in_path(
        p_pp,
        c"".as_ptr(),
        c"pack/*/start/*".as_ptr().cast_mut(),
        dip::ALL + dip::DIR,
        Some(rs_add_start_pack_plugins),
        app_load,
    );
    rs_do_in_path(
        p_pp,
        c"".as_ptr(),
        c"start/*".as_ptr().cast_mut(),
        dip::ALL + dip::DIR,
        Some(rs_add_start_pack_plugins),
        app_load,
    );
}

/// ":packloadall" - Find plugins in the package directories and source them.
///
/// # Safety
///
/// `eap` must be a valid exarg_T pointer.
#[export_name = "ex_packloadall"]
pub unsafe extern "C" fn rs_ex_packloadall(eap: *mut c_void) {
    let did_source = globals::did_source_packages;
    let forceit = (*eap.cast::<ExargT>()).forceit != 0;

    if !did_source || forceit {
        // First do a round to add all directories to 'runtimepath', then load
        // the plugins. This allows for plugins to use an autoload directory
        // of another plugin.
        rs_add_pack_start_dirs();
        rs_load_start_packages();
    }
}

/// Read all the plugin files at startup.
///
/// # Safety
///
/// Accesses global C state (p_lpl, p_rtp, did_source_packages).
#[export_name = "load_plugins"]
pub unsafe extern "C" fn rs_load_plugins() {
    if globals::p_lpl == 0 {
        return;
    }

    let mut rtp_copy: *mut c_char = p_rtp;
    let plugin_pattern = c"plugin/**/*".as_ptr().cast_mut();
    let did_source = globals::did_source_packages;

    if !did_source {
        rtp_copy = xstrdup(p_rtp);
        rs_add_pack_start_dirs();
    }

    // Don't use source_runtime_vim_lua() yet so we can check for :packloadall below.
    // NB: after calling this "rtp_copy" may have been freed if it wasn't copied.
    rs_source_in_path_vim_lua(rtp_copy, plugin_pattern, dip::ALL | dip::NOAFTER);
    pkg_time_msg(c"loading rtp plugins".as_ptr());

    // Only source "start" packages if not done already with a :packloadall
    // command.
    if !did_source {
        xfree(rtp_copy.cast());
        rs_load_start_packages();
    }
    pkg_time_msg(c"loading packages".as_ptr());

    rs_source_runtime_vim_lua(plugin_pattern, dip::ALL | dip::AFTER);
    pkg_time_msg(c"loading after plugins".as_ptr());
}

/// ":packadd[!] {name}" - Add an optional package and load it.
///
/// # Safety
///
/// `eap` must be a valid exarg_T pointer.
#[export_name = "ex_packadd"]
pub unsafe extern "C" fn rs_ex_packadd(eap: *mut c_void) {
    let mut res = OK;

    let eap_ref = &*eap.cast::<ExargT>();
    let arg = eap_ref.arg;
    let forceit = eap_ref.forceit != 0;

    // "pack/*/start/" + arg + NUL, with room for "start" or "opt"
    let arg_len = strlen(arg);
    let len = 13 + arg_len + 5;
    let pat = xmallocz(len).cast::<c_char>();

    let cookie: *mut c_void = if forceit {
        (&raw const APP_ADD_DIR).cast_mut().cast()
    } else {
        (&raw const APP_BOTH).cast_mut().cast()
    };

    // Only look under "start" when loading packages wasn't done yet.
    if !globals::did_source_packages {
        build_packadd_pattern(pat, len, c"start".as_ptr(), arg);

        res = rs_do_in_path(
            p_pp,
            c"".as_ptr(),
            pat,
            dip::ALL + dip::DIR,
            Some(rs_add_start_pack_plugins),
            cookie,
        );
    }

    // Give a "not found" error if nothing was found in 'start' or 'opt'.
    build_packadd_pattern(pat, len, c"opt".as_ptr(), arg);

    let err_flag = if res == FAIL { dip::ERR } else { 0 };
    rs_do_in_path(
        p_pp,
        c"".as_ptr(),
        pat,
        dip::ALL + dip::DIR + err_flag,
        Some(rs_add_opt_pack_plugins),
        cookie,
    );

    xfree(pat.cast());
}

/// Build a "pack/*/{type}/{name}" pattern string.
///
/// # Safety
///
/// `buf` must have at least `len` bytes available.
/// `pack_type` and `name` must be valid null-terminated C strings.
unsafe fn build_packadd_pattern(
    buf: *mut c_char,
    len: usize,
    pack_type: *const c_char,
    name: *const c_char,
) {
    // Write "pack/*/"
    let prefix = b"pack/*/";
    let mut pos = 0usize;
    for &b in prefix {
        if pos >= len - 1 {
            break;
        }
        *buf.add(pos) = b as c_char;
        pos += 1;
    }

    // Append pack_type
    let mut p = pack_type;
    while *p != 0 && pos < len - 1 {
        *buf.add(pos) = *p;
        pos += 1;
        p = p.add(1);
    }

    // Append "/"
    if pos < len - 1 {
        *buf.add(pos) = b'/' as c_char;
        pos += 1;
    }

    // Append name
    let mut p = name;
    while *p != 0 && pos < len - 1 {
        *buf.add(pos) = *p;
        pos += 1;
        p = p.add(1);
    }

    // NUL-terminate
    *buf.add(pos) = 0;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_package_status() {
        assert!(rs_package_can_load(PackageStatus::NotLoaded as c_int));
        assert!(!rs_package_can_load(PackageStatus::Loading as c_int));
        assert!(!rs_package_can_load(PackageStatus::Loaded as c_int));

        assert!(rs_package_is_loaded(PackageStatus::Loaded as c_int));
        assert!(!rs_package_is_loaded(PackageStatus::NotLoaded as c_int));
    }

    #[test]
    fn test_package_type_flags() {
        assert_eq!(rs_package_start_flag(), dip::START);
        assert_eq!(rs_package_opt_flag(), dip::OPT);
    }

    #[test]
    fn test_package_name_valid() {
        unsafe {
            let valid = CString::new("vim-plugin").unwrap();
            assert!(rs_package_name_valid(valid.as_ptr()));

            let empty = CString::new("").unwrap();
            assert!(!rs_package_name_valid(empty.as_ptr()));

            let with_slash = CString::new("plugin/sub").unwrap();
            assert!(!rs_package_name_valid(with_slash.as_ptr()));

            let with_wild = CString::new("plugin*").unwrap();
            assert!(!rs_package_name_valid(with_wild.as_ptr()));

            assert!(!rs_package_name_valid(std::ptr::null()));
        }
    }

    #[test]
    fn test_plugin_dirs() {
        assert_eq!(rs_plugin_dir_count(), PLUGIN_DIR_COUNT);
        assert!(rs_plugin_dir_count() > 0);

        // First should be "plugin"
        let first = rs_get_plugin_dir(0);
        assert!(!first.is_null());

        // Out of bounds returns null
        let oob = rs_get_plugin_dir(1000);
        assert!(oob.is_null());
    }

    #[test]
    fn test_build_packadd_pattern() {
        unsafe {
            let mut buf = [0i8; 256];
            let pack_type = CString::new("start").unwrap();
            let name = CString::new("myplugin").unwrap();
            build_packadd_pattern(buf.as_mut_ptr(), 256, pack_type.as_ptr(), name.as_ptr());
            let result = std::ffi::CStr::from_ptr(buf.as_ptr());
            assert_eq!(result.to_str().unwrap(), "pack/*/start/myplugin");

            let pack_type = CString::new("opt").unwrap();
            let name = CString::new("foo").unwrap();
            build_packadd_pattern(buf.as_mut_ptr(), 256, pack_type.as_ptr(), name.as_ptr());
            let result = std::ffi::CStr::from_ptr(buf.as_ptr());
            assert_eq!(result.to_str().unwrap(), "pack/*/opt/foo");
        }
    }
}
