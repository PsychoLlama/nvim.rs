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

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::{FAIL, given, strict_bool_arg};
use crate::src::nvim::eval::typval::{
    tv_check_for_number_arg, tv_check_for_opt_bool_arg, tv_check_for_opt_number_arg,
    tv_check_for_string_arg, tv_get_bool, tv_get_number, tv_get_number_chk, tv_get_string,
    tv_get_string_chk,
};
use crate::src::nvim::mbyte::{
    mb_cptr2char_adv, mb_ptr2char_adv, utf_ptr2char, utf_ptr2len, utfc_ptr2len,
};
use crate::src::nvim::memory::xmemdupz;
use crate::src::nvim::os::libc::strlen;
use crate::src::nvim::types::{EvalFuncData, VAR_STRING, int64_t, size_t, typval_T, varnumber_T};

/// The character-length function a `countcc`/`comp` flag selects: with
/// composing characters counted separately, or folded into their base.
type Ptr2Len = unsafe extern "C" fn(*const c_char) -> c_int;

fn char_len_fn(count_composing: bool) -> Ptr2Len {
    if count_composing {
        utf_ptr2len
    } else {
        utfc_ptr2len
    }
}

/// The code point at `p`, as the C reads it: through `utf_ptr2char` for a
/// multi-byte character and as a **signed** `char` otherwise, so a stray
/// byte over 0x7f is negative and never counts as a surrogate pair.
unsafe fn code_point(p: *const c_char, char_len: c_int) -> c_int {
    unsafe {
        if char_len > 1 {
            utf_ptr2char(p)
        } else {
            *p as c_int
        }
    }
}

/// `byteidx()` and `byteidxcomp()`: the byte offset of the `idx`-th
/// character, or with the third argument set, of the `idx`-th UTF-16 unit.
///
/// `comp` is the `byteidxcomp()` spelling, which counts a composing
/// character as one of its own.
unsafe fn byteidx_common(argvars: *mut typval_T, rettv: *mut typval_T, comp: bool) {
    unsafe {
        (*rettv).vval.v_number = -1;

        let str = tv_get_string_chk(argvars);
        let mut idx = tv_get_number_chk(argvars.add(1), ptr::null_mut());
        if str.is_null() || idx < 0 {
            return;
        }

        let utf16idx = if given(&*argvars.add(2)) {
            match strict_bool_arg(argvars.add(2)) {
                Some(flag) => flag,
                None => return,
            }
        } else {
            false
        };

        let char_len = char_len_fn(comp);
        let mut t = str;
        while idx > 0 {
            if *t == 0 {
                return; // End of string before the index was reached.
            }
            if utf16idx {
                let clen = char_len(t);
                if code_point(t, clen) > 0xffff {
                    idx -= 1;
                }
                // The last unit of a surrogate pair leaves `t` on the
                // character it belongs to, which is the answer.
                if idx > 0 {
                    t = t.offset(clen as isize);
                }
            } else {
                t = t.offset(char_len(t) as isize);
            }
            idx -= 1;
        }
        (*rettv).vval.v_number = t.offset_from(str) as varnumber_T;
    }
}

/// "byteidx()" function
pub unsafe extern "C" fn f_byteidx(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe { byteidx_common(argvars, rettv, false) }
}

/// "byteidxcomp()" function
pub unsafe extern "C" fn f_byteidxcomp(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe { byteidx_common(argvars, rettv, true) }
}

/// "charidx()" function: the character index of a byte (or UTF-16) offset.
pub unsafe extern "C" fn f_charidx(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1;

        if tv_check_for_string_arg(argvars, 0) == FAIL
            || tv_check_for_number_arg(argvars, 1) == FAIL
            || tv_check_for_opt_bool_arg(argvars, 2) == FAIL
            || (given(&*argvars.add(2)) && tv_check_for_opt_bool_arg(argvars, 3) == FAIL)
        {
            return;
        }

        let str = tv_get_string_chk(argvars);
        let mut idx = tv_get_number_chk(argvars.add(1), ptr::null_mut());
        if str.is_null() || idx < 0 {
            return;
        }

        let mut countcc = false;
        let mut utf16idx = false;
        if given(&*argvars.add(2)) {
            countcc = tv_get_bool(argvars.add(2)) != 0;
            if given(&*argvars.add(3)) {
                utf16idx = tv_get_bool(argvars.add(3)) != 0;
            }
        }

        let char_len = char_len_fn(countcc);
        let mut p = str;
        let mut len: c_int = 0;
        while if utf16idx {
            idx >= 0
        } else {
            p <= str.offset(idx as isize)
        } {
            if *p == 0 {
                // An index of exactly the string's length in bytes (or
                // UTF-16 units) answers the string's length in characters.
                if if utf16idx {
                    idx == 0
                } else {
                    p == str.offset(idx as isize)
                } {
                    (*rettv).vval.v_number = len as varnumber_T;
                }
                return;
            }
            if utf16idx {
                idx -= 1;
                if code_point(p, char_len(p)) > 0xffff {
                    idx -= 1;
                }
            }
            p = p.offset(char_len(p) as isize);
            len += 1;
        }

        (*rettv).vval.v_number = (len - 1).max(0) as varnumber_T;
    }
}

/// "strgetchar()" function: the code point of the `idx`-th character.
pub unsafe extern "C" fn f_strgetchar(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1;

        let str = tv_get_string_chk(argvars);
        if str.is_null() {
            return;
        }
        let mut error = false;
        let mut charidx = tv_get_number_chk(argvars.add(1), &raw mut error);
        if error {
            return;
        }

        let len = strlen(str);
        let mut byteidx: size_t = 0;
        while charidx >= 0 && byteidx < len {
            if charidx == 0 {
                (*rettv).vval.v_number = utf_ptr2char(str.add(byteidx)) as varnumber_T;
                break;
            }
            charidx -= 1;
            byteidx += utf_ptr2len(str.add(byteidx)) as size_t;
        }
    }
}

/// "strutf16len()" function: the string's length in UTF-16 code units.
pub unsafe extern "C" fn f_strutf16len(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1;

        if tv_check_for_string_arg(argvars, 0) == FAIL
            || tv_check_for_opt_bool_arg(argvars, 1) == FAIL
        {
            return;
        }
        let countcc = given(&*argvars.add(1)) && tv_get_bool(argvars.add(1)) != 0;

        let next_char: unsafe extern "C" fn(*mut *const c_char) -> c_int = if countcc {
            mb_cptr2char_adv
        } else {
            mb_ptr2char_adv
        };

        let mut s = tv_get_string(argvars);
        let mut len: varnumber_T = 0;
        while *s != 0 {
            // Anything over U+FFFF is a surrogate pair: two units.
            len += 1 + varnumber_T::from(next_char(&raw mut s) > 0xffff);
        }
        (*rettv).vval.v_number = len;
    }
}

/// "strcharpart()" function: a substring measured in characters.
pub unsafe extern "C" fn f_strcharpart(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        let p = tv_get_string(argvars);
        let slen = strlen(p);

        let mut nbyte: c_int = 0;
        let mut skipcc = false;
        let mut error = false;
        let mut nchar = tv_get_number_chk(argvars.add(1), &raw mut error);
        if !error {
            if given(&*argvars.add(2)) && given(&*argvars.add(3)) {
                match strict_bool_arg(argvars.add(3)) {
                    Some(flag) => skipcc = flag,
                    None => return,
                }
            }
            if nchar > 0 {
                // Walk `nchar` characters in to find the byte offset.
                while nchar > 0 && (nbyte as size_t) < slen {
                    nbyte += char_len_fn(!skipcc)(p.offset(nbyte as isize));
                    nchar -= 1;
                }
            } else {
                // A negative start is already a byte offset, and stays
                // negative until the overlap is taken below.
                nbyte = nchar as c_int;
            }
        }

        let mut len: c_int = if given(&*argvars.add(2)) {
            let mut charlen = tv_get_number(argvars.add(2)) as c_int;
            let mut len = 0;
            while charlen > 0 && nbyte + len < slen as c_int {
                let off = nbyte + len;
                // Offsets before the string count one byte each, so a
                // negative start still consumes its share of `charlen`.
                len += if off < 0 {
                    1
                } else {
                    char_len_fn(!skipcc)(p.offset(off as isize))
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

        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string =
            xmemdupz(p.offset(nbyte as isize) as *const c_void, len as size_t) as *mut c_char;
    }
}

/// "strpart()" function: a substring measured in bytes, or -- with the
/// fourth argument -- in characters starting from a byte offset.
pub unsafe extern "C" fn f_strpart(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        let mut error = false;
        let p = tv_get_string(argvars);
        let slen = strlen(p) as varnumber_T;

        let mut n = tv_get_number_chk(argvars.add(1), &raw mut error);
        let mut len = if error {
            0
        } else if given(&*argvars.add(2)) {
            tv_get_number(argvars.add(2))
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

        if given(&*argvars.add(2)) && given(&*argvars.add(3)) {
            // `len` was a character count after all: re-measure it.
            let mut off = n as int64_t;
            while off < slen as int64_t && len > 0 {
                off += utfc_ptr2len(p.offset(off as isize)) as int64_t;
                len -= 1;
            }
            len = (off - n as int64_t) as varnumber_T;
        }

        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string =
            xmemdupz(p.offset(n as isize) as *const c_void, len as size_t) as *mut c_char;
    }
}

/// "utf16idx()" function: the UTF-16 index of a byte (or character) offset.
pub unsafe extern "C" fn f_utf16idx(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1;

        if tv_check_for_string_arg(argvars, 0) == FAIL
            || tv_check_for_opt_number_arg(argvars, 1) == FAIL
            || tv_check_for_opt_bool_arg(argvars, 2) == FAIL
            || (given(&*argvars.add(2)) && tv_check_for_opt_bool_arg(argvars, 3) == FAIL)
        {
            return;
        }

        let str = tv_get_string_chk(argvars);
        let mut idx = tv_get_number_chk(argvars.add(1), ptr::null_mut());
        if str.is_null() || idx < 0 {
            return;
        }

        let mut countcc = false;
        let mut charidx = false;
        if given(&*argvars.add(2)) {
            countcc = tv_get_bool(argvars.add(2)) != 0;
            if given(&*argvars.add(3)) {
                charidx = tv_get_bool(argvars.add(3)) != 0;
            }
        }

        let char_len = char_len_fn(countcc);
        let mut p = str;
        let mut len: c_int = 0;
        // The answer is the index of the *start* of the character the offset
        // lands in, so it trails `len` by one iteration.
        let mut utf16idx: c_int = 0;
        while if charidx {
            idx >= 0
        } else {
            p <= str.offset(idx as isize)
        } {
            if *p == 0 {
                // An index of exactly the string's length in bytes (or
                // characters) answers its length in UTF-16 units.
                if if charidx {
                    idx == 0
                } else {
                    p == str.offset(idx as isize)
                } {
                    (*rettv).vval.v_number = len as varnumber_T;
                }
                return;
            }
            utf16idx = len;
            let clen = char_len(p);
            if code_point(p, clen) > 0xffff {
                len += 1;
            }
            p = p.offset(clen as isize);
            if charidx {
                idx -= 1;
            }
            len += 1;
        }

        (*rettv).vval.v_number = utf16idx as varnumber_T;
    }
}
