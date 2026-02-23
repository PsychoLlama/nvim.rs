//! Shell integration for Ex commands.
//!
//! This module provides types and utilities for shell-related Ex commands:
//! - `:!` - Execute shell command
//! - `:range!cmd` - Filter lines through shell command
//! - `:shell` - Start interactive shell
//!
//! ## Implementation Notes
//!
//! The actual shell execution is performed by Neovim's `call_shell()` function.
//! This module provides:
//! - Type definitions for shell flags and options
//! - Command argument processing helpers
//! - Bang substitution (replacing `!` with previous command)

use std::ffi::{c_char, c_int};

use crate::range::{LineNr, LineRange};

// =============================================================================
// Shell Flags
// =============================================================================

bitflags::bitflags! {
    /// Flags for shell command execution.
    ///
    /// These correspond to the `kShellOpt*` flags in C.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ShellFlags: c_int {
        /// Filtering text (`:range!cmd`)
        const FILTER = 0x01;
        /// Expanding wildcards
        const EXPAND = 0x02;
        /// Redirecting output
        const DO_OUT = 0x04;
        /// Don't print error returned by command
        const SILENT = 0x08;
        /// Read lines and insert into buffer (`:r !cmd`)
        const READ = 0x10;
        /// Write lines from buffer (`:w !cmd`)
        const WRITE = 0x20;
        /// Hide messages
        const HIDE_MESS = 0x40;
    }
}

impl ShellFlags {
    /// Create flags from C integer.
    #[inline]
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        ShellFlags::from_bits_truncate(value)
    }

    /// Convert to C integer.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self.bits()
    }

    /// Check if this is a filter operation.
    #[inline]
    #[must_use]
    pub fn is_filter(&self) -> bool {
        self.contains(ShellFlags::FILTER)
    }

    /// Check if output should be captured.
    #[inline]
    #[must_use]
    pub fn captures_output(&self) -> bool {
        self.contains(ShellFlags::DO_OUT)
    }
}

// =============================================================================
// Shell Command Types
// =============================================================================

/// Type of shell command execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ShellCommandType {
    /// Simple command execution (`:!cmd`)
    #[default]
    Execute,
    /// Filter lines through command (`:range!cmd`)
    Filter,
    /// Read command output into buffer (`:r !cmd`)
    Read,
    /// Write buffer content to command (`:w !cmd`)
    Write,
    /// Interactive shell (`:shell`)
    Interactive,
}

impl ShellCommandType {
    /// Get the default shell flags for this command type.
    #[must_use]
    pub fn default_flags(&self) -> ShellFlags {
        match self {
            ShellCommandType::Execute => ShellFlags::empty(),
            ShellCommandType::Filter => ShellFlags::FILTER,
            ShellCommandType::Read => ShellFlags::READ | ShellFlags::DO_OUT,
            ShellCommandType::Write => ShellFlags::WRITE,
            ShellCommandType::Interactive => ShellFlags::empty(),
        }
    }

    /// Check if this command type uses a line range.
    #[must_use]
    pub const fn uses_range(&self) -> bool {
        matches!(self, ShellCommandType::Filter | ShellCommandType::Write)
    }
}

// =============================================================================
// Shell Command Options
// =============================================================================

/// Options for executing a shell command.
#[derive(Debug, Clone, Default)]
pub struct ShellOptions {
    /// The command to execute (None for interactive shell).
    pub command: Option<String>,
    /// Flags controlling execution.
    pub flags: ShellFlags,
    /// Line range for filter operations.
    pub range: Option<LineRange>,
    /// Whether to use input from buffer.
    pub do_input: bool,
    /// Whether to capture output.
    pub do_output: bool,
}

impl ShellOptions {
    /// Create options for simple command execution.
    #[must_use]
    pub fn execute(command: &str) -> Self {
        Self {
            command: Some(command.to_string()),
            flags: ShellFlags::empty(),
            range: None,
            do_input: false,
            do_output: false,
        }
    }

    /// Create options for filter operation.
    #[must_use]
    pub fn filter(command: &str, range: LineRange) -> Self {
        Self {
            command: Some(command.to_string()),
            flags: ShellFlags::FILTER,
            range: Some(range),
            do_input: true,
            do_output: true,
        }
    }

    /// Create options for reading command output.
    #[must_use]
    pub fn read(command: &str, line: LineNr) -> Self {
        Self {
            command: Some(command.to_string()),
            flags: ShellFlags::READ | ShellFlags::DO_OUT,
            range: Some(LineRange::single(line)),
            do_input: false,
            do_output: true,
        }
    }

    /// Create options for writing to command.
    #[must_use]
    pub fn write(command: &str, range: LineRange) -> Self {
        Self {
            command: Some(command.to_string()),
            flags: ShellFlags::WRITE,
            range: Some(range),
            do_input: true,
            do_output: false,
        }
    }

    /// Create options for interactive shell.
    #[must_use]
    pub fn interactive() -> Self {
        Self {
            command: None,
            flags: ShellFlags::empty(),
            range: None,
            do_input: false,
            do_output: false,
        }
    }

    /// Check if this is an interactive shell.
    #[must_use]
    pub fn is_interactive(&self) -> bool {
        self.command.is_none()
    }
}

// =============================================================================
// Bang Substitution
// =============================================================================

/// Result of bang substitution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BangSubstResult {
    /// Substitution succeeded with the new command.
    Success(String),
    /// No previous command available for substitution.
    NoPreviousCommand,
}

/// Check if a command string contains an unescaped bang.
///
/// Returns the position of the first unescaped `!` if found.
#[must_use]
pub fn find_unescaped_bang(cmd: &str) -> Option<usize> {
    let bytes = cmd.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'!' {
            // Check if it's escaped by a preceding backslash
            if i == 0 || bytes[i - 1] != b'\\' {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Check if a command needs previous command substitution.
///
/// A command needs substitution if:
/// - `forceit` is true (`:!!`)
/// - The command contains an unescaped `!`
#[must_use]
pub fn needs_bang_substitution(cmd: &str, forceit: bool) -> bool {
    forceit || find_unescaped_bang(cmd).is_some()
}

/// Check if a command contains any bang character (escaped or not).
#[must_use]
fn contains_bang(cmd: &str) -> bool {
    cmd.contains('!')
}

/// Substitute bangs in a command string with the previous command.
///
/// - Escaped bangs (`\!`) become literal bangs (`!`)
/// - Unescaped bangs are replaced with `prev_cmd`
/// - If `forceit` is true, prepend the previous command
#[must_use]
pub fn substitute_bangs(cmd: &str, prev_cmd: Option<&str>, forceit: bool) -> BangSubstResult {
    // Fast path: no bangs at all and not forceit
    if !forceit && !contains_bang(cmd) {
        return BangSubstResult::Success(cmd.to_string());
    }

    // Check if we actually need a previous command
    let needs_prev = forceit || find_unescaped_bang(cmd).is_some();

    // We need previous command but don't have it
    let prev = if needs_prev {
        match prev_cmd {
            Some(p) if !p.is_empty() => p,
            _ => return BangSubstResult::NoPreviousCommand,
        }
    } else {
        "" // Not used, but makes the code cleaner
    };

    let mut result = String::with_capacity(cmd.len() + prev.len());

    // If forceit, prepend previous command
    if forceit {
        result.push_str(prev);
    }

    // Process the command string
    let bytes = cmd.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'!' {
            if i > 0 && bytes[i - 1] == b'\\' {
                // Escaped bang - remove the backslash and keep the bang
                // We already added the backslash, so remove it
                result.pop();
                result.push('!');
            } else {
                // Unescaped bang - substitute
                result.push_str(prev);
            }
        } else {
            result.push(bytes[i] as char);
        }
        i += 1;
    }

    BangSubstResult::Success(result)
}

// =============================================================================
// Shell Result
// =============================================================================

/// Result of a shell command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellResult {
    /// Exit code from the shell command.
    pub exit_code: i32,
    /// Number of lines affected (for filter operations).
    pub lines_affected: Option<LineNr>,
}

impl ShellResult {
    /// Create a successful result.
    #[must_use]
    pub const fn success() -> Self {
        Self {
            exit_code: 0,
            lines_affected: None,
        }
    }

    /// Create a result with exit code.
    #[must_use]
    pub const fn with_exit_code(exit_code: i32) -> Self {
        Self {
            exit_code,
            lines_affected: None,
        }
    }

    /// Create a result with lines affected.
    #[must_use]
    pub const fn with_lines(exit_code: i32, lines: LineNr) -> Self {
        Self {
            exit_code,
            lines_affected: Some(lines),
        }
    }

    /// Check if the command succeeded.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}

// =============================================================================
// Shell Error
// =============================================================================

/// Error type for shell operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellError {
    /// Shell commands are not allowed in secure mode.
    SecureMode,
    /// No previous command available for bang substitution.
    NoPreviousCommand,
    /// Invalid line range.
    InvalidRange,
    /// Shell command failed with exit code.
    CommandFailed(i32),
    /// Could not create temporary file.
    TempFileError,
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::SecureMode => write!(f, "E145: Shell commands not allowed in secure mode"),
            ShellError::NoPreviousCommand => write!(f, "E34: No previous command"),
            ShellError::InvalidRange => write!(f, "Invalid line range for filter"),
            ShellError::CommandFailed(code) => {
                write!(f, "Shell command failed with exit code {code}")
            }
            ShellError::TempFileError => write!(f, "Could not create temporary file"),
        }
    }
}

// =============================================================================
// Environment Helpers
// =============================================================================

/// Special characters that need escaping in shell commands.
pub const SHELL_SPECIAL_CHARS: &str = "\t \"&'$;<>()\\|";

/// Check if a character needs escaping in shell commands.
#[inline]
#[must_use]
pub fn needs_shell_escape(c: char) -> bool {
    SHELL_SPECIAL_CHARS.contains(c)
}

/// Count how many characters in a string need shell escaping.
#[must_use]
pub fn count_shell_special(s: &str) -> usize {
    s.chars().filter(|&c| needs_shell_escape(c)).count()
}

// =============================================================================
// FFI Exports
// =============================================================================

extern "C" {
    // make_filter_cmd FFI
    fn nvim_excmds_shell_name_tail() -> *const c_char;
    fn nvim_excmds_get_p_srr() -> *const c_char;
    fn nvim_excmds_get_p_shq() -> *const c_char;
    fn nvim_excmds_xmalloc(size: usize) -> *mut std::ffi::c_void;
}

/// Find the position of an unquoted `|` character in a command string.
///
/// Skips characters inside double-quotes and after backslashes (rem_backslash behavior).
/// Used for non-Unix shell redirection placement.
fn find_pipe_pos(cmd: &[u8]) -> Option<usize> {
    let mut in_quote = false;
    let mut i = 0;
    while i < cmd.len() {
        match cmd[i] {
            b'"' => in_quote = !in_quote,
            b'|' if !in_quote => return Some(i),
            b'\\' if !in_quote => {
                // rem_backslash: skip next char if it would be a backslash-escape
                // On non-Unix this skips the backslash for some chars
                if i + 1 < cmd.len() {
                    i += 1; // skip the escaped char
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Create a shell command from a command string, input redirection file and
/// output redirection file. Replaces the C `make_filter_cmd` function.
///
/// Returns an xmalloc-allocated null-terminated C string.
///
/// # Safety
/// - `cmd` must be a valid null-terminated C string.
/// - `itmp` must be null or a valid null-terminated C string.
/// - `otmp` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_make_filter_cmd(
    cmd: *const c_char,
    itmp: *const c_char,
    otmp: *const c_char,
    do_in: c_int,
) -> *mut c_char {
    use std::ffi::CStr;

    let cmd_bytes = CStr::from_ptr(cmd).to_bytes();
    let itmp_bytes = if itmp.is_null() {
        None
    } else {
        Some(CStr::from_ptr(itmp).to_bytes())
    };
    let otmp_bytes = if otmp.is_null() {
        None
    } else {
        Some(CStr::from_ptr(otmp).to_bytes())
    };
    let do_in = do_in != 0;

    // Determine shell type from shell name tail
    let shell_tail = CStr::from_ptr(nvim_excmds_shell_name_tail()).to_bytes();
    let is_fish_shell = cfg!(unix) && shell_tail.starts_with(b"fish");
    let is_pwsh = shell_tail.starts_with(b"pwsh") || shell_tail.starts_with(b"powershell");

    // Build the command string in Rust
    let mut result: Vec<u8> = Vec::new();

    if is_pwsh {
        if let Some(itmp_b) = itmp_bytes {
            result.extend_from_slice(b"& { Get-Content ");
            result.extend_from_slice(itmp_b);
            result.extend_from_slice(b" | & ");
            result.extend_from_slice(cmd_bytes);
            result.extend_from_slice(b" }");
        } else if do_in {
            result.extend_from_slice(b" $input | ");
            result.extend_from_slice(cmd_bytes);
        } else {
            result.extend_from_slice(cmd_bytes);
        }
    } else if cfg!(unix) {
        // Unix: wrap command in parens/begin..end when redirecting
        if itmp_bytes.is_some() || otmp_bytes.is_some() {
            if is_fish_shell {
                result.extend_from_slice(b"begin; ");
                result.extend_from_slice(cmd_bytes);
                result.extend_from_slice(b"; end");
            } else {
                result.push(b'(');
                result.extend_from_slice(cmd_bytes);
                result.push(b')');
            }
        } else {
            result.extend_from_slice(cmd_bytes);
        }
        if let Some(itmp_b) = itmp_bytes {
            result.extend_from_slice(b" < ");
            result.extend_from_slice(itmp_b);
        }
    } else {
        // Non-Unix: handle pipe redirection
        result.extend_from_slice(cmd_bytes);

        if let Some(itmp_b) = itmp_bytes {
            let p_shq = CStr::from_ptr(nvim_excmds_get_p_shq()).to_bytes();
            let shq_empty = p_shq.is_empty();

            // If shellquote is empty, find a pipe in buf and truncate there
            let pipe_pos_in_buf = if shq_empty {
                find_pipe_pos(&result)
            } else {
                None
            };

            if let Some(pipe_pos) = pipe_pos_in_buf {
                result.truncate(pipe_pos);
            }

            result.extend_from_slice(b" < ");
            result.extend_from_slice(itmp_b);

            // Re-append the pipe portion from the original cmd
            if shq_empty {
                if let Some(pipe_pos_in_cmd) = find_pipe_pos(cmd_bytes) {
                    result.push(b' ');
                    result.extend_from_slice(&cmd_bytes[pipe_pos_in_cmd..]);
                }
            }
        }
    }

    // Handle output redirection: call rs_append_redir on the buffer
    if let Some(otmp_b) = otmp_bytes {
        let p_srr = CStr::from_ptr(nvim_excmds_get_p_srr()).to_bytes();

        // We need to call rs_append_redir, which operates on a C string buffer.
        // Build the null-terminated buffer first, then call rs_append_redir.
        // Calculate extra space needed for the redirect.
        let extra = p_srr.len() + otmp_b.len() + 4; // extra for " opt fname" or " %s substitute"
        let total = result.len() + extra + 1;
        let buf = nvim_excmds_xmalloc(total) as *mut c_char;

        // Copy result into buf and null-terminate
        std::ptr::copy_nonoverlapping(result.as_ptr().cast::<c_char>(), buf, result.len());
        *buf.add(result.len()) = 0;

        // Append the output redirect: opt=p_srr, fname=otmp
        rs_append_redir(buf, total, nvim_excmds_get_p_srr(), otmp);

        return buf;
    }

    // No output redirection: allocate and copy result
    let total = result.len() + 1;
    let buf = nvim_excmds_xmalloc(total) as *mut c_char;
    std::ptr::copy_nonoverlapping(result.as_ptr().cast::<c_char>(), buf, result.len());
    *buf.add(result.len()) = 0;
    buf
}

/// Create shell flags from C integer.
pub extern "C" fn rs_shell_flags_from_c(value: c_int) -> c_int {
    ShellFlags::from_c(value).to_c()
}

/// Check if a command contains an unescaped bang.
///
/// Returns the position (0-indexed) or -1 if not found.
///
/// # Safety
///
/// `cmd` must be a valid null-terminated C string.
pub unsafe extern "C" fn rs_find_unescaped_bang(cmd: *const std::ffi::c_char) -> c_int {
    if cmd.is_null() {
        return -1;
    }

    let c_str = unsafe { std::ffi::CStr::from_ptr(cmd) };
    let s = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match find_unescaped_bang(s) {
        Some(pos) => pos as c_int,
        None => -1,
    }
}

/// Append output redirection for the given file to the end of the buffer.
///
/// Searches opt for a `%s` placeholder (skipping `%%`):
/// - If found: appends a space then formats opt with fname substituted for `%s`
/// - If not found: appends ` opt fname`
///
/// Replaces the C `append_redir` function.
///
/// # Safety
/// - `buf` must be a valid, writable, null-terminated C string buffer of at least `buflen` bytes.
/// - `opt` must be a valid null-terminated C string.
/// - `fname` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_append_redir(
    buf: *mut std::ffi::c_char,
    buflen: usize,
    opt: *const std::ffi::c_char,
    fname: *const std::ffi::c_char,
) {
    use std::ffi::CStr;

    let opt_bytes = CStr::from_ptr(opt).to_bytes();
    let fname_bytes = CStr::from_ptr(fname).to_bytes();

    // Find current end of buf (strlen)
    let mut end_offset = 0usize;
    while end_offset < buflen && *buf.add(end_offset) != 0 {
        end_offset += 1;
    }

    // Inline helper: write bytes to buf at offset, returns new offset.
    macro_rules! write_bytes {
        ($bytes:expr) => {{
            for &byte in $bytes {
                if end_offset + 1 < buflen {
                    *buf.add(end_offset) = byte as std::ffi::c_char;
                    end_offset += 1;
                }
            }
        }};
    }

    // Find "%s" in opt (skipping "%%"), returns byte index of the '%' in "%s" or None.
    let percent_s_pos = {
        let mut pos = None;
        let mut j = 0;
        while j < opt_bytes.len() {
            if opt_bytes[j] == b'%' && j + 1 < opt_bytes.len() {
                if opt_bytes[j + 1] == b's' {
                    pos = Some(j);
                    break;
                } else if opt_bytes[j + 1] == b'%' {
                    j += 2; // skip %%
                    continue;
                }
            }
            j += 1;
        }
        pos
    };

    // Helper to write a byte slice converting %% -> %
    macro_rules! write_opt_section {
        ($section:expr) => {{
            let section: &[u8] = $section;
            let mut k = 0;
            while k < section.len() {
                if section[k] == b'%' && k + 1 < section.len() && section[k + 1] == b'%' {
                    write_bytes!(b"%");
                    k += 2;
                } else {
                    write_bytes!(&section[k..k + 1]);
                    k += 1;
                }
            }
        }};
    }

    if let Some(ps) = percent_s_pos {
        // Found %s: write ' ' then opt-before-%s then fname then opt-after-%s
        write_bytes!(b" ");
        write_opt_section!(&opt_bytes[..ps]);
        write_bytes!(fname_bytes);
        write_opt_section!(&opt_bytes[ps + 2..]);
    } else {
        // No %s found: write " opt fname"
        write_bytes!(b" ");
        write_bytes!(opt_bytes);
        write_bytes!(b" ");
        write_bytes!(fname_bytes);
    }

    // Null-terminate
    if end_offset < buflen {
        *buf.add(end_offset) = 0;
    } else if buflen > 0 {
        *buf.add(buflen - 1) = 0;
    }
}

/// Check if shell escape is needed for a character.
pub extern "C" fn rs_needs_shell_escape(c: c_int) -> c_int {
    if !(0..=127).contains(&c) {
        return 0;
    }
    c_int::from(needs_shell_escape(c as u8 as char))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_flags() {
        let flags = ShellFlags::FILTER | ShellFlags::DO_OUT;
        assert!(flags.is_filter());
        assert!(flags.captures_output());

        let flags = ShellFlags::empty();
        assert!(!flags.is_filter());
        assert!(!flags.captures_output());
    }

    #[test]
    fn test_shell_flags_roundtrip() {
        let flags = ShellFlags::FILTER | ShellFlags::READ | ShellFlags::SILENT;
        let c_val = flags.to_c();
        let back = ShellFlags::from_c(c_val);
        assert_eq!(flags, back);
    }

    #[test]
    fn test_shell_command_type() {
        assert!(!ShellCommandType::Execute.uses_range());
        assert!(ShellCommandType::Filter.uses_range());
        assert!(!ShellCommandType::Read.uses_range());
        assert!(ShellCommandType::Write.uses_range());
        assert!(!ShellCommandType::Interactive.uses_range());
    }

    #[test]
    fn test_shell_command_type_flags() {
        let flags = ShellCommandType::Filter.default_flags();
        assert!(flags.contains(ShellFlags::FILTER));

        let flags = ShellCommandType::Read.default_flags();
        assert!(flags.contains(ShellFlags::READ));
        assert!(flags.contains(ShellFlags::DO_OUT));
    }

    #[test]
    fn test_shell_options() {
        let opts = ShellOptions::execute("ls -la");
        assert_eq!(opts.command, Some("ls -la".to_string()));
        assert!(!opts.is_interactive());

        let opts = ShellOptions::interactive();
        assert!(opts.is_interactive());
        assert!(opts.command.is_none());
    }

    #[test]
    fn test_shell_options_filter() {
        let range = LineRange::new(1, 10);
        let opts = ShellOptions::filter("sort", range);
        assert!(opts.do_input);
        assert!(opts.do_output);
        assert!(opts.flags.is_filter());
        assert_eq!(opts.range, Some(range));
    }

    #[test]
    fn test_find_unescaped_bang() {
        assert_eq!(find_unescaped_bang("hello"), None);
        assert_eq!(find_unescaped_bang("hello!"), Some(5));
        assert_eq!(find_unescaped_bang("!hello"), Some(0));
        assert_eq!(find_unescaped_bang("he!llo"), Some(2));
        assert_eq!(find_unescaped_bang("hello\\!"), None);
        assert_eq!(find_unescaped_bang("hello\\!!"), Some(7));
    }

    #[test]
    fn test_needs_bang_substitution() {
        assert!(!needs_bang_substitution("ls", false));
        assert!(needs_bang_substitution("ls", true));
        assert!(needs_bang_substitution("ls!", false));
        assert!(!needs_bang_substitution("ls\\!", false));
    }

    #[test]
    fn test_substitute_bangs_no_subst() {
        let result = substitute_bangs("ls -la", None, false);
        assert_eq!(result, BangSubstResult::Success("ls -la".to_string()));
    }

    #[test]
    fn test_substitute_bangs_forceit() {
        let result = substitute_bangs("", Some("ls"), true);
        assert_eq!(result, BangSubstResult::Success("ls".to_string()));

        let result = substitute_bangs(" -la", Some("ls"), true);
        assert_eq!(result, BangSubstResult::Success("ls -la".to_string()));
    }

    #[test]
    fn test_substitute_bangs_inline() {
        let result = substitute_bangs("echo ! | grep x", Some("ls"), false);
        assert_eq!(
            result,
            BangSubstResult::Success("echo ls | grep x".to_string())
        );
    }

    #[test]
    fn test_substitute_bangs_escaped() {
        let result = substitute_bangs("echo \\!", Some("ls"), false);
        assert_eq!(result, BangSubstResult::Success("echo !".to_string()));
    }

    #[test]
    fn test_substitute_bangs_no_prev() {
        let result = substitute_bangs("!", None, false);
        assert_eq!(result, BangSubstResult::NoPreviousCommand);

        let result = substitute_bangs("!", Some(""), false);
        assert_eq!(result, BangSubstResult::NoPreviousCommand);
    }

    #[test]
    fn test_shell_result() {
        let result = ShellResult::success();
        assert!(result.is_success());
        assert_eq!(result.exit_code, 0);

        let result = ShellResult::with_exit_code(1);
        assert!(!result.is_success());
        assert_eq!(result.exit_code, 1);

        let result = ShellResult::with_lines(0, 10);
        assert!(result.is_success());
        assert_eq!(result.lines_affected, Some(10));
    }

    #[test]
    fn test_shell_error_display() {
        assert!(ShellError::SecureMode.to_string().contains("E145"));
        assert!(ShellError::NoPreviousCommand.to_string().contains("E34"));
        assert!(ShellError::CommandFailed(42).to_string().contains("42"));
    }

    #[test]
    fn test_needs_shell_escape() {
        assert!(needs_shell_escape('\t'));
        assert!(needs_shell_escape(' '));
        assert!(needs_shell_escape('"'));
        assert!(needs_shell_escape('&'));
        assert!(needs_shell_escape('|'));
        assert!(!needs_shell_escape('a'));
        assert!(!needs_shell_escape('1'));
    }

    #[test]
    fn test_count_shell_special() {
        assert_eq!(count_shell_special("hello"), 0);
        assert_eq!(count_shell_special("hello world"), 1);
        // "a | b & c" has 4 spaces + 1 pipe + 1 ampersand = 6 special chars
        assert_eq!(count_shell_special("a | b & c"), 6);
    }

    #[test]
    fn test_rs_needs_shell_escape() {
        assert_eq!(rs_needs_shell_escape(b' ' as c_int), 1);
        assert_eq!(rs_needs_shell_escape(b'a' as c_int), 0);
        assert_eq!(rs_needs_shell_escape(-1), 0);
        assert_eq!(rs_needs_shell_escape(200), 0);
    }
}
