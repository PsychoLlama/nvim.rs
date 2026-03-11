//! Expansion and completion functions for runtime paths.
//!
//! Implements `ExpandRTDir_int`, `ExpandRTDir`, `expand_runtime_cmd`,
//! and `ExpandPackAddDir` which were previously in `src/nvim/runtime.c`.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::dip;

// =============================================================================
// runtime_expand_flags (moved from C static variable)
// =============================================================================

/// Flags controlling what :runtime command expansion searches.
/// Was `static int runtime_expand_flags` in runtime.c.
pub(crate) static RUNTIME_EXPAND_FLAGS: AtomicI32 = AtomicI32::new(0);

/// Get the runtime_expand_flags value (called from commands.rs and C).
#[no_mangle]
pub unsafe extern "C" fn nvim_rt_cmd_get_runtime_expand_flags() -> c_int {
    RUNTIME_EXPAND_FLAGS.load(Ordering::Relaxed)
}

/// Set the runtime_expand_flags value (called from commands.rs and C).
#[no_mangle]
pub unsafe extern "C" fn nvim_rt_cmd_set_runtime_expand_flags(val: c_int) {
    RUNTIME_EXPAND_FLAGS.store(val, Ordering::Relaxed);
}

// =============================================================================
// garray_T layout
// =============================================================================

/// Growing array structure matching C `garray_T`.
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

impl GArray {
    const fn new_char_ptr(growsize: c_int) -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: std::mem::size_of::<*mut c_char>() as c_int,
            ga_growsize: growsize,
            ga_data: ptr::null_mut(),
        }
    }

    fn items(&self) -> *mut *mut c_char {
        self.ga_data.cast()
    }
}

// =============================================================================
// Extern declarations
// =============================================================================

extern "C" {
    static p_rtp: *mut c_char;
    static p_pp: *mut c_char;

    fn globpath(
        path: *const c_char,
        file: *const c_char,
        ga: *mut GArray,
        expand_options: c_int,
        dirs: bool,
    );
    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GArray, n: c_int);
    fn ga_remove_duplicate_strings(gap: *mut GArray);

    fn path_tail(fname: *const c_char) -> *mut c_char;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn vim_ispathsep(c: c_int) -> bool;

    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xstrdup(str: *const c_char) -> *mut c_char;
    fn snprintf(s: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strncasecmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

/// `WILD_ADD_SLASH` flag for globpath expand_options.
const WILD_ADD_SLASH: c_int = 0x10;

// =============================================================================
// ga_append helper
// =============================================================================

/// Append a `*mut c_char` to `GArray` (equivalent to `GA_APPEND(char *, ga, val)`).
unsafe fn ga_append_str(ga: *mut GArray, val: *mut c_char) {
    ga_grow(ga, 1);
    *(*ga).items().add((*ga).ga_len as usize) = val;
    (*ga).ga_len += 1;
}

// =============================================================================
// mb_ptr_back helper
// =============================================================================

/// Move pointer backward by one multibyte character (equivalent to `MB_PTR_BACK`).
unsafe fn mb_ptr_back(base: *const c_char, p: *mut c_char) -> *mut c_char {
    p.sub(utf_head_off(base, p.sub(1)) as usize + 1)
}

// =============================================================================
// ExpandRTDir_int
// =============================================================================

/// Core runtime directory expansion.
///
/// Constructs glob patterns for each entry in `dirnames` and calls `globpath`,
/// then strips extensions and path prefixes from results.
///
/// Ported from `static void ExpandRTDir_int` in runtime.c.
#[allow(clippy::too_many_lines)]
unsafe fn expand_rt_dir_int(
    pat: *const c_char,
    pat_len: usize,
    flags: c_int,
    keep_ext: bool,
    gap: *mut GArray,
    dirnames: *const *const c_char,
) {
    let mut i = 0;
    while !(*dirnames.add(i)).is_null() {
        let dirname = *dirnames.add(i);
        let dirname_len = strlen(dirname);
        let buf_len = dirname_len + pat_len + 64;
        let buf = xmalloc(buf_len).cast::<c_char>();
        let mut glob_flags: c_int = 0;
        let mut expand_dirs = false;

        // Build base pattern: {dirname}/{pat}*.{vim,lua}
        snprintf(
            buf,
            buf_len,
            c"%s%s%s%s".as_ptr(),
            if dirname_len > 0 {
                dirname
            } else {
                c"".as_ptr()
            },
            if dirname_len > 0 {
                c"/".as_ptr()
            } else {
                c"".as_ptr()
            },
            pat,
            c"*.{vim,lua}".as_ptr(),
        );

        'expand: loop {
            if (flags & dip::NORTP) == 0 {
                globpath(p_rtp, buf, gap, glob_flags, expand_dirs);
            }

            if (flags & dip::START) != 0 {
                snprintf(
                    buf,
                    buf_len,
                    c"pack/*/start/*/%s%s%s%s".as_ptr(),
                    if dirname_len > 0 {
                        dirname
                    } else {
                        c"".as_ptr()
                    },
                    if dirname_len > 0 {
                        c"/".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    pat,
                    if expand_dirs {
                        c"*".as_ptr()
                    } else {
                        c"*.{vim,lua}".as_ptr()
                    },
                );
                globpath(p_pp, buf, gap, glob_flags, expand_dirs);

                snprintf(
                    buf,
                    buf_len,
                    c"start/*/%s%s%s%s".as_ptr(),
                    if dirname_len > 0 {
                        dirname
                    } else {
                        c"".as_ptr()
                    },
                    if dirname_len > 0 {
                        c"/".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    pat,
                    if expand_dirs {
                        c"*".as_ptr()
                    } else {
                        c"*.{vim,lua}".as_ptr()
                    },
                );
                globpath(p_pp, buf, gap, glob_flags, expand_dirs);
            }

            if (flags & dip::OPT) != 0 {
                snprintf(
                    buf,
                    buf_len,
                    c"pack/*/opt/*/%s%s%s%s".as_ptr(),
                    if dirname_len > 0 {
                        dirname
                    } else {
                        c"".as_ptr()
                    },
                    if dirname_len > 0 {
                        c"/".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    pat,
                    if expand_dirs {
                        c"*".as_ptr()
                    } else {
                        c"*.{vim,lua}".as_ptr()
                    },
                );
                globpath(p_pp, buf, gap, glob_flags, expand_dirs);

                snprintf(
                    buf,
                    buf_len,
                    c"opt/*/%s%s%s%s".as_ptr(),
                    if dirname_len > 0 {
                        dirname
                    } else {
                        c"".as_ptr()
                    },
                    if dirname_len > 0 {
                        c"/".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    pat,
                    if expand_dirs {
                        c"*".as_ptr()
                    } else {
                        c"*.{vim,lua}".as_ptr()
                    },
                );
                globpath(p_pp, buf, gap, glob_flags, expand_dirs);
            }

            // Second round for directories (when dirname is empty)
            if dirname_len == 0 && !expand_dirs {
                snprintf(buf, buf_len, c"%s*".as_ptr(), pat);
                glob_flags = WILD_ADD_SLASH;
                expand_dirs = true;
                continue 'expand;
            }
            break;
        }

        xfree(buf.cast());
        i += 1;
    }

    // Count path separators in pat
    let mut pat_pathsep_cnt: c_int = 0;
    for k in 0..pat_len {
        if vim_ispathsep(c_int::from(*pat.add(k))) {
            pat_pathsep_cnt += 1;
        }
    }

    // Strip extensions and path prefixes from results
    for k in 0..((*gap).ga_len as usize) {
        let m: *mut c_char = *(*gap).items().add(k);
        let mut s = m;
        let mut e = s.add(strlen(s));
        // Strip .vim or .lua extension if not keeping extension
        if !keep_ext
            && e.offset_from(s) > 4
            && (strncasecmp(e.sub(4), c".vim".as_ptr(), 4) == 0
                || strncasecmp(e.sub(4), c".lua".as_ptr(), 4) == 0)
        {
            e = e.sub(4);
            *e = 0;
        }

        let mut match_pathsep_cnt: c_int = if e > s && *e.sub(1) == b'/' as c_char {
            -1
        } else {
            0
        };
        s = e;
        while s > m {
            s = mb_ptr_back(m, s);
            if vim_ispathsep(c_int::from(*s)) {
                match_pathsep_cnt += 1;
                if match_pathsep_cnt > pat_pathsep_cnt {
                    break;
                }
            }
        }
        s = s.add(1);
        if s != m {
            let move_len = e.offset_from(s) as usize + 1;
            memmove(m.cast(), s.cast(), move_len);
        }
    }

    if (*gap).ga_len == 0 {
        return;
    }

    ga_remove_duplicate_strings(gap);
}

// =============================================================================
// Public functions
// =============================================================================

/// Expand color scheme, compiler or filetype names from 'runtimepath'.
///
/// Replaces `int ExpandRTDir` in runtime.c.
///
/// # Safety
/// All pointer arguments must be valid.
#[export_name = "ExpandRTDir"]
pub unsafe extern "C" fn rs_expand_rt_dir(
    pat: *mut c_char,
    flags: c_int,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    dirnames: *const *const c_char,
) -> c_int {
    *num_file = 0;
    *file = ptr::null_mut();

    let mut ga = GArray::new_char_ptr(10);
    ga_init(&raw mut ga, ga.ga_itemsize, ga.ga_growsize);

    expand_rt_dir_int(pat, strlen(pat), flags, false, &raw mut ga, dirnames);

    if ga.ga_len == 0 {
        return 1; // FAIL
    }

    *file = ga.ga_data.cast();
    *num_file = ga.ga_len;
    0 // OK
}

/// Handle command line completion for the :runtime command.
///
/// Replaces `int expand_runtime_cmd` in runtime.c.
///
/// # Safety
/// All pointer arguments must be valid.
#[export_name = "expand_runtime_cmd"]
pub unsafe extern "C" fn rs_expand_runtime_cmd(
    pat: *mut c_char,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    *num_matches = 0;
    *matches = ptr::null_mut();

    let mut ga = GArray::new_char_ptr(10);
    ga_init(&raw mut ga, ga.ga_itemsize, ga.ga_growsize);

    let pat_len = strlen(pat);
    let dirnames: [*const c_char; 2] = [c"".as_ptr(), ptr::null()];
    let expand_flags = RUNTIME_EXPAND_FLAGS.load(Ordering::Relaxed);
    expand_rt_dir_int(
        pat,
        pat_len,
        expand_flags,
        true,
        &raw mut ga,
        dirnames.as_ptr(),
    );

    // Try to complete [where] argument values when no file match was found.
    if expand_flags == 0 {
        let where_values: [*const c_char; 4] = [
            c"START".as_ptr(),
            c"OPT".as_ptr(),
            c"PACK".as_ptr(),
            c"ALL".as_ptr(),
        ];
        for &wv in &where_values {
            if strncmp(pat, wv, pat_len) == 0 {
                ga_append_str(&raw mut ga, xstrdup(wv));
            }
        }
    }

    if ga.ga_len == 0 {
        return 1; // FAIL
    }

    *matches = ga.ga_data.cast();
    *num_matches = ga.ga_len;
    0 // OK
}

/// Expand :packadd plugin names from 'packpath'.
///
/// Replaces `int ExpandPackAddDir` in runtime.c.
///
/// # Safety
/// All pointer arguments must be valid.
#[export_name = "ExpandPackAddDir"]
pub unsafe extern "C" fn rs_expand_pack_add_dir(
    pat: *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
) -> c_int {
    *num_file = 0;
    *file = ptr::null_mut();

    let pat_len = strlen(pat);
    let mut ga = GArray::new_char_ptr(10);
    ga_init(&raw mut ga, ga.ga_itemsize, ga.ga_growsize);

    let buflen = pat_len + 26;
    let s = xmalloc(buflen).cast::<c_char>();
    snprintf(s, buflen, c"pack/*/opt/%s*".as_ptr(), pat);
    globpath(p_pp, s, &raw mut ga, 0, true);
    snprintf(s, buflen, c"opt/%s*".as_ptr(), pat);
    globpath(p_pp, s, &raw mut ga, 0, true);
    xfree(s.cast());

    for k in 0..(ga.ga_len as usize) {
        let m: *mut c_char = *ga.items().add(k);
        let tail = path_tail(m);
        let tail_len = strlen(tail) + 1;
        memmove(m.cast(), tail.cast(), tail_len);
    }

    if ga.ga_len == 0 {
        return 1; // FAIL
    }

    ga_remove_duplicate_strings(&raw mut ga);

    *file = ga.ga_data.cast();
    *num_file = ga.ga_len;
    0 // OK
}
