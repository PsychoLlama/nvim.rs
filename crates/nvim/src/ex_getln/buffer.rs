//! The `ccline.cmdbuff` allocation, and pasting into it.
//!
//! [`realloc_cmdbuff`] is the one every caller has to be careful of: it moves
//! the buffer, so nothing may hold a pointer into it across a call.
//! [`cmdline_paste`] and [`ccheck_abbr`] are the two writers that go through
//! the register and abbreviation machinery, and the `*_fnameescape` helpers
//! escape a file name on its way in.
//!
//! [`Cc`] is the command line itself: one wrapper over `ccline` whose `Deref`
//! carries every field access in the file, derived afresh at each use because
//! the register and abbreviation machinery re-enters and can replace the
//! whole structure.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{Ctrl_A, Ctrl_BSL, Ctrl_C, Ctrl_F, Ctrl_L, Ctrl_N, Ctrl_P, Ctrl_V, Ctrl_W};
use core::ops::{Deref, DerefMut};

// ---------------------------------------------------------------------------
// The command line being edited.

/// `ccline`, the command line the editor is on.
///
/// Every reader here would otherwise spell `(*ccline.ptr()).field`; the two
/// `Deref` impls state the obligation once. A value is always derived through
/// [`Cc::current`] at the point of use and never held across a call that can
/// re-enter command-line mode -- [`save_cmdline`] replaces the whole
/// structure, and [`realloc_cmdbuff`] moves the text out from under any
/// pointer into it.
#[derive(Clone, Copy)]
struct Cc(*mut CmdlineInfo);

impl Deref for Cc {
    type Target = CmdlineInfo;

    fn deref(&self) -> &CmdlineInfo {
        // SAFETY: `ccline` is a live editor global.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Cc {
    fn deref_mut(&mut self) -> &mut CmdlineInfo {
        // SAFETY: `ccline` is a live editor global.
        unsafe { &mut *self.0 }
    }
}

impl Cc {
    fn current() -> Self {
        Cc(ccline.ptr())
    }

    /// The command line's own bytes: `cmdbuff[..cmdlen]`.
    ///
    /// `cmdbuff` is NULL between [`dealloc_cmdbuff`] and the next
    /// [`alloc_cmdbuff`] and `slice::from_raw_parts` may not be handed a null
    /// pointer even with a length of zero, so the empty line is a separate
    /// arm. The slice must not outlive the next [`realloc_cmdbuff`] or
    /// [`put_on_cmdline`] -- both move the allocation -- which is why every
    /// caller takes it inside the expression that reads it rather than
    /// binding it across a call.
    fn bytes<'a>(self) -> &'a [::core::ffi::c_char] {
        if self.cmdbuff.is_null() {
            return &[];
        }
        let (from, n) = (self.cmdbuff, self.cmdlen.max(0) as usize);
        // SAFETY: `cmdlen` bytes of the live `cmdbuff` allocation.
        unsafe { ::core::slice::from_raw_parts(from, n) }
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
    // SAFETY: appends to the typeahead.
    unsafe { stuffcharReadbuff(c) };
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
    if exec_from_reg.get() && unsafe { vpeekc() } == ':' as ::core::ffi::c_int {
        // SAFETY: consumes the byte `vpeekc` just reported.
        unsafe { vgetc() };
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
    cc.cmdpos >= cc.cmdlen
}

// ---------------------------------------------------------------------------
// The allocation.

/// Deallocate the command-line buffer, updating its size and length.
pub(crate) fn dealloc_cmdbuff() {
    let mut cc = Cc::current();
    free(cc.cmdbuff);
    cc.cmdbuff = ::core::ptr::null_mut();
    cc.cmdbufflen = 0;
    cc.cmdlen = 0;
}

/// Allocate a new command-line buffer into `ccline.cmdbuff`/`cmdbufflen`.
pub(crate) fn alloc_cmdbuff(mut len: ::core::ffi::c_int) {
    // Give some extra space to avoid having to allocate all the time.
    if len < 80 {
        len = 100;
    } else {
        len += 20;
    }
    let mut cc = Cc::current();
    cc.cmdbuff = alloc(len as size_t);
    cc.cmdbufflen = len;
}

/// Re-allocate the command line to `len` plus something extra.
///
/// This *moves* the buffer.  `xp_pattern` is the one pointer into it
/// upstream knows about and re-derives here; anything else holding a
/// pointer or an offset into `cmdbuff` across a call is a bug — the
/// completion code deliberately keeps indices rather than pointers for
/// that reason.
pub fn realloc_cmdbuff(len: ::core::ffi::c_int) {
    if len < Cc::current().cmdbufflen {
        return; // no need to resize
    }
    let old = Cc::current().cmdbuff;
    alloc_cmdbuff(len); // will get some more

    // There isn't always a NUL after the command, but it may need to be
    // there, so copy up to the NUL and add one.
    let cc = Cc::current();
    let (to, n) = (cc.cmdbuff, cc.cmdlen as size_t);
    // SAFETY: `cmdlen` bytes moved from the old allocation into the new one,
    // which `alloc_cmdbuff` has just made at least that long.
    unsafe { memmove(to as *mut _, old as *const _, n) };
    // SAFETY: one past the text, inside the new allocation.
    unsafe { *cc.cmdbuff.add(cc.cmdlen.max(0) as usize) = NUL as ::core::ffi::c_char };

    move_xp_pattern(cc, old);
    free(old);
}

/// If `xp_pattern` pointed inside the old `cmdbuff` it has to be adjusted to
/// point into the newly allocated memory.
fn move_xp_pattern(mut cc: Cc, old: *mut ::core::ffi::c_char) {
    if cc.xpc.is_null() {
        return;
    }
    // SAFETY: a live completion context.
    let xpc = unsafe { &mut *cc.xpc };
    if xpc.xp_pattern.is_null()
        || xpc.xp_context == EXPAND_NOTHING
        || xpc.xp_context == EXPAND_UNSUCCESSFUL
    {
        return;
    }
    // SAFETY: as upstream -- `xp_pattern` either points into `old` or into
    // something else entirely, and the difference decides which.
    let i = unsafe { xpc.xp_pattern.offset_from(old) } as ::core::ffi::c_int;
    if i >= 0 && i <= cc.cmdlen {
        xpc.xp_pattern = cc.cmdbuff.wrapping_offset(i as isize);
    }
}

/// Save `ccline`, because obtaining the `=` register may execute
/// `normal :cmd` and overwrite it.
pub(crate) unsafe fn save_cmdline(ccp: *mut CmdlineInfo) {
    // SAFETY: the caller's promise -- a slot to save into.
    unsafe { *ccp = ccline.get() };
    ccline.set(CMDLINE_INFO_INIT);
    let mut cc = Cc::current();
    cc.prev_ccline = ccp;
    cc.cmdbuff = ::core::ptr::null_mut(); // signal that ccline is not in use
}

/// Restore `ccline` after it has been saved with [`save_cmdline`].
pub(crate) unsafe fn restore_cmdline(ccp: *mut CmdlineInfo) {
    // SAFETY: the caller's promise -- a saved command line.
    ccline.set(unsafe { *ccp });
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
    textlock.set(textlock.get() + 1);
    let got_special = special_reg(regname, &raw mut arg, &raw mut allocated);
    textlock.set(textlock.get() - 1);

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
    let end = cc.cmdbuff.wrapping_offset(cc.cursor());
    let mut w = end;
    while w > cc.cmdbuff {
        let len = head_off(cc.cmdbuff, w.wrapping_offset(-1)) + 1;
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

    let (buff, col, mincol) = (cc.cmdbuff, cc.cmdpos, spos as ::core::ffi::c_int);
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
