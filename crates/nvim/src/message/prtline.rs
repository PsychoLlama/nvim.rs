//! `:print` and `:list`: one buffer line onto the message area.
//!
//! [`msg_prt_line`] is the only message path that knows about `'listchars'`,
//! `'tabstop'` and the lead/trail/multispace distinctions, which is why it
//! duplicates so much of the drawing code's character loop.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::types::MB_MAXBYTES;
use core::ffi::{c_char, c_int};
use core::ptr;

/// The longest a `schar_T` renders to, matching upstream's `MAX_SCHAR_SIZE`.
const MAX_SCHAR_SIZE: usize = 32;

/// Show one line of buffer text, as `:print` and `:list` do.
///
/// Runs a cut-down version of the drawing code's character loop: each source
/// byte becomes one or more *cells*, and a cell that stands for several
/// columns (a tab, a `<xx>` escape) leaves the rest of them queued in
/// `extra_*` for the following turns of the loop.
///
/// # Safety
/// `s` must point at a NUL-terminated string.
pub unsafe fn msg_prt_line(s: *const c_char, list: bool) {
    unsafe {
        let mut s = s;
        // `'list'` on the window forces the listing form whatever the caller
        // asked for.
        let list = list || (*curwin.get()).w_onebuf_opt.wo_list != 0;
        let lcs = || (*curwin.get()).w_p_lcs_chars;

        // Where the trailing whitespace starts, and where the leading
        // whitespace ends; both null when no 'listchars' item needs them.
        let mut trail = ptr::null();
        let mut lead = ptr::null();
        if list {
            if lcs().trail != 0 {
                trail = s.add(strlen(s));
                while trail > s && ascii_iswhite(*trail.sub(1) as c_int) {
                    trail = trail.sub(1);
                }
            }
            if lcs().lead != 0 || !lcs().leadmultispace.is_null() || lcs().leadtab1 != 0 {
                lead = s;
                while ascii_iswhite(*lead as c_int) {
                    lead = lead.add(1);
                }
                // In a line of nothing but spaces they all count as trailing.
                if *lead == 0 {
                    lead = ptr::null();
                }
            }
        }

        // Output a space for an empty line, or it would not overwrite what is
        // already on the row.
        if *s == 0 && !(list && lcs().eol != 0) {
            msg_putchar(b' ' as c_int);
        }

        let mut col = 0;
        let mut hl_id = 0;
        let mut in_multispace = false;
        let mut multispace_pos = 0;
        // Cells still owed for the character just consumed: `extra_fill` for
        // all but the last, `extra_last` for the last one, or, when neither is
        // set, one byte at a time out of `extra_text`.
        let mut extra_left = 0;
        let mut extra_fill: schar_T = 0;
        let mut extra_last: schar_T = 0;
        let mut extra_text: *const c_char = ptr::null();

        while !got_int.get() {
            let sc: schar_T;
            if extra_left > 0 {
                extra_left -= 1;
                sc = if extra_left == 0 && extra_last != 0 {
                    extra_last
                } else if extra_fill != 0 {
                    extra_fill
                } else {
                    debug_assert!(!extra_text.is_null());
                    let byte = *extra_text as u8;
                    extra_text = extra_text.add(1);
                    byte as schar_T
                };
            } else {
                let len = utfc_ptr2len(s);
                if len > 1 {
                    // A multi-byte character goes out whole, not as a cell.
                    col += utf_ptr2cells(s);
                    let mut buf = [0 as c_char; MB_MAXBYTES + 1];
                    if len >= MB_MAXBYTES as c_int {
                        xstrlcpy(buf.as_mut_ptr(), c"?".as_ptr(), buf.len());
                    } else if lcs().nbsp != 0
                        && list
                        && (utf_ptr2char(s) == 160 || utf_ptr2char(s) == 0x202f)
                    {
                        schar_get(buf.as_mut_ptr(), lcs().nbsp);
                    } else {
                        ptr::copy_nonoverlapping(s, buf.as_mut_ptr(), len as usize);
                        buf[len as usize] = 0;
                    }
                    msg_puts(buf.as_ptr());
                    s = s.add(len as usize);
                    continue;
                }

                hl_id = 0;
                let c = *s as u8 as c_int;
                s = s.add(1);
                if c >= 0x80 {
                    // Illegal byte.
                    col += utf_char2cells(c);
                    msg_putchar(c);
                    continue;
                }
                extra_fill = 0;
                extra_last = 0;
                if list {
                    // `s` is already past `c`, so `s[-2]` is the byte before
                    // it -- the lookbehind that makes the *second* space of a
                    // run count as multispace too.
                    in_multispace = c == b' ' as c_int
                        && (*s == b' ' as c_char || (col > 0 && *s.sub(2) == b' ' as c_char));
                    if !in_multispace {
                        multispace_pos = 0;
                    }
                }

                if c == TAB && (!list || lcs().tab1 != 0) {
                    // How wide the tab is depends on where it starts.
                    extra_left = tabstop_padding(
                        col as colnr_T,
                        (*curbuf.get()).b_p_ts,
                        (*curbuf.get()).b_p_vts_array,
                    ) - 1;
                    if list {
                        let (mut tab1, mut tab2, mut tab3) = (lcs().tab1, lcs().tab2, lcs().tab3);
                        if !lead.is_null() && s <= lead && lcs().leadtab1 != 0 {
                            tab1 = lcs().leadtab1;
                            tab2 = lcs().leadtab2;
                            tab3 = lcs().leadtab3;
                        }
                        sc = if extra_left == 0 && tab3 != 0 {
                            tab3
                        } else {
                            tab1
                        };
                        extra_fill = tab2;
                        extra_last = tab3;
                        hl_id = HLF_0;
                    } else {
                        sc = b' ' as schar_T;
                        extra_fill = b' ' as schar_T;
                    }
                } else if c == NUL && list && lcs().eol != 0 {
                    // One more turn of the loop, which reads the NUL out of
                    // `extra_text` and stops.
                    extra_text = c"".as_ptr();
                    extra_left = 1;
                    sc = lcs().eol;
                    hl_id = HLF_AT;
                    s = s.sub(1);
                } else if c != NUL && byte2cells(c) > 1 {
                    // An unprintable byte, shown as `<xx>`.
                    extra_left = byte2cells(c) - 1;
                    extra_text = transchar_byte_buf(ptr::null(), c);
                    sc = *extra_text as schar_T;
                    extra_text = extra_text.add(1);
                    // Its own highlight, so `<ff>` can be told apart from the
                    // same four characters typed literally.
                    hl_id = HLF_0;
                } else if c == b' ' as c_int {
                    hl_id = HLF_0;
                    sc = if !lead.is_null()
                        && s <= lead
                        && in_multispace
                        && !lcs().leadmultispace.is_null()
                    {
                        cycle(lcs().leadmultispace, &mut multispace_pos)
                    } else if !lead.is_null() && s <= lead && lcs().lead != 0 {
                        lcs().lead
                    } else if !trail.is_null() && s > trail {
                        lcs().trail
                    } else if in_multispace && !lcs().multispace.is_null() {
                        cycle(lcs().multispace, &mut multispace_pos)
                    } else if list && lcs().space != 0 {
                        lcs().space
                    } else {
                        hl_id = 0;
                        b' ' as schar_T
                    };
                } else {
                    sc = c as schar_T;
                }
            }

            if sc == 0 {
                break;
            }
            emit(sc, hl_id, &mut col);
        }
        msg_clr_eos();
    }
}

/// Put one cell on the message area.
unsafe fn emit(sc: schar_T, hl_id: c_int, col: &mut c_int) {
    unsafe {
        // TODO(bfredl): this is such baloney. need msg_put_schar
        let mut buf = [0 as c_char; MAX_SCHAR_SIZE];
        schar_get(buf.as_mut_ptr(), sc);
        msg_puts_hl(buf.as_ptr(), hl_id, false);
        *col += 1;
    }
}

/// One character of a cycling `'listchars'` sequence, wrapping at its end.
unsafe fn cycle(seq: *const schar_T, at: &mut c_int) -> schar_T {
    unsafe {
        let sc = *seq.offset(*at as isize);
        *at += 1;
        if *seq.offset(*at as isize) == 0 {
            *at = 0;
        }
        sc
    }
}
