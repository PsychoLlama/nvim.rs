//! Moving around a UTF-8 buffer.
//!
//! Everything that takes a pointer into a string and answers another
//! position: [`utf_head_off`] walks back to the start of the grapheme cluster
//! covering a byte, [`utfc_next`] steps forward one, [`utf_cp_bounds_len`]
//! gives both ends of the codepoint at once, and `mb_charlen`/`mb_utflen`
//! count characters over a span.
//!
//! All of it has to survive bytes that are not UTF-8 at all: a buffer can
//! hold anything, and the cursor can be anywhere in it. The rule throughout
//! is that a byte which is not part of a valid sequence is its own character,
//! one byte long — so a walk always terminates and never steps past the
//! position it was asked about.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::NUL;
use core::ffi::{c_char, c_int, c_void};

/// The most bytes a UTF-8 sequence can occupy in this port's decoders, which
/// is how far back a byte's lead can possibly be.
const MAX_SEQUENCE: isize = 6;

/// Count the characters in `s[..len]`, adding the totals to `codepoints` and
/// `codeunits`.
///
/// The two differ by the UTF-16 surrogate pairs: a character above the BMP is
/// one codepoint but two UTF-16 code units, which is what the LSP-facing
/// callers need.
///
/// # Safety
///
/// `s` must point at `len` readable bytes; both counters must be writable.
pub unsafe fn mb_utflen(
    s: *const c_char,
    len: size_t,
    codepoints: *mut size_t,
    codeunits: *mut size_t,
) {
    unsafe {
        let mut count: size_t = 0;
        let mut surrogate_pairs: size_t = 0;
        let mut i: size_t = 0;
        while i < len {
            let (c, clen) = char_at(s, i, len);
            count += 1;
            if c > 0xffff {
                surrogate_pairs += 1;
            }
            i += clen;
        }
        *codepoints += count;
        *codeunits += count + surrogate_pairs;
    }
}

/// The byte offset just past the character at index `index`, counted in
/// codepoints or in UTF-16 code units, or −1 if the string is shorter.
///
/// # Safety
///
/// `s` must point at `len` readable bytes.
pub unsafe fn mb_utf_index_to_bytes(
    s: *const c_char,
    len: size_t,
    index: size_t,
    use_utf16_units: bool,
) -> ssize_t {
    unsafe {
        if index == 0 {
            return 0;
        }
        let mut count: size_t = 0;
        let mut i: size_t = 0;
        while i < len {
            let (c, clen) = char_at(s, i, len);
            count += 1;
            if use_utf16_units && c > 0xffff {
                count += 1;
            }
            if count >= index {
                return (i + clen) as ssize_t;
            }
            i += clen;
        }
        -1
    }
}

/// The codepoint at byte `i` of `s[..len]`, and how many bytes it occupies.
///
/// A single byte is taken at face value rather than decoded, which is what
/// makes an invalid byte count as one character.
///
/// # Safety
///
/// `s` must point at `len` readable bytes and `i` must be less than `len`.
unsafe fn char_at(s: *const c_char, i: size_t, len: size_t) -> (c_int, size_t) {
    unsafe {
        let p = s.add(i);
        let clen = utf_ptr2len_len(p, (len - i) as c_int) as size_t;
        let c = if clen > 1 {
            utf_ptr2char(p)
        } else {
            *p as u8 as c_int
        };
        (c, clen)
    }
}

/// Does a cluster always end at this character, whatever precedes it?
fn always_break(bc: c_int) -> bool {
    bc == UTF8PROC_BOUNDCLASS_CONTROL as c_int
}

/// Do these two boundclasses always break between them?
///
/// The cheap half of the grapheme-cluster rules, answered from the two
/// boundclasses alone. `utf8proc_grapheme_break` is the full test; this is
/// what [`utf_head_off`] can use while walking *backwards*, where the stateful
/// one cannot be run.
fn always_break_two(bc1: c_int, bc2: c_int) -> bool {
    (bc1 != UTF8PROC_BOUNDCLASS_PREPEND as c_int && bc2 == UTF8PROC_BOUNDCLASS_OTHER as c_int)
        || (bc1 >= UTF8PROC_BOUNDCLASS_CR as c_int && bc1 <= UTF8PROC_BOUNDCLASS_CONTROL as c_int)
        || (bc2 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as c_int
            && (bc1 == UTF8PROC_BOUNDCLASS_OTHER as c_int
                || bc1 == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC as c_int))
}

/// The boundclass of a decoded codepoint.
fn boundclass(code: int32_t) -> c_int {
    utf8proc_get_property(code).boundclass as c_int
}

/// How far back from `p` the grapheme cluster covering it starts.
///
/// Three answers, in the order they are looked for: 0 if `p` is ASCII or part
/// of an illegal sequence (an illegal byte is its own character); the offset
/// to the lead byte if the character it starts always breaks; and otherwise
/// the result of backtracking over the cluster and then walking *forwards*
/// through it, because a backwards walk can overshoot and only a forwards one
/// knows where each cluster really ends.
///
/// # Safety
///
/// `base` must point at the start of a NUL-terminated string and `p` into it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_head_off(base_in: *const c_char, p_in: *const c_char) -> c_int {
    unsafe {
        if (*p_in as u8) < 0x80 {
            return 0;
        }
        let base = base_in as *const u8;
        let p = p_in as *const u8;

        // Walk back over continuation bytes to the lead byte of a sequence,
        // stopping at `base` or after a sequence's worth. It may stop *on* a
        // continuation byte when the sequence is overlong; the caller finds
        // that out by decoding. A closure, because it inherits this block.
        let step_back_to_lead = |mut from: *const u8, measure_from: *const u8| {
            while from > base
                && utf_is_trail_byte(*from)
                && measure_from.offset_from(from) < MAX_SEQUENCE
            {
                from = from.offset(-1);
            }
            from
        };

        let mut start = step_back_to_lead(p, p);
        let last_len = utf8len_tab[*start as usize];
        let mut cur_code = utf_ptr2char_info_impl(start, last_len as uintptr_t);
        if cur_code < 0 || p.offset_from(start) >= last_len as isize {
            return 0; // `p` is part of an illegal sequence
        }
        // Nothing past the character `p` sits in may be read while scanning
        // forwards: beyond it the bytes have not been validated.
        let safe_end = start.offset(last_len as isize);

        let mut cur_bc = boundclass(cur_code);
        if always_break(cur_bc) || start == base {
            return p.offset_from(start) as c_int;
        }

        // Backtrack over the cluster. This can go too far -- the backwards
        // walk cannot see where a cluster *ends* -- which the forwards scan
        // below corrects.
        let p_start = start;
        let mut cur_pos = start;
        while *start.offset(-1) != 0 {
            start = start.offset(-1);
            if *start < 0x80 {
                break; // ASCII never combines: done
            }
            start = step_back_to_lead(start, cur_pos);

            let prev_len = utf8len_tab[*start as usize] as c_int;
            let prev_code = utf_ptr2char_info_impl(start, prev_len as uintptr_t);
            if prev_code < 0 || (prev_len as isize) < cur_pos.offset_from(start) {
                start = cur_pos; // resume at the valid sequence after the junk
                break;
            }
            let prev_bc = boundclass(prev_code);
            if always_break_two(prev_bc, cur_bc)
                && !crate::arabic::arabic_combine(prev_code, cur_code)
            {
                start = cur_pos; // the previous character is not in this cluster
                break;
            }
            if start == base {
                break;
            }
            cur_pos = start;
            cur_bc = prev_bc;
            cur_code = prev_code;
        }

        // Never moved: `p` is inside the first character looked at.
        if start == p_start && last_len as isize > p.offset_from(start) {
            return p.offset_from(start) as c_int;
        }

        let mut q = start;
        while q < p {
            // Where the cluster *ends* does not matter; reaching `p`'s
            // codepoint is enough.
            let len = utfc_ptr2len_len(q as *const c_char, safe_end.offset_from(q) as c_int);
            if q.offset(len as isize) > p {
                return p.offset_from(q) as c_int;
            }
            q = q.offset(len as isize);
        }
        0
    }
}

/// The character after `cur`, when the byte after it is not ASCII.
///
/// The slow half of [`utfc_next`]: it has to decode each following character
/// to find out whether it composes onto this one.
///
/// # Safety
///
/// `cur` must describe a character in a NUL-terminated string, and the byte
/// after it must be `>= 0x80` — [`utfc_next`] guarantees both.
pub unsafe fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo {
    unsafe {
        let mut prev_code = cur.chr.value;
        let mut next = cur.ptr.offset(cur.chr.len as isize) as *mut u8;
        let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
        debug_assert!(*next >= 0x80, "*next >= 0x80");
        loop {
            let next_len = utf8len_tab[*next as usize];
            let next_code = utf_ptr2char_info_impl(next, next_len as uintptr_t);
            if !utf_iscomposing(prev_code, next_code, &raw mut state) {
                return StrCharInfo {
                    ptr: next as *mut c_char,
                    chr: CharInfo {
                        value: next_code,
                        len: if next_code < 0 { 1 } else { next_len as c_int },
                    },
                };
            }
            prev_code = next_code;
            next = next.offset(next_len as isize);
            if *next < 0x80 {
                return StrCharInfo {
                    ptr: next as *mut c_char,
                    chr: CharInfo {
                        value: *next as int32_t,
                        len: 1,
                    },
                };
            }
        }
    }
}

/// Copy one whole character (composing marks included) from `*fp` to `*tp`,
/// advancing both.
///
/// # Safety
///
/// `*fp` must point at a NUL-terminated string and `*tp` must have room for
/// the character.
pub unsafe fn mb_copy_char(fp: *mut *const c_char, tp: *mut *mut c_char) {
    unsafe {
        let l = utfc_ptr2len(*fp) as size_t;
        memmove(*tp as *mut c_void, *fp as *const c_void, l);
        *tp = (*tp).add(l);
        *fp = (*fp).add(l);
    }
}

/// How many bytes forward from `p` the next character starts, when `p` is in
/// the middle of one. Zero when `p` is already at a character start.
///
/// # Safety
///
/// `base` must point at the start of a NUL-terminated string and `p` into it.
pub unsafe fn mb_off_next(base: *const c_char, p: *const c_char) -> c_int {
    unsafe {
        let head_off = utf_head_off(base, p);
        if head_off == 0 {
            return 0;
        }
        utfc_ptr2len(p.offset(-(head_off as isize))) - head_off
    }
}

/// Both ends of the codepoint covering `p`, as offsets from it.
///
/// The answer for anything that is not a valid sequence is `(0, 1)` — "this
/// byte, on its own" — which is what makes every caller's arithmetic safe on
/// arbitrary bytes. Unlike [`utf_head_off`] this is about the *codepoint*,
/// not the cluster: composing characters are separate.
///
/// # Safety
///
/// `base <= p_in`, `p_len > 0`, and `p_in` must have `p_len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf_cp_bounds_len(
    base: *const c_char,
    p_in: *const c_char,
    p_len: c_int,
) -> CharBoundsOff {
    /// "This byte is its own character."
    const JUST_THIS_BYTE: CharBoundsOff = CharBoundsOff {
        begin_off: 0,
        end_off: 1,
    };
    unsafe {
        debug_assert!(base <= p_in && p_len > 0, "base <= p_in && p_len > 0");
        let b = base as *const u8;
        let p = p_in as *const u8;
        if *p < 0x80 {
            return JUST_THIS_BYTE;
        }

        // How far back the lead byte may be: never before `base`, and never
        // more than a sequence's worth.
        let max_first_off = -p.offset_from(b).min(MB_MAXCHAR as isize - 1) as c_int;
        let mut first_off: c_int = 0;
        while utf_is_trail_byte(*p.offset(first_off as isize)) {
            if first_off == max_first_off {
                return JUST_THIS_BYTE;
            }
            first_off -= 1;
        }

        // The sequence has to be complete *and* within `p_len`.
        let max_end_off = utf8len_tab[*p.offset(first_off as isize) as usize] as c_int + first_off;
        if max_end_off <= 0 || max_end_off > p_len {
            return JUST_THIS_BYTE;
        }
        for end_off in 1..max_end_off {
            if !utf_is_trail_byte(*p.offset(end_off as isize)) {
                return JUST_THIS_BYTE;
            }
        }
        CharBoundsOff {
            begin_off: -first_off as int8_t,
            end_off: max_end_off as int8_t,
        }
    }
}

/// [`utf_cp_bounds_len`] over a NUL-terminated string, where the length does
/// not bound anything.
///
/// # Safety
///
/// `base` must point at the start of a NUL-terminated string and `p_in` into
/// it.
pub unsafe fn utf_cp_bounds(base: *const c_char, p_in: *const c_char) -> CharBoundsOff {
    unsafe { utf_cp_bounds_len(base, p_in, INT_MAX) }
}

/// Move the cursor off the middle of a character, if it is in one.
///
/// # Safety
///
/// The editor's globals must be live.
pub unsafe fn mb_adjust_cursor() {
    unsafe { mark_mb_adjustpos(curbuf.get(), &raw mut (*curwin.get()).w_cursor) }
}

/// Pull `win`'s cursor back onto a character start, and clear a `coladd` that
/// no longer means anything.
///
/// `coladd == 1` is `'virtualedit'` saying "one cell into the character".
/// That is only meaningful for a character that is *drawn* more than one cell
/// wide; a Tab has its own rules, and an unprintable character is drawn as an
/// escape whose cells are not the character's.
///
/// # Safety
///
/// `win_` must be a live `win_T`.
pub unsafe fn mb_check_adjust_col(win_: *mut c_void) {
    unsafe {
        let win = win_ as *mut win_T;
        let oldcol = (*win).w_cursor.col;
        if oldcol == 0 {
            return;
        }
        let p = ml_get_buf((*win).w_buffer, (*win).w_cursor.lnum);
        let len = strlen(p) as colnr_T;
        if len == 0 || oldcol < 0 {
            (*win).w_cursor.col = 0;
        } else {
            if oldcol > len {
                (*win).w_cursor.col = len - 1;
            }
            (*win).w_cursor.col -= utf_head_off(p, p.offset((*win).w_cursor.col as isize));
        }
        let at_cursor = p.offset((*win).w_cursor.col as isize);
        if (*win).w_cursor.coladd == 1
            && *at_cursor as c_int != TAB
            && vim_isprintc(utf_ptr2char(at_cursor))
            && ptr2cells(at_cursor) > 1
        {
            (*win).w_cursor.coladd = 0;
        }
    }
}

/// The start of the character before `p`, or `p` itself at the start of the
/// line.
///
/// # Safety
///
/// `line` must point at the start of a NUL-terminated string and `p` into it.
pub unsafe fn mb_prevptr(line: *mut c_char, p: *mut c_char) -> *mut c_char {
    unsafe {
        if p <= line {
            return p;
        }
        p.offset(-(utf_head_off(line, p.offset(-1)) as isize + 1))
    }
}

/// How many characters a NUL-terminated string holds. A null pointer is zero.
///
/// # Safety
///
/// `str` must be null or point at a NUL-terminated string.
pub unsafe fn mb_charlen(str: *const c_char) -> c_int {
    unsafe {
        if str.is_null() {
            return 0;
        }
        let mut p = str;
        let mut count = 0;
        while *p != NUL as c_char {
            p = p.offset(utfc_ptr2len(p) as isize);
            count += 1;
        }
        count
    }
}

/// [`mb_charlen`] over at most `len` bytes.
///
/// # Safety
///
/// `str` must point at `len` readable bytes.
pub unsafe fn mb_charlen_len(str: *const c_char, len: c_int) -> c_int {
    unsafe {
        let mut p = str;
        let mut count = 0;
        while *p != NUL as c_char && p < str.offset(len as isize) {
            p = p.offset(utfc_ptr2len(p) as isize);
            count += 1;
        }
        count
    }
}

/// `ptr` paired with its codepoint: the start of a character and the
/// character itself. Composing characters are not consulted.
///
/// # Safety
///
/// `ptr` must point into a NUL-terminated string.
#[inline(always)]
pub unsafe fn utf_ptr2str_char_info(ptr: *mut c_char) -> StrCharInfo {
    unsafe {
        StrCharInfo {
            ptr,
            chr: utf_ptr2char_info(ptr),
        }
    }
}

/// The character after `cur`, treating a following composing character as
/// part of the *current* one.
///
/// The ASCII case is inlined because it is almost all of them; everything
/// else defers to [`utfc_next_impl`].
///
/// # Safety
///
/// `cur.ptr` must point into a NUL-terminated string, at a character start.
#[inline(always)]
pub unsafe fn utfc_next(cur: StrCharInfo) -> StrCharInfo {
    unsafe {
        let next = cur.ptr.offset(cur.chr.len as isize) as *mut u8;
        if *next < 0x80 {
            return StrCharInfo {
                ptr: next as *mut c_char,
                chr: CharInfo {
                    value: *next as int32_t,
                    len: 1,
                },
            };
        }
        utfc_next_impl(cur)
    }
}
