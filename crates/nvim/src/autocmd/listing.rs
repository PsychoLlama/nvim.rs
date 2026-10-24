//! Printing autocommands back, and asking whether one exists.
//!
//! [`au_show_for_event`] is `:autocmd`'s listing for one event -- the group
//! header, the pattern column, the command, and the `Last set from` line a
//! `:verbose` listing adds.  [`au_exists`] answers
//! `exists('#Group#Event#pat')` in all four of its shapes, [`has_autocmd`]
//! the pattern query behind it, and [`set_context_in_autocmd`] is
//! command-line completion.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::memory::XString;
use crate::types::{AutoHandler, ExpandContext, PatId, ScriptCtx};
use crate::winlayer::Buf;

/// `:autocmd` with no event: list every event's autocommands.
pub(crate) fn au_show_for_all_events(group: ::core::ffi::c_int, pat: &[u8]) {
    for event in AutoEvent::all() {
        au_show_for_event(group, event, pat);
    }
}

/// One row of a listing, copied out of the table: printing can run Lua,
/// which can define an autocommand, so nothing holds the table across it.
struct ShownRow {
    pat_id: PatId,
    group: ::core::ffi::c_int,
    pattern: XString,
    handler: XString,
    is_command: bool,
    desc: Option<XString>,
    script_ctx: ScriptCtx,
}

/// What row `idx` of `event` holds for a listing restricted to `group`
/// and the pattern `wanted` (every pattern, when empty): `Err` past the
/// last row, `Ok(None)` for a row that is deleted or filtered out.
fn shown_row(
    event: AutoEvent,
    idx: usize,
    group: ::core::ffi::c_int,
    wanted: &[u8],
) -> Result<Option<ShownRow>, ()> {
    with_event_cmds(event, |cmds| {
        let cmd = cmds.cmds.get(idx).ok_or(())?;
        // Skip a row `aucmd_del` has marked.
        let Some(pat_id) = cmd.pat else {
            return Ok(None);
        };
        let pat = cmds.pat(pat_id);
        // Accept the row when the group matches (or none was asked for)
        // and the pattern matches (or none was asked for).
        if group != AUGROUP_ALL && pat.group != group {
            return Ok(None);
        }
        if !wanted.is_empty() && pat.pat[..] != *wanted {
            return Ok(None);
        }
        Ok(Some(ShownRow {
            pat_id,
            group: pat.group,
            pattern: pat.pat.clone(),
            handler: aucmd_handler_to_string(cmd),
            is_command: matches!(cmd.handler, AutoHandler::Cmd(_)),
            desc: cmd.desc.clone(),
            script_ctx: cmd.script_ctx,
        }))
    })
}

/// List the autocommands defined for `event`, restricted to `group` and to
/// the comma-separated pattern list `pat` when either is given.
///
/// `got_int` is checked between every write: a listing is interruptible,
/// and each check is at the point where upstream put it.
pub(crate) fn au_show_for_event(group: ::core::ffi::c_int, event: AutoEvent, pat: &[u8]) {
    if !has_event(event) {
        return;
    }

    // An empty pattern shows every autocommand for the event.
    let (mut start, mut len) = (0, 0);
    if !pat.is_empty() {
        (start, len) = aucmd_span_pattern(pat);
        // Only commas: nothing to show.
        if len == 0 {
            return;
        }
    }

    let mut last_group = AUGROUP_ERROR;

    // One pass per pattern in the list.
    loop {
        let end = start + len;
        // `<buffer[=X]>` is normalised, as it was when it was defined,
        // so the comparison below can be a plain one.
        let wanted = normalized_pattern(&pat[start..end]);
        let mut last_pat: Option<PatId> = None;

        let mut idx = 0;
        // The size is re-read every round.
        while let Ok(row) = shown_row(event, idx, group, &wanted) {
            idx += 1;
            let Some(row) = row else {
                continue;
            };

            // The group and event headline only when the group changed.
            if row.group != last_group {
                last_group = row.group;
                let group_name = augroup_name(row.group);

                if got_int.get() {
                    return;
                }
                msg_putchar('\n' as ::core::ffi::c_int);
                if got_int.get() {
                    return;
                }
                if row.group != AUGROUP_DEFAULT {
                    // A group whose name is gone is one `:augroup!`
                    // renamed out from under its autocommands.
                    match &group_name {
                        None => msg_str_hl(get_deleted_augroup(), HLF_E, false),
                        Some(name) => msg_str_hl(name.as_cstr(), HLF_T, false),
                    }
                    msg_str(c"  ");
                }
                msg_str_hl(event_nr2name(event), HLF_T, false);
            }

            // The pattern only when it changed.
            if last_pat != Some(row.pat_id) {
                last_pat = Some(row.pat_id);
                msg_putchar('\n' as ::core::ffi::c_int);
                if got_int.get() {
                    return;
                }
                msg_advance(4);
                msg_display(row.pattern.as_cstr(), 0, false);
            }

            if got_int.get() {
                return;
            }
            if msg_col.get() >= 14 {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            msg_advance(14);
            if got_int.get() {
                return;
            }

            match &row.desc {
                None => {
                    // A command is transliterated, a callback is not.
                    if row.is_command {
                        msg_display(row.handler.as_cstr(), 0, false);
                    } else {
                        msg_str_hl(row.handler.as_cstr(), HLF_8, false);
                    }
                }
                Some(desc) => {
                    // Upstream's 100-byte scratch buffer, and its cut.
                    let msglen = 100;
                    let shown = if row.is_command {
                        crate::formatted!(msglen, b"%s [%s]", row.handler.as_cstr(), desc.as_cstr())
                    } else {
                        msg_str_hl(row.handler.as_cstr(), HLF_8, false);
                        crate::formatted!(msglen, b" [%s]", desc.as_cstr())
                    };
                    msg_display(XString::from_vec(shown).as_cstr(), 0, false);
                }
            }

            if p_verbose() > 0 {
                last_set_msg(row.script_ctx);
            }
            if got_int.get() {
                return;
            }
        }

        (start, len) = aucmd_span_pattern(&pat[end..]);
        start += end;
        if len == 0 {
            break;
        }
    }
}

/// Whether any autocommand for `event` would match the file `name`
/// opened in `buffer`.
pub fn has_autocmd(event: AutoEvent, name: &::core::ffi::CStr, buffer: Option<Buf>) -> bool {
    let tail = path_tail(name);
    let fname = full_name_save(name, false);
    // The size is re-read every round. A program is taken out of the table
    // for its match and put back after it, so nothing holds the table
    // while the matcher runs (it can poll for input).
    let mut idx = 0;
    while let Some(row) = with_event_cmds_mut(event, |cmds| {
        let cmd = cmds.cmds.get(idx)?;
        let probe = cmd.pat.map(|id| {
            let pat = cmds.pat_mut(id);
            (id, pat.buflocal_nr, pat.reg_prog.take(), pat.allow_dirs)
        });
        Some(probe)
    }) {
        idx += 1;
        let Some((id, buflocal_nr, mut prog, allow_dirs)) = row else {
            continue;
        };
        // A buffer-local pattern is matched by buffer number, every other
        // one against the file name.
        let matched = if buflocal_nr == 0 {
            match_file_pat(&mut prog, fname.as_cstr(), Some(name), tail, allow_dirs)
        } else {
            buffer.is_some_and(|b| buflocal_nr == b.handle)
        };
        with_event_cmds_mut(event, |cmds| cmds.pat_mut(id).reg_prog = prog);
        if matched {
            return true;
        }
    }
    false
}

/// Command-line completion for `:autocmd` (`doautocmd` false) and
/// `:doautocmd`/`:doautoall` (true).
///
/// The argument starts at `arg` in the completion's line. Answers where the
/// next command to expand starts instead, or `None` when it has set
/// `expand` itself.
pub fn set_context_in_autocmd(expand: &mut Expand, arg: usize, doautocmd: bool) -> Option<usize> {
    let line = expand.line.as_cstr().to_bytes();
    // The byte at `at`, a NUL past the end, as the C walk reads it.
    let at = |at: usize| line.get(at).copied().unwrap_or(0);
    let white = |c: u8| ascii_iswhite(::core::ffi::c_int::from(c));

    // Skip a group name if there is one.
    autocmd_include_groups.set(false);
    let start = arg;
    let (mut group, skipped) = arg_augroup_get(&line[arg.min(line.len())..]);
    let mut arg = arg + skipped;

    // A group name and nothing else is what is being completed, unless
    // it was already followed by a space. `arg - 1` is only reached when a
    // group name was skipped, so it is that name's last byte or a blank.
    if at(arg) == 0 && group != AUGROUP_ALL && !white(at(arg - 1)) {
        arg = start;
        group = AUGROUP_ALL;
    }

    // Skip over the event name, keeping the start of the last one in a
    // comma-separated list.
    let mut p = arg;
    while at(p) != 0 && !white(at(p)) {
        if at(p) == b',' {
            arg = p + 1;
        }
        p += 1;
    }
    if at(p) == 0 {
        if group == AUGROUP_ALL {
            autocmd_include_groups.set(true);
        }
        expand.context = ExpandContext::Events;
        expand.pattern = arg;
        return None;
    }

    // Skip over the pattern, whose whitespace may be backslash-escaped.
    arg = p + crate::charset::skip::white(&line[p..]);
    while at(arg) != 0 && (!white(at(arg)) || at(arg - 1) == b'\\') {
        arg += 1;
    }
    if at(arg) != 0 {
        // What follows is the command, which the caller expands.
        return Some(arg);
    }

    expand.context = if doautocmd {
        ExpandContext::Files
    } else {
        ExpandContext::Nothing
    };
    None
}

/// `exists('#…')`, in all four shapes: `#Group`, `#Event`, `#Event#pat`
/// and `#Group#Event#pat`.
pub fn au_exists(arg: &::core::ffi::CStr) -> bool {
    /// `text` cut at its first `#`: what is before it, and what is after it
    /// when there is one.
    fn split_hash(text: &[u8]) -> (&[u8], Option<&[u8]>) {
        match text.iter().position(|&c| c == b'#') {
            Some(at) => (&text[..at], Some(&text[at + 1..])),
            None => (text, None),
        }
    }

    let (first, rest) = split_hash(arg.to_bytes());
    // The first field is a group name if it names one, and an event
    // otherwise.
    let mut group = augroup_find(first);
    let (event_name, pattern) = if group == AUGROUP_ERROR {
        group = AUGROUP_ALL;
        (first, rest)
    } else if let Some(rest) = rest {
        // "Group#Event" or "Group#Event#pat".
        split_hash(rest)
    } else {
        // Just "Group", and it exists.
        return true;
    };

    let (Some(event), _) = event_name2nr(event_name) else {
        return false;
    };

    // `<buffer>` means curbuf; `<buffer=N>` is already normalised, so
    // `path_fnamecmp` handles it.
    let buflocal = pattern.is_some_and(|pattern| pattern.eq_ignore_ascii_case(b"<buffer>"));
    let current = if buflocal { Buf::current().handle } else { 0 };
    with_event_cmds(event, |cmds| {
        cmds.live().any(|(_, pat)| {
            (group == AUGROUP_ALL || pat.group == group)
                && pattern.is_none_or(|pattern| {
                    if buflocal {
                        pat.buflocal_nr == current
                    } else {
                        path_fnamecmp(&pat.pat, pattern) == 0
                    }
                })
        })
    })
}
