//! Writing a Vimscript file that restores the current state: `:mkexrc`,
//! `:mkvimrc`, `:mkview` and `:mksession`, plus the `:loadview` that reads a
//! view back.
//!
//! **The output is an on-disk format.** What these write is a script that
//! later gets `:source`d, sometimes by a much older or newer nvim, so every
//! byte of it is a contract: the window sizes are emitted as arithmetic on
//! `&lines`/`&columns` so a session restores into a differently-sized
//! terminal, `badd +N` carries a line number, and `normal! 0N|` carries a
//! column. A change that keeps the file semantically right but moves a digit
//! is still a breaking change.
//!
//! Four commands, two option sets, one writer:
//!
//! - `:mkexrc` and `:mkvimrc` write mappings and options and nothing else.
//! - `:mkview` writes one window: [`view::put_view`], filtered by
//!   'viewoptions'.
//! - `:mksession` writes the whole editor: [`session::makeopens`], filtered
//!   by 'sessionoptions', which is `put_view` per window plus the buffer
//!   list, the argument list, the tab pages and the window layout.
//!
//! Upstream distinguishes the two option sets by passing `&ssop_flags` or
//! `&vop_flags` and then comparing the pointer back against one of them --
//! `flagp == &ssop_flags` reads as "this is a session, not a view".
//! [`SessionOpts`] names that choice instead, so nothing here needs the
//! address of an option word.
//!
//! [`SessionFile`] is the other half of the same idea. Every function here
//! writes to one `FILE *` and answers "did the write succeed"; wrapping the
//! handle once, with the contract stated at [`SessionFile::new`], makes the
//! hundred-odd `fprintf` sites safe calls. Only the ones that write *bytes
//! from the editor* -- a file name, a tag, an option value -- stay unsafe
//! and go through `fputs`, because those are not necessarily UTF-8 and Rust
//! formatting would replace what is not.
//!
//! Original: `src/nvim/ex_session.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

mod session;
mod view;

use crate::arglist::alist_name;
use crate::ascii::ascii_isdigit;
use crate::autocmd::{EVENT_SESSIONWRITEPOST, apply_autocmds};
use crate::buffer::{bt_help, bt_nofilename, bt_terminal};
use crate::eval::vars::set_vim_var_string;
use crate::ex_docmd::{open_exfile, vim_mkdir_emsg};
use crate::ex_getln::vim_strsave_fnameescape;
use crate::file_search::vim_chdirfile;
use crate::fileio::shorten_fnames;
use crate::global_cell::GlobalCell;
use crate::main::{
    curbuf, curtab, curwin, e_noname, e_notopen, e_prev_dir, e_write, globaldir, no_hlsearch,
    p_acd, p_hls, p_vdir, ssop_flags, vop_flags,
};
use crate::mapping::makemap;
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xfree, xmalloc, xmemcpyz};
use crate::message::emsg;
use crate::option::makeset;
use crate::options::{
    OptSsopFlags, kOptSsopFlagBlank, kOptSsopFlagCurdir, kOptSsopFlagHelp, kOptSsopFlagOptions,
    kOptSsopFlagSesdir, kOptSsopFlagSkiprtp, kOptSsopFlagTerminal,
};
use crate::os::cshim::{gettext, putc};
use crate::os::env::home_replace_save;
use crate::os::fs::{os_chdir, os_dirname, os_isdir};
use crate::path::{add_pathsep, vim_FullName, vim_ispathsep};
use crate::runtime::do_source;
use crate::semsg_c;
use crate::types::{
    CMD_mksession, CMD_mkview, CMD_mkvimrc, CdCause, FILE, VV_THIS_SESSION, aentry_T, buf_T,
    exarg_T, garray_T, size_t, win_T,
};
use ::libc::{fclose, fprintf, fputs, strcpy, strlen};
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::{fmt, ptr};

use session::makeopens;
use view::put_view;

/// Constants the transpiler copied in from the headers this module includes.
mod flag {
    use super::{CdCause, c_char, c_int, c_uint};

    /// `vim_chdirfile`'s reason code: not a `:cd`, so no autocommand.
    pub const kCdCauseOther: CdCause = -1;
    /// `vim_strsave_fnameescape`: escape for an Ex command line.
    pub const VSE_NONE: c_int = 0;
    /// `do_source`: this is not a vimrc.
    pub const DOSO_NONE: c_int = 0;
    /// `makeset` scopes.
    pub const OPT_GLOBAL: c_uint = 1;
    pub const OPT_LOCAL: c_uint = 2;
    /// `makeset`: skip 'runtimepath' and 'packpath'.
    pub const OPT_SKIPRTP: c_uint = 128;

    pub const OK: c_int = 1;
    pub const FAIL: c_int = 0;
    pub const NUL: c_char = 0;
    pub const MAXPATHL: c_int = 4096;

    /// Frame layouts.
    pub const FR_LEAF: c_char = 0;
    pub const FR_COL: c_char = 2;
}

use flag::{DOSO_NONE, FAIL, MAXPATHL, NUL, OK, OPT_GLOBAL, OPT_SKIPRTP, VSE_NONE};

/// The default file name of each command that has one.
const SESSION_FILE: &CStr = c"Session.vim";
const VIMRC_FILE: &CStr = c".nvimrc";
const EXRC_FILE: &CStr = c".exrc";

/// Whether a `:lcd` or `:tcd` has been written for this session. Once one
/// has, short file names are no longer safe: the script's working directory
/// at the point a later name is read back is no longer known.
static did_lcd: GlobalCell<bool> = GlobalCell::new(false);

// -- The two option sets ---------------------------------------------------

/// Which option word filters what gets written.
///
/// Upstream threads a `unsigned *` and asks `flagp == &ssop_flags`; this is
/// the same question with a name. The distinction is not only which flags
/// apply: a session knows the working directory it will be sourced in and a
/// view does not, so several decisions turn on the *kind* rather than on any
/// flag.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionOpts {
    /// 'sessionoptions', for `:mksession` -- and for `:mkvimrc`/`:mkexrc`,
    /// which never reach the parts that read a flag.
    Session,
    /// 'viewoptions', for `:mkview`.
    View,
}

impl SessionOpts {
    /// The option word's current value.
    fn flags(self) -> OptSsopFlags {
        match self {
            Self::Session => ssop_flags.get(),
            Self::View => vop_flags.get(),
        }
    }

    /// Whether every bit of `mask` is set.
    pub(crate) fn has(self, mask: OptSsopFlags) -> bool {
        self.flags() & mask != 0
    }

    /// Whether this is a session rather than a view.
    pub(crate) fn is_session(self) -> bool {
        self == Self::Session
    }
}

// -- The file being written ------------------------------------------------

/// The session file, and the one place its `FILE *` is dereferenced.
///
/// Every method answers `true` when the write succeeded, which is the
/// `OK`/`FAIL` of the C inverted into the shape `?`-free code wants.
#[derive(Clone, Copy)]
pub(crate) struct SessionFile(*mut FILE);

impl SessionFile {
    /// # Safety
    /// `fd` must be a stream open for writing that outlives this value.
    /// Nothing else may write to it meanwhile.
    pub(crate) unsafe fn new(fd: *mut FILE) -> Self {
        Self(fd)
    }

    /// The raw handle, for the writers that live in other modules
    /// (`makemap`, `makeset`, `put_folds`).
    pub(crate) fn raw(self) -> *mut FILE {
        self.0
    }

    /// Write `s` and a newline: C's `put_line`.
    pub(crate) fn line(self, s: &CStr) -> bool {
        self.puts(s) && self.eol()
    }

    /// Write a newline: C's `put_eol`.
    pub(crate) fn eol(self) -> bool {
        // SAFETY: the handle is open for writing, per `new`.
        unsafe { putc(b'\n' as c_int, self.0) >= 0 }
    }

    /// Write a literal's bytes.
    pub(crate) fn puts(self, s: &CStr) -> bool {
        // SAFETY: as above; `s` is NUL-terminated by construction.
        unsafe { fputs(s.as_ptr(), self.0) >= 0 }
    }

    /// Write formatted text. Every call site formats numbers into a literal
    /// template, so the result is ASCII and holds no interior NUL; anything
    /// carrying editor bytes uses [`Self::bytes`] instead.
    pub(crate) fn write(self, args: fmt::Arguments<'_>) -> bool {
        let mut text = args.to_string();
        text.push('\0');
        // SAFETY: as above; `text` is NUL-terminated and outlives the call.
        unsafe { fputs(text.as_ptr().cast::<c_char>(), self.0) >= 0 }
    }

    /// Write a C string's bytes verbatim.
    ///
    /// # Safety
    /// `p` must be NUL-terminated.
    pub(crate) unsafe fn bytes(self, p: *const c_char) -> bool {
        // SAFETY: as above, plus the caller's contract on `p`.
        unsafe { fputs(p, self.0) >= 0 }
    }
}

/// `put_eol()`, for the option and mapping writers that still take a raw
/// handle.
///
/// # Safety
/// `fd` is open for writing.
pub unsafe fn put_eol(fd: *mut FILE) -> c_int {
    // SAFETY: caller contract.
    if unsafe { putc(b'\n' as c_int, fd) } < 0 {
        return FAIL;
    }
    OK
}

/// `put_line()`: `s` followed by a newline.
///
/// # Safety
/// `fd` is open for writing and `s` NUL-terminated.
pub unsafe fn put_line(fd: *mut FILE, s: *mut c_char) -> c_int {
    // SAFETY: caller contract.
    if unsafe { fprintf(fd, c"%s\n".as_ptr(), s) } < 0 {
        return FAIL;
    }
    OK
}

// -- File names ------------------------------------------------------------

/// The buffer name to write for `buf`.
///
/// The short name is only usable when the working directory at the moment
/// the session is sourced is known -- so not for a view, not under 'acd',
/// and not once a `:lcd` has been written.
///
/// # Safety
/// `buf` is a live buffer.
unsafe fn ses_get_fname(buf: *mut buf_T, opts: SessionOpts) -> *mut c_char {
    // SAFETY: caller contract.
    unsafe {
        if !(*buf).b_sfname.is_null()
            && opts.is_session()
            && opts.has(kOptSsopFlagCurdir | kOptSsopFlagSesdir)
            && p_acd.get() == 0
            && !did_lcd.get()
        {
            return (*buf).b_sfname;
        }
        (*buf).b_ffname
    }
}

/// Write `buf`'s name, and a newline when `add_eol`.
///
/// # Safety
/// `buf` is a live buffer.
unsafe fn ses_fname(out: SessionFile, buf: *mut buf_T, opts: SessionOpts, add_eol: bool) -> bool {
    // SAFETY: caller contract.
    unsafe {
        let name = ses_get_fname(buf, opts);
        ses_put_fname(out, name) && (!add_eol || out.eol())
    }
}

/// `name` with `$HOME` shortened to `~`, backslashes turned into forward
/// slashes (the legacy `slash` flag is always on) and the characters a
/// command line would eat escaped. Owned by the caller.
///
/// # Safety
/// `name` is NUL-terminated.
unsafe fn ses_escape_fname(name: *mut c_char) -> *mut c_char {
    // SAFETY: caller contract; `home_replace_save` answers an owned,
    // NUL-terminated copy, and `utfc_ptr2len` advances by a whole character
    // so the scan never lands inside one.
    unsafe {
        let sname = home_replace_save(ptr::null_mut::<buf_T>(), name);
        let mut p = sname;
        while *p != NUL {
            if *p == b'\\' as c_char {
                *p = b'/' as c_char;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        let escaped = vim_strsave_fnameescape(sname, VSE_NONE);
        xfree(sname.cast::<c_void>());
        escaped
    }
}

/// Write `name` as an escaped file name.
///
/// # Safety
/// `name` is NUL-terminated.
unsafe fn ses_put_fname(out: SessionFile, name: *mut c_char) -> bool {
    // SAFETY: caller contract; the escaped copy is NUL-terminated and freed
    // here.
    unsafe {
        let p = ses_escape_fname(name);
        let ok = out.bytes(p);
        xfree(p.cast::<c_void>());
        ok
    }
}

/// Write an argument list: the `cmd` that selects which one, `%argdel` to
/// empty it, then one `$argadd` per entry. Entries with no name are skipped
/// (which only happens out of memory).
///
/// # Safety
/// `gap` is a live `aentry_T` garray.
unsafe fn ses_arglist(out: SessionFile, cmd: &CStr, gap: *mut garray_T, fullname: bool) -> bool {
    if !out.puts(cmd) || !out.eol() || !out.line(c"%argdel") {
        return false;
    }
    // SAFETY: caller contract; each entry's name is NUL-terminated.
    unsafe {
        for i in 0..(*gap).ga_len {
            let mut name = alist_name(((*gap).ga_data as *mut aentry_T).offset(i as isize));
            if name.is_null() {
                continue;
            }
            let mut full = ptr::null_mut::<c_char>();
            if fullname {
                full = xmalloc(MAXPATHL as size_t).cast::<c_char>();
                vim_FullName(name, full, MAXPATHL as size_t, false);
                name = full;
            }
            let escaped = ses_escape_fname(name);
            let ok = out.puts(c"$argadd ") && out.bytes(escaped) && out.eol();
            xfree(escaped.cast::<c_void>());
            xfree(full.cast::<c_void>());
            if !ok {
                return false;
            }
        }
    }
    true
}

/// Whether window `wp` belongs in the session at all. A floating window
/// never does (#18432); the rest is what 'sessionoptions' says about the
/// kind of buffer it holds.
///
/// # Safety
/// `wp` is a live window.
pub(crate) unsafe fn ses_do_win(wp: *mut win_T) -> bool {
    // SAFETY: caller contract; a window always has a buffer.
    unsafe {
        if (*wp).w_floating {
            return false;
        }
        let buf = (*wp).w_buffer;
        if (*buf).b_fname.is_null()
            // The contents of a "nofile" buffer cannot be restored.
            || ((*buf).terminal.is_null() && bt_nofilename(buf))
        {
            return ssop_flags.get() & kOptSsopFlagBlank != 0;
        }
        if bt_help(buf) {
            return ssop_flags.get() & kOptSsopFlagHelp != 0;
        }
        if bt_terminal(buf) {
            return ssop_flags.get() & kOptSsopFlagTerminal != 0;
        }
        true
    }
}

// -- `:loadview` -----------------------------------------------------------

/// `:loadview [nr]`.
///
/// # Safety
/// `eap` is the current Ex command.
pub unsafe fn ex_loadview(eap: *mut exarg_T) {
    // SAFETY: caller contract; `fname` is owned and NUL-terminated.
    unsafe {
        let fname = get_view_file(*(*eap).arg);
        if fname.is_null() {
            return;
        }
        if do_source(fname, false, DOSO_NONE, ptr::null_mut()) == FAIL {
            semsg_c!(gettext(e_notopen.as_ptr()), fname);
        }
        xfree(fname.cast::<c_void>());
    }
}

/// The name of the view file for the current buffer, in 'viewdir'.
///
/// The buffer's path is flattened into a single name, since no directory is
/// created for it: a path separator becomes `=+`, an `=` becomes `==`. The
/// digit `c` (or NUL for `:mkview` with no argument) and `.vim` follow.
///
/// **`.vim` lands past the terminator when `c` is NUL**, so the name on disk
/// ends in `=`. Upstream does exactly this and `:loadview` calls the same
/// function, so views round-trip; do not "fix" it, every existing 'viewdir'
/// depends on it.
///
/// # Safety
/// Main thread; `curbuf` is live.
unsafe fn get_view_file(c: c_char) -> *mut c_char {
    // SAFETY: `curbuf` is live, 'viewdir' is a NUL-terminated option string,
    // and `retval` is sized below for every byte written into it.
    unsafe {
        if (*curbuf.get()).b_ffname.is_null() {
            emsg(gettext(e_noname.as_ptr()));
            return ptr::null_mut();
        }
        let sname = home_replace_save(ptr::null_mut::<buf_T>(), (*curbuf.get()).b_ffname);

        // One extra byte for each character that doubles.
        let mut extra = 0usize;
        let mut p = sname;
        while *p != NUL {
            if *p == b'=' as c_char || vim_ispathsep(*p as c_int) {
                extra += 1;
            }
            p = p.offset(1);
        }

        let retval = xmalloc(strlen(sname) + extra + strlen(p_vdir.get()) + 9).cast::<c_char>();
        strcpy(retval, p_vdir.get());
        add_pathsep(retval);
        let mut s = retval.add(strlen(retval));
        p = sname;
        while *p != NUL {
            if *p == b'=' as c_char {
                *s = b'=' as c_char;
                *s.offset(1) = b'=' as c_char;
                s = s.offset(2);
            } else if vim_ispathsep(*p as c_int) {
                *s = b'=' as c_char;
                *s.offset(1) = b'+' as c_char;
                s = s.offset(2);
            } else {
                *s = *p;
                s = s.offset(1);
            }
            p = p.offset(1);
        }
        *s = b'=' as c_char;
        *s.offset(1) = c;
        s = s.offset(2);
        xmemcpyz(s.cast::<c_void>(), c".vim".as_ptr().cast::<c_void>(), 4);

        xfree(sname.cast::<c_void>());
        retval
    }
}

// -- `:mkexrc`, `:mkvimrc`, `:mkview`, `:mksession` ------------------------

/// The four `:mk*` commands.
///
/// Two legacy 'sessionoptions'/'viewoptions' flags are always on: line
/// endings are LF, and file names are written with `/`.
///
/// # Safety
/// `eap` is the current Ex command with a NUL-terminated argument.
pub unsafe fn ex_mkrc(eap: *mut exarg_T) {
    // SAFETY: caller contract.
    let cmdidx = unsafe { (*eap).cmdidx };
    // `:mkview` and `:mksession` write a *state*; the other two write only
    // mappings and options.
    let view_session = cmdidx == CMD_mksession || cmdidx == CMD_mkview;

    // Short file names are usable until a ":lcd" is written. They are also
    // not used when 'acd' is set, which is checked separately.
    did_lcd.set(false);

    // ":mkview" or ":mkview 9": the name comes from 'viewdir'.
    let mut view_file = ptr::null_mut::<c_char>();
    // SAFETY: caller contract; `eap.arg` is NUL-terminated.
    let fname = unsafe {
        let arg = (*eap).arg;
        if cmdidx == CMD_mkview
            && (*arg == NUL || (ascii_isdigit(*arg as c_int) && *arg.offset(1) == NUL))
        {
            (*eap).forceit = 1;
            view_file = get_view_file(*arg);
            if view_file.is_null() {
                return;
            }
            // The 'viewdir' may still need creating.
            if !os_isdir(p_vdir.get()) {
                vim_mkdir_emsg(p_vdir.get(), 0o755);
            }
            view_file
        } else if *arg != NUL {
            arg
        } else if cmdidx == CMD_mkvimrc {
            VIMRC_FILE.as_ptr().cast_mut()
        } else if cmdidx == CMD_mksession {
            SESSION_FILE.as_ptr().cast_mut()
        } else {
            EXRC_FILE.as_ptr().cast_mut()
        }
    };
    let using_vdir = !view_file.is_null();

    // SAFETY: `fname` is NUL-terminated, and `fd` is used only while open.
    unsafe {
        let fd = open_exfile(fname, (*eap).forceit, c"wb".as_ptr().cast_mut());
        if !fd.is_null() {
            let out = SessionFile::new(fd);
            let failed = write_rc(out, eap, fname, view_session, using_vdir);
            // `fclose` answers nonzero on a write error the buffering hid,
            // and must run whether or not anything failed above.
            let close_failed = fclose(fd) != 0;
            if failed || close_failed {
                emsg(gettext(e_write.as_ptr()));
            } else if cmdidx == CMD_mksession {
                // A successful session write sets v:this_session.
                let full = xmalloc(MAXPATHL as size_t).cast::<c_char>();
                if vim_FullName(fname, full, MAXPATHL as size_t, false) == OK {
                    set_vim_var_string(VV_THIS_SESSION, full, -1);
                }
                xfree(full.cast::<c_void>());
            }
        }
        xfree(view_file.cast::<c_void>());
        apply_autocmds(
            EVENT_SESSIONWRITEPOST,
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            curbuf.get(),
        );
    }
}

/// The body of [`ex_mkrc`] once the file is open: answers whether anything
/// failed. The trailing modeline is written either way, as upstream does.
///
/// # Safety
/// `eap` is the current Ex command and `fname` the name `out` was opened
/// under.
unsafe fn write_rc(
    out: SessionFile,
    eap: *mut exarg_T,
    fname: *mut c_char,
    view_session: bool,
    using_vdir: bool,
) -> bool {
    // SAFETY: caller contract.
    let cmdidx = unsafe { (*eap).cmdidx };
    let opts = if cmdidx == CMD_mkview {
        SessionOpts::View
    } else {
        SessionOpts::Session
    };
    let mut failed = false;

    if cmdidx == CMD_mkvimrc {
        // Upstream ignores this one write's result.
        let _ = out.line(c"version 6.0");
    }
    if cmdidx == CMD_mksession && !out.line(c"let SessionLoad = 1") {
        failed = true;
    }

    // Mappings and options: everything for the two rc commands, and for
    // `:mksession` only when "options" is in 'sessionoptions'.
    if !view_session || (cmdidx == CMD_mksession && opts.has(kOptSsopFlagOptions)) {
        let mut flags = OPT_GLOBAL;
        if cmdidx == CMD_mksession && opts.has(kOptSsopFlagSkiprtp) {
            flags |= OPT_SKIPRTP;
        }
        // SAFETY: both writers take the open handle and nothing else.
        failed |= unsafe {
            makemap(out.raw(), ptr::null_mut()) == FAIL
                || makeset(out.raw(), flags as c_int, 0) == FAIL
        };
    }

    if !failed && view_session {
        if !out.line(
            c"let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1",
        ) {
            failed = true;
        }
        if cmdidx == CMD_mksession {
            // SAFETY: `fname` is NUL-terminated.
            failed |= unsafe { !write_session(out, fname) };
        } else {
            // SAFETY: `curwin`/`curtab` are live.
            failed |= unsafe { !put_view(out, curwin.get(), curtab.get(), !using_vdir, opts, -1) };
        }
        if !out.line(c"let &g:so = s:so_save | let &g:siso = s:siso_save") {
            failed = true;
        }
        if p_hls.get() != 0 && !out.line(c"set hlsearch") {
            failed = true;
        }
        if no_hlsearch.get() && !out.line(c"nohlsearch") {
            failed = true;
        }
        if !out.line(c"doautoall SessionLoadPost") {
            failed = true;
        }
        if cmdidx == CMD_mksession && !out.line(c"unlet SessionLoad") {
            failed = true;
        }
    }
    if !out.line(c"\" vim: set ft=vim :") {
        failed = true;
    }
    failed
}

/// `:mksession`'s body: change to whatever directory the file names should
/// be relative to, write the session, and change back.
///
/// # Safety
/// `fname` is the NUL-terminated name of the session file.
unsafe fn write_session(out: SessionFile, fname: *mut c_char) -> bool {
    // SAFETY: `dirnow` is our own MAXPATHL buffer, and `fname` is the
    // caller's.
    unsafe {
        let dirnow = xmalloc(MAXPATHL as size_t).cast::<c_char>();
        if os_dirname(dirnow, MAXPATHL as size_t) == FAIL || os_chdir(dirnow) != 0 {
            *dirnow = NUL;
        }
        let known = *dirnow != NUL;
        let to_sesdir = known && ssop_flags.get() & kOptSsopFlagSesdir != 0;
        let to_globaldir =
            known && ssop_flags.get() & kOptSsopFlagCurdir != 0 && !globaldir.get().is_null();
        if to_sesdir {
            if vim_chdirfile(fname, flag::kCdCauseOther) == OK {
                shorten_fnames(1);
            }
        } else if to_globaldir && os_chdir(globaldir.get()) == 0 {
            shorten_fnames(1);
        }

        let ok = makeopens(out, dirnow);

        // Restore the original directory.
        if to_sesdir || to_globaldir {
            if os_chdir(dirnow) != 0 {
                emsg(gettext(e_prev_dir.as_ptr()));
            }
            shorten_fnames(1);
        }
        xfree(dirnow.cast::<c_void>());
        ok
    }
}
