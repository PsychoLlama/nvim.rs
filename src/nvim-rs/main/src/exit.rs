//! Exit and cleanup
//!
//! This module provides Rust implementations for Neovim's
//! exit handling and cleanup routines.

use std::ffi::c_int;

// =============================================================================
// Exit Codes
// =============================================================================

/// Standard exit codes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Successful exit
    Success = 0,
    /// General error
    Error = 1,
    /// Command-line usage error
    Usage = 2,
    /// Cannot open input file
    NoInput = 3,
    /// Internal software error
    Internal = 70,
    /// OS error (e.g., can't fork)
    OsErr = 71,
    /// Critical OS file missing
    OsFile = 72,
    /// Cannot create output file
    CantCreate = 73,
    /// I/O error
    IoErr = 74,
    /// Temporary failure
    TempFail = 75,
    /// Configuration error
    Config = 78,
}

impl ExitCode {
    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Success,
            2 => Self::Usage,
            3 => Self::NoInput,
            70 => Self::Internal,
            71 => Self::OsErr,
            72 => Self::OsFile,
            73 => Self::CantCreate,
            74 => Self::IoErr,
            75 => Self::TempFail,
            78 => Self::Config,
            _ => Self::Error,
        }
    }

    /// Check if exit code indicates success.
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }
}

// =============================================================================
// Exit Reason
// =============================================================================

/// Reason for exit.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExitReason {
    /// Normal user-requested exit (:q, ZZ)
    #[default]
    Normal = 0,
    /// Exit after help/version
    Help = 1,
    /// Exit due to error
    Error = 2,
    /// Exit due to signal
    Signal = 3,
    /// Exit due to fatal error
    Fatal = 4,
    /// Exit from script (exit())
    Script = 5,
    /// Exit from remote command
    Remote = 6,
}

impl ExitReason {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Help,
            2 => Self::Error,
            3 => Self::Signal,
            4 => Self::Fatal,
            5 => Self::Script,
            6 => Self::Remote,
            _ => Self::Normal,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if exit allows saving files.
    #[must_use]
    pub const fn allows_save(self) -> bool {
        matches!(self, Self::Normal | Self::Script | Self::Remote)
    }

    /// Check if exit should run autocmds.
    #[must_use]
    pub const fn runs_autocmds(self) -> bool {
        matches!(self, Self::Normal | Self::Script)
    }
}

// =============================================================================
// Exit State
// =============================================================================

/// State during exit process.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ExitState {
    /// Exit code to use
    pub code: c_int,
    /// Reason for exit
    pub reason: c_int,
    /// Exit is in progress
    pub exiting: bool,
    /// Cleanup phase
    pub cleanup_phase: c_int,
    /// Force exit (ignore errors)
    pub force: bool,
    /// Preserve session
    pub preserve_session: bool,
    /// Write viminfo/shada
    pub write_shada: bool,
}

impl ExitState {
    /// Create new exit state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            code: 0,
            reason: ExitReason::Normal as c_int,
            exiting: false,
            cleanup_phase: 0,
            force: false,
            preserve_session: false,
            write_shada: true,
        }
    }

    /// Start exit process.
    pub fn begin(&mut self, code: ExitCode, reason: ExitReason) {
        self.code = code.to_c_int();
        self.reason = reason.to_c_int();
        self.exiting = true;
    }

    /// Get current reason.
    #[must_use]
    pub const fn get_reason(&self) -> ExitReason {
        ExitReason::from_c_int(self.reason)
    }

    /// Check if should write shada.
    #[must_use]
    pub const fn should_write_shada(&self) -> bool {
        self.write_shada && self.get_reason().allows_save()
    }
}

// =============================================================================
// Cleanup Phases
// =============================================================================

/// Cleanup phases during exit.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CleanupPhase {
    /// Not cleaning up
    #[default]
    None = 0,
    /// Run VimLeave autocmds
    Autocmds = 1,
    /// Close all buffers
    Buffers = 2,
    /// Save shada file
    Shada = 3,
    /// Close all windows
    Windows = 4,
    /// Free memory
    Memory = 5,
    /// Final cleanup
    Final = 6,
}

impl CleanupPhase {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Autocmds,
            2 => Self::Buffers,
            3 => Self::Shada,
            4 => Self::Windows,
            5 => Self::Memory,
            6 => Self::Final,
            _ => Self::None,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Get next phase.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::None => Self::Autocmds,
            Self::Autocmds => Self::Buffers,
            Self::Buffers => Self::Shada,
            Self::Shada => Self::Windows,
            Self::Windows => Self::Memory,
            Self::Memory => Self::Final,
            Self::Final => Self::Final,
        }
    }

    /// Check if cleanup is complete.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Final)
    }
}

// =============================================================================
// Preserved State
// =============================================================================

/// Information to preserve across restart.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PreservedState {
    /// Number of files to reopen
    pub file_count: c_int,
    /// Cursor line in first file
    pub cursor_line: c_int,
    /// Cursor column in first file
    pub cursor_col: c_int,
    /// View top line
    pub topline: c_int,
    /// Window layout ID
    pub layout_id: c_int,
}

impl PreservedState {
    /// Create new preserved state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            file_count: 0,
            cursor_line: 1,
            cursor_col: 0,
            topline: 1,
            layout_id: 0,
        }
    }

    /// Check if there's anything to preserve.
    #[must_use]
    pub const fn has_content(&self) -> bool {
        self.file_count > 0
    }
}

// =============================================================================
// Phase 3: getout
// =============================================================================

// autocmd event constants (from build/include/auevents_enum.generated.h)
const EVENT_VIMLEAVEPRE: c_int = 131;
const EVENT_VIMLEAVE: c_int = 130;

unsafe extern "C" {
    // C helpers for getout (defined in main.c, Phase 3)
    fn nvim_getout_exmode_adjust(exitval: c_int) -> c_int;
    fn nvim_getout_set_vv_exiting(exitval: c_int);
    fn nvim_getout_trigger_bufwinleave();
    fn nvim_getout_trigger_bufunload();
    fn nvim_getout_apply_autocmd_event(event: c_int);
    fn nvim_getout_should_write_shada() -> bool;
    fn nvim_getout_handle_emsg();
    fn nvim_getout_restore_title();
    fn nvim_getout_do_restart();

    // Plain C functions needed by getout
    fn time_finish();
    fn invoke_all_defer();
    fn profile_dump();
    fn garbage_collect(testing: bool) -> bool;
    fn os_exit(r: c_int) -> !;
    fn rs_shada_write_file(file: *const std::ffi::c_char, nomerge: bool) -> c_int;

    // Globals
    static mut exiting: bool;
    static mut ui_client_channel_id: u64;
    static mut v_dying: c_int;
    static mut restarting: bool;
    static mut garbage_collect_at_exit: bool;
    static mut did_emsg: c_int;
}

/// Exit Nvim cleanly: fire autocmds, write ShaDa, restore title, exit.
///
/// # Safety
/// Must be called from the main thread. `ui_client_channel_id` must be 0.
#[unsafe(export_name = "getout")]
pub unsafe extern "C" fn rs_getout(exitval: c_int) -> ! {
    debug_assert!(ui_client_channel_id == 0);
    exiting = true;

    // Make sure startuptimes have been flushed.
    time_finish();

    // On error during Ex mode, exit with a non-zero code.
    // POSIX requires this, although it's not 100% clear from the standard.
    let exitval = nvim_getout_exmode_adjust(exitval);

    // Set v:exiting.
    nvim_getout_set_vv_exiting(exitval);

    // Invoke all deferred functions in the function stack.
    invoke_all_defer();

    // hash_debug_results() is a static inline no-op in hashtab.h; nothing to call.

    if v_dying <= 1 {
        // Trigger BufWinLeave for all windows, but only once per buffer.
        nvim_getout_trigger_bufwinleave();

        // Trigger BufUnload for buffers that are loaded.
        nvim_getout_trigger_bufunload();

        // Trigger VimLeavePre (unblocking autocmds if needed).
        nvim_getout_apply_autocmd_event(EVENT_VIMLEAVEPRE);
    }

    if nvim_getout_should_write_shada() {
        // Write out the registers, history, marks etc, to the ShaDa file.
        rs_shada_write_file(std::ptr::null(), false);
    }

    if v_dying <= 1 {
        // Trigger VimLeave (unblocking autocmds if needed).
        nvim_getout_apply_autocmd_event(EVENT_VIMLEAVE);
    }

    profile_dump();

    if did_emsg != 0 {
        // Give the user a chance to read the (error) message.
        // TODO(justinmk): this may call getout(0), clobbering exitval...
        nvim_getout_handle_emsg();
    }

    // Apply 'titleold'.
    nvim_getout_restore_title();

    if restarting {
        nvim_getout_do_restart();
    }

    if garbage_collect_at_exit {
        garbage_collect(false);
    }

    // MSWIN: os_icon_reset / os_title_reset omitted (Windows-only).

    os_exit(exitval)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code() {
        assert!(ExitCode::Success.is_success());
        assert!(!ExitCode::Error.is_success());
        assert_eq!(ExitCode::from_c_int(0), ExitCode::Success);
        assert_eq!(ExitCode::from_c_int(99), ExitCode::Error);
    }

    #[test]
    fn test_exit_reason() {
        assert!(ExitReason::Normal.allows_save());
        assert!(!ExitReason::Fatal.allows_save());
        assert!(ExitReason::Normal.runs_autocmds());
        assert!(!ExitReason::Error.runs_autocmds());
    }

    #[test]
    fn test_exit_state() {
        let mut state = ExitState::new();
        assert!(!state.exiting);

        state.begin(ExitCode::Error, ExitReason::Fatal);
        assert!(state.exiting);
        assert_eq!(state.code, 1);
        assert!(!state.should_write_shada());
    }

    #[test]
    fn test_cleanup_phase() {
        let phase = CleanupPhase::None;
        assert!(!phase.is_complete());

        let next = phase.next();
        assert_eq!(next, CleanupPhase::Autocmds);

        let final_phase = CleanupPhase::Final;
        assert!(final_phase.is_complete());
        assert_eq!(final_phase.next(), CleanupPhase::Final);
    }

    #[test]
    fn test_preserved_state() {
        let state = PreservedState::new();
        assert!(!state.has_content());

        let mut state = state;
        state.file_count = 2;
        assert!(state.has_content());
    }
}
