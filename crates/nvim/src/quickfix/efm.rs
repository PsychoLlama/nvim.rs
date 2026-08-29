//! Compiling `'errorformat'` into regular expressions.
//!
//! The option is a comma-separated list of formats. [`Efm::compile`] splits
//! it and turns each part into a [`Format`] holding a compiled pattern plus
//! a note of which submatch each `%` conversion captures; `parse.rs` then
//! runs those patterns over the lines being read.
//!
//! [`Format::compile_regpat`] is the translator. `%f`, `%l`, `%m` and the rest
//! become capture groups built from the [`FMT_PAT`] table, `%*` takes a
//! scanf conversion, a prefix such as `%E` or `%D` is recorded rather than
//! matched, and every other character is copied with the regexp atoms
//! escaped.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::regexp::{RE_MAGIC, RE_STRING};
use crate::semsg;
use core::ffi::{CStr, c_char};
use core::ptr;

/// How many `%` conversions there are. Each may appear at most once in one
/// format, and [`Format::addr`] has a slot per conversion.
pub(crate) const FMT_PATTERNS: usize = 14;
/// The index of `%m` (the error message) in [`FMT_PAT`] and [`Format::addr`].
pub(crate) const FMT_PATTERN_M: usize = 8;
/// The index of `%r` (the rest of a single-line file message).
pub(crate) const FMT_PATTERN_R: usize = 9;

/// The regexp each `%` conversion expands to, in [`Format::addr`] order.
///
/// Keep in sync with `parse.rs`'s handler table: slot *i* here is parsed by
/// handler *i* there.
pub(crate) static FMT_PAT: [(u8, &[u8]); FMT_PATTERNS] = [
    (b'f', b".\\+"),     // 0, only used when at the end
    (b'b', b"\\d\\+"),   // 1
    (b'n', b"\\d\\+"),   // 2
    (b'l', b"\\d\\+"),   // 3
    (b'e', b"\\d\\+"),   // 4
    (b'c', b"\\d\\+"),   // 5
    (b'k', b"\\d\\+"),   // 6
    (b't', b"."),        // 7
    (b'm', b".\\+"),     // 8, FMT_PATTERN_M
    (b'r', b".*"),       // 9, FMT_PATTERN_R
    (b'p', b"[-\t .]*"), // 10
    (b'v', b"\\d\\+"),   // 11
    (b's', b".\\+"),     // 12
    (b'o', b".\\+"),     // 13
];

/// One part of `'errorformat'`, compiled.
pub(crate) struct Format {
    /// The compiled pattern. Owned: freed when the `Format` is dropped, and
    /// replaced in place by [`Format::exec`] when the engine rewrites it.
    prog: *mut regprog_T,
    /// Which submatch each `%` conversion captured, 1-based; 0 when the
    /// conversion does not appear in this format.
    addr: [u8; FMT_PATTERNS],
    /// The prefix character this format was given — one of `DXAEWINCZGOPQ`
    /// — or 0 for a plain format.
    prefix: u8,
    /// `-` (drop this line) or `+` (take the whole line as the message), or
    /// 0.
    flags: u8,
    /// `%>` was used: the next line should start matching at this format
    /// rather than at the first.
    conthere: bool,
}

impl Drop for Format {
    fn drop(&mut self) {
        // SAFETY: `prog` is this format's own compiled pattern, or null.
        unsafe { vim_regfree(self.prog) };
    }
}

impl Format {
    const fn new() -> Format {
        Format {
            prog: ptr::null_mut(),
            addr: [0; FMT_PATTERNS],
            prefix: 0,
            flags: 0,
            conthere: false,
        }
    }

    /// The prefix character, or 0.
    pub(crate) fn prefix(&self) -> u8 {
        self.prefix
    }

    /// The `-`/`+` flag, or 0.
    pub(crate) fn flags(&self) -> u8 {
        self.flags
    }

    /// `%>` was used on this format: the next line starts matching here.
    pub(crate) fn conthere(&self) -> bool {
        self.conthere
    }

    /// The submatch conversion `idx` captured, or 0 if it is not in this
    /// format. `idx` indexes [`FMT_PAT`].
    pub(crate) fn submatch(&self, idx: usize) -> usize {
        self.addr[idx] as usize
    }

    /// Run this format's pattern over `line`, answering the match.
    ///
    /// Case is always ignored when looking for an error. The engine may
    /// hand back a different `regprog_T` than it was given (a pattern is
    /// recompiled the first time it is used with different settings), so
    /// the answer is stored back.
    ///
    /// # Safety
    ///
    /// `line` must be NUL-terminated.
    pub(crate) unsafe fn exec(&mut self, line: *mut c_char) -> Option<regmatch_T> {
        let mut regmatch = regmatch_T {
            regprog: self.prog,
            rm_ic: true,
            ..Default::default()
        };
        // SAFETY: the caller's line is NUL-terminated; `regprog` is ours.
        let matched = unsafe { vim_regexec(&mut regmatch, line, 0) };
        self.prog = regmatch.regprog;
        matched.then_some(regmatch)
    }
}

/// A compiled `'errorformat'`: every part, in order.
pub(crate) struct Efm {
    formats: Vec<Format>,
    /// Set by `%>`: the format the *next* line should start matching at.
    /// Survives between lines, and — as upstream does — between commands
    /// that reuse the same cached `'errorformat'`.
    resume: Option<usize>,
}

impl Efm {
    /// Compile `'errorformat'`, or answer `None` if any part is bad.
    ///
    /// # Safety
    ///
    /// `efm` must be NUL-terminated.
    pub(crate) unsafe fn compile(efm: *const c_char) -> Option<Efm> {
        // SAFETY: the caller's option value is NUL-terminated.
        let mut rest = unsafe { CStr::from_ptr(efm) };
        let mut formats = Vec::new();

        while !rest.is_empty() {
            // Everything from this part's start through the option's
            // terminating NUL: the walk reads one byte past the part in
            // several places, exactly as upstream does.
            let part = rest.to_bytes_with_nul();
            let len = option_part_len(part);
            let mut fmt = Format::new();
            let pat = fmt.compile_regpat(part, len)?;
            // SAFETY: `compile_regpat` NUL-terminates what it builds.
            fmt.prog = unsafe { vim_regcomp(pat.as_ptr().cast(), RE_MAGIC + RE_STRING) };
            if fmt.prog.is_null() {
                return None;
            }
            formats.push(fmt);
            // SAFETY: `len` indexes within the part, so the remainder is
            // still NUL-terminated.
            let after = unsafe { skip_to_option_part(part.as_ptr().add(len).cast()) };
            // SAFETY: ditto.
            rest = unsafe { CStr::from_ptr(after) };
        }

        if formats.is_empty() {
            emsg(gettext(c"E378: 'errorformat' contains no pattern"));
            return None;
        }

        Some(Efm {
            formats,
            resume: None,
        })
    }

    /// The format to start matching the next line at, consuming a pending
    /// `%>`. Zero — the first format — when there is none.
    pub(crate) fn take_resume(&mut self) -> usize {
        self.resume.take().unwrap_or(0)
    }

    /// Record that `%>` was seen on format `idx`.
    pub(crate) fn set_resume(&mut self, idx: usize) {
        self.resume = Some(idx);
    }

    /// How many parts the option has.
    pub(crate) fn len(&self) -> usize {
        self.formats.len()
    }

    /// One part.
    pub(crate) fn format(&mut self, idx: usize) -> &mut Format {
        &mut self.formats[idx]
    }
}

/// The length of one `'errorformat'` part: up to the next unescaped comma,
/// or the end of the option.
fn option_part_len(efm: &[u8]) -> usize {
    let mut len = 0;
    while efm[len] != 0 && efm[len] != b',' {
        if efm[len] == b'\\' && efm[len + 1] != 0 {
            len += 1;
        }
        len += 1;
    }
    len
}

impl Format {
    /// Translate one `'errorformat'` part into a regexp pattern, recording
    /// in `self` the prefix, flags and submatch numbers it names.
    ///
    /// `part` runs from the start of the part all the way to the option's
    /// terminating NUL, which is included; `len` is the part's own length.
    /// Several of the reads below look one byte past `len` — at the comma,
    /// or at the NUL — which is why the slice is not trimmed.
    fn compile_regpat(&mut self, part: &[u8], len: usize) -> Option<Vec<u8>> {
        let mut pat = Vec::with_capacity(len * 4 + 32);
        pat.push(b'^');
        let mut round = 0;
        let mut i = 0;
        while i < len {
            if part[i] != b'%' {
                if part[i] == b'\\' && i + 1 < len {
                    // An escaped character is copied verbatim.
                    i += 1;
                } else if b".*^$~[".contains(&part[i]) {
                    pat.push(b'\\'); // escape regexp atoms
                }
                if part[i] != 0 {
                    pat.push(part[i]);
                }
                i += 1;
                continue;
            }

            i += 1;
            let conv = part[i];
            if let Some(idx) = FMT_PAT.iter().position(|&(c, _)| c == conv) {
                self.push_conversion(&mut pat, part, i, idx, round)?;
                round += 1;
            } else if conv == b'*' {
                i = push_scanf(&mut pat, part, i + 1, len)?;
            } else if b"%\\.^$~[".contains(&conv) {
                pat.push(conv); // regexp magic characters
            } else if conv == b'#' {
                pat.push(b'*');
            } else if conv == b'>' {
                self.conthere = true;
            } else if i == 1 {
                // A prefix is allowed only at the start of the part.
                i = self.analyze_prefix(part, i)?;
            } else {
                semsg!("E377: Invalid %{} in format string", char::from(conv));
                return None;
            }
            i += 1;
        }
        pat.push(b'$');
        pat.push(0);
        Some(pat)
    }

    /// Append the capture group for the `%` conversion `idx`, whose letter
    /// is at `part[at]`. `round` is how many conversions came before it.
    fn push_conversion(
        &mut self,
        pat: &mut Vec<u8>,
        part: &[u8],
        at: usize,
        idx: usize,
        round: u8,
    ) -> Option<()> {
        if self.addr[idx] != 0 {
            semsg!("E372: Too many %{} in format string", char::from(part[at]));
            return None;
        }
        // Only `%r` may appear under `%O`/`%P`/`%Q`, and it may appear
        // nowhere else; the other conversions are barred from the
        // directory and file prefixes entirely.
        let file_prefix = b"OPQ".contains(&self.prefix);
        if (idx != 0 && idx < FMT_PATTERN_R && (file_prefix || b"DX".contains(&self.prefix)))
            || (idx == FMT_PATTERN_R && !file_prefix)
        {
            semsg!(
                "E373: Unexpected %{} in format string",
                char::from(part[at])
            );
            return None;
        }
        self.addr[idx] = round + 1;
        pat.extend_from_slice(b"\\(");
        if part[at] == b'f' && part[at + 1] != 0 {
            if part[at + 1] != b'\\' && part[at + 1] != b'%' {
                // A file name may contain spaces, which `\f` does not
                // cover, and for "%f:%l:%m" there may be a ":" in the name
                // too. Match up to the next occurrence of whatever follows
                // instead, relying on the ":999:" after it to anchor.
                pat.extend_from_slice(b".\\{-1,}");
            } else {
                // Followed by `\` or `%`: take as many file-name characters
                // as possible.
                pat.extend_from_slice(b"\\f\\+");
            }
        } else {
            pat.extend_from_slice(FMT_PAT[idx].1);
        }
        pat.extend_from_slice(b"\\)");
        Some(())
    }

    /// Record the `%D`/`%E`/… prefix, and the `+`/`-` flag that may precede
    /// it, starting at `part[at]`. Answers the index of the prefix letter.
    fn analyze_prefix(&mut self, part: &[u8], at: usize) -> Option<usize> {
        let mut at = at;
        if b"+-".contains(&part[at]) {
            self.flags = part[at];
            at += 1;
        }
        if b"DXAEWINCZGOPQ".contains(&part[at]) {
            self.prefix = part[at];
            Some(at)
        } else {
            semsg!(
                "E376: Invalid %{} in format string prefix",
                char::from(part[at])
            );
            None
        }
    }
}

/// Append the regexp for a scanf-like `%*` conversion beginning at
/// `part[at]`, answering the index of its last character.
///
/// Only the two conversions upstream accepts are supported: a `[…]` set and
/// a single `\`-escaped class such as `%*\D`. Both are followed by `\+`, so
/// `%*[^:]` means "one or more characters that are not a colon".
fn push_scanf(pat: &mut Vec<u8>, part: &[u8], at: usize, len: usize) -> Option<usize> {
    let mut at = at;
    if part[at] != b'[' && part[at] != b'\\' {
        semsg!(
            "E375: Unsupported %{} in format string",
            char::from(part[at])
        );
        return None;
    }
    pat.push(part[at]);
    if part[at] == b'[' {
        // %*[^a-z0-9] etc.
        if part[at + 1] == b'^' {
            at += 1;
            pat.push(part[at]);
        }
        if at < len {
            at += 1;
            pat.push(part[at]); // could be ']', which then means itself
            while at < len {
                at += 1;
                pat.push(part[at]);
                if part[at] == b']' {
                    break;
                }
            }
            if at == len {
                emsg(gettext(c"E374: Missing ] in format string"));
                return None;
            }
        }
    } else if at < len {
        // %*\D, %*\s etc.
        at += 1;
        pat.push(part[at]);
    }
    pat.extend_from_slice(b"\\+");
    Some(at)
}
