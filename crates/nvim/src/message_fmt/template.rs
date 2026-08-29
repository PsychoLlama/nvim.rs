//! The message-template interpreter: what renders a *translated* message.
//!
//! An untranslated message never comes here -- `format_args!` has already
//! rendered it, checked against its arguments at compile time. This is the
//! other half of [the module's trade-off](super): a catalogue's template is
//! text that arrives at runtime, so something has to read it.
//!
//! The language is vim's `printf`, because that is what every msgid in this
//! tree is written in and what a catalogue built against nvim's own messages
//! carries, plus Rust's `{}` and `{0}` for a catalogue keyed on the Rust
//! spelling. Arguments arrive as [`TrArg`] -- a `Display` and nothing else --
//! so a template that asks for a conversion the argument is not gets the
//! argument's own rendering rather than a misread machine word.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::TrArg;
use core::fmt::Write as _;

/// A conversion's alignment and padding, read off the template.
struct Pad {
    left: bool,
    zero: bool,
    width: usize,
    precision: Option<usize>,
}

/// Render `template` against `args`.
///
/// The template language is vim's `printf` -- what every msgid in this tree
/// and every catalogue built against one is written in -- plus Rust's `{}`
/// and `{0}`, so that a catalogue keyed on the Rust spelling of a message
/// works too. Anything the parser does not recognise is emitted verbatim: a
/// translator's mistake shows up in the message, it does not lose the rest of
/// it.
pub(crate) fn render_template(template: &str, args: &[TrArg<'_>]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let b = template.as_bytes();
    let mut i = 0;
    let mut next = 0;
    while i < b.len() {
        match b[i] {
            b'%' => {
                if let Some(rest) = c_conversion(template, i, args, &mut next, &mut out) {
                    i = rest;
                } else {
                    out.push('%');
                    i += 1;
                }
            }
            b'{' => {
                if let Some(rest) = brace_conversion(template, i, args, &mut next, &mut out) {
                    i = rest;
                } else {
                    out.push('{');
                    i += 1;
                }
            }
            b'}' if b.get(i + 1) == Some(&b'}') => {
                out.push('}');
                i += 2;
            }
            _ => {
                // Emit the whole character, not the byte: the template is a
                // `str` and the output has to stay one.
                let len = char_len(b[i]);
                out.push_str(&template[i..i + len]);
                i += len;
            }
        }
    }
    out
}

/// The length in bytes of the UTF-8 character starting with `first`.
fn char_len(first: u8) -> usize {
    match first {
        0x00..=0x7f => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        _ => 4,
    }
}

/// Read the `%`-conversion at `at` and write its argument. Answers the index
/// just past it, or `None` when what is there is not a conversion.
fn c_conversion(
    template: &str,
    at: usize,
    args: &[TrArg<'_>],
    next: &mut usize,
    out: &mut String,
) -> Option<usize> {
    let b = template.as_bytes();
    let mut i = at + 1;
    if b.get(i) == Some(&b'%') {
        out.push('%');
        return Some(i + 1);
    }
    // `%N$` -- a translator reordering the arguments.
    let mut explicit = None;
    let digits = digit_run(b, i);
    if digits > i && b.get(digits) == Some(&b'$') {
        explicit = Some(number(&b[i..digits])?.checked_sub(1)?);
        i = digits + 1;
    }
    let mut left = false;
    let mut zero = false;
    while let Some(&c) = b.get(i) {
        match c {
            b'-' => left = true,
            b'0' => zero = true,
            b'+' | b' ' | b'#' | b'\'' => {}
            _ => break,
        }
        i += 1;
    }
    let width = match b.get(i) {
        Some(&b'*') => {
            i += 1;
            take(args, next, explicit_none())
                .and_then(as_usize)
                .unwrap_or(0)
        }
        _ => {
            let end = digit_run(b, i);
            let w = if end > i { number(&b[i..end])? } else { 0 };
            i = end;
            w
        }
    };
    let precision = if b.get(i) == Some(&b'.') {
        i += 1;
        match b.get(i) {
            Some(&b'*') => {
                i += 1;
                Some(
                    take(args, next, explicit_none())
                        .and_then(as_usize)
                        .unwrap_or(0),
                )
            }
            _ => {
                let end = digit_run(b, i);
                let p = if end > i { number(&b[i..end])? } else { 0 };
                i = end;
                Some(p)
            }
        }
    } else {
        None
    };
    while matches!(b.get(i), Some(b'h' | b'l' | b'L' | b'z' | b'j' | b't')) {
        i += 1;
    }
    let conv = *b.get(i)?;
    if !matches!(
        conv,
        b's' | b'd' | b'i' | b'u' | b'c' | b'p' | b'x' | b'X' | b'o' | b'f' | b'e' | b'E' | b'g'
    ) {
        return None;
    }
    let arg = take(args, next, explicit)?;
    let pad = Pad {
        left,
        zero,
        width,
        precision,
    };
    write_arg(out, arg, &pad, conv);
    Some(i + 1)
}

/// Read the `{}`-conversion at `at` and write its argument, Rust's spelling
/// of the same thing. Only `{{`, `}}`, `{}` and `{N}` are recognised; a
/// format spec is not, because a catalogue has no business carrying one.
fn brace_conversion(
    template: &str,
    at: usize,
    args: &[TrArg<'_>],
    next: &mut usize,
    out: &mut String,
) -> Option<usize> {
    let b = template.as_bytes();
    if b.get(at + 1) == Some(&b'{') {
        out.push('{');
        return Some(at + 2);
    }
    let end = digit_run(b, at + 1);
    if b.get(end) != Some(&b'}') {
        return None;
    }
    let explicit = if end > at + 1 {
        Some(number(&b[at + 1..end])?)
    } else {
        None
    };
    let arg = take(args, next, explicit)?;
    let _ = write!(out, "{arg}");
    Some(end + 1)
}

/// The index just past the run of digits starting at `from`.
fn digit_run(b: &[u8], from: usize) -> usize {
    let mut i = from;
    while matches!(b.get(i), Some(b'0'..=b'9')) {
        i += 1;
    }
    i
}

/// `digits` as a number, or `None` for one no message would carry.
fn number(digits: &[u8]) -> Option<usize> {
    let mut n: usize = 0;
    for &d in digits {
        n = n.checked_mul(10)?.checked_add(usize::from(d - b'0'))?;
    }
    (n < 4096).then_some(n)
}

/// The `explicit` argument for a `*` width, which never takes one.
fn explicit_none() -> Option<usize> {
    None
}

/// The argument a conversion refers to: `explicit`'s if it named one,
/// otherwise the next unconsumed one.
fn take<'a>(args: &[TrArg<'a>], next: &mut usize, explicit: Option<usize>) -> Option<TrArg<'a>> {
    match explicit {
        Some(i) => args.get(i).copied(),
        None => {
            let arg = args.get(*next).copied();
            *next += 1;
            arg
        }
    }
}

/// A `*` width or precision, read back off the argument's own rendering.
fn as_usize(arg: TrArg<'_>) -> Option<usize> {
    arg.to_string().trim().parse().ok()
}

/// Write `arg` padded as `pad` says, in the base `conv` asks for.
fn write_arg(out: &mut String, arg: TrArg<'_>, pad: &Pad, conv: u8) {
    let mut text = match pad.precision {
        Some(p) => format!("{arg:.p$}"),
        None => arg.to_string(),
    };
    if matches!(conv, b'x' | b'X' | b'o')
        && let Ok(n) = text.trim().parse::<i128>()
    {
        text = match conv {
            b'x' => format!("{n:x}"),
            b'X' => format!("{n:X}"),
            _ => format!("{n:o}"),
        };
    }
    let len = text.chars().count();
    if len >= pad.width {
        out.push_str(&text);
        return;
    }
    let fill = pad.width - len;
    if pad.left {
        out.push_str(&text);
        out.extend(core::iter::repeat_n(' ', fill));
    } else if pad.zero {
        let (sign, digits) = match text.strip_prefix('-') {
            Some(rest) => ("-", rest),
            None => ("", text.as_str()),
        };
        out.push_str(sign);
        out.extend(core::iter::repeat_n('0', fill));
        out.push_str(digits);
    } else {
        out.extend(core::iter::repeat_n(' ', fill));
        out.push_str(&text);
    }
}

#[cfg(test)]
mod tests {
    use super::{TrArg, render_template};

    /// Render `template` against `args`, which is what a translated message
    /// does. No catalogue in this tree reaches here, so the cases build the
    /// templates by hand.
    fn render(template: &str, args: &[&dyn core::fmt::Display]) -> String {
        let args: Vec<TrArg<'_>> = args.iter().map(TrArg::of).collect();
        render_template(template, &args)
    }

    #[test]
    fn a_template_with_no_conversion_is_itself() {
        assert_eq!(render("E32: No file name", &[]), "E32: No file name");
    }

    #[test]
    fn the_conversions_the_tree_uses() {
        assert_eq!(render("say %s", &[&"hi"]), "say hi");
        assert_eq!(render("n=%d", &[&7]), "n=7");
        assert_eq!(render("n=%i", &[&7]), "n=7");
        assert_eq!(render("n=%ld", &[&-7i64]), "n=-7");
        assert_eq!(render("n=%lld", &[&-7i64]), "n=-7");
        assert_eq!(render("n=%u", &[&7u32]), "n=7");
        assert_eq!(render("n=%zu", &[&7usize]), "n=7");
        assert_eq!(render("c=%c", &[&'x']), "c=x");
        assert_eq!(render("p=%p", &[&"0x1"]), "p=0x1");
        assert_eq!(render("100%% sure", &[]), "100% sure");
    }

    #[test]
    fn width_and_precision() {
        assert_eq!(render("[%5ld]", &[&42]), "[   42]");
        assert_eq!(render("[%-5ld]", &[&42]), "[42   ]");
        assert_eq!(render("[%05d]", &[&42]), "[00042]");
        assert_eq!(render("[%05d]", &[&-42]), "[-0042]");
        assert_eq!(render("[%3d]", &[&123456]), "[123456]");
        assert_eq!(render("[%.3s]", &[&"abcdef"]), "[abc]");
        // `%.*s` takes its precision from the argument before the string,
        // which is how the tree spells "at most this many".
        assert_eq!(render("[%.*s]", &[&2, &"abcdef"]), "[ab]");
    }

    #[test]
    fn hexadecimal_and_octal_reread_the_argument() {
        assert_eq!(render("%lx", &[&255]), "ff");
        assert_eq!(render("%X", &[&255]), "FF");
        assert_eq!(render("%o", &[&8]), "10");
        // Not a number: the argument renders itself rather than vanishing.
        assert_eq!(render("%x", &[&"zz"]), "zz");
    }

    #[test]
    fn a_translator_may_reorder_the_arguments() {
        assert_eq!(
            render("%2$s before %1$s", &[&"second", &"first"]),
            "first before second"
        );
        assert_eq!(render("{1} then {0}", &[&"a", &"b"]), "b then a");
    }

    #[test]
    fn rusts_own_spelling_works_too() {
        assert_eq!(render("say {}", &[&"hi"]), "say hi");
        assert_eq!(render("{} and {}", &[&1, &2]), "1 and 2");
        assert_eq!(render("{{literal}}", &[]), "{literal}");
    }

    #[test]
    fn a_conversion_the_argument_is_not_still_renders() {
        // The variadic macros read a machine word here and crashed. A
        // `Display` cannot: it prints what it is.
        assert_eq!(render("%d", &[&"not a number"]), "not a number");
        assert_eq!(render("%s", &[&12]), "12");
    }

    #[test]
    fn a_broken_template_keeps_the_rest_of_the_message() {
        // More conversions than arguments: the extra one is left as text.
        assert_eq!(render("%s and %s", &[&"one"]), "one and %s");
        // Not a conversion at all -- the regexp messages are full of these.
        assert_eq!(render("after \\%[dxouU]", &[]), "after \\%[dxouU]");
        assert_eq!(render("atom '\\%#=1'", &[]), "atom '\\%#=1'");
        assert_eq!(render("unmatched \\%(", &[]), "unmatched \\%(");
        assert_eq!(render("trailing %", &[]), "trailing %");
        assert_eq!(render("brace { and }", &[]), "brace { and }");
    }

    #[test]
    fn non_ascii_text_survives() {
        assert_eq!(render("… %s …", &[&"ünïcode"]), "… ünïcode …");
    }
}
