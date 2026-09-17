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
//! [`Expander`] is the wildcard expander and [`StrArray`] the `GArray` of
//! owned strings `globpath()` and `readdir_core` fill; both hand out their
//! names as a slice, and [`StrArray`] frees itself, which is upstream's
//! `ga_clear_strings` on every path out.  **The order of those names is
//! behaviour** -- `readdir_core` and `gen_expand_wildcards` sort them
//! themselves -- so nothing here re-orders anything.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::{
    FINDFILE_DIR, FINDFILE_FILE, RetList, XP_PREFIX_NONE, kDirectionNotSet, nr_arg, str_arg,
    str_arg_chk,
};
use crate::cmdexpand::{WildMode, WildOpts, expand_cleanup, expand_init, expand_one, globpath};
use crate::eval::eval_expr_typval;
use crate::eval::typval::CallFrame;
use crate::eval::typval::NumBuf;
use crate::eval::typval::{TV_INITIAL_VALUE, tv_clear, tv_get_number_chk};
use crate::eval::vars::{prepare_vimvar, restore_vimvar, set_vim_var_string};
use crate::file_search::{FileNameOpts, find_file_in_path_option, vim_findfile_cleanup};
use crate::fileio::readdir_core;
use crate::garray::{ga_clear_strings, ga_concat_strings, ga_init};
use crate::memory::xfree;
use crate::option::local_or_global_raw;
use crate::option::vars::{P_PATH, p_wic};
use crate::optionstr::LocalOptStr;
use crate::types::{
    BackslashEscape, EvalFuncData, Expand, ExpandContext, GArray, Pos, ScriptCtx, TypVal, VAR_LIST,
    VAR_STRING, VarNumber, Vv, kListLenUnknown, ptrdiff_t, size_t,
};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::{ptr, slice};

// ---------------------------------------------------------------------
// The two things that hold the names found
// ---------------------------------------------------------------------

/// The wildcard expander, over file names.
struct Expander(Expand);

impl Expander {
    /// A fresh expander with 'wildignorecase' folded into the caller's
    /// options, as `glob()` wants it.
    fn new() -> Self {
        let mut xpc = Expand {
            xp_pattern: ptr::null_mut(),
            xp_context: ExpandContext::Nothing,
            xp_pattern_len: 0,
            xp_prefix: XP_PREFIX_NONE,
            xp_arg: ptr::null_mut(),
            xp_luaref: 0,
            xp_script_ctx: ScriptCtx::NONE,
            xp_backslash: BackslashEscape::NONE,
            xp_shell: false,
            xp_numfiles: 0,
            xp_col: 0,
            xp_selected: 0,
            xp_orig: ptr::null_mut(),
            xp_files: ptr::null_mut(),
            xp_line: ptr::null_mut(),
            xp_buf: [0; 1025],
            xp_search_dir: kDirectionNotSet,
            xp_pre_incsearch_pos: Pos {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
        };
        // SAFETY: a fresh local, which is all `expand_init` writes over.
        unsafe { expand_init(&raw mut xpc) };
        xpc.xp_context = ExpandContext::Files;
        Self(xpc)
    }

    /// Expand `pat`.  `WildMode::All` answers the matches joined into the string
    /// this returns; `WildMode::AllKeep` leaves them in [`Expand::files`].
    fn one(&mut self, pat: &CStr, options: WildOpts, mode: WildMode) -> *mut c_char {
        let (p, orig) = (pat.as_ptr().cast_mut(), ptr::null_mut());
        // SAFETY: an initialised expander and a NUL-terminated pattern, which
        // `expand_one` only reads; a NULL `orig` asks for no old match.
        unsafe { expand_one(&raw mut self.0, p, orig, options, mode) }
    }

    /// The names a `WildMode::AllKeep` expansion left behind, in the order
    /// `gen_expand_wildcards` sorted them into.
    fn files(&self) -> &[*mut c_char] {
        if self.0.xp_numfiles <= 0 {
            return &[];
        }
        // SAFETY: `xp_numfiles` is how many names `xp_files` holds, and it is
        // positive here.
        unsafe { slice::from_raw_parts(self.0.xp_files, self.0.xp_numfiles as usize) }
    }

    /// How many names the last expansion left: -1 until one succeeds, which
    /// is `kListLenUnknown` and what the List is then allocated with.
    fn count(&self) -> c_int {
        self.0.xp_numfiles
    }

    fn cleanup(&mut self) {
        // SAFETY: an initialised expander.
        unsafe { expand_cleanup(&raw mut self.0) };
    }
}

/// A `GArray` of owned strings -- what `globpath()` and `readdir_core`
/// fill, and what frees them.
struct StrArray(GArray);

impl Drop for StrArray {
    fn drop(&mut self) {
        // SAFETY: an initialised array whose items are all owned strings.
        unsafe { ga_clear_strings(&raw mut self.0) };
    }
}

impl StrArray {
    fn new() -> Self {
        let mut ga = GArray {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ptr::null_mut(),
        };
        // SAFETY: a fresh local.
        unsafe { ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 10) };
        Self(ga)
    }

    fn raw(&mut self) -> *mut GArray {
        &raw mut self.0
    }

    /// The names, in the order they were put in.
    fn names(&self) -> &[*const c_char] {
        if self.0.ga_len <= 0 {
            return &[];
        }
        // SAFETY: the array holds `ga_len` items of one pointer each.
        unsafe { slice::from_raw_parts(self.0.ga_data.cast(), self.0.ga_len as usize) }
    }

    fn len(&self) -> c_int {
        self.0.ga_len
    }

    /// The names joined by `sep`, as one owned string.
    fn joined(&self, sep: &CStr) -> *mut c_char {
        // SAFETY: an initialised array of NUL-terminated strings.
        unsafe { ga_concat_strings(&raw const self.0, sep.as_ptr()) }
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

/// The 'path' a search walks: the buffer's own when it set one, else the
/// global option.
///
/// The option's own buffer, not a copy: `find_file_in_path_option` keeps its
/// position in a static and resumes it on a later call. See
/// [`crate::option::local_or_global_raw`].
fn search_path() -> *mut c_char {
    // SAFETY: `curbuf` names the live current buffer, and its option values
    // are NUL-terminated.
    unsafe { local_or_global_raw(&Buf::current().b_p_path, P_PATH) }
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
    let mut path = search_path();
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
        result.write_string(xpc.one(pat, options, WildMode::All));
        return;
    }
    xpc.one(pat, options, WildMode::AllKeep);
    let list = RetList::alloc(result, xpc.count() as ptrdiff_t);
    for &name in xpc.files() {
        list.push(name);
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

    let mut found = StrArray::new();
    let path = str_arg(args, 0, &mut numbuf).as_ptr().cast_mut();
    // SAFETY: two NUL-terminated strings, which `globpath` only reads, and an
    // initialised array for it to append the matches to.
    unsafe { globpath(path, file.as_ptr().cast_mut(), found.raw(), flags, false) };

    if result.v_type() == VAR_STRING {
        result.write_string(found.joined(c"\n"));
        return;
    }
    let list = RetList::alloc(result, found.len() as ptrdiff_t);
    for &name in found.names() {
        list.push(name);
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

    let mut found = StrArray::new();
    // SAFETY: `path` is NUL-terminated, `expr` is null or the argument the
    // filter reads back through, and the array is a fresh one to fill.
    let ret = unsafe { readdir_core(found.raw(), path, expr, Some(readdir_checkitem)) };
    if ret.is_ok() {
        for &name in found.names() {
            list.push(name);
        }
    }
}
