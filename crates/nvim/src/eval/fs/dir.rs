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

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::{__S_IFMT, FAIL, Owned, no_fileinfo, str_arg, str_arg_chk};
use crate::cstr;
use crate::eval::typval::NumBuf;
use crate::eval::typval::{tv_check_for_string_arg, tv_get_number_chk, tv_get_string_buf};
use crate::eval::userfunc::{add_defer, can_add_defer};
use crate::eval::window::find_win_by_nr;
use crate::event::libuv::uv_strerror;
use crate::ex_cmds::check_secure;
use crate::ex_docmd::{changedir_func, vim_mkdir_emsg};
use crate::fileio::{delete_recursive, vim_copyfile, vim_rename, vim_tempname};
use crate::memory::{xfree, xstrdup, xstrlcpy};
use crate::message::emsg;
use crate::message::{e_invarg, e_invargNval, e_invexpr2, e_mkdir};
use crate::message_fmt::{c_str, emsg_text};
use crate::os::cshim::gettext_ptr;
use crate::os::fs::{os_dirname, os_fileinfo_link, os_mkdir_recurse, os_remove, os_rmdir};
use crate::os::state::globaldir;
use crate::path::{full_name_save, path_tail, path_tail_with_sep};
use crate::tr_c;
use crate::types::{
    CdScope, EvalFuncData, MAXPATHL, OK, Tabpage, TypVal, VAR_NUMBER, VAR_STRING, VarNumber,
    Window, kCdScopeGlobal, kCdScopeInvalid, kCdScopeTabpage, kCdScopeWindow, size_t, uint64_t,
};
use crate::window::find_tabpage;
use crate::winlayer::{TabPage, Win};
use ::libc::abort;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

// ---------------------------------------------------------------------
// The safe layer this family adds
// ---------------------------------------------------------------------

/// Whether the sandbox forbids touching the tree, having reported it.
fn secure() -> bool {
    // SAFETY: reads the sandbox depth and may report; no arguments.
    check_secure()
}

/// Whether a deferred call can be registered, having reported if not.
fn can_defer() -> bool {
    can_add_defer()
}

/// Whether argument `i` is a String, having reported if not.
fn is_string_arg(args: &[TypVal], i: usize) -> bool {
    // SAFETY: the argument vector's own base, and `i` an index into it.
    tv_check_for_string_arg(args, i).is_ok()
}

/// Argument `i` as a path, spelled into `buf` when it is not already a
/// string; empty -- and reported -- for a type that has no string form.
///
/// Upstream's `tv_get_string_buf`, which is the *unchecked* form: the three
/// builtins below carry on with the empty string rather than returning.
fn path_arg<'a>(args: &[TypVal], i: usize, buf: &'a mut NumBuf) -> &'a CStr {
    str_arg_chk(args, i, buf).unwrap_or(c"")
}

/// As [`path_arg`], but kept raw, because `mkdir()` writes the trailing
/// separators off the path *in place* -- in whatever storage the argument
/// gave it, which is upstream's own doing and not something a `&CStr` may
/// share provenance with.
fn path_arg_raw(args: &[TypVal], i: usize, buf: &mut NumBuf) -> *mut c_char {
    // SAFETY: a live typval and a scratch of the length the callee is
    // promised; the answer is NUL-terminated and never NULL.
    unsafe { tv_get_string_buf(&args[i], buf.as_mut_ptr()).cast_mut() }
}

/// The current directory of the process, into `cwd`; false when the OS will
/// not say.
fn os_cwd(cwd: &Owned) -> bool {
    // SAFETY: `cwd` holds `MAXPATHL` bytes and a terminator slot after them.
    unsafe { os_dirname(cwd.0, MAXPATHL as size_t).is_ok() }
}

/// Copy the NUL-terminated `from` into `cwd`, truncating at [`MAXPATHL`].
fn set_cwd(cwd: &Owned, from: *const c_char) {
    // SAFETY: `cwd` holds `MAXPATHL` writable bytes and `from` is
    // NUL-terminated.
    unsafe { xstrlcpy(cwd.0, from, MAXPATHL as size_t) };
}

/// The window's own directory, or NULL when it has none.
fn win_localdir(win: Win) -> *mut c_char {
    win.w_localdir
}

/// The tabpage's own directory, or NULL when it has none.
fn tab_localdir(tabpage: TabPage) -> *mut c_char {
    tabpage.tp_localdir
}

/// Tabpage number `n`, or NULL when there is none.
fn find_tab(n: c_int) -> Option<TabPage> {
    find_tabpage(n)
}

/// The window argument 0 names within `tabpage`, or NULL when there is none.
fn find_win(args: &[TypVal], tabpage: Option<TabPage>) -> Option<Win> {
    // SAFETY: a live typval; an absent tab page reads as the current one.
    unsafe { find_win_by_nr(&args[0], tabpage) }
}

/// Change to `dir` in `scope`; false -- having reported -- when it fails.
fn changedir(dir: *mut c_char, scope: CdScope) -> bool {
    // SAFETY: `dir` is the argument's own NUL-terminated string, or NULL,
    // which the callee tests for.
    unsafe { changedir_func(dir, scope) }
}

/// The String argument `i` holds, raw, because `chdir()` hands the callee
/// the argument's own storage.
fn string_of(tv: &TypVal) -> *mut c_char {
    tv.string_or_null()
}

// ---------------------------------------------------------------------
// The messages
// ---------------------------------------------------------------------

/// Report the plain message `msg`, translated.
fn err0(msg: *const c_char) {
    // SAFETY: `msg` is NUL-terminated, which is all `gettext` and `emsg` ask.
    unsafe { emsg(gettext_ptr(msg)) };
}

/// Report the one-`%s` message `fmt`, translated, about `a`.
fn err1(fmt: &'static CStr, a: *const c_char) {
    // SAFETY: `a` is a NUL-terminated string.
    let a = unsafe { c_str(a) };
    emsg_text(tr_c!(fmt, a));
}

/// Report the two-`%s` message `fmt`, translated, about `a` and `b`.
fn err2(fmt: &'static CStr, a: *const c_char, b: *const c_char) {
    // SAFETY: both are NUL-terminated strings.
    let (a, b) = unsafe { (c_str(a), c_str(b)) };
    emsg_text(tr_c!(fmt, a, b));
}

/// libuv's name for the error code `error`.
fn strerror(error: c_int) -> *const c_char {
    // SAFETY: `uv_strerror` answers a NUL-terminated string for any code.
    unsafe { uv_strerror(error) }
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
    tp: *mut Tabpage,
    win: *mut Window,
}

impl Scope {
    fn read(args: &[TypVal], default_to_window: bool) -> Option<Self> {
        let (win_i, tab_i) = (kCdScopeWindow as usize, kCdScopeTabpage as usize);
        let mut s = Self {
            scope: kCdScopeInvalid,
            number: [0, 0],
            tp: TabPage::current_raw(),
            win: Win::current_raw(),
        };

        // Preconditions and scope extraction together.
        for i in win_i..=tab_i {
            // With no argument there are no more scopes after it.
            if args.len() <= i {
                break;
            }
            if !args.get(i).is_some_and(|arg| arg.v_type() == VAR_NUMBER) {
                err0(e_invarg.as_ptr());
                return None;
            }
            s.number[i] = number_of(&args[i]) as c_int;
            // It is an error for a scope number to be less than -1.
            if s.number[i] < -1 {
                err0(e_invarg.as_ptr());
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
            s.tp = find_tab(s.number[tab_i]).map_or(ptr::null_mut(), TabPage::raw);
            if s.tp.is_null() {
                err0(c"E5000: Cannot find tab number.".as_ptr());
                return None;
            }
        }

        // And the window in `tp` by number.
        if s.number[win_i] >= 0 {
            if s.number[tab_i] < 0 {
                err0(c"E5001: Higher scope cannot be -1 if lower scope is >= 0.".as_ptr());
                return None;
            }
            if s.number[win_i] > 0 {
                s.win = find_win(args, unsafe { TabPage::from_raw(s.tp) })
                    .map_or(ptr::null_mut(), Win::raw);
                if s.win.is_null() {
                    err0(c"E5002: Cannot find window number.".as_ptr());
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
    result.write_string(ptr::null_mut());
    if args[0].v_type() != VAR_STRING {
        // Returning an empty string means it failed.  No error message, for
        // historic reasons.
        return;
    }

    // The answer is the directory that is current now.  It is taken before
    // the scope is parsed, so a bad scope reports *and* answers it.
    {
        let cwd = Owned::zeroed(MAXPATHL as usize);
        if os_cwd(&cwd) {
            result.write_string(Owned::dup(cwd.cstr()).into_raw());
        }
    }

    let mut scope = kCdScopeGlobal;
    if args.len() > 1 {
        let s = str_arg(args, 1, &mut numbuf);
        scope = match s.to_bytes() {
            b"global" => kCdScopeGlobal,
            b"tabpage" => kCdScopeTabpage,
            b"window" => kCdScopeWindow,
            _ => {
                err2(e_invargNval, c"scope".as_ptr(), s.as_ptr());
                return;
            }
        };
    } else if !win_localdir(Win::current()).is_null() {
        scope = kCdScopeWindow;
    } else if !tab_localdir(TabPage::current()).is_null() {
        scope = kCdScopeTabpage;
    }

    if !changedir(string_of(&args[0]), scope) {
        // Directory change failed: answer the empty string after all.
        // SAFETY: the answer taken above is nvim's heap, or NULL.
        unsafe { xfree(result.string_or_null().cast::<c_void>()) };
        result.write_string(ptr::null_mut());
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
        err0(e_invarg.as_ptr());
        return;
    }

    let mut nbuf = NumBuf::new();
    let flags = if args.len() > 1 {
        path_arg(args, 1, &mut nbuf)
    } else {
        c""
    };
    let name = name.as_ptr();
    let done = |ret: c_int| -> VarNumber { if ret == 0 { 0 } else { -1 } };
    result.write_number(match flags.to_bytes() {
        // SAFETY: `name` is NUL-terminated; each callee only reads it.
        b"" => done(unsafe { os_remove(cstr::at(name)) }),
        b"d" => done(unsafe { os_rmdir(cstr::at(name)) }),
        b"rf" => VarNumber::from(unsafe { delete_recursive(name) }),
        _ => {
            err1(e_invexpr2, flags.as_ptr());
            return;
        }
    });
}

/// `filecopy({from}, {to})`: copy a regular file or a symlink, answering
/// whether it worked.
pub fn f_filecopy(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    result.write_number(0);
    if secure() || !is_string_arg(args, 0) || !is_string_arg(args, 1) {
        return;
    }

    let mut info = no_fileinfo();
    let from = str_arg(args, 0, &mut numbuf);
    // SAFETY: `from` is NUL-terminated, and `info` is this frame's own.
    let known = unsafe { os_fileinfo_link(from.as_ptr(), &raw mut info) };
    // `S_ISREG` and `S_ISLNK`: only a plain file or a symlink is copied.
    const S_IFREG: uint64_t = 0o100000;
    const S_IFLNK: uint64_t = 0o120000;
    let kind = info.stat.st_mode & __S_IFMT as uint64_t;
    if known && (kind == S_IFREG || kind == S_IFLNK) {
        let (from, to) = (
            str_arg(args, 0, &mut numbuf2).as_ptr(),
            str_arg(args, 1, &mut numbuf3).as_ptr(),
        );
        // SAFETY: both are NUL-terminated.
        result.write_number((unsafe { vim_copyfile(from, to) } == OK) as VarNumber);
    }
}

/// `getcwd([{win} [, {tab}]])`: the working directory of the scope the
/// arguments name, always as a string.
pub fn f_getcwd(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(ptr::null_mut());
    let Some(s) = Scope::read(args, false) else {
        return;
    };

    let cwd = Owned::zeroed(MAXPATHL as usize);
    // The narrowest local directory that is actually set, entered at the
    // rung the scope names and falling one rung at a time: the window's,
    // then its tabpage's, then the global one, and finally the OS's own.
    let mut from: *const c_char = ptr::null();
    if s.scope == kCdScopeWindow {
        debug_assert!(!s.win.is_null(), "win");
        from = win_localdir(unsafe { Win::new(s.win) });
    }
    if from.is_null() && (kCdScopeWindow..=kCdScopeTabpage).contains(&s.scope) {
        debug_assert!(!s.tp.is_null(), "tp");
        from = tab_localdir(unsafe { TabPage::new(s.tp) });
    }
    if from.is_null() && (kCdScopeWindow..=kCdScopeGlobal).contains(&s.scope) {
        // `globaldir` is not always set.
        from = globaldir.get();
    }
    if from.is_null() && !os_cwd(&cwd) {
        // Answer the empty string on failure.
        from = c"".as_ptr();
    }

    if !from.is_null() {
        set_cwd(&cwd, from);
    }
    result.write_string(Owned::dup(cwd.cstr()).into_raw());
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
            debug_assert!(!s.win.is_null(), "win");
            !win_localdir(unsafe { Win::new(s.win) }).is_null() as VarNumber
        }
        kCdScopeTabpage => {
            debug_assert!(!s.tp.is_null(), "tp");
            !tab_localdir(unsafe { TabPage::new(s.tp) }).is_null() as VarNumber
        }
        kCdScopeInvalid => {
            // We should never get here: the read above defaulted it.
            // SAFETY: `abort` does not return.
            unsafe { abort() };
        }
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
    let dir = path_arg_raw(args, 0, &mut buf);
    // SAFETY: `dir` is NUL-terminated.
    if unsafe { *dir } == 0 {
        return;
    }
    strip_trailing_seps(dir);

    let mut defer = false;
    let mut defer_recurse = false;
    let mut created = ptr::null_mut();
    if args.len() > 1 {
        if args.len() > 2 {
            // With no error flag the failure answer is -1 rather than 0, and
            // -1 is exactly what the test below looks for.
            // SAFETY: a live typval; a null flag asks for that answer.
            prot = unsafe { tv_get_number_chk(&args[2], ptr::null_mut()) } as c_int;
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
            let mut failed_dir = ptr::null_mut();
            let want = if defer || defer_recurse {
                &raw mut created
            } else {
                ptr::null_mut()
            };
            // SAFETY: `dir` is NUL-terminated and the two out-parameters are
            // this frame's own; both answer a string in nvim's heap.
            let ret = unsafe { os_mkdir_recurse(dir, prot, &raw mut failed_dir, want) };
            if ret != 0 {
                err2(e_mkdir, failed_dir, strerror(ret));
                drop(Owned(failed_dir));
                result.write_number(FAIL as VarNumber);
                return;
            }
            status = OK as VarNumber;
        }
    }
    if status == FAIL as VarNumber {
        // SAFETY: `dir` is NUL-terminated; the callee reports its own error.
        status = VarNumber::from(unsafe { vim_mkdir_emsg(dir, prot) }.is_ok());
    }
    result.write_number(status);

    // The "D" and "R" flags: deferred deletion of the created directory.
    if status == OK as VarNumber && created.is_null() && (defer || defer_recurse) {
        // SAFETY: `dir` is NUL-terminated; the answer is nvim's heap.
        created = unsafe { full_name_save(dir, false) };
    }
    if !created.is_null() {
        defer_delete(created, defer_recurse);
    }
}

/// Cut the trailing separators off `dir` when its last component is empty --
/// in place, in whatever storage the argument gave.
fn strip_trailing_seps(dir: *mut c_char) {
    // SAFETY: `dir` is NUL-terminated, and both callees answer a pointer
    // inside it, so the terminator lands inside the same string.
    if unsafe { *path_tail(dir) } == 0 {
        unsafe { *path_tail_with_sep(dir) = 0 };
    }
}

/// Register `delete({created}, "d"|"rf")` to run when the calling function
/// returns -- `mkdir()`'s `D` and `R` flags.
fn defer_delete(created: *mut c_char, recurse: bool) {
    let how = if recurse { c"rf" } else { c"d" };
    // SAFETY: a NUL-terminated literal; the copy is nvim's heap.
    let how = unsafe { xstrdup(how.as_ptr()) };
    let string = |s| TypVal::String(s);
    let mut tv = [string(created), string(how)];
    let name = c"delete".as_ptr().cast_mut();
    // SAFETY: two arguments, at `tv`, whose contents the callee takes over.
    unsafe { add_defer(name, &mut tv) };
}

/// `rename({from}, {to})`: move a file, 0 on success.
pub fn f_rename(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if secure() {
        result.write_number(-1 as VarNumber);
        return;
    }
    let mut buf = NumBuf::new();
    let (from, to) = (
        str_arg(args, 0, &mut numbuf).as_ptr(),
        path_arg(args, 1, &mut buf).as_ptr(),
    );
    // SAFETY: both are NUL-terminated.
    result.write_number(unsafe { vim_rename(from, to) } as VarNumber);
}

/// `tempname()`: a fresh name in the session's own temporary directory.
pub fn f_tempname(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(vim_tempname());
}
