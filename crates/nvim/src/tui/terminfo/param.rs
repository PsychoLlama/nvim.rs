//! terminfo's parameterised strings.
//!
//! A capability like `cursor_address` is a little stack program:
//! `\033[%i%p1%d;%p2%dH` pushes two parameters, increments them and prints
//! them. This module runs those programs. It is a rewrite of the interpreter
//! nvim took from NetBSD's libterminfo, with the same operators, the same
//! stack depth and the same "produce nothing at all rather than a truncated
//! sequence" behaviour when the output does not fit.
//!
//! The printf-style conversions (`%d`, `%s` and friends) used to be handed to
//! the C library with a format string assembled at runtime. They are rendered
//! here instead, which is what lets the whole interpreter be safe code. Two
//! consequences are worth naming, both in territory the C left undefined:
//!
//! - Upstream passed an `int` where the assembled format said `%ld`, so the
//!   upper half of the value was whatever happened to be in the register.
//!   The rendering below prints the parameter as the `int` it is.
//! - A width above 10000, or a second `.`, made upstream give up on the
//!   conversion and assemble a lone `%` for `snprintf`, which printed
//!   nothing. Giving up here means the same: no conversion happens, and what
//!   followed the offending character is printed as the literal text it now
//!   is.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use crate::narrow::number_as_int;

/// Nested so these short names stay out of the flat namespace the unit-test
/// cdefs are generated into.
mod limits {
    /// How deep the operand stack goes. Overflowing it fails the whole
    /// expansion, which is the only way a well-formed capability can fail
    /// without running out of output room.
    pub const STACK_DEPTH: usize = 20;

    /// The widest a rendered `long` can be, which upstream used as the floor
    /// for how much room a numeric conversion demands.
    pub const LONG_STR_MAX: usize = 21;
}

use limits::{LONG_STR_MAX, STACK_DEPTH};

/// One of the nine parameters a capability is expanded with.
#[derive(Clone, Copy, Default)]
pub struct Param<'a> {
    pub num: i64,
    pub string: Option<&'a [u8]>,
}

/// The output buffer, filled left to right. Nothing is ever partially
/// written: a conversion that will not fit fails the expansion first.
pub struct Out<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Out<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Out { buf, pos: 0 }
    }

    /// How many bytes were written.
    pub fn len(&self) -> usize {
        self.pos
    }

    /// Whether nothing has been written yet.
    pub fn is_empty(&self) -> bool {
        self.pos == 0
    }

    fn room(&self) -> usize {
        self.buf.len() - self.pos
    }

    fn push(&mut self, byte: u8) {
        self.buf[self.pos] = byte;
        self.pos += 1;
    }

    /// One literal byte. NUL is written as 0x80, because the result is a
    /// NUL-terminated C string and a real NUL would end it early; upstream
    /// also insisted on room for that terminator, hence the two.
    fn put_char(&mut self, byte: u8) -> bool {
        if self.room() < 2 {
            return false;
        }
        self.push(if byte == 0 { 0o200 } else { byte });
        true
    }
}

/// A writer that drops anything past `budget`, the way the truncating
/// `snprintf` upstream called did. The caller has already checked that the
/// budget fits in the output.
struct Capped<'a, 'b> {
    out: &'a mut Out<'b>,
    budget: usize,
}

impl Capped<'_, '_> {
    fn byte(&mut self, byte: u8) {
        if self.budget > 0 {
            self.budget -= 1;
            self.out.push(byte);
        }
    }

    fn bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.byte(byte);
        }
    }

    fn repeat(&mut self, byte: u8, count: usize) {
        for _ in 0..count {
            self.byte(byte);
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Slot<'a> {
    num: i64,
    string: Option<&'a [u8]>,
}

#[derive(Default)]
struct Stack<'a> {
    slots: [Slot<'a>; STACK_DEPTH],
    depth: usize,
}

impl<'a> Stack<'a> {
    /// Reports whether the push fit.
    fn push(&mut self, slot: Slot<'a>) -> bool {
        if self.depth >= STACK_DEPTH {
            return false;
        }
        self.slots[self.depth] = slot;
        self.depth += 1;
        true
    }

    /// Popping an empty stack yields a zero/absent slot, as upstream's did.
    fn pop(&mut self) -> Slot<'a> {
        match self.depth.checked_sub(1) {
            Some(d) => {
                self.depth = d;
                self.slots[d]
            }
            None => Slot::default(),
        }
    }
}

/// The numeric conversions, which differ only in base and case.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Radix {
    Dec,
    Oct,
    LowerHex,
    UpperHex,
}

/// A parsed printf-style conversion spec.
#[derive(Default)]
struct Spec {
    /// `#`: force a base prefix.
    alt: bool,
    /// ` `: a space where a sign would go.
    space: bool,
    /// `0`: pad with zeros rather than spaces.
    zero: bool,
    /// `-`: pad on the right. Only reachable after a `:`, because a bare `-`
    /// is terminfo's subtraction operator.
    left: bool,
    width: usize,
    /// `Some` once a `.` has been seen, even as `%.d`.
    precision: Option<usize>,
    /// A width above 10000 or a second `.`. The parse gives up on the spot,
    /// so the command character is whatever caused it rather than a
    /// conversion -- this only ever reaches the arms that ignore it, and is
    /// kept because it is where upstream's `done == 2` went.
    invalid: bool,
}

impl Spec {
    /// The room a conversion demands, which upstream derived from the widest
    /// of the width and the precision.
    fn output_len(&self) -> usize {
        if self.invalid {
            0
        } else {
            self.width.max(self.precision.unwrap_or(0))
        }
    }

    /// Render an integer. `value` is the parameter truncated to an `int`,
    /// which is the width the conversion was given.
    fn render_int(&self, value: c_int, radix: Radix, out: &mut Capped) {
        let mut digits = [0u8; 32];
        let mut n = digits.len();
        let (magnitude, sign) = match radix {
            Radix::Dec => (u64::from(value.unsigned_abs()), value < 0),
            // The other conversions read their argument as unsigned.
            _ => (u64::from(value.cast_unsigned()), false),
        };
        let (base, alphabet) = match radix {
            Radix::Dec => (10, b"0123456789abcdef"),
            Radix::Oct => (8, b"0123456789abcdef"),
            Radix::LowerHex => (16, b"0123456789abcdef"),
            Radix::UpperHex => (16, b"0123456789ABCDEF"),
        };
        let mut rest = magnitude;
        loop {
            n -= 1;
            let digit = usize::try_from(rest % base).expect("a digit is below the radix");
            digits[n] = alphabet[digit];
            rest /= base;
            if rest == 0 {
                break;
            }
        }
        let mut body = &digits[n..];
        // A precision of zero prints nothing at all for a zero value.
        if self.precision == Some(0) && magnitude == 0 {
            body = &[];
        }

        let mut prefix: &[u8] = b"";
        if sign {
            prefix = b"-";
        } else if self.space && radix == Radix::Dec {
            prefix = b" ";
        }
        if self.alt {
            match radix {
                Radix::Oct if body.first() != Some(&b'0') => prefix = b"0",
                Radix::LowerHex if magnitude != 0 => prefix = b"0x",
                Radix::UpperHex if magnitude != 0 => prefix = b"0X",
                _ => {}
            }
        }

        let precision = self.precision.unwrap_or(0);
        let zeros = precision.saturating_sub(body.len());
        // The zero flag is ignored when the conversion is left-aligned or
        // carries its own precision.
        let pad_zeros = if self.zero && !self.left && self.precision.is_none() {
            self.width.saturating_sub(prefix.len() + zeros + body.len())
        } else {
            0
        };
        let content = prefix.len() + pad_zeros + zeros + body.len();
        let pad = self.width.saturating_sub(content);

        if !self.left {
            out.repeat(b' ', pad);
        }
        out.bytes(prefix);
        out.repeat(b'0', pad_zeros + zeros);
        out.bytes(body);
        if self.left {
            out.repeat(b' ', pad);
        }
    }

    /// Render a string: the precision truncates it, the width pads it.
    fn render_str(&self, value: &[u8], out: &mut Capped) {
        let body = &value[..self.precision.unwrap_or(value.len()).min(value.len())];
        let pad = self.width.saturating_sub(body.len());
        if !self.left {
            out.repeat(b' ', pad);
        }
        out.bytes(body);
        if self.left {
            out.repeat(b' ', pad);
        }
    }
}

/// Expand `capability` into `out`, reporting whether it succeeded. Failure
/// leaves whatever was written before it behind; every caller throws the
/// buffer away.
///
/// `params` is mutable because `%i` increments the first two of them in
/// place, which the caller can see -- that is how one-based coordinates are
/// spelled, and why callers that mean to retry hand over a copy.
pub fn expand(capability: &[u8], params: &mut [Param; 9], out: &mut Out) -> bool {
    let mut stack = Stack::default();
    // terminfo's dynamic (a-z) and static (A-Z) variables. Neither survives
    // the call, which upstream noted and nothing depends on.
    let mut dynamic = [0i64; 26];
    let mut statics = [0i64; 26];
    let mut i = 0;

    while i < capability.len() {
        let literal = capability[i];
        i += 1;
        if literal != b'%' {
            if !out.put_char(literal) {
                return false;
            }
            continue;
        }
        let Some(&introducer) = capability.get(i) else {
            // A capability ending in a bare `%`. Upstream read one byte past
            // the string here; there is nothing to do but stop.
            break;
        };
        i += 1;
        if introducer == b'%' {
            if !out.put_char(b'%') {
                return false;
            }
            continue;
        }

        let (spec, command) = parse_spec(capability, &mut i, introducer);
        match command {
            b'c' => {
                let value = stack.pop().num;
                // `%c` prints the low byte, as upstream's `(char)` did.
                let [low, ..] = value.cast_unsigned().to_le_bytes();
                if !out.put_char(low) {
                    return false;
                }
            }
            b's' => {
                if let Some(value) = stack.pop().string {
                    let room = value.len().max(spec.output_len());
                    if out.room() < room + 1 {
                        return false;
                    }
                    spec.render_str(value, &mut Capped { out, budget: room });
                }
            }
            b'l' => {
                let len = stack.pop().string.map_or(0, <[u8]>::len);
                if !stack.push(Slot {
                    num: i64::try_from(len).expect("a terminfo string is short"),
                    string: None,
                }) {
                    return false;
                }
            }
            b'd' | b'o' | b'x' | b'X' => {
                let value = stack.pop().num;
                let room = spec.output_len().max(LONG_STR_MAX);
                if out.room() < room + 2 {
                    return false;
                }
                if !spec.invalid {
                    let radix = match command {
                        b'd' => Radix::Dec,
                        b'o' => Radix::Oct,
                        b'x' => Radix::LowerHex,
                        _ => Radix::UpperHex,
                    };
                    spec.render_int(
                        number_as_int(value),
                        radix,
                        &mut Capped {
                            out,
                            budget: room + 1,
                        },
                    );
                }
            }
            b'p' => {
                if let Some(&digit @ b'1'..=b'9') = capability.get(i) {
                    i += 1;
                    let param = params[(digit - b'1') as usize];
                    if !stack.push(Slot {
                        num: param.num,
                        string: param.string,
                    }) {
                        return false;
                    }
                }
            }
            // `%P` and `%g` deliberately leave the variable's letter in the
            // capability: upstream did not consume it, so it is printed as a
            // literal afterwards. Terminals nvim knows about do not use
            // either operator.
            b'P' => {
                let value = stack.pop().num;
                match capability.get(i) {
                    Some(&name @ b'a'..=b'z') => dynamic[(name - b'a') as usize] = value,
                    Some(&name @ b'A'..=b'Z') => statics[(name - b'A') as usize] = value,
                    _ => {}
                }
            }
            b'g' => {
                let value = match capability.get(i) {
                    Some(&name @ b'a'..=b'z') => Some(dynamic[(name - b'a') as usize]),
                    Some(&name @ b'A'..=b'Z') => Some(statics[(name - b'A') as usize]),
                    _ => None,
                };
                if let Some(value) = value
                    && !stack.push(Slot {
                        num: value,
                        string: None,
                    })
                {
                    return false;
                }
            }
            b'i' => {
                params[0].num += 1;
                params[1].num += 1;
            }
            b'\'' => {
                let literal = capability.get(i).copied().unwrap_or(0);
                i += 1;
                if !stack.push(Slot {
                    num: i64::from(literal),
                    string: None,
                }) {
                    return false;
                }
                skip_past(capability, &mut i, b'\'');
            }
            b'{' => {
                let mut value: i64 = 0;
                while let Some(&digit @ b'0'..=b'9') = capability.get(i) {
                    // Wrapping, because a hostile description can spell a
                    // constant far past what a long holds.
                    value = value.wrapping_mul(10).wrapping_add(i64::from(digit - b'0'));
                    i += 1;
                }
                if !stack.push(Slot {
                    num: value,
                    string: None,
                }) {
                    return false;
                }
                skip_past(capability, &mut i, b'}');
            }
            b'+' | b'-' | b'*' | b'/' | b'm' | b'A' | b'O' | b'&' | b'|' | b'^' | b'=' | b'<'
            | b'>' => {
                let rhs = stack.pop().num;
                let lhs = stack.pop().num;
                let value = binary_op(command, lhs, rhs);
                if !stack.push(Slot {
                    num: value,
                    string: None,
                }) {
                    return false;
                }
            }
            b'!' | b'~' => {
                let value = stack.pop().num;
                let value = if command == b'!' {
                    i64::from(value == 0)
                } else {
                    !value
                };
                if !stack.push(Slot {
                    num: value,
                    string: None,
                }) {
                    return false;
                }
            }
            // `%?` opens a conditional and does nothing on its own; `%;`
            // closes one and is likewise only a marker for the skips below.
            b'?' | b';' => {}
            b't' => {
                if stack.pop().num == 0 {
                    skip_branch(capability, &mut i, true);
                }
            }
            b'e' => skip_branch(capability, &mut i, false),
            _ => {}
        }
    }
    true
}

/// Parse the flags, width and precision between `%` and the command
/// character, leaving `i` just past the command.
fn parse_spec(capability: &[u8], i: &mut usize, introducer: u8) -> (Spec, u8) {
    /// A parsed width or precision. The digit loop below rejects anything
    /// over 10000, so the value is always small and never negative.
    fn as_width(value: i64) -> usize {
        usize::try_from(value).expect("a width is bounded by 10000")
    }

    /// How long a format string upstream could assemble, which bounded this
    /// loop even when nothing terminated it.
    const SPEC_MAX: usize = 64;

    let mut spec = Spec::default();
    let mut command = introducer;
    // Counts what upstream would have written into its format buffer, `%`
    // included, so an endless run of flags stops in the same place.
    let mut written = 1;
    let mut value: i64 = 0;
    // A `:` marks the following `-` as a flag rather than the subtraction
    // operator.
    let mut minus_is_flag = false;
    let mut seen_digit = false;

    while written < SPEC_MAX {
        match command {
            b'c' | b's' => {
                break;
            }
            b'd' | b'o' | b'x' | b'X' => {
                break;
            }
            b'#' => {
                spec.alt = true;
                written += 1;
            }
            b' ' => {
                spec.space = true;
                written += 1;
            }
            b'.' => {
                written += 1;
                if spec.precision.is_none() {
                    spec.width = as_width(value);
                    spec.precision = Some(0);
                } else {
                    spec.invalid = true;
                }
                value = 0;
                seen_digit = false;
            }
            b':' => minus_is_flag = true,
            b'-' => {
                if !minus_is_flag {
                    break;
                }
                spec.left = true;
                written += 1;
            }
            b'0'..=b'9' => {
                if !seen_digit && command == b'0' && spec.precision.is_none() {
                    spec.zero = true;
                }
                seen_digit = true;
                value = value * 10 + i64::from(command - b'0');
                if value > 10000 {
                    spec.invalid = true;
                } else {
                    written += 1;
                }
            }
            _ => break,
        }
        if spec.invalid {
            break;
        }
        match capability.get(*i) {
            Some(&next) => {
                command = next;
                *i += 1;
            }
            None => {
                command = 0;
                break;
            }
        }
    }
    if !spec.invalid {
        match spec.precision {
            None => spec.width = as_width(value),
            Some(_) => spec.precision = Some(as_width(value)),
        }
    }
    (spec, command)
}

fn binary_op(op: u8, lhs: i64, rhs: i64) -> i64 {
    match op {
        // Wrapping throughout: the operands come from the description and
        // from parameters, and upstream wrapped rather than trapping.
        b'+' => lhs.wrapping_add(rhs),
        b'-' => lhs.wrapping_sub(rhs),
        b'*' => lhs.wrapping_mul(rhs),
        // Division and remainder by zero yield zero rather than trapping.
        b'/' => {
            if rhs == 0 {
                0
            } else {
                lhs.wrapping_div(rhs)
            }
        }
        b'm' => {
            if rhs == 0 {
                0
            } else {
                lhs.wrapping_rem(rhs)
            }
        }
        b'A' => i64::from(lhs != 0 && rhs != 0),
        b'O' => i64::from(lhs != 0 || rhs != 0),
        b'&' => lhs & rhs,
        b'|' => lhs | rhs,
        b'^' => lhs ^ rhs,
        b'=' => i64::from(lhs == rhs),
        b'<' => i64::from(lhs < rhs),
        b'>' => i64::from(lhs > rhs),
        _ => 0,
    }
}

/// Skip to just past `terminator`, or to the end of the capability.
fn skip_past(capability: &[u8], i: &mut usize, terminator: u8) {
    while let Some(&byte) = capability.get(*i) {
        *i += 1;
        if byte == terminator {
            return;
        }
    }
}

/// Skip the branch of a conditional that is not taken.
///
/// `stop_at_else` is what separates `%t` (the untaken then-branch, which ends
/// at either `%e` or `%;`) from `%e` (the untaken else-branch, which runs to
/// `%;`). Nested `%?`s are counted so an inner conditional's `%;` does not
/// end the outer one.
fn skip_branch(capability: &[u8], i: &mut usize, stop_at_else: bool) {
    let mut depth = 0usize;
    while let Some(&byte) = capability.get(*i) {
        if byte == b'%' {
            *i += 1;
            match capability.get(*i) {
                Some(b'?') => depth += 1,
                Some(b';') => {
                    if depth == 0 {
                        *i += 1;
                        return;
                    }
                    depth -= 1;
                }
                Some(b'e') if stop_at_else && depth == 0 => {
                    *i += 1;
                    return;
                }
                _ => {}
            }
        }
        *i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(capability: &str, nums: &[i64]) -> Option<String> {
        let mut buf = [0u8; 256];
        run_in(&mut buf, capability, nums)
    }

    fn run_in(buf: &mut [u8], capability: &str, nums: &[i64]) -> Option<String> {
        let mut params = [Param::default(); 9];
        for (slot, &num) in params.iter_mut().zip(nums) {
            slot.num = num;
        }
        run_with_in(buf, params, capability)
    }

    fn run_with(params: [Param; 9], capability: &str) -> Option<String> {
        let mut buf = [0u8; 256];
        run_with_in(&mut buf, params, capability)
    }

    fn run_with_in(buf: &mut [u8], mut params: [Param; 9], capability: &str) -> Option<String> {
        let mut out = Out::new(buf);
        if !expand(capability.as_bytes(), &mut params, &mut out) {
            return None;
        }
        let n = out.len();
        Some(String::from_utf8_lossy(&buf[..n]).into_owned())
    }

    #[test]
    fn literals_pass_through() {
        assert_eq!(run("\x1b[H", &[]).as_deref(), Some("\x1b[H"));
        assert_eq!(run("100%%", &[]).as_deref(), Some("100%"));
    }

    #[test]
    fn cursor_address_is_one_based() {
        let got = run("\x1b[%i%p1%d;%p2%dH", &[7, 12]);
        assert_eq!(got.as_deref(), Some("\x1b[8;13H"));
    }

    #[test]
    fn increment_is_visible_to_the_caller() {
        let mut params = [Param::default(); 9];
        params[0].num = 4;
        params[1].num = 9;
        let mut buf = [0u8; 16];
        assert!(expand(b"%i", &mut params, &mut Out::new(&mut buf)));
        assert_eq!((params[0].num, params[1].num), (5, 10));
    }

    #[test]
    fn conversions_render_like_printf() {
        assert_eq!(run("%p1%d", &[-3]).as_deref(), Some("-3"));
        assert_eq!(run("%p1%o", &[8]).as_deref(), Some("10"));
        assert_eq!(run("%p1%x", &[255]).as_deref(), Some("ff"));
        assert_eq!(run("%p1%X", &[255]).as_deref(), Some("FF"));
        assert_eq!(run("%p1%02d", &[7]).as_deref(), Some("07"));
        assert_eq!(run("%p1%3d", &[7]).as_deref(), Some("  7"));
        assert_eq!(run("%p1%:-3d", &[7]).as_deref(), Some("7  "));
        assert_eq!(run("%p1%.3d", &[7]).as_deref(), Some("007"));
        assert_eq!(run("%p1%#x", &[255]).as_deref(), Some("0xff"));
        assert_eq!(run("%p1%#x", &[0]).as_deref(), Some("0"));
        assert_eq!(run("%p1%#o", &[8]).as_deref(), Some("010"));
        assert_eq!(run("%p1% d", &[7]).as_deref(), Some(" 7"));
        assert_eq!(run("%p1%.0d", &[0]).as_deref(), Some(""));
    }

    /// The parameter is an `int`, and the other conversions read it unsigned.
    #[test]
    fn conversions_are_int_wide() {
        assert_eq!(run("%p1%d", &[1 << 32]).as_deref(), Some("0"));
        assert_eq!(run("%p1%x", &[-1]).as_deref(), Some("ffffffff"));
    }

    #[test]
    fn a_bare_minus_is_subtraction() {
        assert_eq!(run("%p1%p2%-%d", &[10, 4]).as_deref(), Some("6"));
        assert_eq!(run("%{9}%{4}%-%d", &[]).as_deref(), Some("5"));
    }

    #[test]
    fn arithmetic_and_comparison() {
        assert_eq!(run("%{6}%{7}%*%d", &[]).as_deref(), Some("42"));
        assert_eq!(run("%{7}%{0}%/%d", &[]).as_deref(), Some("0"));
        assert_eq!(run("%{7}%{0}%m%d", &[]).as_deref(), Some("0"));
        assert_eq!(run("%{5}%{5}%=%d", &[]).as_deref(), Some("1"));
        assert_eq!(run("%{4}%{5}%<%d", &[]).as_deref(), Some("1"));
        assert_eq!(run("%{4}%{5}%>%d", &[]).as_deref(), Some("0"));
        assert_eq!(run("%{12}%{10}%&%d", &[]).as_deref(), Some("8"));
        assert_eq!(run("%{0}%!%d", &[]).as_deref(), Some("1"));
        assert_eq!(run("%{0}%~%d", &[]).as_deref(), Some("-1"));
        assert_eq!(run("%'A'%d", &[]).as_deref(), Some("65"));
    }

    /// A hostile description can spell a constant far past what fits.
    #[test]
    fn a_huge_constant_wraps_rather_than_panicking() {
        assert!(run("%{99999999999999999999}%d", &[]).is_some());
    }

    #[test]
    fn a_character_conversion_prints_the_low_byte() {
        // `%c` is upstream's `(char)`: everything above the low byte is
        // dropped rather than rejected, and a negative value wraps.
        assert_eq!(run("%p1%c", &[i64::from(b'A')]).as_deref(), Some("A"));
        assert_eq!(run("%p1%c", &[0x4141_4141_4141_4141]).as_deref(), Some("A"));
        assert_eq!(run("%p1%c", &[-191]).as_deref(), Some("A"));
    }

    #[test]
    fn the_unsigned_conversions_read_a_negative_parameter_as_an_int() {
        // `conversions_are_int_wide` pins `%d` and `%x`; the point here is
        // that `%o` and `%X` read the same 32 unsigned bits, and that a
        // width above the parameter still pads the wrapped digits.
        assert_eq!(run("%p1%X", &[-255]).as_deref(), Some("FFFFFF01"));
        assert_eq!(run("%p1%o", &[-1]).as_deref(), Some("37777777777"));
        assert_eq!(
            run("%p1%12x", &[0x1_0000_0007]).as_deref(),
            Some("           7")
        );
    }

    #[test]
    fn conditionals_pick_a_branch() {
        let cap = "%?%p1%tyes%eno%;!";
        assert_eq!(run(cap, &[1]).as_deref(), Some("yes!"));
        assert_eq!(run(cap, &[0]).as_deref(), Some("no!"));
    }

    #[test]
    fn nested_conditionals_match_their_own_terminators() {
        let cap = "%?%p1%t%?%p2%tab%ecd%;%eef%;.";
        assert_eq!(run(cap, &[1, 1]).as_deref(), Some("ab."));
        assert_eq!(run(cap, &[1, 0]).as_deref(), Some("cd."));
        assert_eq!(run(cap, &[0, 1]).as_deref(), Some("ef."));
    }

    /// `set_attributes` as the `ansi` description writes it: nine
    /// conditionals over nine parameters.
    #[test]
    fn set_attributes_of_the_ansi_description() {
        let cap = "\x1b[0;10%?%p1%t;7%;%?%p2%t;4%;%?%p3%t;7%;%?%p4%t;5%;\
                   %?%p6%t;1%;%?%p7%t;8%;%?%p9%t;11%;m";
        assert_eq!(run(cap, &[0; 9]).as_deref(), Some("\x1b[0;10m"));
        assert_eq!(
            run(cap, &[0, 1, 0, 0, 0, 1]).as_deref(),
            Some("\x1b[0;10;4;1m")
        );
    }

    #[test]
    fn strings_render_and_pad() {
        let mut params = [Param::default(); 9];
        params[0].string = Some(b"hello");
        assert_eq!(run_with(params, "[%p1%s]").as_deref(), Some("[hello]"));
        assert_eq!(
            run_with(params, "%p1%8s|%p1%.2s").as_deref(),
            Some("   hello|he")
        );
    }

    #[test]
    fn string_length_is_pushable() {
        let mut params = [Param::default(); 9];
        params[0].string = Some(b"abcd");
        assert_eq!(run_with(params, "%p1%l%d").as_deref(), Some("4"));
        // A parameter with no string at all has length zero.
        assert_eq!(run("%p2%l%d", &[]).as_deref(), Some("0"));
    }

    /// The stack is 20 deep: the twenty-first push fails the expansion, and
    /// nothing after it runs. This was `test/unit/tui/terminfo_spec.lua`'s
    /// only case, which is why that spec no longer exists.
    #[test]
    fn the_stack_is_twenty_deep() {
        let fits = "%p1".repeat(20) + "%c";
        let overflows = "%p1".repeat(21) + "%c";
        assert_eq!(run(&fits, &[i64::from(b'A')]).as_deref(), Some("A"));
        assert_eq!(run(&overflows, &[i64::from(b'A')]), None);
    }

    /// Output that does not fit fails rather than truncating, and a literal
    /// byte needs room for the terminator the caller will add.
    #[test]
    fn a_full_buffer_fails_the_expansion() {
        let mut buf = [0u8; 4];
        assert_eq!(run_in(&mut buf, "abc", &[]).as_deref(), Some("abc"));
        let mut buf = [0u8; 3];
        assert_eq!(run_in(&mut buf, "abc", &[]), None);
        // A numeric conversion insists on room for the widest long.
        let mut buf = [0u8; 22];
        assert_eq!(run_in(&mut buf, "%p1%d", &[1]), None);
        let mut buf = [0u8; 23];
        assert_eq!(run_in(&mut buf, "%p1%d", &[1]).as_deref(), Some("1"));
    }

    #[test]
    fn a_nul_byte_is_written_as_0x80() {
        let mut buf = [0u8; 16];
        let mut out = Out::new(&mut buf);
        assert!(expand(b"%{0}%c", &mut [Param::default(); 9], &mut out));
        let written = out.len();
        assert_eq!(&buf[..written], b"\x80");
    }

    /// Preserved quirks: `%P`/`%g` leave the variable name behind, and a
    /// conversion upstream could not assemble prints nothing.
    #[test]
    fn preserved_oddities() {
        assert_eq!(run("%{7}%Pa%ga%d", &[]).as_deref(), Some("aa7"));
        // A spec upstream could not assemble stops being a spec: the rest of
        // it is printed literally.
        assert_eq!(run("%p1%99999d", &[7]).as_deref(), Some("d"));
        assert_eq!(run("%p1%.2.3d", &[7]).as_deref(), Some("3d"));
        // A capability that ends mid-conversion simply stops.
        assert_eq!(run("ab%", &[]).as_deref(), Some("ab"));
        assert_eq!(run("ab%3", &[]).as_deref(), Some("ab"));
    }
}
