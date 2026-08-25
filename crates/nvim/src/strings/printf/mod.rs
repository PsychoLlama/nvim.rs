//! The `vim_snprintf` family: entry points and argument fetchers.
//!
//! Every spelling funnels into `vim_vsnprintf_typval`, which can be handed
//! either a C `va_list` or an array of `typval_T` -- the latter being how
//! Vimscript's `printf()` passes its arguments, and the reason
//! `tv_nr`/`tv_str`/`tv_ptr`/`tv_float` exist: they read one argument out of
//! that array with the type checking C's varargs cannot do.  `kv_do_printf`
//! and `arena_printf` are the two spellings that format into a growable
//! buffer rather than a fixed one, and they go through the *libc* `vsnprintf`
//! rather than through this file's formatter.
//!
//! The variadic entry points stay variadic: turning a variadic call site into
//! a macro is phase 16's tree-wide sweep, and it has to stay mechanical.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, VaList, c_char, c_int, c_void};
use core::ptr;

use super::given;
use crate::eval::encode::encode_tv2echo;
use crate::eval::typval::{tv_get_number_chk, tv_get_string_buf_chk};
use crate::memory::{arena_alloc, arena_alloc_block, xrealloc};
use crate::message::emsg;
use crate::os::cshim::{gettext, vsnprintf};
use crate::types::{
    Arena, String_0, StringBuilder, VAR_FLOAT, VAR_NUMBER, VAR_STRING, float_T, size_t, typval_T,
    varnumber_T,
};
use ::libc::strlen;

// The carve of the transpiled module; see each child's docs.
mod emit;
mod spec;

pub use self::emit::*;

/// Raised when the format asks for an argument `printf()` was not given.
const E_INSUFFICIENT_ARGS: &CStr = c"E766: Insufficient arguments for printf()";
const E_EXPECTED_FLOAT: &CStr = c"E807: Expected Float argument for printf()";

/// The `*idxp`-th Vimscript argument, or `None` with `E766` raised.
///
/// Indexing is one-based -- the C writes `tvs[*idxp - 1]` at every fetcher
/// -- and the array is terminated by a `VAR_UNKNOWN` entry rather than by a
/// count, so that entry is the only bound there is. The index moves on only
/// when an argument was actually there.
unsafe fn next_arg(tvs: *mut typval_T, idxp: &mut c_int) -> Option<*mut typval_T> {
    unsafe {
        let tv = tvs.offset(*idxp as isize - 1);
        if !given(&*tv) {
            emsg(gettext(E_INSUFFICIENT_ARGS.as_ptr()));
            return None;
        }
        *idxp += 1;
        Some(tv)
    }
}

/// The next argument as a number; 0 if it is not one.
pub(crate) unsafe fn tv_nr(tvs: *mut typval_T, idxp: &mut c_int) -> varnumber_T {
    unsafe {
        let Some(tv) = next_arg(tvs, idxp) else {
            return 0;
        };
        let mut err = false;
        let n = tv_get_number_chk(tv, &raw mut err);
        if err { 0 } else { n }
    }
}

/// The next argument as a string.
///
/// A String is read in place and a Number is rendered into `numbuf`, which
/// the caller lends and which must outlive the answer; anything else is
/// rendered as `:echo` would render it, and `*tofree` then owns that.
///
/// # Safety
/// `numbuf` must be writable for `NUMBUFLEN` bytes.
pub(crate) unsafe fn tv_str(
    tvs: *mut typval_T,
    idxp: &mut c_int,
    tofree: &mut *mut c_char,
    numbuf: *mut c_char,
) -> *const c_char {
    unsafe {
        let Some(tv) = next_arg(tvs, idxp) else {
            return ptr::null();
        };
        if matches!((*tv).v_type, VAR_STRING | VAR_NUMBER) {
            *tofree = ptr::null_mut();
            tv_get_string_buf_chk(tv, numbuf)
        } else {
            *tofree = encode_tv2echo(tv, ptr::null_mut());
            *tofree
        }
    }
}

/// The next argument as a pointer, for `%p`.
///
/// Every pointer-shaped value -- String, List, Dict, Blob, Partial --
/// occupies the same union slot, so reading `v_string` reads all of them.
pub(crate) unsafe fn tv_ptr(tvs: *const typval_T, idxp: &mut c_int) -> *const c_void {
    unsafe {
        match next_arg(tvs.cast_mut(), idxp) {
            Some(tv) => (*tv).vval.v_string as *const c_void,
            None => ptr::null(),
        }
    }
}

/// The next argument as a float; a Number is widened, anything else is
/// `E807` and zero.
pub(crate) unsafe fn tv_float(tvs: *mut typval_T, idxp: &mut c_int) -> float_T {
    unsafe {
        let Some(tv) = next_arg(tvs, idxp) else {
            return 0.0;
        };
        match (*tv).v_type {
            VAR_FLOAT => (*tv).vval.v_float,
            VAR_NUMBER => (*tv).vval.v_number as float_T,
            _ => {
                emsg(gettext(E_EXPECTED_FLOAT.as_ptr()));
                0.0
            }
        }
    }
}

/// Append a formatted value to the string already in `str`.
pub unsafe extern "C" fn vim_snprintf_add(
    str: *mut c_char,
    str_m: size_t,
    fmt: *const c_char,
    mut args: ...
) -> c_int {
    unsafe {
        let len = strlen(str);
        let space = str_m.saturating_sub(len);
        vim_vsnprintf(str.add(len), space, fmt, args.clone())
    }
}

/// Write a formatted value to `str`.
///
/// Returns the number of bytes, excluding the NUL, that *would* have been
/// written had `str_m` been large enough — which is why it is not safe to
/// use as a buffer offset. See `vim_snprintf_safelen`.
pub unsafe extern "C" fn vim_snprintf(
    str: *mut c_char,
    str_m: size_t,
    fmt: *const c_char,
    mut args: ...
) -> c_int {
    unsafe { vim_vsnprintf(str, str_m, fmt, args.clone()) }
}

/// Like `vim_snprintf` but with a return value that can safely increment a
/// buffer length: never greater than `str_m - 1`.
pub unsafe extern "C" fn vim_snprintf_safelen(
    str: *mut c_char,
    str_m: size_t,
    fmt: *const c_char,
    mut args: ...
) -> size_t {
    unsafe {
        if str_m == 0 {
            return 0;
        }
        let str_l = vim_vsnprintf_typval(str, str_m, fmt, args.clone(), ptr::null_mut());
        if str_l < 0 {
            *str = 0;
            return 0;
        }
        (str_l as size_t).min(str_m - 1)
    }
}

pub unsafe fn vim_vsnprintf(
    str: *mut c_char,
    str_m: size_t,
    fmt: *const c_char,
    ap: VaList,
) -> c_int {
    unsafe { vim_vsnprintf_typval(str, str_m, fmt, ap, ptr::null_mut()) }
}

/// How infinity prints, for every combination of sign, forced sign, the
/// space-for-positive flag, and the conversion's case.
///
/// The index is `positive * (1 + force_sign + force_sign * space)`, which
/// is 0 for a negative value and 1, 2 or 3 for a positive one depending on
/// which sign flags are set; an uppercase conversion adds 4.
pub(crate) fn infinity_str(
    positive: bool,
    fmt_spec: c_char,
    force_sign: bool,
    space_for_positive: bool,
) -> &'static CStr {
    const TABLE: [&CStr; 8] = [
        c"-inf", c"inf", c"+inf", c" inf", c"-INF", c"INF", c"+INF", c" INF",
    ];
    let force_sign = c_int::from(force_sign);
    let mut idx =
        c_int::from(positive) * (1 + force_sign + force_sign * c_int::from(space_for_positive));
    if (fmt_spec as u8).is_ascii_uppercase() {
        idx += 4;
    }
    TABLE[idx as usize]
}

/// The scratch buffer `vim_vsnprintf_typval` renders one conversion into.
const TMP_LEN: c_int = 350;

/// `vsnprintf` into a `StringBuilder`, growing it to fit.
///
/// The first attempt formats straight into whatever room is left, so the
/// common case is one pass; only if it does not fit is the buffer grown
/// and the format run a second time.
pub unsafe extern "C" fn kv_do_printf(
    str: *mut StringBuilder,
    fmt: *const c_char,
    mut args: ...
) -> c_int {
    unsafe {
        let remaining = (*str).capacity - (*str).size;
        let tail = if (*str).items.is_null() {
            ptr::null_mut()
        } else {
            (*str).items.add((*str).size)
        };
        let mut printed = vsnprintf(tail, remaining, fmt, args.clone());
        if printed < 0 {
            return -1;
        }

        if printed as size_t >= remaining {
            // `kv_ensure_space`, with room for the terminator.
            let wanted = (*str).size + printed as size_t + 1;
            if (*str).capacity < wanted {
                // Round up to a power of two.
                let mut capacity = wanted - 1;
                for shift in [1, 2, 4, 8, 16] {
                    capacity |= capacity >> shift;
                }
                (*str).capacity = capacity + 1;
                (*str).items =
                    xrealloc((*str).items as *mut c_void, (*str).capacity) as *mut c_char;
            }
            debug_assert!(!(*str).items.is_null());
            printed = vsnprintf(
                (*str).items.add((*str).size),
                (*str).capacity - (*str).size,
                fmt,
                args.clone(),
            );
            if printed < 0 {
                return -1;
            }
        }

        (*str).size += printed as size_t;
        printed
    }
}

/// `vsnprintf` into an arena.
///
/// The happy path formats into the rest of the current block and only
/// charges the arena for what it used; if it does not fit, a block of
/// exactly the right size is taken and the format run again.
pub unsafe extern "C" fn arena_printf(
    arena: *mut Arena,
    fmt: *const c_char,
    mut args: ...
) -> String_0 {
    unsafe {
        let mut remaining: size_t = 0;
        let mut buf = ptr::null_mut::<c_char>();
        if !arena.is_null() {
            if (*arena).cur_blk.is_null() {
                arena_alloc_block(arena);
            }
            remaining = (*arena).size - (*arena).pos;
            buf = (*arena).cur_blk.add((*arena).pos);
        }

        let mut printed = vsnprintf(buf, remaining, fmt, args.clone());
        if printed < 0 {
            return String_0::NULL;
        }

        if printed as size_t >= remaining {
            buf = arena_alloc(arena, printed as size_t + 1, false) as *mut c_char;
            printed = vsnprintf(buf, printed as size_t + 1, fmt, args.clone());
            if printed < 0 {
                return String_0::NULL;
            }
        } else {
            (*arena).pos += printed as size_t + 1;
        }

        String_0::from_raw_parts(buf, printed as size_t)
    }
}
