//! What the editor remembers about scripts it has sourced, and how a sourced
//! script is left.
//!
//! `SCRIPT_ITEMS` is the registry -- one entry per script ever sourced, with
//! its name, its script-local variables, its `<SID>` and its profiling
//! counters, reached by its [`ScriptId`] through [`with_script_item`] and the
//! readers beside it; [`ex_scriptnames`], [`f_getscriptinfo`],
//! [`get_scriptname`] and [`find_script_by_name`] are its users, and
//! [`script_autoload`] is the lookup that turns `foo#bar()` into an
//! `autoload/foo.vim` to source.
//!
//! [`ex_finish`] and [`do_finish`] are how a script stops early; the reader a
//! running script is pulled through is [`super::reader`]'s.

#![forbid(unsafe_code)]

use super::*;
use crate::cstr;
use crate::eval::typval::NumBuf;
use crate::ex_docmd::SourceKind;
use crate::ex_eval::CsFlags;
use crate::formatted;
use crate::mbyte::canonize;
use crate::memory::ThinCString;
use crate::message_fmt::msg_cstr;
use crate::os::env::expand_env_save_opt_of;
use crate::os::env::home_replace_in;
use crate::regexp::OwnedProg;
use crate::semsg;
use crate::types::{IOSIZE, ListRef, MAXPATHL};
use core::ffi::{CStr, c_int};
use std::ffi::CString;

/// Bytes `autoload_name` puts in front of the name it is given.
const AUTOLOAD_PREFIX: &[u8] = b"autoload/";
/// ...and what it puts after the last package separator.
const AUTOLOAD_SUFFIX: &[u8] = b".vim";

// ---------------------------------------------------------------------------
// The registry.

/// How many scripts the editor has sourced -- upstream's
/// `script_items.ga_len`, which is also the highest live script id.
pub(crate) fn script_count() -> c_int {
    // The registry only grows, and an editor that has sourced 2^31 scripts
    // has run out of ids long before it runs out of `c_int`.
    SCRIPT_ITEMS.with(|items| c_int::try_from(items.len()).unwrap_or(c_int::MAX))
}

/// Run `f` on the registry entry for script `sid`, which must be a script
/// the editor has sourced ([`script_id_valid`]) -- upstream's `SCRIPT_ITEM`.
///
/// The registry is borrowed for the call: `f` reads and updates the entry's
/// fields and runs no user code, which is all the profiler and the scope
/// lookups do with one.
///
/// # Panics
/// When `sid` names no script.
pub(crate) fn with_script_item<R>(sid: ScriptId, f: impl FnOnce(&mut ScriptItem) -> R) -> R {
    debug_assert!(script_id_valid(sid), "script id out of range");
    let idx = usize::try_from(sid - 1).expect("script id out of range");
    SCRIPT_ITEMS.with_mut(|items| f(&mut items[idx]))
}

/// Is `sid` a script the editor has sourced -- upstream's `SCRIPT_ID_VALID`?
pub(crate) fn script_id_valid(sid: c_int) -> bool {
    sid > 0 && sid <= script_count()
}

/// A copy of script `sid`'s file name; `None` for an id with no script or a
/// script with no name (an anonymous chunk).
pub(crate) fn script_name(sid: ScriptId) -> Option<XString> {
    if !script_id_valid(sid) {
        return None;
    }
    with_script_item(sid, |si| si.sn_name.clone())
}

/// Allocate the next script ID and its `ScriptItem`, named `name`.  IDs are
/// never reused, so the registry is grown to cover every ID up to the new
/// one; the intervening items exist and are empty.
pub(crate) fn new_script_item(name: Option<XString>) -> ScriptId {
    /// The highest script ID handed out so far.
    static LAST_CURRENT_SID: GlobalCell<ScriptId> = GlobalCell::new(0);

    let sid = LAST_CURRENT_SID.get() + 1;
    LAST_CURRENT_SID.set(sid);
    // The registry is grown one id at a time: `new_script_vars` reaches it,
    // so the push has to be a borrow that has been let go by then.
    while script_count() < sid {
        let added = SCRIPT_ITEMS.with_mut(|items| {
            items.push(ScriptItem::new());
            ScriptId::try_from(items.len()).unwrap_or(ScriptId::MAX)
        });
        new_script_vars(added);
    }
    with_script_item(sid, |si| si.sn_name = name);
    sid
}

/// Script `sid`'s file name with the home directory as `~`, cut to `room`
/// bytes; `None` for an id with no script or a script with no name.
pub(crate) fn script_display_name(sid: ScriptId, room: usize) -> Option<XString> {
    let name = script_name(sid)?;
    Some(home_replace_in(None, name.as_cstr(), room, true))
}

/// Was script `sid` written in Lua?
pub fn script_is_lua(sid: ScriptId) -> bool {
    if sid == SID_LUA {
        return true;
    }
    script_id_valid(sid) && with_script_item(sid, |si| si.sn_lua)
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
    let found = SCRIPT_ITEMS.with(|items| {
        items.iter().rposition(|si| {
            si.sn_name
                .as_ref()
                .is_some_and(|sn_name| path_fnamecmp(sn_name, name.to_bytes()) == 0)
        })
    });
    found.map_or(-1, |idx| c_int::try_from(idx + 1).unwrap_or(c_int::MAX))
}

// ---------------------------------------------------------------------------
// `:scriptnames`.

/// `":scriptnames"`, and `":script {id}"` which edits the script instead.
pub fn ex_scriptnames(excmd: &mut ExArg) {
    let (by_number, has_arg) = (
        excmd.addr_count > 0,
        excmd.line.byte_at(excmd.line.arg) != 0,
    );
    if by_number || has_arg {
        edit_script(excmd, by_number);
        return;
    }

    msg_ext_set_kind(c"list_cmd");
    let mut sid: ScriptId = 1;
    // The registry is re-read every round, because the output below can
    // pause for the user, who can run anything.
    while sid <= script_count() && !got_int.get() {
        let listed = sid;
        sid += 1;
        let Some(name) = script_name(listed) else {
            continue;
        };
        let shortname = home_replace_in(None, name.as_cstr(), MAXPATHL as usize, true);
        let row = formatted!(IOSIZE as usize, b"%3d: %s", listed, shortname.as_cstr());
        let row = cstr::owned(&row);
        if !message_filtered(&row) {
            if msg_col.get() > 0 {
                msg_putchar('\n' as c_int);
            }
            msg_display(&row, 0, false);
            line_breakcheck();
        }
    }
}

/// `":script {id}"` / `":script {file}"`: open the named script in a window.
fn edit_script(excmd: &mut ExArg, by_number: bool) {
    if by_number {
        let sid = ScriptId::try_from(excmd.line2).unwrap_or(0);
        let Some(name) = script_name(sid) else {
            emsg(gettext(e_invarg));
            return;
        };
        excmd.set_arg_text(&name);
    } else {
        let expanded = expand_env_save_opt_of(excmd.line.cstr_from(excmd.line.arg), false);
        excmd.set_arg_text(&expanded);
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
            let fmt = gettext(c"API client (channel id %lu)").to_bytes();
            return cstr::owned(&formatted!(IOSIZE as usize, fmt, script_ctx.sc_chan));
        }
        sid => {
            // Every other `sc_sid` is a registry index.
            let Some(sname) = script_name(sid) else {
                let fmt = gettext(c"anonymous :source (script id %d)").to_bytes();
                return cstr::owned(&formatted!(IOSIZE as usize, fmt, sid));
            };
            if !fold_home {
                return CString::from(sname.as_cstr());
            }
            let room = sname.len() + 3;
            return CString::from(home_replace_in(None, sname.as_cstr(), room, true).as_cstr());
        }
    };
    gettext(fixed).to_owned()
}

// ---------------------------------------------------------------------------
// `getscriptinfo()`.

/// The script-local functions defined in the script with id `sid`, as a list of
/// their names.
fn get_script_local_funcs(sid: ScriptId) -> ListRef {
    let funcs: Vec<_> = all_funcs()
        .into_iter()
        .filter(|func| func.script_ctx.get().sc_sid == sid)
        .collect();
    let list = tv_list_alloc(ptrdiff_t::try_from(funcs.len()).unwrap_or(ptrdiff_t::MAX));
    for func in &funcs {
        list.edit().push_str(Some(func.printable_name().as_cstr()));
    }
    list
}

/// Which scripts `getscriptinfo()` was asked about.
enum ScriptQuery {
    /// No argument, or one that named neither key: every script, names only.
    All,
    /// `{'sid': n}`: that one script, with its variables and functions too.
    Sid(VarNumber),
    /// `{'name': pat}`: the scripts whose path matches the pattern.
    Matching(OwnedProg),
    /// The argument was rejected; an error is already pending.
    Rejected,
}

/// `"getscriptinfo()"` function
pub fn f_getscriptinfo(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_list_alloc_ret(result, ptrdiff_t::try_from(script_count()).unwrap_or(0));
    if tv_check_for_opt_dict_arg(args, 0).is_err() {
        return;
    }
    let mut query = script_query(args);
    if !matches!(query, ScriptQuery::Rejected) {
        let list = result.list_handle().expect("the list allocated above");
        report_scripts(&list, &mut query);
    }
}

/// Read `getscriptinfo()`'s optional dict argument.
///
/// A `name` pattern that fails to compile is not an error, it just filters
/// nothing. The pattern matches with the 'ignorecase' of the moment it was
/// read.
fn script_query(args: &[TypVal]) -> ScriptQuery {
    let mut numbuf = NumBuf::new();
    let Some(arg) = args.first().filter(|a| a.v_type() == VAR_DICT) else {
        return ScriptQuery::All;
    };
    // The tag was tested above, so this is the argument's own dictionary.
    let dict = arg.dict_shared().map(|dict| &**dict);

    if let Some(sid_di) = dict_find(dict, b"sid") {
        let Ok(sid) = tv_get_number_chk(&sid_di.di_tv) else {
            return ScriptQuery::Rejected;
        };
        if sid <= 0 {
            let arg1 = msg_cstr(numbuf.string(&sid_di.di_tv));
            semsg!("E475: Invalid value for argument {}: {arg1}", "sid");
            return ScriptQuery::Rejected;
        }
        return ScriptQuery::Sid(sid);
    }

    let pat: Option<ThinCString> = dict_get_string_alloc(dict, b"name");
    pat.and_then(|pat| OwnedProg::compile(pat.as_cstr(), RE_MAGIC + RE_STRING))
        .map_or(ScriptQuery::All, ScriptQuery::Matching)
}

/// Append one dict per script `query` selects to `list`.
fn report_scripts(list: &ListRef, query: &mut ScriptQuery) {
    let total = VarNumber::from(script_count());
    // A `sid` query asks about exactly one script, and answers nothing at all
    // when that script does not exist.
    let (first, last) = match *query {
        ScriptQuery::Sid(sid) => (sid, sid.min(total)),
        _ => (1, total),
    };
    let ignore_case = p_ic();

    for sid in first..=last {
        let sid = ScriptId::try_from(sid).unwrap_or(0);
        let Some(name) = script_name(sid) else {
            continue;
        };
        if let ScriptQuery::Matching(prog) = query
            && prog.exec(name.as_cstr(), 0, ignore_case).is_none()
        {
            continue;
        }

        let d = tv_dict_alloc();
        {
            let dict = d.edit();
            let _ = dict.add_str(b"name", Some(name.as_cstr()));
            let _ = dict.add_number(b"sid", VarNumber::from(sid));
            let _ = dict.add_number(b"version", 1);
            // Vim9 autoload script (:h vim9-autoload), not applicable to Nvim.
            let _ = dict.add_bool(b"autoload", kBoolVarFalse);
        }

        // A script ID was specified, so report that script in full.
        if let ScriptQuery::Sid(_) = *query {
            let sv_dict = crate::eval::vars::script_scope_dict(sid)
                .expect("a sourced script has an s: scope");
            let vars = dict_copy(None, &sv_dict, true, get_copy_id());
            let _ = d.edit().add_dict(b"variables", vars);
            let funcs = get_script_local_funcs(sid);
            let _ = d.edit().add_list(b"functions", Some(funcs));
        }
        list.edit().push_dict(Some(d));
    }
}

// ---------------------------------------------------------------------------
// Leaving a script.

/// Are we sourcing a script, from a file or a buffer or a string?
pub fn sourcing_a_script(excmd: &ExArg) -> bool {
    excmd.source_kind() == SourceKind::Script
}

/// `":scriptencoding"`: set encoding conversion for a sourced script.
pub fn ex_scriptencoding(excmd: &mut ExArg) {
    if !sourcing_a_script(excmd) {
        emsg(gettext(
            c"E167: :scriptencoding used outside of a sourced file",
        ));
        return;
    }
    let name = if excmd.line.arg().is_empty() {
        XString::new()
    } else {
        canonize(excmd.line.arg())
    };
    // Set up for conversion from the specified encoding to 'encoding'.
    let cookie = excmd.sourced_script().expect("checked: a sourced script");
    let _ = p_enc(|value| cookie.conv.setup(Some(name.as_cstr()), Some(value)));
}

/// `":finish"`: mark a sourced file as finished.
pub fn ex_finish(excmd: &mut ExArg) {
    if sourcing_a_script(excmd) {
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
    if reanimate && let Some(cookie) = excmd.sourced_script() {
        cookie.finished = false;
    }
    // Clean up (and deactivate) conditionals, but stop when a try
    // conditional not in its finally clause -- which then is to be executed
    // next -- is found.  In that case make the `":finish"` pending for
    // execution at the `":endtry"`.  Otherwise, finish normally.
    let cond = cond_stack_of(excmd);
    if let Some(at) = cleanup_conditionals(cond, CsFlags::NONE, true) {
        cond.with(|cs| cs.pending[at] = CSTP_FINISH);
        report_pending(PendingAction::Made, CSTP_FINISH, PendingValue::None);
    } else if let Some(cookie) = excmd.sourced_script() {
        cookie.finished = true;
    }
}

// ---------------------------------------------------------------------------
// Autoload.

/// The autoload script path for a function or variable name: `#` becomes
/// `/`, everything after the last `#` is dropped, and `.vim` takes its place.
///
/// `foo#bar#baz` becomes `autoload/foo/bar.vim`.  The caller must make sure
/// `name` contains `AUTOLOAD_CHAR`.
pub(crate) fn autoload_name_of(name: &[u8]) -> XString {
    let mut out =
        Vec::with_capacity(AUTOLOAD_PREFIX.len() + name.len() + AUTOLOAD_SUFFIX.len() + 1);
    out.extend_from_slice(AUTOLOAD_PREFIX);
    out.extend_from_slice(name);

    // Everything from the last separator on is the member name, which the
    // suffix replaces; the separators before it are directories.
    let separator = u8::try_from(AUTOLOAD_CHAR).expect("`#` is a byte");
    let cut = out.iter().rposition(|&b| b == separator).unwrap_or(0);
    out.truncate(cut);
    for byte in &mut out {
        if *byte == separator {
            *byte = b'/';
        }
    }
    out.extend_from_slice(AUTOLOAD_SUFFIX);
    XString::from_vec(out)
}

/// The line number to report for a message raised by a running command.
///
/// A sourced script tracks its own read position, because the execution
/// stack's number lags behind by the one line `getsourceline` reads ahead.
/// Only when the command reads the script directly: inside a loop the
/// stack's number is the replayed line's.
pub(crate) fn sourced_lnum_of(excmd: &ExArg) -> LineNr {
    match excmd.source.as_ref() {
        Some(source) if !source.is_loop() => source
            .script()
            .map_or_else(sourcing_lnum, |cookie| cookie.sourcing_lnum),
        _ => sourcing_lnum(),
    }
}

/// If `name` has a package name, try autoloading the script for it.
///
/// Returns true if a package was loaded.  `reload` loads the script again even
/// when it is already known.
///
/// `name` is only read before anything runs: the search is for a path built
/// from it.
pub(crate) fn script_autoload_named(name: &[u8], reload: bool) -> bool {
    // If there is no `#` after name[0] there is no package name.
    let separator = u8::try_from(AUTOLOAD_CHAR).expect("`#` is a byte");
    if !matches!(name.iter().position(|&b| b == separator), Some(1..)) {
        return false;
    }

    let scriptname = autoload_name_of(name);
    let path: &[u8] = &scriptname;
    let known =
        LOADED_AUTOLOADS.with(|loaded| loaded.iter().any(|entry| same_autoload(entry, path)));

    // Was it loaded already?
    if !reload && known {
        return false;
    }

    // Remember the name if it wasn't loaded already.
    if !known {
        LOADED_AUTOLOADS.with_mut(|loaded| loaded.push(path.to_vec()));
    }

    // Try loading the package from `$VIMRUNTIME/autoload/<name>.vim`.  The
    // `ret_sid` keeps `source_found` from loading the same script twice.
    let mut ret_sid: c_int = 0;
    do_in_runtimepath_visit(
        scriptname.as_cstr(),
        RuntimeOpts::START,
        &mut |found, all| source_found(found, all, Some(&mut ret_sid)),
    )
    .is_ok()
}

/// Are these the same autoload path? The shared `autoload/` prefix is skipped
/// -- every entry carries it, so comparing it again buys nothing.
fn same_autoload(entry: &[u8], path: &[u8]) -> bool {
    let skip = AUTOLOAD_PREFIX.len();
    entry.get(skip..) == path.get(skip..)
}
