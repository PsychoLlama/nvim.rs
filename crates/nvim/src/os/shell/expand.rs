//! Wildcard expansion by handing the patterns to the shell.
//!
//! Nvim has no glob of its own for this: it writes a command that makes the
//! user's `'shell'` print the expanded names into a temp file, runs it, and
//! reads the file back. Which command, and how the names come back separated,
//! depends on which shell it is — see [`ShellStyle`].

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::ascii::ascii_iswhite;
use crate::charset::backslash_halved;
use crate::cstr;
use crate::drawscreen::state::cmdline_row;
use crate::fileio::temp_name;
use crate::guard::{sandbox, secure};
use crate::memory::XString;
use crate::message::{e_notmp, e_wildexpand};
use crate::message::{emsg, msg, msg_putchar, msg_start};
use crate::message_fmt::msg_cstr;
use crate::os::cshim::gettext;
use crate::os::fs::{can_execute, dir_exists, os_remove, path_exists};
use crate::os::time::os_delay;
use crate::path::{ExpandFlags, add_pathsep, has_wildcard, invocation_path_tail, path_tail};
use crate::semsg;
use crate::types::{Failed, READBIN};
use crate::ui::state::Rows;
use core::ops::Range;

/// The `vimglob()` shell function, for a POSIX shell.
const SH_VIMGLOB_FUNC: &str =
    "vimglob() { while [ $# -ge 1 ]; do echo \"$1\"; shift; done }; vimglob >";
/// Turning on bash's `globstar`, so `**` works. bash >= 4 only.
const SH_GLOBSTAR_OPT: &str = "[[ ${BASH_VERSINFO[0]} -ge 4 ]] && shopt -s globstar; ";

/// How the shell is asked to print the expansion, and therefore how the names
/// come back separated. There is no portable answer, so each family gets the
/// spelling that works best for it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ShellStyle {
    /// `echo`, space separated. A shell we do not recognise; stay safe.
    Echo,
    /// `glob`, NUL separated. Better than `echo` on \*csh.
    Glob,
    /// `vimglob()`, newline separated. Any \*sh\*.
    VimGlob,
    /// `print -N`, newline *or* NUL separated. Better than `glob` on \*zsh.
    Print,
    /// `` `cmd` `` — run the pattern directly, newline separated.
    Backtick,
    /// `vimglob()` with `globstar` on, newline separated. bash.
    GlobStar,
}

impl ShellStyle {
    /// How the names come back.
    fn separator(self) -> Separator {
        match self {
            ShellStyle::Echo => Separator::Space,
            ShellStyle::Backtick | ShellStyle::VimGlob | ShellStyle::GlobStar => Separator::Newline,
            ShellStyle::Glob | ShellStyle::Print => Separator::Nul,
        }
    }
}

enum Separator {
    Space,
    Newline,
    Nul,
}

/// Copy the patterns through unexpanded — what happens when there is nothing
/// to expand, and what [`ExpandFlags::NOTFOUND`] asks for when the expansion
/// finds nothing. The backslashes are halved, to be compatible with
/// `expand_filename()`.
fn save_patterns(patterns: &[XString]) -> Vec<XString> {
    patterns
        .iter()
        .map(|pat| XString::from_vec(backslash_halved(pat)))
        .collect()
}

fn have_wildcard(patterns: &[XString]) -> bool {
    patterns.iter().any(|pat| has_wildcard(pat))
}

fn have_dollars(patterns: &[XString]) -> bool {
    patterns.iter().any(|pat| pat.contains(&b'$'))
}

/// Which style the current `'shell'` wants, given the patterns.
fn pick_shell_style(patterns: &[XString]) -> ShellStyle {
    // `cmd` expansion runs the pattern itself.
    if let [first] = patterns
        && first.len() > 2
        && first.starts_with(b"`")
        && first.ends_with(b"`")
    {
        return ShellStyle::Backtick;
    }
    p_sh(|sh| {
        let bytes = sh.to_bytes();
        if bytes.ends_with(b"csh") {
            return ShellStyle::Glob;
        }
        if bytes.ends_with(b"zsh") {
            return ShellStyle::Print;
        }
        let tail = path_tail(sh).to_bytes();
        if contains(tail, b"bash") {
            ShellStyle::GlobStar
        } else if contains(tail, b"sh") {
            ShellStyle::VimGlob
        } else {
            ShellStyle::Echo
        }
    })
}

/// `strstr` over bytes.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Whether `'shell'` is fish, which needs `begin; … end` rather than `( … )`.
fn is_fish_shell() -> bool {
    p_sh(|sh| invocation_path_tail(sh).to_bytes().starts_with(b"fish"))
}

/// Escape one pattern into the shell command, backslashing
/// [`SHELL_SPECIAL`] outside backticks.
fn push_escaped_pattern(command: &mut Vec<u8>, pat: &[u8], flags: ExpandFlags) {
    let mut intick = false;
    let mut at = 0;
    while at < pat.len() {
        let b = pat[at];
        if b == b'`' {
            intick = !intick;
        } else if b == b'\\' && at + 1 < pat.len() {
            // Drop the backslash and take the next byte literally — but keep
            // it inside backticks, before a special character, and before a
            // backtick.
            let next = pat[at + 1];
            if intick || SHELL_SPECIAL.contains(&next) || next == b'`' {
                command.push(b'\\');
            }
            at += 1;
            command.push(pat[at]);
            at += 1;
            continue;
        } else if !intick
            && (!flags.has(ExpandFlags::KEEPDOLLAR) || b != b'$')
            && SHELL_SPECIAL.contains(&b)
        {
            // Not inside backticks, and not `$var` under KEEPDOLLAR.
            command.push(b'\\');
        }
        command.push(b);
        at += 1;
    }
}

/// The shell command that writes the expansion into `tempname`, and whether
/// it ends in a `&` that has to be moved after the redirection.
fn build_command(
    style: ShellStyle,
    patterns: &[XString],
    tempname: &[u8],
    flags: ExpandFlags,
) -> (Vec<u8>, bool) {
    let fish = is_fish_shell();
    let mut command: Vec<u8> = Vec::new();
    let mut ampersand = false;

    if style == ShellStyle::Backtick {
        // Turn "`command; command& `" into "(command; command )".
        let inner: &[u8] = &patterns[0];
        command.extend_from_slice(if fish { b"begin; " } else { b"(" });
        // Exclude the leading backtick; the trailing one is overwritten.
        command.extend_from_slice(&inner[1..]);
        let last = command.len() - 1;
        if fish {
            command[last] = b';';
            command.extend_from_slice(b" end");
        } else {
            command[last] = b')';
        }
        // Upstream steps back from the byte it just wrote, so the ')' or ';'
        // is not itself considered.
        let mut at = last.saturating_sub(1);
        while at > 0 && ascii_iswhite(command[at] as c_int) {
            at -= 1;
        }
        if command[at] == b'&' {
            // The '&' moves after the redirection, below.
            ampersand = true;
            command[at] = b' ';
        }
        command.push(b'>');
    } else {
        if style == ShellStyle::Glob {
            // `nonomatch` is only valid on csh-likes; elsewhere it would set
            // the positional parameters.
            command.extend_from_slice(if flags.has(ExpandFlags::NOTFOUND) {
                b"set nonomatch; ".as_slice()
            } else {
                b"unset nonomatch; ".as_slice()
            });
        }
        match style {
            ShellStyle::Glob => command.extend_from_slice(b"glob >"),
            ShellStyle::Print => command.extend_from_slice(b"print -N >"),
            ShellStyle::VimGlob => command.extend_from_slice(SH_VIMGLOB_FUNC.as_bytes()),
            ShellStyle::GlobStar => {
                command.extend_from_slice(SH_GLOBSTAR_OPT.as_bytes());
                command.extend_from_slice(SH_VIMGLOB_FUNC.as_bytes());
            }
            _ => command.extend_from_slice(b"echo >"),
        }
    }

    command.extend_from_slice(tempname);

    if style != ShellStyle::Backtick {
        for pat in patterns {
            command.push(b' ');
            push_escaped_pattern(&mut command, pat, flags);
        }
    }

    if ampersand {
        // After the redirection, not before it.
        command.push(b'&');
    }
    (command, ampersand)
}

/// The byte ranges of the individual names in the shell's output.
///
/// Upstream does this in two passes over the same buffer — one to count so it
/// can size the pointer array, one to split — and they agree only by
/// inspection. Here there is one pass and a `Vec`.
fn split_entries(buffer: &[u8], sep: &Separator, check_spaces: bool) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut at = 0;
    match sep {
        // Upstream writes a '\n' one past the content and stops at the first
        // one, so a name containing a newline ends the list.
        Separator::Space => {
            while at < buffer.len() && buffer[at] != b'\n' {
                let start = at;
                while at < buffer.len() && buffer[at] != b' ' && buffer[at] != b'\n' {
                    at += 1;
                }
                out.push(start..at);
                // skipwhite
                while at < buffer.len() && (buffer[at] == b' ' || buffer[at] == b'\t') {
                    at += 1;
                }
            }
        }
        Separator::Newline => {
            while at < buffer.len() {
                let start = at;
                while at < buffer.len() && buffer[at] != b'\n' {
                    at += 1;
                }
                out.push(start..at);
                if at < buffer.len() {
                    at += 1;
                }
                while at < buffer.len() && (buffer[at] == b' ' || buffer[at] == b'\t') {
                    at += 1;
                }
            }
        }
        Separator::Nul => {
            if buffer.is_empty() {
                return out;
            }
            while at <= buffer.len() {
                let start = at;
                while at < buffer.len() && buffer[at] != 0 && !(check_spaces && buffer[at] == b' ')
                {
                    at += 1;
                }
                out.push(start..at);
                at += 1;
            }
        }
    }
    out
}

/// Some zsh builds separate with spaces rather than NULs. Only believe that
/// while no NUL has ever been seen, because once one has, embedded spaces in
/// file names have to keep working.
static did_find_nul: GlobalCell<bool> = GlobalCell::new(false);

/// Expand `patterns` through the shell, into the names it answers.
///
/// `flags` is the set `expand_wildcards()` takes — when matching fails but
/// [`ExpandFlags::NOTFOUND`] is set, or there was nothing to expand, the
/// patterns themselves are answered instead.
pub fn os_expand_wildcards(
    patterns: &[XString],
    flags: ExpandFlags,
) -> Result<Vec<XString>, Failed> {
    // Whether an executable may be looked for along `$PATH`: not when this
    // is `expand_shellcmd` doing the looking.
    let use_path = !flags.has(ExpandFlags::SHELLCMD);

    // With no wildcards, copy the names across rather than starting a
    // shell — that saves a great deal of time.
    if !have_wildcard(patterns) {
        return Ok(save_patterns(patterns));
    }

    // No shell command inside the sandbox, and no backticks in `secure`.
    if sandbox.get() != 0 && check_secure() {
        return Err(Failed);
    }
    if secure.get() != 0 && patterns.iter().any(|pat| pat.contains(&b'`')) && check_secure() {
        return Err(Failed);
    }

    let Some(tempname) = temp_name() else {
        emsg(gettext(e_notmp));
        return Err(Failed);
    };
    let style = pick_shell_style(patterns);
    let (command, ampersand) = build_command(style, patterns, &tempname, flags);
    let command = CString::new(command).expect("no interior NUL in a shell command");

    let mut shellopts = ShellOpts::EXPAND | ShellOpts::SILENT;
    if flags.has(ExpandFlags::SILENT) {
        shellopts |= ShellOpts::HIDE_MESS;
    }
    // With zsh -G a pattern that matches nothing is dropped from the
    // argument list; otherwise zsh errors out and expands nothing else.
    // With csh -f the shell variables from .cshrc are not expanded, so it
    // is only usable when no pattern holds a `$`.
    let extra_shell_arg = match style {
        ShellStyle::Print => c"-G".as_ptr().cast_mut(),
        ShellStyle::Glob if !have_dollars(patterns) => c"-f".as_ptr().cast_mut(),
        _ => ptr::null_mut(),
    };

    // SAFETY: a NUL-terminated command this frame owns, and a static extra
    // argument or none.
    let failed =
        unsafe { call_shell(command.as_ptr().cast_mut(), shellopts, extra_shell_arg) } != 0;
    // A backgrounded command needs a moment to create the temp file, but
    // is not waited for.
    if ampersand {
        os_delay(10, true);
    }

    if failed {
        os_remove(tempname.as_cstr());
        // With interactive completion the message is not printed.
        if !flags.has(ExpandFlags::SILENT) {
            msg_putchar('\n' as c_int); // clear the bottom line quickly
            cmdline_row.set(Rows.get() - 1); // continue on the last line
            msg(gettext(e_wildexpand), 0);
            msg_start(); // do not overwrite this message
        }
        // A failed `cmd` expansion must not list `cmd` as a match, even
        // under NOTFOUND.
        if style == ShellStyle::Backtick {
            return Err(Failed);
        }
        return not_found(patterns, flags);
    }

    let mut buffer = match read_temp_file(tempname.as_cstr(), flags) {
        Read::Content(buffer) => buffer,
        Read::NotFound => return not_found(patterns, flags),
        Read::Failed => return Err(Failed),
    };
    drop(tempname);

    // Upstream writes a sentinel one past the content; the vector carries
    // the room for it.
    let len = buffer.len();
    buffer.push(0);

    let mut check_spaces = false;
    let mut content = len;
    match style.separator() {
        Separator::Space => buffer[len] = b'\n',
        Separator::Newline => buffer[len] = 0,
        Separator::Nul => {
            if style == ShellStyle::Print && !did_find_nul.get() {
                // A NUL anywhere in the output proves this zsh uses them.
                if len != 0
                    && CStr::from_bytes_until_nul(&buffer).is_ok_and(|s| s.count_bytes() < len)
                {
                    did_find_nul.set(true);
                } else {
                    check_spaces = true;
                }
            }
            // STYLE_PRINT already ends in a NUL; STYLE_GLOB needs one.
            if len != 0 && buffer[len - 1] == 0 {
                content = len - 1;
            }
        }
    }

    let entries = split_entries(&buffer[..content], &style.separator(), check_spaces);
    if entries.is_empty() {
        // Happens with /bin/sh and `:e $NO_SUCH_VAR<Tab>`: it expands to
        // nothing rather than reporting an error.
        return not_found(patterns, flags);
    }

    let mut found = Vec::with_capacity(entries.len());
    for entry in entries {
        // Each entry ends at its separator, which upstream overwrites with
        // a NUL; a NUL inside it ends it early, as it did there.
        let mut name = cstr::until_nul(&buffer[entry]).to_vec();
        let name_cstr = cstr::owned(&name);

        // Require the file to exist; helps when using /bin/sh.
        if !flags.has(ExpandFlags::NOTFOUND) && !path_exists(&name_cstr) {
            continue;
        }
        let dir = dir_exists(&name_cstr);
        if (dir && !flags.has(ExpandFlags::DIR)) || (!dir && !flags.has(ExpandFlags::FILE)) {
            continue;
        }
        // Skip what is not executable, when that is being checked for.
        if !dir && flags.has(ExpandFlags::EXEC) && !can_execute(&name_cstr, use_path) {
            continue;
        }

        if dir {
            // A directory name gets a trailing '/'.
            add_pathsep(&mut name);
        }
        found.push(XString::from_vec(name));
    }

    if found.is_empty() {
        // Every entry was rejected.
        return not_found(patterns, flags);
    }
    Ok(found)
}

/// How reading the shell's temp file turned out. The two failures are not
/// interchangeable: a file that would not open falls back to the patterns
/// under [`ExpandFlags::NOTFOUND`], while one that would not seek or read is a
/// hard
/// `FAIL`.
enum Read {
    Content(Vec<u8>),
    NotFound,
    Failed,
}

/// Read the temp file the shell wrote into, reporting whatever went wrong.
fn read_temp_file(tempname: &CStr, flags: ExpandFlags) -> Read {
    // SAFETY: a NUL-terminated name; `fd` is closed on every path out.
    unsafe {
        let fd = fopen(tempname.as_ptr(), READBIN.as_ptr());
        if fd.is_null() {
            // Something went wrong — perhaps a file name with a special
            // character in it.
            if !flags.has(ExpandFlags::SILENT) {
                msg(gettext(e_wildexpand), 0);
                msg_start(); // do not overwrite this message
            }
            return Read::NotFound;
        }
        if fseek(fd, 0, SEEK_END) < 0 {
            fclose(fd);
            return Read::Failed;
        }
        let templen = ftell(fd);
        if templen < 0 {
            fclose(fd);
            return Read::Failed;
        }
        let len = templen as usize;
        fseek(fd, 0, SEEK_SET);

        let mut buffer: Vec<u8> = vec![0; len];
        let readlen = fread(buffer.as_mut_ptr().cast(), 1, len, fd);
        fclose(fd);
        os_remove(tempname);
        if readlen as usize != len {
            semsg!("E485: Can't read file {}", msg_cstr(tempname));
            return Read::Failed;
        }
        Read::Content(buffer)
    }
}

/// Upstream's `notfound:` label: under `ExpandFlags::NOTFOUND` the patterns
/// themselves become the answer.
fn not_found(patterns: &[XString], flags: ExpandFlags) -> Result<Vec<XString>, Failed> {
    if !flags.has(ExpandFlags::NOTFOUND) {
        return Err(Failed);
    }
    Ok(save_patterns(patterns))
}
