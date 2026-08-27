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

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::Lock;
use crate::keycodes::{Ctrl_A, Ctrl_BSL, Ctrl_C, Ctrl_F, Ctrl_L, Ctrl_N, Ctrl_P, Ctrl_V, Ctrl_W};
use crate::types::{ExpandContext, NUL};

// ---------------------------------------------------------------------------
// The command line being edited.
//
// `Cc` is `Live<CmdlineInfo>`, declared beside `Live` itself; the projections
// the command line needs are here, which an inherent impl on a local type may
// be in any module of the defining crate. The whole of `ex_getln/` plus the
// `cmdexpand/` entry points that used to take a `*mut CmdlineInfo` go through
// the handle instead.

impl Cc {
    /// The command line the editor is on.
    ///
    /// Safe: `ccline` is a `&'static GlobalCell`, which is exactly the
    /// promise [`Live::new`] has to be told. The *cell* never moves; its
    /// contents do, which is why the handle is derived afresh at each use
    /// rather than held across a call that can re-enter command-line mode --
    /// [`save_cmdline`] moves the whole structure onto the saved stack, and
    /// [`realloc_cmdbuff`] moves the text out from under any pointer into it.
    pub(crate) fn current() -> Self {
        // SAFETY: a `&'static GlobalCell`'s address is live for the whole
        // run, which is the promise.
        unsafe { Cc::new(ccline.ptr()) }
    }

    /// Whether the command line is waiting for a single key: C's `one_key`.
    pub(crate) fn one_key(self) -> bool {
        self.one_key
    }

    /// Whether a `:[N]` style number prompt is up: C's `mouse_used != NULL`.
    pub(crate) fn mouse_used(self) -> bool {
        !self.mouse_used.is_null()
    }

    /// C's `xpc`: the completion in progress, NULL when there is none.
    pub(crate) fn xpc(self) -> *mut expand_T {
        self.xpc
    }

    /// The command line's own bytes: C's `cmdbuff[..cmdlen]`.
    ///
    /// The slice must not outlive the next [`Cc::reserve`] or
    /// [`put_on_cmdline`] -- both may move the allocation -- which is why
    /// every caller takes it inside the expression that reads it rather than
    /// binding it across a call. That is also why the lifetime is free: the
    /// handle is a pointer, so a borrow it carried would say nothing true.
    pub(crate) fn bytes<'a>(mut self) -> &'a [::core::ffi::c_char] {
        let text = self.cmdbuff.bytes();
        // SAFETY: the borrow is the caller's obligation, stated above.
        unsafe { ::core::slice::from_raw_parts(text.as_ptr(), text.len()) }
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

    /// C's `cmdbuff`: the text and its terminator, NULL when no command line
    /// is in use.
    pub(crate) fn text(mut self) -> *mut ::core::ffi::c_char {
        self.cmdbuff.as_mut_ptr()
    }

    /// C's `cmdbuff + i`.
    ///
    /// `wrapping_offset`, not `offset`: with no command line in use the base
    /// is NULL, and several callers compute an address they then compare
    /// rather than read.
    pub(crate) fn at(mut self, i: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
        self.cmdbuff.as_mut_ptr().wrapping_offset(i as isize)
    }

    /// C's `realloc_cmdbuff`: make room for `want` bytes, terminator
    /// included.
    pub(crate) fn reserve(mut self, want: ::core::ffi::c_int) {
        self.cmdbuff.reserve(want);
    }

    /// C's `cmdlen = n`, with the terminator that goes with it.
    pub(crate) fn set_len(mut self, n: ::core::ffi::c_int) {
        self.cmdbuff.set_len(n);
    }

    /// Replace the text, opening a command line if none was in use.
    pub(crate) fn set_text(mut self, bytes: &[::core::ffi::c_char]) {
        self.cmdbuff.set(bytes);
    }

    /// Replace the text with a NUL-terminated string's bytes.
    ///
    /// # Safety
    ///
    /// `s` must be a live NUL-terminated string that does not point into this
    /// command line's own buffer.
    pub(crate) unsafe fn set_cstr(self, s: *const ::core::ffi::c_char) {
        // SAFETY: the caller's promise -- a NUL-terminated string.
        let text = unsafe { ::core::slice::from_raw_parts(s, len(s)) };
        self.set_text(text);
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
    /// `getcmdline()`'s answer is an `xmalloc`ed C string its caller frees,
    /// so the bytes are copied out rather than the `Vec` released: an
    /// allocation the editor made is not one `xfree` may take. Answers NULL
    /// when no command line was in use, which is what the callers test.
    pub(crate) fn release(self) -> *mut ::core::ffi::c_char {
        if !self.in_use() {
            return ::core::ptr::null_mut();
        }
        let text = self.bytes();
        let out = alloc(text.len() as size_t + 1);
        // SAFETY: `text.len() + 1` bytes were just asked for, and `xmalloc`
        // aborts rather than answering null.
        let copy = unsafe { ::core::slice::from_raw_parts_mut(out, text.len() + 1) };
        copy[..text.len()].copy_from_slice(text);
        copy[text.len()] = NUL as ::core::ffi::c_char;
        self.close();
        out
    }

    /// The byte offset the cursor is on.
    fn cursor(self) -> isize {
        self.cmdpos as isize
    }
}

// ---------------------------------------------------------------------------
// The neighbours that are still transpiled, one wrapper each.

fn free<T>(p: *mut T) {
    // SAFETY: `xmalloc`ed, or null.
    unsafe { xfree(p as *mut ::core::ffi::c_void) };
}

fn alloc(len: size_t) -> *mut ::core::ffi::c_char {
    // SAFETY: aborts rather than answering null.
    unsafe { xmalloc(len) as *mut ::core::ffi::c_char }
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    // SAFETY: a NUL-terminated string.
    unsafe { *p as ::core::ffi::c_int }
}

fn len(p: *const ::core::ffi::c_char) -> size_t {
    // SAFETY: a NUL-terminated string.
    unsafe { strlen(p) }
}

/// `STRCPY(dst, src)`: copy `src` and its NUL.
fn copy_str(dst: *mut ::core::ffi::c_char, src: *const ::core::ffi::c_char) {
    // SAFETY: a NUL-terminated source, and room for it and its NUL.
    unsafe { strcpy(dst, src) };
}

fn is_word_char(c: ::core::ffi::c_int) -> bool {
    // SAFETY: reads the 'iskeyword' tables.
    unsafe { vim_iswordc(c) }
}

/// Whether `a` and `b` agree over their first `n` bytes.
fn same_prefix(
    a: *const ::core::ffi::c_char,
    b: *const ::core::ffi::c_char,
    n: size_t,
    ignore_case: bool,
) -> bool {
    let cmp = if ignore_case { strncasecmp } else { strncmp };
    // SAFETY: two strings of at least `n` bytes.
    unsafe { cmp(a, b, n) == 0 }
}

fn stuff_char(c: ::core::ffi::c_int) {
    stuff_readbuf_char(c);
}

// ---------------------------------------------------------------------------
// Getting a command line.

/// Get an Ex command line for the `:` command.
///
/// `c` is normally `:`, and NUL for `:append`; `indent` is the indent for
/// inside conditionals.  Registered as a `LineGetter` in several tables, so
/// this one keeps its C ABI.
pub unsafe fn getexline(
    c: ::core::ffi::c_int,
    _cookie: *mut ::core::ffi::c_void,
    indent: ::core::ffi::c_int,
    do_concat: bool,
) -> *mut ::core::ffi::c_char {
    // When executing a register, remove the ':' in front of each line.
    // SAFETY: peeks at the typeahead.
    if exec_from_reg.get() && vpeekc() == ':' as ::core::ffi::c_int {
        // SAFETY: consumes the byte `vpeekc` just reported.
        vgetc();
    }
    // SAFETY: reads a whole command line, re-entering the editor.
    unsafe { getcmdline(c, 1, indent, do_concat) }
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
    let old = cc.text();
    cc.reserve(len);
    if cc.text() != old {
        move_xp_pattern(cc, old);
    }
}

/// If `xp_pattern` pointed inside the old text it has to be adjusted to point
/// into the newly allocated memory.
fn move_xp_pattern(mut cc: Cc, old: *mut ::core::ffi::c_char) {
    if cc.xpc.is_null() {
        return;
    }
    // SAFETY: a live completion context.
    let xpc = unsafe { &mut *cc.xpc };
    if xpc.xp_pattern.is_null()
        || xpc.xp_context == ExpandContext::Nothing
        || xpc.xp_context == ExpandContext::Unsuccessful
    {
        return;
    }
    // SAFETY: as upstream -- `xp_pattern` either points into `old` or into
    // something else entirely, and the difference decides which.
    let i = unsafe { xpc.xp_pattern.offset_from(old) } as ::core::ffi::c_int;
    if i >= 0 && i <= cc.len() {
        xpc.xp_pattern = cc.at(i);
    }
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
///
/// The closure is a leaf that takes the address of a `Box`'s target -- which
/// the `Vec` cannot move, however it grows -- and does nothing else.
pub(crate) fn cmdline_at(depth: usize) -> Option<Cc> {
    if depth == 0 {
        return Some(Cc::current());
    }
    saved_cmdlines.with_mut(|stack| {
        let i = stack.len().checked_sub(depth)?;
        // SAFETY: the address of a `Box`'s target, which the `Vec` cannot
        // move however it grows; the entry lives until it is popped, and a
        // `Cc` is derived afresh at each use.
        Some(unsafe { Cc::new(&raw mut *stack[i]) })
    })
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
    let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut();
    let mut allocated: bool = false;
    let got_special = {
        let _locked = Lock::text();
        special_reg(regname, &raw mut arg, &raw mut allocated)
    };

    if !got_special {
        // SAFETY: a register name the check above accepted.
        return unsafe { cmdline_paste_reg(regname, literally, remcr) };
    }

    // Got the value of a special register in "arg".
    if arg.is_null() {
        return false;
    }
    let mut p = arg;
    // With 'incsearch' set and CTRL-R CTRL-W used: skip the duplicate
    // part of the word.
    if p_is.get() != 0 && regname == Ctrl_W {
        let skip = duplicate_word_len(arg);
        p = p.wrapping_add(skip as usize);
    }

    // SAFETY: the register's own text, or a slice of it.
    unsafe { cmdline_paste_str(p, literally) };
    if allocated {
        free(arg);
    }
    true
}

/// How much of `arg` the last word on the command line already spells.
///
/// The word is looked for in `cmdbuff` afresh: nothing above has moved it,
/// but nothing below may hold it either.
fn duplicate_word_len(arg: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let cc = Cc::current();
    // Locate the start of the last word in the cmd buffer.
    let end = cc.text().wrapping_offset(cc.cursor());
    let mut w = end;
    while w > cc.text() {
        let len = head_off(cc.text(), w.wrapping_offset(-1)) + 1;
        if !is_word_char(char_at(w.wrapping_offset(-(len as isize)))) {
            break;
        }
        w = w.wrapping_offset(-(len as isize));
    }
    let len = end.addr().wrapping_sub(w.addr()) as ::core::ffi::c_int;
    if same_prefix(w, arg, len as size_t, p_ic.get() != 0) {
        len
    } else {
        0
    }
}

fn head_off(base: *const ::core::ffi::c_char, p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    // SAFETY: `p` points into the string starting at `base`.
    unsafe { utf_head_off(base, p) }
}

fn char_at(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    // SAFETY: the start of a character in a NUL-terminated string.
    unsafe { utf_ptr2char(p) }
}

fn is_yank_reg(regname: ::core::ffi::c_int) -> bool {
    // SAFETY: reads the register tables.
    unsafe { valid_yank_reg(regname, false) }
}

/// `get_spec_reg()` for reading: fills `arg` and says whether it did.
///
/// Runs user code -- `'` and `=` evaluate an expression -- which is why the
/// caller raises `textlock` around it and holds no command line across it.
fn special_reg(
    regname: ::core::ffi::c_int,
    arg: *mut *mut ::core::ffi::c_char,
    allocated: *mut bool,
) -> bool {
    // SAFETY: two slots of the caller's own.
    unsafe { get_spec_reg(regname, arg, allocated, true) }
}

/// Put a string on the command line.
///
/// With `literally` set the text is inserted as-is; otherwise it is stuffed
/// back as if typed — which does not leave the command line, but does mean
/// every character that would end it has to be quoted with CTRL-V.
pub unsafe fn cmdline_paste_str(mut s: *const ::core::ffi::c_char, literally: bool) {
    if literally {
        // SAFETY: a NUL-terminated string, whose length it works out itself.
        unsafe { put_on_cmdline(s, -1, true) };
        return;
    }
    while byte(s) != NUL {
        let cv = byte(s) as uint8_t as ::core::ffi::c_int;
        if cv == Ctrl_V && byte(s.wrapping_add(1)) != 0 {
            s = s.wrapping_add(1);
        }
        // SAFETY: a NUL-terminated string; `s` is left on the next character.
        let c = unsafe { mb_cptr2char_adv(&raw mut s) };
        if cv == Ctrl_V
            || c == ESC
            || c == Ctrl_C
            || c == CAR
            || c == NL
            || c == Ctrl_L
            || (c == Ctrl_BSL && byte(s) == Ctrl_N)
        {
            stuff_char(Ctrl_V);
        }
        stuff_char(c);
    }
}

/// Check whether typing `c` completes an abbreviation on the command line.
pub(crate) fn ccheck_abbr(c: ::core::ffi::c_int) -> bool {
    if p_paste.get() != 0 || no_abbr.get() {
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
        && line[spos] == '\'' as ::core::ffi::c_char
        && line[spos + 2] == ',' as ::core::ffi::c_char
        && line[spos + 3] == '\'' as ::core::ffi::c_char
    {
        spos += 5;
    } else {
        // Check the abbreviation from the start of the command line.
        spos = 0;
    }

    let (buff, col, mincol) = (cc.text(), cc.cmdpos, spos as ::core::ffi::c_int);
    // SAFETY: the live command line, and two offsets inside it.
    unsafe { check_abbr(c, buff, col, mincol) }
}

// ---------------------------------------------------------------------------
// Escaping a file name on its way in.

/// Escape the special characters in `fname`, depending on `what`:
/// `VSE_NONE` for a file-name argument after a Vim command, `VSE_SHELL` for
/// a shell command, `VSE_BUFFER` for `:buffer`.  Answers allocated memory.
pub unsafe fn vim_strsave_fnameescape(
    fname: *const ::core::ffi::c_char,
    what: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let esc = if what == VSE_SHELL {
        SHELL_ESC_CHARS.as_ptr()
    } else if what == VSE_BUFFER {
        BUFFER_ESC_CHARS.as_ptr()
    } else {
        PATH_ESC_CHARS.as_ptr()
    };
    let mut p = escaped(fname, esc);
    if what == VSE_SHELL && csh_like_shell() {
        // csh and similar shells need two backslashes before '!': one
        // is taken by Vim, one by the shell.
        let s = escaped(p, c"!".as_ptr());
        free(p);
        p = s;
    }
    // '>' and '+' are special at the start of some commands, e.g.
    // ":edit" and ":write". "cd -" has a special meaning.
    let first = byte(p);
    if first == '>' as ::core::ffi::c_int
        || first == '+' as ::core::ffi::c_int
        || (first == '-' as ::core::ffi::c_int && byte(p.wrapping_add(1)) == NUL)
    {
        p = with_backslash(p);
    }
    p
}

/// `vim_strsave_escaped()`: a copy of `s` with a backslash before every byte
/// in `esc_chars`.
fn escaped(
    s: *const ::core::ffi::c_char,
    esc_chars: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    // SAFETY: two NUL-terminated strings; the answer is `xmalloc`ed.
    unsafe { vim_strsave_escaped(s, esc_chars) }
}

/// Put a backslash before the file name in `pp`, which is allocated memory.
pub unsafe fn escape_fname(pp: *mut *mut ::core::ffi::c_char) {
    // SAFETY: the caller's promise -- a slot holding allocated memory.
    unsafe { *pp = with_backslash(*pp) };
}

/// A fresh copy of `name` with a backslash in front, `name` freed.
fn with_backslash(name: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let p = alloc(len(name).wrapping_add(2));
    // SAFETY: two bytes were asked for beyond the name's own length.
    unsafe { *p = '\\' as ::core::ffi::c_char };
    copy_str(p.wrapping_add(1), name);
    free(name);
    p
}

/// For each name in `files[..num_files]`: if `orig_pat` starts with `~/`,
/// put the home directory back as `~`.
pub unsafe fn tilde_replace(
    orig_pat: *mut ::core::ffi::c_char,
    num_files: ::core::ffi::c_int,
    files: *mut *mut ::core::ffi::c_char,
) {
    if byte(orig_pat) != '~' as ::core::ffi::c_int || !vim_ispathsep(byte(orig_pat.wrapping_add(1)))
    {
        return;
    }
    let n = num_files.max(0) as usize;
    // SAFETY: the caller's promise -- `num_files` names.
    for file in unsafe { ::core::slice::from_raw_parts_mut(files, n) } {
        // SAFETY: an allocated file name.
        let p = unsafe { home_replace_save(::core::ptr::null_mut::<buf_T>(), *file) };
        free(*file);
        *file = p;
    }
}
