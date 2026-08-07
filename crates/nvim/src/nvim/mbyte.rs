#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::charset::{char2cells, ptr2cells, vim_isprintc, vim_iswordc_tab};
use crate::src::nvim::cursor::get_cursor_pos_ptr;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::src::nvim::eval::typval::{
    tv_check_for_string_arg, tv_get_string, tv_get_string_buf, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_list, tv_list_append_number,
};
use crate::src::nvim::eval::typval::{tv_list_first, tv_list_len};
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::schar_from_buf;
use crate::src::nvim::keycodes::{K_SPECIAL, KE_FILLER};
use crate::src::nvim::main::{
    IObuff, cmp_flags, curbuf, curwin, e_listreq, fenc_default, p_ambw, p_emoji, p_enc,
};
use crate::src::nvim::mark::mark_mb_adjustpos;
use crate::src::nvim::memline::ml_get_buf;
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup};
use crate::src::nvim::message::{emsg, msg, semsg};
use crate::src::nvim::r#move::changed_window_setting_all;
use crate::src::nvim::options::{kOptCmpFlagInternal, kOptCmpFlagKeepascii};
use crate::src::nvim::optionstr::check_chars_options;
use crate::src::nvim::os::env::os_getenv_noalloc;
use crate::src::nvim::os::libc::{
    __ctype_b_loc, __errno_location, gettext, iconv, iconv_close, iconv_open, memcmp, memcpy,
    memmove, qsort, setlocale, snprintf, strchr, strcmp, strcpy, strlen, strncasecmp, strncmp,
    tolower, toupper,
};
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    CONV_9_TO_UTF8, CONV_ICONV, CONV_NONE, CONV_TO_LATIN1, CONV_TO_LATIN9, CONV_TO_UTF8,
    CharBoundsOff, CharInfo, EvalFuncData, GraphemeState, StrCharInfo, VAR_LIST, VAR_NUMBER,
    VAR_STRING, colnr_T, expand_T, iconv_t, int8_t, int32_t, int64_t, list_T, listitem_T, pos_T,
    ptrdiff_t, schar_T, size_t, ssize_t, typval_T, uint8_t, uint32_t, uint64_t, uintptr_t,
    utf8proc_int32_t, varnumber_T, vimconv_T, win_T,
};
use crate::src::nvim::utf8proc::{
    UTF8PROC_BOUNDCLASS_CONTROL, UTF8PROC_BOUNDCLASS_CR, UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC,
    UTF8PROC_BOUNDCLASS_OTHER, UTF8PROC_BOUNDCLASS_PREPEND, UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR,
    UTF8PROC_CASEFOLD, UTF8PROC_CATEGORY_ME, UTF8PROC_CATEGORY_MN, utf8proc_decompose_char,
    utf8proc_get_property, utf8proc_grapheme_break, utf8proc_grapheme_break_stateful,
    utf8proc_property_t, utf8proc_tolower, utf8proc_toupper,
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

unsafe extern "C" {
    #[cfg(not(miri))]
    fn towlower(__wc: wint_t) -> wint_t;
    #[cfg(not(miri))]
    fn towupper(__wc: wint_t) -> wint_t;
    fn nl_langinfo(__item: nl_item) -> *mut ::core::ffi::c_char;
}

// Miri cannot call libc. The tests never call setlocale, so glibc would run
// these in the C locale, where they fold ASCII only — which is exactly what
// these definitions do.
#[cfg(miri)]
fn towlower(__wc: wint_t) -> wint_t {
    u8::try_from(__wc).map_or(__wc, |b| b.to_ascii_lowercase() as wint_t)
}
#[cfg(miri)]
fn towupper(__wc: wint_t) -> wint_t {
    u8::try_from(__wc).map_or(__wc, |b| b.to_ascii_uppercase() as wint_t)
}
pub type wint_t = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LC_CTYPE: ::core::ffi::c_int = __LC_CTYPE;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub unsafe extern "C" fn show_utf8() {
    unsafe {
        let mut line: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
        let mut len: ::core::ffi::c_int = utfc_ptr2len(line);
        if len == 0 as ::core::ffi::c_int {
            msg(
                b"NUL\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
            return;
        }
        let mut rlen: size_t = 0 as size_t;
        let mut clen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < len {
            if clen == 0 as ::core::ffi::c_int {
                if i > 0 as ::core::ffi::c_int {
                    strcpy(
                        (IObuff.ptr() as *mut ::core::ffi::c_char).offset(rlen as isize),
                        b"+ \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    );
                    rlen = rlen.wrapping_add(2 as size_t);
                }
                clen = utf_ptr2len(line.offset(i as isize));
            }
            assert!(
                (1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t > rlen,
                "IOSIZE > rlen"
            );
            snprintf(
                (IObuff.ptr() as *mut ::core::ffi::c_char).offset(rlen as isize),
                (IOSIZE as size_t).wrapping_sub(rlen),
                b"%02x \0".as_ptr() as *const ::core::ffi::c_char,
                if *line.offset(i as isize) as ::core::ffi::c_int == NL {
                    NUL
                } else {
                    *line.offset(i as isize) as uint8_t as ::core::ffi::c_int
                },
            );
            clen -= 1;
            rlen = rlen.wrapping_add(strlen(
                (IObuff.ptr() as *mut ::core::ffi::c_char).offset(rlen as isize),
            ));
            if rlen > (IOSIZE - 20 as ::core::ffi::c_int) as size_t {
                break;
            }
            i += 1;
        }
        msg(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
}
pub unsafe extern "C" fn utf_find_illegal() {
    unsafe {
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        let mut vimconv: vimconv_T = vimconv_T {
            vc_type: 0,
            vc_factor: 0,
            vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            vc_fail: false,
        };
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
        if enc_canon_props((*curbuf.get()).b_p_fenc) & ENC_8BIT as ::core::ffi::c_int != 0 {
            convert_setup(&raw mut vimconv, p_enc.get(), (*curbuf.get()).b_p_fenc);
        }
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        '_theend: {
            loop {
                let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                if vimconv.vc_type != CONV_NONE as ::core::ffi::c_int {
                    xfree(tofree as *mut ::core::ffi::c_void);
                    tofree = string_convert(&raw mut vimconv, p, ::core::ptr::null_mut::<size_t>());
                    if tofree.is_null() {
                        break;
                    }
                    p = tofree;
                }
                while *p as ::core::ffi::c_int != NUL {
                    let mut len: ::core::ffi::c_int = utf_ptr2len(p);
                    if *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
                        && (len == 1 as ::core::ffi::c_int || utf_char2len(utf_ptr2char(p)) != len)
                    {
                        if vimconv.vc_type == CONV_NONE as ::core::ffi::c_int {
                            (*curwin.get()).w_cursor.col +=
                                p.offset_from(get_cursor_pos_ptr()) as colnr_T;
                        } else {
                            let mut l: ::core::ffi::c_int = 0;
                            len = p.offset_from(tofree) as ::core::ffi::c_int;
                            p = get_cursor_pos_ptr();
                            while *p as ::core::ffi::c_int != NUL && {
                                let c2rust_fresh1 = len;
                                len = len - 1;
                                c2rust_fresh1 > 0 as ::core::ffi::c_int
                            } {
                                l = utf_ptr2len(p);
                                (*curwin.get()).w_cursor.col += l;
                                p = p.offset(l as isize);
                            }
                        }
                        break '_theend;
                    } else {
                        p = p.offset(len as isize);
                    }
                }
                if (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_ml.ml_line_count {
                    break;
                }
                (*curwin.get()).w_cursor.lnum += 1;
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            }
            (*curwin.get()).w_cursor = pos;
            beep_flush();
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        convert_setup(
            &raw mut vimconv,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
    }
}
pub unsafe extern "C" fn utf_valid_string(
    mut s: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> bool {
    unsafe {
        let mut p: *const uint8_t = s as *mut uint8_t;
        while if end.is_null() {
            (*p as ::core::ffi::c_int != NUL) as ::core::ffi::c_int
        } else {
            (p < end as *mut uint8_t as *const uint8_t) as ::core::ffi::c_int
        } != 0
        {
            let mut l: ::core::ffi::c_int =
                (*utf8len_tab_zero.ptr())[*p as usize] as ::core::ffi::c_int;
            if l == 0 as ::core::ffi::c_int {
                return false_0 != 0;
            }
            if !end.is_null() && p.offset(l as isize) > end as *mut uint8_t as *const uint8_t {
                return false_0 != 0;
            }
            p = p.offset(1);
            loop {
                l -= 1;
                if l <= 0 as ::core::ffi::c_int {
                    break;
                }
                let c2rust_fresh12 = p;
                p = p.offset(1);
                if *c2rust_fresh12 as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
                    != 0x80 as ::core::ffi::c_int
                {
                    return false_0 != 0;
                }
            }
        }
        return true_0 != 0;
    }
}
pub unsafe extern "C" fn mb_unescape(
    pp: *mut *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        static buf: GlobalCell<[::core::ffi::c_char; 6]> = GlobalCell::new([0; 6]);
        let mut buf_idx: size_t = 0 as size_t;
        let mut str: *mut uint8_t = *pp as *mut uint8_t;
        let mut str_idx: size_t = 0 as size_t;
        while *str.offset(str_idx as isize) as ::core::ffi::c_int != NUL && buf_idx < 4 as size_t {
            if *str.offset(str_idx as isize) as ::core::ffi::c_int == K_SPECIAL
                && *str.offset(str_idx.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                    == KS_SPECIAL
                && *str.offset(str_idx.wrapping_add(2 as size_t) as isize) as ::core::ffi::c_int
                    == KE_FILLER
            {
                let c2rust_fresh13 = buf_idx;
                buf_idx = buf_idx.wrapping_add(1);
                (*buf.ptr())[c2rust_fresh13 as usize] = K_SPECIAL as ::core::ffi::c_char;
                str_idx = str_idx.wrapping_add(2 as size_t);
            } else {
                if *str.offset(str_idx as isize) as ::core::ffi::c_int == K_SPECIAL {
                    break;
                }
                let c2rust_fresh14 = buf_idx;
                buf_idx = buf_idx.wrapping_add(1);
                (*buf.ptr())[c2rust_fresh14 as usize] =
                    *str.offset(str_idx as isize) as ::core::ffi::c_char;
            }
            (*buf.ptr())[buf_idx as usize] = NUL as ::core::ffi::c_char;
            if utf_ptr2len(buf.ptr() as *mut ::core::ffi::c_char) > 1 as ::core::ffi::c_int {
                *pp = (str as *const ::core::ffi::c_char)
                    .offset(str_idx as isize)
                    .offset(1 as ::core::ffi::c_int as isize);
                return buf.ptr() as *mut ::core::ffi::c_char;
            }
            if ((*buf.ptr())[0 as ::core::ffi::c_int as usize] as uint8_t as ::core::ffi::c_int)
                < 128 as ::core::ffi::c_int
            {
                break;
            }
            str_idx = str_idx.wrapping_add(1);
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}
pub const __LC_CTYPE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
