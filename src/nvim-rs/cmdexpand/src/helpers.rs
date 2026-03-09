//! Leaf utility functions for command-line completion.

use libc::{c_char, c_int};

use crate::context::backslash::{XP_BS_COMMA, XP_BS_NONE, XP_BS_THREE};
use crate::context::vse_flags::{VSE_BUFFER, VSE_NONE, VSE_SHELL};
use crate::context::wild_options::{WILD_ESCAPE, WILD_HOME_REPLACE};
use crate::context::ExpandContext;
use crate::ExpandHandle;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // C functions with int return to avoid bool ABI issues
    fn nvim_cmdexpand_vim_ispathsep(c: c_int) -> c_int;
    fn nvim_cmdexpand_rem_backslash(p: *const c_char) -> c_int;
    fn nvim_cmdexpand_mb_ptr_adv_len(p: *const c_char) -> c_int;

    // String manipulation (these are existing C functions)
    fn vim_strsave_escaped(s: *const c_char, esc: *const c_char) -> *mut c_char;
    fn vim_strsave_fnameescape(s: *const c_char, what: c_int) -> *mut c_char;
    fn escape_fname(pp: *mut *mut c_char);
    fn tilde_replace(orig: *const c_char, numfiles: c_int, files: *mut *mut c_char);
    fn xfree(ptr: *mut libc::c_void);

    // Path utilities (existing `nvim_` wrappers)
    fn nvim_path_tail(p: *const c_char) -> *const c_char;
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;

    // `expand_T` complex operations (kept in C due to macros)
    fn nvim_expand_clear(xp: ExpandHandle);
    fn nvim_expand_free_wild(xp: ExpandHandle);
    fn nvim_expand_clear_orig(xp: ExpandHandle);

    // Static `cmdline_orig` management
    fn nvim_clear_cmdline_orig();
}

// =============================================================================
// Helper: `showmatches_gettail`
// =============================================================================

/// Return the tail of file name path `s`, ignoring a trailing "/".
///
/// # Safety
///
/// `s` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_showmatches_gettail(s: *mut c_char, eager: c_int) -> *mut c_char {
    let eager = eager != 0;
    let mut t = s;
    let mut had_sep = false;
    let mut p = s;

    while *p != 0 {
        if nvim_cmdexpand_vim_ispathsep(c_int::from(*p)) != 0 {
            if eager {
                t = p.add(1);
            } else {
                had_sep = true;
            }
        } else if had_sep {
            t = p;
            had_sep = false;
        }
        let adv = nvim_cmdexpand_mb_ptr_adv_len(p);
        p = p.add(adv as usize);
    }
    t
}

// =============================================================================
// Helper: `expand_showtail`
// =============================================================================

/// Return true if we only need to show the tail of completion matches.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_expand_showtail(xp: ExpandHandle) -> c_int {
    let ctx = (*xp).xp_context;

    // When not completing file names a "/" may mean something different.
    if ctx != ExpandContext::Files.to_raw()
        && ctx != ExpandContext::Shellcmd.to_raw()
        && ctx != ExpandContext::Directories.to_raw()
    {
        return 0;
    }

    let pattern = (*xp).xp_pattern;
    if pattern.is_null() {
        return 0;
    }

    let end = nvim_path_tail(pattern);
    if end == pattern {
        // No path separator
        return 0;
    }

    let mut s = pattern;
    while s.cast_const() < end {
        if nvim_cmdexpand_rem_backslash(s) != 0 {
            s = s.add(1);
        } else {
            let c = *s as u8;
            if c == b'*' || c == b'?' || c == b'[' {
                return 0;
            }
        }
        s = s.add(1);
    }
    1
}

// =============================================================================
// Helper: `wildescape`
// =============================================================================

/// Escape special characters in the cmdline completion matches.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `str_` must be a valid C string.
/// `files` must point to `numfiles` valid C string pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_wildescape(
    xp: ExpandHandle,
    str_: *const c_char,
    numfiles: c_int,
    files: *mut *mut c_char,
) {
    let ctx = (*xp).xp_context;
    let backslash = (*xp).xp_backslash;
    let shell = c_int::from((*xp).xp_shell);

    let vse_what = if ctx == ExpandContext::Buffers.to_raw() {
        VSE_BUFFER
    } else {
        VSE_NONE
    };

    if ctx == ExpandContext::Files.to_raw()
        || ctx == ExpandContext::FilesInPath.to_raw()
        || ctx == ExpandContext::Shellcmd.to_raw()
        || ctx == ExpandContext::Buffers.to_raw()
        || ctx == ExpandContext::Directories.to_raw()
        || ctx == ExpandContext::DirsInCdpath.to_raw()
    {
        for i in 0..numfiles as isize {
            // For ":set path=" we need to escape spaces twice
            if backslash & XP_BS_THREE != 0 {
                let pat = if backslash & XP_BS_COMMA != 0 {
                    c" ,".as_ptr()
                } else {
                    c" ".as_ptr()
                };
                let p = vim_strsave_escaped(*files.offset(i), pat);
                xfree((*files.offset(i)).cast::<libc::c_void>());
                *files.offset(i) = p;
                // BACKSLASH_IN_FILENAME is not defined on Linux
            } else if backslash & XP_BS_COMMA != 0
                && !nvim_vim_strchr(*files.offset(i), c_int::from(b',')).is_null()
            {
                let p = vim_strsave_escaped(*files.offset(i), c",".as_ptr());
                xfree((*files.offset(i)).cast::<libc::c_void>());
                *files.offset(i) = p;
            }

            // BACKSLASH_IN_FILENAME is not defined on Linux
            let fnameescape_what = if shell != 0 { VSE_SHELL } else { vse_what };
            let p = vim_strsave_fnameescape(*files.offset(i), fnameescape_what);
            xfree((*files.offset(i)).cast::<libc::c_void>());
            *files.offset(i) = p;

            // If 'str' starts with "\~", replace "~" at start of files[i] with "\~".
            if *str_ as u8 == b'\\'
                && *str_.add(1) as u8 == b'~'
                && *(*files.offset(i)) as u8 == b'~'
            {
                escape_fname(files.offset(i));
            }
        }
        (*xp).xp_backslash = XP_BS_NONE;

        // If the first file starts with a '+' escape it.
        if numfiles > 0 && *(*files.offset(0)) as u8 == b'+' {
            escape_fname(files.offset(0));
        }
    } else if ctx == ExpandContext::Tags.to_raw() {
        for i in 0..numfiles as isize {
            let p = vim_strsave_escaped(*files.offset(i), c"\\|\"".as_ptr());
            xfree((*files.offset(i)).cast::<libc::c_void>());
            *files.offset(i) = p;
        }
    }
}

// =============================================================================
// Helper: `ExpandEscape`
// =============================================================================

/// Escape special characters in cmdline completion matches (wrapper).
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `str_` must be a valid C string.
/// `files` must point to `numfiles` valid C string pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_expand_escape(
    xp: ExpandHandle,
    str_: *mut c_char,
    numfiles: c_int,
    files: *mut *mut c_char,
    options: c_int,
) {
    // May change home directory back to "~"
    if options & WILD_HOME_REPLACE != 0 {
        tilde_replace(str_, numfiles, files);
    }

    if options & WILD_ESCAPE != 0 {
        rs_wildescape(xp, str_, numfiles, files);
    }
}

// =============================================================================
// Phase 2: Expand struct operations
// =============================================================================

/// Prepare an expand structure for use.
///
/// Zeros the struct, then sets `xp_backslash` to `XP_BS_NONE`,
/// `xp_prefix` to `XP_PREFIX_NONE` (0), and `xp_numfiles` to -1.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle.
#[unsafe(export_name = "ExpandInit")]
pub unsafe extern "C" fn rs_expand_init(xp: ExpandHandle) {
    nvim_expand_clear(xp);
    (*xp).xp_backslash = XP_BS_NONE;
    (*xp).xp_prefix = XP_PREFIX_NONE;
    (*xp).xp_numfiles = -1;
}

/// Cleanup an expand structure after use.
///
/// Frees the wild matches if `xp_numfiles >= 0`, resets `xp_numfiles` to -1,
/// and frees `xp_orig`.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle.
#[unsafe(export_name = "ExpandCleanup")]
pub unsafe extern "C" fn rs_expand_cleanup(xp: ExpandHandle) {
    if (*xp).xp_numfiles >= 0 {
        nvim_expand_free_wild(xp);
        (*xp).xp_numfiles = -1;
    }
    nvim_expand_clear_orig(xp);
}

/// Clear the static `cmdline_orig` variable.
#[unsafe(export_name = "clear_cmdline_orig")]
pub extern "C" fn rs_clear_cmdline_orig() {
    // SAFETY: `nvim_clear_cmdline_orig` is a simple accessor that frees and
    // NULLs the static `cmdline_orig` variable.
    unsafe { nvim_clear_cmdline_orig() }
}

/// `XP_PREFIX_NONE` value (0).
const XP_PREFIX_NONE: c_int = 0;
