//! One step of a firing walk.
//!
//! [`getnextac`] is the `do_cmdline` line source an autocommand body is
//! executed through: each call hands back the next matching command,
//! advancing [`aucmd_next`] over the pattern list until one matches the file
//! name being fired for.  `au_callback` is the same step for an autocommand
//! whose body is a Lua callback rather than a Vimscript command.
//!
//! Nothing here may hold the table across a call into user code.  The
//! walk runs *while* the handler it is running can define and delete
//! autocommands, so each step reads the table afresh through its row index
//! and pattern id: a define appends, a delete only marks (`pat` goes
//! `None` under the index), and the walk's own bound is the `ausize`
//! snapshot `apply_autocmds_group` took -- which is what keeps a handler
//! that defines more autocommands for its own event from looping forever.
//! Compaction, the one thing that moves rows, waits for the last walk.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::lua::executor::nlua_call_ref_quiet;
use crate::memory::XString;
use crate::smsg;
use crate::types::{AutoHandler, EstackInfo, FrameName, PatId};

/// The buffer number `apc`'s `<buffer=N>` patterns match: its own until
/// the walk is running, then the running walks' slot, which
/// [`aubuflocal_remove`](super::aubuflocal_remove) clears when the buffer
/// goes.
fn walk_bufnr(apc: &AutoPatCmd) -> BufferHandle {
    match apc.walk {
        Some(at) => active_walk_bufnrs.with(|nrs| nrs.get(at).copied().unwrap_or(0)),
        None => apc.arg_bufnr,
    }
}

/// What [`aucmd_next`] makes of row `idx`.
enum Step {
    /// Deleted, or its pattern does not match: go on.
    Skip,
    /// The same pattern the walk last stopped at, which matched then.
    Same(PatId),
    /// A new pattern that matches, and the execution stack's name for it.
    Matched(PatId, XString),
}

/// What the table says of row `idx` before any matching.
enum Probe {
    Skip,
    Same(PatId),
    /// A file pattern: its program, taken out for the match, and whether
    /// it may match a directory.
    File(PatId, Option<crate::regexp::OwnedProg>, bool),
    Matched(PatId),
}

/// The names one [`aucmd_next`] call matches every row against, measured
/// once rather than per row.
struct Names<'a> {
    fname: &'a CStr,
    sfname: Option<&'a CStr>,
    tail: &'a CStr,
    bufnr: BufferHandle,
}

/// The execution stack's name for an autocommand of `event` on `pattern`:
/// `"%s Autocommands for \"%s\""`, translated.
fn sourcing_name(event: AutoEvent, pattern: &[u8]) -> XString {
    const TEMPLATE: &CStr = c"%s Autocommands for \"%s\"";
    let name = event_nr2name(event);
    let s = gettext(TEMPLATE);
    if s == TEMPLATE {
        // Untranslated, so the format is a concatenation: the formatter's
        // two passes are most of what a firing costs when nothing else
        // happens.
        let mut out = XString::with_capacity(name.count_bytes() + pattern.len() + 20);
        out.push_cstr(name);
        out.push_bytes(b" Autocommands for \"");
        out.push_bytes(pattern);
        out.push_byte(b'"');
        return out;
    }
    let room = s.count_bytes() + name.count_bytes() + pattern.len() + 1;
    XString::from_vec(crate::formatted!(room, s.to_bytes(), name, pattern))
}

/// Whether row `idx` of `apc`'s event is one to run.
///
/// The program is taken out of the table for the match and put back after
/// it, so nothing holds the table while the matcher runs -- it can poll for
/// input, which runs fast events. A pattern a nested firing finds with its
/// program out matches nothing, which is what upstream's matcher, taking
/// the program into its `regmatch_T`, did too.
fn step_at(apc: &AutoPatCmd, names: &Names<'_>, idx: usize) -> Step {
    let probe = with_event_cmds_mut(apc.event, |cmds| {
        // Skip deleted autocommands.
        let Some(id) = cmds.cmds[idx].pat else {
            return Probe::Skip;
        };
        // Skip the matching when the pattern did not change.
        if Some(id) == apc.lastpat {
            return Probe::Same(id);
        }
        let pat = cmds.pat_mut(id);
        // Skip the ones that don't match the group...
        if apc.group != AUGROUP_ALL && apc.group != pat.group {
            return Probe::Skip;
        }
        // ...or the pattern, or the buffer number.
        if pat.buflocal_nr == 0 {
            Probe::File(id, pat.reg_prog.take(), pat.allow_dirs)
        } else if pat.buflocal_nr == names.bufnr {
            Probe::Matched(id)
        } else {
            Probe::Skip
        }
    });
    let (id, prog) = match probe {
        Probe::Skip => return Step::Skip,
        Probe::Same(id) => return Step::Same(id),
        Probe::Matched(id) => (id, None),
        Probe::File(id, mut prog, allow_dirs) => {
            let matched =
                match_file_pat(&mut prog, names.fname, names.sfname, names.tail, allow_dirs);
            if !matched {
                // Rows and patterns do not go while a walk runs, so the
                // slot is the one the program came out of.
                with_event_cmds_mut(apc.event, |cmds| cmds.pat_mut(id).reg_prog = prog);
                return Step::Skip;
            }
            (id, prog)
        }
    };
    let event = apc.event;
    with_event_cmds_mut(event, |cmds| {
        let pat = cmds.pat_mut(id);
        if prog.is_some() {
            pat.reg_prog = prog;
        }
        Step::Matched(id, sourcing_name(event, &pat.pat))
    })
}

/// Advance `apc` to the next autocommand whose pattern matches, updating
/// the execution-stack entry when the pattern changes.
///
/// When there is no next one, `lastpat` is left `None` and `auidx` at
/// `usize::MAX`, which is how [`getnextac`] and its caller see the end.
pub(crate) fn aucmd_next(apc: &mut AutoPatCmd) {
    debug_assert!(apc.ausize <= with_event_cmds(apc.event, EventCmds::len));

    let found = {
        let names = Names {
            fname: apc.fname.as_cstr(),
            sfname: apc.sfname.as_ref().map(XString::as_cstr),
            tail: apc.fname.cstr_at(apc.tail),
            bufnr: walk_bufnr(apc),
        };
        let mut found = None;
        // `ausize` is the snapshot, not the current length: a handler that
        // defines more autocommands for its own event must not extend the
        // walk it is running under.
        for idx in apc.auidx..apc.ausize {
            if got_int.get() {
                break;
            }
            match step_at(apc, &names, idx) {
                Step::Skip => {}
                Step::Same(id) => {
                    found = Some((idx, id, None));
                    break;
                }
                Step::Matched(id, namep) => {
                    found = Some((idx, id, Some(namep)));
                    break;
                }
            }
        }
        found
    };

    let Some((idx, id, namep)) = found else {
        // Nothing left: clear the ETYPE_AUCMD stack entry.
        crate::runtime::set_sourcing_name(FrameName::None);
        crate::runtime::with_innermost(|entry| entry.es_info = EstackInfo::None);

        apc.lastpat = None;
        apc.auidx = usize::MAX;
        return;
    };

    if let Some(namep) = namep {
        if p_verbose() >= 8 {
            verbose_enter();
            let namep = crate::message_fmt::msg_cstr(namep.as_cstr());
            smsg!(0, "Executing {namep}");
            verbose_leave();
        }

        // Point the execution stack at this autocommand, and at the script
        // context of the command the walk is on.
        crate::runtime::set_sourcing_name(FrameName::Owned(namep));
        let sctx = apc.script_ctx;
        crate::runtime::with_innermost(|entry| {
            entry.es_info = EstackInfo::Autocommand(sctx);
        });
    }

    apc.lastpat = Some(id);
    apc.auidx = idx;

    line_breakcheck();
}

/// What a callback handler is, held outside the table for its call.
enum Handler {
    /// A Lua function, by its registry reference -- which is all the call
    /// needs, as upstream's bit copy of the row was.
    Lua(LuaRef),
    /// A funcref or partial, counted for the length of the call: the
    /// handler may delete its own autocommand.
    Vim(Callback),
}

/// Run an autocommand whose handler is a callback rather than an Ex
/// command.
///
/// Answers whether the callback asked to be deleted, which only a Lua one
/// can do (by returning `true`).
fn au_callback(
    handler: Handler,
    id: int64_t,
    group: ::core::ffi::c_int,
    apc: &AutoPatCmd,
    event_data: Option<&Object>,
) -> bool {
    let luaref = match handler {
        Handler::Vim(mut callback) => {
            let mut rettv = TV_INITIAL_VALUE;
            callback_call(&callback, &[], &mut rettv);
            callback.clear();
            return false;
        }
        Handler::Lua(luaref) => luaref,
    };

    let mut data = DictBuf::<7>::new();
    data.insert(c"id", Object::integer(id));
    data.insert(
        c"event",
        Object::string(String_0::from_cstr(event_nr2name(apc.event))),
    );
    data.insert(
        c"file",
        Object::string(
            apc.afile_orig
                .as_ref()
                .map_or(String_0::NULL, |file| String_0::from_bytes(file)),
        ),
    );
    data.insert(
        c"match",
        Object::string(autocmd_match.with(|name| {
            name.as_ref()
                .map_or(String_0::NULL, |name| String_0::from_bytes(name))
        })),
    );
    data.insert(c"buf", Object::integer(Integer::from(autocmd_bufnr.get())));
    if let Some(event_data) = event_data {
        // The publisher's object stays the publisher's: the entry gets a
        // copy.
        data.insert(c"data", event_data.clone());
    }
    match group {
        AUGROUP_ERROR => std::process::abort(),
        // The pseudo-groups are not something a handler can be told.
        AUGROUP_DEFAULT | AUGROUP_ALL | AUGROUP_DELETED => {}
        group => {
            data.insert(c"group", Object::integer(Integer::from(group)));
        }
    }

    let mut args = ArrayBuf::<1>::new();
    args.push(data.object());

    // `args` stands for the length of the call.
    let result = nlua_call_ref_quiet(luaref, None, args.array(), kRetNilBool);
    matches!(result, Object::Boolean(true))
}

/// What [`getnextac`] reads off the row it runs, before anything runs.
struct Running {
    once: bool,
    nested: bool,
    script_ctx: ScriptCtx,
    id: int64_t,
    group: ::core::ffi::c_int,
    /// The command line, or `None` for a callback.
    command: Option<XString>,
    /// The callback, for a row that has one.
    callback: Option<Handler>,
}

/// The next command of an autocommand walk, for `do_cmdline` -- the
/// [`Autocmd`](crate::ex_docmd::LineSource::Autocmd) arm of its line source:
/// one line per matching autocommand, `None` when the walk is over.
/// `data` is the firing's `data`, for a Lua callback's argument.
///
/// A callback handler has no line to give back, so this answers an empty
/// line for it -- "not the end, keep going".
pub fn getnextac(apc: &mut AutoPatCmd, data: Option<&Object>) -> Option<XString> {
    aucmd_next(apc);
    let pat_id = apc.lastpat?;
    let idx = apc.auidx;

    let row = with_event_cmds(apc.event, |cmds| {
        debug_assert!(idx < cmds.len());
        // The row the walk just stopped at, which `aucmd_next` only stops
        // at while it still has a pattern.
        let cmd = &cmds.cmds[idx];
        debug_assert!(cmd.pat.is_some());
        let (command, callback) = match &cmd.handler {
            AutoHandler::Cmd(line) => (Some(line.clone()), None),
            AutoHandler::Fn(Callback::Lua(luaref)) => (None, Some(Handler::Lua(*luaref))),
            AutoHandler::Fn(callback) => (None, Some(Handler::Vim(callback.duplicate()))),
        };
        Running {
            once: cmd.once,
            nested: cmd.nested,
            script_ctx: cmd.script_ctx,
            id: cmd.id,
            group: cmds.pat(pat_id).group,
            command,
            callback,
        }
    });
    let mut oneshot = row.once;

    if p_verbose() >= 9 {
        let handler_str =
            with_event_cmds(apc.event, |cmds| aucmd_handler_to_string(&cmds.cmds[idx]));
        verbose_enter_scroll();
        let shown = crate::message_fmt::msg_cstr(handler_str.as_cstr());
        smsg!(0, "autocommand {shown}");
        // Don't overwrite this either.
        msg_str(c"\n");
        verbose_leave_scroll();
    }

    // `autocmd_nested` has to be set before any Lua runs, or a nested
    // event fired from the callback sees the wrong value.
    autocmd_nested.set(row.nested);
    current_sctx.set(row.script_ctx);
    apc.script_ctx = row.script_ctx;
    // The stack frame carries the context of the command now running.
    let sctx = row.script_ctx;
    crate::runtime::with_innermost(|entry| {
        if let EstackInfo::Autocommand(running) = &mut entry.es_info {
            *running = sctx;
        }
    });

    let retval = match (row.command, row.callback) {
        (Some(command), _) => command,
        (None, callback) => {
            let callback = callback.expect("a row without a command has a callback");
            // Mark a `++once` handler removed *before* running it, so a
            // `:doautocmd` from inside it cannot run it again (#25526); it
            // keeps its count of the pattern, which the deletion below
            // gives up.
            if oneshot {
                with_event_cmds_mut(apc.event, |cmds| cmds.cmds[idx].pat = None);
            }
            let rv = au_callback(callback, row.id, row.group, apc, data);
            if oneshot {
                // By index: rows do not move while a walk runs.
                with_event_cmds_mut(apc.event, |cmds| cmds.cmds[idx].pat = Some(pat_id));
            }
            // A callback returning true asks to be deleted.
            oneshot = oneshot || rv;

            // HACK(tjdevries): we just return "not-null" and keep going.
            // Fixing it means either teaching `do_cmdline` to take
            // something other than a string, or looping over the matches
            // here instead of being pulled.
            XString::new()
        }
    };

    // Delete a one-shot autocommand in anticipation of its execution.
    if oneshot {
        aucmd_del(apc.event, idx);
    }

    if apc.auidx < apc.ausize {
        apc.auidx += 1;
    } else {
        apc.auidx = usize::MAX;
    }

    Some(retval)
}
