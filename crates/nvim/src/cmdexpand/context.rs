//! Classifying the command line: which completion applies where.
//!
//! [`set_expand_context`] is the entry point; [`set_cmd_index`] parses the
//! range and the command name, and the `set_context_in_*` helpers handle the
//! commands whose argument is not a plain file name.  The big per-command
//! switch is in [`super::cmdname`].

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::guard::Suppress;
use crate::types::CmdIdx;
use crate::types::{ExpandContext, NUL};
use core::ffi::{c_char, c_int};
use core::ptr;

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

/// Set the completion context in `expand` from the command line being edited.
///
/// `expand.xp_context` ends up one of the `EXPAND_*` values, with `xp_pattern`
/// pointing at the text to expand.
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
pub unsafe fn set_expand_context(expand: *mut Expand) {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    let ccline = Cc::current();

    // Handle search commands: '/' or '?'.
    if (ccline.cmdfirstc == '/' as c_int || ccline.cmdfirstc == '?' as c_int)
        && may_expand_pattern.get()
    {
        expand.xp_context = ExpandContext::PatternInBuf;
        expand.xp_search_dir = if ccline.cmdfirstc == '/' as c_int {
            FORWARD
        } else {
            BACKWARD
        };
        expand.xp_pattern = ccline.text();
        expand.xp_pattern_len = ccline.cmdpos as size_t;
        search_first_line.set(0); // Search entire buffer
        return;
    }

    // Only handle ':', '>', or '=' command-lines, or expression input.
    if ccline.cmdfirstc != ':' as c_int
        && ccline.cmdfirstc != '>' as c_int
        && ccline.cmdfirstc != '=' as c_int
        && ccline.input_fn == 0
    {
        expand.xp_context = ExpandContext::Nothing;
        return;
    }

    // Fallback to command-line expansion.
    unsafe {
        set_cmd_context(
            expand.raw(),
            ccline.text(),
            ccline.len(),
            ccline.cmdpos,
            true,
        )
    };
}

/// Set the index of a built-in or user defined command `cmd` in `eap->cmdidx`.
///
/// For user defined commands the completion context is set in `expand` and the
/// completion flags in `complp`.
///
/// Returns a pointer to the text after the command, or NULL for failure.
///
/// # Safety
///
/// `cmd` must point at a NUL-terminated string. `args` must point at the
/// command's `ExArg`. `expand` must point at a live `Expand` context,
/// unaliased for the call. `complp` must point at a live `ExpandContext`,
/// unaliased for the call.
pub(crate) unsafe fn set_cmd_index(
    cmd: *const c_char,
    args: *mut ExArg,
    expand: *mut Expand,
    complp: *mut ExpandContext,
) -> *const c_char {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    // Both name scans are this loop.  Two monomorphic closures rather
    // than one taking a `fn(u8) -> bool`: the predicate is called once
    // per byte of every command line, and behind a function pointer it
    // cannot inline (`cmdctx` measured +11% that way).
    let skip_alpha = |mut p: *const c_char| {
        while is_cmd_alpha(unsafe { *p } as u8) {
            p = unsafe { p.add(1) };
        }
        p
    };
    let skip_alnum = |mut p: *const c_char| {
        while is_cmd_alnum(unsafe { *p } as u8) {
            p = unsafe { p.add(1) };
        }
        p
    };

    let mut p: *const c_char;
    let fuzzy = unsafe { cmdline_fuzzy_complete(cmd) };

    // Isolate the command and search for it in the command table.
    // Exceptions:
    // - the 'k' command can directly be followed by any character, but do
    //   accept "keepmarks", "keepalt" and "keepjumps".  Bypass also when
    //   'ignorecase' is set so a lowercase ":kz" still completes a user
    //   command like :Kz, and for fuzzy matching as that can find matches
    //   anywhere in the command name.
    // - the 's' command can be followed directly by 'c', 'g', 'i', 'I' or
    //   'r'.
    if !fuzzy
        && p_ic.get() == 0
        && unsafe { *cmd } as c_int == 'k' as c_int
        && unsafe { *cmd.add(1) } as c_int != 'e' as c_int
    {
        unsafe { (*args).cmdidx = CmdIdx::k };
        p = unsafe { cmd.add(1) };
    } else {
        p = skip_alpha(cmd);
        // A user command may contain digits.
        if (unsafe { *cmd } as u8).is_ascii_uppercase() {
            p = skip_alnum(p);
        }
        // For python 3.x: ":py3*" commands completion.
        if unsafe { *cmd } as c_int == 'p' as c_int
            && unsafe { *cmd.add(1) } as c_int == 'y' as c_int
            && p == unsafe { cmd.add(2) }
            && unsafe { *p } as c_int == '3' as c_int
        {
            p = skip_alpha(unsafe { p.add(1) });
        }
        // Check for non-alpha command.
        if p == cmd && !unsafe { vim_strchr(c"@*!=><&~#".as_ptr(), *p as u8 as c_int) }.is_null() {
            p = unsafe { p.add(1) };
        }
        let len = unsafe { p.offset_from(cmd) } as size_t;

        if len == 0 {
            expand.xp_context = ExpandContext::Unsuccessful;
            return ptr::null();
        }

        unsafe { (*args).cmdidx = excmd_get_cmdidx(cmd, len) };

        // User defined commands support alphanumeric characters.  Also
        // when doing fuzzy expansion for non-shell commands.
        if (unsafe { *cmd } as u8).is_ascii_uppercase()
            || (fuzzy && unsafe { (*args).cmdidx } != CmdIdx::bang && unsafe { *p } as c_int != NUL)
        {
            p = skip_alnum(p);
        }
    }

    // If the cursor is touching the command, and it ends in an
    // alphanumeric character, complete the command name.
    if unsafe { *p } as c_int == NUL && (unsafe { *p.sub(1) } as u8).is_ascii_alphanumeric() {
        return ptr::null();
    }

    if unsafe { (*args).cmdidx } == CmdIdx::SIZE {
        if unsafe { *cmd } as c_int == 's' as c_int
            && !unsafe { vim_strchr(c"cgriI".as_ptr(), *cmd.add(1) as u8 as c_int) }.is_null()
        {
            unsafe { (*args).cmdidx = CmdIdx::substitute };
            p = unsafe { cmd.add(1) };
        } else if (unsafe { *cmd } as u8).is_ascii_uppercase() {
            unsafe { (*args).cmd = cmd as *mut c_char };
            p = unsafe {
                find_ucmd(
                    args,
                    p as *mut c_char,
                    ptr::null_mut(),
                    expand.raw(),
                    complp,
                )
            };
            if p.is_null() {
                unsafe { (*args).cmdidx = CmdIdx::SIZE }; // Ambiguous user command.
            }
        }
    }
    if unsafe { (*args).cmdidx } == CmdIdx::SIZE {
        // Not still touching the command and it was an illegal one.
        expand.xp_context = ExpandContext::Unsuccessful;
        return ptr::null();
    }

    p
}

/// Set the completion context for a command argument with wild card
/// characters.
///
/// # Safety
///
/// `args` must point at the command's `ExArg`. `arg` must point at a NUL-
/// terminated string. `expand` must point at a live `Expand` context,
/// unaliased for the call. `complp` must point at a live `ExpandContext`,
/// unaliased for the call.
pub(crate) unsafe fn set_context_for_wildcard_arg(
    args: *mut ExArg,
    arg: *const c_char,
    usefilter: bool,
    expand: *mut Expand,
    complp: *mut ExpandContext,
) {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    let mut in_quote = false;
    let mut bow: *const c_char = ptr::null(); // Beginning of word.
    let mut len: size_t;

    // Allow spaces within back-quotes to count as part of the argument
    // being expanded.
    expand.xp_pattern = unsafe { skipwhite(arg) };
    let mut p: *const c_char = expand.xp_pattern;
    while unsafe { *p } as c_int != NUL {
        let mut c = unsafe { utf_ptr2char(p) };
        if c == '\\' as c_int && unsafe { *p.add(1) } as c_int != NUL {
            p = unsafe { p.add(1) };
        } else if c == '`' as c_int {
            if !in_quote {
                expand.xp_pattern = p as *mut c_char;
                bow = unsafe { p.add(1) };
            }
            in_quote = !in_quote;
            // An argument can contain just about everything, except
            // characters that end the command and white space.
        } else if c == '|' as c_int || c == '\n' as c_int || c == '"' as c_int || ascii_iswhite(c) {
            len = 0; // avoid getting stuck when space is in 'isfname'
            while unsafe { *p } as c_int != NUL {
                c = unsafe { utf_ptr2char(p) };
                if c == '`' as c_int || unsafe { vim_isfilec_or_wc(c) } {
                    break;
                }
                len = unsafe { utfc_ptr2len(p) } as size_t;
                p = unsafe { p.add(utfc_ptr2len(p) as usize) };
            }
            if in_quote {
                bow = p;
            } else {
                expand.xp_pattern = p as *mut c_char;
            }
            p = unsafe { p.sub(len as usize) };
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }

    // If we are still inside the quotes, and we passed a space, just
    // expand from there.
    if !bow.is_null() && in_quote {
        expand.xp_pattern = bow as *mut c_char;
    }
    expand.xp_context = ExpandContext::Files;

    // For a shell command more chars need to be escaped. `:!` and
    // `:terminal` run one; so does an explicitly shell-flavoured context.
    // SAFETY: the caller's argument block, when there is one.
    let runs_a_shell =
        !args.is_null() && matches!(unsafe { (*args).cmdidx }, CmdIdx::bang | CmdIdx::terminal);
    if usefilter || runs_a_shell || unsafe { *complp } == ExpandContext::ShellCmdLine {
        expand.xp_shell = true;
        // When still after the command name expand executables.
        if expand.xp_pattern == unsafe { skipwhite(arg) } {
            expand.xp_context = ExpandContext::ShellCmd;
        }
    }

    // Check for environment variable.
    if unsafe { *expand.xp_pattern } as c_int == '$' as c_int {
        p = unsafe { expand.xp_pattern.add(1) };
        while unsafe { *p } as c_int != NUL {
            if !unsafe { vim_is_ident_char(*p as u8 as c_int) } {
                break;
            }
            p = unsafe { p.add(1) };
        }
        if unsafe { *p } as c_int == NUL {
            expand.xp_context = ExpandContext::EnvVars;
            unsafe { expand.xp_pattern = expand.xp_pattern.add(1) };
            // Avoid that the assignment uses ExpandContext::Files again.
            if unsafe { *complp } != ExpandContext::UserDefined
                && unsafe { *complp } != ExpandContext::UserList
            {
                unsafe { *complp = ExpandContext::EnvVars };
            }
        }
    }
    // Check for user names.
    if unsafe { *expand.xp_pattern } as c_int == '~' as c_int {
        p = unsafe { expand.xp_pattern.add(1) };
        while unsafe { *p } as c_int != NUL && unsafe { *p } as c_int != '/' as c_int {
            p = unsafe { p.add(1) };
        }
        // Complete ~user only if it partially matches a user name.  A full
        // match ~user<Tab> will be replaced by the user's home directory,
        // i.e. something like ~user<Tab> -> /home/user/.
        let user = unsafe { expand.xp_pattern.add(1) };
        if unsafe { *p } as c_int == NUL
            && p > user as *const c_char
            && match_user(unsafe { CStr::from_ptr(user) }) != UserMatch::None
        {
            expand.xp_context = ExpandContext::User;
            unsafe { expand.xp_pattern = expand.xp_pattern.add(1) };
        }
    }
}

/// Set the completion context for the `++opt=arg` argument.  Always NULL.
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn set_context_in_argopt(
    expand: *mut Expand,
    arg: *const c_char,
) -> *const c_char {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    let p = unsafe { vim_strchr(arg, '=' as c_int) };
    expand.xp_pattern = if p.is_null() {
        arg as *mut c_char
    } else {
        unsafe { p.add(1) }
    };
    expand.xp_context = ExpandContext::Argopt;
    ptr::null()
}

/// Set the completion context for the `:filter` command.
///
/// Returns a pointer to the next command after the `:filter` command.
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn set_context_in_filter_cmd(
    expand: *mut Expand,
    mut arg: *const c_char,
) -> *const c_char {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    if unsafe { *arg } as c_int != NUL {
        arg = unsafe { skip_vimgrep_pat(arg as *mut c_char, ptr::null_mut(), ptr::null_mut()) };
    }
    if arg.is_null() || unsafe { *arg } as c_int == NUL {
        expand.xp_context = ExpandContext::Nothing;
        return ptr::null();
    }
    unsafe { skipwhite(arg) }
}

/// Set the completion context for the `:match` command.
///
/// Returns a pointer to the next command after the `:match` command.
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn set_context_in_match_cmd(
    expand: *mut Expand,
    mut arg: *const c_char,
) -> *const c_char {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    if unsafe { *arg } as c_int == NUL || ends_excmd(unsafe { *arg } as c_int) == 0 {
        // Also complete "None".
        set_context_in_echohl_cmd(&mut expand, arg);
        arg = unsafe { skipwhite(skiptowhite(arg)) };
        if unsafe { *arg } as c_int != NUL {
            expand.xp_context = ExpandContext::Nothing;
            arg = unsafe {
                skip_regexp(
                    (arg as *mut c_char).add(1),
                    *arg as u8 as c_int,
                    magic_isset() as c_int,
                )
            };
        }
    }
    unsafe { find_nextcmd(arg) }
}

/// The next command after a `:global` or a `:v` command, or NULL if there is
/// none.
///
/// # Safety
///
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn find_cmd_after_global_cmd(mut arg: *const c_char) -> *const c_char {
    let delim = unsafe { *arg } as u8 as c_int; // Get the delimiter.
    if delim != 0 {
        arg = unsafe { arg.add(1) }; // Skip delimiter if there is one.
    }

    while unsafe { *arg } as c_int != NUL && unsafe { *arg } as u8 as c_int != delim {
        if unsafe { *arg } as c_int == '\\' as c_int && unsafe { *arg.add(1) } as c_int != NUL {
            arg = unsafe { arg.add(1) };
        }
        arg = unsafe { arg.add(1) };
    }
    if unsafe { *arg } as c_int != NUL {
        return unsafe { arg.add(1) };
    }

    ptr::null()
}

/// The next command after a `:substitute` or a `:&` command, or NULL if there
/// is none.
///
/// # Safety
///
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn find_cmd_after_substitute_cmd(mut arg: *const c_char) -> *const c_char {
    let delim = unsafe { *arg } as u8 as c_int;
    if delim != 0 {
        // Skip "from" part.
        arg = unsafe { arg.add(1) };
        arg = unsafe { skip_regexp(arg as *mut c_char, delim, magic_isset() as c_int) };

        if unsafe { *arg } as c_int != NUL && unsafe { *arg } as c_int == delim {
            // Skip "to" part.
            arg = unsafe { arg.add(1) };
            while unsafe { *arg } as c_int != NUL && unsafe { *arg } as u8 as c_int != delim {
                if unsafe { *arg } as c_int == '\\' as c_int
                    && unsafe { *arg.add(1) } as c_int != NUL
                {
                    arg = unsafe { arg.add(1) };
                }
                arg = unsafe { arg.add(1) };
            }
            if unsafe { *arg } as c_int != NUL {
                // Skip delimiter.
                arg = unsafe { arg.add(1) };
            }
        }
    }
    while unsafe { *arg } as c_int != 0
        && unsafe { strchr(c"|\"#".as_ptr(), *arg as c_int) }.is_null()
    {
        arg = unsafe { arg.add(1) };
    }
    if unsafe { *arg } as c_int != NUL {
        return arg;
    }

    ptr::null()
}

/// The next command after a `:isearch`/`:dsearch`/`:ilist`/`:dlist`/`:ijump`/
/// `:psearch`/`:djump`/`:isplit`/`:dsplit` command, or NULL if there is none.
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn find_cmd_after_isearch_cmd(
    expand: *mut Expand,
    mut arg: *const c_char,
) -> *const c_char {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    // Skip count.
    arg = unsafe { skipwhite(skipdigits(arg)) };
    if unsafe { *arg } as c_int != '/' as c_int {
        return ptr::null();
    }

    // Match regexp, not just whole words.
    arg = unsafe { arg.add(1) };
    while unsafe { *arg } as c_int != 0 && unsafe { *arg } as c_int != '/' as c_int {
        if unsafe { *arg } as c_int == '\\' as c_int && unsafe { *arg.add(1) } as c_int != NUL {
            arg = unsafe { arg.add(1) };
        }
        arg = unsafe { arg.add(1) };
    }
    if unsafe { *arg } != 0 {
        arg = unsafe { skipwhite(arg.add(1)) };

        // Check for trailing illegal characters.
        if unsafe { *arg } as c_int == NUL
            || unsafe { strchr(c"|\"\n".as_ptr(), *arg as c_int) }.is_null()
        {
            expand.xp_context = ExpandContext::Nothing;
        } else {
            return arg;
        }
    }

    ptr::null()
}

/// Set the completion context for the `:unlet` command.  Always NULL.
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn set_context_in_unlet_cmd(
    expand: *mut Expand,
    mut arg: *const c_char,
) -> *const c_char {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    loop {
        expand.xp_pattern = unsafe { strchr(arg, ' ' as c_int) };
        if expand.xp_pattern.is_null() {
            break;
        }
        arg = unsafe { expand.xp_pattern.add(1) };
    }

    expand.xp_context = ExpandContext::UserVars;
    expand.xp_pattern = arg as *mut c_char;

    if unsafe { *expand.xp_pattern } as c_int == '$' as c_int {
        expand.xp_context = ExpandContext::EnvVars;
        unsafe { expand.xp_pattern = expand.xp_pattern.add(1) };
    }

    ptr::null()
}

/// Set the completion context for the `:language` command.  Always NULL.
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn set_context_in_lang_cmd(
    expand: *mut Expand,
    arg: *const c_char,
) -> *const c_char {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    let p = unsafe { skiptowhite(arg) };
    if unsafe { *p } as c_int == NUL {
        expand.xp_context = ExpandContext::Language;
        expand.xp_pattern = arg as *mut c_char;
    } else {
        let len = unsafe { p.offset_from(arg) } as size_t;
        let named = [c"messages", c"ctype", c"time", c"collate"]
            .iter()
            .any(|kind| unsafe { cstr::prefix_eq(arg, kind.as_ptr(), len) });
        if named {
            expand.xp_context = ExpandContext::Locales;
            expand.xp_pattern = unsafe { skipwhite(p) };
        } else {
            expand.xp_context = ExpandContext::Nothing;
        }
    }

    ptr::null()
}

/// Set the completion context for the `:breakadd` command.  Always NULL.
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
/// `arg` must point at a NUL-terminated string. `cmdidx` must be an
/// initialized `CmdIdx` whose pointer fields point at live data for the call.
pub(crate) unsafe fn set_context_in_breakadd_cmd(
    expand: *mut Expand,
    arg: *const c_char,
    cmdidx: CmdIdx,
) -> *const c_char {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    expand.xp_context = ExpandContext::Breakpoint;
    expand.xp_pattern = arg as *mut c_char;

    breakpt_expand_what.set(if cmdidx == CmdIdx::breakadd {
        EXP_BREAKPT_ADD
    } else if cmdidx == CmdIdx::breakdel {
        EXP_BREAKPT_DEL
    } else {
        EXP_PROFDEL
    });

    let mut p = unsafe { skipwhite(arg) };
    if unsafe { *p } as c_int == NUL {
        return ptr::null();
    }
    let subcmd_start = p;

    if unsafe { cstr::starts_with(p, b"file ") } || unsafe { cstr::starts_with(p, b"func ") } {
        // :breakadd file [lnum] <filename>
        // :breakadd func [lnum] <funcname>
        p = unsafe { skipwhite(p.add(4)) };

        // Skip line number (if specified).
        if ascii_isdigit(unsafe { *p } as c_int) {
            p = unsafe { skipdigits(p) };
            if unsafe { *p } as c_int != ' ' as c_int {
                expand.xp_context = ExpandContext::Nothing;
                return ptr::null();
            }
            p = unsafe { skipwhite(p) };
        }
        expand.xp_context = if unsafe { cstr::starts_with(subcmd_start, b"file") } {
            ExpandContext::Files
        } else {
            ExpandContext::UserFunc
        };
        expand.xp_pattern = p as *mut c_char;
    } else if unsafe { cstr::starts_with(p, b"expr ") } {
        // :breakadd expr <expression>
        expand.xp_context = ExpandContext::Expression;
        expand.xp_pattern = unsafe { skipwhite(p.add(5)) };
    }

    ptr::null()
}

/// Set the completion context for the `:scriptnames` command.  Always NULL.
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn set_context_in_scriptnames_cmd(
    expand: *mut Expand,
    arg: *const c_char,
) -> *const c_char {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    expand.xp_context = ExpandContext::Nothing;
    expand.xp_pattern = ptr::null_mut();

    let p = unsafe { skipwhite(arg) };
    if ascii_isdigit(unsafe { *p } as c_int) {
        return ptr::null();
    }

    expand.xp_context = ExpandContext::Scriptnames;
    expand.xp_pattern = p;

    ptr::null()
}

/// Set the completion context for the `:filetype` command.  Always NULL.
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
/// `arg` must point at a NUL-terminated string.
pub(crate) unsafe fn set_context_in_filetype_cmd(
    expand: *mut Expand,
    arg: *const c_char,
) -> *const c_char {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    expand.xp_context = ExpandContext::FiletypeCmd;
    expand.xp_pattern = arg as *mut c_char;
    filetype_expand_what.set(FiletypeWhat::All);

    let mut p = unsafe { skipwhite(arg) };
    if unsafe { *p } as c_int == NUL {
        return ptr::null();
    }

    let mut saw_plugin = false;
    let mut saw_indent = false;

    loop {
        if unsafe { cstr::starts_with(p, b"plugin") } {
            saw_plugin = true;
            p = unsafe { skipwhite(p.add(6)) };
            continue;
        }
        if unsafe { cstr::starts_with(p, b"indent") } {
            saw_indent = true;
            p = unsafe { skipwhite(p.add(6)) };
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

    expand.xp_pattern = p;

    ptr::null()
}

/// Set the completion context for commands that involve a search pattern and a
/// line range (e.g. `:s`, `:g`, `:v`).
///
/// # Safety
///
/// `expand` must point at a live `Expand` context, unaliased for the call.
pub(crate) unsafe fn set_context_with_pattern(expand: *mut Expand) {
    // SAFETY: the caller's contract -- `expand` is the live expansion
    // context, which outlives this call.
    let mut expand = unsafe { Xp::new(expand) };
    let ccline = Cc::current();

    let no_emsg = Suppress::emsg();
    let mut skiplen = 0;
    let mut dummy = 0;
    let mut patlen = 0;
    let retval = unsafe {
        parse_pattern_and_range(
            pre_incsearch_pos.get(),
            &raw mut dummy,
            &raw mut skiplen,
            &raw mut patlen,
        )
    };
    drop(no_emsg);

    // Check if cursor is within search pattern.
    if !retval || ccline.cmdpos <= skiplen || ccline.cmdpos > skiplen + patlen {
        return;
    }

    expand.xp_pattern = ccline.at(skiplen);
    expand.xp_pattern_len = (ccline.cmdpos - skiplen) as size_t;
    expand.xp_context = ExpandContext::PatternInBuf;
    expand.xp_search_dir = FORWARD;
}
