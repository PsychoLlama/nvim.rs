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

// libc for strlen
extern crate libc;

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

    // do_shell FFI
    fn nvim_excmds_get_p_warn() -> c_int;
    fn nvim_excmds_get_autocmd_busy() -> c_int;
    fn nvim_excmds_get_msg_silent() -> c_int;
    fn nvim_excmds_any_buf_changed() -> c_int;
    fn nvim_excmds_emsg_by_id(id: c_int);
    fn nvim_excmds_call_shell(cmd: *mut c_char, flags: c_int);
    fn nvim_excmds_set_msg_didout(val: c_int);
    fn nvim_excmds_set_did_check_timestamps(val: c_int);
    fn nvim_excmds_set_need_check_timestamps(val: c_int);
    fn nvim_excmds_set_msg_row(val: c_int);
    fn nvim_excmds_set_msg_col(val: c_int);
    fn nvim_excmds_apply_autocmds_shellcmdpost();
    fn nvim_get_Rows() -> c_int;

    // do_bang FFI
    fn rs_check_secure() -> c_int;
    fn nvim_excmds_get_msg_scroll() -> c_int;
    fn nvim_excmds_set_msg_scroll(val: c_int);
    fn nvim_excmds_autowrite_all();
    fn nvim_excmds_get_bangredo() -> c_int;
    fn nvim_excmds_set_bangredo(val: c_int);
    fn nvim_excmds_vim_strsave_escaped(s: *const c_char, chars: *const c_char) -> *mut c_char;
    fn nvim_excmds_append_to_redobuff_lit(s: *const c_char, len: c_int);
    fn nvim_excmds_append_to_redobuff(s: *const c_char);
    fn nvim_excmds_ui_cursor_goto(row: c_int, col: c_int);
    fn nvim_excmds_get_msg_row() -> c_int;
    fn nvim_excmds_get_msg_col() -> c_int;
    fn nvim_excmds_do_shell_wrapper(cmd: *mut c_char, flags: c_int);
    fn nvim_excmds_do_filter_wrapper(
        line1: c_int,
        line2: c_int,
        eap: *mut crate::ExArgHandle,
        cmd: *mut c_char,
        do_in: bool,
        do_out: bool,
    );
    fn nvim_excmds_apply_autocmds_shellfilterpost();
    fn msg_start();
    fn msg_putchar(c: c_int);
    fn nvim_excmds_msg_outtrans(s: *const c_char);
    fn msg_clr_eos();
    pub fn xmalloc(size: usize) -> *mut std::ffi::c_void;
    pub fn xfree(ptr: *mut std::ffi::c_void);
    fn skipwhite(p: *const c_char) -> *const c_char;
}

/// Inline wrapper for strlen to avoid extern declarations of libc.
#[inline(always)]
unsafe fn cstrlen(s: *const c_char) -> usize {
    libc::strlen(s)
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

// =============================================================================
// do_bang / prevcmd management
// =============================================================================

/// The previous shell command for `!` substitution.
///
/// Managed with C allocator (`xmalloc`/`xfree`) so the pointer is compatible
/// with any C code that might read it.
static mut PREVCMD: Option<*mut c_char> = None;

/// Free the previous shell command. Called from EXITFREE cleanup.
///
/// # Safety
/// Must only be called once during process exit.
#[no_mangle]
pub unsafe extern "C" fn rs_free_prev_shellcmd() {
    #[allow(static_mut_refs)]
    if let Some(ptr) = PREVCMD.take() {
        xfree(ptr as *mut std::ffi::c_void);
    }
}

/// Execute a shell command. Replaces the C `do_shell` function.
///
/// When `cmd` is NULL, starts an interactive shell.
///
/// # Safety
/// `cmd` must be null or a valid null-terminated C string (mutable for call_shell).
#[no_mangle]
pub unsafe extern "C" fn rs_do_shell(cmd: *mut c_char, flags: c_int) {
    // Disallow shell commands in secure mode
    if rs_check_secure() != 0 {
        crate::msg_end();
        return;
    }

    // For autocommands we want to get the output on the current screen.
    crate::msg_putchar(b'\r' as c_int); // put cursor at start of line
    crate::msg_putchar(b'\n' as c_int); // may shift screen one line up

    // Warning message before calling the shell
    if nvim_excmds_get_p_warn() != 0
        && nvim_excmds_get_autocmd_busy() == 0
        && nvim_excmds_get_msg_silent() == 0
        && nvim_excmds_any_buf_changed() != 0
    {
        nvim_excmds_emsg_by_id(11); // msg_puts_no_write_warning
    }

    // This ui_cursor_goto is required for when the '\n' resulted in a
    // "delete line 1" command to the terminal.
    let row = nvim_excmds_get_msg_row();
    let col = nvim_excmds_get_msg_col();
    nvim_excmds_ui_cursor_goto(row, col);
    nvim_excmds_call_shell(cmd, flags);

    if nvim_excmds_get_msg_silent() == 0 {
        nvim_excmds_set_msg_didout(1);
    }
    nvim_excmds_set_did_check_timestamps(0);
    nvim_excmds_set_need_check_timestamps(1);

    // Put the message cursor at the end of the screen to avoid wait_return()
    // overwriting the text the external command showed.
    let rows = nvim_get_Rows();
    nvim_excmds_set_msg_row(rows - 1);
    nvim_excmds_set_msg_col(0);

    nvim_excmds_apply_autocmds_shellcmdpost();
}

/// Handle the `:!cmd` command. Also for `:r !cmd` and `:w !cmd`.
///
/// Bangs in the argument are replaced with the previously entered command.
/// Manages the `prevcmd` static. Dispatches to `do_shell` or `do_filter`.
///
/// # Safety
/// All pointer arguments must be non-null and valid.
#[no_mangle]
#[allow(static_mut_refs)]
pub unsafe extern "C" fn rs_do_bang(
    addr_count: c_int,
    eap: *mut crate::ExArgHandle,
    forceit: bool,
    do_in: bool,
    do_out: bool,
) {
    // Disallow shell commands in secure mode
    if rs_check_secure() != 0 {
        return;
    }

    let arg = crate::nvim_exarg_get_arg(eap as *const _);
    let line1 = crate::nvim_exarg_get_line1(eap as *const _);
    let line2 = crate::nvim_exarg_get_line2(eap as *const _);

    if addr_count == 0 {
        // :! -- don't scroll here
        let scroll_save = nvim_excmds_get_msg_scroll();
        nvim_excmds_set_msg_scroll(0);
        nvim_excmds_autowrite_all();
        nvim_excmds_set_msg_scroll(scroll_save);
    }

    // Build the command by bang-substituting.
    // We work entirely with xmalloc-allocated C strings so we can transfer
    // ownership to PREVCMD without copying.

    let mut ins_prevcmd = forceit;

    // Skip leading whitespace
    let mut trailarg: *const c_char = skipwhite(arg);

    // `newcmd` is the accumulated command string (xmalloc-allocated or NULL)
    let mut newcmd: *mut c_char = std::ptr::null_mut();

    loop {
        let trailarg_len = cstrlen(trailarg);
        let newcmd_len: usize = if newcmd.is_null() { 0 } else { cstrlen(newcmd) };
        let prevcmd_len: usize = if ins_prevcmd {
            match PREVCMD {
                Some(ptr) if !ptr.is_null() => cstrlen(ptr),
                _ => {
                    // Need prevcmd but it's not set
                    nvim_excmds_emsg_by_id(1); // e_noprev
                    if !newcmd.is_null() {
                        xfree(newcmd as *mut std::ffi::c_void);
                    }
                    return;
                }
            }
        } else {
            0
        };

        let len = trailarg_len + newcmd_len + prevcmd_len + 1;
        let t = xmalloc(len) as *mut c_char;
        *t = 0; // NUL-terminate

        // Concatenate: newcmd + prevcmd + trailarg
        if !newcmd.is_null() {
            let nlen = newcmd_len;
            std::ptr::copy_nonoverlapping(newcmd, t, nlen);
        }
        if ins_prevcmd {
            if let Some(ptr) = PREVCMD {
                let nlen = newcmd_len;
                std::ptr::copy_nonoverlapping(ptr, t.add(nlen), prevcmd_len);
            }
        }
        let base_len = newcmd_len + prevcmd_len;
        std::ptr::copy_nonoverlapping(trailarg, t.add(base_len), trailarg_len);
        *t.add(base_len + trailarg_len) = 0;

        if !newcmd.is_null() {
            xfree(newcmd as *mut std::ffi::c_void);
        }
        newcmd = t;

        // Scan the suffix (starting right after the previously appended newcmd+prevcmd)
        // for '!', which is replaced by the previous command. "\!" becomes "!".
        let mut p: *mut c_char = newcmd.add(base_len);
        trailarg = std::ptr::null();
        while *p != 0 {
            if *p == b'!' as c_char {
                if p > newcmd && *p.sub(1) == b'\\' as c_char {
                    // STRMOVE(p-1, p): shift string left by 1 at p-1
                    let src = p;
                    let dst = p.sub(1);
                    let remaining = cstrlen(src) + 1; // include NUL
                    std::ptr::copy(src, dst, remaining);
                    // p stays at the same address (which is now the char after '!')
                    // but we don't advance -- the '!' was moved to p-1 and
                    // we continue scanning from p (which shifted left)
                } else {
                    // Split at this '!': null-terminate here, mark trailarg
                    trailarg = p.add(1);
                    *p = 0;
                    ins_prevcmd = true;
                    break;
                }
            } else {
                p = p.add(1);
            }
        }

        if trailarg.is_null() {
            break;
        }
    }

    // Only update PREVCMD if there's actually a command to run.
    let newcmd_empty = *newcmd == 0;
    let mut free_newcmd = false;
    if !newcmd_empty {
        if let Some(old) = PREVCMD.take() {
            xfree(old as *mut std::ffi::c_void);
        }
        PREVCMD = Some(newcmd);
    } else {
        free_newcmd = true;
    }

    // Handle bangredo: put cmd in redo buffer
    if nvim_excmds_get_bangredo() != 0 {
        let prevcmd_ptr = match PREVCMD {
            Some(ptr) if !ptr.is_null() => ptr,
            _ => {
                nvim_excmds_emsg_by_id(1); // e_noprev
                if free_newcmd {
                    xfree(newcmd as *mut std::ffi::c_void);
                }
                return;
            }
        };

        // Escape % and # in the command for redo buffer
        let escaped = nvim_excmds_vim_strsave_escaped(prevcmd_ptr, c"%#".as_ptr());
        nvim_excmds_append_to_redobuff_lit(escaped, -1);
        xfree(escaped as *mut std::ffi::c_void);
        nvim_excmds_append_to_redobuff(c"\n".as_ptr());
        nvim_excmds_set_bangredo(0);
    }

    // Add shell quotes if p_shq is non-empty
    let p_shq = nvim_excmds_get_p_shq();
    if *p_shq != 0 {
        let prevcmd_ptr = match PREVCMD {
            Some(ptr) if !ptr.is_null() => ptr,
            _ => {
                if free_newcmd {
                    xfree(newcmd as *mut std::ffi::c_void);
                }
                return;
            }
        };

        if free_newcmd {
            xfree(newcmd as *mut std::ffi::c_void);
        }

        let shq_len = cstrlen(p_shq);
        let prevcmd_len = cstrlen(prevcmd_ptr);
        let quoted_len = prevcmd_len + 2 * shq_len + 1;
        newcmd = xmalloc(quoted_len) as *mut c_char;

        // Build: p_shq + prevcmd + p_shq
        std::ptr::copy_nonoverlapping(p_shq, newcmd, shq_len);
        std::ptr::copy_nonoverlapping(prevcmd_ptr, newcmd.add(shq_len), prevcmd_len);
        std::ptr::copy_nonoverlapping(p_shq, newcmd.add(shq_len + prevcmd_len), shq_len);
        *newcmd.add(shq_len + prevcmd_len + shq_len) = 0;
        free_newcmd = true;
    }

    if addr_count == 0 {
        // :! -- echo the command and execute
        msg_start();
        nvim_excmds_emsg_by_id(10); // msg_ext_set_kind_shell_cmd
        msg_putchar(b':' as c_int);
        msg_putchar(b'!' as c_int);
        nvim_excmds_msg_outtrans(newcmd);
        msg_clr_eos();
        let row = nvim_excmds_get_msg_row();
        let col = nvim_excmds_get_msg_col();
        nvim_excmds_ui_cursor_goto(row, col);

        nvim_excmds_do_shell_wrapper(newcmd, 0);
    } else {
        // :range! -- filter through shell command
        // Note: This may recursively call do_bang() again (via autocommands).
        nvim_excmds_do_filter_wrapper(line1, line2, eap, newcmd, do_in, do_out);
        nvim_excmds_apply_autocmds_shellfilterpost();
    }

    if free_newcmd {
        xfree(newcmd as *mut std::ffi::c_void);
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
// do_filter (Phase 5 migration)
// =============================================================================

use crate::ExArgHandle;

extern "C" {
    // do_filter accessors
    fn nvim_excmds_get_p_stmp() -> c_int;
    fn nvim_excmds_curbuf_op_start_lnum() -> c_int;
    fn nvim_excmds_curbuf_op_end_lnum() -> c_int;
    fn nvim_excmds_curbuf_set_op_start_lnum(lnum: c_int);
    fn nvim_excmds_curbuf_set_op_end_lnum(lnum: c_int);
    fn nvim_excmds_curwin_cursor_save() -> u64;
    fn nvim_excmds_curwin_cursor_restore(saved: u64);
    fn nvim_excmds_cmdmod_save_clear_lockmarks() -> c_int;
    fn nvim_excmds_cmdmod_restore_flags(saved: c_int);
    fn nvim_cmdmod_has_lockmarks() -> c_int;
    fn nvim_excmds_cmdmod_has_keepmarks_now() -> c_int;
    fn nvim_excmds_vim_tempname() -> *mut c_char;
    fn nvim_excmds_buf_write_filter(
        itmp: *const c_char,
        line1: c_int,
        line2: c_int,
        eap: *mut ExArgHandle,
    ) -> c_int;
    fn nvim_excmds_no_wait_return_inc();
    fn nvim_excmds_no_wait_return_dec();
    fn nvim_excmds_readfile_filter(
        otmp: *const c_char,
        line2: c_int,
        eap: *mut ExArgHandle,
    ) -> c_int;
    fn nvim_excmds_call_shell_filter(cmd: *const c_char, flags: c_int);
    fn nvim_excmds_after_shell();
    fn nvim_excmds_clear_got_int();
    fn nvim_excmds_del_lines(count: c_int);
    fn nvim_excmds_write_lnum_adjust(offset: c_int);
    fn nvim_excmds_redraw_curbuf_later_valid();
    fn nvim_excmds_invalidate_botline();
    fn nvim_excmds_p_cpo_no_remmark() -> c_int;
    fn nvim_excmds_fold_update_curwin(top: c_int, bot: c_int);
    fn nvim_excmds_msg_lines_filtered(linecount: c_int);
    fn nvim_excmds_error_msg(error_id: c_int, arg: *const c_char);
    fn nvim_excmds_wait_return_false();
    fn nvim_excmds_curbuf_op_save(out_start: *mut u64, out_end: *mut u64);
    fn nvim_excmds_curbuf_op_restore(saved_start: u64, saved_end: u64);
    fn nvim_excmds_curbuf_op_adjust_lnum(delta: c_int);
    fn nvim_bw_os_remove(path: *const c_char) -> c_int;
    fn nvim_excmds_curbuf_ml_line_count() -> c_int;
    fn nvim_excmds_get_curbuf_ptr() -> *mut std::ffi::c_void;
    fn nvim_excmds_aborting() -> c_int;
    fn os_breakcheck();
    fn u_save(top: c_int, bot: c_int) -> c_int;
    fn nvim_curwin_set_cursor_lnum(lnum: c_int);
    fn nvim_curwin_set_cursor_col(col: c_int);
    fn nvim_excmds_changed_line_abv_curs();
    fn beginline(flags: c_int);
    fn msgmore(n: c_int);
    fn appended_lines_mark(lnum: c_int, count: c_int);
}

/// BL_WHITE | BL_FIX constants for beginline()
const BL_WHITE: c_int = 1;
const BL_FIX: c_int = 4;

// Error IDs for nvim_excmds_error_msg dispatcher (see ex_cmds_shim.c)
const ERR_E482: c_int = 1; // "E482: Can't create file %s"
const ERR_E_NOTREAD: c_int = 2; // e_notread with fname
const ERR_E_NOTMP: c_int = 3; // e_notmp (no arg)
const ERR_E135: c_int = 4; // E135 filter autocommand error

// kShellOpt* constants -- verified by _Static_assert in ex_cmds_shim.c
const K_SHELL_OPT_FILTER: c_int = 1;
const K_SHELL_OPT_READ: c_int = 16;
const K_SHELL_OPT_WRITE: c_int = 32;
const K_SHELL_OPT_DO_OUT: c_int = 4;

/// Filter lines through an external command. Replaces C `do_filter`.
///
/// # Safety
/// All pointer arguments must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_do_filter(
    line1: c_int,
    line2: c_int,
    eap: *mut ExArgHandle,
    cmd: *const c_char,
    do_in: c_int,
    do_out: c_int,
) {
    let do_in = do_in != 0;
    let do_out = do_out != 0;

    // Early return if no command
    if *cmd == 0 {
        return;
    }

    // Save state for cleanup
    let old_curbuf = nvim_excmds_get_curbuf_ptr();
    let stmp = nvim_excmds_get_p_stmp() != 0;
    let mut orig_start: u64 = 0;
    let mut orig_end: u64 = 0;
    nvim_excmds_curbuf_op_save(&mut orig_start, &mut orig_end);

    // Save and modify cmdmod.cmod_flags (disable CMOD_LOCKMARKS)
    let save_cmod_flags = nvim_excmds_cmdmod_save_clear_lockmarks();

    let cursor_save = nvim_excmds_curwin_cursor_save();
    let linecount = line2 - line1 + 1;
    nvim_curwin_set_cursor_lnum(line1);
    nvim_curwin_set_cursor_col(0);
    nvim_excmds_changed_line_abv_curs();
    nvim_excmds_invalidate_botline();

    let k_shell_opt_do_out = K_SHELL_OPT_DO_OUT;
    let k_shell_opt_read = K_SHELL_OPT_READ;
    let k_shell_opt_write = K_SHELL_OPT_WRITE;
    let k_shell_opt_filter = K_SHELL_OPT_FILTER;

    let mut shell_flags: c_int = 0;
    if do_out {
        shell_flags |= k_shell_opt_do_out;
    }

    let mut itmp: *mut c_char = std::ptr::null_mut();
    let mut otmp: *mut c_char = std::ptr::null_mut();

    // Determine piping vs temp file strategy
    if !do_in && do_out && !stmp {
        // Pipe stdout only
        shell_flags |= k_shell_opt_read;
        nvim_curwin_set_cursor_lnum(line2);
    } else if do_in && !do_out && !stmp {
        // Pipe stdin only
        shell_flags |= k_shell_opt_write;
        nvim_excmds_curbuf_set_op_start_lnum(line1);
        nvim_excmds_curbuf_set_op_end_lnum(line2);
    } else if do_in && do_out && !stmp {
        // Pipe both stdin and stdout
        shell_flags |= k_shell_opt_read | k_shell_opt_write;
        nvim_excmds_curbuf_set_op_start_lnum(line1);
        nvim_excmds_curbuf_set_op_end_lnum(line2);
        nvim_curwin_set_cursor_lnum(line2);
    } else {
        // Use temp files
        if do_in {
            itmp = nvim_excmds_vim_tempname();
            if itmp.is_null() {
                nvim_excmds_error_msg(ERR_E_NOTMP, std::ptr::null());
                goto_filterend(
                    save_cmod_flags,
                    old_curbuf,
                    orig_start,
                    orig_end,
                    itmp,
                    otmp,
                );
                return;
            }
        }
        if do_out {
            otmp = nvim_excmds_vim_tempname();
            if otmp.is_null() {
                nvim_excmds_error_msg(ERR_E_NOTMP, std::ptr::null());
                goto_filterend(
                    save_cmod_flags,
                    old_curbuf,
                    orig_start,
                    orig_end,
                    itmp,
                    otmp,
                );
                return;
            }
        }
    }

    nvim_excmds_no_wait_return_inc();

    // Write input temp file if needed
    if !itmp.is_null() {
        if nvim_excmds_buf_write_filter(itmp, line1, line2, eap) == 0 {
            msg_putchar(b'\n' as c_int); // Keep message from buf_write()
            nvim_excmds_no_wait_return_dec();
            if nvim_excmds_aborting() == 0 {
                nvim_excmds_error_msg(ERR_E482, itmp);
            }
            goto_filterend(
                save_cmod_flags,
                old_curbuf,
                orig_start,
                orig_end,
                itmp,
                otmp,
            );
            return;
        }
        if nvim_excmds_get_curbuf_ptr() != old_curbuf {
            goto_filterend(
                save_cmod_flags,
                old_curbuf,
                orig_start,
                orig_end,
                itmp,
                otmp,
            );
            return;
        }
    }

    if !do_out {
        msg_putchar(b'\n' as c_int);
    }

    // Create shell command
    let cmd_buf = rs_make_filter_cmd(cmd, itmp, otmp, c_int::from(do_in));
    nvim_excmds_ui_cursor_goto(nvim_get_Rows() - 1, 0);

    if do_out {
        if u_save(line2, line2 + 1) == 0 {
            // FAIL
            xfree(cmd_buf as *mut std::ffi::c_void);
            // goto error
            nvim_excmds_curwin_cursor_restore(cursor_save);
            nvim_excmds_no_wait_return_dec();
            nvim_excmds_wait_return_false();
            goto_filterend(
                save_cmod_flags,
                old_curbuf,
                orig_start,
                orig_end,
                itmp,
                otmp,
            );
            return;
        }
        nvim_excmds_redraw_curbuf_later_valid();
    }

    let mut read_linecount = nvim_excmds_curbuf_ml_line_count();

    // Run the shell command
    nvim_excmds_call_shell_filter(cmd_buf, k_shell_opt_filter | shell_flags);
    xfree(cmd_buf as *mut std::ffi::c_void);

    nvim_excmds_after_shell();
    os_breakcheck();
    nvim_excmds_clear_got_int();

    if do_out {
        if !otmp.is_null() {
            if nvim_excmds_readfile_filter(otmp, line2, eap) == 0 {
                if nvim_excmds_aborting() == 0 {
                    msg_putchar(b'\n' as c_int);
                    nvim_excmds_error_msg(ERR_E_NOTREAD, otmp);
                }
                // goto error
                nvim_excmds_curwin_cursor_restore(cursor_save);
                nvim_excmds_no_wait_return_dec();
                nvim_excmds_wait_return_false();
                goto_filterend(
                    save_cmod_flags,
                    old_curbuf,
                    orig_start,
                    orig_end,
                    itmp,
                    otmp,
                );
                return;
            }
            if nvim_excmds_get_curbuf_ptr() != old_curbuf {
                goto_filterend(
                    save_cmod_flags,
                    old_curbuf,
                    orig_start,
                    orig_end,
                    itmp,
                    otmp,
                );
                return;
            }
        }

        read_linecount = nvim_excmds_curbuf_ml_line_count() - read_linecount;

        if (shell_flags & k_shell_opt_read) != 0 {
            nvim_excmds_curbuf_set_op_start_lnum(line2 + 1);
            let cursor_lnum = nvim_excmds_curwin_cursor_save() >> 32;
            nvim_excmds_curbuf_set_op_end_lnum(cursor_lnum as c_int);
            appended_lines_mark(line2, read_linecount);
        }

        if do_in {
            if nvim_excmds_cmdmod_has_keepmarks_now() != 0 || nvim_excmds_p_cpo_no_remmark() != 0 {
                // KEXTMARK_NOOP = 0
                const KEXTMARK_NOOP: c_int = 0;
                const MAXLNUM: c_int = 0x7FFF_FFFF;
                if read_linecount >= linecount {
                    // move all marks
                    crate::nvim_excmds_mark_adjust(line1, line2, linecount, 0, KEXTMARK_NOOP);
                } else {
                    // move marks from valid range, delete marks in deleted lines
                    crate::nvim_excmds_mark_adjust(
                        line1,
                        line1 + read_linecount - 1,
                        linecount,
                        0,
                        KEXTMARK_NOOP,
                    );
                    crate::nvim_excmds_mark_adjust(
                        line1 + read_linecount,
                        line2,
                        MAXLNUM,
                        0,
                        KEXTMARK_NOOP,
                    );
                }
            }

            nvim_curwin_set_cursor_lnum(line1);
            nvim_excmds_del_lines(linecount);
            nvim_excmds_curbuf_op_adjust_lnum(-linecount);
            nvim_excmds_write_lnum_adjust(-linecount);
            let op_start = nvim_excmds_curbuf_op_start_lnum();
            let op_end = nvim_excmds_curbuf_op_end_lnum();
            nvim_excmds_fold_update_curwin(op_start, op_end);
        } else {
            // ":r !cmd" - put cursor on last new line
            let op_start = nvim_excmds_curbuf_op_start_lnum();
            let op_end = nvim_excmds_curbuf_op_end_lnum();
            let new_linecount = op_end - op_start + 1;
            nvim_curwin_set_cursor_lnum(op_end);
            // Update linecount for report message
            let _ = new_linecount;
        }

        beginline(BL_WHITE | BL_FIX);
        nvim_excmds_no_wait_return_dec();

        if (linecount as i64) > crate::nvim_excmds_p_report() {
            if do_in {
                nvim_excmds_msg_lines_filtered(linecount);
            } else {
                // For ":r !cmd" we report different linecount
                let op_start = nvim_excmds_curbuf_op_start_lnum();
                let op_end = nvim_excmds_curbuf_op_end_lnum();
                let new_linecount = op_end - op_start + 1;
                msgmore(new_linecount);
            }
        }
    } else {
        // error path: restore cursor
        nvim_excmds_curwin_cursor_restore(cursor_save);
        nvim_excmds_no_wait_return_dec();
        nvim_excmds_wait_return_false();
    }

    goto_filterend(
        save_cmod_flags,
        old_curbuf,
        orig_start,
        orig_end,
        itmp,
        otmp,
    );
}

/// Helper for the filterend cleanup code (shared between goto paths).
///
/// # Safety
/// Called from rs_do_filter with valid state.
unsafe fn goto_filterend(
    save_cmod_flags: c_int,
    old_curbuf: *mut std::ffi::c_void,
    orig_start: u64,
    orig_end: u64,
    itmp: *mut c_char,
    otmp: *mut c_char,
) {
    nvim_excmds_cmdmod_restore_flags(save_cmod_flags);

    if nvim_excmds_get_curbuf_ptr() != old_curbuf {
        nvim_excmds_no_wait_return_dec();
        nvim_excmds_error_msg(ERR_E135, std::ptr::null());
    } else if nvim_cmdmod_has_lockmarks() != 0 {
        nvim_excmds_curbuf_op_restore(orig_start, orig_end);
    }

    if !itmp.is_null() {
        nvim_bw_os_remove(itmp);
    }
    if !otmp.is_null() {
        nvim_bw_os_remove(otmp);
    }
    xfree(itmp as *mut std::ffi::c_void);
    xfree(otmp as *mut std::ffi::c_void);
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
