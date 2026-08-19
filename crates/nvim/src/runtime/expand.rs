//! Command-line completion for the runtime commands.
//!
//! [`ExpandRTDir`] completes a file name against a set of 'runtimepath'
//! subdirectories -- what `:colorscheme`, `:compiler`, `:runtime` and friends
//! offer -- and [`ExpandPackAddDir`] does the same for `:packadd` against
//! 'packpath'.  [`expand_runtime_cmd`] is `:runtime`'s own two-stage
//! completion, where the first word may be one of the `START`/`OPT`/`PACK`/
//! `ALL` qualifiers and everything after it is a path.
//!
//! Every match is offered as a *bare* name: the directory the search started
//! from and the `.vim`/`.lua` extension are trimmed back off again, keeping
//! only as many path components as the pattern itself spelled
//! ([`trim_match`]).

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::WildOpts;

use crate::types::{FAIL, OK};
use core::ffi::{CStr, c_char, c_int};
use core::{ptr, slice};

/// What a completion round looks for: script files, or — on the second round,
/// where a directory name is being completed — anything.
const SCRIPTS: &CStr = c"*.{vim,lua}";
const ANYTHING: &CStr = c"*";

/// Write `{prefix}{dir}/{pat}{suffix}` into `buf`, dropping the directory and
/// its separator when `dir` is empty.
///
/// # Safety
/// `buf` must be writable for `buf_len` bytes; `dir` and `pat` must be
/// NUL-terminated.
unsafe fn build_pattern(
    buf: *mut c_char,
    buf_len: size_t,
    prefix: &CStr,
    dir: *mut c_char,
    pat: *mut c_char,
    suffix: &CStr,
) {
    // SAFETY: the caller's buffer and strings; `snprintf` truncates within
    // `buf_len` and NUL-terminates.
    unsafe {
        let (dir, sep) = if *dir != 0 {
            (dir.cast_const(), c"/".as_ptr())
        } else {
            (c"".as_ptr(), c"".as_ptr())
        };
        snprintf(
            buf,
            buf_len,
            c"%s%s%s%s%s".as_ptr(),
            prefix.as_ptr(),
            dir,
            sep,
            pat,
            suffix.as_ptr(),
        );
    }
}

/// Glob `pat` under `dir` in 'runtimepath' and in 'packpath''s package trees,
/// collecting the matches in `gap`.
///
/// When `dir` is empty this runs twice: once for script files, once more with
/// `WildOpts::ADD_SLASH` for directories, which is what makes `:runtime` complete a
/// subdirectory name.
///
/// # Safety
/// `buf` must be writable for `buf_len` bytes; `dir`, `pat` must be
/// NUL-terminated and `gap` an initialised array of allocated strings.
unsafe fn glob_rounds(
    pat: *mut c_char,
    buf: *mut c_char,
    buf_len: size_t,
    dir: *mut c_char,
    flags: c_int,
    gap: *mut garray_T,
) {
    let mut glob_flags = WildOpts::NONE;
    let mut expand_dirs = false;
    // SAFETY: the caller's buffer and strings, and `globpath` only appends.
    unsafe {
        build_pattern(buf, buf_len, c"", dir, pat, SCRIPTS);
        loop {
            if flags & DIP_NORTP as c_int == 0 {
                globpath(p_rtp.get(), buf, gap, glob_flags, expand_dirs);
            }
            let suffix = if expand_dirs { ANYTHING } else { SCRIPTS };
            if flags & DIP_START as c_int != 0 {
                for prefix in [c"pack/*/start/*/", c"start/*/"] {
                    build_pattern(buf, buf_len, prefix, dir, pat, suffix);
                    globpath(p_pp.get(), buf, gap, glob_flags, expand_dirs);
                }
            }
            if flags & DIP_OPT as c_int != 0 {
                for prefix in [c"pack/*/opt/*/", c"opt/*/"] {
                    build_pattern(buf, buf_len, prefix, dir, pat, suffix);
                    globpath(p_pp.get(), buf, gap, glob_flags, expand_dirs);
                }
            }
            // Second round, for directories.
            if *dir != 0 || expand_dirs {
                return;
            }
            snprintf(buf, buf_len, c"%s*".as_ptr(), pat);
            glob_flags = WildOpts::ADD_SLASH;
            expand_dirs = true;
        }
    }
}

/// Cut a match back to the name the user is completing: no `.vim`/`.lua`
/// extension, and no more leading path components than the pattern spelled.
///
/// Rewrites the string in place, which is what the garray holds.
///
/// # Safety
/// `matched` must be a NUL-terminated allocated string.
unsafe fn trim_match(matched: *mut c_char, keep_ext: bool, pat_pathsep_cnt: c_int) {
    // SAFETY: `matched` is NUL-terminated, and every walk below stays between
    // its first byte and its terminator.
    unsafe {
        let mut e = matched.add(strlen(matched));
        if e.offset_from(matched) > 4
            && !keep_ext
            && (strncasecmp(e.sub(4), c".vim".as_ptr(), 4) == 0
                || strncasecmp(e.sub(4), c".lua".as_ptr(), 4) == 0)
        {
            e = e.sub(4);
            *e = 0;
        }

        // A trailing slash is a component the pattern did not have to spell.
        let mut match_pathsep_cnt = if e > matched && *e.sub(1) == b'/' as c_char {
            -1
        } else {
            0
        };
        let mut s = e;
        while s > matched {
            if vim_ispathsep(*s as c_int) {
                match_pathsep_cnt += 1;
                if match_pathsep_cnt > pat_pathsep_cnt {
                    break;
                }
            }
            s = s.sub(utf_head_off(matched, s.sub(1)) as usize + 1);
        }
        s = s.add(1);
        if s != matched {
            debug_assert!(e.offset_from(s) + 1 >= 0, "(e - s) + 1 >= 0");
            memmove(matched.cast(), s.cast(), (e.offset_from(s) as size_t) + 1);
        }
    }
}

/// The garray's strings as a slice.
///
/// # Safety
/// `gap` must be an initialised array of `char *`.
unsafe fn ga_strings<'a>(gap: *mut garray_T) -> &'a [*mut c_char] {
    // SAFETY: the caller's garray, `ga_len` entries long.
    unsafe {
        if (*gap).ga_len <= 0 {
            return &[];
        }
        slice::from_raw_parts((*gap).ga_data.cast::<*mut c_char>(), (*gap).ga_len as usize)
    }
}

/// Collect the completion matches for `pat` under each of `dirnames`.
///
/// # Safety
/// `pat` must be NUL-terminated with length `pat_len`, `gap` an initialised
/// array of allocated strings, and `dirnames` a NULL-terminated array of
/// NUL-terminated directory names.
unsafe fn ExpandRTDir_int(
    pat: *mut c_char,
    pat_len: size_t,
    flags: c_int,
    keep_ext: bool,
    gap: *mut garray_T,
    dirnames: *mut *mut c_char,
) {
    // TODO(bfredl): this is bullshit, expandpath should not reinvent path
    // logic.
    let mut i = 0;
    // SAFETY: `dirnames` is NULL-terminated.
    while !unsafe { *dirnames.add(i) }.is_null() {
        // SAFETY: as above; `buf` is sized for the longest pattern built into
        // it (the longest prefix is fifteen bytes, the longest suffix eleven).
        unsafe {
            let dir = *dirnames.add(i);
            let buf_len = strlen(dir) + pat_len + 64;
            let buf = xmalloc(buf_len).cast::<c_char>();
            glob_rounds(pat, buf, buf_len, dir, flags, gap);
            xfree(buf.cast());
        }
        i += 1;
    }

    // SAFETY: `pat` has `pat_len` readable bytes.
    let pat_pathsep_cnt = unsafe { slice::from_raw_parts(pat, pat_len) }
        .iter()
        .filter(|&&b| vim_ispathsep(b as c_int))
        .count() as c_int;

    // SAFETY: the garray holds `ga_len` allocated strings.
    for &matched in unsafe { ga_strings(gap) } {
        // SAFETY: as above.
        unsafe { trim_match(matched, keep_ext, pat_pathsep_cnt) };
    }

    // SAFETY: as above.
    if unsafe { (*gap).ga_len } > 0 {
        // Sort and remove the duplicates that several `dirnames` produce.
        // SAFETY: as above.
        unsafe { ga_remove_duplicate_strings(gap) };
    }
}

/// A garray of `char *` with room for ten.
fn new_string_garray() -> garray_T {
    let mut ga = garray_T::default();
    // SAFETY: `ga` is a fresh local, and `ga_init` only writes its fields.
    unsafe { ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 10) };
    ga
}

/// Hand a completed garray to an out-parameter pair, or answer FAIL when it
/// is empty.
///
/// # Safety
/// Both out-parameters must be writable, and `ga` must own its strings.
unsafe fn take_matches(ga: garray_T, num_file: *mut c_int, file: *mut *mut *mut c_char) -> c_int {
    if ga.ga_len <= 0 {
        return FAIL;
    }
    // SAFETY: the caller's out-parameters; the garray's buffer is handed over.
    unsafe {
        *file = ga.ga_data.cast();
        *num_file = ga.ga_len;
    }
    OK
}

/// Expand color scheme, compiler or filetype names.
///
/// Searches `{runtimepath}/{dirnames}/{pat}.{vim,lua}`; `DIP_START` adds
/// `{packpath}/pack/*/start/*/{dirnames}/...` and `DIP_OPT` the `opt`
/// equivalent.  `dirnames` is an array of one or more directory names.
///
/// # Safety
/// As [`ExpandRTDir_int`]; both out-parameters must be writable.
pub unsafe fn ExpandRTDir(
    pat: *mut c_char,
    flags: c_int,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    dirnames: *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's out-parameters and strings.
    unsafe {
        *num_file = 0;
        *file = ptr::null_mut();
        let mut ga = new_string_garray();
        ExpandRTDir_int(pat, strlen(pat), flags, false, &raw mut ga, dirnames);
        take_matches(ga, num_file, file)
    }
}

/// The `[where]` qualifiers `:runtime` completion offers.
///
/// Deliberately a second copy of the words `get_runtime_cmd_flags` parses:
/// what completion proposes and what the command accepts are separate
/// questions, and upstream spells them separately too.
const WHERE_VALUES: [&CStr; 4] = [c"START", c"OPT", c"PACK", c"ALL"];

/// Command-line completion for the `:runtime` command.
///
/// # Safety
/// `pat` must be NUL-terminated and both out-parameters writable.
pub unsafe fn expand_runtime_cmd(
    pat: *mut c_char,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's out-parameters and pattern.
    unsafe {
        *num_matches = 0;
        *matches = ptr::null_mut();
        let mut ga = new_string_garray();
        let pat_len = strlen(pat);
        let mut dirnames = [c"".as_ptr().cast_mut(), ptr::null_mut()];
        ExpandRTDir_int(
            pat,
            pat_len,
            runtime_expand_flags.get(),
            true,
            &raw mut ga,
            dirnames.as_mut_ptr(),
        );

        // Complete the [where] argument too, when none was given.
        if runtime_expand_flags.get() == 0 {
            for value in WHERE_VALUES {
                if strncmp(pat, value.as_ptr(), pat_len) == 0 {
                    ga_grow(&raw mut ga, 1);
                    *ga.ga_data.cast::<*mut c_char>().offset(ga.ga_len as isize) =
                        xstrdup(value.as_ptr());
                    ga.ga_len += 1;
                }
            }
        }

        take_matches(ga, num_matches, matches)
    }
}

/// Expand `:packadd` names: `{packpath}/pack/*/opt/{pat}`.
///
/// # Safety
/// `pat` must be NUL-terminated and both out-parameters writable.
pub unsafe fn ExpandPackAddDir(
    pat: *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's out-parameters and pattern; `s` is owned and freed
    // below.
    unsafe {
        *num_file = 0;
        *file = ptr::null_mut();
        let mut ga = new_string_garray();

        let buflen = strlen(pat) + 26;
        let s = xmalloc(buflen).cast::<c_char>();
        for fmt in [c"pack/*/opt/%s*", c"opt/%s*"] {
            snprintf(s, buflen, fmt.as_ptr(), pat);
            globpath(p_pp.get(), s, &raw mut ga, WildOpts::NONE, true);
        }
        xfree(s.cast());

        // Offer the package name, not the path it was found at.
        for &matched in ga_strings(&raw mut ga) {
            let tail = path_tail(matched);
            memmove(matched.cast(), tail.cast(), strlen(tail) + 1);
        }

        if ga.ga_len <= 0 {
            return FAIL;
        }
        // Sort and remove the duplicates the two patterns can produce.
        ga_remove_duplicate_strings(&raw mut ga);
        take_matches(ga, num_file, file)
    }
}
