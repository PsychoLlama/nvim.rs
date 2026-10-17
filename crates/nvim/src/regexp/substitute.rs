//! Building the replacement text of a substitution.
//!
//! [`regtilde_owned`] runs first, before the pattern is even compiled: it expands
//! the `~` that stands for the previous replacement and remembers this one
//! for the next. [`regsub_to_vec`] and the [`regsub_multi_len`]/
//! [`regsub_multi_into`] pair then expand the
//! replacement against a finished match — `&` and `\1`..`\9` for the
//! captures, `\u`/`\U`/`\l`/`\L`/`\e`/`\E` for case folding, `\r`/`\n`/
//! `\t`/`\b` for the control characters, and `\=` for a Vimscript
//! expression.
//!
//! **Every caller expands twice.** The first pass only measures the result;
//! the caller allocates that many bytes and calls again to fill them. Both
//! passes walk the same code and advance the same cursor — [`Out`] is the
//! only thing that knows which pass it is — so any disagreement between them
//! would write past the end of the caller's buffer, which [`Out`] refuses
//! (`E_NOT_ENOUGH_SPACE`). Keep them in step: nothing here may take a
//! different route depending on the pass, except where the second pass
//! deliberately reuses what the first computed (the `\=` result).

#![forbid(unsafe_code)]

use crate::cstr;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

use super::api::in_match_scope;
use super::submatch::call_replacement;
use super::{
    CAR, E_SUBSTITUTE_NESTING_TOO_DEEP, NL, REGSUB_BACKSLASH, REGSUB_COPY, REGSUB_MAGIC,
    RegSubMatch, TAB, can_f_submatch, reg_prev_sub, rsm,
};
use crate::eval::eval_to_string;
use crate::global_cell::GlobalCell;
use crate::keycodes::{Ctrl_H, K_SPECIAL};
use crate::mbyte::{
    c_cluster_len, char_at, char_len, encode_char, mb_tolower, mb_toupper, utf_char2len,
};
use crate::memline::held_line;
use crate::memory::XString;
use crate::message::{e_null, e_re_damg, e_resulting_text_too_long};
use crate::message::{emsg, iemsg};
use crate::os::cshim::gettext;
use crate::pos::MAXCOL;
use crate::strings::escaped_bytes;
use crate::types::{ColNr, LineNr, NUL, RegMMatch, RegMatch, TypVal};

/// How deep a `\=` expression may nest substitutions before it is more
/// likely to be a mistake than an intention.
const MAX_REGSUB_NESTING: usize = 4;

/// How far into that nesting we currently are.
static NESTING: GlobalCell<usize> = GlobalCell::new(0);

/// What each nesting level's `\=` expression evaluated to. The measuring
/// pass fills a slot and the copying pass empties it, so the expression —
/// which may have side effects, and need not be deterministic — runs once
/// per substitution rather than once per pass.
///
/// The borrow is per slot rather than around the evaluation, and that is the
/// whole point: what runs between a read and the matching write is arbitrary
/// Vimscript, which may start a substitution of its own — that is what the
/// nesting level counts — so nothing may hold this cell across it.
static EVAL_RESULT: GlobalCell<[Option<Vec<u8>>; MAX_REGSUB_NESTING]> =
    GlobalCell::new([None, None, None, None]);

/// The message a pass that would overrun the caller's buffer reports. That
/// means the two passes disagreed, which is a bug here rather than in the
/// user's pattern.
const E_NOT_ENOUGH_SPACE: &CStr = c"vim_regsub_both(): not enough space";

/// The case hook `\u`, `\U`, `\l` and `\L` install for the rest of a
/// replacement.
type CaseFolder = fn(c_int) -> c_int;

/// The case hooks in force: `\u` and `\l` fold one character, `\U` and `\L`
/// fold every character until `\e` or `\E`.
#[derive(Default)]
struct Case {
    once: Option<CaseFolder>,
    rest: Option<CaseFolder>,
}

impl Case {
    /// Fold `c`, consuming a one-character hook if one is pending.
    fn fold(&mut self, c: c_int) -> c_int {
        match self.once.take().or(self.rest) {
            Some(fold) => fold(c),
            None => c,
        }
    }
}

/// The caller's replacement buffer and the cursor into it.
///
/// `at` advances identically on both passes; only the copying pass has a
/// `dest` to write into, and only it can run out of room -- in which case
/// the passes have disagreed, see the module docs.
struct Out<'d> {
    dest: Option<&'d mut [u8]>,
    at: usize,
}

impl<'d> Out<'d> {
    /// The destination for one substitution pass: `None` for the measuring
    /// pass, which writes nothing.
    fn new(dest: Option<&'d mut [u8]>) -> Self {
        Out { dest, at: 0 }
    }

    /// Whether this is the copying pass.
    fn copy(&self) -> bool {
        self.dest.is_some()
    }

    /// Is there room for `n` more bytes? The measuring pass has no buffer to
    /// overrun and always says yes.
    fn room(&self, n: usize) -> bool {
        match &self.dest {
            Some(dest) if self.at + n > dest.len() => {
                iemsg(E_NOT_ENOUGH_SPACE);
                false
            }
            _ => true,
        }
    }

    /// Step over `n` bytes without writing them.
    fn skip(&mut self, n: usize) {
        self.at += n;
    }

    /// Append one byte. The caller has already asked for the room.
    fn push(&mut self, byte: u8) {
        if let Some(dest) = &mut self.dest {
            dest[self.at] = byte;
        }
        self.skip(1);
    }

    /// Append the character `c`, leaving the cursor on its *last* byte so
    /// that [`Out::push_composing`] can write a tail after it. Both are then
    /// stepped over by the closing `skip(1)`, which is how upstream's
    /// `dst += charlen - 1; …; dst++` reads.
    fn push_char(&mut self, c: c_int) -> bool {
        let charlen = usize::try_from(utf_char2len(c)).unwrap_or(1);
        if !self.room(charlen) {
            return false;
        }
        if let Some(dest) = &mut self.dest {
            encode_char(c, &mut dest[self.at..]);
        }
        self.skip(charlen - 1);
        true
    }

    /// Append `tail`, the composing characters that follow the base
    /// character [`Out::push_char`] just wrote, after it.
    fn push_composing(&mut self, tail: &[u8]) -> bool {
        if !self.room(tail.len()) {
            return false;
        }
        if let Some(dest) = &mut self.dest {
            dest[self.at + 1..self.at + 1 + tail.len()].copy_from_slice(tail);
        }
        self.skip(tail.len());
        true
    }

    /// Write the terminator at the cursor, when there is room for it.
    fn terminate(&mut self) {
        if let Some(byte) = self.dest.as_mut().and_then(|dest| dest.get_mut(self.at)) {
            *byte = NUL as u8;
        }
    }
}

/// How an expansion ended.
enum Outcome {
    /// The whole replacement was expanded.
    Done,
    /// A capture's line changed between the match and the expansion, so the
    /// text ran into a NUL. Upstream jumps past the terminating NUL for
    /// this, leaving the buffer as far as it got.
    Damaged,
    /// The copying pass would have overrun the caller's buffer.
    NoSpace,
}

/// [`regtilde_owned`]'s walk: `source` with every `~` expanded, and whether
/// an expansion took a fresh string (which is when upstream's caller can
/// tell an expansion from an edit in place).
fn regtilde(mut source: Vec<u8>, magic: bool, preview: bool) -> (Vec<u8>, bool) {
    // What a `~` expands into is a copy, so storing the new text below
    // cannot pull it out from under the expansion.
    let prev: Option<XString> = reg_prev_sub.with(Clone::clone);
    let tilde: &[u8] = if magic { b"~" } else { b"\\~" };

    // The fresh string the first expansion makes; until then `source`
    // itself is edited, where a dropped tilde pulls the tail down.
    let mut fresh: Option<Vec<u8>> = None;
    let mut error = false;

    let mut p = 0;
    loop {
        let newsub = fresh.as_mut().unwrap_or(&mut source);
        if p >= newsub.len() {
            break;
        }
        if !newsub[p..].starts_with(tilde) {
            if newsub[p] == b'\\' && p + 1 < newsub.len() {
                // An escaped character cannot be a tilde.
                p += 1;
            }
            p += c_cluster_len(&newsub[p..]).max(1);
            continue;
        }

        let prefixlen = p; // not including the tilde
        let postfix = p + tilde.len();
        let prev_len = prev.as_ref().map_or(0, |prev| prev.len());
        let tmpsublen = newsub.len() - tilde.len() + prev_len;

        let Some(prev) = prev.as_ref().filter(|_| tmpsublen != 0) else {
            // Nothing to expand into: drop the tilde and rescan from where
            // it was.
            newsub.drain(p..postfix);
            continue;
        };
        // Text longer than MAXCOL causes trouble further downstream.
        if tmpsublen > MAXCOL as usize {
            emsg(gettext(e_resulting_text_too_long));
            error = true;
            break;
        }

        let mut tmpsub = Vec::with_capacity(tmpsublen);
        tmpsub.extend_from_slice(&newsub[..prefixlen]);
        tmpsub.extend_from_slice(prev);
        tmpsub.extend_from_slice(&newsub[postfix..]);
        fresh = Some(tmpsub);
        // Rescan from just past what the tilde expanded into.
        p = prefixlen + prev_len;
    }

    if error {
        return (source, false);
    }

    let is_fresh = fresh.is_some();
    let newsub = fresh.unwrap_or(source);
    // A preview must not disturb what the next real substitution's `~`
    // means. Otherwise store a *copy*: a recursive call could replace the
    // text `newsub` came from.
    if !preview {
        reg_prev_sub.set((!newsub.is_empty()).then(|| XString::from_bytes(&newsub)));
    }
    (newsub, is_fresh)
}

/// The bytes of `text`, its terminator left off.
fn payload(text: XString) -> Vec<u8> {
    let mut bytes = text.into_vec();
    bytes.pop();
    bytes
}

/// Expand every `~` in `source` into the previous replacement text, and —
/// unless this is an 'inccommand' `preview` pass — remember the result as
/// what the *next* `~` stands for. With `magic` clear the tilde has to be
/// written `\~`.
///
/// Answers the text, and whether that took a fresh string -- which is when
/// upstream's caller can tell an expansion from an edit in place (a `~`
/// with nothing to stand for is dropped from `source` itself).
pub(crate) fn regtilde_owned(source: XString, magic: bool, preview: bool) -> (XString, bool) {
    let (text, fresh) = regtilde(payload(source), magic, preview);
    (XString::from_vec(text), fresh)
}

/// The size of what `source` -- or `expr`, when the replacement is an
/// expression -- expands to after the string match `matches` over `line`,
/// terminator included, written into `dest` on the copying pass. 0 on an
/// error.
///
/// `line` is the text `matches` ran over, which its capture spans index;
/// neither it nor the match may have changed since
/// [`vim_regexec`](super::vim_regexec) filled them.
fn regsub_string(
    matches: &RegMatch,
    line: &CStr,
    source: Option<&[u8]>,
    expr: Option<&TypVal>,
    dest: Option<&mut [u8]>,
    flags: c_int,
) -> usize {
    // The capture spans are offsets into `line`, which `copy_capture` and
    // `submatch()` resolve them in. A string replacement has no lines to
    // cross, so a `\n` in it is a literal newline rather than a line break.
    let rex = Expanding {
        matched: Matched::Str {
            matches,
            line: line.to_bytes(),
        },
        line_lbr: true,
    };
    in_match_scope(None, |_| regsub_both(rex, source, expr, dest, flags))
}

/// What `source` -- or `expr`, when the replacement is an expression --
/// stands for after the string match `matches` over `line`. Both passes run
/// here. `None` when the measuring pass answered no length, which is how
/// the expansion reports a failure.
pub(crate) fn regsub_to_vec(
    matches: &RegMatch,
    line: &CStr,
    source: Option<&[u8]>,
    expr: Option<&TypVal>,
) -> Option<Vec<u8>> {
    let magic = REGSUB_MAGIC as c_int;
    let len = regsub_string(matches, line, source, expr, None, magic);
    let size = Some(len).filter(|&size| size > 0)?;
    let mut out = vec![0u8; size];
    let copy = REGSUB_COPY as c_int | magic;
    regsub_string(matches, line, source, expr, Some(&mut out[..]), copy);
    out.truncate(size - 1);
    Some(out)
}

/// The expansion of `source` for a buffer match, whose captures can span
/// lines from `lnum` on; see [`regsub_string`].
fn regsub_buffer(
    matches: &RegMMatch,
    lnum: LineNr,
    source: &[u8],
    dest: Option<&mut [u8]>,
    flags: c_int,
) -> usize {
    // A buffer match always works on `curbuf`.
    let buf = Buf::current();
    let rex = Expanding {
        matched: Matched::Buffer {
            matches,
            buf,
            firstlnum: lnum,
            maxline: buf.b_ml.ml_line_count - lnum,
        },
        line_lbr: false,
    };
    in_match_scope(Some(lnum), |_| {
        regsub_both(rex, Some(source), None, dest, flags)
    })
}

/// The match a replacement is expanded against.
#[derive(Clone, Copy)]
enum Matched<'a> {
    /// A string match, and the string it ran over.
    Str {
        matches: &'a RegMatch,
        line: &'a [u8],
    },
    /// A buffer match from line `firstlnum` on, with `maxline` lines below.
    Buffer {
        matches: &'a RegMMatch,
        buf: Buf,
        firstlnum: LineNr,
        maxline: LineNr,
    },
}

/// The match a replacement is expanded against, and how its line breaks
/// read.
#[derive(Clone, Copy)]
struct Expanding<'a> {
    matched: Matched<'a>,
    /// Is a newline in the replacement a literal one?
    line_lbr: bool,
}

impl Expanding<'_> {
    /// Does the match's program fail its magic check? Reports so when it
    /// does. A match with no program has nothing to check.
    fn magic_wrong(self) -> bool {
        let prog = match self.matched {
            Matched::Str { matches, .. } => matches.regprog.as_ref(),
            Matched::Buffer { matches, .. } => matches.regprog.as_ref(),
        };
        prog.is_some_and(super::OwnedProg::magic_wrong)
    }
}

/// Line `lnum` of a buffer match, its terminator not included: `None`
/// above line 1, empty past the end.
///
/// The slice is the memline's cached line, good until another line of the
/// buffer is read.
fn getline(buffer: &Buf, firstlnum: LineNr, maxline: LineNr, lnum: LineNr) -> Option<&[u8]> {
    if firstlnum + lnum < 1 {
        None
    } else if lnum > maxline {
        Some(b"")
    } else {
        let line = held_line(buffer, firstlnum + lnum);
        Some(&line[..line.len() - 1])
    }
}

/// The expansion itself, against the match `rex` describes.
/// Returns the size of the result including its NUL, or 0 on an error.
fn regsub_both(
    rex: Expanding<'_>,
    source: Option<&[u8]>,
    expr: Option<&TypVal>,
    dest: Option<&mut [u8]>,
    flags: c_int,
) -> usize {
    if source.is_none() && expr.is_none() {
        emsg(gettext(e_null));
        return 0;
    }
    if rex.magic_wrong() {
        return 0;
    }
    if NESTING.get() == MAX_REGSUB_NESTING {
        emsg(gettext(E_SUBSTITUTE_NESTING_TOO_DEEP));
        return 0;
    }

    let source = source.unwrap_or_default();
    let mut out = Out::new(dest.filter(|_| flags & REGSUB_COPY as c_int != 0));
    // A caller-supplied function, or a replacement that starts `\=`, is
    // a Vimscript expression rather than replacement text.
    let outcome = if expr.is_some() || source.starts_with(b"\\=") {
        eval_replacement(rex, source, expr, flags, &mut out);
        Outcome::Done
    } else {
        expand_replacement(rex, source, flags, &mut out)
    };

    match outcome {
        Outcome::NoSpace => return 0,
        Outcome::Done => out.terminate(),
        // Deliberately unterminated: upstream jumps past the NUL write.
        Outcome::Damaged => {}
    }
    out.at + 1
}

/// The `\=` replacement: evaluate an expression and take its value as the
/// replacement text.
///
/// Only the measuring pass evaluates. It stashes the text in `EVAL_RESULT`
/// and the copying pass takes it from there, so the expression runs once
/// per substitution however many passes the caller makes.
fn eval_replacement(
    rex: Expanding<'_>,
    source: &[u8],
    expr: Option<&TypVal>,
    flags: c_int,
    out: &mut Out,
) {
    let nested = NESTING.get();

    if let Some(dest) = &mut out.dest {
        let Some(text) = EVAL_RESULT.with(|slots| slots[nested].as_ref().map(Vec::len)) else {
            return;
        };
        // A result that no longer fits means the measuring pass saw a
        // different one; leave it for a later pass rather than overrun.
        if text < dest.len() {
            let text = EVAL_RESULT
                .with_mut(|slots| slots[nested].take())
                .unwrap_or_default();
            dest[..text.len()].copy_from_slice(&text);
            dest[text.len()] = NUL as u8;
            out.skip(text.len());
        }
        return;
    }

    EVAL_RESULT.with_mut(|slots| slots[nested] = None);

    // The expression may itself run a substitution. `submatch()` has to
    // keep answering about the outermost match, so hand it this one and
    // put back whatever the caller above was showing.
    let outer_can_f_submatch = can_f_submatch.get();
    can_f_submatch.set(true);
    let snapshot = match rex.matched {
        Matched::Buffer {
            matches,
            buf,
            firstlnum,
            maxline,
        } => RegSubMatch::of_buffer(buf, firstlnum, maxline, matches),
        Matched::Str { matches, line } => RegSubMatch::of_string(matches, line),
    };
    let outer_rsm = rsm.with_mut(|cell| core::mem::replace(cell, snapshot));

    NESTING.set(nested + 1);
    let text = match expr {
        None => {
            // A copy: the expression may run another `:s` and replace the
            // text this one came from. It stops at a NUL, as the C string
            // it was.
            let expr = XString::from_bytes(cstr::until_nul(&source[2..]));
            eval_to_string(&expr, true, false).map(payload)
        }
        Some(expr) => call_replacement(expr),
    };
    NESTING.set(nested);

    let text = text.map(|mut text| {
        if line_breaks_to_cr(&mut text, rex.line_lbr) && flags & REGSUB_BACKSLASH as c_int != 0 {
            // The backslashes will be consumed downstream; double them
            // so they survive.
            text = payload(escaped_bytes(&text, b"\\"));
        }
        out.skip(text.len());
        text
    });
    EVAL_RESULT.with_mut(|slots| slots[nested] = text);

    can_f_submatch.set(outer_can_f_submatch);
    if outer_can_f_submatch {
        rsm.set(outer_rsm);
    }
}

/// Rewrite the newlines in an expression's result as carriage returns,
/// which is what the substitution machinery downstream reads as a line
/// break — unless the caller came from `vim_regexec_nl`, where a newline is
/// literal. Reports whether the text contained a backslash escape.
fn line_breaks_to_cr(text: &mut [u8], literal_nl: bool) -> bool {
    let mut had_backslash = false;
    let mut s = 0;
    while s < text.len() && text[s] != NUL as u8 {
        if text[s] == NL as u8 && !literal_nl {
            text[s] = CAR as u8;
        } else if text[s] == b'\\' && cstr::byte_at(text, s + 1) != NUL as u8 {
            // Skip the escaped character — but convert it too, so that
            // `:s/abc\\\ndef/\="aaa\\\nbbb"/` breaks the line.
            s += 1;
            if text[s] == NL as u8 && !literal_nl {
                text[s] = CAR as u8;
            }
            had_backslash = true;
        }
        s += c_cluster_len(&text[s..]).max(1);
    }
    had_backslash
}

/// The ordinary replacement: copy `source` into `out`, expanding the
/// capture references and the escapes as they come. The text stops at a
/// NUL.
fn expand_replacement(rex: Expanding<'_>, source: &[u8], flags: c_int, out: &mut Out) -> Outcome {
    let source = cstr::until_nul(source);
    let at = |i: usize| cstr::byte_at(source, i);
    let magic = flags & REGSUB_MAGIC as c_int != 0;
    let backslash = flags & REGSUB_BACKSLASH as c_int != 0;
    let mut case = Case::default();
    let mut src = 0;

    loop {
        let mut c = c_int::from(at(src));
        src += 1;
        if c == NUL {
            return Outcome::Done;
        }

        // Which capture this stands for, or -1 for ordinary text.
        let mut no = -1;
        if c == '&' as c_int && magic {
            no = 0;
        } else if c == '\\' as c_int && at(src) != NUL as u8 {
            if at(src) == b'&' && !magic {
                src += 1;
                no = 0;
            } else if at(src).is_ascii_digit() {
                no = c_int::from(at(src) - b'0');
                src += 1;
            } else if b"uUlLeE".contains(&at(src)) {
                let hook = at(src);
                src += 1;
                match hook {
                    b'u' => case.once = Some(mb_toupper),
                    b'U' => case.rest = Some(mb_toupper),
                    b'l' => case.once = Some(mb_tolower),
                    b'L' => case.rest = Some(mb_tolower),
                    // `\e` and `\E`, the only other members of the set
                    // the lookup above accepted, end both.
                    _ => case = Case::default(),
                }
                continue;
            }
        }

        if no >= 0 {
            match copy_capture(rex, no as usize, &mut case, backslash, out) {
                Outcome::Done => continue,
                stopped => return stopped,
            }
        }

        // A special key travels as its own three bytes.
        if c == K_SPECIAL && at(src) != NUL as u8 && at(src + 1) != NUL as u8 {
            if !out.room(3) {
                return Outcome::NoSpace;
            }
            out.push(K_SPECIAL as u8);
            out.push(at(src));
            out.push(at(src + 1));
            src += 2;
            continue;
        }

        if c == '\\' as c_int && at(src) != NUL as u8 {
            match at(src) {
                b'r' => c = CAR,
                b'n' => c = NL,
                b't' => c = TAB,
                // `\e` already means "end the case fold", so there is no
                // escape for ESC here.
                b'b' => c = Ctrl_H,
                escaped => {
                    // A backslash the caller will strip later: double it
                    // so the literal survives, e.g. an inserted CR.
                    if backslash {
                        if !out.room(1) {
                            return Outcome::NoSpace;
                        }
                        out.push(b'\\');
                    }
                    c = c_int::from(escaped);
                }
            }
            src += 1;
        } else {
            c = char_at(&source[src - 1..]);
        }

        let first = &source[src - 1..];
        let totlen = c_cluster_len(first).max(1);
        if !out.push_char(case.fold(c)) {
            return Outcome::NoSpace;
        }
        // Anything past the base character is composing marks, which are
        // copied as they stand.
        let clen = char_len(first);
        if clen < totlen && !out.push_composing(&first[clen..totlen]) {
            return Outcome::NoSpace;
        }
        src += totlen - 1;
        out.skip(1);
    }
}

/// Copy what capture `no` matched. For a buffer match that can span lines,
/// in which case the line breaks are written as carriage returns.
fn copy_capture(
    rex: Expanding<'_>,
    no: usize,
    case: &mut Case,
    backslash: bool,
    out: &mut Out,
) -> Outcome {
    match rex.matched {
        Matched::Str { matches, line } => {
            // The span is an offset into the string the match ran over.
            let Some(span) = matches.group(no) else {
                // A capture that did not participate contributes nothing.
                return Outcome::Done;
            };
            let at = span.start.min(line.len());
            let len = isize::try_from(span.end - span.start).unwrap_or(isize::MAX);
            copy_run(&line[at..], len, case, backslash, out)
        }
        Matched::Buffer {
            matches,
            buf,
            firstlnum,
            maxline,
        } => {
            let mut clnum = matches.startpos[no].lnum;
            let end = matches.endpos[no];
            if clnum < 0 || end.lnum < 0 {
                return Outcome::Done;
            }
            let mut col = matches.startpos[no].col;
            loop {
                // How much of the capture is on line `clnum`.
                let Some(line) = getline(&buf, firstlnum, maxline, clnum) else {
                    return Outcome::Done;
                };
                let line_len = ColNr::try_from(line.len()).unwrap_or(ColNr::MAX);
                let len = if end.lnum == clnum {
                    end.col - col
                } else {
                    line_len - col
                };
                let text = line
                    .get(usize::try_from(col).unwrap_or(0)..)
                    .unwrap_or_default();
                let len = isize::try_from(len).unwrap_or(0);
                match copy_run(text, len, case, backslash, out) {
                    Outcome::Done => {}
                    stopped => return stopped,
                }
                if end.lnum == clnum {
                    return Outcome::Done;
                }
                // The capture continues on the next line.
                if !out.room(1) {
                    return Outcome::NoSpace;
                }
                out.push(CAR as u8);
                clnum += 1;
                col = 0;
            }
        }
    }
}

/// Copy `len` bytes of a capture from the start of `text`, folding case as
/// the hooks say. Running into the end of `text` first means the line
/// changed since the match ran -- and so does a count that steps past zero
/// inside a character, as upstream's does.
fn copy_run(text: &[u8], len: isize, case: &mut Case, backslash: bool, out: &mut Out) -> Outcome {
    let mut s = 0;
    let mut len = len;
    while len != 0 {
        let Some(&byte) = text.get(s).filter(|&&byte| byte != NUL as u8) else {
            // The line is shorter than it was when the match ran.
            if out.copy() {
                iemsg(gettext(e_re_damg));
            }
            return Outcome::Damaged;
        };

        if backslash && (byte == CAR as u8 || byte == b'\\') {
            // A bare CR would become a line break and a bare backslash
            // would be halved away; double them.
            if !out.room(2) {
                return Outcome::NoSpace;
            }
            out.push(b'\\');
            out.push(byte);
        } else {
            let c = case.fold(char_at(&text[s..]));
            // Composing characters are copied one at a time, so step to
            // the base character's last byte first.
            let tail = char_len(&text[s..]) - 1;
            s += tail;
            len -= isize::try_from(tail).unwrap_or(0);
            if !out.push_char(c) {
                return Outcome::NoSpace;
            }
            out.skip(1);
        }
        s += 1;
        len -= 1;
    }
    Outcome::Done
}

/// The measuring pass over `matches`' last match: the size of what `source`
/// expands to, terminator included, or `None` on an error.
pub(crate) fn regsub_multi_len(
    matches: &RegMMatch,
    lnum: LineNr,
    source: &[u8],
    flags: c_int,
) -> Option<usize> {
    let flags = flags & !(REGSUB_COPY as c_int);
    let size = regsub_buffer(matches, lnum, source, None, flags);
    Some(size).filter(|&size| size > 0)
}

/// The writing pass: expand `source` into `dest`, which bounds the write.
/// `dest` is the room the measuring pass asked for; an expansion that no
/// longer fits stops short of its terminator.
pub(crate) fn regsub_multi_into(
    matches: &RegMMatch,
    lnum: LineNr,
    source: &[u8],
    dest: &mut [u8],
    flags: c_int,
) {
    regsub_buffer(
        matches,
        lnum,
        source,
        Some(dest),
        flags | REGSUB_COPY as c_int,
    );
}
