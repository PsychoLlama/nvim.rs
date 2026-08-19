//! CTRL-A and CTRL-X -- incrementing the number under the cursor.
//!
//! [`op_addsub`] is the operator wrapper: over a Visual region it runs
//! [`do_addsub`] once per line, and `g CTRL-A` grows the amount by the count
//! each time a line actually changed, which is how a column of numbers becomes
//! a sequence.
//!
//! [`do_addsub`] is the per-line work, and it is four questions:
//!
//! 1. **where is the number** ([`find_number_start`] outside Visual mode,
//!    [`visual_skip_to_number`] inside it). 'nrformats' decides what counts as
//!    one, and the hexadecimal and binary patterns overlap -- `0b1` is a valid
//!    hex number -- so the scan has to back off and retry;
//! 2. **is it negative** ([`minus_before`]), which 'nrformats' `u` and `k` can
//!    both veto;
//! 3. **what is the new value** ([`add_or_subtract`]), in *unsigned* 64-bit
//!    arithmetic with the sign kept beside it, so that wrapping past zero
//!    flips the sign rather than the bit pattern;
//! 4. **how is it written back** ([`render_number`]), preserving the original
//!    spelling: the `0x`/`0b`/`0` prefix, the case of the hex digits, and
//!    enough leading zeros to keep the number the same width.
//!
//! A single alphabetic character is the fifth case ('nrformats' `p`), and it
//! short-circuits all of the above.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::smsg_c;
use core::ffi::{c_char, c_int, c_ulong, c_void};

use super::*;
use crate::types::FAIL;

/// Case of the hex digits last seen, so that `0xAB` increments to `0xAC` and
/// `0xab` to `0xac`.
///
/// A `static` in C too: it survives between calls on purpose, so that a number
/// with no letters in it (`0x10`) keeps the case of the last one that had.
static HEX_UPPER: GlobalCell<bool> = GlobalCell::new(false);

/// The 'nrformats' letters, read once per call.
struct NrFormats {
    /// `x` -- `0x1f` is a number.
    hex: bool,
    /// `o` -- `017` is a number.
    oct: bool,
    /// `b` -- `0b101` is a number.
    bin: bool,
    /// `p` -- a single letter is a "number".
    alpha: bool,
    /// `u` -- a `-` is never a sign.
    unsigned: bool,
    /// `k` -- a `-` is only a sign after white space.
    blank: bool,
}

impl NrFormats {
    /// Read the current buffer's 'nrformats'.
    ///
    /// # Safety
    /// The current buffer's option string must be valid.
    unsafe fn current() -> Self {
        unsafe {
            let has = |c: u8| !vim_strchr((*curbuf.get()).b_p_nf, c_int::from(c)).is_null();
            NrFormats {
                hex: has(b'x'),
                oct: has(b'o'),
                bin: has(b'b'),
                alpha: has(b'p'),
                unsigned: has(b'u'),
                blank: has(b'k'),
            }
        }
    }
}

/// What the scan decided about a `-` in front of the number.
enum Minus {
    /// There is none, or it does not belong to the number.
    Absent,
    /// The number is negative.
    Negative,
    /// 'nrformats' `k`: the `-` does not follow white space, so it is a dash
    /// rather than a sign and the number is treated as unsigned.
    BlankUnsigned,
}

/// CTRL-A and CTRL-X as an operator.
///
/// `g_cmd` is `g CTRL-A`: add `prenum1` to the first changed line, twice that
/// to the second, and so on.
///
/// # Safety
/// `oap` must point to a live `oparg_T` describing a region of the current
/// buffer.
pub unsafe fn op_addsub(oap: *mut oparg_T, prenum1: linenr_T, g_cmd: bool) {
    unsafe {
        // 'foldexpr' may be re-evaluated part way through, and it must not see
        // the buffer mid-operation.
        *disable_fold_update.ptr() += 1;

        if !VIsual_active.get() {
            let mut pos = (*curwin.get()).w_cursor;
            if u_save_cursor() == FAIL {
                *disable_fold_update.ptr() -= 1;
                return;
            }
            let changed = do_addsub((*oap).op_type, &raw mut pos, 0, prenum1);
            *disable_fold_update.ptr() -= 1;
            if changed {
                changed_lines(curbuf.get(), pos.lnum, 0, pos.lnum + 1, 0, true);
            }
            return;
        }

        if u_save((*oap).start.lnum - 1, (*oap).end.lnum + 1) == FAIL {
            *disable_fold_update.ptr() -= 1;
            return;
        }

        let mut bd = block_def::ZERO;
        let mut change_cnt: ssize_t = 0;
        let mut startpos = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut amount = prenum1;

        let mut pos = (*oap).start;
        while pos.lnum <= (*oap).end.lnum {
            let length = addsub_line_span(oap, &mut bd, &mut pos);
            let one_change = do_addsub((*oap).op_type, &raw mut pos, length, amount);
            if one_change {
                if change_cnt == 0 {
                    startpos = (*curbuf.get()).b_op_start;
                }
                change_cnt += 1;
                if g_cmd {
                    amount += prenum1;
                }
            }
            pos.lnum += 1;
        }

        *disable_fold_update.ptr() -= 1;
        if change_cnt != 0 {
            changed_lines(
                curbuf.get(),
                (*oap).start.lnum,
                0,
                (*oap).end.lnum + 1,
                0,
                true,
            );
        } else if (*oap).is_VIsual {
            // Nothing changed, so the selection has to come off the screen.
            redraw_curbuf_later(UPD_INVERTED);
        }
        if change_cnt > 0 && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as c_int == 0 {
            (*curbuf.get()).b_op_start = startpos;
        }
        if change_cnt > p_report.get() as ssize_t {
            smsg_c!(
                0,
                ngettext(
                    c"%ld lines changed".as_ptr(),
                    c"%ld lines changed".as_ptr(),
                    change_cnt as c_ulong,
                ),
                change_cnt as int64_t,
            );
        }
    }
}

/// The part of `pos.lnum` the operator covers: sets `pos.col` and answers the
/// length in bytes.
///
/// # Safety
/// `oap` must point to a live `oparg_T`; `pos.lnum` must be inside it.
unsafe fn addsub_line_span(oap: *mut oparg_T, bd: &mut block_def, pos: &mut pos_T) -> c_int {
    unsafe {
        if (*oap).motion_type == kMTBlockWise {
            block_prep(oap, &raw mut *bd, pos.lnum, false);
            pos.col = bd.textcol;
            return bd.textlen;
        }
        if (*oap).motion_type == kMTLineWise {
            (*curwin.get()).w_cursor.col = 0;
            pos.col = 0;
            return ml_get_len(pos.lnum);
        }

        // Charwise: the first and last lines are clipped to the region.
        if pos.lnum == (*oap).start.lnum && !(*oap).inclusive {
            dec(&raw mut (*oap).end);
        }
        let mut length = ml_get_len(pos.lnum);
        pos.col = 0;
        if pos.lnum == (*oap).start.lnum {
            pos.col += (*oap).start.col;
            length -= (*oap).start.col;
        }
        if pos.lnum == (*oap).end.lnum {
            length = ml_get_len((*oap).end.lnum);
            (*oap).end.col = (*oap).end.col.min(length - 1);
            length = (*oap).end.col - pos.col + 1;
        }
        length
    }
}

/// Add `prenum1` to (or subtract it from) the number at `pos`.
///
/// `length` is the region the caller allows the number to occupy; it is 0
/// outside Visual mode, where the number's own extent decides instead.
///
/// # Safety
/// `pos` must name a position in the current buffer.
pub unsafe fn do_addsub(
    op_type: OpType,
    pos: *mut pos_T,
    mut length: c_int,
    prenum1: linenr_T,
) -> bool {
    unsafe {
        let fmt = NrFormats::current();
        let visual = VIsual_active.get();
        let save_cursor = (*curwin.get()).w_cursor;

        let mut save_coladd: colnr_T = 0;
        if virtual_active(curwin.get()) {
            save_coladd = (*pos).coladd;
            (*pos).coladd = 0;
        }

        (*curwin.get()).w_cursor = *pos;
        let ptr = ml_get((*pos).lnum);
        let linelen = ml_get_len((*pos).lnum);
        let mut col = (*pos).col;

        let mut did_change = false;
        if col + c_int::from(save_coladd != 0) < linelen {
            let mut negative = false;
            let mut was_positive = true;
            let mut blank_unsigned = false;

            if !visual {
                col = find_number_start(ptr, (*pos).col, &fmt);
            } else {
                match visual_skip_to_number(ptr, col, length, &fmt) {
                    Some((c, l)) => {
                        col = c;
                        length = l;
                    }
                    // The selection holds no number at all.
                    None => return finish_addsub(visual, false, save_cursor, save_coladd),
                }
                match minus_before(ptr, col, (*pos).col, &fmt) {
                    Minus::Absent => {}
                    Minus::Negative => {
                        negative = true;
                        was_positive = false;
                    }
                    Minus::BlankUnsigned => blank_unsigned = true,
                }
            }

            let firstdigit = *ptr.offset(col as isize) as u8 as c_int;
            let is_alpha = fmt.alpha && ascii_isalpha(firstdigit);
            if !ascii_isdigit(firstdigit) && !is_alpha {
                beep_flush();
            } else {
                let (startpos, endpos) = if is_alpha {
                    bump_alpha_char(firstdigit, op_type, prenum1, col)
                } else {
                    replace_number(
                        op_type,
                        &mut length,
                        prenum1,
                        &fmt,
                        Scan {
                            ptr,
                            linelen,
                            col,
                            firstdigit,
                            visual,
                            negative,
                            was_positive,
                            blank_unsigned,
                        },
                    )
                };
                did_change = true;

                if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as c_int == 0 {
                    (*curbuf.get()).b_op_start = startpos;
                    (*curbuf.get()).b_op_end = endpos;
                    if (*curbuf.get()).b_op_end.col > 0 {
                        (*curbuf.get()).b_op_end.col -= 1;
                    }
                }
            }
        }

        finish_addsub(visual, did_change, save_cursor, save_coladd)
    }
}

/// Put the cursor back where the caller expects it, and answer `did_change`.
///
/// # Safety
/// Operates on the current window's cursor.
unsafe fn finish_addsub(
    visual: bool,
    did_change: bool,
    save_cursor: pos_T,
    save_coladd: colnr_T,
) -> bool {
    unsafe {
        if visual {
            (*curwin.get()).w_cursor = save_cursor;
        } else if did_change {
            (*curwin.get()).w_set_curswant = true_0;
        } else if virtual_active(curwin.get()) {
            (*curwin.get()).w_cursor.coladd = save_coladd;
        }
        did_change
    }
}

/// Outside Visual mode: find the column the number under `start_col` begins at.
///
/// Works backwards from the cursor, because the cursor may be *inside* the
/// number. The awkward part is that the hexadecimal and binary patterns
/// overlap -- every binary digit is also a hex digit -- so a backwards scan
/// over hex digits can run past the start of a `0b...` number; the scan is
/// then redone over decimal digits only. When neither prefix is found, it
/// falls back to searching forwards for a digit and then backwards to that
/// number's first one.
///
/// # Safety
/// `ptr` must be a NUL-terminated line and `start_col` a column in it.
unsafe fn find_number_start(ptr: *mut c_char, start_col: colnr_T, fmt: &NrFormats) -> colnr_T {
    unsafe {
        let byte = |c: colnr_T| *ptr.offset(c as isize) as c_int;
        // Step back one character, not one byte.
        let back = |c: colnr_T| {
            let c = c - 1;
            c - utf_head_off(ptr, ptr.offset(c as isize))
        };
        // `0x`/`0b` at `col`, with a digit of that base after it.
        let prefixed_at = |c: colnr_T, upper: u8, lower: u8, digit: fn(c_int) -> bool| {
            c > 0
                && (byte(c) == c_int::from(upper) || byte(c) == c_int::from(lower))
                && byte(c - 1) == '0' as c_int
                && utf_head_off(ptr, ptr.offset(c as isize).offset(-1)) == 0
                && digit(byte(c + 1))
        };

        let mut col = start_col;
        if fmt.bin {
            while col > 0 && ascii_isbdigit(byte(col)) {
                col = back(col);
            }
        }
        if fmt.hex {
            while col > 0 && ascii_isxdigit(byte(col)) {
                col = back(col);
            }
        }
        if fmt.bin && fmt.hex && !prefixed_at(col, b'X', b'x', ascii_isxdigit) {
            // Binary and hexadecimal overlap: rescan over decimal digits.
            col = (*curwin.get()).w_cursor.col;
            while col > 0 && ascii_isdigit(byte(col)) {
                col = back(col);
            }
        }

        if (fmt.hex && prefixed_at(col, b'X', b'x', ascii_isxdigit))
            || (fmt.bin && prefixed_at(col, b'B', b'b', ascii_isbdigit))
        {
            // On the base letter of a `0x`/`0b` number: move onto the `0`.
            return back(col);
        }

        // No prefix: search forwards for a digit, then back to its number's
        // first one.
        col = start_col;
        while byte(col) != NUL
            && !ascii_isdigit(byte(col))
            && !(fmt.alpha && ascii_isalpha(byte(col)))
        {
            col += 1;
        }
        while col > 0 && ascii_isdigit(byte(col - 1)) && !(fmt.alpha && ascii_isalpha(byte(col))) {
            col -= 1;
        }
        col
    }
}

/// Inside Visual mode: skip forwards to the first number in the selection.
///
/// Answers the column it starts at and how much of the selection is left, or
/// `None` when the selection runs out first.
///
/// # Safety
/// `ptr` must be a NUL-terminated line and `col` a column in it.
unsafe fn visual_skip_to_number(
    ptr: *mut c_char,
    mut col: colnr_T,
    mut length: c_int,
    fmt: &NrFormats,
) -> Option<(colnr_T, c_int)> {
    unsafe {
        let byte = |c: colnr_T| *ptr.offset(c as isize) as c_int;
        while byte(col) != NUL
            && length > 0
            && !ascii_isdigit(byte(col))
            && !(fmt.alpha && ascii_isalpha(byte(col)))
        {
            let mb_len = utfc_ptr2len(ptr.offset(col as isize));
            col += mb_len;
            length -= mb_len;
        }
        (length != 0).then_some((col, length))
    }
}

/// Is the character in front of `col` a minus sign belonging to the number?
///
/// `min_col` is the first column the caller is willing to look before: the
/// selection's start in Visual mode, 0 outside it.
///
/// # Safety
/// `ptr` must be a NUL-terminated line and `col` a column in it.
unsafe fn minus_before(ptr: *mut c_char, col: colnr_T, min_col: colnr_T, fmt: &NrFormats) -> Minus {
    unsafe {
        if !(col > min_col
            && *ptr.offset((col - 1) as isize) as c_int == '-' as c_int
            && utf_head_off(ptr, ptr.offset(col as isize).offset(-1)) == 0
            && !fmt.unsigned)
        {
            return Minus::Absent;
        }
        if fmt.blank && col >= 2 && !ascii_iswhite(*ptr.offset((col - 2) as isize) as c_int) {
            Minus::BlankUnsigned
        } else {
            Minus::Negative
        }
    }
}

/// 'nrformats' `p`: step a single letter along the alphabet, clamped at `a`/`A`
/// and `z`/`Z`.
///
/// Answers the `'[`/`']` positions.
///
/// # Safety
/// The cursor must be on the line holding `col`.
unsafe fn bump_alpha_char(
    mut firstdigit: c_int,
    op_type: OpType,
    prenum1: linenr_T,
    col: colnr_T,
) -> (pos_T, pos_T) {
    unsafe {
        // The letter's ordinal within its own case.
        let ord = linenr_T::from(if firstdigit < 'a' as c_int {
            firstdigit - 'A' as c_int
        } else {
            firstdigit - 'a' as c_int
        });
        let upper = *(*__ctype_b_loc()).offset(firstdigit as isize) as c_int
            & _ISupper as ::core::ffi::c_ushort as c_int
            != 0;
        if op_type == OP_NR_SUB {
            if ord < prenum1 {
                firstdigit = if upper { 'A' as c_int } else { 'a' as c_int };
            } else {
                firstdigit -= prenum1 as c_int;
            }
        } else if 26 - ord - 1 < prenum1 {
            firstdigit = if upper { 'Z' as c_int } else { 'z' as c_int };
        } else {
            firstdigit += prenum1 as c_int;
        }

        (*curwin.get()).w_cursor.col = col;
        let startpos = (*curwin.get()).w_cursor;
        del_char(false);
        ins_char(firstdigit);
        let endpos = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.col = col;
        (startpos, endpos)
    }
}

/// What the scan for the number found, handed to [`replace_number`].
struct Scan {
    /// The line the number is in.
    ptr: *mut c_char,
    /// Its length in bytes.
    linelen: c_int,
    /// Column the number starts at.
    col: colnr_T,
    /// First byte of the number, which decides whether leading zeros are kept.
    firstdigit: c_int,
    /// A Visual selection is active.
    visual: bool,
    /// A `-` in front of the number belongs to it.
    negative: bool,
    /// The number was *not* negative before the operation.
    was_positive: bool,
    /// 'nrformats' `k` vetoed the sign; a wrap must stick rather than go
    /// negative.
    blank_unsigned: bool,
}

/// Replace the number at `scan.col` with the result of adding `prenum1`.
///
/// Answers the `'[`/`']` positions.
///
/// # Safety
/// `scan.ptr` must be the current buffer's line at `pos.lnum`, and the cursor
/// must be on it.
unsafe fn replace_number(
    op_type: OpType,
    length: &mut c_int,
    prenum1: linenr_T,
    fmt: &NrFormats,
    scan: Scan,
) -> (pos_T, pos_T) {
    unsafe {
        let Scan {
            ptr,
            linelen,
            mut col,
            firstdigit,
            visual,
            mut negative,
            was_positive,
            mut blank_unsigned,
        } = scan;

        if !visual {
            match minus_before(ptr, col, 0, fmt) {
                Minus::Absent => {}
                Minus::Negative => {
                    col -= 1;
                    negative = true;
                }
                Minus::BlankUnsigned => blank_unsigned = true,
            }
        }

        // How far the number may run. Only bounded in Visual mode, and not for
        // a linewise selection or one opened with `$`.
        let mut maxlen = 0;
        if visual && VIsual_mode.get() != 'V' as c_int {
            maxlen = if (*curbuf.get()).b_visual.vi_curswant == MAXCOL {
                linelen - col
            } else {
                *length
            };
        }

        // `pre` is the base marker: 'x'/'X' hex, 'b'/'B' binary, '0' octal, 0
        // decimal.
        let mut pre: c_int = 0;
        let mut n: uvarnumber_T = 0;
        let mut overflow = false;
        vim_str2nr(
            ptr.offset(col as isize),
            &raw mut pre,
            length,
            (if fmt.bin { STR2NR_BIN as c_int } else { 0 })
                + (if fmt.oct { STR2NR_OCT as c_int } else { 0 })
                + (if fmt.hex { STR2NR_HEX as c_int } else { 0 }),
            ::core::ptr::null_mut(),
            &raw mut n,
            maxlen,
            false,
            &raw mut overflow,
        );

        // A leading `-` is not a sign for hex, octal or binary.
        if pre != 0 && negative {
            col += 1;
            *length -= 1;
            negative = false;
        }

        let subtract = (op_type == OP_NR_SUB) ^ negative;
        (n, negative) = add_or_subtract(n, prenum1, subtract, negative, overflow, pre != 0);

        if (fmt.unsigned || blank_unsigned) && negative {
            // Stick at 0 going down and at 2^64 - 1 going up.
            n = if subtract { 0 } else { uvarnumber_T::MAX };
            negative = false;
        }

        if visual && !was_positive && !negative && col > 0 {
            // The `-` has to go.
            col -= 1;
            *length += 1;
        }

        // Delete the old number.
        (*curwin.get()).w_cursor.col = col;
        let startpos = (*curwin.get()).w_cursor;
        let mut todel = *length;
        let mut c = gchar_cursor();
        // The `-` is not part of the length: only the part after it keeps its
        // width.
        if c == '-' as c_int {
            *length -= 1;
        }
        while todel > 0 {
            todel -= 1;
            if c < 0x100
                && *(*__ctype_b_loc()).offset(c as isize) as c_int
                    & _ISalpha as ::core::ffi::c_ushort as c_int
                    != 0
            {
                HEX_UPPER.set(
                    *(*__ctype_b_loc()).offset(c as isize) as c_int
                        & _ISupper as ::core::ffi::c_ushort as c_int
                        != 0,
                );
            }
            del_char(false);
            c = gchar_cursor();
        }

        render_number(
            n,
            pre,
            *length,
            firstdigit,
            negative,
            visual,
            was_positive,
            fmt,
        );

        let endpos = (*curwin.get()).w_cursor;
        if (*curwin.get()).w_cursor.col != 0 {
            (*curwin.get()).w_cursor.col -= 1;
        }
        (startpos, endpos)
    }
}

/// Apply the increment, in unsigned arithmetic with the sign beside it.
///
/// Answers the new magnitude and sign. A decimal number that wraps past zero
/// changes sign and keeps its magnitude, which is why the two's complement is
/// taken by hand rather than letting the bit pattern stand; a prefixed number
/// (`pre`) wraps as a bit pattern instead, which is what a hex counter should
/// do. `overflow` means the *original* did not fit in 64 bits, and then
/// nothing is added at all.
fn add_or_subtract(
    mut n: uvarnumber_T,
    prenum1: linenr_T,
    subtract: bool,
    mut negative: bool,
    overflow: bool,
    prefixed: bool,
) -> (uvarnumber_T, bool) {
    let oldn = n;
    if !overflow {
        n = if subtract {
            n.wrapping_sub(prenum1 as uvarnumber_T)
        } else {
            n.wrapping_add(prenum1 as uvarnumber_T)
        };
    }

    if !prefixed {
        if subtract {
            if n > oldn {
                n = (!n).wrapping_add(1);
                negative = !negative;
            }
        } else if n < oldn {
            n = !n;
            negative = !negative;
        }
        if n == 0 {
            negative = false;
        }
    }
    (n, negative)
}

/// Write the new number in at the cursor, in the old one's spelling.
///
/// `length` is what is left of the original's width after the sign and the
/// prefix, and is spent on leading zeros so that the number stays the same
/// width -- except when it would then read as octal.
///
/// # Safety
/// The cursor must be where the old number was deleted from.
#[allow(clippy::too_many_arguments)]
unsafe fn render_number(
    n: uvarnumber_T,
    pre: c_int,
    mut length: c_int,
    firstdigit: c_int,
    negative: bool,
    visual: bool,
    was_positive: bool,
    fmt: &NrFormats,
) {
    unsafe {
        // Sized before the decrements below, as upstream does: with many
        // leading zeros the prefix can be long, so this is deliberately
        // generous rather than exact.
        let buf = xmalloc(length as size_t + NUMBUFLEN as size_t) as *mut c_char;
        let mut at = buf;
        if negative && (!visual || was_positive) {
            *at = '-' as c_char;
            at = at.offset(1);
        }
        if pre != 0 {
            *at = '0' as c_char;
            at = at.offset(1);
            length -= 1;
        }
        if pre == 'b' as c_int || pre == 'B' as c_int || pre == 'x' as c_int || pre == 'X' as c_int
        {
            *at = pre as c_char;
            at = at.offset(1);
            length -= 1;
        }

        // The digits themselves.
        let mut digits: [c_char; NUMBUFLEN as usize] = [0; NUMBUFLEN as usize];
        let digits_len = if pre == 'b' as c_int || pre == 'B' as c_int {
            format_binary(n, &mut digits)
        } else {
            let format = if pre == 0 {
                c"%lu"
            } else if pre == '0' as c_int {
                c"%lo"
            } else if HEX_UPPER.get() {
                c"%lX"
            } else {
                c"%lx"
            };
            vim_snprintf(
                &raw mut digits as *mut c_char,
                digits.len(),
                format.as_ptr(),
                n,
            )
        };
        length -= digits_len;

        // Keep the total width by padding with zeros -- unless the result
        // would then look like an octal number.
        if firstdigit == '0' as c_int && !(fmt.oct && pre == 0) {
            while length > 0 {
                length -= 1;
                *at = '0' as c_char;
                at = at.offset(1);
            }
        }
        *at = NUL as c_char;

        let mut buflen = at.offset_from(buf) as c_int;
        strcpy(
            buf.offset(buflen as isize),
            &raw const digits as *const c_char,
        );
        buflen += digits_len;

        ins_str(buf, buflen as size_t);
        xfree(buf as *mut c_void);
    }
}

/// Write `n` in binary, most significant one-bit first; answers its length.
///
/// Truncates rather than overflowing `out`, which is why it is not a
/// `vim_snprintf` call like the other three bases.
fn format_binary(n: uvarnumber_T, out: &mut [c_char; NUMBUFLEN as usize]) -> c_int {
    // Skip the leading zeros.
    let mut bits = 8 * ::core::mem::size_of::<uvarnumber_T>();
    while bits > 0 && (n >> (bits - 1)) & 0x1 == 0 {
        bits -= 1;
    }

    let mut len = 0;
    while bits > 0 && len < NUMBUFLEN as usize - 1 {
        bits -= 1;
        out[len] = if (n >> bits) & 0x1 != 0 {
            b'1' as c_char
        } else {
            b'0' as c_char
        };
        len += 1;
    }
    out[len] = NUL as c_char;
    len as c_int
}
