//! Multibyte text: UTF-8, the encodings around it, and everything that has
//! to know how wide a character is.
//!
//! nvim's internal encoding is always UTF-8; this family is what makes that
//! true at the edges and useful in the middle.
//!
//! - [`utf8`] is the codec, and [`utf8::tables`] the byte-length tables the
//!   whole tree indexes.
//! - [`walk`] moves around a buffer: character starts, grapheme clusters,
//!   cursor adjustment.
//! - [`cells`] answers display width, which is what the screen is laid out
//!   from, and carries `setcellwidths()`.
//! - [`class`] and [`case`] are the questions word motions and
//!   case-insensitive matching ask.
//! - [`linebreak`] is where `'linebreak'` may break a line.
//! - [`encoding`] canonicalises encoding names and [`convert`] converts
//!   between them -- the only place iconv is touched.
//!
//! What is left here is the shared C-level vocabulary, the foreign
//! declarations the children reach through `use super::*`, and four
//! editor-facing utilities that belong to no seam: `g8` ([`show_utf8`]),
//! `8g8` ([`utf_find_illegal`]), a validity check and the `K_SPECIAL`
//! unescaper.
//!
//! **Eleven symbols here are exported by name and may not change
//! signature.** Nine are FFI'd from the LuaJIT unit specs; `utf8len_tab` and
//! `utf_ptr2char_info_impl` are compiled *against* by `unit-fixtures.so`, so
//! changing either is a C compile failure inside `just unittest`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};

use crate::ascii::ascii_iswhite;
use crate::charset::{char2cells, ptr2cells, vim_isprintc, vim_iswordc_tab};
use crate::cursor::get_cursor_pos_ptr;
use crate::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::eval::typval::{
    tv_check_for_string_arg, tv_get_string, tv_get_string_buf, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_list, tv_list_append_number, tv_list_first, tv_list_len,
};
use crate::getchar::beep_flush;
use crate::global_cell::GlobalCell;
use crate::grid::schar_from_buf;
use crate::keycodes::{K_SPECIAL, KE_FILLER};
use crate::main::{cmp_flags, curbuf, curwin, e_listreq, fenc_default, p_ambw, p_emoji, p_enc};
use crate::mark::mark_mb_adjustpos;
use crate::memline::ml_get_buf;
use crate::memory::{xfree, xmalloc, xmemdupz, xstrdup};
use crate::message::{emsg, msg};
use crate::r#move::changed_window_setting_all;
use crate::options::{kOptCmpFlagInternal, kOptCmpFlagKeepascii};
use crate::optionstr::check_chars_options;
use crate::os::cshim::{__ctype_b_loc, gettext, memmove, snprintf, strchr, strncasecmp};
use crate::os::env::{env_buf, os_getenv_into};
use crate::pos::MAXCOL;
use crate::strings::vim_strchr;
use crate::types::{
    CONV_9_TO_UTF8, CONV_ICONV, CONV_NONE, CONV_NONE_INIT, CONV_TO_LATIN1, CONV_TO_LATIN9,
    CONV_TO_UTF8, CharBoundsOff, CharInfo, EvalFuncData, GraphemeState, IOSIZE, MB_MAXCHAR, NUL,
    StrCharInfo, VAR_LIST, VAR_NUMBER, VAR_STRING, colnr_T, expand_T, iconv_t, int8_t, int32_t,
    list_T, ptrdiff_t, schar_T, size_t, ssize_t, typval_T, uint8_t, uint64_t, uintptr_t,
    utf8proc_int32_t, varnumber_T, vimconv_T, win_T,
};
use crate::utf8proc::{
    UTF8PROC_BOUNDCLASS_CONTROL, UTF8PROC_BOUNDCLASS_CR, UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC,
    UTF8PROC_BOUNDCLASS_OTHER, UTF8PROC_BOUNDCLASS_PREPEND, UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR,
    UTF8PROC_CASEFOLD, UTF8PROC_CATEGORY_ME, UTF8PROC_CATEGORY_MN, utf8proc_decompose_char,
    utf8proc_get_property, utf8proc_grapheme_break, utf8proc_grapheme_break_stateful,
    utf8proc_property_t, utf8proc_tolower, utf8proc_toupper,
};
use ::libc::{
    __errno_location, iconv, iconv_close, iconv_open, memcmp, setlocale, strcmp, strcpy, strlen,
    tolower, toupper,
};

// The carve of the transpiled module; see each child's docs.
mod case;
mod cells;
mod class;
mod convert;
mod encoding;
mod linebreak;
mod utf8;
mod walk;

pub use self::case::*;
pub use self::cells::*;
pub use self::class::*;
pub use self::convert::*;
pub use self::encoding::*;
pub use self::linebreak::*;
pub use self::utf8::*;
pub use self::walk::*;

/// The C-level vocabulary every transpiled module carries a copy of;
/// consolidating them tree-wide is a family of its own, not this slice's.
pub const TAB: c_int = '\t' as c_int;
pub const NL: c_int = '\n' as c_int;
/// `setlocale`'s category for character classification.
pub const LC_CTYPE: c_int = 0;
/// `SIZE_MAX`, which `iconv` answers with on failure.
pub const SIZE_MAX: ::core::ffi::c_ulong = usize::MAX as ::core::ffi::c_ulong;
/// `INT_MAX`, the "no limit" length `utf_cp_bounds` passes.
pub const INT_MAX: c_int = c_int::MAX;
/// How many bytes a `schar_T` can hold, NUL included.
pub const MAX_SCHAR_SIZE: c_int = 32;
/// The second byte of an escaped literal `K_SPECIAL`; see [`mb_unescape`].
pub const KS_SPECIAL: c_int = 254;
/// `g8`: show the bytes of the character under the cursor, in hex.
///
/// A composing sequence is shown as its characters separated by `+`.
///
/// # Safety
///
/// The editor's globals must be live.
pub unsafe fn show_utf8() {
    // The hex dump. Upstream shares `IObuff`, which `msg` writes again.
    let mut hex = [0 as c_char; IOSIZE as usize];
    unsafe {
        // The whole grapheme cluster, composing characters included.
        let line = get_cursor_pos_ptr();
        let len = utfc_ptr2len(line);
        if len == 0 {
            msg(c"NUL".as_ptr(), 0);
            return;
        }

        let out = hex.as_mut_ptr();
        let mut rlen: size_t = 0;
        let mut clen = 0;
        for i in 0..len {
            if clen == 0 {
                // The start of another character in the cluster.
                if i > 0 {
                    strcpy(out.add(rlen), c"+ ".as_ptr().cast_mut());
                    rlen += 2;
                }
                clen = utf_ptr2len(line.offset(i as isize));
            }
            debug_assert!(IOSIZE as size_t > rlen, "IOSIZE > rlen");
            let byte = *line.offset(i as isize);
            snprintf(
                out.add(rlen),
                IOSIZE as size_t - rlen,
                c"%02x ".as_ptr(),
                // A NUL is stored in the buffer as a newline.
                if byte as c_int == NL {
                    NUL
                } else {
                    byte as u8 as c_int
                },
            );
            clen -= 1;
            rlen += strlen(out.add(rlen));
            if rlen > (IOSIZE - 20) as size_t {
                break;
            }
        }
        msg(out, 0);
    }
}

/// `8g8`: move the cursor to the next byte that is not valid UTF-8.
///
/// "Not valid" is either too few continuation bytes -- which `utf_ptr2len`
/// reports as a length of 1 -- or too many, an overlong sequence, which is
/// caught by re-encoding the decoded character and finding a shorter length.
///
/// When the file is being *stored* in an 8-bit encoding, the search runs over
/// the converted line rather than the buffer's, and the column has to be
/// walked back character by character, because the two do not share offsets.
///
/// # Safety
///
/// The editor's globals must be live.
pub unsafe fn utf_find_illegal() {
    unsafe {
        let start = (*curwin.get()).w_cursor;
        let mut vimconv = CONV_NONE_INIT;
        let mut tofree: *mut c_char = core::ptr::null_mut();

        if enc_canon_props((*curbuf.get()).b_p_fenc) & ENC_8BIT != 0 {
            // 'encoding' is utf-8 but the file is 8-bit, so what is illegal is
            // decided after converting back to the file's encoding.
            convert_setup(&raw mut vimconv, p_enc.get(), (*curbuf.get()).b_p_fenc);
        }

        (*curwin.get()).w_cursor.coladd = 0;
        let found = 'search: loop {
            let mut p = get_cursor_pos_ptr();
            if vimconv.vc_type != CONV_NONE {
                xfree(tofree as *mut c_void);
                tofree = string_convert(&raw mut vimconv, p, core::ptr::null_mut());
                if tofree.is_null() {
                    break false;
                }
                p = tofree;
            }

            while *p != NUL as c_char {
                let len = utf_ptr2len(p);
                if *p as u8 >= 0x80 && (len == 1 || utf_char2len(utf_ptr2char(p)) != len) {
                    if vimconv.vc_type == CONV_NONE {
                        (*curwin.get()).w_cursor.col +=
                            p.offset_from(get_cursor_pos_ptr()) as colnr_T;
                    } else {
                        // `p` is an offset into the *converted* line; step the
                        // real line forward by that many bytes' worth of
                        // characters to find the matching column.
                        let mut left = p.offset_from(tofree) as c_int;
                        let mut q = get_cursor_pos_ptr();
                        while *q != NUL as c_char && left > 0 {
                            left -= 1;
                            let l = utf_ptr2len(q);
                            (*curwin.get()).w_cursor.col += l;
                            q = q.offset(l as isize);
                        }
                    }
                    break 'search true;
                }
                p = p.offset(len as isize);
            }

            if (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_ml.ml_line_count {
                break false;
            }
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0;
        };

        if !found {
            (*curwin.get()).w_cursor = start;
            beep_flush();
        }
        xfree(tofree as *mut c_void);
        convert_setup(
            &raw mut vimconv,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
}

/// Is every byte of `s` part of a well-formed UTF-8 sequence?
///
/// `end` bounds the check; a null `end` means "up to the NUL". A sequence
/// that runs past `end` is a failure, not a truncation: this is the strict
/// answer, for callers deciding whether to accept a whole string at all.
///
/// # Safety
///
/// `s` must be NUL-terminated when `end` is null, and reach `end` otherwise.
pub unsafe fn utf_valid_string(s: *const c_char, end: *const c_char) -> bool {
    unsafe {
        let mut p = s as *const u8;
        let end = end as *const u8;
        while if end.is_null() { *p != 0 } else { p < end } {
            let mut l = utf8len_tab_zero[*p as usize] as c_int;
            if l == 0 {
                return false; // not a lead byte
            }
            if !end.is_null() && p.offset(l as isize) > end {
                return false; // the sequence runs past the end
            }
            p = p.offset(1);
            l -= 1;
            while l > 0 {
                if !utf_is_trail_byte(*p) {
                    return false;
                }
                p = p.offset(1);
                l -= 1;
            }
        }
        true
    }
}

/// Decode one character out of a key-encoded string, advancing `*pp` past it.
///
/// Typeahead and mappings escape a literal `K_SPECIAL` byte as the three-byte
/// `K_SPECIAL KS_SPECIAL KE_FILLER`, so a multibyte character containing that
/// byte cannot be read directly. This unescapes as it goes and answers as
/// soon as it has a *multibyte* character.
///
/// Answers null when the next character is ASCII, when a real special key is
/// reached (one cannot be part of a multibyte character), or when four bytes
/// have gone by without a character — four being the longest sequence anyone
/// types.
///
/// The result points into a shared static buffer and is only valid until the
/// next call.
///
/// # Safety
///
/// `*pp` must be a NUL-terminated string.
pub unsafe fn mb_unescape(pp: *mut *const c_char) -> *const c_char {
    unsafe {
        static buf: GlobalCell<[c_char; MB_MAXCHAR]> = GlobalCell::new([0; MB_MAXCHAR]);
        let out = buf.ptr() as *mut c_char;
        let str = *pp as *const u8;
        let mut buf_idx = 0;
        let mut str_idx = 0;
        while *str.add(str_idx) != 0 && buf_idx < 4 {
            if *str.add(str_idx) as c_int == K_SPECIAL {
                if *str.add(str_idx + 1) as c_int != KS_SPECIAL
                    || *str.add(str_idx + 2) as c_int != KE_FILLER
                {
                    break; // a real special key, which is never multibyte
                }
                *out.add(buf_idx) = K_SPECIAL as c_char;
                str_idx += 2;
            } else {
                *out.add(buf_idx) = *str.add(str_idx) as c_char;
            }
            buf_idx += 1;
            *out.add(buf_idx) = NUL as c_char;

            // An illegal sequence answers 1 here, so this only fires on a
            // character that is really multibyte.
            if utf_ptr2len(out) > 1 {
                *pp = (str as *const c_char).add(str_idx + 1);
                return out;
            }
            if (*out as u8) < 128 {
                break; // ASCII: nothing more can make it multibyte
            }
            str_idx += 1;
        }
        core::ptr::null()
    }
}
