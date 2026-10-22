//! The command line's own text buffer, and pasting into it.
//!
//! [`realloc_cmdbuff`] is the one every caller has to be careful of: it may
//! move the buffer, so nothing may hold a pointer into it across a call.
//! [`cmdline_paste`] and [`ccheck_abbr`] are the two writers that go through
//! the register and abbreviation machinery, and the `*_fnameescape` helpers
//! escape a file name on its way in.
//!
//! [`Cc`] is the command line itself: one wrapper over `ccline` whose `Deref`
//! carries every field access in `ex_getln/` and its `cmdexpand/` neighbours,
//! derived afresh at each use because the register and abbreviation machinery
//! re-enters and can replace the whole structure.

#![forbid(unsafe_code)]

use super::*;
use crate::cstr;
use crate::guard::Lock;
use crate::keycodes::{Ctrl_A, Ctrl_BSL, Ctrl_C, Ctrl_F, Ctrl_L, Ctrl_N, Ctrl_P, Ctrl_V, Ctrl_W};
use crate::memory::XString;
use core::ffi::CStr;

// ---------------------------------------------------------------------------
// The command line being edited.
//
// `Cc` is `Live<CmdlineInfo>`, declared beside `Live` itself; the projections
// the command line needs are here, which an inherent impl on a local type may
// be in any module of the defining crate. The whole of `ex_getln/` plus the
// `cmdexpand/` entry points that used to take a `*mut CmdlineInfo` go through
// the handle instead.

impl Cc {
    /// Whether the command line is waiting for a single key: C's `one_key`.
    pub(crate) fn one_key(self) -> bool {
        self.one_key
    }

    /// Whether a `:[N]` style number prompt is up: C's `mouse_used != NULL`.
    pub(crate) fn mouse_prompt(self) -> bool {
        self.mouse_prompt
    }

    /// Run `f` on the completion in progress, or on `None` when there is
    /// none, for a reader that runs no user code.
    ///
    /// The completion is moved out of the command line for the call, so `f`
    /// may read the rest of the line through any handle without reaching
    /// what it is handed.
    pub(crate) fn with_xpc<R>(mut self, f: impl FnOnce(Option<&mut Expand>) -> R) -> R {
        let mut xpc = self.xpc.take();
        let answer = f(xpc.as_mut());
        self.xpc = xpc;
        answer
    }

    /// Run `f` on the command line's completion, which its key loop holds,
    /// moved out of the line for the length of the call.
    ///
    /// The completion runs user code -- `customlist` functions, Lua
    /// `ui_attach` callbacks, backtick expressions in file names -- and
    /// that code can ask about the command line's completion. While `f`
    /// holds it, it finds an empty one in its place instead of one this code
    /// holds mutably, which is what upstream's own readers would mostly have
    /// seen mid-expansion anyway.
    pub(crate) fn with_completion<R>(mut self, f: impl FnOnce(&mut Expand) -> R) -> R {
        let mut xpc = core::mem::take(self.xpc_mut());
        let answer = f(&mut xpc);
        // Back into the line the loop is on: the cell, which a command line
        // the call suspended has been restored into by now.
        if let Some(slot) = self.xpc.as_mut() {
            *slot = xpc;
        }
        answer
    }

    /// The completion the key loop holds, for a read that runs no user code.
    pub(crate) fn xpc(&self) -> &Expand {
        self.xpc.as_ref().expect("a command line in its key loop")
    }

    /// [`Cc::xpc`] for a write.
    pub(crate) fn xpc_mut(&mut self) -> &mut Expand {
        self.xpc.as_mut().expect("a command line in its key loop")
    }

    /// The command line's own bytes, as bytes: [`Cc::bytes`] by its older
    /// name, which the completion code reads by.
    pub(crate) fn text_bytes(&self) -> &[u8] {
        self.cmdbuff.bytes()
    }

    /// Replace `[from, to)` of the text with `with`, moving the rest of the
    /// line along, after making room for `extra` more bytes than the line
    /// has: one completion inserted over the word it completes. The cursor
    /// is the caller's to move.
    pub(crate) fn replace_range(
        mut self,
        from: ::core::ffi::c_int,
        to: ::core::ffi::c_int,
        with: &[u8],
        extra: ::core::ffi::c_int,
    ) {
        realloc_cmdbuff(self, self.len() + extra);
        self.cmdbuff.splice(from, to, with);
    }

    /// The command line's own bytes: C's `cmdbuff[..cmdlen]`.
    ///
    /// The borrow is of this handle, not of the line: the line moves under
    /// [`Cc::reserve`] and [`put_on_cmdline`], and the register and
    /// abbreviation machinery can replace it outright, so a reader takes the
    /// slice inside the expression that reads it, or copies it, rather than
    /// holding it across a call.
    pub(crate) fn bytes(&self) -> &[u8] {
        self.cmdbuff.bytes()
    }

    /// The byte at `i`, the terminator and what follows it reading as NUL.
    pub(crate) fn byte_at(&self, i: ::core::ffi::c_int) -> u8 {
        self.cmdbuff
            .byte_at(usize::try_from(i).unwrap_or(usize::MAX))
    }

    /// C's `cmdlen`.
    pub(crate) fn len(self) -> ::core::ffi::c_int {
        self.cmdbuff.len()
    }

    /// Whether the command line holds no text: C's `cmdlen == 0`.
    ///
    /// Not [`Cc::in_use`], which asks whether there is a command line at
    /// all: a line that is in use may still be empty.
    pub(crate) fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Whether a command line is in use: C's `cmdbuff != NULL`.
    pub(crate) fn in_use(self) -> bool {
        self.cmdbuff.in_use()
    }

    /// C's `realloc_cmdbuff`: make room for `want` bytes, terminator
    /// included.
    pub(crate) fn reserve(mut self, want: ::core::ffi::c_int) {
        self.cmdbuff.reserve(want);
    }

    /// Replace the text, opening a command line if none was in use.
    pub(crate) fn set_text(mut self, bytes: &[u8]) {
        self.cmdbuff.set(bytes);
    }

    /// C's `alloc_cmdbuff`: an empty command line with room for `want` bytes.
    pub(crate) fn open(mut self, want: ::core::ffi::c_int) {
        self.cmdbuff.open(want);
    }

    /// C's `dealloc_cmdbuff`: no command line in use.
    pub(crate) fn close(mut self) {
        self.cmdbuff.close();
    }

    /// Hand the text to a caller that owns it, closing the command line.
    ///
    /// `None` when no command line was in use, which is what the callers
    /// test. The answer is the text up to its first NUL: what C's caller
    /// read of the `char *` it was handed.
    pub(crate) fn release(self) -> Option<XString> {
        if !self.in_use() {
            return None;
        }
        let text = XString::from_bytes(cstr::until_nul(self.bytes()));
        self.close();
        Some(text)
    }
}

// ---------------------------------------------------------------------------
// The neighbours that are still transpiled, one wrapper each.

fn is_word_char(c: ::core::ffi::c_int) -> bool {
    vim_iswordc(c)
}

fn stuff_char(c: ::core::ffi::c_int) {
    stuff_readbuf_char(c);
}

// ---------------------------------------------------------------------------
// Getting a command line.

/// Get an Ex command line for the `:` command.
///
/// `c` is normally `:`, and NUL for `:append`; `indent` is the indent for
/// inside conditionals. The [`Typed`](crate::ex_docmd::LineSource::Typed)
/// arm of `do_cmdline`'s line source; `None` when the line was abandoned.
pub fn getexline(
    c: ::core::ffi::c_int,
    indent: ::core::ffi::c_int,
    do_concat: bool,
) -> Option<XString> {
    // When executing a register, remove the ':' in front of each line.
    // SAFETY: peeks at the typeahead.
    if exec_from_reg.get() && vpeekc() == ':' as ::core::ffi::c_int {
        // SAFETY: consumes the byte `vpeekc` just reported.
        vgetc();
    }
    getcmdline(c, 1, indent, do_concat)
}

/// A command line read for a command that wants more of its body and has no
/// source to read it from: `getcmdline` with no count, owned.
pub(crate) fn cmdline_input(
    c: ::core::ffi::c_int,
    indent: ::core::ffi::c_int,
    do_concat: bool,
) -> Option<XString> {
    getcmdline(c, 0, indent, do_concat)
}

/// The pattern `/` or `?` reads, given `count`; `None` when it was
/// abandoned.
pub(crate) fn search_cmdline(
    firstc: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
) -> Option<XString> {
    getcmdline(firstc, count, 0, true)
}

pub fn cmdline_overstrike() -> bool {
    Cc::current().overstrike != 0
}

/// Whether the cursor is at the end of the command line.
pub fn cmdline_at_end() -> bool {
    let cc = Cc::current();
    cc.cmdpos >= cc.len()
}

// ---------------------------------------------------------------------------
// The allocation.

/// Close the command line: C's `dealloc_cmdbuff`.
pub(crate) fn dealloc_cmdbuff() {
    Cc::current().close();
}

/// Make room for `len` bytes of command line, terminator included.
///
/// C's `realloc_cmdbuff`, and it may still *move* the buffer, but only when
/// the buffer it is asked to grow is the one being written -- `Cc` carries
/// which. `xp_pattern` is the one pointer into it upstream knows about and
/// re-derives here; anything else holding a pointer or an offset into the
/// text across a call is a bug, which is why the completion code keeps
/// indices.
pub(crate) fn realloc_cmdbuff(cc: Cc, len: ::core::ffi::c_int) {
    cc.reserve(len);
}

/// Suspend `ccline` onto the saved stack, because obtaining the `=` register
/// may execute `normal :cmd` and overwrite it.
///
/// The command line *moves* out of the cell: it owns `cmdbuff` and a
/// highlight `Callback`, and leaving a second copy behind in `ccline` would
/// be two owners of both. What is left is [`CMDLINE_INFO_INIT`], whose null
/// `cmdbuff` is the signal that no command line is in use.
pub(crate) fn save_cmdline() {
    // Both closures are leaves -- a move out of the cell and a push onto a
    // `Vec` -- so neither exclusive borrow can overlap another.
    let saved = ccline.with_mut(|cc| core::mem::replace(cc, CMDLINE_INFO_INIT));
    saved_cmdlines.with_mut(|stack| stack.push(Box::new(saved)));
}

/// Resume the command line [`save_cmdline`] suspended.
///
/// Panics if nothing was suspended: the two callers pair the calls on a
/// `did_save_ccline` flag, and an unpaired restore would otherwise silently
/// leave the wrong line current.
pub(crate) fn restore_cmdline() {
    let saved = saved_cmdlines.with_mut(|stack| stack.pop());
    ccline.set(*saved.expect("restore_cmdline without save_cmdline"));
}

/// The command line `depth` levels out: 0 is the one being edited, 1 the one
/// suspended under it, and so on.  `None` past the bottom of the stack.
pub(crate) fn cmdline_at(depth: usize) -> Option<Cc> {
    if depth == 0 {
        Some(Cc::current())
    } else {
        Cc::saved(depth)
    }
}

// ---------------------------------------------------------------------------
// Putting text into the command line.

/// Paste a yank register into the command line, for CTRL-R.
///
/// `insert_reg()` can't be used here, because special characters from the
/// register contents would be interpreted as commands.  `literally` inserts
/// the text as-is rather than as typed; `remcr` removes a trailing CR.
/// Answers false for failure.
pub(crate) fn cmdline_paste(regname: ::core::ffi::c_int, literally: bool, remcr: bool) -> bool {
    // Check for a valid regname; also accept the special characters
    // CTRL-R takes on the command line.
    if regname != Ctrl_F
        && regname != Ctrl_P
        && regname != Ctrl_W
        && regname != Ctrl_A
        && regname != Ctrl_L
        && !is_yank_reg(regname)
    {
        return false;
    }

    // A register containing CTRL-R can cause an endless loop. Allow
    // using CTRL-C to break out of it.
    line_breakcheck();
    if got_int.get() {
        return false;
    }

    // "textlock" avoids nasty things like going to another buffer while
    // evaluating an expression.
    let special = {
        let _locked = Lock::text();
        special_reg(regname)
    };

    let Some(arg) = special else {
        return cmdline_paste_reg(regname, literally, remcr);
    };

    // Got the value of a special register in "arg".
    let Some(arg) = arg else {
        return false;
    };
    let mut skip = 0;
    // With 'incsearch' set and CTRL-R CTRL-W used: skip the duplicate
    // part of the word.
    if p_is() && regname == Ctrl_W {
        skip = duplicate_word_len(&arg);
    }

    cmdline_paste_str(arg.cstr_at(skip), literally);
    true
}

/// How much of `arg` the last word on the command line already spells.
///
/// The word is looked for in `cmdbuff` afresh: nothing above has moved it,
/// but nothing below may hold it either.
fn duplicate_word_len(arg: &[u8]) -> usize {
    let cc = Cc::current();
    let line = cc.bytes();
    // Locate the start of the last word in the cmd buffer.
    let end = usize::try_from(cc.cmdpos).unwrap_or(0).min(line.len());
    let mut w = end;
    while w > 0 {
        let len = head_off(line, w - 1) + 1;
        if !is_word_char(c_char_info(line, w - len).value) {
            break;
        }
        w -= len;
    }
    let len = end - w;
    let word = &line[w..end];
    let same = if p_ic() {
        cstr::ncase_eq_ascii(word, arg, len)
    } else {
        cstr::ncmp_eq(word, arg, len)
    };
    if same { len } else { 0 }
}

fn is_yank_reg(regname: ::core::ffi::c_int) -> bool {
    valid_yank_reg(regname, false)
}

/// `get_spec_reg()` for reading: `None` when `regname` is not a computed
/// register, `Some(None)` when it has nothing in it.
///
/// Runs user code -- `'` and `=` evaluate an expression -- which is why the
/// caller raises `textlock` around it and holds no command line across it.
fn special_reg(regname: ::core::ffi::c_int) -> Option<Option<XString>> {
    get_spec_reg(regname, true)
}

/// Put a string on the command line.
///
/// With `literally` set the text is inserted as-is; otherwise it is stuffed
/// back as if typed — which does not leave the command line, but does mean
/// every character that would end it has to be quoted with CTRL-V.
pub fn cmdline_paste_str(s: &CStr, literally: bool) {
    let text = s.to_bytes();
    if literally {
        put_on_cmdline(text, true);
        return;
    }
    let mut at = 0;
    while at < text.len() {
        let cv = ::core::ffi::c_int::from(text[at]);
        if cv == Ctrl_V && at + 1 < text.len() {
            at += 1;
        }
        let info = c_char_info(text, at);
        let c = info.value;
        at += usize::try_from(info.len).unwrap_or(1).max(1);
        if cv == Ctrl_V
            || c == ESC
            || c == Ctrl_C
            || c == CAR
            || c == NL
            || c == Ctrl_L
            || (c == Ctrl_BSL && cstr::byte_at(text, at) == Ctrl_N as u8)
        {
            stuff_char(Ctrl_V);
        }
        stuff_char(c);
    }
}

/// Check whether typing `c` completes an abbreviation on the command line.
pub(crate) fn ccheck_abbr(c: ::core::ffi::c_int) -> bool {
    if p_paste() || no_abbr.get() {
        // no abbreviations, or in paste mode
        return false;
    }
    let cc = Cc::current();
    let line = cc.bytes();

    // Do not consider '<,'> to be part of the mapping; skip leading
    // whitespace first. This actually accepts any mark.
    let mut spos = line
        .iter()
        .position(|&ch| !ascii_iswhite(ch as ::core::ffi::c_int))
        .unwrap_or(line.len());
    if line.len() - spos > 5
        && line[spos] == b'\''
        && line[spos + 2] == b','
        && line[spos + 3] == b'\''
    {
        spos += 5;
    } else {
        // Check the abbreviation from the start of the command line.
        spos = 0;
    }

    // A copy of the line before the cursor: the expansion can run user code,
    // which may change the command line.
    let col = usize::try_from(cc.cmdpos).unwrap_or(0);
    let text = cc.text_bytes();
    let text = text[..col.min(text.len())].to_vec();
    check_abbr(c, text, spos as ::core::ffi::c_int)
}

// ---------------------------------------------------------------------------
// Escaping a file name on its way in.

/// Escape the special characters in `fname`, depending on `what`:
/// `VSE_NONE` for a file-name argument after a Vim command, `VSE_SHELL` for
/// a shell command, `VSE_BUFFER` for `:buffer`: upstream's
/// `vim_strsave_fnameescape`.
pub(crate) fn fnameescape(fname: &CStr, what: ::core::ffi::c_int) -> XString {
    let esc = if what == VSE_SHELL {
        SHELL_ESC_CHARS
    } else if what == VSE_BUFFER {
        BUFFER_ESC_CHARS
    } else {
        PATH_ESC_CHARS
    };
    let mut p = escaped_bytes(fname.to_bytes(), esc.to_bytes());
    if what == VSE_SHELL && csh_like_shell() {
        // csh and similar shells need two backslashes before '!': one
        // is taken by Vim, one by the shell.
        p = escaped_bytes(&p, b"!");
    }
    // '>' and '+' are special at the start of some commands, e.g.
    // ":edit" and ":write". "cd -" has a special meaning.
    let name: &[u8] = &p;
    if matches!(name.first(), Some(b'>' | b'+')) || name == b"-" {
        let mut quoted = Vec::with_capacity(name.len() + 1);
        quoted.push(b'\\');
        quoted.extend_from_slice(name);
        p = XString::from_vec(quoted);
    }
    p
}

/// If `orig_pat` starts with `~/`, put the home directory back as `~` in
/// each of `files`.
pub(crate) fn tilde_replace_matches(orig_pat: &[u8], files: &mut [XString]) {
    if orig_pat.first() != Some(&b'~')
        || !vim_ispathsep(::core::ffi::c_int::from(
            orig_pat.get(1).copied().unwrap_or(0),
        ))
    {
        return;
    }
    for file in files {
        // A name is at most as long as its expansion, plus the `~/`.
        let room = file.len() + 3;
        *file = crate::os::env::home_replace_in(None, file.as_cstr(), room, true);
    }
}
