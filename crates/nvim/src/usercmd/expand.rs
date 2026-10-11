//! Running a user command: expanding the `<...>` codes of its replacement
//! text and handing the result to `do_cmdline()`.
//!
//! [`expand_replacement`] builds the expanded body in one `Vec<u8>`.
//! Upstream walks the replacement twice -- once with a null destination to
//! measure, once to write into exactly that room -- and every helper here
//! once followed that shape; now each appends what it expands to.
//! [`uc_check_code`] is one `<code>`, [`uc_mods`] the `<mods>` one (also
//! called on its own, to reproduce a command line's modifiers), and
//! [`uc_split_args`] the `<f-args>` splitter.
//!
//! The `q-`/`f-` prefixes are a quoting level rather than a code of their
//! own: `<q-args>` is the argument as one Vim string literal, `<f-args>` is
//! it split on unescaped whitespace into a comma-separated list of them.
//!
//! The command's entry is read under its table's borrow and nothing from it
//! survives the borrow but copies: the expanded body (or, for a Lua command,
//! a [`LuaCommand`]). The body can then redefine or delete the very command
//! that is running.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use super::Scope;
use crate::ascii::ascii_iswhite;
use crate::cstr;
use crate::ex_docmd::state::cmdmod;
use crate::ex_docmd::{DoCmdOpts, do_cmdline};
use crate::guard::Script;
use crate::keycodes::{K_SPECIAL, KE_FILLER};
use crate::lua::executor::nlua_do_ucmd;
use crate::mbyte::c_cluster_len;
use crate::types::CmdIdx;
use crate::types::{
    CmdLine, CmdMod, CmdModFlags, ExArg, ExArgt, LuaRef, ScriptCtx, UserCmd, int64_t,
};
use crate::window::tab_index;
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT};
use crate::winlayer::TabPage;
use core::ffi::{CStr, c_int};
use std::ffi::CString;
use std::rc::Rc;

/// The second byte of an escaped `K_SPECIAL`.
const KS_SPECIAL: u8 = 254;

/// One argument of the whitespace-separated argument list, unescaped into
/// `out` (which is cleared first).
///
/// The iteration protocol is upstream's: `end` carries the position the
/// last call stopped at, and the answer is "that was the last one". Both
/// `nvim_parse_cmd()` and the Lua command callback split arguments with it.
pub(crate) fn uc_split_args_iter(arg: &[u8], end: &mut usize, out: &mut Vec<u8>) -> bool {
    out.clear();
    let arglen = arg.len();
    if arglen == 0 {
        return true;
    }
    let mut pos = *end;
    // Upstream looks one byte past the argument, where the NUL is; nothing
    // past the end can be whitespace.
    let white_at = |i: usize| arg.get(i).is_some_and(|&b| ascii_iswhite(c_int::from(b)));

    while pos < arglen && white_at(pos) {
        pos += 1;
    }

    while pos < arglen - 1 {
        // A backslash escapes a backslash or a separator, and nothing else.
        if arg[pos] == b'\\' && (arg[pos + 1] == b'\\' || white_at(pos + 1)) {
            pos += 1;
        }
        out.push(arg[pos]);
        if white_at(pos + 1) {
            *end = pos + 1;
            return false;
        }
        pos += 1;
    }
    if pos < arglen && !white_at(pos) {
        out.push(arg[pos]);
    }
    true
}

/// At most how many arguments `arg` splits into -- the number of runs of
/// non-whitespace, which escaped separators can only reduce.
pub(crate) fn uc_nargs_upper_bound(arg: &[u8]) -> usize {
    let mut was_white = true; // space before the first argument
    let mut nargs = 0;
    for &byte in arg {
        let is_white = ascii_iswhite(c_int::from(byte));
        if was_white && !is_white {
            nargs += 1;
        }
        was_white = is_white;
    }
    nargs
}

/// `<f-args>`: the arguments as a comma-separated list of Vim string
/// literals.
fn uc_split_args(line: &CmdLine) -> Vec<u8> {
    let mut out = Vec::<u8>::new();
    out.push(b'"');
    if line.args.is_empty() {
        quote_line(line.rest_of(line.arg), &mut out);
    } else {
        let argc = line.args.len();
        for (i, &(at, len)) in line.args.iter().enumerate() {
            // A character is read as the line has it, so a cluster may run
            // past the span's end.
            quote_span(line.rest_of(at), len, &mut out);
            if i != argc - 1 {
                out.extend_from_slice(b"\", \"");
            }
        }
    }
    out.push(b'"');
    out
}

/// The whole argument string `arg`, split on unescaped whitespace.
fn quote_line(arg: &[u8], out: &mut Vec<u8>) {
    let at = |i: usize| cstr::byte_at(arg, i);
    let mut p = 0;
    while at(p) != 0 {
        let (first, second) = (at(p), at(p + 1));
        if first == b'\\' && second == b'\\' {
            out.extend_from_slice(b"\\\\");
            p += 2;
        } else if first == b'\\' && ascii_iswhite(c_int::from(second)) {
            // An escaped separator stands for one literal separator.
            out.push(second);
            p += 2;
        } else if first == b'\\' || first == b'"' {
            out.push(b'\\');
            out.push(first);
            p += 1;
        } else if ascii_iswhite(c_int::from(first)) {
            while ascii_iswhite(c_int::from(at(p))) {
                p += 1;
            }
            if at(p) == 0 {
                break;
            }
            out.extend_from_slice(b"\", \"");
        } else {
            p += copy_char(&arg[p..], out);
        }
    }
}

/// One already-split argument, the first `len` bytes of `text`, as the
/// body of a literal.
fn quote_span(text: &[u8], len: usize, out: &mut Vec<u8>) {
    let mut p = 0;
    while p < len {
        let byte = cstr::byte_at(text, p);
        if byte == b'\\' || byte == b'"' {
            out.push(b'\\');
            out.push(byte);
            p += 1;
        } else {
            p += copy_char(text.get(p..).unwrap_or_default(), out);
        }
    }
}

/// Copy the whole character `text` starts with, combining marks and all,
/// and answer how far that moved: at least one byte, so a walk always
/// advances.
fn copy_char(text: &[u8], out: &mut Vec<u8>) -> usize {
    let len = c_cluster_len(text);
    out.extend_from_slice(&text[..len]);
    len.max(1)
}

/// Append one modifier to `out`, separated from the previous one.
fn add_cmd_modifier(out: &mut Vec<u8>, name: &[u8], multi_mods: &mut bool) {
    if *multi_mods {
        out.push(b' ');
    }
    out.extend_from_slice(name);
    *multi_mods = true;
}

/// The window-placement modifiers of `cmod`, in the order upstream emits
/// them, one word each.
pub(crate) fn win_cmd_modifier_words(cmod: &CmdMod) -> Vec<CString> {
    let split = |flag: u32| cmod.cmod_split & flag as c_int != 0;
    let mut words: Vec<CString> = Vec::new();
    let mut add = |name: &CStr, present: bool| {
        if present {
            words.push(name.to_owned());
        }
    };
    // `:aboveleft`/`:leftabove`, `:belowright`/`:rightbelow`, `:botright`.
    add(c"aboveleft", split(WSP_ABOVE));
    add(c"belowright", split(WSP_BELOW));
    add(c"botright", split(WSP_BOT));
    if cmod.cmod_tab > 0 {
        let tabnr = cmod.cmod_tab - 1;
        // For compatibility, a tab number that is `:tab`'s own default is
        // left off.
        if tabnr == tab_index(TabPage::current()) {
            add(c"tab", true);
        } else {
            let text = CString::new(format!("{tabnr}tab")).expect("no NUL");
            add(&text, true);
        }
    }
    add(c"topleft", split(WSP_TOP));
    add(c"vertical", split(WSP_VERT));
    add(c"horizontal", split(WSP_HOR));
    words
}

/// [`win_cmd_modifier_words`] joined by spaces, as a command line spells
/// them.
pub(crate) fn win_cmd_modifiers(cmod: &CmdMod) -> Vec<u8> {
    win_cmd_modifier_words(cmod)
        .iter()
        .map(|word| word.to_bytes())
        .collect::<Vec<_>>()
        .join(&b' ')
}

/// The modifiers `cmod` carries, as the text that would produce them,
/// appended to `out`; `quote` wraps them in `"`.
pub(crate) fn uc_mods_into(out: &mut Vec<u8>, cmod: &CmdMod, quote: bool) {
    /// The modifiers that are nothing but a flag.
    static MOD_ENTRIES: [(CmdModFlags, &[u8]); 12] = [
        (CmdModFlags::BROWSE, b"browse"),
        (CmdModFlags::CONFIRM, b"confirm"),
        (CmdModFlags::HIDE, b"hide"),
        (CmdModFlags::KEEPALT, b"keepalt"),
        (CmdModFlags::KEEPJUMPS, b"keepjumps"),
        (CmdModFlags::KEEPMARKS, b"keepmarks"),
        (CmdModFlags::KEEPPATTERNS, b"keeppatterns"),
        (CmdModFlags::LOCKMARKS, b"lockmarks"),
        (CmdModFlags::NOSWAPFILE, b"noswapfile"),
        (CmdModFlags::UNSILENT, b"unsilent"),
        (CmdModFlags::NOAUTOCMD, b"noautocmd"),
        (CmdModFlags::SANDBOX, b"sandbox"),
    ];
    let (flags, verbose) = (cmod.cmod_flags, cmod.cmod_verbose);
    let mut multi_mods = false;
    if quote {
        out.push(b'"');
    }
    for &(flag, name) in &MOD_ENTRIES {
        if flags.has(flag) {
            add_cmd_modifier(out, name, &mut multi_mods);
        }
    }
    if flags.has(CmdModFlags::SILENT) {
        let name: &[u8] = if flags.has(CmdModFlags::ERRSILENT) {
            b"silent!"
        } else {
            b"silent"
        };
        add_cmd_modifier(out, name, &mut multi_mods);
    }
    if verbose > 0 {
        let value = verbose - 1;
        let name = if value == 1 {
            b"verbose".to_vec()
        } else {
            format!("{value}verbose").into_bytes()
        };
        add_cmd_modifier(out, &name, &mut multi_mods);
    }
    for word in win_cmd_modifier_words(cmod) {
        add_cmd_modifier(out, word.to_bytes(), &mut multi_mods);
    }
    if quote {
        out.push(b'"');
    }
}

/// [`uc_mods_into`] a buffer of its own.
pub(crate) fn uc_mods(cmod: &CmdMod, quote: bool) -> Vec<u8> {
    let mut out = Vec::new();
    uc_mods_into(&mut out, cmod, quote);
    out
}

/// The `<...>` codes a replacement string may contain.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Code {
    Args,
    Bang,
    Count,
    Line1,
    Line2,
    Range,
    Mods,
    Register,
    Lt,
    /// Not a code at all: only the `<` is copied.
    None,
}

impl Code {
    /// The code `name` spells -- the text between the `<` and, inclusive,
    /// the `>`.
    ///
    /// Upstream compares with `STRNICMP(p, "args>", l)`, a prefix test, but
    /// `l` counts through the `>` and no prefix of `args>` ends in one, so
    /// only the whole word can match. The case-blindness is real: `<Args>`
    /// is `<args>`.
    fn parse(name: &[u8]) -> Code {
        const CODES: [(&[u8], Code); 10] = [
            (b"args>", Code::Args),
            (b"bang>", Code::Bang),
            (b"count>", Code::Count),
            (b"line1>", Code::Line1),
            (b"line2>", Code::Line2),
            (b"range>", Code::Range),
            (b"lt>", Code::Lt),
            (b"reg>", Code::Register),
            (b"register>", Code::Register),
            (b"mods>", Code::Mods),
        ];
        for (text, code) in CODES {
            if name.len() == text.len() && name.eq_ignore_ascii_case(text) {
                return code;
            }
        }
        Code::None
    }
}

/// How a code wants its expansion quoted.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Quote {
    /// `<args>`: as it stands.
    None,
    /// `<q-args>`: one string literal.
    One,
    /// `<f-args>`: split into a list of string literals.
    Split,
}

/// `body` between a pair of `quote` bytes, or `body` alone.
fn put_quoted(out: &mut Vec<u8>, quote: Option<u8>, body: &[u8]) {
    if let Some(q) = quote {
        out.push(q);
    }
    out.extend_from_slice(body);
    if let Some(q) = quote {
        out.push(q);
    }
}

/// Expand one `<code>` -- `code` is its text, `<` and `>` included -- onto
/// `out`. Answers `false` when it is not a code, in which case only the `<`
/// has been copied.
///
/// `split` is the run's `<f-args>` cache: splitting is done once.
fn uc_check_code(
    code: &[u8],
    out: &mut Vec<u8>,
    cmd: &UserCmd,
    args: &ExArg,
    split: &mut Option<Vec<u8>>,
) -> bool {
    let inner = &code[1..];
    let (quote, name) = match inner {
        [b'q' | b'Q', b'-', rest @ ..] => (Quote::One, rest),
        [b'f' | b'F', b'-', rest @ ..] => (Quote::Split, rest),
        rest => (Quote::None, rest),
    };
    let quotes = |byte: u8| (quote != Quote::None).then_some(byte);

    match Code::parse(name) {
        Code::Args => expand_args(out, args, quote, split),
        Code::Bang => {
            let body: &[u8] = if args.forceit { b"!" } else { b"" };
            put_quoted(out, quotes(b'"'), body);
        }
        code @ (Code::Line1 | Code::Line2 | Code::Range | Code::Count) => {
            let num: int64_t = match code {
                Code::Line1 => int64_t::from(args.line1),
                Code::Line2 => int64_t::from(args.line2),
                Code::Range => int64_t::from(args.addr_count),
                // `<count>` is the range's end, or the command's default.
                _ if args.addr_count > 0 => int64_t::from(args.line2),
                _ => cmd.default_count,
            };
            put_quoted(out, quotes(b'"'), num.to_string().as_bytes());
        }
        Code::Mods => cmdmod.with(|cmod| uc_mods_into(out, cmod, quote != Quote::None)),
        Code::Register => {
            let register = [u8::try_from(args.regname).unwrap_or_default()];
            let body: &[u8] = if args.regname != 0 { &register } else { b"" };
            put_quoted(out, quotes(b'\''), body);
        }
        Code::Lt => out.push(b'<'),
        Code::None => {
            // Not recognised: copy the `<` and say so.
            out.push(b'<');
            return false;
        }
    }
    true
}

/// `<args>`, `<q-args>` or `<f-args>`.
fn expand_args(out: &mut Vec<u8>, args: &ExArg, quote: Quote, split: &mut Option<Vec<u8>>) {
    let arg = args.line.arg();
    if arg.is_empty() {
        if quote == Quote::One {
            out.extend_from_slice(b"''");
        }
        return;
    }
    // A command declared to take a single argument does not split it, so
    // that `:Cmd %` works when `%` stands for "a b c".
    let quote = if args.argt.has(ExArgt::NOSPC) && quote == Quote::Split {
        Quote::One
    } else {
        quote
    };
    match quote {
        Quote::None => out.extend_from_slice(arg),
        Quote::One => {
            out.push(b'"');
            for &byte in arg {
                if byte == b'\\' || byte == b'"' {
                    out.push(b'\\');
                }
                out.push(byte);
            }
            out.push(b'"');
        }
        // Splitting is expensive, so it is done once and cached.
        Quote::Split => {
            let split = split.get_or_insert_with(|| uc_split_args(&args.line));
            out.extend_from_slice(split);
        }
    }
}

/// What a Lua command's run needs of its entry, copied out of the table:
/// the callback may redefine or delete the command while it runs.
pub(crate) struct LuaCommand {
    pub(crate) name: Rc<CStr>,
    pub(crate) argt: ExArgt,
    pub(crate) default_count: int64_t,
    /// The reference to call: the command's own, or its preview's.
    pub(crate) luaref: LuaRef,
}

/// How one run goes, decided under the table's borrow.
enum Run {
    Lua(LuaCommand),
    Vim {
        body: Vec<u8>,
        keepscript: bool,
        script_ctx: ScriptCtx,
    },
}

/// Run one user command: expand its replacement text and execute it.
///
/// `preview` runs the `'inccommand'` callback instead, which only a Lua
/// command can have.
pub(crate) fn do_ucmd(excmd: &mut ExArg, preview: bool) -> c_int {
    let scope = if excmd.cmdidx == CmdIdx::USER {
        Scope::Global
    } else {
        Scope::Buffer
    };
    let useridx = usize::try_from(excmd.useridx).expect("`find_ucmd` set the index");
    // Nothing under the borrow runs user code: the expansion only reads the
    // entry, the command's arguments and its modifiers.
    let run = scope.with(|cmds| {
        let cmd = &cmds[useridx];
        let lua = |luaref| {
            Run::Lua(LuaCommand {
                name: cmd.name.clone(),
                argt: cmd.argt,
                default_count: cmd.default_count,
                luaref,
            })
        };
        if preview {
            debug_assert!(cmd.preview_luaref > 0, "cmd->uc_preview_luaref > 0");
            lua(cmd.preview_luaref)
        } else if cmd.luaref > 0 {
            lua(cmd.luaref)
        } else {
            Run::Vim {
                body: expand_replacement(cmd, excmd),
                keepscript: cmd.argt.has(ExArgt::KEEPSCRIPT),
                script_ctx: cmd.script_ctx,
            }
        }
    });

    match run {
        Run::Lua(call) => {
            let retv = nlua_do_ucmd(&call, excmd, preview);
            if preview { retv } else { 0 }
        }
        Run::Vim {
            body,
            keepscript,
            script_ctx,
        } => {
            // The command body runs with the defining script's id, unless it
            // asked to keep the caller's.
            let script_ctx = (!keepscript).then(|| Script::sid(script_ctx.sc_sid));
            let opts = DoCmdOpts::VERBOSE | DoCmdOpts::NOWAIT | DoCmdOpts::KEYTYPED;
            let _ = do_cmdline(Some(cstr::until_nul(&body)), excmd.source_mut(), opts);
            drop(script_ctx);
            0
        }
    }
}

/// `cmd`'s replacement text with every `<code>` expanded.
fn expand_replacement(cmd: &UserCmd, args: &ExArg) -> Vec<u8> {
    let rep = cmd.replacement.to_bytes();
    let mut out = Vec::with_capacity(rep.len());
    let mut split = None;
    let find = |from: usize, byte: u8| {
        rep.get(from..)
            .and_then(|tail| tail.iter().position(|&b| b == byte))
            .map(|at| from + at)
    };
    let mut p = 0;
    loop {
        let start = find(p, b'<');
        let end = start.and_then(|start| find(start + 1, b'>'));

        if unescape_k_special(rep, &mut p, &mut out, start, end) {
            continue;
        }
        let (Some(start), Some(end)) = (start, end) else {
            break;
        };
        let end = end + 1; // include the '>'

        // Everything up to the '<' is copied as it stands.
        out.extend_from_slice(&rep[p..start]);
        p = if uc_check_code(&rep[start..end], &mut out, cmd, args, &mut split) {
            end
        } else {
            // Not a code: carry on after the '<'.
            start + 1
        };
    }
    out.extend_from_slice(&rep[p..]);
    out
}

/// `K_SPECIAL` is stored escaped, as for a mapping, but `do_cmdline()` does
/// not undo that -- so undo it here, copying everything up to it and the
/// byte itself. Answers whether one was found and handled.
///
/// Only an escape before the next `<...>` (or with none left) counts, as
/// upstream's writing pass has it.
fn unescape_k_special(
    rep: &[u8],
    p: &mut usize,
    out: &mut Vec<u8>,
    start: Option<usize>,
    end: Option<usize>,
) -> bool {
    let at = |i: usize| cstr::byte_at(rep, i);
    let Some(ksp) = rep[*p..]
        .iter()
        .position(|&b| c_int::from(b) == K_SPECIAL)
        .map(|at| *p + at)
    else {
        return false;
    };
    if !(start.is_none_or(|start| ksp < start) || end.is_none())
        || at(ksp + 1) != KS_SPECIAL
        || c_int::from(at(ksp + 2)) != KE_FILLER
    {
        return false;
    }
    out.extend_from_slice(&rep[*p..ksp]);
    out.push(rep[ksp]);
    *p = ksp + 3;
    true
}
