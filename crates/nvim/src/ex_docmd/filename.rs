//! What a command's file argument stands for: `%`, `#`, the `<…>` family,
//! wildcards, and the backtick form.
//!
//! Expansion rewrites the command line in place: every replacement is
//! spliced into the `ExArg`'s own line by [`repl_cmdline`], which moves the
//! cursors past it.
#![forbid(unsafe_code)]
use crate::cmdexpand::{WildMode, WildOpts, expand_one};
use crate::cstr;
use crate::ex_cmds::newlnum;
use crate::optionstr::{OptString, local_or_global};
use crate::types::{CmdIdx, EcmdCmd};
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use crate::arglist::arg_all_bytes;
use crate::autocmd::state::{autocmd_bufnr, autocmd_fname, autocmd_fname_full, autocmd_match};
use crate::buffer::find_buf;
use crate::charset::{backslash_halved, getdigits_int_at};
use crate::eval::fs::modify_fname;
use crate::eval::skip_expr;
use crate::eval::typval::{NumBuf, list_find_str};
use crate::eval::vars::get_vim_var_list_handle;
use crate::ex_docmd::scan::skip_grep_pat;
use crate::ex_docmd::sourcing_entry;
use crate::ex_docmd::state::ESCAPE_CHARS;
use crate::ex_docmd::{
    ESTACK_SCRIPT, ESTACK_SFILE, ESTACK_STACK, FIND_EVAL, FIND_IDENT, FIND_STRING, VALID_HEAD,
    VALID_PATH, e_no_autocommand_buffer_number_to_substitute_for_abuf,
    e_no_autocommand_file_name_to_substitute_for_afile,
    e_no_autocommand_match_name_to_substitute_for_amatch, e_no_call_stack_to_substitute_for_stack,
    e_no_line_number_to_use_for_sflnum, e_no_line_number_to_use_for_slnum,
    e_no_script_file_name_to_substitute_for_script, e_no_source_file_name_to_substitute_for_sfile,
};
use crate::file_search::{FileNameOpts, file_name_at_cursor_owned};
use crate::message::{e_usingsid, emsg, msg_make};
use crate::normal::ident_under_cursor;
use crate::option::vars::{P_GP, P_MP, p_wic};
use crate::os::cshim::gettext;
use crate::os::env::{expand_env_esc_of, expand_env_save_opt_of};
use crate::path::{full_name_save, has_wildcard, path_try_shorten_fname, tail_index};
use crate::quickfix::grep_internal;
use crate::runtime::estack_sfile_owned;
use crate::runtime::state::current_sctx;
use crate::strings::{escaped_bytes, replaced_all};
use crate::types::{CmdLine, ExArg, ExArgt, Expand, ExpandContext, Failed, LineNr, MAXPATHL, Vv};
use crate::winlayer::Buf;

/// `:make` and `:grep` are 'makeprg'/'grepprg' with `$*` replaced by the
/// argument — spliced in here, before `%` and `#` are expanded, so that
/// the program string can use them too.
///
/// Answers whether the line was replaced. It is replaced *whole*: what the
/// user typed is gone and the program string is the command line, with
/// `arg` at its start. The cursors into the old line -- `cmd`, and the
/// API's argument vector -- mean nothing afterwards, which is why the
/// caller drops them.
pub fn replace_makeprg(excmd: &mut ExArg) -> bool {
    let idx = excmd.cmdidx;
    let is_grep = idx == CmdIdx::grep
        || idx == CmdIdx::lgrep
        || idx == CmdIdx::grepadd
        || idx == CmdIdx::lgrepadd;
    let is_make = idx == CmdIdx::make || idx == CmdIdx::lmake;
    // `grep_internal` means 'grepprg' is `internal`, which is not a
    // program at all.
    if !(is_make || is_grep) || grep_internal(excmd.cmdidx) {
        return false;
    }

    let buf = Buf::current();
    let (local, global) = if is_grep {
        (&buf.b_p_gp, P_GP)
    } else {
        (&buf.b_p_mp, P_MP)
    };
    let program = local_or_global(local, global).get();

    excmd.line.arg = excmd.line.skip_white(excmd.line.arg);
    let arg = excmd.line.arg;
    let replaced = replaced_all(&program, b"$*", excmd.line.rest_of(arg));
    let mut line: Vec<u8> = if let Some(replaced) = replaced {
        replaced.to_vec()
    } else {
        // No `$*`: the argument goes on the end.
        let mut line = Vec::with_capacity(program.len() + excmd.line.rest_of(arg).len() + 2);
        line.extend_from_slice(&program);
        line.push(b' ');
        line.extend_from_slice(excmd.line.rest_of(arg));
        line
    };
    line.push(0);

    msg_make(excmd.line.cstr_from(arg));
    excmd.line = CmdLine::from_vec(line);
    true
}

/// Expand every `%`, `#`, `` `cmd` `` and `<…>` in a command's file
/// argument, then expand wildcards if the command takes exactly one name.
///
/// Every cursor into the line is re-read from the `CmdLine` after each
/// replacement: an expansion *grows the buffer*, so an offset held across
/// one is stale.
pub(crate) fn expand_filename(
    excmd: &mut ExArg,
    errormsgp: &mut Option<CString>,
) -> Result<(), Failed> {
    // A `:vimgrep` pattern is not a file name, so the scan starts after
    // it.
    let mut at = skip_grep_pat(excmd);
    let mut has_wildcards = has_wildcard(excmd.line.rest_of(at));

    while excmd.line.byte_at(at) != 0 {
        if excmd.line.byte_at(at) == b'`' && excmd.line.byte_at(at + 1) == b'=' {
            // `` `=expr` `` is evaluated much later, by the shell
            // expansion; step over it without touching it.
            at += 2 + skip_expr(excmd.line.rest_of(at + 2));
            if excmd.line.byte_at(at) == b'`' {
                at += 1;
            }
            continue;
        }
        if !b"%#<".contains(&excmd.line.byte_at(at)) {
            at += 1;
            continue;
        }

        // The items are read relative to the argument: a backslash before
        // one quotes it, but only from inside the argument.
        let base = excmd.line.arg;
        let found = eval_vars(
            excmd.line.cstr_from(base),
            at - base,
            Some(&mut excmd.do_ecmd_lnum),
            true,
        );
        let (mut repl, srclen, escaped) = match found {
            Expansion::Refused(msg) => {
                *errormsgp = Some(msg.to_owned());
                return Err(Failed);
            }
            Expansion::Skip(len) => {
                at += len;
                continue;
            }
            Expansion::Quoted => {
                // Take the backslash out; the item stays, and the scan
                // resumes on the byte after its first.
                excmd.line.drop_byte(at - 1);
                continue;
            }
            Expansion::Text {
                text,
                used,
                escaped,
            } => (text, used, escaped),
        };

        if repl.contains(&b'$') || repl.contains(&b'~') {
            repl = expand_env_save_opt_of(&cstr::owned(&repl), false).to_vec();
        }

        // A name that will be handed to a *file* argument gets the
        // shell-special characters escaped — but not for the commands
        // that hand the whole argument to a shell themselves, and not
        // for a name that came back already escaped (`##`).
        let idx = excmd.cmdidx;
        let escape_chars = ESCAPE_CHARS.to_bytes();
        if !excmd.usefilter
            && !escaped
            && idx != CmdIdx::bang
            && idx != CmdIdx::grep
            && idx != CmdIdx::grepadd
            && idx != CmdIdx::lgrep
            && idx != CmdIdx::lgrepadd
            && idx != CmdIdx::lmake
            && idx != CmdIdx::make
            && idx != CmdIdx::terminal
            && !excmd.argt.has(ExArgt::NOSPC)
            && repl.iter().any(|byte| escape_chars.contains(byte))
        {
            repl = escaped_bytes(&repl, escape_chars).to_vec();
        }
        // A `!` in the replacement would be read as "the previous
        // command" by the shell-command line parser.
        if (excmd.usefilter || idx == CmdIdx::bang || idx == CmdIdx::terminal)
            && repl.contains(&b'!')
        {
            repl = escaped_bytes(&repl, b"!").to_vec();
        }

        at = repl_cmdline(excmd, at, srclen, &repl);
    }

    // `ExArgt::NOSPC` means the argument is one file name, so wildcards in
    // it can be expanded to exactly one match.
    if !excmd.argt.has(ExArgt::NOSPC) || excmd.usefilter {
        return Ok(());
    }

    let (start, len) = (excmd.line.arg, excmd.line.rest_of(excmd.line.arg).len());
    if has_wildcards {
        // Environment variables first: they may hold the wildcards, or
        // may be all that looked like one.
        let arg = excmd.line.rest_of(start);
        if arg.contains(&b'$') || arg.contains(&b'~') {
            let out = expand_env_esc_of(excmd.line.cstr_from(start), true, true, None);
            has_wildcards = has_wildcard(&out);
            repl_cmdline(excmd, start, len, &out);
        }
    }
    if !has_wildcards {
        // The backslashes go, in place: the argument only gets shorter.
        let halved = backslash_halved(excmd.line.rest_of(excmd.line.arg));
        let at = excmd.line.arg;
        let text = excmd.line.buffer_mut();
        text[at..at + halved.len()].copy_from_slice(&halved);
        text[at + halved.len()] = 0;
        return Ok(());
    }

    let mut xpc = Expand::new();
    xpc.context = ExpandContext::Files;
    let mut options = WildOpts::LIST_NOTFOUND | WildOpts::NOERROR | WildOpts::ADD_SLASH;
    if p_wic() {
        options |= WildOpts::ICASE;
    }
    let arg = CString::new(excmd.line.rest_of(excmd.line.arg)).unwrap_or_default();
    let Some(expanded) = expand_one(&mut xpc, Some(&arg), None, options, WildMode::ExpandFree)
    else {
        return Err(Failed);
    };
    let text = expanded.as_cstr().to_bytes().to_vec();
    let (start, len) = (excmd.line.arg, excmd.line.rest_of(excmd.line.arg).len());
    repl_cmdline(excmd, start, len, &text);
    Ok(())
}

/// Replace the `srclen` bytes at `at` with `repl`, in place.
///
/// Everything in the `ExArg` that addresses the line past the replacement
/// moves with it: `cmd`, `arg`, `nextcmd`, the API's argument vector and
/// `do_ecmd_cmd`. Answers where the text after the replacement now starts,
/// which is where the caller's scan resumes.
pub(crate) fn repl_cmdline(excmd: &mut ExArg, at: usize, srclen: usize, repl: &[u8]) -> usize {
    let delta = excmd.line.splice(at, srclen, repl);

    // The `+cmd` argument is the one cursor `CmdLine::splice` does not know
    // about; a bare `+` is the shared `$`, which is not in the line at all.
    if let EcmdCmd::At(off) = excmd.do_ecmd_cmd
        && off > at
    {
        excmd.do_ecmd_cmd = EcmdCmd::At(off.wrapping_add_signed(delta));
    }
    at + repl.len()
}

/// The `%`, `#` and `<…>` items, in the order `find_cmdline_var` answers
/// them. The index *is* the answer, so the order is load-bearing.
const SPECIALS: [&CStr; 15] = [
    c"%",        // the current file
    c"#",        // the alternate file, or `#99`
    c"<cword>",  // the word under the cursor
    c"<cWORD>",  // the WORD under the cursor
    c"<cexpr>",  // the expression under the cursor
    c"<cfile>",  // the path name under the cursor
    c"<sfile>",  // the `:source`d file's name
    c"<slnum>",  // its line number
    c"<stack>",  // the call stack
    c"<script>", // the script file's name
    c"<afile>",  // the autocommand's file name
    c"<abuf>",   // its buffer number
    c"<amatch>", // what its pattern matched
    c"<sflnum>", // the script file's line number
    c"<SID>",    // the script ID, as `<SNR>123_`
];

const SPEC_PERC: usize = 0;
const SPEC_HASH: usize = 1;
const SPEC_CWORD: usize = 2;
const SPEC_CCWORD: usize = 3;
const SPEC_CEXPR: usize = 4;
const SPEC_CFILE: usize = 5;
const SPEC_SFILE: usize = 6;
const SPEC_SLNUM: usize = 7;
const SPEC_STACK: usize = 8;
const SPEC_SCRIPT: usize = 9;
const SPEC_AFILE: usize = 10;
const SPEC_ABUF: usize = 11;
const SPEC_AMATCH: usize = 12;
const SPEC_SFLNUM: usize = 13;
const SPEC_SID: usize = 14;

/// The special item `text` starts with, as its index in `SPECIALS` and its
/// length; the slice form of [`find_cmdline_var`].
pub(crate) fn cmdline_var_at(text: &[u8]) -> Option<(usize, usize)> {
    SPECIALS.iter().enumerate().find_map(|(i, spec)| {
        let spec = spec.to_bytes();
        text.starts_with(spec).then_some((i, spec.len()))
    })
}

/// What [`eval_vars`] found at an offset.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Expansion {
    /// Nothing to substitute: the caller steps over this many bytes.
    Skip(usize),
    /// A backslash before the item quotes it: the caller deletes the
    /// backslash, and the item stays as it is.
    Quoted,
    /// What the item stands for, and how many bytes of the source it
    /// replaces, its modifiers included. `escaped` is `##`, whose names come
    /// back escaped already.
    Text {
        text: Vec<u8>,
        used: usize,
        escaped: bool,
    },
    /// The expansion failed; the message is empty when there is nothing to
    /// say (a `<cword>` with no word, say), and the caller stops.
    Refused(&'static CStr),
}

/// The empty message an expansion that failed quietly answers.
const QUIET: &CStr = c"";

/// Expand the special item at `at` in `src` (`%`, `#`, `<cword>`, ...),
/// with any `:p:h`-style modifiers after it.
///
/// `src` is the whole string the item is read from: the byte before `at`
/// is what says a backslash quotes it. `lnum` is set to `newlnum::LAST` by
/// `#`, which means "the alternate file's last position". With
/// `empty_is_error`, an expansion that produced nothing is
/// [`Expansion::Refused`] rather than [`Expansion::Skip`].
pub(crate) fn eval_vars(
    src: &CStr,
    at: usize,
    lnum: Option<&mut LineNr>,
    empty_is_error: bool,
) -> Expansion {
    let whole = src.to_bytes();
    let text = &whole[at..];
    // The item and what follows it, as `modify_fname` reads its modifiers.
    let mods = CStr::from_bytes_until_nul(&src.to_bytes_with_nul()[at..])
        .expect("a tail of a terminated string");
    let byte = |i: usize| text.get(i).copied().unwrap_or(0);

    let Some((spec_idx, mut used)) = cmdline_var_at(text) else {
        return Expansion::Skip(1);
    };
    // A backslash before it means "take it literally".
    if at > 0 && whole[at - 1] == b'\\' {
        return Expansion::Quoted;
    }

    if matches!(spec_idx, SPEC_CWORD | SPEC_CCWORD | SPEC_CEXPR) {
        let what = match spec_idx {
            SPEC_CWORD => FIND_IDENT as c_int | FIND_STRING as c_int,
            SPEC_CEXPR => FIND_IDENT as c_int | FIND_STRING as c_int | FIND_EVAL as c_int,
            _ => FIND_STRING as c_int,
        };
        // No modifiers: `<cword>:t` leaves the `:t` where it is.
        let ident = ident_under_cursor(what);
        if ident.is_empty() {
            return Expansion::Refused(QUIET);
        }
        return Expansion::Text {
            text: ident,
            used,
            escaped: false,
        };
    }

    let mut valid = VALID_HEAD as c_int | VALID_PATH as c_int;
    let mut tilde_file = false;
    let mut skip_mod = false;
    let mut escaped = false;
    let name: CString = match spec_idx {
        SPEC_PERC => match Buf::current().name.shown() {
            None => {
                valid = 0;
                CString::default()
            }
            Some(name) => {
                tilde_file = name == c"~";
                name.to_owned()
            }
        },
        SPEC_HASH if byte(1) == b'#' => {
            // `##` is the whole argument list, already escaped.
            used = 2;
            escaped = true;
            skip_mod = true;
            cstr::owned(&arg_all_bytes())
        }
        SPEC_HASH => {
            let start = if byte(1) == b'<' { 2 } else { 1 };
            let (i, mut past) = getdigits_int_at(text, start, false, 0);
            // `#-` is not a negative buffer number; give the `-` back.
            if past == 2 && byte(1) == b'-' {
                past -= 1;
            }
            used = past;
            if byte(1) == b'<' && i != 0 {
                // `#<3` is the third entry of `v:oldfiles`.
                if used < 2 {
                    return Expansion::Skip(1);
                }
                let oldfiles = get_vim_var_list_handle(Vv::Oldfiles);
                let mut numbuf = NumBuf::new();
                match list_find_str(oldfiles.as_deref(), i - 1, &mut numbuf) {
                    None => return Expansion::Refused(QUIET),
                    Some(name) => name.to_owned(),
                }
            } else {
                if i == 0 && byte(1) == b'<' && used > 1 {
                    used = 1;
                }
                let Some(buf) = find_buf(i) else {
                    return Expansion::Refused(gettext(
                        c"E194: No alternate file name to substitute for '#'",
                    ));
                };
                if let Some(lnum) = lnum {
                    *lnum = newlnum::LAST as LineNr;
                }
                match buf.name.shown() {
                    None => {
                        valid = 0;
                        CString::default()
                    }
                    Some(name) => {
                        tilde_file = name == c"~";
                        name.to_owned()
                    }
                }
            }
        }
        SPEC_CFILE => match file_name_at_cursor_owned(FileNameOpts::MESS | FileNameOpts::HYP, 1) {
            None => return Expansion::Refused(QUIET),
            Some(name) => name.as_cstr().to_owned(),
        },
        SPEC_AFILE => {
            // The autocommand's file name is shortened on first use and the
            // shortened form is kept.
            if autocmd_fname.with(Option::is_some) && !autocmd_fname_full.get() {
                autocmd_fname_full.set(true);
                let short = autocmd_fname.with(|name| name.clone().unwrap_or_default());
                let mut name = full_name_save(short.as_cstr(), false);
                // Cut as upstream's `xstrlcpy` into the MAXPATHL buffer.
                name.truncate(MAXPATHL as usize - 1);
                autocmd_fname.set(Some(name));
            }
            let Some(name) =
                autocmd_fname.with(|name| name.as_ref().map(|n| n.as_cstr().to_owned()))
            else {
                return Expansion::Refused(gettext(
                    e_no_autocommand_file_name_to_substitute_for_afile,
                ));
            };
            path_try_shorten_fname(&name).to_owned()
        }
        SPEC_ABUF => {
            if autocmd_bufnr.get() <= 0 {
                return Expansion::Refused(gettext(
                    e_no_autocommand_buffer_number_to_substitute_for_abuf,
                ));
            }
            number(autocmd_bufnr.get())
        }
        SPEC_AMATCH => {
            match autocmd_match.with(|name| name.as_ref().map(|n| n.as_cstr().to_owned())) {
                None => {
                    return Expansion::Refused(gettext(
                        e_no_autocommand_match_name_to_substitute_for_amatch,
                    ));
                }
                Some(name) => name,
            }
        }
        SPEC_SFILE | SPEC_STACK | SPEC_SCRIPT => {
            let (which, msg) = match spec_idx {
                SPEC_SFILE => (ESTACK_SFILE, e_no_source_file_name_to_substitute_for_sfile),
                SPEC_STACK => (ESTACK_STACK, e_no_call_stack_to_substitute_for_stack),
                _ => (
                    ESTACK_SCRIPT,
                    e_no_script_file_name_to_substitute_for_script,
                ),
            };
            match estack_sfile_owned(which) {
                None => return Expansion::Refused(gettext(msg)),
                Some(name) => name.as_cstr().to_owned(),
            }
        }
        SPEC_SLNUM => {
            let entry = &sourcing_entry();
            if entry.es_name.is_null() || entry.es_lnum == 0 {
                return Expansion::Refused(gettext(e_no_line_number_to_use_for_slnum));
            }
            number(entry.es_lnum)
        }
        SPEC_SFLNUM => {
            // The line the *script* is on, which is the script's own
            // offset plus the line inside it.
            let lnum = current_sctx.get().sc_lnum + sourcing_entry().es_lnum;
            if lnum == 0 {
                return Expansion::Refused(gettext(e_no_line_number_to_use_for_sflnum));
            }
            number(lnum)
        }
        SPEC_SID => {
            let sid = current_sctx.get().sc_sid;
            if sid <= 0 {
                return Expansion::Refused(gettext(e_usingsid));
            }
            cstr::owned(format!("<SNR>{sid}_").as_bytes())
        }
        _ => unreachable!("an index into `SPECIALS`"),
    };

    let mut result = name.into_bytes();
    if byte(used) == b'<' {
        // A trailing `<` drops the extension.
        used += 1;
        if let Some(dot) = result.iter().rposition(|&b| b == b'.')
            && dot >= tail_index(&result)
        {
            result.truncate(dot);
        }
    } else if !skip_mod {
        let name = cstr::owned(&result);
        let (reached, modified) = modify_fname(mods, &mut used, tilde_file, &name, result.len());
        valid |= reached;
        result = modified;
    }

    if result.is_empty() || valid != VALID_HEAD as c_int + VALID_PATH as c_int {
        if !empty_is_error {
            return Expansion::Skip(used);
        }
        return Expansion::Refused(if valid != VALID_HEAD as c_int + VALID_PATH as c_int {
            gettext(c"E499: Empty file name for '%' or '#', only works with \":p:h\"")
        } else {
            gettext(c"E500: Evaluates to an empty string")
        });
    }
    Expansion::Text {
        text: result,
        used,
        escaped,
    }
}

/// A number, as the items that are one spell it.
fn number(n: impl core::fmt::Display) -> CString {
    cstr::owned(n.to_string().as_bytes())
}

/// Expand every `<sfile>` in `arg`, in a fresh copy.
///
/// Answers `None` after reporting, when an expansion failed.
pub(crate) fn expand_sfile(arg: &[u8]) -> Option<Vec<u8>> {
    let mut result = arg.to_vec();
    result.push(0);
    let mut p = 0;
    while result[p] != 0 {
        if !result[p..].starts_with(b"<sfile>") {
            p += 1;
            continue;
        }
        let src = CStr::from_bytes_until_nul(&result).expect("terminated just above");
        match eval_vars(src, p, None, true) {
            Expansion::Refused(msg) => {
                if !msg.is_empty() {
                    emsg(msg);
                }
                return None;
            }
            Expansion::Skip(len) => p += len,
            // The backslash goes; the item stays.
            Expansion::Quoted => {
                result.remove(p - 1);
            }
            Expansion::Text { text, used, .. } => {
                result.splice(p..p + used, text.iter().copied());
                p += text.len();
            }
        }
    }
    result.pop();
    Some(result)
}
