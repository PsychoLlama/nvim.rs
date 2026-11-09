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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::{Str2NrBases, str2nr_scan};
use crate::ex_docmd::cmdmod_has;
use crate::guard::Suppress;
use crate::message_fmt::report_msg;
use crate::narrow::len_as_int;
use crate::os::cshim::ctype_bits;
use crate::tr_plural;
use crate::types::Key;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::ascii::NUL;
use crate::normal::{visual_active, visual_mode};
use crate::optionstr::OptString;

/// Case of the hex digits last seen, so that `0xAB` increments to `0xAC` and
/// `0xab` to `0xac`.
///
/// A `static` in C too: it survives between calls on purpose, so that a number
/// with no letters in it (`0x10`) keeps the case of the last one that had.
static HEX_UPPER: GlobalCell<bool> = GlobalCell::new(false);

/// The 'nrformats' letters, read once per call.
#[derive(Debug)]
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
    fn current() -> Self {
        let has = |c: u8| Buf::current().b_p_nf.has_byte(c);
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

/// What the scan decided about a `-` in front of the number.
#[derive(Debug)]
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
/// `op` describes a region of the current buffer; everything below works on
/// that region and on the cursor line.
pub fn op_addsub(op: &mut OpArg, prenum1: i32, g_cmd: bool) {
    // 'foldexpr' may be re-evaluated part way through, and it must not see
    // the buffer mid-operation.
    let folds_frozen = Suppress::fold_update();

    if !visual_active() {
        let mut pos = Win::current().w_cursor;
        if u_save_cursor().is_err() {
            return;
        }
        let changed = do_addsub(op.op_type, &mut pos, 0, prenum1);
        drop(folds_frozen);
        if changed {
            changed_lines(Buf::current(), pos.lnum, ColNr::ZERO, pos.lnum + 1, 0, true);
        }
        return;
    }

    let (above, below) = (op.start.lnum - 1, op.end.lnum + 1);
    if u_save(above, below).is_err() {
        return;
    }

    let mut bd = BlockDef::ZERO;
    let mut change_cnt: ssize_t = 0;
    let mut startpos = Pos {
        lnum: LineNr::ZERO,
        col: ColNr::ZERO,
        coladd: 0,
    };
    let mut amount = prenum1;

    let mut pos = op.start;
    while pos.lnum <= op.end.lnum {
        let length = addsub_line_span(op, &mut bd, &mut pos);
        let one_change = do_addsub(op.op_type, &mut pos, length, amount);
        if one_change {
            if change_cnt == 0 {
                startpos = Buf::current().b_op_start;
            }
            change_cnt += 1;
            if g_cmd {
                amount += prenum1;
            }
        }
        pos.lnum += 1;
    }

    drop(folds_frozen);
    if change_cnt != 0 {
        let (first, last) = (op.start.lnum, op.end.lnum + 1);
        changed_lines(Buf::current(), first, ColNr::ZERO, last, 0, true);
    } else if op.is_visual {
        // Nothing changed, so the selection has to come off the screen.
        redraw_curbuf_later(UPD_INVERTED);
    }
    if change_cnt > 0 && !cmdmod_has(CmdModFlags::LOCKMARKS) {
        Buf::current().b_op_start = startpos;
    }
    // A count of lines, which no buffer takes past `i64`.
    let changed = i64::try_from(change_cnt).unwrap_or(i64::MAX);
    if changed > p_report() {
        let fmt = ngettext(
            c"%ld lines changed",
            c"%ld lines changed",
            changed.cast_unsigned(),
        );
        let _: bool = report_msg(0, || tr_plural!(fmt, changed));
    }
}

/// The part of `pos.lnum` the operator covers: sets `pos.col` and answers the
/// length in bytes.
///
/// `pos.lnum` must be a line of the region.
fn addsub_line_span(op: &mut OpArg, bd: &mut BlockDef, pos: &mut Pos) -> c_int {
    // Every line touched below is one of the region's.
    if op.motion_type == MT_BLOCK_WISE {
        block_prep(bd, op, pos.lnum, false);
        pos.col = bd.textcol;
        return bd.textlen;
    }
    if op.motion_type == MT_LINE_WISE {
        Win::current().w_cursor.col = ColNr::ZERO;
        pos.col = ColNr::ZERO;
        return Lines::current().line_len(pos.lnum).get();
    }

    // Charwise: the first and last lines are clipped to the region.
    if pos.lnum == op.start.lnum && !op.inclusive {
        dec(&mut op.end);
    }
    let mut length = Lines::current().line_len(pos.lnum).get();
    pos.col = ColNr::ZERO;
    if pos.lnum == op.start.lnum {
        pos.col = op.start.col;
        length -= op.start.col.get();
    }
    if pos.lnum == op.end.lnum {
        let end_len = Lines::current().line_len(op.end.lnum);
        op.end.col = op.end.col.min(end_len - 1);
        length = op.end.col - pos.col + 1;
    }
    length
}

/// Add `prenum1` to (or subtract it from) the number at `pos`.
///
/// `length` is the region the caller allows the number to occupy; it is 0
/// outside Visual mode, where the number's own extent decides instead. `pos`
/// names a position in the current buffer.
pub fn do_addsub(op_type: OpType, pos: &mut Pos, mut length: c_int, prenum1: i32) -> bool {
    let fmt = NrFormats::current();
    let visual = visual_active();
    let save_cursor = Win::current().w_cursor;

    let mut save_coladd: i32 = 0;
    if virtual_active(Win::current()) {
        save_coladd = pos.coladd;
        pos.coladd = 0;
    }

    Win::current().w_cursor = *pos;
    // A copy: the number is *replaced* below, by deletions and insertions
    // that re-enter the editor, and the scan's reads of the line have to
    // survive them.  The copy keeps the line's terminator, which every walk
    // here stops on.
    let text = Lines::current().line_copy(pos.lnum);
    let linelen = ColNr::try_from(text.len()).unwrap_or(ColNr::MAX);
    let mut col = pos.col;

    let mut did_change = false;
    if col + c_int::from(save_coladd != 0) < linelen {
        let mut negative = false;
        let mut was_positive = true;
        let mut blank_unsigned = false;

        if !visual {
            col = find_number_start(&text, pos.col, &fmt);
        } else {
            match visual_skip_to_number(&text, col, length, &fmt) {
                Some((c, l)) => {
                    col = c;
                    length = l;
                }
                // The selection holds no number at all.
                None => return finish_addsub(visual, false, save_cursor, save_coladd),
            }
            match minus_before(&text, col, pos.col, &fmt) {
                Minus::Absent => {}
                Minus::Negative => {
                    negative = true;
                    was_positive = false;
                }
                Minus::BlankUnsigned => blank_unsigned = true,
            }
        }

        let firstdigit = byte_at(&text, col.as_usize());
        let is_alpha = fmt.alpha && firstdigit.is_ascii_alphabetic();
        if !firstdigit.is_ascii_digit() && !is_alpha {
            beep_flush();
        } else {
            let (startpos, endpos) = if is_alpha {
                bump_alpha_char(firstdigit, op_type, prenum1, col)
            } else {
                let scan = Scan {
                    text: &text,
                    linelen: linelen.get(),
                    col,
                    firstdigit,
                    visual,
                    negative,
                    was_positive,
                    blank_unsigned,
                };
                replace_number(op_type, &mut length, prenum1, &fmt, scan)
            };
            did_change = true;

            if !cmdmod_has(CmdModFlags::LOCKMARKS) {
                Buf::current().b_op_start = startpos;
                Buf::current().b_op_end = endpos;
                if Buf::current().b_op_end.col > ColNr::ZERO {
                    Buf::current().b_op_end.col -= 1;
                }
            }
        }
    }

    finish_addsub(visual, did_change, save_cursor, save_coladd)
}

/// Put the cursor back where the caller expects it, and answer `did_change`.
fn finish_addsub(visual: bool, did_change: bool, save_cursor: Pos, save_coladd: i32) -> bool {
    if visual {
        Win::current().w_cursor = save_cursor;
    } else if did_change {
        Win::current().w_set_curswant = true;
    } else if virtual_active(Win::current()) {
        Win::current().w_cursor.coladd = save_coladd;
    }
    did_change
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
fn find_number_start(text: &[u8], start_col: ColNr, fmt: &NrFormats) -> ColNr {
    // Every column the walks below reach is one of `text`'s, the terminator
    // included -- which `byte_at` answers as NUL -- and the walks stop there.
    let byte = |c: ColNr| c_int::from(byte_at(text, c.as_usize()));
    // Step back one character, not one byte.
    let back = |c: ColNr| {
        let c = c - 1;
        c - ColNr::from_index(head_off(text, c.as_usize()))
    };
    // `0x`/`0b` at `col`, with a digit of that base after it.
    let prefixed_at = |c: ColNr, upper: u8, lower: u8, digit: fn(c_int) -> bool| {
        c > ColNr::ZERO
            && (byte(c) == c_int::from(upper) || byte(c) == c_int::from(lower))
            && byte(c - 1) == c_int::from(b'0')
            && head_off(text, (c - 1).as_usize()) == 0
            && digit(byte(c + 1))
    };

    let mut col = start_col;
    if fmt.bin {
        while col > ColNr::ZERO && ascii_isbdigit(byte(col)) {
            col = ColNr::new(back(col));
        }
    }
    if fmt.hex {
        while col > ColNr::ZERO && ascii_isxdigit(byte(col)) {
            col = ColNr::new(back(col));
        }
    }
    if fmt.bin && fmt.hex && !prefixed_at(col, b'X', b'x', ascii_isxdigit) {
        // Binary and hexadecimal overlap: rescan over decimal digits.
        col = Win::current().w_cursor.col;
        while col > ColNr::ZERO && ascii_isdigit(byte(col)) {
            col = ColNr::new(back(col));
        }
    }

    if (fmt.hex && prefixed_at(col, b'X', b'x', ascii_isxdigit))
        || (fmt.bin && prefixed_at(col, b'B', b'b', ascii_isbdigit))
    {
        // On the base letter of a `0x`/`0b` number: move onto the `0`.
        return ColNr::new(back(col));
    }

    // No prefix: search forwards for a digit, then back to its number's
    // first one.
    col = start_col;
    while byte(col) != NUL && !ascii_isdigit(byte(col)) && !(fmt.alpha && ascii_isalpha(byte(col)))
    {
        col += 1;
    }
    while col > ColNr::ZERO
        && ascii_isdigit(byte(col - 1))
        && !(fmt.alpha && ascii_isalpha(byte(col)))
    {
        col -= 1;
    }
    col
}

/// Inside Visual mode: skip forwards to the first number in the selection.
///
/// Answers the column it starts at and how much of the selection is left, or
/// `None` when the selection runs out first.
///
fn visual_skip_to_number(
    text: &[u8],
    mut col: ColNr,
    mut length: c_int,
    fmt: &NrFormats,
) -> Option<(ColNr, c_int)> {
    let byte = |c: ColNr| c_int::from(byte_at(text, c.as_usize()));
    while byte(col) != NUL
        && length > 0
        && !ascii_isdigit(byte(col))
        && !(fmt.alpha && ascii_isalpha(byte(col)))
    {
        let mb_len = len_as_int(cluster_len(&text[col.as_usize().min(text.len())..]));
        col += mb_len;
        length -= mb_len;
    }
    (length != 0).then_some((col, length))
}

/// Is the character in front of `col` a minus sign belonging to the number?
///
/// `min_col` is the first column the caller is willing to look before: the
/// selection's start in Visual mode, 0 outside it.
///
fn minus_before(text: &[u8], col: ColNr, min_col: ColNr, fmt: &NrFormats) -> Minus {
    // Each read below is guarded by the bound that keeps it inside the line.
    if !(col > min_col
        && byte_at(text, (col - 1).as_usize()) == b'-'
        && head_off(text, (col - 1).as_usize()) == 0
        && !fmt.unsigned)
    {
        return Minus::Absent;
    }
    if fmt.blank
        && col >= ColNr::new(2)
        && !ascii_iswhite(c_int::from(byte_at(text, (col - 2).as_usize())))
    {
        Minus::BlankUnsigned
    } else {
        Minus::Negative
    }
}

/// 'nrformats' `p`: step a single letter along the alphabet, clamped at `a`/`A`
/// and `z`/`Z`.
///
/// Answers the `'[`/`']` positions.
fn bump_alpha_char(firstdigit: u8, op_type: OpType, prenum1: i32, col: ColNr) -> (Pos, Pos) {
    // The letter's ordinal within its own case.
    let base = if firstdigit < b'a' { b'A' } else { b'a' };
    let ord = i32::from(firstdigit - base);
    // The C library's own locale table: `firstdigit` is an ASCII letter.
    let upper = ctype_bits(firstdigit) & IS_UPPER != 0;
    // A letter `ord` places from `base`, which the clamps keep in 0..=25.
    let letter = |ord: i32| base + u8::try_from(ord).expect("a letter's ordinal");
    let bumped = if op_type == OpType::NrSub {
        if ord < prenum1 {
            if upper { b'A' } else { b'a' }
        } else {
            letter(ord - prenum1)
        }
    } else if 26 - ord - 1 < prenum1 {
        if upper { b'Z' } else { b'z' }
    } else {
        letter(ord + prenum1)
    };

    Win::current().w_cursor.col = col;
    let startpos = Win::current().w_cursor;
    let _ = del_char(false);
    ins_char(Key::ascii(bumped));
    let endpos = Win::current().w_cursor;
    Win::current().w_cursor.col = col;
    (startpos, endpos)
}

/// What the scan for the number found, handed to [`replace_number`].
#[derive(Debug)]
struct Scan<'a> {
    /// The line the number is in, copied out of the buffer.
    text: &'a [u8],
    /// Its length in bytes.
    linelen: c_int,
    /// Column the number starts at.
    col: ColNr,
    /// First byte of the number, which decides whether leading zeros are kept.
    firstdigit: u8,
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
fn replace_number(
    op_type: OpType,
    length: &mut c_int,
    prenum1: i32,
    fmt: &NrFormats,
    scan: Scan,
) -> (Pos, Pos) {
    let Scan {
        text,
        linelen,
        mut col,
        firstdigit,
        visual,
        mut negative,
        was_positive,
        mut blank_unsigned,
    } = scan;

    if !visual {
        match minus_before(text, col, ColNr::ZERO, fmt) {
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
    if visual && !visual_mode().is_line() {
        maxlen = if Buf::current().b_visual.vi_curswant == ColNr::MAX {
            linelen - col.get()
        } else {
            *length
        };
    }

    let bases = Str2NrBases::BIN.when(fmt.bin)
        | Str2NrBases::OCT.when(fmt.oct)
        | Str2NrBases::HEX.when(fmt.hex);
    // `text` is a copy of the line, read before anything below changes the
    // buffer; its end ends the number, as the terminator did, and a bounded
    // scan sees nothing past its bound, as the pointer form's slice of
    // `maxlen` bytes did.
    let at = usize::try_from(col).unwrap_or(0).min(text.len());
    let end = match usize::try_from(maxlen) {
        Ok(bound @ 1..) => (at + bound).min(text.len()),
        _ => text.len(),
    };
    let parsed = str2nr_scan(&text[at..end], maxlen, bases, false, false);
    *length = c_int::try_from(parsed.len).unwrap_or(c_int::MAX);
    let overflow = parsed.overflow;
    // The base marker: 'x'/'X' hex, 'b'/'B' binary, '0' octal, 0 decimal.
    let pre = parsed.prefix;
    let mut n: UVarNumber = parsed.magnitude;

    // A leading `-` is not a sign for hex, octal or binary.
    if pre != 0 && negative {
        col += 1;
        *length -= 1;
        negative = false;
    }

    let subtract = (op_type == OpType::NrSub) ^ negative;
    (n, negative) = add_or_subtract(n, prenum1, subtract, negative, overflow, pre != 0);

    if (fmt.unsigned || blank_unsigned) && negative {
        // Stick at 0 going down and at 2^64 - 1 going up.
        n = if subtract { 0 } else { UVarNumber::MAX };
        negative = false;
    }

    if visual && !was_positive && !negative && col > ColNr::ZERO {
        // The `-` has to go.
        col -= 1;
        *length += 1;
    }

    // Delete the old number.
    Win::current().w_cursor.col = col;
    let startpos = Win::current().w_cursor;
    let mut todel = *length;
    let mut c = gchar_cursor();
    // The `-` is not part of the length: only the part after it keeps its
    // width.
    if c == c_int::from(b'-') {
        *length -= 1;
    }
    while todel > 0 {
        todel -= 1;
        // The C library's own locale table, indexed by a byte value.
        let class = u8::try_from(c).map_or(0, ctype_bits);
        if class & IS_ALPHA != 0 {
            HEX_UPPER.set(class & IS_UPPER != 0);
        }
        let _ = del_char(false);
        c = gchar_cursor();
    }

    let len = *length;
    render_number(n, pre, len, firstdigit, negative, visual, was_positive, fmt);

    let endpos = Win::current().w_cursor;
    if Win::current().w_cursor.col != ColNr::ZERO {
        Win::current().w_cursor.col -= 1;
    }
    (startpos, endpos)
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
    mut n: UVarNumber,
    prenum1: i32,
    subtract: bool,
    mut negative: bool,
    overflow: bool,
    prefixed: bool,
) -> (UVarNumber, bool) {
    let oldn = n;
    if !overflow {
        n = if subtract {
            n.wrapping_sub(i64::from(prenum1).cast_unsigned())
        } else {
            n.wrapping_add(i64::from(prenum1).cast_unsigned())
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
#[allow(clippy::too_many_arguments)]
fn render_number(
    n: UVarNumber,
    pre: u8,
    mut length: c_int,
    firstdigit: u8,
    negative: bool,
    visual: bool,
    was_positive: bool,
    fmt: &NrFormats,
) {
    // Sized as upstream does, before the decrements below: with many
    // leading zeros the prefix can be long.
    let mut buf = Vec::with_capacity(usize::try_from(length).unwrap_or(0) + NUMBUFLEN);
    if negative && (!visual || was_positive) {
        buf.push(b'-');
    }
    if pre != 0 {
        buf.push(b'0');
        length -= 1;
    }
    if pre == b'b' || pre == b'B' || pre == b'x' || pre == b'X' {
        buf.push(pre);
        length -= 1;
    }

    // The digits themselves.
    let digits = if pre == b'b' || pre == b'B' {
        format_binary(n)
    } else if pre == 0 {
        format!("{n}").into_bytes()
    } else if pre == b'0' {
        format!("{n:o}").into_bytes()
    } else if HEX_UPPER.get() {
        format!("{n:X}").into_bytes()
    } else {
        format!("{n:x}").into_bytes()
    };
    length -= len_as_int(digits.len());

    // Keep the total width by padding with zeros -- unless the result
    // would then look like an octal number.
    if firstdigit == b'0' && !(fmt.oct && pre == 0) {
        while length > 0 {
            length -= 1;
            buf.push(b'0');
        }
    }
    buf.extend_from_slice(&digits);
    ins_str(&buf);
}

/// `n` in binary, most significant one-bit first.
///
/// At most `NUMBUFLEN - 1` digits, which a 64-bit number never reaches.
fn format_binary(n: UVarNumber) -> Vec<u8> {
    // Skip the leading zeros.
    let mut bits = 8 * ::core::mem::size_of::<UVarNumber>();
    while bits > 0 && (n >> (bits - 1)) & 0x1 == 0 {
        bits -= 1;
    }

    let mut out = Vec::with_capacity(bits);
    while bits > 0 && out.len() < NUMBUFLEN - 1 {
        bits -= 1;
        out.push(if (n >> bits) & 0x1 != 0 { b'1' } else { b'0' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 'nrformats' with only the bases named, which is what keeps
    /// [`find_number_start`] out of its `bin`-and-`hex` rescan -- the one
    /// branch that reads the cursor, and so the one the library harness
    /// cannot reach.
    fn formats(hex: bool, bin: bool) -> NrFormats {
        NrFormats {
            hex,
            oct: false,
            bin,
            alpha: false,
            unsigned: false,
            blank: false,
        }
    }

    #[test]
    fn a_number_starts_where_its_digits_do() {
        let fmt = formats(true, false);
        // From inside the number, back to its first digit.
        assert_eq!(
            find_number_start(b"x = 123;", ColNr::new(5), &fmt),
            ColNr::new(4)
        );
        assert_eq!(
            find_number_start(b"x = 123;", ColNr::new(4), &fmt),
            ColNr::new(4)
        );
        // From before it, forwards to the first digit.
        assert_eq!(
            find_number_start(b"x = 123;", ColNr::ZERO, &fmt),
            ColNr::new(4)
        );
        // A `0x` prefix is part of the number.
        assert_eq!(
            find_number_start(b"n = 0x1f;", ColNr::new(7), &fmt),
            ColNr::new(4)
        );
        // Without `x` in 'nrformats' the letter is not a prefix and the
        // backwards scans do not run at all: the answer is the first digit
        // at or after the column, and the `f` of `1f` is not one.
        assert_eq!(
            find_number_start(b"n = 0x1f;", ColNr::new(6), &formats(false, false)),
            ColNr::new(6)
        );
        assert_eq!(
            find_number_start(b"n = 0x1f;", ColNr::new(7), &formats(false, false)),
            ColNr::new(9)
        );
    }

    #[test]
    fn a_binary_prefix_is_recognised_on_its_own() {
        let fmt = formats(false, true);
        assert_eq!(
            find_number_start(b"n = 0b101;", ColNr::new(8), &fmt),
            ColNr::new(4)
        );
    }

    #[test]
    fn the_selection_skips_forward_to_a_number() {
        let fmt = formats(true, false);
        // Six bytes of "ab 12": the digits start at 3.
        assert_eq!(
            visual_skip_to_number(b"ab 12", ColNr::ZERO, 5, &fmt),
            Some((ColNr::new(3), 2))
        );
        // The selection runs out first.
        assert_eq!(visual_skip_to_number(b"abcd 1", ColNr::ZERO, 4, &fmt), None);
        // A multibyte character is one step, not one byte.
        let line = "\u{4e00}9".as_bytes();
        assert_eq!(
            visual_skip_to_number(line, ColNr::ZERO, 4, &fmt),
            Some((ColNr::new(3), 1))
        );
    }

    #[test]
    fn a_minus_belongs_to_the_number_unless_the_option_says_not() {
        let plain = formats(true, false);
        assert!(matches!(
            minus_before(b"x -12", ColNr::new(3), ColNr::ZERO, &plain),
            Minus::Negative
        ));
        // No `-` there at all.
        assert!(matches!(
            minus_before(b"x 12", ColNr::new(2), ColNr::ZERO, &plain),
            Minus::Absent
        ));
        // `min_col` is how far back the caller allows the look.
        assert!(matches!(
            minus_before(b"x -12", ColNr::new(3), ColNr::new(3), &plain),
            Minus::Absent
        ));
        // 'nrformats' `u`: a `-` is never a sign.
        let mut unsigned = formats(true, false);
        unsigned.unsigned = true;
        assert!(matches!(
            minus_before(b"x -12", ColNr::new(3), ColNr::ZERO, &unsigned),
            Minus::Absent
        ));
        // 'nrformats' `k`: a `-` is only a sign after white space.
        let mut blank = formats(true, false);
        blank.blank = true;
        assert!(matches!(
            minus_before(b"x -12", ColNr::new(3), ColNr::ZERO, &blank),
            Minus::Negative
        ));
        assert!(matches!(
            minus_before(b"xx-12", ColNr::new(3), ColNr::ZERO, &blank),
            Minus::BlankUnsigned
        ));
    }
}
