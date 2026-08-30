//! Escaping and quoting.
//!
//! `vim_strsave_escaped_ext` is the general one -- a copy with a chosen escape
//! character inserted before every byte in a set, optionally including the
//! backslashes `rem_backslash()` would eat -- and `vim_strsave_shellescape` the
//! specialised one, which wraps a string in single quotes for `system()` and
//! knows what csh and fish need escaped on top of that.  `vim_strnsave_unquoted`
//! is the inverse used by `shell_build_argv`: it resolves shell double-quoting,
//! `unquote` being the byte walk both of its passes share.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{c_char, c_int};
use core::{ptr, slice};

use super::vim_strchr;
use crate::charset::rem_backslash;
use crate::ex_docmd::find_cmdline_var;
use crate::mbyte::{mb_copy_char, utfc_ptr2len};
use crate::memory::{xmalloc, xmallocz};
use crate::option::{csh_like_shell, fish_like_shell};
use crate::types::size_t;

/// Walk `src` as `vim_strnsave_unquoted` reads it, feeding kept bytes to
/// `emit`: unescaped double quotes toggle quote mode and vanish; inside
/// quotes `\\` and `\"` collapse to the escaped byte; everything else
/// (including other backslash sequences) passes through untouched.
pub(crate) fn unquote(src: &[u8], emit: &mut impl FnMut(u8)) {
    let mut inquote = false;
    let mut i = 0;
    while i < src.len() {
        let b = src[i];
        if b == b'"' {
            inquote = !inquote;
        } else if b == b'\\' && inquote && i + 1 < src.len() && matches!(src[i + 1], b'\\' | b'"') {
            i += 1;
            emit(src[i]);
        } else {
            emit(b);
        }
        i += 1;
    }
}

pub unsafe fn vim_strsave_escaped(string: *const c_char, esc_chars: *const c_char) -> *mut c_char {
    unsafe { vim_strsave_escaped_ext(string, esc_chars, b'\\' as c_char, false) }
}

/// Copy `string`, prefixing `cc` to every byte in `esc_chars` (and, with
/// `bsl`, to the backslashes `rem_backslash` flags). Multibyte characters
/// are copied whole and never escaped.
pub unsafe fn vim_strsave_escaped_ext(
    string: *const c_char,
    esc_chars: *const c_char,
    cc: c_char,
    bsl: bool,
) -> *mut c_char {
    // First pass: measure (1 for the terminating NUL).
    let mut length: size_t = 1;
    let mut p = string;
    while unsafe { *p } != 0 {
        let l = unsafe { utfc_ptr2len(p) as size_t };
        if l > 1 {
            length = length.wrapping_add(l);
            p = unsafe { p.add(l) };
            continue;
        }
        if !unsafe { vim_strchr(esc_chars, *p as u8 as c_int) }.is_null()
            || (bsl && unsafe { rem_backslash(p) })
        {
            length = length.wrapping_add(1);
        }
        length = length.wrapping_add(1);
        p = unsafe { p.add(1) };
    }

    let escaped_string = unsafe { xmalloc(length) as *mut c_char };
    let mut p2 = escaped_string;
    let mut p = string;
    while unsafe { *p } != 0 {
        let l = unsafe { utfc_ptr2len(p) as size_t };
        if l > 1 {
            unsafe { ptr::copy_nonoverlapping(p, p2, l) };
            p2 = unsafe { p2.add(l) };
            p = unsafe { p.add(l) };
            continue;
        }
        if !unsafe { vim_strchr(esc_chars, *p as u8 as c_int) }.is_null()
            || (bsl && unsafe { rem_backslash(p) })
        {
            unsafe { *p2 = cc };
            p2 = unsafe { p2.add(1) };
        }
        unsafe { *p2 = *p };
        p2 = unsafe { p2.add(1) };
        p = unsafe { p.add(1) };
    }
    unsafe { *p2 = 0 };
    escaped_string
}

/// Copy `length` bytes of `string` with shell-style double-quoting
/// resolved (see `unquote`), NUL-terminated.
pub unsafe fn vim_strnsave_unquoted(string: *const c_char, length: size_t) -> *mut c_char {
    if length == 0 {
        return unsafe { xmallocz(0) as *mut c_char };
    }
    let src = unsafe { slice::from_raw_parts(string as *const u8, length) };
    let mut n: size_t = 0;
    unquote(src, &mut |_| n += 1);
    let ret = unsafe { xmallocz(n) as *mut c_char };
    let out = unsafe { slice::from_raw_parts_mut(ret as *mut u8, n) };
    let mut o = 0;
    unquote(src, &mut |b| {
        out[o] = b;
        o += 1;
    });
    ret
}

/// Single-quote `string` for the shell, doubling embedded quotes
/// (`'` → `'\''`) and — depending on the shell flavor and flags — escaping
/// newlines, `!`, `\`, and `%`/`#` cmdline specials.
pub unsafe fn vim_strsave_shellescape(
    string: *const c_char,
    do_special: bool,
    do_newline: bool,
) -> *mut c_char {
    let csh_like = csh_like_shell();
    let fish_like = fish_like_shell();
    let mut l: size_t = 0;

    // First pass: measure (3 = the surrounding quotes plus NUL).
    let mut length: size_t = unsafe { cstr::bytes_at(string) }.len().wrapping_add(3);
    let mut p = string;
    while unsafe { *p } != 0 {
        if unsafe { *p } == b'\'' as c_char {
            length = length.wrapping_add(3);
        }
        if (unsafe { *p } == b'\n' as c_char && (csh_like || do_newline))
            || (unsafe { *p } == b'!' as c_char && (csh_like || do_special))
        {
            length = length.wrapping_add(1);
            if csh_like && do_special {
                length = length.wrapping_add(1);
            }
        }
        if do_special && unsafe { find_cmdline_var(p, &mut l) } >= 0 {
            length = length.wrapping_add(1); // insert backslash
            p = unsafe { p.add(l.wrapping_sub(1)) };
        }
        if unsafe { *p } == b'\\' as c_char && fish_like {
            length = length.wrapping_add(1);
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }

    let escaped_string = unsafe { xmalloc(length) as *mut c_char };
    let mut d = escaped_string;
    unsafe { *d = b'\'' as c_char };
    d = unsafe { d.add(1) };
    let mut p = string;
    while unsafe { *p } != 0 {
        if unsafe { *p } == b'\'' as c_char {
            // A single-quoted string cannot contain a quote: close it,
            // emit an escaped quote, and reopen.
            for &b in b"'\\''" {
                unsafe { *d = b as c_char };
                d = unsafe { d.add(1) };
            }
            p = unsafe { p.add(1) };
            continue;
        }
        if (unsafe { *p } == b'\n' as c_char && (csh_like || do_newline))
            || (unsafe { *p } == b'!' as c_char && (csh_like || do_special))
        {
            unsafe { *d = b'\\' as c_char };
            d = unsafe { d.add(1) };
            if csh_like && do_special {
                unsafe { *d = b'\\' as c_char };
                d = unsafe { d.add(1) };
            }
            unsafe { *d = *p };
            d = unsafe { d.add(1) };
            p = unsafe { p.add(1) };
            continue;
        }
        if do_special && unsafe { find_cmdline_var(p, &mut l) } >= 0 {
            unsafe { *d = b'\\' as c_char }; // insert backslash
            d = unsafe { d.add(1) };
            unsafe { ptr::copy_nonoverlapping(p, d, l) }; // copy the var
            d = unsafe { d.add(l) };
            p = unsafe { p.add(l) };
            continue;
        }
        if unsafe { *p } == b'\\' as c_char && fish_like {
            unsafe { *d = b'\\' as c_char };
            d = unsafe { d.add(1) };
            unsafe { *d = *p };
            d = unsafe { d.add(1) };
            p = unsafe { p.add(1) };
            continue;
        }
        unsafe { mb_copy_char(&mut p, &mut d) };
    }
    unsafe { *d = b'\'' as c_char };
    d = unsafe { d.add(1) };
    unsafe { *d = 0 };
    escaped_string
}
