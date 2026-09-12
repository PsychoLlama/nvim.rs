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
#![allow(unsafe_code)]

use crate::cstr;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::strict_bool_arg;
use crate::eval::typval::{
    NumBuf, tv_check_for_number_arg, tv_check_for_opt_bool_arg, tv_check_for_opt_number_arg,
    tv_check_for_string_arg, tv_get_bool, tv_get_number, tv_get_number_chk,
};
use crate::mbyte::{char_at, char_len, cluster_len, mb_cptr2char_adv, mb_ptr2char_adv};
use crate::memory::xmemdupz;
use crate::types::{EvalFuncData, TypVal, VAR_STRING, VarNumber, int64_t, size_t};

/// The character-length rule a `countcc`/`comp` flag selects: composing
/// characters counted separately, or folded into their base.
///
/// This was a function pointer until `utfc_ptr2len` shed its C ABI and the
/// two stopped having the same type. A choice between two rules is what the
/// flag actually is, and it reads better at the call sites than a pointer
/// did.
#[derive(Clone, Copy)]
struct CharLen {
    separate_composing: bool,
}

impl CharLen {
    fn new(separate_composing: bool) -> CharLen {
        CharLen { separate_composing }
    }

    /// The length of the character at the start of `bytes` under this rule.
    fn of(self, bytes: &[u8]) -> usize {
        if self.separate_composing {
            char_len(bytes)
        } else {
            cluster_len(bytes)
        }
    }
}

/// The code point at the start of `bytes`, as the C reads it: decoded for a
/// multi-byte character and as a **signed** `char` otherwise, so a stray
/// byte over 0x7f is negative and never counts as a surrogate pair.
fn code_point(bytes: &[u8], char_len: usize) -> c_int {
    if char_len > 1 {
        char_at(bytes)
    } else {
        c_int::from(bytes[0].cast_signed())
    }
}

/// `byteidx()` and `byteidxcomp()`: the byte offset of the `idx`-th
/// character, or with the third argument set, of the `idx`-th UTF-16 unit.
///
/// `comp` is the `byteidxcomp()` spelling, which counts a composing
/// character as one of its own.
///
/// # Safety
///
/// `args` must point at an initialized typval, unaliased for the call.
/// `result` must point at the caller's return slot: an initialized typval it
/// owns and will clear.
unsafe fn byteidx_common(args: &[TypVal], result: &mut TypVal, comp: bool) {
    let mut numbuf = NumBuf::new();
    (*result).write_number(-1);

    let str = unsafe { numbuf.string_chk(&args[0]) };
    let mut idx = unsafe { tv_get_number_chk(&args[1], ptr::null_mut()) };
    if str.is_null() || idx < 0 {
        return;
    }

    let utf16idx = if args.len() > 2 {
        match unsafe { strict_bool_arg(&args[2]) } {
            Some(flag) => flag,
            None => return,
        }
    } else {
        false
    };

    let char_len = CharLen::new(comp);
    // SAFETY: `string_chk` answered a NUL-terminated string.
    let bytes = unsafe { cstr::bytes_at(str) };
    let mut at = 0;
    while idx > 0 {
        let Some(rest) = bytes.get(at..).filter(|rest| !rest.is_empty()) else {
            return; // End of string before the index was reached.
        };
        if utf16idx {
            let clen = char_len.of(rest);
            if code_point(rest, clen) > 0xffff {
                idx -= 1;
            }
            // The last unit of a surrogate pair leaves the cursor on the
            // character it belongs to, which is the answer.
            if idx > 0 {
                at += clen;
            }
        } else {
            at += char_len.of(rest);
        }
        idx -= 1;
    }
    (*result).write_number(at as VarNumber);
}

/// "byteidx()" function
pub fn f_byteidx(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    unsafe { byteidx_common(args, result, false) }
}

/// "byteidxcomp()" function
pub fn f_byteidxcomp(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    unsafe { byteidx_common(args, result, true) }
}

/// "charidx()" function: the character index of a byte (or UTF-16) offset.
pub fn f_charidx(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);

    if tv_check_for_string_arg(args, 0).is_err()
        || tv_check_for_number_arg(args, 1).is_err()
        || tv_check_for_opt_bool_arg(args, 2).is_err()
        || (args.len() > 2 && tv_check_for_opt_bool_arg(args, 3).is_err())
    {
        return;
    }

    let str = unsafe { numbuf.string_chk(&args[0]) };
    let mut idx = unsafe { tv_get_number_chk(&args[1], ptr::null_mut()) };
    if str.is_null() || idx < 0 {
        return;
    }

    let mut countcc = false;
    let mut utf16idx = false;
    if args.len() > 2 {
        countcc = tv_get_bool(&args[2]) != 0;
        if args.len() > 3 {
            utf16idx = tv_get_bool(&args[3]) != 0;
        }
    }

    let char_len = CharLen::new(countcc);
    // SAFETY: the argument was checked to be a string, so it is
    // NUL-terminated.
    let bytes = unsafe { cstr::bytes_at(str) };
    let mut at: VarNumber = 0;
    let mut len: c_int = 0;
    while if utf16idx { idx >= 0 } else { at <= idx } {
        let rest = &bytes[at as usize..];
        if rest.is_empty() {
            // An index of exactly the string's length in bytes (or
            // UTF-16 units) answers the string's length in characters.
            if if utf16idx { idx == 0 } else { at == idx } {
                result.write_number(VarNumber::from(len));
            }
            return;
        }
        if utf16idx {
            idx -= 1;
            if code_point(rest, char_len.of(rest)) > 0xffff {
                idx -= 1;
            }
        }
        at += char_len.of(rest) as VarNumber;
        len += 1;
    }

    result.write_number((len - 1).max(0) as VarNumber);
}

/// "strgetchar()" function: the code point of the `idx`-th character.
pub fn f_strgetchar(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);

    let str = unsafe { numbuf.string_chk(&args[0]) };
    if str.is_null() {
        return;
    }
    let mut error = false;
    let mut charidx = unsafe { tv_get_number_chk(&args[1], &raw mut error) };
    if error {
        return;
    }

    // SAFETY: `string_chk` answered a NUL-terminated string.
    let bytes = unsafe { cstr::bytes_at(str) };
    let mut byteidx: size_t = 0;
    while charidx >= 0 && byteidx < bytes.len() {
        if charidx == 0 {
            result.write_number(VarNumber::from(char_at(&bytes[byteidx..])));
            break;
        }
        charidx -= 1;
        byteidx += char_len(&bytes[byteidx..]);
    }
}

/// "strutf16len()" function: the string's length in UTF-16 code units.
pub fn f_strutf16len(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);

    if tv_check_for_string_arg(args, 0).is_err() || tv_check_for_opt_bool_arg(args, 1).is_err() {
        return;
    }
    let countcc = args.len() > 1 && tv_get_bool(&args[1]) != 0;

    let next_char: unsafe fn(*mut *const c_char) -> c_int = if countcc {
        mb_cptr2char_adv
    } else {
        mb_ptr2char_adv
    };

    let mut s = unsafe { numbuf.string(&args[0]) };
    let mut len: VarNumber = 0;
    while unsafe { *s } != 0 {
        // Anything over U+FFFF is a surrogate pair: two units.
        len += 1 + VarNumber::from(unsafe { next_char(&raw mut s) } > 0xffff);
    }
    result.write_number(len);
}

/// "strcharpart()" function: a substring measured in characters.
pub fn f_strcharpart(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let p = unsafe { numbuf.string(&args[0]) };
    // SAFETY: the argument was converted to a NUL-terminated string.
    let bytes = unsafe { cstr::bytes_at(p) };
    let slen = bytes.len();

    let mut nbyte: c_int = 0;
    let mut skipcc = false;
    let mut error = false;
    let mut nchar = unsafe { tv_get_number_chk(&args[1], &raw mut error) };
    if !error {
        if args.len() > 2 && args.len() > 3 {
            match unsafe { strict_bool_arg(&args[3]) } {
                Some(flag) => skipcc = flag,
                None => return,
            }
        }
        if nchar > 0 {
            // Walk `nchar` characters in to find the byte offset.
            while nchar > 0 && (nbyte as size_t) < slen {
                nbyte += CharLen::new(!skipcc).of(&bytes[nbyte as usize..]) as c_int;
                nchar -= 1;
            }
        } else {
            // A negative start is already a byte offset, and stays
            // negative until the overlap is taken below.
            nbyte = nchar as c_int;
        }
    }

    let mut len: c_int = if args.len() > 2 {
        let mut charlen = tv_get_number(&args[2]) as c_int;
        let mut len = 0;
        while charlen > 0 && nbyte + len < slen as c_int {
            let off = nbyte + len;
            // Offsets before the string count one byte each, so a
            // negative start still consumes its share of `charlen`.
            len += if off < 0 {
                1
            } else {
                CharLen::new(!skipcc).of(&bytes[off as usize..]) as c_int
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

    result.write_empty(VAR_STRING);
    let from = unsafe { p.offset(nbyte as isize) } as *const c_void;
    let part = unsafe { xmemdupz(from, len as size_t) } as *mut c_char;
    result.write_string(part);
}

/// "strpart()" function: a substring measured in bytes, or -- with the
/// fourth argument -- in characters starting from a byte offset.
pub fn f_strpart(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut error = false;
    let p = unsafe { numbuf.string(&args[0]) };
    // SAFETY: the argument was converted to a NUL-terminated string.
    let bytes = unsafe { cstr::bytes_at(p) };
    let slen = bytes.len() as VarNumber;

    let mut n = unsafe { tv_get_number_chk(&args[1], &raw mut error) };
    let mut len = if error {
        0
    } else if args.len() > 2 {
        tv_get_number(&args[2])
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

    if args.len() > 2 && args.len() > 3 {
        // `len` was a character count after all: re-measure it.
        let mut off = n as int64_t;
        while off < slen as int64_t && len > 0 {
            off += cluster_len(&bytes[off as usize..]) as int64_t;
            len -= 1;
        }
        len = (off - n as int64_t) as VarNumber;
    }

    result.write_empty(VAR_STRING);
    let from = unsafe { p.offset(n as isize) } as *const c_void;
    let part = unsafe { xmemdupz(from, len as size_t) } as *mut c_char;
    result.write_string(part);
}

/// "utf16idx()" function: the UTF-16 index of a byte (or character) offset.
pub fn f_utf16idx(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);

    if tv_check_for_string_arg(args, 0).is_err()
        || tv_check_for_opt_number_arg(args, 1).is_err()
        || tv_check_for_opt_bool_arg(args, 2).is_err()
        || (args.len() > 2 && tv_check_for_opt_bool_arg(args, 3).is_err())
    {
        return;
    }

    let str = unsafe { numbuf.string_chk(&args[0]) };
    let mut idx = unsafe { tv_get_number_chk(&args[1], ptr::null_mut()) };
    if str.is_null() || idx < 0 {
        return;
    }

    let mut countcc = false;
    let mut charidx = false;
    if args.len() > 2 {
        countcc = tv_get_bool(&args[2]) != 0;
        if args.len() > 3 {
            charidx = tv_get_bool(&args[3]) != 0;
        }
    }

    let char_len = CharLen::new(countcc);
    // SAFETY: the argument was checked to be a string, so it is
    // NUL-terminated.
    let bytes = unsafe { cstr::bytes_at(str) };
    let mut len: c_int = 0;
    // The answer is the index of the *start* of the character the offset
    // lands in, so it trails `len` by one iteration.
    let mut utf16idx: c_int = 0;
    let mut at: VarNumber = 0;
    while if charidx { idx >= 0 } else { at <= idx } {
        let rest = &bytes[at as usize..];
        if rest.is_empty() {
            // An index of exactly the string's length in bytes (or
            // characters) answers its length in UTF-16 units.
            if if charidx { idx == 0 } else { at == idx } {
                result.write_number(VarNumber::from(len));
            }
            return;
        }
        utf16idx = len;
        let clen = char_len.of(rest);
        if code_point(rest, clen) > 0xffff {
            len += 1;
        }
        at += clen as VarNumber;
        if charidx {
            idx -= 1;
        }
        len += 1;
    }

    result.write_number(utf16idx as VarNumber);
}
