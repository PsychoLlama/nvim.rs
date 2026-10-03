//! Looking for files -- `glob()`, `globpath()`, `finddir()`, `findfile()` and
//! `readdir()`.
//!
//! `f_glob` and `f_globpath` expand a wildcard pattern through the same
//! `expand_one`/`globpath` machinery the command line uses, so they answer to
//! 'wildignore', 'suffixes' and 'wildignorecase'; `findfilendir` is the shared
//! body of `finddir()`/`findfile()`, which walk 'path' looking for a name
//! rather than expanding a pattern; and `f_readdir` lists one directory,
//! optionally filtering each entry through a callback that `readdir_checkitem`
//! evaluates (so the filter re-enters the evaluator on every name).
//!
//! Each of the four answers either one string or a List of them, decided by a
//! flag argument before anything is expanded -- which is why the List has to
//! be reachable from `rettv` (as [`RetList`]) rather than held as a local.
//!
//! # What holds the results
//!
//! [`Expander`] is the wildcard expander, and `readdir_core` answers a
//! `Vec` of owned names; both hand out their names as a slice and free
//! them on every path out.  **The order of those names is
//! behaviour** -- `readdir_core` and `gen_expand_wildcards` sort them
//! themselves -- so nothing here re-orders anything.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::{FINDFILE_DIR, FINDFILE_FILE, RetList, nr_arg, str_arg, str_arg_chk};
use crate::cmdexpand::{WildMode, WildOpts, expand_cleanup, expand_one, globpath};
use crate::eval::eval_expr_typval;
use crate::eval::typval::CallFrame;
use crate::eval::typval::NumBuf;
use crate::eval::typval::{TV_INITIAL_VALUE, tv_clear, tv_get_number_chk};
use crate::eval::vars::{prepare_vimvar, restore_vimvar, set_vim_var_string};
use crate::file_search::{FileNameOpts, find_file_in_path_option, vim_findfile_cleanup};
use crate::fileio::readdir_core;
use crate::memory::{XString, xfree};
use crate::option::vars::p_wic;
use crate::optionstr::OptString;
use crate::path::buffer_path;
use crate::types::{
    EvalFuncData, Expand, ExpandContext, TypVal, VAR_LIST, VAR_STRING, VarNumber, Vv,
    kListLenUnknown, ptrdiff_t, size_t,
};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

// ---------------------------------------------------------------------
// The two things that hold the names found
// ---------------------------------------------------------------------

/// The wildcard expander, over file names.
struct Expander(Expand);

impl Expander {
    /// A fresh expander for file names.
    fn new() -> Self {
        let mut xpc = Expand::new();
        xpc.context = ExpandContext::Files;
        Self(xpc)
    }

    /// Expand `pat`.  `WildMode::All` answers the matches joined into the string
    /// this returns; `WildMode::AllKeep` leaves them in [`Expander::files`].
    fn one(&mut self, pat: &CStr, options: WildOpts, mode: WildMode) -> Option<XString> {
        expand_one(&mut self.0, Some(pat), None, options, mode)
    }

    /// The names a `WildMode::AllKeep` expansion left behind, in the order
    /// `gen_expand_wildcards` sorted them into.
    fn files(&self) -> &[XString] {
        self.0.matches()
    }

    /// How many names the last expansion left: -1 until one succeeds, which
    /// is `kListLenUnknown` and what the List is then allocated with.
    fn count(&self) -> c_int {
        self.0.match_count()
    }

    fn cleanup(&mut self) {
        expand_cleanup(&mut self.0);
    }
}

// ---------------------------------------------------------------------
// Small wrappers over what the four builtins reach for
// ---------------------------------------------------------------------

/// Answer a List rather than a String.  Which list is decided later, once
/// the number of matches is known.
/// Answer nothing, under whichever tag [`ret_list`] left behind: `{list}`
/// asks for a List and the rest of these answer a String, and the empty
/// form of both is a NULL payload.
fn empty_answer(result: &mut TypVal) {
    if result.v_type() == VAR_LIST {
        result.write_list(None);
    } else {
        result.write_string(ptr::null_mut());
    }
}

fn ret_list(result: &mut TypVal) {
    result.write_list(None);
}

fn free(p: *mut c_char) {
    // SAFETY: `p` is an owned string, or NULL.
    unsafe { xfree(p.cast::<c_void>()) };
}

/// The suffixes `findfile()` tries, and none for `finddir()`.
fn suffixes(find_what: c_int) -> *mut c_char {
    if find_what == FINDFILE_DIR as c_int {
        return c"".as_ptr().cast_mut();
    }
    Buf::current().b_p_sua.value_ptr()
}

/// Set `v:val`, or clear it when `name` is NULL.
fn set_val(name: *const c_char) {
    let len: ptrdiff_t = if name.is_null() { 0 } else { -1 };
    // SAFETY: `Vv::Val` names a `v:` variable, and a length of -1 promises a
    // NUL-terminated string, which every directory entry's name is.
    unsafe { set_vim_var_string(Vv::Val, name, len) };
}

// ---------------------------------------------------------------------
// The builtins
// ---------------------------------------------------------------------

/// The shared body of `finddir()` and `findfile()`: walk 'path' for `count`
/// matches of a name, answering the last one -- or, for a negative count,
/// all of them as a List.
fn findfilendir(args: &[TypVal], result: &mut TypVal, find_what: c_int) {
    let mut numbuf = NumBuf::new();
    let mut fresult: *mut c_char = ptr::null_mut();
    let mut path = buffer_path();
    let mut count = 1;
    let mut error = false;

    result.write_string(ptr::null_mut());
    let fname = str_arg(args, 0, &mut numbuf);

    let mut pathbuf = NumBuf::new();
    if args.len() > 1 {
        match str_arg_chk(args, 1, &mut pathbuf) {
            None => error = true,
            Some(p) => {
                if !p.to_bytes().is_empty() {
                    path = p.as_ptr().cast_mut();
                }
                if args.len() > 2 {
                    count = nr_arg(args, 2, &mut error) as c_int;
                }
            }
        }
    }
    if count < 0 {
        RetList::alloc(result, kListLenUnknown as c_int as ptrdiff_t);
    }
    if fname.to_bytes().is_empty() || error {
        return;
    }

    let (mut to_find, mut ctx): (*mut c_char, *mut c_char) = (ptr::null_mut(), ptr::null_mut());
    let (name, len) = (fname.as_ptr().cast_mut(), fname.to_bytes().len() as size_t);
    let (sua, mut first) = (suffixes(find_what), true);
    loop {
        // The previous answer, which was either copied into the List or is
        // about to be replaced.
        free(fresult);
        // SAFETY: `curbuf` names the live current buffer.
        let rel = Buf::current().name.full_ptr();
        // Only the first round is given the name; the ones after it continue
        // the walk the context remembers.
        let (p, n) = if first {
            (name, len)
        } else {
            (ptr::null_mut(), 0)
        };
        let (f2f, c) = (&raw mut to_find, &raw mut ctx);
        // `findfile()` is quiet and takes the name as written: no message,
        // no `'includeexpr'`, no relative-path preference.
        let quiet = FileNameOpts::NONE;
        // SAFETY: `p` is NUL-terminated with `n` bytes, or NULL; `path` and
        // `sua` are option strings; `rel` is the current buffer's own name;
        // and the two out-parameters carry the walk's state from one round
        // to the next.
        fresult = unsafe {
            find_file_in_path_option(p, n, quiet, first, path, find_what, rel, sua, f2f, c)
        };
        first = false;
        if !fresult.is_null() && result.v_type() == VAR_LIST {
            RetList::of(result).push(fresult);
        }
        let more = result.v_type() == VAR_LIST || {
            count -= 1;
            count > 0
        };
        if !more || fresult.is_null() {
            break;
        }
    }
    free(to_find);
    // SAFETY: the context this call's own loop built, or NULL.
    unsafe { vim_findfile_cleanup(ctx.cast::<c_void>()) };

    // The List answer appended a copy of each match and only leaves the
    // loop on a NULL, so there is nothing left to hand back there.
    if result.v_type() == VAR_STRING {
        result.write_string(fresult);
    }
}

/// `finddir({name} [, {path} [, {count}]])`.
pub fn f_finddir(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    findfilendir(args, result, FINDFILE_DIR as c_int);
}

/// `findfile({name} [, {path} [, {count}]])`.
pub fn f_findfile(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    findfilendir(args, result, FINDFILE_FILE as c_int);
}

/// `glob({pattern} [, {nosuf} [, {list} [, {alllinks}]]])`.
///
/// A non-zero `{nosuf}` keeps the matches 'wildignore' would drop and leaves
/// the ones 'suffixes' would push to the end where they are; `{list}` asks
/// for a List rather than newline-joined text.
pub fn f_glob(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut options = WildOpts::SILENT | WildOpts::USE_NL;
    let mut error = false;

    result.write_empty(VAR_STRING);
    if args.len() > 1 {
        if nr_arg(args, 1, &mut error) != 0 {
            options |= WildOpts::KEEP_ALL;
        }
        if args.len() > 2 {
            if nr_arg(args, 2, &mut error) != 0 {
                ret_list(result);
            }
            if args.len() > 3 && nr_arg(args, 3, &mut error) != 0 {
                options |= WildOpts::ALLLINKS;
            }
        }
    }
    if error {
        empty_answer(result);
        return;
    }

    let mut xpc = Expander::new();
    if p_wic() {
        options |= WildOpts::ICASE;
    }
    let pat = str_arg(args, 0, &mut numbuf);
    if result.v_type() == VAR_STRING {
        result.write_string(
            xpc.one(pat, options, WildMode::All)
                .map_or(ptr::null_mut(), XString::into_raw),
        );
        return;
    }
    xpc.one(pat, options, WildMode::AllKeep);
    let list = RetList::alloc(result, xpc.count() as ptrdiff_t);
    for name in xpc.files() {
        list.push(name.as_ptr());
    }
    xpc.cleanup();
}

/// `globpath({path}, {pattern} [, {nosuf} [, {list} [, {alllinks}]]])`: the
/// pattern expanded once under every directory in `{path}`.
pub fn f_globpath(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut flags = WildOpts::IGNORE_COMPLETESLASH;
    let mut error = false;

    result.write_empty(VAR_STRING);
    if args.len() > 2 {
        if nr_arg(args, 2, &mut error) != 0 {
            flags |= WildOpts::KEEP_ALL;
        }
        if args.len() > 3 {
            if nr_arg(args, 3, &mut error) != 0 {
                ret_list(result);
            }
            if args.len() > 4 && nr_arg(args, 4, &mut error) != 0 {
                flags |= WildOpts::ALLLINKS;
            }
        }
    }

    let mut buf1 = NumBuf::new();
    let file = str_arg_chk(args, 1, &mut buf1);
    let (Some(file), false) = (file, error) else {
        empty_answer(result);
        return;
    };

    let path = str_arg(args, 0, &mut numbuf);
    let found = globpath(path, file, flags, false);

    if result.v_type() == VAR_STRING {
        let mut joined = XString::new();
        for (i, name) in found.iter().enumerate() {
            if i > 0 {
                joined.push_byte(b'\n');
            }
            joined.push_cstr(name.as_cstr());
        }
        result.write_string(joined.into_raw());
        return;
    }
    let list = RetList::alloc(result, ptrdiff_t::try_from(found.len()).unwrap_or(0));
    for name in &found {
        list.push(name.as_ptr());
    }
}

/// The per-entry filter `readdir()` hands `readdir_core`: evaluate the
/// caller's expression with the name as `v:val` and as its one argument.
///
/// Answers 1 to keep the entry, 0 to skip it, -1 to stop the walk -- and 1
/// when there is no expression at all.
///
/// # Safety
/// `context` is null, or the `TypVal` `f_readdir` handed `readdir_core`; and
/// `name` is a NUL-terminated entry name.
unsafe fn readdir_checkitem(context: *mut c_void, name: *const c_char) -> VarNumber {
    if context.is_null() {
        return 1;
    }
    // SAFETY: the caller's contract.
    let expr = unsafe { &mut *context.cast::<TypVal>() };

    let mut save_val = TV_INITIAL_VALUE;
    prepare_vimvar(Vv::Val, &mut save_val);
    set_val(name);

    // The callee only reads it, so the frame names the caller's string
    // rather than copying it.
    let argv = CallFrame::naming([TypVal::String(name.cast_mut())]);

    let mut rettv = TV_INITIAL_VALUE;
    let mut retval = 0;
    let ran = eval_expr_typval(expr, false, argv.args(), &mut rettv);
    if ran.is_ok() {
        retval = tv_get_number_chk(&rettv).unwrap_or(-1);
        tv_clear(&mut rettv);
    }

    set_val(ptr::null());
    restore_vimvar(Vv::Val, &mut save_val);
    retval
}

/// `readdir({directory} [, {expr}])`: the entries of one directory, sorted,
/// with `{expr}` deciding which of them to keep.
pub fn f_readdir(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let list = RetList::alloc(result, kListLenUnknown as c_int as ptrdiff_t);
    let path = str_arg(args, 0, &mut numbuf).as_ptr();
    // No filter expression is a null context, which the callback reads as
    // "keep everything".
    let expr = args
        .get(1)
        .map_or(ptr::null_mut(), |tv| ptr::from_ref(tv).cast_mut().cast());

    // SAFETY: `path` is NUL-terminated, and `expr` is null or the argument
    // the filter reads back through.
    if let Ok(found) = unsafe { readdir_core(path, expr, Some(readdir_checkitem)) } {
        for name in &found {
            list.push(name.as_ptr());
        }
    }
}
