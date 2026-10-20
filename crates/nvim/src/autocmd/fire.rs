//! Firing: `apply_autocmds` and the walk it sets up.
//!
//! [`apply_autocmds_group`] is the whole event: it decides whether anything
//! matches, saves and swaps the editor state an autocommand is allowed to
//! see (`<afile>`, `<abuf>`, `v:event`, the search patterns, the redo
//! buffer), runs the matching commands through `do_cmdline` with an
//! `AutoPatCmd` as the walk's cursor, then unwinds all of it.  The
//! three `apply_autocmds*` entry points above it differ only in what they
//! pass and what they return.
//!
//! The walk's cursor, `patcmd`, owns copies of every name it matches
//! against, so the caller's names are only read before anything runs. What
//! upstream also published the cursor for, a handler wiping out the buffer
//! the walk matches `<buffer=N>` against, goes through the running walks'
//! numbers in the autocommand state instead: the walk pushes its number when
//! it starts running and keeps the index (`AutoPatCmd::walk`), and
//! `aubuflocal_remove` clears the number there.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::ex_docmd::{DoCmdOpts, LineSource};
use crate::getchar::KeyBuffer;
use crate::guard::{Depth, Script, Suppress};
use crate::memory::XString;
use crate::optionstr::{OptString, OptStringRef};
use crate::types::{FAIL, MAXPATHL, OK};
use crate::winlayer::{Buf, Win, tab_windows};

/// An empty `SaveRedo`; `save_redobuff` fills it in.
const SAVE_REDO_INIT: SaveRedo = SaveRedo {
    sr_redobuff: KeyBuffer::EMPTY,
    sr_old_redobuff: KeyBuffer::EMPTY,
};

/// The events whose `<afile>` is not a file name, so nothing is set.
fn afile_is_not_a_name(event: AutoEvent) -> bool {
    matches!(
        event,
        AutoEvent::ColorScheme
            | AutoEvent::ColorSchemePre
            | AutoEvent::OptionSet
            | AutoEvent::ModeChanged
            | AutoEvent::MarkSet
    )
}

/// The events whose file-name argument is a name of something else -- a
/// command line, a filetype, a signal -- and must not be expanded to a
/// full path.
fn afile_is_not_expanded(event: AutoEvent) -> bool {
    matches!(
        event,
        AutoEvent::CmdlineChanged
            | AutoEvent::CmdlineEnter
            | AutoEvent::CmdlineLeavePre
            | AutoEvent::CmdlineLeave
            | AutoEvent::CmdUndefined
            | AutoEvent::CursorMovedC
            | AutoEvent::CmdwinEnter
            | AutoEvent::CmdwinLeave
            | AutoEvent::ColorScheme
            | AutoEvent::ColorSchemePre
            | AutoEvent::DirChanged
            | AutoEvent::DirChangedPre
            | AutoEvent::FileType
            | AutoEvent::FuncUndefined
            | AutoEvent::MarkSet
            | AutoEvent::MenuPopup
            | AutoEvent::ModeChanged
            | AutoEvent::OptionSet
            | AutoEvent::Progress
            | AutoEvent::QuickFixCmdPost
            | AutoEvent::QuickFixCmdPre
            | AutoEvent::RemoteReply
            | AutoEvent::Signal
            | AutoEvent::SpellFileMissing
            | AutoEvent::Syntax
            | AutoEvent::TabClosed
            | AutoEvent::User
            | AutoEvent::WinClosed
            | AutoEvent::WinResized
            | AutoEvent::WinScrolled
    )
}

/// The events that do not set or reset the `Changed` flag themselves, so
/// it is put back afterwards.
fn keeps_changed_flag(event: AutoEvent) -> bool {
    matches!(
        event,
        AutoEvent::BufReadPost
            | AutoEvent::BufWritePost
            | AutoEvent::FileAppendPost
            | AutoEvent::VimLeave
            | AutoEvent::VimLeavePre
    )
}

/// Fire `event`, in every group, with no file name: the patterns match the
/// buffer's name, as no `fname` asks of [`apply_autocmds`].
pub fn fire_autocmds(event: AutoEvent, force: bool, buffer: Option<Buf>) -> bool {
    apply_autocmds(event, None, None, force, buffer)
}

/// Fire `event`, in every group, matching the patterns against `pattern`
/// (or the buffer's name, for `None`) with `afile` as `<afile>`, and `data`
/// as the handlers' `data` -- `nvim_exec_autocmds`' `data`, a `Progress`
/// message's fields.
pub(crate) fn fire_autocmds_for(
    event: AutoEvent,
    pattern: Option<&CStr>,
    afile: Option<&CStr>,
    force: bool,
    buffer: Option<Buf>,
    data: Option<&Object>,
) -> bool {
    apply_autocmds_group(
        event,
        pattern,
        afile,
        force,
        AUGROUP_ALL,
        buffer,
        None,
        data,
    )
}

/// Fire `event`, in every group.
///
/// The names are copied before any autocommand runs. A caller hands in its
/// own copy of a name an autocommand could change (a buffer's, an
/// option's): a borrow passed in lives for the whole call, handlers and
/// all.
pub fn apply_autocmds(
    event: AutoEvent,
    fname: Option<&CStr>,
    fname_io: Option<&CStr>,
    force: bool,
    buffer: Option<Buf>,
) -> bool {
    apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        AUGROUP_ALL,
        buffer,
        None,
        None,
    )
}

/// `fire_autocmds_for` passing an `ExArg` on, so that `v:cmdarg` and
/// `v:cmdbang` are set for the handlers: fire `event` in every group,
/// matching the patterns against `pattern` (or the buffer's name, for
/// `None`) with `afile` as `<afile>`.
///
/// Both names are the caller's for the whole call, while the handlers run:
/// a caller naming a buffer hands in a copy of its name.
pub fn apply_autocmds_exarg(
    event: AutoEvent,
    pattern: Option<&CStr>,
    afile: Option<&CStr>,
    force: bool,
    buffer: Option<Buf>,
    excmd: Option<&mut ExArg>,
) -> bool {
    apply_autocmds_group(
        event,
        pattern,
        afile,
        force,
        AUGROUP_ALL,
        buffer,
        excmd,
        None,
    )
}

/// [`apply_autocmds_retval`] with no names: `buffer`'s event, which may turn
/// `retval` into `FAIL`.
pub(crate) fn fire_autocmds_retval(
    event: AutoEvent,
    buffer: Buf,
    retval: &mut ::core::ffi::c_int,
) -> bool {
    apply_autocmds_retval(event, None, None, false, buffer, retval)
}

/// [`apply_autocmds`] threaded through a caller's `OK`/`FAIL`: it does
/// nothing once that says to abort, and turns it to `FAIL` if a handler
/// aborted.
pub fn apply_autocmds_retval(
    event: AutoEvent,
    fname: Option<&CStr>,
    fname_io: Option<&CStr>,
    force: bool,
    buffer: Buf,
    retval: &mut ::core::ffi::c_int,
) -> bool {
    if should_abort(*retval) {
        return false;
    }
    let did_cmd = apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        AUGROUP_ALL,
        Some(buffer),
        None,
        None,
    );
    if did_cmd && aborting() {
        *retval = FAIL;
    }
    did_cmd
}

/// Whether 'eventignorewin' keeps `event` from firing for `buffer`: it is
/// per window, so the question is whether *every* window showing the
/// buffer ignores it. Only window-local events can be listed there.
fn ignored_in_every_window(event: AutoEvent, buffer: Option<Buf>) -> bool {
    if !event_row(event).win_local {
        return false;
    }
    if buffer == Buf::current_or_none() {
        return event_ignored(event, Win::current().w_onebuf_opt.wo_eiw.bytes(), true);
    }
    if !buffer.is_some_and(|b| b.b_nwindows > 0) {
        return false;
    }
    tab_windows()
        .filter(|wp| Some(wp.buffer()) == buffer)
        .all(|wp| event_ignored(event, wp.w_onebuf_opt.wo_eiw.bytes(), true))
}

/// Fire `event` for `buffer`, running every autocommand in `group` whose
/// pattern matches.
///
/// Answers whether any autocommand was executed.  The body is one labeled
/// block, which is upstream's `goto BYPASS_AU`: every reason not to fire
/// leaves it, and the two things that happen either way -- wiping a
/// buffer's own autocommands, and remembering that `FileType` ran -- are
/// after it.
///
/// `fname` and `fname_io` are copied before anything runs; see
/// [`apply_autocmds`] for what a caller hands in. `data` is the caller's,
/// which a Lua handler is given a copy of.
pub fn apply_autocmds_group(
    event: AutoEvent,
    fname: Option<&CStr>,
    fname_io: Option<&CStr>,
    force: bool,
    group: ::core::ffi::c_int,
    buffer: Option<Buf>,
    excmd: Option<&mut ExArg>,
    data: Option<&Object>,
) -> bool {
    let mut retval = false;
    let mut did_save_redobuff = false;
    let mut save_redo = SAVE_REDO_INIT;
    let save_key_typed = KeyTyped.get();

    'bypass: {
        // Nothing to fire, or firing is off.
        if !has_event(event) || is_autocmd_blocked() {
            break 'bypass;
        }
        // While autocommands are running, a new one only fires when it
        // was asked for explicitly (`++nested`).
        if autocmd_busy.get() && !(force || autocmd_nested.get()) {
            break 'bypass;
        }
        // An error, interrupt or uncaught exception is pending.
        if aborting() {
            break 'bypass;
        }
        // FileChangedShell never nests: it would loop forever.
        if filechangeshell_busy.get()
            && (event == AutoEvent::FileChangedShell || event == AutoEvent::FileChangedShellPost)
        {
            break 'bypass;
        }
        if p_ei(|ei| event_ignored(event, ei.to_bytes(), false)) {
            break 'bypass;
        }
        if ignored_in_every_window(event, buffer) {
            break 'bypass;
        }

        // Nesting is allowed but bounded: it is easy to write an
        // endless loop.
        if autocmd_nesting.get() == 10 {
            emsg(gettext(E_AUTOCOMMAND_NESTING_TOO_DEEP));
            break 'bypass;
        }
        // `:all` and `:ball` turn these off while they shuffle windows.
        if (autocmd_no_enter.get() != 0
            && (event == AutoEvent::WinEnter || event == AutoEvent::BufEnter))
            || (autocmd_no_leave.get() != 0
                && (event == AutoEvent::WinLeave || event == AutoEvent::BufLeave))
        {
            break 'bypass;
        }

        // Save the autocmd_* variables and what we know about the
        // current buffer.
        let save_autocmd_fname = autocmd_fname.take();
        let save_autocmd_fname_full = autocmd_fname_full.get();
        let save_autocmd_bufnr = autocmd_bufnr.get();
        let save_autocmd_match = autocmd_match.take();
        let save_autocmd_busy = autocmd_busy.get();
        let save_autocmd_nested = autocmd_nested.get();
        let save_changed = Buf::current().b_changed != 0;
        let old_curbuf = Buf::current_raw();

        // `<afile>`.  A copy, so renaming a buffer or changing
        // directory cannot invalidate it.
        let afile_orig: Option<XString> = if let Some(fname_io) = fname_io {
            Some(XString::from_cstr(fname_io))
        } else if afile_is_not_a_name(event) {
            None
        } else if let Some(fname) =
            fname.filter(|fname| ends_excmd(::core::ffi::c_int::from(cstr::first(fname))) == 0)
        {
            Some(XString::from_cstr(fname))
        } else {
            buffer.and_then(|b| b.name.full().map(XString::from_cstr))
        };
        // The unexpanded `<afile>` is kept for the API's `file` field; the
        // expansion's copy is cut at MAXPATHL, as upstream's is: `eval_vars`
        // resolves the full path in that much room later.
        if let Some(afile) = &afile_orig {
            let mut copy = afile.clone();
            copy.truncate(MAXPATHL as usize);
            autocmd_fname.set(Some(copy));
        }
        autocmd_fname_full.set(false);

        // `<abuf>`.
        autocmd_bufnr.set(buffer.map_or(0, |b| b.handle));

        // The name to match the patterns against.  Always a full path,
        // in case a pattern has `allow_dirs` set.
        let mut sfname: Option<XString> = None;
        let fname: XString = match fname.filter(|fname| !fname.is_empty()) {
            None => match (buffer, event) {
                (None, _) => XString::new(),
                (Some(b), AutoEvent::Syntax) => b.b_p_syn.get(),
                (Some(b), AutoEvent::FileType) => b.b_p_ft.get(),
                (Some(b), _) => {
                    sfname = b.name.short().map(XString::from_cstr);
                    b.name.full().map_or_else(XString::new, XString::from_cstr)
                }
            },
            Some(fname) => {
                sfname = Some(XString::from_cstr(fname));
                if afile_is_not_expanded(event) {
                    // Don't expand it later either.
                    autocmd_fname_full.set(true);
                    XString::from_cstr(fname)
                } else {
                    full_name_save(fname, false)
                }
            }
        };

        // `<amatch>`.
        autocmd_match.set(Some(fname.clone()));

        // Don't redraw while running autocommands.
        let redraw_off = Suppress::redraw();

        // `es_name` and `es_lnum` are filled in by `aucmd_next`.
        estack_push(ETYPE_AUCMD, crate::types::FrameName::None, 0);

        // Restored below, in the run of teardown before the call stack is;
        // the handlers this fires each point it at themselves.
        let sctx = Script::saved();

        let mut wait_time: ProfTime = 0;
        if do_profiling.get() == PROF_YES {
            // Doesn't count for the caller itself.
            wait_time = prof_child_enter();
        }

        // Don't use the caller's function-local variables.
        let call_stack_aside = CallStackAside::new();

        // Only the outermost firing saves the search patterns and the
        // redo buffer.
        if !autocmd_busy.get() {
            save_search_patterns();
            if !ins_compl_active() {
                save_redobuff(&mut save_redo);
                did_save_redobuff = true;
            }
            Buf::current().b_did_filetype = Buf::current().b_keep_filetype;
        }

        // Some commands need to know autocommands are running.
        autocmd_busy.set(true);
        filechangeshell_busy.set(event == AutoEvent::FileChangedShell);
        let nested = Depth::of(autocmd_nesting);

        // Remembered for `did_filetype()` and `:setfiletype`, but only once
        // 'filetype' is set: a FileType for an empty one (`:set ft=`) must
        // not stop a later `:setf` from detecting the real type.
        if event == AutoEvent::FileType && Buf::current().b_p_ft.first_byte() != 0 {
            Buf::current().b_did_filetype = true;
        }

        let tail = fname.len() - path_tail(fname.as_cstr()).count_bytes();

        // The walk's cursor.
        let mut patcmd = AutoPatCmd {
            // `aucmd_next` sets `lastpat` back to `None` when there is
            // nothing left to run.
            lastpat: None,
            auidx: 0,
            // The size is snapshotted so that patterns added by a
            // handler cannot extend this walk into an endless loop.
            ausize: with_event_cmds(event, EventCmds::len),
            afile_orig,
            fname,
            sfname,
            tail,
            group,
            event,
            // `getnextac` fills this in from the autocommand it runs.
            script_ctx: ScriptCtx::NONE,
            arg_bufnr: autocmd_bufnr.get(),
            walk: None,
        };
        aucmd_next(&mut patcmd);

        // Something matched: run the autocommands.
        if patcmd.lastpat.is_some() {
            let bufnr = patcmd.arg_bufnr;
            let walk = active_walk_bufnrs.with_mut(|nrs| {
                nrs.push(bufnr);
                nrs.len() - 1
            });
            patcmd.walk = Some(walk);

            // `v:cmdarg`/`v:cmdbang`, only when a pattern matched.
            let save_cmdbang = get_vim_var_nr(Vv::Cmdbang);
            let bang = excmd.as_deref().map(|command| command.forceit);
            let save_cmdarg = match excmd {
                None => None,
                Some(command) => {
                    let saved = set_cmdarg(Some(command), None);
                    set_vim_var_nr(Vv::Cmdbang, VarNumber::from(bang.unwrap_or(false)));
                    saved
                }
            };
            retval = true;

            // Make the cursor and topline valid.  The outermost firing
            // saves them for `reset_lnums`; a nested one only corrects.
            if autocmd_nesting.get() == 1 {
                check_lnums(true);
            } else {
                check_lnums_nested(true);
            }

            let save_did_emsg = did_emsg.get();
            let save_ex_pressedreturn = get_pressedreturn();

            // `getnextac` is what iterates: it is pulled once per
            // matching autocommand.
            let _ = do_cmdline(
                None,
                Some(&mut LineSource::Autocmd(&mut patcmd, data)),
                DoCmdOpts::NOWAIT | DoCmdOpts::VERBOSE | DoCmdOpts::REPEAT,
            );

            did_emsg.set(did_emsg.get() + save_did_emsg);
            set_pressedreturn(save_ex_pressedreturn);

            if autocmd_nesting.get() == 1 {
                // Restore the cursor and topline unless they changed.
                reset_lnums();
            }

            if bang.is_some() {
                set_cmdarg(None, save_cmdarg);
                set_vim_var_nr(Vv::Cmdbang, save_cmdbang);
            }
            // The walk is over. A nested one has popped its own number
            // already; anything above this one's would be a walk that did
            // not, and is gone with it.
            active_walk_bufnrs.with_mut(|nrs| nrs.truncate(walk));
        }

        drop(redraw_off);
        autocmd_busy.set(save_autocmd_busy);
        filechangeshell_busy.set(false);
        autocmd_nested.set(save_autocmd_nested);
        // The frame takes the name `aucmd_next` left there with it.
        estack_pop();
        autocmd_fname.set(save_autocmd_fname);
        autocmd_fname_full.set(save_autocmd_fname_full);
        autocmd_bufnr.set(save_autocmd_bufnr);
        autocmd_match.set(save_autocmd_match);
        drop(sctx);
        drop(call_stack_aside);
        if do_profiling.get() == PROF_YES {
            prof_child_exit(wait_time);
        }
        KeyTyped.set(save_key_typed);
        drop(patcmd);
        // The handlers below run at the enclosing level, not this one.
        drop(nested);

        // The outermost firing puts the search patterns and the redo
        // buffer back, and frees what the handlers deferred.
        if !autocmd_busy.get() {
            restore_search_patterns();
            if did_save_redobuff {
                restore_redobuff(&mut save_redo);
            }
            Buf::current().b_did_filetype = false;
            free_deferred();
        }

        // Only if we are still in the same buffer.
        if Buf::current_raw() == old_curbuf && keeps_changed_flag(event) {
            if Buf::current().b_changed != ::core::ffi::c_int::from(save_changed) {
                need_maketitle.set(true);
            }
            Buf::current().b_changed = ::core::ffi::c_int::from(save_changed);
        }

        // The patterns and commands marked deleted can really go now.
        au_cleanup();
    }

    // Wiping a buffer takes its buffer-local autocommands with it,
    // whether or not anything fired.
    if let (AutoEvent::BufWipeout, Some(buffer)) = (event, buffer) {
        aubuflocal_remove(buffer);
    }
    if ::core::ffi::c_int::from(retval) == OK && event == AutoEvent::FileType {
        Buf::current().b_au_did_filetype = true;
    }

    retval
}
