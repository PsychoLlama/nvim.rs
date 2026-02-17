//! Context-setting helper functions for command-line completion.
//!
//! These are string parsers that determine what type of completion is
//! appropriate for a given command-line position.

use libc::{c_char, c_int};

use crate::context::ExpandContext;
use crate::ExpandHandle;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // expand_T field setters
    fn nvim_expand_set_context(xp: ExpandHandle, context: c_int);
    fn nvim_expand_set_pattern(xp: ExpandHandle, pattern: *mut c_char);
    fn nvim_expand_set_pattern_len(xp: ExpandHandle, len: usize);
    fn nvim_expand_set_search_dir(xp: ExpandHandle, dir: c_int);
    fn nvim_expand_set_shell(xp: ExpandHandle, shell: c_int);

    // String utility functions
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn skiptowhite(p: *const c_char) -> *const c_char;
    fn skipdigits(p: *const c_char) -> *mut c_char;
    fn skip_regexp(p: *mut c_char, delim: c_int, magic: c_int) -> *mut c_char;
    fn rs_skip_vimgrep_pat(p: *mut c_char, s: *mut *mut c_char, flags: *mut c_int) -> *mut c_char;
    fn find_nextcmd(p: *const c_char) -> *const c_char;
    fn ends_excmd(c: c_int) -> c_int;
    fn rs_magic_isset() -> c_int;
    fn match_user(name: *const c_char) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // Character class checks (inline functions in C, so need wrappers)
    fn nvim_cmdexpand_ascii_iswhite(c: c_int) -> c_int;
    fn nvim_cmdexpand_vim_isfilec_or_wc(c: c_int) -> c_int;
    fn nvim_cmdexpand_vim_isIDc(c: c_int) -> c_int;

    // Context-setting for echohl (used by set_context_in_match_cmd)
    fn set_context_in_echohl_cmd(xp: ExpandHandle, arg: *const c_char);

    // Cmdline info for set_context_with_pattern
    fn nvim_cmdexpand_get_cmdpos() -> c_int;
    fn nvim_cmdexpand_get_cmdbuff() -> *mut c_char;
    fn nvim_cmdexpand_parse_pattern_and_range(skiplen: *mut c_int, patlen: *mut c_int) -> c_int;
    fn nvim_cmdexpand_emsg_off_inc();
    fn nvim_cmdexpand_emsg_off_dec();

    // Static variable accessors
    fn nvim_cmdexpand_set_breakpt_expand_what(val: c_int);
    fn nvim_cmdexpand_set_filetype_expand_what(val: c_int);

    // expand_T pattern getter
    fn nvim_cmdexpand_get_xp_pattern(xp: ExpandHandle) -> *mut c_char;
}

// =============================================================================
// Constants
// =============================================================================

/// `FORWARD` direction (1).
const FORWARD: c_int = 1;

/// Check if byte is an ASCII digit (inline in C, so replicated here).
#[inline]
const fn is_ascii_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Breakpoint expand-what values.
const EXP_BREAKPT_ADD: c_int = 0;
const EXP_BREAKPT_DEL: c_int = 1;
const EXP_PROFDEL: c_int = 2;

/// Filetype expand-what values.
const EXP_FILETYPECMD_ALL: c_int = 0;
const EXP_FILETYPECMD_PLUGIN: c_int = 1;
const EXP_FILETYPECMD_INDENT: c_int = 2;
const EXP_FILETYPECMD_ONOFF: c_int = 3;

/// Flags for filetype subcommand parsing.
const EXPAND_FILETYPECMD_PLUGIN_FLAG: c_int = 0x01;
const EXPAND_FILETYPECMD_INDENT_FLAG: c_int = 0x02;

/// Breakpoint command type passed from C wrapper.
const BREAKPT_CMD_ADD: c_int = 0;
const BREAKPT_CMD_DEL: c_int = 1;

// =============================================================================
// `find_cmd_after_global_cmd`
// =============================================================================

/// Returns a pointer to the next command after a `:global` or `:v` command.
///
/// # Safety
///
/// `arg` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn rs_find_cmd_after_global_cmd(arg: *const c_char) -> *const c_char {
    let delim = *arg as u8;
    let mut p = arg;
    if delim != 0 {
        p = p.add(1); // Skip delimiter
    }

    while *p != 0 && *p as u8 != delim {
        if *p as u8 == b'\\' && *p.add(1) != 0 {
            p = p.add(1);
        }
        p = p.add(1);
    }
    if *p != 0 {
        return p.add(1);
    }
    std::ptr::null()
}

// =============================================================================
// `find_cmd_after_substitute_cmd`
// =============================================================================

/// Returns a pointer to the next command after a `:substitute` or `:&` command.
///
/// # Safety
///
/// `arg` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_find_cmd_after_substitute_cmd(arg: *const c_char) -> *const c_char {
    let delim = *arg as u8;
    let mut p = arg;

    if delim != 0 {
        // Skip "from" part.
        p = p.add(1);
        p = skip_regexp(p.cast_mut(), c_int::from(delim), rs_magic_isset());

        if *p != 0 && *p as u8 == delim {
            // Skip "to" part.
            p = p.add(1);
            while *p != 0 && *p as u8 != delim {
                if *p as u8 == b'\\' && *p.add(1) != 0 {
                    p = p.add(1);
                }
                p = p.add(1);
            }
            if *p != 0 {
                p = p.add(1); // Skip delimiter
            }
        }
    }

    // Skip flags until we hit |, ", # or NUL
    while *p != 0 {
        let c = *p as u8;
        if c == b'|' || c == b'"' || c == b'#' {
            break;
        }
        p = p.add(1);
    }
    if *p != 0 {
        return p;
    }
    std::ptr::null()
}

// =============================================================================
// `find_cmd_after_isearch_cmd`
// =============================================================================

/// Returns a pointer to the next command after a `:isearch`/`:dsearch` etc.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `arg` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_find_cmd_after_isearch_cmd(
    xp: ExpandHandle,
    arg: *const c_char,
) -> *const c_char {
    // Skip count.
    let mut p: *const c_char = skipwhite(skipdigits(arg));
    if *p as u8 != b'/' {
        return std::ptr::null();
    }

    // Match regexp, not just whole words.
    p = p.add(1);
    while *p != 0 && *p as u8 != b'/' {
        if *p as u8 == b'\\' && *p.add(1) != 0 {
            p = p.add(1);
        }
        p = p.add(1);
    }
    if *p != 0 {
        p = skipwhite(p.add(1));

        // Check for trailing command separator characters.
        if *p != 0 {
            let c = *p as u8;
            if c == b'|' || c == b'"' || c == b'\n' {
                return p;
            }
        }
        nvim_expand_set_context(xp, ExpandContext::Nothing.to_raw());
    }
    std::ptr::null()
}

// =============================================================================
// `set_context_in_argopt`
// =============================================================================

/// Set the completion context for the `++opt=arg` argument.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `arg` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_in_argopt(
    xp: ExpandHandle,
    arg: *const c_char,
) -> *const c_char {
    let p = nvim_vim_strchr(arg, c_int::from(b'='));
    if p.is_null() {
        nvim_expand_set_pattern(xp, arg.cast_mut());
    } else {
        nvim_expand_set_pattern(xp, p.add(1).cast_mut());
    }
    nvim_expand_set_context(xp, ExpandContext::Argopt.to_raw());
    std::ptr::null()
}

// =============================================================================
// `set_context_in_filter_cmd`
// =============================================================================

/// Set the completion context for the `:filter` command.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `arg` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_in_filter_cmd(
    xp: ExpandHandle,
    arg: *const c_char,
) -> *const c_char {
    let mut p = arg;
    if *p != 0 {
        p = rs_skip_vimgrep_pat(p.cast_mut(), std::ptr::null_mut(), std::ptr::null_mut());
    }
    if p.is_null() || *p == 0 {
        nvim_expand_set_context(xp, ExpandContext::Nothing.to_raw());
        return std::ptr::null();
    }
    skipwhite(p)
}

// =============================================================================
// `set_context_in_match_cmd`
// =============================================================================

/// Set the completion context for the `:match` command.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `arg` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_in_match_cmd(
    xp: ExpandHandle,
    arg: *const c_char,
) -> *const c_char {
    let mut p = arg;
    if *p == 0 || ends_excmd(c_int::from(*p as u8)) == 0 {
        // also complete "None"
        set_context_in_echohl_cmd(xp, p);
        p = skipwhite(skiptowhite(p));
        if *p != 0 {
            nvim_expand_set_context(xp, ExpandContext::Nothing.to_raw());
            p = skip_regexp(p.add(1).cast_mut(), c_int::from(*p as u8), rs_magic_isset());
        }
    }
    find_nextcmd(p)
}

// =============================================================================
// `set_context_in_unlet_cmd`
// =============================================================================

/// Set the completion context for the `:unlet` command.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `arg` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_in_unlet_cmd(
    xp: ExpandHandle,
    arg: *const c_char,
) -> *const c_char {
    let mut a = arg;
    loop {
        let p = libc::strchr(a, c_int::from(b' '));
        if p.is_null() {
            break;
        }
        a = p.add(1);
    }

    nvim_expand_set_context(xp, ExpandContext::UserVars.to_raw());
    nvim_expand_set_pattern(xp, a.cast_mut());

    if *a as u8 == b'$' {
        nvim_expand_set_context(xp, ExpandContext::EnvVars.to_raw());
        nvim_expand_set_pattern(xp, a.add(1).cast_mut());
    }

    std::ptr::null()
}

// =============================================================================
// `set_context_in_lang_cmd`
// =============================================================================

/// Set the completion context for the `:language` command.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `arg` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_in_lang_cmd(
    xp: ExpandHandle,
    arg: *const c_char,
) -> *const c_char {
    let p = skiptowhite(arg);
    if *p == 0 {
        nvim_expand_set_context(xp, ExpandContext::Language.to_raw());
        nvim_expand_set_pattern(xp, arg.cast_mut());
    } else {
        let len = p.offset_from(arg) as usize;
        let matches_keyword =
            |kw: &[u8]| len == kw.len() && libc::strncmp(arg, kw.as_ptr().cast(), len) == 0;
        if matches_keyword(b"messages")
            || matches_keyword(b"ctype")
            || matches_keyword(b"time")
            || matches_keyword(b"collate")
        {
            nvim_expand_set_context(xp, ExpandContext::Locales.to_raw());
            nvim_expand_set_pattern(xp, skipwhite(p));
        } else {
            nvim_expand_set_context(xp, ExpandContext::Nothing.to_raw());
        }
    }
    std::ptr::null()
}

// =============================================================================
// `set_context_in_breakadd_cmd`
// =============================================================================

/// Set the completion context for the `:breakadd` command.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `arg` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_in_breakadd_cmd(
    xp: ExpandHandle,
    arg: *const c_char,
    breakpt_cmd_type: c_int,
) -> *const c_char {
    nvim_expand_set_context(xp, ExpandContext::Breakpoint.to_raw());
    nvim_expand_set_pattern(xp, arg.cast_mut());

    if breakpt_cmd_type == BREAKPT_CMD_ADD {
        nvim_cmdexpand_set_breakpt_expand_what(EXP_BREAKPT_ADD);
    } else if breakpt_cmd_type == BREAKPT_CMD_DEL {
        nvim_cmdexpand_set_breakpt_expand_what(EXP_BREAKPT_DEL);
    } else {
        nvim_cmdexpand_set_breakpt_expand_what(EXP_PROFDEL);
    }

    let p = skipwhite(arg);
    if *p == 0 {
        return std::ptr::null();
    }

    if libc::strncmp(p, c"file ".as_ptr(), 5) == 0 || libc::strncmp(p, c"func ".as_ptr(), 5) == 0 {
        let is_file = *p as u8 == b'f' && *p.add(1) as u8 == b'i'; // "file" vs "func"
        let mut q = skipwhite(p.add(4));

        // skip line number (if specified)
        if is_ascii_digit(*q as u8) {
            q = skipdigits(q);
            if *q as u8 != b' ' {
                nvim_expand_set_context(xp, ExpandContext::Nothing.to_raw());
                return std::ptr::null();
            }
            q = skipwhite(q);
        }
        if is_file {
            nvim_expand_set_context(xp, ExpandContext::Files.to_raw());
        } else {
            nvim_expand_set_context(xp, ExpandContext::UserFunc.to_raw());
        }
        nvim_expand_set_pattern(xp, q);
    } else if libc::strncmp(p, c"expr ".as_ptr(), 5) == 0 {
        nvim_expand_set_context(xp, ExpandContext::Expression.to_raw());
        nvim_expand_set_pattern(xp, skipwhite(p.add(5)));
    }

    std::ptr::null()
}

// =============================================================================
// `set_context_in_scriptnames_cmd`
// =============================================================================

/// Set the completion context for the `:scriptnames` command.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `arg` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_in_scriptnames_cmd(
    xp: ExpandHandle,
    arg: *const c_char,
) -> *const c_char {
    nvim_expand_set_context(xp, ExpandContext::Nothing.to_raw());
    nvim_expand_set_pattern(xp, std::ptr::null_mut());

    let p = skipwhite(arg);
    if is_ascii_digit(*p as u8) {
        return std::ptr::null();
    }

    nvim_expand_set_context(xp, ExpandContext::Scriptnames.to_raw());
    nvim_expand_set_pattern(xp, p);

    std::ptr::null()
}

// =============================================================================
// `set_context_in_filetype_cmd`
// =============================================================================

/// Set the completion context for the `:filetype` command.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `arg` must be a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_in_filetype_cmd(
    xp: ExpandHandle,
    arg: *const c_char,
) -> *const c_char {
    nvim_expand_set_context(xp, ExpandContext::Filetypecmd.to_raw());
    nvim_expand_set_pattern(xp, arg.cast_mut());
    nvim_cmdexpand_set_filetype_expand_what(EXP_FILETYPECMD_ALL);

    let mut p = skipwhite(arg);
    if *p == 0 {
        return std::ptr::null();
    }

    let mut val: c_int = 0;

    loop {
        if libc::strncmp(p, c"plugin".as_ptr(), 6) == 0 {
            val |= EXPAND_FILETYPECMD_PLUGIN_FLAG;
            p = skipwhite(p.add(6));
            continue;
        }
        if libc::strncmp(p, c"indent".as_ptr(), 6) == 0 {
            val |= EXPAND_FILETYPECMD_INDENT_FLAG;
            p = skipwhite(p.add(6));
            continue;
        }
        break;
    }

    if (val & EXPAND_FILETYPECMD_PLUGIN_FLAG) != 0 && (val & EXPAND_FILETYPECMD_INDENT_FLAG) != 0 {
        nvim_cmdexpand_set_filetype_expand_what(EXP_FILETYPECMD_ONOFF);
    } else if (val & EXPAND_FILETYPECMD_PLUGIN_FLAG) != 0 {
        nvim_cmdexpand_set_filetype_expand_what(EXP_FILETYPECMD_INDENT);
    } else if (val & EXPAND_FILETYPECMD_INDENT_FLAG) != 0 {
        nvim_cmdexpand_set_filetype_expand_what(EXP_FILETYPECMD_PLUGIN);
    }

    nvim_expand_set_pattern(xp, p);

    std::ptr::null()
}

// =============================================================================
// `set_context_with_pattern`
// =============================================================================

/// Sets the completion context for commands that involve a search pattern
/// and a line range (e.g., `:s`, `:g`, `:v`).
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_with_pattern(xp: ExpandHandle) {
    nvim_cmdexpand_emsg_off_inc();
    let mut skiplen: c_int = 0;
    let mut patlen: c_int = 0;
    let retval = nvim_cmdexpand_parse_pattern_and_range(&raw mut skiplen, &raw mut patlen);
    nvim_cmdexpand_emsg_off_dec();

    let cmdpos = nvim_cmdexpand_get_cmdpos();
    if retval == 0 || cmdpos <= skiplen || cmdpos > skiplen + patlen {
        return;
    }

    let cmdbuff = nvim_cmdexpand_get_cmdbuff();
    nvim_expand_set_pattern(xp, cmdbuff.add(skiplen as usize));
    nvim_expand_set_pattern_len(xp, (cmdpos - skiplen) as usize);
    nvim_expand_set_context(xp, ExpandContext::PatternInBuf.to_raw());
    nvim_expand_set_search_dir(xp, FORWARD);
}

// =============================================================================
// `set_context_for_wildcard_arg`
// =============================================================================

/// Set the completion context for a command argument with wildcard characters.
///
/// # Safety
///
/// `xp` must be a valid `expand_T` handle. `arg` must be a valid C string.
/// `is_shell_cmd` should be 1 if usefilter or cmdidx is `CMD_bang`/`CMD_terminal`.
/// `complp` must be a valid pointer to a mutable int.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_for_wildcard_arg(
    arg: *const c_char,
    is_shell_cmd: c_int,
    xp: ExpandHandle,
    complp: *mut c_int,
) {
    let mut in_quote = false;
    let mut bow: *const c_char = std::ptr::null();

    let pattern_start = skipwhite(arg);
    nvim_expand_set_pattern(xp, pattern_start);
    let mut p: *const c_char = pattern_start;

    while *p != 0 {
        let c = utf_ptr2char(p);
        if c == c_int::from(b'\\') && *p.add(1) != 0 {
            p = p.add(1);
        } else if c == c_int::from(b'`') {
            if !in_quote {
                nvim_expand_set_pattern(xp, p.cast_mut());
                bow = p.add(1);
            }
            in_quote = !in_quote;
        } else if c == c_int::from(b'|')
            || c == c_int::from(b'\n')
            || c == c_int::from(b'"')
            || nvim_cmdexpand_ascii_iswhite(c) != 0
        {
            let mut len: usize = 0;
            while *p != 0 {
                let c2 = utf_ptr2char(p);
                if c2 == c_int::from(b'`') || nvim_cmdexpand_vim_isfilec_or_wc(c2) != 0 {
                    break;
                }
                len = utfc_ptr2len(p) as usize;
                p = p.add(utfc_ptr2len(p) as usize);
            }
            if in_quote {
                bow = p;
            } else {
                nvim_expand_set_pattern(xp, p.cast_mut());
            }
            p = p.wrapping_sub(len);
        }
        p = p.add(utfc_ptr2len(p) as usize);
    }

    // If we are still inside the quotes, and we passed a space, just
    // expand from there.
    if !bow.is_null() && in_quote {
        nvim_expand_set_pattern(xp, bow.cast_mut());
    }
    nvim_expand_set_context(xp, ExpandContext::Files.to_raw());

    // For a shell command more chars need to be escaped.
    if is_shell_cmd != 0 || *complp == ExpandContext::Shellcmdline.to_raw() {
        #[cfg(not(windows))]
        nvim_expand_set_shell(xp, 1);
        // When still after the command name expand executables.
        if nvim_cmdexpand_get_xp_pattern(xp) == skipwhite(arg) {
            nvim_expand_set_context(xp, ExpandContext::Shellcmd.to_raw());
        }
    }

    // Check for environment variable.
    let pat = nvim_cmdexpand_get_xp_pattern(xp);
    if *pat as u8 == b'$' {
        let mut q = pat.add(1);
        while *q != 0 {
            if nvim_cmdexpand_vim_isIDc(c_int::from(*q as u8)) == 0 {
                break;
            }
            q = q.add(1);
        }
        if *q == 0 {
            nvim_expand_set_context(xp, ExpandContext::EnvVars.to_raw());
            nvim_expand_set_pattern(xp, pat.add(1));
            if *complp != ExpandContext::UserDefined.to_raw()
                && *complp != ExpandContext::UserList.to_raw()
            {
                *complp = ExpandContext::EnvVars.to_raw();
            }
        }
    }

    // Check for user names.
    let pat = nvim_cmdexpand_get_xp_pattern(xp);
    if *pat as u8 == b'~' {
        let mut q = pat.add(1);
        while *q != 0 && *q as u8 != b'/' {
            q = q.add(1);
        }
        if *q == 0 && q > pat.add(1) && match_user(pat.add(1)) >= 1 {
            nvim_expand_set_context(xp, ExpandContext::User.to_raw());
            nvim_expand_set_pattern(xp, pat.add(1));
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(EXP_BREAKPT_ADD, 0);
        assert_eq!(EXP_BREAKPT_DEL, 1);
        assert_eq!(EXP_PROFDEL, 2);
    }
}
