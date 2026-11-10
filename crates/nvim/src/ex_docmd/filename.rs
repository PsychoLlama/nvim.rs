//! What a command's file argument stands for: `%`, `#`, the `<…>` family,
//! wildcards, and the backtick form.
//!
//! Expansion rewrites the command line in place: every replacement is
//! spliced into the `ExArg`'s own line by [`repl_cmdline`], which moves the
//! cursors past it.
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
use crate::cmdexpand::{WildMode, WildOpts, expand_one};
use crate::cstr;
use crate::env::{expand_env, expand_env_escaped};
use crate::ex_cmds::newlnum;
use crate::normal::{FIND_EVAL, FIND_IDENT, FIND_STRING, ident_under_cursor};
use crate::optionstr::{OptString, local_or_global};
use crate::runtime::defs::{ESTACK_SCRIPT, ESTACK_SFILE, ESTACK_STACK};
use crate::types::Handle;
use crate::types::{CmdIdx, EcmdCmd};
use core::ffi::CStr;
use core::ffi::c_int;
use std::ffi::CString;

use crate::arglist::arg_all_bytes;
use crate::autocmd::state::{AUTOCMD_BUFNR, AUTOCMD_FNAME, AUTOCMD_FNAME_FULL, AUTOCMD_MATCH};
use crate::buffer::find_buf;
use crate::charset::{backslash_halved, getdigits_int_at};
use crate::cmdexpand::defs::{Expand, ExpandContext};
use crate::eval::fs::modify_fname;
use crate::eval::skip_expr;
use crate::eval::typval::{NumBuf, list_find_str};
use crate::eval::vars::get_vim_var_list_handle;
use crate::ex_docmd::CmdError;
use crate::ex_docmd::scan::skip_grep_pat;
use crate::ex_docmd::state::ESCAPE_CHARS;
use crate::ex_docmd::{
    E_NO_AUTOCOMMAND_BUFFER_NUMBER_TO_SUBSTITUTE_FOR_ABUF,
    E_NO_AUTOCOMMAND_FILE_NAME_TO_SUBSTITUTE_FOR_AFILE,
    E_NO_AUTOCOMMAND_MATCH_NAME_TO_SUBSTITUTE_FOR_AMATCH, E_NO_CALL_STACK_TO_SUBSTITUTE_FOR_STACK,
    E_NO_LINE_NUMBER_TO_USE_FOR_SFLNUM, E_NO_LINE_NUMBER_TO_USE_FOR_SLNUM,
    E_NO_SCRIPT_FILE_NAME_TO_SUBSTITUTE_FOR_SCRIPT, E_NO_SOURCE_FILE_NAME_TO_SUBSTITUTE_FOR_SFILE,
    VALID_HEAD, VALID_PATH,
};
use crate::file_search::{FileNameOpts, file_name_at_cursor};
use crate::message::{E_USINGSID, emsg, msg_make};
use crate::option::vars::{P_GP, P_MP, p_wic};
use crate::os::cshim::gettext;
use crate::path::MAXPATHL;
use crate::path::{full_name_save, has_wildcard, path_try_shorten_fname, tail_index};
use crate::quickfix::grep_internal;
use crate::runtime::estack_sfile;
use crate::runtime::state::CURRENT_SCTX;
use crate::strings::{escaped_bytes, replaced_all};
use crate::types::{CmdLine, ExArg, ExArgt, Failed, LineNr, Vv};
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

impl ExArg<'_> {
    /// Expand every `%`, `#`, `` `cmd` `` and `<…>` in a command's file
    /// argument, then expand wildcards if the command takes exactly one name.
    ///
    /// Every cursor into the line is re-read from the `CmdLine` after each
    /// replacement: an expansion *grows the buffer*, so an offset held across
    /// one is stale.
    pub(crate) fn expand_filename(&mut self) -> Result<(), CmdError> {
        // A `:vimgrep` pattern is not a file name, so the scan starts after
        // it.
        let mut at = skip_grep_pat(self);
        let mut has_wildcards = has_wildcard(self.line.rest_of(at));

        while self.line.byte_at(at) != 0 {
            if self.line.byte_at(at) == b'`' && self.line.byte_at(at + 1) == b'=' {
                // `` `=expr` `` is evaluated much later, by the shell
                // expansion; step over it without touching it.
                at += 2 + skip_expr(self.line.rest_of(at + 2));
                if self.line.byte_at(at) == b'`' {
                    at += 1;
                }
                continue;
            }
            if !b"%#<".contains(&self.line.byte_at(at)) {
                at += 1;
                continue;
            }

            // The items are read relative to the argument: a backslash before
            // one quotes it, but only from inside the argument.
            let base = self.line.arg;
            let found = eval_vars(
                self.line.cstr_from(base),
                at - base,
                Some(&mut self.do_ecmd_lnum),
                true,
            );
            let (mut repl, srclen, escaped) = match found {
                Expansion::Quiet => return Err(CmdError::Quiet),
                Expansion::Refused(msg) => return Err(CmdError::Message(msg.to_owned())),
                Expansion::Skip(len) => {
                    at += len;
                    continue;
                }
                Expansion::Quoted => {
                    // Take the backslash out; the item stays, and the scan
                    // resumes on the byte after its first.
                    self.line.drop_byte(at - 1);
                    continue;
                }
                Expansion::Text {
                    text,
                    used,
                    escaped,
                } => (text, used, escaped),
            };

            if repl.contains(&b'$') || repl.contains(&b'~') {
                repl = expand_env(&cstr::owned(&repl), false).to_vec();
            }

            // A name that will be handed to a *file* argument gets the
            // shell-special characters escaped — but not for the commands
            // that hand the whole argument to a shell themselves, and not
            // for a name that came back already escaped (`##`).
            let idx = self.cmdidx;
            let escape_chars = ESCAPE_CHARS.to_bytes();
            if !self.usefilter
                && !escaped
                && idx != CmdIdx::bang
                && idx != CmdIdx::grep
                && idx != CmdIdx::grepadd
                && idx != CmdIdx::lgrep
                && idx != CmdIdx::lgrepadd
                && idx != CmdIdx::lmake
                && idx != CmdIdx::make
                && idx != CmdIdx::terminal
                && !self.argt.has(ExArgt::NOSPC)
                && repl.iter().any(|byte| escape_chars.contains(byte))
            {
                repl = escaped_bytes(&repl, escape_chars).to_vec();
            }
            // A `!` in the replacement would be read as "the previous
            // command" by the shell-command line parser.
            if (self.usefilter || idx == CmdIdx::bang || idx == CmdIdx::terminal)
                && repl.contains(&b'!')
            {
                repl = escaped_bytes(&repl, b"!").to_vec();
            }

            at = repl_cmdline(self, at, srclen, &repl);
        }

        // `ExArgt::NOSPC` means the argument is one file name, so wildcards in
        // it can be expanded to exactly one match.
        if !self.argt.has(ExArgt::NOSPC) || self.usefilter {
            return Ok(());
        }

        let (start, len) = (self.line.arg, self.line.rest_of(self.line.arg).len());
        if has_wildcards {
            // Environment variables first: they may hold the wildcards, or
            // may be all that looked like one.
            let arg = self.line.rest_of(start);
            if arg.contains(&b'$') || arg.contains(&b'~') {
                let out = expand_env_escaped(self.line.cstr_from(start), true, true, None);
                has_wildcards = has_wildcard(&out);
                repl_cmdline(self, start, len, &out);
            }
        }
        if !has_wildcards {
            // The backslashes go, in place: the argument only gets shorter.
            let halved = backslash_halved(self.line.rest_of(self.line.arg));
            let at = self.line.arg;
            let text = self.line.buffer_mut();
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
        let arg = CString::new(self.line.rest_of(self.line.arg)).unwrap_or_default();
        let Some(expanded) = expand_one(&mut xpc, Some(&arg), None, options, WildMode::ExpandFree)
        else {
            return Err(CmdError::Quiet);
        };
        let text = expanded.as_cstr().to_bytes().to_vec();
        let (start, len) = (self.line.arg, self.line.rest_of(self.line.arg).len());
        repl_cmdline(self, start, len, &text);
        Ok(())
    }
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
    /// The expansion failed with this message, and the caller stops.
    Refused(&'static CStr),
    /// The expansion failed with nothing to say (a `<cword>` with no word,
    /// a `<cfile>` whose lookup already reported), and the caller stops.
    Quiet,
}

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
    let Some((spec_idx, mut used)) = cmdline_var_at(text) else {
        return Expansion::Skip(1);
    };
    // A backslash before it means "take it literally".
    if at > 0 && whole[at - 1] == b'\\' {
        return Expansion::Quoted;
    }
    if matches!(spec_idx, SPEC_CWORD | SPEC_CCWORD | SPEC_CEXPR) {
        return text_under_cursor(spec_idx, used);
    }
    let named = match item_name(spec_idx, text, &mut used, lnum) {
        Ok(named) => named,
        Err(answer) => return answer,
    };
    // The item and what follows it, as `modify_fname` reads its modifiers.
    let mods = CStr::from_bytes_until_nul(&src.to_bytes_with_nul()[at..])
        .expect("a tail of a terminated string");
    named.modified(text, mods, used, empty_is_error)
}

/// `<cword>`, `<cWORD>` or `<cexpr>`: the text under the cursor, with no
/// modifiers (`<cword>:t` leaves the `:t` where it is).
fn text_under_cursor(spec_idx: usize, used: usize) -> Expansion {
    let what = match spec_idx {
        SPEC_CWORD => FIND_IDENT | FIND_STRING,
        SPEC_CEXPR => FIND_IDENT | FIND_STRING | FIND_EVAL,
        _ => FIND_STRING,
    };
    let ident = ident_under_cursor(what);
    if ident.is_empty() {
        return Expansion::Quiet;
    }
    Expansion::Text {
        text: ident,
        used,
        escaped: false,
    }
}

/// A special item's name, before its modifiers apply.
struct Named {
    name: CString,
    /// What `modify_fname` counts as reached already: a buffer with no name
    /// has neither a head nor a path.
    valid: c_int,
    /// The name is `~`, which `modify_fname` must not expand.
    tilde_file: bool,
    /// `##`, the whole argument list: modifiers do not apply, and the names
    /// come back escaped already.
    arglist: bool,
}

/// Both of `modify_fname`'s "reached" bits: a name it can work on.
const VALID_NAME: c_int = VALID_HEAD.cast_signed() + VALID_PATH.cast_signed();

impl Named {
    /// A name that is a path as it stands.
    fn path(name: CString) -> Self {
        Self {
            name,
            valid: VALID_NAME,
            tilde_file: false,
            arglist: false,
        }
    }

    /// `buffer`'s name as shown, or no name at all.
    fn of_buffer(buffer: Buf) -> Self {
        match buffer.name.shown() {
            None => Self {
                valid: 0,
                ..Self::path(CString::default())
            },
            Some(name) => Self {
                tilde_file: name == c"~",
                ..Self::path(name.to_owned())
            },
        }
    }

    /// The name with the modifiers after the item applied, `used` bytes of
    /// the item read so far: a trailing `<` drops the extension, anything
    /// else is `modify_fname`'s `:p:h:t…`.
    fn modified(
        self,
        text: &[u8],
        mods: &CStr,
        mut used: usize,
        empty_is_error: bool,
    ) -> Expansion {
        let mut valid = self.valid;
        let mut result = self.name.into_bytes();
        if text.get(used) == Some(&b'<') {
            used += 1;
            if let Some(dot) = result.iter().rposition(|&b| b == b'.')
                && dot >= tail_index(&result)
            {
                result.truncate(dot);
            }
        } else if !self.arglist {
            let name = cstr::owned(&result);
            let (reached, modified) =
                modify_fname(mods, &mut used, self.tilde_file, &name, result.len());
            valid |= reached;
            result = modified;
        }

        if result.is_empty() || valid != VALID_NAME {
            if !empty_is_error {
                return Expansion::Skip(used);
            }
            return Expansion::Refused(if valid != VALID_NAME {
                gettext(c"E499: Empty file name for '%' or '#', only works with \":p:h\"")
            } else {
                gettext(c"E500: Evaluates to an empty string")
            });
        }
        Expansion::Text {
            text: result,
            used,
            escaped: self.arglist,
        }
    }
}

/// The name the item at the start of `text` stands for, `used` its length
/// so far (`#` and its digits lengthen it). `Err` is [`eval_vars`]'s answer
/// when there is no name to modify.
fn item_name(
    spec_idx: usize,
    text: &[u8],
    used: &mut usize,
    lnum: Option<&mut LineNr>,
) -> Result<Named, Expansion> {
    Ok(match spec_idx {
        SPEC_PERC => Named::of_buffer(Buf::current()),
        SPEC_HASH if text.get(1) == Some(&b'#') => {
            *used = 2;
            Named {
                arglist: true,
                ..Named::path(cstr::owned(&arg_all_bytes()))
            }
        }
        SPEC_HASH => alternate_name(text, used, lnum)?,
        SPEC_CFILE => {
            let name = file_name_at_cursor(FileNameOpts::MESS | FileNameOpts::HYP, 1, None)
                .ok_or(Expansion::Quiet)?;
            Named::path(name.as_cstr().to_owned())
        }
        SPEC_AFILE => Named::path(autocmd_file_name().map_err(Expansion::Refused)?),
        SPEC_ABUF => {
            if AUTOCMD_BUFNR.get() <= 0 {
                return Err(Expansion::Refused(gettext(
                    E_NO_AUTOCOMMAND_BUFFER_NUMBER_TO_SUBSTITUTE_FOR_ABUF,
                )));
            }
            Named::path(number(AUTOCMD_BUFNR.get()))
        }
        SPEC_AMATCH => {
            let name = AUTOCMD_MATCH
                .with(|name| name.as_ref().map(|n| n.as_cstr().to_owned()))
                .ok_or_else(|| {
                    Expansion::Refused(gettext(
                        E_NO_AUTOCOMMAND_MATCH_NAME_TO_SUBSTITUTE_FOR_AMATCH,
                    ))
                })?;
            Named::path(name)
        }
        SPEC_SFILE | SPEC_STACK | SPEC_SCRIPT => {
            Named::path(sourcing_name(spec_idx).map_err(Expansion::Refused)?)
        }
        SPEC_SLNUM => {
            let lnum = crate::runtime::sourcing_lnum();
            if !crate::runtime::has_sourcing_name() || lnum == LineNr::ZERO {
                return Err(Expansion::Refused(gettext(
                    E_NO_LINE_NUMBER_TO_USE_FOR_SLNUM,
                )));
            }
            Named::path(number(lnum))
        }
        SPEC_SFLNUM => {
            // The line the *script* is on, which is the script's own offset
            // plus the line inside it.
            let lnum = CURRENT_SCTX.get().sc_lnum + crate::runtime::sourcing_lnum().get();
            if lnum == LineNr::ZERO {
                return Err(Expansion::Refused(gettext(
                    E_NO_LINE_NUMBER_TO_USE_FOR_SFLNUM,
                )));
            }
            Named::path(number(lnum))
        }
        SPEC_SID => {
            let sid = CURRENT_SCTX.get().sc_sid;
            if sid <= 0 {
                return Err(Expansion::Refused(gettext(E_USINGSID)));
            }
            Named::path(cstr::owned(format!("<SNR>{sid}_").as_bytes()))
        }
        _ => unreachable!("an index into `SPECIALS`"),
    })
}

/// `#`, `#N`, `#-` or `#<N`: an alternate buffer's name, or the Nth entry of
/// `v:oldfiles`. `used` becomes the item's length with its digits.
fn alternate_name(
    text: &[u8],
    used: &mut usize,
    lnum: Option<&mut LineNr>,
) -> Result<Named, Expansion> {
    let byte = |i: usize| text.get(i).copied().unwrap_or(0);
    let start = if byte(1) == b'<' { 2 } else { 1 };
    let (i, mut past) = getdigits_int_at(text, start, false, 0);
    // `#-` is not a negative buffer number; give the `-` back.
    if past == 2 && byte(1) == b'-' {
        past -= 1;
    }
    *used = past;
    if byte(1) == b'<' && i != 0 {
        // `#<3` is the third entry of `v:oldfiles`.
        if *used < 2 {
            return Err(Expansion::Skip(1));
        }
        let oldfiles = get_vim_var_list_handle(Vv::Oldfiles);
        let mut numbuf = NumBuf::new();
        let name = list_find_str(oldfiles.as_deref(), i - 1, &mut numbuf)
            .map_err(|Failed| Expansion::Quiet)?;
        return Ok(Named::path(name.to_owned()));
    }
    if i == 0 && byte(1) == b'<' && *used > 1 {
        *used = 1;
    }
    let buf = find_buf(Handle::from_number(i)).ok_or_else(|| {
        Expansion::Refused(gettext(
            c"E194: No alternate file name to substitute for '#'",
        ))
    })?;
    if let Some(lnum) = lnum {
        *lnum = newlnum::LAST as LineNr;
    }
    Ok(Named::of_buffer(buf))
}

/// `<afile>`: the autocommand's file name, shortened on first use and kept
/// shortened. Refuses with the message to report.
fn autocmd_file_name() -> Result<CString, &'static CStr> {
    if AUTOCMD_FNAME.with(Option::is_some) && !AUTOCMD_FNAME_FULL.get() {
        AUTOCMD_FNAME_FULL.set(true);
        let short = AUTOCMD_FNAME.with(|name| name.clone().unwrap_or_default());
        let mut name = full_name_save(short.as_cstr(), false);
        // Cut as upstream's `xstrlcpy` into the MAXPATHL buffer.
        name.truncate(MAXPATHL as usize - 1);
        AUTOCMD_FNAME.set(Some(name));
    }
    let name = AUTOCMD_FNAME
        .with(|name| name.as_ref().map(|n| n.as_cstr().to_owned()))
        .ok_or_else(|| gettext(E_NO_AUTOCOMMAND_FILE_NAME_TO_SUBSTITUTE_FOR_AFILE))?;
    Ok(path_try_shorten_fname(&name).to_owned())
}

/// `<sfile>`, `<stack>` or `<script>`: what the execution stack says is
/// being sourced. Refuses with the message to report.
fn sourcing_name(spec_idx: usize) -> Result<CString, &'static CStr> {
    let (which, msg) = match spec_idx {
        SPEC_SFILE => (ESTACK_SFILE, E_NO_SOURCE_FILE_NAME_TO_SUBSTITUTE_FOR_SFILE),
        SPEC_STACK => (ESTACK_STACK, E_NO_CALL_STACK_TO_SUBSTITUTE_FOR_STACK),
        _ => (
            ESTACK_SCRIPT,
            E_NO_SCRIPT_FILE_NAME_TO_SUBSTITUTE_FOR_SCRIPT,
        ),
    };
    estack_sfile(which)
        .map(|name| name.as_cstr().to_owned())
        .ok_or_else(|| gettext(msg))
}

/// A number, as the items that are one spell it.
fn number(n: impl core::fmt::Display) -> CString {
    cstr::owned(n.to_string().as_bytes())
}

/// Expand every `<sfile>` in `arg`, in a fresh copy.
///
/// Answers `Err(Failed)` when an expansion failed, its message (if it had
/// one) already out.
pub(crate) fn expand_sfile(arg: &[u8]) -> Result<Vec<u8>, Failed> {
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
                emsg(msg);
                return Err(Failed);
            }
            Expansion::Quiet => return Err(Failed),
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
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{Expansion, Named};

    /// A trailing `<` drops the extension of the name's tail, and only of
    /// its tail; `##` takes no modifier and comes back escaped.
    #[test]
    fn a_trailing_angle_drops_the_tail_extension() {
        let drop_ext =
            |name: &core::ffi::CStr| Named::path(name.to_owned()).modified(b"%<", c"%<", 1, true);
        assert_eq!(
            drop_ext(c"dir/foo.c"),
            Expansion::Text {
                text: b"dir/foo".to_vec(),
                used: 2,
                escaped: false
            }
        );
        assert_eq!(
            drop_ext(c"a.d/foo"),
            Expansion::Text {
                text: b"a.d/foo".to_vec(),
                used: 2,
                escaped: false
            }
        );
        let arglist = Named {
            arglist: true,
            ..Named::path(c"a\\ b c".to_owned())
        };
        assert_eq!(
            arglist.modified(b"##:t", c"##:t", 2, true),
            Expansion::Text {
                text: b"a\\ b c".to_vec(),
                used: 2,
                escaped: true
            }
        );
        // A nameless buffer is nothing to substitute, and says so only when
        // asked to.
        let nameless = || Named {
            valid: 0,
            ..Named::path(c"".to_owned())
        };
        assert_eq!(
            nameless().modified(b"%<", c"%<", 1, false),
            Expansion::Skip(2)
        );
    }
}
