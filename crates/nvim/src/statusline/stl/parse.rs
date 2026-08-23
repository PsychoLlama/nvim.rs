//! The format string, read left to right.
//!
//! Everything here is a pure function of the bytes: the `%-0<min>.<max>`
//! width prefix, the alphabet of item letters, and the `vim_snprintf`
//! template a number item prints through. The stage that evaluates an item
//! is [`super::item`]; this one only decides what the format *said*.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use crate::cstr;

use core::ffi::{CStr, c_int};

use super::{NumberBase, kNumBaseHexadecimal};

/// Widest an item may ask to be: upstream bounds both the minimum width and
/// the maximum width at this.
pub(super) const MAX_ITEM_WIDTH: c_int = 50;

/// The width an item gets when it does not ask for one.
pub(super) const DEFAULT_MAXWID: c_int = 9999;

/// Every letter the format language has an item for.
///
/// Upstream's `STL_ALL`, which lists `T`, `X` and `@` twice; the duplicates
/// are harmless and are kept so the two spellings stay comparable.
const STL_ALL: &[u8] = b"fFtcvVlLnkoObBrRhHyYwWmMqpPaNSCs{=<*#$TX@TX@";

/// The width and alignment prefix of one `%` item.
#[derive(Clone, Copy)]
pub(super) struct Spec {
    /// The minimum width, negative when the item is left-aligned. Some item
    /// letters read it before it is clamped and signed, as their argument.
    pub minwid: c_int,
    /// The maximum width, past which the item is truncated.
    pub maxwid: c_int,
    /// Whether a number item is padded with zeros rather than blanks.
    pub zeropad: bool,
    /// Whether the item is left-aligned, which is what makes [`Self::minwid`]
    /// negative once it is clamped.
    pub left_align: bool,
}

impl Spec {
    /// Read `%0`, `%-` and the first digit group, which is as far as the
    /// item letters that overload the minimum width need.
    pub(super) fn read(fmt: &[u8], p: &mut usize) -> Self {
        // Numbers are left-padded with zeros.
        let zeropad = fmt[*p] == b'0';
        if zeropad {
            *p += 1;
        }
        // The item is left-aligned, which is tracked as a negative width.
        let left_align = *p < fmt.len() && fmt[*p] == b'-';
        if left_align {
            *p += 1;
        }
        let minwid = if *p < fmt.len() && fmt[*p].is_ascii_digit() {
            digits(fmt, p, 0)
        } else {
            0
        };
        Spec {
            minwid,
            maxwid: DEFAULT_MAXWID,
            zeropad,
            left_align,
        }
    }

    /// Read the `.<maxwid>` half, then bound the minimum width and give it
    /// the sign that says which way the item is aligned.
    pub(super) fn finish(&mut self, fmt: &[u8], p: &mut usize) {
        if *p < fmt.len() && fmt[*p] == b'.' {
            *p += 1;
            if *p < fmt.len() && fmt[*p].is_ascii_digit() {
                self.maxwid = digits(fmt, p, MAX_ITEM_WIDTH);
            }
        }
        self.minwid = self.minwid.min(MAX_ITEM_WIDTH) * if self.left_align { -1 } else { 1 };
    }
}

/// Read a decimal number, answering `def` when it does not fit an `int`.
///
/// This is `getdigits_int(&fmt_p, false, def)` over a slice: `strtoimax`
/// consumes the digits either way, and a value it or the narrowing cannot
/// represent comes back as the default. That is why a twenty-digit width is
/// not a very wide item but no width at all.
pub(super) fn digits(fmt: &[u8], p: &mut usize, def: c_int) -> c_int {
    let start = *p;
    while *p < fmt.len() && fmt[*p].is_ascii_digit() {
        *p += 1;
    }
    let mut value: i64 = 0;
    for &byte in &fmt[start..*p] {
        value = value
            .saturating_mul(10)
            .saturating_add(i64::from(byte - b'0'));
        if value > i64::from(c_int::MAX) {
            return def;
        }
    }
    value as c_int
}

/// Whether `byte` names an item.
///
/// A NUL never does, which is what stops the walk at the end of a format
/// ending in something like `%0`.
pub(super) fn is_item_letter(byte: u8) -> bool {
    byte != 0 && STL_ALL.contains(&byte)
}

/// How a number item prints: the `vim_snprintf` template, the value, and the
/// width or exponent it takes as its argument.
pub(super) struct NumPlan {
    /// The NUL-terminated template. At most `-%0*X>%X` plus the terminator.
    template: [u8; 10],
    /// The value, reduced by the base when it did not fit.
    pub num: c_int,
    /// The width argument, for the plain form.
    pub width: c_int,
    /// How many powers of the base were divided out, for the reduced form.
    pub exp: Option<c_int>,
}

impl NumPlan {
    /// The template, for the caller to hand to `vim_snprintf`.
    pub(super) fn template(&self) -> &CStr {
        cstr::in_bytes(&self.template)
    }
}

/// Decide how `num` prints.
///
/// `alt_virtcol` is `%V`, which prints a `-` in front of the number and pays
/// a column for it. When the digits do not fit in `maxwid` the number is
/// reduced by the base and the count of divisions is printed after a `>`, so
/// that 14532 in four columns reads `14>3`.
pub(super) fn number_plan(
    alt_virtcol: bool,
    zeropad: bool,
    base: NumberBase,
    num: c_int,
    minwid: c_int,
    maxwid: c_int,
) -> NumPlan {
    let mut template = [0u8; 10];
    let mut len = 0usize;
    let mut push = |template: &mut [u8; 10], byte: u8| {
        template[len] = byte;
        len += 1;
    };
    let mut width = minwid;
    if alt_virtcol {
        push(&mut template, b'-');
        width -= 1;
    }
    push(&mut template, b'%');
    if zeropad {
        push(&mut template, b'0');
    }
    // The `*` takes the width as an argument rather than spelling it here.
    push(&mut template, b'*');
    let digit = if base == kNumBaseHexadecimal {
        b'X'
    } else {
        b'd'
    };
    push(&mut template, digit);

    // How many characters the number takes when printed.
    let mut num_chars = 1;
    let mut n = num;
    while n >= base as c_int {
        num_chars += 1;
        n /= base as c_int;
    }
    if alt_virtcol {
        // The `-` added above takes one more.
        num_chars += 1;
    }

    if num_chars <= maxwid {
        return NumPlan {
            template,
            num,
            width,
            exp: None,
        };
    }

    // The exponent takes two more characters of its own.
    num_chars += 2;
    let exp = num_chars - maxwid;
    let mut num = num;
    while {
        let before = num_chars;
        num_chars -= 1;
        before > maxwid
    } {
        num /= base as c_int;
    }
    push(&mut template, b'>');
    push(&mut template, b'%');
    push(&mut template, digit);
    NumPlan {
        template,
        num,
        width: 0,
        exp: Some(exp),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statusline::kNumBaseDecimal;

    fn spec(fmt: &[u8]) -> (Spec, usize) {
        let mut p = 0;
        let mut spec = Spec::read(fmt, &mut p);
        spec.finish(fmt, &mut p);
        (spec, p)
    }

    #[test]
    fn width_prefix_is_read_in_order() {
        let (s, p) = spec(b"0-12.34x");
        assert!(s.zeropad && s.left_align);
        assert_eq!((s.minwid, s.maxwid, p), (-12, 34, 7));

        let (s, p) = spec(b"7f");
        assert!(!s.zeropad && !s.left_align);
        assert_eq!((s.minwid, s.maxwid, p), (7, DEFAULT_MAXWID, 1));
    }

    #[test]
    fn widths_are_bounded_at_fifty() {
        assert_eq!(spec(b"49f").0.minwid, 49);
        assert_eq!(spec(b"50f").0.minwid, 50);
        assert_eq!(spec(b"51f").0.minwid, 50);
        assert_eq!(spec(b"2147483647f").0.minwid, 50);
        // The maximum width defaults to 50 rather than to none.
        assert_eq!(spec(b".99f").0.maxwid, 99);
    }

    #[test]
    fn a_width_too_wide_for_an_int_is_no_width_at_all() {
        // Twenty digits overflow, so the default comes back: no minimum
        // width, and the *default* maximum rather than 9999.
        assert_eq!(spec(b"12345678901234567890f").0.minwid, 0);
        assert_eq!(spec(b".12345678901234567890f").0.maxwid, MAX_ITEM_WIDTH);
        assert_eq!(digits(b"4294967296", &mut 0, -7), -7);
        assert_eq!(digits(b"2147483647", &mut 0, -7), 2147483647);
    }

    #[test]
    fn every_item_letter_is_recognised() {
        for byte in b"fFtcvVlLnkoObBrRhHyYwWmMqpPaNSCs{=<*#$TX@" {
            assert!(is_item_letter(*byte), "{}", *byte as char);
        }
        for byte in b"\0 dej!)(}." {
            assert!(!is_item_letter(*byte), "{}", *byte as char);
        }
    }

    #[test]
    fn a_number_that_fits_prints_plainly() {
        let plan = number_plan(false, false, kNumBaseDecimal, 42, 4, DEFAULT_MAXWID);
        assert_eq!(plan.template().to_bytes(), b"%*d");
        assert_eq!((plan.num, plan.width, plan.exp), (42, 4, None));

        let plan = number_plan(false, true, kNumBaseHexadecimal, 255, 4, DEFAULT_MAXWID);
        assert_eq!(plan.template().to_bytes(), b"%0*X");
    }

    #[test]
    fn a_number_too_wide_is_reduced_by_its_base() {
        // 14532 in four columns reads "14>3": three divisions by ten.
        let plan = number_plan(false, false, kNumBaseDecimal, 14532, 0, 4);
        assert_eq!(plan.template().to_bytes(), b"%*d>%d");
        assert_eq!((plan.num, plan.width, plan.exp), (14, 0, Some(3)));
    }

    #[test]
    fn the_alternate_virtual_column_pays_for_its_sign() {
        let plan = number_plan(true, false, kNumBaseDecimal, 7, 3, DEFAULT_MAXWID);
        assert_eq!(plan.template().to_bytes(), b"-%*d");
        assert_eq!((plan.num, plan.width), (7, 2));
    }
}
