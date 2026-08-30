//! Index conversion, and the substring builtins built on it.
//!
//! Vimscript addresses a string three ways -- by byte, by character and by
//! UTF-16 code unit -- and this is every conversion between them:
//! `byteidx()`/`byteidxcomp()`/`charidx()`/`utf16idx()` and the `strutf16len()`
//! that counts them.  `strgetchar()`, `strcharpart()` and `strpart()` are the
//! substring extractors that take their bounds in those units.
//!
//! Two axes recur.  **Composing characters** either belong to the base
//! character (`utfc_ptr2len`) or count on their own (`utf_ptr2len`), which is
//! the difference between `byteidx()` and `byteidxcomp()` and the meaning of
//! every `countcc`/`skipcc` argument.  **A code point above U+FFFF is two
//! UTF-16 units**, which is the only reason the utf16 walks differ from the
//! character walks at all.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::{given, strict_bool_arg};
use crate::eval::typval::{
    NumBuf, tv_check_for_number_arg, tv_check_for_opt_bool_arg, tv_check_for_opt_number_arg,
    tv_check_for_string_arg, tv_get_bool, tv_get_number, tv_get_number_chk,
};
use crate::mbyte::{mb_cptr2char_adv, mb_ptr2char_adv, utf_ptr2char, utf_ptr2len, utfc_ptr2len};
use crate::memory::xmemdupz;
use crate::types::{EvalFuncData, VAR_STRING, int64_t, size_t, typval_T, varnumber_T};

/// The character-length rule a `countcc`/`comp` flag selects: composing
/// characters counted separately, or folded into their base.
///
/// This was a function pointer until `utfc_ptr2len` shed its C ABI and the
/// two stopped having the same type. A choice between two rules is what the
/// flag actually is, and it reads better at the call sites than a pointer
/// did.
#[derive(Clone, Copy)]
struct CharLen {
    count_composing: bool,
}

impl CharLen {
    fn new(count_composing: bool) -> CharLen {
        CharLen { count_composing }
    }

    /// The length of the character at `p` under this rule.
    ///
    /// # Safety
    ///
    /// `p` must point at a NUL-terminated string.
    unsafe fn of(self, p: *const c_char) -> c_int {
        if self.count_composing {
            // SAFETY: the caller's contract.
            unsafe { utf_ptr2len(p) }
        } else {
            // SAFETY: the caller's contract.
            unsafe { utfc_ptr2len(p) }
        }
    }
}

/// The code point at `p`, as the C reads it: through `utf_ptr2char` for a
/// multi-byte character and as a **signed** `char` otherwise, so a stray
/// byte over 0x7f is negative and never counts as a surrogate pair.
unsafe fn code_point(p: *const c_char, char_len: c_int) -> c_int {
    if char_len > 1 {
        // SAFETY: the caller's contract.
        unsafe { utf_ptr2char(p) }
    } else {
        // SAFETY: as above; one readable byte is enough here.
        unsafe { *p as c_int }
    }
}

/// `byteidx()` and `byteidxcomp()`: the byte offset of the `idx`-th
/// character, or with the third argument set, of the `idx`-th UTF-16 unit.
///
/// `comp` is the `byteidxcomp()` spelling, which counts a composing
/// character as one of its own.
unsafe fn byteidx_common(argvars: *mut typval_T, rettv: *mut typval_T, comp: bool) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).vval.v_number = -1 };

    let str = unsafe { numbuf.string_chk(argvars) };
    let mut idx = unsafe { tv_get_number_chk(argvars.add(1), ptr::null_mut()) };
    if str.is_null() || idx < 0 {
        return;
    }

    let utf16idx = if given(unsafe { &*argvars.add(2) }) {
        match unsafe { strict_bool_arg(argvars.add(2)) } {
            Some(flag) => flag,
            None => return,
        }
    } else {
        false
    };

    let char_len = CharLen::new(comp);
    let mut t = str;
    while idx > 0 {
        if unsafe { *t } == 0 {
            return; // End of string before the index was reached.
        }
        if utf16idx {
            let clen = unsafe { char_len.of(t) };
            if unsafe { code_point(t, clen) } > 0xffff {
                idx -= 1;
            }
            // The last unit of a surrogate pair leaves `t` on the
            // character it belongs to, which is the answer.
            if idx > 0 {
                t = unsafe { t.offset(clen as isize) };
            }
        } else {
            t = unsafe { t.offset(char_len.of(t) as isize) };
        }
        idx -= 1;
    }
    unsafe { (*rettv).vval.v_number = t.offset_from(str) as varnumber_T };
}

/// "byteidx()" function
pub unsafe fn f_byteidx(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { byteidx_common(argvars, rettv, false) }
}

/// "byteidxcomp()" function
pub unsafe fn f_byteidxcomp(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { byteidx_common(argvars, rettv, true) }
}

/// "charidx()" function: the character index of a byte (or UTF-16) offset.
pub unsafe fn f_charidx(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).vval.v_number = -1 };

    if unsafe { tv_check_for_string_arg(argvars, 0) }.is_err()
        || unsafe { tv_check_for_number_arg(argvars, 1) }.is_err()
        || unsafe { tv_check_for_opt_bool_arg(argvars, 2) }.is_err()
        || (given(unsafe { &*argvars.add(2) })
            && unsafe { tv_check_for_opt_bool_arg(argvars, 3) }.is_err())
    {
        return;
    }

    let str = unsafe { numbuf.string_chk(argvars) };
    let mut idx = unsafe { tv_get_number_chk(argvars.add(1), ptr::null_mut()) };
    if str.is_null() || idx < 0 {
        return;
    }

    let mut countcc = false;
    let mut utf16idx = false;
    if given(unsafe { &*argvars.add(2) }) {
        countcc = unsafe { tv_get_bool(argvars.add(2)) } != 0;
        if given(unsafe { &*argvars.add(3) }) {
            utf16idx = unsafe { tv_get_bool(argvars.add(3)) } != 0;
        }
    }

    let char_len = CharLen::new(countcc);
    let mut p = str;
    let mut len: c_int = 0;
    while if utf16idx {
        idx >= 0
    } else {
        p <= unsafe { str.offset(idx as isize) }
    } {
        if unsafe { *p } == 0 {
            // An index of exactly the string's length in bytes (or
            // UTF-16 units) answers the string's length in characters.
            if if utf16idx {
                idx == 0
            } else {
                p == unsafe { str.offset(idx as isize) }
            } {
                unsafe { (*rettv).vval.v_number = len as varnumber_T };
            }
            return;
        }
        if utf16idx {
            idx -= 1;
            if unsafe { code_point(p, char_len.of(p)) } > 0xffff {
                idx -= 1;
            }
        }
        p = unsafe { p.offset(char_len.of(p) as isize) };
        len += 1;
    }

    unsafe { (*rettv).vval.v_number = (len - 1).max(0) as varnumber_T };
}

/// "strgetchar()" function: the code point of the `idx`-th character.
pub unsafe fn f_strgetchar(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).vval.v_number = -1 };

    let str = unsafe { numbuf.string_chk(argvars) };
    if str.is_null() {
        return;
    }
    let mut error = false;
    let mut charidx = unsafe { tv_get_number_chk(argvars.add(1), &raw mut error) };
    if error {
        return;
    }

    let len = unsafe { cstr::bytes_at(str) }.len();
    let mut byteidx: size_t = 0;
    while charidx >= 0 && byteidx < len {
        if charidx == 0 {
            unsafe { (*rettv).vval.v_number = utf_ptr2char(str.add(byteidx)) as varnumber_T };
            break;
        }
        charidx -= 1;
        byteidx += unsafe { utf_ptr2len(str.add(byteidx)) as size_t };
    }
}

/// "strutf16len()" function: the string's length in UTF-16 code units.
pub unsafe fn f_strutf16len(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).vval.v_number = -1 };

    if unsafe { tv_check_for_string_arg(argvars, 0) }.is_err()
        || unsafe { tv_check_for_opt_bool_arg(argvars, 1) }.is_err()
    {
        return;
    }
    let countcc = given(unsafe { &*argvars.add(1) }) && unsafe { tv_get_bool(argvars.add(1)) } != 0;

    let next_char: unsafe fn(*mut *const c_char) -> c_int = if countcc {
        mb_cptr2char_adv
    } else {
        mb_ptr2char_adv
    };

    let mut s = unsafe { numbuf.string(argvars) };
    let mut len: varnumber_T = 0;
    while unsafe { *s } != 0 {
        // Anything over U+FFFF is a surrogate pair: two units.
        len += 1 + varnumber_T::from(unsafe { next_char(&raw mut s) } > 0xffff);
    }
    unsafe { (*rettv).vval.v_number = len };
}

/// "strcharpart()" function: a substring measured in characters.
pub unsafe fn f_strcharpart(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let p = unsafe { numbuf.string(argvars) };
    let slen = unsafe { cstr::bytes_at(p) }.len();

    let mut nbyte: c_int = 0;
    let mut skipcc = false;
    let mut error = false;
    let mut nchar = unsafe { tv_get_number_chk(argvars.add(1), &raw mut error) };
    if !error {
        if given(unsafe { &*argvars.add(2) }) && given(unsafe { &*argvars.add(3) }) {
            match unsafe { strict_bool_arg(argvars.add(3)) } {
                Some(flag) => skipcc = flag,
                None => return,
            }
        }
        if nchar > 0 {
            // Walk `nchar` characters in to find the byte offset.
            while nchar > 0 && (nbyte as size_t) < slen {
                nbyte += unsafe { CharLen::new(!skipcc).of(p.offset(nbyte as isize)) };
                nchar -= 1;
            }
        } else {
            // A negative start is already a byte offset, and stays
            // negative until the overlap is taken below.
            nbyte = nchar as c_int;
        }
    }

    let mut len: c_int = if given(unsafe { &*argvars.add(2) }) {
        let mut charlen = unsafe { tv_get_number(argvars.add(2)) as c_int };
        let mut len = 0;
        while charlen > 0 && nbyte + len < slen as c_int {
            let off = nbyte + len;
            // Offsets before the string count one byte each, so a
            // negative start still consumes its share of `charlen`.
            len += if off < 0 {
                1
            } else {
                unsafe { CharLen::new(!skipcc).of(p.offset(off as isize)) }
            };
            charlen -= 1;
        }
        len
    } else {
        slen as c_int - nbyte // Default: everything from `nbyte` on.
    };

    // Only the overlap between the requested part and the string.
    if nbyte < 0 {
        len += nbyte;
        nbyte = 0;
    } else if nbyte as size_t > slen {
        nbyte = slen as c_int;
    }
    if len < 0 {
        len = 0;
    } else if nbyte + len > slen as c_int {
        len = slen as c_int - nbyte;
    }

    unsafe { (*rettv).v_type = VAR_STRING };
    let from = unsafe { p.offset(nbyte as isize) } as *const c_void;
    let part = unsafe { xmemdupz(from, len as size_t) } as *mut c_char;
    unsafe { (*rettv).vval.v_string = part };
}

/// "strpart()" function: a substring measured in bytes, or -- with the
/// fourth argument -- in characters starting from a byte offset.
pub unsafe fn f_strpart(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut error = false;
    let p = unsafe { numbuf.string(argvars) };
    let slen = unsafe { cstr::bytes_at(p).len() as varnumber_T };

    let mut n = unsafe { tv_get_number_chk(argvars.add(1), &raw mut error) };
    let mut len = if error {
        0
    } else if given(unsafe { &*argvars.add(2) }) {
        unsafe { tv_get_number(argvars.add(2)) }
    } else {
        slen - n // Default: everything from `n` on.
    };

    // Only the overlap between the requested part and the string.
    if n < 0 {
        len += n;
        n = 0;
    } else if n > slen {
        n = slen;
    }
    if len < 0 {
        len = 0;
    } else if n + len > slen {
        len = slen - n;
    }

    if given(unsafe { &*argvars.add(2) }) && given(unsafe { &*argvars.add(3) }) {
        // `len` was a character count after all: re-measure it.
        let mut off = n as int64_t;
        while off < slen as int64_t && len > 0 {
            off += unsafe { utfc_ptr2len(p.offset(off as isize)) as int64_t };
            len -= 1;
        }
        len = (off - n as int64_t) as varnumber_T;
    }

    unsafe { (*rettv).v_type = VAR_STRING };
    let from = unsafe { p.offset(n as isize) } as *const c_void;
    let part = unsafe { xmemdupz(from, len as size_t) } as *mut c_char;
    unsafe { (*rettv).vval.v_string = part };
}

/// "utf16idx()" function: the UTF-16 index of a byte (or character) offset.
pub unsafe fn f_utf16idx(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    unsafe { (*rettv).vval.v_number = -1 };

    if unsafe { tv_check_for_string_arg(argvars, 0) }.is_err()
        || unsafe { tv_check_for_opt_number_arg(argvars, 1) }.is_err()
        || unsafe { tv_check_for_opt_bool_arg(argvars, 2) }.is_err()
        || (given(unsafe { &*argvars.add(2) })
            && unsafe { tv_check_for_opt_bool_arg(argvars, 3) }.is_err())
    {
        return;
    }

    let str = unsafe { numbuf.string_chk(argvars) };
    let mut idx = unsafe { tv_get_number_chk(argvars.add(1), ptr::null_mut()) };
    if str.is_null() || idx < 0 {
        return;
    }

    let mut countcc = false;
    let mut charidx = false;
    if given(unsafe { &*argvars.add(2) }) {
        countcc = unsafe { tv_get_bool(argvars.add(2)) } != 0;
        if given(unsafe { &*argvars.add(3) }) {
            charidx = unsafe { tv_get_bool(argvars.add(3)) } != 0;
        }
    }

    let char_len = CharLen::new(countcc);
    let mut p = str;
    let mut len: c_int = 0;
    // The answer is the index of the *start* of the character the offset
    // lands in, so it trails `len` by one iteration.
    let mut utf16idx: c_int = 0;
    while if charidx {
        idx >= 0
    } else {
        p <= unsafe { str.offset(idx as isize) }
    } {
        if unsafe { *p } == 0 {
            // An index of exactly the string's length in bytes (or
            // characters) answers its length in UTF-16 units.
            if if charidx {
                idx == 0
            } else {
                p == unsafe { str.offset(idx as isize) }
            } {
                unsafe { (*rettv).vval.v_number = len as varnumber_T };
            }
            return;
        }
        utf16idx = len;
        let clen = unsafe { char_len.of(p) };
        if unsafe { code_point(p, clen) } > 0xffff {
            len += 1;
        }
        p = unsafe { p.offset(clen as isize) };
        if charidx {
            idx -= 1;
        }
        len += 1;
    }

    unsafe { (*rettv).vval.v_number = utf16idx as varnumber_T };
}
