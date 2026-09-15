//! What the editor remembers about scripts it has sourced, and how a sourced
//! script is read and left.
//!
//! `script_items` is the registry -- one entry per script ever sourced, with
//! its name, its script-local variables, its `<SID>` and its profiling
//! counters.  [`scripts`] and [`script_item`] are how the rest of the family
//! reaches it; [`ex_scriptnames`], [`f_getscriptinfo`], [`get_scriptname`] and
//! [`find_script_by_name`] are its readers, and [`script_autoload`] is the
//! lookup that turns `foo#bar()` into an `autoload/foo.vim` to source.
//!
//! [`getsourceline`] and `get_one_sourceline` are the reader `do_cmdline` pulls
//! from while a script runs -- the place `\` continuation lines are joined,
//! 'scriptencoding' conversion is applied and the debugger gets its per-line
//! hook.  [`ex_finish`] and [`source_finished`] are how a script stops early.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::memory::handoff::owned_cstr;
use crate::message_fmt::c_str;
use crate::semsg;

use crate::cstr;
use crate::eval::typval::NumBuf;
use crate::ex_eval::CsFlags;
use crate::option::cpo_has;
use crate::types::{CpoFlag, IOSIZE, ListRef, MAXPATHL, NUL};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::{ptr, slice};
use std::ffi::CString;

/// Offset of `uf_name` inside `UserFunc`: the function table's hash keys point
/// at that inline buffer, so backing up by this recovers the function.  This is
/// the transpiled `HI2UF`, the same constant profile.rs and userfunc.rs use.
const UF_NAME_OFFSET: usize = 240;

/// Bytes `autoload_name` puts in front of the name it is given.
const AUTOLOAD_PREFIX: &[u8] = b"autoload/";
/// ...and what it puts after the last package separator, NUL included.
const AUTOLOAD_SUFFIX: &[u8] = b".vim\0";

// ---------------------------------------------------------------------------
// The registry.

/// How many scripts the editor has sourced -- upstream's
/// `script_items.ga_len`, which is also the highest live script id.
pub(crate) fn script_count() -> c_int {
    // The registry only grows, and an editor that has sourced 2^31 scripts
    // has run out of ids long before it runs out of `c_int`.
    script_items.with(|items| items.len() as c_int)
}

/// The registry entry for script `sid` -- upstream's `SCRIPT_ITEM`.
///
/// Null for an id outside `1..=`[`script_count`], which upstream's macro
/// would read past the end for; debug builds fail the assertion instead.
pub(crate) fn script_item(sid: ScriptId) -> *mut ScriptItem {
    debug_assert!(script_id_valid(sid), "script id out of range");
    let idx = usize::try_from(sid - 1).ok();
    script_items.with(|items| {
        idx.and_then(|idx| items.get(idx).copied())
            .unwrap_or(ptr::null_mut())
    })
}

/// Is `sid` a script the editor has sourced -- upstream's `SCRIPT_ID_VALID`?
pub(crate) fn script_id_valid(sid: c_int) -> bool {
    sid > 0 && sid <= script_count()
}

/// Was script `sid` written in Lua?
pub fn script_is_lua(sid: ScriptId) -> bool {
    if sid == SID_LUA {
        return true;
    }
    if !script_id_valid(sid) {
        return false;
    }
    // SAFETY: checked just above.
    unsafe { (*script_item(sid)).sn_lua }
}

/// Find an already loaded script `name`, and return its script ID.
///
/// Returns -1 when there is none.  We used to check the inode here, but that
/// does not work: a script that is edited and written may get a different inode
/// even though to the user it is the same script, and a deleted script's inode
/// may be re-used by a differently named one.
pub fn find_script_by_name(name: &CStr) -> c_int {
    // Nothing in the closure sources a script, so holding the borrow over the
    // walk is sound.
    let found = script_items.with(|items| {
        items.iter().rposition(|&si| {
            // SAFETY: a registry slot always holds a live `ScriptItem`, and
            // `path_fnamecmp` only reads the two NUL-terminated names.
            unsafe { !(*si).sn_name.is_null() && path_fnamecmp(cstr::at((*si).sn_name), name) == 0 }
        })
    });
    found.map_or(-1, |idx| idx as c_int + 1)
}

// ---------------------------------------------------------------------------
// `:scriptnames`.

/// `":scriptnames"`, and `":script {id}"` which edits the script instead.
pub fn ex_scriptnames(excmd: &mut ExArg) {
    // SAFETY: `excmd` is the command's own argument block.
    let (by_number, has_arg) = unsafe { (excmd.addr_count > 0, *excmd.arg != NUL as c_char) };
    if by_number || has_arg {
        // SAFETY: same block; `edit_script` only reads it and `do_exedit`.
        edit_script(excmd, by_number);
        return;
    }

    // SAFETY: `msg_ext_set_kind` copies the literal.
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    // The listing pauses for the user, which can run anything; both buffers
    // are this frame's rather than the shared scratch upstream reuses.
    let mut shortname = [0 as c_char; MAXPATHL as usize];
    let mut row = [0 as c_char; IOSIZE as usize];
    let mut sid: ScriptId = 1;
    while sid <= script_count() && !got_int.get() {
        // SAFETY: `sid` is in range, and the registry is re-read every round
        // because the output below can pause for the user.
        let name = unsafe { (*script_item(sid)).sn_name };
        let listed = sid;
        sid += 1;
        if name.is_null() {
            continue;
        }
        // SAFETY: both buffers are sized as the calls below are told, and
        // outlive every one of them.
        let namebuff = shortname.as_mut_ptr();
        unsafe { home_replace(None, name, namebuff, MAXPATHL as size_t, true) };
        let iobuff = row.as_mut_ptr();
        unsafe {
            vim_snprintf(
                iobuff,
                IOSIZE as size_t,
                c"%3d: %s".as_ptr(),
                listed,
                namebuff,
            )
        };
        if !message_filtered(unsafe { cstr::at(iobuff) }) {
            if msg_col.get() > 0 {
                msg_putchar('\n' as c_int);
            }
            msg_display(unsafe { cstr::at(iobuff) }, 0, false);
            line_breakcheck();
        }
    }
}

/// `":script {id}"` / `":script {file}"`: open the named script in a window.
fn edit_script(excmd: &mut ExArg, by_number: bool) {
    let mut path = [0 as c_char; MAXPATHL as usize];
    if by_number {
        if !script_id_valid(excmd.line2 as c_int) {
            emsg(gettext(e_invarg));
            return;
        }
        unsafe { excmd.arg = (*script_item(excmd.line2 as ScriptId)).sn_name };
    } else {
        let namebuff = path.as_mut_ptr();
        unsafe { expand_env(excmd.arg, namebuff, MAXPATHL) };
        excmd.arg = namebuff;
    }
    do_exedit(excmd, None);
}

/// A script's name, for `":verbose set"` -- the text appended to "Last set
/// from ".
///
/// The negative script IDs are the contexts that have no file: a modeline,
/// the `--cmd` or `-c` command line, and so on. `fold_home` asks for a file
/// path with the home directory folded back to `~`; without it the
/// registry's own spelling is answered.
///
/// The answer is owned. Upstream answers a pointer into the shared `IObuff`
/// for the two contexts it has to format, and a caller holding one of those
/// across anything that shows a message loses it.
pub(crate) fn get_scriptname(script_ctx: ScriptCtx, fold_home: bool) -> CString {
    let mut named = [0 as c_char; IOSIZE as usize];
    let fixed = match script_ctx.sc_sid {
        SID_MODELINE => c"modeline",
        SID_CMDARG => c"--cmd argument",
        SID_CARG => c"-c argument",
        SID_ENV => c"environment variable",
        SID_ERROR => c"error handler",
        SID_WINLAYOUT => c"changed window size",
        SID_LUA => c"Lua",
        SID_STR => c"anonymous :source",
        SID_API_CLIENT => {
            // SAFETY: `named` is `IOSIZE` writable bytes.
            unsafe {
                snprintf(
                    named.as_mut_ptr(),
                    IOSIZE as size_t,
                    gettext(c"API client (channel id %lu)").as_ptr(),
                    script_ctx.sc_chan,
                )
            };
            return cstr::in_chars(&named).to_owned();
        }
        _ => {
            // SAFETY: every other `sc_sid` is a registry index.
            let sname = unsafe { (*script_item(script_ctx.sc_sid)).sn_name };
            if sname.is_null() {
                // SAFETY: `named` is `IOSIZE` writable bytes.
                unsafe {
                    snprintf(
                        named.as_mut_ptr(),
                        IOSIZE as size_t,
                        gettext(c"anonymous :source (script id %d)").as_ptr(),
                        script_ctx.sc_sid,
                    )
                };
                return cstr::in_chars(&named).to_owned();
            }
            if !fold_home {
                // SAFETY: the registry's own NUL-terminated name.
                return unsafe { CStr::from_ptr(sname) }.to_owned();
            }
            // SAFETY: the registry's name, folded into a fresh allocation.
            return unsafe {
                let folded = home_replace_save(None, sname);
                let owned = CStr::from_ptr(folded).to_owned();
                xfree(folded.cast::<c_void>());
                owned
            };
        }
    };
    // SAFETY: `gettext` returns a pointer into its own catalogue.
    unsafe { CStr::from_ptr(gettext(fixed).as_ptr()) }.to_owned()
}

/// The line number to report for a message raised under `fgetline`.
///
/// A sourced script tracks its own read position, because the execution stack's
/// number lags behind by the one line `getsourceline` reads ahead.
///
/// # Safety
///
/// `fgetline` must be an initialized `LineGetter` whose pointer fields point
/// at live data for the call. `cookie` must be the payload `fgetline` was
/// registered with, live for the call.
pub unsafe fn get_sourced_lnum(fgetline: LineGetter, cookie: *mut c_void) -> LineNr {
    if !getline_is_source(fgetline) {
        return sourcing_lnum();
    }
    // SAFETY: a `getsourceline` reader always carries a `SourceCookie`.
    unsafe { (*cookie.cast::<SourceCookie>()).sourcing_lnum }
}

/// Is `fgetline` the reader [`getsourceline`] installs?
fn getline_is_source(fgetline: LineGetter) -> bool {
    fgetline.is_some_and(|f| ptr::fn_addr_eq(f, getsourceline as LineGetterFn))
}

// ---------------------------------------------------------------------------
// `getscriptinfo()`.

/// The script-local functions defined in the script with id `sid`, as a list of
/// their names.
fn get_script_local_funcs(sid: ScriptId) -> ListRef {
    let functbl = func_tbl_get();
    // SAFETY: the process-wide function table, which outlives this walk, and
    // a fresh list with at most one entry per function.
    let list = unsafe { tv_list_alloc((*functbl).ht_used as ptrdiff_t) };
    let l = list.as_ptr();

    for hi in unsafe { tv_ht_iter(functbl) } {
        // SAFETY: an occupied slot's key is a `UserFunc`'s inline name buffer,
        // so backing up by that field's offset recovers the function.
        let fp = unsafe { &*hi.hi_key.byte_sub(UF_NAME_OFFSET).cast::<UserFunc>() };
        if fp.uf_script_ctx.sc_sid != sid {
            continue;
        }
        let name = if fp.uf_name_exp.is_null() {
            (&raw const fp.uf_name).cast::<c_char>()
        } else {
            fp.uf_name_exp
        };
        // SAFETY: `name` is NUL-terminated, which the -1 length asks for.
        unsafe { (*l).push_string(name, -1) };
    }
    list
}

/// Which scripts `getscriptinfo()` was asked about.
enum ScriptQuery {
    /// No argument, or one that named neither key: every script, names only.
    All,
    /// `{'sid': n}`: that one script, with its variables and functions too.
    Sid(VarNumber),
    /// `{'name': pat}`: the scripts whose path matches the caller's `regmatch`.
    Matching,
    /// The argument was rejected; an error is already pending.
    Rejected,
}

/// `"getscriptinfo()"` function
pub fn f_getscriptinfo(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `result` is the caller's return slot, `args` its arguments.
    tv_list_alloc_ret(result, script_count() as ptrdiff_t);
    if tv_check_for_opt_dict_arg(args, 0).is_err() {
        return;
    }
    // The pattern's source string is freed on the way out, as upstream does,
    // even when it did not compile.
    let mut pat: *mut c_char = ptr::null_mut();
    // One `RegMatch` for the whole run, not one per script: `vim_regexec`
    // may swap the compiled program out from under it when the NFA engine
    // gives up and the backtracker recompiles the pattern.
    let mut regmatch = empty_regmatch();
    // SAFETY: as above.
    let query = unsafe { script_query(args, &mut pat, &mut regmatch) };

    if !matches!(query, ScriptQuery::Rejected) {
        // SAFETY: `result` holds the list allocated above.
        let l = result.list_or_null();
        // SAFETY: nothing in the loop sources a script.
        unsafe { report_scripts(l, &query, &mut regmatch) };
    }

    // SAFETY: both were allocated by the call that produced `query`; either
    // may be null, which both frees accept.
    unsafe { vim_regfree(regmatch.regprog) };
    unsafe { xfree(pat.cast::<c_void>()) };
}

/// Read `getscriptinfo()`'s optional dict argument.
///
/// `pat` receives the raw `name` string so the caller can free it whatever
/// happens, and `regmatch` its compiled form; a pattern that fails to compile
/// is not an error, it just filters nothing.
///
/// # Safety
///
/// `args` must be the builtin's argument vector.
unsafe fn script_query(
    args: &[TypVal],
    pat: *mut *mut c_char,
    regmatch: &mut RegMatch,
) -> ScriptQuery {
    let mut numbuf = NumBuf::new();
    let Some(arg) = args.first().filter(|a| a.v_type() == VAR_DICT) else {
        return ScriptQuery::All;
    };
    // The tag was tested above, so this is the argument's own dictionary.
    let dict = arg.dict_or_null();

    // SAFETY: the argument's own dictionary.
    if let Some(sid_di) = dict_find(unsafe { dict.as_ref() }, b"sid") {
        let Ok(sid) = tv_get_number_chk(&sid_di.di_tv) else {
            return ScriptQuery::Rejected;
        };
        if sid <= 0 {
            // SAFETY: the message borrows the item's string form.
            let arg1 = unsafe { c_str(numbuf.string_ptr(&sid_di.di_tv)) };
            semsg!("E475: Invalid value for argument {}: {arg1}", "sid");
            return ScriptQuery::Rejected;
        }
        return ScriptQuery::Sid(sid);
    }

    // SAFETY: the string is allocated for us and handed straight to the caller.
    unsafe { *pat = dict_get_string_alloc((dict).as_ref(), b"name") };
    if !unsafe { *pat }.is_null() {
        regmatch.regprog = unsafe { vim_regcomp(*pat, RE_MAGIC + RE_STRING) };
    }
    if regmatch.regprog.is_null() {
        ScriptQuery::All
    } else {
        ScriptQuery::Matching
    }
}

/// Append one dict per script `query` selects to `l`.
///
/// # Safety
///
/// `l` must be a live list, and `query` must still own its compiled pattern.
unsafe fn report_scripts(l: *mut List, query: &ScriptQuery, regmatch: &mut RegMatch) {
    let total = VarNumber::from(script_count());
    // A `sid` query asks about exactly one script, and answers nothing at all
    // when that script does not exist.
    let (first, last) = match *query {
        ScriptQuery::Sid(sid) => (sid, sid.min(total)),
        _ => (1, total),
    };

    for sid in first..=last {
        // SAFETY: `sid` is in range, and nothing in the body sources a script.
        let si = script_item(sid as ScriptId);
        // SAFETY: a registry slot always holds a live `ScriptItem`.
        let name = unsafe { (*si).sn_name };
        if name.is_null() {
            continue;
        }
        // SAFETY: the pattern compiled, and `name` is NUL-terminated.
        if matches!(query, ScriptQuery::Matching) && !unsafe { vim_regexec(regmatch, name, 0) } {
            continue;
        }

        // SAFETY: a fresh dict, handed to the list before anything else sees it.
        let d_held = tv_dict_alloc();
        let d = d_held.as_ptr();
        unsafe { (*l).push_dict(Some(d_held)) };
        unsafe { dict_add_str(d, c"name", name) };
        unsafe { dict_add_nr(d, c"sid", sid) };
        unsafe { dict_add_nr(d, c"version", 1) };
        // Vim9 autoload script (:h vim9-autoload), not applicable to Nvim.
        unsafe { dict_add_bool(d, c"autoload", kBoolVarFalse) };

        // A script ID was specified, so report that script in full.
        if let ScriptQuery::Sid(_) = *query {
            let sv_dict = unsafe { &raw mut (*(*si).sn_vars).sv_dict };
            let vars = unsafe { dict_copy(ptr::null(), sv_dict, true, get_copy_id()) };
            let (key, klen) = (c"variables".as_ptr(), c"variables".count_bytes());
            let _ = unsafe { (*d).add_dict(cstr::slice_at(key, klen), vars) };
            let funcs = get_script_local_funcs(sid as ScriptId);
            let (key, klen) = (c"functions".as_ptr(), c"functions".count_bytes());
            let _ = unsafe { (*d).add_list(cstr::slice_at(key, klen), Some(funcs)) };
        }
    }
}

/// An unprogrammed `RegMatch` carrying the current 'ignorecase'.
fn empty_regmatch() -> RegMatch {
    RegMatch {
        regprog: ptr::null_mut(),
        startp: [ptr::null_mut(); 10],
        endp: [ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: p_ic.get() != 0,
    }
}

/// `tv_dict_add_*` take the key and its length separately; upstream spells that
/// pair `S_LEN(key)`.
///
/// # Safety
///
/// `d` must point at a live dictionary, unaliased for the call. `val` must
/// point at a NUL-terminated string.
unsafe fn dict_add_str(d: *mut Dict, key: &CStr, val: *const c_char) {
    let _ = unsafe { (*d).add_str(key.to_bytes(), val) };
}

/// # Safety
///
/// `d` must point at a live dictionary, unaliased for the call.
unsafe fn dict_add_nr(d: *mut Dict, key: &CStr, nr: VarNumber) {
    let _ = unsafe { (*d).add_number(key.to_bytes(), nr) };
}

/// # Safety
///
/// `d` must point at a live dictionary, unaliased for the call.
unsafe fn dict_add_bool(d: *mut Dict, key: &CStr, val: BoolVarValue) {
    let _ = unsafe { (*d).add_bool(key.to_bytes(), val) };
}

// ---------------------------------------------------------------------------
// Reading a script.

/// Get one full line from a sourced file, for `do_cmdline()` under
/// `do_source()`.
///
/// Returns the line in allocated memory, or null at end of file or on error.
///
/// # Safety
///
/// `cookie` must be the payload this callback was registered with, live for
/// the call.
pub unsafe fn getsourceline(
    _c: c_int,
    cookie: *mut c_void,
    _indent: c_int,
    do_concat: bool,
) -> *mut c_char {
    let sp = cookie.cast::<SourceCookie>();
    // SAFETY: the cookie belongs to the script currently being sourced.
    let from_buf_or_str = unsafe { (*sp).source_from_buf_or_str };

    // If breakpoints have been added or deleted we need to look again.
    // SAFETY: as above.
    if unsafe { (*sp).dbg_tick } < debug_tick.get() && !from_buf_or_str {
        // SAFETY: as above.
        unsafe { refresh_breakpoint(sp) };
    }
    if do_profiling.get() == PROF_YES {
        script_line_end();
    }
    // Set the current sourcing line number.
    // SAFETY: as above.
    set_sourcing_lnum(unsafe { (*sp).sourcing_lnum } + 1);

    // SAFETY: as above.
    let mut line = unsafe { next_line(sp) };
    if !line.is_null() && do_profiling.get() == PROF_YES {
        script_line_start();
    }

    // Only concatenate lines starting with a `\` when 'cpoptions' does not
    // contain the 'C' flag.
    // SAFETY: `p_cpo` is the option's own string.
    if !line.is_null() && do_concat && !cpo_has(CpoFlag::CONCAT) {
        // SAFETY: as above.
        line = unsafe { concat_continuations(sp, line) };
    }

    // Convert the encoding of the script line.
    // SAFETY: as above; `string_convert` returns fresh memory or null.
    if !line.is_null() && unsafe { (*sp).conv.vc_type } != CONV_NONE {
        let converted = unsafe { string_convert(&raw mut (*sp).conv, line, ptr::null_mut()) };
        if !converted.is_null() {
            unsafe { xfree(line.cast::<c_void>()) };
            line = converted;
        }
    }

    // Did we encounter a breakpoint?
    // SAFETY: as above.
    let breakpoint = unsafe { (*sp).breakpoint };
    if !from_buf_or_str && breakpoint != 0 && breakpoint <= sourcing_lnum() {
        // SAFETY: as above; `fname` is the script's path.
        dbg_breakpoint(unsafe { (*sp).fname }, sourcing_lnum());
        unsafe { refresh_breakpoint(sp) };
    }

    line
}

/// Look up the next breakpoint in the script `source` is reading, and remember the
/// debugger's tick so we only look again when something changed.
///
/// # Safety
///
/// `source` must be a file-backed source cookie.
unsafe fn refresh_breakpoint(source: *mut SourceCookie) {
    unsafe { (*source).breakpoint = dbg_find_breakpoint(true, (*source).fname, sourcing_lnum()) };
    unsafe { (*source).dbg_tick = debug_tick.get() };
}

/// The next line of the script, using the one `getsourceline` read ahead if
/// there is one.  `source.fp` is null when the source is a string rather than a file.
///
/// # Safety
///
/// `source` must be the live source cookie.
unsafe fn next_line(source: *mut SourceCookie) -> *mut c_char {
    if unsafe { (*source).finished }
        || (!unsafe { (*source).source_from_buf_or_str } && unsafe { (*source).fp }.is_null())
    {
        return ptr::null_mut();
    }
    if unsafe { (*source).nextline }.is_null() {
        return unsafe { get_one_sourceline(source) };
    }
    let line = unsafe { (*source).nextline };
    unsafe { (*source).nextline = ptr::null_mut() };
    unsafe { (*source).sourcing_lnum += 1 };
    line
}

/// Join the `\`-continuation lines that follow `line` onto it.
///
/// We always have to read the next line to find out, so it is kept in
/// `source.nextline`.  A comment between continuation lines (`"\ `) counts as one.
///
/// # Safety
///
/// `source` must be the live source cookie and `line` its freshly read line.
unsafe fn concat_continuations(source: *mut SourceCookie, line: *mut c_char) -> *mut c_char {
    // Compensate for the one line read-ahead.
    unsafe { (*source).sourcing_lnum -= 1 };
    unsafe { (*source).nextline = get_one_sourceline(source) };
    if unsafe { (*source).nextline }.is_null()
        || !unsafe { starts_continuation(skipwhite((*source).nextline)) }
    {
        return line;
    }

    // SAFETY (this body): `line` and every `nextline` are NUL-terminated
    // lines this function owns.
    let mut joined = unsafe { cstr::bytes_at(line) }.to_vec();
    while !unsafe { (*source).nextline }.is_null()
        && unsafe {
            let next = cstr::bytes_at((*source).nextline);
            concat_continued_line(&mut joined, (*source).nextline, next.len())
        }
    {
        unsafe { xfree((*source).nextline.cast::<c_void>()) };
        unsafe { (*source).nextline = get_one_sourceline(source) };
    }
    unsafe { xfree(line.cast::<c_void>()) };
    owned_cstr(joined)
}

/// Does `p` begin a continuation -- a `\`, or the `"\ ` that comments one out?
///
/// # Safety
///
/// `p` must point into a NUL-terminated line.
unsafe fn starts_continuation(p: *const c_char) -> bool {
    unsafe {
        *p == b'\\' as c_char
            || (*p == b'"' as c_char && *p.add(1) == b'\\' as c_char && *p.add(2) == b' ' as c_char)
    }
}

/// Read one line of the script into fresh memory, or null at end of file.
///
/// A line can span several reads: the buffer may be too short for it, and a
/// newline escaped with an odd run of CTRL-V's does not end it.
///
/// # Safety
///
/// `source` must be the live source cookie.
unsafe fn get_one_sourceline(source: *mut SourceCookie) -> *mut c_char {
    let mut line: Vec<u8> = Vec::new();
    // SAFETY: `source` is the caller's cookie, throughout.
    unsafe { (*source).sourcing_lnum += 1 };

    // Loop until there is a finished line (or end-of-file).
    let mut have_read = false;
    loop {
        if unsafe { (*source).source_from_buf_or_str } {
            if !unsafe { next_buffered_line(source, &mut line) } {
                break;
            }
            // A buffer or string line is handed over whole: it cannot hold a
            // newline, so neither the continuation test nor the escaped-NL
            // test below can fire for one.
            have_read = true;
        } else {
            let Some(filled_the_chunk) = (unsafe { read_file_chunk(source, &mut line) }) else {
                break;
            };
            have_read = true;
            // If the line was longer than the chunk, read more.
            if filled_the_chunk && line.last() != Some(&b'\n') {
                continue;
            }
            if line.last() == Some(&b'\n') {
                if escaped_newline(&line) {
                    unsafe { (*source).sourcing_lnum += 1 };
                    continue;
                }
                // Remove the NL.
                line.pop();
            }
        }

        // Check for CTRL-C here now and then, so a recursive `:so` can be
        // broken out of.
        line_breakcheck();
        break;
    }

    if have_read {
        return owned_cstr(line);
    }
    ptr::null_mut()
}

/// Append the next line of the buffer or string being sourced. Answers
/// whether there was one.
///
/// # Safety
///
/// `source` must be a buffer- or string-backed source cookie.
unsafe fn next_buffered_line(source: *mut SourceCookie, line: &mut Vec<u8>) -> bool {
    // SAFETY: the caller's cookie, whose lines outlive the copy.
    let Some(next) = unsafe { &(*source).buflines }.get(unsafe { (*source).buf_lnum } as usize)
    else {
        return false;
    };
    line.extend_from_slice(next.to_bytes());
    unsafe { (*source).buf_lnum += 1 };
    true
}

/// `fgets` one chunk onto the end of `line`, retrying when a signal
/// interrupts. Answers whether the read used every byte it was offered --
/// which is how a line longer than one chunk announces itself -- or `None`
/// at end of file.
///
/// # Safety
///
/// `source` must be a file-backed source cookie.
unsafe fn read_file_chunk(source: *mut SourceCookie, line: &mut Vec<u8>) -> Option<bool> {
    /// How much one `fgets` reads at a time, terminator included: upstream
    /// grew its buffer by at least this much before every read. Function-
    /// local so it stays out of the unit lane's generated cdefs, which
    /// publish every module-level `const`.
    const SOURCE_CHUNK: usize = 120;

    let filled = line.len();
    line.reserve(SOURCE_CHUNK);
    loop {
        // SAFETY: the reserve above put `SOURCE_CHUNK` writable bytes at
        // `filled`, which is exactly what `fgets` is allowed to touch (it
        // writes at most one fewer, plus the terminator). `source`'s file is the
        // caller's promise.
        let rest = unsafe {
            *__errno_location() = 0;
            let dst = line.as_mut_ptr().add(filled).cast::<c_char>();
            if fgets(dst, SOURCE_CHUNK as c_int, (*source).fp).is_null() {
                if *__errno_location() != EINTR {
                    return None;
                }
                continue;
            }
            cstr::bytes_at(dst).len()
        };
        // SAFETY: `fgets` wrote `rest` bytes plus a terminator, all inside
        // the reserved room.
        unsafe { line.set_len(filled + rest) };
        return Some(rest == SOURCE_CHUNK - 1);
    }
}

/// Is the newline at the end of `line` escaped?
///
/// It is when an odd number of CTRL-V's precede it.  Upstream compares the
/// parity of the length against the parity of the index just before that run,
/// which is faster than counting the run and says the same thing.
fn escaped_newline(line: &[u8]) -> bool {
    let len = line.len() as isize;
    let mut c = len - 2;
    while c >= 0 && line[c as usize] as c_int == Ctrl_V {
        c -= 1;
    }
    (len & 1) != (c & 1)
}

// ---------------------------------------------------------------------------
// Leaving a script.

/// Are we sourcing a script, from a file or a buffer or a string?
pub fn sourcing_a_script(excmd: &mut ExArg) -> c_int {
    // SAFETY: `excmd` is the running command's block.
    let same = unsafe {
        getline_equal(
            excmd.ea_getline,
            excmd.cookie,
            Some(getsourceline as LineGetterFn),
        )
    };
    same as c_int
}

/// `":scriptencoding"`: set encoding conversion for a sourced script.
pub fn ex_scriptencoding(excmd: &mut ExArg) {
    // SAFETY: `excmd` is the running command's block.
    if sourcing_a_script(excmd) == 0 {
        emsg(gettext(
            c"E167: :scriptencoding used outside of a sourced file",
        ));
        return;
    }
    let name = if unsafe { *excmd.arg } != NUL as c_char {
        unsafe { enc_canonize(excmd.arg) }
    } else {
        excmd.arg
    };
    // Set up for conversion from the specified encoding to 'encoding'.
    let sp = unsafe { getline_cookie(excmd.ea_getline, excmd.cookie) }.cast::<SourceCookie>();
    let _ = unsafe { convert_setup(&raw mut (*sp).conv, name, p_enc.get()) };
    if name != excmd.arg {
        unsafe { xfree(name.cast::<c_void>()) };
    }
}

/// `":finish"`: mark a sourced file as finished.
pub fn ex_finish(excmd: &mut ExArg) {
    // SAFETY: `excmd` is the running command's block.
    if sourcing_a_script(excmd) != 0 {
        do_finish(excmd, false);
    } else {
        emsg(gettext(c"E168: :finish used outside of a sourced file"));
    }
}

/// Mark a sourced file as finished, possibly making the `":finish"` pending.
///
/// Also called for a pending finish at the `":endtry"` or after returning from
/// an extra `do_cmdline()`; `reanimate` says which.
pub fn do_finish(excmd: &mut ExArg, reanimate: bool) {
    // SAFETY: `excmd` is the running command's block, and its cookie is a
    // `SourceCookie` because `ex_finish` checked before calling.
    if reanimate {
        unsafe { (*source_cookie(excmd)).finished = false };
    }
    // Clean up (and deactivate) conditionals, but stop when a try
    // conditional not in its finally clause -- which then is to be executed
    // next -- is found.  In that case make the `":finish"` pending for
    // execution at the `":endtry"`.  Otherwise, finish normally.
    let idx = unsafe { cleanup_conditionals(excmd.cstack, CsFlags::NONE, true) };
    if idx >= 0 {
        unsafe { (*excmd.cstack).cs_pending[idx as usize] = CSTP_FINISH as c_char };
        unsafe { report_make_pending(CSTP_FINISH, NULL_0) };
    } else {
        unsafe { (*source_cookie(excmd)).finished = true };
    }
}

/// The cookie of the script `excmd` is running under.
///
/// # Safety
///
/// `excmd`'s reader must be [`getsourceline`].
unsafe fn source_cookie(excmd: &mut ExArg) -> *mut SourceCookie {
    unsafe { getline_cookie(excmd.ea_getline, excmd.cookie).cast::<SourceCookie>() }
}

/// Did a sourced file have the `":finish"` command?  If so, don't give an error
/// message for a missing `":endif"`.  False when not sourcing a file.
///
/// # Safety
///
/// `fgetline` must be an initialized `LineGetter` whose pointer fields point
/// at live data for the call. `cookie` must be the payload `fgetline` was
/// registered with, live for the call.
pub unsafe fn source_finished(fgetline: LineGetter, cookie: *mut c_void) -> bool {
    // SAFETY: `getline_equal` reads the reader's own bookkeeping; the cookie is
    // only dereferenced once that says it is a sourced script's.
    let sourced = unsafe { getline_equal(fgetline, cookie, Some(getsourceline as LineGetterFn)) };
    sourced && unsafe { (*getline_cookie(fgetline, cookie).cast::<SourceCookie>()).finished }
}

// ---------------------------------------------------------------------------
// Autoload.

/// The autoload script name for a function or variable name: `#` becomes `/`,
/// everything after the last `#` is dropped, and `.vim` takes its place.
///
/// `foo#bar#baz` becomes `autoload/foo/bar.vim`.  The caller must make sure
/// `name` contains `AUTOLOAD_CHAR`; the result is `xmalloc`ed.
///
/// # Safety
///
/// `name` must point at a NUL-terminated string.
pub unsafe fn autoload_name(name: *const c_char, name_len: size_t) -> *mut c_char {
    // SAFETY: the caller's `name` is `name_len` readable bytes.
    let name = unsafe { slice::from_raw_parts(name.cast::<u8>(), name_len) };
    let mut out = Vec::with_capacity(AUTOLOAD_PREFIX.len() + name_len + AUTOLOAD_SUFFIX.len());
    out.extend_from_slice(AUTOLOAD_PREFIX);
    out.extend_from_slice(name);

    // Everything from the last separator on is the member name, which the
    // suffix replaces; the separators before it are directories.
    let cut = out
        .iter()
        .rposition(|&b| b == AUTOLOAD_CHAR as u8)
        .unwrap_or(0);
    out.truncate(cut);
    for byte in &mut out {
        if *byte == AUTOLOAD_CHAR as u8 {
            *byte = b'/';
        }
    }
    out.extend_from_slice(AUTOLOAD_SUFFIX);

    // The callers free this with `xfree`, so it has to come from `xmalloc`.
    // SAFETY: `xmalloc` returns `out.len()` writable bytes or does not return.
    let scriptname = unsafe { xmalloc(out.len()) }.cast::<u8>();
    unsafe { ptr::copy_nonoverlapping(out.as_ptr(), scriptname, out.len()) };
    scriptname.cast::<c_char>()
}

/// If `name` has a package name, try autoloading the script for it.
///
/// Returns true if a package was loaded.  `reload` loads the script again even
/// when it is already known.
///
/// # Safety
///
/// `name` must point at a NUL-terminated string.
pub unsafe fn script_autoload(name: *const c_char, name_len: size_t, reload: bool) -> bool {
    // SAFETY: the caller's `name` is `name_len` readable bytes.
    let bytes = unsafe { slice::from_raw_parts(name.cast::<u8>(), name_len) };
    // If there is no `#` after name[0] there is no package name.
    if !matches!(
        bytes.iter().position(|&b| b == AUTOLOAD_CHAR as u8),
        Some(1..)
    ) {
        return false;
    }

    // SAFETY: `name` is as the caller described it.
    let scriptname = unsafe { autoload_name(name, name_len) };
    // SAFETY: `scriptname` is the freshly built path.
    let path = unsafe { CStr::from_ptr(scriptname) }.to_bytes();
    let known = ga_loaded.with(|loaded| loaded.iter().any(|entry| same_autoload(entry, path)));

    // Was it loaded already?
    if !reload && known {
        // SAFETY: ours, and nothing took ownership of it.
        unsafe { xfree(scriptname.cast::<c_void>()) };
        return false;
    }

    // Remember the name if it wasn't loaded already.
    if !known {
        ga_loaded.with_mut(|loaded| loaded.push(path.to_vec()));
    }

    // Try loading the package from `$VIMRUNTIME/autoload/<name>.vim`.  The
    // `ret_sid` cookie keeps `source_callback` from loading the same script
    // twice.
    let mut ret_sid: c_int = 0;
    // SAFETY: `scriptname` outlives the search, and `source_callback` takes
    // its cookie as the `int *` we pass.
    let ret = unsafe {
        do_in_runtimepath(
            scriptname,
            RuntimeOpts::START,
            Some(source_callback as DoInRuntimepathCBFn),
            (&raw mut ret_sid).cast::<c_void>(),
        )
        .is_ok()
    };

    // SAFETY: still ours -- `ga_loaded` keeps its own copy of the path.
    unsafe { xfree(scriptname.cast::<c_void>()) };
    ret
}

/// Are these the same autoload path? The shared `autoload/` prefix is skipped
/// -- every entry carries it, so comparing it again buys nothing.
fn same_autoload(entry: &[u8], path: &[u8]) -> bool {
    let skip = AUTOLOAD_PREFIX.len();
    entry.get(skip..) == path.get(skip..)
}
