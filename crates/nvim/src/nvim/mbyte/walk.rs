//! Moving around a UTF-8 buffer.
//!
//! Everything that takes a pointer into a string and produces another position:
//! `utf_head_off` walks back to the start of the character (composing sequence
//! included) covering a byte, `utfc_next` steps forward, `utf_cp_bounds` gives
//! both ends at once, and `mb_charlen`/`mb_utflen` count characters over a span.
//! `mb_check_adjust_col` and `mb_adjust_cursor` are the two that move a *cursor*
//! off the middle of a character.  `always_break`/`always_break_two` are the
//! grapheme-cluster rules `utf_head_off` and `utfc_next` share -- boundclass
//! questions, not the `'linebreak'` ones next door.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;

pub const MB_MAXCHAR: C2Rust_Unnamed_18 = 6;

pub unsafe extern "C" fn mb_utflen(
    mut s: *const ::core::ffi::c_char,
    mut len: size_t,
    mut codepoints: *mut size_t,
    mut codeunits: *mut size_t,
) {
    unsafe {
        let mut count: size_t = 0 as size_t;
        let mut extra: size_t = 0 as size_t;
        let mut clen: size_t = 0;
        let mut i: size_t = 0 as size_t;
        while i < len {
            clen = utf_ptr2len_len(
                s.offset(i as isize),
                len.wrapping_sub(i) as ::core::ffi::c_int,
            ) as size_t;
            let mut c: ::core::ffi::c_int = if clen > 1 as size_t {
                utf_ptr2char(s.offset(i as isize))
            } else {
                *s.offset(i as isize) as uint8_t as ::core::ffi::c_int
            };
            count = count.wrapping_add(1);
            if c > 0xffff as ::core::ffi::c_int {
                extra = extra.wrapping_add(1);
            }
            i = i.wrapping_add(clen);
        }
        *codepoints = (*codepoints).wrapping_add(count);
        *codeunits = (*codeunits).wrapping_add(count.wrapping_add(extra));
    }
}

pub unsafe extern "C" fn mb_utf_index_to_bytes(
    mut s: *const ::core::ffi::c_char,
    mut len: size_t,
    mut index: size_t,
    mut use_utf16_units: bool,
) -> ssize_t {
    unsafe {
        let mut count: size_t = 0 as size_t;
        let mut clen: size_t = 0;
        if index == 0 as size_t {
            return 0 as ssize_t;
        }
        let mut i: size_t = 0 as size_t;
        while i < len {
            clen = utf_ptr2len_len(
                s.offset(i as isize),
                len.wrapping_sub(i) as ::core::ffi::c_int,
            ) as size_t;
            let mut c: ::core::ffi::c_int = if clen > 1 as size_t {
                utf_ptr2char(s.offset(i as isize))
            } else {
                *s.offset(i as isize) as uint8_t as ::core::ffi::c_int
            };
            count = count.wrapping_add(1);
            if use_utf16_units as ::core::ffi::c_int != 0 && c > 0xffff as ::core::ffi::c_int {
                count = count.wrapping_add(1);
            }
            if count >= index {
                return i.wrapping_add(clen) as ssize_t;
            }
            i = i.wrapping_add(clen);
        }
        return -1 as ssize_t;
    }
}

fn always_break(mut bc: ::core::ffi::c_int) -> bool {
    return bc == UTF8PROC_BOUNDCLASS_CONTROL as ::core::ffi::c_int;
}

fn always_break_two(mut bc1: ::core::ffi::c_int, mut bc2: ::core::ffi::c_int) -> bool {
    return bc1 != UTF8PROC_BOUNDCLASS_PREPEND as ::core::ffi::c_int
        && bc2 == UTF8PROC_BOUNDCLASS_OTHER as ::core::ffi::c_int
        || bc1 >= UTF8PROC_BOUNDCLASS_CR as ::core::ffi::c_int
            && bc1 <= UTF8PROC_BOUNDCLASS_CONTROL as ::core::ffi::c_int
        || bc2 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as ::core::ffi::c_int
            && (bc1 == UTF8PROC_BOUNDCLASS_OTHER as ::core::ffi::c_int
                || bc1 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as ::core::ffi::c_int);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_head_off(
    mut base_in: *const ::core::ffi::c_char,
    mut p_in: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if (*p_in as uint8_t as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let mut base: *const uint8_t = base_in as *mut uint8_t;
        let mut p: *const uint8_t = p_in as *mut uint8_t;
        let mut start: *const uint8_t = p;
        while start > base
            && *start as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
                == 0x80 as ::core::ffi::c_int
            && p.offset_from(start) < 6 as isize
        {
            start = start.offset(-1);
        }
        let last_len: uint8_t = (*utf8len_tab.ptr())[*start as usize];
        let mut cur_code: int32_t = utf_ptr2CharInfo_impl(start, last_len as uintptr_t);
        if cur_code < 0 as int32_t || p.offset_from(start) >= last_len as isize {
            return 0 as ::core::ffi::c_int;
        }
        let safe_end: *const uint8_t = start.offset(last_len as ::core::ffi::c_int as isize);
        let mut cur_bc: ::core::ffi::c_int =
            (*utf8proc_get_property(cur_code as utf8proc_int32_t)).boundclass as ::core::ffi::c_int;
        if always_break(cur_bc) as ::core::ffi::c_int != 0 || start == base {
            return p.offset_from(start) as ::core::ffi::c_int;
        }
        let mut cur_pos: *const uint8_t = start;
        let p_start: *const uint8_t = start;
        while *start.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
            start = start.offset(-1);
            if (*start as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
                break;
            }
            while start > base
                && *start as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int
                    == 0x80 as ::core::ffi::c_int
                && cur_pos.offset_from(start) < 6 as isize
            {
                start = start.offset(-1);
            }
            let mut prev_len: ::core::ffi::c_int =
                (*utf8len_tab.ptr())[*start as usize] as ::core::ffi::c_int;
            let mut prev_code: int32_t = utf_ptr2CharInfo_impl(start, prev_len as uintptr_t);
            if prev_code < 0 as int32_t || (prev_len as isize) < cur_pos.offset_from(start) {
                start = cur_pos;
                break;
            } else {
                let mut prev_bc: ::core::ffi::c_int =
                    (*utf8proc_get_property(prev_code as utf8proc_int32_t)).boundclass
                        as ::core::ffi::c_int;
                if always_break_two(prev_bc, cur_bc) as ::core::ffi::c_int != 0
                    && !crate::src::nvim::arabic::arabic_combine(
                        prev_code as ::core::ffi::c_int,
                        cur_code as ::core::ffi::c_int,
                    )
                {
                    start = cur_pos;
                    break;
                } else {
                    if start == base {
                        break;
                    }
                    cur_pos = start;
                    cur_bc = prev_bc;
                    cur_code = prev_code;
                }
            }
        }
        if start == p_start && last_len as isize > p.offset_from(start) {
            return p.offset_from(start) as ::core::ffi::c_int;
        }
        let mut q: *const uint8_t = start;
        while q < p {
            let mut len: ::core::ffi::c_int = utfc_ptr2len_len(
                q as *const ::core::ffi::c_char,
                safe_end.offset_from(q) as ::core::ffi::c_int,
            );
            if q.offset(len as isize) > p {
                return p.offset_from(q) as ::core::ffi::c_int;
            }
            q = q.offset(len as isize);
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn utfc_next_impl(mut cur: StrCharInfo) -> StrCharInfo {
    unsafe {
        let mut prev_code: int32_t = cur.chr.value;
        let mut next: *mut uint8_t = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
        let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
        assert!(
            *next as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int,
            "*next >= 0x80"
        );
        loop {
            let next_len: uint8_t = (*utf8len_tab.ptr())[*next as usize];
            let next_code: int32_t = utf_ptr2CharInfo_impl(next, next_len as uintptr_t);
            if !utf_iscomposing(
                prev_code as ::core::ffi::c_int,
                next_code as ::core::ffi::c_int,
                &raw mut state,
            ) {
                return StrCharInfo {
                    ptr: next as *mut ::core::ffi::c_char,
                    chr: CharInfo {
                        value: next_code,
                        len: if next_code < 0 as int32_t {
                            1 as ::core::ffi::c_int
                        } else {
                            next_len as ::core::ffi::c_int
                        },
                    },
                };
            }
            prev_code = next_code;
            next = next.offset(next_len as ::core::ffi::c_int as isize);
            if ((*next as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
                as ::core::ffi::c_long
                != 0
            {
                return StrCharInfo {
                    ptr: next as *mut ::core::ffi::c_char,
                    chr: CharInfo {
                        value: *next as int32_t,
                        len: 1 as ::core::ffi::c_int,
                    },
                };
            }
        }
    }
}

pub unsafe extern "C" fn mb_copy_char(
    fp: *mut *const ::core::ffi::c_char,
    tp: *mut *mut ::core::ffi::c_char,
) {
    unsafe {
        let l: size_t = utfc_ptr2len(*fp) as size_t;
        memmove(
            *tp as *mut ::core::ffi::c_void,
            *fp as *const ::core::ffi::c_void,
            l,
        );
        *tp = (*tp).offset(l as isize);
        *fp = (*fp).offset(l as isize);
    }
}

pub unsafe extern "C" fn mb_off_next(
    mut base: *const ::core::ffi::c_char,
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut head_off: ::core::ffi::c_int = utf_head_off(base, p);
        if head_off == 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        return utfc_ptr2len(p.offset(-(head_off as isize))) - head_off;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_cp_bounds_len(
    mut base: *const ::core::ffi::c_char,
    mut p_in: *const ::core::ffi::c_char,
    mut p_len: ::core::ffi::c_int,
) -> CharBoundsOff {
    unsafe {
        assert!(
            base <= p_in && p_len > 0 as ::core::ffi::c_int,
            "base <= p_in && p_len > 0"
        );
        let b: *const uint8_t = base as *mut uint8_t;
        let p: *const uint8_t = p_in as *mut uint8_t;
        if (*p as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint {
            return CharBoundsOff {
                begin_off: 0 as int8_t,
                end_off: 1 as int8_t,
            };
        }
        let max_first_off: ::core::ffi::c_int = -if (p.offset_from(b) as ::core::ffi::c_int)
            < MB_MAXCHAR as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        {
            p.offset_from(b) as ::core::ffi::c_int
        } else {
            MB_MAXCHAR as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        };
        let mut first_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while utf_is_trail_byte(*p.offset(first_off as isize)) {
            if first_off == max_first_off {
                return CharBoundsOff {
                    begin_off: 0 as int8_t,
                    end_off: 1 as int8_t,
                };
            }
            first_off -= 1;
        }
        let max_end_off: ::core::ffi::c_int =
            (*utf8len_tab.ptr())[*p.offset(first_off as isize) as usize] as ::core::ffi::c_int
                + first_off;
        if max_end_off <= 0 as ::core::ffi::c_int || max_end_off > p_len {
            return CharBoundsOff {
                begin_off: 0 as int8_t,
                end_off: 1 as int8_t,
            };
        }
        let mut end_off: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while end_off < max_end_off {
            if !utf_is_trail_byte(*p.offset(end_off as isize)) {
                return CharBoundsOff {
                    begin_off: 0 as int8_t,
                    end_off: 1 as int8_t,
                };
            }
            end_off += 1;
        }
        return CharBoundsOff {
            begin_off: -first_off as int8_t,
            end_off: max_end_off as int8_t,
        };
    }
}

pub unsafe extern "C" fn utf_cp_bounds(
    mut base: *const ::core::ffi::c_char,
    mut p_in: *const ::core::ffi::c_char,
) -> CharBoundsOff {
    unsafe {
        return utf_cp_bounds_len(base, p_in, INT_MAX);
    }
}

pub unsafe extern "C" fn mb_adjust_cursor() {
    unsafe {
        mark_mb_adjustpos(curbuf.get(), &raw mut (*curwin.get()).w_cursor);
    }
}

pub unsafe extern "C" fn mb_check_adjust_col(mut win_: *mut ::core::ffi::c_void) {
    unsafe {
        let mut win: *mut win_T = win_ as *mut win_T;
        let mut oldcol: colnr_T = (*win).w_cursor.col;
        if oldcol != 0 as ::core::ffi::c_int {
            let mut p: *mut ::core::ffi::c_char = ml_get_buf((*win).w_buffer, (*win).w_cursor.lnum);
            let mut len: colnr_T = strlen(p) as colnr_T;
            if len == 0 as ::core::ffi::c_int || oldcol < 0 as ::core::ffi::c_int {
                (*win).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            } else {
                if oldcol > len {
                    (*win).w_cursor.col =
                        (len as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
                }
                (*win).w_cursor.col -= utf_head_off(p, p.offset((*win).w_cursor.col as isize));
            }
            if (*win).w_cursor.coladd == 1 as ::core::ffi::c_int
                && *p.offset((*win).w_cursor.col as isize) as ::core::ffi::c_int != TAB
                && vim_isprintc(utf_ptr2char(p.offset((*win).w_cursor.col as isize)))
                    as ::core::ffi::c_int
                    != 0
                && ptr2cells(p.offset((*win).w_cursor.col as isize)) > 1 as ::core::ffi::c_int
            {
                (*win).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
        }
    }
}

pub unsafe extern "C" fn mb_prevptr(
    mut line: *mut ::core::ffi::c_char,
    mut p: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if p > line {
            p = p.offset(
                -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                    + 1 as ::core::ffi::c_int) as isize),
            );
        }
        return p;
    }
}

pub unsafe extern "C" fn mb_charlen(mut str: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const ::core::ffi::c_char = str;
        let mut count: ::core::ffi::c_int = 0;
        if p.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        count = 0 as ::core::ffi::c_int;
        while *p as ::core::ffi::c_int != NUL {
            p = p.offset(utfc_ptr2len(p) as isize);
            count += 1;
        }
        return count;
    }
}

pub unsafe extern "C" fn mb_charlen_len(
    mut str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const ::core::ffi::c_char = str;
        let mut count: ::core::ffi::c_int = 0;
        count = 0 as ::core::ffi::c_int;
        while *p as ::core::ffi::c_int != NUL && p < str.offset(len as isize) {
            p = p.offset(utfc_ptr2len(p) as isize);
            count += 1;
        }
        return count;
    }
}

/// `cur` paired with its codepoint: the start of a character and the
/// character itself. Composing characters are not consulted.
///
/// # Safety
/// `ptr` must point into a NUL-terminated string.
#[inline(always)]
pub unsafe fn utf_ptr2StrCharInfo(ptr: *mut ::core::ffi::c_char) -> StrCharInfo {
    unsafe {
        StrCharInfo {
            ptr,
            chr: utf_ptr2CharInfo(ptr),
        }
    }
}

/// The character after `cur`, treating a following composing character as
/// part of the *current* one. The ASCII case is inlined; everything else
/// defers to `utfc_next_impl`.
///
/// # Safety
/// `cur.ptr` must point into a NUL-terminated string, at a character start.
#[inline(always)]
pub unsafe fn utfc_next(cur: StrCharInfo) -> StrCharInfo {
    unsafe {
        let next = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
        if *next < 0x80 {
            return StrCharInfo {
                ptr: next as *mut ::core::ffi::c_char,
                chr: CharInfo {
                    value: *next as int32_t,
                    len: 1,
                },
            };
        }
        utfc_next_impl(cur)
    }
}
