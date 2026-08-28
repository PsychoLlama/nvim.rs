//! `vim_vsnprintf_typval()`: the formatter itself.
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
//!   is the whole reason `str_l` and `avail` are separate.
//! - [`Args`] is where a conversion's argument comes from -- a C `va_list`,
//!   or the `typval_T` array Vimscript's `printf()` passes instead.  Only the
//!   `va_list` needs positioning, which is why `%N$` costs a whole pre-pass
//!   (see [`super::spec`]).
//! - [`Body`] is where a rendered conversion ended up: in the scratch buffer,
//!   or at a pointer the caller already has (the format string itself for
//!   `%%`, the argument for `%s`).

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{
    CStr, VaList, c_char, c_double, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void,
};
use core::ptr;

use super::spec::{
    MAX_ALLOWED_STRING_WIDTH, format_overflow_error, get_unsigned_int, parse_fmt_types, skip_to_arg,
};
use super::{TMP_LEN, infinity_str, tv_float, tv_nr, tv_ptr, tv_str};
use crate::ascii::ascii_isdigit;
use crate::mbyte::{utf_ptr2cells, utfc_ptr2len};
use crate::memory::{xfree, xmemscan, xstrchrnul, xstrlcpy};
use crate::message::emsg;
use crate::os::cshim::{gettext, memmove, snprintf};
use crate::strings::vim_strchr;
use crate::types::{
    VAR_UNKNOWN, int16_t, intmax_t, ptrdiff_t, size_t, typval_T, uint16_t, uintmax_t,
};
use ::libc::strlen;

const E_TOO_MANY_ARGS: &CStr = c"E767: Too many arguments to printf()";

/// The scratch buffer one conversion is rendered into.
const TMP: usize = TMP_LEN as usize;

// ---------------------------------------------------------------------
// The destination
// ---------------------------------------------------------------------

/// The output buffer, and the length the result would have had.
///
/// `produced` keeps counting past the end of `buf` -- that is the return
/// value `vim_snprintf` documents -- so it is not an index. `fits` records
/// whether the *last* write stayed inside the buffer; once it is false
/// nothing more is written, but `produced` still grows.
struct Sink {
    buf: *mut c_char,
    capacity: size_t,
    produced: size_t,
    fits: bool,
}

impl Sink {
    fn new(buf: *mut c_char, capacity: size_t) -> Self {
        Sink {
            buf,
            capacity,
            produced: 0,
            fits: 0 < capacity,
        }
    }

    /// Room left in the buffer. Only meaningful while `fits` holds --
    /// `produced` runs past `capacity` once the output is truncated.
    fn avail(&self) -> size_t {
        self.capacity - self.produced
    }

    /// Account for `n` bytes of output, whether or not they were written.
    fn advance(&mut self, n: size_t) {
        debug_assert!(n <= size_t::MAX - self.produced);
        self.produced += n;
    }

    /// Append `n` bytes from `src`, truncating at the end of the buffer.
    unsafe fn copy(&mut self, src: *const c_char, n: size_t) {
        if self.fits {
            let avail = self.avail();
            unsafe { memmove(self.buf.add(self.produced).cast(), src.cast(), n.min(avail)) };
            self.fits = n < avail;
        }
        self.advance(n);
    }

    /// Append `n` copies of `byte`, truncating at the end of the buffer.
    unsafe fn fill(&mut self, byte: u8, n: size_t) {
        if self.fits {
            let avail = self.avail();
            unsafe { ptr::write_bytes(self.buf.add(self.produced), byte, n.min(avail)) };
            self.fits = n < avail;
        }
        self.advance(n);
    }

    /// NUL-terminate, at the end of the output or of the buffer.
    unsafe fn terminate(&self) {
        if self.capacity > 0 {
            unsafe { *self.buf.add(self.produced.min(self.capacity - 1)) = 0 };
        }
    }
}

// ---------------------------------------------------------------------
// The arguments
// ---------------------------------------------------------------------

/// Where a conversion's argument comes from.
///
/// `tvs` non-null means Vimscript's `printf()`, whose arguments are a
/// `VAR_UNKNOWN`-terminated `typval_T` array that can be indexed; otherwise
/// it is a C `va_list`, which can only be read forwards -- hence `position`,
/// `ap_start` and the recorded `ap_types`.
struct Args<'f> {
    tvs: *mut typval_T,
    ap: VaList<'f>,
    ap_start: VaList<'f>,
    ap_types: *mut *const c_char,
    /// One-based index of the argument to read next.
    arg_idx: c_int,
    /// Where the `va_list` actually is.
    arg_cur: c_int,
    fmt: *const c_char,
}

impl<'f> Args<'f> {
    fn reads_typvals(&self) -> bool {
        !self.tvs.is_null()
    }

    /// Move the `va_list` onto argument `arg_idx`.
    unsafe fn position(&mut self) {
        unsafe {
            skip_to_arg(
                self.ap_types,
                self.ap_start.clone(),
                &raw mut self.ap,
                &raw mut self.arg_idx,
                &raw mut self.arg_cur,
                self.fmt,
            )
        };
    }

    /// `numbuf` is scratch a Number argument is rendered into; it must
    /// outlive the answer.
    unsafe fn next_string(
        &mut self,
        tofree: &mut *mut c_char,
        numbuf: *mut c_char,
    ) -> *const c_char {
        if self.reads_typvals() {
            unsafe { tv_str(self.tvs, &mut self.arg_idx, tofree, numbuf) }
        } else {
            unsafe { self.position() };
            unsafe { self.ap.next_arg::<*const c_char>() }
        }
    }

    unsafe fn next_pointer(&mut self) -> *const c_void {
        if self.reads_typvals() {
            unsafe { tv_ptr(self.tvs, &mut self.arg_idx) }
        } else {
            unsafe { self.position() };
            unsafe { self.ap.next_arg::<*mut c_void>() as *const c_void }
        }
    }

    unsafe fn next_float(&mut self) -> c_double {
        if self.reads_typvals() {
            unsafe { tv_float(self.tvs, &mut self.arg_idx) }
        } else {
            unsafe { self.position() };
            unsafe { self.ap.next_arg::<c_double>() }
        }
    }
}

/// One argument, read at the C type `$ty`.
///
/// Only the `va_list` side has a type to read at; the typval side hands
/// back a Number and the C narrows it with a cast, which is what the `as`
/// here reproduces. A length modifier is therefore *unobservable* through
/// `printf()` for everything wider than the Number -- `%llu` and `%zu` read
/// the same 64-bit field.
macro_rules! next_number {
    ($args:expr, $ty:ty) => {
        if $args.reads_typvals() {
            unsafe { tv_nr($args.tvs, &mut $args.arg_idx) as $ty }
        } else {
            unsafe { $args.position() };
            unsafe { $args.ap.next_arg::<$ty>() }
        }
    };
}

// ---------------------------------------------------------------------
// One conversion
// ---------------------------------------------------------------------

/// A parsed `%` conversion: everything between the `%` and the end of the
/// specifier, plus the two padding counts the render step fills in.
struct Conversion {
    min_field_width: size_t,
    precision: size_t,
    precision_specified: bool,
    zero_padding: bool,
    justify_left: bool,
    alternate_form: bool,
    force_sign: bool,
    /// A positive value is prefixed with a space rather than a `+`. Set by
    /// the ` ` flag, cleared by `+`, which is why `%+ d` prints `+`.
    space_for_positive: bool,
    /// `\0`, `h`, `l`, `L` (for `ll`) or `z`.
    length_modifier: u8,
    fmt_spec: u8,
    /// Zeros inserted between the sign/prefix and the digits.
    zeros_to_pad: size_t,
    /// How far into the rendered text those zeros go.
    zero_insertion_ind: size_t,
}

/// Where a rendered conversion ended up.
enum Body {
    /// The first `n` bytes of the caller's scratch buffer.
    Tmp(size_t),
    /// `n` bytes at a pointer the caller does not own.
    At(*const c_char, size_t),
}

/// Read the number a `*` field width or precision refers to.
///
/// `digstart` is only used to quote the offending digits in `E1510`; the
/// `va_list` spelling clamps instead of raising, because an internal
/// `vim_snprintf` has no user to blame.
unsafe fn star_argument(
    args: &mut Args,
    p: &mut *const c_char,
    digstart: *const c_char,
) -> Result<c_int, ()> {
    // `*N$` addresses the width argument positionally.
    if ascii_isdigit(unsafe { **p as c_int }) {
        args.arg_idx =
            unsafe { get_unsigned_int(digstart, p, args.reads_typvals()) }.ok_or(())? as c_int;
        *p = unsafe { p.add(1) }; // step over the '$'
    }
    let mut j = next_number!(args, c_int);
    if j > MAX_ALLOWED_STRING_WIDTH {
        if args.reads_typvals() {
            unsafe { format_overflow_error(digstart) };
            return Err(());
        }
        j = MAX_ALLOWED_STRING_WIDTH;
    }
    Ok(j)
}

/// Parse one conversion, leaving `*p` on its final character.
///
/// Reads arguments as it goes: a `*` width or precision consumes one before
/// the conversion's own argument.
unsafe fn parse_conversion(
    args: &mut Args,
    p: &mut *const c_char,
    tvs_present: bool,
) -> Result<Conversion, ()> {
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

    *p = unsafe { p.add(1) }; // step over the '%'

    // A leading run of digits followed by '$' names the argument.
    let mut pos_arg = -1;
    let mut ptype = *p;
    while ascii_isdigit(unsafe { *ptype as c_int }) {
        ptype = unsafe { ptype.add(1) };
    }
    if unsafe { *ptype as u8 } == b'$' {
        let digstart = *p;
        pos_arg = unsafe { get_unsigned_int(digstart, p, tvs_present) }.ok_or(())? as c_int;
        *p = unsafe { p.add(1) }; // step over the '$'
    }

    loop {
        match unsafe { **p as u8 } {
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
        *p = unsafe { p.add(1) };
    }

    // Field width. A negative `*` width means left-justified.
    if unsafe { **p as u8 } == b'*' {
        let digstart = unsafe { p.add(1) };
        *p = unsafe { p.add(1) };
        let j = unsafe { star_argument(args, p, digstart) }?;
        if j >= 0 {
            c.min_field_width = j as size_t;
        } else {
            c.min_field_width = -j as size_t;
            c.justify_left = true;
        }
    } else if ascii_isdigit(unsafe { **p as c_int }) {
        let digstart = *p;
        c.min_field_width =
            unsafe { get_unsigned_int(digstart, p, tvs_present) }.ok_or(())? as size_t;
    }

    // Precision. A negative `*` precision is as good as none.
    if unsafe { **p as u8 } == b'.' {
        *p = unsafe { p.add(1) };
        c.precision_specified = true;
        if ascii_isdigit(unsafe { **p as c_int }) {
            let digstart = *p;
            c.precision =
                unsafe { get_unsigned_int(digstart, p, tvs_present) }.ok_or(())? as size_t;
        } else if unsafe { **p as u8 } == b'*' {
            // Note the asymmetry with the width above: this `digstart`
            // includes the `*`, so `E1510` quotes it too.
            let digstart = *p;
            *p = unsafe { p.add(1) };
            let j = unsafe { star_argument(args, p, digstart) }?;
            if j >= 0 {
                c.precision = j as size_t;
            } else {
                c.precision_specified = false;
                c.precision = 0;
            }
        }
    }

    if matches!(unsafe { **p as u8 }, b'h' | b'l' | b'z') {
        c.length_modifier = unsafe { **p as u8 };
        *p = unsafe { p.add(1) };
        if c.length_modifier == b'l' && unsafe { **p as u8 } == b'l' {
            c.length_modifier = b'L'; // ll
            *p = unsafe { p.add(1) };
        }
    }

    // Synonyms, each implying a length modifier of its own.
    c.fmt_spec = unsafe { **p as u8 };
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
    if tvs_present
        && c.length_modifier == 0
        && matches!(c.fmt_spec, b'd' | b'u' | b'o' | b'x' | b'X')
    {
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

/// `%%`, `%c`, `%s` and `%S`.
///
/// `%S` is the one conversion that measures in *display cells*: its
/// precision bounds the cell count, and the field width is then corrected
/// by the difference between bytes and cells so that padding still lines
/// up on screen.
unsafe fn render_string(
    c: &mut Conversion,
    args: &mut Args,
    p: *const c_char,
    tmp: &mut [c_char; TMP],
    tofree: &mut *mut c_char,
) -> Body {
    match c.fmt_spec {
        b'%' => Body::At(p, 1),
        b'c' => {
            // The C points at a `uchar` local; the scratch buffer is
            // the same one byte.
            tmp[0] = next_number!(args, c_int) as u8 as c_char;
            Body::Tmp(1)
        }
        // b's' | b'S'
        _ => {
            // `tmp` is untouched on this branch and outlives the
            // answer, so it doubles as the Number scratch.
            let str_arg = unsafe { args.next_string(tofree, tmp.as_mut_ptr()) };
            if str_arg.is_null() {
                return Body::At(c"[NULL]".as_ptr(), 6);
            }
            let mut str_arg_l = if !c.precision_specified {
                unsafe { strlen(str_arg) }
            } else if c.precision == 0 {
                0
            } else {
                // Never look past the precision. (The 2^31 clamp is
                // upstream's, for a `memchr` that disliked more.)
                unsafe {
                    xmemscan(str_arg.cast(), 0, c.precision.min(0x7fffffff))
                        .cast::<c_char>()
                        .offset_from(str_arg) as size_t
                }
            };

            if c.fmt_spec == b'S' {
                let mut cells: size_t = 0;
                let mut end = str_arg;
                while unsafe { *end } != 0 {
                    let cell = unsafe { utf_ptr2cells(end) as size_t };
                    if c.precision_specified && cells + cell > c.precision {
                        break;
                    }
                    cells += cell;
                    end = unsafe { end.offset(utfc_ptr2len(end) as isize) };
                }
                str_arg_l = unsafe { end.offset_from(str_arg) as size_t };
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
            Body::At(str_arg, str_arg_l)
        }
    }
}

/// `%d`, `%u`, `%b`, `%B`, `%o`, `%x`, `%X` and `%p`.
///
/// The argument is read at the width its length modifier names, rendered
/// into `tmp` through libc's `snprintf` (or, for `%b`, bit by bit), and
/// then `zero_insertion_ind` is moved past whatever must stay in front of
/// the zeros -- a `-` sign, or an alternate-form `0x` prefix.
unsafe fn render_integer(c: &mut Conversion, args: &mut Args, tmp: &mut [c_char; TMP]) -> Body {
    // `arg_sign` is 0 for zero, 1 for positive, -1 for negative; an
    // unsigned value is never negative.
    let mut arg_sign = 0;
    let mut arg: intmax_t = 0;
    let mut uarg: uintmax_t = 0;
    let mut ptr_arg = ptr::null::<c_void>();

    if c.fmt_spec == b'p' {
        ptr_arg = unsafe { args.next_pointer() };
        if !ptr_arg.is_null() {
            arg_sign = 1;
        }
    } else if matches!(c.fmt_spec, b'b' | b'B') {
        uarg = next_number!(args, c_ulonglong) as uintmax_t;
        arg_sign = c_int::from(uarg != 0);
    } else if c.fmt_spec == b'd' {
        arg = match c.length_modifier {
            b'h' => next_number!(args, c_int) as int16_t as intmax_t,
            b'l' => next_number!(args, c_long) as intmax_t,
            b'L' => next_number!(args, c_longlong) as intmax_t,
            b'z' => next_number!(args, ptrdiff_t) as intmax_t,
            _ => next_number!(args, c_int) as intmax_t,
        };
        arg_sign = match arg {
            0 => 0,
            n if n > 0 => 1,
            _ => -1,
        };
    } else {
        uarg = match c.length_modifier {
            b'h' => next_number!(args, c_uint) as uint16_t as uintmax_t,
            b'l' => next_number!(args, c_ulong) as uintmax_t,
            b'L' => next_number!(args, c_ulonglong) as uintmax_t,
            b'z' => next_number!(args, size_t) as uintmax_t,
            _ => next_number!(args, c_uint) as uintmax_t,
        };
        arg_sign = c_int::from(uarg != 0);
    }

    let mut str_arg_l: size_t = 0;
    // A precision on an integer means "at least this many digits", so
    // the zero flag has nothing left to do.
    if c.precision_specified {
        c.zero_padding = false;
    }

    // Whatever has to precede the digits goes in first.
    if c.fmt_spec == b'd' {
        if c.force_sign && arg_sign >= 0 {
            tmp[str_arg_l] = if c.space_for_positive { b' ' } else { b'+' } as c_char;
            str_arg_l += 1;
        }
    } else if c.alternate_form && arg_sign != 0 && matches!(c.fmt_spec, b'x' | b'X' | b'b' | b'B') {
        tmp[str_arg_l] = b'0' as c_char;
        tmp[str_arg_l + 1] = c.fmt_spec as c_char;
        str_arg_l += 2;
    }

    c.zero_insertion_ind = str_arg_l;
    if !c.precision_specified {
        c.precision = 1;
    }

    // `%.0d` of zero prints nothing at all.
    if !(c.precision == 0 && arg_sign == 0) {
        match c.fmt_spec {
            b'p' => {
                str_arg_l += unsafe {
                    snprintf(
                        tmp.as_mut_ptr().add(str_arg_l),
                        TMP - str_arg_l,
                        c"%p".as_ptr(),
                        ptr_arg,
                    ) as size_t
                };
            }
            b'd' => {
                str_arg_l += unsafe {
                    snprintf(
                        tmp.as_mut_ptr().add(str_arg_l),
                        TMP - str_arg_l,
                        c"%ld".as_ptr(),
                        arg,
                    ) as size_t
                };
            }
            b'b' | b'B' => {
                // Binary has no libc conversion: skip the leading
                // zeros, then emit one character per remaining bit.
                let mut bits = uintmax_t::BITS as usize;
                while bits > 0 && uarg >> (bits - 1) & 1 == 0 {
                    bits -= 1;
                }
                while bits > 0 {
                    bits -= 1;
                    tmp[str_arg_l] = if uarg >> bits & 1 != 0 { b'1' } else { b'0' } as c_char;
                    str_arg_l += 1;
                }
            }
            _ => {
                // `PRIuMAX` is "lu", so the conversion character is the
                // last byte and any of u/o/x/X can be dropped in.
                let mut f = *b"%lu\0";
                f[2] = c.fmt_spec;
                str_arg_l += unsafe {
                    snprintf(
                        tmp.as_mut_ptr().add(str_arg_l),
                        TMP - str_arg_l,
                        f.as_ptr().cast::<c_char>(),
                        uarg,
                    ) as size_t
                };
            }
        }
        debug_assert!(str_arg_l < TMP);

        // Zeros go after the sign and after an `0x`/`0b` prefix.
        if c.zero_insertion_ind < str_arg_l && tmp[c.zero_insertion_ind] as u8 == b'-' {
            c.zero_insertion_ind += 1;
        }
        if c.zero_insertion_ind + 1 < str_arg_l
            && tmp[c.zero_insertion_ind] as u8 == b'0'
            && matches!(
                tmp[c.zero_insertion_ind + 1] as u8,
                b'x' | b'X' | b'b' | b'B'
            )
        {
            c.zero_insertion_ind += 2;
        }
    }

    let num_of_digits = str_arg_l - c.zero_insertion_ind;
    // `%#o` guarantees a leading zero, which it buys with precision.
    if c.alternate_form
        && c.fmt_spec == b'o'
        && !(c.zero_insertion_ind < str_arg_l && tmp[c.zero_insertion_ind] as u8 == b'0')
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
        let n = c.min_field_width as ptrdiff_t - (str_arg_l + c.zeros_to_pad) as ptrdiff_t;
        if n > 0 {
            c.zeros_to_pad += n as size_t;
        }
    }

    Body::Tmp(str_arg_l)
}

/// `%f`, `%F`, `%e`, `%E`, `%g` and `%G`.
///
/// Everything but infinity and NaN is handed to libc's `snprintf` with a
/// format built here, because the exact digits are the platform's business.
/// `%g` is not passed through: it is resolved to `%f` or `%e` first, and
/// the trailing zeros it would have dropped are removed afterwards.
unsafe fn render_float(c: &mut Conversion, args: &mut Args, tmp: &mut [c_char; TMP]) -> Body {
    let f = unsafe { args.next_float() };
    // Not `f.abs()`: the C tests `f < 0`, so -0.0 stays -0.0.
    let abs_f = if f < 0.0 { -f } else { f };
    let mut remove_trailing_zeroes = false;

    if matches!(c.fmt_spec, b'g' | b'G') {
        // The range in which `%g` chooses fixed notation.
        c.fmt_spec = if (0.001..10000000.0).contains(&abs_f) || abs_f == 0.0 {
            if c.fmt_spec.is_ascii_uppercase() {
                b'F'
            } else {
                b'f'
            }
        } else if c.fmt_spec == b'g' {
            b'e'
        } else {
            b'E'
        };
        remove_trailing_zeroes = true;
    }

    // A fixed-notation value this large would not fit the scratch
    // buffer, so it prints as infinity too.
    if f.is_infinite() || (matches!(c.fmt_spec, b'f' | b'F') && abs_f > 1.0e307) {
        unsafe {
            xstrlcpy(
                tmp.as_mut_ptr(),
                infinity_str(
                    f > 0.0,
                    c.fmt_spec as c_char,
                    c.force_sign,
                    c.space_for_positive,
                )
                .as_ptr(),
                TMP,
            )
        };
        c.zero_padding = false;
        return Body::Tmp(unsafe { strlen(tmp.as_ptr()) });
    }
    if f.is_nan() {
        let nan = if c.fmt_spec.is_ascii_uppercase() {
            c"NAN"
        } else {
            c"nan"
        };
        unsafe { memmove(tmp.as_mut_ptr().cast(), nan.as_ptr().cast(), 4) };
        c.zero_padding = false;
        return Body::Tmp(3);
    }

    // Build the format libc gets: '%', an optional sign flag, an
    // optional precision, and the conversion.
    let mut format = [0 as c_char; 40];
    format[0] = b'%' as c_char;
    let mut l: size_t = 1;
    if c.force_sign {
        format[l] = if c.space_for_positive { b' ' } else { b'+' } as c_char;
        l += 1;
    }
    if c.precision_specified {
        // Bound the precision so the result still fits `tmp`: a fixed
        // conversion also spends digits on the integer part.
        let mut max_prec = (TMP_LEN - 10) as size_t;
        if matches!(c.fmt_spec, b'f' | b'F') && abs_f > 1.0 {
            max_prec -= abs_f.log10() as size_t;
        }
        c.precision = c.precision.min(max_prec);
        l += unsafe {
            snprintf(
                format.as_mut_ptr().add(l),
                format.len() - l,
                c".%d".as_ptr(),
                c.precision as c_int,
            ) as size_t
        };
    }
    debug_assert!(l + 1 < format.len());
    // libc has no `%F`; it prints the same digits as `%f`.
    format[l] = if c.fmt_spec == b'F' { b'f' } else { c.fmt_spec } as c_char;
    format[l + 1] = 0;

    let mut str_arg_l = unsafe { snprintf(tmp.as_mut_ptr(), TMP, format.as_ptr(), f) as size_t };
    debug_assert!(str_arg_l < TMP);

    if remove_trailing_zeroes {
        str_arg_l = unsafe { trim_float(c, tmp, str_arg_l) };
    } else {
        str_arg_l = unsafe { trim_exponent_width(c, tmp, str_arg_l) };
    }

    // A zero-padded signed value keeps its sign in front of the zeros.
    if c.zero_padding && c.min_field_width > str_arg_l && (tmp[0] as u8 == b'-' || c.force_sign) {
        c.zeros_to_pad = c.min_field_width - str_arg_l;
        c.zero_insertion_ind = 1;
    }
    Body::Tmp(str_arg_l)
}

/// Delete one byte at `at`, terminator included, and report the new length.
unsafe fn delete_byte(at: *mut c_char, len: size_t) -> size_t {
    unsafe { memmove(at.cast(), at.add(1).cast(), strlen(at.add(1)) + 1) };
    len - 1
}

/// `%g`'s trailing-zero removal.
///
/// In fixed notation the zeros are at the end; in exponential notation they
/// are in front of the exponent, and the exponent itself also loses its `+`
/// and its own leading zeros first.
unsafe fn trim_float(c: &Conversion, tmp: &mut [c_char; TMP], mut len: size_t) -> size_t {
    let mut tp;
    if matches!(c.fmt_spec, b'f' | b'F') {
        tp = unsafe { tmp.as_mut_ptr().add(len).sub(1) };
    } else {
        // `as_mut_ptr`, not `as_ptr`: `delete_byte` writes through what
        // this hands back, and a pointer derived from a *shared* borrow
        // of `tmp` only grants read permission (Stacked Borrows).
        tp = unsafe {
            vim_strchr(
                tmp.as_mut_ptr().cast_const(),
                if c.fmt_spec == b'e' { b'e' } else { b'E' } as c_int,
            )
        };
        if tp.is_null() {
            return len;
        }
        if unsafe { *tp.add(1) as u8 } == b'+' {
            len = unsafe { delete_byte(tp.add(1), len) };
        }
        // Leading zeros of the exponent, past its sign.
        let mut i = if unsafe { *tp.add(1) as u8 } == b'-' {
            2
        } else {
            1
        };
        while unsafe { *tp.add(i) as u8 } == b'0' {
            len = unsafe { delete_byte(tp.add(i), len) };
        }
        tp = unsafe { tp.sub(1) };
    }

    // An explicit precision asked for those zeros; keep them.
    if !c.precision_specified {
        // Never past `tmp[2]`, so `0.0` keeps a digit either side of
        // the point.
        while tp > unsafe { tmp.as_mut_ptr().add(2) }
            && unsafe { *tp as u8 } == b'0'
            && unsafe { *tp.sub(1) as u8 } != b'.'
        {
            len = unsafe { delete_byte(tp, len) };
            tp = unsafe { tp.sub(1) };
        }
    }
    len
}

/// Normalise an exponent that libc padded to three digits down to two.
unsafe fn trim_exponent_width(c: &Conversion, tmp: &mut [c_char; TMP], len: size_t) -> size_t {
    // Only the conversion's own case is looked for, so `%f` -- which
    // has no exponent -- never matches.
    let tp = unsafe {
        vim_strchr(
            tmp.as_ptr(),
            if c.fmt_spec == b'e' { b'e' } else { b'E' } as c_int,
        )
    };
    if !tp.is_null()
        && matches!(unsafe { *tp.add(1) as u8 }, b'+' | b'-')
        && unsafe { *tp.add(2) as u8 } == b'0'
        && ascii_isdigit(unsafe { *tp.add(3) as c_int })
        && ascii_isdigit(unsafe { *tp.add(4) as c_int })
    {
        return unsafe { delete_byte(tp.add(2), len) };
    }
    len
}

// ---------------------------------------------------------------------
// The driver
// ---------------------------------------------------------------------

/// Write the padded rendering of one conversion.
///
/// Three pieces in order: the field-width padding when right-justified, the
/// zeros (which go *inside* the rendered text, after its sign or `0x`
/// prefix), the text itself, and the field-width padding when left-justified.
unsafe fn emit_conversion(sink: &mut Sink, c: &Conversion, body: *const c_char, len: size_t) {
    let padding = || {
        debug_assert!(len <= size_t::MAX - c.zeros_to_pad);
        c.min_field_width.saturating_sub(len + c.zeros_to_pad)
    };

    if !c.justify_left {
        unsafe { sink.fill(if c.zero_padding { b'0' } else { b' ' }, padding()) };
    }

    // Without zeros to insert there is no split, so the whole body is
    // written in one go below.
    let split = if c.zeros_to_pad == 0 {
        0
    } else {
        if c.zero_insertion_ind > 0 {
            unsafe { sink.copy(body, c.zero_insertion_ind) };
        }
        unsafe { sink.fill(b'0', c.zeros_to_pad) };
        c.zero_insertion_ind
    };

    if len > split {
        unsafe { sink.copy(body.add(split), len - split) };
    }

    if c.justify_left {
        unsafe { sink.fill(b' ', padding()) };
    }
}

/// The formatter.
///
/// Returns the length the result *would* have had, excluding the NUL, so a
/// return value at or past `str_m` means the output was truncated.
pub unsafe fn vim_vsnprintf_typval<'f>(
    str: *mut c_char,
    str_m: size_t,
    fmt: *const c_char,
    ap_start: VaList<'f>,
    tvs: *mut typval_T,
) -> c_int {
    let mut ap_types = ptr::null_mut::<*const c_char>();
    let mut num_posarg = 0;
    if unsafe { parse_fmt_types(&mut ap_types, &mut num_posarg, fmt, tvs) }.is_err() {
        return 0;
    }

    let mut args = Args {
        tvs,
        ap: ap_start.clone(),
        ap_start,
        ap_types,
        arg_idx: 1,
        arg_cur: 0,
        fmt,
    };
    let mut sink = Sink::new(str, str_m);
    let mut p = if fmt.is_null() { c"".as_ptr() } else { fmt };
    let tvs_present = !tvs.is_null();

    'error: {
        while unsafe { *p } != 0 {
            if unsafe { *p as u8 } != b'%' {
                // A run of literal text, copied through in one step.
                let n = unsafe { xstrchrnul(p.add(1), b'%' as c_char).offset_from(p) as size_t };
                unsafe { sink.copy(p, n) };
                p = unsafe { p.add(n) };
                continue;
            }

            let Ok(mut c) = (unsafe { parse_conversion(&mut args, &mut p, tvs_present) }) else {
                break 'error;
            };
            let mut tmp = [0 as c_char; TMP];
            let mut tofree = ptr::null_mut::<c_char>();

            let body = match c.fmt_spec {
                b'%' | b'c' | b's' | b'S' => unsafe {
                    render_string(&mut c, &mut args, p, &mut tmp, &mut tofree)
                },
                b'd' | b'u' | b'b' | b'B' | b'o' | b'x' | b'X' | b'p' => unsafe {
                    render_integer(&mut c, &mut args, &mut tmp)
                },
                b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => unsafe {
                    render_float(&mut c, &mut args, &mut tmp)
                },
                _ => {
                    // Not a conversion at all: the character is copied
                    // through, flags and width discarded.
                    c.zero_padding = false;
                    c.justify_left = true;
                    c.min_field_width = 0;
                    Body::At(p, size_t::from(unsafe { *p } != 0))
                }
            };
            let (body, len) = match body {
                Body::Tmp(len) => (tmp.as_ptr(), len),
                Body::At(at, len) => (at, len),
            };

            if unsafe { *p } != 0 {
                p = unsafe { p.add(1) }; // step over the conversion character
            }
            unsafe { emit_conversion(&mut sink, &c, body, len) };
            unsafe { xfree(tofree.cast()) };
        }

        unsafe { sink.terminate() };
        // `printf()` complains about arguments it was not asked for.
        let unused = if num_posarg != 0 {
            num_posarg
        } else {
            args.arg_idx - 1
        };
        if tvs_present && unsafe { (*tvs.offset(unused as isize)).v_type } != VAR_UNKNOWN {
            unsafe { emsg(gettext(E_TOO_MANY_ARGS.as_ptr())) };
        }
    }

    unsafe { xfree(ap_types.cast()) };
    sink.produced as c_int
}
