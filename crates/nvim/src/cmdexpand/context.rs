//! Classifying the command line: which completion applies where.
//!
//! [`set_expand_context`] is the entry point; [`set_cmd_index`] parses the
//! range and the command name, and the `set_context_in_*` helpers handle the
//! commands whose argument is not a plain file name.  The big per-command
//! switch is in [`super::cmdname`].
//!
//! Every walk here is over the text being completed, `text`, which is the
//! context's own [`Expand::line`] as it stands while the context is worked
//! out: cut at the cursor. Positions are offsets into it, which is what
//! [`Expand::pattern`] holds too.

#![forbid(unsafe_code)]

use super::*;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{vim_is_ident_char, vim_isfilec_or_wc};
use crate::cstr;
use crate::ex_cmds::skip_vimgrep_pat_at;
use crate::ex_docmd::{ends_excmd, excmd_get_cmdidx_bytes};
use crate::ex_getln::parse_pattern_and_range_of;
use crate::guard::Suppress;
use crate::mbyte::char_at;
use crate::mbyte::cluster_len;
use crate::option::magic_isset;
use crate::option::vars::p_ic;
use crate::os::users::{UserMatch, match_user};
use crate::regexp::skip_regexp_at;
use crate::search::state::search_first_line;
use crate::search::{BACKWARD, FORWARD};
use crate::strings::has_char;
use crate::syntax::set_context_in_echohl_cmd;
use crate::types::{CmdIdx, CmdLine, ExArg, ExpandContext};
use crate::usercmd::find_ucmd;
use crate::winlayer::Cc;
use core::ffi::{CStr, c_int};

/// `ASCII_ISALPHA(c) || c == '*'` — the bytes a built-in command name is made
/// of, `*` being accepted as a wildcard.
fn is_cmd_alpha(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'*'
}

/// `ASCII_ISALNUM(c) || c == '*'` — the same for a user command, which may
/// also carry digits.
fn is_cmd_alnum(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'*'
}

/// The byte at `i`, or NUL past the end: how the C walks saw their string.
pub(super) fn byte(text: &[u8], i: usize) -> u8 {
    text.get(i).copied().unwrap_or(0)
}

/// Past the blanks at `i`: upstream's `skipwhite`.
pub(super) fn skipwhite(text: &[u8], mut i: usize) -> usize {
    while ascii_iswhite(c_int::from(byte(text, i))) {
        i += 1;
    }
    i
}

/// To the next blank or the end: upstream's `skiptowhite`.
pub(super) fn skiptowhite(text: &[u8], mut i: usize) -> usize {
    while byte(text, i) != 0 && !ascii_iswhite(c_int::from(byte(text, i))) {
        i += 1;
    }
    i
}

/// Past the digits at `i`: upstream's `skipdigits`.
fn skipdigits(text: &[u8], mut i: usize) -> usize {
    while ascii_isdigit(c_int::from(byte(text, i))) {
        i += 1;
    }
    i
}

/// Whether the byte at `i` is one of `set`; never for the NUL at the end.
pub(super) fn is_one_of(set: &CStr, text: &[u8], i: usize) -> bool {
    has_char(set, c_int::from(byte(text, i)))
}

/// The command after the next `|` or newline from `i`: upstream's
/// `find_nextcmd`.
fn find_nextcmd(text: &[u8], i: usize) -> Option<usize> {
    let rest = text.get(i..).unwrap_or_default();
    rest.iter()
        .position(|&c| c == b'|' || c == b'\n')
        .map(|at| i + at + 1)
}

/// The length of the character at `i`, composing characters included:
/// upstream's `utfc_ptr2len`, 0 at the end.
pub(super) fn char_len(text: &[u8], i: usize) -> usize {
    cluster_len(text.get(i..).unwrap_or_default())
}

/// Set the completion context in `expand` from the command line being edited.
///
/// `expand.context` ends up one of the `EXPAND_*` values, with
/// `expand.pattern` at the text to expand.
pub fn set_expand_context(expand: &mut Expand) {
    let ccline = Cc::current();

    // Handle search commands: '/' or '?'.
    if (ccline.cmdfirstc == c_int::from(b'/') || ccline.cmdfirstc == c_int::from(b'?'))
        && may_expand_pattern.get()
    {
        expand.context = ExpandContext::PatternInBuf;
        expand.search_dir = if ccline.cmdfirstc == c_int::from(b'/') {
            FORWARD
        } else {
            BACKWARD
        };
        expand.line = owned(ccline.text_bytes());
        expand.pattern = 0;
        expand.pattern_len = usize::try_from(ccline.cmdpos).unwrap_or(0);
        search_first_line.set(0); // Search entire buffer
        return;
    }

    // Only handle ':', '>', or '=' command-lines, or expression input.
    if ccline.cmdfirstc != c_int::from(b':')
        && ccline.cmdfirstc != c_int::from(b'>')
        && ccline.cmdfirstc != c_int::from(b'=')
        && ccline.input_fn == 0
    {
        expand.context = ExpandContext::Nothing;
        return;
    }

    // Fallback to command-line expansion.
    let line = ccline.text_bytes().to_vec();
    set_cmd_context(expand, &line, ccline.cmdpos, true);
}

/// Set the index of a built-in or user defined command at `cmd` in
/// `excmd.cmdidx`.
///
/// For user defined commands the completion context is set in `expand` and the
/// completion type in `complp`.
///
/// Returns where the text after the command starts, or `None` for "nothing
/// more to parse" -- an illegal command, or the cursor still in its name.
pub(crate) fn set_cmd_index(
    text: &CStr,
    cmd: usize,
    excmd: &mut ExArg,
    expand: &mut Expand,
    complp: &mut ExpandContext,
) -> Option<usize> {
    let t = text.to_bytes();
    // Both name scans are this loop.  Two monomorphic closures rather
    // than one taking a `fn(u8) -> bool`: the predicate is called once
    // per byte of every command line, and behind a function pointer it
    // cannot inline (`cmdctx` measured +11% that way).
    let skip_alpha = |mut p: usize| {
        while is_cmd_alpha(byte(t, p)) {
            p += 1;
        }
        p
    };
    let skip_alnum = |mut p: usize| {
        while is_cmd_alnum(byte(t, p)) {
            p += 1;
        }
        p
    };

    let mut p;
    let fuzzy = cmdline_fuzzy_complete(&t[cmd..]);

    // Isolate the command and search for it in the command table.
    // Exceptions:
    // - the 'k' command can directly be followed by any character, but do
    //   accept "keepmarks", "keepalt" and "keepjumps".  Bypass also when
    //   'ignorecase' is set so a lowercase ":kz" still completes a user
    //   command like :Kz, and for fuzzy matching as that can find matches
    //   anywhere in the command name.
    // - the 's' command can be followed directly by 'c', 'g', 'i', 'I' or
    //   'r'.
    if !fuzzy && !p_ic() && byte(t, cmd) == b'k' && byte(t, cmd + 1) != b'e' {
        excmd.cmdidx = CmdIdx::k;
        p = cmd + 1;
    } else {
        p = skip_alpha(cmd);
        // A user command may contain digits.
        if byte(t, cmd).is_ascii_uppercase() {
            p = skip_alnum(p);
        }
        // For python 3.x: ":py3*" commands completion.
        if byte(t, cmd) == b'p' && byte(t, cmd + 1) == b'y' && p == cmd + 2 && byte(t, p) == b'3' {
            p = skip_alpha(p + 1);
        }
        // Check for non-alpha command.
        if p == cmd && is_one_of(c"@*!=><&~#", t, p) {
            p += 1;
        }
        let len = p - cmd;

        if len == 0 {
            expand.context = ExpandContext::Unsuccessful;
            return None;
        }

        excmd.cmdidx = excmd_get_cmdidx_bytes(&t[cmd..], len);

        // User defined commands support alphanumeric characters.  Also
        // when doing fuzzy expansion for non-shell commands.
        if byte(t, cmd).is_ascii_uppercase()
            || (fuzzy && excmd.cmdidx != CmdIdx::bang && byte(t, p) != 0)
        {
            p = skip_alnum(p);
        }
    }

    // If the cursor is touching the command, and it ends in an
    // alphanumeric character, complete the command name.
    if byte(t, p) == 0 && byte(t, p - 1).is_ascii_alphanumeric() {
        return None;
    }

    if excmd.cmdidx == CmdIdx::SIZE {
        if byte(t, cmd) == b's' && is_one_of(c"cgriI", t, cmd + 1) {
            excmd.cmdidx = CmdIdx::substitute;
            p = cmd + 1;
        } else if byte(t, cmd).is_ascii_uppercase() {
            // `find_ucmd` measures the typed name as the distance from the
            // command word, so both ends have to be in the same buffer: the
            // command gets a copy of the line, and the answer comes back as
            // an offset into that copy, which is the same text.
            let typed = p - cmd;
            excmd.line = CmdLine::from_bytes(&t[cmd..]);
            match find_ucmd(excmd, typed, None, Some(expand), Some(complp)) {
                Some(at) => p = cmd + at,
                None => excmd.cmdidx = CmdIdx::SIZE, // Ambiguous user command.
            }
        }
    }
    if excmd.cmdidx == CmdIdx::SIZE {
        // Not still touching the command and it was an illegal one.
        expand.context = ExpandContext::Unsuccessful;
        return None;
    }

    Some(p)
}

/// Set the completion context for a command argument with wild card
/// characters, starting at `arg`.
pub(crate) fn set_context_for_wildcard_arg(
    cmdidx: Option<CmdIdx>,
    text: &CStr,
    arg: usize,
    usefilter: bool,
    expand: &mut Expand,
    complp: &mut ExpandContext,
) {
    let t = text.to_bytes();
    let mut in_quote = false;
    let mut bow = None; // Beginning of word.

    // Allow spaces within back-quotes to count as part of the argument
    // being expanded.
    expand.pattern = skipwhite(t, arg);
    let mut p = expand.pattern;
    while byte(t, p) != 0 {
        let mut c = char_at(&t[p..]);
        if c == c_int::from(b'\\') && byte(t, p + 1) != 0 {
            p += 1;
        } else if c == c_int::from(b'`') {
            if !in_quote {
                expand.pattern = p;
                bow = Some(p + 1);
            }
            in_quote = !in_quote;
            // An argument can contain just about everything, except
            // characters that end the command and white space.
        } else if c == c_int::from(b'|')
            || c == c_int::from(b'\n')
            || c == c_int::from(b'"')
            || ascii_iswhite(c)
        {
            let mut len = 0; // avoid getting stuck when space is in 'isfname'
            while byte(t, p) != 0 {
                c = char_at(&t[p..]);
                if c == c_int::from(b'`') || vim_isfilec_or_wc(c) {
                    break;
                }
                len = char_len(t, p);
                p += len;
            }
            if in_quote {
                bow = Some(p);
            } else {
                expand.pattern = p;
            }
            p -= len;
        }
        p += char_len(t, p);
    }

    // If we are still inside the quotes, and we passed a space, just
    // expand from there.
    if let Some(bow) = bow
        && in_quote
    {
        expand.pattern = bow;
    }
    expand.context = ExpandContext::Files;

    // For a shell command more chars need to be escaped. `:!` and
    // `:terminal` run one; so does an explicitly shell-flavoured context.
    let runs_a_shell = cmdidx.is_some_and(|idx| matches!(idx, CmdIdx::bang | CmdIdx::terminal));
    if usefilter || runs_a_shell || *complp == ExpandContext::ShellCmdLine {
        expand.shell = true;
        // When still after the command name expand executables.
        if expand.pattern == skipwhite(t, arg) {
            expand.context = ExpandContext::ShellCmd;
        }
    }

    // Check for environment variable.
    if byte(t, expand.pattern) == b'$' {
        p = expand.pattern + 1;
        while byte(t, p) != 0 && vim_is_ident_char(c_int::from(byte(t, p))) {
            p += 1;
        }
        if byte(t, p) == 0 {
            expand.context = ExpandContext::EnvVars;
            expand.pattern += 1;
            // Avoid that the assignment uses ExpandContext::Files again.
            if *complp != ExpandContext::UserDefined && *complp != ExpandContext::UserList {
                *complp = ExpandContext::EnvVars;
            }
        }
    }
    // Check for user names.
    if byte(t, expand.pattern) == b'~' {
        p = expand.pattern + 1;
        while byte(t, p) != 0 && byte(t, p) != b'/' {
            p += 1;
        }
        // Complete ~user only if it partially matches a user name.  A full
        // match ~user<Tab> will be replaced by the user's home directory,
        // i.e. something like ~user<Tab> -> /home/user/.
        let user = expand.pattern + 1;
        if byte(t, p) == 0 && p > user && match_user(cstr::suffix(text, user)) != UserMatch::None {
            expand.context = ExpandContext::User;
            expand.pattern += 1;
        }
    }
}

/// Set the completion context for the `++opt=arg` argument at `arg`.
/// Always answers "nothing more".
pub(crate) fn set_context_in_argopt(expand: &mut Expand, text: &CStr, arg: usize) -> Option<usize> {
    let t = text.to_bytes();
    expand.pattern = match t[arg..].iter().position(|&c| c == b'=') {
        Some(eq) => arg + eq + 1,
        None => arg,
    };
    expand.context = ExpandContext::Argopt;
    None
}

/// Set the completion context for the `:filter` command.
///
/// Returns where the command after `:filter`'s pattern starts.
pub(crate) fn set_context_in_filter_cmd(
    expand: &mut Expand,
    text: &CStr,
    arg: usize,
) -> Option<usize> {
    let t = text.to_bytes();
    let mut arg = Some(arg);
    if let Some(at) = arg
        && byte(t, at) != 0
    {
        arg = skip_vimgrep_pat_at(cstr::suffix(text, at)).map(|len| at + len);
    }
    match arg {
        Some(at) if byte(t, at) != 0 => Some(skipwhite(t, at)),
        _ => {
            expand.context = ExpandContext::Nothing;
            None
        }
    }
}

/// Set the completion context for the `:match` command.
///
/// Returns where the next command after the `:match` command starts.
pub(crate) fn set_context_in_match_cmd(
    expand: &mut Expand,
    text: &CStr,
    mut arg: usize,
) -> Option<usize> {
    let t = text.to_bytes();
    if byte(t, arg) == 0 || ends_excmd(c_int::from(byte(t, arg))) == 0 {
        // Also complete "None".
        set_context_in_echohl_cmd(expand, arg);
        arg = skipwhite(t, skiptowhite(t, arg));
        if byte(t, arg) != 0 {
            expand.context = ExpandContext::Nothing;
            let delim = c_int::from(byte(t, arg));
            let pattern = &text.to_bytes_with_nul()[arg + 1..];
            arg += 1 + skip_regexp_at(pattern, delim, c_int::from(magic_isset()));
        }
    }
    find_nextcmd(t, arg)
}

/// Where the next command after a `:global` or a `:v` command starts.
pub(crate) fn find_cmd_after_global_cmd(text: &CStr, mut arg: usize) -> Option<usize> {
    let t = text.to_bytes();
    let delim = byte(t, arg); // Get the delimiter.
    if delim != 0 {
        arg += 1; // Skip delimiter if there is one.
    }

    while byte(t, arg) != 0 && byte(t, arg) != delim {
        if byte(t, arg) == b'\\' && byte(t, arg + 1) != 0 {
            arg += 1;
        }
        arg += 1;
    }
    (byte(t, arg) != 0).then_some(arg + 1)
}

/// Where the next command after a `:substitute` or a `:&` command starts.
pub(crate) fn find_cmd_after_substitute_cmd(text: &CStr, mut arg: usize) -> Option<usize> {
    let t = text.to_bytes();
    let delim = byte(t, arg);
    if delim != 0 {
        // Skip "from" part.
        arg += 1;
        let pattern = &text.to_bytes_with_nul()[arg..];
        arg += skip_regexp_at(pattern, c_int::from(delim), c_int::from(magic_isset()));

        if byte(t, arg) != 0 && byte(t, arg) == delim {
            // Skip "to" part.
            arg += 1;
            while byte(t, arg) != 0 && byte(t, arg) != delim {
                if byte(t, arg) == b'\\' && byte(t, arg + 1) != 0 {
                    arg += 1;
                }
                arg += 1;
            }
            if byte(t, arg) != 0 {
                // Skip delimiter.
                arg += 1;
            }
        }
    }
    while byte(t, arg) != 0 && !is_one_of(c"|\"#", t, arg) {
        arg += 1;
    }
    (byte(t, arg) != 0).then_some(arg)
}

/// Where the next command after a `:isearch`/`:dsearch`/`:ilist`/`:dlist`/
/// `:ijump`/`:psearch`/`:djump`/`:isplit`/`:dsplit` command starts.
pub(crate) fn find_cmd_after_isearch_cmd(
    expand: &mut Expand,
    text: &CStr,
    arg: usize,
) -> Option<usize> {
    let t = text.to_bytes();
    // Skip count.
    let mut arg = skipwhite(t, skipdigits(t, arg));
    if byte(t, arg) != b'/' {
        return None;
    }

    // Match regexp, not just whole words.
    arg += 1;
    while byte(t, arg) != 0 && byte(t, arg) != b'/' {
        if byte(t, arg) == b'\\' && byte(t, arg + 1) != 0 {
            arg += 1;
        }
        arg += 1;
    }
    if byte(t, arg) != 0 {
        arg = skipwhite(t, arg + 1);

        // Check for trailing illegal characters.
        if byte(t, arg) == 0 || !is_one_of(c"|\"\n", t, arg) {
            expand.context = ExpandContext::Nothing;
        } else {
            return Some(arg);
        }
    }

    None
}

/// Set the completion context for the `:unlet` command.  Always answers
/// "nothing more".
pub(crate) fn set_context_in_unlet_cmd(
    expand: &mut Expand,
    text: &CStr,
    arg: usize,
) -> Option<usize> {
    let t = text.to_bytes();
    // Only the last argument is completed.
    let arg = match t[arg..].iter().rposition(|&c| c == b' ') {
        Some(space) => arg + space + 1,
        None => arg,
    };

    expand.context = ExpandContext::UserVars;
    expand.pattern = arg;

    if byte(t, expand.pattern) == b'$' {
        expand.context = ExpandContext::EnvVars;
        expand.pattern += 1;
    }

    None
}

/// Set the completion context for the `:language` command.  Always answers
/// "nothing more".
pub(crate) fn set_context_in_lang_cmd(
    expand: &mut Expand,
    text: &CStr,
    arg: usize,
) -> Option<usize> {
    let t = text.to_bytes();
    let p = skiptowhite(t, arg);
    if byte(t, p) == 0 {
        expand.context = ExpandContext::Language;
        expand.pattern = arg;
    } else {
        let word = &t[arg..p];
        let named = [c"messages", c"ctype", c"time", c"collate"]
            .iter()
            .any(|kind| kind.to_bytes().starts_with(word));
        if named {
            expand.context = ExpandContext::Locales;
            expand.pattern = skipwhite(t, p);
        } else {
            expand.context = ExpandContext::Nothing;
        }
    }

    None
}

/// Set the completion context for the `:breakadd` command.  Always answers
/// "nothing more".
pub(crate) fn set_context_in_breakadd_cmd(
    expand: &mut Expand,
    text: &CStr,
    arg: usize,
    cmdidx: CmdIdx,
) -> Option<usize> {
    let t = text.to_bytes();
    expand.context = ExpandContext::Breakpoint;
    expand.pattern = arg;

    breakpt_expand_what.set(if cmdidx == CmdIdx::breakadd {
        BreakptWhat::Add
    } else if cmdidx == CmdIdx::breakdel {
        BreakptWhat::Del
    } else {
        BreakptWhat::ProfDel
    });

    let mut p = skipwhite(t, arg);
    if byte(t, p) == 0 {
        return None;
    }
    let subcmd = &t[p..];

    if subcmd.starts_with(b"file ") || subcmd.starts_with(b"func ") {
        // :breakadd file [lnum] <filename>
        // :breakadd func [lnum] <funcname>
        p = skipwhite(t, p + 4);

        // Skip line number (if specified).
        if ascii_isdigit(c_int::from(byte(t, p))) {
            p = skipdigits(t, p);
            if byte(t, p) != b' ' {
                expand.context = ExpandContext::Nothing;
                return None;
            }
            p = skipwhite(t, p);
        }
        expand.context = if subcmd.starts_with(b"file") {
            ExpandContext::Files
        } else {
            ExpandContext::UserFunc
        };
        expand.pattern = p;
    } else if subcmd.starts_with(b"expr ") {
        // :breakadd expr <expression>
        expand.context = ExpandContext::Expression;
        expand.pattern = skipwhite(t, p + 5);
    }

    None
}

/// Set the completion context for the `:scriptnames` command.  Always
/// answers "nothing more".
pub(crate) fn set_context_in_scriptnames_cmd(
    expand: &mut Expand,
    text: &CStr,
    arg: usize,
) -> Option<usize> {
    let t = text.to_bytes();
    expand.context = ExpandContext::Nothing;

    let p = skipwhite(t, arg);
    if ascii_isdigit(c_int::from(byte(t, p))) {
        return None;
    }

    expand.context = ExpandContext::Scriptnames;
    expand.pattern = p;

    None
}

/// Set the completion context for the `:filetype` command.  Always answers
/// "nothing more".
pub(crate) fn set_context_in_filetype_cmd(
    expand: &mut Expand,
    text: &CStr,
    arg: usize,
) -> Option<usize> {
    let t = text.to_bytes();
    expand.context = ExpandContext::FiletypeCmd;
    expand.pattern = arg;
    filetype_expand_what.set(FiletypeWhat::All);

    let mut p = skipwhite(t, arg);
    if byte(t, p) == 0 {
        return None;
    }

    let mut saw_plugin = false;
    let mut saw_indent = false;

    loop {
        if t[p..].starts_with(b"plugin") {
            saw_plugin = true;
            p = skipwhite(t, p + 6);
            continue;
        }
        if t[p..].starts_with(b"indent") {
            saw_indent = true;
            p = skipwhite(t, p + 6);
            continue;
        }
        break;
    }

    // Whichever half is already spelled out is the half not to offer
    // again; naming both leaves only "on"/"off".
    filetype_expand_what.set(match (saw_plugin, saw_indent) {
        (true, true) => FiletypeWhat::OnOff,
        (true, false) => FiletypeWhat::Indent,
        (false, true) => FiletypeWhat::Plugin,
        (false, false) => FiletypeWhat::All,
    });

    expand.pattern = p;

    None
}

/// Set the completion context for commands that involve a search pattern and a
/// line range (e.g. `:s`, `:g`, `:v`).
pub(crate) fn set_context_with_pattern(expand: &mut Expand) {
    let ccline = Cc::current();

    let no_emsg = Suppress::emsg();
    let parsed = parse_pattern_and_range_of(pre_incsearch_pos.get());
    drop(no_emsg);

    // Check if cursor is within search pattern.
    let Some((skiplen, patlen)) = parsed else {
        return;
    };
    if ccline.cmdpos <= skiplen || ccline.cmdpos > skiplen + patlen {
        return;
    }

    expand.pattern = usize::try_from(skiplen).unwrap_or(0);
    expand.pattern_len = usize::try_from(ccline.cmdpos - skiplen).unwrap_or(0);
    expand.context = ExpandContext::PatternInBuf;
    expand.search_dir = FORWARD;
}
