//! The formatter itself.
//!
//! printf's whole output side.  The format is walked once; every run of
//! literal text is copied through, and every conversion is parsed into a
//! [`Conversion`], rendered into a scratch buffer (or pointed at in place),
//! and then padded out to its field width.
//!
//! Three things carry the shape:
//!
//! - [`Sink`] is the destination.  The return value is the length the result
//!   *would* have had, so counting continues after the buffer is full; that
//!   is the whole reason `produced` and the buffer's length are separate.
//! - [`Args`] is where a conversion's argument comes from -- an array of
//!   typed values, or the `TypVal` array Vimscript's `printf()` passes
//!   instead.  Both are indexed, so `%N$` only moves the index; the
//!   positional pre-pass (see [`super::spec`]) is there to reject a format
//!   that uses one position at two types.
//! - [`Body`] is where a rendered conversion ended up: in the scratch buffer,
//!   in the format string itself (`%%`), or in the argument (`%s`).

#![forbid(unsafe_code)]

use core::ffi::{CStr, c_int, c_uint};
use core::fmt::Write as _;
use std::borrow::Cow;

use super::float::render_float;
use super::spec::{
    MAX_ALLOWED_STRING_WIDTH, PosTypes, byte_at, format_overflow_error, get_unsigned_int,
    parse_fmt_types,
};
use super::{FmtArg, tv_float, tv_nr, tv_ptr, tv_str};
use crate::ascii::ascii_isdigit;
use crate::eval::typval::NumBuf;
use crate::mbyte::{cells_at, cluster_len};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::strings::raw::Values;
use crate::types::TypVal;

const E_TOO_MANY_ARGS: &CStr = c"E767: Too many arguments to printf()";

/// The scratch buffer one conversion is rendered into.
pub(super) const TMP: usize = 350;

// ---------------------------------------------------------------------
// The destination
// ---------------------------------------------------------------------

/// The output buffer, and the length the result would have had.
///
/// `produced` keeps counting past the end of `buf` -- that is the return
/// value `vim_snprintf` documents -- so it is not an index. `fits` records
/// whether the *last* write stayed inside the buffer; once it is false
/// nothing more is written, but `produced` still grows.
struct Sink<'o> {
    buf: &'o mut [u8],
    produced: usize,
    fits: bool,
}

impl<'o> Sink<'o> {
    fn new(buf: &'o mut [u8]) -> Self {
        let fits = !buf.is_empty();
        Sink {
            buf,
            produced: 0,
            fits,
        }
    }

    /// Room left in the buffer. Only meaningful while `fits` holds --
    /// `produced` runs past the buffer once the output is truncated.
    fn avail(&self) -> usize {
        self.buf.len() - self.produced
    }

    /// Account for `n` bytes of output, whether or not they were written.
    fn advance(&mut self, n: usize) {
        debug_assert!(n <= usize::MAX - self.produced);
        self.produced += n;
    }

    /// Append `src`, truncating at the end of the buffer.
    fn copy(&mut self, src: &[u8]) {
        if self.fits {
            let avail = self.avail();
            let n = src.len().min(avail);
            self.buf[self.produced..self.produced + n].copy_from_slice(&src[..n]);
            self.fits = src.len() < avail;
        }
        self.advance(src.len());
    }

    /// Append `n` copies of `byte`, truncating at the end of the buffer.
    fn fill(&mut self, byte: u8, n: usize) {
        if self.fits {
            let avail = self.avail();
            self.buf[self.produced..self.produced + n.min(avail)].fill(byte);
            self.fits = n < avail;
        }
        self.advance(n);
    }

    /// NUL-terminate, at the end of the output or of the buffer.
    fn terminate(&mut self) {
        if let Some(last) = self.buf.len().checked_sub(1) {
            self.buf[self.produced.min(last)] = 0;
        }
    }
}

// ---------------------------------------------------------------------
// The arguments
// ---------------------------------------------------------------------

/// Where a conversion's arguments come from.
pub(in crate::strings) enum Source<'a> {
    /// Values a Rust caller passed (the `vim_snprintf!` macros).
    Values(Values<'a>),
    /// Vimscript's `printf()` arguments, read with its type checks and its
    /// errors.
    Typvals(&'a [TypVal]),
}

/// The arguments of one `printf`, and the one-based index of the next.
pub(super) struct Args<'a> {
    src: Source<'a>,
    arg_idx: c_int,
}

impl<'a> Args<'a> {
    fn typvals(&self) -> Option<&'a [TypVal]> {
        match self.src {
            Source::Typvals(tvs) => Some(tvs),
            Source::Values(_) => None,
        }
    }

    /// The value argument `arg_idx` names, moving the index on.
    fn next_value(&mut self) -> Option<FmtArg<'a>> {
        let Source::Values(values) = &self.src else {
            return None;
        };
        let at = usize::try_from(self.arg_idx - 1).ok();
        self.arg_idx += 1;
        values.get(at?)
    }

    /// The next argument's 64 bits, as an integer conversion reads them;
    /// the caller narrows to the width its length modifier names, as the C
    /// narrowed what it read out of the `va_list`.
    fn next_bits(&mut self) -> u64 {
        if let Some(tvs) = self.typvals() {
            return tv_nr(tvs, &mut self.arg_idx) as u64;
        }
        match self.next_value() {
            Some(FmtArg::Int(n)) => n as u64,
            Some(FmtArg::UInt(n)) => n,
            Some(FmtArg::Str(text)) => text.as_ptr().addr() as u64,
            Some(FmtArg::Ptr(at)) => at.addr() as u64,
            Some(FmtArg::Float(_)) | None => 0,
        }
    }

    pub(super) fn next_float(&mut self) -> f64 {
        if let Some(tvs) = self.typvals() {
            return tv_float(tvs, &mut self.arg_idx);
        }
        match self.next_value() {
            Some(FmtArg::Float(f)) => f,
            Some(FmtArg::Int(n)) => n as f64,
            Some(FmtArg::UInt(n)) => n as f64,
            _ => 0.0,
        }
    }

    /// The next argument's address, for `%p`.
    fn next_address(&mut self) -> usize {
        if let Some(tvs) = self.typvals() {
            return tv_ptr(tvs, &mut self.arg_idx);
        }
        match self.next_value() {
            Some(FmtArg::Ptr(at)) => at.addr(),
            Some(FmtArg::Str(text)) => text.as_ptr().addr(),
            Some(FmtArg::Int(n)) => n as usize,
            Some(FmtArg::UInt(n)) => n as usize,
            Some(FmtArg::Float(_)) | None => 0,
        }
    }

    /// The next argument's text, `None` for a null one. `max` bounds how far
    /// a C string may be read (the precision of `%.Ns`); the answer may run
    /// past it otherwise. A Number is rendered into `numbuf`.
    fn next_string<'b>(
        &mut self,
        numbuf: &'b mut NumBuf,
        max: Option<usize>,
    ) -> Option<Cow<'b, [u8]>>
    where
        'a: 'b,
    {
        if let Some(tvs) = self.typvals() {
            return tv_str(tvs, &mut self.arg_idx, numbuf);
        }
        let arg = self.next_value()?;
        let Source::Values(values) = &self.src else {
            return None;
        };
        values.text(arg, max).map(Cow::Borrowed)
    }
}

// ---------------------------------------------------------------------
// One conversion
// ---------------------------------------------------------------------

/// A parsed `%` conversion: everything between the `%` and the end of the
/// specifier, plus the two padding counts the render step fills in.
pub(super) struct Conversion {
    pub(super) min_field_width: usize,
    pub(super) precision: usize,
    pub(super) precision_specified: bool,
    pub(super) zero_padding: bool,
    pub(super) justify_left: bool,
    pub(super) alternate_form: bool,
    pub(super) force_sign: bool,
    /// A positive value is prefixed with a space rather than a `+`. Set by
    /// the ` ` flag, cleared by `+`, which is why `%+ d` prints `+`.
    pub(super) space_for_positive: bool,
    /// `\0`, `h`, `l`, `L` (for `ll`) or `z`.
    pub(super) length_modifier: u8,
    pub(super) fmt_spec: u8,
    /// Zeros inserted between the sign/prefix and the digits.
    pub(super) zeros_to_pad: usize,
    /// How far into the rendered text those zeros go.
    pub(super) zero_insertion_ind: usize,
}

/// Where a rendered conversion ended up.
pub(super) enum Body<'b> {
    /// The first `n` bytes of the caller's scratch buffer.
    Tmp(usize),
    /// Bytes the caller does not own: the format's own, or a literal.
    At(&'b [u8]),
    /// The first `n` bytes of an argument's text.
    Text(Cow<'b, [u8]>, usize),
}

/// Read the number a `*` field width or precision refers to.
///
/// `digstart` is only used to quote the offending digits in `E1510`; the
/// value-array spelling clamps instead of raising, because an internal
/// `vim_snprintf` has no user to blame.
fn star_argument(args: &mut Args, fmt: &[u8], p: &mut usize, digstart: usize) -> Result<c_int, ()> {
    let typed = args.typvals().is_some();
    // `*N$` addresses the width argument positionally.
    if ascii_isdigit(c_int::from(byte_at(fmt, *p))) {
        args.arg_idx = get_unsigned_int(fmt, digstart, p, typed).ok_or(())? as c_int;
        *p += 1; // step over the '$'
    }
    let mut j = args.next_bits() as c_int;
    if j > MAX_ALLOWED_STRING_WIDTH {
        if typed {
            format_overflow_error(fmt, digstart);
            return Err(());
        }
        j = MAX_ALLOWED_STRING_WIDTH;
    }
    Ok(j)
}

/// Parse one conversion, leaving `*p` on its final character.
///
/// `*p` stands on the `%` of a conversion in `fmt`. Reads arguments as it
/// goes: a `*` width or precision consumes one before the conversion's own
/// argument.
fn parse_conversion(args: &mut Args, fmt: &[u8], p: &mut usize) -> Result<Conversion, ()> {
    let typed = args.typvals().is_some();
    let mut c = Conversion {
        min_field_width: 0,
        precision: 0,
        precision_specified: false,
        zero_padding: false,
        justify_left: false,
        alternate_form: false,
        force_sign: false,
        space_for_positive: true,
        length_modifier: 0,
        fmt_spec: 0,
        zeros_to_pad: 0,
        zero_insertion_ind: 0,
    };
    let at = |i: usize| byte_at(fmt, i);
    let digit = |i: usize| ascii_isdigit(c_int::from(byte_at(fmt, i)));

    *p += 1; // step over the '%'

    // A leading run of digits followed by '$' names the argument.
    let mut pos_arg = -1;
    let mut ptype = *p;
    while digit(ptype) {
        ptype += 1;
    }
    if at(ptype) == b'$' {
        let digstart = *p;
        pos_arg = get_unsigned_int(fmt, digstart, p, typed).ok_or(())? as c_int;
        *p += 1; // step over the '$'
    }

    loop {
        match at(*p) {
            b'0' => c.zero_padding = true,
            b'-' => c.justify_left = true,
            b'+' => {
                c.force_sign = true;
                c.space_for_positive = false;
            }
            b' ' => c.force_sign = true,
            b'#' => c.alternate_form = true,
            b'\'' => {} // accepted and ignored
            _ => break,
        }
        *p += 1;
    }

    // Field width. A negative `*` width means left-justified.
    if at(*p) == b'*' {
        let digstart = *p + 1;
        *p += 1;
        let j = star_argument(args, fmt, p, digstart)?;
        c.min_field_width = j.unsigned_abs() as usize;
        if j < 0 {
            c.justify_left = true;
        }
    } else if digit(*p) {
        let digstart = *p;
        c.min_field_width = get_unsigned_int(fmt, digstart, p, typed).ok_or(())? as usize;
    }

    // Precision. A negative `*` precision is as good as none.
    if at(*p) == b'.' {
        *p += 1;
        c.precision_specified = true;
        if digit(*p) {
            let digstart = *p;
            c.precision = get_unsigned_int(fmt, digstart, p, typed).ok_or(())? as usize;
        } else if at(*p) == b'*' {
            // Note the asymmetry with the width above: this `digstart`
            // includes the `*`, so `E1510` quotes it too.
            let digstart = *p;
            *p += 1;
            let j = star_argument(args, fmt, p, digstart)?;
            if j >= 0 {
                c.precision = j as usize;
            } else {
                c.precision_specified = false;
                c.precision = 0;
            }
        }
    }

    if matches!(at(*p), b'h' | b'l' | b'z') {
        c.length_modifier = at(*p);
        *p += 1;
        if c.length_modifier == b'l' && at(*p) == b'l' {
            c.length_modifier = b'L'; // ll
            *p += 1;
        }
    }

    // Synonyms, each implying a length modifier of its own.
    c.fmt_spec = at(*p);
    match c.fmt_spec {
        b'i' => c.fmt_spec = b'd',
        b'D' => {
            c.fmt_spec = b'd';
            c.length_modifier = b'l';
        }
        b'U' => {
            c.fmt_spec = b'u';
            c.length_modifier = b'l';
        }
        b'O' => {
            c.fmt_spec = b'o';
            c.length_modifier = b'l';
        }
        _ => {}
    }

    // Every Vimscript Number is 64-bit, so an integer conversion with
    // no length modifier still has to be read at the widest type.
    if typed && c.length_modifier == 0 && matches!(c.fmt_spec, b'd' | b'u' | b'o' | b'x' | b'X') {
        c.length_modifier = b'L';
    }

    if pos_arg != -1 {
        args.arg_idx = pos_arg;
    }
    Ok(c)
}

// ---------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------

/// `%%`, `%c`, `%s` and `%S`, the conversion's character at `fmt[p]`.
///
/// `%S` is the one conversion that measures in *display cells*: its
/// precision bounds the cell count, and the field width is then corrected
/// by the difference between bytes and cells so that padding still lines
/// up on screen.
fn render_string<'b, 'a: 'b>(
    c: &mut Conversion,
    args: &mut Args<'a>,
    fmt: &'b [u8],
    p: usize,
    tmp: &mut [u8; TMP],
    numbuf: &'b mut NumBuf,
) -> Body<'b> {
    match c.fmt_spec {
        b'%' => Body::At(&fmt[p..=p]),
        b'c' => {
            // The C points at a `uchar` local; the scratch buffer is
            // the same one byte.
            tmp[0] = args.next_bits() as u8;
            Body::Tmp(1)
        }
        // b's' | b'S'
        _ => {
            // Never look past the precision -- except that `%S` walks the
            // whole string for its cells. (The 2^31 clamp is upstream's,
            // for a `memchr` that disliked more.)
            let cap = c.precision.min(0x7fffffff);
            let max = (c.precision_specified && c.fmt_spec == b's').then_some(cap);
            let Some(text) = args.next_string(numbuf, max) else {
                return Body::At(b"[NULL]");
            };
            let mut str_arg_l = if !c.precision_specified {
                text.len()
            } else if c.precision == 0 {
                0
            } else {
                text.len().min(cap)
            };

            if c.fmt_spec == b'S' {
                let mut cells: usize = 0;
                let mut at = 0;
                while at < text.len() {
                    let rest = &text[at..];
                    let cell = cells_at(rest).cast_unsigned() as usize;
                    if c.precision_specified && cells + cell > c.precision {
                        break;
                    }
                    cells += cell;
                    at += cluster_len(rest);
                }
                str_arg_l = at;
                if c.min_field_width != 0 {
                    // Pad to a *cell* width: the field width is stated
                    // in cells and the padder counts bytes, so the
                    // difference is added on. It is *signed* -- a
                    // string can take more cells than bytes, and then
                    // the field has to shrink -- and it can ask for a
                    // width below zero, which is not a width at all.
                    //
                    // Upstream writes this in `size_t` and the
                    // subtraction underflows there: `printf('%3S',
                    // "\xe9\xe8\xfc")` asks for 3 - 9, gets ~2^64, and
                    // the editor exits with `E41: Out of memory!`
                    // (O-B15-2).
                    c.min_field_width = c
                        .min_field_width
                        .saturating_add_signed(str_arg_l as isize - cells as isize);
                }
            }
            Body::Text(text, str_arg_l)
        }
    }
}

/// Text rendered into the scratch buffer, from an offset on.
pub(super) struct TmpWriter<'t> {
    pub(super) buf: &'t mut [u8; TMP],
    pub(super) len: usize,
}

impl core::fmt::Write for TmpWriter<'_> {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let end = self.len + text.len();
        let room = self.buf.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        room.copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// `%d`, `%u`, `%b`, `%B`, `%o`, `%x`, `%X` and `%p`.
///
/// The argument is read at the width its length modifier names, rendered
/// into `tmp` as libc's `%ld`/`%lu`/`%lo`/`%lx`/`%p` would (or, for `%b`,
/// bit by bit), and then `zero_insertion_ind` is moved past whatever must
/// stay in front of the zeros -- a `-` sign, or an alternate-form `0x`
/// prefix.
fn render_integer(c: &mut Conversion, args: &mut Args, tmp: &mut [u8; TMP]) -> Body<'static> {
    // `arg_sign` is 0 for zero, 1 for positive, -1 for negative; an
    // unsigned value is never negative.
    let mut arg_sign = 0;
    let mut arg: i64 = 0;
    let mut uarg: u64 = 0;
    let mut address = 0usize;

    if c.fmt_spec == b'p' {
        address = args.next_address();
        if address != 0 {
            arg_sign = 1;
        }
    } else if matches!(c.fmt_spec, b'b' | b'B') {
        uarg = args.next_bits();
        arg_sign = c_int::from(uarg != 0);
    } else if c.fmt_spec == b'd' {
        let bits = args.next_bits();
        arg = match c.length_modifier {
            b'h' => i64::from(bits as c_int as i16),
            b'l' | b'L' => bits as i64,
            b'z' => bits as isize as i64,
            _ => i64::from(bits as c_int),
        };
        arg_sign = match arg {
            0 => 0,
            n if n > 0 => 1,
            _ => -1,
        };
    } else {
        let bits = args.next_bits();
        uarg = match c.length_modifier {
            b'h' => u64::from(bits as c_uint as u16),
            b'l' | b'L' => bits,
            b'z' => bits as usize as u64,
            _ => u64::from(bits as c_uint),
        };
        arg_sign = c_int::from(uarg != 0);
    }

    let mut str_arg_l: usize = 0;
    // A precision on an integer means "at least this many digits", so
    // the zero flag has nothing left to do.
    if c.precision_specified {
        c.zero_padding = false;
    }

    // Whatever has to precede the digits goes in first.
    if c.fmt_spec == b'd' {
        if c.force_sign && arg_sign >= 0 {
            tmp[str_arg_l] = if c.space_for_positive { b' ' } else { b'+' };
            str_arg_l += 1;
        }
    } else if c.alternate_form && arg_sign != 0 && matches!(c.fmt_spec, b'x' | b'X' | b'b' | b'B') {
        tmp[str_arg_l] = b'0';
        tmp[str_arg_l + 1] = c.fmt_spec;
        str_arg_l += 2;
    }

    c.zero_insertion_ind = str_arg_l;
    if !c.precision_specified {
        c.precision = 1;
    }

    // `%.0d` of zero prints nothing at all.
    if !(c.precision == 0 && arg_sign == 0) {
        let mut out = TmpWriter {
            buf: tmp,
            len: str_arg_l,
        };
        let written = match c.fmt_spec {
            // glibc's `%p`.
            b'p' if address == 0 => out.write_str("(nil)"),
            b'p' => write!(out, "{address:#x}"),
            b'd' => write!(out, "{arg}"),
            b'b' | b'B' => {
                // Binary has no libc conversion: skip the leading
                // zeros, then emit one character per remaining bit.
                let mut bits = u64::BITS;
                while bits > 0 && uarg >> (bits - 1) & 1 == 0 {
                    bits -= 1;
                }
                while bits > 0 {
                    bits -= 1;
                    out.buf[out.len] = if uarg >> bits & 1 != 0 { b'1' } else { b'0' };
                    out.len += 1;
                }
                Ok(())
            }
            b'o' => write!(out, "{uarg:o}"),
            b'x' => write!(out, "{uarg:x}"),
            b'X' => write!(out, "{uarg:X}"),
            _ => write!(out, "{uarg}"),
        };
        debug_assert!(written.is_ok() && out.len < TMP);
        str_arg_l = out.len;

        // Zeros go after the sign and after an `0x`/`0b` prefix.
        if c.zero_insertion_ind < str_arg_l && tmp[c.zero_insertion_ind] == b'-' {
            c.zero_insertion_ind += 1;
        }
        if c.zero_insertion_ind + 1 < str_arg_l
            && tmp[c.zero_insertion_ind] == b'0'
            && matches!(tmp[c.zero_insertion_ind + 1], b'x' | b'X' | b'b' | b'B')
        {
            c.zero_insertion_ind += 2;
        }
    }

    let num_of_digits = str_arg_l - c.zero_insertion_ind;
    // `%#o` guarantees a leading zero, which it buys with precision.
    if c.alternate_form
        && c.fmt_spec == b'o'
        && !(c.zero_insertion_ind < str_arg_l && tmp[c.zero_insertion_ind] == b'0')
        && (!c.precision_specified || c.precision < num_of_digits + 1)
    {
        c.precision = num_of_digits + 1;
    }
    if num_of_digits < c.precision {
        c.zeros_to_pad = c.precision - num_of_digits;
    }
    // With `%0`, the field width is made up of zeros rather than
    // spaces — so it is the *zero* count that grows.
    if !c.justify_left && c.zero_padding {
        let used = str_arg_l + c.zeros_to_pad;
        c.zeros_to_pad += c.min_field_width.saturating_sub(used);
    }

    Body::Tmp(str_arg_l)
}

// ---------------------------------------------------------------------
// The driver
// ---------------------------------------------------------------------

/// Write the padded rendering of one conversion.
///
/// Three pieces in order: the field-width padding when right-justified, the
/// zeros (which go *inside* the rendered text, after its sign or `0x`
/// prefix), the text itself, and the field-width padding when left-justified.
fn emit_conversion(sink: &mut Sink, c: &Conversion, body: &[u8]) {
    let len = body.len();
    let padding = || {
        debug_assert!(len <= usize::MAX - c.zeros_to_pad);
        c.min_field_width.saturating_sub(len + c.zeros_to_pad)
    };

    if !c.justify_left {
        sink.fill(if c.zero_padding { b'0' } else { b' ' }, padding());
    }

    // Without zeros to insert there is no split, so the whole body is
    // written in one go below.
    let split = if c.zeros_to_pad == 0 {
        0
    } else {
        if c.zero_insertion_ind > 0 {
            sink.copy(&body[..c.zero_insertion_ind]);
        }
        sink.fill(b'0', c.zeros_to_pad);
        c.zero_insertion_ind
    };

    if len > split {
        sink.copy(&body[split..]);
    }

    if c.justify_left {
        sink.fill(b' ', padding());
    }
}

/// The formatter: `fmt` (which holds no NUL) with the arguments `src` holds,
/// into `out`.
///
/// Returns the length the result *would* have had, excluding the NUL, so an
/// answer at or past `out.len()` means the output was truncated. A format the
/// positional pass rejects answers 0 and leaves `out` alone; one a conversion
/// rejects midway (an `E1510`) leaves what was written so far, unterminated.
pub(in crate::strings) fn format(out: &mut [u8], fmt: &[u8], src: Source<'_>) -> usize {
    let tvs = match src {
        Source::Typvals(tvs) => Some(tvs),
        Source::Values(_) => None,
    };
    let mut types = PosTypes::new();
    if parse_fmt_types(&mut types, fmt, tvs.is_some(), tvs.map(<[TypVal]>::len)).is_err() {
        return 0;
    }
    let num_posarg = types.len();

    let mut args = Args { src, arg_idx: 1 };
    let mut sink = Sink::new(out);
    let mut tmp = [0u8; TMP];
    let mut numbuf = NumBuf::new();
    let mut p = 0;

    while p < fmt.len() {
        if fmt[p] != b'%' {
            // A run of literal text, copied through in one step.
            let end = fmt[p + 1..]
                .iter()
                .position(|&b| b == b'%')
                .map_or(fmt.len(), |at| p + 1 + at);
            sink.copy(&fmt[p..end]);
            p = end;
            continue;
        }

        let Ok(mut c) = parse_conversion(&mut args, fmt, &mut p) else {
            return sink.produced;
        };

        let body = match c.fmt_spec {
            b'%' | b'c' | b's' | b'S' => {
                render_string(&mut c, &mut args, fmt, p, &mut tmp, &mut numbuf)
            }
            b'd' | b'u' | b'b' | b'B' | b'o' | b'x' | b'X' | b'p' => {
                render_integer(&mut c, &mut args, &mut tmp)
            }
            b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => render_float(&mut c, &mut args, &mut tmp),
            _ => {
                // Not a conversion at all: the character is copied
                // through, flags and width discarded.
                c.zero_padding = false;
                c.justify_left = true;
                c.min_field_width = 0;
                Body::At(&fmt[p.min(fmt.len())..(p + 1).min(fmt.len())])
            }
        };
        let text: &[u8] = match &body {
            Body::Tmp(len) => &tmp[..*len],
            Body::At(at) => at,
            Body::Text(text, len) => &text[..*len],
        };

        if p < fmt.len() {
            p += 1; // step over the conversion character
        }
        emit_conversion(&mut sink, &c, text);
    }

    sink.terminate();
    // `printf()` complains about arguments it was not asked for.
    let unused = if num_posarg != 0 {
        num_posarg
    } else {
        usize::try_from(args.arg_idx - 1).unwrap_or(usize::MAX)
    };
    if tvs.is_some_and(|tvs| unused < tvs.len()) {
        emsg(gettext(E_TOO_MANY_ARGS));
    }
    sink.produced
}
