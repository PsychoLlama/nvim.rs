//! Building the replacement text of a substitution.
//!
//! [`regtilde`] runs first, before the pattern is even compiled: it expands
//! the `~` that stands for the previous replacement and remembers this one
//! for the next. [`vim_regsub`] and [`vim_regsub_multi`] then expand the
//! replacement against a finished match — `&` and `\1`..`\9` for the
//! captures, `\u`/`\U`/`\l`/`\L`/`\e`/`\E` for case folding, `\r`/`\n`/
//! `\t`/`\b` for the control characters, and `\=` for a Vimscript
//! expression.
//!
//! **Every caller expands twice.** The first pass has [`REGSUB_COPY`] clear
//! and only measures the result; the caller allocates that many bytes and
//! calls again with the flag set to fill them. Both passes walk the same
//! code and advance the same cursor — [`Out`] is the only thing that knows
//! which pass it is — so any disagreement between them writes past the end
//! of the caller's buffer. Keep them in step: nothing here may take a
//! different route depending on `copy`, except where the second pass
//! deliberately reuses what the first computed (the `\=` result).

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::strings::has_char;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int};

use super::api::in_match_scope;
use super::submatch::call_replacement;
use super::{
    CAR, E_SUBSTITUTE_NESTING_TOO_DEEP, NL, REGSUB_BACKSLASH, REGSUB_COPY, REGSUB_MAGIC,
    RegSubMatch, TAB, can_f_submatch, prog_magic_wrong, reg_prev_sub, rsm,
};
use crate::eval::eval_to_string;
use crate::global_cell::GlobalCell;
use crate::keycodes::{Ctrl_H, K_SPECIAL};
use crate::mbyte::{
    mb_tolower, mb_toupper, utf_char2bytes, utf_char2len, utf_ptr2char, utf_ptr2len, utfc_ptr2len,
};
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::memory::{XString, xfree, xmalloc};
use crate::message::{e_null, e_re_damg, e_resulting_text_too_long};
use crate::message::{emsg, iemsg};
use crate::os::cshim::gettext;
use crate::pos::MAXCOL;
use crate::strings::escaped_bytes;
use crate::types::{ColNr, LineNr, NUL, RegMMatch, RegMatch, RegProg, TypVal};
use crate::winlayer::Live;
use ::libc::strcpy;

/// How deep a `\=` expression may nest substitutions before it is more
/// likely to be a mistake than an intention.
const MAX_REGSUB_NESTING: c_int = 4;

/// How far into that nesting we currently are.
static NESTING: GlobalCell<c_int> = GlobalCell::new(0);

/// What each nesting level's `\=` expression evaluated to. The measuring
/// pass fills a slot and the copying pass empties it, so the expression —
/// which may have side effects, and need not be deterministic — runs once
/// per substitution rather than once per pass.
static EVAL_RESULT: GlobalCell<[*mut c_char; MAX_REGSUB_NESTING as usize]> =
    GlobalCell::new([core::ptr::null_mut(); MAX_REGSUB_NESTING as usize]);

/// What level `nested` has stashed, without taking it.
///
/// The borrow is per slot rather than around the evaluation, and that is the
/// whole point: what runs between a read and the matching write is arbitrary
/// Vimscript, which may start a substitution of its own — that is what the
/// nesting level counts — so nothing may hold this cell across it.
fn stashed(nested: usize) -> *mut c_char {
    EVAL_RESULT.with(|slots| slots[nested])
}

/// Put `text` in level `nested`'s slot. The caller owns whatever was there.
fn stash(nested: usize, text: *mut c_char) {
    EVAL_RESULT.with_mut(|slots| slots[nested] = text);
}

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
/// `at` advances identically on both passes; `copy` decides whether
/// anything is written on the way. Only the copying pass can run out of
/// room, and if it does the passes have disagreed — see the module docs.
struct Out {
    dest: *mut c_char,
    at: *mut c_char,
    destlen: c_int,
    copy: bool,
}

impl Out {
    /// The destination for one substitution pass.
    ///
    /// # Safety
    ///
    /// When `copy` is set, `dest` must point at `destlen` writable bytes the
    /// caller owns and keeps alive for the pass: `push`, `push_char` and
    /// `push_composing` are safe `fn`s that write through the cursor and
    /// bound themselves on `dest + destlen` and nothing else. The measuring
    /// pass (`copy` unset) writes nothing, so `dest` may be null there.
    unsafe fn new(dest: *mut c_char, destlen: c_int, copy: bool) -> Self {
        Out {
            dest,
            at: dest,
            destlen,
            copy,
        }
    }

    /// Is there room for `n` more bytes? The measuring pass has no buffer to
    /// overrun and always says yes.
    fn room(&self, n: isize) -> bool {
        if self.copy
            && self.at.wrapping_offset(n) > self.dest.wrapping_offset(self.destlen as isize)
        {
            iemsg(E_NOT_ENOUGH_SPACE);
            return false;
        }
        true
    }

    /// Step over `n` bytes without writing them.
    fn skip(&mut self, n: isize) {
        self.at = self.at.wrapping_offset(n);
    }

    /// Append one byte. The caller has already asked for the room.
    fn push(&mut self, byte: c_char) {
        if self.copy {
            // SAFETY: `room` cleared this byte, so `at` is inside `dest`.
            unsafe { *self.at = byte };
        }
        self.skip(1);
    }

    /// Append the character `c`, leaving the cursor on its *last* byte so
    /// that [`Out::push_composing`] can write a tail after it. Both are then
    /// stepped over by the closing `skip(1)`, which is how upstream's
    /// `dst += charlen - 1; …; dst++` reads.
    fn push_char(&mut self, c: c_int) -> bool {
        // SAFETY: `room` clears the write below, and `utf_char2len` /
        // `utf_char2bytes` are pure over a code point and a buffer with that
        // many bytes free.
        let charlen = utf_char2len(c);
        if !self.room(charlen as isize) {
            return false;
        }
        if self.copy {
            unsafe { utf_char2bytes(c, self.at) };
        }
        self.skip((charlen - 1) as isize);
        true
    }

    /// Append the composing characters that follow the base character at
    /// `first` — bytes `clen..totlen` of it — after the character
    /// [`Out::push_char`] just wrote.
    fn push_composing(&mut self, first: *const c_char, clen: c_int, totlen: c_int) -> bool {
        let tail = (totlen - clen) as isize;
        if !self.room(tail) {
            return false;
        }
        if self.copy {
            // SAFETY: `room` cleared `tail` bytes at `at + 1`, and `first`
            // has `totlen` valid bytes because `utfc_ptr2len` said so.
            let dest = unsafe { self.at.offset(1) };
            let src = unsafe { first.offset(clen as isize) };
            unsafe { dest.cast::<u8>().copy_from(src.cast(), tail as usize) };
        }
        self.skip(tail);
        true
    }

    /// How many bytes the expansion has accounted for so far.
    fn written(&self) -> isize {
        self.at as isize - self.dest as isize
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

/// Expand every `~` in `source` into the previous replacement text, and —
/// unless this is an 'inccommand' `preview` pass — remember the result as
/// what the *next* `~` stands for.
///
/// With `magic` clear the tilde has to be written `\~`. Returns `source`
/// itself when nothing was expanded and a freshly allocated string
/// otherwise; callers tell the two apart by comparing pointers.
///
/// # Safety
///
/// `source` must point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn regtilde(source: *mut c_char, magic: c_int, preview: bool) -> *mut c_char {
    // SAFETY: `source` is the caller's NUL-terminated replacement text.
    // What a `~` expands into is a copy, so storing the new text below cannot
    // pull it out from under the expansion.
    let prev: Option<XString> = reg_prev_sub.with(Clone::clone);
    let tilde = if magic != 0 { c"~" } else { c"\\~" };
    let tildelen = tilde.count_bytes();

    let mut newsub = source;
    // Zero means "not measured yet"; a measured length is at least the
    // tilde's own.
    let mut newsublen: usize = 0;
    let mut error = false;

    let mut p = newsub;
    while unsafe { *p } != 0 {
        if !unsafe { cstr::prefix_eq(p, tilde.as_ptr(), tildelen) } {
            if unsafe { *p } == b'\\' as c_char && unsafe { *p.offset(1) } != 0 {
                // An escaped character cannot be a tilde.
                p = unsafe { p.offset(1) };
            }
            p = unsafe { p.offset(utfc_ptr2len(p) as isize) };
            continue;
        }

        let prefixlen = unsafe { p.offset_from(newsub) } as usize; // not including the tilde
        let postfix = unsafe { p.add(tildelen) };
        if newsublen == 0 {
            newsublen = unsafe { cstr::bytes_at(newsub) }.len();
        }
        newsublen -= tildelen;
        let postfixlen = newsublen - prefixlen;
        let prev_len = prev.as_ref().map_or(0, |prev| prev.len());
        let tmpsublen = prefixlen + prev_len + postfixlen;

        let Some(prev) = prev.as_ref().filter(|_| tmpsublen != 0) else {
            // Nothing to expand into: drop the tilde, NUL included, and
            // rescan from where it was.
            unsafe { p.cast::<u8>().copy_from(postfix.cast(), postfixlen + 1) };
            continue;
        };
        // Text longer than MAXCOL causes trouble further downstream.
        if tmpsublen > MAXCOL as usize {
            emsg(gettext(e_resulting_text_too_long));
            error = true;
            break;
        }

        let tmpsub: *mut c_char = unsafe { xmalloc(tmpsublen + 1) }.cast();
        unsafe { tmpsub.cast::<u8>().copy_from(newsub.cast(), prefixlen) };
        let dest = unsafe { tmpsub.add(prefixlen) };
        let into = dest.cast::<u8>();
        unsafe { into.copy_from(prev.as_ptr().cast(), prev_len) };
        let expanded = prefixlen + prev_len;
        unsafe { strcpy(tmpsub.add(expanded), postfix) };

        if newsub != source {
            unsafe { xfree(newsub.cast()) };
        }
        newsub = tmpsub;
        newsublen = tmpsublen;
        // Rescan from just past what the tilde expanded into.
        p = unsafe { newsub.add(expanded) };
    }

    if error {
        if newsub != source {
            unsafe { xfree(newsub.cast()) };
        }
        return source;
    }

    // A preview must not disturb what the next real substitution's `~`
    // means. Otherwise store a *copy*: a recursive call could free the
    // text `newsub` points into.
    if !preview {
        newsublen = unsafe { p.offset_from(newsub) } as usize;
        // SAFETY: `newsub` holds `newsublen` bytes of replacement text.
        reg_prev_sub.set((newsublen != 0).then(|| {
            XString::from_bytes(unsafe { core::slice::from_raw_parts(newsub.cast(), newsublen) })
        }));
    }
    newsub
}

/// Expand `source` into `dest` using the captures of the string match
/// `matches`, or the result of `expr` when the replacement is an
/// expression. Returns the length written, plus one for the NUL.
///
/// `line` is the text `matches` ran over, which its capture spans index;
/// neither it nor the match may have changed since
/// [`vim_regexec`](super::vim_regexec) filled them.
///
/// # Safety
///
/// `source` must point at a NUL-terminated string, unaliased for the call.
/// `expr` must point at an initialized typval, unaliased for the call.
/// `dest` must point at `destlen` writable bytes, unaliased for the call,
/// whenever `flags` sets `REGSUB_COPY`; the measuring pass writes nothing
/// and takes a null.
pub(crate) unsafe fn vim_regsub(
    matches: &mut RegMatch,
    line: &CStr,
    source: *mut c_char,
    expr: Option<&TypVal>,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
) -> c_int {
    // The capture spans are offsets into `line`, which `copy_capture` and
    // `submatch()` resolve them in. A string replacement has no lines to
    // cross, so a `\n` in it is a literal newline rather than a line break.
    let rex = Expanding {
        match_: matches,
        mmatch: core::ptr::null_mut(),
        line: line.to_bytes_with_nul(),
        buf: Buf::current(),
        firstlnum: 0,
        maxline: 0,
        line_lbr: true,
    };
    // SAFETY: the arguments are the caller's.
    in_match_scope(None, |_| unsafe {
        vim_regsub_both(rex, source, expr, dest, destlen, flags)
    })
}

/// [`vim_regsub`] into an owned buffer: what `source` -- or `expr`, when
/// the replacement is an expression -- stands for after the string match
/// `matches` over `line`. Both passes run here. `None` when the measuring
/// pass answered no length, which is how the expansion reports a failure.
///
/// `source` is lent exclusively because a `\=` replacement is evaluated
/// in place, and the evaluator may cut it while it runs.
pub(crate) fn regsub_to_vec(
    matches: &mut RegMatch,
    line: &CStr,
    source: Option<&mut XString>,
    expr: Option<&TypVal>,
) -> Option<Vec<u8>> {
    let source = source.map_or(core::ptr::null_mut(), |text| text.as_mut_ptr());
    let magic = REGSUB_MAGIC as c_int;
    // The measuring pass writes nothing, but wants somewhere to not write.
    let mut nowhere: c_char = 0;
    // SAFETY: `source` is null or an owned NUL-terminated buffer the caller
    // lent for the call; `expr` is the caller's; the measuring pass writes
    // nothing through `nowhere`.
    let len = unsafe { vim_regsub(matches, line, source, expr, &raw mut nowhere, 0, magic) };
    let size = usize::try_from(len).ok().filter(|&size| size > 0)?;
    let mut out = vec![0u8; size];
    let copy = REGSUB_COPY as c_int | magic;
    // SAFETY: as above, and `out` holds the `len` bytes the measuring pass
    // asked for, terminator included.
    unsafe {
        vim_regsub(
            matches,
            line,
            source,
            expr,
            out.as_mut_ptr().cast(),
            len,
            copy,
        )
    };
    out.truncate(size - 1);
    Some(out)
}

/// [`vim_regsub`] for a buffer match, whose captures can span lines from
/// `lnum` on.
///
/// # Safety
///
/// `rmp` must point at a live `RegMMatch`, unaliased for the call. `source`
/// must point at a NUL-terminated string, unaliased for the call. `dest` must point
/// at `destlen` writable bytes, unaliased for the call, whenever `flags` sets
/// `REGSUB_COPY`; the measuring pass writes nothing and takes a null.
pub(crate) unsafe fn vim_regsub_multi(
    rmp: *mut RegMMatch,
    lnum: LineNr,
    source: *mut c_char,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
) -> c_int {
    // A buffer match always works on `curbuf`.
    let buf = Buf::current();
    let rex = Expanding {
        match_: core::ptr::null_mut(),
        mmatch: rmp,
        line: &[],
        buf,
        firstlnum: lnum,
        maxline: buf.b_ml.ml_line_count - lnum,
        line_lbr: false,
    };
    // SAFETY: as `vim_regsub`.
    in_match_scope(Some(lnum), |_| unsafe {
        vim_regsub_both(rex, source, None, dest, destlen, flags)
    })
}

/// The match a replacement is expanded against.
#[derive(Clone, Copy)]
struct Expanding<'a> {
    /// One of the two is null; `line` (with its NUL) is a string match's.
    match_: *mut RegMatch,
    mmatch: *mut RegMMatch,
    line: &'a [u8],
    buf: Buf,
    firstlnum: LineNr,
    maxline: LineNr,
    /// Is a newline in the replacement a literal one?
    line_lbr: bool,
}

impl Expanding<'_> {
    fn regprog(self) -> *mut RegProg {
        // SAFETY: the caller's live match structure, whichever this is.
        if self.match_.is_null() {
            unsafe { (*self.mmatch).regprog }
        } else {
            unsafe { (*self.match_).regprog }
        }
    }

    /// Line `lnum` of a buffer match: null above line 1, empty past the end.
    fn getline(self, lnum: LineNr) -> *mut c_char {
        if self.firstlnum + lnum < 1 {
            core::ptr::null_mut()
        } else if lnum > self.maxline {
            c"".as_ptr().cast_mut()
        } else {
            ml_get_buf(self.buf, self.firstlnum + lnum)
        }
    }

    /// Its length; 0 for either stand-in.
    fn getline_len(self, lnum: LineNr) -> ColNr {
        let inside = self.firstlnum + lnum >= 1 && lnum <= self.maxline;
        if inside {
            ml_get_buf_len(self.buf, self.firstlnum + lnum)
        } else {
            0
        }
    }
}

/// The expansion itself, against the match `rex` describes.
/// Returns the size of the result including its NUL, or 0 on an error.
///
/// # Safety
///
/// `source` must point at a NUL-terminated string, unaliased for the call.
/// `expr` must point at an initialized typval, unaliased for the call. `dest`
/// must point at `destlen` writable bytes, unaliased for the call, whenever
/// `flags` sets `REGSUB_COPY`.
unsafe fn vim_regsub_both(
    rex: Expanding<'_>,
    source: *mut c_char,
    expr: Option<&TypVal>,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
) -> c_int {
    if (source.is_null() && expr.is_none()) || dest.is_null() {
        emsg(gettext(e_null));
        return 0;
    }
    if prog_magic_wrong(rex.regprog()) != 0 {
        return 0;
    }
    if NESTING.get() == MAX_REGSUB_NESTING {
        emsg(gettext(E_SUBSTITUTE_NESTING_TOO_DEEP));
        return 0;
    }

    // SAFETY: the caller's promise -- on a copying pass `dest` holds
    // `destlen` writable bytes.
    let mut out = unsafe { Out::new(dest, destlen, flags & REGSUB_COPY as c_int != 0) };
    // A caller-supplied function, or a replacement that starts `\=`, is
    // a Vimscript expression rather than replacement text.
    let outcome = if expr.is_some()
        || (unsafe { *source } == b'\\' as c_char && unsafe { *source.offset(1) } == b'=' as c_char)
    {
        unsafe { eval_replacement(rex, source, expr, flags, &mut out) };
        Outcome::Done
    } else {
        unsafe { expand_replacement(rex, source, flags, &mut out) }
    };

    match outcome {
        Outcome::NoSpace => return 0,
        Outcome::Done => {
            if out.copy {
                unsafe { *out.at = NUL as c_char };
            }
        }
        // Deliberately unterminated: upstream jumps past the NUL write.
        Outcome::Damaged => {}
    }
    (out.written() + 1) as c_int
}

/// The `\=` replacement: evaluate an expression and take its value as the
/// replacement text.
///
/// Only the measuring pass evaluates. It stashes the text in `EVAL_RESULT`
/// and the copying pass takes it from there, so the expression runs once
/// per substitution however many passes the caller makes.
///
/// # Safety
///
/// `source` must point at a NUL-terminated string, unaliased for the call.
/// `expr` must point at an initialized typval, unaliased for the call.
unsafe fn eval_replacement(
    rex: Expanding<'_>,
    source: *mut c_char,
    expr: Option<&TypVal>,
    flags: c_int,
    out: &mut Out,
) {
    // SAFETY: `source` is the caller's replacement text and `expr` its
    // callable, both live for the call. Nothing raw is held across the
    // evaluation below, which can run arbitrary Vimscript.
    let nested = NESTING.get() as usize;

    if out.copy {
        let text = stashed(nested);
        if text.is_null() {
            return;
        }
        let len = unsafe { cstr::bytes_at(text) }.len();
        // A result that no longer fits means the measuring pass saw a
        // different one; leave it for a later pass rather than overrun.
        if len < out.destlen as usize {
            unsafe { strcpy(out.dest, text) };
            out.skip(len as isize);
            unsafe { xfree(text.cast()) };
            stash(nested, core::ptr::null_mut());
        }
        return;
    }

    unsafe { xfree(stashed(nested).cast()) };
    stash(nested, core::ptr::null_mut());

    // The expression may itself run a substitution. `submatch()` has to
    // keep answering about the outermost match, so hand it this one and
    // put back whatever the caller above was showing.
    let outer_can_f_submatch = can_f_submatch.get();
    can_f_submatch.set(true);
    // The snapshot is taken whole: the expression may run a search that
    // reuses whatever the caller's match structure points at.
    // SAFETY: whichever match structure this is, the caller's and live.
    let snapshot = if rex.match_.is_null() {
        RegSubMatch::of_buffer(rex.buf, rex.firstlnum, rex.maxline, unsafe { &*rex.mmatch })
    } else {
        RegSubMatch::of_string(unsafe { &*rex.match_ }, rex.line)
    };
    let outer_rsm = rsm.with_mut(|cell| core::mem::replace(cell, snapshot));

    NESTING.set(nested as c_int + 1);
    let mut text = match expr {
        None => {
            // A copy: the expression may run another `:s` and replace the
            // text this one came from.
            let expr = XString::from_cstr(unsafe { CStr::from_ptr(source.offset(2)) });
            eval_to_string(&expr, true, false).map_or(core::ptr::null_mut(), XString::into_raw)
        }
        Some(expr) => call_replacement(expr),
    };
    NESTING.set(nested as c_int);

    if !text.is_null() {
        if unsafe { line_breaks_to_cr(text, rex.line_lbr) }
            && flags & REGSUB_BACKSLASH as c_int != 0
        {
            // The backslashes will be consumed downstream; double them
            // so they survive.
            let doubled = unsafe { escaped_bytes(cstr::bytes_at(text), b"\\") }.into_raw();
            unsafe { xfree(text.cast()) };
            text = doubled;
        }
        out.skip(unsafe { cstr::bytes_at(text) }.len() as isize);
    }
    stash(nested, text);

    can_f_submatch.set(outer_can_f_submatch);
    if outer_can_f_submatch {
        rsm.set(outer_rsm);
    }
}

/// Rewrite the newlines in an expression's result as carriage returns,
/// which is what the substitution machinery downstream reads as a line
/// break — unless the caller came from `vim_regexec_nl`, where a newline is
/// literal. Reports whether the text contained a backslash escape.
///
/// # Safety
///
/// `text` must point at a NUL-terminated string, unaliased for the call.
unsafe fn line_breaks_to_cr(text: *mut c_char, literal_nl: bool) -> bool {
    // SAFETY: `text` is a NUL-terminated allocation this module owns.
    let mut had_backslash = false;
    let mut s = text;
    while unsafe { *s } != NUL as c_char {
        if unsafe { *s } == NL as c_char && !literal_nl {
            unsafe { *s = CAR as c_char };
        } else if unsafe { *s } == b'\\' as c_char && unsafe { *s.offset(1) } != NUL as c_char {
            // Skip the escaped character — but convert it too, so that
            // `:s/abc\\\ndef/\="aaa\\\nbbb"/` breaks the line.
            s = unsafe { s.offset(1) };
            if unsafe { *s } == NL as c_char && !literal_nl {
                unsafe { *s = CAR as c_char };
            }
            had_backslash = true;
        }
        s = unsafe { s.offset(utfc_ptr2len(s) as isize) };
    }
    had_backslash
}

/// The ordinary replacement: copy `source` into `out`, expanding the
/// capture references and the escapes as they come.
///
/// # Safety
///
/// `source` must point at a NUL-terminated string, unaliased for the call.
unsafe fn expand_replacement(
    rex: Expanding<'_>,
    source: *mut c_char,
    flags: c_int,
    out: &mut Out,
) -> Outcome {
    // SAFETY: `source` is NUL-terminated, and `rex` describes a live match
    // whose captures point into text that has not moved.
    let magic = flags & REGSUB_MAGIC as c_int != 0;
    let backslash = flags & REGSUB_BACKSLASH as c_int != 0;
    let mut case = Case::default();
    let mut src = source;

    loop {
        let mut c = unsafe { *src } as u8 as c_int;
        src = unsafe { src.offset(1) };
        if c == NUL {
            return Outcome::Done;
        }

        // Which capture this stands for, or -1 for ordinary text.
        let mut no = -1;
        if c == '&' as c_int && magic {
            no = 0;
        } else if c == '\\' as c_int && unsafe { *src } != NUL as c_char {
            if unsafe { *src } == b'&' as c_char && !magic {
                src = unsafe { src.offset(1) };
                no = 0;
            } else if (unsafe { *src } as u8).is_ascii_digit() {
                no = unsafe { *src } as c_int - '0' as c_int;
                src = unsafe { src.offset(1) };
            } else if has_char(c"uUlLeE", unsafe { *src } as u8 as c_int) {
                let hook = unsafe { *src } as u8;
                src = unsafe { src.offset(1) };
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
            match copy_capture(rex, no, &mut case, backslash, out) {
                Outcome::Done => continue,
                stopped => return stopped,
            }
        }

        // A special key travels as its own three bytes.
        if c == K_SPECIAL
            && unsafe { *src } != NUL as c_char
            && unsafe { *src.offset(1) } != NUL as c_char
        {
            if !out.room(3) {
                return Outcome::NoSpace;
            }
            out.push(c as c_char);
            out.push(unsafe { *src });
            out.push(unsafe { *src.offset(1) });
            src = unsafe { src.offset(2) };
            continue;
        }

        if c == '\\' as c_int && unsafe { *src } != NUL as c_char {
            match unsafe { *src } as u8 {
                b'r' => {
                    c = CAR;
                    src = unsafe { src.offset(1) };
                }
                b'n' => {
                    c = NL;
                    src = unsafe { src.offset(1) };
                }
                b't' => {
                    c = TAB;
                    src = unsafe { src.offset(1) };
                }
                // `\e` already means "end the case fold", so there is no
                // escape for ESC here.
                b'b' => {
                    c = Ctrl_H;
                    src = unsafe { src.offset(1) };
                }
                _ => {
                    // A backslash the caller will strip later: double it
                    // so the literal survives, e.g. an inserted CR.
                    if backslash {
                        if !out.room(1) {
                            return Outcome::NoSpace;
                        }
                        out.push(b'\\' as c_char);
                    }
                    c = unsafe { *src } as u8 as c_int;
                    src = unsafe { src.offset(1) };
                }
            }
        } else {
            c = unsafe { utf_ptr2char(src.offset(-1)) };
        }

        let first = unsafe { src.offset(-1) };
        let totlen = unsafe { utfc_ptr2len(first) };
        if !out.push_char(case.fold(c)) {
            return Outcome::NoSpace;
        }
        // Anything past the base character is composing marks, which are
        // copied as they stand.
        let clen = unsafe { utf_ptr2len(first) };
        if clen < totlen && !out.push_composing(first, clen, totlen) {
            return Outcome::NoSpace;
        }
        src = unsafe { src.offset((totlen - 1) as isize) };
        out.skip(1);
    }
}

/// Copy what capture `no` matched. For a buffer match that can span lines,
/// in which case the line breaks are written as carriage returns.
fn copy_capture(
    rex: Expanding<'_>,
    no: c_int,
    case: &mut Case,
    backslash: bool,
    out: &mut Out,
) -> Outcome {
    // SAFETY: `rex` describes a live match; `no` is a single digit and both
    // capture arrays hold `NSUBEXP` = 10 slots.
    let multi = rex.match_.is_null();
    let no = no as usize;

    // Where the capture starts, how much of it is on that line, and —
    // for a buffer match — which line that is.
    let mut clnum: LineNr = 0;
    let mut len: c_int = 0;
    let mut s = if multi {
        // SAFETY: a buffer match's `reg_mmatch` is the caller's live structure.
        let mmatch = unsafe { Live::new(rex.mmatch) };
        clnum = mmatch.startpos[no].lnum;
        if clnum < 0 || mmatch.endpos[no].lnum < 0 {
            core::ptr::null_mut()
        } else {
            len = if mmatch.endpos[no].lnum == clnum {
                mmatch.endpos[no].col - mmatch.startpos[no].col
            } else {
                rex.getline_len(clnum) - mmatch.startpos[no].col
            };
            unsafe { rex.getline(clnum).offset(mmatch.startpos[no].col as isize) }
        }
    } else {
        // SAFETY: a string match's `reg_match` is the caller's live structure.
        let match_ = unsafe { Live::new(rex.match_) };
        match match_.group(no) {
            None => core::ptr::null_mut(),
            Some(span) => {
                len = (span.end - span.start) as c_int;
                // The span is an offset into the string the match ran over.
                let at = span.start.min(rex.line.len() - 1);
                rex.line[at..].as_ptr().cast::<c_char>().cast_mut()
            }
        }
    };
    // A capture that did not participate contributes nothing.
    if s.is_null() {
        return Outcome::Done;
    }

    loop {
        if len == 0 {
            // SAFETY: as above.
            let mmatch = unsafe { Live::new(rex.mmatch) };
            if !multi || mmatch.endpos[no].lnum == clnum {
                return Outcome::Done;
            }
            // The capture continues on the next line.
            if !out.room(1) {
                return Outcome::NoSpace;
            }
            out.push(CAR as c_char);
            clnum += 1;
            s = rex.getline(clnum);
            len = if mmatch.endpos[no].lnum == clnum {
                mmatch.endpos[no].col
            } else {
                rex.getline_len(clnum)
            };
            continue;
        }
        if unsafe { *s } == NUL as c_char {
            // The line is shorter than it was when the match ran.
            if out.copy {
                iemsg(gettext(e_re_damg));
            }
            return Outcome::Damaged;
        }

        if backslash && (unsafe { *s } == CAR as c_char || unsafe { *s } == b'\\' as c_char) {
            // A bare CR would become a line break and a bare backslash
            // would be halved away; double them.
            if !out.room(2) {
                return Outcome::NoSpace;
            }
            out.push(b'\\' as c_char);
            out.push(unsafe { *s });
        } else {
            let c = case.fold(unsafe { utf_ptr2char(s) });
            // Composing characters are copied one at a time, so step to
            // the base character's last byte first.
            let tail = unsafe { utf_ptr2len(s) } - 1;
            s = unsafe { s.offset(tail as isize) };
            len -= tail;
            if !out.push_char(c) {
                return Outcome::NoSpace;
            }
            out.skip(1);
        }
        s = unsafe { s.offset(1) };
        len -= 1;
    }
}

/// [`regtilde`] over a replacement the caller owns: `source` with every
/// `~` expanded into the previous replacement (and remembered as the next
/// one, unless `preview`), and whether that took a fresh string -- which
/// is when upstream's caller can tell an expansion from an edit in place
/// (a `~` with nothing to stand for is dropped from `source` itself).
pub(crate) fn regtilde_owned(mut source: XString, magic: bool, preview: bool) -> (XString, bool) {
    let raw = source.as_mut_ptr();
    // SAFETY: an owned NUL-terminated buffer, lent for the call. The answer
    // is `raw` itself -- edited in place, where a dropped tilde pulls the
    // tail and its terminator down -- or a fresh `xmalloc` string that
    // nothing else holds.
    let expanded = unsafe { regtilde(raw, c_int::from(magic), preview) };
    if expanded == raw {
        // SAFETY: still the caller's terminated buffer, now maybe shorter.
        let len = unsafe { cstr::bytes_at(raw) }.len();
        source.truncate(len);
        (source, false)
    } else {
        // SAFETY: as above -- a block of the `xmalloc` family, ours alone.
        (unsafe { XString::from_raw(expanded) }, true)
    }
}

/// [`vim_regsub_multi`]'s measuring pass over `matches`' last match: the
/// size of what `source` expands to, terminator included, or `None` on an
/// error.
///
/// `source` is lent exclusively because a `\=` replacement is evaluated
/// in place, and the evaluator may cut it while it runs.
pub(crate) fn regsub_multi_len(
    matches: &mut super::OwnedMMatch,
    lnum: LineNr,
    source: &mut XString,
    flags: c_int,
) -> Option<usize> {
    // The measuring pass writes nothing, but wants somewhere to not write.
    let mut nowhere: c_char = 0;
    // SAFETY: the match record is the holder's, with its live program;
    // `source` is the caller's terminated buffer; without `REGSUB_COPY`
    // nothing is written through `nowhere`.
    let size = unsafe {
        vim_regsub_multi(
            matches.record(),
            lnum,
            source.as_mut_ptr(),
            &raw mut nowhere,
            0,
            flags & !(REGSUB_COPY as c_int),
        )
    };
    usize::try_from(size).ok().filter(|&size| size > 0)
}

/// [`vim_regsub_multi`]'s writing pass: expand `source` into `dest`, which
/// bounds the write. `dest` is the room the measuring pass asked for; an
/// expansion that no longer fits stops short of its terminator.
pub(crate) fn regsub_multi_into(
    matches: &mut super::OwnedMMatch,
    lnum: LineNr,
    source: &mut XString,
    dest: &mut [u8],
    flags: c_int,
) {
    let destlen = c_int::try_from(dest.len()).unwrap_or(c_int::MAX);
    // SAFETY: as the measuring pass; `dest` is `destlen` writable bytes,
    // and the expansion bounds every write on that length.
    unsafe {
        vim_regsub_multi(
            matches.record(),
            lnum,
            source.as_mut_ptr(),
            dest.as_mut_ptr().cast(),
            destlen,
            flags | REGSUB_COPY as c_int,
        )
    };
}
