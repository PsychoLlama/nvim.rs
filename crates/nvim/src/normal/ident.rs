//! The identifier under the cursor, and the commands that look it up:
//! tags, `:help`, 'keywordprg', a declaration, a file name.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::identfind::find_ident_under_cursor;
use crate::cstr;
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::newlnum;
use crate::strings::has_char;
use crate::winlayer::{Buf, Win};
use core::ptr;

use crate::buffer::buf_hide;
use crate::charset::{skipwhite, vim_iswordp};
use crate::cmdhist::{add_to_history, init_history};
use crate::cursor::{check_cursor_lnum, get_cursor_line_ptr};
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds::do_ecmd;
use crate::ex_cmds2::autowrite;
use crate::ex_docmd::do_cmdline_cmd;
use crate::ex_getln::vim_strsave_fnameescape;
use crate::file_search::grab_file_name;
use crate::keycodes::Ctrl_RSB;
use crate::mapping::add_map;
use crate::mark::setpcmark;
use crate::mbyte::{mb_prevptr, utfc_ptr2len};
use crate::memory::{strequal, xfree, xmalloc, xrealloc};
use crate::message::e_noident;
use crate::message::emsg;
use crate::normal::{
    CmdArgRef, DT_POP, FIND_IDENT, FIND_STRING, HIST_SEARCH, POUND, VSE_NONE, check_clear_op_quit,
    check_text_or_curbuf_locked, clear_op, get_visual_text, normal_search, visual_active,
};
use crate::ops::clear_oparg;
use crate::option::magic_isset;
use crate::option::vars::p_kp;
use crate::os::cshim::{gettext, snprintf};
use crate::search::state::no_smartcase;
use crate::state::MODE_TERMINAL;
use crate::state::mode::restart_edit;
use crate::strings::{vim_strsave_shellescape, xstrnsave};
use crate::tag::do_tag;
use crate::tag::state::g_tag_at_cursor;
use crate::types::{CmdArg, ColNr, LineNr, NUL, OpArg, int64_t, size_t, uint8_t};
use crate::undo::curbuf_is_changed;
use crate::window::check_can_set_curbuf_disabled;
use ::libc::strcpy;
use core::ffi::{CStr, c_char, c_int, c_void};

/// Run one of the identifier commands from outside the command loop, with a
/// command argument built for it. `CTRL-W ]` and friends use this.
pub(crate) fn do_nv_ident(c1: c_int, c2: c_int) {
    // SAFETY: both structures are plain data, and both are filled in before
    // `nv_ident` reads them.
    let mut oa: OpArg = unsafe { core::mem::zeroed() };
    let mut ca: CmdArg = unsafe { core::mem::zeroed() };
    unsafe { clear_oparg(&raw mut oa) };
    ca.oap = &raw mut oa;
    ca.cmdchar = c1;
    ca.nchar = c2;
    unsafe { nv_ident(&raw mut ca) };
}

/// The command line `nv_ident` builds: `size` writable bytes with the first
/// `len` in use, always NUL-terminated. Freed by hand -- `build_keywordprg_cmd`
/// hands it back already freed when there is nothing to look up.
struct CmdBuf {
    ptr: *mut c_char,
    size: size_t,
    len: size_t,
}

impl CmdBuf {
    /// An empty command of `size` bytes.
    fn new(size: size_t) -> Self {
        // SAFETY: `xmalloc` answers `size` writable bytes and never null.
        let ptr = unsafe { xmalloc(size) } as *mut c_char;
        // SAFETY: `size` is well over the one byte a terminator needs.
        unsafe { *ptr = NUL as c_char };
        Self { ptr, size, len: 0 }
    }
    /// The command so far, NUL-terminated.
    fn as_ptr(&self) -> *mut c_char {
        self.ptr
    }
    /// How many of its bytes are in use.
    fn used(&self) -> size_t {
        self.len
    }
    /// Where the next byte goes.
    fn tail(&self) -> *mut c_char {
        // SAFETY: `len` of the `size` bytes are in use, so this is in bounds.
        unsafe { self.ptr.add(self.len) }
    }
    /// Replace the command with `s`.
    fn set(&mut self, s: &CStr) {
        self.len = 0;
        self.append(s);
    }
    /// Append `s` as it stands.
    fn append(&mut self, s: &CStr) {
        // SAFETY: the buffer was sized for the whole command, `s` included.
        unsafe { strcpy(self.tail(), s.as_ptr()) };
        self.len += s.count_bytes() as size_t;
    }
    /// Append `s` through `snprintf`, which truncates rather than overruns.
    fn push(&mut self, s: &CStr) {
        self.push_str(c"%s", s.as_ptr());
    }
    /// Append `fmt` filled with one NUL-terminated string.
    fn push_str(&mut self, fmt: &CStr, arg: *const c_char) {
        // SAFETY: the tail has the `size - len` bytes `snprintf` is told of,
        // and `arg` matches the one `%s` in `fmt`.
        let wrote = unsafe { snprintf(self.tail(), self.size - self.len, fmt.as_ptr(), arg) };
        self.len += wrote as size_t;
    }
    /// Append `fmt` filled with one number.
    fn push_num(&mut self, fmt: &CStr, arg: int64_t) {
        // SAFETY: as `push_str`; `arg` matches the one `%ld` in `fmt`.
        let wrote = unsafe { snprintf(self.tail(), self.size - self.len, fmt.as_ptr(), arg) };
        self.len += wrote as size_t;
    }
    /// Resize around what is in the buffer plus `n` more bytes, and append
    /// them from `s`.
    fn append_grown(&mut self, s: *const c_char, n: size_t) {
        let size = self.len + n + 1;
        // SAFETY: it came from `xmalloc`, and `size` holds what is in it now
        // plus `s` and a terminator.
        self.ptr = unsafe { xrealloc(self.ptr as *mut c_void, size) } as *mut c_char;
        self.size = size;
        // SAFETY: the tail now has room for `s`'s `n` bytes and its NUL.
        unsafe { strcpy(self.tail(), s) };
        self.len += n;
    }
    /// Take the length from where an in-place append stopped.
    fn ends_at(&mut self, end: *mut c_char) {
        // SAFETY: `end` is inside the same allocation `self.ptr` addresses.
        self.len = unsafe { end.offset_from(self.ptr) } as size_t;
    }
    /// Release the buffer. The command must not be used again.
    fn free(&mut self) {
        // SAFETY: the buffer came from `xmalloc`/`xrealloc` above.
        unsafe { xfree(self.ptr as *mut c_void) };
        self.ptr = ptr::null_mut();
        self.size = 0;
        self.len = 0;
    }
}

/// Build the command `K` should run into `out`. Answers the length of the
/// identifier still to be appended, or 0 when there is nothing to look up --
/// in which case `out` has already been freed.
///
/// # Safety
///
/// `cmd_arg` must point at the command's `CmdArg`, live for the call. `kp`
/// must point at the NUL-terminated 'keywordprg', and `ptr_arg` at a `*mut
/// c_char` holding the identifier's first of `n` bytes -- it is advanced past
/// what is consumed, so both must stay live until the call returns.
#[allow(clippy::too_many_arguments)]
unsafe fn build_keywordprg_cmd(
    cmd_arg: *mut CmdArg,
    kp: *mut c_char,
    kp_help: bool,
    kp_ex: bool,
    ptr_arg: &mut *mut c_char,
    mut n: size_t,
    out: &mut CmdBuf,
) -> size_t {
    // SAFETY (throughout): `cmd_arg` is live, and nothing below reaches back into it.
    let count0 = unsafe { (*cmd_arg).count0 };
    if kp_help {
        out.set(c"help! ");
        return n;
    }
    if kp_ex {
        // An Ex 'keywordprg' takes the word as its argument, after an
        // optional count.
        out.push_str(c"%s ", kp);
        if count0 != 0 {
            out.push_num(c"%ld ", count0 as int64_t);
        }
        return n;
    }

    // A shell 'keywordprg' runs in a terminal in a new tab. Leading
    // dashes would look like options to it.
    let mut word = *ptr_arg;
    // SAFETY: `word` walks the identifier, which is NUL-terminated.
    while unsafe { *word } as c_int == '-' as c_int && n > 0 {
        // SAFETY: the byte at `word` is a `-`, so the next one is in the line.
        word = unsafe { word.offset(1) };
        n -= 1;
    }
    if n == 0 {
        emsg(gettext(e_noident));
        out.free();
        *ptr_arg = word;
        return 0;
    }
    // `man` and `man -s` take the count as a section number, which goes
    // in front of the word rather than becoming a line range.
    // SAFETY: 'keywordprg' is a NUL-terminated option string.
    let isman = unsafe { cstr::eq_bytes(kp, b"man") };
    let isman_s = unsafe { cstr::eq_bytes(kp, b"man -s") };
    if count0 != 0 && !(isman || isman_s) {
        out.push_num(c".,.+%ld", (count0 - 1) as int64_t);
    }
    // SAFETY: a NUL-terminated literal command.
    let _ = unsafe { do_cmdline_cmd(c"tabnew".as_ptr()) };
    out.push(c"terminal ");
    if count0 == 0 && isman_s {
        // `man -s` with no section is just `man`.
        out.push(c"man ");
    } else {
        out.push_str(c"%s ", kp);
    }
    if count0 != 0 && (isman || isman_s) {
        out.push_num(c"%ld ", count0 as int64_t);
    }
    *ptr_arg = word;
    n
}

/// The characters that have to be backslash-escaped for the command being
/// built, given what it is.
fn ident_escapes(cmdchar: c_int, tag_cmd: bool) -> &'static CStr {
    if cmdchar == '*' as c_int {
        return if magic_isset() {
            c"/.*~[^$\\"
        } else {
            c"/^$\\"
        };
    }
    if cmdchar == '#' as c_int {
        return if magic_isset() {
            c"/?.*~[^$\\"
        } else {
            c"/?^$\\"
        };
    }
    if !tag_cmd {
        return c"\\|\"\n*?[";
    }
    // A help tag may contain any of these, so nothing is escaped.
    // SAFETY: 'filetype' is a NUL-terminated option string.
    if unsafe { cstr::eq_bytes(Buf::current().b_p_ft, b"help") } {
        c""
    } else {
        c"\\|\"\n["
    }
}

/// Copy the byte `src` points at to where `dst` points, advancing both.
/// # Safety
/// `*src` addresses a readable byte and `*dst` a writable one.
unsafe fn copy_byte(dst: &mut *mut c_char, src: &mut *mut c_char) {
    // SAFETY: the caller promises both bytes, and one past each is an
    // address a pointer may hold.
    unsafe { **dst = **src };
    (*dst, *src) = unsafe { (dst.offset(1), src.offset(1)) };
}

/// Copy `n` bytes of the identifier into the command being built, escaping
/// whatever the command cannot take literally.
/// # Safety
/// `dest` has room for twice `n` plus a terminator, which is what `nv_ident`
/// sized the buffer for, and `*src` is `n` bytes of a NUL-terminated line.
unsafe fn append_escaped(
    dest: *mut c_char,
    src: &mut *mut c_char,
    mut n: size_t,
    escapes: &CStr,
) -> *mut c_char {
    let mut out = dest;
    // The caller reads the source pointer back: `*` and `#` ask whether
    // the character *before* where this stopped was a word character, to
    // decide whether the search anchors at the end too.
    let p = src;
    while n > 0 {
        n -= 1;
        // SAFETY: `p` walks the identifier, which is NUL-terminated.
        let c = unsafe { **p } as uint8_t as c_int;
        // SAFETY: `escapes` is a NUL-terminated literal.
        if has_char(escapes, c) {
            // SAFETY: the caller promises the room.
            unsafe { *out = '\\' as c_char };
            out = unsafe { out.offset(1) };
        }
        // `utfc_ptr2len` answers 0 at a NUL, so this is `(size_t)-1`
        // there and the inner loop then runs until `n` is spent --
        // upstream's behaviour, and what stops a NUL ending the copy
        // early.
        // SAFETY: `p` points at a character of a NUL-terminated line.
        let trailing = (unsafe { utfc_ptr2len(*p) } - 1) as size_t;
        let mut i: size_t = 0;
        while i < trailing && n > 0 {
            // SAFETY: as above, and `out` still has the caller's room.
            unsafe { copy_byte(&mut out, p) };
            i += 1;
            n -= 1;
        }
        unsafe { copy_byte(&mut out, p) };
    }
    unsafe { *out = NUL as c_char };
    out
}

/// `*`, `#`, `K`, `]`, `CTRL-]` and their `g` forms: look up the identifier
/// under the cursor.
///
/// # Safety
///
/// `cmd_arg` must point at the command's `CmdArg`, unaliased for the call.
pub(crate) unsafe fn nv_ident(cmd_arg: *mut CmdArg) {
    // SAFETY: `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let (typed, nchar) = unsafe { ((*cmd_arg).cmdchar, (*cmd_arg).nchar) };
    // The `g` forms carry the real command in `nchar`.
    let g_cmd = typed == 'g' as c_int;
    let mut cmdchar = if g_cmd { nchar } else { typed };
    if cmdchar == POUND {
        cmdchar = '#' as c_int;
    }

    let mut word: *mut c_char = ptr::null_mut();
    let mut n: size_t = 0;
    // Three of the commands take a Visual selection instead of the word
    // under the cursor.
    let mut visual_sel = false;
    if cmdchar == ']' as c_int || cmdchar == Ctrl_RSB || cmdchar == 'K' as c_int {
        // SAFETY: `cmd_arg` is live and `word`/`n` are this frame's own.
        if visual_active() && !unsafe { get_visual_text(cmd_arg, &raw mut word, &raw mut n) } {
            return;
        }
        visual_sel = !word.is_null();
        if check_clear_op_quit(ca.op()) {
            return;
        }
    }
    if word.is_null() {
        let mut ident_offset: c_int = 0;
        // `*` and `#` fall back to the string under the cursor when there
        // is no identifier there.
        let searchy = cmdchar == '*' as c_int || cmdchar == '#' as c_int;
        let find_type = FIND_IDENT as c_int | if searchy { FIND_STRING as c_int } else { 0 };
        // SAFETY: both out-parameters are this frame's own.
        n = unsafe { find_ident_under_cursor(&raw mut word, find_type, &raw mut ident_offset) };
        if n == 0 {
            clear_op(ca.op());
            return;
        }
    }

    // 'keywordprg', which decides what `K` does.
    // SAFETY: 'keywordprg' is a NUL-terminated option string.
    let kp = if unsafe { *Buf::current().b_p_kp } as c_int == NUL {
        p_kp.get()
    } else {
        Buf::current().b_p_kp
    };
    // SAFETY: `kp` is NUL-terminated, as are the literals.
    let kp_helpbang = unsafe { strequal(kp, c":help!".as_ptr()) };
    let kp_help = kp_helpbang
        || unsafe { *kp } as c_int == NUL
        || unsafe { strequal(kp, c":he".as_ptr()) }
        || unsafe { strequal(kp, c":help".as_ptr()) };
    if kp_help && !kp_helpbang {
        // SAFETY: `word` points into a NUL-terminated buffer line.
        if unsafe { *skipwhite(word) } as c_int == NUL {
            emsg(gettext(e_noident));
            return;
        }
    }
    // SAFETY: `kp` is NUL-terminated.
    let kp_ex = unsafe { *kp } as c_int == ':' as c_int;

    // Room for the command, the word with every byte escaped, and a
    // terminator.
    let kplen = unsafe { cstr::bytes_at(kp) }.len();
    let mut out = CmdBuf::new(n.wrapping_mul(2).wrapping_add(30).wrapping_add(kplen));

    // Whether the command being built is a tag lookup, which decides the
    // escaping below.
    let mut tag_cmd = false;
    match u8::try_from(cmdchar) {
        Ok(b'*' | b'#') => {
            // These become a search, so the cursor moves to the start of
            // the word first.
            // SAFETY: `word` points into the cursor's own line.
            setpcmark();
            let col = unsafe { word.offset_from(get_cursor_line_ptr()) } as ColNr;
            Win::current().w_cursor.col = col;
            if !g_cmd && unsafe { vim_iswordp(word) } {
                // The plain forms anchor at a word boundary.
                out.set(c"\\<");
            }
            no_smartcase.set(true);
        }
        Ok(b'K') => {
            // SAFETY: all of these are live, and `word` is this frame's own.
            n = unsafe {
                build_keywordprg_cmd(cmd_arg, kp, kp_help, kp_ex, &mut word, n, &mut out)
            };
            if n == 0 {
                return;
            }
        }
        Ok(b']') => {
            tag_cmd = true;
            out.set(c"tselect ");
        }
        // CTRL-] and everything else: a plain tag jump.
        _ => {
            tag_cmd = true;
            let count0 = unsafe { (*cmd_arg).count0 };
            let cmd: &CStr = if Buf::current().b_help {
                c"help! "
            } else if g_cmd {
                c"tjump "
            } else if count0 == 0 {
                c"tag "
            } else {
                // A count picks which of the matching tags to jump to.
                out.push_num(c":%ldtag ", count0 as int64_t);
                c""
            };
            if !cmd.is_empty() {
                out.set(cmd);
            }
        }
    }

    if cmdchar == 'K' as c_int && kp_helpbang && !visual_sel {
        // `:help!` with no selection opens the help index rather than
        // looking anything up.
        out.set(c"help!");
    } else if cmdchar == 'K' as c_int && !kp_help {
        // A shell or Ex command takes the word quoted, not escaped
        // character by character.
        // SAFETY: `word` is `n` bytes of a buffer line.
        let owned = unsafe { xstrnsave(word, n) };
        // SAFETY: `owned` is a NUL-terminated copy of the word.
        let quoted = if kp_ex {
            unsafe { vim_strsave_fnameescape(owned, VSE_NONE as c_int) }
        } else {
            unsafe { vim_strsave_shellescape(owned, true, true) }
        };
        // SAFETY: `owned` and `quoted` came from the allocations above, and
        // `quoted` is NUL-terminated.
        unsafe { xfree(owned as *mut c_void) };
        let plen = unsafe { cstr::bytes_at(quoted) }.len();
        out.append_grown(quoted, plen);
        unsafe { xfree(quoted as *mut c_void) };
    } else {
        let escapes = ident_escapes(cmdchar, tag_cmd);
        // SAFETY: the buffer holds twice the word; `word` is `n` line bytes.
        let end = unsafe { append_escaped(out.tail(), &mut word, n, escapes) };
        out.ends_at(end);
    }

    if cmdchar == '*' as c_int || cmdchar == '#' as c_int {
        // SAFETY: `word` points into the cursor's own line.
        if !g_cmd && unsafe { vim_iswordp(mb_prevptr(get_cursor_line_ptr(), word)) } {
            out.append(c"\\>");
        }
        // The search goes into the history as if it had been typed.
        init_history();
        // SAFETY: the command holds `used()` bytes.
        let entry = unsafe { core::slice::from_raw_parts(out.as_ptr() as *const u8, out.used()) };
        add_to_history(HIST_SEARCH as c_int, entry, true, NUL as u8);
        let star = cmdchar == '*' as c_int;
        let dir = if star { '/' as c_int } else { '?' as c_int };
        let cmd = out.as_ptr();
        let used = out.used();
        unsafe { normal_search(cmd_arg, dir, cmd, used, 0, ptr::null_mut()) };
    } else {
        // `taglist()` and friends need to know the tag came from under
        // the cursor rather than from a command line.
        g_tag_at_cursor.set(true);
        let _ = unsafe { do_cmdline_cmd(out.as_ptr()) };
        g_tag_at_cursor.set(false);
        if cmdchar == 'K' as c_int && !kp_ex && !kp_help {
            // The terminal 'keywordprg' opened above: let <Esc> close it.
            restart_edit.set('i' as c_int);
            let lhs = c"<esc>".as_ptr() as *mut c_char;
            let rhs = c"<Cmd>bdelete!<CR>".as_ptr() as *mut c_char;
            // SAFETY: both sides are NUL-terminated literals.
            unsafe { add_map(lhs, rhs, MODE_TERMINAL, true) };
        }
    }
    out.free();
}

/// `CTRL-T`: back up the tag stack.
///
/// # Safety
///
/// `cmd_arg` must point at the command's `CmdArg`, unaliased for the call.
pub(crate) unsafe fn nv_tagpop(cmd_arg: *mut CmdArg) {
    // SAFETY: `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    if check_clear_op_quit(ca.op()) {
        return;
    }
    let none = c"".as_ptr() as *mut c_char;
    // SAFETY: `cmd_arg` is live and `none` is an empty NUL-terminated literal.
    unsafe { do_tag(none, DT_POP as c_int, (*cmd_arg).count1, 0, true) };
}

/// `gf`, `gF` and `[f`: edit the file named under the cursor.
///
/// # Safety
///
/// `cmd_arg` must point at the command's `CmdArg`, unaliased for the call.
pub(crate) unsafe fn nv_gotofile(cmd_arg: *mut CmdArg) {
    // SAFETY: `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument, and
    // the current window and buffer are live.
    if unsafe { check_text_or_curbuf_locked((*cmd_arg).oap) } || !check_can_set_curbuf_disabled() {
        return;
    }
    // `gF` also takes a line number off the end of the name.
    let mut lnum: LineNr = -1;
    // SAFETY: `lnum` is this frame's own out-parameter.
    let name = unsafe { grab_file_name((*cmd_arg).count1, &raw mut lnum) };
    if name.is_null() {
        clear_op(ca.op());
        return;
    }
    // Leaving the only window on a changed buffer that cannot be hidden
    // means writing it first.
    let must_write =
        curbuf_is_changed() && Buf::current().b_nwindows <= 1 && !buf_hide(Buf::current());
    if must_write {
        let _ = autowrite(Buf::current(), false);
    }
    setpcmark();
    let hidden = buf_hide(Buf::current());
    let hide = EcmdFlags::HIDE.when(hidden);
    let last = newlnum::LAST as LineNr;
    let win = Some(Win::current().id());
    // SAFETY: `name` is a NUL-terminated file name.
    let opened = unsafe { do_ecmd(0, name, ptr::null_mut(), None, last, hide, win) };
    if opened.is_ok() && unsafe { (*cmd_arg).nchar } == 'F' as c_int && lnum >= 0 {
        Win::current().w_cursor.lnum = lnum;
        check_cursor_lnum(Win::current());
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    }
    // SAFETY: `name` came from `grab_file_name`.
    unsafe { xfree(name as *mut c_void) };
}
