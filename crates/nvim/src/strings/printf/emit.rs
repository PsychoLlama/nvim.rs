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
//!   or the `TypVal` array Vimscript's `printf()` passes instead.  Only the
//!   `va_list` needs positioning, which is why `%N$` costs a whole pre-pass
//!   (see [`super::spec`]).
//! - [`Body`] is where a rendered conversion ended up: in the scratch buffer,
//!   or at a pointer the caller already has (the format string itself for
//!   `%%`, the argument for `%s`).

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use core::ffi::{
    CStr, VaList, c_char, c_double, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void,
};
use core::ptr;

use super::float::render_float;
use super::spec::{
    MAX_ALLOWED_STRING_WIDTH, format_overflow_error, get_unsigned_int, parse_fmt_types, skip_to_arg,
};
use super::{TMP_LEN, tv_float, tv_nr, tv_ptr, tv_str};
use crate::ascii::ascii_isdigit;
use crate::mbyte::{cells_at, cluster_len};
use crate::memory::{xfree, xmemscan, xstrchrnul};
use crate::message::emsg;
use crate::os::cshim::{gettext, snprintf};
use crate::types::{TypVal, int16_t, intmax_t, ptrdiff_t, size_t, uint16_t, uintmax_t};

const E_TOO_MANY_ARGS: &CStr = c"E767: Too many arguments to printf()";

/// The scratch buffer one conversion is rendered into.
pub(super) const TMP: usize = TMP_LEN as usize;

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
    /// A sink over the `capacity` bytes at `buf`.
    ///
    /// # Safety
    ///
    /// `buf` must point at `capacity` writable bytes the caller owns,
    /// unaliased for the sink's lifetime. Nothing below checks the buffer --
    /// only the count it was given -- so this pair *is* the bound, and the
    /// safe writers below are safe only because it was established here.
    unsafe fn new(buf: *mut c_char, capacity: size_t) -> Self {
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
    ///
    /// # Safety
    ///
    /// `src` must point at `n` readable bytes.
    unsafe fn copy(&mut self, src: *const c_char, n: size_t) {
        if self.fits {
            let avail = self.avail();
            unsafe {
                self.buf
                    .add(self.produced)
                    .cast::<u8>()
                    .copy_from(src.cast(), n.min(avail))
            };
            self.fits = n < avail;
        }
        self.advance(n);
    }

    /// Append `n` copies of `byte`, truncating at the end of the buffer.
    fn fill(&mut self, byte: u8, n: size_t) {
        if self.fits {
            let avail = self.avail();
            // SAFETY: `fits` holds, so `produced <= capacity` and the write
            // stops at `avail` -- inside the window `Sink::new` was given.
            unsafe { ptr::write_bytes(self.buf.add(self.produced), byte, n.min(avail)) };
            self.fits = n < avail;
        }
        self.advance(n);
    }

    /// NUL-terminate, at the end of the output or of the buffer.
    fn terminate(&self) {
        if self.capacity > 0 {
            // SAFETY: the offset is clamped to the last byte of the window
            // `Sink::new` was given.
            unsafe { *self.buf.add(self.produced.min(self.capacity - 1)) = 0 };
        }
    }
}

// ---------------------------------------------------------------------
// The arguments
// ---------------------------------------------------------------------

/// Where a conversion's argument comes from.
///
/// A `tvs` slice means Vimscript's `printf()`, whose arguments can be
/// indexed; otherwise it is a C `va_list`, which can only be read forwards --
/// hence `position`, `ap_start` and the recorded `ap_types`.
pub(super) struct Args<'f> {
    tvs: Option<&'f [TypVal]>,
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
    /// The arguments of one `printf`, ready to be walked against `fmt`.
    ///
    /// # Safety
    ///
    /// `fmt` must be NUL-terminated, and `ap_types` the table
    /// `parse_fmt_types` filled in for exactly that format. With no `tvs`,
    /// `ap_start` must hold exactly the arguments `fmt` names at exactly
    /// those types -- a `va_list` carries no length, so reading one against
    /// the wrong format is what this constructor exists to make visible.
    unsafe fn new(
        tvs: Option<&'f [TypVal]>,
        ap_start: VaList<'f>,
        ap_types: *mut *const c_char,
        fmt: *const c_char,
    ) -> Self {
        Args {
            tvs,
            ap: ap_start.clone(),
            ap_start,
            ap_types,
            arg_idx: 1,
            arg_cur: 0,
            fmt,
        }
    }

    fn typvals(&self) -> Option<&'f [TypVal]> {
        self.tvs
    }

    /// Move the `va_list` onto argument `arg_idx`.
    ///
    /// # Safety
    ///
    /// `self` must be an `Args` built for the format it is being walked against:
    /// its `ap_types` table must be the one `parse_fmt_types` filled in for
    /// `fmt`, its `arg_cur` must say where `ap` really is, and reaching an
    /// argument behind the cursor re-reads every argument in between at the
    /// recorded types.
    unsafe fn position(&mut self) {
        // Bound in the order the call would have evaluated them: the clone
        // reads `ap_start` before the three field addresses are taken, and
        // those three address disjoint fields.
        let types = self.ap_types;
        let start = self.ap_start.clone();
        let ap = &raw mut self.ap;
        let idx = &raw mut self.arg_idx;
        let cur = &raw mut self.arg_cur;
        let fmt = self.fmt;
        unsafe { skip_to_arg(types, start, ap, idx, cur, fmt) };
    }

    /// `numbuf` is scratch a Number argument is rendered into; it must
    /// outlive the answer.
    ///
    /// # Safety
    ///
    /// `numbuf` must point at a NUL-terminated string, unaliased for the call.
    unsafe fn next_string(
        &mut self,
        tofree: &mut *mut c_char,
        numbuf: *mut c_char,
    ) -> *const c_char {
        if let Some(tvs) = self.typvals() {
            unsafe { tv_str(tvs, &mut self.arg_idx, tofree, numbuf) }
        } else {
            unsafe { self.position() };
            unsafe { self.ap.next_arg::<*const c_char>() }
        }
    }

    /// # Safety
    ///
    /// `self` must be an `Args` built for the format it is being walked against:
    /// its `ap_types` table must be the one `parse_fmt_types` filled in for
    /// `fmt`, its `arg_cur` must say where `ap` really is, and the argument it
    /// reaches must have been passed as a pointer.
    unsafe fn next_pointer(&mut self) -> *const c_void {
        if let Some(tvs) = self.typvals() {
            tv_ptr(tvs, &mut self.arg_idx)
        } else {
            unsafe { self.position() };
            unsafe { self.ap.next_arg::<*mut c_void>() as *const c_void }
        }
    }

    /// # Safety
    ///
    /// `self` must be an `Args` built for the format it is being walked against:
    /// its `ap_types` table must be the one `parse_fmt_types` filled in for
    /// `fmt`, its `arg_cur` must say where `ap` really is, and the argument it
    /// reaches must have been passed as a `double`.
    pub(super) unsafe fn next_float(&mut self) -> c_double {
        if let Some(tvs) = self.typvals() {
            tv_float(tvs, &mut self.arg_idx)
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
        if let Some(tvs) = $args.typvals() {
            tv_nr(tvs, &mut $args.arg_idx) as $ty
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
pub(super) struct Conversion {
    pub(super) min_field_width: size_t,
    pub(super) precision: size_t,
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
    pub(super) zeros_to_pad: size_t,
    /// How far into the rendered text those zeros go.
    pub(super) zero_insertion_ind: size_t,
}

/// Where a rendered conversion ended up.
pub(super) enum Body {
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
///
/// # Safety
///
/// `digstart` must point at a NUL-terminated string.
unsafe fn star_argument(
    args: &mut Args,
    p: &mut *const c_char,
    digstart: *const c_char,
) -> Result<c_int, ()> {
    // `*N$` addresses the width argument positionally.
    if ascii_isdigit(unsafe { **p as c_int }) {
        args.arg_idx =
            unsafe { get_unsigned_int(digstart, p, args.typvals().is_some()) }.ok_or(())? as c_int;
        *p = unsafe { p.add(1) }; // step over the '$'
    }
    let mut j = next_number!(args, c_int);
    if j > MAX_ALLOWED_STRING_WIDTH {
        if args.typvals().is_some() {
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
///
/// # Safety
///
/// `p` must point at a cursor standing on the `%` of a conversion in the NUL-
/// terminated format `args` was built for; it is advanced past what is
/// parsed. `args` carries the same obligation as `Args::next`: its argument
/// list and type table must match that format.
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
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
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
                unsafe { cstr::bytes_at(str_arg) }.len()
            } else if c.precision == 0 {
                0
            } else {
                // Never look past the precision. (The 2^31 clamp is
                // upstream's, for a `memchr` that disliked more.)
                let cap = c.precision.min(0x7fffffff);
                let end = unsafe { xmemscan(str_arg.cast(), 0, cap) }.cast::<c_char>();
                unsafe { end.offset_from(str_arg) as size_t }
            };

            if c.fmt_spec == b'S' {
                // SAFETY: the caller's argument is a NUL-terminated string.
                let bytes = unsafe { cstr::bytes_at(str_arg) };
                let mut cells: size_t = 0;
                let mut at = 0;
                while at < bytes.len() {
                    let rest = &bytes[at..];
                    let cell = cells_at(rest).cast_unsigned() as size_t;
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
///
/// # Safety
///
/// The argument `args` reaches next must have been passed at the type `c`'s
/// conversion and length modifier name -- a `va_list` is read blind, and
/// reading a `double` as an `int` is undefined behaviour.
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
                let out = unsafe { tmp.as_mut_ptr().add(str_arg_l) };
                let room = TMP - str_arg_l;
                str_arg_l += unsafe { snprintf(out, room, c"%p".as_ptr(), ptr_arg) as size_t };
            }
            b'd' => {
                let out = unsafe { tmp.as_mut_ptr().add(str_arg_l) };
                let room = TMP - str_arg_l;
                str_arg_l += unsafe { snprintf(out, room, c"%ld".as_ptr(), arg) as size_t };
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
                let out = unsafe { tmp.as_mut_ptr().add(str_arg_l) };
                let room = TMP - str_arg_l;
                str_arg_l +=
                    unsafe { snprintf(out, room, f.as_ptr().cast::<c_char>(), uarg) as size_t };
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

// ---------------------------------------------------------------------
// The driver
// ---------------------------------------------------------------------

/// Write the padded rendering of one conversion.
///
/// Three pieces in order: the field-width padding when right-justified, the
/// zeros (which go *inside* the rendered text, after its sign or `0x`
/// prefix), the text itself, and the field-width padding when left-justified.
///
/// # Safety
///
/// `body` must point at `len` readable bytes.
unsafe fn emit_conversion(sink: &mut Sink, c: &Conversion, body: *const c_char, len: size_t) {
    let padding = || {
        debug_assert!(len <= size_t::MAX - c.zeros_to_pad);
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
            unsafe { sink.copy(body, c.zero_insertion_ind) };
        }
        sink.fill(b'0', c.zeros_to_pad);
        c.zero_insertion_ind
    };

    if len > split {
        unsafe { sink.copy(body.add(split), len - split) };
    }

    if c.justify_left {
        sink.fill(b' ', padding());
    }
}

/// The formatter.
///
/// Returns the length the result *would* have had, excluding the NUL, so a
/// return value at or past `str_m` means the output was truncated.
///
/// # Safety
///
/// `fmt` must point at a NUL-terminated format. Either `tvs` points at an
/// array of initialized typvals long enough for the conversions `fmt` names,
/// or it is null and `ap_start` holds exactly those arguments at exactly
/// those types -- neither list carries its own length. `str` must point at
/// `str_m` writable bytes the caller owns, unaliased for the call.
pub unsafe fn vim_vsnprintf_typval<'f>(
    str: *mut c_char,
    str_m: size_t,
    fmt: *const c_char,
    ap_start: VaList<'f>,
    tvs: Option<&[TypVal]>,
) -> c_int {
    let mut ap_types = ptr::null_mut::<*const c_char>();
    let mut num_posarg = 0;
    if unsafe { parse_fmt_types(&mut ap_types, &mut num_posarg, fmt, tvs) }.is_err() {
        return 0;
    }

    // SAFETY: the caller's promise about `fmt`/`tvs`/`ap_start`, and
    // `ap_types` is the table `parse_fmt_types` just filled in for `fmt`.
    let mut args = unsafe { Args::new(tvs, ap_start, ap_types, fmt) };
    // SAFETY: the caller's promise -- `str_m` writable bytes at `str`.
    let mut sink = unsafe { Sink::new(str, str_m) };
    let mut p = if fmt.is_null() { c"".as_ptr() } else { fmt };

    'error: {
        while unsafe { *p } != 0 {
            if unsafe { *p as u8 } != b'%' {
                // A run of literal text, copied through in one step.
                let n = unsafe { xstrchrnul(p.add(1), b'%' as c_char).offset_from(p) as size_t };
                unsafe { sink.copy(p, n) };
                p = unsafe { p.add(n) };
                continue;
            }

            let Ok(mut c) = (unsafe { parse_conversion(&mut args, &mut p, tvs.is_some()) }) else {
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

        sink.terminate();
        // `printf()` complains about arguments it was not asked for.
        let unused = if num_posarg != 0 {
            num_posarg
        } else {
            args.arg_idx - 1
        };
        if tvs.is_some_and(|tvs| usize::try_from(unused).is_ok_and(|n| n < tvs.len())) {
            emsg(gettext(E_TOO_MANY_ARGS));
        }
    }

    unsafe { xfree(ap_types.cast()) };
    sink.produced as c_int
}
