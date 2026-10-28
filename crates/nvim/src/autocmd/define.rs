//! `:autocmd` itself -- parsing it, and installing what it says.
//!
//! [`do_autocmd`] splits the command into group, event list, pattern and
//! command, then loops [`do_autocmd_event`] over the events;
//! [`autocmd_register`] is the one place an `AutoCmd` (and, when it is new,
//! its `AutoPat`) is created, and the same entry point the API's `nvim_create_autocmd`
//! reaches.  The `arg_*` helpers are the pieces of the parse the API
//! shares, and [`autocmd_delete_id`] is the deletion by id that
//! `nvim_del_autocmd` is.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::charset::skip;
use crate::cstr;
use crate::lua::executor::nlua_set_sctx;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::os::env::expand::expand_env_save_opt_of;
use crate::semsg;
use crate::types::{AutoHandler, Failed};
use crate::winlayer::TabPage;
use crate::winlayer::{Win, tabs};

/// Whether `c` is a blank, as the parse below reads one.
fn white(c: u8) -> bool {
    ascii_iswhite(::core::ffi::c_int::from(c))
}

/// `:autocmd [group] {event} {pat} [++once] [++nested] {cmd}`, and every
/// shorter spelling of it: listing, deleting, and `:autocmd *`.
///
/// The argument is walked by offset; the next command after a `|` is
/// recorded as an offset into the line.
pub fn do_autocmd(excmd: &mut ExArg, forceit: ::core::ffi::c_int) {
    let arg_start = excmd.line.arg;
    let line = excmd.line.arg().to_vec();
    // The byte at `at`, a NUL past the end, as the C walk read it.
    let at = |at: usize| line.get(at).copied().unwrap_or(0);
    let mut nested = false;
    let mut once = false;

    // `arg` is where the event list starts; the end of the line reads as the
    // empty string.
    let (group, mut arg) = if at(0) == b'|' {
        excmd.line.next = Some(arg_start + 1);
        (AUGROUP_ALL, line.len())
    } else {
        arg_augroup_get(&line)
    };

    // Validate every event name before doing anything.
    let Some(events_len) = arg_event_skip(&line[arg..], group != AUGROUP_ALL) else {
        return;
    };
    let mut pat = arg + events_len;
    pat += skip::white(&line[pat..]);

    let pattern: XString;
    let cmd: Vec<u8>;
    if at(pat) == b'|' {
        excmd.line.next = Some(arg_start + pat + 1);
        pattern = XString::new();
        cmd = Vec::new();
    } else {
        // Scan over the pattern, whose whitespace may be backslash-escaped.
        let mut end = pat;
        while at(end) != 0 && (!white(at(end)) || end.checked_sub(1).map_or(0, at) == b'\\') {
            end += 1;
        }
        let raw = &line[pat..end];
        let mut cmd_at = if at(end) != 0 { end + 1 } else { end };

        pattern = if raw.contains(&b'$') || raw.contains(&b'~') {
            expand_env_save_opt_of(XString::from_bytes(raw).as_cstr(), false)
        } else {
            XString::from_bytes(raw)
        };

        cmd_at += skip::white(&line[cmd_at.min(line.len())..]);

        // Two passes, so the flags may be given in either order.
        let mut invalid_flags = false;
        for _ in 0..2 {
            if at(cmd_at) == 0 {
                continue;
            }
            invalid_flags |= arg_autocmd_flag_get(&mut once, &line, &mut cmd_at, b"++once");
            invalid_flags |= arg_autocmd_flag_get(&mut nested, &line, &mut cmd_at, b"++nested");
            // The deprecated spelling of `++nested`.
            invalid_flags |= arg_autocmd_flag_get(&mut nested, &line, &mut cmd_at, b"nested");
        }
        if invalid_flags {
            return;
        }

        cmd = if at(cmd_at) != 0 {
            let Some(expanded) = expand_sfile(cstr::until_nul(&line[cmd_at..])) else {
                return;
            };
            expanded
        } else {
            Vec::new()
        };
    }
    let cmd = cstr::until_nul(&cmd);
    let pattern = &pattern[..];

    // No command and no `!`: this is a listing.
    let is_showing = forceit == 0 && cmd.is_empty();
    let all_events = matches!(at(arg), b'*' | b'|' | 0);

    if is_showing {
        msg_ext_set_kind(c"list_cmd");
        msg_title(gettext(c"\n--- Autocommands ---"));

        if all_events {
            au_show_for_all_events(group, pattern);
        } else {
            // `arg_event_skip` has already refused an unknown name.
            let (event, _) = event_name2nr(&line[arg..]);
            let event = event.expect("the event list was checked before it was walked");
            au_show_for_event(group, event, pattern);
        }
    } else if all_events {
        if !cmd.is_empty() {
            emsg(gettext(e_cannot_define_autocommands_for_all_events));
        } else {
            do_all_autocmd_events(pattern, once, nested, cmd, forceit != 0, group);
        }
    } else {
        while at(arg) != 0 && at(arg) != b'|' && !white(at(arg)) {
            // `arg_event_skip` has already refused an unknown name.
            let (event, consumed) = event_name2nr(&line[arg..]);
            arg += consumed;
            let event = event.expect("the event list was checked before it was walked");
            if do_autocmd_event(event, pattern, once, nested, cmd, forceit != 0, group).is_err() {
                break;
            }
        }
    }
}

/// [`do_autocmd_event`] for every event: what `:autocmd *` means.
pub fn do_all_autocmd_events(
    pat: &[u8],
    once: bool,
    nested: bool,
    cmd: &[u8],
    del: bool,
    group: ::core::ffi::c_int,
) {
    for event in AutoEvent::all() {
        if do_autocmd_event(event, pat, once, nested, cmd, del, group).is_err() {
            return;
        }
    }
}

/// `:autocmd` for one event, over each of the comma-separated patterns.
///
/// An empty `cmd` with `del` deletes; a non-empty one adds.  Both together
/// are `:autocmd! {event} {pat} {cmd}`, which deletes the existing
/// autocommands on the pattern and then appends a fresh one.
pub fn do_autocmd_event(
    event: AutoEvent,
    pat: &[u8],
    once: bool,
    nested: bool,
    cmd: &[u8],
    del: bool,
    group: ::core::ffi::c_int,
) -> Result<(), Failed> {
    // Listing all patterns goes through `au_show_for_event` instead.
    debug_assert!(!pat.is_empty() || del);

    let is_adding_cmd = !cmd.is_empty();
    let findgroup = if group == AUGROUP_ALL {
        current_augroup.get()
    } else {
        group
    };

    // `:autocmd! {event}`: every pattern goes.
    if pat.is_empty() && del {
        aucmd_del_for_event_and_group(event, findgroup);
        return Ok(());
    }

    let (mut start, mut len) = aucmd_span_pattern(pat);
    while len != 0 {
        let end = start + len;
        // `<buffer[=X]>` is normalised on the way in, so every later
        // comparison against a stored pattern is a plain one.
        let item = normalized_pattern(&pat[start..end]);

        if del {
            // Only the commands go; a pattern a new command is about to be
            // appended with is made again below.
            aucmd_del_where(event, |stored| {
                stored.group == findgroup && stored.pat[..] == item[..]
            });
        }

        if is_adding_cmd {
            let _ = autocmd_register(
                0,
                event,
                &item,
                group,
                once,
                nested,
                None,
                Some(cmd),
                &Callback::None,
            );
        }

        (start, len) = aucmd_span_pattern(&pat[end..]);
        start += end;
    }

    // The patterns and commands marked above can really go now.
    au_cleanup();
    Ok(())
}

/// Create one autocommand: the only place one is made, reached from both
/// `:autocmd` and `nvim_create_autocmd`.
///
/// The handler is `handler_cmd` when there is one, and a counted copy of
/// `handler_fn` otherwise. A pattern holding a NUL is refused, as a C
/// string shorter than its count was.
pub fn autocmd_register(
    id: int64_t,
    event: AutoEvent,
    pat: &[u8],
    group: ::core::ffi::c_int,
    once: bool,
    nested: bool,
    desc: Option<&[u8]>,
    handler_cmd: Option<&[u8]>,
    handler_fn: &Callback,
) -> Result<(), Failed> {
    // 0 is not a valid group.
    debug_assert!(group != 0);

    if pat.contains(&0) {
        return Err(Failed);
    }

    let findgroup = if group == AUGROUP_ALL {
        current_augroup.get()
    } else {
        group
    };

    let is_buflocal = aupat_is_buflocal(pat);
    let buflocal_nr = if is_buflocal {
        aupat_get_buflocal_nr(pat)
    } else {
        0
    };
    let pat = normalized_pattern(pat);
    let pat = &pat[..];

    // Reuse the pattern of the last live autocommand when it is the
    // same one, so `:autocmd` twice on a pattern compiles one regexp.
    let reused = with_event_cmds(event, |cmds| {
        cmds.cmds
            .iter()
            .rev()
            .find_map(|cmd| cmd.pat)
            .filter(|&id| {
                let stored = cmds.pat(id);
                stored.group == findgroup && stored.pat[..] == *pat
            })
    });

    let pat_id = match reused {
        Some(id) => id,
        None => {
            // A buffer-local pattern needs a buffer that exists.
            if is_buflocal && (buflocal_nr == 0 || find_buf(buflocal_nr).is_none()) {
                semsg!("E680: <buffer={}>: invalid buffer number ", buflocal_nr);
                return Err(Failed);
            }

            let mut new_pat = AutoPat {
                refcount: 0,
                pat: XString::from_bytes(pat),
                reg_prog: None,
                group: 0,
                buflocal_nr,
                allow_dirs: false,
            };
            if !is_buflocal {
                let reg_pat = file_pat_to_reg_pat(pat, pat.len());
                if let Some((reg_pat, allow_dirs)) = &reg_pat {
                    new_pat.allow_dirs = *allow_dirs;
                    new_pat.reg_prog = vim_regcomp(reg_pat.as_cstr(), RE_MAGIC);
                }
                if reg_pat.is_none() || new_pat.reg_prog.is_none() {
                    return Err(Failed);
                }
            }

            // The events below compare against state sampled the last time
            // they fired.  The *first* autocommand for one has to seed that
            // state, or it fires immediately on a difference that predates
            // it.
            seed_first_autocmd(event);

            // Spelled out rather than reusing `findgroup`: they are the
            // same value (nothing above can change `current_augroup`), but
            // upstream asks the question twice.
            new_pat.group = if group == AUGROUP_ALL {
                current_augroup.get()
            } else {
                group
            };
            with_event_cmds_mut(event, |cmds| cmds.add_pat(new_pat))
        }
    };

    let handler = match handler_cmd {
        Some(cmd) => AutoHandler::Cmd(XString::from_bytes(cstr::until_nul(cmd))),
        None => AutoHandler::Fn(handler_fn.duplicate()),
    };
    let mut script_ctx = current_sctx.get();
    // `SOURCING_LNUM`: the line of the innermost execution-stack frame.
    script_ctx.sc_lnum += crate::runtime::sourcing_lnum();
    nlua_set_sctx(&mut script_ctx);
    let cmd = AutoCmd {
        pat: Some(pat_id),
        id,
        desc: desc.map(|desc| XString::from_bytes(cstr::until_nul(desc))),
        handler,
        script_ctx,
        once,
        nested,
    };
    with_event_cmds_mut(event, |cmds| {
        cmds.pat_mut(pat_id).refcount += 1;
        cmds.cmds.push(cmd);
    });

    Ok(())
}

/// Seed what the first autocommand for `event` compares against.
fn seed_first_autocmd(event: AutoEvent) {
    if event == AutoEvent::ModeChanged && !has_event(AutoEvent::ModeChanged) {
        last_mode.set(get_mode());
    }
    if (event == AutoEvent::CursorMoved && !has_event(AutoEvent::CursorMoved))
        || (event == AutoEvent::CursorMovedI && !has_event(AutoEvent::CursorMovedI))
    {
        last_cursormoved_win.set(Win::current_or_none().map(Win::id));
        last_cursormoved.set(Win::current().w_cursor);
    }
    if (event == AutoEvent::WinScrolled || event == AutoEvent::WinResized)
        && !(has_event(AutoEvent::WinScrolled) || has_event(AutoEvent::WinResized))
    {
        let save_curtab = TabPage::current();
        for tp in tabs() {
            unuse_tabpage(TabPage::current());
            use_tabpage(tp);
            snapshot_windows_scroll_size();
        }
        unuse_tabpage(TabPage::current());
        use_tabpage(save_curtab);
    }
}

/// Where the first pattern in a comma-separated list starts and how long
/// it is.
///
/// Leading commas are skipped, and a comma inside braces or after a
/// backslash (`*.\{obj,o\}`) does not end a pattern.
pub fn aucmd_span_pattern(pat: &[u8]) -> (usize, usize) {
    let start = pat.iter().take_while(|&&c| c == b',').count();
    let mut p = start;
    let mut brace_level = 0i32;
    while p < pat.len()
        && pat[p] != 0
        && (pat[p] != b',' || brace_level != 0 || (p > start && pat[p - 1] == b'\\'))
    {
        if pat[p] == b'{' {
            brace_level += 1;
        } else if pat[p] == b'}' {
            brace_level -= 1;
        }
        p += 1;
    }
    (start, p - start)
}

/// Whether `do_modelines` should be called, and how far past a leading
/// `<nomodeline>` to step. False means the argument began with it.
pub fn check_nomodeline(arg: &[u8]) -> (bool, usize) {
    match arg.strip_prefix(b"<nomodeline>") {
        Some(rest) => (false, 12 + skip::white(rest)),
        None => (true, 0),
    }
}

/// Delete the autocommand with this id, wherever it is.
///
/// Only autocommands created through the API have one.
pub fn autocmd_delete_id(id: int64_t) -> bool {
    debug_assert!(id > 0);

    let mut success = false;
    for event in AutoEvent::all() {
        with_event_cmds_mut(event, |cmds| {
            for idx in 0..cmds.len() {
                if cmds.cmds[idx].id == id {
                    cmds.delete(idx);
                    success = true;
                }
            }
        });
    }
    if success {
        au_need_clean.set(true);
    }
    success
}

/// An autocommand's handler as a string, whichever kind it is.
pub fn aucmd_handler_to_string(ac: &AutoCmd) -> XString {
    match &ac.handler {
        AutoHandler::Cmd(cmd) => cmd.clone(),
        AutoHandler::Fn(callback) => XString::from_cstr(callback_to_string(callback).as_cstr()),
    }
}

/// How far past `:autocmd`'s comma-separated event list at the head of
/// `arg` what follows starts -- or `None`, having raised an error, when a
/// name is not an event.
///
/// `have_group` only picks the wording: without a group, the leading word
/// could have been meant as one.
pub(crate) fn arg_event_skip(arg: &[u8], have_group: bool) -> Option<usize> {
    let arg = cstr::until_nul(arg);
    let at = |at: usize| arg.get(at).copied().unwrap_or(0);
    if at(0) == b'*' {
        if at(1) != 0 && !white(at(1)) {
            let arg = msg_bytes(arg);
            semsg!("E215: Illegal character after *: {arg}");
            return None;
        }
        return Some(1);
    }

    let mut pat = 0;
    while at(pat) != 0 && at(pat) != b'|' && !white(at(pat)) {
        let (event, consumed) = event_name2nr(&arg[pat..]);
        if event.is_none() {
            let shown = msg_bytes(&arg[pat..]);
            if have_group {
                semsg!("E216: No such event: {shown}");
            } else {
                semsg!("E216: No such group or event: {shown}");
            }
            return None;
        }
        pat += consumed;
    }
    Some(pat)
}

/// Take a leading `++once`/`++nested`/`nested` off the command at `*cmd`
/// in `line`, setting `*flag`.
///
/// Answers *true* on the error case -- the flag given twice -- so the
/// caller can `|=` the three calls together.
fn arg_autocmd_flag_get(flag: &mut bool, line: &[u8], cmd: &mut usize, pattern: &[u8]) -> bool {
    let rest = &line[(*cmd).min(line.len())..];
    if rest.starts_with(pattern) && white(rest.get(pattern.len()).copied().unwrap_or(0)) {
        if *flag {
            let pattern = msg_bytes(pattern);
            semsg!("E983: Duplicate argument: {pattern}");
            return true;
        }
        *flag = true;
        *cmd += pattern.len();
        *cmd += skip::white(&line[*cmd..]);
    }
    false
}
