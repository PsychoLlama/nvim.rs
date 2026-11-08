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
//! [`SessionFile`] is the other half of the same idea: the file being
//! written, buffered, and every writer here answers "did the write succeed"
//! through it. Text from the editor -- a file name, a tag, an option value --
//! goes out as bytes, because it is not necessarily UTF-8; only the numbers
//! go through Rust formatting.
//!
//! Original: `src/nvim/ex_session.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

mod session;
mod view;

use crate::autocmd::fire_autocmds;
use crate::buffer::{buf_is_help, buf_is_nofilename, buf_is_terminal};
use crate::env::home_to_tilde;
use crate::eval::vars::set_vim_var_string;
use crate::ex_docmd::{create_exfile, vim_mkdir_emsg};
use crate::ex_getln::fnameescape;
use crate::file_search::vim_chdirfile;
use crate::fileio::{IoFailed, shorten_fnames};
use crate::global_cell::GlobalCell;
use crate::mapping::makemap;
use crate::mbyte::cluster_len;
use crate::memory::ThinCString;
use crate::memory::XString;
use crate::message::emsg;
use crate::message::{e_noname, e_prev_dir, e_write};
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::option::defs::OptionSetFlags;
use crate::option::makeset;
use crate::option::vars::{P_VDIR, p_acd, p_hls, ssop_flags, vop_flags};
use crate::options::{
    OptSsopFlags, kOptSsopFlagBlank, kOptSsopFlagCurdir, kOptSsopFlagHelp, kOptSsopFlagOptions,
    kOptSsopFlagSesdir, kOptSsopFlagSkiprtp, kOptSsopFlagTerminal,
};
use crate::os::cshim::gettext;
use crate::os::fs::{dir_exists, dirname_within, os_chdir};
use crate::path::MAXPATHL;
use crate::path::{after_pathsep, vim_full_name, vim_ispathsep};
use crate::runtime::do_source;
use crate::search::state::no_hlsearch;
use crate::tr;
use crate::types::AutoEvent;
use crate::types::CmdIdx;
use crate::types::PATHSEPSTR;
use crate::types::{CdCause, ExArg, Vv};
use crate::window::{DirOwner, own_dir};
use crate::winlayer::Win;
use crate::winlayer::{Buf, TabPage};
use core::ffi::{CStr, c_int};
use core::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};

use session::makeopens;
use view::put_view;

/// Constants the transpiler copied in from the headers this module includes.
mod flag {
    use super::{CdCause, c_int};

    /// `vim_chdirfile`'s reason code: not a `:cd`, so no autocommand.
    pub(super) const kCdCauseOther: CdCause = -1;
    /// `vim_strsave_fnameescape`: escape for an Ex command line.
    pub(super) const VSE_NONE: c_int = 0;

    /// Frame layouts.
    pub(super) const FR_LEAF: core::ffi::c_char = 0;
    pub(super) const FR_COL: core::ffi::c_char = 2;
}

use flag::VSE_NONE;

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

/// What every session writer answers: `Err` when a write did not reach the
/// file. Nothing has been said; `:mk*` says E80 for the whole file.
pub(crate) type Written = Result<(), IoFailed>;

/// The session file, buffered.
///
/// A write error the buffer holds back surfaces at the next write that
/// reaches the file, or at [`SessionFile::finish`].
pub(crate) struct SessionFile(BufWriter<File>);

impl SessionFile {
    /// Write into `file`.
    fn new(file: File) -> Self {
        Self(BufWriter::new(file))
    }

    /// Flush what is buffered and close the file: `Err` when something did
    /// not reach it -- the error `fclose` answered upstream.
    fn finish(self) -> Written {
        self.0.into_inner().map(drop).map_err(|_| IoFailed)
    }

    /// Write `s` and a newline: C's `put_line`.
    pub(crate) fn line(&mut self, s: &CStr) -> Written {
        self.puts(s)?;
        self.eol()
    }

    /// Write a newline: C's `put_eol`.
    pub(crate) fn eol(&mut self) -> Written {
        self.put_bytes(b"\n")
    }

    /// Write a literal's bytes.
    pub(crate) fn puts(&mut self, s: &CStr) -> Written {
        self.put_bytes(s.to_bytes())
    }

    /// Write formatted text. Every call site formats numbers into a literal
    /// template, so the result is ASCII; anything carrying editor bytes uses
    /// [`Self::put_bytes`] instead.
    pub(crate) fn write(&mut self, args: fmt::Arguments<'_>) -> Written {
        self.0.write_fmt(args).map_err(|_| IoFailed)
    }

    /// Write `bytes` verbatim.
    pub(crate) fn put_bytes(&mut self, bytes: &[u8]) -> Written {
        self.0.write_all(bytes).map_err(|_| IoFailed)
    }
}

/// The [`std::io::Write`] the option, mapping and fold writers render into.
impl Write for SessionFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

// -- File names ------------------------------------------------------------

/// The buffer name to write for `buffer`.
///
/// The short name is only usable when the working directory at the moment
/// the session is sourced is known -- so not for a view, not under 'acd',
/// and not once a `:lcd` has been written.
fn ses_get_fname(buffer: Buf, opts: SessionOpts) -> Option<XString> {
    let short = buffer.name.short();
    let name = if short.is_some()
        && opts.is_session()
        && opts.has(kOptSsopFlagCurdir | kOptSsopFlagSesdir)
        && !p_acd()
        && !did_lcd.get()
    {
        short
    } else {
        buffer.name.full()
    };
    name.map(XString::from_cstr)
}

/// Write `buffer`'s name, and a newline when `add_eol`.
fn ses_fname(out: &mut SessionFile, buffer: Buf, opts: SessionOpts, add_eol: bool) -> Written {
    let name = ses_get_fname(buffer, opts).unwrap_or_default();
    ses_put_fname(out, name.as_cstr())?;
    if add_eol { out.eol() } else { Ok(()) }
}

/// `name` with `$HOME` shortened to `~`, backslashes turned into forward
/// slashes (the legacy `slash` flag is always on) and the characters a
/// command line would eat escaped.
fn ses_escape_fname(name: &CStr) -> XString {
    // Room for the name and "~/", as `home_replace_save` sized it.
    let sname = home_to_tilde(None, name, name.count_bytes() + 3, true);
    let mut text = sname.to_vec();
    // A character at a time, so the scan never lands inside one.
    let mut at = 0;
    while at < text.len() {
        if text[at] == b'\\' {
            text[at] = b'/';
        }
        at += cluster_len(&text[at..]).max(1);
    }
    let text = XString::from_vec(text);
    fnameescape(text.as_cstr(), VSE_NONE)
}

/// Write `name` as an escaped file name.
fn ses_put_fname(out: &mut SessionFile, name: &CStr) -> Written {
    out.put_bytes(&ses_escape_fname(name))
}

/// Write an argument list: the `cmd` that selects which one, `%argdel` to
/// empty it, then one `$argadd` per entry.
fn ses_arglist(out: &mut SessionFile, cmd: &CStr, names: &[XString], fullname: bool) -> Written {
    out.puts(cmd)?;
    out.eol()?;
    out.line(c"%argdel")?;
    for name in names {
        let full_name;
        let mut name = name.as_cstr();
        if fullname {
            let full = vim_full_name(name, MAXPATHL as usize, false).unwrap_or_else(|name| name);
            full_name = XString::from_vec(full);
            name = full_name.as_cstr();
        }
        let escaped = ses_escape_fname(name);
        out.puts(c"$argadd ")?;
        out.put_bytes(&escaped)?;
        out.eol()?;
    }
    Ok(())
}

/// Whether window `window` belongs in the session at all. A floating window
/// never does (#18432); the rest is what 'sessionoptions' says about the
/// kind of buffer it holds.
pub(crate) fn ses_do_win(win: Win) -> bool {
    if win.w_floating {
        return false;
    }
    let buf = win.buffer();
    if buf.name.is_unnamed()
        // The contents of a "nofile" buffer cannot be restored.
        || (buf.terminal.is_null() && buf_is_nofilename(Some(buf)))
    {
        return ssop_flags.get() & kOptSsopFlagBlank != 0;
    }
    if buf_is_help(Some(buf)) {
        return ssop_flags.get() & kOptSsopFlagHelp != 0;
    }
    if buf_is_terminal(Some(buf)) {
        return ssop_flags.get() & kOptSsopFlagTerminal != 0;
    }
    true
}

// -- `:loadview` -----------------------------------------------------------

/// Why `:mkview` or `:loadview` did nothing; said once, by the command.
#[derive(Debug, thiserror::Error)]
enum ViewError {
    /// The buffer has no name to make a view file name from.
    #[error("{}", msg_cstr(gettext(e_noname)))]
    NoFileName,
    /// The view file would not open.
    #[error("{}", tr!("E484: Can't open file {}", msg_cstr(.0.as_cstr())))]
    CannotOpen(ThinCString),
}

const _: () = assert!(size_of::<ViewError>() <= 16);

impl ViewError {
    fn report(&self) {
        emsg_text(self.to_string());
    }
}

/// `:loadview [nr]`.
pub(crate) fn ex_loadview(excmd: &mut ExArg) {
    if let Err(err) = load_view(excmd.line.byte_at(excmd.line.arg)) {
        err.report();
    }
}

/// [`ex_loadview`]'s work.
fn load_view(c: u8) -> Result<(), ViewError> {
    let fname = get_view_file(c)?;
    // The name is this frame's own, which the view cannot reach.
    if do_source(fname.as_cstr()).is_err() {
        return Err(ViewError::CannotOpen(ThinCString::from_cstr(
            fname.as_cstr(),
        )));
    }
    Ok(())
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
/// depends on it. The answer keeps those bytes; its C string stops at the
/// NUL.
fn get_view_file(c: u8) -> Result<XString, ViewError> {
    let Some(full) = Buf::current().name.full().map(XString::from_cstr) else {
        return Err(ViewError::NoFileName);
    };
    let sname = home_to_tilde(None, full.as_cstr(), full.len() + 3, true);

    let mut view = XString::from_bytes(&P_VDIR.get());
    // `add_pathsep`, over the buffer just built.
    let terminated = after_pathsep(&view, view.len());
    if !view.is_empty() && !terminated {
        view.push_cstr(PATHSEPSTR);
    }

    // Each `=` and each path separator doubles, so that the flattened name
    // can be turned back into a path.
    for &byte in sname.iter() {
        if byte == b'=' {
            view.push_str("==");
        } else if vim_ispathsep(c_int::from(byte)) {
            view.push_str("=+");
        } else {
            view.push_byte(byte);
        }
    }
    view.push_byte(b'=');
    view.push_byte(c);
    view.push_str(".vim");
    Ok(view)
}

// -- `:mkexrc`, `:mkvimrc`, `:mkview`, `:mksession` ------------------------

/// The four `:mk*` commands.
///
/// Two legacy 'sessionoptions'/'viewoptions' flags are always on: line
/// endings are LF, and file names are written with `/`.
pub(crate) fn ex_mkrc(excmd: &mut ExArg) {
    let cmdidx = excmd.cmdidx;
    // `:mkview` and `:mksession` write a *state*; the other two write only
    // mappings and options.
    let view_session = cmdidx == CmdIdx::mksession || cmdidx == CmdIdx::mkview;

    // Short file names are usable until a ":lcd" is written. They are also
    // not used when 'acd' is set, which is checked separately.
    did_lcd.set(false);

    // A copy of the name: the command is lent on to the writer with it.
    let arg = excmd.line.arg().to_vec();
    let first = arg.first().copied().unwrap_or(0);
    // ":mkview" or ":mkview 9": the name comes from 'viewdir'.
    let using_vdir =
        cmdidx == CmdIdx::mkview && (arg.is_empty() || (first.is_ascii_digit() && arg.len() == 1));
    let fname = if using_vdir {
        excmd.forceit = true;
        let view_file = match get_view_file(first) {
            Ok(view_file) => view_file,
            Err(err) => return err.report(),
        };
        // The 'viewdir' may still need creating.
        let vdir = P_VDIR.get();
        if !dir_exists(vdir.as_cstr()) {
            let _ = vim_mkdir_emsg(vdir.as_cstr(), 0o755);
        }
        view_file
    } else if !arg.is_empty() {
        XString::from_vec(arg)
    } else if cmdidx == CmdIdx::mkvimrc {
        XString::from_cstr(VIMRC_FILE)
    } else if cmdidx == CmdIdx::mksession {
        XString::from_cstr(SESSION_FILE)
    } else {
        XString::from_cstr(EXRC_FILE)
    };
    let fname = fname.as_cstr();

    if let Ok(file) = create_exfile(fname, excmd.forceit) {
        let mut out = SessionFile::new(file);
        let written = write_rc(&mut out, excmd, fname, view_session, using_vdir);
        // The close answers a write error the buffering hid, and must run
        // whether or not anything failed above.
        let closed = out.finish();
        if written.and(closed).is_err() {
            emsg(gettext(e_write));
        } else if cmdidx == CmdIdx::mksession {
            // A successful session write sets v:this_session.
            if let Ok(full) = vim_full_name(fname, MAXPATHL as usize, false) {
                set_vim_var_string(Vv::ThisSession, Some(&full));
            }
        }
    }
    fire_autocmds(AutoEvent::SessionWritePost, false, Buf::current_or_none());
}

/// The body of [`ex_mkrc`] once the file is open: `Err` when anything
/// failed. The trailing modeline is written either way, as upstream does,
/// so a failure is kept rather than returned at once.
fn write_rc(
    out: &mut SessionFile,
    excmd: &mut ExArg,
    fname: &CStr,
    view_session: bool,
    using_vdir: bool,
) -> Written {
    let cmdidx = excmd.cmdidx;
    let opts = if cmdidx == CmdIdx::mkview {
        SessionOpts::View
    } else {
        SessionOpts::Session
    };
    let mut failed = false;

    if cmdidx == CmdIdx::mkvimrc {
        // Upstream ignores this one write's result.
        let _ = out.line(c"version 6.0");
    }
    if cmdidx == CmdIdx::mksession {
        failed |= out.line(c"let SessionLoad = 1").is_err();
    }

    // Mappings and options: everything for the two rc commands, and for
    // `:mksession` only when "options" is in 'sessionoptions'.
    if !view_session || (cmdidx == CmdIdx::mksession && opts.has(kOptSsopFlagOptions)) {
        let mut flags = OptionSetFlags::GLOBAL;
        if cmdidx == CmdIdx::mksession && opts.has(kOptSsopFlagSkiprtp) {
            flags |= OptionSetFlags::SKIPRTP;
        }
        failed |= makemap(out, None).is_err() || makeset(out, flags, 0).is_err();
    }

    if !failed && view_session {
        failed |= out
            .line(c"let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1")
            .is_err();
        failed |= if cmdidx == CmdIdx::mksession {
            write_session(out, fname)
        } else {
            let (win, tab) = (Win::current(), TabPage::current());
            put_view(out, win, tab, !using_vdir, opts, -1)
        }
        .is_err();
        failed |= out
            .line(c"let &g:so = s:so_save | let &g:siso = s:siso_save")
            .is_err();
        if p_hls() {
            failed |= out.line(c"set hlsearch").is_err();
        }
        if no_hlsearch.get() {
            failed |= out.line(c"nohlsearch").is_err();
        }
        failed |= out.line(c"doautoall SessionLoadPost").is_err();
        if cmdidx == CmdIdx::mksession {
            failed |= out.line(c"unlet SessionLoad").is_err();
        }
    }
    failed |= out.line(c"\" vim: set ft=vim :").is_err();
    if failed { Err(IoFailed) } else { Ok(()) }
}

/// `:mksession`'s body: change to whatever directory the file names should
/// be relative to, write the session, and change back.
fn write_session(out: &mut SessionFile, fname: &CStr) -> Written {
    // The directory to come back to; empty when it cannot be known (or
    // cannot be gone back to).
    let dirnow = dirname_within(MAXPATHL as usize)
        .ok()
        .filter(|dir| crate::cstr::with_terminated(dir, os_chdir) == 0)
        .map_or_else(XString::new, XString::from_vec);
    let known = !dirnow.is_empty();
    let globaldir = own_dir(DirOwner::Global);
    let to_sesdir = known && ssop_flags.get() & kOptSsopFlagSesdir != 0;
    let to_globaldir = known && ssop_flags.get() & kOptSsopFlagCurdir != 0 && globaldir.is_some();
    if to_sesdir {
        if vim_chdirfile(fname, flag::kCdCauseOther).is_ok() {
            shorten_fnames(true);
        }
    } else if to_globaldir
        && let Some(dir) = &globaldir
        && os_chdir(dir.as_cstr()) == 0
    {
        shorten_fnames(true);
    }

    let written = makeopens(out, &dirnow);

    // Restore the original directory.
    if to_sesdir || to_globaldir {
        if os_chdir(dirnow.as_cstr()) != 0 {
            emsg(gettext(e_prev_dir));
        }
        shorten_fnames(true);
    }
    written
}
