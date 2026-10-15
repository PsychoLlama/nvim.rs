//! Sourcing a script -- `:source`, `:runtime`'s visit, and `nvim_exec2`.
//!
//! `do_source_ext` is the whole of it, and it reads its lines from one of
//! three places (see [`Origin`]): a file, the current buffer, or a string the
//! API handed over.  Whichever it is, the shape is the same -- resolve a name,
//! find or create the `ScriptItem`, push it on the execution stack, run it,
//! and unwind all of that whatever happens -- so [`source_bracket`] reads as
//! that sequence of named stages.  Everything else here is an entry point into
//! it. The lines themselves are read through the [`SourceCookie`] the run
//! owns, which holds the file or the lines a buffer or a string was split
//! into.

#![forbid(unsafe_code)]

use super::*;
use crate::autocmd::fire_autocmds_for;
use crate::cstr;
use crate::formatted;
use crate::lua::executor::nlua_exec_cstr;
use crate::memory::XString;
use crate::message_fmt::{msg_bytes, msg_cstr, msg_cstr_opt, report_msg, to_bytes};
use crate::os::env::env_set;
use crate::semsg;
use crate::smsg;
use crate::tr_c;
use crate::usercmd::win_cmd_modifiers;
use crate::winlayer::Buf;

use crate::ex_docmd::{DoCmdOpts, LineSource};
use crate::guard::Script;
use crate::memline::Lines;
use crate::optionstr::OptStringRef;
use crate::os::cshim::gettext;
use crate::os::env::expand_env_save_opt_of;
use crate::os::fs::dir_exists;
use crate::types::{FAIL, IOSIZE, OK};
use core::ffi::{CStr, c_int};
use std::ffi::{CString, OsStr};
use std::fs::File;
use std::io::BufReader;
use std::os::unix::ffi::OsStrExt;

/// `:source` with `fname`, or without it when `fname` is empty.
///
/// `fname` is the caller's copy: the script runs with it borrowed.
fn cmd_source(fname: &CStr, excmd: Option<&mut ExArg>) {
    let named = !fname.is_empty();
    let forceit = excmd.as_deref().is_some_and(|command| command.forceit);
    if named
        && excmd
            .as_deref()
            .is_some_and(|command| command.addr_count > 0)
    {
        // A range only makes sense when the lines come from a buffer.
        emsg(gettext(e_norange));
        return;
    }
    let sourced = if let Some(command) = excmd {
        if !named {
            if forceit {
                emsg(gettext(e_argreq));
            } else {
                cmd_source_buffer(command, false);
            }
            true
        } else if forceit {
            // `:source!` feeds the file to the editor as typed keys.
            let busy = global_busy.get() != 0
                || listcmd_busy.get()
                || !command.line.next.is_none()
                || command
                    .cond_stack
                    .is_some_and(|cond| cond.with(|cs| cs.idx >= 0));
            openscript(fname, busy);
            true
        } else {
            false
        }
    } else {
        false
    };
    if !sourced && do_source(fname, false, DOSO_NONE, None) == FAIL {
        let fname = msg_cstr(fname);
        semsg!("E484: Can't open file {fname}");
    }
}

/// `:source`.
pub fn ex_source(excmd: &mut ExArg) {
    // A copy: the command is lent on with it, and `:source` of a buffer
    // reads the command's own line.
    let fname = excmd.line.cstr_from(excmd.line.arg).to_owned();
    cmd_source(&fname, Some(excmd));
}

/// `:options`, which is `:source` of the option window script with the
/// command modifiers passed along in the environment.
pub fn ex_options(_excmd: &mut ExArg) {
    let mods = CString::new(cmdmod.with(win_cmd_modifiers)).expect("no NUL in a word");
    let _ = env_set(c"OPTWIN_CMD", &mods);
    cmd_source(SYS_OPTWIN_FILE, None);
}

/// Open `name` for reading. std opens every descriptor `O_CLOEXEC`, which is
/// what upstream's `fopen_noinh_readbin` set by hand.
fn open_readbin(name: &CStr) -> Option<BufReader<File>> {
    File::open(OsStr::from_bytes(name.to_bytes()))
        .ok()
        .map(BufReader::new)
}

/// Collect `excmd`'s range of the current buffer into a cookie, and answer
/// it with the name to show for those lines: the buffer's own file name, or
/// a synthetic `:source buffer=N` when it has none.
fn do_source_buffer_init(excmd: &ExArg, ex_lua: bool) -> Option<(SourceCookie, XString)> {
    let buf = Buf::current_or_none()?;
    let fname = match buf.name.full() {
        Some(ffname) => XString::from_cstr(ffname),
        None => {
            let fmt: &[u8] = if ex_lua {
                b":{range}lua buffer=%d"
            } else {
                b":source buffer=%d"
            };
            XString::from_vec(formatted!(IOSIZE as usize, fmt, buf.handle))
        }
    };
    let (line1, line2) = (excmd.line1, excmd.line2);
    let mut lines = Lines::in_buffer(buf);
    let copied = (line1..=line2)
        .map(|lnum| CString::new(lines.line(lnum)).expect("a buffer line holds no NUL"))
        .collect();
    let mut cookie = SourceCookie::from_lines(copied);
    // The first line the reader hands out is `line1`, so the counter starts
    // one below it.
    cookie.sourcing_lnum = line1 - 1;
    Some((cookie, fname))
}

/// Split `text` into lines for a cookie to read.
fn do_source_str_init(text: &CStr) -> SourceCookie {
    let mut lines = Vec::new();
    let mut rest = text.to_bytes();
    while !rest.is_empty() {
        let eol = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
        lines.push(CString::new(&rest[..eol]).expect("a sourced line holds no NUL"));
        // Step over the newline, unless this was the last line -- which
        // ends at the terminator instead.
        rest = rest.get(eol + 1..).unwrap_or_default();
    }
    SourceCookie::from_lines(lines)
}

/// Source the current buffer's lines, as Vimscript or (with `ex_lua`) as Lua.
pub fn cmd_source_buffer(excmd: &ExArg, ex_lua: bool) {
    let mut req = SourceRequest::new(Origin::Buffer { excmd, ex_lua }, None);
    do_source_ext(&mut req);
}

/// Source `text` as Vimscript, under `traceback_name`.
///
/// The name is decorated with where the call came from, so an `nvim_exec2()`
/// nested inside a script says so.
///
/// `text` is copied into the script's lines before any of them runs, and
/// the name is only ever read, for messages.
pub fn do_source_str(text: &CStr, traceback_name: &CStr) -> c_int {
    // Upstream's `snprintf` into 256 bytes of "%s called at %s:%d", spelled
    // out: this runs for every `nvim_exec2()` from inside a script.
    let decorated = sourcing_name_copy().map(|name| {
        let lnum = sourcing_lnum().to_string();
        let mut text = Vec::with_capacity(256);
        text.extend_from_slice(traceback_name.to_bytes());
        text.extend_from_slice(b" called at ");
        text.extend_from_slice(&name);
        text.push(b':');
        text.extend_from_slice(lnum.as_bytes());
        text.truncate(255);
        cstr::owned(&text)
    });
    let name = decorated.as_deref().unwrap_or(traceback_name);
    let mut req = SourceRequest::new(Origin::Str { text }, Some(name));
    do_source_ext(&mut req)
}

/// Where a `do_source_ext` call reads its lines from.
///
/// The C asks this with two null checks in seven separate places; naming it
/// is what makes the stages below readable.
enum Origin<'a, 'e> {
    /// The current buffer, over the command's range; `ex_lua` sources it as
    /// Lua regardless of what it looks like.
    Buffer { excmd: &'a ExArg<'e>, ex_lua: bool },
    /// A string handed in by the API. The request's name is only a traceback
    /// name, and no `ScriptItem` is allocated.
    Str { text: &'a CStr },
    /// A file on disk named by the request's name.
    File,
}

/// One call into [`do_source_ext`], minus the cookie it fills in.
struct SourceRequest<'a, 'e> {
    origin: Origin<'a, 'e>,
    /// The name as the caller spelled it -- messages only; the resolved name
    /// is the run's. `None` for a buffer.
    fname: Option<&'a CStr>,
    /// Try the other spelling of the init file's name as well.
    check_other: bool,
    /// A `DOSO_` value: whether this is the user's init file.
    is_vimrc: c_int,
    /// Where to report the script ID, if the caller wants it.
    ret_sid: Option<&'a mut c_int>,
}

impl<'a, 'e> SourceRequest<'a, 'e> {
    /// The request `origin` and `fname` describe, with the init-file fields
    /// at their defaults; [`do_source`] is the only caller that sets those.
    fn new(origin: Origin<'a, 'e>, fname: Option<&'a CStr>) -> Self {
        SourceRequest {
            origin,
            fname,
            check_other: false,
            is_vimrc: DOSO_NONE,
            ret_sid: None,
        }
    }

    fn is_str(&self) -> bool {
        matches!(self.origin, Origin::Str { .. })
    }
}

/// Load a cookie from the buffer or the string, and answer it with the name
/// this source will be known by.  `Err` is `do_source_ext`'s whole return
/// value: no current buffer, or a directory (the one case that reports
/// itself).
fn source_name(req: &SourceRequest) -> Result<(SourceCookie, XString), c_int> {
    match req.origin {
        Origin::Buffer { excmd, ex_lua } => do_source_buffer_init(excmd, ex_lua).ok_or(FAIL),
        Origin::Str { text } => {
            let name = XString::from_cstr(req.fname.unwrap_or(c""));
            Ok((do_source_str_init(text), name))
        }
        Origin::File => {
            let fname = req.fname.unwrap_or(c"");
            let expanded = expand_env_save_opt_of(fname, false);
            let name = fix_fname(expanded.as_cstr());
            if !dir_exists(name.as_cstr()) {
                return Ok((SourceCookie::new(), name));
            }
            let fname = msg_cstr(fname);
            smsg!(0, "Cannot source a directory: \"{fname}\"");
            Err(FAIL)
        }
    }
}

/// Fire `event` with the script's name as `<amatch>` and `<afile>`.
fn fire_source_event(event: AutoEvent, fname_exp: &CStr) -> bool {
    let name = Some(fname_exp);
    fire_autocmds_for(event, name, name, false, Buf::current_or_none(), None)
}

/// Run the `SourceCmd` and `SourcePre` autocommands.
///
/// `Some` when a `SourceCmd` handler took the file over, in which case
/// nothing else happens: the handler *is* the sourcing.
fn source_autocmds(fname_exp: &CStr) -> Option<c_int> {
    // The handlers may run arbitrary script; the name is the run's own.
    if has_autocmd(AutoEvent::SourceCmd, fname_exp, None)
        && fire_source_event(AutoEvent::SourceCmd, fname_exp)
    {
        let retval = if aborting() { FAIL } else { OK };
        if retval == OK {
            fire_source_event(AutoEvent::SourcePost, fname_exp);
        }
        return Some(retval);
    }
    fire_source_event(AutoEvent::SourcePre, fname_exp);
    None
}

/// Open the script file.
///
/// With `check_other` -- which only the init-file search sets -- a miss is
/// retried under the other spelling of the leading character, so `.nvimrc`
/// finds `_nvimrc` and `.exrc` finds `_exrc`.  The retry rewrites the name,
/// so later messages name the file that was actually opened.
fn open_script(cookie: &mut SourceCookie, fname_exp: &mut XString, check_other: bool) {
    if !matches!(cookie.lines, ScriptLines::None) {
        // A buffer or a string: the lines are loaded already.
        return;
    }
    if let Some(file) = open_readbin(fname_exp.as_cstr()) {
        cookie.lines = ScriptLines::File(file);
        return;
    }
    if !check_other {
        return;
    }
    let tail = tail_index(fname_exp);
    let leader = fname_exp.get(tail).copied().unwrap_or(0);
    let rest = fname_exp.get(tail + 1..).unwrap_or_default();
    if (leader == b'.' || leader == b'_')
        && (rest.eq_ignore_ascii_case(b"nvimrc") || rest.eq_ignore_ascii_case(b"exrc"))
    {
        let mut renamed = fname_exp.to_vec();
        renamed[tail] = if leader == b'_' { b'.' } else { b'_' };
        *fname_exp = XString::from_vec(renamed);
        if let Some(file) = open_readbin(fname_exp.as_cstr()) {
            cookie.lines = ScriptLines::File(file);
        }
    }
}

/// One of the four `'verbose' > 1` sourcing traces.
///
/// `plain` is used at the top level and `numbered` -- which takes the line
/// number first -- when something else is already executing.
fn verbose_source_msg(plain: &'static CStr, numbered: &'static CStr, fname: Option<&CStr>) {
    let named = with_sourcing_name(|name| name.is_some());
    let lnum = int64_t::from(sourcing_lnum());
    verbose_enter();
    let fname = msg_cstr_opt(fname);
    if named {
        let _: bool = report_msg(0, || tr_c!(numbered, lnum, fname));
    } else {
        let _: bool = report_msg(0, || tr_c!(plain, fname));
    }
    verbose_leave();
}

/// Find or create the `ScriptItem` this source runs under, answering its id.
/// Sourcing a string allocates no item at all.
fn register_script(
    req: &mut SourceRequest,
    sid: &mut c_int,
    fname_exp: &XString,
) -> Option<ScriptId> {
    if *sid > 0 {
        // Loading the same script again.
        return Some(*sid);
    }
    if req.is_str() {
        return None;
    }
    *sid = new_script_item(Some(fname_exp.clone()));
    let is_lua = path_with_extension(fname_exp.as_cstr(), c"lua");
    with_script_item(*sid, |si| si.sn_lua = is_lua);
    if let Some(ret_sid) = req.ret_sid.as_deref_mut() {
        *ret_sid = *sid;
    }
    Some(*sid)
}

/// Arm and start script `sid`'s profile timer.
fn profile_script_start(sid: ScriptId, fname_exp: &CStr) {
    // Asked before the item is borrowed: the lists are the debugger's.
    let forced = if with_script_item(sid, |si| si.sn_prof_on) {
        None
    } else {
        profiling_forced(true, fname_exp)
    };
    with_script_item(sid, |si| {
        if let Some(forceit) = forced {
            profile_init(si);
            si.sn_pr_force = forceit;
        }
        if si.sn_prof_on {
            si.sn_pr_count += 1;
            si.sn_pr_start = profile_start();
            si.sn_pr_children = profile_zero();
        }
    });
}

/// Fold the elapsed time into the script's profile totals.
///
/// The script is the current one, looked up again rather than carried across
/// the run.
fn profile_script_stop(wait_start: ProfTime) {
    with_script_item(current_sctx.get().sc_sid, |si| {
        if si.sn_prof_on {
            si.sn_pr_start = profile_end(si.sn_pr_start);
            si.sn_pr_start = profile_sub_wait(wait_start, si.sn_pr_start);
            si.sn_pr_total = profile_add(si.sn_pr_total, si.sn_pr_start);
            si.sn_pr_self = profile_self(si.sn_pr_self, si.sn_pr_start, si.sn_pr_children);
        }
    });
}

/// Whether the current buffer is Lua by 'filetype' or by file name.
fn curbuf_is_lua() -> bool {
    let buf = Buf::current();
    let ft_is_lua = buf.b_p_ft.cstr() == c"lua";
    ft_is_lua
        || buf
            .name
            .shown()
            .is_some_and(|name| path_with_extension(name, c"lua"))
}

/// Whether treesitter parses `excmd`'s range of the current buffer as Lua --
/// which is what makes a fenced Lua block inside a help file `:source`able.
fn range_is_lua(excmd: &ExArg) -> bool {
    let (handle, line1, line2) = (Buf::current().handle, excmd.line1, excmd.line2);
    let args = Array::from(vec![
        integer_obj(Integer::from(handle)),
        integer_obj(Integer::from(line1)),
        integer_obj(Integer::from(line2)),
    ]);
    let src = c"return require('vim._core.util').source_is_lua(...)";
    match nlua_exec_cstr(src, args, kRetNilBool) {
        Ok(value) => value.as_boolean() == Some(true),
        Err(mut err) => {
            err.clear();
            false
        }
    }
}

/// Strip a UTF-8 BOM off the first line, setting up the conversion it implies.
fn strip_bom(conv: &mut VimConv, firstline: Option<XString>) -> Option<XString> {
    let first = firstline?;
    let Some(rest) = first.strip_prefix(b"\xef\xbb\xbf") else {
        return Some(first);
    };
    let _ = p_enc(|value| conv.setup(Some(c"utf-8"), Some(value)));
    let recoded = conv.convert(rest).unwrap_or_else(|| rest.to_vec());
    Some(XString::from_vec(recoded))
}

/// Run the script's lines, in whichever language they turn out to be.
fn execute_source(
    req: &SourceRequest,
    cookie: &mut SourceCookie,
    sid: Option<ScriptId>,
    fname_exp: &CStr,
) {
    if let Origin::Buffer { excmd, ex_lua } = req.origin
        && (ex_lua || curbuf_is_lua() || range_is_lua(excmd))
    {
        nlua_exec_lines(cookie.buffered_lines(), fname_exp);
        return;
    }
    if sid.is_some_and(|sid| with_script_item(sid, |si| si.sn_lua)) {
        nlua_exec_file(fname_exp);
        return;
    }
    // The first line is read up front, so a BOM can be sniffed off it.
    let first = cookie.getsourceline(true);
    let firstline = strip_bom(&mut cookie.conv, first);
    let flags = DoCmdOpts::VERBOSE | DoCmdOpts::NOWAIT | DoCmdOpts::REPEAT;
    let first = firstline.as_deref();
    let _ = do_cmdline(first, Some(&mut LineSource::Script(cookie)), flags);
}

/// Everything from opening the script to closing it, in the order it has to
/// come back down: the profile timers, the funccal stack, `current_sctx` and
/// the execution stack all outlive the run and are restored here.
///
/// `save_debug_break_level` was read before any of this ran.
fn source_bracket(
    req: &mut SourceRequest,
    cookie: &mut SourceCookie,
    mut fname_exp: XString,
    save_debug_break_level: c_int,
) -> c_int {
    let mut sid = if req.is_str() {
        SID_STR
    } else {
        find_script_by_name(fname_exp.as_cstr())
    };
    if sid > 0
        && let Some(ret_sid) = req.ret_sid.as_deref_mut()
    {
        // Already loaded, and the caller only wanted the ID.
        *ret_sid = sid;
        return OK;
    }
    if !req.is_str()
        && let Some(retval) = source_autocmds(fname_exp.as_cstr())
    {
        return retval;
    }

    open_script(cookie, &mut fname_exp, req.check_other);
    if matches!(cookie.lines, ScriptLines::None) {
        if p_verbose() > 1 {
            let numbered = c"line %ld: could not source \"%s\"";
            verbose_source_msg(c"could not source \"%s\"", numbered, req.fname);
        }
        return FAIL;
    }

    // The file exists.  Everything set up from here has to come back down
    // before this function returns.
    if p_verbose() > 1 {
        let numbered = c"line %ld: sourcing \"%s\"";
        verbose_source_msg(c"sourcing \"%s\"", numbered, req.fname);
    }
    if req.is_vimrc == DOSO_VIMRC {
        vimrc_found(fname_exp.as_cstr(), c"MYVIMRC");
    }

    cookie.breakpoint = dbg_find_breakpoint_named(true, fname_exp.as_cstr(), 0);
    cookie.fname = fname_exp.clone();
    cookie.dbg_tick = debug_tick.get();
    cookie.level = ex_nesting_level.get();

    // Start measuring load time, if --startuptime opened the log.
    let time_log = startup_timing();
    let (rel_time, start_time) = if !time_log { (0, 0) } else { time_push() };
    let profiling = do_profiling.get() == PROF_YES;
    let wait_start = if profiling { prof_child_enter() } else { 0 };

    // Don't use the calling function's local variables.
    let call_stack_aside = CallStackAside::new();
    let script_ctx = Script::saved();

    // Always use a new sequence number.
    let seq = LAST_SCRIPT_SEQ.get() + 1;
    LAST_SCRIPT_SEQ.set(seq);
    current_sctx.with_mut(|sctx| sctx.sc_seq = seq);

    let item = register_script(req, &mut sid, &fname_exp);
    debug_assert!(
        item.is_none() == req.is_str(),
        "(si != NULL) == (str == NULL)"
    );

    // Sourcing a string from a Lua script keeps the Lua script's SID, so
    // `:verbose` still names something useful.
    if !req.is_str() || !script_is_lua(current_sctx.get().sc_sid) {
        current_sctx.with_mut(|sctx| {
            sctx.sc_sid = sid;
            sctx.sc_lnum = 0;
        });
    }

    // Keep the sourcing name and line number, for recursive calls. A script
    // sourced again goes by the registry's spelling of its name, which
    // `find_script_by_name` matched without regard to case or separators.
    let frame_name = item
        .and_then(script_name)
        .unwrap_or_else(|| fname_exp.clone());
    estack_push(ETYPE_SCRIPT, FrameName::Owned(frame_name.clone()), 0);
    if profiling && let Some(sid) = item {
        profile_script_start(sid, frame_name.as_cstr());
    }
    cookie.conv.vc_type = CONV_NONE;

    execute_source(req, cookie, item, fname_exp.as_cstr());

    if profiling && item.is_some() {
        profile_script_stop(wait_start);
    }
    if got_int.get() {
        emsg(gettext(e_interr));
    }
    estack_pop();
    if p_verbose() > 1 {
        let resumed = sourcing_name_copy();
        verbose_enter();
        let fname = msg_cstr_opt(req.fname);
        smsg!(0, "finished sourcing {fname}");
        if let Some(resumed) = resumed {
            let resumed = msg_bytes(&resumed);
            smsg!(0, "continuing in {resumed}");
        }
        verbose_leave();
    }
    if time_log {
        let label = format!("sourcing {}", msg_cstr_opt(req.fname));
        time_msg(&to_bytes(&label), Some(start_time));
        time_pop(rel_time);
    }
    let trigger_source_post = !got_int.get();

    // After a `:finish` in debug mode, break at the first command of the next
    // sourced file.
    if save_debug_break_level > ex_nesting_level.get()
        && debug_break_level.get() == ex_nesting_level.get()
    {
        debug_break_level.set(debug_break_level.get() + 1);
    }
    drop(script_ctx);
    drop(call_stack_aside);
    if profiling {
        prof_child_exit(wait_start);
    }
    // Close the file and drop the lines and the conversion before
    // `SourcePost` runs, as upstream's cleanup did.
    *cookie = SourceCookie::new();
    if !req.is_str() && trigger_source_post {
        fire_source_event(AutoEvent::SourcePost, fname_exp.as_cstr());
    }
    OK
}

/// Read a script and run it.
///
/// Answers FAIL when the file could not be opened, OK otherwise.  When a
/// `ScriptItem` was found or created, `ret_sid` -- if given -- gets its ID,
/// and a script that has one already is *not* run again.
fn do_source_ext(req: &mut SourceRequest) -> c_int {
    let save_debug_break_level = debug_break_level.get();
    let (mut cookie, fname_exp) = match source_name(req) {
        Ok(loaded) => loaded,
        Err(retval) => return retval,
    };
    source_bracket(req, &mut cookie, fname_exp, save_debug_break_level)
}

/// [`do_source_ext`] for a file: the spelling every caller outside this module
/// uses.  `check_other` also tries the other init-file name; `is_vimrc` is a
/// `DOSO_` value.
///
/// `fname` is borrowed for the whole run of the script, which can run
/// anything: hand in a copy of your own, never an option's value or a
/// buffer's line.
pub fn do_source(
    fname: &CStr,
    check_other: bool,
    is_vimrc: c_int,
    ret_sid: Option<&mut c_int>,
) -> c_int {
    let mut req = SourceRequest {
        check_other,
        is_vimrc,
        ret_sid,
        ..SourceRequest::new(Origin::File, Some(fname))
    };
    do_source_ext(&mut req)
}
