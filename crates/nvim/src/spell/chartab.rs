//! Which characters are letters, and what case they are in.
//!
//! Spell checking cannot use the locale's idea of a letter: a `.spl` file
//! has to mean the same thing wherever it is loaded. So every language
//! carries its own answer, and [`spelltab`] holds the one currently in
//! force — a byte-indexed table of "is a word character", "is upper case",
//! and the folded and upper-cased forms.
//!
//! Only bytes below 256 live in the table. Wider characters are classified
//! by [`utf_class`] instead, through [`spell_mb_isword_class`], and folded
//! and upper-cased by the general [`utf_fold`]/[`mb_toupper`].
//!
//! [`init_spell_chartab`] fills the table from the current encoding at
//! startup; a `.spl` file's `FOL`/`LOW`/`UPP` section replaces it wholesale
//! and sets [`did_set_spelltab`].
//!
//! On top of that sit the case helpers the rest of the subsystem uses:
//! [`captype`] classifies a word's capitalisation into the `WF_*` flags the
//! word tree stores, and [`make_case_word`] applies those flags back to a
//! folded word to reconstruct how it should be spelled.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use crate::main::curwin;
use crate::mbyte::{
    mb_cptr2char_adv, mb_get_class, mb_islower, mb_isupper, mb_ptr2char_adv, mb_toupper,
    utf_char2bytes, utf_class, utf_fold, utf_ptr2char, utfc_ptr2len,
};
use crate::memory::xstrlcpy;
use crate::strings::vim_strchr;
use crate::types::{MB_MAXBYTES, spelltab_T, uint8_t, win_T};
use ::libc::strcpy;

use super::{FAIL, MAXWLEN, NUL, OK, WF_ALLCAP, WF_KEEPCAP, WF_ONECAP, did_set_spelltab, spelltab};

/// Fill `sp` with the ASCII answers: digits and letters are word
/// characters, `A`-`Z` are upper case, and everything else maps to itself.
///
/// Digits are included even though a word may not *start* with one; that
/// restriction is handled where words are looked up.
pub unsafe fn clear_spell_chartab(sp: *mut spelltab_T) {
    unsafe {
        let sp = &mut *sp;
        sp.st_isw = [false; 256];
        sp.st_isu = [false; 256];
        for i in 0..256 {
            sp.st_fold[i] = i as uint8_t;
            sp.st_upper[i] = i as uint8_t;
        }
        for i in b'0'..=b'9' {
            sp.st_isw[i as usize] = true;
        }
        for i in b'A'..=b'Z' {
            sp.st_isw[i as usize] = true;
            sp.st_isu[i as usize] = true;
            sp.st_fold[i as usize] = i + 0x20;
        }
        for i in b'a'..=b'z' {
            sp.st_isw[i as usize] = true;
            sp.st_upper[i as usize] = i - 0x20;
        }
    }
}

/// Reset [`spelltab`] to what the current encoding says, discarding
/// anything a `.spl` file installed. Called once at startup and again
/// whenever the spell files are reloaded.
pub unsafe fn init_spell_chartab() {
    unsafe {
        did_set_spelltab.set(false);
        clear_spell_chartab(spelltab.ptr());
        let tab = &mut *spelltab.ptr();
        for i in 128..256 {
            let f = utf_fold(i);
            let u = mb_toupper(i);

            tab.st_isu[i as usize] = mb_isupper(i);
            tab.st_isw[i as usize] = tab.st_isu[i as usize] || mb_islower(i);
            // The folded and upper-cased values of 0xb5 differ between
            // latin1 and utf-8, which used to raise E763 for no good
            // reason. Keep the latin1 answer for both.
            tab.st_fold[i as usize] = if f < 256 { f as uint8_t } else { i as uint8_t };
            tab.st_upper[i as usize] = if u < 256 { u as uint8_t } else { i as uint8_t };
        }
    }
}

/// Whether `p` points at a word character.
///
/// A "midword" character — one listed in the language's `MIDWORD` — counts
/// as a word character when a word character follows it, so that
/// `they're` is one word but `they there` is two. That only works past the
/// first character of a word, which is all the callers need.
pub unsafe fn spell_iswordp(p: *const c_char, wp: *const win_T) -> bool {
    unsafe {
        let syn = &*(*wp).w_s;
        let l = utfc_ptr2len(p);
        let mut s = p;
        if l == 1 {
            // Be quick for ASCII.
            if syn.b_spell_ismw[*p as uint8_t as usize] {
                s = p.offset(1);
            }
        } else {
            let c = utf_ptr2char(p);
            let midword = if c < 256 {
                syn.b_spell_ismw[c as usize]
            } else {
                !syn.b_spell_ismw_mb.is_null() && !vim_strchr(syn.b_spell_ismw_mb, c).is_null()
            };
            if midword {
                s = p.offset(l as isize);
            }
        }

        let c = utf_ptr2char(s);
        if c > 255 {
            return spell_mb_isword_class(mb_get_class(s), wp);
        }
        (*spelltab.ptr()).st_isw[c as usize]
    }
}

/// Whether `p` points at a word character, ignoring midword characters.
pub unsafe fn spell_iswordp_nmw(p: *const c_char, wp: *const win_T) -> bool {
    unsafe {
        let c = utf_ptr2char(p);
        if c > 255 {
            return spell_mb_isword_class(mb_get_class(p), wp);
        }
        (*spelltab.ptr()).st_isw[c as usize]
    }
}

/// Whether a character class from [`utf_class`] is a word character.
///
/// Only meaningful above 255. Unicode sub- and superscripts are excluded;
/// with `'spelloptions'` containing `cjk` the East Asian scripts are too.
unsafe fn spell_mb_isword_class(cl: c_int, wp: *const win_T) -> bool {
    unsafe {
        if (*(*wp).w_s).b_cjk != 0 {
            return cl == 2 || cl == 0x2800;
        }
        cl >= 2 && cl != 0x2070 && cl != 0x2080 && cl != 3
    }
}

/// Wide-character [`spell_iswordp`]: `w` is the tail of a character array
/// starting at the position of interest.
pub(super) unsafe fn spell_iswordp_w(w: &[c_int], wp: *const win_T) -> bool {
    unsafe {
        let syn = &*(*wp).w_s;
        let midword = if w[0] < 256 {
            syn.b_spell_ismw[w[0] as usize]
        } else {
            !syn.b_spell_ismw_mb.is_null() && !vim_strchr(syn.b_spell_ismw_mb, w[0]).is_null()
        };
        let c = if midword { w[1] } else { w[0] };

        if c > 255 {
            return spell_mb_isword_class(utf_class(c), wp);
        }
        (*spelltab.ptr()).st_isw[c as usize]
    }
}

/// Case-fold `str[..len]` into `buf`, NUL terminated, using the character
/// definitions from the `.spl` file. Folding may change the length.
///
/// Returns `FAIL` when the result does not fit, having still terminated
/// what was written.
pub unsafe fn spell_casefold(
    wp: *const win_T,
    str: *const c_char,
    len: c_int,
    buf: *mut c_char,
    buflen: c_int,
) -> c_int {
    unsafe {
        if len >= buflen {
            *buf = NUL as c_char;
            return FAIL;
        }

        let mut outi = 0;
        let end = str.offset(len as isize);
        let mut p = str;
        while p < end {
            if outi + MB_MAXBYTES as c_int > buflen {
                *buf.offset(outi as isize) = NUL as c_char;
                return FAIL;
            }
            let mut c = mb_cptr2char_adv(&raw mut p);
            if c == 0x3a3 || c == 0x3c2 {
                // Greek sigma folds to the final form at the end of a word
                // and to the medial form anywhere else.
                c = if p == end || !spell_iswordp(p, wp) {
                    0x3c2
                } else {
                    0x3c3
                };
            } else if c >= 128 {
                c = utf_fold(c);
            } else {
                c = (*spelltab.ptr()).st_fold[c as usize] as c_int;
            }
            outi += utf_char2bytes(c, buf.offset(outi as isize));
        }
        *buf.offset(outi as isize) = NUL as c_char;
        OK
    }
}

/// Classify the capitalisation of `word` (up to `end`, or its NUL when
/// `end` is null) as one of `WF_ONECAP`, `WF_ALLCAP`, `WF_KEEPCAP`, or
/// zero for all lower case.
///
/// `WF_KEEPCAP` means the pattern is neither of the simple ones — "MacBeth"
/// — so the word tree has to store it spelled out.
pub unsafe fn captype(word: *const c_char, end: *const c_char) -> c_int {
    unsafe {
        // Skip over any leading non-word characters.
        let at_end = |p: *const c_char| {
            if end.is_null() { *p == 0 } else { p >= end }
        };
        let mut p = word;
        while !spell_iswordp_nmw(p, curwin.get()) {
            if at_end(p) {
                return 0;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }

        let c = mb_ptr2char_adv(&raw mut p);
        let mut allcap = is_upper(c);
        let firstcap = allcap;
        let mut past_second = false;

        while !at_end(p) {
            if spell_iswordp_nmw(p, curwin.get()) {
                if !is_upper(utf_ptr2char(p)) {
                    // A lower-case letter after two upper-case ones, or
                    // after a mix, cannot be described by a flag.
                    if past_second && allcap {
                        return WF_KEEPCAP as c_int;
                    }
                    allcap = false;
                } else if !allcap {
                    return WF_KEEPCAP as c_int;
                }
                past_second = true;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }

        if allcap {
            WF_ALLCAP as c_int
        } else if firstcap {
            WF_ONECAP as c_int
        } else {
            0
        }
    }
}

/// Whether `c` is upper case, by the spell table below 128 and by the
/// general rules above it.
unsafe fn is_upper(c: c_int) -> bool {
    unsafe {
        if c >= 128 {
            mb_isupper(c)
        } else {
            (*spelltab.ptr()).st_isu[c as usize]
        }
    }
}

/// The upper-case form of `c`, by the spell table below 128 and the
/// general rules above it.
pub(super) unsafe fn spell_toupper(c: c_int) -> c_int {
    unsafe {
        if c >= 128 {
            mb_toupper(c)
        } else {
            (*spelltab.ptr()).st_upper[c as usize] as c_int
        }
    }
}

/// Copy `word` into `wcopy`, upper-casing (or folding) the first character.
/// `wcopy` must hold [`MAXWLEN`] bytes.
pub unsafe fn onecap_copy(word: *const c_char, wcopy: *mut c_char, upper: bool) {
    unsafe {
        let mut p = word;
        let mut c = mb_cptr2char_adv(&raw mut p);
        c = if upper {
            if c >= 128 {
                mb_toupper(c)
            } else {
                (*spelltab.ptr()).st_upper[c as usize] as c_int
            }
        } else if c >= 128 {
            utf_fold(c)
        } else {
            (*spelltab.ptr()).st_fold[c as usize] as c_int
        };
        let l = utf_char2bytes(c, wcopy);
        xstrlcpy(wcopy.offset(l as isize), p, MAXWLEN - l as usize);
    }
}

/// Copy `word` into `wcopy` with every character upper-cased. `wcopy` must
/// hold [`MAXWLEN`] bytes; the copy stops short rather than overflow.
pub unsafe fn allcap_copy(word: *const c_char, wcopy: *mut c_char) {
    unsafe {
        let mut d = wcopy;
        let mut s = word;
        while *s != 0 {
            let mut c = mb_cptr2char_adv(&raw mut s);
            if c == 0xdf {
                // German sharp s upper-cases to two characters; the second
                // goes through the usual path below.
                c = 'S' as c_int;
                if d.offset_from(wcopy) >= MAXWLEN as isize - 1 {
                    break;
                }
                *d = c as c_char;
                d = d.offset(1);
            } else if c >= 128 {
                c = mb_toupper(c);
            } else {
                c = (*spelltab.ptr()).st_upper[c as usize] as c_int;
            }

            if d.offset_from(wcopy) >= MAXWLEN as isize - MB_MAXBYTES as isize {
                break;
            }
            d = d.offset(utf_char2bytes(c, d) as isize);
        }
        *d = NUL as c_char;
    }
}

/// The length in bytes of the first `flen` bytes' worth of *characters* of
/// `fword`, measured in the unfolded `word`.
///
/// Folding can change how many bytes a character takes, so a length into
/// the folded word has to be converted before it can index the original.
pub unsafe fn nofold_len(fword: *mut c_char, flen: c_int, word: *mut c_char) -> c_int {
    unsafe {
        let mut i = 0;
        let mut p = fword;
        let end = fword.offset(flen as isize);
        while p < end {
            i += 1;
            p = p.offset(utfc_ptr2len(p) as isize);
        }

        let mut p = word;
        while i > 0 {
            i -= 1;
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        p.offset_from(word) as c_int
    }
}

/// Write the folded word `fword` into `cword` with the capitalisation that
/// `flags` (as produced by [`captype`]) describes.
pub unsafe fn make_case_word(fword: *mut c_char, cword: *mut c_char, flags: c_int) {
    unsafe {
        if flags & WF_ALLCAP as c_int != 0 {
            allcap_copy(fword, cword);
        } else if flags & WF_ONECAP as c_int != 0 {
            onecap_copy(fword, cword, true);
        } else {
            strcpy(cword, fword);
        }
    }
}

/// Whether byte `n` appears in the NUL-terminated `str`. Like `strchr()`
/// but independent of the locale.
pub unsafe fn byte_in_str(str: *mut uint8_t, n: c_int) -> bool {
    unsafe {
        let mut p = str;
        while *p != 0 {
            if *p as c_int == n {
                return true;
            }
            p = p.offset(1);
        }
        false
    }
}
