//! Changing the tree, and the current directory -- `chdir()`, `getcwd()`,
//! `haslocaldir()`, `mkdir()`, `delete()`, `rename()`, `filecopy()` and
//! `tempname()`.
//!
//! Everything here has a side effect the next builtin can see, which is why
//! it is grouped: `f_chdir`/`f_getcwd`/`f_haslocaldir` are the
//! window/tab/global scope ladder over the current directory, and the rest
//! create, move, copy or remove files and directories.  `f_mkdir`'s `D`/`R`
//! flags register a deferred cleanup with the calling function, so the effect
//! can outlive the call.
//!
//! # The scope ladder
//!
//! `getcwd()` and `haslocaldir()` take the same `[{win} [, {tab}]]` and
//! upstream carries two byte-identical copies of the walk that reads them.
//! [`Scope`] is that walk, once: it resolves the arguments to a rung of the
//! ladder plus the window and tabpage they name, and reports E474/E5000/
//! E5001/E5002 itself.  The one difference between the two builtins -- that
//! `haslocaldir()` defaults to window scope when nothing was asked for -- is
//! its `default_to_window` argument.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::{str_arg, str_arg_chk, tail_with_sep};
use crate::eval::typval::NumBuf;
use crate::eval::typval::{tv_check_for_string_arg, tv_get_number_chk};
use crate::eval::userfunc::{add_defer, can_add_defer};
use crate::eval::window::find_win_by_nr;
use crate::ex_cmds::check_secure;
use crate::ex_docmd::{DirOwner, change_dir, own_dir, vim_mkdir_emsg};
use crate::fileio::{delete_recursive, rename_file, temp_name, vim_copyfile};
use crate::memory::ThinCString;
use crate::message::emsg;
use crate::message::{e_invarg, e_invargNval, e_invexpr2, e_mkdir};
use crate::message_fmt::{emsg_text, msg_cstr, msg_cstr_opt};
use crate::os::cshim::gettext;
use crate::os::fs::{current_dir, link_info, mkdir_recurse, os_remove, os_rmdir};
use crate::path::{full_name_of, tail_index};
use crate::tr_c;
use crate::types::{
    CdScope, EvalFuncData, FAIL, MAXPATHL, OK, TypVal, VAR_NUMBER, VAR_STRING, VarNumber,
    kCdScopeGlobal, kCdScopeInvalid, kCdScopeTabpage, kCdScopeWindow, uint64_t,
};
use crate::window::find_tabpage;
use crate::winlayer::{TabPage, Win};
use core::ffi::{CStr, c_int};
use std::ffi::CString;

// ---------------------------------------------------------------------
// The safe layer this family adds
// ---------------------------------------------------------------------

/// Whether the sandbox forbids touching the tree, having reported it.
fn secure() -> bool {
    check_secure()
}

/// Whether a deferred call can be registered, having reported if not.
fn can_defer() -> bool {
    can_add_defer()
}

/// Whether argument `i` is a String, having reported if not.
fn is_string_arg(args: &[TypVal], i: usize) -> bool {
    tv_check_for_string_arg(args, i).is_ok()
}

/// Argument `i` as a path, spelled into `buf` when it is not already a
/// string; empty -- and reported -- for a type that has no string form.
///
/// Upstream's `tv_get_string_buf`, which is the *unchecked* form: the three
/// builtins below carry on with the empty string rather than returning.
fn path_arg<'a>(args: &'a [TypVal], i: usize, buf: &'a mut NumBuf) -> &'a CStr {
    str_arg_chk(args, i, buf).unwrap_or(c"")
}

/// Whether the window has a directory of its own.
fn win_has_localdir(win: Win) -> bool {
    !win.w_localdir.is_null()
}

/// Whether the tabpage has a directory of its own.
fn tab_has_localdir(tabpage: TabPage) -> bool {
    !tabpage.tp_localdir.is_null()
}

/// Tabpage number `n`, or NULL when there is none.
fn find_tab(n: c_int) -> Option<TabPage> {
    find_tabpage(n)
}

/// The window argument 0 names within `tabpage`, or NULL when there is none.
fn find_win(args: &[TypVal], tabpage: Option<TabPage>) -> Option<Win> {
    find_win_by_nr(&args[0], tabpage)
}

// ---------------------------------------------------------------------
// The messages
// ---------------------------------------------------------------------

/// Report the plain message `msg`, translated.
fn err0(msg: &'static CStr) {
    emsg(gettext(msg));
}

/// Report the one-`%s` message `fmt`, translated, about `a`.
fn err1(fmt: &'static CStr, a: &CStr) {
    emsg_text(tr_c!(fmt, msg_cstr(a)));
}

/// Report the two-`%s` message `fmt`, translated, about `a` and `b`.
fn err2(fmt: &'static CStr, a: Option<&CStr>, b: &CStr) {
    emsg_text(tr_c!(fmt, msg_cstr_opt(a), msg_cstr(b)));
}

// ---------------------------------------------------------------------
// The scope ladder
// ---------------------------------------------------------------------

/// Which rung of the window/tabpage/global ladder the arguments name, and
/// the objects they name it on.
struct Scope {
    /// The narrowest scope asked for, or [`kCdScopeInvalid`] for none.
    scope: CdScope,
    /// The `{win}` and `{tab}` numbers, indexed by their `CdScope`: -1 skips
    /// the scope and moves the answer one rung up, 0 means the current
    /// object, and a positive number names one.
    number: [c_int; 2],
    /// The tab page and window the numbers name, found while reading the
    /// arguments and used before anything else runs.
    tp: Option<TabPage>,
    win: Option<Win>,
}

impl Scope {
    fn read(args: &[TypVal], default_to_window: bool) -> Option<Self> {
        let (win_i, tab_i) = (kCdScopeWindow as usize, kCdScopeTabpage as usize);
        let mut s = Self {
            scope: kCdScopeInvalid,
            number: [0, 0],
            tp: TabPage::current_or_none(),
            win: Win::current_or_none(),
        };

        // Preconditions and scope extraction together.
        for i in win_i..=tab_i {
            // With no argument there are no more scopes after it.
            if args.len() <= i {
                break;
            }
            if !args.get(i).is_some_and(|arg| arg.v_type() == VAR_NUMBER) {
                err0(e_invarg);
                return None;
            }
            s.number[i] = number_of(&args[i]) as c_int;
            // It is an error for a scope number to be less than -1.
            if s.number[i] < -1 {
                err0(e_invarg);
                return None;
            }
            // Use the narrowest scope the caller asked for.
            if s.number[i] >= 0 && s.scope == kCdScopeInvalid {
                s.scope = i as CdScope;
            } else if s.number[i] < 0 {
                s.scope = i as CdScope + 1;
            }
        }

        // Called without any arguments, `haslocaldir()` means window scope.
        if default_to_window && s.scope == kCdScopeInvalid {
            s.scope = kCdScopeWindow;
        }

        // Find the tabpage by number.
        if s.number[tab_i] > 0 {
            s.tp = find_tab(s.number[tab_i]);
            if s.tp.is_none() {
                err0(c"E5000: Cannot find tab number.");
                return None;
            }
        }

        // And the window in `tp` by number.
        if s.number[win_i] >= 0 {
            if s.number[tab_i] < 0 {
                err0(c"E5001: Higher scope cannot be -1 if lower scope is >= 0.");
                return None;
            }
            if s.number[win_i] > 0 {
                s.win = find_win(args, s.tp);
                if s.win.is_none() {
                    err0(c"E5002: Cannot find window number.");
                    return None;
                }
            }
        }
        Some(s)
    }
}

/// The Number argument `tv` holds.
fn number_of(tv: &TypVal) -> VarNumber {
    tv.number_or_zero()
}

// ---------------------------------------------------------------------
// The builtins
// ---------------------------------------------------------------------

/// `chdir({dir})`: change directory in the narrowest scope that is already
/// local, answering the directory that was current before.
pub fn f_chdir(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_string(None);
    if args[0].v_type() != VAR_STRING {
        // Returning an empty string means it failed.  No error message, for
        // historic reasons.
        return;
    }

    // The answer is the directory that is current now.  It is taken before
    // the scope is parsed, so a bad scope reports *and* answers it.
    if let Some(cwd) = current_dir() {
        result.write_string(Some(ThinCString::from(cwd)));
    }

    let mut scope = kCdScopeGlobal;
    if args.len() > 1 {
        let s = str_arg(args, 1, &mut numbuf);
        scope = match s.to_bytes() {
            b"global" => kCdScopeGlobal,
            b"tabpage" => kCdScopeTabpage,
            b"window" => kCdScopeWindow,
            _ => {
                err2(e_invargNval, Some(c"scope"), s);
                return;
            }
        };
    } else if win_has_localdir(Win::current()) {
        scope = kCdScopeWindow;
    } else if tab_has_localdir(TabPage::current()) {
        scope = kCdScopeTabpage;
    }

    // A copy: the DirChangedPre autocommand runs while the name is read.
    let dir = args[0].string_ref().map(|dir| CString::from(dir.as_cstr()));
    if !dir.is_some_and(|dir| change_dir(dir, scope)) {
        // Directory change failed: answer the empty string after all.
        drop(result.take_string());
    }
}

/// `delete({fname} [, {flags}])`: remove a file, an empty directory (`d`) or
/// a whole tree (`rf`).
pub fn f_delete(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1 as VarNumber);
    if secure() {
        return;
    }
    let name = str_arg(args, 0, &mut numbuf);
    if name.to_bytes().is_empty() {
        err0(e_invarg);
        return;
    }

    let mut nbuf = NumBuf::new();
    let flags = if args.len() > 1 {
        path_arg(args, 1, &mut nbuf)
    } else {
        c""
    };
    let done = |ret: c_int| -> VarNumber { if ret == 0 { 0 } else { -1 } };
    result.write_number(match flags.to_bytes() {
        b"" => done(os_remove(name)),
        b"d" => done(os_rmdir(name)),
        b"rf" => VarNumber::from(delete_recursive(name)),
        _ => {
            err1(e_invexpr2, flags);
            return;
        }
    });
}

/// `filecopy({from}, {to})`: copy a regular file or a symlink, answering
/// whether it worked.
pub fn f_filecopy(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    result.write_number(0);
    if secure() || !is_string_arg(args, 0) || !is_string_arg(args, 1) {
        return;
    }

    let from = str_arg(args, 0, &mut numbuf);
    // `S_ISREG` and `S_ISLNK`: only a plain file or a symlink is copied.
    const S_IFREG: uint64_t = 0o100000;
    const S_IFLNK: uint64_t = 0o120000;
    let copyable = link_info(from).is_some_and(|info| {
        let kind = info.stat.st_mode & super::__S_IFMT as uint64_t;
        kind == S_IFREG || kind == S_IFLNK
    });
    if copyable {
        let to = str_arg(args, 1, &mut numbuf2);
        result.write_number((vim_copyfile(from, to) == OK) as VarNumber);
    }
}

/// `getcwd([{win} [, {tab}]])`: the working directory of the scope the
/// arguments name, always as a string.
pub fn f_getcwd(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
    let Some(s) = Scope::read(args, false) else {
        return;
    };

    // The narrowest local directory that is actually set, entered at the
    // rung the scope names and falling one rung at a time: the window's,
    // then its tabpage's, then the global one, and finally the OS's own.
    let mut dir = None;
    if s.scope == kCdScopeWindow {
        let win = s.win.expect("a window scope names a window");
        dir = own_dir(DirOwner::Window(win));
    }
    if dir.is_none() && (kCdScopeWindow..=kCdScopeTabpage).contains(&s.scope) {
        let tp = s.tp.expect("a tab page scope names a tab page");
        dir = own_dir(DirOwner::TabPage(tp));
    }
    if dir.is_none() && (kCdScopeWindow..=kCdScopeGlobal).contains(&s.scope) {
        // `globaldir` is not always set.
        dir = own_dir(DirOwner::Global);
    }
    // The empty string when the OS will not say either.
    let dir = dir.or_else(current_dir);
    let mut cwd = dir.map_or_else(Vec::new, |dir| dir.as_cstr().to_bytes().to_vec());
    // Upstream copies it into a `MAXPATHL` buffer, terminator included.
    cwd.truncate(MAXPATHL as usize - 1);
    result.write_string(Some(ThinCString::from_vec(cwd)));
}

/// `haslocaldir([{win} [, {tab}]])`: whether the scope the arguments name
/// has a directory of its own.
pub fn f_haslocaldir(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0 as VarNumber);
    let Some(s) = Scope::read(args, true) else {
        return;
    };

    result.write_number(match s.scope {
        kCdScopeWindow => {
            let win = s.win.expect("a window scope names a window");
            win_has_localdir(win) as VarNumber
        }
        kCdScopeTabpage => {
            let tp = s.tp.expect("a tab page scope names a tab page");
            tab_has_localdir(tp) as VarNumber
        }
        // We should never get here: the read above defaulted it.
        kCdScopeInvalid => std::process::abort(),
        // The global scope never has a local directory.
        _ => 0 as VarNumber,
    });
}

/// `mkdir({name} [, {flags} [, {prot}]])`: create a directory, with `p`
/// creating the parents too and `D`/`R` registering its removal for when the
/// calling function returns.
pub fn f_mkdir(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // Upstream's default, which is *not* the 0777 the shell's `mkdir` uses.
    let mut prot: c_int = 0o755;
    // Held in a local and written back at each exit, so that the answer is
    // only ever written into the union, never read back out of it.
    let mut status = FAIL as VarNumber;
    result.write_number(status);
    if secure() {
        return;
    }

    let mut buf = NumBuf::new();
    // A copy: upstream cuts the trailing separators off in whatever storage
    // the argument gave it.
    let dir = strip_trailing_seps(str_arg(args, 0, &mut buf).to_bytes());
    if dir.as_bytes().is_empty() {
        return;
    }

    let mut defer = false;
    let mut defer_recurse = false;
    let mut created = None;
    if args.len() > 1 {
        if args.len() > 2 {
            // With no error flag the failure answer is -1 rather than 0, and
            // -1 is exactly what the test below looks for.
            prot = tv_get_number_chk(&args[2]).unwrap_or(-1) as c_int;
            if prot == -1 {
                return;
            }
        }
        // The flags are ASCII, so a plain byte search is `vim_strchr`.
        let arg2 = str_arg(args, 1, &mut numbuf).to_bytes();
        defer = arg2.contains(&b'D');
        defer_recurse = arg2.contains(&b'R');
        if (defer || defer_recurse) && !can_defer() {
            return;
        }
        if arg2.contains(&b'p') {
            match mkdir_recurse(&dir, prot) {
                Ok(first) => {
                    // Only the deferred delete wants the first directory made.
                    if defer || defer_recurse {
                        created = first.map(ThinCString::from);
                    }
                }
                Err(failure) => {
                    err2(e_mkdir, failure.dir.as_deref(), failure.why);
                    result.write_number(FAIL as VarNumber);
                    return;
                }
            }
            status = OK as VarNumber;
        }
    }
    if status == FAIL as VarNumber {
        // The callee reports its own error.
        status = VarNumber::from(vim_mkdir_emsg(&dir, prot).is_ok());
    }
    result.write_number(status);

    // The "D" and "R" flags: deferred deletion of the created directory.
    if status == OK as VarNumber && created.is_none() && (defer || defer_recurse) {
        created = Some(ThinCString::from(full_name_of(&dir, false)));
    }
    if let Some(created) = created {
        defer_delete(created, defer_recurse);
    }
}

/// `dir` with the trailing separators cut off when its last component is
/// empty.
fn strip_trailing_seps(dir: &[u8]) -> CString {
    let end = if tail_index(dir) == dir.len() {
        tail_with_sep(dir)
    } else {
        dir.len()
    };
    CString::new(&dir[..end]).expect("a C string's bytes hold no NUL")
}

/// Register `delete({created}, "d"|"rf")` to run when the calling function
/// returns -- `mkdir()`'s `D` and `R` flags.
fn defer_delete(created: ThinCString, recurse: bool) {
    let how = if recurse { c"rf" } else { c"d" };
    let mut tv = [
        TypVal::string(Some(created)),
        TypVal::string(Some(ThinCString::from_cstr(how))),
    ];
    // The callee takes the two arguments' contents over.
    add_defer(c"delete", &mut tv);
}

/// `rename({from}, {to})`: move a file, 0 on success.
pub fn f_rename(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if secure() {
        result.write_number(-1 as VarNumber);
        return;
    }
    let mut buf = NumBuf::new();
    let from = str_arg(args, 0, &mut numbuf);
    let to = path_arg(args, 1, &mut buf);
    result.write_number(if rename_file(from, to) { 0 } else { -1 });
}

/// `tempname()`: a fresh name in the session's own temporary directory.
pub fn f_tempname(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(temp_name().map(ThinCString::from));
}
