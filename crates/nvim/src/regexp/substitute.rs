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

use crate::cstr;
use core::ffi::{c_char, c_int};

use super::api::with_rex;
use super::submatch::{Rsm, clear_submatch_list, fill_submatch_list};
use super::{
    CAR, E_SUBSTITUTE_NESTING_TOO_DEEP, NL, REGSUB_BACKSLASH, REGSUB_COPY, REGSUB_MAGIC, Rex, TAB,
    can_f_submatch, prog_magic_wrong, reg_getline, reg_getline_len, reg_prev_sub, reg_prev_sublen,
    regsubmatch_T, rsm,
};
use crate::eval::typval::{tv_clear, tv_get_string_buf_chk, tv_list_len};
use crate::eval::userfunc::call_func;
use crate::eval::{eval_to_string, partial_name};
use crate::global_cell::GlobalCell;
use crate::keycodes::{Ctrl_H, K_SPECIAL};
use crate::main::{curbuf, e_null, e_re_damg, e_resulting_text_too_long};
use crate::mbyte::{
    mb_tolower, mb_toupper, utf_char2bytes, utf_char2len, utf_ptr2char, utf_ptr2len, utfc_ptr2len,
};
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::message::{emsg, iemsg};
use crate::os::cshim::gettext;
use crate::pos::MAXCOL;
use crate::strings::{vim_strchr, vim_strsave_escaped, xstrnsave};
use crate::types::{
    NUL, VAR_FUNC, VAR_LIST, VAR_PARTIAL, VAR_STRING, VAR_UNKNOWN, VarLock, funcexe_T, linenr_T,
    partial_T, regmatch_T, regmmatch_T, staticList10_T, typval_T,
};
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
const E_NOT_ENOUGH_SPACE: &core::ffi::CStr = c"vim_regsub_both(): not enough space";

/// How large a buffer `tv_get_string_buf_chk` wants for a number.
const NUMBUFLEN: usize = 65;

/// A `funcexe_T` that asks for nothing.
const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0,
    fe_lastline: 0,
    fe_doesrange: core::ptr::null_mut(),
    fe_evaluate: false,
    fe_partial: core::ptr::null_mut(),
    fe_selfdict: core::ptr::null_mut(),
    fe_basetv: core::ptr::null_mut(),
    fe_found_var: false,
};

/// The case hook `\u`, `\U`, `\l` and `\L` install for the rest of a
/// replacement.
type CaseFolder = fn(c_int) -> c_int;

/// The case hooks in force: `\u` and `\l` fold one character, `\U` and `\L`
/// fold every character until `\e` or `\E`.
#[derive(Clone, Copy, Default)]
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
    fn new(dest: *mut c_char, destlen: c_int, copy: bool) -> Self {
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
pub(crate) unsafe fn regtilde(source: *mut c_char, magic: c_int, preview: bool) -> *mut c_char {
    // SAFETY: `source` is the caller's NUL-terminated replacement text, and
    // `reg_prev_sub` is null or a NUL-terminated string of `reg_prev_sublen`
    // bytes that this function is the sole writer of.
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
        let tmpsublen = prefixlen + reg_prev_sublen.get() + postfixlen;

        if tmpsublen == 0 || reg_prev_sub.get().is_null() {
            // Nothing to expand into: drop the tilde, NUL included, and
            // rescan from where it was.
            unsafe { p.cast::<u8>().copy_from(postfix.cast(), postfixlen + 1) };
            continue;
        }
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
        unsafe { into.copy_from(reg_prev_sub.get().cast(), reg_prev_sublen.get()) };
        let expanded = prefixlen + reg_prev_sublen.get();
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
        unsafe { xfree(reg_prev_sub.get().cast()) };
        reg_prev_sub.set(if newsublen == 0 {
            core::ptr::null_mut()
        } else {
            unsafe { xstrnsave(newsub, newsublen) }
        });
        reg_prev_sublen.set(newsublen);
    }
    newsub
}

/// Expand `source` into `dest` using the captures of the string match
/// `rmp`, or the result of `expr` when the replacement is an expression.
/// Returns the length written, plus one for the NUL.
///
/// The match must not have changed since [`vim_regexec`](super::vim_regexec)
/// ran: the captures point straight into the matched text.
pub(crate) unsafe fn vim_regsub(
    rmp: *mut regmatch_T,
    source: *mut c_char,
    expr: *mut typval_T,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
) -> c_int {
    // SAFETY: the arguments are the caller's; `with_rex` makes the context
    // ours for the call and restores any outer match's after it.
    with_rex(|| {
        let rex = unsafe { Rex::acquire() };
        rex.set_reg_match(rmp);
        rex.set_reg_mmatch(core::ptr::null_mut());
        rex.set_reg_maxline(0);
        rex.set_reg_buf(curbuf.get());
        // A string replacement has no lines to cross, so a `\n` in it
        // is a literal newline rather than a line break.
        rex.set_reg_line_lbr(true);
        unsafe { vim_regsub_both(rex, source, expr, dest, destlen, flags) }
    })
}

/// [`vim_regsub`] for a buffer match, whose captures can span lines from
/// `lnum` on.
pub(crate) unsafe fn vim_regsub_multi(
    rmp: *mut regmmatch_T,
    lnum: linenr_T,
    source: *mut c_char,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
) -> c_int {
    // SAFETY: as `vim_regsub`. A buffer match always works on `curbuf`.
    with_rex(|| {
        let rex = unsafe { Rex::acquire() };
        rex.set_reg_match(core::ptr::null_mut());
        rex.set_reg_mmatch(rmp);
        rex.set_reg_buf(curbuf.get());
        rex.set_reg_firstlnum(lnum);
        rex.set_reg_maxline(unsafe { (*curbuf.get()).b_ml.ml_line_count } - lnum);
        rex.set_reg_line_lbr(false);
        unsafe { vim_regsub_both(rex, source, core::ptr::null_mut(), dest, destlen, flags) }
    })
}

/// The expansion itself, against whatever match `rex` currently describes.
/// Returns the size of the result including its NUL, or 0 on an error.
unsafe fn vim_regsub_both(
    rex: Rex,
    source: *mut c_char,
    expr: *mut typval_T,
    dest: *mut c_char,
    destlen: c_int,
    flags: c_int,
) -> c_int {
    if (source.is_null() && expr.is_null()) || dest.is_null() {
        emsg(gettext(e_null));
        return 0;
    }
    if prog_magic_wrong(rex) != 0 {
        return 0;
    }
    if NESTING.get() == MAX_REGSUB_NESTING {
        emsg(gettext(E_SUBSTITUTE_NESTING_TOO_DEEP));
        return 0;
    }

    let mut out = Out::new(dest, destlen, flags & REGSUB_COPY as c_int != 0);
    // A caller-supplied function, or a replacement that starts `\=`, is
    // a Vimscript expression rather than replacement text.
    let outcome = if !expr.is_null()
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
unsafe fn eval_replacement(
    rex: Rex,
    source: *mut c_char,
    expr: *mut typval_T,
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
    // `replace` rather than a `get` and a `set`: the snapshot holds the
    // outer match's structures, and a copy alongside the cell would be a
    // second holder of them for the length of the evaluation.
    let outer_rsm = rsm.replace(regsubmatch_T {
        sm_match: rex.reg_match(),
        sm_mmatch: rex.reg_mmatch(),
        sm_firstlnum: rex.reg_firstlnum(),
        sm_maxline: rex.reg_maxline(),
        sm_line_lbr: rex.reg_line_lbr() as c_int,
    });

    NESTING.set(nested as c_int + 1);
    let mut text = if expr.is_null() {
        unsafe { eval_to_string(source.offset(2), true, false) }
    } else {
        unsafe { call_replacement(expr) }
    };
    NESTING.set(nested as c_int);

    if !text.is_null() {
        if unsafe { line_breaks_to_cr(text) } && flags & REGSUB_BACKSLASH as c_int != 0 {
            // The backslashes will be consumed downstream; double them
            // so they survive.
            let doubled = unsafe { vim_strsave_escaped(text, c"\\".as_ptr()) };
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

/// Call `expr` — a funcref or a partial — with the submatches as its one
/// argument, and return its result as an allocated string. Null when the
/// call failed, which has already reported itself.
unsafe fn call_replacement(expr: *mut typval_T) -> *mut c_char {
    // SAFETY: `expr` is the caller's live callable.
    // `fill_submatch_list` fills this in place if the function takes an
    // argument at all, so it must outlive the call.
    let mut match_list: staticList10_T = unsafe { core::mem::zeroed() };
    match_list.sl_list.lv_lock = VarLock::Fixed;
    let mut argv: [typval_T; 2] = unsafe { core::mem::zeroed() };
    argv[0].v_type = VAR_LIST;
    argv[0].vval.v_list = &raw mut match_list.sl_list;

    let mut rettv: typval_T = unsafe { core::mem::zeroed() };
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = core::ptr::null_mut();

    let mut funcexe = FUNCEXE_INIT;
    funcexe.fe_argv_func = Some(fill_submatch_list);
    funcexe.fe_evaluate = true;
    let name = if unsafe { (*expr).v_type } == VAR_FUNC {
        Some(unsafe { (*expr).vval.v_string })
    } else if unsafe { (*expr).v_type } == VAR_PARTIAL {
        let partial: *mut partial_T = unsafe { (*expr).vval.v_partial };
        funcexe.fe_partial = partial;
        Some(unsafe { partial_name(partial) })
    } else {
        None
    };
    if let Some(name) = name {
        let rettv = &raw mut rettv;
        let argv = argv.as_mut_ptr();
        let funcexe = &raw mut funcexe;
        let _ = unsafe { call_func(name, -1, rettv, 1, argv, funcexe) };
    }
    if unsafe { tv_list_len(&raw mut match_list.sl_list) } > 0 {
        // A non-empty list means `fill_submatch_list` ran and allocated.
        unsafe { clear_submatch_list(&raw mut match_list) };
    }

    // An unknown return type means the call failed and has already said
    // so; there is no second error to report.
    let text = if rettv.v_type == VAR_UNKNOWN {
        core::ptr::null_mut()
    } else {
        let mut buf: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        let s = unsafe { tv_get_string_buf_chk(&raw mut rettv, buf.as_mut_ptr()) };
        if s.is_null() {
            core::ptr::null_mut()
        } else {
            unsafe { xstrdup(s) }
        }
    };
    unsafe { tv_clear(&raw mut rettv) };
    text
}

/// Rewrite the newlines in an expression's result as carriage returns,
/// which is what the substitution machinery downstream reads as a line
/// break — unless the caller came from `vim_regexec_nl`, where a newline is
/// literal. Reports whether the text contained a backslash escape.
unsafe fn line_breaks_to_cr(text: *mut c_char) -> bool {
    // SAFETY: `text` is a NUL-terminated allocation this module owns.
    let literal_nl = unsafe { Rsm::acquire() }.line_lbr();
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
unsafe fn expand_replacement(
    rex: Rex,
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
            } else if !unsafe { vim_strchr(c"uUlLeE".as_ptr(), *src as u8 as c_int) }.is_null() {
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
            match unsafe { copy_capture(rex, no, &mut case, backslash, out) } {
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
unsafe fn copy_capture(
    rex: Rex,
    no: c_int,
    case: &mut Case,
    backslash: bool,
    out: &mut Out,
) -> Outcome {
    // SAFETY: `rex` describes a live match; `no` is a single digit and both
    // capture arrays hold `NSUBEXP` = 10 slots.
    let multi = rex.multi();
    let no = no as usize;

    // Where the capture starts, how much of it is on that line, and —
    // for a buffer match — which line that is.
    let mut clnum: linenr_T = 0;
    let mut len: c_int = 0;
    let mut s = if multi {
        // SAFETY: a buffer match's `reg_mmatch` is the caller's live structure.
        let mmatch = unsafe { Live::new(rex.reg_mmatch()) };
        clnum = mmatch.startpos[no].lnum;
        if clnum < 0 || mmatch.endpos[no].lnum < 0 {
            core::ptr::null_mut()
        } else {
            len = if mmatch.endpos[no].lnum == clnum {
                mmatch.endpos[no].col - mmatch.startpos[no].col
            } else {
                reg_getline_len(rex, clnum) - mmatch.startpos[no].col
            };
            unsafe { reg_getline(rex, clnum).offset(mmatch.startpos[no].col as isize) }
        }
    } else {
        // SAFETY: a string match's `reg_match` is the caller's live structure.
        let match_ = unsafe { Live::new(rex.reg_match()) };
        let start = match_.startp[no];
        if match_.endp[no].is_null() {
            core::ptr::null_mut()
        } else {
            // SAFETY: both slots point into the same matched string.
            len = unsafe { match_.endp[no].offset_from(start) } as c_int;
            start
        }
    };
    // A capture that did not participate contributes nothing.
    if s.is_null() {
        return Outcome::Done;
    }

    loop {
        if len == 0 {
            // SAFETY: as above.
            let mmatch = unsafe { Live::new(rex.reg_mmatch()) };
            if !multi || mmatch.endpos[no].lnum == clnum {
                return Outcome::Done;
            }
            // The capture continues on the next line.
            if !out.room(1) {
                return Outcome::NoSpace;
            }
            out.push(CAR as c_char);
            clnum += 1;
            s = reg_getline(rex, clnum);
            len = if mmatch.endpos[no].lnum == clnum {
                mmatch.endpos[no].col
            } else {
                reg_getline_len(rex, clnum)
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
