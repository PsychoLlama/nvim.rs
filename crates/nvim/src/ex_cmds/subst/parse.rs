//! Reading a `:s` command line, and remembering it for the next one.
//!
//! [`skip_substitute`] walks the pattern/replacement/flags off the command
//! line without interpreting them (`:s` is its own little language, and the
//! delimiter may be almost any character -- [`check_regexp_delim`] rejects the
//! ones that would be ambiguous), [`sub_parse_flags`] turns the trailing
//! letters into `subflags_T`, and [`old_sub`] is the `~` replacement text
//! carried from the last `:s`.  [`sub_joining_lines`] is the `\n`-in-the-
//! pattern case, which joins rather than substitutes, and [`sub_grow_buf`] is
//! the output buffer's growth policy.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::do_sub_msg;
use crate::cmdhist::add_to_history;
use crate::ex_cmds::{
    _ISalpha, EXFLAG_LIST, EXFLAG_NR, EXFLAG_PRINT, FAIL, HIST_SEARCH, NUL, OK, kSubHonorOptions,
    kSubIgnoreCase, kSubMatchCase, subflags_T,
};
use crate::ex_docmd::ex_may_print;
use crate::global_cell::GlobalCell;
use crate::main::{curbuf, curwin, p_gd, sub_nlines, sub_nsubs};
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xcalloc, xfree, xrealloc};
use crate::message::emsg;
use crate::ops::do_join;
use crate::option::magic_isset;
use crate::os::libc::{__ctype_b_loc, gettext, memset, strlen};
use crate::regexp::{RE_LAST, RE_SUBST};
use crate::search::save_re_pat;
use crate::types::{SubReplacementString, Timestamp, exarg_T, linenr_T, size_t};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::{ptr, slice};

/// The previous `:s` replacement string, which a bare `:s` and a `~` in a
/// replacement both reach for.  Its two pointers are `xmalloc`ed and owned
/// here; shada reads and writes it through [`sub_get_replacement`] and
/// [`sub_set_replacement`].
pub(crate) static old_sub: GlobalCell<SubReplacementString> =
    GlobalCell::new(SubReplacementString {
        sub: ptr::null_mut(),
        timestamp: 0 as Timestamp,
        additional_data: ptr::null_mut(),
    });

/// The `:s` flags in force.  Process-wide, because `:&&` and a bare `:s`
/// reuse the previous command's flags, and because a `\=` replacement can
/// run another `:s` that must not leave its own flags behind (`do_sub` saves
/// and restores this around the expression).
pub(crate) static subflags: GlobalCell<subflags_T> = GlobalCell::new(subflags_T {
    do_all: false,
    do_ask: false,
    do_count: false,
    do_error: true,
    do_print: false,
    do_list: false,
    do_number: false,
    do_ic: kSubHonorOptions,
});

/// Set by `do_sub` when a `:global` is running, so that `global_exe` puts the
/// cursor on the first non-blank once the whole command is done.
pub(crate) static global_need_beginline: GlobalCell<bool> = GlobalCell::new(false);

/// Copy the old substitute replacement string out.
///
/// # Safety
/// `ret_sub` must be writable.
pub unsafe fn sub_get_replacement(ret_sub: *mut SubReplacementString) {
    // SAFETY: caller's contract.
    unsafe { *ret_sub = old_sub.get() };
}

/// Set the substitute string and its timestamp.
///
/// `sub` must already be in allocated memory: it is taken, not copied.
///
/// # Safety
/// Main thread; `sub`'s two pointers must be `xmalloc`ed or null, and no
/// other owner may hold what this displaces.
pub unsafe fn sub_set_replacement(sub: SubReplacementString) {
    let old = old_sub.get();
    // SAFETY: both pointers were this module's own allocations.
    unsafe {
        xfree(old.sub as *mut c_void);
        if sub.additional_data != old.additional_data {
            xfree(old.additional_data as *mut c_void);
        }
    }
    old_sub.set(sub);
}

/// Recognise `:%s/\n//` and turn it into a join command, which is much more
/// efficient.
///
/// The pattern must be exactly `\n`, the replacement empty, and the flags
/// either absent or one of `g`, `l`, `p`, `#` -- anything else is a real
/// substitution.  `save` says whether to remember the pattern (a preview
/// must not).
///
/// Returns true when `:substitute` was handled as a join, including under
/// `eap->skip`, where nothing is done at all.
///
/// # Safety
/// Main thread; `eap`, `sub` and `cmd` must be live and `pat` live or null.
pub(crate) unsafe fn sub_joining_lines(
    eap: *mut exarg_T,
    pat: *mut c_char,
    patlen: size_t,
    sub: *const c_char,
    cmd: *const c_char,
    save: bool,
    keeppatterns: bool,
) -> bool {
    // SAFETY: caller's contract -- three NUL-terminated strings.
    let joins = unsafe {
        !pat.is_null()
            && CStr::from_ptr(pat).to_bytes() == b"\\n"
            && *sub as c_int == NUL
            && matches!(
                CStr::from_ptr(cmd).to_bytes(),
                [] | [b'g' | b'l' | b'p' | b'#']
            )
    };
    if !joins {
        return false;
    }
    // SAFETY: caller's contract.
    if unsafe { (*eap).skip } != 0 {
        return true;
    }

    // SAFETY: caller's contract; the current window and buffer are live.
    let joined_lines_count = unsafe {
        (*curwin.get()).w_cursor.lnum = (*eap).line1;
        (*eap).flags = match *cmd as u8 {
            b'l' => EXFLAG_LIST,
            b'#' => EXFLAG_NR,
            b'p' => EXFLAG_PRINT,
            _ => (*eap).flags,
        };
        // The number of lines joined is the number of lines in the range,
        // plus one more if this is not the end of the file.
        (*eap).line2 - (*eap).line1
            + 1 as linenr_T
            + linenr_T::from((*eap).line2 < (*curbuf.get()).b_ml.ml_line_count)
    };
    if joined_lines_count > 1 as linenr_T {
        // SAFETY: the range is inside the buffer; message state is ready.
        unsafe {
            do_join(joined_lines_count as size_t, false, true, false, true);
            sub_nsubs.set(joined_lines_count - 1 as linenr_T);
            sub_nlines.set(1 as linenr_T);
            do_sub_msg(false);
            ex_may_print(eap);
        }
    }

    if save {
        // SAFETY: `pat` is `patlen` bytes long, by the caller's contract.
        unsafe {
            if !keeppatterns {
                save_re_pat(RE_SUBST as c_int, pat, patlen, magic_isset());
            }
            // Put the pattern in the search history.
            add_to_history(
                HIST_SEARCH as c_int,
                slice::from_raw_parts(pat as *const u8, patlen),
                true,
                NUL as u8,
            );
        }
    }
    true
}

/// Make room for `needed_len` more bytes of replacement text, answering where
/// to write them.
///
/// A little more than is strictly necessary is allocated, to keep the
/// reallocation out of the inner loop.  `new_start` is null on the first
/// call and owns the buffer afterwards.
///
/// # Safety
/// `*new_start` must be null or an `xmalloc`ed NUL-terminated buffer of
/// `*new_start_len` bytes.
pub(crate) unsafe fn sub_grow_buf(
    new_start: &mut *mut c_char,
    new_start_len: &mut c_int,
    mut needed_len: c_int,
) -> *mut c_char {
    if new_start.is_null() {
        // Get space for a temporary buffer to substitute into, with extra to
        // avoid too many calls to xmalloc()/free().
        *new_start_len = needed_len + 50 as c_int;
        // SAFETY: a fresh zeroed allocation of the size just chosen.
        unsafe {
            *new_start = xcalloc(1 as size_t, *new_start_len as size_t) as *mut c_char;
            **new_start = NUL as c_char;
        }
        return *new_start;
    }

    // Check whether the temporary buffer is long enough to substitute into.
    // If not, make it larger (again with a bit extra).
    // SAFETY: caller's contract -- a NUL-terminated buffer.
    let len = unsafe { strlen(*new_start) };
    needed_len += len as c_int;
    if needed_len > *new_start_len {
        let prev_new_start_len = *new_start_len as size_t;
        *new_start_len = needed_len + 50 as c_int;
        let added_len = (*new_start_len as size_t).wrapping_sub(prev_new_start_len);
        // SAFETY: the buffer is ours to grow, and the tail past the old
        // length is what `memset` clears.
        unsafe {
            *new_start =
                xrealloc(*new_start as *mut c_void, *new_start_len as size_t) as *mut c_char;
            memset(
                (*new_start).add(prev_new_start_len) as *mut c_void,
                0 as c_int,
                added_len,
            );
        }
    }
    // SAFETY: `len` is the buffer's own string length.
    unsafe { (*new_start).add(len) }
}

/// Read `:substitute`'s trailing `{flags}` into `subflags`, answering where
/// the flags stopped.
///
/// A leading `&` keeps the previous flags; otherwise they are reset from
/// 'gdefault' and the defaults first.  `g` and `c` toggle, `r` is never a
/// toggle but redirects `which_pat`, and the first unknown letter ends the
/// run.
///
/// # Safety
/// Main thread; `cmd` must be a live NUL-terminated string.
pub(crate) unsafe fn sub_parse_flags(
    cmd: *mut c_char,
    flags: &mut subflags_T,
    which_pat: &mut c_int,
) -> *mut c_char {
    // SAFETY: caller's contract.
    let bytes = unsafe { CStr::from_ptr(cmd) }.to_bytes();

    // Find the trailing options.  When '&' is used, keep the old ones.
    let mut i = 0;
    if bytes.first() == Some(&b'&') {
        i = 1;
    } else {
        flags.do_all = p_gd.get() != 0;
        flags.do_ask = false;
        flags.do_error = true;
        flags.do_print = false;
        flags.do_list = false;
        flags.do_count = false;
        flags.do_number = false;
        flags.do_ic = kSubHonorOptions;
    }
    while i < bytes.len() {
        // Note that 'g' and 'c' are always inverted, and 'r' never is.
        match bytes[i] {
            b'g' => flags.do_all = !flags.do_all,
            b'c' => flags.do_ask = !flags.do_ask,
            b'n' => flags.do_count = true,
            b'e' => flags.do_error = !flags.do_error,
            b'r' => *which_pat = RE_LAST as c_int, // use last used regexp
            b'p' => flags.do_print = true,
            b'#' => {
                flags.do_print = true;
                flags.do_number = true;
            }
            b'l' => {
                flags.do_print = true;
                flags.do_list = true;
            }
            b'i' => flags.do_ic = kSubIgnoreCase, // ignore case
            b'I' => flags.do_ic = kSubMatchCase,  // don't ignore case
            _ => break,
        }
        i += 1;
    }
    if flags.do_count {
        flags.do_ask = false;
    }
    cmd.wrapping_add(i)
}

/// Skip the `sub` part of `:s/pat/sub/`, where `delimiter` separates the
/// parts.
///
/// The closing delimiter is replaced by a NUL in place, so the replacement
/// ends where it should; an unterminated one simply runs to the end.
///
/// # Safety
/// `start` must be a live, writable, NUL-terminated string.
pub(crate) unsafe fn skip_substitute(start: *mut c_char, delimiter: c_int) -> *mut c_char {
    // SAFETY: caller's contract.
    let bytes = unsafe { CStr::from_ptr(start) }.to_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] as c_int == delimiter {
            // End delimiter found: replace it with a NUL.
            // SAFETY: `i` indexes the string the caller may write.
            unsafe { *start.add(i) = NUL as c_char };
            return start.wrapping_add(i + 1);
        }
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 1; // skip escaped characters
        }
        // SAFETY: `start + i` is a non-NUL byte of the string, so the
        // character length is at least one and stops at the terminator.
        i += unsafe { utfc_ptr2len(start.add(i)) } as usize;
    }
    start.wrapping_add(bytes.len())
}

/// Reject a delimiter that would make the command ambiguous.
///
/// # Safety
/// Message state.  `c` must be a `char` value (`-128..=255`), which is what
/// the ctype table is indexed by.
pub(crate) unsafe fn check_regexp_delim(c: c_int) -> c_int {
    // SAFETY: caller's contract -- the ctype table covers `-128..=255`.
    let isalpha = unsafe { *(*__ctype_b_loc()).offset(c as isize) } as c_int & _ISalpha as c_int;
    if isalpha != 0 {
        // SAFETY: a live message string.
        unsafe {
            emsg(gettext(
                c"E146: Regular expressions can't be delimited by letters".as_ptr(),
            ))
        };
        return FAIL;
    }
    OK
}
