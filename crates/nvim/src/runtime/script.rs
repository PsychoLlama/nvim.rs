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

use super::*;

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{ptr, slice};

/// Offset of `uf_name` inside `ufunc_T`: the function table's hash keys point
/// at that inline buffer, so backing up by this recovers the function.  This is
/// the transpiled `HI2UF`, the same constant profile.rs and userfunc.rs use.
const UF_NAME_OFFSET: usize = 240;

/// Bytes `autoload_name` puts in front of the name it is given.
const AUTOLOAD_PREFIX: &[u8] = b"autoload/";
/// ...and what it puts after the last package separator, NUL included.
const AUTOLOAD_SUFFIX: &[u8] = b".vim\0";

// ---------------------------------------------------------------------------
// The registry.

/// Every script the editor has sourced, script 1 first.
///
/// # Safety
///
/// The slice borrows `script_items`' buffer, which registering a new script
/// reallocates.  No caller may hold it across a `:source`.
pub(crate) unsafe fn scripts<'a>() -> &'a [*mut scriptitem_T] {
    let ga = script_items.get();
    if ga.ga_data.is_null() {
        return &[];
    }
    // SAFETY: `script_items` is a garray of `scriptitem_T *` whose first
    // `ga_len` elements are live, and the caller promises not to hold the
    // borrow across a registration.
    unsafe { slice::from_raw_parts(ga.ga_data.cast::<*mut scriptitem_T>(), ga.ga_len as usize) }
}

/// The registry entry for script `sid` -- upstream's `SCRIPT_ITEM`.
///
/// # Safety
///
/// `sid` must satisfy [`script_id_valid`].
pub(crate) unsafe fn script_item(sid: scid_T) -> *mut scriptitem_T {
    debug_assert!(script_id_valid(sid), "script id out of range");
    // SAFETY: the caller vouched for `sid`, and script IDs are 1-based.
    unsafe {
        *script_items
            .get()
            .ga_data
            .cast::<*mut scriptitem_T>()
            .offset(sid as isize - 1)
    }
}

/// Is `sid` a script the editor has sourced -- upstream's `SCRIPT_ID_VALID`?
pub(crate) fn script_id_valid(sid: c_int) -> bool {
    sid > 0 && sid <= script_items.get().ga_len
}

/// Was script `sid` written in Lua?
pub unsafe extern "C" fn script_is_lua(sid: scid_T) -> bool {
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
pub unsafe extern "C" fn find_script_by_name(name: *mut c_char) -> c_int {
    // SAFETY: nothing below sources a script, so the borrow stays valid.
    for (idx, &si) in unsafe { scripts() }.iter().enumerate().rev() {
        // SAFETY: a registry slot always holds a live `scriptitem_T`, and
        // `path_fnamecmp` only reads the two NUL-terminated names.
        if unsafe { !(*si).sn_name.is_null() && path_fnamecmp((*si).sn_name, name) == 0 } {
            return idx as c_int + 1;
        }
    }
    -1
}

// ---------------------------------------------------------------------------
// `:scriptnames`.

/// `":scriptnames"`, and `":script {id}"` which edits the script instead.
pub unsafe fn ex_scriptnames(eap: *mut exarg_T) {
    // SAFETY: `eap` is the command's own argument block.
    let (by_number, has_arg) = unsafe { ((*eap).addr_count > 0, *(*eap).arg != NUL as c_char) };
    if by_number || has_arg {
        // SAFETY: same block; `edit_script` only reads it and `do_exedit`.
        unsafe { edit_script(eap, by_number) };
        return;
    }

    // SAFETY: `msg_ext_set_kind` copies the literal.
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    let mut sid: scid_T = 1;
    while sid <= script_items.get().ga_len && !got_int.get() {
        // SAFETY: `sid` is in range, and the registry is re-read every round
        // because the output below can pause for the user.
        let name = unsafe { (*script_item(sid)).sn_name };
        let listed = sid;
        sid += 1;
        if name.is_null() {
            continue;
        }
        // SAFETY: `NameBuff` and `IObuff` are the shared scratch buffers, both
        // sized as the calls below are told.
        unsafe {
            let namebuff = NameBuff.ptr().cast::<c_char>();
            home_replace(ptr::null(), name, namebuff, MAXPATHL as size_t, true);
            let iobuff = IObuff.ptr().cast::<c_char>();
            vim_snprintf(
                iobuff,
                IOSIZE as size_t,
                c"%3d: %s".as_ptr(),
                listed,
                namebuff,
            );
            if !message_filtered(iobuff) {
                if msg_col.get() > 0 {
                    msg_putchar('\n' as c_int);
                }
                msg_outtrans(iobuff, 0, false);
                line_breakcheck();
            }
        }
    }
}

/// `":script {id}"` / `":script {file}"`: open the named script in a window.
///
/// # Safety
///
/// `eap` must be the live `:script` command block.
unsafe fn edit_script(eap: *mut exarg_T, by_number: bool) {
    unsafe {
        if by_number {
            if !script_id_valid((*eap).line2 as c_int) {
                emsg(gettext(&raw const e_invarg as *const c_char));
                return;
            }
            (*eap).arg = (*script_item((*eap).line2 as scid_T)).sn_name;
        } else {
            let namebuff = NameBuff.ptr().cast::<c_char>();
            expand_env((*eap).arg, namebuff, MAXPATHL);
            (*eap).arg = namebuff;
        }
        do_exedit(eap, ptr::null_mut());
    }
}

/// A pointer to a script's name, for `":verbose set"` -- the text appended to
/// "Last set from ".
///
/// The negative script IDs are the contexts that have no file: a modeline, the
/// `--cmd` or `-c` command line, and so on.  When `should_free` is non-null and
/// the name really is a file path, the result is a `home_replace_save()` copy
/// and `*should_free` is set.
pub unsafe extern "C" fn get_scriptname(script_ctx: sctx_T, should_free: *mut bool) -> *mut c_char {
    // SAFETY: an out-parameter the caller owns, when it passed one at all.
    unsafe {
        if !should_free.is_null() {
            *should_free = false;
        }
    }

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
            // SAFETY: `IObuff` is the shared scratch buffer, `IOSIZE` long.
            return unsafe {
                let iobuff = IObuff.ptr().cast::<c_char>();
                snprintf(
                    iobuff,
                    IOSIZE as size_t,
                    gettext(c"API client (channel id %lu)".as_ptr()),
                    script_ctx.sc_chan,
                );
                iobuff
            };
        }
        _ => {
            // SAFETY: every other `sc_sid` is a registry index.
            let sname = unsafe { (*script_item(script_ctx.sc_sid)).sn_name };
            if sname.is_null() {
                // SAFETY: as above.
                return unsafe {
                    let iobuff = IObuff.ptr().cast::<c_char>();
                    snprintf(
                        iobuff,
                        IOSIZE as size_t,
                        gettext(c"anonymous :source (script id %d)".as_ptr()),
                        script_ctx.sc_sid,
                    );
                    iobuff
                };
            }
            if should_free.is_null() {
                return sname;
            }
            // SAFETY: the caller asked for an owned copy and said so.
            return unsafe {
                *should_free = true;
                home_replace_save(ptr::null_mut(), sname)
            };
        }
    };
    // SAFETY: `gettext` returns a pointer into its own catalogue.
    unsafe { gettext(fixed.as_ptr()) }
}

/// The line number to report for a message raised under `fgetline`.
///
/// A sourced script tracks its own read position, because the execution stack's
/// number lags behind by the one line `getsourceline` reads ahead.
pub unsafe extern "C" fn get_sourced_lnum(fgetline: LineGetter, cookie: *mut c_void) -> linenr_T {
    if !getline_is_source(fgetline) {
        return sourcing_lnum();
    }
    // SAFETY: a `getsourceline` reader always carries a `source_cookie_T`.
    unsafe { (*cookie.cast::<source_cookie_T>()).sourcing_lnum }
}

/// Is `fgetline` the reader [`getsourceline`] installs?
fn getline_is_source(fgetline: LineGetter) -> bool {
    fgetline.is_some_and(|f| ptr::fn_addr_eq(f, getsourceline as LineGetterFn))
}

// ---------------------------------------------------------------------------
// `getscriptinfo()`.

/// The script-local functions defined in the script with id `sid`, as a list of
/// their names.
///
/// # Safety
///
/// The global function table must be walkable, which it is outside a rehash.
unsafe fn get_script_local_funcs(sid: scid_T) -> *mut list_T {
    // SAFETY: the process-wide function table, which outlives this walk.
    let functbl = unsafe { &*func_tbl_get() };
    // SAFETY: a fresh list, at most one entry per function.
    let l = unsafe { tv_list_alloc(functbl.ht_used as ptrdiff_t) };

    for hi in tv_ht_iter(functbl) {
        // SAFETY: an occupied slot's key is a `ufunc_T`'s inline name buffer,
        // so backing up by that field's offset recovers the function.
        let fp = unsafe { &*(*hi).hi_key.byte_sub(UF_NAME_OFFSET).cast::<ufunc_T>() };
        if fp.uf_script_ctx.sc_sid != sid {
            continue;
        }
        let name = if fp.uf_name_exp.is_null() {
            (&raw const fp.uf_name).cast::<c_char>()
        } else {
            fp.uf_name_exp
        };
        // SAFETY: `name` is NUL-terminated, which the -1 length asks for.
        unsafe { tv_list_append_string(l, name, -1) };
    }
    l
}

/// Which scripts `getscriptinfo()` was asked about.
enum ScriptQuery {
    /// No argument, or one that named neither key: every script, names only.
    All,
    /// `{'sid': n}`: that one script, with its variables and functions too.
    Sid(varnumber_T),
    /// `{'name': pat}`: the scripts whose path matches the caller's `regmatch`.
    Matching,
    /// The argument was rejected; an error is already pending.
    Rejected,
}

/// `"getscriptinfo()"` function
pub unsafe extern "C" fn f_getscriptinfo(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: `rettv` is the caller's return slot, `argvars` its arguments.
    unsafe {
        tv_list_alloc_ret(rettv, script_items.get().ga_len as ptrdiff_t);
        if tv_check_for_opt_dict_arg(argvars, 0) == FAIL {
            return;
        }
    }
    // The pattern's source string is freed on the way out, as upstream does,
    // even when it did not compile.
    let mut pat: *mut c_char = ptr::null_mut();
    // One `regmatch_T` for the whole run, not one per script: `vim_regexec`
    // may swap the compiled program out from under it when the NFA engine
    // gives up and the backtracker recompiles the pattern.
    let mut regmatch = empty_regmatch();
    // SAFETY: as above.
    let query = unsafe { script_query(argvars, &mut pat, &mut regmatch) };

    if !matches!(query, ScriptQuery::Rejected) {
        // SAFETY: `rettv` holds the list allocated above.
        let l = unsafe { (*rettv).vval.v_list };
        // SAFETY: nothing in the loop sources a script.
        unsafe { report_scripts(l, &query, &mut regmatch) };
    }

    // SAFETY: both were allocated by the call that produced `query`; either
    // may be null, which both frees accept.
    unsafe {
        vim_regfree(regmatch.regprog);
        xfree(pat.cast::<c_void>());
    }
}

/// Read `getscriptinfo()`'s optional dict argument.
///
/// `pat` receives the raw `name` string so the caller can free it whatever
/// happens, and `regmatch` its compiled form; a pattern that fails to compile
/// is not an error, it just filters nothing.
///
/// # Safety
///
/// `argvars` must be the builtin's argument vector.
unsafe fn script_query(
    argvars: *mut typval_T,
    pat: *mut *mut c_char,
    regmatch: &mut regmatch_T,
) -> ScriptQuery {
    // SAFETY: the caller's argument vector; argument 0 always exists.
    let arg = unsafe { &*argvars };
    if arg.v_type != VAR_DICT {
        return ScriptQuery::All;
    }
    // SAFETY: a `VAR_DICT` argument carries a live dict.
    let dict = unsafe { arg.vval.v_dict };

    // SAFETY: `tv_dict_find` only reads the dict and the key literal.
    let sid_di = unsafe { tv_dict_find(dict, c"sid".as_ptr(), c"sid".count_bytes() as ptrdiff_t) };
    if !sid_di.is_null() {
        let mut error = false;
        // SAFETY: `sid_di` is a live item of `dict`.
        let sid = unsafe { tv_get_number_chk(&raw mut (*sid_di).di_tv, &raw mut error) };
        if error {
            return ScriptQuery::Rejected;
        }
        if sid <= 0 {
            // SAFETY: as above; the message borrows the item's string form.
            unsafe {
                semsg_c!(
                    gettext(&raw const e_invargNval as *const c_char),
                    c"sid".as_ptr(),
                    tv_get_string(&raw mut (*sid_di).di_tv),
                );
            }
            return ScriptQuery::Rejected;
        }
        return ScriptQuery::Sid(sid);
    }

    // SAFETY: the string is allocated for us and handed straight to the caller.
    unsafe {
        *pat = tv_dict_get_string(dict, c"name".as_ptr(), true);
        if !(*pat).is_null() {
            regmatch.regprog = vim_regcomp(*pat, RE_MAGIC + RE_STRING);
        }
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
unsafe fn report_scripts(l: *mut list_T, query: &ScriptQuery, regmatch: &mut regmatch_T) {
    let total = script_items.get().ga_len as varnumber_T;
    // A `sid` query asks about exactly one script, and answers nothing at all
    // when that script does not exist.
    let (first, last) = match *query {
        ScriptQuery::Sid(sid) => (sid, sid.min(total)),
        _ => (1, total),
    };

    for sid in first..=last {
        // SAFETY: `sid` is in range, and nothing in the body sources a script.
        let si = unsafe { script_item(sid as scid_T) };
        // SAFETY: a registry slot always holds a live `scriptitem_T`.
        let name = unsafe { (*si).sn_name };
        if name.is_null() {
            continue;
        }
        // SAFETY: the pattern compiled, and `name` is NUL-terminated.
        if matches!(query, ScriptQuery::Matching) && !unsafe { vim_regexec(regmatch, name, 0) } {
            continue;
        }

        // SAFETY: a fresh dict, handed to the list before anything else sees it.
        unsafe {
            let d = tv_dict_alloc();
            tv_list_append_dict(l, d);
            dict_add_str(d, c"name", name);
            dict_add_nr(d, c"sid", sid);
            dict_add_nr(d, c"version", 1);
            // Vim9 autoload script (:h vim9-autoload), not applicable to Nvim.
            dict_add_bool(d, c"autoload", kBoolVarFalse);

            // A script ID was specified, so report that script in full.
            if let ScriptQuery::Sid(_) = *query {
                let sv_dict = &raw mut (*(*si).sn_vars).sv_dict;
                let vars = tv_dict_copy(ptr::null(), sv_dict, true, get_copyID());
                tv_dict_add_dict(d, c"variables".as_ptr(), c"variables".count_bytes(), vars);
                let funcs = get_script_local_funcs(sid as scid_T);
                tv_dict_add_list(d, c"functions".as_ptr(), c"functions".count_bytes(), funcs);
            }
        }
    }
}

/// An unprogrammed `regmatch_T` carrying the current 'ignorecase'.
fn empty_regmatch() -> regmatch_T {
    regmatch_T {
        regprog: ptr::null_mut(),
        startp: [ptr::null_mut(); 10],
        endp: [ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: p_ic.get() != 0,
    }
}

/// `tv_dict_add_*` take the key and its length separately; upstream spells that
/// pair `S_LEN(key)`.
unsafe fn dict_add_str(d: *mut dict_T, key: &CStr, val: *const c_char) {
    unsafe { tv_dict_add_str(d, key.as_ptr(), key.count_bytes(), val) };
}

unsafe fn dict_add_nr(d: *mut dict_T, key: &CStr, nr: varnumber_T) {
    unsafe { tv_dict_add_nr(d, key.as_ptr(), key.count_bytes(), nr) };
}

unsafe fn dict_add_bool(d: *mut dict_T, key: &CStr, val: BoolVarValue) {
    unsafe { tv_dict_add_bool(d, key.as_ptr(), key.count_bytes(), val) };
}

// ---------------------------------------------------------------------------
// Reading a script.

/// Get one full line from a sourced file, for `do_cmdline()` under
/// `do_source()`.
///
/// Returns the line in allocated memory, or null at end of file or on error.
pub unsafe extern "C" fn getsourceline(
    _c: c_int,
    cookie: *mut c_void,
    _indent: c_int,
    do_concat: bool,
) -> *mut c_char {
    let sp = cookie.cast::<source_cookie_T>();
    // SAFETY: the cookie belongs to the script currently being sourced.
    let from_buf_or_str = unsafe { (*sp).source_from_buf_or_str };

    // If breakpoints have been added or deleted we need to look again.
    // SAFETY: as above.
    if unsafe { (*sp).dbg_tick } < debug_tick.get() && !from_buf_or_str {
        // SAFETY: as above.
        unsafe { refresh_breakpoint(sp) };
    }
    if do_profiling.get() == PROF_YES {
        // SAFETY: paired with the `script_line_start` below.
        unsafe { script_line_end() };
    }
    // Set the current sourcing line number.
    // SAFETY: as above.
    set_sourcing_lnum(unsafe { (*sp).sourcing_lnum } + 1);

    // SAFETY: as above.
    let mut line = unsafe { next_line(sp) };
    if !line.is_null() && do_profiling.get() == PROF_YES {
        // SAFETY: paired with the `script_line_end` above.
        unsafe { script_line_start() };
    }

    // Only concatenate lines starting with a `\` when 'cpoptions' does not
    // contain the 'C' flag.
    // SAFETY: `p_cpo` is the option's own string.
    if !line.is_null() && do_concat && unsafe { vim_strchr(p_cpo.get(), CPO_CONCAT) }.is_null() {
        // SAFETY: as above.
        line = unsafe { concat_continuations(sp, line) };
    }

    // Convert the encoding of the script line.
    // SAFETY: as above; `string_convert` returns fresh memory or null.
    if !line.is_null() && unsafe { (*sp).conv }.vc_type != CONV_NONE {
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
        unsafe {
            dbg_breakpoint((*sp).fname, sourcing_lnum());
            refresh_breakpoint(sp);
        }
    }

    line
}

/// Look up the next breakpoint in the script `sp` is reading, and remember the
/// debugger's tick so we only look again when something changed.
///
/// # Safety
///
/// `sp` must be a file-backed source cookie.
unsafe fn refresh_breakpoint(sp: *mut source_cookie_T) {
    unsafe {
        (*sp).breakpoint = dbg_find_breakpoint(true, (*sp).fname, sourcing_lnum());
        (*sp).dbg_tick = debug_tick.get();
    }
}

/// The next line of the script, using the one `getsourceline` read ahead if
/// there is one.  `fp` is null when the source is a string rather than a file.
///
/// # Safety
///
/// `sp` must be the live source cookie.
unsafe fn next_line(sp: *mut source_cookie_T) -> *mut c_char {
    unsafe {
        if (*sp).finished || (!(*sp).source_from_buf_or_str && (*sp).fp.is_null()) {
            return ptr::null_mut();
        }
        if (*sp).nextline.is_null() {
            return get_one_sourceline(sp);
        }
        let line = (*sp).nextline;
        (*sp).nextline = ptr::null_mut();
        (*sp).sourcing_lnum += 1;
        line
    }
}

/// Join the `\`-continuation lines that follow `line` onto it.
///
/// We always have to read the next line to find out, so it is kept in
/// `sp->nextline`.  A comment between continuation lines (`"\ `) counts as one.
///
/// # Safety
///
/// `sp` must be the live source cookie and `line` its freshly read line.
unsafe fn concat_continuations(sp: *mut source_cookie_T, line: *mut c_char) -> *mut c_char {
    unsafe {
        // Compensate for the one line read-ahead.
        (*sp).sourcing_lnum -= 1;
        (*sp).nextline = get_one_sourceline(sp);
        if (*sp).nextline.is_null() || !starts_continuation(skipwhite((*sp).nextline)) {
            return line;
        }

        let mut ga = GA_EMPTY_INIT_VALUE;
        ga_init(&raw mut ga, size_of::<c_char>() as c_int, 400);
        ga_concat(&raw mut ga, line);
        while !(*sp).nextline.is_null()
            && concat_continued_line(&raw mut ga, 400, (*sp).nextline, strlen((*sp).nextline))
        {
            xfree((*sp).nextline.cast::<c_void>());
            (*sp).nextline = get_one_sourceline(sp);
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        xfree(line.cast::<c_void>());
        ga.ga_data.cast::<c_char>()
    }
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
/// `sp` must be the live source cookie.
unsafe fn get_one_sourceline(sp: *mut source_cookie_T) -> *mut c_char {
    // Use a growarray to store the sourced line.
    let mut ga = GA_EMPTY_INIT_VALUE;
    // SAFETY: `ga` is a local garray, and `sp` is the caller's cookie.
    unsafe {
        ga_init(&raw mut ga, 1, 250);
        (*sp).sourcing_lnum += 1;
    }

    // Loop until there is a finished line (or end-of-file).
    let mut have_read = false;
    loop {
        // Make room to read at least 120 (more) characters.
        // SAFETY: as above.
        let len = unsafe {
            ga_grow(&raw mut ga, 120);
            if (*sp).source_from_buf_or_str {
                match next_buffered_line(sp, &raw mut ga) {
                    Some(len) => len,
                    None => break,
                }
            } else {
                match read_file_chunk(sp, &raw mut ga) {
                    Some(len) => len,
                    None => break,
                }
            }
        };
        have_read = true;
        ga.ga_len = len;
        let buf = ga.ga_data.cast::<c_char>();

        // If the line was longer than the buffer, read more.
        // SAFETY: `len` bytes were just written into `buf`.
        if ga.ga_maxlen - ga.ga_len == 1 && unsafe { *buf.add(len as usize - 1) } != b'\n' as c_char
        {
            continue;
        }

        // SAFETY: as above.
        if len >= 1 && unsafe { *buf.add(len as usize - 1) } == b'\n' as c_char {
            // SAFETY: as above.
            if unsafe { escaped_newline(buf, len) } {
                // SAFETY: `sp` is the caller's cookie.
                unsafe { (*sp).sourcing_lnum += 1 };
                continue;
            }
            // Remove the NL.
            // SAFETY: as above.
            unsafe { *buf.add(len as usize - 1) = NUL as c_char };
        }

        // Check for CTRL-C here now and then, so a recursive `:so` can be
        // broken out of.
        // SAFETY: no arguments, main thread only.
        line_breakcheck();
        break;
    }

    if have_read {
        return ga.ga_data.cast::<c_char>();
    }
    // SAFETY: `ga` owns whatever it grew.
    unsafe { xfree(ga.ga_data) };
    ptr::null_mut()
}

/// Append the next line of the buffer or string being sourced, NUL included.
///
/// Returns the new length of `ga`, or `None` once every line is processed.
///
/// # Safety
///
/// `sp` must be a buffer- or string-backed source cookie, and `ga` the line
/// being built.
unsafe fn next_buffered_line(sp: *mut source_cookie_T, ga: *mut garray_T) -> Option<c_int> {
    unsafe {
        if (*sp).buf_lnum >= (*sp).buflines.ga_len {
            return None;
        }
        let lines = (*sp).buflines.ga_data.cast::<*mut c_char>();
        ga_concat(ga, *lines.add((*sp).buf_lnum as usize));
        (*sp).buf_lnum += 1;
        ga_grow(ga, 1);
        *(*ga).ga_data.cast::<c_char>().add((*ga).ga_len as usize) = NUL as c_char;
        (*ga).ga_len += 1;
        Some((*ga).ga_len)
    }
}

/// `fgets` one chunk onto the end of `ga`, retrying when a signal interrupts.
///
/// Returns the new length of `ga`, or `None` at end of file.
///
/// # Safety
///
/// `sp` must be a file-backed source cookie, and `ga` the line being built.
unsafe fn read_file_chunk(sp: *mut source_cookie_T, ga: *mut garray_T) -> Option<c_int> {
    unsafe {
        let filled = (*ga).ga_len;
        let buf = (*ga).ga_data.cast::<c_char>();
        loop {
            *__errno_location() = 0;
            if !fgets(buf.add(filled as usize), (*ga).ga_maxlen - filled, (*sp).fp).is_null() {
                return Some(filled + strlen(buf.add(filled as usize)) as c_int);
            }
            if *__errno_location() != EINTR {
                return None;
            }
        }
    }
}

/// Is the newline at the end of `buf[..len]` escaped?
///
/// It is when an odd number of CTRL-V's precede it.  Upstream compares the
/// parity of `len` against the parity of the index just before that run, which
/// is faster than counting the run and says the same thing.
///
/// # Safety
///
/// `buf` must hold at least `len` bytes.
unsafe fn escaped_newline(buf: *const c_char, len: c_int) -> bool {
    let mut c = len - 2;
    // SAFETY: `c` stays inside `buf[..len]`.
    while c >= 0 && unsafe { *buf.add(c as usize) } as c_int == Ctrl_V {
        c -= 1;
    }
    (len & 1) != (c & 1)
}

// ---------------------------------------------------------------------------
// Leaving a script.

/// Are we sourcing a script, from a file or a buffer or a string?
pub unsafe extern "C" fn sourcing_a_script(eap: *mut exarg_T) -> c_int {
    // SAFETY: `eap` is the running command's block.
    let same = unsafe {
        getline_equal(
            (*eap).ea_getline,
            (*eap).cookie,
            Some(getsourceline as LineGetterFn),
        )
    };
    same as c_int
}

/// `":scriptencoding"`: set encoding conversion for a sourced script.
pub unsafe fn ex_scriptencoding(eap: *mut exarg_T) {
    // SAFETY: `eap` is the running command's block.
    unsafe {
        if sourcing_a_script(eap) == 0 {
            emsg(gettext(
                c"E167: :scriptencoding used outside of a sourced file".as_ptr(),
            ));
            return;
        }
        let name = if *(*eap).arg != NUL as c_char {
            enc_canonize((*eap).arg)
        } else {
            (*eap).arg
        };
        // Set up for conversion from the specified encoding to 'encoding'.
        let sp = getline_cookie((*eap).ea_getline, (*eap).cookie).cast::<source_cookie_T>();
        convert_setup(&raw mut (*sp).conv, name, p_enc.get());
        if name != (*eap).arg {
            xfree(name.cast::<c_void>());
        }
    }
}

/// `":finish"`: mark a sourced file as finished.
pub unsafe fn ex_finish(eap: *mut exarg_T) {
    // SAFETY: `eap` is the running command's block.
    unsafe {
        if sourcing_a_script(eap) != 0 {
            do_finish(eap, false);
        } else {
            emsg(gettext(
                c"E168: :finish used outside of a sourced file".as_ptr(),
            ));
        }
    }
}

/// Mark a sourced file as finished, possibly making the `":finish"` pending.
///
/// Also called for a pending finish at the `":endtry"` or after returning from
/// an extra `do_cmdline()`; `reanimate` says which.
pub unsafe extern "C" fn do_finish(eap: *mut exarg_T, reanimate: bool) {
    // SAFETY: `eap` is the running command's block, and its cookie is a
    // `source_cookie_T` because `ex_finish` checked before calling.
    unsafe {
        if reanimate {
            (*source_cookie(eap)).finished = false;
        }
        // Clean up (and deactivate) conditionals, but stop when a try
        // conditional not in its finally clause -- which then is to be executed
        // next -- is found.  In that case make the `":finish"` pending for
        // execution at the `":endtry"`.  Otherwise, finish normally.
        let idx = cleanup_conditionals((*eap).cstack, 0, true);
        if idx >= 0 {
            (*(*eap).cstack).cs_pending[idx as usize] = CSTP_FINISH as c_char;
            report_make_pending(CSTP_FINISH, NULL_0);
        } else {
            (*source_cookie(eap)).finished = true;
        }
    }
}

/// The cookie of the script `eap` is running under.
///
/// # Safety
///
/// `eap`'s reader must be [`getsourceline`].
unsafe fn source_cookie(eap: *mut exarg_T) -> *mut source_cookie_T {
    unsafe { getline_cookie((*eap).ea_getline, (*eap).cookie).cast::<source_cookie_T>() }
}

/// Did a sourced file have the `":finish"` command?  If so, don't give an error
/// message for a missing `":endif"`.  False when not sourcing a file.
pub unsafe extern "C" fn source_finished(fgetline: LineGetter, cookie: *mut c_void) -> bool {
    // SAFETY: `getline_equal` reads the reader's own bookkeeping; the cookie is
    // only dereferenced once that says it is a sourced script's.
    unsafe {
        getline_equal(fgetline, cookie, Some(getsourceline as LineGetterFn))
            && (*getline_cookie(fgetline, cookie).cast::<source_cookie_T>()).finished
    }
}

// ---------------------------------------------------------------------------
// Autoload.

/// The autoload script name for a function or variable name: `#` becomes `/`,
/// everything after the last `#` is dropped, and `.vim` takes its place.
///
/// `foo#bar#baz` becomes `autoload/foo/bar.vim`.  The caller must make sure
/// `name` contains `AUTOLOAD_CHAR`; the result is `xmalloc`ed.
pub unsafe extern "C" fn autoload_name(name: *const c_char, name_len: size_t) -> *mut c_char {
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
    unsafe {
        let scriptname = xmalloc(out.len()).cast::<u8>();
        ptr::copy_nonoverlapping(out.as_ptr(), scriptname, out.len());
        scriptname.cast::<c_char>()
    }
}

/// If `name` has a package name, try autoloading the script for it.
///
/// Returns true if a package was loaded.  `reload` loads the script again even
/// when it is already known.
pub unsafe extern "C" fn script_autoload(
    name: *const c_char,
    name_len: size_t,
    reload: bool,
) -> bool {
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
    let known = unsafe { loaded_index(scriptname) };
    let loaded_len = ga_loaded.get().ga_len;

    // Was it loaded already?
    if !reload && known < loaded_len {
        // SAFETY: ours, and nothing took ownership of it.
        unsafe { xfree(scriptname.cast::<c_void>()) };
        return false;
    }

    // Remember the name if it wasn't loaded already; `ga_loaded` takes it over.
    let mut tofree = scriptname;
    if known == loaded_len {
        // SAFETY: `ga_loaded` is a garray of owned `char *`.
        unsafe { ga_append_ptr(ga_loaded.ptr(), scriptname) };
        tofree = ptr::null_mut();
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
            DIP_START as c_int,
            Some(source_callback as DoInRuntimepathCBFn),
            (&raw mut ret_sid).cast::<c_void>(),
        ) == OK
    };

    // SAFETY: null unless `ga_loaded` declined it, in which case it is ours.
    unsafe { xfree(tofree.cast::<c_void>()) };
    ret
}

/// Where `scriptname` sits in `ga_loaded`, or its length when it is not there.
///
/// The shared `autoload/` prefix is skipped -- it is the same on every entry.
///
/// # Safety
///
/// `scriptname` must be an autoload path, so at least as long as the prefix.
unsafe fn loaded_index(scriptname: *const c_char) -> c_int {
    let ga = ga_loaded.get();
    let skip = AUTOLOAD_PREFIX.len();
    for i in 0..ga.ga_len {
        // SAFETY: `ga_loaded` holds `ga_len` NUL-terminated paths, each with
        // the same prefix as `scriptname`.
        if unsafe {
            let entry = *ga.ga_data.cast::<*mut c_char>().add(i as usize);
            strcmp(entry.add(skip), scriptname.add(skip)) == 0
        } {
            return i;
        }
    }
    ga.ga_len
}

/// `GA_APPEND` for a garray of owned `char *`.
///
/// # Safety
///
/// `ga` must be a garray whose item type is `*mut c_char`.
unsafe fn ga_append_ptr(ga: *mut garray_T, value: *mut c_char) {
    unsafe {
        ga_grow(ga, 1);
        *(*ga)
            .ga_data
            .cast::<*mut c_char>()
            .add((*ga).ga_len as usize) = value;
        (*ga).ga_len += 1;
    }
}
