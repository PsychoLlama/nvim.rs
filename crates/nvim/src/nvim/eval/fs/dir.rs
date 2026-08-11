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

use super::{
    __S_IFMT, Args, FAIL, MAXPATHL, NUMBUFLEN, OK, Owned, frame, no_fileinfo, numbuf, ret_string,
    str_arg, str_arg_buf,
};
use crate::semsg_c;
use crate::src::nvim::eval::typval::{
    tv_check_for_string_arg, tv_get_number_chk, tv_get_string_buf,
};
use crate::src::nvim::eval::userfunc::{add_defer, can_add_defer};
use crate::src::nvim::eval::window::find_win_by_nr;
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_docmd::{changedir_func, vim_mkdir_emsg};
use crate::src::nvim::fileio::{delete_recursive, vim_copyfile, vim_rename, vim_tempname};
use crate::src::nvim::main::{
    curtab, curwin, e_invarg, e_invargNval, e_invexpr2, e_mkdir, globaldir,
};
use crate::src::nvim::memory::{xfree, xstrdup, xstrlcpy};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::fs::{
    os_dirname, os_fileinfo_link, os_mkdir_recurse, os_remove, os_rmdir,
};
use crate::src::nvim::os::libc::{abort, gettext};
use crate::src::nvim::path::{FullName_save, path_tail, path_tail_with_sep};
use crate::src::nvim::types::{
    CdScope, EvalFuncData, VAR_NUMBER, VAR_STRING, VAR_UNLOCKED, kCdScopeGlobal, kCdScopeInvalid,
    kCdScopeTabpage, kCdScopeWindow, size_t, tabpage_T, typval_T, typval_vval_union, uint64_t,
    varnumber_T, win_T,
};
use crate::src::nvim::window::find_tabpage;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

// ---------------------------------------------------------------------
// The safe layer this family adds
// ---------------------------------------------------------------------

/// Whether the sandbox forbids touching the tree, having reported it.
fn secure() -> bool {
    // SAFETY: reads the sandbox depth and may report; no arguments.
    unsafe { check_secure() }
}

/// Whether a deferred call can be registered, having reported if not.
fn can_defer() -> bool {
    // SAFETY: reads the call stack and may report; no arguments.
    unsafe { can_add_defer() }
}

/// Whether argument `i` is a String, having reported if not.
fn is_string_arg(args: Args<'_>, i: usize) -> bool {
    // SAFETY: the argument vector's own base, and `i` an index into it.
    unsafe { tv_check_for_string_arg(args.ptr(0), i as c_int) != FAIL }
}

/// Argument `i` as a path, spelled into `buf` when it is not already a
/// string; empty -- and reported -- for a type that has no string form.
///
/// Upstream's `tv_get_string_buf`, which is the *unchecked* form: the three
/// builtins below carry on with the empty string rather than returning.
fn path_arg<'a>(args: Args<'_>, i: usize, buf: &'a mut [c_char; NUMBUFLEN]) -> &'a CStr {
    str_arg_buf(args, i, buf).unwrap_or(c"")
}

/// As [`path_arg`], but kept raw, because `mkdir()` writes the trailing
/// separators off the path *in place* -- in whatever storage the argument
/// gave it, which is upstream's own doing and not something a `&CStr` may
/// share provenance with.
fn path_arg_raw(args: Args<'_>, i: usize, buf: &mut [c_char; NUMBUFLEN]) -> *mut c_char {
    // SAFETY: a live typval and a scratch of the length the callee is
    // promised; the answer is NUL-terminated and never NULL.
    unsafe { tv_get_string_buf(args.ptr(i), buf.as_mut_ptr()).cast_mut() }
}

/// The current directory of the process, into `cwd`; false when the OS will
/// not say.
fn os_cwd(cwd: &Owned) -> bool {
    // SAFETY: `cwd` holds `MAXPATHL` bytes and a terminator slot after them.
    unsafe { os_dirname(cwd.0, MAXPATHL as size_t) != FAIL }
}

/// Copy the NUL-terminated `from` into `cwd`, truncating at [`MAXPATHL`].
fn set_cwd(cwd: &Owned, from: *const c_char) {
    // SAFETY: `cwd` holds `MAXPATHL` writable bytes and `from` is
    // NUL-terminated.
    unsafe { xstrlcpy(cwd.0, from, MAXPATHL as size_t) };
}

/// The window's own directory, or NULL when it has none.
fn win_localdir(win: *mut win_T) -> *mut c_char {
    // SAFETY: a live window.
    unsafe { (*win).w_localdir }
}

/// The tabpage's own directory, or NULL when it has none.
fn tab_localdir(tp: *mut tabpage_T) -> *mut c_char {
    // SAFETY: a live tabpage.
    unsafe { (*tp).tp_localdir }
}

/// Tabpage number `n`, or NULL when there is none.
fn find_tab(n: c_int) -> *mut tabpage_T {
    // SAFETY: a plain lookup over the tabpage list.
    unsafe { find_tabpage(n) }
}

/// The window argument 0 names within `tp`, or NULL when there is none.
fn find_win(args: Args<'_>, tp: *mut tabpage_T) -> *mut win_T {
    // SAFETY: a live typval and a live tabpage.
    unsafe { find_win_by_nr(args.ptr(0), tp) }
}

/// Change to `dir` in `scope`; false -- having reported -- when it fails.
fn changedir(dir: *mut c_char, scope: CdScope) -> bool {
    // SAFETY: `dir` is the argument's own NUL-terminated string, or NULL,
    // which the callee tests for.
    unsafe { changedir_func(dir, scope) }
}

/// The String argument `i` holds, raw, because `chdir()` hands the callee
/// the argument's own storage.
fn string_of(tv: &typval_T) -> *mut c_char {
    // SAFETY: only reached under a `VAR_STRING` tag, which is what makes
    // `v_string` the live arm.
    unsafe { tv.vval.v_string }
}

// ---------------------------------------------------------------------
// The messages
// ---------------------------------------------------------------------

/// Report the plain message `msg`, translated.
fn err0(msg: *const c_char) {
    // SAFETY: `msg` is NUL-terminated, which is all `gettext` and `emsg` ask.
    unsafe { emsg(gettext(msg)) };
}

/// Report the one-`%s` message `fmt`, translated, about `a`.
fn err1(fmt: *const c_char, a: *const c_char) {
    // SAFETY: `fmt` is a NUL-terminated format taking one string, and `a` is
    // a NUL-terminated string.
    unsafe { semsg_c!(gettext(fmt), a) };
}

/// Report the two-`%s` message `fmt`, translated, about `a` and `b`.
fn err2(fmt: *const c_char, a: *const c_char, b: *const c_char) {
    // SAFETY: `fmt` is a NUL-terminated format taking two strings, and both
    // are NUL-terminated.
    unsafe { semsg_c!(gettext(fmt), a, b) };
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
    tp: *mut tabpage_T,
    win: *mut win_T,
}

impl Scope {
    fn read(args: Args<'_>, default_to_window: bool) -> Option<Self> {
        let (win_i, tab_i) = (kCdScopeWindow as usize, kCdScopeTabpage as usize);
        let mut s = Self {
            scope: kCdScopeInvalid,
            number: [0, 0],
            tp: curtab.get(),
            win: curwin.get(),
        };

        // Preconditions and scope extraction together.
        for i in win_i..=tab_i {
            // With no argument there are no more scopes after it.
            if !args.has(i) {
                break;
            }
            if args.ty(i) != VAR_NUMBER {
                err0(e_invarg.as_ptr());
                return None;
            }
            s.number[i] = number_of(args.get(i)) as c_int;
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
            s.tp = find_tab(s.number[tab_i]);
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
                s.win = find_win(args, s.tp);
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
fn number_of(tv: &typval_T) -> varnumber_T {
    // SAFETY: only reached under a `VAR_NUMBER` tag, which is what makes
    // `v_number` the live arm.
    unsafe { tv.vval.v_number }
}

// ---------------------------------------------------------------------
// The builtins
// ---------------------------------------------------------------------

/// `chdir({dir})`: change directory in the narrowest scope that is already
/// local, answering the directory that was current before.
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 1..2, and `rettv`
/// a cleared result.
pub unsafe extern "C" fn f_chdir(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    ret_string(rettv, ptr::null_mut());
    if args.ty(0) != VAR_STRING {
        // Returning an empty string means it failed.  No error message, for
        // historic reasons.
        return;
    }

    // The answer is the directory that is current now.  It is taken before
    // the scope is parsed, so a bad scope reports *and* answers it.
    {
        let cwd = Owned::zeroed(MAXPATHL as usize);
        if os_cwd(&cwd) {
            rettv.vval.v_string = Owned::dup(cwd.cstr()).into_raw();
        }
    }

    let mut scope = kCdScopeGlobal;
    if args.has(1) {
        let s = str_arg(args, 1);
        scope = match s.to_bytes() {
            b"global" => kCdScopeGlobal,
            b"tabpage" => kCdScopeTabpage,
            b"window" => kCdScopeWindow,
            _ => {
                err2(e_invargNval.as_ptr(), c"scope".as_ptr(), s.as_ptr());
                return;
            }
        };
    } else if !win_localdir(curwin.get()).is_null() {
        scope = kCdScopeWindow;
    } else if !tab_localdir(curtab.get()).is_null() {
        scope = kCdScopeTabpage;
    }

    if !changedir(string_of(args.get(0)), scope) {
        // Directory change failed: answer the empty string after all.
        // SAFETY: the answer taken above is nvim's heap, or NULL.
        unsafe { xfree(rettv.vval.v_string.cast::<c_void>()) };
        rettv.vval.v_string = ptr::null_mut();
    }
}

/// `delete({fname} [, {flags}])`: remove a file, an empty directory (`d`) or
/// a whole tree (`rf`).
///
/// # Safety
/// As [`f_chdir`].
pub unsafe extern "C" fn f_delete(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1 as varnumber_T;
    if secure() {
        return;
    }
    let name = str_arg(args, 0);
    if name.to_bytes().is_empty() {
        err0(e_invarg.as_ptr());
        return;
    }

    let mut nbuf = numbuf();
    let flags = if args.has(1) {
        path_arg(args, 1, &mut nbuf)
    } else {
        c""
    };
    let name = name.as_ptr();
    let done = |ret: c_int| -> varnumber_T { if ret == 0 { 0 } else { -1 } };
    rettv.vval.v_number = match flags.to_bytes() {
        // SAFETY: `name` is NUL-terminated; each callee only reads it.
        b"" => done(unsafe { os_remove(name) }),
        b"d" => done(unsafe { os_rmdir(name) }),
        b"rf" => varnumber_T::from(unsafe { delete_recursive(name) }),
        _ => {
            err1(e_invexpr2.as_ptr(), flags.as_ptr());
            return;
        }
    };
}

/// `filecopy({from}, {to})`: copy a regular file or a symlink, answering
/// whether it worked.
///
/// # Safety
/// As [`f_chdir`], arity 2.
pub unsafe extern "C" fn f_filecopy(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = false as varnumber_T;
    if secure() || !is_string_arg(args, 0) || !is_string_arg(args, 1) {
        return;
    }

    let mut info = no_fileinfo();
    let from = str_arg(args, 0);
    // SAFETY: `from` is NUL-terminated, and `info` is this frame's own.
    let known = unsafe { os_fileinfo_link(from.as_ptr(), &raw mut info) };
    // `S_ISREG` and `S_ISLNK`: only a plain file or a symlink is copied.
    const S_IFREG: uint64_t = 0o100000;
    const S_IFLNK: uint64_t = 0o120000;
    let kind = info.stat.st_mode & __S_IFMT as uint64_t;
    if known && (kind == S_IFREG || kind == S_IFLNK) {
        let (from, to) = (str_arg(args, 0).as_ptr(), str_arg(args, 1).as_ptr());
        // SAFETY: both are NUL-terminated.
        rettv.vval.v_number = (unsafe { vim_copyfile(from, to) } == OK) as varnumber_T;
    }
}

/// `getcwd([{win} [, {tab}]])`: the working directory of the scope the
/// arguments name, always as a string.
///
/// # Safety
/// As [`f_chdir`], arity 0..2.
pub unsafe extern "C" fn f_getcwd(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    ret_string(rettv, ptr::null_mut());
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
        from = win_localdir(s.win);
    }
    if from.is_null() && (kCdScopeWindow..=kCdScopeTabpage).contains(&s.scope) {
        debug_assert!(!s.tp.is_null(), "tp");
        from = tab_localdir(s.tp);
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
    rettv.vval.v_string = Owned::dup(cwd.cstr()).into_raw();
}

/// `haslocaldir([{win} [, {tab}]])`: whether the scope the arguments name
/// has a directory of its own.
///
/// # Safety
/// As [`f_getcwd`].
pub unsafe extern "C" fn f_haslocaldir(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0 as varnumber_T;
    let Some(s) = Scope::read(args, true) else {
        return;
    };

    rettv.vval.v_number = match s.scope {
        kCdScopeWindow => {
            debug_assert!(!s.win.is_null(), "win");
            !win_localdir(s.win).is_null() as varnumber_T
        }
        kCdScopeTabpage => {
            debug_assert!(!s.tp.is_null(), "tp");
            !tab_localdir(s.tp).is_null() as varnumber_T
        }
        kCdScopeInvalid => {
            // We should never get here: the read above defaulted it.
            // SAFETY: `abort` does not return.
            unsafe { abort() };
        }
        // The global scope never has a local directory.
        _ => 0 as varnumber_T,
    };
}

/// `mkdir({name} [, {flags} [, {prot}]])`: create a directory, with `p`
/// creating the parents too and `D`/`R` registering its removal for when the
/// calling function returns.
///
/// # Safety
/// As [`f_chdir`], arity 1..3.
pub unsafe extern "C" fn f_mkdir(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // Upstream's default, which is *not* the 0777 the shell's `mkdir` uses.
    let mut prot: c_int = 0o755;
    // Held in a local and written back at each exit, so that the answer is
    // only ever written into the union, never read back out of it.
    let mut result = FAIL as varnumber_T;
    rettv.vval.v_number = result;
    if secure() {
        return;
    }

    let mut buf = numbuf();
    let dir = path_arg_raw(args, 0, &mut buf);
    // SAFETY: `dir` is NUL-terminated.
    if unsafe { *dir } == 0 {
        return;
    }
    strip_trailing_seps(dir);

    let mut defer = false;
    let mut defer_recurse = false;
    let mut created = ptr::null_mut();
    if args.has(1) {
        if args.has(2) {
            // With no error flag the failure answer is -1 rather than 0, and
            // -1 is exactly what the test below looks for.
            // SAFETY: a live typval; a null flag asks for that answer.
            prot = unsafe { tv_get_number_chk(args.ptr(2), ptr::null_mut()) } as c_int;
            if prot == -1 {
                return;
            }
        }
        // The flags are ASCII, so a plain byte search is `vim_strchr`.
        let arg2 = str_arg(args, 1).to_bytes();
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
                err2(e_mkdir.as_ptr(), failed_dir, strerror(ret));
                drop(Owned(failed_dir));
                rettv.vval.v_number = FAIL as varnumber_T;
                return;
            }
            result = OK as varnumber_T;
        }
    }
    if result == FAIL as varnumber_T {
        // SAFETY: `dir` is NUL-terminated; the callee reports its own error.
        result = unsafe { vim_mkdir_emsg(dir, prot) } as varnumber_T;
    }
    rettv.vval.v_number = result;

    // The "D" and "R" flags: deferred deletion of the created directory.
    if result == OK as varnumber_T && created.is_null() && (defer || defer_recurse) {
        // SAFETY: `dir` is NUL-terminated; the answer is nvim's heap.
        created = unsafe { FullName_save(dir, false) };
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
    unsafe {
        if *path_tail(dir) == 0 {
            *path_tail_with_sep(dir) = 0;
        }
    }
}

/// Register `delete({created}, "d"|"rf")` to run when the calling function
/// returns -- `mkdir()`'s `D` and `R` flags.
fn defer_delete(created: *mut c_char, recurse: bool) {
    let how = if recurse { c"rf" } else { c"d" };
    // SAFETY: a NUL-terminated literal; the copy is nvim's heap.
    let how = unsafe { xstrdup(how.as_ptr()) };
    let string = |s| typval_T {
        v_type: VAR_STRING,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_string: s },
    };
    let mut tv = [string(created), string(how)];
    let name = c"delete".as_ptr().cast_mut();
    // SAFETY: two arguments, at `tv`, whose contents the callee takes over.
    unsafe { add_defer(name, 2, tv.as_mut_ptr()) };
}

/// `rename({from}, {to})`: move a file, 0 on success.
///
/// # Safety
/// As [`f_chdir`], arity 2.
pub unsafe extern "C" fn f_rename(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    if secure() {
        rettv.vval.v_number = -1 as varnumber_T;
        return;
    }
    let mut buf = numbuf();
    let (from, to) = (
        str_arg(args, 0).as_ptr(),
        path_arg(args, 1, &mut buf).as_ptr(),
    );
    // SAFETY: both are NUL-terminated.
    rettv.vval.v_number = unsafe { vim_rename(from, to) } as varnumber_T;
}

/// `tempname()`: a fresh name in the session's own temporary directory.
///
/// # Safety
/// As [`f_chdir`], arity 0.
pub unsafe extern "C" fn f_tempname(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (_, rettv) = frame!(argvars, rettv);
    // SAFETY: answers a fresh string in nvim's heap, or NULL.
    ret_string(rettv, unsafe { vim_tempname() });
}
