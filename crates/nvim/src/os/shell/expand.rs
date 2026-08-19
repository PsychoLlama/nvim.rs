//! Wildcard expansion by handing the patterns to the shell.
//!
//! Nvim has no glob of its own for this: it writes a command that makes the
//! user's `'shell'` print the expanded names into a temp file, runs it, and
//! reads the file back. Which command, and how the names come back separated,
//! depends on which shell it is — see [`ShellStyle`].

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ascii::ascii_iswhite;
use crate::charset::backslash_halve;
use crate::fileio::vim_tempname;
use crate::main::{
    Rows, cmdline_row, e_cant_read_file_str, e_notmp, e_wildexpand, sandbox, secure,
};
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::message::{emsg, msg, msg_putchar, msg_start};
use crate::os::cshim::gettext;
use crate::os::fs::{os_can_exe, os_isdir, os_path_exists, os_remove};
use crate::os::time::os_delay;
use crate::path::{ExpandFlags, add_pathsep, invocation_path_tail, path_has_wildcard, path_tail};
use crate::semsg_c;
use crate::strings::vim_strchr;
use crate::types::{FAIL, OK, READBIN};
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
/// finds
/// nothing.
///
/// # Safety
/// `pat[0..num_pat]` must be NUL-terminated strings, and `num_file`/`file`
/// writable.
unsafe fn save_patterns(
    num_pat: c_int,
    pat: *mut *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
) {
    // SAFETY: the caller's contract.
    unsafe {
        *file = xmalloc(num_pat as usize * size_of::<*mut c_char>()) as *mut *mut c_char;
        for i in 0..num_pat as isize {
            let s = xstrdup(*pat.offset(i));
            // Be compatible with `expand_filename()`: halve the backslashes.
            backslash_halve(s);
            *(*file).offset(i) = s;
        }
        *num_file = num_pat;
    }
}

/// # Safety
/// `file[0..num]` must be NUL-terminated strings.
unsafe fn have_wildcard(num: c_int, file: *mut *mut c_char) -> bool {
    // SAFETY: the caller's contract.
    unsafe { (0..num as isize).any(|i| path_has_wildcard(*file.offset(i))) }
}

/// # Safety
/// `file[0..num]` must be NUL-terminated strings.
unsafe fn have_dollars(num: c_int, file: *mut *mut c_char) -> bool {
    // SAFETY: the caller's contract.
    unsafe { (0..num as isize).any(|i| !vim_strchr(*file.offset(i), '$' as c_int).is_null()) }
}

/// Which style the current `'shell'` wants, given the patterns.
///
/// # Safety
/// `pat[0..num_pat]` must be NUL-terminated strings.
unsafe fn pick_shell_style(num_pat: c_int, pat: *mut *mut c_char) -> ShellStyle {
    // SAFETY: the caller's contract; `p_sh` is a NUL-terminated option value.
    unsafe {
        // `cmd` expansion runs the pattern itself.
        if num_pat == 1 {
            let first = CStr::from_ptr(*pat).to_bytes();
            if first.len() > 2 && first.starts_with(b"`") && first.ends_with(b"`") {
                return ShellStyle::Backtick;
            }
        }
        let sh = CStr::from_ptr(p_sh.get()).to_bytes();
        if sh.ends_with(b"csh") {
            return ShellStyle::Glob;
        }
        if sh.ends_with(b"zsh") {
            return ShellStyle::Print;
        }
        let tail = CStr::from_ptr(path_tail(p_sh.get())).to_bytes();
        if contains(tail, b"bash") {
            ShellStyle::GlobStar
        } else if contains(tail, b"sh") {
            ShellStyle::VimGlob
        } else {
            ShellStyle::Echo
        }
    }
}

/// `strstr` over bytes.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Whether `'shell'` is fish, which needs `begin; … end` rather than `( … )`.
fn is_fish_shell() -> bool {
    // SAFETY: `p_sh` is a NUL-terminated option value, and
    // `invocation_path_tail` answers a pointer inside it.
    unsafe { CStr::from_ptr(invocation_path_tail(p_sh.get(), ptr::null_mut())).to_bytes() }
        .starts_with(b"fish")
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
///
/// # Safety
/// `pat[0..num_pat]` must be NUL-terminated strings.
unsafe fn build_command(
    style: ShellStyle,
    num_pat: c_int,
    pat: *mut *mut c_char,
    tempname: &[u8],
    flags: ExpandFlags,
) -> (Vec<u8>, bool) {
    let fish = is_fish_shell();
    let mut command: Vec<u8> = Vec::new();
    let mut ampersand = false;

    if style == ShellStyle::Backtick {
        // Turn "`command; command& `" into "(command; command )".
        // SAFETY: the caller's contract.
        let inner = unsafe { CStr::from_ptr(*pat) }.to_bytes();
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
        for i in 0..num_pat as isize {
            command.push(b' ');
            // SAFETY: the caller's contract.
            let pat = unsafe { CStr::from_ptr(*pat.offset(i)) }.to_bytes();
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

/// Expand `pat[0..num_pat]` through the shell.
///
/// `num_file` and `file` receive the count and a newly allocated array of
/// newly allocated names; whatever `*file` held is not freed. `flags` is a
/// combination of `EW_*` as `expand_wildcards()` uses them — when matching
/// fails but [`ExpandFlags::NOTFOUND`] is set, or there was nothing to expand,
/// the
/// patterns themselves are copied through instead.
///
/// Answers `OK` or `FAIL`; on `FAIL`, `*file` is NULL.
///
/// # Safety
/// `pat[0..num_pat]` must be NUL-terminated strings; `num_file` and `file`
/// must be non-NULL and writable.
pub unsafe fn os_expand_wildcards(
    num_pat: c_int,
    pat: *mut *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    flags: ExpandFlags,
) -> c_int {
    // SAFETY: the caller's contract, for the whole body. Every pattern is
    // read as a `CStr`, and every pointer written out is freshly allocated.
    unsafe {
        // Default: no files found.
        *num_file = 0;
        *file = ptr::null_mut();

        // With no wildcards, copy the names across rather than starting a
        // shell — that saves a great deal of time.
        if !have_wildcard(num_pat, pat) {
            save_patterns(num_pat, pat, num_file, file);
            return OK;
        }

        // No shell command inside the sandbox, and no backticks in `secure`.
        if sandbox.get() != 0 && check_secure() {
            return FAIL;
        }
        if secure.get() != 0 {
            for i in 0..num_pat as isize {
                if !vim_strchr(*pat.offset(i), '`' as c_int).is_null() && check_secure() {
                    return FAIL;
                }
            }
        }

        let tempname = vim_tempname();
        if tempname.is_null() {
            emsg(gettext((&raw const e_notmp).cast()));
            return FAIL;
        }
        let style = pick_shell_style(num_pat, pat);
        let (command, ampersand) = build_command(
            style,
            num_pat,
            pat,
            CStr::from_ptr(tempname).to_bytes(),
            flags,
        );
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
            ShellStyle::Glob if !have_dollars(num_pat, pat) => c"-f".as_ptr().cast_mut(),
            _ => ptr::null_mut(),
        };

        let failed = call_shell(command.as_ptr().cast_mut(), shellopts, extra_shell_arg) != 0;
        // A backgrounded command needs a moment to create the temp file, but
        // is not waited for.
        if ampersand {
            os_delay(10, true);
        }

        if failed {
            os_remove(tempname);
            xfree(tempname.cast());
            // With interactive completion the message is not printed.
            if !flags.has(ExpandFlags::SILENT) {
                msg_putchar('\n' as c_int); // clear the bottom line quickly
                cmdline_row.set(Rows.get() - 1); // continue on the last line
                msg(gettext((&raw const e_wildexpand).cast()), 0);
                msg_start(); // do not overwrite this message
            }
            // A failed `cmd` expansion must not list `cmd` as a match, even
            // under NOTFOUND.
            if style == ShellStyle::Backtick {
                return FAIL;
            }
            return not_found(num_pat, pat, num_file, file, flags);
        }

        let mut buffer = match read_temp_file(tempname, flags) {
            Read::Content(buffer) => buffer,
            Read::NotFound => return not_found(num_pat, pat, num_file, file, flags),
            Read::Failed => return FAIL,
        };
        xfree(tempname.cast());

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
            return not_found(num_pat, pat, num_file, file, flags);
        }

        // Terminate every entry in place, as upstream does by overwriting the
        // separators — each range is then a C string inside `buffer`.
        for entry in &entries {
            buffer[entry.end] = 0;
        }

        let out = xmalloc(entries.len() * size_of::<*mut c_char>()) as *mut *mut c_char;
        let mut kept = 0isize;
        for entry in &entries {
            let name = buffer[entry.start..].as_ptr().cast::<c_char>();

            // Require the file to exist; helps when using /bin/sh.
            if !flags.has(ExpandFlags::NOTFOUND) && !os_path_exists(name) {
                continue;
            }
            let dir = os_isdir(name);
            if (dir && !flags.has(ExpandFlags::DIR)) || (!dir && !flags.has(ExpandFlags::FILE)) {
                continue;
            }
            // Skip what is not executable, when that is being checked for.
            if !dir
                && flags.has(ExpandFlags::EXEC)
                && !os_can_exe(name, ptr::null_mut(), !flags.has(ExpandFlags::SHELLCMD))
            {
                continue;
            }

            let p = xmalloc(strlen(name) + 1 + dir as usize) as *mut c_char;
            strcpy(p, name);
            if dir {
                // A directory name gets a trailing '/'.
                add_pathsep(p);
            }
            *out.offset(kept) = p;
            kept += 1;
        }

        if kept == 0 {
            // Every entry was rejected.
            xfree(out.cast());
            return not_found(num_pat, pat, num_file, file, flags);
        }
        *num_file = kept as c_int;
        *file = out;
        OK
    }
}

/// How reading the shell's temp file turned out. The two failures are not
/// interchangeable: a file that would not open falls back to the patterns
/// under [`ExpandFlags::NOTFOUND`], while one that would not seek or read is a
/// hard
/// `FAIL`. `tempname` is freed on every path but the successful one.
enum Read {
    Content(Vec<u8>),
    NotFound,
    Failed,
}

/// Read the temp file the shell wrote into, reporting whatever went wrong.
///
/// # Safety
/// `tempname` must be a NUL-terminated path this owns nothing of.
unsafe fn read_temp_file(tempname: *mut c_char, flags: ExpandFlags) -> Read {
    // SAFETY: the caller's contract; `fd` is closed on every path out.
    unsafe {
        let fd = fopen(tempname, READBIN.as_ptr());
        if fd.is_null() {
            // Something went wrong — perhaps a file name with a special
            // character in it.
            if !flags.has(ExpandFlags::SILENT) {
                msg(gettext((&raw const e_wildexpand).cast()), 0);
                msg_start(); // do not overwrite this message
            }
            xfree(tempname.cast());
            return Read::NotFound;
        }
        if fseek(fd, 0, SEEK_END) < 0 {
            xfree(tempname.cast());
            fclose(fd);
            return Read::Failed;
        }
        let templen = ftell(fd);
        if templen < 0 {
            xfree(tempname.cast());
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
            semsg_c!(gettext((&raw const e_cant_read_file_str).cast()), tempname);
            xfree(tempname.cast());
            return Read::Failed;
        }
        Read::Content(buffer)
    }
}

/// Upstream's `notfound:` label: under `ExpandFlags::NOTFOUND` the patterns themselves
/// become the answer.
///
/// # Safety
/// As [`os_expand_wildcards`].
unsafe fn not_found(
    num_pat: c_int,
    pat: *mut *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    flags: ExpandFlags,
) -> c_int {
    if !flags.has(ExpandFlags::NOTFOUND) {
        return FAIL;
    }
    // SAFETY: the caller's contract.
    unsafe { save_patterns(num_pat, pat, num_file, file) };
    OK
}
